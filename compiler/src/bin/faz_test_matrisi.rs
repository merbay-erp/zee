//! Cargo test envanterini derleyici fazlarına eksiksiz sahipletir ve çalıştırır.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output};
use std::time::{Duration, Instant};

const MATRIS_YOLU: &str = "compiler/tests/fixtures/faz-test-matrisi-v1.tsv";
const DOKUMAN_YOLU: &str = "docs/faz-test-matrisi.md";
const LISTELEME_ESZAMANLILIGI: usize = 8;
const ZORUNLU_FAZLAR: [&str; 22] = [
    "lexer",
    "parser",
    "ast",
    "cozumleyici",
    "tur",
    "hir",
    "morfoloji",
    "runtime",
    "io",
    "eszamanlilik",
    "web_guvenlik",
    "http",
    "paket",
    "registry",
    "tedarik_zinciri",
    "lsp",
    "wasm",
    "cli",
    "proje",
    "semantic_regresyon",
    "uctan_uca",
    "muhe_kapilari",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum HedefTuru {
    Lib,
    Bin,
    Test,
    Doc,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct HedefKimligi {
    tur: HedefTuru,
    ad: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Secici {
    hedef: HedefKimligi,
    filtre: Option<String>,
}

#[derive(Clone, Debug)]
struct EkKapsam {
    secici: Secici,
    fazlar: Vec<String>,
}

#[derive(Clone, Debug)]
struct Faz {
    kimlik: String,
    baslik: String,
    seciciler: Vec<Secici>,
    regresyonlar: Vec<String>,
    fuzz: Vec<String>,
    conformance: Vec<String>,
    asagi_akis: Vec<String>,
    ek_kapsam: Vec<EkKapsam>,
    not: String,
}

#[derive(Clone, Debug)]
struct Hedef {
    kimlik: HedefKimligi,
    calistirilabilir: Option<PathBuf>,
    testler: Vec<String>,
}

#[derive(Default)]
struct FazSonucu {
    beklenen: usize,
    gecen: usize,
    kalan: usize,
    atlanan: usize,
    sure: Duration,
    hatalar: Vec<String>,
}

enum Kip {
    Denetle,
    DokumanYaz,
}

struct Ayarlar {
    kip: Kip,
    depo: PathBuf,
    rapor: Option<PathBuf>,
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

fn argumanlari_oku() -> Result<Ayarlar, String> {
    let mut kip = Kip::Denetle;
    let mut depo = None;
    let mut rapor = None;
    let mut argumanlar = env::args().skip(1);
    while let Some(arguman) = argumanlar.next() {
        match arguman.as_str() {
            "--denetle" => kip = Kip::Denetle,
            "--dokuman-yaz" => kip = Kip::DokumanYaz,
            "--rapor" => {
                rapor = Some(PathBuf::from(
                    argumanlar
                        .next()
                        .ok_or_else(|| "--rapor bir çıktı yolu ister".to_string())?,
                ));
            }
            "--depo" => {
                depo = Some(PathBuf::from(
                    argumanlar
                        .next()
                        .ok_or_else(|| "--depo bir yol ister".to_string())?,
                ));
            }
            "--yardim" | "-h" => {
                return Err(
                    "kullanım: faz_test_matrisi [--denetle|--dokuman-yaz] [--rapor YOL] [--depo YOL]"
                        .into(),
                );
            }
            _ => return Err(format!("bilinmeyen argüman: {arguman}")),
        }
    }
    if matches!(kip, Kip::DokumanYaz) && rapor.is_some() {
        return Err("--dokuman-yaz ile dinamik --rapor birlikte kullanılamaz".into());
    }
    let baslangic = depo.unwrap_or(env::current_dir().map_err(|hata| hata.to_string())?);
    Ok(Ayarlar {
        kip,
        depo: depo_kokunu_bul(baslangic)?,
        rapor,
    })
}

fn listeyi_oku(alan: &str) -> Vec<String> {
    if alan == "-" {
        Vec::new()
    } else {
        alan.split(';').map(str::to_string).collect()
    }
}

fn seciciyi_oku(metin: &str) -> Result<Secici, String> {
    let alanlar = metin.split('|').collect::<Vec<_>>();
    let (tur, ad, filtre) = match alanlar.as_slice() {
        ["test", ad] => (HedefTuru::Test, *ad, None),
        ["lib", filtre] => (HedefTuru::Lib, "dil", Some((*filtre).to_string())),
        ["bin", ad, filtre] => (HedefTuru::Bin, *ad, Some((*filtre).to_string())),
        ["doc", ad] => (HedefTuru::Doc, *ad, None),
        _ => return Err(format!("geçersiz faz seçicisi: {metin}")),
    };
    if ad.is_empty() || filtre.as_deref() == Some("") {
        return Err(format!("boş hedef/filtre taşıyan faz seçicisi: {metin}"));
    }
    Ok(Secici {
        hedef: HedefKimligi {
            tur,
            ad: ad.to_string(),
        },
        filtre,
    })
}

fn ek_kapsami_oku(alan: &str) -> Result<Vec<EkKapsam>, String> {
    if alan == "-" {
        return Ok(Vec::new());
    }
    let mut sonuc = Vec::new();
    let mut seciciler = BTreeSet::new();
    for kayit in alan.split(';') {
        let (secici, fazlar) = kayit
            .split_once('>')
            .ok_or_else(|| format!("geçersiz ek kapsam kaydı: {kayit}"))?;
        let secici = seciciyi_oku(secici)?;
        if !seciciler.insert(secici.clone()) {
            return Err(format!("yinelenen ek kapsam seçicisi: {secici:?}"));
        }
        let fazlar = fazlar.split(',').map(str::to_string).collect::<Vec<_>>();
        if fazlar.is_empty() || fazlar.iter().any(String::is_empty) {
            return Err(format!("boş faz taşıyan ek kapsam kaydı: {kayit}"));
        }
        sonuc.push(EkKapsam { secici, fazlar });
    }
    Ok(sonuc)
}

fn fazlari_oku(metin: &str) -> Result<Vec<Faz>, String> {
    let mut fazlar = Vec::new();
    let mut kimlikler = BTreeSet::new();
    let mut tum_seciciler = BTreeSet::new();
    for (sira, satir) in metin.lines().enumerate() {
        if satir.is_empty() || satir.starts_with('#') {
            continue;
        }
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let (
            kimlik,
            baslik,
            seciciler,
            regresyonlar,
            fuzz,
            conformance,
            asagi_akis,
            not,
            ek_kapsam,
        ) = match alanlar.as_slice() {
            [kimlik, baslik, seciciler, regresyonlar, fuzz, conformance, asagi_akis, not] => (
                *kimlik,
                *baslik,
                *seciciler,
                *regresyonlar,
                *fuzz,
                *conformance,
                *asagi_akis,
                *not,
                "-",
            ),
            [kimlik, baslik, seciciler, regresyonlar, fuzz, conformance, asagi_akis, not, ek_kapsam] => {
                (
                    *kimlik,
                    *baslik,
                    *seciciler,
                    *regresyonlar,
                    *fuzz,
                    *conformance,
                    *asagi_akis,
                    *not,
                    *ek_kapsam,
                )
            }
            _ => {
                return Err(format!(
                    "faz matrisi satırı {} sekiz veya dokuz alan taşımalı",
                    sira + 1
                ));
            }
        };
        if !kimlikler.insert(kimlik.to_string()) {
            return Err(format!("yinelenen faz kimliği: {kimlik}"));
        }
        if baslik.is_empty() || not.is_empty() {
            return Err(format!("{kimlik}: başlık ve kapsam notu zorunludur"));
        }
        let seciciler = listeyi_oku(seciciler)
            .into_iter()
            .map(|secici| seciciyi_oku(&secici))
            .collect::<Result<Vec<_>, _>>()?;
        if seciciler.is_empty() {
            return Err(format!("{kimlik}: en az bir test seçicisi ister"));
        }
        for secici in &seciciler {
            if !tum_seciciler.insert(secici.clone()) {
                return Err(format!("yinelenen test seçicisi: {secici:?}"));
            }
        }
        fazlar.push(Faz {
            kimlik: kimlik.to_string(),
            baslik: baslik.to_string(),
            seciciler,
            regresyonlar: listeyi_oku(regresyonlar),
            fuzz: listeyi_oku(fuzz),
            conformance: listeyi_oku(conformance),
            asagi_akis: listeyi_oku(asagi_akis),
            ek_kapsam: ek_kapsami_oku(ek_kapsam)?,
            not: not.to_string(),
        });
    }
    if fazlar.is_empty() {
        return Err("faz matrisi boş olamaz".into());
    }
    for faz in &fazlar {
        let mut gorulen = BTreeSet::new();
        for hedef in &faz.asagi_akis {
            if hedef == &faz.kimlik {
                return Err(format!("{} kendisini aşağı akış ilan edemez", faz.kimlik));
            }
            if !kimlikler.contains(hedef) {
                return Err(format!(
                    "{} bilinmeyen aşağı akış fazı: {hedef}",
                    faz.kimlik
                ));
            }
            if !gorulen.insert(hedef) {
                return Err(format!("{} yinelenen aşağı akış fazı: {hedef}", faz.kimlik));
            }
        }
        let sahipli_seciciler = faz.seciciler.iter().collect::<BTreeSet<_>>();
        for kapsam in &faz.ek_kapsam {
            if !sahipli_seciciler.contains(&kapsam.secici) {
                return Err(format!(
                    "{} ek kapsam seçicisi bu fazın birincil sahibi değil: {:?}",
                    faz.kimlik, kapsam.secici
                ));
            }
            let mut kapsananlar = BTreeSet::new();
            for kapsanan in &kapsam.fazlar {
                if kapsanan == &faz.kimlik {
                    return Err(format!(
                        "{} birincil fazı ek kapsam olarak yinelenemez",
                        faz.kimlik
                    ));
                }
                if !kimlikler.contains(kapsanan) {
                    return Err(format!(
                        "{} bilinmeyen ek kapsam fazı: {kapsanan}",
                        faz.kimlik
                    ));
                }
                if !kapsananlar.insert(kapsanan) {
                    return Err(format!(
                        "{} yinelenen ek kapsam fazı: {kapsanan}",
                        faz.kimlik
                    ));
                }
            }
        }
    }
    Ok(fazlar)
}

fn kaynaklari_dogrula(depo: &Path, fazlar: &[Faz]) -> Result<(), String> {
    for faz in fazlar {
        if faz.regresyonlar.is_empty() {
            return Err(format!(
                "{} fazı en az bir kalıcı regresyon kaynağı ister",
                faz.kimlik
            ));
        }
        for yol in faz
            .regresyonlar
            .iter()
            .chain(&faz.fuzz)
            .chain(&faz.conformance)
        {
            if !depo.join(yol).exists() {
                return Err(format!("{} fazının ilişki kaynağı yok: {yol}", faz.kimlik));
            }
        }
    }
    Ok(())
}

fn zorunlu_fazlari_dogrula(fazlar: &[Faz]) -> Result<(), String> {
    let mevcut = fazlar
        .iter()
        .map(|faz| faz.kimlik.as_str())
        .collect::<BTreeSet<_>>();
    let zorunlu = ZORUNLU_FAZLAR.into_iter().collect::<BTreeSet<_>>();
    if mevcut != zorunlu {
        return Err(format!(
            "faz sözleşmesi farklı; zorunlu={zorunlu:?}, mevcut={mevcut:?}"
        ));
    }
    Ok(())
}

fn komut_ciktisi(mut komut: Command, baglam: &str) -> Result<Output, String> {
    let cikti = komut
        .output()
        .map_err(|hata| format!("{baglam} başlatılamadı: {hata}"))?;
    if !cikti.status.success() {
        return Err(format!(
            "{baglam} başarısız:\n{}{}",
            String::from_utf8_lossy(&cikti.stdout),
            String::from_utf8_lossy(&cikti.stderr)
        ));
    }
    Ok(cikti)
}

fn test_listesini_oku(cikti: &[u8], baglam: &str) -> Result<Vec<String>, String> {
    let metin = std::str::from_utf8(cikti).map_err(|_| format!("{baglam} çıktısı UTF-8 değil"))?;
    let mut testler = Vec::new();
    for satir in metin.lines() {
        if let Some(ad) = satir.strip_suffix(": test") {
            testler.push(ad.to_string());
        }
    }
    testler.sort();
    if testler.windows(2).any(|pencere| pencere[0] == pencere[1]) {
        return Err(format!("{baglam} yinelenen test kimliği üretti"));
    }
    Ok(testler)
}

fn hedefi_listele(kimlik: HedefKimligi, yol: PathBuf) -> Result<Hedef, String> {
    let mut liste_komutu = Command::new(&yol);
    liste_komutu.args(["--list", "--format", "terse"]);
    let liste = komut_ciktisi(liste_komutu, &format!("{kimlik:?} test listesi"))?;
    let testler = test_listesini_oku(&liste.stdout, &format!("{kimlik:?}"))?;
    Ok(Hedef {
        kimlik,
        calistirilabilir: Some(yol),
        testler,
    })
}

fn cargo_envanteri(depo: &Path) -> Result<BTreeMap<HedefKimligi, Hedef>, String> {
    let compiler = depo.join("compiler");
    let mut komut = Command::new("cargo");
    komut
        .args(["test", "--locked", "--no-run", "--message-format=json"])
        .current_dir(&compiler)
        .env("CARGO_TERM_COLOR", "never");
    let cikti = komut_ciktisi(komut, "faz matrisi test derlemesi")?;
    let stdout = std::str::from_utf8(&cikti.stdout)
        .map_err(|_| "Cargo JSON çıktısı UTF-8 değil".to_string())?;
    let mut ham_hedefler = BTreeMap::new();
    for (sira, satir) in stdout.lines().enumerate() {
        if satir.trim().is_empty() {
            continue;
        }
        let deger: Value = serde_json::from_str(satir)
            .map_err(|hata| format!("Cargo JSON satırı {} bozuk: {hata}", sira + 1))?;
        if deger["reason"] != "compiler-artifact" || deger["profile"]["test"] != true {
            continue;
        }
        let Some(calistirilabilir) = deger["executable"].as_str() else {
            continue;
        };
        let ad = deger["target"]["name"]
            .as_str()
            .ok_or_else(|| "Cargo test hedefi ad taşımıyor".to_string())?;
        let turler = deger["target"]["kind"]
            .as_array()
            .ok_or_else(|| format!("Cargo hedefi tür taşımıyor: {ad}"))?;
        let tur = if turler.iter().any(|tur| tur == "test") {
            HedefTuru::Test
        } else if turler.iter().any(|tur| tur == "lib") {
            HedefTuru::Lib
        } else if turler.iter().any(|tur| tur == "bin") {
            HedefTuru::Bin
        } else {
            continue;
        };
        let kimlik = HedefKimligi {
            tur,
            ad: ad.to_string(),
        };
        if ham_hedefler
            .insert(kimlik.clone(), PathBuf::from(calistirilabilir))
            .is_some()
        {
            return Err(format!("Cargo yinelenen test hedefi üretti: {kimlik:?}"));
        }
    }

    let mut hedefler = BTreeMap::new();
    let mut listelenecekler = Vec::new();
    for (kimlik, yol) in ham_hedefler {
        if kimlik.tur == HedefTuru::Test {
            hedefler.insert(
                kimlik.clone(),
                Hedef {
                    kimlik,
                    calistirilabilir: Some(yol),
                    testler: Vec::new(),
                },
            );
        } else {
            listelenecekler.push((kimlik, yol));
        }
    }
    for parca in listelenecekler.chunks(LISTELEME_ESZAMANLILIGI) {
        let isler = parca
            .iter()
            .map(|(kimlik, yol)| {
                let kimlik = kimlik.clone();
                let yol = yol.clone();
                std::thread::spawn(move || hedefi_listele(kimlik, yol))
            })
            .collect::<Vec<_>>();
        for is in isler {
            let hedef = is
                .join()
                .map_err(|_| "test listesi iş parçacığı panikledi".to_string())??;
            hedefler.insert(hedef.kimlik.clone(), hedef);
        }
    }

    let mut doc_komutu = Command::new("cargo");
    doc_komutu
        .args([
            "test", "--locked", "--doc", "--", "--list", "--format", "terse",
        ])
        .current_dir(&compiler)
        .env("CARGO_TERM_COLOR", "never");
    let doc_ciktisi = komut_ciktisi(doc_komutu, "doctest listesi")?;
    let kimlik = HedefKimligi {
        tur: HedefTuru::Doc,
        ad: "dil".to_string(),
    };
    hedefler.insert(
        kimlik.clone(),
        Hedef {
            kimlik,
            calistirilabilir: None,
            testler: test_listesini_oku(&doc_ciktisi.stdout, "doctest listesi")?,
        },
    );
    Ok(hedefler)
}

fn secici_testi_kapsar(secici: &Secici, hedef: &HedefKimligi, test: &str) -> bool {
    if secici.hedef != *hedef {
        return false;
    }
    secici
        .filtre
        .as_ref()
        .is_none_or(|filtre| test.contains(filtre))
}

fn sahipligi_dogrula(
    fazlar: &[Faz],
    hedefler: &BTreeMap<HedefKimligi, Hedef>,
) -> Result<(), String> {
    let tum_seciciler = fazlar
        .iter()
        .flat_map(|faz| {
            faz.seciciler
                .iter()
                .map(move |secici| (&faz.kimlik, secici))
        })
        .collect::<Vec<_>>();
    let entegrasyonlar = hedefler
        .keys()
        .filter(|kimlik| kimlik.tur == HedefTuru::Test)
        .cloned()
        .collect::<BTreeSet<_>>();
    let sahipli_entegrasyonlar = tum_seciciler
        .iter()
        .filter(|(_, secici)| secici.hedef.tur == HedefTuru::Test)
        .map(|(_, secici)| secici.hedef.clone())
        .collect::<BTreeSet<_>>();
    if entegrasyonlar != sahipli_entegrasyonlar {
        return Err(format!(
            "entegrasyon hedef sahipliği farklı; Cargo={entegrasyonlar:?}, matris={sahipli_entegrasyonlar:?}"
        ));
    }

    let mut secici_eslesmeleri = BTreeMap::<Secici, usize>::new();
    for hedef in hedefler.values() {
        if hedef.kimlik.tur == HedefTuru::Test {
            continue;
        }
        for test in &hedef.testler {
            let sahipler = tum_seciciler
                .iter()
                .filter(|(_, secici)| secici_testi_kapsar(secici, &hedef.kimlik, test))
                .collect::<Vec<_>>();
            if sahipler.len() != 1 {
                return Err(format!(
                    "{:?}::{test} tam bir faz sahibi ister; bulunan={}",
                    hedef.kimlik,
                    sahipler.len()
                ));
            }
            let (_, secici) = sahipler[0];
            *secici_eslesmeleri.entry((*secici).clone()).or_default() += 1;
        }
    }
    for (faz, secici) in tum_seciciler {
        if secici.hedef.tur == HedefTuru::Test {
            continue;
        }
        if secici_eslesmeleri.get(secici).copied().unwrap_or_default() == 0 {
            return Err(format!(
                "{faz} fazındaki seçici hiçbir testle eşleşmedi: {secici:?}"
            ));
        }
    }
    Ok(())
}

fn seciciyi_yaz(secici: &Secici) -> String {
    match (&secici.hedef.tur, &secici.filtre) {
        (HedefTuru::Test, None) => format!("test:{}", secici.hedef.ad),
        (HedefTuru::Lib, Some(filtre)) => format!("lib:{filtre}"),
        (HedefTuru::Bin, Some(filtre)) => format!("bin:{}:{filtre}", secici.hedef.ad),
        (HedefTuru::Doc, None) => format!("doc:{}", secici.hedef.ad),
        _ => format!("geçersiz:{secici:?}"),
    }
}

fn kod_listesi(degerler: &[String]) -> String {
    if degerler.is_empty() {
        "—".to_string()
    } else {
        degerler
            .iter()
            .map(|deger| format!("`{}`", deger.replace('|', "\\|")))
            .collect::<Vec<_>>()
            .join("<br>")
    }
}

fn ek_kapsami_yaz(faz: &Faz) -> Vec<String> {
    faz.ek_kapsam
        .iter()
        .map(|kapsam| {
            format!(
                "{} → {}",
                seciciyi_yaz(&kapsam.secici),
                kapsam.fazlar.join(", ")
            )
        })
        .collect()
}

fn capraz_kapsayan_seciciler(fazlar: &[Faz], hedef_faz: &str) -> Vec<String> {
    fazlar
        .iter()
        .flat_map(|birincil| {
            birincil
                .ek_kapsam
                .iter()
                .filter(move |kapsam| kapsam.fazlar.iter().any(|faz| faz == hedef_faz))
                .map(|kapsam| format!("{} / {}", birincil.kimlik, seciciyi_yaz(&kapsam.secici)))
        })
        .collect()
}

fn kanonik_dokuman(fazlar: &[Faz]) -> String {
    let mut metin = String::from(
        "# Faza özgü test matrisi\n\n\
         > Bu dosya `cargo run --locked --bin faz_test_matrisi -- --dokuman-yaz` ile\n\
         > üretilir. Faz sahipliği ve saldırı yüzeyi ilişkileri elle değiştirilmez;\n\
         > gerçek test sayıları her platformun dinamik CI raporunda çıkar.\n\n",
    );
    metin.push_str(&format!(
        "- Şema: `zee-faz-test-matrisi-2`\n- Faz: **{}**\n- Sayım: Her CI işletim sisteminde derlenen gerçek test envanteri\n\n",
        fazlar.len()
    ));
    metin.push_str("## Envanter ve saldırı yüzeyi ilişkileri\n\n");
    metin.push_str("| Faz | Kalıcı regresyon kaynağı | Fuzz | Conformance | Aşağı akış |\n");
    metin.push_str("|---|---|---|---|---|\n");
    for faz in fazlar {
        metin.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            faz.baslik,
            kod_listesi(&faz.regresyonlar),
            kod_listesi(&faz.fuzz),
            kod_listesi(&faz.conformance),
            kod_listesi(&faz.asagi_akis)
        ));
    }
    metin.push_str("\n## Birincil sahiplik\n\n");
    metin.push_str("Bir test tam bir birincil faza aittir. Seçici düzeyindeki isteğe bağlı\n");
    metin.push_str("ek kapsam, test grubunun gerçekten yokladığı diğer fazları bildirir.\n\n");
    metin.push_str("| Faz | Cargo/libtest seçicileri | Ek kapsam | Kapsam |\n|---|---|---|---|\n");
    for faz in fazlar {
        let seciciler = faz.seciciler.iter().map(seciciyi_yaz).collect::<Vec<_>>();
        metin.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            faz.baslik,
            kod_listesi(&seciciler),
            kod_listesi(&ek_kapsami_yaz(faz)),
            faz.not.replace('|', "\\|")
        ));
    }
    metin.push_str("\n## Gerçek blast radius\n\n");
    metin.push_str("Bir faz değiştiğinde kendi birincil gruplarına ek olarak aşağıdaki çapraz\n");
    metin.push_str(
        "seçiciler doğrudan kanıt taşır. Aşağı akış sütunu mimari yayılımı gösterir.\n\n",
    );
    metin.push_str(
        "| Değişen faz | Birincil test grupları | Çapraz kapsayan test grupları | Aşağı akış |\n",
    );
    metin.push_str("|---|---|---|---|\n");
    for faz in fazlar {
        let birincil = faz.seciciler.iter().map(seciciyi_yaz).collect::<Vec<_>>();
        metin.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            faz.baslik,
            kod_listesi(&birincil),
            kod_listesi(&capraz_kapsayan_seciciler(fazlar, &faz.kimlik)),
            kod_listesi(&faz.asagi_akis)
        ));
    }
    metin.push_str(
        "\n## Çalıştırma sözleşmesi\n\n\
         `cargo run --locked --bin faz_test_matrisi -- --denetle --rapor \
         target/faz-test-matrisi.md` önce bütün test hedeflerini JSON Cargo çıktısından\n\
         derler. Her gerçek test kimliğinin tam bir faz sahibi olduğunu, ek kapsamın\n\
         yalnız bilinen fazlara ve kendi birincil seçicisine bağlandığını ve bu belgenin\n\
         güncel kaldığını doğrular; ardından fazları ayrı çalıştırıp test sayısı,\n\
         pass/fail/ignored ve duvar süresini dinamik Markdown raporuna yazar. `cfg`\n\
         koşullu testler nedeniyle sayı işletim sistemine göre değişebilir; sahiplik\n\
         kuralı her platformda yeniden kanıtlanır. Derleme\n\
         hatası, sahipsiz/yinelenen test, boş seçici, bayat belge, beklenmeyen test\n\
         sayısı veya başarısız test kapıyı kapatır. CI raporu step summary ve indirilebilir\n\
         artefakt olarak yayımlar; süre gözlemseldir, performans eşiği değildir.\n",
    );
    metin
}

fn belgeyi_denetle(depo: &Path, beklenen: &str) -> Result<(), String> {
    let yol = depo.join(DOKUMAN_YOLU);
    let mevcut =
        fs::read_to_string(&yol).map_err(|hata| format!("{} okunamadı: {hata}", yol.display()))?;
    if mevcut != beklenen {
        return Err(format!(
            "{} bayat; `cargo run --locked --bin faz_test_matrisi -- --dokuman-yaz` çalıştırın",
            yol.display()
        ));
    }
    Ok(())
}

fn belgeyi_yaz(depo: &Path, metin: &str) -> Result<(), String> {
    let yol = depo.join(DOKUMAN_YOLU);
    fs::write(&yol, metin).map_err(|hata| format!("{} yazılamadı: {hata}", yol.display()))
}

fn ozet_sayisini_oku(parca: &str, etiket: &str) -> Option<usize> {
    let konum = parca.find(etiket)?;
    parca[..konum].split_whitespace().next_back()?.parse().ok()
}

fn test_ozetini_oku(cikti: &[u8]) -> Result<(usize, usize, usize), String> {
    let metin = std::str::from_utf8(cikti).map_err(|_| "test çıktısı UTF-8 değil".to_string())?;
    let satir = metin
        .lines()
        .find(|satir| satir.starts_with("test result:"))
        .ok_or_else(|| "libtest sonuç satırı bulunamadı".to_string())?;
    let parcalar = satir.split(';').collect::<Vec<_>>();
    let gecen = parcalar
        .iter()
        .find_map(|parca| ozet_sayisini_oku(parca, " passed"))
        .ok_or_else(|| "geçen test sayısı ayrıştırılamadı".to_string())?;
    let kalan = parcalar
        .iter()
        .find_map(|parca| ozet_sayisini_oku(parca, " failed"))
        .ok_or_else(|| "kalan test sayısı ayrıştırılamadı".to_string())?;
    let atlanan = parcalar
        .iter()
        .find_map(|parca| ozet_sayisini_oku(parca, " ignored"))
        .ok_or_else(|| "atlanan test sayısı ayrıştırılamadı".to_string())?;
    Ok((gecen, kalan, atlanan))
}

fn seciciyi_calistir(depo: &Path, hedef: &Hedef, secici: &Secici) -> Result<Output, String> {
    if secici.hedef.tur == HedefTuru::Doc {
        let mut komut = Command::new("cargo");
        komut
            .args(["test", "--locked", "--doc"])
            .current_dir(depo.join("compiler"))
            .env("CARGO_TERM_COLOR", "never");
        return komut
            .output()
            .map_err(|hata| format!("doctest başlatılamadı: {hata}"));
    }
    let calistirilabilir = hedef
        .calistirilabilir
        .as_ref()
        .ok_or_else(|| format!("çalıştırılabilir hedef yok: {:?}", hedef.kimlik))?;
    let mut komut = Command::new(calistirilabilir);
    if let Some(filtre) = &secici.filtre {
        komut.arg(filtre);
    }
    komut
        .current_dir(depo.join("compiler"))
        .env("RUST_BACKTRACE", "1")
        .output()
        .map_err(|hata| format!("{} başlatılamadı: {hata}", seciciyi_yaz(secici)))
}

fn fazlari_calistir(
    depo: &Path,
    fazlar: &[Faz],
    hedefler: &BTreeMap<HedefKimligi, Hedef>,
) -> BTreeMap<String, FazSonucu> {
    let mut sonuclar = BTreeMap::new();
    for faz in fazlar {
        let mut sonuc = FazSonucu::default();
        let baslangic = Instant::now();
        for secici in &faz.seciciler {
            let Some(hedef) = hedefler.get(&secici.hedef) else {
                sonuc
                    .hatalar
                    .push(format!("hedef envanterde yok: {:?}", secici.hedef));
                continue;
            };
            let bilinen_beklenen = (secici.hedef.tur != HedefTuru::Test).then(|| {
                hedef
                    .testler
                    .iter()
                    .filter(|test| secici_testi_kapsar(secici, &hedef.kimlik, test))
                    .count()
            });
            sonuc.beklenen += bilinen_beklenen.unwrap_or_default();
            match seciciyi_calistir(depo, hedef, secici) {
                Ok(cikti) => match test_ozetini_oku(&cikti.stdout) {
                    Ok((gecen, kalan, atlanan)) => {
                        let calisan = gecen + kalan + atlanan;
                        if let Some(beklenen) = bilinen_beklenen {
                            if calisan != beklenen {
                                sonuc.hatalar.push(format!(
                                    "{} için beklenen {beklenen} test yerine {calisan} sonuç alındı",
                                    seciciyi_yaz(secici)
                                ));
                            }
                        } else {
                            sonuc.beklenen += calisan;
                            if calisan == 0 {
                                sonuc.hatalar.push(format!(
                                    "{} entegrasyon hedefi hiç test çalıştırmadı",
                                    seciciyi_yaz(secici)
                                ));
                            }
                        }
                        sonuc.gecen += gecen;
                        sonuc.kalan += kalan;
                        sonuc.atlanan += atlanan;
                        if !cikti.status.success() || kalan > 0 {
                            sonuc.hatalar.push(format!(
                                "{} başarısız:\n{}{}",
                                seciciyi_yaz(secici),
                                String::from_utf8_lossy(&cikti.stdout),
                                String::from_utf8_lossy(&cikti.stderr)
                            ));
                        }
                    }
                    Err(hata) => sonuc.hatalar.push(format!(
                        "{}: {hata}\n{}{}",
                        seciciyi_yaz(secici),
                        String::from_utf8_lossy(&cikti.stdout),
                        String::from_utf8_lossy(&cikti.stderr)
                    )),
                },
                Err(hata) => sonuc.hatalar.push(hata),
            }
        }
        sonuc.sure = baslangic.elapsed();
        sonuclar.insert(faz.kimlik.clone(), sonuc);
    }
    sonuclar
}

fn dinamik_rapor(fazlar: &[Faz], sonuclar: &BTreeMap<String, FazSonucu>) -> String {
    let toplam = sonuclar.values().map(|sonuc| sonuc.beklenen).sum::<usize>();
    let gecen = sonuclar.values().map(|sonuc| sonuc.gecen).sum::<usize>();
    let kalan = sonuclar.values().map(|sonuc| sonuc.kalan).sum::<usize>();
    let atlanan = sonuclar.values().map(|sonuc| sonuc.atlanan).sum::<usize>();
    let sure = sonuclar.values().map(|sonuc| sonuc.sure).sum::<Duration>();
    let basarili = sonuclar
        .values()
        .all(|sonuc| sonuc.hatalar.is_empty() && sonuc.kalan == 0);
    let mut metin = format!(
        "# Zee faz test sonucu\n\n- Sonuç: **{}**\n- Toplam: **{gecen}/{toplam} geçti**, {kalan} kaldı, {atlanan} atlandı\n- Faz süreleri toplamı: **{:.3} sn**\n\n",
        if basarili { "GEÇTİ" } else { "KALDI" },
        sure.as_secs_f64()
    );
    metin.push_str("| Faz | Sonuç | Geçen/Beklenen | Kalan | Atlanan | Duvar süresi | Regresyon | Fuzz | Conformance | Çapraz kapsama | Aşağı akış |\n");
    metin.push_str("|---|---|---:|---:|---:|---:|---|---|---|---|---|\n");
    for faz in fazlar {
        let sonuc = &sonuclar[&faz.kimlik];
        let durum = if sonuc.hatalar.is_empty() && sonuc.kalan == 0 {
            "GEÇTİ"
        } else {
            "KALDI"
        };
        metin.push_str(&format!(
            "| {} | {} | {}/{} | {} | {} | {:.3} sn | {} | {} | {} | {} | {} |\n",
            faz.baslik,
            durum,
            sonuc.gecen,
            sonuc.beklenen,
            sonuc.kalan,
            sonuc.atlanan,
            sonuc.sure.as_secs_f64(),
            kod_listesi(&faz.regresyonlar),
            kod_listesi(&faz.fuzz),
            kod_listesi(&faz.conformance),
            kod_listesi(&capraz_kapsayan_seciciler(fazlar, &faz.kimlik)),
            kod_listesi(&faz.asagi_akis)
        ));
    }
    let hatalar = fazlar
        .iter()
        .flat_map(|faz| {
            sonuclar[&faz.kimlik]
                .hatalar
                .iter()
                .map(move |hata| (&faz.baslik, hata))
        })
        .collect::<Vec<_>>();
    if !hatalar.is_empty() {
        metin.push_str("\n## Hatalar\n\n");
        for (faz, hata) in hatalar {
            metin.push_str(&format!("### {faz}\n\n```text\n{hata}\n```\n\n"));
        }
    }
    metin
}

fn raporu_yaz(depo: &Path, yol: &Path, metin: &str) -> Result<PathBuf, String> {
    let tam_yol = if yol.is_absolute() {
        yol.to_path_buf()
    } else {
        depo.join("compiler").join(yol)
    };
    if let Some(ebeveyn) = tam_yol.parent() {
        fs::create_dir_all(ebeveyn)
            .map_err(|hata| format!("{} oluşturulamadı: {hata}", ebeveyn.display()))?;
    }
    fs::write(&tam_yol, metin)
        .map_err(|hata| format!("{} yazılamadı: {hata}", tam_yol.display()))?;
    Ok(tam_yol)
}

fn calistir() -> Result<(), String> {
    let ayarlar = argumanlari_oku()?;
    let matris_yolu = ayarlar.depo.join(MATRIS_YOLU);
    let matris = fs::read_to_string(&matris_yolu)
        .map_err(|hata| format!("{} okunamadı: {hata}", matris_yolu.display()))?;
    let fazlar = fazlari_oku(&matris)?;
    zorunlu_fazlari_dogrula(&fazlar)?;
    kaynaklari_dogrula(&ayarlar.depo, &fazlar)?;
    let belge = kanonik_dokuman(&fazlar);
    if matches!(ayarlar.kip, Kip::DokumanYaz) {
        belgeyi_yaz(&ayarlar.depo, &belge)?;
        println!("Faz matrisi belgesi güncellendi: {DOKUMAN_YOLU}");
        return Ok(());
    }
    belgeyi_denetle(&ayarlar.depo, &belge)?;
    let hedefler = cargo_envanteri(&ayarlar.depo)?;
    sahipligi_dogrula(&fazlar, &hedefler)?;
    let sonuclar = fazlari_calistir(&ayarlar.depo, &fazlar, &hedefler);
    let rapor = dinamik_rapor(&fazlar, &sonuclar);
    println!("{rapor}");
    if let Some(yol) = &ayarlar.rapor {
        let tam_yol = raporu_yaz(&ayarlar.depo, yol, &rapor)?;
        eprintln!("Faz matrisi dinamik raporu: {}", tam_yol.display());
    }
    if sonuclar
        .values()
        .any(|sonuc| !sonuc.hatalar.is_empty() || sonuc.kalan > 0)
    {
        return Err("faza özgü test matrisi başarısız".into());
    }
    Ok(())
}

fn main() -> ExitCode {
    match calistir() {
        Ok(()) => ExitCode::SUCCESS,
        Err(hata) => {
            eprintln!("faz test matrisi hatası: {hata}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn hedef(tur: HedefTuru, ad: &str, testler: &[&str]) -> Hedef {
        Hedef {
            kimlik: HedefKimligi {
                tur,
                ad: ad.to_string(),
            },
            calistirilabilir: None,
            testler: testler.iter().map(|test| (*test).to_string()).collect(),
        }
    }

    #[test]
    fn secici_turleri_kesin_ayristirilir() {
        assert_eq!(
            seciciyi_oku("test|parser_kurtarma_testi")
                .expect("test seçicisi")
                .hedef
                .tur,
            HedefTuru::Test
        );
        assert_eq!(
            seciciyi_oku("lib|hir::testler::")
                .expect("lib seçicisi")
                .filtre
                .as_deref(),
            Some("hir::testler::")
        );
        assert!(seciciyi_oku("bin|eksik").is_err());
    }

    #[test]
    fn yinelenen_faz_ve_secici_reddedilir() {
        let satir = "a\tA\ttest|x\t-\t-\t-\tb\tnot\t-";
        let metin = format!("{satir}\n{satir}\n");
        assert!(fazlari_oku(&metin).is_err());
    }

    #[test]
    fn zorunlu_fazlardan_biri_sessizce_dusurulemez() {
        let fazlar = fazlari_oku("lexer\tLexer\ttest|x\tx\t-\t-\t-\tnot\t-\n").expect("faz");
        assert!(zorunlu_fazlari_dogrula(&fazlar).is_err());
    }

    #[test]
    fn her_test_tam_bir_faz_sahibi_ister() {
        let fazlar = fazlari_oku("a\tA\tlib|mod::\t-\t-\t-\t-\tnot\t-\n").expect("faz");
        let kimlik = HedefKimligi {
            tur: HedefTuru::Lib,
            ad: "dil".to_string(),
        };
        let hedefler = BTreeMap::from([(
            kimlik,
            hedef(HedefTuru::Lib, "dil", &["mod::bir", "baska::iki"]),
        )]);
        assert!(sahipligi_dogrula(&fazlar, &hedefler).is_err());
    }

    #[test]
    fn libtest_ozeti_gecen_kalan_ve_atlanani_ayirir() {
        let cikti = b"test result: FAILED. 7 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        assert_eq!(test_ozetini_oku(cikti).expect("özet"), (7, 2, 1));
    }

    #[test]
    fn iliski_listesi_markdown_hucresinde_kodlanir() {
        assert_eq!(kod_listesi(&[]), "—");
        assert_eq!(
            kod_listesi(&["a/b".to_string(), "c|d".to_string()]),
            "`a/b`<br>`c\\|d`"
        );
    }

    #[test]
    fn ek_kapsam_sahipli_secici_ve_bilinen_faz_ister() {
        let gecerli = "a\tA\ttest|x\tx\t-\t-\tb\tnot\ttest|x>b\n\
                       b\tB\ttest|y\ty\t-\t-\t-\tnot\t-\n";
        let fazlar = fazlari_oku(gecerli).expect("ek kapsam ayrıştırılmalı");
        assert_eq!(capraz_kapsayan_seciciler(&fazlar, "b"), ["a / test:x"]);

        let sahipsiz = "a\tA\ttest|x\tx\t-\t-\tb\tnot\ttest|z>b\n\
                        b\tB\ttest|y\ty\t-\t-\t-\tnot\t-\n";
        assert!(fazlari_oku(sahipsiz).is_err());
        let bilinmeyen = "a\tA\ttest|x\tx\t-\t-\t-\tnot\ttest|x>y\n";
        assert!(fazlari_oku(bilinmeyen).is_err());
        let birincil_tekrari = "a\tA\ttest|x\tx\t-\t-\t-\tnot\ttest|x>a\n";
        assert!(fazlari_oku(birincil_tekrari).is_err());
    }
}
