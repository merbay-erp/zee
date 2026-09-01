//! Yapısal belge tazelik kapısı: yeni karar/spec dosyası indekslenmeden ve
//! yerel Markdown bağlantısı kırıkken toplu iş tamamlanamaz.

use std::path::{Path, PathBuf};

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
