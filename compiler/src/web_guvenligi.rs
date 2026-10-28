//! K-088 sunucu tarafı oturum, CSRF ve yetki çekirdeği.
//!
//! Depo yalnız belirteç özetini saklar. Anonim oturum form CSRF'si üretir;
//! başarılı giriş aynı oturumu kullanmaz, yeni kimlik ve CSRF ile döndürür.

use crate::agac::RotaErisimi;
use crate::guvenlik::sabit_zamanli_esit;
use std::collections::BTreeSet;

mod depo;
mod oran_bellek;
pub use oran_bellek::{BellekOranAyari, OranDoluluk};
mod oturum_yonetimi;
use depo::{Depo, DepoDurumu};

pub const OTURUM_OMRU_SANIYE: i64 = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .web()
    .oturum_omru_saniye();
pub const ANONIM_OTURUM_OMRU_SANIYE: i64 = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .web()
    .anonim_oturum_omru_saniye();
pub const AZAMI_OTURUM_SAYISI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .web()
    .oturum_sayisi();
pub const AZAMI_ANONIM_OTURUM_SAYISI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .web()
    .anonim_oturum_sayisi();

/// Kayar yenilemenin aşamayacağı, oluşturmadan sayılan mutlak oturum tavanı (30 gün).
pub const OTURUM_MUTLAK_TAVAN_SANIYE: i64 = 30 * 24 * 60 * 60;
/// Proje ayarıyla seçilebilecek en kısa kimlikli oturum ömrü (5 dakika).
pub const OTURUM_OMRU_ALT_SANIYE: i64 = 5 * 60;

/// K-330: kimlikli oturumun ömrü ve kayar yenilemesi. Anonim (giriş öncesi)
/// oturum bu ayardan etkilenmez; her zaman kısa ve mutlaktır.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OturumAyari {
    omur_saniye: i64,
    kayar: bool,
}

impl Default for OturumAyari {
    fn default() -> Self {
        Self {
            omur_saniye: OTURUM_OMRU_SANIYE,
            kayar: false,
        }
    }
}

impl OturumAyari {
    /// Sınır dışı ömür fail-closed reddedilir; proje bildirimi aynı aralığı tanıyla denetler.
    pub fn yeni(omur_saniye: i64, kayar: bool) -> Result<Self, String> {
        if !(OTURUM_OMRU_ALT_SANIYE..=OTURUM_MUTLAK_TAVAN_SANIYE).contains(&omur_saniye) {
            return Err(format!(
                "oturum ömrü {omur_saniye} sn; {OTURUM_OMRU_ALT_SANIYE}–{OTURUM_MUTLAK_TAVAN_SANIYE} sn aralığında olmalı"
            ));
        }
        Ok(Self { omur_saniye, kayar })
    }

    pub const fn omur_saniye(self) -> i64 {
        self.omur_saniye
    }

    pub const fn kayar(self) -> bool {
        self.kayar
    }

    /// Kayar yenilemede yeni son geçerlilik: `şimdi + ömür`, ama oluşturmadan
    /// sayılan mutlak tavanı asla aşmaz ve var olan geçerliliği kısaltmaz.
    fn kaydir(self, oturum: &Oturum, an_ms: i64) -> i64 {
        let tavan = oturum
            .olusturma_ms
            .saturating_add(OTURUM_MUTLAK_TAVAN_SANIYE * 1000);
        an_ms
            .saturating_add(self.omur_saniye * 1000)
            .min(tavan)
            .max(oturum.son_gecerlilik_ms)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebReddi {
    pub durum: u16,
    pub mesaj: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YeniOturum {
    pub belirtec: String,
    pub azami_omur_saniye: i64,
}

/// Uygulamaya açılabilen, sır taşımayan oturum görünümü. `kimlik`, ham çerez
/// belirteci değil depodaki SHA-256 özetidir ve yalnız iptal hedefi olarak
/// kullanılmalıdır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OturumBilgisi {
    pub kimlik: String,
    pub kullanici: String,
    pub rol: String,
    pub olusturma_ms: i64,
    pub son_erisim_ms: i64,
    pub son_gecerlilik_ms: i64,
}

pub use oturum_yonetimi::oturum_satiri;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub(super) struct Oturum {
    pub(super) csrf: String,
    pub(super) kullanici: Option<String>,
    pub(super) rol: Option<String>,
    pub(super) son_gecerlilik_ms: i64,
    pub(super) son_erisim_ms: i64,
    #[cfg_attr(not(target_arch = "wasm32"), serde(default))]
    pub(super) olusturma_ms: i64,
    pub(super) olusturma_sirasi: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub(super) struct OranKaydi {
    pub(super) pencere_sonu_ms: i64,
    pub(super) sayi: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitTuru {
    Ucnokta,
    Csrf,
    Giris,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitKarari {
    pub izinli: bool,
    pub yeniden_dene_saniye: i64,
}

#[derive(Clone, Default)]
struct BekleyenOturumIslemi {
    silinecekler: BTreeSet<String>,
    eklenecekler: Vec<(String, Oturum)>,
}

pub(super) struct OturumGeriAlmaKaydi {
    pub(super) anahtar: String,
    pub(super) onceki: Option<Oturum>,
    pub(super) beklenen: Option<Oturum>,
}

pub(super) struct OturumGeriAlma {
    pub(super) kayitlar: Vec<OturumGeriAlmaKaydi>,
}

pub struct WebOturumCommit {
    geri: Option<OturumGeriAlma>,
}

#[derive(Clone)]
pub struct WebGuvenligi {
    depo: Depo,
    oran_bellek: Option<oran_bellek::OranBellek>,
    gelen_anahtar: Option<String>,
    gelen_oturum: Option<Oturum>,
    istemci_kimligi: Option<String>,
    bekleyen: BekleyenOturumIslemi,
    azami_oturum: usize,
    azami_anonim_oturum: usize,
    oturum_ayari: OturumAyari,
    /// Bu istekte kayar yenileme yapıldıysa çerezin yeni Max-Age değeri.
    kayar_cerez_omru: Option<i64>,
}

impl Default for WebGuvenligi {
    fn default() -> Self {
        Self {
            depo: Depo::default(),
            oran_bellek: None,
            gelen_anahtar: None,
            gelen_oturum: None,
            istemci_kimligi: None,
            bekleyen: BekleyenOturumIslemi::default(),
            azami_oturum: AZAMI_OTURUM_SAYISI,
            azami_anonim_oturum: AZAMI_ANONIM_OTURUM_SAYISI,
            oturum_ayari: OturumAyari::default(),
            kayar_cerez_omru: None,
        }
    }
}

impl WebGuvenligi {
    pub fn yeni() -> Self {
        Self::default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn kalici(yol: impl Into<std::path::PathBuf>) -> Self {
        Self {
            depo: Depo::kalici(yol.into()),
            ..Self::default()
        }
    }

    /// Oturum/revoke kalıcı ve ortak; oran sayacı yalnız bu süreç ve clone'ları içindir.
    /// Yeniden başlatma kotayı sıfırlar. Çok worker için ortak proxy kotası gerekir.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn kalici_bellek_oranli(yol: impl Into<std::path::PathBuf>) -> Self {
        Self::kalici_bellek_oran_ayarli(yol, BellekOranAyari::default())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn kalici_bellek_oran_ayarli(
        yol: impl Into<std::path::PathBuf>,
        ayar: BellekOranAyari,
    ) -> Self {
        Self {
            oran_bellek: Some(oran_bellek::OranBellek::yeni(ayar)),
            ..Self::kalici(yol)
        }
    }

    /// Proje bildirimindeki kimlikli oturum ömrü/kayar ayarını uygular.
    pub fn oturum_ayarini_belirle(&mut self, ayar: OturumAyari) {
        self.oturum_ayari = ayar;
    }

    pub fn oturum_ayari(&self) -> OturumAyari {
        self.oturum_ayari
    }

    /// Kayar yenileme bu istekte kimlikli oturumun geçerliliğini uzattıysa
    /// çağıranın aynı belirteçle yeniden yazacağı çerezin Max-Age değeri.
    pub fn kayar_cerez_omru(&self) -> Option<i64> {
        self.kayar_cerez_omru
    }

    #[cfg(test)]
    fn sinirli(azami_oturum: usize, azami_anonim_oturum: usize) -> Self {
        Self {
            azami_oturum,
            azami_anonim_oturum: azami_anonim_oturum.min(azami_oturum),
            ..Self::default()
        }
    }

    /// Her istekten önce çağrılır. Süresi dolmuş kayıt aynı anda iptal edilir;
    /// erişim LRU sırasını günceller. Mutlak kipte geçerlilik anı uzamaz; kayar
    /// kipte yalnız kimlikli oturum `şimdi + ömür`e (mutlak tavanla) taşınır.
    pub fn istegi_baslat_kimlikle(
        &mut self,
        belirtec: Option<&str>,
        istemci_kimligi: &str,
        an_ms: i64,
    ) -> Result<(), String> {
        self.istegi_geri_al();
        self.istemci_kimligi = Some(istemci_kimligi.to_string());
        let gelen_anahtar = belirtec.map(anahtar);
        let aranan = gelen_anahtar.clone();
        let ayar = self.oturum_ayari;
        let mut kayar_cerez_omru = None;
        // Belirteç yoksa bu kipte oturum deposuna dokunmak gerekmez. Temizlik
        // oturum okuma/commit sırasında yapılır; revoke okumaları önbelleklenmez.
        let oturum = if self.oran_bellek.is_some() && aranan.is_none() {
            None
        } else {
            self.depo.guncelle(|durum| {
                durum
                    .oturumlar
                    .retain(|_, oturum| oturum.son_gecerlilik_ms > an_ms);
                if let Some(anahtar) = &aranan {
                    if let Some(oturum) = durum.oturumlar.get_mut(anahtar) {
                        oturum.son_erisim_ms = an_ms;
                        kayar_cerez_omru = None;
                        if ayar.kayar && oturum.kullanici.is_some() {
                            oturum.son_gecerlilik_ms = ayar.kaydir(oturum, an_ms);
                            kayar_cerez_omru =
                                Some(oturum.son_gecerlilik_ms.saturating_sub(an_ms) / 1000);
                        }
                        return Ok(Some(oturum.clone()));
                    }
                }
                Ok(None)
            })?
        };
        self.gelen_anahtar = gelen_anahtar;
        self.gelen_oturum = oturum;
        self.kayar_cerez_omru = kayar_cerez_omru;
        Ok(())
    }

    /// Form GET'inde synchronizer token verir; oturum yoksa kısa ömürlü anonim
    /// oturum oluşturup çağıranın güvenli çerez yazmasını ister.
    pub fn csrf_belirteci<F>(
        &mut self,
        an_ms: i64,
        mut uret: F,
    ) -> Result<(String, Option<YeniOturum>), String>
    where
        F: FnMut() -> Result<String, String>,
    {
        if let Some(oturum) = self.gecerli_oturum(an_ms) {
            return Ok((oturum.csrf.clone(), None));
        }
        self.oturum_yerini_denetle(true)?;
        let belirtec = uret()?;
        let csrf = uret()?;
        let omur = ANONIM_OTURUM_OMRU_SANIYE;
        let oturum = Oturum {
            csrf: csrf.clone(),
            kullanici: None,
            rol: None,
            son_gecerlilik_ms: an_ms.saturating_add(omur * 1000),
            son_erisim_ms: an_ms,
            olusturma_ms: an_ms,
            olusturma_sirasi: 0,
        };
        let anahtar = anahtar(&belirtec);
        self.bekleyen
            .eklenecekler
            .push((anahtar.clone(), oturum.clone()));
        self.gelen_anahtar = Some(anahtar);
        self.gelen_oturum = Some(oturum);
        Ok((
            csrf,
            Some(YeniOturum {
                belirtec,
                azami_omur_saniye: omur,
            }),
        ))
    }

    pub fn denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
        an_ms: i64,
    ) -> Result<(), WebReddi> {
        let oturum = self.gecerli_oturum(an_ms);
        if csrf_gerekli {
            let Some(oturum) = oturum else {
                return Err(WebReddi {
                    durum: 403,
                    mesaj: "CSRF doğrulaması için geçerli form oturumu yok",
                });
            };
            let Some(gelen) = csrf else {
                return Err(WebReddi {
                    durum: 403,
                    mesaj: "CSRF belirteci eksik",
                });
            };
            if !sabit_zamanli_esit(gelen, &oturum.csrf) {
                return Err(WebReddi {
                    durum: 403,
                    mesaj: "CSRF belirteci geçersiz",
                });
            }
        }

        match erisim {
            RotaErisimi::HerkeseAcik => Ok(()),
            RotaErisimi::Oturumlu => match oturum {
                Some(oturum) if oturum.kullanici.is_some() => Ok(()),
                _ => Err(WebReddi {
                    durum: 401,
                    mesaj: "bu rota kimliği doğrulanmış oturum istiyor",
                }),
            },
            RotaErisimi::Rol(istenen) => match oturum {
                Some(oturum) if oturum.kullanici.is_none() => Err(WebReddi {
                    durum: 401,
                    mesaj: "bu rota kimliği doğrulanmış oturum istiyor",
                }),
                Some(oturum) if oturum.rol.as_deref() == Some(istenen.as_str()) => Ok(()),
                Some(_) => Err(WebReddi {
                    durum: 403,
                    mesaj: "oturum bu rota için gerekli role sahip değil",
                }),
                None => Err(WebReddi {
                    durum: 401,
                    mesaj: "bu rota kimliği doğrulanmış oturum istiyor",
                }),
            },
        }
    }

    /// Girişte session fixation'ı önlemek için anonim/eski oturumu siler ve
    /// hem oturum kimliğini hem CSRF belirtecini döndürür.
    pub fn oturum_ac<F>(
        &mut self,
        kullanici: String,
        rol: String,
        an_ms: i64,
        mut uret: F,
    ) -> Result<YeniOturum, String>
    where
        F: FnMut() -> Result<String, String>,
    {
        self.gecerli_oturumu_sil();
        self.oturum_yerini_denetle(false)?;
        let belirtec = uret()?;
        let csrf = uret()?;
        let omur = self.oturum_ayari.omur_saniye;
        let oturum = Oturum {
            csrf,
            kullanici: Some(kullanici),
            rol: Some(rol),
            son_gecerlilik_ms: an_ms.saturating_add(omur * 1000),
            son_erisim_ms: an_ms,
            olusturma_ms: an_ms,
            olusturma_sirasi: 0,
        };
        let anahtar = anahtar(&belirtec);
        self.bekleyen
            .eklenecekler
            .push((anahtar.clone(), oturum.clone()));
        self.gelen_anahtar = Some(anahtar);
        self.gelen_oturum = Some(oturum);
        Ok(YeniOturum {
            belirtec,
            azami_omur_saniye: omur,
        })
    }

    pub fn oturum_kapat(&mut self) {
        self.gecerli_oturumu_sil();
        self.gelen_anahtar = None;
        self.gelen_oturum = None;
    }

    pub fn oturum_sayisi(&self) -> Result<usize, String> {
        self.depo.guncelle(|durum| {
            let mut gorunum = durum.clone();
            for anahtar in &self.bekleyen.silinecekler {
                gorunum.oturumlar.remove(anahtar);
            }
            for (anahtar, oturum) in &self.bekleyen.eklenecekler {
                oturum_yeri_ac(
                    &mut gorunum,
                    oturum.kullanici.is_none(),
                    self.azami_oturum,
                    self.azami_anonim_oturum,
                )?;
                gorunum.oturumlar.insert(anahtar.clone(), oturum.clone());
            }
            Ok(gorunum.oturumlar.len())
        })
    }

    fn gecerli_oturum(&self, an_ms: i64) -> Option<&Oturum> {
        self.gelen_oturum
            .as_ref()
            .filter(|oturum| oturum.son_gecerlilik_ms > an_ms)
    }

    fn gecerli_oturumu_sil(&mut self) {
        self.kayar_cerez_omru = None;
        if let Some(anahtar) = &self.gelen_anahtar {
            let onceki = self.bekleyen.eklenecekler.len();
            self.bekleyen
                .eklenecekler
                .retain(|(aday, _)| aday != anahtar);
            if self.bekleyen.eklenecekler.len() == onceki {
                self.bekleyen.silinecekler.insert(anahtar.clone());
            }
        }
        self.gelen_oturum = None;
    }

    fn oturum_yerini_denetle(&self, anonim: bool) -> Result<(), String> {
        self.depo.guncelle(|durum| {
            let mut gorunum = durum.clone();
            for anahtar in &self.bekleyen.silinecekler {
                gorunum.oturumlar.remove(anahtar);
            }
            for (anahtar, oturum) in &self.bekleyen.eklenecekler {
                oturum_yeri_ac(
                    &mut gorunum,
                    oturum.kullanici.is_none(),
                    self.azami_oturum,
                    self.azami_anonim_oturum,
                )?;
                gorunum.oturumlar.insert(anahtar.clone(), oturum.clone());
            }
            oturum_yeri_ac(
                &mut gorunum,
                anonim,
                self.azami_oturum,
                self.azami_anonim_oturum,
            )
        })
    }

    pub fn istegi_tamamla(&mut self, an_ms: i64) -> Result<WebOturumCommit, String> {
        let bekleyen = std::mem::take(&mut self.bekleyen);
        if bekleyen.silinecekler.is_empty() && bekleyen.eklenecekler.is_empty() {
            return Ok(WebOturumCommit { geri: None });
        }
        let azami_oturum = self.azami_oturum;
        let azami_anonim = self.azami_anonim_oturum;
        let geri = self.depo.oturum_islemlerini_uygula(|durum| {
            durum
                .oturumlar
                .retain(|_, oturum| oturum.son_gecerlilik_ms > an_ms);
            for anahtar in &bekleyen.silinecekler {
                durum.oturumlar.remove(anahtar);
            }
            for (anahtar, mut oturum) in bekleyen.eklenecekler.clone() {
                oturum_yeri_ac(
                    durum,
                    oturum.kullanici.is_none(),
                    azami_oturum,
                    azami_anonim,
                )?;
                oturum.olusturma_sirasi = durum.sonraki_olusturma_sirasi;
                durum.sonraki_olusturma_sirasi = durum.sonraki_olusturma_sirasi.saturating_add(1);
                durum.oturumlar.insert(anahtar, oturum);
            }
            Ok(())
        })?;
        Ok(WebOturumCommit { geri: Some(geri) })
    }

    pub fn commiti_geri_al(&mut self, mut commit: WebOturumCommit) -> Result<(), String> {
        match commit.geri.take() {
            Some(geri) => self.depo.oturum_islemlerini_geri_al(geri),
            None => Ok(()),
        }
    }

    pub fn istegi_geri_al(&mut self) {
        self.kayar_cerez_omru = None;
        self.bekleyen = BekleyenOturumIslemi::default();
        self.gelen_anahtar = None;
        self.gelen_oturum = None;
        self.istemci_kimligi = None;
    }

    pub fn rate_limit_artir(
        &self,
        tur: RateLimitTuru,
        kapsam: &str,
        an_ms: i64,
    ) -> Result<RateLimitKarari, String> {
        let istemci = self.istemci_kimligi.as_deref().unwrap_or("bilinmeyen");
        let (etiket, sinir, pencere_saniye) = rate_limit_profili(tur);
        let ham = format!(
            "zee-rate-1\0{}:{}\0{}:{}\0{}:{}",
            etiket.len(),
            etiket,
            istemci.len(),
            istemci,
            kapsam.len(),
            kapsam
        );
        let anahtar = crate::guvenlik::sha256_hex(ham.as_bytes());
        let pencere_ms = pencere_saniye.saturating_mul(1_000);
        let azami_anahtar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .web()
            .oran_anahtari_sayisi();
        if let Some(depo) = &self.oran_bellek {
            return depo.artir(anahtar, an_ms, pencere_ms, sinir, tur);
        }
        self.depo.guncelle(|durum| {
            durum
                .oranlar
                .retain(|_, kayit| kayit.pencere_sonu_ms > an_ms);
            if !durum.oranlar.contains_key(&anahtar) && durum.oranlar.len() >= azami_anahtar {
                return Err(
                    "web oran sınırı anahtar kapasitesi dolu; yeni kimlik fail-closed reddedildi"
                        .into(),
                );
            }
            let kayit = durum.oranlar.entry(anahtar.clone()).or_insert(OranKaydi {
                pencere_sonu_ms: an_ms.saturating_add(pencere_ms),
                sayi: 0,
            });
            kayit.sayi = kayit.sayi.saturating_add(1);
            Ok(RateLimitKarari {
                izinli: kayit.sayi <= sinir,
                yeniden_dene_saniye: kayit
                    .pencere_sonu_ms
                    .saturating_sub(an_ms)
                    .saturating_add(999)
                    / 1_000,
            })
        })
    }
}

fn rate_limit_profili(tur: RateLimitTuru) -> (&'static str, u32, i64) {
    let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.web();
    match tur {
        RateLimitTuru::Ucnokta => (
            "uçnokta",
            sinirlar.ucnokta_orani(),
            sinirlar.ucnokta_penceresi_saniye(),
        ),
        RateLimitTuru::Csrf => (
            "csrf",
            sinirlar.csrf_orani(),
            sinirlar.csrf_penceresi_saniye(),
        ),
        RateLimitTuru::Giris => (
            "giriş",
            sinirlar.giris_orani(),
            sinirlar.giris_penceresi_saniye(),
        ),
    }
}

fn oturum_yeri_ac(
    durum: &mut DepoDurumu,
    anonim: bool,
    azami_oturum: usize,
    azami_anonim: usize,
) -> Result<(), String> {
    if anonim && anonim_oturum_sayisi(durum) >= azami_anonim && !en_eski_anonimi_sil(durum) {
        return Err("anonim web oturumu kapasitesi dolu".into());
    }
    while durum.oturumlar.len() >= azami_oturum {
        if !en_eski_anonimi_sil(durum) {
            return Err("web oturumu kapasitesi dolu; yeni giriş reddedildi".into());
        }
    }
    Ok(())
}

fn anonim_oturum_sayisi(durum: &DepoDurumu) -> usize {
    durum
        .oturumlar
        .values()
        .filter(|oturum| oturum.kullanici.is_none())
        .count()
}

fn en_eski_anonimi_sil(durum: &mut DepoDurumu) -> bool {
    let aday = durum
        .oturumlar
        .iter()
        .filter(|(_, oturum)| oturum.kullanici.is_none())
        .min_by_key(|(_, oturum)| (oturum.son_erisim_ms, oturum.olusturma_sirasi))
        .map(|(anahtar, _)| anahtar.clone());
    aday.map(|anahtar| durum.oturumlar.remove(&anahtar).is_some())
        .unwrap_or(false)
}

fn anahtar(belirtec: &str) -> String {
    crate::guvenlik::sha256_hex(belirtec.as_bytes())
}

pub fn cerez_adi_gecerli(ad: &str) -> bool {
    !ad.is_empty()
        && ad
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

pub fn cerez_degeri_gecerli(deger: &str) -> bool {
    deger.is_ascii()
        && !deger
            .bytes()
            .any(|b| b <= 0x20 || b == 0x7f || matches!(b, b';' | b',' | b'"' | b'\\'))
}

pub fn yerel_yonlendirme_gecerli(adres: &str) -> bool {
    adres.starts_with('/')
        && !adres.starts_with("//")
        && !adres.bytes().any(|b| b == b'\r' || b == b'\n')
}

#[cfg(test)]
mod tests;
