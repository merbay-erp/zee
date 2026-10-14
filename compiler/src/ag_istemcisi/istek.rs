use super::*;
use std::collections::BTreeSet;

fn denetle(
    url: &str,
    yontem: &str,
    basliklar: &[(String, String)],
    govde: &str,
    sure: i64,
    politika: &YetkinlikPolitikasi,
) -> Result<(), String> {
    if !url.starts_with("https://") || !matches!(yontem, "GET" | "POST") {
        return Err("HTTP isteği HTTPS ve GET/POST ister".into());
    }
    if !(1..=VARSAYILAN_ZAMAN_ASIMI_MS).contains(&sure)
        || govde.len() > AZAMI_YANIT_BAYTI - AZAMI_BASLIK_BAYTI
        || basliklar.len() > 64
        || (yontem == "GET" && !govde.is_empty())
    {
        return Err("HTTP istek süre veya boyut sınırı aşıldı".into());
    }
    politika
        .ag_istegini_denetle(url)
        .map_err(|_| "HTTP hedefi izinli değil")?;
    let mut gorulen = BTreeSet::new();
    let mut toplam = 0usize;
    for (ad, deger) in basliklar {
        let kucuk = ad.to_ascii_lowercase();
        toplam = toplam
            .saturating_add(ad.len())
            .saturating_add(deger.len())
            .saturating_add(4);
        if ureq::http::HeaderName::from_bytes(ad.as_bytes()).is_err()
            || !deger.bytes().all(|b| (32..=126).contains(&b))
            || !gorulen.insert(kucuk.clone())
            || toplam > AZAMI_BASLIK_BAYTI
            || matches!(
                kucuk.as_str(),
                "host"
                    | "content-length"
                    | "transfer-encoding"
                    | "connection"
                    | "proxy-connection"
                    | "proxy-authorization"
                    | "upgrade"
                    | "expect"
            )
        {
            return Err("HTTP başlıkları geçersiz veya çerçeveleme alanı yasak".into());
        }
    }
    Ok(())
}

pub fn istek(
    url: &str,
    yontem: &str,
    basliklar: &[(String, String)],
    govde: &str,
    sure: i64,
    politika: &YetkinlikPolitikasi,
) -> Result<(i64, String), String> {
    denetle(url, yontem, basliklar, govde, sure, politika)?;
    let _izin =
        crate::kaynak_sinirlari::baglanti_izni_al().map_err(|_| "HTTP bağlantı bütçesi dolu")?;
    let sure = Duration::from_millis(sure as u64);
    let config = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .proxy(None)
        .timeout_global(Some(sure))
        .timeout_resolve(Some(sure))
        .max_response_header_size(AZAMI_BASLIK_BAYTI)
        .build();
    let agent = ureq::Agent::with_parts(
        config,
        DefaultConnector::default(),
        PolitikaliCozucu {
            ic: DefaultResolver::default(),
            politika: politika.clone(),
        },
    );
    ajanla(&agent, url, yontem, basliklar, govde)
}

fn basliklari_ekle<B>(
    mut istek: ureq::RequestBuilder<B>,
    basliklar: &[(String, String)],
) -> Result<ureq::RequestBuilder<B>, String> {
    let alanlar = istek
        .headers_mut()
        .ok_or("HTTP istek başlıkları kurulamadı")?;
    for (ad, deger) in basliklar {
        let ad = ureq::http::HeaderName::from_bytes(ad.as_bytes())
            .map_err(|_| "HTTP başlığı geçersiz")?;
        let mut deger =
            ureq::http::HeaderValue::from_str(deger).map_err(|_| "HTTP başlığı geçersiz")?;
        deger.set_sensitive(true);
        alanlar.insert(ad, deger);
    }
    Ok(istek)
}

fn ajanla(
    agent: &ureq::Agent,
    url: &str,
    yontem: &str,
    basliklar: &[(String, String)],
    govde: &str,
) -> Result<(i64, String), String> {
    let yanit = if yontem == "GET" {
        basliklari_ekle(agent.get(url), basliklar)?.call()
    } else {
        basliklari_ekle(agent.post(url), basliklar)?.send(govde.as_bytes())
    };
    let mut yanit = yanit.map_err(|_| "HTTP taşıma işlemi başarısız")?;
    let durum = i64::from(yanit.status().as_u16());
    let govde = govdeyi_sinirli_oku(yanit.body_mut(), AZAMI_YANIT_BAYTI - AZAMI_BASLIK_BAYTI)
        .map_err(|_| "HTTP yanıt boyutu veya okuma sınırı aşıldı")?;
    let govde = String::from_utf8(govde).map_err(|_| "HTTP yanıtı UTF-8 metin değil")?;
    Ok((durum, govde))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yetkinlik::{AgHedefi, Yetkinlik};
    use std::io::{Read, Write};
    fn politika(url: &str) -> YetkinlikPolitikasi {
        let origin = url.split('/').take(3).collect::<Vec<_>>().join("/");
        YetkinlikPolitikasi::proje(
            BTreeSet::from([Yetkinlik::Ag, Yetkinlik::YerelAg]),
            BTreeSet::from([AgHedefi::bildirimden(&origin).unwrap()]),
        )
    }
    fn test_ajani(p: &YetkinlikPolitikasi, ms: u64) -> ureq::Agent {
        let cert = ureq::tls::Certificate::from_pem(include_bytes!(
            "../../tests/fixtures/http-post-tls/ca.pem"
        ))
        .unwrap();
        let tls = ureq::tls::TlsConfig::builder()
            .root_certs(ureq::tls::RootCerts::new_with_certs(&[cert]))
            .build();
        let c = ureq::Agent::config_builder()
            .tls_config(tls)
            .http_status_as_error(false)
            .max_redirects(0)
            .proxy(None)
            .timeout_global(Some(Duration::from_millis(ms)))
            .timeout_resolve(Some(Duration::from_millis(ms)))
            .max_response_header_size(AZAMI_BASLIK_BAYTI)
            .build();
        ureq::Agent::with_parts(
            c,
            DefaultConnector::default(),
            PolitikaliCozucu {
                ic: DefaultResolver::default(),
                politika: p.clone(),
            },
        )
    }
    fn sunucu(code: u16, body: &str, delay: u64) -> (String, std::thread::JoinHandle<String>) {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let url = format!("https://{}/verify", listener.local_addr().unwrap());
        let identity = native_tls::Identity::from_pkcs12(
            include_bytes!("../../tests/fixtures/http-post-tls/server.p12"),
            "zee-fixture",
        )
        .unwrap();
        let acceptor = native_tls::TlsAcceptor::new(identity).unwrap();
        let body = body.to_string();
        let thread = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let Ok(mut stream) = acceptor.accept(stream) else {
                return String::new();
            };
            let mut bytes = Vec::new();
            let mut part = [0; 4096];
            loop {
                let n = stream.read(&mut part).unwrap();
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&part[..n]);
                if let Some(end) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..end]);
                    let len = headers
                        .lines()
                        .filter_map(|x| x.split_once(':'))
                        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
                        .map(|(_, v)| v.trim().parse::<usize>().unwrap())
                        .unwrap_or(0);
                    if bytes.len() >= end + 4 + len {
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(delay));
            let reply=format!("HTTP/1.1 {code} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nLocation: https://other.example/\r\n\r\n{body}",body.len());
            let _ = stream.write_all(reply.as_bytes());
            String::from_utf8(bytes).unwrap()
        });
        (url, thread)
    }
    #[test]
    fn tls_post_baslik_govde_json_ve_non2xx_durum_kayipsizdir() {
        for (code, body) in [
            (200, r#"{"success":true,"hostname":"site.example"}"#),
            (403, r#"{"success":false}"#),
            (302, r#"{"hostname":"other.example"}"#),
        ] {
            let (url, server) = sunucu(code, body, 0);
            let p = politika(&url);
            let h = vec![
                ("Authorization".into(), "Bearer TEST_SENTINEL".into()),
                (
                    "Content-Type".into(),
                    "application/x-www-form-urlencoded".into(),
                ),
            ];
            denetle(
                &url,
                "POST",
                &h,
                "response=a%2Bb&remoteip=203.0.113.7",
                3000,
                &p,
            )
            .unwrap();
            let r = ajanla(
                &test_ajani(&p, 3000),
                &url,
                "POST",
                &h,
                "response=a%2Bb&remoteip=203.0.113.7",
            )
            .unwrap();
            assert_eq!(r, (i64::from(code), body.into()));
            let wire = server.join().unwrap();
            assert!(wire.contains("POST /verify HTTP/1.1"));
            assert!(wire
                .to_ascii_lowercase()
                .contains("authorization: bearer test_sentinel"));
            assert!(wire.ends_with("response=a%2Bb&remoteip=203.0.113.7"));
        }
    }
    #[test]
    fn tls_dogrulamasi_kapatilmaz_ve_yanit_timeout_sirri_gostermez() {
        let (url, server) = sunucu(200, "TEST_SENTINEL", 0);
        let p = politika(&url);
        let e = istek(&url, "POST", &[], "TEST_SENTINEL", 2000, &p).unwrap_err();
        assert!(!e.contains("TEST_SENTINEL"));
        server.join().unwrap();
        let (url, server) = sunucu(200, "TEST_SENTINEL", 300);
        let p = politika(&url);
        let e = ajanla(&test_ajani(&p, 100), &url, "POST", &[], "TEST_SENTINEL").unwrap_err();
        assert!(!e.contains("TEST_SENTINEL"));
        server.join().unwrap();
    }
    #[test]
    fn baslik_degerleri_debugde_sensitivedir() {
        let agent = ureq::agent();
        let r = basliklari_ekle(
            agent.post("https://service.example/"),
            &[("X-Api-Key".into(), "TEST_SENTINEL".into())],
        )
        .unwrap();
        assert!(r
            .headers_ref()
            .unwrap()
            .get("x-api-key")
            .unwrap()
            .is_sensitive());
        assert!(!format!("{:?}", r.headers_ref()).contains("TEST_SENTINEL"));
    }
    #[test]
    fn https_yontem_sure_baslik_tekrar_framing_ve_boyut_kapilari() {
        let p = YetkinlikPolitikasi::gelistirici();
        let url = "https://service.example/";
        for (u, m, t) in [
            ("http://service.example/", "POST", 1000),
            (url, "PUT", 1000),
            (url, "POST", 0),
            (url, "POST", 30001),
        ] {
            assert!(denetle(u, m, &[], "", t, &p).is_err());
        }
        for h in [
            vec![("Host".into(), "other.example".into())],
            vec![("Content-Length".into(), "3".into())],
            vec![("Transfer-Encoding".into(), "chunked".into())],
            vec![("Authorization".into(), "a\r\nX: b".into())],
            vec![("A".into(), "x".into()), ("a".into(), "y".into())],
            vec![("X".into(), "x".repeat(AZAMI_BASLIK_BAYTI))],
            (0..65).map(|i| (format!("X-{i}"), "x".into())).collect(),
        ] {
            assert!(denetle(url, "POST", &h, "", 1000, &p).is_err());
        }
        assert!(denetle(url, "GET", &[], "body", 1000, &p).is_err());
        assert!(denetle(url, "POST", &[], &"x".repeat(AZAMI_YANIT_BAYTI), 1000, &p).is_err());
        assert!(denetle(
            url,
            "POST",
            &[("Authorization".into(), "Bearer ok".into())],
            "{}",
            5000,
            &p
        )
        .is_ok());
    }
}
