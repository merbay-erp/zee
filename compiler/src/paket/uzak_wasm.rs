//! WebAssembly derlemesinde registry transportu yoktur. Ortak proje grafiği
//! yerel paketler için çalışır; uzak bildirim boş çözümle fail-closed kalır.

use crate::proje::{RegistryBildirimi, UzakBagimlilik};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryCozumPolitikasi {
    pub cevrimdisi: bool,
    pub yanked_kabul_gerekcesi: Option<String>,
    pub kritik_duyuru_kabul_gerekcesi: Option<String>,
}

impl Default for RegistryCozumPolitikasi {
    fn default() -> Self {
        Self {
            cevrimdisi: true,
            yanked_kabul_gerekcesi: None,
            kritik_duyuru_kabul_gerekcesi: None,
        }
    }
}

impl RegistryCozumPolitikasi {
    pub fn yeni(
        cevrimdisi: bool,
        yanked_kabul_gerekcesi: Option<String>,
        kritik_duyuru_kabul_gerekcesi: Option<String>,
    ) -> Result<Self, String> {
        for (ad, gerekce) in [
            ("yanked kabul", yanked_kabul_gerekcesi.as_deref()),
            (
                "kritik duyuru kabul",
                kritik_duyuru_kabul_gerekcesi.as_deref(),
            ),
        ] {
            if gerekce.is_some_and(|gerekce| {
                gerekce.trim().is_empty()
                    || gerekce.len() > 1024
                    || gerekce.chars().any(char::is_control)
            }) {
                return Err(format!(
                    "{} gerekçesi boş, denetim karakterli veya 1024 byte'tan uzun olamaz.",
                    ad
                ));
            }
        }
        Ok(Self {
            cevrimdisi,
            yanked_kabul_gerekcesi,
            kritik_duyuru_kabul_gerekcesi,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct UzakPaketKimligi {
    registry: RegistryBildirimi,
    bagimlilik: UzakBagimlilik,
}

#[derive(Debug, Clone)]
pub(super) struct WasmMetadataSurumleri {
    pub root: u64,
    pub timestamp: u64,
    pub timestamp_sha256: String,
    pub snapshot: u64,
    pub snapshot_sha256: String,
    pub targets: u64,
    pub targets_sha256: String,
}

#[derive(Debug, Clone)]
pub(super) struct UzakPaketKilidi {
    pub registry_kok_surumu: u64,
    pub registry_kok_sha256: String,
    pub metadata: WasmMetadataSurumleri,
    pub yayinci_anahtar_kimligi: String,
    pub arsiv_sha256: String,
    pub sbom_sha256: String,
    pub provenance_sha256: String,
    pub yayin_sha256: String,
    pub yanked: bool,
    pub kritik_duyurular: Vec<String>,
    pub yanked_kabul_gerekcesi: Option<String>,
    pub kritik_duyuru_kabul_gerekcesi: Option<String>,
    pub yanked_politika_anahtari: String,
    pub kritik_politika_anahtari: String,
}

#[derive(Debug, Clone)]
pub(super) struct UzakPaketCozumu {
    pub kok: PathBuf,
    pub kilit: UzakPaketKilidi,
}

pub(super) type UzakPaketCozumleri = BTreeMap<UzakPaketKimligi, UzakPaketCozumu>;

impl UzakPaketKimligi {
    pub(super) fn yeni(registry: &RegistryBildirimi, bagimlilik: &UzakBagimlilik) -> Self {
        Self {
            registry: registry.clone(),
            bagimlilik: bagimlilik.clone(),
        }
    }
}

pub(super) fn paketleri_hazirla(
    _ana_kok: &Path,
    _ana_bildirim_kaynagi: Option<&str>,
    _politika: &RegistryCozumPolitikasi,
) -> Result<UzakPaketCozumleri, super::ProjeYuklemeHatasi> {
    // Uzak bildirim varsa GrafikKurucu exact çözümü bulamaz ve P016 üretir.
    // Böylece playground/WASM hiçbir ağ veya doğrulanmamış fallback açmaz.
    Ok(BTreeMap::new())
}
