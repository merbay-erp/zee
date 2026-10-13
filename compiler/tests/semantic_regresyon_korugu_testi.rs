use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::{Command, Output};
#[cfg(unix)]
use std::time::{SystemTime, UNIX_EPOCH};

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler depo içinde olmalı")
        .to_path_buf()
}

#[cfg(unix)]
fn gecici_depo() -> PathBuf {
    let damga = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("saat")
        .as_nanos();
    std::env::temp_dir().join(format!("zee-semantic-koruk-{}-{damga}", std::process::id()))
}

#[cfg(unix)]
fn git(kok: &Path, argumanlar: &[&str]) -> Output {
    Command::new("git")
        .args(argumanlar)
        .current_dir(kok)
        .output()
        .expect("git çalışmalı")
}

#[cfg(unix)]
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

#[cfg(unix)]
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

#[cfg(unix)]
fn korugu_calistir(kok: &Path, taban: &str) -> Output {
    Command::new("bash")
        .arg(depo().join("scripts/semantic-regresyon-korugu.sh"))
        .arg(taban)
        .current_dir(kok)
        .output()
        .expect("koruk çalışmalı")
}

// K-182: bash betiği (semantic regresyon koruğu) POSIX kabuk ister; CI onu yalnız
// ubuntu'da koşar, Windows Git Bash desteklenen koruk hostu değildir.
#[cfg(unix)]
#[test]
fn commit_mesajindan_bagimsiz_beyan_ve_fixture_zorunludur() {
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
    std::fs::create_dir_all(kok.join("docs")).expect("docs klasörü");
    std::fs::write(
        kok.join("docs/compiler-degisiklik-beyanlari-v1.tsv"),
        format!(
            "# zee-compiler-degisiklik-beyanlari-1\n\
             # enforcement_parent\t{taban}\n\
             # commit\tsinif\tkanit\tgerekce\n"
        ),
    )
    .expect("beyan manifesti yazılmalı");
    commit(&kok, "semantic beyan kapısı");

    std::fs::write(kok.join("compiler/src/lib.rs"), "pub fn duzeltilmis() {}\n")
        .expect("düzeltme yazılmalı");
    let fixed_by = commit(&kok, "anlatımdan bağımsız bir başlık");
    let eksik = korugu_calistir(&kok, &taban);
    assert!(!eksik.status.success(), "fixture'sız bug fix geçmemeli");
    assert!(String::from_utf8_lossy(&eksik.stderr)
        .contains("COMPILER KAYNAK COMMIT'İ TEKİL SEMANTIC BEYAN TAŞIMIYOR"));

    let yeni = format!(
        "yeni\tK-002\t{fixed_by}\t-\t0.8.0-dev\tchecker\trun\t-\t-\t0\t-\tregression/checker/yeni.dil\n"
    );
    let manifest = std::fs::read_to_string(kok.join("regression/v2.tsv")).expect("manifest");
    std::fs::write(kok.join("regression/v2.tsv"), format!("{manifest}{yeni}"))
        .expect("provenance yazılmalı");
    let beyan_yolu = kok.join("docs/compiler-degisiklik-beyanlari-v1.tsv");
    let beyanlar = std::fs::read_to_string(&beyan_yolu).expect("beyan manifesti");
    std::fs::write(
        &beyan_yolu,
        format!(
            "{beyanlar}{fixed_by}\tsemantic-bugfix\tyeni\tİsmi ne olursa olsun bu compiler değişikliği exact fixture gerektirir.\n"
        ),
    )
    .expect("semantic beyan yazılmalı");
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

/// Kabuktan bağımsız sözleşme kanıtı: beyan sınıfları ve fail-closed mesajları
/// her platformda betik metninden doğrulanır (K-182: Windows koruk hostu
/// değildir ama hedef boş kalamaz).
#[test]
fn koruk_betigi_beyan_siniflarini_ve_fail_closed_mesajlarini_kabloluyor() {
    let betik = std::fs::read_to_string(depo().join("scripts/semantic-regresyon-korugu.sh"))
        .expect("koruk betiği okunmalı");
    for kanit in [
        "semantic-bugfix)",
        "semantic-change)",
        "maintenance)",
        "SEMANTIC BUGFIX EXACT FIXTURE PROVENANCE'I TAŞIMIYOR",
        "SEMANTIC DEĞİŞİKLİK NORMATİF KANIT TAŞIMIYOR",
        "COMPILER KAYNAK COMMIT'İ TEKİL SEMANTIC BEYAN TAŞIMIYOR",
        "COMPILER DEĞİŞİKLİK BEYANI GEREKÇESİ YETERSİZ",
    ] {
        assert!(betik.contains(kanit), "koruk betiği kanıtı eksik: {kanit}");
    }
}

#[cfg(unix)]
#[test]
fn iki_kaynak_dalinin_merge_sha_beyani_iki_korukte_de_zorunludur() {
    let kok = gecici_depo();
    for dizin in ["regression", "compiler/src", "docs"] {
        std::fs::create_dir_all(kok.join(dizin)).unwrap();
    }
    basarili_git(&kok, &["init", "-q", "-b", "ana"]);
    std::fs::write(
        kok.join("regression/v2.tsv"),
        "# zee-semantic-regresyon-2\n",
    )
    .unwrap();
    std::fs::write(kok.join("compiler/src/lib.rs"), "// taban\n").unwrap();
    let taban = commit(&kok, "taban");
    let semantic =
        format!("# zee-compiler-degisiklik-beyanlari-1\n# enforcement_parent\t{taban}\n");
    let freeze = format!("# zee-core-freeze-beyanlari-1\n# enforcement_parent\t{taban}\n");
    std::fs::write(
        kok.join("docs/compiler-degisiklik-beyanlari-v1.tsv"),
        &semantic,
    )
    .unwrap();
    std::fs::write(kok.join("docs/core-freeze-beyanlari-v1.tsv"), &freeze).unwrap();
    std::fs::write(
        kok.join("docs/dogfood-projeleri-v1.tsv"),
        "# zee-dogfood-projeleri-1\n",
    )
    .unwrap();
    commit(&kok, "kapılar");
    basarili_git(&kok, &["checkout", "-q", "-b", "urun"]);
    std::fs::write(kok.join("compiler/src/urun.rs"), "// ürün açıklaması\n").unwrap();
    // grep -q pipe'ı erken kapatırsa pipefail gerçek Git çıktısında SIGPIPE
    // üretir. Küçük sentetik merge bu hatayı göstermediğinden boruyu doldur.
    for sira in 0..1024 {
        std::fs::write(
            kok.join("docs")
                .join(format!("buyuk-fark-{sira:04}-{}.md", "a".repeat(96))),
            "Kaynakla birlikte incelenen belge.\n",
        )
        .unwrap();
    }
    let urun = commit(&kok, "ürün açıklaması");
    basarili_git(&kok, &["checkout", "-q", "ana"]);
    std::fs::write(kok.join("compiler/src/lib.rs"), "// ana dal açıklaması\n").unwrap();
    let ana = commit(&kok, "ana açıklama");
    basarili_git(&kok, &["merge", "--no-commit", "--no-ff", "urun"]);
    let birlesim = commit(&kok, "iki kaynak dalını birleştir");
    let beyan_yaz = |merge_var: bool| {
        let sha_lar = if merge_var {
            vec![&urun, &ana, &birlesim]
        } else {
            vec![&urun, &ana]
        };
        let gerekce =
            "Yalnız kaynak yorumları birleşir; executable davranış, grammar ve tanı değişmez.";
        let mut s = semantic.clone();
        let mut f = freeze.clone();
        for sha in sha_lar {
            s.push_str(&format!("{sha}\tmaintenance\t-\t{gerekce}\n"));
            f.push_str(&format!("{sha}\tmaintenance\t-\t-\t-\t-\t{gerekce}\t-\n"));
        }
        std::fs::write(kok.join("docs/compiler-degisiklik-beyanlari-v1.tsv"), s).unwrap();
        std::fs::write(kok.join("docs/core-freeze-beyanlari-v1.tsv"), f).unwrap();
    };
    beyan_yaz(false);
    for betik in ["semantic-regresyon-korugu.sh", "core-freeze-korugu.sh"] {
        let calistir = || {
            Command::new("bash")
                .arg(depo().join("scripts").join(betik))
                .arg(&taban)
                .current_dir(&kok)
                .output()
                .unwrap()
        };
        let eksik = calistir();
        assert!(
            !eksik.status.success(),
            "merge SHA beyansız geçemez: {betik}"
        );
        beyan_yaz(true);
        let tam = calistir();
        assert!(
            tam.status.success(),
            "{betik}: {}",
            String::from_utf8_lossy(&tam.stderr)
        );
        beyan_yaz(false);
    }
    std::fs::remove_dir_all(kok).unwrap();
}
