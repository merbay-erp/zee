//! K-164/ADR-072 ikinci gerçek Zee ürünü: `dogfood/kanit-ozeti` depo kayıt
//! defterlerinden `docs/kanit-ozeti.md` üretir. Bu kapı ürünü hermetik IO ile
//! gerçek kayıt defteri içerikleri üzerinde koşar, üretilen sayfanın depodaki
//! sayfayla bayt bayt aynı olduğunu (tazelik) ve sayıların bağımsız Rust
//! sayımıyla tuttuğunu (doğruluk) ister.

use dil::yorumlayici::ToplayanIo;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const GIRIS: &str = "dogfood/kanit-ozeti/kaynak/ana.dil";
const CIKTI: &str = "docs/kanit-ozeti.md";
const KAYIT_DEFTERLERI: [&str; 9] = [
    "regression/v2.tsv",
    "docs/guvenlik-bulgulari-v1.tsv",
    "docs/spec-madde-kaniti-v1.tsv",
    "docs/dogfood-projeleri-v1.tsv",
    "dogfood/korpus-v1.tsv",
    "docs/deprecation-kayitlari-v1.tsv",
    "docs/soak-gecmisi-v1.tsv",
    "docs/compiler-degisiklik-beyanlari-v1.tsv",
    "docs/core-freeze-beyanlari-v1.tsv",
];

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
            .unwrap_or_else(|hata| panic!("kanit-ozeti derlenmeli: {hata:?}"));
    let bildirim = dil::proje::bildirimi_oku(&oku("dogfood/kanit-ozeti/proje.dil"))
        .expect("ürün bildirimi geçerli olmalı");
    dil::cozumleyici::yetkinlikleri_denetle(&program, &bildirim.yetkinlik_politikasi())
        .expect("ürün yalnız dosya-okuma/dosya-yazma ister");
    program
}

/// Ürünü gerçek kayıt defteri içerikleriyle hermetik olarak koşar; yazılan sayfayı verir.
fn urunu_kos(program: &dil::agac::Program) -> (String, Vec<String>) {
    let mut io = ToplayanIo::yeni(Vec::new());
    for defter in KAYIT_DEFTERLERI {
        io.dosyalar
            .insert(format!("../../../{defter}"), oku(defter));
    }
    dil::yorumlayici::calistir_io(program, &mut io).expect("kanit-ozeti çalışmalı");
    let sayfa = io
        .dosyalar
        .remove(&format!("../../../{CIKTI}"))
        .expect("ürün docs/kanit-ozeti.md yazmalı");
    (sayfa, io.cikti)
}

fn veri_satirlari(defter: &str) -> Vec<Vec<String>> {
    let metin = oku(defter);
    let baslik = metin
        .lines()
        .rfind(|s| s.starts_with("# ") && s.contains('\t'))
        .map(|s| {
            s.trim_start_matches("# ")
                .split('\t')
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .expect("başlık satırı");
    metin
        .lines()
        .filter(|s| !s.trim().is_empty() && !s.trim().starts_with('#'))
        .map(|s| {
            let alanlar = s.split('\t').collect::<Vec<_>>();
            baslik
                .iter()
                .enumerate()
                .map(|(i, _)| alanlar.get(i).map(|a| a.to_string()).unwrap_or_default())
                .collect()
        })
        .collect()
}

fn sutun_indeksi(defter: &str, ad: &str) -> usize {
    let metin = oku(defter);
    let baslik = metin
        .lines()
        .rfind(|s| s.starts_with("# ") && s.contains('\t'))
        .expect("başlık");
    baslik
        .trim_start_matches("# ")
        .split('\t')
        .position(|s| s == ad)
        .unwrap_or_else(|| panic!("{defter}: {ad} sütunu yok"))
}

fn sayim(defter: &str, sutun: &str) -> BTreeMap<String, usize> {
    let indeks = sutun_indeksi(defter, sutun);
    let mut sayaclar = BTreeMap::new();
    for satir in veri_satirlari(defter) {
        *sayaclar.entry(satir[indeks].clone()).or_insert(0) += 1;
    }
    sayaclar
}

#[test]
fn kanit_ozeti_urunu_derlenir_ve_butun_testleri_gecer() {
    let program = urunu_derle();
    let sonuclar = dil::programi_dene(&program);
    assert!(
        sonuclar.len() >= 19,
        "ürün en az 19 birim testi taşır: {}",
        sonuclar.len()
    );
    for sonuc in &sonuclar {
        assert!(
            sonuc.hata.is_none(),
            "ürün testi kaldı: {} → {:?}",
            sonuc.ad,
            sonuc.hata
        );
    }
    let birimler = sonuclar
        .iter()
        .filter_map(|s| s.ad.split_once(": ").map(|(b, _)| b))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        birimler.len() >= 7,
        "testler girişin doğrudan kullandığı yedi rapor birimine yayılmalı: {birimler:?}"
    );
}

#[test]
fn kanit_ozeti_sayfasi_depodakiyle_bayt_bayt_aynidir() {
    let program = urunu_derle();
    let (sayfa, cikti) = urunu_kos(&program);
    let (ikinci, _) = urunu_kos(&program);
    assert_eq!(sayfa, ikinci, "ürün çıktısı deterministik olmalı");
    let depodaki = oku(CIKTI);
    assert_eq!(
        sayfa, depodaki,
        "docs/kanit-ozeti.md bayat: `cd compiler && cargo run --locked -- çalıştır ../{GIRIS}` ile yenile"
    );
    let satir_sayisi = sayfa.trim_end_matches('\n').lines().count();
    assert_eq!(
        cikti,
        vec![format!("docs/kanit-ozeti.md yazıldı: {satir_sayisi} satır")]
    );
}

#[test]
fn kanit_ozeti_sayilari_bagimsiz_sayimla_tutar() {
    let sayfa = oku(CIKTI);
    let regresyon = veri_satirlari("regression/v2.tsv");
    let tam = regresyon
        .iter()
        .filter(|s| {
            s[sutun_indeksi("regression/v2.tsv", "fixed_by")]
                .chars()
                .count()
                == 40
        })
        .count();
    assert!(sayfa.contains(&format!(
        "Toplam {} vaka; `-` tanısız çalışma/eşzamanlılık vakasıdır. Tam 40 karakterlik `fixed_by` commit'i taşıyan vaka: {tam}/{}.",
        regresyon.len(),
        regresyon.len()
    )));
    for (faz, adet) in sayim("regression/v2.tsv", "faz") {
        assert!(
            sayfa.contains(&format!("| {faz} | {adet} |")),
            "regresyon faz satırı eksik: {faz} {adet}"
        );
    }
    let guvenlik = veri_satirlari("docs/guvenlik-bulgulari-v1.tsv");
    assert!(sayfa.contains(&format!("Toplam bulgu: {}.", guvenlik.len())));
    let onem = sutun_indeksi("docs/guvenlik-bulgulari-v1.tsv", "onem");
    let durum = sutun_indeksi("docs/guvenlik-bulgulari-v1.tsv", "durum");
    for seviye in ["kritik", "yuksek", "orta", "dusuk"] {
        let say = |d: &str| {
            guvenlik
                .iter()
                .filter(|s| s[onem] == seviye && s[durum] == d)
                .count()
        };
        let (kapali, kabul, acik) = (say("kapali"), say("kabul"), say("acik"));
        assert!(
            sayfa.contains(&format!(
                "| {seviye} | {kapali} | {kabul} | {acik} | {} |",
                kapali + kabul + acik
            )),
            "güvenlik satırı eksik: {seviye}"
        );
    }
    let acik_agir = guvenlik
        .iter()
        .filter(|s| (s[onem] == "kritik" || s[onem] == "yuksek") && s[durum] == "acik")
        .count();
    assert_eq!(acik_agir, 0);
    assert!(sayfa.contains("Kapı: açık kritik/yüksek bulgu 0 → **GEÇTİ**"));
    let spec = sayim("docs/spec-madde-kaniti-v1.tsv", "durum");
    let kanitli = spec.get("kanitli").copied().unwrap_or(0);
    let kismi = spec.get("kismi").copied().unwrap_or(0);
    let acik = spec.get("acik").copied().unwrap_or(0);
    let toplam = kanitli + kismi + acik;
    assert!(sayfa.contains(&format!(
        "| **Toplam** | {kanitli} | {kismi} | {acik} | {toplam} | {} |",
        kanitli * 100 / toplam
    )));
    let korpus = veri_satirlari("dogfood/korpus-v1.tsv");
    assert!(sayfa.contains(&format!("Toplam korpus vakası: {}.", korpus.len())));
    let degisiklik = veri_satirlari("docs/compiler-degisiklik-beyanlari-v1.tsv");
    let freeze = veri_satirlari("docs/core-freeze-beyanlari-v1.tsv");
    assert!(sayfa.contains(&format!(
        "Toplam: {} compiler değişiklik beyanı, {} core freeze beyanı.",
        degisiklik.len(),
        freeze.len()
    )));
}
