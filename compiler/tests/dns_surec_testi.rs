//! K-390 (RFC-0044 B1, ADR-169): gerçek `dil` süreci + loopback'te sahte
//! UDP/TCP çözücü. Stub istemcinin kimlik/soru eşlemesi, TC→TCP geçişi,
//! hata sınıfları ve `dns_dnssec: "zorunlu"` açılış sınaması.

use std::io::{Read, Write};
use std::net::{TcpListener, UdpSocket};
use std::process::Command;
use std::sync::{Arc, Mutex};

type Isleyici = dyn Fn(&[u8]) -> Vec<Vec<u8>> + Send + Sync;

struct SahteCozucu {
    port: u16,
    sorgular: Arc<Mutex<Vec<Vec<u8>>>>,
}

/// UDP ve (aynı kapıda) TCP dinleyen sahte çözücü.
fn sahte_cozucu(udp: Arc<Isleyici>, tcp: Option<Arc<Isleyici>>) -> SahteCozucu {
    let (soket, dinleyici) = loop {
        let soket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = soket.local_addr().unwrap().port();
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(dinleyici) => break (soket, dinleyici),
            Err(_) => continue,
        }
    };
    let port = soket.local_addr().unwrap().port();
    let sorgular = Arc::new(Mutex::new(Vec::new()));
    let kayit = sorgular.clone();
    std::thread::spawn(move || {
        let mut tampon = [0u8; 2048];
        while let Ok((n, kaynak)) = soket.recv_from(&mut tampon) {
            kayit.lock().unwrap().push(tampon[..n].to_vec());
            for yanit in udp(&tampon[..n]) {
                let _ = soket.send_to(&yanit, kaynak);
            }
        }
    });
    if let Some(tcp) = tcp {
        std::thread::spawn(move || {
            for akis in dinleyici.incoming() {
                let Ok(mut akis) = akis else { continue };
                let mut boy = [0u8; 2];
                if akis.read_exact(&mut boy).is_err() {
                    continue;
                }
                let mut sorgu = vec![0u8; usize::from(u16::from_be_bytes(boy))];
                if akis.read_exact(&mut sorgu).is_err() {
                    continue;
                }
                for yanit in tcp(&sorgu) {
                    let mut cerceve = (yanit.len() as u16).to_be_bytes().to_vec();
                    cerceve.extend_from_slice(&yanit);
                    let _ = akis.write_all(&cerceve);
                }
            }
        });
    }
    SahteCozucu { port, sorgular }
}

/// Sorgunun kimlik + soru bölümünü kopyalayıp yanıt başlığı kurar.
fn yanit(sorgu: &[u8], bayrak: u16, cevaplar: &[Vec<u8>]) -> Vec<u8> {
    let mut i = 12;
    while sorgu[i] != 0 {
        i += 1 + usize::from(sorgu[i]);
    }
    let soru_sonu = i + 5;
    let mut p = sorgu[..2].to_vec();
    p.extend_from_slice(&bayrak.to_be_bytes());
    p.extend_from_slice(&1u16.to_be_bytes());
    p.extend_from_slice(&(cevaplar.len() as u16).to_be_bytes());
    p.extend_from_slice(&[0, 0, 0, 0]);
    p.extend_from_slice(&sorgu[12..soru_sonu]);
    for cevap in cevaplar {
        p.extend_from_slice(cevap);
    }
    p
}

fn kayit(tur: u16, veri: &[u8]) -> Vec<u8> {
    let mut p = vec![0xC0, 0x0C];
    p.extend_from_slice(&tur.to_be_bytes());
    p.extend_from_slice(&1u16.to_be_bytes());
    p.extend_from_slice(&60u32.to_be_bytes());
    p.extend_from_slice(&(veri.len() as u16).to_be_bytes());
    p.extend_from_slice(veri);
    p
}

fn mx(oncelik: u16, etiket: &str) -> Vec<u8> {
    let mut veri = oncelik.to_be_bytes().to_vec();
    veri.push(etiket.len() as u8);
    veri.extend_from_slice(etiket.as_bytes());
    veri.extend_from_slice(&[0xC0, 0x0C]);
    kayit(15, &veri)
}

const PROGRAM: &str = "\
sonuç \"ornek.test\" için MX kayıtlarını sorgulamayı dene olsun
sonuç başarılıysa
    kayıtlar sonucun değeri olsun
    kayıtlardaki her satır için
        satırın \"hedef\" değeri yaz
        satırın \"doğrulanmış\" değeri yaz
değilse
    hata sonucun hatası olsun
    hatanın kodunu yaz
    veri hatanın verisi olsun
    verinin \"tür\" değeri yaz
";

fn calistir(port: u16, kip: &str, program: &str) -> (i32, String, String) {
    let dizin = std::env::temp_dir().join(format!("zee-dns-{}-{}", std::process::id(), port));
    let _ = std::fs::remove_dir_all(&dizin);
    std::fs::create_dir_all(&dizin).unwrap();
    std::fs::write(
        dizin.join("proje.dil"),
        format!(
            "proje \"dns\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
             yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"127.0.0.1:{port}\" olsun\n\
             dns_dnssec \"{kip}\" olsun\n"
        ),
    )
    .unwrap();
    std::fs::write(dizin.join("ana.dil"), program).unwrap();
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(&dizin)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dizin);
    (
        cikti.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&cikti.stdout).into_owned(),
        String::from_utf8_lossy(&cikti.stderr).into_owned(),
    )
}

#[test]
fn eslesmeyen_paketler_atilir_dogru_yanit_kullanilir() {
    let cozucu = sahte_cozucu(
        Arc::new(|sorgu: &[u8]| {
            let mut yanlis_kimlik = yanit(sorgu, 0x81A0, &[mx(1, "sahte")]);
            yanlis_kimlik[0] ^= 0xFF;
            let mut yanlis_soru = yanit(sorgu, 0x81A0, &[mx(1, "sahte")]);
            let son = yanlis_soru.len() - mx(1, "sahte").len() - 4;
            yanlis_soru[son + 1] ^= 0x01; // soru türü MX → başka tür
            vec![
                yanlis_kimlik,
                yanlis_soru,
                b"cop".to_vec(),
                yanit(sorgu, 0x81A0, &[mx(20, "mx2"), mx(10, "mx1")]),
            ]
        }),
        None,
    );
    let (kod, cikti, hata) = calistir(cozucu.port, "bilgi", PROGRAM);
    assert_eq!(kod, 0, "{hata}");
    assert_eq!(cikti, "mx1.ornek.test\nyanlış\nmx2.ornek.test\nyanlış\n");
    let sorgular = cozucu.sorgular.lock().unwrap();
    assert_eq!(sorgular.len(), 1);
    assert_eq!(&sorgular[0][2..4], &[0x01, 0x20], "RD + AD bitleri");
    assert_eq!(
        &sorgular[0][sorgular[0].len() - 11..][..3],
        &[0, 0, 41],
        "EDNS OPT"
    );
}

#[test]
fn kesik_udp_yaniti_tcp_ile_yinelenir() {
    let cozucu = sahte_cozucu(
        Arc::new(|sorgu: &[u8]| vec![yanit(sorgu, 0x8380, &[])]),
        Some(Arc::new(|sorgu: &[u8]| {
            vec![yanit(sorgu, 0x8180, &[mx(5, "tcp")])]
        })),
    );
    let (kod, cikti, hata) = calistir(cozucu.port, "bilgi", PROGRAM);
    assert_eq!(kod, 0, "{hata}");
    assert_eq!(cikti, "tcp.ornek.test\nyanlış\n");
}

#[test]
fn servfail_bozuk_ve_sinir_disi_yanit_gecici_hatadir() {
    let durumlar: [(Arc<Isleyici>, &str); 3] = [
        (
            Arc::new(|s: &[u8]| vec![yanit(s, 0x8182, &[])]),
            "sunucu_hatası",
        ),
        (
            Arc::new(|s: &[u8]| {
                let mut p = yanit(s, 0x8180, &[mx(1, "a")]);
                p.push(0);
                vec![p]
            }),
            "bozuk_yanıt",
        ),
        (
            Arc::new(|s: &[u8]| {
                let mut p = yanit(s, 0x8180, &[]);
                p[6..8].copy_from_slice(&65u16.to_be_bytes());
                vec![p]
            }),
            "boyut_aşımı",
        ),
    ];
    for (isleyici, tur) in durumlar {
        let cozucu = sahte_cozucu(isleyici, None);
        let (kod, cikti, hata) = calistir(cozucu.port, "bilgi", PROGRAM);
        assert_eq!(kod, 0, "{hata}");
        assert_eq!(cikti, format!("C031\n{tur}\n"));
    }
}

#[test]
fn yanitsiz_cozucu_zaman_asimi_verir() {
    let cozucu = sahte_cozucu(Arc::new(|_: &[u8]| Vec::new()), None);
    let baslangic = std::time::Instant::now();
    let (kod, cikti, hata) = calistir(cozucu.port, "bilgi", PROGRAM);
    assert_eq!(kod, 0, "{hata}");
    assert_eq!(cikti, "C031\nzaman_aşımı\n");
    let sure = baslangic.elapsed();
    assert!(sure.as_secs_f64() >= 4.5 && sure.as_secs() < 30, "{sure:?}");
}

#[test]
fn dnssec_zorunlu_kipi_acilista_ad_bitini_sinar() {
    // Kök SOA AD=0: süreç kaynak yürütülmeden çıkış 2.
    let dogrulamayan = sahte_cozucu(
        Arc::new(|s: &[u8]| {
            let soa = [vec![0], vec![0], vec![0; 20]].concat();
            vec![yanit(s, 0x8180, &[kayit(6, &soa)])]
        }),
        None,
    );
    let (kod, cikti, hata) = calistir(dogrulamayan.port, "zorunlu", PROGRAM);
    assert_eq!(kod, 2);
    assert!(cikti.is_empty());
    assert!(hata.contains("dns_dnssec zorunlu"), "{hata}");
    // Doğrulayan çözücü: sınama geçer, AD=1 MX `doğru`.
    let dogrulayan = sahte_cozucu(
        Arc::new(|s: &[u8]| {
            if s.len() > 12 && s[12] == 0 {
                let soa = [vec![0], vec![0], vec![0; 20]].concat();
                vec![yanit(s, 0x81A0, &[kayit(6, &soa)])]
            } else {
                vec![yanit(s, 0x81A0, &[mx(1, "mx")])]
            }
        }),
        None,
    );
    let (kod, cikti, hata) = calistir(dogrulayan.port, "zorunlu", PROGRAM);
    assert_eq!(kod, 0, "{hata}");
    assert_eq!(cikti, "mx.ornek.test\ndoğru\n");
    // Bilgi kipinde AD=1 güvenlik kararına bağlanmaz.
    let (kod, cikti, _) = calistir(dogrulayan.port, "bilgi", PROGRAM);
    assert_eq!(kod, 0);
    assert_eq!(cikti, "mx.ornek.test\nyanlış\n");
}

#[test]
fn cozucu_yoksa_ag_hatasi_ve_dnssec_sinamasi_baslatmaz() {
    let bos = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = bos.local_addr().unwrap().port();
    drop(bos);
    let (kod, cikti, hata) = calistir(port, "bilgi", PROGRAM);
    assert_eq!(kod, 0, "{hata}");
    assert!(
        cikti == "C031\nağ\n" || cikti == "C031\nzaman_aşımı\n",
        "{cikti}"
    );
    let (kod, _, hata) = calistir(port, "zorunlu", PROGRAM);
    assert_eq!(kod, 2);
    assert!(hata.contains("dns_dnssec zorunlu"));
}

#[test]
fn onbellek_yok_her_sorgu_cozucuye_yeni_kimlikle_gider() {
    let cozucu = sahte_cozucu(
        Arc::new(|sorgu: &[u8]| vec![yanit(sorgu, 0x8180, &[mx(1, "mx")])]),
        None,
    );
    let program = "\
3 kez tekrarla
    sonuç \"ornek.test\" için MX kayıtlarını sorgulamayı dene olsun
    sonuç başarılıysa
        \"tamam\" yaz
";
    let (kod, cikti, hata) = calistir(cozucu.port, "bilgi", program);
    assert_eq!(kod, 0, "{hata}");
    assert_eq!(cikti, "tamam\ntamam\ntamam\n");
    let sorgular = cozucu.sorgular.lock().unwrap();
    assert_eq!(
        sorgular.len(),
        3,
        "aynı ad üç kez çözücüye gitmeli (önbellek yok)"
    );
    let kimlikler: std::collections::BTreeSet<_> = sorgular.iter().map(|s| [s[0], s[1]]).collect();
    assert!(
        kimlikler.len() >= 2,
        "sorgu kimlikleri rastgele olmalı: {kimlikler:?}"
    );
}
