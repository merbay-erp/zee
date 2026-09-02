//! Oturum runtime'ını saklama biçiminden ayıran depo sınırı.

use super::{OranKaydi, Oturum, OturumGeriAlma, OturumGeriAlmaKaydi};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
mod kalici;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub(super) struct DepoDurumu {
    pub(super) bicim_surumu: u32,
    pub(super) degisiklik_sirasi: u64,
    pub(super) sonraki_olusturma_sirasi: u64,
    pub(super) oturumlar: BTreeMap<String, Oturum>,
    pub(super) oranlar: BTreeMap<String, OranKaydi>,
}

impl Default for DepoDurumu {
    fn default() -> Self {
        Self {
            bicim_surumu: 1,
            degisiklik_sirasi: 0,
            sonraki_olusturma_sirasi: 0,
            oturumlar: BTreeMap::new(),
            oranlar: BTreeMap::new(),
        }
    }
}

#[derive(Clone)]
pub(super) enum Depo {
    ProcessIci(Arc<Mutex<DepoDurumu>>),
    #[cfg(not(target_arch = "wasm32"))]
    Kalici(Arc<kalici::KaliciDepo>),
}

impl Default for Depo {
    fn default() -> Self {
        Self::ProcessIci(Arc::new(Mutex::new(DepoDurumu::default())))
    }
}

impl Depo {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn kalici(yol: std::path::PathBuf) -> Self {
        Self::Kalici(Arc::new(kalici::KaliciDepo::yeni(yol)))
    }

    pub(super) fn guncelle<R, F>(&self, islem: F) -> Result<R, String>
    where
        F: FnMut(&mut DepoDurumu) -> Result<R, String>,
    {
        match self {
            Self::ProcessIci(durum) => {
                let mut durum = durum
                    .lock()
                    .map_err(|_| "process-içi web deposu kilidi zehirlendi".to_string())?;
                let mut islem = islem;
                let sonuc = islem(&mut durum)?;
                durum.degisiklik_sirasi = durum.degisiklik_sirasi.saturating_add(1);
                Ok(sonuc)
            }
            #[cfg(not(target_arch = "wasm32"))]
            Self::Kalici(depo) => depo.guncelle(islem),
        }
    }

    pub(super) fn oturum_islemlerini_uygula<F>(&self, islem: F) -> Result<OturumGeriAlma, String>
    where
        F: FnMut(&mut DepoDurumu) -> Result<(), String>,
    {
        let mut islem = islem;
        self.guncelle(|durum| {
            let onceki = durum.oturumlar.clone();
            islem(durum)?;
            let mut anahtarlar = onceki.keys().cloned().collect::<BTreeSet<_>>();
            anahtarlar.extend(durum.oturumlar.keys().cloned());
            let kayitlar = anahtarlar
                .into_iter()
                .filter_map(|anahtar| {
                    let onceki_deger = onceki.get(&anahtar).cloned();
                    let beklenen = durum.oturumlar.get(&anahtar).cloned();
                    (onceki_deger != beklenen).then_some(OturumGeriAlmaKaydi {
                        anahtar,
                        onceki: onceki_deger,
                        beklenen,
                    })
                })
                .collect();
            Ok(OturumGeriAlma { kayitlar })
        })
    }

    pub(super) fn oturum_islemlerini_geri_al(&self, geri: OturumGeriAlma) -> Result<(), String> {
        if geri.kayitlar.is_empty() {
            return Ok(());
        }
        self.guncelle(|durum| {
            for kayit in &geri.kayitlar {
                if durum.oturumlar.get(&kayit.anahtar) != kayit.beklenen.as_ref() {
                    return Err(
                        "web oturum transaction'ı başka bir süreç aynı kaydı değiştirdiği için geri alınamadı"
                            .into(),
                    );
                }
            }
            for kayit in &geri.kayitlar {
                match &kayit.onceki {
                    Some(oturum) => {
                        durum
                            .oturumlar
                            .insert(kayit.anahtar.clone(), oturum.clone());
                    }
                    None => {
                        durum.oturumlar.remove(&kayit.anahtar);
                    }
                }
            }
            Ok(())
        })
    }
}
