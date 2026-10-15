use super::*;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;

fn ayar() -> EpostaAyari {
    EpostaAyari::ayristir(
        "smtp://sentinel:SENTINEL_PASSWORD@localhost:2525?from=sender@example.invalid&name=Zee",
    )
    .unwrap()
}

fn ileti() -> Ileti<'static> {
    Ileti {
        alici: "recipient@example.invalid",
        konu: "Katalog Şalça",
        govde: "Merhaba\r\n.nokta",
        kimlik: "fixture123",
        tarih: "Wed, 14 Oct 2026 11:00:00 +0000",
    }
}

fn smtp_server<T: Read + Write>(stream: T) -> String {
    smtp_server_diyalog(stream, true)
}

fn smtp_server_diyalog<T: Read + Write>(stream: T, karsilama: bool) -> String {
    let mut stream = BufReader::new(stream);
    if karsilama {
        stream.get_mut().write_all(b"220 test fixture\r\n").unwrap();
    }
    let mut line = String::new();
    stream.read_line(&mut line).unwrap();
    assert_eq!(line, "EHLO zee.invalid\r\n");
    stream.get_mut().write_all(b"250 fixture\r\n").unwrap();
    for (beklenen, yanit) in [
        ("AUTH PLAIN ", "235 ok\r\n"),
        ("MAIL FROM:<", "250 ok\r\n"),
        ("RCPT TO:<", "250 ok\r\n"),
        ("DATA", "354 send\r\n"),
    ] {
        line.clear();
        stream.read_line(&mut line).unwrap();
        assert!(line.starts_with(beklenen), "SMTP fixture sırası");
        stream.get_mut().write_all(yanit.as_bytes()).unwrap();
    }
    let mut data = String::new();
    loop {
        line.clear();
        assert!(stream.read_line(&mut line).unwrap() > 0);
        if line == ".\r\n" {
            break;
        }
        data.push_str(&line);
    }
    stream.get_mut().write_all(b"250 accepted\r\n").unwrap();
    line.clear();
    stream.read_line(&mut line).unwrap();
    assert_eq!(line, "QUIT\r\n");
    stream.get_mut().write_all(b"221 bye\r\n").unwrap();
    data
}

#[test]
fn bos_mime_secenegi_plaintext_iletiyi_bayt_bayt_korur() {
    let ayar = ayar();
    let ileti = ileti();
    assert_eq!(
        ileti_kur(&ayar, &ileti, &MimeSecenekleri::default()).unwrap(),
        super::super::ileti_kur(&ayar, &ileti)
    );
}

#[test]
fn alternative_mixed_reply_to_ve_unicode_dosya_adi_kodlanir() {
    let ayar = ayar();
    let ileti = ileti();
    let pdf = b"%PDF-1.7\n\x00\xff\r\n.nokta";
    let ekler = [
        EkDosya {
            ad: "Katalog Şalça.pdf",
            tur: EkTuru::Pdf,
            baytlar: pdf,
        },
        EkDosya {
            ad: "resim.png",
            tur: EkTuru::Png,
            baytlar: b"\x89PNG\r\n\x1a\n\x00",
        },
    ];
    let secenek = MimeSecenekleri {
        yanit_adresi: Some("reply@example.invalid"),
        html: Some("<p>Merhaba &amp; Şalça</p>"),
        ekler: &ekler,
    };
    let wire = ileti_kur(&ayar, &ileti, &secenek).unwrap();
    assert!(wire.contains("Reply-To: reply@example.invalid\r\n"));
    assert!(wire.contains("multipart/mixed"));
    assert!(wire.contains("multipart/alternative"));
    assert!(wire.contains("filename*=UTF-8''Katalog%20%C5%9Eal%C3%A7a.pdf\r\n"));
    assert!(wire.contains(&bayt_govdesi(pdf)));
    assert!(!wire.contains("SENTINEL_PASSWORD"));
    assert!(wire.is_ascii());
    assert!(wire.split("\r\n").all(|line| line.len() <= 998));
    assert!(wire
        .split("\r\n")
        .filter(|line| !line.is_empty()
            && line
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=')))
        .all(|line| line.len() <= 76));
}

#[test]
fn baslik_kimlik_ve_ek_adinda_injection_reddedilir() {
    for bad in ["a\r\nBcc: attacker@example.invalid", "a\nb", "a\u{0}"] {
        let mut yanlis = ileti();
        yanlis.konu = bad;
        assert!(ileti_kur(&ayar(), &yanlis, &MimeSecenekleri::default()).is_err());
        let mut yanlis = ileti();
        yanlis.kimlik = bad;
        assert!(ileti_kur(&ayar(), &yanlis, &MimeSecenekleri::default()).is_err());
        assert!(ek_adini_denetle(bad).is_err());
    }
    for bad in ["../x.pdf", "/x.pdf", "a\\x.pdf", "", ".", ".."] {
        assert!(ek_adini_denetle(bad).is_err());
    }
    for bad in ["reply@example.invalid\r\nBcc: x@y.z", "<x@y.z>", "a b@x.y"] {
        let secenek = MimeSecenekleri {
            yanit_adresi: Some(bad),
            ..MimeSecenekleri::default()
        };
        assert!(ileti_kur(&ayar(), &ileti(), &secenek).is_err());
    }
    let secenek = MimeSecenekleri {
        ekler: &[EkDosya {
            ad: "x.pdf",
            tur: EkTuru::Png,
            baytlar: b"PNG",
        }],
        ..MimeSecenekleri::default()
    };
    assert!(ileti_kur(&ayar(), &ileti(), &secenek).is_err());
    assert!(EkTuru::dosya_adindan("a.exe").is_err());
}

#[test]
fn uzun_unicode_basliklar_75_sozcuk_ve_998_smtp_satir_sinirinda_katlanir() {
    let mut ayar = ayar();
    ayar.gonderen_adi = "Ş😀a".repeat(36);
    let konu = "İ😀Katalog ".repeat(35);
    let mut ileti = ileti();
    ileti.konu = &konu;
    let wire = ileti_kur(
        &ayar,
        &ileti,
        &MimeSecenekleri {
            html: Some("<p>Test</p>"),
            ..MimeSecenekleri::default()
        },
    )
    .unwrap();
    for sozcuk in wire
        .split_whitespace()
        .filter(|s| s.starts_with("=?UTF-8?B?"))
    {
        assert!(sozcuk.len() <= 75);
        assert!(sozcuk.ends_with("?="));
    }
    assert!(wire.split("\r\n").all(|s| s.len() <= 998));
    let mut bad = ileti;
    let alici = format!("{}@example.invalid", "a".repeat(65));
    bad.alici = &alici;
    assert!(ileti_kur(&ayar, &bad, &MimeSecenekleri::default()).is_err());
    let alici = format!("{}@example.invalid", "a".repeat(64));
    bad.alici = &alici;
    assert!(ileti_kur(&ayar, &bad, &MimeSecenekleri::default()).is_ok());
    let buyuk = format!("a@{}.invalid", "a".repeat(254));
    bad.alici = &buyuk;
    assert!(ileti_kur(&ayar, &bad, &MimeSecenekleri::default()).is_err());
}

#[test]
fn ek_adet_tek_ve_toplam_boyut_ile_metin_sinirlidir() {
    let kucuk = EkDosya {
        ad: "x.pdf",
        tur: EkTuru::Pdf,
        baytlar: b"x",
    };
    let ekler = vec![kucuk; AZAMI_EK_ADEDI + 1];
    assert!(ileti_kur(
        &ayar(),
        &ileti(),
        &MimeSecenekleri {
            ekler: &ekler,
            ..MimeSecenekleri::default()
        }
    )
    .is_err());
    let buyuk = vec![0; AZAMI_EK_BAYTI + 1];
    let ekler = [EkDosya {
        ad: "x.pdf",
        tur: EkTuru::Pdf,
        baytlar: &buyuk,
    }];
    assert!(ileti_kur(
        &ayar(),
        &ileti(),
        &MimeSecenekleri {
            ekler: &ekler,
            ..MimeSecenekleri::default()
        }
    )
    .is_err());
    let tek = vec![0; AZAMI_EK_BAYTI];
    let ekler = [
        EkDosya {
            ad: "a.pdf",
            tur: EkTuru::Pdf,
            baytlar: &tek,
        },
        EkDosya {
            ad: "b.pdf",
            tur: EkTuru::Pdf,
            baytlar: &tek,
        },
        EkDosya {
            ad: "c.pdf",
            tur: EkTuru::Pdf,
            baytlar: b"x",
        },
    ];
    assert!(ileti_kur(
        &ayar(),
        &ileti(),
        &MimeSecenekleri {
            ekler: &ekler,
            ..MimeSecenekleri::default()
        }
    )
    .is_err());
    let metin = "x".repeat(AZAMI_METIN_BAYTI + 1);
    let mut bad = ileti();
    bad.govde = &metin;
    assert!(ileti_kur(&ayar(), &bad, &MimeSecenekleri::default()).is_err());
    assert!(ileti_kur(
        &ayar(),
        &ileti(),
        &MimeSecenekleri {
            html: Some(&metin),
            ..MimeSecenekleri::default()
        }
    )
    .is_err());
}

#[test]
fn smtp_dotstuffing_durumunda_terminator_enjekte_edilemez() {
    assert_eq!(
        smtp_verisi(".a\r\n..b\r\n.\r\n"),
        "..a\r\n...b\r\n..\r\n\r\n."
    );
}

#[test]
fn mime_metin_satirlari_crlfdir_legacy_plain_baytlari_degismez() {
    let mut ileti = ileti();
    ileti.govde = "a\nb\rc\r\nd";
    let secenek = MimeSecenekleri {
        html: Some("<p>a\nb</p>"),
        ..MimeSecenekleri::default()
    };
    let wire = ileti_kur(&ayar(), &ileti, &secenek).unwrap();
    assert!(wire.contains(&bayt_govdesi(b"a\r\nb\r\nc\r\nd")));
    assert!(wire.contains(&bayt_govdesi(b"<p>a\r\nb</p>")));
    assert_eq!(
        ileti_kur(&ayar(), &ileti, &MimeSecenekleri::default()).unwrap(),
        super::super::ileti_kur(&ayar(), &ileti)
    );
}

#[test]
fn gercek_loopback_smtp_mime_ve_binary_eki_tek_data_ile_gonderir() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let sunucu = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        smtp_server(stream)
    });
    let mut ayar = ayar();
    ayar.konak = "127.0.0.1".into();
    ayar.kapi = port;
    let ekler = [EkDosya {
        ad: "x.pdf",
        tur: EkTuru::Pdf,
        baytlar: b"%PDF-1.7\n\x00\xff",
    }];
    let secenek = MimeSecenekleri {
        yanit_adresi: Some("reply@example.invalid"),
        html: Some("<p>Şalça</p>"),
        ekler: &ekler,
    };
    let wire = ileti_kur(&ayar, &ileti(), &secenek).unwrap();
    gonder(&ayar, &ileti(), &secenek, Duration::from_secs(2), &|ip| {
        if ip.is_loopback() {
            Ok(())
        } else {
            Err("IP ret".into())
        }
    })
    .unwrap();
    assert_eq!(
        sunucu.join().unwrap(),
        smtp_verisi(&wire).trim_end_matches('.')
    );
}

#[test]
fn smtp_sunucu_echo_hatasi_credential_ve_govdeyi_sizdirmaz() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let sunucu = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.write_all(b"220 fixture\r\n").unwrap();
        let mut stream = BufReader::new(stream);
        let mut line = String::new();
        stream.read_line(&mut line).unwrap();
        stream.get_mut().write_all(b"250 fixture\r\n").unwrap();
        line.clear();
        stream.read_line(&mut line).unwrap();
        stream
            .get_mut()
            .write_all(format!("535 SENTINEL_PASSWORD SENTINEL_BODY {}", line).as_bytes())
            .unwrap();
    });
    let mut ayar = ayar();
    ayar.konak = "127.0.0.1".into();
    ayar.kapi = port;
    let hata = gonder(
        &ayar,
        &ileti(),
        &MimeSecenekleri::default(),
        Duration::from_secs(2),
        &|_| Ok(()),
    )
    .unwrap_err();
    sunucu.join().unwrap();
    assert!(hata.contains("ayrıntılar gizlendi"));
    assert!(!hata.contains("SENTINEL"));
    assert!(!hata.contains(&base64(b"\0sentinel\0SENTINEL_PASSWORD")));
}

#[test]
fn yavas_karsilama_ve_tls_el_sikismasi_tek_son_tarihte_kesilir() {
    // Warm/setup work is outside this IO cancellation test. The peer stays
    // open and sends no response until the call returns; EOF cannot pass it.
    // The two-second channel timeout is a hang guard, not a latency benchmark.
    let connector = native_tls::TlsConnector::builder().build().unwrap();
    // macOS nativeTLS context creation and error text may initialize platform
    // services synchronously. Warm the complete path before the IO hang guard.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (release, hold) = std::sync::mpsc::channel();
    let warm_peer = thread::spawn(move || {
        let (_stream, _) = listener.accept().unwrap();
        let _ = hold.recv_timeout(Duration::from_secs(15));
    });
    let mut warm_ayar = ayar();
    warm_ayar.konak = "127.0.0.1".into();
    warm_ayar.kapi = port;
    warm_ayar.implicit_tls = true;
    let warm_wire = ileti_kur(&warm_ayar, &ileti(), &MimeSecenekleri::default()).unwrap();
    assert!(gonder_ic(
        &warm_ayar,
        ileti().alici,
        &warm_wire,
        Duration::from_millis(60),
        &|_| Ok(()),
        &connector,
    )
    .is_err());
    release.send(()).unwrap();
    warm_peer.join().unwrap();
    for tls in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (baglandi, baglanti) = std::sync::mpsc::channel();
        let (birak, bekle) = std::sync::mpsc::channel();
        let sunucu = thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            baglandi.send(()).unwrap();
            let _ = bekle.recv_timeout(Duration::from_secs(4));
        });
        let mut ayar = ayar();
        ayar.konak = "127.0.0.1".into();
        ayar.kapi = port;
        ayar.implicit_tls = tls;
        let wire = ileti_kur(&ayar, &ileti(), &MimeSecenekleri::default()).unwrap();
        let connector = connector.clone();
        let (bitti, sonuc) = std::sync::mpsc::channel();
        let istemci = thread::spawn(move || {
            let bas = Instant::now();
            let hata = gonder_ic(
                &ayar,
                ileti().alici,
                &wire,
                Duration::from_millis(60),
                &|_| Ok(()),
                &connector,
            );
            let _ = bitti.send((hata, bas.elapsed()));
        });
        baglanti
            .recv_timeout(Duration::from_secs(2))
            .expect("peer bağlantıyı kabul etmeli");
        let (hata, sure) = sonuc
            .recv_timeout(Duration::from_secs(2))
            .expect("cevapsız peer açıkken son tarih çağrıyı kesmeli");
        assert!(hata.is_err(), "tls={tls} elapsed={sure:?}");
        assert!(!hata.unwrap_err().contains("SENTINEL"));
        birak.send(()).unwrap();
        istemci.join().unwrap();
        sunucu.join().unwrap();
    }
}

#[test]
fn trusted_localhost_tls_gercek_mime_data_yolunu_tasir_untrusted_reddedilir() {
    let identity = native_tls::Identity::from_pkcs12(
        include_bytes!("../../../tests/fixtures/smtp-mime/server.p12"),
        "zee-fixture",
    )
    .unwrap();
    let acceptor = native_tls::TlsAcceptor::new(identity).unwrap();
    let listener = TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let sunucu = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        smtp_server(acceptor.accept(stream).unwrap())
    });
    let mut builder = native_tls::TlsConnector::builder();
    builder.add_root_certificate(
        native_tls::Certificate::from_pem(include_bytes!(
            "../../../tests/fixtures/smtp-mime/ca.pem"
        ))
        .unwrap(),
    );
    let tls = builder.build().unwrap();
    let mut ayar = ayar();
    ayar.kapi = port;
    ayar.implicit_tls = true;
    let wire = ileti_kur(
        &ayar,
        &ileti(),
        &MimeSecenekleri {
            html: Some("<p>Test</p>"),
            ..MimeSecenekleri::default()
        },
    )
    .unwrap();
    gonder_ic(
        &ayar,
        ileti().alici,
        &wire,
        Duration::from_secs(2),
        &|ip| {
            if ip.is_loopback() {
                Ok(())
            } else {
                Err("IP ret".into())
            }
        },
        &tls,
    )
    .unwrap();
    assert!(sunucu.join().unwrap().contains("multipart/alternative"));
    let identity = native_tls::Identity::from_pkcs12(
        include_bytes!("../../../tests/fixtures/smtp-mime/server.p12"),
        "zee-fixture",
    )
    .unwrap();
    let acceptor = native_tls::TlsAcceptor::new(identity).unwrap();
    let listener = TcpListener::bind("localhost:0").unwrap();
    ayar.kapi = listener.local_addr().unwrap().port();
    let sunucu = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let _ = acceptor.accept(stream);
    });
    assert!(gonder(
        &ayar,
        &ileti(),
        &MimeSecenekleri::default(),
        Duration::from_secs(2),
        &|_| Ok(())
    )
    .is_err());
    sunucu.join().unwrap();
}

#[test]
fn starttls_yeniden_ehlo_ve_auth_oncesinde_gercek_tls_kullanir() {
    let identity = native_tls::Identity::from_pkcs12(
        include_bytes!("../../../tests/fixtures/smtp-mime/server.p12"),
        "zee-fixture",
    )
    .unwrap();
    let acceptor = native_tls::TlsAcceptor::new(identity).unwrap();
    let listener = TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let sunucu = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut stream = BufReader::new(stream);
        stream.get_mut().write_all(b"220 fixture\r\n").unwrap();
        let mut line = String::new();
        stream.read_line(&mut line).unwrap();
        assert_eq!(line, "EHLO zee.invalid\r\n");
        stream
            .get_mut()
            .write_all(b"250-STARTTLS\r\n250 AUTH PLAIN\r\n")
            .unwrap();
        line.clear();
        stream.read_line(&mut line).unwrap();
        assert_eq!(line, "STARTTLS\r\n");
        stream.get_mut().write_all(b"220 TLS\r\n").unwrap();
        let tls = acceptor.accept(stream.into_inner()).unwrap();
        smtp_server_diyalog(tls, false)
    });
    let mut builder = native_tls::TlsConnector::builder();
    builder.add_root_certificate(
        native_tls::Certificate::from_pem(include_bytes!(
            "../../../tests/fixtures/smtp-mime/ca.pem"
        ))
        .unwrap(),
    );
    let tls = builder.build().unwrap();
    let mut ayar = ayar();
    ayar.kapi = port;
    let wire = ileti_kur(
        &ayar,
        &ileti(),
        &MimeSecenekleri {
            html: Some("<p>STARTTLS</p>"),
            ..MimeSecenekleri::default()
        },
    )
    .unwrap();
    gonder_ic(
        &ayar,
        ileti().alici,
        &wire,
        Duration::from_secs(2),
        &|ip| {
            if ip.is_loopback() {
                Ok(())
            } else {
                Err("IP ret".into())
            }
        },
        &tls,
    )
    .unwrap();
    assert!(sunucu.join().unwrap().contains("multipart/alternative"));
}

#[test]
fn ek_loader_yetkisiz_ve_buyuk_boyutu_reddeder_izin_mtime_yazmaz() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SIRA: AtomicU64 = AtomicU64::new(0);
    let kok = std::env::temp_dir().join(format!(
        "zee-k232-ek-{}-{}",
        std::process::id(),
        SIRA.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&kok).unwrap();
    let pdf = kok.join("katalog.pdf");
    std::fs::write(&pdf, b"%PDF-1.7\n\x00\xff").unwrap();
    let once = std::fs::metadata(&pdf).unwrap();
    let mut acma = 0;
    assert!(ekleri_oku(&["katalog.pdf".into()], &mut |_| {
        acma += 1;
        Err("dosya-okuma kapalı SENTINEL_PASSWORD".into())
    })
    .is_err());
    assert_eq!(acma, 1);
    let ekler = ekleri_oku(&["katalog.pdf".into()], &mut |yol| {
        std::fs::File::open(kok.join(yol)).map_err(|_| "erişim reddi".into())
    })
    .unwrap();
    assert_eq!(ekler[0].baytlar, b"%PDF-1.7\n\x00\xff");
    let sonra = std::fs::metadata(&pdf).unwrap();
    assert_eq!(once.modified().unwrap(), sonra.modified().unwrap());
    assert_eq!(
        once.permissions().readonly(),
        sonra.permissions().readonly()
    );
    assert!(ekleri_oku(&["katalog.pdf".into()], &mut |_| {
        let f = std::fs::OpenOptions::new().write(true).open(&pdf).unwrap();
        f.set_len(AZAMI_EK_BAYTI as u64 + 1).unwrap();
        std::fs::File::open(&pdf).map_err(|_| "erişim reddi".into())
    })
    .is_err());
    let mut cagrildi = false;
    assert!(ekleri_oku(&vec!["katalog.pdf".into(); 9], &mut |_| {
        cagrildi = true;
        Err("olmamalı".into())
    })
    .is_err());
    assert!(!cagrildi);
    std::fs::remove_dir_all(kok).unwrap();
}

#[test]
fn ek_loader_kok_disina_cikis_ve_symlink_icin_host_reddini_asamaz() {
    let yollar = ["../dis.pdf".into(), "/dis.pdf".into(), "bag.pdf".into()];
    for yol in yollar {
        let hata = ekleri_oku(&[yol], &mut |_| {
            Err("proje-kökü veya sembolik bağ reddi SENTINEL_PASSWORD".into())
        })
        .unwrap_err();
        assert!(!hata.contains("SENTINEL"));
        assert!(hata.contains("erişimi reddedildi"));
    }
    let ekler = ekleri_oku(&[], &mut |_| panic!("eksiz gönderim dosya açmaz")).unwrap();
    assert!(ekler.is_empty());
}
