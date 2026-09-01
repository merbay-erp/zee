//! K-088 sunucu tarafı oturum, CSRF ve yetki çekirdeği.
//!
//! Depo yalnız belirteç özetini saklar. Anonim oturum form CSRF'si üretir;
//! başarılı giriş aynı oturumu kullanmaz, yeni kimlik ve CSRF ile döndürür.

use crate::agac::RotaErisimi;
use crate::guvenlik::sabit_zamanli_esit;
use std::collections::HashMap;

pub const OTURUM_OMRU_SANIYE: i64 = 30 * 60;
pub const ANONIM_OTURUM_OMRU_SANIYE: i64 = 10 * 60;

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
}

#[derive(Default)]
pub struct WebGuvenligi {
    oturumlar: HashMap<String, Oturum>,
    gelen_belirtec: Option<String>,
}

impl WebGuvenligi {
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Her istekten önce çağrılır. Süresi dolmuş kayıt aynı anda iptal edilir.
    pub fn istegi_baslat(&mut self, belirtec: Option<&str>, an_ms: i64) {
        self.gelen_belirtec = belirtec.map(str::to_string);
        self.suresi_dolani_sil(an_ms);
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
        let belirtec = uret()?;
        let csrf = uret()?;
        let omur = ANONIM_OTURUM_OMRU_SANIYE;
        self.oturumlar.insert(
            anahtar(&belirtec),
            Oturum {
                csrf: csrf.clone(),
                kullanici: None,
                rol: None,
                son_gecerlilik_ms: an_ms.saturating_add(omur * 1000),
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
        let belirtec = uret()?;
        let csrf = uret()?;
        let omur = OTURUM_OMRU_SANIYE;
        self.oturumlar.insert(
            anahtar(&belirtec),
            Oturum {
                csrf,
                kullanici: Some(kullanici),
                rol: Some(rol),
                son_gecerlilik_ms: an_ms.saturating_add(omur * 1000),
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
    fn suresi_dolan_oturum_401_olur() {
        let mut depo = WebGuvenligi::yeni();
        depo.istegi_baslat(None, 0);
        let yeni = depo
            .oturum_ac("Zeynep".into(), "okur".into(), 0, uretici())
            .unwrap();
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
