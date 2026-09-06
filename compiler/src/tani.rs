//! Türkçe tanı (diagnostic) altyapısı.
//!
//! Master plan bölüm 9'daki biçim hedeflenir:
//! kod + açıklama + kaynak satırı + işaret + öneri.

use std::fmt;

pub(crate) use crate::tani_politikasi::AZAMI_TANI_SAYISI;

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
        Tani {
            kod: kod.into(),
            mesaj,
            satir,
            sutun,
            uzunluk: uzunluk.max(1),
            oneri: None,
        }
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
        write!(
            f,
            "HATA {} (satır {}): {}",
            self.kod, self.satir, self.mesaj
        )
    }
}

/// Fazlardan gelen tanıları kaynak konumunda kararlı sıraya koyar ve bütçeler.
/// Tanı mesajındaki ilk tırnaklı sözü döndürür; A003 için `her X için`
/// kalıbının ortasındaki X'tir.
fn tirnakli_soz(mesaj: &str) -> Option<String> {
    let bas = mesaj.find('"')? + 1;
    let son = mesaj[bas..].find('"')? + bas;
    let soz = &mesaj[bas..son];
    let soz = soz
        .strip_prefix("her ")
        .and_then(|k| k.strip_suffix(" için"))
        .unwrap_or(soz);
    Some(soz.to_string())
}

/// İki yüzeyin aynı kökten geldiğini kabaca sınar: kısa olanın son harfi
/// atılmış hali uzun olanın önekiyse (ünsüz yumuşaması dahil) aynı köktür.
fn ayni_kok_mu(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let (kisa, uzun) = if a.chars().count() <= b.chars().count() {
        (a, b)
    } else {
        (b, a)
    };
    let harfler = kisa.chars().collect::<Vec<_>>();
    harfler.len() >= 3 && uzun.starts_with(&harfler[..harfler.len() - 1].iter().collect::<String>())
}

/// Çoklu tanı hattında aynı tanımsız ad ya da yapı için A001/A003/A007 yalnız
/// ilk kaynak konumunda yayımlanır; tek eksik tanımın her kullanımda (ekli
/// biçimleri dahil) yeniden raporlanması gürültüdür (K-166, ADR-024 §8).
pub(crate) fn tanimsiz_ad_tekrarlarini_ayikla(
    tanilar: &mut Vec<Tani>,
    kok_nedeni_raporlanan: &[String],
) {
    tanilar.sort_by_key(|tani| (tani.satir, tani.sutun));
    let mut raporlanan: Vec<String> = kok_nedeni_raporlanan.to_vec();
    tanilar.retain(|tani| {
        if !matches!(tani.kod.as_str(), "A001" | "A003" | "A007") {
            return true;
        }
        let Some(soz) = tirnakli_soz(&tani.mesaj) else {
            return true;
        };
        if raporlanan.iter().any(|onceki| ayni_kok_mu(onceki, &soz)) {
            return false;
        }
        raporlanan.push(soz);
        true
    });
}

pub(crate) fn tanilari_sirala_ve_sinirla(tanilar: &mut Vec<Tani>) {
    tanilar.sort_by(|sol, sag| {
        (sol.satir, sol.sutun, &sol.kod, &sol.mesaj)
            .cmp(&(sag.satir, sag.sutun, &sag.kod, &sag.mesaj))
    });
    tanilar.truncate(AZAMI_TANI_SAYISI);
}
