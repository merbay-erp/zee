//! İçerik-adresli nesne deposunun yol, özet ve sınırlı okuma ilkelleri.

use super::super::RegistryHatasi;
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(super) fn nesne_yolu(cache: &Path, ozet: &str) -> Result<PathBuf, RegistryHatasi> {
    let ozet = sha256_onunu_ayir(ozet, "cache özeti")?;
    Ok(cache.join("nesneler").join("sha256").join(ozet))
}

pub(super) fn sha256_onunu_ayir<'a>(ozet: &'a str, ad: &str) -> Result<&'a str, RegistryHatasi> {
    let ham = ozet.strip_prefix("sha256:").unwrap_or(ozet);
    if ham.len() != 64
        || !ham
            .bytes()
            .all(|bayt| bayt.is_ascii_digit() || (b'a'..=b'f').contains(&bayt))
    {
        return Err(RegistryHatasi::tasima(format!(
            "{} 32-byte küçük harfli SHA-256 değil.",
            ad
        )));
    }
    Ok(ham)
}

pub(super) fn sinirli_dosya_oku(
    yol: &Path,
    azami: usize,
    kesin_boyut: Option<u64>,
    ad: &str,
) -> Result<Vec<u8>, RegistryHatasi> {
    let dosya = std::fs::File::open(yol).map_err(|hata| {
        RegistryHatasi::tasima(format!("{} \"{}\" açılamadı: {}.", ad, yol.display(), hata))
    })?;
    let metadata = dosya.metadata().map_err(|hata| {
        RegistryHatasi::tasima(format!("{} metadata'sı okunamadı: {}.", ad, hata))
    })?;
    if !metadata.is_file()
        || metadata.len() > azami as u64
        || kesin_boyut.is_some_and(|boyut| boyut != metadata.len())
    {
        return Err(RegistryHatasi::tasima(format!(
            "{} cache nesnesi dosya/boyut sınırıyla uyuşmuyor.",
            ad
        )));
    }
    let mut baytlar = Vec::with_capacity(metadata.len() as usize);
    dosya
        .take(azami.saturating_add(1) as u64)
        .read_to_end(&mut baytlar)
        .map_err(|hata| RegistryHatasi::tasima(format!("{} okunamadı: {}.", ad, hata)))?;
    if baytlar.len() > azami || kesin_boyut.is_some_and(|boyut| boyut != baytlar.len() as u64) {
        return Err(RegistryHatasi::tasima(format!(
            "{} okunurken boyut sınırı değişti.",
            ad
        )));
    }
    Ok(baytlar)
}

pub(super) fn kanonik_json<T: Serialize>(deger: &T, ad: &str) -> Result<Vec<u8>, RegistryHatasi> {
    let mut baytlar = serde_json::to_vec_pretty(deger)
        .map_err(|hata| RegistryHatasi::tasima(format!("{} JSON üretilemedi: {}.", ad, hata)))?;
    baytlar.push(b'\n');
    Ok(baytlar)
}
