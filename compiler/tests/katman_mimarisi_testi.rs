//! K-149/ADR-046 production modül sahipliği ve bağımlılık yönü kapısı.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const SEMA: &str = "# zee-katman-mimarisi-1";
const KATMANLAR: &[&str] = &[
    "temel",
    "model",
    "altyapi",
    "sozdizimi",
    "semantik",
    "proje",
    "web",
    "runtime",
    "adapter",
    "muhendislik",
];

#[derive(Debug)]
struct Politika {
    katman: String,
    bagimliliklar: BTreeSet<String>,
    sorumluluk: String,
}

fn depo_koku() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler depo kökünde olmalı")
        .to_path_buf()
}

fn politikayi_oku() -> BTreeMap<String, Politika> {
    let yol = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/katman-mimarisi-v1.tsv");
    let metin = fs::read_to_string(yol).expect("katman politikası okunmalı");
    assert_eq!(metin.lines().next(), Some(SEMA));
    assert!(metin.ends_with('\n') && !metin.contains('\r'));
    let mut politikalar = BTreeMap::new();
    for (sira, satir) in metin.lines().enumerate() {
        if satir.is_empty() || satir.starts_with('#') {
            continue;
        }
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [modul, katman, bagimliliklar, sorumluluk] = alanlar.as_slice() else {
            panic!("katman politikası satır {} dört alan taşımalı", sira + 1);
        };
        assert!(
            modul
                .bytes()
                .all(|bayt| bayt.is_ascii_lowercase() || bayt == b'_'),
            "geçersiz modül kimliği: {modul}"
        );
        assert!(KATMANLAR.contains(katman), "bilinmeyen katman: {katman}");
        assert!(!sorumluluk.is_empty(), "{modul}: sorumluluk boş olamaz");
        let bagimliliklar = if *bagimliliklar == "-" {
            BTreeSet::new()
        } else {
            let degerler = bagimliliklar
                .split(',')
                .map(str::to_string)
                .collect::<Vec<_>>();
            let mut sirali = degerler.clone();
            sirali.sort();
            sirali.dedup();
            assert_eq!(degerler, sirali, "{modul}: bağımlılıklar kanonik değil");
            degerler.into_iter().collect()
        };
        assert!(
            politikalar
                .insert(
                    (*modul).into(),
                    Politika {
                        katman: (*katman).into(),
                        bagimliliklar,
                        sorumluluk: (*sorumluluk).into(),
                    },
                )
                .is_none(),
            "yinelenen modül: {modul}"
        );
    }
    politikalar
}

fn rs_dosyalarini_topla(kok: &Path, dosyalar: &mut Vec<PathBuf>) {
    let mut girdiler = fs::read_dir(kok)
        .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", kok.display()))
        .map(|girdi| girdi.expect("dizin girdisi okunmalı").path())
        .collect::<Vec<_>>();
    girdiler.sort();
    for yol in girdiler {
        if yol.is_dir() {
            rs_dosyalarini_topla(&yol, dosyalar);
        } else if yol.extension().and_then(|uzanti| uzanti.to_str()) == Some("rs") {
            dosyalar.push(yol);
        }
    }
}

fn dosya_sahibi(goreli: &Path) -> String {
    let parcalar = goreli
        .components()
        .map(|parca| parca.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(parcalar.first().map(String::as_str), Some("src"));
    match parcalar.get(1).map(String::as_str) {
        Some("lib.rs") => "lib".into(),
        Some("main.rs" | "cli") => "dil_cli".into(),
        Some("bin") => {
            let kok = Path::new(parcalar.get(2).expect("bin dosyası olmalı"))
                .file_stem()
                .expect("bin kökü olmalı")
                .to_string_lossy();
            if kok == "dillsp" {
                "dillsp".into()
            } else {
                format!("bin_{kok}")
            }
        }
        Some(kok) => Path::new(kok)
            .file_stem()
            .expect("kaynak kökü olmalı")
            .to_string_lossy()
            .into_owned(),
        None => panic!("sahipsiz kaynak yolu: {}", goreli.display()),
    }
}

fn ham_metin_sonu(baytlar: &[u8], baslangic: usize) -> Option<usize> {
    let mut sira = baslangic;
    if baytlar.get(sira) == Some(&b'b') {
        sira += 1;
    }
    if baytlar.get(sira) != Some(&b'r') {
        return None;
    }
    sira += 1;
    let diyez_baslangici = sira;
    while baytlar.get(sira) == Some(&b'#') {
        sira += 1;
    }
    let diyez = sira - diyez_baslangici;
    if baytlar.get(sira) != Some(&b'"') {
        return None;
    }
    sira += 1;
    while sira < baytlar.len() {
        if baytlar[sira] == b'"'
            && baytlar.get(sira + 1..sira + 1 + diyez)
                == Some(&baytlar[diyez_baslangici..diyez_baslangici + diyez])
        {
            return Some(sira + 1 + diyez);
        }
        sira += 1;
    }
    Some(baytlar.len())
}

fn tokenlere_ayir(kaynak: &str) -> Vec<String> {
    let baytlar = kaynak.as_bytes();
    let mut tokenler = Vec::new();
    let mut sira = 0usize;
    while sira < baytlar.len() {
        if baytlar[sira].is_ascii_whitespace() {
            sira += 1;
        } else if baytlar.get(sira..sira + 2) == Some(b"//") {
            sira += 2;
            while sira < baytlar.len() && baytlar[sira] != b'\n' {
                sira += 1;
            }
        } else if baytlar.get(sira..sira + 2) == Some(b"/*") {
            sira += 2;
            let mut derinlik = 1usize;
            while sira < baytlar.len() && derinlik > 0 {
                if baytlar.get(sira..sira + 2) == Some(b"/*") {
                    derinlik += 1;
                    sira += 2;
                } else if baytlar.get(sira..sira + 2) == Some(b"*/") {
                    derinlik -= 1;
                    sira += 2;
                } else {
                    sira += 1;
                }
            }
        } else if let Some(son) = ham_metin_sonu(baytlar, sira) {
            sira = son;
        } else if baytlar[sira] == b'"'
            || (baytlar[sira] == b'b' && baytlar.get(sira + 1) == Some(&b'"'))
        {
            if baytlar[sira] == b'b' {
                sira += 1;
            }
            sira += 1;
            while sira < baytlar.len() {
                match baytlar[sira] {
                    b'\\' => sira = (sira + 2).min(baytlar.len()),
                    b'"' => {
                        sira += 1;
                        break;
                    }
                    _ => sira += 1,
                }
            }
        } else if baytlar[sira] == b'\'' {
            let mut kimlik_sonu = sira + 1;
            while kimlik_sonu < baytlar.len()
                && (baytlar[kimlik_sonu].is_ascii_alphanumeric() || baytlar[kimlik_sonu] == b'_')
            {
                kimlik_sonu += 1;
            }
            if kimlik_sonu > sira + 1 && baytlar.get(kimlik_sonu) != Some(&b'\'') {
                // Rust yaşam süresi (`'a`, `'static`); sonraki tur kimliği okur.
                sira += 1;
            } else {
                // Karakter sabiti; kaçışlı ve Unicode içerik kapanışa dek atlanır.
                sira += 1;
                let mut kacis = false;
                while sira < baytlar.len() {
                    if !kacis && baytlar[sira] == b'\'' {
                        sira += 1;
                        break;
                    }
                    kacis = !kacis && baytlar[sira] == b'\\';
                    if baytlar[sira] != b'\\' {
                        kacis = false;
                    }
                    sira += 1;
                }
            }
        } else if baytlar[sira].is_ascii_alphabetic() || baytlar[sira] == b'_' {
            let baslangic = sira;
            sira += 1;
            while sira < baytlar.len()
                && (baytlar[sira].is_ascii_alphanumeric() || baytlar[sira] == b'_')
            {
                sira += 1;
            }
            tokenler.push(kaynak[baslangic..sira].to_string());
        } else if baytlar.get(sira..sira + 2) == Some(b"::") {
            tokenler.push("::".into());
            sira += 2;
        } else {
            tokenler.push((baytlar[sira] as char).to_string());
            sira += 1;
        }
    }
    tokenler
}

fn test_kodunu_cikar(tokenler: &[String]) -> Vec<String> {
    let mut sonuc = Vec::new();
    let mut sira = 0usize;
    while sira < tokenler.len() {
        if tokenler.get(sira).map(String::as_str) == Some("#")
            && tokenler.get(sira + 1).map(String::as_str) == Some("[")
        {
            let mut son = sira + 2;
            let mut koseli = 1usize;
            while son < tokenler.len() && koseli > 0 {
                match tokenler[son].as_str() {
                    "[" => koseli += 1,
                    "]" => koseli -= 1,
                    _ => {}
                }
                son += 1;
            }
            let cfg_test = tokenler.get(sira + 2).map(String::as_str) == Some("cfg")
                && tokenler.get(sira + 3).map(String::as_str) == Some("(")
                && tokenler.get(sira + 4).map(String::as_str) == Some("test")
                && tokenler.get(sira + 5).map(String::as_str) == Some(")");
            if cfg_test {
                sira = son;
                let mut parantez = 0usize;
                let mut koseli = 0usize;
                while sira < tokenler.len() {
                    match tokenler[sira].as_str() {
                        "(" => parantez += 1,
                        ")" => parantez = parantez.saturating_sub(1),
                        "[" => koseli += 1,
                        "]" => koseli = koseli.saturating_sub(1),
                        "{" if parantez == 0 && koseli == 0 => {
                            let mut suslu = 1usize;
                            sira += 1;
                            while sira < tokenler.len() && suslu > 0 {
                                match tokenler[sira].as_str() {
                                    "{" => suslu += 1,
                                    "}" => suslu -= 1,
                                    _ => {}
                                }
                                sira += 1;
                            }
                            break;
                        }
                        ";" | "," if parantez == 0 && koseli == 0 => {
                            sira += 1;
                            break;
                        }
                        "}" if parantez == 0 && koseli == 0 => break,
                        _ => {}
                    }
                    sira += 1;
                }
                continue;
            }
        }
        sonuc.push(tokenler[sira].clone());
        sira += 1;
    }
    sonuc
}

fn kok_sembol_sahibi(sembol: &str) -> &str {
    match sembol {
        "BirimIstegi"
        | "BirimYukleyici"
        | "KokenliBirimYukleyici"
        | "YuklenenBirim"
        | "birim_ozeti"
        | "gomulu_birim"
        | "gomulu_birim_adlari" => "proje",
        "kaynagi_derle"
        | "kaynagi_derle_birimlerle"
        | "kaynagi_derle_kokenlerle"
        | "kaynagi_fazli_derle"
        | "kaynagi_fazli_derle_birimlerle"
        | "kaynagi_fazli_derle_kokenlerle"
        | "kaynagi_tanilari"
        | "kaynagi_tanilari_kokenlerle" => "faz",
        "programi_dene" | "programi_dene_baglanmis" => "yorumlayici",
        _ => "lib",
    }
}

fn use_agaci_hedefleri(
    tokenler: &[String],
    baslangic: usize,
    moduller: &BTreeSet<String>,
    hedefler: &mut BTreeSet<String>,
) -> usize {
    let mut sira = baslangic + 1;
    let mut derinlik = 1usize;
    let mut yeni_kol = true;
    while sira < tokenler.len() && derinlik > 0 {
        match tokenler[sira].as_str() {
            "{" => derinlik += 1,
            "}" => derinlik -= 1,
            "," if derinlik == 1 => yeni_kol = true,
            sembol if derinlik == 1 && yeni_kol => {
                if sembol != "self" {
                    hedefler.insert(if moduller.contains(sembol) {
                        sembol.to_string()
                    } else {
                        kok_sembol_sahibi(sembol).to_string()
                    });
                }
                yeni_kol = false;
            }
            _ => {}
        }
        sira += 1;
    }
    sira
}

fn bagimliliklari_bul(kaynak: &str, moduller: &BTreeSet<String>) -> BTreeSet<String> {
    let tokenler = test_kodunu_cikar(&tokenlere_ayir(kaynak));
    assert!(
        !tokenler.windows(2).any(|pencere| {
            matches!(pencere[0].as_str(), "crate" | "dil") && pencere[1] == "as"
        }) && !tokenler.windows(4).any(|pencere| {
            pencere[0] == "extern"
                && pencere[1] == "crate"
                && pencere[2] == "self"
                && pencere[3] == "as"
        }) && !tokenler.windows(5).any(|pencere| {
            matches!(pencere[0].as_str(), "crate" | "dil")
                && pencere[1] == "::"
                && pencere[2] == "{"
                && pencere[3] == "self"
                && pencere[4] == "as"
        }),
        "crate/dil kökü takma adla katman taramasından gizlenemez"
    );
    let mut hedefler = BTreeSet::new();
    let mut sira = 0usize;
    while sira + 2 < tokenler.len() {
        let koklu =
            matches!(tokenler[sira].as_str(), "crate" | "dil") && tokenler[sira + 1] == "::";
        if koklu {
            if tokenler[sira + 2] == "{" {
                sira = use_agaci_hedefleri(&tokenler, sira + 2, moduller, &mut hedefler);
                continue;
            }
            let sembol = &tokenler[sira + 2];
            hedefler.insert(if moduller.contains(sembol) {
                sembol.clone()
            } else {
                kok_sembol_sahibi(sembol).to_string()
            });
            sira += 3;
        } else if tokenler[sira] == "super" && tokenler[sira + 1] == "::" {
            let mut hedef = sira + 2;
            while tokenler.get(hedef).is_some_and(|sembol| sembol == "super")
                && tokenler.get(hedef + 1).is_some_and(|sembol| sembol == "::")
            {
                hedef += 2;
            }
            if let Some(sembol) = tokenler
                .get(hedef)
                .filter(|sembol| moduller.contains(*sembol))
            {
                hedefler.insert(sembol.clone());
            }
            sira = hedef.saturating_add(1);
        } else {
            if moduller.contains(&tokenler[sira]) && tokenler[sira + 1] == "::" {
                hedefler.insert(tokenler[sira].clone());
            }
            sira += 1;
        }
    }
    hedefler
}

fn katman_izinli(kaynak: &str, hedef: &str) -> bool {
    let izinliler: &[&str] = match kaynak {
        "temel" => &["temel"],
        "model" => &["temel", "model"],
        "altyapi" => &["temel", "model", "altyapi"],
        "sozdizimi" => &["temel", "model", "altyapi", "sozdizimi"],
        "semantik" => &["temel", "model", "altyapi", "sozdizimi", "semantik"],
        "proje" => &[
            "temel",
            "model",
            "altyapi",
            "sozdizimi",
            "semantik",
            "proje",
        ],
        "web" => &["temel", "model", "altyapi", "proje", "web"],
        "runtime" => &[
            "temel",
            "model",
            "altyapi",
            "sozdizimi",
            "semantik",
            "proje",
            "web",
            "runtime",
        ],
        "adapter" => &[
            "temel",
            "model",
            "altyapi",
            "sozdizimi",
            "semantik",
            "proje",
            "web",
            "runtime",
            "adapter",
        ],
        "muhendislik" => KATMANLAR,
        _ => return false,
    };
    izinliler.contains(&hedef)
}

#[test]
fn production_modulleri_ve_bagimliliklari_incelenmis_tabanla_birebir() {
    let depo = depo_koku();
    let derleyici = depo.join("compiler");
    let kaynak_koku = derleyici.join("src");
    let politikalar = politikayi_oku();
    let moduller = politikalar.keys().cloned().collect::<BTreeSet<_>>();
    let mut dosyalar = Vec::new();
    rs_dosyalarini_topla(&kaynak_koku, &mut dosyalar);
    let mut sahipler = BTreeSet::new();
    let mut gercek = BTreeMap::<String, BTreeSet<String>>::new();
    for yol in dosyalar {
        let goreli = yol
            .strip_prefix(&derleyici)
            .expect("kaynak compiler içinde olmalı");
        let sahip = dosya_sahibi(goreli);
        sahipler.insert(sahip.clone());
        if yol.file_name().and_then(|ad| ad.to_str()) == Some("testler.rs") {
            continue;
        }
        let kaynak = fs::read_to_string(&yol).expect("Rust kaynağı okunmalı");
        gercek
            .entry(sahip.clone())
            .or_default()
            .extend(bagimliliklari_bul(&kaynak, &moduller));
    }
    for (sahip, hedefler) in &mut gercek {
        hedefler.remove(sahip);
    }
    assert_eq!(
        sahipler, moduller,
        "yeni/kayıp production modülü fixture incelemesi ister"
    );

    let mut farklar = Vec::new();
    for (modul, politika) in &politikalar {
        let bulunan = gercek.get(modul).cloned().unwrap_or_default();
        if bulunan != politika.bagimliliklar {
            farklar.push(format!(
                "{modul}: beklenen={:?}, bulunan={bulunan:?}",
                politika.bagimliliklar
            ));
        }
        for hedef in &bulunan {
            let hedef_politikasi = politikalar
                .get(hedef)
                .unwrap_or_else(|| panic!("{modul}: bilinmeyen hedef {hedef}"));
            assert!(
                katman_izinli(&politika.katman, &hedef_politikasi.katman),
                "{modul} ({}) -> {hedef} ({}) ters katman bağımlılığı; {}",
                politika.katman,
                hedef_politikasi.katman,
                politika.sorumluluk
            );
        }
    }
    assert!(
        farklar.is_empty(),
        "bağımlılık tabanı bilinçli mimari inceleme istiyor:\n{}",
        farklar.join("\n")
    );
}

#[test]
fn tarayici_yorum_metin_ham_metin_ve_test_kodunu_bagimlilik_saymaz() {
    let moduller = ["agac", "yorumlayici"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let kaynak = r###"
        use crate::agac::Program;
        use super::super::yorumlayici;
        fn omur<'a>(deger: &'a str) -> &'a str { deger }
        const KARAKTER: char = '}';
        // crate::yorumlayici::calistir();
        const YAZI: &str = "crate::yorumlayici::calistir()";
        const HAM: &str = r#"crate::yorumlayici::{x}"#;
        #[cfg(test)]
        mod testler { use crate::yorumlayici; }
    "###;
    assert_eq!(
        bagimliliklari_bul(kaynak, &moduller),
        BTreeSet::from(["agac".to_string(), "yorumlayici".to_string()])
    );
    assert!(
        std::panic::catch_unwind(|| bagimliliklari_bul("use crate as gizli;", &moduller)).is_err()
    );
    assert!(std::panic::catch_unwind(|| {
        bagimliliklari_bul("extern crate self as gizli;", &moduller)
    })
    .is_err());
    assert!(std::panic::catch_unwind(|| {
        bagimliliklari_bul("use crate::{self as gizli};", &moduller)
    })
    .is_err());
}

#[test]
fn cekirdek_katmanlar_yukari_dogru_bagimlanamaz() {
    assert!(katman_izinli("semantik", "sozdizimi"));
    assert!(katman_izinli("runtime", "semantik"));
    assert!(!katman_izinli("sozdizimi", "semantik"));
    assert!(!katman_izinli("semantik", "runtime"));
    assert!(!katman_izinli("runtime", "adapter"));
    assert!(!katman_izinli("adapter", "muhendislik"));
}
