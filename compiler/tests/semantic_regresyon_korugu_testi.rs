use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler depo içinde olmalı")
        .to_path_buf()
}

fn gecici_depo() -> PathBuf {
    let damga = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("saat")
        .as_nanos();
    std::env::temp_dir().join(format!("zee-semantic-koruk-{}-{damga}", std::process::id()))
}

fn git(kok: &Path, argumanlar: &[&str]) -> Output {
    Command::new("git")
        .args(argumanlar)
        .current_dir(kok)
        .output()
        .expect("git çalışmalı")
}

fn basarili_git(kok: &Path, argumanlar: &[&str]) -> String {
    let cikti = git(kok, argumanlar);
    assert!(
        cikti.status.success(),
        "git {:?}: {}",
        argumanlar,
        String::from_utf8_lossy(&cikti.stderr)
    );
    String::from_utf8(cikti.stdout)
        .expect("git çıktısı UTF-8")
        .trim()
        .to_string()
}

fn commit(kok: &Path, mesaj: &str) -> String {
    basarili_git(kok, &["add", "."]);
    basarili_git(
        kok,
        &[
            "-c",
            "user.name=Zee Test",
            "-c",
            "user.email=zee@example.invalid",
            "commit",
            "-q",
            "-m",
            mesaj,
        ],
    );
    basarili_git(kok, &["rev-parse", "HEAD"])
}

fn korugu_calistir(kok: &Path, taban: &str) -> Output {
    Command::new("bash")
        .arg(depo().join("scripts/semantic-regresyon-korugu.sh"))
        .arg(taban)
        .current_dir(kok)
        .output()
        .expect("koruk çalışmalı")
}

#[test]
fn fixture_siz_bug_fix_reddedilir_ve_provenance_yeniden_yazilamaz() {
    let kok = gecici_depo();
    std::fs::create_dir_all(kok.join("regression")).expect("regression klasörü");
    std::fs::create_dir_all(kok.join("compiler/src")).expect("compiler klasörü");
    basarili_git(&kok, &["init", "-q"]);
    std::fs::write(
        kok.join("regression/v2.tsv"),
        "# zee-semantic-regresyon-2\n# alanlar\n\
         eski\tK-001\t0000000000000000000000000000000000000000\t-\t0.8.0-dev\tparser\trun\t-\t-\t0\t-\tregression/parser/eski.dil\n",
    )
    .expect("manifest yazılmalı");
    std::fs::write(kok.join("compiler/src/lib.rs"), "pub fn eski() {}\n")
        .expect("kaynak yazılmalı");
    let taban = commit(&kok, "başlangıç");

    std::fs::write(kok.join("compiler/src/lib.rs"), "pub fn duzeltilmis() {}\n")
        .expect("düzeltme yazılmalı");
    let fixed_by = commit(&kok, "fix: compiler hatasını düzelt");
    let eksik = korugu_calistir(&kok, &taban);
    assert!(!eksik.status.success(), "fixture'sız bug fix geçmemeli");
    assert!(String::from_utf8_lossy(&eksik.stderr)
        .contains("COMPILER BUG DÜZELTMESİ REGRESYON PROVENANCE'I TAŞIMIYOR"));

    let yeni = format!(
        "yeni\tK-002\t{fixed_by}\t-\t0.8.0-dev\tchecker\trun\t-\t-\t0\t-\tregression/checker/yeni.dil\n"
    );
    let manifest = std::fs::read_to_string(kok.join("regression/v2.tsv")).expect("manifest");
    std::fs::write(kok.join("regression/v2.tsv"), format!("{manifest}{yeni}"))
        .expect("provenance yazılmalı");
    let provenance = commit(&kok, "regresyon provenance kaydı");
    assert!(
        korugu_calistir(&kok, &taban).status.success(),
        "exact fixed_by sahibi fixture geçmeli"
    );

    let manifest = std::fs::read_to_string(kok.join("regression/v2.tsv")).expect("manifest");
    std::fs::write(
        kok.join("regression/v2.tsv"),
        manifest.replacen(
            "0000000000000000000000000000000000000000",
            "1111111111111111111111111111111111111111",
            1,
        ),
    )
    .expect("bozuk provenance yazılmalı");
    commit(&kok, "provenance yeniden yazımı");
    let yeniden_yazim = korugu_calistir(&kok, &provenance);
    assert!(!yeniden_yazim.status.success());
    assert!(String::from_utf8_lossy(&yeniden_yazim.stderr)
        .contains("SEMANTIC REGRESYON PROVENANCE'I YENİDEN YAZILDI"));

    std::fs::remove_dir_all(&kok).expect("geçici depo temizlenmeli");
}
