//! Son beş golden: HTTP (24), sunucu (25), eşzamanlılık (26), zaman aşımı (27),
//! ESP32 simülatörü (29) — hepsi hermetik IO ile deterministik.

use dil::yorumlayici::{calistir_io, ToplayanIo};

fn golden(ad: &str) -> String {
    let yol = format!("{}/../golden/{}", env!("CARGO_MANIFEST_DIR"), ad);
    std::fs::read_to_string(&yol).unwrap_or_else(|_| panic!("golden bulunamadı: {}", yol))
}

#[test]
fn golden_24_http_istemcisi() {
    let program = dil::kaynagi_derle(&golden("24-http-istemcisi.dil")).expect("24 derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.http_yanitlari.insert(
        "https://ornek.dev/durum".into(),
        (200, "çalışıyor".into()),
    );
    calistir_io(&program, &mut io).expect("24 çalışmalı");
    assert_eq!(io.cikti, vec!["Durum: 200", "çalışıyor"]);
}

#[test]
fn http_baglanti_hatasi_turkce() {
    let kaynak = "cevap \"https://yok.example\" adresinden gelen yanıt olsun\ncevabın gövdesini yaz\n";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    let hata = calistir_io(&program, &mut io).expect_err("bağlantı hatası");
    assert_eq!(hata.kod, "C018");
}

#[test]
fn golden_25_web_sunucusu() {
    let program = dil::kaynagi_derle(&golden("25-web-sunucusu.dil")).expect("25 derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec!["/durum".into(), "/selam".into(), "/kayip".into()].into();
    calistir_io(&program, &mut io).expect("25 çalışmalı");
    assert_eq!(
        io.sunucu_yanitlari,
        vec![
            ("/durum".to_string(), "çalışıyor".to_string()),
            ("/selam".to_string(), "Merhaba ziyaretçi".to_string()),
            ("/kayip".to_string(), "aranan sayfa yok: /kayip".to_string()),
        ]
    );
}

#[test]
fn golden_26_paralel_gorevler() {
    let cikti = dil::kaynagi_calistir(&golden("26-paralel-gorevler.dil")).expect("26 çalışmalı");
    assert_eq!(cikti, vec!["Ayşe profili", "Ayşe faturaları"]);
}

#[test]
fn bekle_oncesi_erisim_derleme_hatasi() {
    // RFC-0011 §1: görev sonucuna bekle'den önce erişim T033.
    let kaynak = "\
işlem bir ver
    1 döndür

eşzamanlı olarak
    görev bir ver

görev yaz
hepsini bekle
";
    let hata = dil::kaynagi_calistir(kaynak).expect_err("T033 bekleniyor");
    assert_eq!(hata.kod, "T033");
}

#[test]
fn golden_27_zaman_asimi() {
    let program = dil::kaynagi_derle(&golden("27-zaman-asimi.dil")).expect("27 derlenmeli");

    // Yetişen durum: an ölçümleri 0 → 10ms; gövde çıktısı görünür, yetişmezse koşulmaz.
    let mut io = ToplayanIo::yeni(Vec::new());
    io.an_degerleri = vec![0, 10].into();
    io.http_yanitlari.insert(
        "https://ornek.dev/rapor".into(),
        (200, "rapor içeriği".into()),
    );
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti, vec!["rapor içeriği"]);

    // Geç kalan durum: 0 → 9000ms (> 5 saniye) → yetişmezse kolu da koşulur.
    let mut io = ToplayanIo::yeni(Vec::new());
    io.an_degerleri = vec![0, 9000].into();
    io.http_yanitlari.insert(
        "https://ornek.dev/rapor".into(),
        (200, "rapor içeriği".into()),
    );
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(
        io.cikti,
        vec!["rapor içeriği", "Zaman aşımı, sonra tekrar dene"]
    );
}

#[test]
fn golden_29_esp32_simulatoru() {
    let program = dil::kaynagi_derle(&golden("29-esp32-led.dil")).expect("29 derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.sensorler.insert("kapı".into(), true);
    calistir_io(&program, &mut io).expect("29 çalışmalı");
    let mut beklenen = vec!["[ışık] kırmızı yandı".to_string()];
    for _ in 0..10 {
        beklenen.push("[ışık] mavi yandı".into());
        beklenen.push("[ışık] mavi söndü".into());
    }
    assert_eq!(io.cikti, beklenen);

    // Kapı kapalıyken yeşil yanar (varsayılan sensör durumu).
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("29 çalışmalı");
    assert_eq!(io.cikti[0], "[ışık] yeşil yandı");
}

#[test]
fn golden_31_birimler() {
    let klasor = format!("{}/../golden", env!("CARGO_MANIFEST_DIR"));
    let mut yukleyici = |ad: &str| -> Result<String, String> {
        std::fs::read_to_string(format!("{}/{}.dil", klasor, ad)).map_err(|e| e.to_string())
    };
    let program = dil::kaynagi_derle_birimlerle(&golden("31-birimler.dil"), &mut yukleyici)
        .expect("31 derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("31 çalışmalı");
    assert_eq!(io.cikti, vec!["Ödenecek: 59,88 lira"]);

    // Birimin kendi testi de dene kapsamında.
    let sonuclar = dil::programi_dene(&program);
    assert!(sonuclar.iter().any(|s| s.ad == "hesap_araclari: kdv doğru eklenir"));
    assert!(sonuclar.iter().all(|s| s.hata.is_none()));
}

#[test]
fn golden_32_ondalik_market() {
    let cikti = dil::kaynagi_calistir(&golden("32-ondalik-market.dil")).expect("32 çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "Tutar: 49,975 lira",
            "Yuvarlak: 50 lira",
            "Ondalıklar tam: sürpriz yok",
        ]
    );
}
