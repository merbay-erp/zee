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
    std::env::temp_dir().join(format!("zee-core-freeze-{}-{damga}", std::process::id()))
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
        .arg(depo().join("scripts/core-freeze-korugu.sh"))
        .arg(taban)
        .current_dir(kok)
        .output()
        .expect("koruk çalışmalı")
}

#[test]
fn feature_gercek_dogfood_kaniti_olmadan_freeze_kapisini_gecemez() {
    let kok = gecici_depo();
    for klasor in [
        "compiler/src",
        "docs",
        "spec",
        "regression",
        "projeler/catli",
    ] {
        std::fs::create_dir_all(kok.join(klasor)).expect("fixture klasörü");
    }
    basarili_git(&kok, &["init", "-q"]);
    std::fs::write(kok.join("compiler/src/lib.rs"), "pub fn cekirdek() {}\n")
        .expect("başlangıç kaynağı");
    std::fs::write(kok.join("spec/ozellik.md"), "# Kanıt\n").expect("karar kanıtı");
    std::fs::write(kok.join("regression/istek.dil"), "1 yaz\n").expect("reproducer");
    std::fs::write(kok.join("projeler/catli/proje.dil"), "proje catli\n").expect("ürün kanıtı");
    let freeze_parent = commit(&kok, "core freeze başlangıcı");

    std::fs::write(
        kok.join("docs/compiler-degisiklik-beyanlari-v1.tsv"),
        format!(
            "# zee-compiler-degisiklik-beyanlari-1\n\
             # enforcement_parent\t{freeze_parent}\n\
             # commit\tsinif\tkanit\tgerekce\n"
        ),
    )
    .expect("semantic manifest");
    std::fs::write(
        kok.join("docs/core-freeze-beyanlari-v1.tsv"),
        format!(
            "# zee-core-freeze-beyanlari-1\n\
             # enforcement_parent\t{freeze_parent}\n\
             # commit\tsinif\turun\tis\treproducer\tetkilenen_proje\tminimalite\tkarar\n"
        ),
    )
    .expect("freeze manifest");
    commit(&kok, "freeze kapısı");

    std::fs::write(
        kok.join("compiler/src/lib.rs"),
        "pub fn cekirdek() {}\npub fn yeni_ozellik() {}\n",
    )
    .expect("feature kaynağı");
    let feature = commit(&kok, "yeni özellik");
    let semantic_yolu = kok.join("docs/compiler-degisiklik-beyanlari-v1.tsv");
    let semantic = std::fs::read_to_string(&semantic_yolu).expect("semantic manifest");
    std::fs::write(
        &semantic_yolu,
        format!(
            "{semantic}{feature}\tsemantic-change\tspec/ozellik.md\tYeni davranış normatif karara bağlı kullanıcı görünür değişikliktir.\n"
        ),
    )
    .expect("semantic beyan");
    commit(&kok, "semantic beyan");

    let eksik = korugu_calistir(&kok, &freeze_parent);
    assert!(!eksik.status.success(), "beyansız feature geçmemeli");
    assert!(String::from_utf8_lossy(&eksik.stderr)
        .contains("CORE FREEZE COMPILER COMMIT'İ TEKİL BEYAN TAŞIMIYOR"));

    let freeze_yolu = kok.join("docs/core-freeze-beyanlari-v1.tsv");
    let freeze = std::fs::read_to_string(&freeze_yolu).expect("freeze manifest");
    std::fs::write(
        &freeze_yolu,
        format!(
            "{freeze}{feature}\tmaintenance\t-\t-\t-\t-\tSemantic davranışın değişmediği iddia edilse de sınıf eşleşmesi aranır.\t-\n"
        ),
    )
    .expect("yanlış maintenance");
    commit(&kok, "yanlış maintenance kaçışı");
    let yanlis = korugu_calistir(&kok, &freeze_parent);
    assert!(!yanlis.status.success());
    assert!(String::from_utf8_lossy(&yanlis.stderr)
        .contains("CORE FREEZE MAINTENANCE BEYANI UYUŞMUYOR"));

    let freeze = std::fs::read_to_string(&freeze_yolu).expect("freeze manifest");
    let yanlis_satir = freeze.lines().last().expect("yanlış satır");
    let dogfood_satiri = format!(
        "{feature}\tdogfood-change\tcatli-itwise-admin\tK-163\tregression/istek.dil\tprojeler/catli/proje.dil\tGerçek ürünün istediği en küçük compiler yüzeyiyle sınırlıdır.\tspec/ozellik.md"
    );
    std::fs::write(
        &freeze_yolu,
        format!(
            "{}\n",
            freeze.replace(yanlis_satir, &dogfood_satiri).trim_end()
        ),
    )
    .expect("dogfood beyanı");
    let dogfood_commit = commit(&kok, "gerçek dogfood provenance");
    assert!(
        korugu_calistir(&kok, &freeze_parent).status.success(),
        "tam dogfood kanıtı geçmeli"
    );

    let freeze = std::fs::read_to_string(&freeze_yolu).expect("freeze manifest");
    std::fs::write(
        &freeze_yolu,
        freeze.replace("Gerçek ürünün istediği", "Başka bir ürünün istediği"),
    )
    .expect("provenance yeniden yazımı");
    commit(&kok, "geçmiş beyanı yeniden yaz");
    let yeniden_yazim = korugu_calistir(&kok, &dogfood_commit);
    assert!(!yeniden_yazim.status.success());
    assert!(String::from_utf8_lossy(&yeniden_yazim.stderr)
        .contains("CORE FREEZE BEYANI YENİDEN YAZILDI"));

    std::fs::remove_dir_all(&kok).expect("geçici depo temizlenmeli");
}
