//! K-166/ADR-070 tanı kalitesi kapısı: golden korpusuna deterministik
//! "acemi hatası" mutasyonları uygulanır; her hata sınıfı için ilk tanının
//! kodu, işaretin hatalı satıra düşmesi, önerinin varlığı ve tek hatadan doğan
//! tanı gürültüsü ölçülür. Rapor `docs/tani-kalitesi.md`, eşikler sabittir,
//! gerekçeli istisnalar `docs/tani-kalitesi-istisnalari-v1.tsv` içindedir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const RAPOR_YOLU: &str = "docs/tani-kalitesi.md";
const ISTISNA_YOLU: &str = "docs/tani-kalitesi-istisnalari-v1.tsv";
const ISTISNA_SEMASI: &str = "# zee-tani-kalitesi-istisnalari-1";
/// Bir sınıf en az bu kadar vakayla "sık" sayılır ve kapıya girer.
const SIK_ESIGI: usize = 3;
/// En çok bu kadar sınıf kapıya alınır (frekansa göre).
const AZAMI_SINIF: usize = 50;
const ASGARI_ISARET_YUZDE: usize = 90;
const AZAMI_GURULTU_MEDYAN: usize = 2;
const AZAMI_GURULTU_TEPE: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mutasyon {
    operator: &'static str,
    kaynak: String,
    beklenen_satir: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Vaka {
    golden: String,
    operator: &'static str,
    beklenen_satir: usize,
    kod: String,
    satir: usize,
    isaret_dogru: bool,
    oneri_var: bool,
    gurultu: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sinif {
    operator: String,
    kod: String,
    vaka: usize,
    isaret_yuzde: usize,
    oneri_yuzde: usize,
    gurultu_medyan: usize,
    gurultu_tepe: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Kip {
    Denetle,
    RaporYaz,
}

fn depo_kokunu_bul(mut yol: PathBuf) -> Result<PathBuf, String> {
    loop {
        if yol.join("README.md").is_file() && yol.join("compiler/Cargo.toml").is_file() {
            return Ok(yol);
        }
        if !yol.pop() {
            return Err("depo kökü bulunamadı".into());
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
            "--depo" => {
                depo = Some(PathBuf::from(
                    argumanlar.next().ok_or("--depo bir yol ister")?,
                ))
            }
            _ => return Err("kullanım: tani_kalitesi [--denetle|--rapor-yaz] [--depo YOL]".into()),
        }
    }
    let depo = match depo {
        Some(d) => d,
        None => depo_kokunu_bul(std::env::current_dir().map_err(|h| h.to_string())?)?,
    };
    Ok((kip, depo))
}

fn yorum_mu(satir: &str) -> bool {
    satir.trim_start().starts_with('#')
}

fn girintili_mi(satir: &str) -> bool {
    satir.starts_with("    ") && !satir.trim().is_empty()
}

fn ilk<F: Fn(usize, &str) -> bool>(satirlar: &[String], f: F) -> Option<usize> {
    satirlar
        .iter()
        .enumerate()
        .find(|(i, s)| f(*i, s))
        .map(|(i, _)| i)
}

fn blok_acici_mi(satirlar: &[String], i: usize) -> bool {
    !satirlar[i].starts_with(' ')
        && !yorum_mu(&satirlar[i])
        && !satirlar[i].trim().is_empty()
        && satirlar.get(i + 1).is_some_and(|s| girintili_mi(s))
}

fn kosul_satiri_mi(satir: &str) -> bool {
    let govde = satir.trim_end();
    govde.ends_with(" ise") || govde.ends_with("se") || govde.ends_with("sa")
}

fn kelime_olarak_gecer(satir: &str, ad: &str) -> bool {
    satir
        .split(|k: char| !k.is_alphanumeric() && k != '_')
        .any(|k| k == ad)
}

/// Deterministik acemi hataları; her operatör kaynağa en fazla bir kez uygulanır.
fn mutasyonlar(kaynak: &str) -> Vec<Mutasyon> {
    let satirlar = kaynak.lines().map(str::to_string).collect::<Vec<_>>();
    let mut sonuc = Vec::new();
    let birlestir = |s: &[String]| s.join("\n") + "\n";

    if let Some(i) = ilk(&satirlar, |_, s| girintili_mi(s) && !yorum_mu(s)) {
        let mut m = satirlar.clone();
        m[i] = m[i].trim_start_matches(' ').to_string();
        sonuc.push(Mutasyon {
            operator: "girinti-sil",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
        let mut m = satirlar.clone();
        m[i] = format!("\t{}", &m[i][4..]);
        sonuc.push(Mutasyon {
            operator: "sekme-girinti",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
    }
    if let Some(i) = ilk(&satirlar, |_, s| s.contains('"') && !yorum_mu(s)) {
        let mut m = satirlar.clone();
        let k = m[i].rfind('"').expect("tırnak");
        m[i].remove(k);
        sonuc.push(Mutasyon {
            operator: "tirnak-kapatma",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
    }
    if let Some(i) = ilk(&satirlar, |_, s| {
        !s.starts_with(' ')
            && !yorum_mu(s)
            && !s.starts_with('"')
            && s.trim_end().ends_with(" olsun")
            && s.split(' ').count() >= 3
    }) {
        let parcalar = satirlar[i].trim_end().split(' ').collect::<Vec<_>>();
        let ad = parcalar[0].to_string();
        let mut m = satirlar.clone();
        m[i] = format!("{} = {}", ad, parcalar[1..parcalar.len() - 1].join(" "));
        sonuc.push(Mutasyon {
            operator: "esittir",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
        let mut m = satirlar.clone();
        m[i] = parcalar[..parcalar.len() - 1].join(" ");
        sonuc.push(Mutasyon {
            operator: "olsun-eksik",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
        if let Some(kullanim) = satirlar
            .iter()
            .enumerate()
            .skip(i + 1)
            .find(|(_, s)| {
                !yorum_mu(s) && kelime_olarak_gecer(s, &ad)
                    || (!yorum_mu(s)
                        && s.split(|k: char| !k.is_alphanumeric() && k != '_')
                            .any(|k| k.starts_with(&ad) && k != ad))
            })
            .map(|(j, _)| j)
        {
            let mut m = satirlar.clone();
            m[i] = format!("{}x {}", ad, parcalar[1..].join(" "));
            sonuc.push(Mutasyon {
                operator: "ad-yazim",
                kaynak: birlestir(&m),
                beklenen_satir: kullanim + 1,
            });
        }
    }
    if let Some(i) = ilk(&satirlar, |_, s| {
        s.trim_end().ends_with(" yaz") && !yorum_mu(s)
    }) {
        let mut m = satirlar.clone();
        let govde = m[i].trim_end().trim_end_matches("yaz").to_string();
        m[i] = format!("{govde}print");
        sonuc.push(Mutasyon {
            operator: "ingilizce-print",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
        let mut m = satirlar.clone();
        m[i] = format!("{govde}Yaz");
        sonuc.push(Mutasyon {
            operator: "buyuk-harf",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
    }
    if let Some(i) = ilk(&satirlar, |_, s| {
        !yorum_mu(s)
            && s.as_bytes()
                .windows(3)
                .any(|w| w[0].is_ascii_digit() && w[1] == b',' && w[2].is_ascii_digit())
    }) {
        let mut m = satirlar.clone();
        let baytlar = m[i].as_bytes();
        let konum = baytlar
            .windows(3)
            .position(|w| w[0].is_ascii_digit() && w[1] == b',' && w[2].is_ascii_digit())
            .expect("virgül")
            + 1;
        m[i].replace_range(konum..konum + 1, ".");
        sonuc.push(Mutasyon {
            operator: "nokta-ondalik",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
    }
    if let Some(i) = ilk(&satirlar, |i, s| {
        blok_acici_mi(&satirlar, i) && kosul_satiri_mi(s)
    }) {
        let mut m = satirlar.clone();
        let govde = m[i].trim_end();
        m[i] = if let Some(on) = govde.strip_suffix(" ise") {
            format!("({on}) ise")
        } else {
            format!("({govde})")
        };
        sonuc.push(Mutasyon {
            operator: "parantez",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
    }
    if let Some(i) = ilk(&satirlar, |i, _| blok_acici_mi(&satirlar, i)) {
        let mut m = satirlar[..=i].to_vec();
        let mut j = i + 1;
        while j < satirlar.len() && (girintili_mi(&satirlar[j]) || satirlar[j].trim().is_empty()) {
            j += 1;
        }
        m.extend_from_slice(&satirlar[j..]);
        sonuc.push(Mutasyon {
            operator: "bos-blok",
            kaynak: birlestir(&m),
            beklenen_satir: i + 1,
        });
    }
    sonuc
}

fn golden_kaynaklari(depo: &Path) -> Result<Vec<(String, String)>, String> {
    let mut yollar = std::fs::read_dir(depo.join("golden"))
        .map_err(|h| format!("golden okunamadı: {h}"))?
        .filter_map(|g| g.ok().map(|g| g.path()))
        .filter(|y| y.extension().and_then(|u| u.to_str()) == Some("dil"))
        .collect::<Vec<_>>();
    yollar.sort();
    let mut sonuc = Vec::new();
    for yol in yollar {
        let icerik = std::fs::read_to_string(&yol)
            .map_err(|h| format!("{} okunamadı: {h}", yol.display()))?;
        if icerik.contains("birimini kullan") || icerik.contains("paketini kullan") {
            continue;
        }
        sonuc.push((
            yol.file_name().expect("ad").to_string_lossy().into_owned(),
            icerik,
        ));
    }
    Ok(sonuc)
}

fn vakalari_olc(goldenler: &[(String, String)]) -> Vec<Vaka> {
    let mut vakalar = Vec::new();
    for (golden, kaynak) in goldenler {
        for mutasyon in mutasyonlar(kaynak) {
            let mut yukleyici = |ad: &str| -> Result<String, String> {
                Err(format!("{ad} birimi tanı kalitesi korpusunda yüklenmez"))
            };
            let tanilar = dil::kaynagi_tanilari(&mutasyon.kaynak, &mut yukleyici);
            let Some(ilk) = tanilar.first() else {
                vakalar.push(Vaka {
                    golden: golden.clone(),
                    operator: mutasyon.operator,
                    beklenen_satir: mutasyon.beklenen_satir,
                    kod: "TEMİZ".into(),
                    satir: 0,
                    isaret_dogru: false,
                    oneri_var: false,
                    gurultu: 0,
                });
                continue;
            };
            vakalar.push(Vaka {
                golden: golden.clone(),
                operator: mutasyon.operator,
                beklenen_satir: mutasyon.beklenen_satir,
                kod: ilk.kod.clone(),
                satir: ilk.satir,
                isaret_dogru: ilk.satir.abs_diff(mutasyon.beklenen_satir) <= 1,
                oneri_var: ilk.oneri.as_deref().is_some_and(|o| !o.trim().is_empty()),
                gurultu: tanilar.len(),
            });
        }
    }
    vakalar
}

fn siniflar(vakalar: &[Vaka]) -> Vec<Sinif> {
    let mut gruplar: BTreeMap<(String, String), Vec<&Vaka>> = BTreeMap::new();
    for vaka in vakalar {
        gruplar
            .entry((vaka.operator.to_string(), vaka.kod.clone()))
            .or_default()
            .push(vaka);
    }
    let mut siniflar = gruplar
        .into_iter()
        .map(|((operator, kod), uyeler)| {
            let n = uyeler.len();
            let mut gurultu = uyeler.iter().map(|v| v.gurultu).collect::<Vec<_>>();
            gurultu.sort_unstable();
            Sinif {
                operator,
                kod,
                vaka: n,
                isaret_yuzde: uyeler.iter().filter(|v| v.isaret_dogru).count() * 100 / n,
                oneri_yuzde: uyeler.iter().filter(|v| v.oneri_var).count() * 100 / n,
                gurultu_medyan: gurultu[n / 2],
                gurultu_tepe: *gurultu.last().expect("n>0"),
            }
        })
        .collect::<Vec<_>>();
    siniflar.sort_by(|a, b| {
        b.vaka
            .cmp(&a.vaka)
            .then_with(|| (&a.operator, &a.kod).cmp(&(&b.operator, &b.kod)))
    });
    siniflar
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Istisna {
    operator: String,
    kod: String,
    olcut: String,
    gerekce: String,
    is_kimligi: String,
}

fn istisnalari_coz(metin: &str) -> Result<Vec<Istisna>, String> {
    let mut satirlar = metin.lines();
    if satirlar.next() != Some(ISTISNA_SEMASI) {
        return Err(format!("istisna dosyası {ISTISNA_SEMASI:?} ile başlamalı"));
    }
    let mut sonuc = Vec::new();
    for satir in satirlar.filter(|s| !s.is_empty() && !s.starts_with('#')) {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [operator, kod, olcut, gerekce, is_kimligi] = alanlar.as_slice() else {
            return Err(format!("istisna satırı beş alan taşımalı: {satir:?}"));
        };
        if !matches!(*olcut, "isaret" | "oneri" | "gurultu") {
            return Err(format!(
                "{operator}/{kod}: ölçüt isaret|oneri|gurultu olmalı"
            ));
        }
        if gerekce.chars().count() < 40 || !is_kimligi.starts_with("K-") {
            return Err(format!(
                "{operator}/{kod}: istisna en az 40 karakter gerekçe ve K-işi ister"
            ));
        }
        sonuc.push(Istisna {
            operator: (*operator).to_string(),
            kod: (*kod).to_string(),
            olcut: (*olcut).to_string(),
            gerekce: (*gerekce).to_string(),
            is_kimligi: (*is_kimligi).to_string(),
        });
    }
    Ok(sonuc)
}

fn ihlaller(siniflar: &[Sinif], istisnalar: &[Istisna]) -> Vec<String> {
    let istisnali = |s: &Sinif, olcut: &str| {
        istisnalar
            .iter()
            .any(|i| i.operator == s.operator && i.kod == s.kod && i.olcut == olcut)
    };
    let mut sonuc = Vec::new();
    for sinif in siniflar
        .iter()
        .filter(|s| s.vaka >= SIK_ESIGI)
        .take(AZAMI_SINIF)
    {
        let ad = format!("{}/{}", sinif.operator, sinif.kod);
        if sinif.kod == "TEMİZ" {
            sonuc.push(format!(
                "{ad}: mutasyon tanı üretmedi ({} vaka)",
                sinif.vaka
            ));
            continue;
        }
        if sinif.oneri_yuzde < 100 && !istisnali(sinif, "oneri") {
            sonuc.push(format!(
                "{ad}: öneri %{} (RFC-0010 her tanı öneri taşır)",
                sinif.oneri_yuzde
            ));
        }
        if sinif.isaret_yuzde < ASGARI_ISARET_YUZDE && !istisnali(sinif, "isaret") {
            sonuc.push(format!(
                "{ad}: işaret hatalı satırda %{} < %{ASGARI_ISARET_YUZDE}",
                sinif.isaret_yuzde
            ));
        }
        if (sinif.gurultu_medyan > AZAMI_GURULTU_MEDYAN || sinif.gurultu_tepe > AZAMI_GURULTU_TEPE)
            && !istisnali(sinif, "gurultu")
        {
            sonuc.push(format!("{ad}: gürültü medyan {} tepe {} (sınır {AZAMI_GURULTU_MEDYAN}/{AZAMI_GURULTU_TEPE})", sinif.gurultu_medyan, sinif.gurultu_tepe));
        }
    }
    sonuc
}

fn rapor_metni(
    vakalar: &[Vaka],
    siniflar: &[Sinif],
    istisnalar: &[Istisna],
    ihlaller: &[String],
) -> String {
    let mut rapor = String::new();
    rapor.push_str("# Tanı kalitesi raporu\n\n<!-- `cd compiler && cargo run --locked --bin tani_kalitesi -- --rapor-yaz` üretir. Elle değiştirme. -->\n\n");
    rapor.push_str(&format!(
        "K-166/ADR-070 kapısı: golden korpusuna deterministik acemi hatası mutasyonları uygulanır ({} vaka, {} hata sınıfı). Sınıf = (operatör, ilk tanı kodu); en az {SIK_ESIGI} vakalı sınıflar frekans sırasıyla en çok {AZAMI_SINIF} taneye kadar kapıya girer. Ölçütler: öneri %100, işaret hatalı satırda ≥ %{ASGARI_ISARET_YUZDE}, gürültü medyan ≤ {AZAMI_GURULTU_MEDYAN} ve tepe ≤ {AZAMI_GURULTU_TEPE}. “En sık” sıralaması gerçek kullanıcı verisi (K-161/K-162) gelene kadar bu korpusun frekansıdır.\n\n",
        vakalar.len(), siniflar.len()
    ));
    rapor.push_str("| Sıra | Operatör | Kod | Vaka | İşaret | Öneri | Gürültü medyan | Gürültü tepe | Kapı |\n|---:|---|---|---:|---:|---:|---:|---:|---|\n");
    for (sira, sinif) in siniflar.iter().enumerate() {
        let kapida = sinif.vaka >= SIK_ESIGI && sira < AZAMI_SINIF;
        rapor.push_str(&format!(
            "| {} | `{}` | {} | {} | %{} | %{} | {} | {} | {} |\n",
            sira + 1,
            sinif.operator,
            sinif.kod,
            sinif.vaka,
            sinif.isaret_yuzde,
            sinif.oneri_yuzde,
            sinif.gurultu_medyan,
            sinif.gurultu_tepe,
            if kapida { "evet" } else { "seyrek" }
        ));
    }
    rapor.push_str("\n## İstisnalar\n\n");
    if istisnalar.is_empty() {
        rapor.push_str("Yok.\n");
    }
    for i in istisnalar {
        rapor.push_str(&format!(
            "- `{}`/{} {} — {} ({})\n",
            i.operator, i.kod, i.olcut, i.gerekce, i.is_kimligi
        ));
    }
    rapor.push_str("\n## İhlaller\n\n");
    if ihlaller.is_empty() {
        rapor.push_str("Yok.\n");
    }
    for ihlal in ihlaller {
        rapor.push_str(&format!("- {ihlal}\n"));
    }
    rapor.push_str("\n## Vakalar\n\n| Golden | Operatör | Beklenen satır | Kod | Satır | Öneri | Gürültü |\n|---|---|---:|---|---:|---|---:|\n");
    for v in vakalar {
        rapor.push_str(&format!(
            "| {} | `{}` | {} | {} | {} | {} | {} |\n",
            v.golden,
            v.operator,
            v.beklenen_satir,
            v.kod,
            v.satir,
            if v.oneri_var { "var" } else { "yok" },
            v.gurultu
        ));
    }
    rapor
}

fn calistir() -> Result<(), String> {
    let (kip, depo) = argumanlari_oku()?;
    let goldenler = golden_kaynaklari(&depo)?;
    let vakalar = vakalari_olc(&goldenler);
    let siniflar = siniflar(&vakalar);
    let istisnalar = match std::fs::read_to_string(depo.join(ISTISNA_YOLU)) {
        Ok(metin) => istisnalari_coz(&metin)?,
        Err(_) => Vec::new(),
    };
    let ihlaller = ihlaller(&siniflar, &istisnalar);
    let rapor = rapor_metni(&vakalar, &siniflar, &istisnalar, &ihlaller);
    let rapor_yolu = depo.join(RAPOR_YOLU);
    match kip {
        Kip::RaporYaz => {
            dil::kalici_dosya::atomik_yaz(&rapor_yolu, rapor.as_bytes())
                .map_err(|h| format!("{} yazılamadı: {h}", rapor_yolu.display()))?;
            println!(
                "Tanı kalitesi raporu güncellendi: {} vaka, {} sınıf, {} ihlal.",
                vakalar.len(),
                siniflar.len(),
                ihlaller.len()
            );
        }
        Kip::Denetle => {
            let mut hatalar = ihlaller.clone();
            if std::fs::read_to_string(&rapor_yolu).ok().as_deref() != Some(rapor.as_str()) {
                hatalar.push("tanı kalitesi raporu bayat; `cargo run --locked --bin tani_kalitesi -- --rapor-yaz` çalıştır".into());
            }
            if !hatalar.is_empty() {
                return Err(format!(
                    "tanı kalitesi incelemesi gerekli:\n- {}",
                    hatalar.join("\n- ")
                ));
            }
            println!(
                "Tanı kalitesi temiz: {} vaka, {} sınıf, rapor güncel.",
                vakalar.len(),
                siniflar.len()
            );
        }
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

    const ORNEK: &str =
        "# yorum\nsayı 5 olsun\nsayı 3 ten büyükse\n    \"büyük\" yaz\nfiyat 3,5 olsun\nsayı yaz\n";

    #[test]
    fn mutasyonlar_deterministik_ve_beklenen_satiri_tasir() {
        let m = mutasyonlar(ORNEK);
        let ops = m.iter().map(|x| x.operator).collect::<Vec<_>>();
        assert_eq!(
            ops,
            [
                "girinti-sil",
                "sekme-girinti",
                "tirnak-kapatma",
                "esittir",
                "olsun-eksik",
                "ad-yazim",
                "ingilizce-print",
                "buyuk-harf",
                "nokta-ondalik",
                "parantez",
                "bos-blok"
            ]
        );
        assert_eq!(mutasyonlar(ORNEK), m);
        let ad = m.iter().find(|x| x.operator == "ad-yazim").unwrap();
        assert_eq!(
            ad.beklenen_satir, 3,
            "ad-yazım beklentisi ilk kullanım satırıdır"
        );
        assert!(ad.kaynak.starts_with("# yorum\nsayıx 5 olsun\n"));
        let bos = m.iter().find(|x| x.operator == "bos-blok").unwrap();
        assert!(!bos.kaynak.contains("\"büyük\" yaz"));
        assert!(m
            .iter()
            .find(|x| x.operator == "parantez")
            .unwrap()
            .kaynak
            .contains("(sayı 3 ten büyükse)"));
        assert!(m
            .iter()
            .find(|x| x.operator == "nokta-ondalik")
            .unwrap()
            .kaynak
            .contains("3.5"));
    }

    #[test]
    fn siniflar_frekansa_gore_siralanir_ve_esikler_ihlali_adlandirir() {
        let vaka = |op: &'static str, kod: &str, isaret: bool, oneri: bool, gurultu: usize| Vaka {
            golden: "g".into(),
            operator: op,
            beklenen_satir: 1,
            kod: kod.into(),
            satir: 1,
            isaret_dogru: isaret,
            oneri_var: oneri,
            gurultu,
        };
        let vakalar = vec![
            vaka("a", "S001", true, true, 1),
            vaka("a", "S001", true, true, 1),
            vaka("a", "S001", true, false, 9),
            vaka("b", "S002", false, true, 1),
        ];
        let s = siniflar(&vakalar);
        assert_eq!(s[0].operator, "a");
        assert_eq!(s[0].oneri_yuzde, 66);
        assert_eq!(s[0].gurultu_tepe, 9);
        let ihl = ihlaller(&s, &[]);
        assert!(ihl.iter().any(|i| i.contains("a/S001: öneri")));
        assert!(ihl.iter().any(|i| i.contains("gürültü")));
        assert!(
            !ihl.iter().any(|i| i.contains("b/S002")),
            "seyrek sınıf kapıya girmez"
        );
        let istisna = Istisna {
            operator: "a".into(),
            kod: "S001".into(),
            olcut: "gurultu".into(),
            gerekce: "x".into(),
            is_kimligi: "K-1".into(),
        };
        assert!(!ihlaller(&s, &[istisna])
            .iter()
            .any(|i| i.contains("gürültü")));
    }

    #[test]
    fn istisna_dosyasi_fail_closed_cozulur() {
        let iyi = format!(
            "{ISTISNA_SEMASI}\na\tS001\tgurultu\t{}\tK-166\n",
            "g".repeat(40)
        );
        assert_eq!(istisnalari_coz(&iyi).unwrap().len(), 1);
        assert!(istisnalari_coz(&format!(
            "{ISTISNA_SEMASI}\na\tS001\tbelirsiz\t{}\tK-166\n",
            "g".repeat(40)
        ))
        .is_err());
        assert!(istisnalari_coz(&format!(
            "{ISTISNA_SEMASI}\na\tS001\tgurultu\tkısa\tK-166\n"
        ))
        .is_err());
        assert!(istisnalari_coz("# baska\n").is_err());
    }
}
