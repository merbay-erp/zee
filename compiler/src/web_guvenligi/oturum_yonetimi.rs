use super::*;

pub fn oturum_satiri(bilgi: OturumBilgisi) -> Vec<(String, String)> {
    vec![
        ("kimlik".into(), bilgi.kimlik),
        ("kullanıcı".into(), bilgi.kullanici),
        ("rol".into(), bilgi.rol),
        ("oluşturulma_ms".into(), bilgi.olusturma_ms.to_string()),
        ("son_erişim_ms".into(), bilgi.son_erisim_ms.to_string()),
        (
            "son_geçerlilik_ms".into(),
            bilgi.son_gecerlilik_ms.to_string(),
        ),
    ]
}

impl WebGuvenligi {
    pub fn oturum_bilgisi(&self, an_ms: i64) -> Option<OturumBilgisi> {
        let oturum = self.gecerli_oturum(an_ms)?;
        Some(OturumBilgisi {
            kimlik: self.gelen_anahtar.clone()?,
            kullanici: oturum.kullanici.clone()?,
            rol: oturum.rol.clone()?,
            olusturma_ms: oturum.olusturma_ms,
            son_erisim_ms: oturum.son_erisim_ms,
            son_gecerlilik_ms: oturum.son_gecerlilik_ms,
        })
    }

    pub fn kullanici_oturumlarini_listele(
        &self,
        kullanici: &str,
        an_ms: i64,
    ) -> Result<Vec<OturumBilgisi>, String> {
        self.depo.guncelle(|durum| {
            let mut gorunum = durum.clone();
            for anahtar in &self.bekleyen.silinecekler {
                gorunum.oturumlar.remove(anahtar);
            }
            for (anahtar, oturum) in &self.bekleyen.eklenecekler {
                gorunum.oturumlar.insert(anahtar.clone(), oturum.clone());
            }
            let mut sonuc = gorunum
                .oturumlar
                .into_iter()
                .filter_map(|(kimlik, oturum)| {
                    if oturum.son_gecerlilik_ms <= an_ms
                        || oturum.kullanici.as_deref() != Some(kullanici)
                    {
                        return None;
                    }
                    Some(OturumBilgisi {
                        kimlik,
                        kullanici: oturum.kullanici?,
                        rol: oturum.rol?,
                        olusturma_ms: oturum.olusturma_ms,
                        son_erisim_ms: oturum.son_erisim_ms,
                        son_gecerlilik_ms: oturum.son_gecerlilik_ms,
                    })
                })
                .collect::<Vec<_>>();
            sonuc.sort_by_key(|bilgi| (bilgi.olusturma_ms, bilgi.kimlik.clone()));
            Ok(sonuc)
        })
    }

    pub fn oturum_kimligini_iptal_et(&mut self, kimlik: &str) -> Result<i64, String> {
        if kimlik.len() != 64
            || !kimlik
                .bytes()
                .all(|bayt| bayt.is_ascii_digit() || (b'a'..=b'f').contains(&bayt))
        {
            return Err("web oturum kimliği 64 haneli küçük harfli SHA-256 olmalı".into());
        }
        let bekleyende = self
            .bekleyen
            .eklenecekler
            .iter()
            .any(|(aday, _)| aday == kimlik);
        let depoda = self
            .depo
            .guncelle(|durum| Ok(durum.oturumlar.contains_key(kimlik)))?;
        if !bekleyende && !depoda {
            return Ok(0);
        }
        self.bekleyen
            .eklenecekler
            .retain(|(aday, _)| aday != kimlik);
        if depoda {
            self.bekleyen.silinecekler.insert(kimlik.to_string());
        }
        if self.gelen_anahtar.as_deref() == Some(kimlik) {
            self.gelen_oturum = None;
        }
        Ok(1)
    }
}
