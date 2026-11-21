//! Kalıcı registry durumunun okunması ve K-361 kilit içi uzlaştırması.
//!
//! Çevrimiçi doğrulama ağ gerektirdiğinden durum kilit dışında okunur;
//! kilit ağ süresince tutulamaz (spec/08 5 sn bekleme sınırı). Yayın ise
//! tek CAS değildir: kilit altında güncel durum yeniden okunur ve zincir
//! sürümleriyle karşılaştırılır.

use super::depo::{kanonik_json, sinirli_dosya_oku};
use super::{KaliciRegistryDurumu, RegistryHatasi, RegistryIstemcisi, DURUM_DOSYASI, DURUM_SEMASI};
use crate::kalici_dosya::{kilit_altinda_guncelle, KilitliGuncellemeHatasi};
use std::cmp::Ordering;

/// Kilit içi yayının sonucu.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Uzlasma {
    /// Yeni durum yayımlandı (başlangıç durumu yerindeydi ya da yeni zincir ileri).
    Yazildi,
    /// Araya giren yazar aynı zinciri yayımlamış; dosyaya dokunulmadı.
    Ayni,
    /// Araya giren yazar daha yeni bir zincir yayımlamış; durum korunur ve
    /// sonuç bu kalıcı durumla yeniden doğrulanmalıdır.
    Geride,
}

impl RegistryIstemcisi {
    pub(super) fn durumu_oku(
        &self,
    ) -> Result<(Option<KaliciRegistryDurumu>, Option<Vec<u8>>), RegistryHatasi> {
        let yol = self.cache_koku.join(DURUM_DOSYASI);
        let baytlar = match sinirli_dosya_oku(&yol, durum_siniri(), None, "registry durumu") {
            Ok(baytlar) => baytlar,
            Err(_) if !yol.exists() => return Ok((None, None)),
            Err(hata) => return Err(hata),
        };
        Ok((Some(durumu_coz(&baytlar)?), Some(baytlar)))
    }

    /// K-361: başlangıçta okunan `beklenen` byte'lar yerindeyse yeni durumu
    /// yazar. Araya başka bir güncelleme girdiyse (eski tek CAS burada P016
    /// ile düşüyordu) güncel durumu aynı kilit penceresinde ayrıştırır:
    /// aynı zincir dokunulmadan, ileri zincir yazılarak kabul edilir; daha
    /// yeni kalıcı durum asla eskisiyle ezilmez (rollback koruması). Aynı
    /// sürümde farklı özet ya da karışık sıra çelişkidir: P013, durum korunur.
    pub(super) fn durumu_uzlastir(
        &self,
        yeni: &KaliciRegistryDurumu,
        beklenen: Option<&[u8]>,
    ) -> Result<Uzlasma, RegistryHatasi> {
        std::fs::create_dir_all(&self.cache_koku).map_err(|hata| {
            RegistryHatasi::tasima(format!("Registry cache klasörü oluşturulamadı: {}.", hata))
        })?;
        let baytlar = kanonik_json(yeni, "registry durumu")?;
        let sonuc = kilit_altinda_guncelle(&self.cache_koku.join(DURUM_DOSYASI), |guncel| {
            let Some(guncel) = guncel.filter(|guncel| Some(guncel.as_slice()) != beklenen) else {
                return Ok((baytlar, Uzlasma::Yazildi));
            };
            if guncel.len() > durum_siniri() {
                return Err(Yazimsiz::Hata(RegistryHatasi::tasima(
                    "Kalıcı registry durumu boyut sınırını aşıyor.",
                )));
            }
            let mevcut = durumu_coz(&guncel).map_err(Yazimsiz::Hata)?;
            if mevcut.sabit_kok_sha256 != yeni.sabit_kok_sha256 {
                return Err(Yazimsiz::Hata(RegistryHatasi::tasima(
                    "Registry cache'i farklı bir ağ dışı root sabitlemesine ait.",
                )));
            }
            match zincir_sirasi(yeni, &mevcut).map_err(Yazimsiz::Hata)? {
                Ordering::Greater => Ok((baytlar, Uzlasma::Yazildi)),
                Ordering::Equal => Err(Yazimsiz::Sonuc(Uzlasma::Ayni)),
                Ordering::Less => Err(Yazimsiz::Sonuc(Uzlasma::Geride)),
            }
        });
        match sonuc {
            Ok(uzlasma) | Err(KilitliGuncellemeHatasi::Islem(Yazimsiz::Sonuc(uzlasma))) => {
                Ok(uzlasma)
            }
            Err(KilitliGuncellemeHatasi::Islem(Yazimsiz::Hata(hata))) => Err(hata),
            Err(KilitliGuncellemeHatasi::Okuma(hata) | KilitliGuncellemeHatasi::Yazma(hata)) => {
                Err(RegistryHatasi::tasima(format!(
                    "Kalıcı registry durumu atomik güncellenemedi: {}.",
                    hata
                )))
            }
        }
    }
}

/// `kilit_altinda_guncelle` yalnız dönüşüm hata dalında yayını keser;
/// dosyaya dokunmayan uzlaşma sonuçları bu kanaldan çıkar.
enum Yazimsiz {
    Sonuc(Uzlasma),
    Hata(RegistryHatasi),
}

fn durum_siniri() -> usize {
    crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
        .metadata()
        .toplam_bayti()
}

fn durumu_coz(baytlar: &[u8]) -> Result<KaliciRegistryDurumu, RegistryHatasi> {
    let durum: KaliciRegistryDurumu = serde_json::from_slice(baytlar).map_err(|hata| {
        RegistryHatasi::tasima(format!("Kalıcı registry durumu okunamadı: {}.", hata))
    })?;
    if durum.sema != DURUM_SEMASI || durum.dogrulama_zamani < 0 {
        return Err(RegistryHatasi::tasima(
            "Kalıcı registry durum şeması veya doğrulama zamanı geçersiz.",
        ));
    }
    if kanonik_json(&durum, "registry durumu")? != baytlar {
        return Err(RegistryHatasi::tasima(
            "Kalıcı registry durumu kanonik JSON değil.",
        ));
    }
    Ok(durum)
}

/// İki kalıcı durumun zincir sırası: root, timestamp, snapshot ve targets
/// rollerinin her biri sürüm ve özetle karşılaştırılır. Aynı sürümde farklı
/// özet ya da bir rolde ileri, ötekinde geri sıra çelişkidir.
pub(super) fn zincir_sirasi(
    yeni: &KaliciRegistryDurumu,
    mevcut: &KaliciRegistryDurumu,
) -> Result<Ordering, RegistryHatasi> {
    let (a, b) = (&yeni.metadata, &mevcut.metadata);
    let roller = [
        (
            "root",
            a.root,
            &yeni.etkin_kok_sha256,
            b.root,
            &mevcut.etkin_kok_sha256,
        ),
        (
            "timestamp",
            a.timestamp,
            &a.timestamp_sha256,
            b.timestamp,
            &b.timestamp_sha256,
        ),
        (
            "snapshot",
            a.snapshot,
            &a.snapshot_sha256,
            b.snapshot,
            &b.snapshot_sha256,
        ),
        (
            "targets",
            a.targets,
            &a.targets_sha256,
            b.targets,
            &b.targets_sha256,
        ),
    ];
    let mut sira = Ordering::Equal;
    for (rol, yeni_surum, yeni_ozet, mevcut_surum, mevcut_ozet) in roller {
        let rol_sirasi = yeni_surum.cmp(&mevcut_surum);
        let celiski = match rol_sirasi {
            Ordering::Equal => yeni_ozet != mevcut_ozet,
            _ => sira != Ordering::Equal && sira != rol_sirasi,
        };
        if celiski {
            return Err(RegistryHatasi::metadata(format!(
                "Eşzamanlı registry güncellemesi çelişkili {} metadata'sı gördü \
                 (sürüm {} / {}); kalıcı durum korunuyor.",
                rol, yeni_surum, mevcut_surum
            )));
        }
        if rol_sirasi != Ordering::Equal {
            sira = rol_sirasi;
        }
    }
    Ok(sira)
}
