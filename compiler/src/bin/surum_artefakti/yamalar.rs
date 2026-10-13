//! K-206 local patch identity: verified tree bytes, not an invented registry checksum.
use super::{hex_coz, hex_yaz, KilitPaketi, KAYNAK_DEPOSU};
use sha2::{Digest, Sha256};
use std::path::{Component, Path};

fn dosyalar(kok: &Path, yol: &Path, sonuc: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
    let entries = std::fs::read_dir(yol).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let tur = entry.file_type().map_err(|e| e.to_string())?;
        if tur.is_symlink() {
            return Err("yerel yama symlink taşıyamaz".into());
        }
        if tur.is_dir() {
            dosyalar(kok, &entry.path(), sonuc)?;
        } else if tur.is_file() {
            let p = entry.path();
            let rel = p
                .strip_prefix(kok)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("yama yolu UTF-8 değil")?
                .replace('\\', "/");
            sonuc.push((rel, std::fs::read(p).map_err(|e| e.to_string())?));
        } else {
            return Err("yerel yama normal dosya/dizin olmalı".into());
        }
    }
    Ok(())
}
pub(super) fn agac_ozeti(kok: &Path) -> Result<String, String> {
    if std::fs::symlink_metadata(kok)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("yama kökü symlink olamaz".into());
    }
    let mut files = Vec::new();
    dosyalar(kok, kok, &mut files)?;
    kanonik_ozet(files)
}

fn kanonik_ozet(mut files: Vec<(String, Vec<u8>)>) -> Result<String, String> {
    files.sort_by(|a, b| a.0.cmp(&b.0));
    if files.is_empty() {
        return Err("yerel yama ağacı boş".into());
    }
    let mut hash = Sha256::new();
    for (path, bytes) in files {
        hash.update((path.len() as u64).to_be_bytes());
        hash.update(path.as_bytes());
        hash.update((bytes.len() as u64).to_be_bytes());
        hash.update(&bytes);
    }
    Ok(hex_yaz(&hash.finalize()))
}

fn git_agaci(repo: &Path, sha: &str, path: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-tree", "-r", "-z", "--full-tree", sha, "--", path])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("yama Git commit/tree okunamadı; full history gerekli".into());
    }
    let mut files = Vec::new();
    for row in output.stdout.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let row = std::str::from_utf8(row).map_err(|_| "Git yama yolu UTF-8 değil")?;
        let (meta, name) = row.split_once('\t').ok_or("Git tree kaydı geçersiz")?;
        let meta = meta.split(' ').collect::<Vec<_>>();
        let [mode, "blob", id] = meta.as_slice() else {
            return Err("Git yama kaydı normal blob değil".into());
        };
        if !matches!(*mode, "100644" | "100755") {
            return Err("Git yama symlink/özel dosya taşıyamaz".into());
        }
        hex_coz::<20>(id, "Git blob SHA")?;
        let rel = name
            .strip_prefix(&format!("{path}/"))
            .ok_or("Git yama kökü uyuşmuyor")?;
        let bytes = std::process::Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["cat-file", "blob", id])
            .output()
            .map_err(|e| e.to_string())?;
        if !bytes.status.success() {
            return Err("Git yama blob okunamadı".into());
        }
        files.push((rel.to_string(), bytes.stdout));
    }
    kanonik_ozet(files)
}

pub(super) fn kaynak(paket: &KilitPaketi, _sha: &str) -> Result<(String, String), String> {
    if paket.kaynak.as_deref() == Some("registry+https://github.com/rust-lang/crates.io-index") {
        let ozet = paket.ozet.as_ref().ok_or("registry paket checksum eksik")?;
        hex_coz::<32>(ozet, "registry checksum")?;
        return Ok((
            format!("https://crates.io/crates/{}/{}", paket.ad, paket.surum),
            ozet.clone(),
        ));
    }
    if paket.ozet.is_some() {
        return Err("yerel paket registry checksum taşıyamaz".into());
    }
    let manifest = include_str!("../../../third_party/patches-v2.tsv");
    let mut lines = manifest.lines();
    if lines.next() != Some("# zee-local-dependency-patches-2") {
        return Err("yerel yama şeması bilinmiyor".into());
    }
    let mut matched = None;
    let mut keys = std::collections::BTreeSet::new();
    for line in lines.filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let fields = line.split('\t').collect::<Vec<_>>();
        let [name, version, path, expected, upstream_version, upstream_hash, upstream_sha, source_sha] =
            fields.as_slice()
        else {
            return Err("yerel yama kaydı sekiz alan ister".into());
        };
        if !keys.insert((*name, *version)) {
            return Err("yerel yama kaydı yinelendi".into());
        }
        hex_coz::<32>(expected, "yama tree hash")?;
        hex_coz::<32>(upstream_hash, "upstream crate hash")?;
        hex_coz::<20>(upstream_sha, "upstream Git SHA")?;
        hex_coz::<20>(source_sha, "Zee dependency Git SHA")?;
        let relative = Path::new(path);
        if !path.starts_with("compiler/third_party/")
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || upstream_version.is_empty()
        {
            return Err("yerel yama yolu/girdi kaydı geçersiz".into());
        }
        if *name == paket.ad && *version == paket.surum {
            let expected_source = format!("git+{KAYNAK_DEPOSU}?rev={source_sha}#{source_sha}");
            if paket.kaynak.as_deref() != Some(expected_source.as_str()) {
                return Err("yerel yama exact kayıtlı Git URL/rev/commit kaynağını ister".into());
            }
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .ok_or("compiler parent yok")?
                .join(path);
            let actual = agac_ozeti(&root)?;
            let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .ok_or("compiler parent yok")?;
            if git_agaci(repo, source_sha, path)? != actual {
                return Err(
                    "yerel yama dosyaları seçili Git dependency commit ağacıyla uyuşmuyor".into(),
                );
            }
            if actual != *expected {
                return Err("yerel yama ağacı kayıtlı SHA-256 ile eşleşmiyor".into());
            }
            matched = Some((format!("{KAYNAK_DEPOSU}/tree/{source_sha}/{path}"), actual));
        }
    }
    matched.ok_or_else(|| format!("SBOM kayıtsız yerel paket: {} {}", paket.ad, paket.surum))
}
