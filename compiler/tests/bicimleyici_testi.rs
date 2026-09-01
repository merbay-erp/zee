//! Biçimleyici testleri — idempotence ve tam parser-token eşdeğerliği.

use dil::ayristirici::{ayristir_tohumla, islem_adlarini_tara};
use dil::bicimleyici::bicimle;
use dil::sozcukleyici::{sozcukle, TokenTur};

fn parser_izi(kaynak: &str) -> Vec<TokenTur> {
    sozcukle(kaynak)
        .expect("sözcüklenmeli")
        .into_iter()
        .map(|token| token.tur)
        .collect()
}

/// Geçerli kaynağın yalnız sunumsal boşluklarını deterministik olarak dağıtır.
/// Metin sabitlerinin ve yorum gövdelerinin byte'larına dokunmaz; girinti
/// genişliğini ikiye katlarken blok derinliğini korur.
fn bosluklari_dagit(kaynak: &str) -> String {
    let mut cikti = String::new();
    for (satir_no, satir) in kaynak.lines().enumerate() {
        let mut karakterler = satir.chars().peekable();
        while karakterler.peek() == Some(&' ') {
            cikti.push_str("  ");
            karakterler.next();
        }

        let mut tirnakta = false;
        let mut kacis = false;
        while let Some(karakter) = karakterler.next() {
            if tirnakta {
                cikti.push(karakter);
                if kacis {
                    kacis = false;
                } else if karakter == '\\' {
                    kacis = true;
                } else if karakter == '"' {
                    tirnakta = false;
                }
                continue;
            }
            if karakter == '"' {
                tirnakta = true;
                cikti.push(karakter);
            } else if karakter == '#' {
                cikti.push(karakter);
                cikti.extend(karakterler);
                break;
            } else if karakter == ' ' {
                cikti.push_str(if satir_no % 2 == 0 { "   " } else { "  " });
            } else {
                cikti.push(karakter);
            }
        }
        cikti.push('\n');
    }
    cikti
}

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
    assert!(
        sayilan >= 33,
        "genişletilmiş korpusun tamamı taranmalı (bulunan: {})",
        sayilan
    );
}

/// B-042/K-119: bütün geçerli golden programlarda biçimleme, parser'ın tam
/// token girdisini (SatirSonu/Girinti/Cikinti dahil) korur. Parser her iki
/// girdiyi de kabul eder; aynı yapısal token izi deterministik parser için
/// kaynak konumları dışındaki aynı AST semantiğinin yürütülebilir kanıtıdır.
#[test]
fn golden_korpus_parse_esdegerligini_korur() {
    let klasor = format!("{}/../golden", env!("CARGO_MANIFEST_DIR"));
    let birim_kaynagi = std::fs::read_to_string(format!("{klasor}/hesap_araclari.dil"))
        .expect("golden birim okunmalı");
    let birim_islemleri =
        islem_adlarini_tara(&sozcukle(&birim_kaynagi).expect("golden birim sözcüklenmeli"));
    let mut girdiler: Vec<_> = std::fs::read_dir(&klasor)
        .expect("golden klasörü okunmalı")
        .map(|girdi| girdi.expect("girdi").path())
        .filter(|yol| {
            yol.file_name()
                .and_then(|ad| ad.to_str())
                .is_some_and(|ad| {
                    ad.len() >= 4
                        && ad.as_bytes()[0].is_ascii_digit()
                        && ad.as_bytes()[1].is_ascii_digit()
                        && ad.as_bytes()[2] == b'-'
                        && ad.ends_with(".dil")
                })
        })
        .collect();
    girdiler.sort();

    for yol in &girdiler {
        let kaynak = std::fs::read_to_string(yol).expect("golden okunmalı");
        let dagitik = bosluklari_dagit(&kaynak);
        let bicimli = bicimle(&dagitik)
            .unwrap_or_else(|hata| panic!("{:?} biçimlenmeli: {hata}", yol.file_name()));

        let once = parser_izi(&dagitik);
        let sonra = parser_izi(&bicimli);
        assert_eq!(once, sonra, "parser izi değişti: {:?}", yol.file_name());
        ayristir_tohumla(
            sozcukle(&dagitik).expect("dağınık kaynak sözcüklenmeli"),
            birim_islemleri.clone(),
        )
        .unwrap_or_else(|hata| panic!("{:?} önce ayrışmalı: {hata}", yol.file_name()));
        ayristir_tohumla(
            sozcukle(&bicimli).expect("biçimli kaynak sözcüklenmeli"),
            birim_islemleri.clone(),
        )
        .unwrap_or_else(|hata| panic!("{:?} sonra ayrışmalı: {hata}", yol.file_name()));
    }
    assert_eq!(girdiler.len(), 33, "sayısal golden korpus bütünü taranmalı");
}

#[test]
fn parser_izi_satir_sinirini_semantik_sayar() {
    let iki_cumle = parser_izi("x 1 olsun\nx yaz\n");
    let tek_satir = parser_izi("x 1 olsun x yaz\n");
    assert_ne!(iki_cumle, tek_satir, "SatirSonu parser izinden düşürülemez");
    assert!(iki_cumle
        .iter()
        .any(|tur| matches!(tur, TokenTur::SatirSonu)));
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
    let girdi =
        "#yorum başı\nx 5 olsun   #  yan yorum\n10 kez tekrarla\n# içerideki yorum\n    x yaz\n";
    let bicimli = bicimle(girdi).unwrap();
    assert!(bicimli.contains("# yorum başı"));
    assert!(bicimli.contains("x 5 olsun  # yan yorum"));
    assert!(
        bicimli.contains("    # içerideki yorum"),
        "iç yorum bloğa hizalanmalı: {bicimli}"
    );
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
            // Windows checkout'u CRLF verebilir; kıyas LF üzerinden yapılır
            // (.gitattributes LF'i zorlar, bu satır ek savunmadır).
            let kaynak = std::fs::read_to_string(&dosya)
                .expect("okunmalı")
                .replace("\r\n", "\n");
            let bir = bicimle(&kaynak)
                .unwrap_or_else(|h| panic!("{:?} biçimlenmeli: {}", dosya.file_name(), h));
            assert_eq!(
                bir,
                kaynak,
                "{:?} resmi biçimde değil — `dil biçimle` koş",
                dosya.file_name()
            );
            let iki = bicimle(&bir).expect("ikinci geçiş");
            assert_eq!(bir, iki, "{:?} idempotent değil", dosya.file_name());
        }
    }
}
