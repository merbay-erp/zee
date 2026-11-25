//! K-398 (urun-h ZM-F030, kanit-ozeti K-164/F006): ekli parametre adı sözlük
//! köküne bağlanır; zee-tr-1 profil çıktısı değişmez, K-398 öncesi yumuşamış
//! yazım aynı parametreye çözülür.

use dil::kaynagi_calistir;
use dil::morfoloji::{baglama_koku, kok_adaylari, profil_uyumluluk_kaydi};

fn calistir(kaynak: &str) -> Vec<String> {
    kaynagi_calistir(kaynak).unwrap_or_else(|tani| panic!("çalışmalı: {tani:?}\n{kaynak}"))
}

fn hata_kodu(kaynak: &str) -> String {
    kaynagi_calistir(kaynak)
        .expect_err("tanı beklenir")
        .kod
        .to_string()
}

/// ZM-F030'un bildirdiği yedi ad: gövdede sözlük kökü çıplak yazılır.
#[test]
fn yumusama_ve_unlu_dusmesi_parametre_adinda_sozluk_kokune_baglanir() {
    for (ekli, kok) in [
        ("kalıbı", "kalıp"),
        ("belirteci", "belirteç"),
        ("eşiği", "eşik"),
        ("adedi", "adet"),
        ("metni", "metin"),
        ("içeriği", "içerik"),
        ("ucu", "uç"),
        ("uzunluğu", "uzunluk"),
        ("izni", "izin"),
    ] {
        let kaynak = format!(
            "işlem yankı yap\n    {ekli} Metin olarak al\n    Metin döndürür\n\n    {kok} döndür\n\n\"z\" için yankı yap yaz\n"
        );
        assert_eq!(calistir(&kaynak), vec!["z"], "{ekli} → {kok}");
    }
}

/// Ekli başvurular K-398 öncesi gibi çözülmeye devam eder.
#[test]
fn ekli_basvurular_ayni_parametreye_cozulur() {
    let kaynak = "\
işlem eşleşme yap
    kalıbı Metin olarak al
    metni Metin olarak al
    Metin döndürür

    sonuç kalıp ile metni olsun
    sonuç kalıbı ile metin ile sonuç olsun
    sonucu döndür

\"a\" ve \"b\" için eşleşme yap yaz
";
    assert_eq!(calistir(kaynak), vec!["abab"]);
}

/// K-398 öncesi programlar yumuşamış gövdeyi (`kalıb`, `metn`) çıplak yazıyordu;
/// aynı parametreye çözülür ve atama hedefi olarak yeni ad doğurmaz.
#[test]
fn eski_yumusamis_yazim_ayni_parametreye_cozulur_ve_atamadir() {
    let kaynak = "\
işlem eski yazım yap
    belirteci Metin olarak al
    metni Metin olarak al
    Metin döndürür

    belirtec \"<\" ile belirtec olsun
    belirtec ile metn ile belirteci döndür

\"x\" ve \"y\" için eski yazım yap yaz
";
    assert_eq!(calistir(kaynak), vec!["<xy<x"]);
}

/// Sözlükte yumuşamayan ya da tek heceli adlar değişmez: `adı` hâlâ `ad`dır,
/// `at` yeni bir ad değildir; `-lı` sıfatı ve `film` gibi kümeler kalır.
#[test]
fn yumusamayan_adlar_eski_baglamasini_korur() {
    for (ekli, kok) in [
        ("adı", "ad"),
        ("kodu", "kod"),
        ("ağı", "ağ"),
        ("posta_kodu", "posta_kod"),
        ("katalogu", "katalog"),
        ("filmi", "film"),
        ("formu", "form"),
        ("gizli", "gizl"),
    ] {
        let kaynak = format!(
            "işlem yankı yap\n    {ekli} Metin olarak al\n    Metin döndürür\n\n    {kok} döndür\n\n\"z\" için yankı yap yaz\n"
        );
        assert_eq!(calistir(&kaynak), vec!["z"], "{ekli} → {kok}");
    }
    assert_eq!(
        hata_kodu(
            "işlem yankı yap\n    adı Metin olarak al\n    Metin döndürür\n\n    at döndür\n\n\"z\" için yankı yap yaz\n"
        ),
        "A001"
    );
}

/// K-398 öncesi `başlık … olsun` ayrı yereldi; artık parametreye atamadır.
/// Farklı türdeki değer sessizce yeni ad açmaz, önerili T002 olur.
#[test]
fn sozluk_kokune_farkli_turde_atama_onerili_t002_olur() {
    let kaynak = "\
işlem başlık yap
    başlığı Metin olarak al
    Metin döndürür

    başlık 5 olsun
    \"x\" döndür

\"a\" için başlık yap yaz
";
    let tani = kaynagi_calistir(kaynak).expect_err("T002 beklenir");
    assert_eq!(tani.kod, "T002");
    assert!(
        tani.oneri
            .as_deref()
            .unwrap_or("")
            .contains("parametresidir"),
        "{tani:?}"
    );
}

/// Parametre sözlük köküyle yeniden atandıktan sonra eski yazım iki ayrı
/// K-398 öncesi değerden hangisini kastettiğini söylemez: A002.
#[test]
fn kokle_atanan_parametrede_eski_yazim_a002_olur() {
    let kaynak = "\
işlem başlık yap
    başlığı Metin olarak al
    Metin döndürür

    başlık \"## \" ile başlığ olsun
    başlığ döndür

\"a\" için başlık yap yaz
";
    let tani = kaynagi_calistir(kaynak).expect_err("A002 beklenir");
    assert_eq!(tani.kod, "A002");
    assert!(tani.mesaj.contains("eski parametre yazımı"), "{tani:?}");
}

/// Seçim yalnız profil aday kümesinden yapılır; profil çıktısı ve
/// uyumluluk parmak izi değişmez.
#[test]
fn baglama_koku_profil_aday_kumesinden_secilir_profil_degismez() {
    for ekli in [
        "kalıbı",
        "belirteci",
        "eşiği",
        "adedi",
        "metni",
        "içeriği",
        "ucu",
    ] {
        let govde: String = {
            let mut harfler: Vec<char> = ekli.chars().collect();
            harfler.pop();
            harfler.into_iter().collect()
        };
        let kok = baglama_koku(&govde).expect("geri çevrim");
        assert!(kok_adaylari(ekli).contains(&kok), "{ekli}: {kok}");
        assert!(kok_adaylari(ekli).contains(&govde), "{ekli}: {govde}");
    }
    for govde in [
        "ad", "kod", "ağ", "dağ", "katalog", "film", "izinl", "modern", "kayb",
    ] {
        assert_eq!(baglama_koku(govde), None, "{govde}");
    }
    assert_eq!(
        profil_uyumluluk_kaydi(),
        include_str!("fixtures/morfoloji-zee-tr-1.sha256"),
        "zee-tr-1 uyumluluk kaydı değişmemeli"
    );
}
