//! Native outbound HTTP(S) istemcisi: rustls + kapalı redirect + DNS sonrası
//! IP capability denetimi. WASM host adaptörü bu modülü kullanmaz.

use crate::yetkinlik::YetkinlikPolitikasi;
use std::time::Duration;
use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};

pub const VARSAYILAN_ZAMAN_ASIMI_MS: i64 = 30_000;
pub const AZAMI_YANIT_BAYTI: usize = 8 * 1024 * 1024;
pub const AZAMI_BASLIK_BAYTI: usize = 64 * 1024;

#[derive(Debug)]
struct PolitikaliCozucu {
    ic: DefaultResolver,
    politika: YetkinlikPolitikasi,
}

impl Resolver for PolitikaliCozucu {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let adresler = self.ic.resolve(uri, config, timeout)?;
        for adres in adresler.iter() {
            self.politika
                .cozulmus_ipyi_denetle(adres.ip())
                .map_err(|neden| {
                    ureq::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        neden,
                    ))
                })?;
        }
        Ok(adresler)
    }
}

pub fn getir(
    url: &str,
    zaman_asimi_ms: Option<i64>,
    politika: &YetkinlikPolitikasi,
) -> Result<(i64, String), String> {
    politika.ag_istegini_denetle(url)?;
    let toplam_ms = zaman_asimi_ms.unwrap_or(VARSAYILAN_ZAMAN_ASIMI_MS);
    if toplam_ms <= 0 {
        return Err("son tarih doldu".into());
    }
    let zaman_asimi = Duration::from_millis(toplam_ms as u64);
    let config = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .proxy(None)
        .timeout_global(Some(zaman_asimi))
        .timeout_resolve(Some(zaman_asimi))
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
    let mut yanit = agent
        .get(url)
        .call()
        .map_err(|hata| format!("HTTP isteği başarısız: {}", hata))?;
    let durum = i64::from(yanit.status().as_u16());
    let govde = govdeyi_sinirli_oku(yanit.body_mut())?;
    Ok((durum, String::from_utf8_lossy(&govde).into_owned()))
}

fn govdeyi_sinirli_oku(govde: &mut ureq::Body) -> Result<Vec<u8>, String> {
    govde
        .with_config()
        .limit((AZAMI_YANIT_BAYTI - AZAMI_BASLIK_BAYTI) as u64)
        .read_to_vec()
        .map_err(|hata| format!("HTTP yanıtı 8 MiB toplam sınırında okunamadı: {}", hata))
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::yetkinlik::{AgHedefi, Yetkinlik};
    use std::collections::BTreeSet;
    use std::io::{Read, Write};

    #[test]
    fn yanit_govdesi_bellek_sinirini_asamaz() {
        let mut govde = ureq::Body::builder().data(vec![0; AZAMI_YANIT_BAYTI]);
        assert!(govdeyi_sinirli_oku(&mut govde)
            .unwrap_err()
            .contains("8 MiB"));
    }

    #[test]
    fn yerel_hedef_acik_izin_ister_ve_redirect_izlenmez() {
        let dinleyici = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let adres = dinleyici.local_addr().unwrap();
        let url = format!("http://{adres}/ilk");
        let sunucu = std::thread::spawn(move || {
            let (mut akis, _) = dinleyici.accept().unwrap();
            let mut istek = [0u8; 1024];
            let _ = akis.read(&mut istek).unwrap();
            akis.write_all(
                b"HTTP/1.1 302 Found\r\nLocation: http://169.254.169.254/latest\r\nContent-Length: 4\r\nConnection: close\r\n\r\ndur!",
            )
            .unwrap();
        });
        let politika = YetkinlikPolitikasi::proje(
            BTreeSet::from([Yetkinlik::Ag, Yetkinlik::YerelAg]),
            BTreeSet::from([AgHedefi::bildirimden(&format!("http://{adres}")).unwrap()]),
        );

        let (durum, govde) = getir(&url, Some(2_000), &politika).expect("ilk yanıt dönmeli");
        assert_eq!(durum, 302);
        assert_eq!(govde, "dur!");
        sunucu.join().unwrap();
    }
}
