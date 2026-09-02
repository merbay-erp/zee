//! Yanked ve kritik duyuru kabul gerekçelerini exact güvenlik olayına bağlar.

use crate::guvenlik::sha256_hex;
use crate::paket::{proje_hatasi, ProjeYuklemeHatasi, KILIT_DOSYASI};
use crate::proje::{RegistryBildirimi, UzakBagimlilik};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub(super) struct KabulGerekceleri {
    pub yanked: Option<String>,
    pub kritik: Option<String>,
}

pub(super) fn politika_anahtari(
    registry: &RegistryBildirimi,
    bagimlilik: &UzakBagimlilik,
) -> String {
    format!(
        "{}|{}@{}",
        registry.kok_sha256, bagimlilik.ad, bagimlilik.surum
    )
}

pub(super) fn yanked_politika_anahtari(temel: &str) -> String {
    format!("{}|yanked", temel)
}

pub(super) fn kritik_politika_on_eki(temel: &str) -> String {
    format!("{}|kritik:", temel)
}

pub(super) fn kritik_politika_anahtari(temel: &str, duyurular: &[String]) -> String {
    let duyurular = duyurular.iter().collect::<BTreeSet<_>>();
    let mut kanonik = Vec::new();
    for duyuru in duyurular {
        kanonik.extend_from_slice(&(duyuru.len() as u64).to_be_bytes());
        kanonik.extend_from_slice(duyuru.as_bytes());
    }
    format!(
        "{}sha256:{}",
        kritik_politika_on_eki(temel),
        sha256_hex(&kanonik)
    )
}

pub(super) fn kilit_politikalarini_oku(
    ana_kok: &Path,
    kaynak: &str,
) -> Result<BTreeMap<String, KabulGerekceleri>, ProjeYuklemeHatasi> {
    let kilit_yolu = ana_kok.join(KILIT_DOSYASI);
    let kilit = match crate::kaynak_sinirlari::kaynak_dosyasi_oku(&kilit_yolu) {
        Ok(kilit) => kilit,
        Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(hata) => {
            return Err(proje_hatasi(
                "P008",
                &format!("proje.kilit politika kayıtları okunamadı: {}.", hata),
                "Kilit dosyasını inceleyip `dil kilitle .` ile yeniden üret.",
                ana_kok.join("proje.dil"),
                kaynak.to_string(),
            ));
        }
    };
    let mut sonuc: BTreeMap<String, KabulGerekceleri> = BTreeMap::new();
    for satir in kilit.lines() {
        let (tur, kalan) = if let Some(kalan) = satir.strip_prefix("yanked_kabul ") {
            ("yanked", kalan)
        } else if let Some(kalan) = satir.strip_prefix("kritik_kabul ") {
            ("kritik", kalan)
        } else {
            continue;
        };
        let alanlar = tirnakli_alanlari_oku(kalan).map_err(|mesaj| {
            proje_hatasi(
                "P008",
                &format!("proje.kilit politika kaydı bozuk: {}", mesaj),
                "Kilit dosyasını `dil kilitle .` ile yeniden üret.",
                kilit_yolu.clone(),
                kilit.clone(),
            )
        })?;
        if alanlar.len() != 2 {
            return Err(proje_hatasi(
                "P008",
                "proje.kilit politika kaydı tam iki Metin alanı taşımalı.",
                "Kilit dosyasını `dil kilitle .` ile yeniden üret.",
                kilit_yolu,
                kilit,
            ));
        }
        if alanlar[1].trim().is_empty()
            || alanlar[1].len() > 1024
            || alanlar[1].chars().any(char::is_control)
        {
            return Err(proje_hatasi(
                "P008",
                "proje.kilit politika gerekçesi boş, denetim karakterli veya çok uzun.",
                "Kabul kararını açık ve kısa bir gerekçeyle yeniden kilitle.",
                ana_kok.join(KILIT_DOSYASI),
                kilit,
            ));
        }
        let kayit = sonuc.entry(alanlar[0].clone()).or_default();
        let hedef = if tur == "yanked" {
            &mut kayit.yanked
        } else {
            &mut kayit.kritik
        };
        if hedef.replace(alanlar[1].clone()).is_some() {
            return Err(proje_hatasi(
                "P008",
                "proje.kilit aynı paket için yinelenen politika kaydı taşıyor.",
                "Kilit dosyasını `dil kilitle .` ile yeniden üret.",
                ana_kok.join(KILIT_DOSYASI),
                String::new(),
            ));
        }
    }
    Ok(sonuc)
}

fn tirnakli_alanlari_oku(metin: &str) -> Result<Vec<String>, String> {
    let mut sonuc = Vec::new();
    let mut karakterler = metin.chars().peekable();
    loop {
        while karakterler.peek().is_some_and(|k| k.is_whitespace()) {
            karakterler.next();
        }
        if karakterler.peek().is_none() {
            return Ok(sonuc);
        }
        if karakterler.next() != Some('"') {
            return Err("alan çift tırnakla başlamıyor.".into());
        }
        let mut alan = String::new();
        let mut kapandi = false;
        while let Some(k) = karakterler.next() {
            match k {
                '"' => {
                    kapandi = true;
                    break;
                }
                '\\' => match karakterler.next() {
                    Some('"') => alan.push('"'),
                    Some('\\') => alan.push('\\'),
                    Some('n') => alan.push('\n'),
                    _ => return Err("bilinmeyen kilit kaçışı.".into()),
                },
                k => alan.push(k),
            }
        }
        if !kapandi {
            return Err("alanın kapanan çift tırnağı yok.".into());
        }
        sonuc.push(alan);
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn kritik_kabul_anahtari_duyuru_kumesine_baglidir() {
        let temel = "sha256:kök|miras@1.2.3";
        let onceki = kritik_politika_anahtari(temel, &["ZEE-1".into()]);
        let ayni = kritik_politika_anahtari(temel, &["ZEE-1".into(), "ZEE-1".into()]);
        let yeni = kritik_politika_anahtari(temel, &["ZEE-1".into(), "ZEE-2".into()]);
        assert_eq!(onceki, ayni);
        assert_ne!(onceki, yeni);
        assert_ne!(yanked_politika_anahtari(temel), onceki);
    }
}
