//! K-088 sunucu tarafı oturum, CSRF ve yetki çekirdeği.
//!
//! Depo yalnız belirteç özetini saklar. Anonim oturum form CSRF'si üretir;
//! başarılı giriş aynı oturumu kullanmaz, yeni kimlik ve CSRF ile döndürür.

use crate::agac::RotaErisimi;
use crate::guvenlik::sabit_zamanli_esit;
use std::collections::HashMap;

pub const OTURUM_OMRU_SANIYE: i64 = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .web()
    .oturum_omru_saniye();
pub const ANONIM_OTURUM_OMRU_SANIYE: i64 = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .web()
    .anonim_oturum_omru_saniye();
pub const AZAMI_OTURUM_SAYISI: usize =
    crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.web().oturum_sayisi();
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

#[derive(Debug, Clone)]
struct Oturum {
    csrf: String,
    kullanici: Option<String>,
    rol: Option<String>,
    son_gecerlilik_ms: i64,
    son_erisim_ms: i64,
    olusturma_sirasi: u64,
}

pub struct WebGuvenligi {
    oturumlar: HashMap<String, Oturum>,
    gelen_belirtec: Option<String>,
    sonraki_olusturma_sirasi: u64,
    azami_oturum: usize,
    azami_anonim_oturum: usize,
}

impl Default for WebGuvenligi {
    fn default() -> Self {
        Self {
            oturumlar: HashMap::new(),
            gelen_belirtec: None,
            sonraki_olusturma_sirasi: 0,
            azami_oturum: AZAMI_OTURUM_SAYISI,
            azami_anonim_oturum: AZAMI_ANONIM_OTURUM_SAYISI,
        }
    }
}

impl WebGuvenligi {
    pub fn yeni() -> Self {
        Self::default()
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
    pub fn istegi_baslat(&mut self, belirtec: Option<&str>, an_ms: i64) {
        self.gelen_belirtec = belirtec.map(str::to_string);
        self.suresi_dolani_sil(an_ms);
        if let Some(belirtec) = belirtec {
            if let Some(oturum) = self.oturumlar.get_mut(&anahtar(belirtec)) {
                oturum.son_erisim_ms = an_ms;
            }
        }
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
        self.oturum_yeri_ac(true)?;
        let belirtec = uret()?;
        let csrf = uret()?;
        let omur = ANONIM_OTURUM_OMRU_SANIYE;
        self.oturum_ekle(
            &belirtec,
            Oturum {
                csrf: csrf.clone(),
                kullanici: None,
                rol: None,
                son_gecerlilik_ms: an_ms.saturating_add(omur * 1000),
                son_erisim_ms: an_ms,
                olusturma_sirasi: 0,
            },
        );
        self.gelen_belirtec = Some(belirtec.clone());
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
        self.oturum_yeri_ac(false)?;
        let belirtec = uret()?;
        let csrf = uret()?;
        let omur = OTURUM_OMRU_SANIYE;
        self.oturum_ekle(
            &belirtec,
            Oturum {
                csrf,
                kullanici: Some(kullanici),
                rol: Some(rol),
                son_gecerlilik_ms: an_ms.saturating_add(omur * 1000),
                son_erisim_ms: an_ms,
                olusturma_sirasi: 0,
            },
        );
        self.gelen_belirtec = Some(belirtec.clone());
        Ok(YeniOturum {
            belirtec,
            azami_omur_saniye: omur,
        })
    }

    pub fn oturum_kapat(&mut self) {
        self.gecerli_oturumu_sil();
        self.gelen_belirtec = None;
    }

    pub fn oturum_sayisi(&self) -> usize {
        self.oturumlar.len()
    }

    fn gecerli_oturum(&self, an_ms: i64) -> Option<&Oturum> {
        let belirtec = self.gelen_belirtec.as_deref()?;
        self.oturumlar
            .get(&anahtar(belirtec))
            .filter(|oturum| oturum.son_gecerlilik_ms > an_ms)
    }

    fn gecerli_oturumu_sil(&mut self) {
        if let Some(belirtec) = &self.gelen_belirtec {
            self.oturumlar.remove(&anahtar(belirtec));
        }
    }

    fn suresi_dolani_sil(&mut self, an_ms: i64) {
        self.oturumlar
            .retain(|_, oturum| oturum.son_gecerlilik_ms > an_ms);
    }

    fn oturum_yeri_ac(&mut self, anonim: bool) -> Result<(), String> {
        if anonim
            && self.anonim_oturum_sayisi() >= self.azami_anonim_oturum
            && !self.en_eski_anonim_oturumu_sil()
        {
            return Err("anonim web oturumu kapasitesi dolu".into());
        }
        while self.oturumlar.len() >= self.azami_oturum {
            if !self.en_eski_anonim_oturumu_sil() {
                return Err("web oturumu kapasitesi dolu; yeni giriş reddedildi".into());
            }
        }
        Ok(())
    }

    fn anonim_oturum_sayisi(&self) -> usize {
        self.oturumlar
            .values()
            .filter(|oturum| oturum.kullanici.is_none())
            .count()
    }

    fn en_eski_anonim_oturumu_sil(&mut self) -> bool {
        let aday = self
            .oturumlar
            .iter()
            .filter(|(_, oturum)| oturum.kullanici.is_none())
            .min_by_key(|(_, oturum)| (oturum.son_erisim_ms, oturum.olusturma_sirasi))
            .map(|(anahtar, _)| anahtar.clone());
        aday
            .map(|anahtar| self.oturumlar.remove(&anahtar).is_some())
            .unwrap_or(false)
    }

    fn oturum_ekle(&mut self, belirtec: &str, mut oturum: Oturum) {
        oturum.olusturma_sirasi = self.sonraki_olusturma_sirasi;
        self.sonraki_olusturma_sirasi = self.sonraki_olusturma_sirasi.saturating_add(1);
        self.oturumlar.insert(anahtar(belirtec), oturum);
    }
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

    #[test]
    fn csrf_anonim_oturuma_baglanir_ve_sahte_deger_reddedilir() {
        let mut depo = WebGuvenligi::yeni();
        depo.istegi_baslat(None, 0);
        let (csrf, yeni) = depo.csrf_belirteci(0, uretici()).expect("csrf");
        let yeni = yeni.expect("anonim oturum");
        depo.istegi_baslat(Some(&yeni.belirtec), 1);
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
        depo.istegi_baslat(None, 0);
        let (_, anonim) = depo.csrf_belirteci(0, &mut uret).expect("csrf");
        let anonim = anonim.unwrap();
        let yeni = depo
            .oturum_ac("Mustafa".into(), "yönetici".into(), 1, &mut uret)
            .expect("giriş");
        assert_ne!(anonim.belirtec, yeni.belirtec, "session fixation");
        depo.istegi_baslat(Some(&yeni.belirtec), 2);
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
        assert_eq!(depo.oturum_sayisi(), 0);
    }

    #[test]
    fn erisim_mutlak_oturum_omrunu_uzatmaz() {
        let mut depo = WebGuvenligi::yeni();
        depo.istegi_baslat(None, 0);
        let yeni = depo
            .oturum_ac("Zeynep".into(), "okur".into(), 0, uretici())
            .unwrap();
        depo.istegi_baslat(
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
        depo.istegi_baslat(Some(&yeni.belirtec), (OTURUM_OMRU_SANIYE + 1) * 1000);
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
        depo.istegi_baslat(None, 0);
        let (csrf_bir, bir) = depo.csrf_belirteci(0, &mut uret).unwrap();
        let bir = bir.unwrap();
        depo.istegi_baslat(None, 1);
        let (csrf_iki, iki) = depo.csrf_belirteci(1, &mut uret).unwrap();
        let iki = iki.unwrap();

        depo.istegi_baslat(Some(&bir.belirtec), 2);
        depo.istegi_baslat(None, 3);
        let (csrf_uc, uc) = depo.csrf_belirteci(3, &mut uret).unwrap();
        let uc = uc.unwrap();
        assert_eq!(depo.oturum_sayisi(), 2);

        depo.istegi_baslat(Some(&iki.belirtec), 4);
        assert_eq!(
            depo.denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_iki), true, 4)
                .unwrap_err()
                .durum,
            403
        );
        depo.istegi_baslat(Some(&bir.belirtec), 5);
        assert!(depo
            .denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_bir), true, 5)
            .is_ok());
        depo.istegi_baslat(Some(&uc.belirtec), 6);
        assert!(depo
            .denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_uc), true, 6)
            .is_ok());
    }

    #[test]
    fn toplam_kota_doluyken_anonim_once_tahliye_edilir() {
        let mut depo = WebGuvenligi::sinirli(2, 2);
        let mut uret = uretici();
        depo.istegi_baslat(None, 0);
        let (_, bir) = depo.csrf_belirteci(0, &mut uret).unwrap();
        let bir = bir.unwrap();
        depo.istegi_baslat(None, 1);
        depo.csrf_belirteci(1, &mut uret).unwrap();
        depo.istegi_baslat(None, 2);
        let giris = depo
            .oturum_ac("Zeynep".into(), "okur".into(), 2, &mut uret)
            .unwrap();
        assert_eq!(depo.oturum_sayisi(), 2);

        depo.istegi_baslat(Some(&bir.belirtec), 3);
        assert_eq!(
            depo.denetle(&RotaErisimi::HerkeseAcik, None, true, 3)
                .unwrap_err()
                .durum,
            403
        );
        depo.istegi_baslat(Some(&giris.belirtec), 4);
        assert!(depo
            .denetle(&RotaErisimi::Oturumlu, None, false, 4)
            .is_ok());
    }

    #[test]
    fn yalniz_kimlikli_kayitlarla_dolu_depo_yeni_girisi_reddeder() {
        let mut depo = WebGuvenligi::sinirli(2, 1);
        let mut uret = uretici();
        depo.istegi_baslat(None, 0);
        depo.oturum_ac("Bir".into(), "okur".into(), 0, &mut uret)
            .unwrap();
        depo.istegi_baslat(None, 1);
        depo.oturum_ac("İki".into(), "okur".into(), 1, &mut uret)
            .unwrap();
        depo.istegi_baslat(None, 2);
        let hata = depo
            .oturum_ac("Üç".into(), "okur".into(), 2, &mut uret)
            .unwrap_err();
        assert!(hata.contains("kapasitesi dolu"));
        assert_eq!(depo.oturum_sayisi(), 2);
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
