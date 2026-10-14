//! K-242: native kampanyanın yanlış/atlanan kanıtları ve gerçek wire proxy retleri.
use std::path::Path;
use std::process::Command;

#[test]
fn parola_goc_kapisi_ve_commit_proxy_negatifleri_atlanamaz() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    for (dosya, vaka_sayisi) in [
        ("scripts/tests/parola-goc-ci-test.py", 13),
        ("scripts/tests/postgresql-commit-proxy-test.py", 10),
    ] {
        let sonuc = Command::new("python3")
            .arg("-I")
            .arg(depo.join(dosya))
            .current_dir(depo)
            .output()
            .expect("Python3 strict kampanya testi çalışmalı");
        assert!(
            sonuc.status.success(),
            "{dosya}: {}{}",
            String::from_utf8_lossy(&sonuc.stdout),
            String::from_utf8_lossy(&sonuc.stderr)
        );
        let rapor = String::from_utf8_lossy(&sonuc.stderr);
        assert!(
            rapor.contains(&format!("\nRan {vaka_sayisi} tests in "))
                && rapor.trim_end().ends_with("\nOK"),
            "{dosya}: bütün vakalar atlamasız yürütülmeli: {rapor}"
        );
    }
}
