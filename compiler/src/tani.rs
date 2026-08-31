//! Türkçe tanı (diagnostic) altyapısı.
//!
//! Master plan bölüm 9'daki biçim hedeflenir:
//! kod + açıklama + kaynak satırı + işaret + öneri.

use std::fmt;

#[derive(Debug, Clone)]
pub struct Tani {
    /// Hata kodu: S### sözdizimi, A### ad çözümleme, T### tür.
    pub kod: String,
    pub mesaj: String,
    /// 1 tabanlı satır numarası.
    pub satir: usize,
    /// 1 tabanlı sütun (karakter cinsinden).
    pub sutun: usize,
    /// İşaretlenecek karakter sayısı (en az 1).
    pub uzunluk: usize,
    pub oneri: Option<String>,
}

impl Tani {
    pub fn yeni(kod: &str, mesaj: String, satir: usize, sutun: usize, uzunluk: usize) -> Tani {
        Tani { kod: kod.into(), mesaj, satir, sutun, uzunluk: uzunluk.max(1), oneri: None }
    }

    pub fn onerili(mut self, oneri: String) -> Tani {
        self.oneri = Some(oneri);
        self
    }

    /// Kaynak metinle birlikte tam Türkçe rapor üretir.
    pub fn raporla(&self, kaynak: &str) -> String {
        let mut cikti = format!("HATA {}\n\n{}\n", self.kod, self.mesaj);
        if let Some(satir_metni) = kaynak.lines().nth(self.satir.saturating_sub(1)) {
            let numara = self.satir.to_string();
            cikti.push_str(&format!("\n{} | {}\n", numara, satir_metni));
            let bosluk = " ".repeat(numara.chars().count() + 3 + self.sutun.saturating_sub(1));
            cikti.push_str(&format!("{}{}\n", bosluk, "^".repeat(self.uzunluk)));
        }
        if let Some(oneri) = &self.oneri {
            cikti.push_str(&format!("\nÖneri:\n{}\n", oneri));
        }
        cikti
    }
}

impl fmt::Display for Tani {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HATA {} (satır {}): {}", self.kod, self.satir, self.mesaj)
    }
}
