//! K-170/ADR-066 birleşik güvenlik sürüm kapısı: bulgu kaydı şeması, açık
//! kritik/yüksek bulgu yasağı ve kapı betiğinin CI kablosu.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const KAYIT: &str = include_str!("../../docs/guvenlik-bulgulari-v1.tsv");
const SEMA: &str = "# zee-guvenlik-bulgulari-1";
const ASGARI_OZET: usize = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Bulgu {
    kimlik: String,
    tarih: String,
    kaynak: String,
    onem: String,
    yuzey: String,
    ozet: String,
    durum: String,
    kapanis: String,
    karar: String,
}

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü")
        .to_path_buf()
}

fn oku(goreli: &str) -> String {
    std::fs::read_to_string(depo().join(goreli)).expect("depo dosyası okunmalı")
}

fn tarih_gecerli(tarih: &str) -> bool {
    let parcalar = tarih.split('-').collect::<Vec<_>>();
    parcalar.len() == 3
        && parcalar[0].len() == 4
        && parcalar[1].len() == 2
        && parcalar[2].len() == 2
        && parcalar
            .iter()
            .all(|parca| parca.bytes().all(|b| b.is_ascii_digit()))
}

fn k_isi_gecerli(kayit: &str) -> bool {
    let Some(sayi) = kayit.strip_prefix("K-") else {
        return false;
    };
    let rakamlar = sayi.bytes().take_while(u8::is_ascii_digit).count();
    rakamlar >= 3 && sayi[rakamlar..].chars().all(|k| k.is_ascii_uppercase())
}

fn kayitlari_coz(metin: &str) -> Result<Vec<Bulgu>, String> {
    let mut satirlar = metin.lines();
    if satirlar.next() != Some(SEMA) {
        return Err(format!("bulgu kaydı {SEMA:?} başlığıyla başlamalı"));
    }
    let mut bulgular: Vec<Bulgu> = Vec::new();
    for satir in satirlar.filter(|s| !s.is_empty() && !s.starts_with('#')) {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [kimlik, tarih, kaynak, onem, yuzey, ozet, durum, kapanis, karar] = alanlar.as_slice()
        else {
            return Err(format!("bulgu satırı dokuz alan taşımalı: {satir:?}"));
        };
        let beklenen = format!("GB-{:03}", bulgular.len() + 1);
        if *kimlik != beklenen {
            return Err(format!(
                "bulgu kimliği ardışık olmalı: {kimlik} yerine {beklenen}"
            ));
        }
        if !tarih_gecerli(tarih) {
            return Err(format!("{kimlik}: tarih YYYY-MM-DD olmalı"));
        }
        if !matches!(
            *kaynak,
            "inceleme" | "fuzz" | "dogfood" | "advisory" | "drift" | "saha"
        ) {
            return Err(format!(
                "{kimlik}: kaynak inceleme|fuzz|dogfood|advisory|drift|saha olmalı"
            ));
        }
        if !matches!(*onem, "kritik" | "yuksek" | "orta" | "dusuk") {
            return Err(format!("{kimlik}: önem kritik|yuksek|orta|dusuk olmalı"));
        }
        if yuzey.is_empty() || !yuzey.chars().all(|k| k.is_ascii_lowercase() || k == '-') {
            return Err(format!("{kimlik}: yüzey küçük ASCII slug olmalı"));
        }
        if ozet.chars().count() < ASGARI_OZET {
            return Err(format!(
                "{kimlik}: özet en az {ASGARI_OZET} karakter olmalı"
            ));
        }
        match *durum {
            "kapali" => {
                if !k_isi_gecerli(kapanis) {
                    return Err(format!("{kimlik}: kapalı bulgu kapanış K-işi ister"));
                }
            }
            "acik" => {
                if !k_isi_gecerli(kapanis) {
                    return Err(format!("{kimlik}: açık bulgu hedef K-işi ister"));
                }
                if matches!(*onem, "kritik" | "yuksek") {
                    return Err(format!(
                        "{kimlik}: açık kritik/yüksek bulgu varken sürüm kapısı geçilemez"
                    ));
                }
            }
            "kabul" => {
                if *kapanis != "-" {
                    return Err(format!("{kimlik}: kabul edilen sınır kapanış işi taşımaz"));
                }
                if *onem == "kritik" {
                    return Err(format!("{kimlik}: kritik önemli sınır kabul edilemez"));
                }
            }
            _ => return Err(format!("{kimlik}: durum acik|kapali|kabul olmalı")),
        }
        let karar_gecerli = (karar.starts_with("adr/")
            || karar.starts_with("spec/")
            || karar.starts_with("rfcs/")
            || karar.starts_with("docs/"))
            && karar.ends_with(".md");
        if !karar_gecerli {
            return Err(format!(
                "{kimlik}: karar adr/, spec/, rfcs/ ya da docs/ Markdown yolu olmalı"
            ));
        }
        bulgular.push(Bulgu {
            kimlik: (*kimlik).to_string(),
            tarih: (*tarih).to_string(),
            kaynak: (*kaynak).to_string(),
            onem: (*onem).to_string(),
            yuzey: (*yuzey).to_string(),
            ozet: (*ozet).to_string(),
            durum: (*durum).to_string(),
            kapanis: (*kapanis).to_string(),
            karar: (*karar).to_string(),
        });
    }
    Ok(bulgular)
}

#[test]
fn bulgu_kaydi_gecerli_ve_acik_kritik_yuksek_bulgu_yoktur() {
    let bulgular = kayitlari_coz(KAYIT).expect("bulgu kaydı geçerli olmalı");
    assert!(bulgular.len() >= 20, "ilk taban en az 20 bulgu taşır");
    let acik_agir = bulgular
        .iter()
        .filter(|b| b.durum == "acik" && matches!(b.onem.as_str(), "kritik" | "yuksek"))
        .map(|b| b.kimlik.clone())
        .collect::<Vec<_>>();
    assert!(
        acik_agir.is_empty(),
        "açık kritik/yüksek bulgu: {acik_agir:?}"
    );
    for bulgu in &bulgular {
        assert!(
            depo().join(&bulgu.karar).is_file(),
            "{}: karar belgesi yok: {}",
            bulgu.kimlik,
            bulgu.karar
        );
        if bulgu.durum == "kapali" {
            let gunluk = oku("kararlar/gunluk.md");
            let backlog = oku("docs/oncelikli-backlog.md");
            assert!(
                gunluk.contains(&bulgu.kapanis) || backlog.contains(&bulgu.kapanis),
                "{}: kapanış işi {} günlük/backlog'da yok",
                bulgu.kimlik,
                bulgu.kapanis
            );
        }
        let _ = (&bulgu.tarih, &bulgu.kaynak, &bulgu.yuzey, &bulgu.ozet);
    }
    let kritik_kapali = bulgular
        .iter()
        .filter(|b| b.onem == "kritik")
        .all(|b| b.durum == "kapali");
    assert!(kritik_kapali, "kritik bulgu yalnız kapalı olabilir");
}

#[test]
fn guvenlik_kapisi_betigi_ci_ve_surum_adayi_adimlarini_tasir() {
    let betik = oku("scripts/guvenlik-kapisi.sh");
    for adim in [
        "set -euo pipefail",
        "--surekli",
        "--surum-adayi",
        "cargo deny --locked check -D warnings",
        "--manifest-path fuzz/Cargo.toml --config deny.toml --locked check -D warnings",
        "cargo check --manifest-path fuzz/Cargo.toml --locked --bins",
        "--test guvenlik_kapisi_testi --test tedarik_kapisi_testi",
        "docs/fuzz-rc-gecmisi-v1.tsv",
        "lexer_parser morfoloji http_istegi wasm_abi",
        "git status --porcelain",
        "cargo clippy --locked --all-targets -- -D warnings",
        "cargo test --locked --no-fail-fast",
        "scripts/core-freeze-korugu.sh",
    ] {
        assert!(betik.contains(adim), "güvenlik kapısı adımı eksik: {adim}");
    }
    let tedarik = oku(".github/workflows/tedarik.yml");
    assert!(tedarik.contains("bash scripts/guvenlik-kapisi.sh --surekli"));
    assert!(tedarik.contains("python3 scripts/tests/sir-yanlis-pozitif-test.py"));
    let scanner = oku("scripts/sir-taramasi.sh");
    assert!(scanner.contains("--ignore-gitleaks-allow"));
    assert!(scanner.contains("sir-yanlis-pozitif-denetle.py"));
    let records = oku("docs/sir-yanlis-pozitif-v1.tsv");
    let rows = records
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 4);
    for row in rows {
        let fields = row.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        assert!(fields[0].starts_with("47858358d6cf65210979fd210dfabae9465e9fb7:"));
        assert!(fields[0].contains(":generic-api-key:"));
        assert_eq!(fields[1].len(), 64);
        assert!(fields[2].starts_with("K-204:") && fields[2].chars().count() >= 40);
    }
    let politika = oku("SECURITY.md");
    for anahtar in [
        "docs/guvenlik-bulgulari-v1.tsv",
        "kritik",
        "yuksek",
        "--surum-adayi",
        "Security Advisories",
    ] {
        assert!(politika.contains(anahtar), "SECURITY.md eksik: {anahtar}");
    }
    let kaynaklar = kayitlari_coz(KAYIT)
        .expect("kayıt")
        .into_iter()
        .map(|b| b.kaynak)
        .collect::<BTreeSet<_>>();
    assert!(
        kaynaklar.contains("fuzz") && kaynaklar.contains("dogfood") && kaynaklar.contains("drift")
    );
}

#[test]
fn kayit_semasi_bozuk_ve_acik_agir_bulguyu_reddeder() {
    let satir = |kimlik: &str, onem: &str, durum: &str, kapanis: &str| {
        format!(
            "{kimlik}\t2026-09-05\tinceleme\t{onem}\tweb\tYeterince uzun bir bulgu özeti metni burada.\t{durum}\t{kapanis}\tadr/066-birlesik-guvenlik-surum-kapisi.md"
        )
    };
    let iyi = format!("{SEMA}\n{}\n", satir("GB-001", "orta", "acik", "K-172"));
    assert_eq!(kayitlari_coz(&iyi).expect("geçerli").len(), 1);
    for (bozuk, mesaj) in [
        (
            satir("GB-001", "yuksek", "acik", "K-172"),
            "sürüm kapısı geçilemez",
        ),
        (
            satir("GB-001", "kritik", "kabul", "-"),
            "kritik önemli sınır kabul edilemez",
        ),
        (
            satir("GB-001", "orta", "kapali", "-"),
            "kapanış K-işi ister",
        ),
        (
            satir("GB-001", "orta", "kabul", "K-100"),
            "kapanış işi taşımaz",
        ),
        (satir("GB-002", "orta", "acik", "K-172"), "ardışık olmalı"),
        (
            satir("GB-001", "belirsiz", "acik", "K-172"),
            "önem kritik|yuksek|orta|dusuk",
        ),
    ] {
        let hata = kayitlari_coz(&format!("{SEMA}\n{bozuk}\n")).unwrap_err();
        assert!(hata.contains(mesaj), "{bozuk:?} → {hata}");
    }
    let kisa = format!(
        "{SEMA}\nGB-001\t2026-09-05\tinceleme\torta\tweb\tkısa\tacik\tK-172\tadr/066-birlesik-guvenlik-surum-kapisi.md\n"
    );
    assert!(kayitlari_coz(&kisa).unwrap_err().contains("özet en az"));
    assert!(kayitlari_coz("# baska\n")
        .unwrap_err()
        .contains("başlığıyla başlamalı"));
}

#[test]
fn acik_depo_kapisi_anonim_politikayi_ve_ci_kablosunu_korur() {
    let workflow = oku(".github/workflows/tedarik.yml");
    assert!(workflow.contains("fetch-depth: 0"));
    assert!(workflow.contains("python3 scripts/tests/acik-depo-test.py"));
    assert!(workflow.contains("python3 scripts/acik-depo-denetle.py"));
    let trusted = oku(".github/workflows/acik-depo.yml");
    assert!(trusted.contains("pull_request_target:"));
    assert!(trusted.contains("ref: master"));
    assert!(trusted.contains("python3 kontrol/scripts/acik-depo-denetle.py --depo kaynak"));
    assert!(oku(".github/workflows/ci.yml").contains("ci-taban-kimligi.py"));
    assert!(!workflow.contains("ci-oz-kaynak.sh"));
    let policy: serde_json::Value = serde_json::from_str(&oku("docs/acik-depo-politikasi-v1.json"))
        .expect("anonim politika JSON");
    assert_eq!(policy["schema"], "zee-acik-depo-politikasi-1");
    assert_eq!(policy["history_base"].as_str().expect("SHA").len(), 40);
    assert!(!policy["rules"].as_array().expect("kurallar").is_empty());
    assert_eq!(policy["history_scope"], "all-reachable");
    assert!(policy["legacy_fingerprints"]
        .as_object()
        .unwrap()
        .is_empty());
    let result = std::process::Command::new(if cfg!(windows) { "python" } else { "python3" })
        .arg(depo().join("scripts/tests/acik-depo-test.py"))
        .current_dir(depo())
        .output()
        .expect("sentetik gizlilik kapısı testleri");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
// K-265 / ADR-124: actual owned-process and synthetic identity-protocol regressions.
#[test]
#[cfg_attr(
    not(unix),
    ignore = "actual POSIX dirfd/process capability unavailable"
)]
fn rc_kaniti_exact_kaynak_ve_kosucu_sahipligi() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    for (script, count) in [("rc-kaniti-test.py", 25), ("rc-kampanyasi-test.py", 9)] {
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests").join(script))
            .output()
            .expect("RC evidence protocol regression runner");
        let report = String::from_utf8_lossy(&result.stderr);
        assert!(result.status.success(), "{script}: {report}");
        assert!(
            report.contains(&format!("Ran {count} tests"))
                && report.contains("OK")
                && !report.contains("skipped"),
            "all protocol cases required: {report}"
        );
    }
}
