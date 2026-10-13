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

pub(super) fn kaynak(paket: &KilitPaketi, sha: &str) -> Result<(String, String), String> {
    if let Some(kaynak) = &paket.kaynak {
        if kaynak != "registry+https://github.com/rust-lang/crates.io-index" {
            return Err("SBOM kayıt dışı registry/git kaynağını kabul etmez".into());
        }
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
    let manifest = include_str!("../../../third_party/patches-v1.tsv");
    let mut lines = manifest.lines();
    if lines.next() != Some("# zee-local-dependency-patches-1") {
        return Err("yerel yama şeması bilinmiyor".into());
    }
    let mut matched = None;
    let mut keys = std::collections::BTreeSet::new();
    for line in lines.filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let fields = line.split('\t').collect::<Vec<_>>();
        let [name, version, path, expected, upstream_version, upstream_hash, upstream_sha] =
            fields.as_slice()
        else {
            return Err("yerel yama kaydı yedi alan ister".into());
        };
        if !keys.insert((*name, *version)) {
            return Err("yerel yama kaydı yinelendi".into());
        }
        hex_coz::<32>(expected, "yama tree hash")?;
        hex_coz::<32>(upstream_hash, "upstream crate hash")?;
        hex_coz::<20>(upstream_sha, "upstream Git SHA")?;
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
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .ok_or("compiler parent yok")?
                .join(path);
            let actual = agac_ozeti(&root)?;
            if actual != *expected {
                return Err("yerel yama ağacı kayıtlı SHA-256 ile eşleşmiyor".into());
            }
            matched = Some((format!("{KAYNAK_DEPOSU}/tree/{sha}/{path}"), actual));
        }
    }
    matched.ok_or_else(|| format!("SBOM kayıtsız yerel paket: {} {}", paket.ad, paket.surum))
}
