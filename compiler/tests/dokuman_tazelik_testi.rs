//! Yapısal belge tazelik kapısı: yeni karar/spec dosyası indekslenmeden ve
//! yerel Markdown bağlantısı kırıkken toplu iş tamamlanamaz.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn markdown_dosyalari(kok: &Path) -> Vec<PathBuf> {
    fn gez(klasor: &Path, sonuc: &mut Vec<PathBuf>) {
        let mut girdiler = std::fs::read_dir(klasor)
            .expect("belge klasörü okunmalı")
            .collect::<Result<Vec<_>, _>>()
            .expect("belge girdileri okunmalı");
        girdiler.sort_by_key(|girdi| girdi.file_name());
        for girdi in girdiler {
            let yol = girdi.path();
            let tur = girdi.file_type().expect("dosya türü");
            if tur.is_dir() {
                gez(&yol, sonuc);
            } else if tur.is_file() && yol.extension().and_then(|e| e.to_str()) == Some("md") {
                sonuc.push(yol);
            }
        }
    }
    let mut sonuc = Vec::new();
    gez(kok, &mut sonuc);
    sonuc
}

fn indeksi_dogrula(klasor: &Path, indeks: &Path, on_ek_uzunlugu: usize) {
    let metin = std::fs::read_to_string(indeks).expect("indeks okunmalı");
    for yol in markdown_dosyalari(klasor) {
        if yol == indeks {
            continue;
        }
        let ad = yol
            .file_name()
            .and_then(|ad| ad.to_str())
            .expect("UTF-8 ad");
        let numarali = ad
            .chars()
            .take(on_ek_uzunlugu)
            .all(|karakter| karakter.is_ascii_digit());
        if numarali {
            assert!(
                metin.contains(&format!("({})", ad)),
                "{} dosyası {} içinde indekslenmemiş",
                yol.display(),
                indeks.display()
            );
        }
    }
}

fn yerel_baglantilari_dogrula(dosya: &Path) {
    let metin = std::fs::read_to_string(dosya).expect("Markdown okunmalı");
    for parca in metin.split("](").skip(1) {
        let Some(mut hedef) = parca.split(')').next() else {
            continue;
        };
        hedef = hedef.trim().trim_start_matches('<').trim_end_matches('>');
        if hedef.is_empty()
            || hedef.starts_with('#')
            || hedef.starts_with("http://")
            || hedef.starts_with("https://")
            || hedef.starts_with("mailto:")
        {
            continue;
        }
        hedef = hedef.split('#').next().unwrap_or(hedef);
        let cozulmus = dosya.parent().expect("belge ebeveyni").join(hedef);
        assert!(
            cozulmus.exists(),
            "{} içindeki {:?} bağlantısı var olmayan {} hedefine gidiyor",
            dosya.display(),
            hedef,
            cozulmus.display()
        );
    }
}

fn numarali_belgeler(klasor: &Path, basamak: usize, sablon: &str) -> BTreeSet<String> {
    markdown_dosyalari(klasor)
        .into_iter()
        .filter_map(|yol| {
            let ad = yol.file_name()?.to_str()?;
            let on_ek = ad.as_bytes().get(..basamak)?;
            if on_ek.iter().all(u8::is_ascii_digit)
                && ad.as_bytes().get(basamak) == Some(&b'-')
                && &ad[..basamak] != sablon
            {
                Some(ad.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn rfc_adr_ve_spec_dosyalari_indeksli() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    indeksi_dogrula(&depo.join("rfcs"), &depo.join("rfcs/README.md"), 4);
    indeksi_dogrula(&depo.join("adr"), &depo.join("adr/README.md"), 3);
    indeksi_dogrula(&depo.join("spec"), &depo.join("spec/README.md"), 2);
}

#[test]
fn yerel_markdown_baglantilari_bayat_degil() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    let mut dosyalar = vec![depo.join("README.md"), depo.join("AGENTS.md")];
    for klasor in ["docs", "rfcs", "adr", "spec"] {
        dosyalar.extend(markdown_dosyalari(&depo.join(klasor)));
    }
    for dosya in dosyalar {
        yerel_baglantilari_dogrula(&dosya);
    }
}

#[test]
fn her_normatif_belge_makine_okunur_test_kaniti_tasir() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    let mut beklenen = BTreeSet::new();
    for (klasor, basamak, sablon) in [("rfcs", 4, "0000"), ("adr", 3, "000"), ("spec", 2, "00")] {
        beklenen.extend(
            numarali_belgeler(&depo.join(klasor), basamak, sablon)
                .into_iter()
                .map(|ad| format!("{klasor}/{ad}")),
        );
    }

    let harita_yolu = depo.join("docs/kanit-haritasi-v1.tsv");
    let harita = std::fs::read_to_string(&harita_yolu).expect("kanıt haritası okunmalı");
    let mut gorulen = BTreeSet::new();
    for (sira, satir) in harita.lines().enumerate() {
        if satir.is_empty() || satir.starts_with('#') {
            continue;
        }
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        assert_eq!(
            alanlar.len(),
            4,
            "kanıt haritası satırı {} dört sekmeli alan taşımalı",
            sira + 1
        );
        let [belge, durum, testler, not] = alanlar.as_slice() else {
            unreachable!("alan sayısı doğrulandı")
        };
        assert!(
            beklenen.contains(*belge),
            "bilinmeyen kanıt belgesi: {belge}"
        );
        assert!(
            gorulen.insert((*belge).to_string()),
            "yinelenen kanıt belgesi: {belge}"
        );
        assert!(matches!(*durum, "kanitli" | "kismi" | "taslak"));
        assert!(!not.trim().is_empty(), "{belge}: kapsam notu boş olamaz");

        if *durum == "taslak" {
            assert_eq!(*testler, "-", "{belge}: taslak kanıtı '-' olmalı");
            continue;
        }
        assert_ne!(*testler, "-", "{belge}: {durum} belge test kanıtı ister");
        for test in testler.split(',') {
            let test_yolu = depo.join(test);
            assert!(test_yolu.is_file(), "{belge}: test kanıtı yok: {test}");
            assert_eq!(
                test_yolu.extension().and_then(|uzanti| uzanti.to_str()),
                Some("rs"),
                "{belge}: kanıt Rust test dosyası olmalı: {test}"
            );
            let kaynak = std::fs::read_to_string(&test_yolu).expect("test kanıtı okunmalı");
            assert!(
                kaynak.contains("#[test]") || kaynak.contains("```compile_fail"),
                "{belge}: {test} yürütülebilir test taşımıyor"
            );
        }
    }

    assert_eq!(
        gorulen, beklenen,
        "kanıt haritası bütün RFC/ADR/spec belgelerini birebir kapsamalı"
    );
    assert_eq!(
        gorulen.len(),
        99,
        "şema-1 belge tabanı beklenmedik biçimde değişti"
    );
}

#[test]
fn readme_canli_depo_sayilari_ureticisiyle_gunceldir() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    let cikti = Command::new(env!("CARGO_BIN_EXE_depo_sayilari"))
        .arg("--denetle")
        .current_dir(depo)
        .output()
        .expect("depo sayıları aracı çalışmalı");
    assert!(
        cikti.status.success(),
        "{}{}",
        String::from_utf8_lossy(&cikti.stdout),
        String::from_utf8_lossy(&cikti.stderr)
    );
}

#[test]
fn kaynak_arsivi_build_ve_kisisel_artifaktlari_disarida_tutar() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    let gitignore =
        std::fs::read_to_string(depo.join(".gitignore")).expect("kök .gitignore okunmalı");
    for kalip in [
        "target/",
        "compiler/fuzz/artifacts/",
        "__MACOSX/",
        "*.profraw",
        "*.profdata",
        "*.zip",
    ] {
        assert!(
            gitignore.lines().any(|satir| satir == kalip),
            "kaynak arşivi hijyeni için .gitignore kalıbı eksik: {kalip}"
        );
    }

    let script = std::fs::read_to_string(depo.join("scripts/temiz-kaynak-arsivi.sh"))
        .expect("temiz arşiv scripti okunmalı");
    assert!(script.contains("git archive --format=zip"));
    assert!(script.contains("unzip -Z1"));
    assert!(script.contains("__MACOSX|target|artifacts"));
    assert!(script.contains("KAYNAK-SHA256.txt"));
    assert!(script.contains("git ls-tree -r --name-only HEAD"));
}
