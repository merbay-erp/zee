//! K-088 sunucu tarafı oturum, CSRF ve yetki çekirdeği.
//!
//! Depo yalnız belirteç özetini saklar. Anonim oturum form CSRF'si üretir;
//! başarılı giriş aynı oturumu kullanmaz, yeni kimlik ve CSRF ile döndürür.

use crate::agac::RotaErisimi;
use crate::guvenlik::sabit_zamanli_esit;
use std::collections::BTreeSet;

mod depo;

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
    gelen_anahtar: Option<String>,
    gelen_oturum: Option<Oturum>,
    istemci_kimligi: Option<String>,
    bekleyen: BekleyenOturumIslemi,
    azami_oturum: usize,
    azami_anonim_oturum: usize,
}

impl Default for WebGuvenligi {
    fn default() -> Self {
        Self {
            depo: Depo::default(),
            gelen_anahtar: None,
            gelen_oturum: None,
            istemci_kimligi: None,
            bekleyen: BekleyenOturumIslemi::default(),
            azami_oturum: AZAMI_OTURUM_SAYISI,
            azami_anonim_oturum: AZAMI_ANONIM_OTURUM_SAYISI,
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

    #[cfg(test)]
    fn sinirli(azami_oturum: usize, azami_anonim_oturum: usize) -> Self {
        Self {
            azami_oturum,
            azami_anonim_oturum: azami_anonim_oturum.min(azami_oturum),
            ..Self::default()
        }
    }

    /// Her istekten önce çağrılır. Süresi dolmuş kayıt aynı anda iptal edilir;
    /// erişim LRU sırasını günceller ama mutlak geçerlilik anını uzatmaz.
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
        let oturum = self.depo.guncelle(|durum| {
            durum
                .oturumlar
                .retain(|_, oturum| oturum.son_gecerlilik_ms > an_ms);
            if let Some(anahtar) = &aranan {
                if let Some(oturum) = durum.oturumlar.get_mut(anahtar) {
                    oturum.son_erisim_ms = an_ms;
                    return Ok(Some(oturum.clone()));
                }
            }
            Ok(None)
        })?;
        self.gelen_anahtar = gelen_anahtar;
        self.gelen_oturum = oturum;
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
        let omur = OTURUM_OMRU_SANIYE;
        let oturum = Oturum {
            csrf,
            kullanici: Some(kullanici),
            rol: Some(rol),
            son_gecerlilik_ms: an_ms.saturating_add(omur * 1000),
            son_erisim_ms: an_ms,
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
        let anahtar = crate::paket::sha256_hex(ham.as_bytes());
        let pencere_ms = pencere_saniye.saturating_mul(1_000);
        let azami_anahtar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .web()
            .oran_anahtari_sayisi();
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
    crate::paket::sha256_hex(belirtec.as_bytes())
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
mod tests {
    use super::*;

    fn uretici() -> impl FnMut() -> Result<String, String> {
        let mut sira = 0;
        move || {
            sira += 1;
            Ok(format!("belirtec-{}", sira))
        }
    }

    fn istegi_baslat(depo: &mut WebGuvenligi, belirtec: Option<&str>, an_ms: i64) {
        if !depo.bekleyen.silinecekler.is_empty() || !depo.bekleyen.eklenecekler.is_empty() {
            depo.istegi_tamamla(an_ms).expect("önceki istek commit'i");
        }
        depo.istegi_baslat_kimlikle(belirtec, "yerel-test", an_ms)
            .expect("istek başlamalı");
    }

    #[test]
    fn csrf_anonim_oturuma_baglanir_ve_sahte_deger_reddedilir() {
        let mut depo = WebGuvenligi::yeni();
        istegi_baslat(&mut depo, None, 0);
        let (csrf, yeni) = depo.csrf_belirteci(0, uretici()).expect("csrf");
        let yeni = yeni.expect("anonim oturum");
        istegi_baslat(&mut depo, Some(&yeni.belirtec), 1);
        assert!(depo
            .denetle(&RotaErisimi::HerkeseAcik, Some(&csrf), true, 1)
            .is_ok());
        assert_eq!(
            depo.denetle(&RotaErisimi::HerkeseAcik, Some("sahte"), true, 1)
                .unwrap_err()
                .durum,
            403
        );
    }

    #[test]
    fn giris_kimligi_dondurur_rolu_ayirir_ve_cikis_iptal_eder() {
        let mut depo = WebGuvenligi::yeni();
        let mut uret = uretici();
        istegi_baslat(&mut depo, None, 0);
        let (_, anonim) = depo.csrf_belirteci(0, &mut uret).expect("csrf");
        let anonim = anonim.unwrap();
        let yeni = depo
            .oturum_ac("Mustafa".into(), "yönetici".into(), 1, &mut uret)
            .expect("giriş");
        assert_ne!(anonim.belirtec, yeni.belirtec, "session fixation");
        istegi_baslat(&mut depo, Some(&yeni.belirtec), 2);
        assert!(depo
            .denetle(&RotaErisimi::Rol("yönetici".into()), None, false, 2)
            .is_ok());
        assert_eq!(
            depo.denetle(&RotaErisimi::Rol("okur".into()), None, false, 2)
                .unwrap_err()
                .durum,
            403
        );
        depo.oturum_kapat();
        assert_eq!(depo.oturum_sayisi().unwrap(), 0);
    }

    #[test]
    fn erisim_mutlak_oturum_omrunu_uzatmaz() {
        let mut depo = WebGuvenligi::yeni();
        istegi_baslat(&mut depo, None, 0);
        let yeni = depo
            .oturum_ac("Zeynep".into(), "okur".into(), 0, uretici())
            .unwrap();
        istegi_baslat(
            &mut depo,
            Some(&yeni.belirtec),
            (OTURUM_OMRU_SANIYE - 1) * 1000,
        );
        assert!(depo
            .denetle(
                &RotaErisimi::Oturumlu,
                None,
                false,
                (OTURUM_OMRU_SANIYE - 1) * 1000,
            )
            .is_ok());
        istegi_baslat(
            &mut depo,
            Some(&yeni.belirtec),
            (OTURUM_OMRU_SANIYE + 1) * 1000,
        );
        assert_eq!(
            depo.denetle(
                &RotaErisimi::Oturumlu,
                None,
                false,
                (OTURUM_OMRU_SANIYE + 1) * 1000,
            )
            .unwrap_err()
            .durum,
            401
        );
    }

    #[test]
    fn anonim_kota_en_uzun_suredir_kullanilmayani_tahliye_eder() {
        let mut depo = WebGuvenligi::sinirli(3, 2);
        let mut uret = uretici();
        istegi_baslat(&mut depo, None, 0);
        let (csrf_bir, bir) = depo.csrf_belirteci(0, &mut uret).unwrap();
        let bir = bir.unwrap();
        istegi_baslat(&mut depo, None, 1);
        let (csrf_iki, iki) = depo.csrf_belirteci(1, &mut uret).unwrap();
        let iki = iki.unwrap();

        istegi_baslat(&mut depo, Some(&bir.belirtec), 2);
        istegi_baslat(&mut depo, None, 3);
        let (csrf_uc, uc) = depo.csrf_belirteci(3, &mut uret).unwrap();
        let uc = uc.unwrap();
        assert_eq!(depo.oturum_sayisi().unwrap(), 2);

        istegi_baslat(&mut depo, Some(&iki.belirtec), 4);
        assert_eq!(
            depo.denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_iki), true, 4)
                .unwrap_err()
                .durum,
            403
        );
        istegi_baslat(&mut depo, Some(&bir.belirtec), 5);
        assert!(depo
            .denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_bir), true, 5)
            .is_ok());
        istegi_baslat(&mut depo, Some(&uc.belirtec), 6);
        assert!(depo
            .denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_uc), true, 6)
            .is_ok());
    }

    #[test]
    fn toplam_kota_doluyken_anonim_once_tahliye_edilir() {
        let mut depo = WebGuvenligi::sinirli(2, 2);
        let mut uret = uretici();
        istegi_baslat(&mut depo, None, 0);
        let (_, bir) = depo.csrf_belirteci(0, &mut uret).unwrap();
        let bir = bir.unwrap();
        istegi_baslat(&mut depo, None, 1);
        depo.csrf_belirteci(1, &mut uret).unwrap();
        istegi_baslat(&mut depo, None, 2);
        let giris = depo
            .oturum_ac("Zeynep".into(), "okur".into(), 2, &mut uret)
            .unwrap();
        assert_eq!(depo.oturum_sayisi().unwrap(), 2);

        istegi_baslat(&mut depo, Some(&bir.belirtec), 3);
        assert_eq!(
            depo.denetle(&RotaErisimi::HerkeseAcik, None, true, 3)
                .unwrap_err()
                .durum,
            403
        );
        istegi_baslat(&mut depo, Some(&giris.belirtec), 4);
        assert!(depo.denetle(&RotaErisimi::Oturumlu, None, false, 4).is_ok());
    }

    #[test]
    fn yalniz_kimlikli_kayitlarla_dolu_depo_yeni_girisi_reddeder() {
        let mut depo = WebGuvenligi::sinirli(2, 1);
        let mut uret = uretici();
        istegi_baslat(&mut depo, None, 0);
        depo.oturum_ac("Bir".into(), "okur".into(), 0, &mut uret)
            .unwrap();
        istegi_baslat(&mut depo, None, 1);
        depo.oturum_ac("İki".into(), "okur".into(), 1, &mut uret)
            .unwrap();
        istegi_baslat(&mut depo, None, 2);
        let hata = depo
            .oturum_ac("Üç".into(), "okur".into(), 2, &mut uret)
            .unwrap_err();
        assert!(hata.contains("kapasitesi dolu"));
        assert_eq!(depo.oturum_sayisi().unwrap(), 2);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn gecici_depo_yolu(ad: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SIRA: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir()
            .join(format!(
                "zee-web-depo-test-{}-{}-{}",
                std::process::id(),
                SIRA.fetch_add(1, Ordering::Relaxed),
                ad
            ))
            .join(".zee/web-durumu-v1.json")
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn gecici_depoyu_sil(yol: &std::path::Path) {
        if let Some(kok) = yol.parent().and_then(std::path::Path::parent) {
            let _ = std::fs::remove_dir_all(kok);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn kalici_depo_iki_worker_login_logout_restart_ve_expiry_paylasir() {
        let yol = gecici_depo_yolu("oturum");
        let mut a = WebGuvenligi::kalici(&yol);
        let mut b = WebGuvenligi::kalici(&yol);
        a.istegi_baslat_kimlikle(None, "203.0.113.1", 0).unwrap();
        let yeni = a
            .oturum_ac("Zeynep".into(), "yönetici".into(), 0, uretici())
            .unwrap();
        a.istegi_tamamla(0).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&yol).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                std::fs::metadata(yol.parent().unwrap())
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }

        b.istegi_baslat_kimlikle(Some(&yeni.belirtec), "203.0.113.1", 1)
            .unwrap();
        assert!(b
            .denetle(&RotaErisimi::Rol("yönetici".into()), None, false, 1)
            .is_ok());
        drop(b);

        let mut yeniden = WebGuvenligi::kalici(&yol);
        yeniden
            .istegi_baslat_kimlikle(Some(&yeni.belirtec), "203.0.113.1", 2)
            .unwrap();
        assert!(yeniden
            .denetle(&RotaErisimi::Oturumlu, None, false, 2)
            .is_ok());
        yeniden.oturum_kapat();
        yeniden.istegi_tamamla(2).unwrap();

        a.istegi_baslat_kimlikle(Some(&yeni.belirtec), "203.0.113.1", 3)
            .unwrap();
        assert_eq!(
            a.denetle(&RotaErisimi::Oturumlu, None, false, 3)
                .unwrap_err()
                .durum,
            401
        );

        let mut c = WebGuvenligi::kalici(&yol);
        c.istegi_baslat_kimlikle(None, "203.0.113.2", 10).unwrap();
        let dolacak = c
            .oturum_ac("Eliz".into(), "okur".into(), 10, uretici())
            .unwrap();
        c.istegi_tamamla(10).unwrap();
        let son = 10 + OTURUM_OMRU_SANIYE * 1_000;
        let mut d = WebGuvenligi::kalici(&yol);
        c.istegi_baslat_kimlikle(Some(&dolacak.belirtec), "203.0.113.2", son)
            .unwrap();
        d.istegi_baslat_kimlikle(Some(&dolacak.belirtec), "203.0.113.2", son)
            .unwrap();
        assert_eq!(
            c.denetle(&RotaErisimi::Oturumlu, None, false, son)
                .unwrap_err()
                .durum,
            401
        );
        assert_eq!(
            d.denetle(&RotaErisimi::Oturumlu, None, false, son)
                .unwrap_err()
                .durum,
            401
        );
        gecici_depoyu_sil(&yol);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn kalici_rate_limit_paralel_artisla_esigi_deldirmez() {
        let yol = gecici_depo_yolu("oran");
        let mut temel = WebGuvenligi::kalici(&yol);
        temel
            .istegi_baslat_kimlikle(None, "198.51.100.9", 0)
            .unwrap();
        let mut isler = Vec::new();
        for _ in 0..10 {
            let depo = temel.clone();
            isler.push(std::thread::spawn(move || -> Result<usize, String> {
                let mut izinli = 0;
                for _ in 0..10 {
                    if depo
                        .rate_limit_artir(RateLimitTuru::Giris, "parola", 0)?
                        .izinli
                    {
                        izinli += 1;
                    }
                }
                Ok(izinli)
            }));
        }
        let izinli = isler
            .into_iter()
            .map(|is| is.join().map_err(|_| "rate worker durdu".to_string())?)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .sum::<usize>();
        assert_eq!(
            izinli,
            crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
                .web()
                .giris_orani() as usize
        );
        gecici_depoyu_sil(&yol);
    }

    #[test]
    fn ucnokta_csrf_ve_giris_pencereleri_ayri_ve_sureli_kalir() {
        let mut depo = WebGuvenligi::yeni();
        depo.istegi_baslat_kimlikle(None, "203.0.113.11", 0)
            .unwrap();
        let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.web();
        for _ in 0..sinirlar.ucnokta_orani() {
            assert!(
                depo.rate_limit_artir(RateLimitTuru::Ucnokta, "GET /", 0)
                    .unwrap()
                    .izinli
            );
        }
        assert!(
            !depo
                .rate_limit_artir(RateLimitTuru::Ucnokta, "GET /", 0)
                .unwrap()
                .izinli
        );
        assert!(
            depo.rate_limit_artir(RateLimitTuru::Ucnokta, "GET /başka", 0)
                .unwrap()
                .izinli
        );
        for _ in 0..sinirlar.csrf_orani() {
            assert!(
                depo.rate_limit_artir(RateLimitTuru::Csrf, "csrf", 0)
                    .unwrap()
                    .izinli
            );
        }
        assert!(
            !depo
                .rate_limit_artir(RateLimitTuru::Csrf, "csrf", 0)
                .unwrap()
                .izinli
        );
        assert!(
            depo.rate_limit_artir(
                RateLimitTuru::Ucnokta,
                "GET /",
                sinirlar.ucnokta_penceresi_saniye() * 1_000,
            )
            .unwrap()
            .izinli
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn kalici_oturum_commiti_yanit_hatasinda_kosullu_geri_alinir() {
        let yol = gecici_depo_yolu("geri-al");
        let mut a = WebGuvenligi::kalici(&yol);
        a.istegi_baslat_kimlikle(None, "192.0.2.1", 0).unwrap();
        let yeni = a
            .oturum_ac("Zeynep".into(), "okur".into(), 0, uretici())
            .unwrap();
        let commit = a.istegi_tamamla(0).unwrap();
        a.commiti_geri_al(commit).unwrap();

        let mut b = WebGuvenligi::kalici(&yol);
        b.istegi_baslat_kimlikle(Some(&yeni.belirtec), "192.0.2.1", 1)
            .unwrap();
        assert_eq!(
            b.denetle(&RotaErisimi::Oturumlu, None, false, 1)
                .unwrap_err()
                .durum,
            401
        );
        gecici_depoyu_sil(&yol);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn bozuk_kalici_depo_fail_closed_kalir() {
        let yol = gecici_depo_yolu("bozuk");
        std::fs::create_dir_all(yol.parent().unwrap()).unwrap();
        std::fs::write(&yol, b"{bozuk").unwrap();
        let mut depo = WebGuvenligi::kalici(&yol);
        let hata = depo
            .istegi_baslat_kimlikle(None, "192.0.2.1", 0)
            .unwrap_err();
        assert!(hata.contains("JSON"));
        gecici_depoyu_sil(&yol);
    }

    #[cfg(unix)]
    #[test]
    fn kalici_depo_sembolik_bagi_reddeder() {
        use std::os::unix::fs::symlink;
        let yol = gecici_depo_yolu("symlink");
        std::fs::create_dir_all(yol.parent().unwrap()).unwrap();
        let hedef = yol.parent().unwrap().join("hedef.json");
        std::fs::write(&hedef, b"{}").unwrap();
        symlink(&hedef, &yol).unwrap();
        let mut depo = WebGuvenligi::kalici(&yol);
        let hata = depo
            .istegi_baslat_kimlikle(None, "192.0.2.1", 0)
            .unwrap_err();
        assert!(hata.contains("sembolik bağ"));
        gecici_depoyu_sil(&yol);

        let dizin_yolu = gecici_depo_yolu("symlink-dizin");
        let kok = dizin_yolu.parent().unwrap().parent().unwrap();
        std::fs::create_dir_all(kok).unwrap();
        let hedef_dizin = kok.join("hedef-dizin");
        std::fs::create_dir(&hedef_dizin).unwrap();
        symlink(&hedef_dizin, dizin_yolu.parent().unwrap()).unwrap();
        let mut depo = WebGuvenligi::kalici(&dizin_yolu);
        let hata = depo
            .istegi_baslat_kimlikle(None, "192.0.2.1", 0)
            .unwrap_err();
        assert!(hata.contains("sembolik bağ"));
        gecici_depoyu_sil(&dizin_yolu);
    }

    #[test]
    fn baslik_degerleri_enjeksiyona_kapali() {
        assert!(cerez_adi_gecerli("tema_1"));
        assert!(!cerez_adi_gecerli("x\r\nX-Test"));
        assert!(cerez_degeri_gecerli("abc-_123"));
        assert!(!cerez_degeri_gecerli("a; Secure"));
        assert!(yerel_yonlendirme_gecerli("/giris"));
        assert!(!yerel_yonlendirme_gecerli("https://kotu.example"));
        assert!(!yerel_yonlendirme_gecerli("//kotu.example"));
    }
}
