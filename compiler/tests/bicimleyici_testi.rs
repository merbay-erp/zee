//! Biçimleyici testleri — v0.1 kabul kriteri: "Formatter idempotent."

use dil::bicimleyici::bicimle;

/// Bütün golden korpus: biçimle(biçimle(x)) == biçimle(x).
#[test]
fn golden_korpus_idempotent() {
    let klasor = format!("{}/../golden", env!("CARGO_MANIFEST_DIR"));
    let mut sayilan = 0;
    let mut girdiler: Vec<_> = std::fs::read_dir(&klasor)
        .expect("golden klasörü okunmalı")
        .map(|g| g.expect("girdi").path())
        .filter(|yol| yol.extension().is_some_and(|u| u == "dil"))
        .collect();
    girdiler.sort();
    for yol in girdiler {
        let kaynak = std::fs::read_to_string(&yol).expect("okunmalı");
        let bir = bicimle(&kaynak)
            .unwrap_or_else(|hata| panic!("{:?} biçimlenmeli: {}", yol.file_name(), hata));
        let iki = bicimle(&bir)
            .unwrap_or_else(|hata| panic!("{:?} ikinci tur: {}", yol.file_name(), hata));
        assert_eq!(bir, iki, "idempotent değil: {:?}", yol.file_name());
        sayilan += 1;
    }
    assert!(sayilan >= 33, "genişletilmiş korpusun tamamı taranmalı (bulunan: {})", sayilan);
}

#[test]
fn dagitik_bosluklar_toparlanir() {
    // Not: "3 ,7" artık S033'tür (RFC-0013 bitişik virgül kuralı); dağınık
    // girdi virgülden-sonra-boşluk biçimleriyle sınanır.
    let girdi = "isim    \"Ayşe\"   olsun\nsayılar 3,  7,   1 listesi olsun\n";
    let beklenen = "isim \"Ayşe\" olsun\nsayılar 3, 7, 1 listesi olsun\n";
    assert_eq!(bicimle(girdi).unwrap(), beklenen);
}

#[test]
fn girinti_dort_bosluga_cekilir() {
    let girdi = "10 kez tekrarla\n        \"Merhaba\" yaz\n";
    let beklenen = "10 kez tekrarla\n    \"Merhaba\" yaz\n";
    assert_eq!(bicimle(girdi).unwrap(), beklenen);
}

#[test]
fn yorumlar_korunur_ve_hizalanir() {
    let girdi = "#yorum başı\nx 5 olsun   #  yan yorum\n10 kez tekrarla\n# içerideki yorum\n    x yaz\n";
    let bicimli = bicimle(girdi).unwrap();
    assert!(bicimli.contains("# yorum başı"));
    assert!(bicimli.contains("x 5 olsun  # yan yorum"));
    assert!(bicimli.contains("    # içerideki yorum"), "iç yorum bloğa hizalanmalı: {bicimli}");
    assert_eq!(bicimle(&bicimli).unwrap(), bicimli);
}

#[test]
fn bicim_tokenlara_dokunmaz() {
    // Metin sabitinin İÇİ asla değişmez (fazla boşluklar dahil).
    let girdi = "mesaj \"iki   boşluk , korunur\" olsun\n";
    let bicimli = bicimle(girdi).unwrap();
    assert!(bicimli.contains("\"iki   boşluk , korunur\""));
}

/// Proje kitaplığı + gömülü kitaplık da biçim hijyenine tabidir: biçimleme
/// idempotenttir VE depodaki dosyalar zaten resmi biçimdedir.
#[test]
fn projeler_ve_kitaplik_bicimli() {
    for klasor in ["projeler", "kitaplik"] {
        let yol = format!("{}/../{}", env!("CARGO_MANIFEST_DIR"), klasor);
        let mut girdiler: Vec<_> = std::fs::read_dir(&yol)
            .expect("klasör okunmalı")
            .map(|g| g.expect("girdi").path())
            .filter(|y| y.extension().is_some_and(|u| u == "dil"))
            .collect();
        girdiler.sort();
        assert!(!girdiler.is_empty());
        for dosya in girdiler {
            let kaynak = std::fs::read_to_string(&dosya).expect("okunmalı");
            let bir = bicimle(&kaynak)
                .unwrap_or_else(|h| panic!("{:?} biçimlenmeli: {}", dosya.file_name(), h));
            assert_eq!(
                bir, kaynak,
                "{:?} resmi biçimde değil — `dil biçimle` koş",
                dosya.file_name()
            );
            let iki = bicimle(&bir).expect("ikinci geçiş");
            assert_eq!(bir, iki, "{:?} idempotent değil", dosya.file_name());
        }
    }
}
