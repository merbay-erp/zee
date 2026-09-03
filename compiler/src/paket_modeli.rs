//! Paket, yayın ve registry katmanlarının paylaştığı davranışsız veri modeli.

#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaketBilgisi {
    pub ad: String,
    pub surum: String,
    pub morfoloji: String,
    pub yol: String,
    pub ozet: String,
    pub dogrudan: bool,
    pub registry_kok_sha256: Option<String>,
    pub arsiv_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaketCiktilari {
    pub paket: PathBuf,
    pub sbom: PathBuf,
    pub provenance: PathBuf,
    pub yayin: PathBuf,
    pub paket_ozeti: String,
    pub yayinci_anahtar_kimligi: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), serde(deny_unknown_fields))]
pub struct YayinDosyasi {
    pub ad: String,
    pub boyut: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), serde(deny_unknown_fields))]
pub struct ImzaliYayin {
    pub sema: String,
    pub paket: String,
    pub surum: String,
    pub morfoloji: String,
    pub yayinci_anahtari: String,
    pub yayinci_anahtar_kimligi: String,
    pub arsiv: YayinDosyasi,
    pub sbom: YayinDosyasi,
    pub provenance: YayinDosyasi,
}
