//! Uzak zee registry metadata güven zinciri.
//!
//! Doğrulayıcı ağdan ya da diskten gelen sınırlı byte dizilerini çevrimdışı
//! kök güveninden başlayarak denetler; istemci katmanıysa HTTPS/statik taşıma,
//! içerik-adresli cache ve kalıcı monoton durumu bu çekirdeğe bağlar.

mod istemci;

pub use istemci::{
    HttpsRegistryTasiyici, RegistryIstemcisi, RegistryPaketCiktisi, RegistryTasiyici,
};

use crate::guvenlik::sha256_hex;
use crate::tedarik::{yayini_dogrula, ImzaliYayin, YayinDosyasi};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const IMZA_ALANI: &[u8] = b"zee-registry-v1\0";
const AZAMI_KOK_BOYUTU: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .kok_bayti();
const AZAMI_TIMESTAMP_BOYUTU: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .timestamp_bayti();
const AZAMI_SNAPSHOT_BOYUTU: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .snapshot_bayti();
const AZAMI_TARGETS_BOYUTU: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .targets_bayti();
const AZAMI_ANAHTAR_SAYISI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .anahtar_sayisi();
const AZAMI_IMZA_SAYISI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .imza_sayisi();
const AZAMI_HEDEF_SAYISI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .hedef_sayisi();
const AZAMI_DUYURU_SAYISI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .duyuru_sayisi();
const AZAMI_ARSIV_BOYUTU: u64 = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .arsiv_bayti();
const AZAMI_SBOM_BOYUTU: u64 = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .sbom_bayti();
const AZAMI_PROVENANCE_BOYUTU: u64 = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .provenance_bayti();
const AZAMI_YAYIN_BOYUTU: u64 = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .registry()
    .yayin_bayti();
const ROLLER: [&str; 4] = ["root", "snapshot", "targets", "timestamp"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryHatasi {
    pub kod: &'static str,
    pub mesaj: String,
}

impl RegistryHatasi {
    fn metadata(mesaj: impl Into<String>) -> Self {
        Self {
            kod: "P013",
            mesaj: mesaj.into(),
        }
    }

    fn hedef(mesaj: impl Into<String>) -> Self {
        Self {
            kod: "P014",
            mesaj: mesaj.into(),
        }
    }

    fn tasima(mesaj: impl Into<String>) -> Self {
        Self {
            kod: "P016",
            mesaj: mesaj.into(),
        }
    }
}

impl fmt::Display for RegistryHatasi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kod, self.mesaj)
    }
}

impl std::error::Error for RegistryHatasi {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryAnahtari {
    pub tur: String,
    pub sema: String,
    pub acik: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolYetkisi {
    pub anahtar_kimlikleri: Vec<String>,
    pub esik: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KokMetadata {
    pub sema: String,
    pub surum: u64,
    pub sona_erme: String,
    pub tutarli_anlik: bool,
    pub anahtarlar: BTreeMap<String, RegistryAnahtari>,
    pub roller: BTreeMap<String, RolYetkisi>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataDosyasi {
    pub surum: u64,
    pub boyut: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampMetadata {
    pub sema: String,
    pub surum: u64,
    pub sona_erme: String,
    pub snapshot: MetadataDosyasi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotMetadata {
    pub sema: String,
    pub surum: u64,
    pub sona_erme: String,
    pub targets: MetadataDosyasi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DuyuruMetadata {
    pub kimlik: String,
    pub paket: String,
    pub etkilenen_surumler: Vec<String>,
    pub onem: String,
    pub duzeltilen_surum: Option<String>,
    pub etkin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HedefMetadata {
    pub paket: String,
    pub surum: String,
    pub morfoloji: String,
    pub yayinci_anahtar_kimligi: String,
    pub yanked: bool,
    pub duyurular: Vec<String>,
    pub arsiv: YayinDosyasi,
    pub sbom: YayinDosyasi,
    pub provenance: YayinDosyasi,
    pub yayin: YayinDosyasi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetsMetadata {
    pub sema: String,
    pub surum: u64,
    pub sona_erme: String,
    pub hedefler: BTreeMap<String, HedefMetadata>,
    pub duyurular: BTreeMap<String, DuyuruMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryImzasi {
    anahtar_kimligi: String,
    ed25519: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataZarfi<T> {
    imzali: T,
    imzalar: Vec<RegistryImzasi>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataSurumleri {
    pub root: u64,
    pub timestamp: u64,
    pub timestamp_sha256: String,
    pub snapshot: u64,
    pub snapshot_sha256: String,
    pub targets: u64,
    pub targets_sha256: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HedefPolitikasi {
    pub yanked_kabul: bool,
    pub kritik_duyuru_kabul: bool,
}

#[derive(Debug, Clone)]
pub struct DogrulanmisRegistry {
    targets: TargetsMetadata,
    surumler: MetadataSurumleri,
}

impl DogrulanmisRegistry {
    pub fn surumler(&self) -> MetadataSurumleri {
        self.surumler.clone()
    }

    pub fn targets(&self) -> &TargetsMetadata {
        &self.targets
    }

    pub fn hedef_sec(
        &self,
        paket: &str,
        surum: &str,
        politika: HedefPolitikasi,
    ) -> Result<&HedefMetadata, RegistryHatasi> {
        let kimlik = format!("{}@{}", paket, surum);
        let hedef = self.targets.hedefler.get(&kimlik).ok_or_else(|| {
            RegistryHatasi::hedef(format!("Registry hedefi bulunamadı: {}.", kimlik))
        })?;
        if hedef.yanked && !politika.yanked_kabul {
            return Err(RegistryHatasi::hedef(format!(
                "{} geri çekilmiş (yanked); yeni kilit varsayılan olarak reddedildi.",
                kimlik
            )));
        }
        for duyuru_kimligi in &hedef.duyurular {
            let duyuru = &self.targets.duyurular[duyuru_kimligi];
            if duyuru.etkin && duyuru.onem == "kritik" && !politika.kritik_duyuru_kabul {
                return Err(RegistryHatasi::hedef(format!(
                    "{} etkin kritik güvenlik duyurusundan etkileniyor: {}.",
                    kimlik, duyuru.kimlik
                )));
            }
        }
        Ok(hedef)
    }

    fn etkin_kritik_duyurular(&self, hedef: &HedefMetadata) -> Vec<String> {
        hedef
            .duyurular
            .iter()
            .filter(|kimlik| {
                self.targets
                    .duyurular
                    .get(*kimlik)
                    .is_some_and(|duyuru| duyuru.etkin && duyuru.onem == "kritik")
            })
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct RegistryDogrulayici {
    kok: KokMetadata,
    durum: MetadataSurumleri,
}

impl RegistryDogrulayici {
    /// İlk root byte'larını ağ dışından gelen SHA-256 sabitlemesiyle açar.
    pub fn sabitlenmis_kok(
        root: &[u8],
        sabit_sha256: &str,
        guncelleme_baslangici: i64,
    ) -> Result<Self, RegistryHatasi> {
        Self::sabitlenmis_kok_durumla(
            root,
            sabit_sha256,
            guncelleme_baslangici,
            MetadataSurumleri::default(),
        )
    }

    /// Kalıcı sürüm durumu bulunan istemci açılışı. Root sürümü diskteki
    /// güvenilen root ile birebir aynı olmalıdır.
    pub fn sabitlenmis_kok_durumla(
        root: &[u8],
        sabit_sha256: &str,
        guncelleme_baslangici: i64,
        mut durum: MetadataSurumleri,
    ) -> Result<Self, RegistryHatasi> {
        zaman_girdisini_dogrula(guncelleme_baslangici)?;
        durum_yapisini_dogrula(&durum)?;
        ozet_bicimini_dogrula(sabit_sha256, "sabit root özeti")?;
        if sabit_sha256 != format!("sha256:{}", sha256_hex(root)) {
            return Err(RegistryHatasi::metadata(
                "Root metadata ağ dışından sabitlenen SHA-256 özetiyle uyuşmuyor.",
            ));
        }
        let zarf: MetadataZarfi<KokMetadata> = zarfi_oku(root, AZAMI_KOK_BOYUTU, "root")?;
        kok_yapisini_dogrula(&zarf.imzali)?;
        rolu_dogrula(&zarf, "root", &zarf.imzali)?;
        sona_ermeyi_dogrula(&zarf.imzali.sona_erme, guncelleme_baslangici, "root")?;
        if durum.root != 0 && durum.root != zarf.imzali.surum {
            return Err(RegistryHatasi::metadata(format!(
                "Kalıcı root sürümü {} ile yüklenen root sürümü {} uyuşmuyor.",
                durum.root, zarf.imzali.surum
            )));
        }
        durum.root = zarf.imzali.surum;
        Ok(Self {
            kok: zarf.imzali,
            durum,
        })
    }

    pub fn kok(&self) -> &KokMetadata {
        &self.kok
    }

    pub fn durum(&self) -> MetadataSurumleri {
        self.durum.clone()
    }

    /// Root rotasyonu ardışıktır ve aynı yeni zarf hem eski hem yeni root
    /// eşiğini sağlamadan güven durumunu değiştirmez.
    pub fn kok_dondur(
        &mut self,
        yeni_root: &[u8],
        guncelleme_baslangici: i64,
    ) -> Result<(), RegistryHatasi> {
        zaman_girdisini_dogrula(guncelleme_baslangici)?;
        let zarf: MetadataZarfi<KokMetadata> = zarfi_oku(yeni_root, AZAMI_KOK_BOYUTU, "root")?;
        kok_yapisini_dogrula(&zarf.imzali)?;
        let beklenen = self
            .kok
            .surum
            .checked_add(1)
            .ok_or_else(|| RegistryHatasi::metadata("Root sürümü u64 sınırını aşıyor."))?;
        if zarf.imzali.surum != beklenen {
            return Err(RegistryHatasi::metadata(format!(
                "Root rotasyonu ardışık değil: {} beklenirken {} geldi.",
                beklenen, zarf.imzali.surum
            )));
        }
        rolu_dogrula(&zarf, "root", &self.kok)?;
        rolu_dogrula(&zarf, "root", &zarf.imzali)?;
        sona_ermeyi_dogrula(&zarf.imzali.sona_erme, guncelleme_baslangici, "root")?;
        self.kok = zarf.imzali;
        self.durum.root = beklenen;
        Ok(())
    }

    /// Timestamp → snapshot → targets zincirini tek başlangıç saatiyle ve
    /// kalıcı sürümlere göre doğrular. Başarı nesnesi uygulanmadıkça hiçbir
    /// görülen sürüm kalıcı duruma geçmez.
    pub fn zinciri_dogrula(
        &self,
        timestamp: &[u8],
        snapshot: &[u8],
        targets: &[u8],
        guncelleme_baslangici: i64,
    ) -> Result<DogrulanmisRegistry, RegistryHatasi> {
        zaman_girdisini_dogrula(guncelleme_baslangici)?;
        sona_ermeyi_dogrula(&self.kok.sona_erme, guncelleme_baslangici, "root")?;

        let timestamp_zarfi: MetadataZarfi<TimestampMetadata> =
            zarfi_oku(timestamp, AZAMI_TIMESTAMP_BOYUTU, "timestamp")?;
        temel_metadatayi_dogrula(
            &timestamp_zarfi.imzali.sema,
            "zee-registry-timestamp-v1",
            timestamp_zarfi.imzali.surum,
            &timestamp_zarfi.imzali.sona_erme,
            guncelleme_baslangici,
            "timestamp",
        )?;
        metadata_gecmisini_dogrula(
            "timestamp",
            timestamp_zarfi.imzali.surum,
            self.durum.timestamp,
            timestamp,
            &self.durum.timestamp_sha256,
        )?;
        metadata_dosyasini_dogrula(
            &timestamp_zarfi.imzali.snapshot,
            AZAMI_SNAPSHOT_BOYUTU,
            "snapshot",
        )?;
        rolu_dogrula(&timestamp_zarfi, "timestamp", &self.kok)?;

        bagli_baytlari_dogrula(snapshot, &timestamp_zarfi.imzali.snapshot, "snapshot")?;
        let snapshot_zarfi: MetadataZarfi<SnapshotMetadata> =
            zarfi_oku(snapshot, AZAMI_SNAPSHOT_BOYUTU, "snapshot")?;
        temel_metadatayi_dogrula(
            &snapshot_zarfi.imzali.sema,
            "zee-registry-snapshot-v1",
            snapshot_zarfi.imzali.surum,
            &snapshot_zarfi.imzali.sona_erme,
            guncelleme_baslangici,
            "snapshot",
        )?;
        if snapshot_zarfi.imzali.surum != timestamp_zarfi.imzali.snapshot.surum {
            return Err(RegistryHatasi::metadata(
                "Snapshot iç sürümü timestamp tarafından bağlanan sürümle uyuşmuyor.",
            ));
        }
        metadata_gecmisini_dogrula(
            "snapshot",
            snapshot_zarfi.imzali.surum,
            self.durum.snapshot,
            snapshot,
            &self.durum.snapshot_sha256,
        )?;
        metadata_dosyasini_dogrula(
            &snapshot_zarfi.imzali.targets,
            AZAMI_TARGETS_BOYUTU,
            "targets",
        )?;
        rolu_dogrula(&snapshot_zarfi, "snapshot", &self.kok)?;

        bagli_baytlari_dogrula(targets, &snapshot_zarfi.imzali.targets, "targets")?;
        let targets_zarfi: MetadataZarfi<TargetsMetadata> =
            zarfi_oku(targets, AZAMI_TARGETS_BOYUTU, "targets")?;
        temel_metadatayi_dogrula(
            &targets_zarfi.imzali.sema,
            "zee-registry-targets-v1",
            targets_zarfi.imzali.surum,
            &targets_zarfi.imzali.sona_erme,
            guncelleme_baslangici,
            "targets",
        )?;
        if targets_zarfi.imzali.surum != snapshot_zarfi.imzali.targets.surum {
            return Err(RegistryHatasi::metadata(
                "Targets iç sürümü snapshot tarafından bağlanan sürümle uyuşmuyor.",
            ));
        }
        metadata_gecmisini_dogrula(
            "targets",
            targets_zarfi.imzali.surum,
            self.durum.targets,
            targets,
            &self.durum.targets_sha256,
        )?;
        targets_yapisini_dogrula(&targets_zarfi.imzali)?;
        rolu_dogrula(&targets_zarfi, "targets", &self.kok)?;

        let targets_surum = targets_zarfi.imzali.surum;
        Ok(DogrulanmisRegistry {
            targets: targets_zarfi.imzali,
            surumler: MetadataSurumleri {
                root: self.kok.surum,
                timestamp: timestamp_zarfi.imzali.surum,
                timestamp_sha256: sha256_hex(timestamp),
                snapshot: snapshot_zarfi.imzali.surum,
                snapshot_sha256: sha256_hex(snapshot),
                targets: targets_surum,
                targets_sha256: sha256_hex(targets),
            },
        })
    }

    /// Yalnız `zinciri_dogrula` başarısından doğan ve mevcut root/durumdan
    /// eski olmayan sürümleri uygular. Önceden doğrulanmış ama daha sonra
    /// bayatlamış bir sonuç da rollback durumuna dönüştürülemez.
    pub fn durumu_uygula(
        &mut self,
        dogrulanmis: &DogrulanmisRegistry,
    ) -> Result<(), RegistryHatasi> {
        if dogrulanmis.surumler.root != self.kok.surum {
            return Err(RegistryHatasi::metadata(
                "Doğrulanmış metadata sonucu artık etkin root sürümüne ait değil.",
            ));
        }
        kayit_gecmisini_dogrula(
            "timestamp",
            dogrulanmis.surumler.timestamp,
            &dogrulanmis.surumler.timestamp_sha256,
            self.durum.timestamp,
            &self.durum.timestamp_sha256,
        )?;
        kayit_gecmisini_dogrula(
            "snapshot",
            dogrulanmis.surumler.snapshot,
            &dogrulanmis.surumler.snapshot_sha256,
            self.durum.snapshot,
            &self.durum.snapshot_sha256,
        )?;
        kayit_gecmisini_dogrula(
            "targets",
            dogrulanmis.surumler.targets,
            &dogrulanmis.surumler.targets_sha256,
            self.durum.targets,
            &self.durum.targets_sha256,
        )?;
        self.durum = dogrulanmis.surumler.clone();
        Ok(())
    }
}

/// Dört yayın dosyasını önce targets rolünün boyut/özet/yayıncı yetkisiyle,
/// sonra yayıncının kendi çapraz doğrulayıcısıyla denetler.
pub fn hedef_yayinini_dogrula(
    hedef: &HedefMetadata,
    yayin: &[u8],
    arsiv: &[u8],
    sbom: &[u8],
    provenance: &[u8],
) -> Result<ImzaliYayin, RegistryHatasi> {
    hedef_baytlarini_dogrula(&hedef.yayin, yayin, "yayın bildirimi")?;
    hedef_baytlarini_dogrula(&hedef.arsiv, arsiv, "kaynak paketi")?;
    hedef_baytlarini_dogrula(&hedef.sbom, sbom, "SBOM")?;
    hedef_baytlarini_dogrula(&hedef.provenance, provenance, "provenance")?;
    let imzali = yayini_dogrula(yayin, arsiv, sbom, provenance)
        .map_err(|hata| RegistryHatasi::hedef(format!("Yayın zinciri doğrulanamadı: {}", hata)))?;
    hedef_yayin_kimligini_dogrula(hedef, &imzali)?;
    Ok(imzali)
}

fn hedef_yayin_kimligini_dogrula(
    hedef: &HedefMetadata,
    yayin: &ImzaliYayin,
) -> Result<(), RegistryHatasi> {
    if hedef.paket != yayin.paket
        || hedef.surum != yayin.surum
        || hedef.morfoloji != yayin.morfoloji
        || hedef.yayinci_anahtar_kimligi != yayin.yayinci_anahtar_kimligi
        || hedef.arsiv != yayin.arsiv
        || hedef.sbom != yayin.sbom
        || hedef.provenance != yayin.provenance
    {
        return Err(RegistryHatasi::hedef(
            "Targets yetkisi yayıncı/paket/sürüm/morfoloji veya yayın dosyalarıyla uyuşmuyor.",
        ));
    }
    Ok(())
}

fn zarfi_oku<T: DeserializeOwned + Serialize>(
    ham: &[u8],
    azami: usize,
    ad: &str,
) -> Result<MetadataZarfi<T>, RegistryHatasi> {
    if ham.len() > azami {
        return Err(RegistryHatasi::metadata(format!(
            "{} metadata {} byte sınırını aşıyor.",
            ad, azami
        )));
    }
    let zarf: MetadataZarfi<T> = serde_json::from_slice(ham).map_err(|hata| {
        RegistryHatasi::metadata(format!(
            "{} metadata geçerli kapalı şema JSON değil: {}.",
            ad, hata
        ))
    })?;
    if zarf.imzalar.len() > AZAMI_IMZA_SAYISI {
        return Err(RegistryHatasi::metadata(format!(
            "{} metadata çok fazla imza taşıyor.",
            ad
        )));
    }
    let imza_kimlikleri = zarf
        .imzalar
        .iter()
        .map(|imza| imza.anahtar_kimligi.clone())
        .collect::<Vec<_>>();
    if !kesin_artiyor(&imza_kimlikleri) {
        return Err(RegistryHatasi::metadata(format!(
            "{} metadata imzaları anahtar kimliğine göre tekil ve sıralı değil.",
            ad
        )));
    }
    let mut kanonik = serde_json::to_vec_pretty(&zarf).map_err(|hata| {
        RegistryHatasi::metadata(format!("{} metadata kanonikleştirilemedi: {}.", ad, hata))
    })?;
    kanonik.push(b'\n');
    if ham != kanonik {
        return Err(RegistryHatasi::metadata(format!(
            "{} metadata zee-registry-v1 kanonik JSON biçiminde değil.",
            ad
        )));
    }
    Ok(zarf)
}

fn rolu_dogrula<T: Serialize>(
    zarf: &MetadataZarfi<T>,
    rol: &str,
    kok: &KokMetadata,
) -> Result<(), RegistryHatasi> {
    let yetki = kok.roller.get(rol).ok_or_else(|| {
        RegistryHatasi::metadata(format!("Root {} rolü yetkisini taşımıyor.", rol))
    })?;
    let imza_girdisi = imza_girdisi(rol, &zarf.imzali)?;
    let izinli = yetki
        .anahtar_kimlikleri
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut dogrulanan = BTreeSet::new();
    for imza in &zarf.imzalar {
        if !izinli.contains(imza.anahtar_kimligi.as_str())
            || dogrulanan.contains(imza.anahtar_kimligi.as_str())
        {
            continue;
        }
        let Some(anahtar) = kok.anahtarlar.get(&imza.anahtar_kimligi) else {
            continue;
        };
        let Ok(acik) = hex_coz::<32>(&anahtar.acik, "registry açık anahtarı") else {
            continue;
        };
        let Ok(imza_baytlari) = hex_coz::<64>(&imza.ed25519, "registry Ed25519 imzası") else {
            continue;
        };
        let Ok(dogrulayici) = VerifyingKey::from_bytes(&acik) else {
            continue;
        };
        let ed25519_imzasi = Signature::from_bytes(&imza_baytlari);
        if dogrulayici
            .verify_strict(&imza_girdisi, &ed25519_imzasi)
            .is_ok()
        {
            dogrulanan.insert(imza.anahtar_kimligi.as_str());
        }
    }
    if dogrulanan.len() < usize::from(yetki.esik) {
        return Err(RegistryHatasi::metadata(format!(
            "{} rolü imza eşiğini sağlamıyor: {} / {}.",
            rol,
            dogrulanan.len(),
            yetki.esik
        )));
    }
    Ok(())
}

fn imza_girdisi<T: Serialize>(rol: &str, imzali: &T) -> Result<Vec<u8>, RegistryHatasi> {
    let mut sonuc = IMZA_ALANI.to_vec();
    sonuc.extend_from_slice(rol.as_bytes());
    sonuc.push(0);
    sonuc.extend_from_slice(&serde_json::to_vec(imzali).map_err(|hata| {
        RegistryHatasi::metadata(format!("{} rolü imza girdisi üretilemedi: {}.", rol, hata))
    })?);
    Ok(sonuc)
}

fn kok_yapisini_dogrula(kok: &KokMetadata) -> Result<(), RegistryHatasi> {
    if kok.sema != "zee-registry-root-v1" || kok.surum == 0 || !kok.tutarli_anlik {
        return Err(RegistryHatasi::metadata(
            "Root şeması/sürümü geçersiz veya tutarlı anlık görünüm kapalı.",
        ));
    }
    rfc3339_utc_saniyesi(&kok.sona_erme)?;
    if kok.anahtarlar.is_empty() || kok.anahtarlar.len() > AZAMI_ANAHTAR_SAYISI {
        return Err(RegistryHatasi::metadata(
            "Root anahtar sayısı 1..256 aralığında olmalı.",
        ));
    }
    for (kimlik, anahtar) in &kok.anahtarlar {
        if anahtar.tur != "ed25519" || anahtar.sema != "ed25519" {
            return Err(RegistryHatasi::metadata(
                "Registry v1 yalnız ed25519 anahtar/şemasını kabul eder.",
            ));
        }
        let acik = hex_coz::<32>(&anahtar.acik, "registry açık anahtarı")?;
        if kimlik != &format!("sha256:{}", sha256_hex(&acik)) {
            return Err(RegistryHatasi::metadata(
                "Root anahtar kimliği açık anahtarın SHA-256 özetiyle uyuşmuyor.",
            ));
        }
        VerifyingKey::from_bytes(&acik).map_err(|_| {
            RegistryHatasi::metadata("Root geçersiz Ed25519 açık anahtarı taşıyor.")
        })?;
    }
    let bulunan = kok
        .roller
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let beklenen = ROLLER.into_iter().collect::<BTreeSet<_>>();
    if bulunan != beklenen {
        return Err(RegistryHatasi::metadata(
            "Root tam olarak root/targets/snapshot/timestamp rollerini taşımalı.",
        ));
    }
    for (rol, yetki) in &kok.roller {
        if yetki.anahtar_kimlikleri.is_empty()
            || yetki.anahtar_kimlikleri.len() > AZAMI_ANAHTAR_SAYISI
            || yetki.esik == 0
            || usize::from(yetki.esik) > yetki.anahtar_kimlikleri.len()
        {
            return Err(RegistryHatasi::metadata(format!(
                "{} rolü anahtar/eşik sınırları geçersiz.",
                rol
            )));
        }
        if !kesin_artiyor(&yetki.anahtar_kimlikleri) {
            return Err(RegistryHatasi::metadata(format!(
                "{} rolü anahtar kimlikleri tekil ve sıralı değil.",
                rol
            )));
        }
        if yetki
            .anahtar_kimlikleri
            .iter()
            .any(|kimlik| !kok.anahtarlar.contains_key(kimlik))
        {
            return Err(RegistryHatasi::metadata(format!(
                "{} rolü root anahtar haritasında olmayan kimlik taşıyor.",
                rol
            )));
        }
    }
    Ok(())
}

fn temel_metadatayi_dogrula(
    sema: &str,
    beklenen_sema: &str,
    surum: u64,
    sona_erme: &str,
    guncelleme_baslangici: i64,
    rol: &str,
) -> Result<(), RegistryHatasi> {
    if sema != beklenen_sema || surum == 0 {
        return Err(RegistryHatasi::metadata(format!(
            "{} metadata şeması veya sürümü geçersiz.",
            rol
        )));
    }
    sona_ermeyi_dogrula(sona_erme, guncelleme_baslangici, rol)
}

fn metadata_dosyasini_dogrula(
    bilgi: &MetadataDosyasi,
    azami: usize,
    ad: &str,
) -> Result<(), RegistryHatasi> {
    if bilgi.surum == 0 || bilgi.boyut == 0 || bilgi.boyut > azami as u64 {
        return Err(RegistryHatasi::metadata(format!(
            "{} metadata bağlı sürüm/boyut sınırı geçersiz.",
            ad
        )));
    }
    ozet_bicimini_dogrula(
        &format!("sha256:{}", bilgi.sha256),
        &format!("{} özeti", ad),
    )
}

fn bagli_baytlari_dogrula(
    baytlar: &[u8],
    bilgi: &MetadataDosyasi,
    ad: &str,
) -> Result<(), RegistryHatasi> {
    let gercek_boyut = u64::try_from(baytlar.len()).map_err(|_| {
        RegistryHatasi::metadata(format!("{} boyutu bu platformda temsil edilemiyor.", ad))
    })?;
    if gercek_boyut != bilgi.boyut || sha256_hex(baytlar) != bilgi.sha256 {
        return Err(RegistryHatasi::metadata(format!(
            "{} byte'ları üst rolün boyut/SHA-256 bağıyla uyuşmuyor.",
            ad
        )));
    }
    Ok(())
}

fn metadata_gecmisini_dogrula(
    rol: &str,
    bulunan: u64,
    onceki: u64,
    baytlar: &[u8],
    onceki_sha256: &str,
) -> Result<(), RegistryHatasi> {
    if bulunan < onceki {
        return Err(RegistryHatasi::metadata(format!(
            "{} rollback saldırısı: kalıcı sürüm {}, gelen sürüm {}.",
            rol, onceki, bulunan
        )));
    }
    if bulunan == onceki
        && onceki != 0
        && (onceki_sha256.len() != 64 || sha256_hex(baytlar) != onceki_sha256)
    {
        return Err(RegistryHatasi::metadata(format!(
            "{} aynı sürüm numarasıyla farklı içerik taşıyor; metadata eşdeğerliği reddedildi.",
            rol
        )));
    }
    Ok(())
}

fn durum_yapisini_dogrula(durum: &MetadataSurumleri) -> Result<(), RegistryHatasi> {
    if durum.root == 0 && (durum.timestamp != 0 || durum.snapshot != 0 || durum.targets != 0) {
        return Err(RegistryHatasi::metadata(
            "Kalıcı metadata durumu root sürümü olmadan çevrimiçi rol sürümü taşıyor.",
        ));
    }
    for (rol, surum, ozet) in [
        ("timestamp", durum.timestamp, &durum.timestamp_sha256),
        ("snapshot", durum.snapshot, &durum.snapshot_sha256),
        ("targets", durum.targets, &durum.targets_sha256),
    ] {
        if (surum == 0 && !ozet.is_empty())
            || (surum != 0 && hex_coz::<32>(ozet, &format!("kalıcı {} özeti", rol)).is_err())
        {
            return Err(RegistryHatasi::metadata(format!(
                "Kalıcı {} sürüm/SHA-256 durumu tutarsız.",
                rol
            )));
        }
    }
    Ok(())
}

fn kayit_gecmisini_dogrula(
    rol: &str,
    bulunan: u64,
    bulunan_sha256: &str,
    onceki: u64,
    onceki_sha256: &str,
) -> Result<(), RegistryHatasi> {
    if bulunan < onceki {
        return Err(RegistryHatasi::metadata(format!(
            "{} doğrulama sonucu uygulanmadan önce bayatladı: kalıcı {}, sonuç {}.",
            rol, onceki, bulunan
        )));
    }
    if bulunan == onceki && bulunan != 0 && bulunan_sha256 != onceki_sha256 {
        return Err(RegistryHatasi::metadata(format!(
            "{} doğrulama sonucu aynı sürümde kalıcı özetle uyuşmuyor.",
            rol
        )));
    }
    Ok(())
}

fn targets_yapisini_dogrula(targets: &TargetsMetadata) -> Result<(), RegistryHatasi> {
    if targets.hedefler.len() > AZAMI_HEDEF_SAYISI || targets.duyurular.len() > AZAMI_DUYURU_SAYISI
    {
        return Err(RegistryHatasi::metadata(
            "Targets hedef/duyuru sayısı güvenlik sınırını aşıyor.",
        ));
    }
    for (kimlik, duyuru) in &targets.duyurular {
        guvenli_kimlik(kimlik, "duyuru kimliği")?;
        if kimlik != &duyuru.kimlik
            || !gecerli_paket_adi(&duyuru.paket)
            || duyuru.etkilenen_surumler.is_empty()
            || !kesin_artiyor(&duyuru.etkilenen_surumler)
            || duyuru
                .etkilenen_surumler
                .iter()
                .any(|surum| !gecerli_surum(surum))
            || !matches!(duyuru.onem.as_str(), "düşük" | "orta" | "yüksek" | "kritik")
            || duyuru
                .duzeltilen_surum
                .as_deref()
                .is_some_and(|surum| !gecerli_surum(surum))
        {
            return Err(RegistryHatasi::metadata(format!(
                "{} güvenlik duyurusu kapalı şema kurallarına uymuyor.",
                kimlik
            )));
        }
    }
    for (kimlik, hedef) in &targets.hedefler {
        if !gecerli_paket_adi(&hedef.paket)
            || !gecerli_surum(&hedef.surum)
            || kimlik != &format!("{}@{}", hedef.paket, hedef.surum)
            || hedef.morfoloji != crate::morfoloji::MORFOLOJI_PROFILI
        {
            return Err(RegistryHatasi::metadata(format!(
                "{} targets paket/sürüm/morfoloji kimliği geçersiz.",
                kimlik
            )));
        }
        ozet_bicimini_dogrula(&hedef.yayinci_anahtar_kimligi, "yayıncı anahtar kimliği")?;
        if !kesin_artiyor(&hedef.duyurular) {
            return Err(RegistryHatasi::metadata(format!(
                "{} duyuru kimlikleri tekil ve sıralı değil.",
                kimlik
            )));
        }
        hedef_dosyasi_dogrula(
            &hedef.arsiv,
            &hedef.paket,
            &hedef.surum,
            ".zep",
            AZAMI_ARSIV_BOYUTU,
        )?;
        hedef_dosyasi_dogrula(
            &hedef.sbom,
            &hedef.paket,
            &hedef.surum,
            ".spdx.json",
            AZAMI_SBOM_BOYUTU,
        )?;
        hedef_dosyasi_dogrula(
            &hedef.provenance,
            &hedef.paket,
            &hedef.surum,
            ".intoto.json",
            AZAMI_PROVENANCE_BOYUTU,
        )?;
        hedef_dosyasi_dogrula(
            &hedef.yayin,
            &hedef.paket,
            &hedef.surum,
            ".zee-yayin.json",
            AZAMI_YAYIN_BOYUTU,
        )?;
        for duyuru_kimligi in &hedef.duyurular {
            let duyuru = targets.duyurular.get(duyuru_kimligi).ok_or_else(|| {
                RegistryHatasi::metadata(format!(
                    "{} var olmayan {} duyurusuna başvuruyor.",
                    kimlik, duyuru_kimligi
                ))
            })?;
            if duyuru.paket != hedef.paket
                || duyuru
                    .etkilenen_surumler
                    .binary_search(&hedef.surum)
                    .is_err()
            {
                return Err(RegistryHatasi::metadata(format!(
                    "{} duyurusu {} hedefini kapsamıyor.",
                    duyuru_kimligi, kimlik
                )));
            }
        }
    }
    Ok(())
}

fn hedef_dosyasi_dogrula(
    dosya: &YayinDosyasi,
    paket: &str,
    surum: &str,
    uzanti: &str,
    azami: u64,
) -> Result<(), RegistryHatasi> {
    let beklenen = format!("{}-{}{}", paket, surum, uzanti);
    if dosya.ad != beklenen || dosya.boyut == 0 || dosya.boyut > azami {
        return Err(RegistryHatasi::metadata(format!(
            "{} hedef dosyası adı/boyutu geçersiz.",
            beklenen
        )));
    }
    ozet_bicimini_dogrula(
        &format!("sha256:{}", dosya.sha256),
        &format!("{} özeti", beklenen),
    )
}

fn hedef_baytlarini_dogrula(
    bilgi: &YayinDosyasi,
    baytlar: &[u8],
    ad: &str,
) -> Result<(), RegistryHatasi> {
    let boyut = u64::try_from(baytlar.len())
        .map_err(|_| RegistryHatasi::hedef(format!("{} boyutu temsil edilemiyor.", ad)))?;
    if bilgi.boyut != boyut || bilgi.sha256 != sha256_hex(baytlar) {
        return Err(RegistryHatasi::hedef(format!(
            "{} targets boyut/SHA-256 bağıyla uyuşmuyor.",
            ad
        )));
    }
    Ok(())
}

fn ozet_bicimini_dogrula(ozet: &str, ad: &str) -> Result<(), RegistryHatasi> {
    let Some(hex) = ozet.strip_prefix("sha256:") else {
        return Err(RegistryHatasi::metadata(format!(
            "{} sha256: önekli olmalı.",
            ad
        )));
    };
    hex_coz::<32>(hex, ad).map(|_| ())
}

fn guvenli_kimlik(kimlik: &str, ad: &str) -> Result<(), RegistryHatasi> {
    if kimlik.is_empty()
        || kimlik.len() > 256
        || kimlik.trim() != kimlik
        || kimlik.chars().any(char::is_control)
        || kimlik.contains(['/', '\\', ':'])
    {
        return Err(RegistryHatasi::metadata(format!("{} güvenli değil.", ad)));
    }
    Ok(())
}

fn gecerli_paket_adi(ad: &str) -> bool {
    !ad.is_empty()
        && ad.len() <= 128
        && ad.trim() == ad
        && !ad.chars().any(char::is_control)
        && !ad.contains(['@', '/', '\\', ':'])
}

fn gecerli_surum(surum: &str) -> bool {
    let parcalar = surum.split('.').collect::<Vec<_>>();
    parcalar.len() == 3
        && parcalar.iter().all(|parca| {
            !parca.is_empty()
                && parca.bytes().all(|bayt| bayt.is_ascii_digit())
                && (parca == &"0" || !parca.starts_with('0'))
                && parca.parse::<u64>().is_ok()
        })
}

fn kesin_artiyor(degerler: &[String]) -> bool {
    degerler.windows(2).all(|cift| cift[0] < cift[1])
}

fn zaman_girdisini_dogrula(saniye: i64) -> Result<(), RegistryHatasi> {
    if saniye < 0 {
        return Err(RegistryHatasi::metadata(
            "Registry güncelleme başlangıç zamanı negatif olamaz.",
        ));
    }
    Ok(())
}

fn sona_ermeyi_dogrula(
    sona_erme: &str,
    guncelleme_baslangici: i64,
    rol: &str,
) -> Result<(), RegistryHatasi> {
    let sona_erme_saniyesi = rfc3339_utc_saniyesi(sona_erme)?;
    if sona_erme_saniyesi <= guncelleme_baslangici {
        return Err(RegistryHatasi::metadata(format!(
            "{} metadata süresi güncelleme başlangıcında dolmuş.",
            rol
        )));
    }
    Ok(())
}

fn rfc3339_utc_saniyesi(metin: &str) -> Result<i64, RegistryHatasi> {
    let baytlar = metin.as_bytes();
    if baytlar.len() != 20
        || baytlar[4] != b'-'
        || baytlar[7] != b'-'
        || baytlar[10] != b'T'
        || baytlar[13] != b':'
        || baytlar[16] != b':'
        || baytlar[19] != b'Z'
    {
        return Err(RegistryHatasi::metadata(
            "Metadata sona_erme zamanı YYYY-MM-DDTHH:MM:SSZ biçiminde UTC olmalı.",
        ));
    }
    let sayi = |bas: usize, son: usize| -> Option<i64> {
        let parca = &baytlar[bas..son];
        if !parca.iter().all(u8::is_ascii_digit) {
            return None;
        }
        std::str::from_utf8(parca).ok()?.parse().ok()
    };
    let yil = sayi(0, 4).ok_or_else(|| RegistryHatasi::metadata("Metadata yılı geçersiz."))?;
    let ay = sayi(5, 7).ok_or_else(|| RegistryHatasi::metadata("Metadata ayı geçersiz."))?;
    let gun = sayi(8, 10).ok_or_else(|| RegistryHatasi::metadata("Metadata günü geçersiz."))?;
    let saat = sayi(11, 13).ok_or_else(|| RegistryHatasi::metadata("Metadata saati geçersiz."))?;
    let dakika =
        sayi(14, 16).ok_or_else(|| RegistryHatasi::metadata("Metadata dakikası geçersiz."))?;
    let saniye =
        sayi(17, 19).ok_or_else(|| RegistryHatasi::metadata("Metadata saniyesi geçersiz."))?;
    if !(1970..=9999).contains(&yil)
        || !(1..=12).contains(&ay)
        || gun == 0
        || gun > aydaki_gun(yil, ay)
        || saat > 23
        || dakika > 59
        || saniye > 59
    {
        return Err(RegistryHatasi::metadata(
            "Metadata sona_erme zamanı geçerli bir UTC anı değil.",
        ));
    }
    let gunler = tarihten_gunler(yil, ay, gun);
    gunler
        .checked_mul(86_400)
        .and_then(|deger| deger.checked_add(saat * 3_600 + dakika * 60 + saniye))
        .ok_or_else(|| RegistryHatasi::metadata("Metadata sona_erme zamanı taşma üretti."))
}

fn aydaki_gun(yil: i64, ay: i64) -> i64 {
    match ay {
        2 if yil % 400 == 0 || (yil % 4 == 0 && yil % 100 != 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn tarihten_gunler(mut yil: i64, ay: i64, gun: i64) -> i64 {
    yil -= i64::from(ay <= 2);
    let donem = yil.div_euclid(400);
    let donem_yili = yil - donem * 400;
    let kaydirilmis_ay = ay + if ay > 2 { -3 } else { 9 };
    let yil_gunu = (153 * kaydirilmis_ay + 2) / 5 + gun - 1;
    let donem_gunu = donem_yili * 365 + donem_yili / 4 - donem_yili / 100 + yil_gunu;
    donem * 146_097 + donem_gunu - 719_468
}

fn hex_coz<const N: usize>(metin: &str, ad: &str) -> Result<[u8; N], RegistryHatasi> {
    if metin.len() != N * 2
        || !metin
            .bytes()
            .all(|bayt| bayt.is_ascii_digit() || (b'a'..=b'f').contains(&bayt))
    {
        return Err(RegistryHatasi::metadata(format!(
            "{} tam {} byte küçük harfli hex olmalı.",
            ad, N
        )));
    }
    let mut sonuc = [0; N];
    for (sira, hedef) in sonuc.iter_mut().enumerate() {
        let basamak = |bayt: u8| {
            if bayt.is_ascii_digit() {
                bayt - b'0'
            } else {
                bayt - b'a' + 10
            }
        };
        *hedef =
            (basamak(metin.as_bytes()[sira * 2]) << 4) | basamak(metin.as_bytes()[sira * 2 + 1]);
    }
    Ok(sonuc)
}

#[cfg(test)]
mod testler {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    struct Anahtarlar {
        root1: SigningKey,
        root2: SigningKey,
        targets: SigningKey,
        snapshot: SigningKey,
        timestamp: SigningKey,
    }

    fn anahtarlar() -> Anahtarlar {
        Anahtarlar {
            root1: SigningKey::from_bytes(&[1; 32]),
            root2: SigningKey::from_bytes(&[2; 32]),
            targets: SigningKey::from_bytes(&[3; 32]),
            snapshot: SigningKey::from_bytes(&[4; 32]),
            timestamp: SigningKey::from_bytes(&[5; 32]),
        }
    }

    fn kimlik(anahtar: &SigningKey) -> String {
        format!("sha256:{}", sha256_hex(&anahtar.verifying_key().to_bytes()))
    }

    fn kok(surum: u64, anahtarlar: &Anahtarlar) -> KokMetadata {
        let mut harita = BTreeMap::new();
        for anahtar in [
            &anahtarlar.root1,
            &anahtarlar.root2,
            &anahtarlar.targets,
            &anahtarlar.snapshot,
            &anahtarlar.timestamp,
        ] {
            harita.insert(
                kimlik(anahtar),
                RegistryAnahtari {
                    tur: "ed25519".into(),
                    sema: "ed25519".into(),
                    acik: hex_yaz(&anahtar.verifying_key().to_bytes()),
                },
            );
        }
        let mut kok_kimlikleri = vec![kimlik(&anahtarlar.root1), kimlik(&anahtarlar.root2)];
        kok_kimlikleri.sort();
        let mut roller = BTreeMap::new();
        roller.insert(
            "root".into(),
            RolYetkisi {
                anahtar_kimlikleri: kok_kimlikleri,
                esik: 2,
            },
        );
        for (rol, anahtar) in [
            ("targets", &anahtarlar.targets),
            ("snapshot", &anahtarlar.snapshot),
            ("timestamp", &anahtarlar.timestamp),
        ] {
            roller.insert(
                rol.into(),
                RolYetkisi {
                    anahtar_kimlikleri: vec![kimlik(anahtar)],
                    esik: 1,
                },
            );
        }
        KokMetadata {
            sema: "zee-registry-root-v1".into(),
            surum,
            sona_erme: "2030-01-01T00:00:00Z".into(),
            tutarli_anlik: true,
            anahtarlar: harita,
            roller,
        }
    }

    fn zarf<T: Serialize + Clone>(rol: &str, imzali: T, anahtarlar: &[&SigningKey]) -> Vec<u8> {
        let girdi = imza_girdisi(rol, &imzali).expect("imza girdisi");
        let mut imzalar = anahtarlar
            .iter()
            .map(|anahtar| RegistryImzasi {
                anahtar_kimligi: kimlik(anahtar),
                ed25519: hex_yaz(&anahtar.sign(&girdi).to_bytes()),
            })
            .collect::<Vec<_>>();
        imzalar.sort_by(|a, b| a.anahtar_kimligi.cmp(&b.anahtar_kimligi));
        let zarf = MetadataZarfi { imzali, imzalar };
        let mut baytlar = serde_json::to_vec_pretty(&zarf).expect("json");
        baytlar.push(b'\n');
        baytlar
    }

    fn dosya(ad: &str, icerik: &[u8]) -> YayinDosyasi {
        YayinDosyasi {
            ad: ad.into(),
            boyut: icerik.len() as u64,
            sha256: sha256_hex(icerik),
        }
    }

    fn hedef() -> HedefMetadata {
        HedefMetadata {
            paket: "miras".into(),
            surum: "1.2.3".into(),
            morfoloji: crate::morfoloji::MORFOLOJI_PROFILI.into(),
            yayinci_anahtar_kimligi: format!("sha256:{}", "11".repeat(32)),
            yanked: false,
            duyurular: Vec::new(),
            arsiv: dosya("miras-1.2.3.zep", b"a"),
            sbom: dosya("miras-1.2.3.spdx.json", b"b"),
            provenance: dosya("miras-1.2.3.intoto.json", b"c"),
            yayin: dosya("miras-1.2.3.zee-yayin.json", b"d"),
        }
    }

    fn zincir(
        anahtarlar: &Anahtarlar,
        timestamp_surum: u64,
        snapshot_surum: u64,
        targets_surum: u64,
        sona_erme: &str,
    ) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mut hedefler = BTreeMap::new();
        hedefler.insert("miras@1.2.3".into(), hedef());
        let targets = zarf(
            "targets",
            TargetsMetadata {
                sema: "zee-registry-targets-v1".into(),
                surum: targets_surum,
                sona_erme: sona_erme.into(),
                hedefler,
                duyurular: BTreeMap::new(),
            },
            &[&anahtarlar.targets],
        );
        let snapshot = zarf(
            "snapshot",
            SnapshotMetadata {
                sema: "zee-registry-snapshot-v1".into(),
                surum: snapshot_surum,
                sona_erme: sona_erme.into(),
                targets: MetadataDosyasi {
                    surum: targets_surum,
                    boyut: targets.len() as u64,
                    sha256: sha256_hex(&targets),
                },
            },
            &[&anahtarlar.snapshot],
        );
        let timestamp = zarf(
            "timestamp",
            TimestampMetadata {
                sema: "zee-registry-timestamp-v1".into(),
                surum: timestamp_surum,
                sona_erme: sona_erme.into(),
                snapshot: MetadataDosyasi {
                    surum: snapshot_surum,
                    boyut: snapshot.len() as u64,
                    sha256: sha256_hex(&snapshot),
                },
            },
            &[&anahtarlar.timestamp],
        );
        (timestamp, snapshot, targets)
    }

    fn dogrulayici(anahtarlar: &Anahtarlar) -> RegistryDogrulayici {
        let kok = zarf(
            "root",
            kok(1, anahtarlar),
            &[&anahtarlar.root1, &anahtarlar.root2],
        );
        let sabit = format!("sha256:{}", sha256_hex(&kok));
        RegistryDogrulayici::sabitlenmis_kok(&kok, &sabit, 0).expect("root")
    }

    #[test]
    fn tam_rol_zinciri_dogrulanir_ve_durum_acikca_uygulanir() {
        let anahtarlar = anahtarlar();
        let mut dogrulayici = dogrulayici(&anahtarlar);
        let (timestamp, snapshot, targets) = zincir(&anahtarlar, 1, 1, 1, "2030-01-01T00:00:00Z");
        let sonuc = dogrulayici
            .zinciri_dogrula(&timestamp, &snapshot, &targets, 1)
            .expect("zincir");
        assert_eq!(dogrulayici.durum().timestamp, 0);
        assert_eq!(
            sonuc
                .hedef_sec("miras", "1.2.3", HedefPolitikasi::default())
                .expect("hedef")
                .paket,
            "miras"
        );
        let eski_sonuc = sonuc.clone();
        dogrulayici.durumu_uygula(&sonuc).expect("durum");
        assert_eq!(dogrulayici.durum().targets, 1);

        let (timestamp, snapshot, targets) = zincir(&anahtarlar, 2, 2, 2, "2030-01-01T00:00:00Z");
        let yeni_sonuc = dogrulayici
            .zinciri_dogrula(&timestamp, &snapshot, &targets, 1)
            .expect("yeni zincir");
        dogrulayici.durumu_uygula(&yeni_sonuc).expect("yeni durum");
        assert!(dogrulayici.durumu_uygula(&eski_sonuc).is_err());
        assert_eq!(dogrulayici.durum().targets, 2);
    }

    #[test]
    fn root_sabitleme_esik_ve_cift_esikli_ardisik_rotasyon_ister() {
        let anahtarlar = anahtarlar();
        let eksik = zarf("root", kok(1, &anahtarlar), &[&anahtarlar.root1]);
        let sabit = format!("sha256:{}", sha256_hex(&eksik));
        assert!(RegistryDogrulayici::sabitlenmis_kok(&eksik, &sabit, 0)
            .unwrap_err()
            .mesaj
            .contains("eşiğini"));

        let tam = zarf(
            "root",
            kok(1, &anahtarlar),
            &[&anahtarlar.root1, &anahtarlar.root2],
        );
        let tam_sabit = format!("sha256:{}", sha256_hex(&tam));
        let bozuk_durum = MetadataSurumleri {
            root: 1,
            timestamp: 1,
            ..MetadataSurumleri::default()
        };
        assert!(
            RegistryDogrulayici::sabitlenmis_kok_durumla(&tam, &tam_sabit, 0, bozuk_durum,)
                .unwrap_err()
                .mesaj
                .contains("tutarsız")
        );

        let mut dogrulayici = dogrulayici(&anahtarlar);
        let yeni_anahtarlar = Anahtarlar {
            root1: SigningKey::from_bytes(&[6; 32]),
            root2: SigningKey::from_bytes(&[7; 32]),
            targets: SigningKey::from_bytes(&[8; 32]),
            snapshot: SigningKey::from_bytes(&[9; 32]),
            timestamp: SigningKey::from_bytes(&[10; 32]),
        };
        let yeni_kok = kok(2, &yeni_anahtarlar);
        let yalniz_eski = zarf(
            "root",
            yeni_kok.clone(),
            &[&anahtarlar.root1, &anahtarlar.root2],
        );
        assert!(dogrulayici.kok_dondur(&yalniz_eski, 1).is_err());
        assert_eq!(dogrulayici.kok().surum, 1);

        let cift = zarf(
            "root",
            yeni_kok,
            &[
                &anahtarlar.root1,
                &anahtarlar.root2,
                &yeni_anahtarlar.root1,
                &yeni_anahtarlar.root2,
            ],
        );
        dogrulayici.kok_dondur(&cift, 1).expect("rotasyon");
        assert_eq!(dogrulayici.kok().surum, 2);
    }

    #[test]
    fn rollback_ve_gecersiz_zincir_fast_forward_durumunu_zehirlemez() {
        let anahtarlar = anahtarlar();
        let mut dogrulayici = dogrulayici(&anahtarlar);
        let (timestamp, snapshot, targets) = zincir(&anahtarlar, 4, 5, 6, "2030-01-01T00:00:00Z");
        let sonuc = dogrulayici
            .zinciri_dogrula(&timestamp, &snapshot, &targets, 1)
            .expect("ilk zincir");
        dogrulayici.durumu_uygula(&sonuc).expect("durum");

        let (eski_timestamp, eski_snapshot, eski_targets) =
            zincir(&anahtarlar, 3, 5, 6, "2030-01-01T00:00:00Z");
        assert!(dogrulayici
            .zinciri_dogrula(&eski_timestamp, &eski_snapshot, &eski_targets, 1)
            .unwrap_err()
            .mesaj
            .contains("rollback"));

        let (es_deger_timestamp, es_deger_snapshot, es_deger_targets) =
            zincir(&anahtarlar, 4, 5, 6, "2031-01-01T00:00:00Z");
        assert!(dogrulayici
            .zinciri_dogrula(
                &es_deger_timestamp,
                &es_deger_snapshot,
                &es_deger_targets,
                1,
            )
            .unwrap_err()
            .mesaj
            .contains("aynı sürüm"));

        let (ileri_timestamp, mut bozuk_snapshot, ileri_targets) =
            zincir(&anahtarlar, 999, 999, 999, "2030-01-01T00:00:00Z");
        *bozuk_snapshot.last_mut().expect("byte") ^= 1;
        assert!(dogrulayici
            .zinciri_dogrula(&ileri_timestamp, &bozuk_snapshot, &ileri_targets, 1)
            .is_err());
        assert_eq!(dogrulayici.durum().timestamp, 4);
    }

    #[test]
    fn expiry_mix_and_match_ve_kanonik_olmayan_metadata_reddedilir() {
        let anahtarlar = anahtarlar();
        let dogrulayici = dogrulayici(&anahtarlar);
        let (timestamp, snapshot, targets) = zincir(&anahtarlar, 1, 1, 1, "1970-01-01T00:00:01Z");
        assert!(dogrulayici
            .zinciri_dogrula(&timestamp, &snapshot, &targets, 1)
            .unwrap_err()
            .mesaj
            .contains("süresi"));

        let (timestamp, snapshot, _) = zincir(&anahtarlar, 2, 2, 2, "2030-01-01T00:00:00Z");
        let (_, _, eski_targets) = zincir(&anahtarlar, 1, 1, 1, "2030-01-01T00:00:00Z");
        assert!(dogrulayici
            .zinciri_dogrula(&timestamp, &snapshot, &eski_targets, 1)
            .unwrap_err()
            .mesaj
            .contains("boyut/SHA-256"));

        let (mut timestamp, snapshot, targets) =
            zincir(&anahtarlar, 3, 3, 3, "2030-01-01T00:00:00Z");
        timestamp.insert(0, b' ');
        assert!(dogrulayici
            .zinciri_dogrula(&timestamp, &snapshot, &targets, 1)
            .unwrap_err()
            .mesaj
            .contains("kanonik"));
    }

    #[test]
    fn metadata_boyut_limiti_ayirmadan_reddedilir() {
        let anahtarlar = anahtarlar();
        let dogrulayici = dogrulayici(&anahtarlar);
        let buyuk = vec![b' '; AZAMI_TIMESTAMP_BOYUTU + 1];
        assert!(dogrulayici
            .zinciri_dogrula(&buyuk, &[], &[], 1)
            .unwrap_err()
            .mesaj
            .contains("sınırını"));
    }

    #[test]
    fn yanked_ve_kritik_duyuru_acik_politika_ister() {
        let anahtarlar = anahtarlar();
        let dogrulayici = dogrulayici(&anahtarlar);
        let (timestamp, snapshot, targets_baytlari) =
            zincir(&anahtarlar, 1, 1, 1, "2030-01-01T00:00:00Z");
        let mut sonuc = dogrulayici
            .zinciri_dogrula(&timestamp, &snapshot, &targets_baytlari, 1)
            .expect("zincir");
        sonuc
            .targets
            .hedefler
            .get_mut("miras@1.2.3")
            .expect("hedef")
            .yanked = true;
        assert_eq!(
            sonuc
                .hedef_sec("miras", "1.2.3", HedefPolitikasi::default())
                .unwrap_err()
                .kod,
            "P014"
        );
        let hedef = sonuc
            .targets
            .hedefler
            .get_mut("miras@1.2.3")
            .expect("hedef");
        hedef.yanked = false;
        hedef.duyurular.push("ZEE-2026-1".into());
        sonuc.targets.duyurular.insert(
            "ZEE-2026-1".into(),
            DuyuruMetadata {
                kimlik: "ZEE-2026-1".into(),
                paket: "miras".into(),
                etkilenen_surumler: vec!["1.2.3".into()],
                onem: "kritik".into(),
                duzeltilen_surum: Some("1.2.4".into()),
                etkin: true,
            },
        );
        assert!(sonuc
            .hedef_sec("miras", "1.2.3", HedefPolitikasi::default())
            .unwrap_err()
            .mesaj
            .contains("kritik"));
        assert!(sonuc
            .hedef_sec(
                "miras",
                "1.2.3",
                HedefPolitikasi {
                    yanked_kabul: false,
                    kritik_duyuru_kabul: true,
                },
            )
            .is_ok());
    }

    #[test]
    fn yanlis_yayinci_targets_yetkisini_gecemez() {
        let mut hedef = hedef();
        let yayin = ImzaliYayin {
            sema: "zee-yayin-v1".into(),
            paket: hedef.paket.clone(),
            surum: hedef.surum.clone(),
            morfoloji: hedef.morfoloji.clone(),
            yayinci_anahtari: "22".repeat(32),
            yayinci_anahtar_kimligi: format!("sha256:{}", "22".repeat(32)),
            arsiv: hedef.arsiv.clone(),
            sbom: hedef.sbom.clone(),
            provenance: hedef.provenance.clone(),
        };
        assert_eq!(
            hedef_yayin_kimligini_dogrula(&hedef, &yayin)
                .unwrap_err()
                .kod,
            "P014"
        );
        hedef.yayinci_anahtar_kimligi = yayin.yayinci_anahtar_kimligi.clone();
        assert!(hedef_yayin_kimligini_dogrula(&hedef, &yayin).is_ok());
    }

    #[test]
    fn rfc3339_utc_takvim_sinirlarini_dogrular() {
        assert_eq!(rfc3339_utc_saniyesi("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(
            rfc3339_utc_saniyesi("2000-02-29T00:00:00Z").unwrap(),
            951_782_400
        );
        assert!(rfc3339_utc_saniyesi("2100-02-29T00:00:00Z").is_err());
        assert!(rfc3339_utc_saniyesi("2030-01-01T00:00:00+00:00").is_err());
    }

    fn hex_yaz(baytlar: &[u8]) -> String {
        const BASAMAK: &[u8; 16] = b"0123456789abcdef";
        let mut sonuc = String::with_capacity(baytlar.len() * 2);
        for bayt in baytlar {
            sonuc.push(BASAMAK[(bayt >> 4) as usize] as char);
            sonuc.push(BASAMAK[(bayt & 0x0f) as usize] as char);
        }
        sonuc
    }
}
