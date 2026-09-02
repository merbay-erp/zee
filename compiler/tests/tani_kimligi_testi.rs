//! Yayımlanmış tanı kodu → anlam bağını sürümler arasında korur.
//!
//! Katalog birebirlik testi kodların varlığını denetler. Bu fixture kapısı ise
//! aynı kodun sessizce başka bir anlama taşınmasını ve kaldırılan kodun yeniden
//! kullanılmasını engeller.

use std::collections::{BTreeMap, BTreeSet};

const KIMLIK_FIXTURE: &str = include_str!("fixtures/tani-kimlikleri-v1.tsv");
const HATA_KATALOGU: &str = include_str!("../../docs/hata-katalogu.md");

#[derive(Debug, Eq, PartialEq)]
struct Kayit<'a> {
    durum: &'a str,
    kimlik: &'a str,
    ozet: &'a str,
}

fn gecerli_kod(kod: &str) -> bool {
    let karakterler = kod.chars().collect::<Vec<_>>();
    karakterler.len() == 4
        && matches!(karakterler[0], 'S' | 'A' | 'T' | 'C' | 'D' | 'P' | 'Ç')
        && karakterler[1..].iter().all(char::is_ascii_digit)
}

fn beklenen_kimlik_ailesi(kod: &str) -> &'static str {
    match kod.chars().next().expect("tanı kodu boş olamaz") {
        'S' => "soz.",
        'A' => "ad.",
        'T' => "tur.",
        'C' => "calisma.",
        'D' => "dogrulama.",
        'P' => "proje.",
        'Ç' => "ic.",
        on_ek => panic!("bilinmeyen tanı ailesi: {on_ek}"),
    }
}

fn fixture_kayitlari() -> BTreeMap<&'static str, Kayit<'static>> {
    assert!(
        KIMLIK_FIXTURE.starts_with("# Zee tanı kimliği fixture'ı — şema 1\n"),
        "fixture şeması açıkça sürümlenmeli"
    );

    let mut kayitlar = BTreeMap::new();
    let mut kimlikler = BTreeSet::new();
    for (indis, satir) in KIMLIK_FIXTURE.lines().enumerate() {
        if satir.is_empty() || satir.starts_with('#') {
            continue;
        }
        let hucreler = satir.split('\t').collect::<Vec<_>>();
        assert_eq!(
            hucreler.len(),
            4,
            "fixture satırı {} dört sekmeli alan taşımalı",
            indis + 1
        );
        let [durum, kod, kimlik, ozet] = hucreler.as_slice() else {
            unreachable!("alan sayısı yukarıda doğrulandı")
        };
        assert!(
            matches!(*durum, "aktif" | "ayrilmis"),
            "{kod}: geçersiz durum {durum}"
        );
        assert!(gecerli_kod(kod), "geçersiz tanı kodu: {kod}");
        assert!(
            kimlik.starts_with(beklenen_kimlik_ailesi(kod)),
            "{kod}: {kimlik} yanlış tanı ailesinde"
        );
        assert!(
            kimlik.chars().all(|karakter| karakter.is_ascii_lowercase()
                || karakter.is_ascii_digit()
                || matches!(karakter, '.' | '_')),
            "{kod}: kararlı kimlik yalnız küçük ASCII, rakam, nokta ve alt çizgi taşımalı: {kimlik}"
        );
        assert!(!ozet.is_empty(), "{kod}: kanonik özet boş olamaz");
        assert!(
            kimlikler.insert(*kimlik),
            "yinelenen kararlı kimlik: {kimlik}"
        );
        assert!(
            kayitlar
                .insert(
                    *kod,
                    Kayit {
                        durum,
                        kimlik,
                        ozet
                    }
                )
                .is_none(),
            "yinelenen fixture kodu: {kod}"
        );
    }
    kayitlar
}

fn katalog_kayitlari() -> BTreeMap<&'static str, Kayit<'static>> {
    let mut kayitlar = BTreeMap::new();
    for satir in HATA_KATALOGU.lines() {
        let hucreler = satir.split('|').map(str::trim).collect::<Vec<_>>();
        let Some(kod) = hucreler.get(1).copied().filter(|kod| gecerli_kod(kod)) else {
            continue;
        };
        let ozet = hucreler
            .get(2)
            .copied()
            .expect("katalog tanısı kanonik özet taşımalı");
        let durum = if ozet.contains("**ayrılmış**") {
            "ayrilmis"
        } else {
            "aktif"
        };
        assert!(
            kayitlar
                .insert(
                    kod,
                    Kayit {
                        durum,
                        kimlik: "",
                        ozet
                    }
                )
                .is_none(),
            "yinelenen katalog kodu: {kod}"
        );
    }
    kayitlar
}

#[test]
fn tani_kodlari_surumler_arasi_kimligini_korur() {
    let fixture = fixture_kayitlari();
    let katalog = katalog_kayitlari();

    assert_eq!(
        fixture.len(),
        155,
        "şema-1 tabanı beklenmedik biçimde değişti"
    );
    assert_eq!(
        katalog.len(),
        fixture.len(),
        "katalog ve kimlik fixture'ı birebir olmalı"
    );

    for (kod, sabit) in &fixture {
        let belgeli = katalog
            .get(kod)
            .unwrap_or_else(|| panic!("{kod}: kimlik fixture'ında var, katalogda yok"));
        assert_eq!(
            belgeli.durum, sabit.durum,
            "{kod}: aktif/ayrılmış durumu değişti"
        );
        assert_eq!(
            belgeli.ozet, sabit.ozet,
            "{kod}: kanonik anlam sessizce değişti; yeni anlam için yeni kod ayır. Yalnız editoryal bir düzeltmeyse fixture değişikliği kod incelemesinde açıkça görünmeli"
        );
    }

    for kod in katalog.keys() {
        assert!(
            fixture.contains_key(kod),
            "{kod}: katalogda var, kimlik fixture'ında yok"
        );
    }
}
