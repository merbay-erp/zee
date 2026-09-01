//! Türkçe tanı (diagnostic) altyapısı.
//!
//! Master plan bölüm 9'daki biçim hedeflenir:
//! kod + açıklama + kaynak satırı + işaret + öneri.

use std::fmt;

/// Tek kaynak doğrulamasında kullanıcıya/LSP'ye gönderilen üst tanı bütçesi.
pub(crate) const AZAMI_TANI_SAYISI: usize = 20;

#[derive(Debug, Clone)]
pub struct Tani {
    /// Hata kodu: S### sözdizimi, A### ad çözümleme, T### tür, P### proje.
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
        cikti.push_str(&format!("\nAyrıntı için: dil hata {}\n", self.kod));
        cikti
    }

    /// Makine-okunur tek satır JSON (editör entegrasyonları: `dil denetle --json`).
    pub fn json(&self) -> String {
        let oneri = match &self.oneri {
            Some(oneri) => format!("\"{}\"", json_kacis(oneri)),
            None => "null".to_string(),
        };
        format!(
            "{{\"kod\":\"{}\",\"mesaj\":\"{}\",\"satir\":{},\"sutun\":{},\"uzunluk\":{},\"oneri\":{}}}",
            json_kacis(&self.kod),
            json_kacis(&self.mesaj),
            self.satir,
            self.sutun,
            self.uzunluk,
            oneri
        )
    }
}

/// JSON metin kaçışı (RFC 8259): tırnak, ters bölü ve kontrol karakterleri.
fn json_kacis(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len());
    for k in metin.chars() {
        match k {
            '"' => cikti.push_str("\\\""),
            '\\' => cikti.push_str("\\\\"),
            '\n' => cikti.push_str("\\n"),
            '\r' => cikti.push_str("\\r"),
            '\t' => cikti.push_str("\\t"),
            k if (k as u32) < 0x20 => cikti.push_str(&format!("\\u{:04x}", k as u32)),
            k => cikti.push(k),
        }
    }
    cikti
}

impl fmt::Display for Tani {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HATA {} (satır {}): {}", self.kod, self.satir, self.mesaj)
    }
}

/// Fazlardan gelen tanıları kaynak konumunda kararlı sıraya koyar ve bütçeler.
pub(crate) fn tanilari_sirala_ve_sinirla(tanilar: &mut Vec<Tani>) {
    tanilar.sort_by(|sol, sag| {
        (sol.satir, sol.sutun, &sol.kod, &sol.mesaj)
            .cmp(&(sag.satir, sag.sutun, &sag.kod, &sag.mesaj))
    });
    tanilar.truncate(AZAMI_TANI_SAYISI);
}
