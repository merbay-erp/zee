//! K-179/ADR-074 üçüncü depo içi ürün: kelime avı (etkileşimli CLI oyunu).
//! Ürün gerçek sözlük dosyasıyla hermetik IO'da koşar: kazanma, kaybetme,
//! kısa tahmin, boş sözlük ve birim testleri.

use dil::yorumlayici::ToplayanIo;
use std::path::{Path, PathBuf};

const GIRIS: &str = "dogfood/kelime-avi/kaynak/ana.dil";
const SOZLUK: &str = "dogfood/kelime-avi/kaynak/kelimeler.txt";

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü")
        .to_path_buf()
}

fn oku(goreli: &str) -> String {
    std::fs::read_to_string(depo().join(goreli))
        .unwrap_or_else(|hata| panic!("{goreli} okunmalı: {hata}"))
}

fn urunu_derle() -> dil::agac::Program {
    let giris = depo().join(GIRIS);
    let klasor = giris.parent().expect("kaynak klasörü").to_path_buf();
    let mut yukleyici = |istek: dil::BirimIstegi<'_>| -> Result<dil::YuklenenBirim, String> {
        let yol = klasor.join(format!("{}.dil", istek.ad));
        std::fs::read_to_string(&yol)
            .map(|kaynak| dil::YuklenenBirim {
                kaynak,
                koken: yol.to_string_lossy().into_owned(),
            })
            .map_err(|hata| hata.to_string())
    };
    let program =
        dil::kaynagi_derle_kokenlerle(&oku(GIRIS), Some(&giris.to_string_lossy()), &mut yukleyici)
            .unwrap_or_else(|hata| panic!("kelime-avi derlenmeli: {hata:?}"));
    let bildirim = dil::proje::bildirimi_oku(&oku("dogfood/kelime-avi/proje.dil"))
        .expect("ürün bildirimi geçerli olmalı");
    dil::cozumleyici::yetkinlikleri_denetle(&program, &bildirim.yetkinlik_politikasi())
        .expect("ürün yalnız dosya-okuma/dosya-yazma ister");
    program
}

fn oyun_io(sozluk: &str, rastgele: i64, argumanlar: &[&str], girdiler: &[&str]) -> ToplayanIo {
    let mut io = ToplayanIo::yeni(girdiler.iter().map(|g| g.to_string()).collect());
    io.dosyalar
        .insert("kelimeler.txt".into(), sozluk.to_string());
    io.rastgele_degerler.push_back(rastgele);
    io.argumanlar = argumanlar.iter().map(|a| a.to_string()).collect();
    io
}

#[test]
fn urun_derlenir_ve_birim_testleri_gecer() {
    let program = urunu_derle();
    let sonuclar = dil::programi_dene(&program);
    let kalan: Vec<String> = sonuclar
        .iter()
        .filter_map(|s| {
            s.hata
                .as_ref()
                .map(|h| format!("{}: {} {}", s.ad, h.kod, h.mesaj))
        })
        .collect();
    assert!(kalan.is_empty(), "birim testleri geçmeli: {kalan:?}");
    assert_eq!(sonuclar.len(), 7, "üç birimin yedi testi koşmalı");
}

#[test]
fn sozluk_bes_harfli_kelimeler_tasir_ve_ucuncu_kelime_kitaptir() {
    let sozluk = oku(SOZLUK);
    let bes_harfli: Vec<&str> = sozluk
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#') && s.chars().count() == 5)
        .collect();
    assert!(
        bes_harfli.len() >= 20,
        "oyun için yeterli kelime: {}",
        bes_harfli.len()
    );
    assert_eq!(
        bes_harfli[2], "kitap",
        "test senaryoları üçüncü kelimeye bağlıdır"
    );
}

#[test]
fn kazanan_oyun_ipucu_verir_skoru_ekler_ve_en_iyileri_gosterir() {
    let program = urunu_derle();
    let mut io = oyun_io(&oku(SOZLUK), 3, &[], &["Zeynep", "kalem", "kitap"]);
    let cikis = dil::yorumlayici::calistir_io_kodla(&program, &mut io).expect("oyun çalışmalı");
    assert_eq!(cikis, 0);
    assert!(
        io.cikti.iter().any(|s| s.starts_with("Merhaba Zeynep!")),
        "{:?}",
        io.cikti
    );
    assert!(
        io.cikti.contains(&"●○···".to_string()),
        "kalem→kitap ipucu: {:?}",
        io.cikti
    );
    assert!(
        io.cikti.contains(&"Bildin! 2 denemede.".to_string()),
        "{:?}",
        io.cikti
    );
    assert_eq!(
        io.dosyalar.get("skorlar.txt").map(String::as_str),
        Some("Zeynep|2|kitap\n")
    );
    assert!(io.cikti.contains(&"--- en iyiler ---".to_string()));
    assert_eq!(
        io.cikti.last().map(String::as_str),
        Some("Zeynep: 2 deneme")
    );
}

#[test]
fn kaybeden_oyun_kelimeyi_soyler_ve_kisa_tahmin_hak_yakmaz() {
    let program = urunu_derle();
    let girdiler = ["abc", "kalem", "kalem", "kalem", "kalem", "kalem", "kalem"];
    let mut io = oyun_io(&oku(SOZLUK), 1, &["Ali"], &girdiler);
    let cikis = dil::yorumlayici::calistir_io_kodla(&program, &mut io).expect("oyun çalışmalı");
    assert_eq!(cikis, 0);
    let uyari = io
        .cikti
        .iter()
        .filter(|s| *s == "Beş harfli bir kelime yaz.")
        .count();
    assert_eq!(uyari, 1, "kısa tahmin bir kez uyarılır, hak yakmaz");
    let ipucu = io.cikti.iter().filter(|s| *s == "·○○○○").count();
    assert_eq!(
        ipucu, 6,
        "altı geçerli tahminin her biri ipucu alır: {:?}",
        io.cikti
    );
    assert!(io
        .cikti
        .contains(&"Hakların bitti. Kelime: elmas".to_string()));
    assert!(
        !io.dosyalar.contains_key("skorlar.txt"),
        "kaybeden skor yazmaz"
    );
    assert!(!io.cikti.iter().any(|s| s == "--- en iyiler ---"));
    assert!(
        io.cikti.iter().any(|s| s.starts_with("Merhaba Ali!")),
        "argüman adı kullanılır"
    );
}

#[test]
fn onceki_skorlar_ozete_girer_ve_en_kucuk_deneme_kazanir() {
    let program = urunu_derle();
    let mut io = oyun_io(&oku(SOZLUK), 2, &["Zeynep"], &["kalem"]);
    io.dosyalar.insert(
        "skorlar.txt".into(),
        "Ali|4|kitap\nZeynep|5|elmas\n".to_string(),
    );
    dil::yorumlayici::calistir_io_kodla(&program, &mut io).expect("oyun çalışmalı");
    assert!(
        io.cikti.contains(&"Bildin! 1 denemede.".to_string()),
        "{:?}",
        io.cikti
    );
    let son_iki: Vec<&str> = io.cikti.iter().rev().take(2).map(String::as_str).collect();
    assert_eq!(son_iki, vec!["Zeynep: 1 deneme", "Ali: 4 deneme"]);
}

#[test]
fn bos_sozluk_acik_mesajla_ikinci_cikis_kodunu_verir() {
    let program = urunu_derle();
    let mut io = oyun_io("# yalnız yorum\n\n", 1, &["Zeynep"], &[]);
    let cikis = dil::yorumlayici::calistir_io_kodla(&program, &mut io).expect("çıkış kodu dönmeli");
    assert_eq!(cikis, 2);
    assert_eq!(
        io.cikti,
        vec!["Sözlük boş: kaynak/kelimeler.txt içine beş harfli kelimeler yaz."]
    );
}
