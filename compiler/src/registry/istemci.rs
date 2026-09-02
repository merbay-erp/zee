//! Registry'nin HTTPS/statik taşıma, doğrulanmış cache ve kalıcı durum katmanı.

use super::*;
use crate::guvenlik::sha256_hex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod depo;
mod tasima;

use depo::{kanonik_json, nesne_yolu, sha256_onunu_ayir, sinirli_dosya_oku};
#[cfg(test)]
use tasima::goreli_yolu_dogrula;
pub use tasima::{HttpsRegistryTasiyici, RegistryTasiyici};

const DURUM_SEMASI: &str = "zee-registry-cache-v1";
const DURUM_DOSYASI: &str = "durum-v1.json";

#[derive(Debug, Clone)]
pub struct RegistryPaketCiktisi {
    pub paket: String,
    pub surum: String,
    pub sabit_kok_sha256: String,
    pub metadata_surumleri: MetadataSurumleri,
    pub yayinci_anahtar_kimligi: String,
    pub arsiv_sha256: String,
    pub sbom_sha256: String,
    pub provenance_sha256: String,
    pub yayin_sha256: String,
    pub yanked: bool,
    pub kritik_duyurular: Vec<String>,
    pub arsiv: PathBuf,
    pub sbom: PathBuf,
    pub provenance: PathBuf,
    pub yayin: PathBuf,
    pub cevrimdisi: bool,
}

#[derive(Debug, Clone)]
pub struct RegistryIstemcisi {
    cache_koku: PathBuf,
    sabit_kok: Vec<u8>,
    sabit_kok_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KaliciRegistryDurumu {
    sema: String,
    sabit_kok_sha256: String,
    etkin_kok_sha256: String,
    dogrulama_zamani: i64,
    metadata: MetadataSurumleri,
}

struct Acilis {
    dogrulayici: RegistryDogrulayici,
    durum: Option<KaliciRegistryDurumu>,
    durum_baytlari: Option<Vec<u8>>,
    etkin_kok: Vec<u8>,
}

struct YayinBaytlari {
    arsiv: Vec<u8>,
    sbom: Vec<u8>,
    provenance: Vec<u8>,
    yayin: Vec<u8>,
}

impl RegistryIstemcisi {
    pub fn yeni(
        cache_koku: impl Into<PathBuf>,
        sabit_kok: Vec<u8>,
        sabit_kok_sha256: impl Into<String>,
    ) -> Self {
        Self {
            cache_koku: cache_koku.into(),
            sabit_kok,
            sabit_kok_sha256: sabit_kok_sha256.into(),
        }
    }

    /// Çevrimdışı açılışta ilk root byte'ı gerekmez; kalıcı durum etkin root
    /// nesnesini pin kimliğiyle açar. Cache yoksa istek P016 ile kapanır.
    pub fn mevcut_cache(
        cache_koku: impl Into<PathBuf>,
        sabit_kok_sha256: impl Into<String>,
    ) -> Self {
        Self::yeni(cache_koku, Vec::new(), sabit_kok_sha256)
    }

    /// Manifestte ağ dışından sabitlenen sürümlü ilk root byte'ını HTTPS
    /// aynadan limitli getirir. Güven kararı çağıranın verdiği SHA-256 pinini
    /// `yeni` açılışında doğrulayınca oluşur.
    pub fn sabit_koku_getir<T: RegistryTasiyici>(
        tasiyici: &mut T,
        surum: u64,
    ) -> Result<Vec<u8>, RegistryHatasi> {
        if surum == 0 {
            return Err(RegistryHatasi::tasima(
                "Sabit registry root sürümü pozitif olmalı.",
            ));
        }
        zorunlu_getir(
            tasiyici,
            &format!("metadata/{}.root.json", surum),
            AZAMI_KOK_BOYUTU,
        )
    }

    pub fn paketi_guncelle<T: RegistryTasiyici>(
        &self,
        tasiyici: &mut T,
        paket: &str,
        surum: &str,
        politika: HedefPolitikasi,
    ) -> Result<RegistryPaketCiktisi, RegistryHatasi> {
        self.paketi_guncelle_zamanla(tasiyici, paket, surum, politika, simdiki_unix_saniyesi()?)
    }

    /// Zaman girdisi açık çevrimiçi güncelleme; test ve tekrar üretilebilir
    /// istemci koşuları aynı saniyeyi kullanabilir.
    pub fn paketi_guncelle_zamanla<T: RegistryTasiyici>(
        &self,
        tasiyici: &mut T,
        paket: &str,
        surum: &str,
        politika: HedefPolitikasi,
        guncelleme_baslangici: i64,
    ) -> Result<RegistryPaketCiktisi, RegistryHatasi> {
        let mut acilis = self.ac(guncelleme_baslangici, false)?;
        self.kokleri_dondur(tasiyici, &mut acilis, guncelleme_baslangici)?;

        let timestamp = zorunlu_getir(tasiyici, "metadata/timestamp.json", AZAMI_TIMESTAMP_BOYUTU)?;
        let timestamp_zarfi: MetadataZarfi<TimestampMetadata> =
            zarfi_oku(&timestamp, AZAMI_TIMESTAMP_BOYUTU, "timestamp")?;
        let snapshot_yolu = format!(
            "metadata/{}.snapshot.json",
            timestamp_zarfi.imzali.snapshot.surum
        );
        let snapshot = zorunlu_getir(tasiyici, &snapshot_yolu, AZAMI_SNAPSHOT_BOYUTU)?;
        let snapshot_zarfi: MetadataZarfi<SnapshotMetadata> =
            zarfi_oku(&snapshot, AZAMI_SNAPSHOT_BOYUTU, "snapshot")?;
        let targets_yolu = format!(
            "metadata/{}.targets.json",
            snapshot_zarfi.imzali.targets.surum
        );
        let targets = zorunlu_getir(tasiyici, &targets_yolu, AZAMI_TARGETS_BOYUTU)?;

        let dogrulanmis = acilis.dogrulayici.zinciri_dogrula(
            &timestamp,
            &snapshot,
            &targets,
            guncelleme_baslangici,
        )?;
        let hedef = dogrulanmis.hedef_sec(paket, surum, politika)?.clone();
        let yayin_baytlari = self.yayin_baytlarini_getir(tasiyici, &hedef)?;
        hedef_yayinini_dogrula(
            &hedef,
            &yayin_baytlari.yayin,
            &yayin_baytlari.arsiv,
            &yayin_baytlari.sbom,
            &yayin_baytlari.provenance,
        )?;
        acilis.dogrulayici.durumu_uygula(&dogrulanmis)?;

        self.nesneyi_yaz(&acilis.etkin_kok, "etkin root")?;
        self.nesneyi_yaz(&timestamp, "timestamp")?;
        self.nesneyi_yaz(&snapshot, "snapshot")?;
        self.nesneyi_yaz(&targets, "targets")?;
        let yollar = self.yayin_baytlarini_cachele(&hedef, &yayin_baytlari)?;

        let yeni_durum = KaliciRegistryDurumu {
            sema: DURUM_SEMASI.into(),
            sabit_kok_sha256: self.sabit_kok_sha256.clone(),
            etkin_kok_sha256: format!("sha256:{}", sha256_hex(&acilis.etkin_kok)),
            dogrulama_zamani: guncelleme_baslangici,
            metadata: acilis.dogrulayici.durum(),
        };
        self.durumu_yaz(&yeni_durum, acilis.durum_baytlari.as_deref())?;
        let kritik_duyurular = dogrulanmis.etkin_kritik_duyurular(&hedef);
        Ok(cikti(&hedef, &yeni_durum, yollar, false, kritik_duyurular))
    }

    /// Ağ ve duvar saati kullanmadan yalnız daha önce tam doğrulanıp kalıcı
    /// duruma bağlanmış metadata ile içerik-adresli nesneleri açar.
    pub fn paketi_cevrimdisi_al(
        &self,
        paket: &str,
        surum: &str,
        politika: HedefPolitikasi,
    ) -> Result<RegistryPaketCiktisi, RegistryHatasi> {
        let acilis = self.ac(0, true)?;
        let durum = acilis
            .durum
            .as_ref()
            .ok_or_else(|| RegistryHatasi::tasima("Çevrimdışı registry durumu bulunamadı."))?;
        let timestamp = self.nesneyi_oku_azami(
            &durum.metadata.timestamp_sha256,
            AZAMI_TIMESTAMP_BOYUTU,
            "timestamp",
        )?;
        let snapshot = self.nesneyi_oku_azami(
            &durum.metadata.snapshot_sha256,
            AZAMI_SNAPSHOT_BOYUTU,
            "snapshot",
        )?;
        let targets = self.nesneyi_oku_azami(
            &durum.metadata.targets_sha256,
            AZAMI_TARGETS_BOYUTU,
            "targets",
        )?;
        let dogrulanmis = acilis.dogrulayici.zinciri_dogrula(
            &timestamp,
            &snapshot,
            &targets,
            durum.dogrulama_zamani,
        )?;
        if dogrulanmis.surumler() != durum.metadata {
            return Err(RegistryHatasi::tasima(
                "Çevrimdışı metadata cache'i kalıcı durumla uyuşmuyor.",
            ));
        }
        let hedef = dogrulanmis.hedef_sec(paket, surum, politika)?.clone();
        let yayin_baytlari = self.yayin_baytlarini_cacheden_oku(&hedef)?;
        hedef_yayinini_dogrula(
            &hedef,
            &yayin_baytlari.yayin,
            &yayin_baytlari.arsiv,
            &yayin_baytlari.sbom,
            &yayin_baytlari.provenance,
        )?;
        let yollar = hedef_yollari(&self.cache_koku, &hedef)?;
        let kritik_duyurular = dogrulanmis.etkin_kritik_duyurular(&hedef);
        Ok(cikti(&hedef, durum, yollar, true, kritik_duyurular))
    }

    fn ac(&self, zaman: i64, cevrimdisi: bool) -> Result<Acilis, RegistryHatasi> {
        let (durum, durum_baytlari) = self.durumu_oku()?;
        if cevrimdisi && durum.is_none() {
            return Err(RegistryHatasi::tasima(
                "Çevrimdışı registry cache miss: kalıcı doğrulama durumu yok.",
            ));
        }
        let (etkin_kok, etkin_ozet, metadata, dogrulama_zamani) = match &durum {
            Some(durum) => {
                if durum.sabit_kok_sha256 != self.sabit_kok_sha256 {
                    return Err(RegistryHatasi::tasima(
                        "Registry cache'i farklı bir ağ dışı root sabitlemesine ait.",
                    ));
                }
                let etkin_hex = sha256_onunu_ayir(&durum.etkin_kok_sha256, "etkin root özeti")?;
                (
                    self.nesneyi_oku_azami(etkin_hex, AZAMI_KOK_BOYUTU, "etkin root")?,
                    durum.etkin_kok_sha256.clone(),
                    durum.metadata.clone(),
                    if cevrimdisi {
                        durum.dogrulama_zamani
                    } else {
                        zaman
                    },
                )
            }
            None => (
                self.sabit_kok.clone(),
                self.sabit_kok_sha256.clone(),
                MetadataSurumleri::default(),
                zaman,
            ),
        };
        let dogrulayici = RegistryDogrulayici::sabitlenmis_kok_durumla(
            &etkin_kok,
            &etkin_ozet,
            dogrulama_zamani,
            metadata,
        )?;
        Ok(Acilis {
            dogrulayici,
            durum,
            durum_baytlari,
            etkin_kok,
        })
    }

    fn kokleri_dondur<T: RegistryTasiyici>(
        &self,
        tasiyici: &mut T,
        acilis: &mut Acilis,
        zaman: i64,
    ) -> Result<(), RegistryHatasi> {
        let sinir = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .registry()
            .kok_rotasyonu();
        for sira in 0..=sinir {
            let sonraki = acilis
                .dogrulayici
                .kok()
                .surum
                .checked_add(1)
                .ok_or_else(|| RegistryHatasi::metadata("Root sürümü u64 sınırını aşıyor."))?;
            let yol = format!("metadata/{}.root.json", sonraki);
            let Some(root) = tasiyici.getir(&yol, AZAMI_KOK_BOYUTU)? else {
                return Ok(());
            };
            if sira == sinir {
                return Err(RegistryHatasi::tasima(format!(
                    "Tek güncelleme {} root rotasyonu sınırını aşıyor.",
                    sinir
                )));
            }
            acilis.dogrulayici.kok_dondur(&root, zaman)?;
            acilis.etkin_kok = root;
        }
        Ok(())
    }

    fn yayin_baytlarini_getir<T: RegistryTasiyici>(
        &self,
        tasiyici: &mut T,
        hedef: &HedefMetadata,
    ) -> Result<YayinBaytlari, RegistryHatasi> {
        Ok(YayinBaytlari {
            arsiv: self.hedef_nesnesini_getir(tasiyici, &hedef.arsiv, "kaynak paketi")?,
            sbom: self.hedef_nesnesini_getir(tasiyici, &hedef.sbom, "SBOM")?,
            provenance: self.hedef_nesnesini_getir(tasiyici, &hedef.provenance, "provenance")?,
            yayin: self.hedef_nesnesini_getir(tasiyici, &hedef.yayin, "yayın bildirimi")?,
        })
    }

    fn hedef_nesnesini_getir<T: RegistryTasiyici>(
        &self,
        tasiyici: &mut T,
        dosya: &YayinDosyasi,
        ad: &str,
    ) -> Result<Vec<u8>, RegistryHatasi> {
        let azami = usize::try_from(dosya.boyut)
            .map_err(|_| RegistryHatasi::tasima(format!("{} boyutu temsil edilemiyor.", ad)))?;
        let cache_yolu = nesne_yolu(&self.cache_koku, &dosya.sha256)?;
        let baytlar = if cache_yolu.exists() {
            let baytlar = self.nesneyi_oku_azami(&dosya.sha256, azami, ad)?;
            if baytlar.len() as u64 != dosya.boyut {
                return Err(RegistryHatasi::tasima(format!(
                    "Bozuk {} cache nesnesi targets boyutuyla uyuşmuyor.",
                    ad
                )));
            }
            baytlar
        } else {
            let yol = format!("hedefler/sha256/{}", dosya.sha256);
            zorunlu_getir(tasiyici, &yol, azami)?
        };
        hedef_baytlarini_dogrula(dosya, &baytlar, ad)?;
        Ok(baytlar)
    }

    fn yayin_baytlarini_cachele(
        &self,
        hedef: &HedefMetadata,
        baytlar: &YayinBaytlari,
    ) -> Result<[PathBuf; 4], RegistryHatasi> {
        let arsiv = self.nesneyi_yaz(&baytlar.arsiv, "kaynak paketi")?;
        let sbom = self.nesneyi_yaz(&baytlar.sbom, "SBOM")?;
        let provenance = self.nesneyi_yaz(&baytlar.provenance, "provenance")?;
        let yayin = self.nesneyi_yaz(&baytlar.yayin, "yayın bildirimi")?;
        for (dosya, yol) in [
            (&hedef.arsiv, &arsiv),
            (&hedef.sbom, &sbom),
            (&hedef.provenance, &provenance),
            (&hedef.yayin, &yayin),
        ] {
            let beklenen = nesne_yolu(&self.cache_koku, &dosya.sha256)?;
            if yol != &beklenen {
                return Err(RegistryHatasi::tasima(
                    "Cache nesnesi targets özeti dışında bir adrese yazıldı.",
                ));
            }
        }
        Ok([arsiv, sbom, provenance, yayin])
    }

    fn yayin_baytlarini_cacheden_oku(
        &self,
        hedef: &HedefMetadata,
    ) -> Result<YayinBaytlari, RegistryHatasi> {
        Ok(YayinBaytlari {
            arsiv: self.hedef_nesnesini_oku(&hedef.arsiv, "kaynak paketi")?,
            sbom: self.hedef_nesnesini_oku(&hedef.sbom, "SBOM")?,
            provenance: self.hedef_nesnesini_oku(&hedef.provenance, "provenance")?,
            yayin: self.hedef_nesnesini_oku(&hedef.yayin, "yayın bildirimi")?,
        })
    }

    fn hedef_nesnesini_oku(
        &self,
        dosya: &YayinDosyasi,
        ad: &str,
    ) -> Result<Vec<u8>, RegistryHatasi> {
        let azami = usize::try_from(dosya.boyut)
            .map_err(|_| RegistryHatasi::tasima(format!("{} boyutu temsil edilemiyor.", ad)))?;
        let baytlar = self.nesneyi_oku_azami(&dosya.sha256, azami, ad)?;
        hedef_baytlarini_dogrula(dosya, &baytlar, ad)?;
        Ok(baytlar)
    }

    fn nesneyi_yaz(&self, baytlar: &[u8], ad: &str) -> Result<PathBuf, RegistryHatasi> {
        let ozet = sha256_hex(baytlar);
        let yol = nesne_yolu(&self.cache_koku, &ozet)?;
        if yol.exists() {
            let bulunan = sinirli_dosya_oku(&yol, baytlar.len(), Some(baytlar.len() as u64), ad)?;
            if bulunan != baytlar {
                return Err(RegistryHatasi::tasima(format!(
                    "Bozuk {} cache nesnesi aynı SHA-256 adresini taşıyor.",
                    ad
                )));
            }
            return Ok(yol);
        }
        let ebeveyn = yol
            .parent()
            .ok_or_else(|| RegistryHatasi::tasima("Cache nesnesinin üst klasörü yok."))?;
        std::fs::create_dir_all(ebeveyn).map_err(|hata| {
            RegistryHatasi::tasima(format!("Cache klasörü oluşturulamadı: {}.", hata))
        })?;
        if let Err(hata) = crate::kalici_dosya::atomik_yaz(&yol, baytlar) {
            if !yol.exists() {
                return Err(RegistryHatasi::tasima(format!(
                    "{} cache nesnesi yazılamadı: {}.",
                    ad, hata
                )));
            }
        }
        let bulunan = sinirli_dosya_oku(&yol, baytlar.len(), Some(baytlar.len() as u64), ad)?;
        if bulunan != baytlar {
            return Err(RegistryHatasi::tasima(format!(
                "Yazılan {} cache nesnesi yeniden doğrulanamadı.",
                ad
            )));
        }
        let mut izin = std::fs::metadata(&yol)
            .map_err(|hata| RegistryHatasi::tasima(format!("Cache izni okunamadı: {}.", hata)))?
            .permissions();
        izin.set_readonly(true);
        std::fs::set_permissions(&yol, izin).map_err(|hata| {
            RegistryHatasi::tasima(format!("Cache nesnesi salt okunur yapılamadı: {}.", hata))
        })?;
        Ok(yol)
    }

    fn nesneyi_oku_azami(
        &self,
        ozet: &str,
        azami: usize,
        ad: &str,
    ) -> Result<Vec<u8>, RegistryHatasi> {
        let yol = nesne_yolu(&self.cache_koku, ozet)?;
        let baytlar = sinirli_dosya_oku(&yol, azami, None, ad).map_err(|hata| {
            RegistryHatasi::tasima(format!("Çevrimdışı cache miss/bozuk nesne: {}", hata.mesaj))
        })?;
        if sha256_hex(&baytlar) != ozet {
            return Err(RegistryHatasi::tasima(format!(
                "Bozuk {} cache nesnesi SHA-256 adresiyle uyuşmuyor.",
                ad
            )));
        }
        Ok(baytlar)
    }

    fn durumu_oku(
        &self,
    ) -> Result<(Option<KaliciRegistryDurumu>, Option<Vec<u8>>), RegistryHatasi> {
        let yol = self.cache_koku.join(DURUM_DOSYASI);
        let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .metadata()
            .toplam_bayti();
        let baytlar = match sinirli_dosya_oku(&yol, azami, None, "registry durumu") {
            Ok(baytlar) => baytlar,
            Err(hata) if !yol.exists() => return Ok((None, None)),
            Err(hata) => return Err(hata),
        };
        let durum: KaliciRegistryDurumu = serde_json::from_slice(&baytlar).map_err(|hata| {
            RegistryHatasi::tasima(format!("Kalıcı registry durumu okunamadı: {}.", hata))
        })?;
        if durum.sema != DURUM_SEMASI || durum.dogrulama_zamani < 0 {
            return Err(RegistryHatasi::tasima(
                "Kalıcı registry durum şeması veya doğrulama zamanı geçersiz.",
            ));
        }
        let kanonik = kanonik_json(&durum, "registry durumu")?;
        if kanonik != baytlar {
            return Err(RegistryHatasi::tasima(
                "Kalıcı registry durumu kanonik JSON değil.",
            ));
        }
        Ok((Some(durum), Some(baytlar)))
    }

    fn durumu_yaz(
        &self,
        durum: &KaliciRegistryDurumu,
        beklenen: Option<&[u8]>,
    ) -> Result<(), RegistryHatasi> {
        std::fs::create_dir_all(&self.cache_koku).map_err(|hata| {
            RegistryHatasi::tasima(format!("Registry cache klasörü oluşturulamadı: {}.", hata))
        })?;
        let baytlar = kanonik_json(durum, "registry durumu")?;
        crate::kalici_dosya::atomik_karsilastir_ve_yaz(
            &self.cache_koku.join(DURUM_DOSYASI),
            beklenen,
            &baytlar,
        )
        .map_err(|hata| {
            RegistryHatasi::tasima(format!(
                "Kalıcı registry durumu atomik güncellenemedi: {}.",
                hata
            ))
        })
    }
}

fn zorunlu_getir<T: RegistryTasiyici>(
    tasiyici: &mut T,
    yol: &str,
    azami: usize,
) -> Result<Vec<u8>, RegistryHatasi> {
    tasiyici
        .getir(yol, azami)?
        .ok_or_else(|| RegistryHatasi::tasima(format!("Registry nesnesi bulunamadı: {}.", yol)))
}

fn hedef_yollari(cache: &Path, hedef: &HedefMetadata) -> Result<[PathBuf; 4], RegistryHatasi> {
    Ok([
        nesne_yolu(cache, &hedef.arsiv.sha256)?,
        nesne_yolu(cache, &hedef.sbom.sha256)?,
        nesne_yolu(cache, &hedef.provenance.sha256)?,
        nesne_yolu(cache, &hedef.yayin.sha256)?,
    ])
}

fn cikti(
    hedef: &HedefMetadata,
    durum: &KaliciRegistryDurumu,
    yollar: [PathBuf; 4],
    cevrimdisi: bool,
    kritik_duyurular: Vec<String>,
) -> RegistryPaketCiktisi {
    let [arsiv, sbom, provenance, yayin] = yollar;
    RegistryPaketCiktisi {
        paket: hedef.paket.clone(),
        surum: hedef.surum.clone(),
        sabit_kok_sha256: durum.sabit_kok_sha256.clone(),
        metadata_surumleri: durum.metadata.clone(),
        yayinci_anahtar_kimligi: hedef.yayinci_anahtar_kimligi.clone(),
        arsiv_sha256: hedef.arsiv.sha256.clone(),
        sbom_sha256: hedef.sbom.sha256.clone(),
        provenance_sha256: hedef.provenance.sha256.clone(),
        yayin_sha256: hedef.yayin.sha256.clone(),
        yanked: hedef.yanked,
        kritik_duyurular,
        arsiv,
        sbom,
        provenance,
        yayin,
        cevrimdisi,
    }
}

fn simdiki_unix_saniyesi() -> Result<i64, RegistryHatasi> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| RegistryHatasi::tasima("Sistem saati Unix başlangıcından önce."))?
        .as_secs()
        .try_into()
        .map_err(|_| RegistryHatasi::tasima("Sistem saati i64 saniye sınırını aşıyor."))
}

#[cfg(test)]
mod testler;
