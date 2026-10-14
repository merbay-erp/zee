//! K-226: deterministic source identity, never wall-clock build time.
use std::env;
use std::path::Path;
use std::process::Command;

fn git(kok: &Path, args: &[&str]) -> Option<String> {
    let cikti = Command::new("git")
        .args(args)
        .current_dir(kok)
        .output()
        .ok()?;
    cikti
        .status
        .success()
        .then(|| String::from_utf8_lossy(&cikti.stdout).trim().to_owned())
}

fn tarih(epoch: u64) -> String {
    let z = (epoch / 86_400) as i64 + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn main() {
    for alan in ["ZEE_SOURCE_SHA", "SOURCE_DATE_EPOCH", "ZEE_RELEASE_CHANNEL"] {
        println!("cargo:rerun-if-env-changed={alan}");
    }
    for yol in [
        "src",
        "build.rs",
        "Cargo.toml",
        "Cargo.lock",
        "third_party",
        "../kitaplik",
        "../docs/hata-katalogu.md",
    ] {
        println!("cargo:rerun-if-changed={yol}");
    }
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest directory");
    let kok = Path::new(&manifest)
        .parent()
        .expect("compiler repository root");
    for yol in ["HEAD", "refs", "packed-refs", "index"] {
        if let Some(yol) = git(kok, &["rev-parse", "--git-path", yol]) {
            println!("cargo:rerun-if-changed={}", kok.join(yol).display());
        }
    }
    let head = git(kok, &["rev-parse", "--verify", "HEAD"]);
    let sha = env::var("ZEE_SOURCE_SHA").ok().or_else(|| head.clone());
    if let Some(sha) = &sha {
        assert!(
            sha.len() == 40
                && sha
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "source SHA must be 40 lowercase hex characters"
        );
        if let Some(head) = &head {
            assert_eq!(sha, head, "declared source SHA differs from checkout HEAD");
        }
    }
    let commit_epoch = git(kok, &["log", "-1", "--format=%ct"]);
    let epoch = env::var("SOURCE_DATE_EPOCH")
        .ok()
        .or_else(|| commit_epoch.clone());
    if let (Some(git_epoch), Some(epoch)) = (&commit_epoch, &epoch) {
        assert_eq!(
            git_epoch, epoch,
            "SOURCE_DATE_EPOCH must equal exact source commit time"
        );
    }
    let date = epoch
        .map(|s| {
            let epoch = s
                .parse::<u64>()
                .expect("source epoch must be unsigned seconds");
            assert!(epoch <= 253_402_300_799, "source date exceeds year 9999");
            tarih(epoch)
        })
        .unwrap_or_else(|| "unknown".into());
    let version = env::var("CARGO_PKG_VERSION").expect("Cargo version");
    let channel = env::var("ZEE_RELEASE_CHANNEL").unwrap_or_else(|_| {
        if version.ends_with("-dev") {
            "development"
        } else {
            "beta"
        }
        .into()
    });
    assert!(
        ["development", "beta", "nightly", "stable"].contains(&channel.as_str()),
        "unsupported release channel"
    );
    assert!(
        channel != "stable" || !version.ends_with("-dev"),
        "development series cannot claim stable channel"
    );
    let dirty = git(
        kok,
        &["status", "--porcelain", "--untracked-files=all", "--", "."],
    )
    .is_some_and(|s| !s.is_empty());
    println!(
        "cargo:rustc-env=ZEE_BUILD_SHA={}",
        sha.unwrap_or_else(|| "unknown".into())
    );
    println!("cargo:rustc-env=ZEE_BUILD_DATE={date}");
    println!("cargo:rustc-env=ZEE_BUILD_CHANNEL={channel}");
    println!("cargo:rustc-env=ZEE_BUILD_DIRTY={dirty}");
}
