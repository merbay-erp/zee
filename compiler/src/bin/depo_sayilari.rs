//! README'deki canlı depo sayılarını tek kaynaktan üretir ve denetler.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const BASLANGIC: &str = "<!-- ZEE-DEPO-SAYILARI:BEGIN -->";
const BITIS: &str = "<!-- ZEE-DEPO-SAYILARI:END -->";

#[derive(Debug)]
struct DepoSayilari {
    golden: usize,
    test: usize,
    tani_aktif: usize,
    tani_ayrilmis: usize,
    rfc: usize,
    rfc_kabul: usize,
    rfc_gecici: usize,
    rfc_taslak: usize,
    adr: usize,
    adr_kabul: usize,
    spec: usize,
}

impl DepoSayilari {
    fn oku(depo: &Path) -> Result<Self, String> {
        let (rfc, rfc_kabul, rfc_gecici, rfc_taslak) = rfc_sayilari(depo)?;
        let (adr, adr_kabul) = adr_sayilari(depo)?;
        let (tani_aktif, tani_ayrilmis) = tani_sayilari(depo)?;
        Ok(Self {
            golden: numarali_dosya_sayisi(&depo.join("golden"), 2, "dil", "00")?,
            test: test_sayisi(depo)?,
            tani_aktif,
            tani_ayrilmis,
            rfc,
            rfc_kabul,
            rfc_gecici,
            rfc_taslak,
            adr,
            adr_kabul,
            spec: numarali_dosya_sayisi(&depo.join("spec"), 2, "md", "00")?,
        })
    }

    fn blok(&self) -> String {
        format!(
            "{BASLANGIC}\n\
<!-- `cd compiler && cargo run --bin depo_sayilari -- --yaz` üretir. Elle değiştirme. -->\n\
### Canlı depo sayıları\n\n\
| Ölçüm | Tek kaynaklı değer |\n\
|---|---:|\n\
| Golden program | **{0}/{0}** |\n\
| Rust + doctest vakası | **{1}** |\n\
| Tanı kimliği | **{2} etkin + {3} ayrılmış** |\n\
| RFC | **{4}** ({5} kabul, {6} geçici kabul, {7} taslak) |\n\
| ADR | **{8}** ({9} kabul) |\n\
| Normatif spec bölümü | **{10}** |\n\
{BITIS}",
            self.golden,
            self.test,
            self.tani_aktif,
            self.tani_ayrilmis,
            self.rfc,
            self.rfc_kabul,
            self.rfc_gecici,
            self.rfc_taslak,
            self.adr,
            self.adr_kabul,
            self.spec
        )
    }
}

fn numarali_dosya_sayisi(
    klasor: &Path,
    basamak: usize,
    uzanti: &str,
    sablon_no: &str,
) -> Result<usize, String> {
    let mut sayi = 0usize;
    for girdi in
        fs::read_dir(klasor).map_err(|hata| format!("{} okunamadı: {hata}", klasor.display()))?
    {
        let girdi = girdi.map_err(|hata| hata.to_string())?;
        if !girdi
            .file_type()
            .map_err(|hata| hata.to_string())?
            .is_file()
        {
            continue;
        }
        let ad = girdi.file_name();
        let Some(ad) = ad.to_str() else { continue };
        let Some((govde, dosya_uzantisi)) = ad.rsplit_once('.') else {
            continue;
        };
        let on_ek = govde.as_bytes().get(..basamak);
        if dosya_uzantisi == uzanti
            && on_ek.is_some_and(|on_ek| on_ek.iter().all(u8::is_ascii_digit))
            && govde.as_bytes().get(basamak) == Some(&b'-')
            && &govde[..basamak] != sablon_no
        {
            sayi += 1;
        }
    }
    Ok(sayi)
}

fn rust_dosyalarini_topla(klasor: &Path, sonuc: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut girdiler = fs::read_dir(klasor)
        .map_err(|hata| format!("{} okunamadı: {hata}", klasor.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|hata| hata.to_string())?;
    girdiler.sort_by_key(|girdi| girdi.file_name());
    for girdi in girdiler {
        let yol = girdi.path();
        let tur = girdi.file_type().map_err(|hata| hata.to_string())?;
        if tur.is_dir() {
            rust_dosyalarini_topla(&yol, sonuc)?;
        } else if tur.is_file() && yol.extension().and_then(|e| e.to_str()) == Some("rs") {
            sonuc.push(yol);
        }
    }
    Ok(())
}

fn test_sayisi(depo: &Path) -> Result<usize, String> {
    let mut dosyalar = Vec::new();
    for klasor in ["compiler/src", "compiler/tests"] {
        rust_dosyalarini_topla(&depo.join(klasor), &mut dosyalar)?;
    }
    let mut sayi = 0usize;
    for dosya in dosyalar {
        let kaynak = fs::read_to_string(&dosya)
            .map_err(|hata| format!("{} okunamadı: {hata}", dosya.display()))?;
        sayi += kaynak
            .lines()
            .filter(|satir| satir.trim() == "#[test]")
            .count();
        sayi += kaynak
            .lines()
            .filter(|satir| {
                let satir = satir.trim_start();
                (satir.starts_with("//!") || satir.starts_with("///"))
                    && satir.contains("```compile_fail")
            })
            .count();
    }
    Ok(sayi)
}

fn tani_sayilari(depo: &Path) -> Result<(usize, usize), String> {
    let yol = depo.join("compiler/tests/fixtures/tani-kimlikleri-v1.tsv");
    let metin =
        fs::read_to_string(&yol).map_err(|hata| format!("{} okunamadı: {hata}", yol.display()))?;
    let mut aktif = 0usize;
    let mut ayrilmis = 0usize;
    for satir in metin.lines().filter(|satir| !satir.starts_with('#')) {
        match satir.split('\t').next().unwrap_or_default() {
            "aktif" => aktif += 1,
            "ayrilmis" => ayrilmis += 1,
            "" => {}
            durum => return Err(format!("Tanı fixture'ında bilinmeyen durum: {durum}")),
        }
    }
    Ok((aktif, ayrilmis))
}

fn durum_satiri(yol: &Path) -> Result<String, String> {
    let metin =
        fs::read_to_string(yol).map_err(|hata| format!("{} okunamadı: {hata}", yol.display()))?;
    metin
        .lines()
        .find(|satir| satir.starts_with("- **Durum:**"))
        .map(str::to_lowercase)
        .ok_or_else(|| format!("{} durum satırı taşımıyor", yol.display()))
}

fn rfc_sayilari(depo: &Path) -> Result<(usize, usize, usize, usize), String> {
    let klasor = depo.join("rfcs");
    let toplam = numarali_dosya_sayisi(&klasor, 4, "md", "0000")?;
    let mut kabul = 0usize;
    let mut gecici = 0usize;
    let mut taslak = 0usize;
    for girdi in fs::read_dir(&klasor).map_err(|hata| hata.to_string())? {
        let yol = girdi.map_err(|hata| hata.to_string())?.path();
        let Some(ad) = yol.file_name().and_then(|ad| ad.to_str()) else {
            continue;
        };
        if ad.len() < 6
            || !ad.as_bytes()[..4].iter().all(u8::is_ascii_digit)
            || ad.starts_with("0000-")
        {
            continue;
        }
        let durum = durum_satiri(&yol)?;
        if durum.contains("geçici kabul") {
            gecici += 1;
        } else if durum.contains("taslak") {
            taslak += 1;
        } else if durum.contains("kabul") {
            kabul += 1;
        } else {
            return Err(format!("{} RFC durumu sınıflandırılamadı", yol.display()));
        }
    }
    if kabul + gecici + taslak != toplam {
        return Err("RFC durum toplamı dosya sayısıyla uyuşmuyor".into());
    }
    Ok((toplam, kabul, gecici, taslak))
}

fn adr_sayilari(depo: &Path) -> Result<(usize, usize), String> {
    let klasor = depo.join("adr");
    let toplam = numarali_dosya_sayisi(&klasor, 3, "md", "000")?;
    let mut kabul = 0usize;
    for girdi in fs::read_dir(&klasor).map_err(|hata| hata.to_string())? {
        let yol = girdi.map_err(|hata| hata.to_string())?.path();
        let Some(ad) = yol.file_name().and_then(|ad| ad.to_str()) else {
            continue;
        };
        if ad.len() < 5
            || !ad.as_bytes()[..3].iter().all(u8::is_ascii_digit)
            || ad.starts_with("000-")
        {
            continue;
        }
        if durum_satiri(&yol)?.contains("kabul") {
            kabul += 1;
        }
    }
    Ok((toplam, kabul))
}

fn depo_kokunu_bul(mut yol: PathBuf) -> Result<PathBuf, String> {
    loop {
        if yol.join("README.md").is_file() && yol.join("compiler/Cargo.toml").is_file() {
            return Ok(yol);
        }
        if !yol.pop() {
            return Err("README.md ve compiler/Cargo.toml taşıyan depo kökü bulunamadı".into());
        }
    }
}

fn readme_blok_araligi(metin: &str) -> Result<(usize, usize), String> {
    if metin.matches(BASLANGIC).count() != 1 || metin.matches(BITIS).count() != 1 {
        return Err("README canlı sayı işaretleri tam birer kez bulunmalı".into());
    }
    let bas = metin
        .find(BASLANGIC)
        .ok_or_else(|| "README canlı sayı başlangıç işareti yok".to_string())?;
    let bitis_bas = metin[bas..]
        .find(BITIS)
        .map(|indis| bas + indis)
        .ok_or_else(|| "README canlı sayı bitiş işareti yok".to_string())?;
    Ok((bas, bitis_bas + BITIS.len()))
}

fn calistir() -> Result<(), String> {
    let mut kip = "yazdır";
    let mut depo_argumani = None;
    let mut argumanlar = std::env::args().skip(1);
    while let Some(arguman) = argumanlar.next() {
        match arguman.as_str() {
            "--denetle" => kip = "denetle",
            "--yaz" => kip = "yaz",
            "--depo" => {
                depo_argumani = Some(PathBuf::from(
                    argumanlar
                        .next()
                        .ok_or_else(|| "--depo bir yol ister".to_string())?,
                ));
            }
            _ => return Err(format!("Bilinmeyen argüman: {arguman}")),
        }
    }
    let baslangic =
        depo_argumani.unwrap_or(std::env::current_dir().map_err(|hata| hata.to_string())?);
    let depo = depo_kokunu_bul(baslangic)?;
    let sayilar = DepoSayilari::oku(&depo)?;
    let beklenen = sayilar.blok();
    if kip == "yazdır" {
        println!("{beklenen}");
        return Ok(());
    }

    let readme_yolu = depo.join("README.md");
    let readme = fs::read_to_string(&readme_yolu)
        .map_err(|hata| format!("{} okunamadı: {hata}", readme_yolu.display()))?;
    let (bas, son) = readme_blok_araligi(&readme)?;
    if kip == "denetle" {
        if readme[bas..son] != beklenen {
            return Err(
                "README canlı depo sayıları bayat; `cargo run --bin depo_sayilari -- --yaz` çalıştır"
                    .into(),
            );
        }
        println!("README canlı depo sayıları güncel: {} test.", sayilar.test);
        return Ok(());
    }

    let mut yeni = String::with_capacity(readme.len() + beklenen.len());
    yeni.push_str(&readme[..bas]);
    yeni.push_str(&beklenen);
    yeni.push_str(&readme[son..]);
    dil::kalici_dosya::atomik_yaz(&readme_yolu, yeni.as_bytes())
        .map_err(|hata| format!("README atomik yazılamadı: {hata}"))?;
    println!(
        "README canlı depo sayıları güncellendi: {} test.",
        sayilar.test
    );
    Ok(())
}

fn main() -> ExitCode {
    match calistir() {
        Ok(()) => ExitCode::SUCCESS,
        Err(hata) => {
            eprintln!("HATA: {hata}");
            ExitCode::FAILURE
        }
    }
}
