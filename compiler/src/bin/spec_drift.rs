//! K-171/ADR-065 madde düzeyi drift kapısı: her normatif spec maddesi
//! (ZORUNLU/YASAK/TANIMLI/AÇIK işaretli paragraf, liste öğesi ya da başlık)
//! exact test işlevi kanıtı taşır; metni değişen, eklenen veya kaybolan madde
//! ile kaybolan test kanıtı fail-closed görünür.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HARITA_YOLU: &str = "docs/spec-madde-kaniti-v1.tsv";
const RAPOR_YOLU: &str = "docs/spec-drift-raporu.md";
const HARITA_SEMASI: &str = "# zee-spec-madde-kaniti-1";
const OZET_UZUNLUGU: usize = 72;
const ISARETCI_KOKLERI: [&str; 5] = ["ZORUNLU", "ZORUNDA", "YASAK", "TANIMLI", "AÇIK"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Madde {
    kimlik: String,
    dosya: String,
    metin: String,
    acik: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Kanit {
    madde: String,
    ozet: String,
    durum: String,
    seciciler: Vec<String>,
    not: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Drift {
    eksik: Vec<Madde>,
    bayat: Vec<String>,
    ihlaller: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Kip {
    Denetle,
    RaporYaz,
    Taslak,
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

fn argumanlari_oku() -> Result<(Kip, PathBuf), String> {
    let mut kip = Kip::Denetle;
    let mut depo = None;
    let mut argumanlar = std::env::args().skip(1);
    while let Some(arguman) = argumanlar.next() {
        match arguman.as_str() {
            "--denetle" => kip = Kip::Denetle,
            "--rapor-yaz" => kip = Kip::RaporYaz,
            "--taslak" => kip = Kip::Taslak,
            "--depo" => {
                depo = Some(PathBuf::from(
                    argumanlar
                        .next()
                        .ok_or_else(|| "--depo bir yol ister".to_string())?,
                ));
            }
            _ => {
                return Err(
                    "kullanım: spec_drift [--denetle|--rapor-yaz|--taslak] [--depo YOL]".into(),
                )
            }
        }
    }
    let depo = match depo {
        Some(depo) => depo,
        None => depo_kokunu_bul(std::env::current_dir().map_err(|hata| hata.to_string())?)?,
    };
    Ok((kip, depo))
}

/// FNV-1a 64; kriptografik değil, yalnız kararlı madde kimliği içindir.
fn parmak_izi(metin: &str) -> String {
    let mut ozet: u64 = 0xcbf2_9ce4_8422_2325;
    for bayt in metin.bytes() {
        ozet ^= u64::from(bayt);
        ozet = ozet.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{ozet:016x}")
}

/// (normatif, açık) çifti. Büyük harfli işaretçi kökleri sayılır; `AÇIK değil`
/// gibi olumsuzlanan AÇIK, davranışın karara bağlandığını söylediği için
/// normatif sayılır.
fn isaretcileri_bul(metin: &str) -> (bool, bool) {
    let mut normatif = false;
    let mut acik = false;
    let kelimeler = metin
        .split(|karakter: char| !karakter.is_alphabetic())
        .filter(|kelime| !kelime.is_empty())
        .collect::<Vec<_>>();
    for (sira, kelime) in kelimeler.iter().enumerate() {
        if !kelime.chars().all(char::is_uppercase) {
            continue;
        }
        for kok in ISARETCI_KOKLERI {
            if !kelime.starts_with(kok) {
                continue;
            }
            let olumsuz = kok == "AÇIK"
                && kelimeler
                    .get(sira + 1)
                    .is_some_and(|sonraki| sonraki.to_lowercase().starts_with("değil"));
            if kok == "AÇIK" && !olumsuz {
                acik = true;
            } else {
                normatif = true;
            }
        }
    }
    (normatif, acik)
}

fn liste_basi_mi(satir: &str) -> bool {
    let kirpik = satir.trim_start();
    if kirpik.starts_with("- ") || kirpik.starts_with("* ") {
        return true;
    }
    let rakamlar = kirpik.chars().take_while(char::is_ascii_digit).count();
    rakamlar > 0 && kirpik[rakamlar..].starts_with(". ")
}

fn metni_sadelestir(parcalar: &[String]) -> String {
    parcalar
        .iter()
        .flat_map(|parca| parca.split_whitespace())
        .collect::<Vec<_>>()
        .join(" ")
}

fn madde_kur(dosya: &str, metin: String) -> Option<Madde> {
    let (normatif, acik) = isaretcileri_bul(&metin);
    if !normatif && !acik {
        return None;
    }
    let kimlik = format!("{dosya}#{}", parmak_izi(&format!("{dosya}\n{metin}")));
    Some(Madde {
        kimlik,
        dosya: dosya.to_string(),
        metin,
        acik: acik && !normatif,
    })
}

fn maddeleri_cikar(dosya: &str, icerik: &str) -> Vec<Madde> {
    let mut maddeler = Vec::new();
    let mut blok: Vec<String> = Vec::new();
    let mut kod_icinde = false;
    let bitir = |blok: &mut Vec<String>, maddeler: &mut Vec<Madde>| {
        if blok.is_empty() {
            return;
        }
        let metin = metni_sadelestir(blok);
        blok.clear();
        if let Some(madde) = madde_kur(dosya, metin) {
            maddeler.push(madde);
        }
    };
    for satir in icerik.lines() {
        if satir.trim_start().starts_with("```") {
            bitir(&mut blok, &mut maddeler);
            kod_icinde = !kod_icinde;
            continue;
        }
        if kod_icinde {
            continue;
        }
        let govde = satir.trim_end();
        if govde.trim().is_empty() {
            bitir(&mut blok, &mut maddeler);
            continue;
        }
        if govde.starts_with('#') {
            bitir(&mut blok, &mut maddeler);
            let baslik = govde.trim_start_matches('#').trim();
            if let Some(madde) = madde_kur(dosya, metni_sadelestir(&[baslik.to_string()])) {
                maddeler.push(madde);
            }
            continue;
        }
        if govde.trim_start().starts_with('|') {
            bitir(&mut blok, &mut maddeler);
            continue;
        }
        if liste_basi_mi(govde) {
            bitir(&mut blok, &mut maddeler);
        }
        blok.push(govde.trim().to_string());
    }
    bitir(&mut blok, &mut maddeler);
    maddeler
}

fn spec_dosyalari(depo: &Path) -> Result<Vec<(String, String)>, String> {
    let klasor = depo.join("spec");
    let mut adlar = Vec::new();
    for girdi in fs::read_dir(&klasor).map_err(|hata| format!("spec/ okunamadı: {hata}"))? {
        let girdi = girdi.map_err(|hata| format!("spec/ girdisi okunamadı: {hata}"))?;
        let ad = girdi.file_name().to_string_lossy().into_owned();
        let numarali = ad.len() > 3
            && ad.as_bytes()[..2].iter().all(u8::is_ascii_digit)
            && ad.as_bytes()[2] == b'-'
            && ad.ends_with(".md");
        if numarali {
            adlar.push(ad);
        }
    }
    adlar.sort();
    let mut sonuc = Vec::new();
    for ad in adlar {
        let icerik = fs::read_to_string(klasor.join(&ad))
            .map_err(|hata| format!("spec/{ad} okunamadı: {hata}"))?;
        sonuc.push((format!("spec/{ad}"), icerik));
    }
    Ok(sonuc)
}

fn butun_maddeler(depo: &Path) -> Result<Vec<Madde>, String> {
    let mut maddeler = Vec::new();
    let mut gorulen = BTreeSet::new();
    for (dosya, icerik) in spec_dosyalari(depo)? {
        for madde in maddeleri_cikar(&dosya, &icerik) {
            if !gorulen.insert(madde.kimlik.clone()) {
                return Err(format!(
                    "{} içinde aynı metinli iki normatif madde var: {}",
                    dosya,
                    ozet(&madde.metin)
                ));
            }
            maddeler.push(madde);
        }
    }
    Ok(maddeler)
}

fn ozet(metin: &str) -> String {
    let harfler = metin.chars().collect::<Vec<_>>();
    if harfler.len() <= OZET_UZUNLUGU {
        return metin.to_string();
    }
    let mut kisa = harfler[..OZET_UZUNLUGU].iter().collect::<String>();
    kisa.push('…');
    kisa
}

fn haritayi_coz(metin: &str) -> Result<Vec<Kanit>, String> {
    let mut satirlar = metin.lines();
    if satirlar.next() != Some(HARITA_SEMASI) {
        return Err(format!(
            "madde kanıt haritası {HARITA_SEMASI:?} başlığıyla başlamalı"
        ));
    }
    let mut kayitlar = Vec::new();
    let mut gorulen = BTreeSet::new();
    for (sira, satir) in satirlar.enumerate() {
        if satir.is_empty() || satir.starts_with('#') {
            continue;
        }
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [madde, ozet, durum, kanit, not] = alanlar.as_slice() else {
            return Err(format!("{}. satır beş sekmeli alan taşımalı", sira + 2));
        };
        if !gorulen.insert((*madde).to_string()) {
            return Err(format!("{madde}: yinelenen madde kaydı"));
        }
        if !matches!(*durum, "kanitli" | "kismi" | "acik") {
            return Err(format!("{madde}: durum kanitli|kismi|acik olmalı"));
        }
        let seciciler = if *kanit == "-" {
            Vec::new()
        } else {
            kanit.split(';').map(str::to_string).collect()
        };
        if *durum == "acik" && !seciciler.is_empty() {
            return Err(format!(
                "{madde}: açık madde test kanıtı taşımaz; karara bağlanınca durum değişir"
            ));
        }
        if *durum != "acik" && seciciler.is_empty() {
            return Err(format!(
                "{madde}: {durum} madde en az bir test seçicisi ister"
            ));
        }
        if *durum != "kanitli" && not.trim().len() < 20 {
            return Err(format!(
                "{madde}: {durum} madde en az 20 karakterlik not ister"
            ));
        }
        for secici in &seciciler {
            let Some((yol, islev)) = secici.split_once("::") else {
                return Err(format!(
                    "{madde}: seçici <dosya>::<işlev> biçiminde olmalı: {secici}"
                ));
            };
            if !yol.ends_with(".rs")
                || islev.is_empty()
                || !islev.chars().all(|k| k.is_ascii_alphanumeric() || k == '_')
            {
                return Err(format!("{madde}: geçersiz seçici {secici}"));
            }
        }
        kayitlar.push(Kanit {
            madde: (*madde).to_string(),
            ozet: (*ozet).to_string(),
            durum: (*durum).to_string(),
            seciciler,
            not: (*not).to_string(),
        });
    }
    Ok(kayitlar)
}

fn seciciyi_dogrula(
    depo: &Path,
    secici: &str,
    onbellek: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let (yol, islev) = secici.split_once("::").expect("seçici biçimi doğrulandı");
    if !onbellek.contains_key(yol) {
        let icerik = fs::read_to_string(depo.join(yol))
            .map_err(|hata| format!("test kanıtı okunamadı: {yol}: {hata}"))?;
        if !icerik.contains("#[test]") {
            return Err(format!("{yol} yürütülebilir test taşımıyor"));
        }
        onbellek.insert(yol.to_string(), icerik);
    }
    let icerik = &onbellek[yol];
    let bulundu =
        icerik.contains(&format!("fn {islev}(")) || icerik.contains(&format!("fn {islev}<"));
    if !bulundu {
        return Err(format!("{yol} içinde {islev} işlevi yok"));
    }
    Ok(())
}

fn drift_hesapla(depo: &Path, maddeler: &[Madde], harita: &[Kanit]) -> Drift {
    let mut drift = Drift::default();
    let madde_haritasi: BTreeMap<&str, &Madde> =
        maddeler.iter().map(|m| (m.kimlik.as_str(), m)).collect();
    let kayitli: BTreeSet<&str> = harita.iter().map(|k| k.madde.as_str()).collect();
    for madde in maddeler {
        if !kayitli.contains(madde.kimlik.as_str()) {
            drift.eksik.push(madde.clone());
        }
    }
    let mut onbellek = BTreeMap::new();
    for kanit in harita {
        let Some(madde) = madde_haritasi.get(kanit.madde.as_str()) else {
            drift.bayat.push(kanit.madde.clone());
            continue;
        };
        if kanit.ozet != ozet(&madde.metin) {
            drift.ihlaller.push(format!(
                "{}: özet bayat; beklenen {:?}",
                kanit.madde,
                ozet(&madde.metin)
            ));
        }
        if madde.acik && kanit.durum != "acik" {
            drift.ihlaller.push(format!(
                "{}: AÇIK madde yalnız acik durumu taşıyabilir",
                kanit.madde
            ));
        }
        if !madde.acik && kanit.durum == "acik" {
            drift.ihlaller.push(format!(
                "{}: normatif madde acik durumu taşıyamaz",
                kanit.madde
            ));
        }
        for secici in &kanit.seciciler {
            if let Err(hata) = seciciyi_dogrula(depo, secici, &mut onbellek) {
                drift.ihlaller.push(format!("{}: {hata}", kanit.madde));
            }
        }
    }
    drift
}

fn rapor_metni(maddeler: &[Madde], harita: &[Kanit], drift: &Drift) -> String {
    let kayit: BTreeMap<&str, &Kanit> = harita.iter().map(|k| (k.madde.as_str(), k)).collect();
    let mut bolumler: BTreeMap<&str, [usize; 4]> = BTreeMap::new();
    for madde in maddeler {
        let sayac = bolumler.entry(madde.dosya.as_str()).or_default();
        sayac[0] += 1;
        match kayit.get(madde.kimlik.as_str()).map(|k| k.durum.as_str()) {
            Some("kanitli") => sayac[1] += 1,
            Some("kismi") => sayac[2] += 1,
            Some("acik") => sayac[3] += 1,
            _ => {}
        }
    }
    let mut rapor = String::new();
    rapor.push_str("# Spec maddesi ↔ test kanıtı drift raporu\n\n");
    rapor.push_str("<!-- `cd compiler && cargo run --locked --bin spec_drift -- --rapor-yaz` üretir. Elle değiştirme. -->\n\n");
    rapor.push_str(
        "Bu rapor K-171/ADR-065 kapısının güncel görünümüdür. Madde; `spec/` altında \
         ZORUNLU/ZORUNDA/YASAK/TANIMLI/AÇIK işaretçisi taşıyan paragraf, liste öğesi ya da \
         başlıktır. Kimlik dosya adı + sadeleştirilmiş metnin parmak izidir; metin \
         değişince kimlik değişir ve kanıt yeniden incelenir.\n\n",
    );
    rapor.push_str("| Bölüm | Madde | Kanıtlı | Kısmi | Açık |\n|---|---:|---:|---:|---:|\n");
    let mut toplam = [0usize; 4];
    for (dosya, sayac) in &bolumler {
        rapor.push_str(&format!(
            "| `{dosya}` | {} | {} | {} | {} |\n",
            sayac[0], sayac[1], sayac[2], sayac[3]
        ));
        for (i, deger) in sayac.iter().enumerate() {
            toplam[i] += deger;
        }
    }
    rapor.push_str(&format!(
        "| **Toplam** | **{}** | **{}** | **{}** | **{}** |\n",
        toplam[0], toplam[1], toplam[2], toplam[3]
    ));
    rapor.push_str("\n## Kısmi kanıtlı maddeler\n\n");
    let kismiler = maddeler
        .iter()
        .filter_map(|m| {
            kayit
                .get(m.kimlik.as_str())
                .filter(|k| k.durum == "kismi")
                .map(|k| (m, *k))
        })
        .collect::<Vec<_>>();
    if kismiler.is_empty() {
        rapor.push_str("Yok.\n");
    }
    for (madde, kanit) in kismiler {
        rapor.push_str(&format!(
            "- `{}` — {} — {}\n",
            madde.kimlik, kanit.ozet, kanit.not
        ));
    }
    rapor.push_str("\n## Açık maddeler\n\n");
    let acikler = maddeler
        .iter()
        .filter_map(|m| {
            kayit
                .get(m.kimlik.as_str())
                .filter(|k| k.durum == "acik")
                .map(|k| (m, *k))
        })
        .collect::<Vec<_>>();
    if acikler.is_empty() {
        rapor.push_str("Yok.\n");
    }
    for (madde, kanit) in acikler {
        rapor.push_str(&format!(
            "- `{}` — {} — {}\n",
            madde.kimlik, kanit.ozet, kanit.not
        ));
    }
    rapor.push_str("\n## Sürüklenme\n\n");
    if drift.eksik.is_empty() && drift.bayat.is_empty() && drift.ihlaller.is_empty() {
        rapor.push_str("Yok: her normatif madde kayıtlı, her kayıt var olan bir maddeye ve var olan test işlevine bağlı.\n");
    }
    for madde in &drift.eksik {
        rapor.push_str(&format!(
            "- KAYITSIZ `{}` — {}\n",
            madde.kimlik,
            ozet(&madde.metin)
        ));
    }
    for kimlik in &drift.bayat {
        rapor.push_str(&format!(
            "- BAYAT `{kimlik}` — madde artık spec'te yok; kayıt incelenmeli\n"
        ));
    }
    for ihlal in &drift.ihlaller {
        rapor.push_str(&format!("- İHLAL {ihlal}\n"));
    }
    rapor.push_str(
        "\nSürüklenme otomatik bir semantik hüküm değildir: kayıtsız madde kanıt ister, bayat \
         kayıt ya silinir ya yeni kimliğe taşınır; seçicinin maddeyi gerçekten kanıtladığı \
         kod incelemesinde değerlendirilir.\n",
    );
    rapor
}

fn taslak_metni(maddeler: &[Madde], harita: &[Kanit]) -> String {
    let kayitli: BTreeSet<&str> = harita.iter().map(|k| k.madde.as_str()).collect();
    let mut cikti = String::new();
    for madde in maddeler
        .iter()
        .filter(|m| !kayitli.contains(m.kimlik.as_str()))
    {
        let (durum, kanit) = if madde.acik {
            ("acik", "-")
        } else {
            ("kanitli", "")
        };
        cikti.push_str(&format!(
            "{}\t{}\t{durum}\t{kanit}\t\n",
            madde.kimlik,
            ozet(&madde.metin)
        ));
    }
    cikti
}

fn atomik_yaz(yol: &Path, icerik: &str) -> Result<(), String> {
    dil::kalici_dosya::atomik_yaz(yol, icerik.as_bytes())
        .map_err(|hata| format!("{} yazılamadı: {hata}", yol.display()))
}

fn calistir() -> Result<(), String> {
    let (kip, depo) = argumanlari_oku()?;
    let maddeler = butun_maddeler(&depo)?;
    let harita_yolu = depo.join(HARITA_YOLU);
    let harita = if harita_yolu.is_file() {
        haritayi_coz(&fs::read_to_string(&harita_yolu).map_err(|hata| hata.to_string())?)?
    } else if kip == Kip::Taslak {
        Vec::new()
    } else {
        return Err(format!("madde kanıt haritası yok: {HARITA_YOLU}"));
    };
    if kip == Kip::Taslak {
        print!("{}", taslak_metni(&maddeler, &harita));
        return Ok(());
    }
    let drift = drift_hesapla(&depo, &maddeler, &harita);
    let rapor = rapor_metni(&maddeler, &harita, &drift);
    let rapor_yolu = depo.join(RAPOR_YOLU);
    if kip == Kip::RaporYaz {
        atomik_yaz(&rapor_yolu, &rapor)?;
        println!("Spec drift raporu güncellendi: {} madde.", maddeler.len());
    }
    let mut hatalar = Vec::new();
    hatalar.extend(
        drift
            .eksik
            .iter()
            .map(|m| format!("kayıtsız madde {}: {}", m.kimlik, ozet(&m.metin))),
    );
    hatalar.extend(
        drift
            .bayat
            .iter()
            .map(|k| format!("bayat kayıt {k}: madde spec'te yok")),
    );
    hatalar.extend(drift.ihlaller.iter().cloned());
    if kip == Kip::Denetle
        && fs::read_to_string(&rapor_yolu).ok().as_deref() != Some(rapor.as_str())
    {
        hatalar.push("spec drift raporu bayat; `cargo run --locked --bin spec_drift -- --rapor-yaz` çalıştır".into());
    }
    if !hatalar.is_empty() {
        return Err(format!(
            "spec drift incelemesi gerekli:\n- {}",
            hatalar.join("\n- ")
        ));
    }
    if kip == Kip::Denetle {
        println!(
            "Spec drift yok: {} madde, {} kayıt, rapor güncel.",
            maddeler.len(),
            harita.len()
        );
    }
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

#[cfg(test)]
mod testler {
    use super::*;

    const ORNEK: &str = "# 99 — Örnek\n\nGiriş paragrafı işaretsizdir.\n\n## Kural (TANIMLI)\n\n- Kodlama **ZORUNLU** UTF-8'dir.\n- Sekme **YASAK** (S003);\n  devam satırı aynı öğedir.\n\n```text\nZORUNLU kod içinde sayılmaz\n```\n\n| tablo | YASAK |\n|---|---|\n\nEdition alanı AÇIKTIR.\n";

    #[test]
    fn maddeler_baslik_paragraf_ve_liste_ogesinden_cikar_kod_ve_tabloyu_atlar() {
        let maddeler = maddeleri_cikar("spec/99-ornek.md", ORNEK);
        let metinler = maddeler
            .iter()
            .map(|m| m.metin.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            metinler,
            [
                "Kural (TANIMLI)",
                "- Kodlama **ZORUNLU** UTF-8'dir.",
                "- Sekme **YASAK** (S003); devam satırı aynı öğedir.",
                "Edition alanı AÇIKTIR."
            ]
        );
        assert!(maddeler[3].acik && !maddeler[0].acik);
        assert_eq!(
            isaretcileri_bul("bugün **AÇIK** değildir; **DESTEKLENMEZ**"),
            (true, false)
        );
        assert_eq!(
            isaretcileri_bul("izolasyon bu sürümde AÇIKTIR"),
            (false, true)
        );
        assert_eq!(isaretcileri_bul("herkese açık rota"), (false, false));
        assert!(maddeler
            .iter()
            .all(|m| m.kimlik.starts_with("spec/99-ornek.md#")));
        assert_eq!(
            maddeleri_cikar("spec/99-ornek.md", ORNEK),
            maddeler,
            "kimlik deterministik"
        );
        assert_ne!(
            maddeleri_cikar("spec/99-ornek.md", &ORNEK.replace("UTF-8", "UTF-16"))[1].kimlik,
            maddeler[1].kimlik,
            "metin değişince kimlik değişir"
        );
    }

    #[test]
    fn harita_semasi_durum_ve_secici_kurallarini_fail_closed_uygular() {
        let iyi = format!("{HARITA_SEMASI}\nspec/x#1\tözet\tkanitli\tcompiler/tests/a.rs::b\t\n");
        assert_eq!(haritayi_coz(&iyi).expect("geçerli").len(), 1);
        for (bozuk, mesaj) in [
            ("spec/x#1\tözet\tbelirsiz\tcompiler/tests/a.rs::b\t", "kanitli|kismi|acik"),
            ("spec/x#1\tözet\tacik\tcompiler/tests/a.rs::b\t", "test kanıtı taşımaz"),
            ("spec/x#1\tözet\tkanitli\t-\t", "en az bir test seçicisi"),
            ("spec/x#1\tözet\tkismi\tcompiler/tests/a.rs::b\tkısa", "20 karakterlik not"),
            ("spec/x#1\tözet\tkanitli\tcompiler/tests/a.rs\t", "<dosya>::<işlev>"),
            ("spec/x#1\tözet\tkanitli\tcompiler/tests/a.rs::b\t\nspec/x#1\tözet\tkanitli\tcompiler/tests/a.rs::b\t", "yinelenen"),
        ] {
            let hata = haritayi_coz(&format!("{HARITA_SEMASI}\n{bozuk}\n")).unwrap_err();
            assert!(hata.contains(mesaj), "{bozuk:?} → {hata}");
        }
        assert!(haritayi_coz("# baska\n")
            .unwrap_err()
            .contains("başlığıyla başlamalı"));
    }

    #[test]
    fn drift_kayitsiz_bayat_ozet_acik_ve_kayip_isleve_karsi_fail_closed() {
        let gecici = std::env::temp_dir().join(format!("zee-spec-drift-{}", std::process::id()));
        fs::create_dir_all(gecici.join("compiler/tests")).expect("geçici klasör");
        fs::write(
            gecici.join("compiler/tests/a.rs"),
            "#[test]\nfn var_olan() {}\n",
        )
        .expect("test dosyası");
        let maddeler = maddeleri_cikar("spec/99-ornek.md", ORNEK);
        let kimlik = |i: usize| maddeler[i].kimlik.clone();
        let harita = vec![
            Kanit {
                madde: kimlik(0),
                ozet: ozet(&maddeler[0].metin),
                durum: "kanitli".into(),
                seciciler: vec!["compiler/tests/a.rs::var_olan".into()],
                not: String::new(),
            },
            Kanit {
                madde: kimlik(1),
                ozet: "bayat özet".into(),
                durum: "kanitli".into(),
                seciciler: vec!["compiler/tests/a.rs::yok".into()],
                not: String::new(),
            },
            Kanit {
                madde: kimlik(3),
                ozet: ozet(&maddeler[3].metin),
                durum: "kanitli".into(),
                seciciler: vec!["compiler/tests/a.rs::var_olan".into()],
                not: String::new(),
            },
            Kanit {
                madde: "spec/99-ornek.md#0000".into(),
                ozet: "x".into(),
                durum: "acik".into(),
                seciciler: vec![],
                not: "artık olmayan madde kaydı".into(),
            },
        ];
        let drift = drift_hesapla(&gecici, &maddeler, &harita);
        let _ = fs::remove_dir_all(&gecici);
        assert_eq!(
            drift
                .eksik
                .iter()
                .map(|m| m.kimlik.clone())
                .collect::<Vec<_>>(),
            [kimlik(2)]
        );
        assert_eq!(drift.bayat, ["spec/99-ornek.md#0000"]);
        assert!(drift.ihlaller.iter().any(|h| h.contains("özet bayat")));
        assert!(drift.ihlaller.iter().any(|h| h.contains("yok işlevi yok")));
        assert!(drift
            .ihlaller
            .iter()
            .any(|h| h.contains("AÇIK madde yalnız acik")));
        let rapor = rapor_metni(&maddeler, &harita, &drift);
        assert!(rapor.contains("KAYITSIZ") && rapor.contains("BAYAT") && rapor.contains("İHLAL"));
        assert!(rapor.contains("| `spec/99-ornek.md` | 4 | 3 | 0 | 0 |"));
    }
}
