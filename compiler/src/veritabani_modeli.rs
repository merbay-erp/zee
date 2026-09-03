//! Veritabanı adaptörleri ile runtime arasındaki davranışsız veri sözleşmesi.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VeritabaniHatasi {
    pub mesaj: String,
    pub veri: Vec<(String, String)>,
}

impl VeritabaniHatasi {
    pub fn commit_sonucu_belirsiz_mi(&self) -> bool {
        self.veri
            .iter()
            .any(|(ad, deger)| ad == "hata_sinifi" && deger == "db.commit_unknown")
    }
}

pub type VeritabaniSatiri = Vec<(String, String)>;
pub type VeritabaniSatirlari = Vec<VeritabaniSatiri>;
pub type VeritabaniOkumaSonucu = Result<VeritabaniSatirlari, VeritabaniHatasi>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VeritabaniBildirimi {
    pub hedef: VeritabaniHedefi,
    pub baglanti_degiskeni: String,
    pub gocler: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VeritabaniHedefi {
    pub konak: String,
    pub kapi: u16,
    pub veritabani: String,
}

impl VeritabaniHedefi {
    pub fn ayristir(yazim: &str) -> Result<Self, String> {
        let kalan = yazim
            .strip_prefix("postgresql://")
            .ok_or_else(|| "hedef postgresql:// şemasıyla başlamalı".to_string())?;
        if kalan.contains(['@', '?', '#', '\\']) {
            return Err("hedef kullanıcı, parola, sorgu veya parça taşıyamaz".into());
        }
        let (otorite, veritabani) = kalan
            .split_once('/')
            .ok_or_else(|| "hedef veritabanı adını taşımalı".to_string())?;
        if veritabani.is_empty()
            || veritabani.contains('/')
            || !veritabani
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        {
            return Err("veritabanı adı yalnız ASCII harf, sayı, _ veya - taşımalı".into());
        }
        let (konak, kapi) = if otorite == "[::1]" {
            ("::1", 5432)
        } else if let Some(kapi) = otorite.strip_prefix("[::1]:") {
            (
                "::1",
                kapi.parse::<u16>()
                    .map_err(|_| "geçersiz PostgreSQL kapısı")?,
            )
        } else if let Some((konak, kapi)) = otorite.rsplit_once(':') {
            (
                konak,
                kapi.parse::<u16>()
                    .map_err(|_| "geçersiz PostgreSQL kapısı")?,
            )
        } else {
            (otorite, 5432)
        };
        if !matches!(konak, "localhost" | "127.0.0.1" | "::1") {
            return Err("ilk PostgreSQL dogfood profili yalnız loopback hedefi kabul eder".into());
        }
        if kapi == 0 {
            return Err("PostgreSQL kapısı 1..65535 arasında olmalı".into());
        }
        Ok(Self {
            konak: konak.to_string(),
            kapi,
            veritabani: veritabani.to_string(),
        })
    }

    pub fn yazimi(&self) -> String {
        let konak = if self.konak == "::1" {
            "[::1]".to_string()
        } else {
            self.konak.clone()
        };
        format!("postgresql://{}:{}/{}", konak, self.kapi, self.veritabani)
    }
}
