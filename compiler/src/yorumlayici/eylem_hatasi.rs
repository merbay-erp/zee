use crate::tani::Tani;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EylemHataSinifi {
    Genel,
    CommitSonucuBelirsiz,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EylemHatasi {
    pub sinif: EylemHataSinifi,
    pub mesaj: String,
}

impl EylemHatasi {
    pub fn genel(mesaj: impl Into<String>) -> Self {
        Self {
            sinif: EylemHataSinifi::Genel,
            mesaj: mesaj.into(),
        }
    }

    pub fn commit_sonucu_belirsiz(mesaj: impl Into<String>) -> Self {
        Self {
            sinif: EylemHataSinifi::CommitSonucuBelirsiz,
            mesaj: mesaj.into(),
        }
    }
}

impl From<String> for EylemHatasi {
    fn from(mesaj: String) -> Self {
        Self::genel(mesaj)
    }
}

impl From<&str> for EylemHatasi {
    fn from(mesaj: &str) -> Self {
        Self::genel(mesaj)
    }
}

impl std::fmt::Display for EylemHatasi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.mesaj)
    }
}

pub(super) fn tani(ad: &str, eylem: &str, hata: &EylemHatasi, satir: usize) -> Tani {
    let mesaj = format!("\"{}\" eylem transaction'ı {}: {}.", ad, eylem, hata.mesaj);
    match hata.sinif {
        EylemHataSinifi::Genel => Tani::yeni("C021", mesaj, satir, 1, 1),
        EylemHataSinifi::CommitSonucuBelirsiz => Tani::yeni("C027", mesaj, satir, 1, 1),
    }
}

pub(super) fn web_mesaji(kod: &str) -> Option<&'static str> {
    match kod {
        "C021" => Some("işlem tamamlanamadı; otomatik tekrar yok"),
        "C027" => Some("işlem sonucu belirsiz; otomatik tekrar yok; uzlaştırma gerekli"),
        _ => None,
    }
}
