//! K-228 CLI plan options are checked before any DB connection.
use std::process::Command;

#[test]
fn goc_plan_kipleri_birlikte_tekrarla_ve_fazla_yolla_reddedilir() {
    for args in [
        vec!["göçür", "--durum", "--kuru"],
        vec!["göçür", "--durum", "--durum"],
        vec!["gocur", "--kuru", "--kuru"],
        vec!["göçür", "--kuru", "bir", "iki"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_dil"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8(out.stderr)
            .unwrap()
            .contains("kipler tekil ve ayrı"));
    }
}

#[test]
fn goc_plan_secimi_yol_olmayan_secenek_gibi_islenir() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dogfood/kanit-ozeti");
    for flag in ["--durum", "--kuru"] {
        for once in [true, false] {
            let mut c = Command::new(env!("CARGO_BIN_EXE_dil"));
            c.arg("göçür");
            if once {
                c.arg(flag).arg(&root);
            } else {
                c.arg(&root).arg(flag);
            }
            let out = c.output().unwrap();
            assert_eq!(out.status.code(), Some(2));
            let error = String::from_utf8(out.stderr).unwrap();
            assert!(error.contains("P015"), "{error}");
            assert!(!error.contains("Bilinmeyen seçenek"));
            assert!(!error.contains("dosyası okunamadı"));
        }
    }
}
