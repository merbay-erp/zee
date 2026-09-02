//! Web oturumu ve ortak oran profili görünümü.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebSinirlari {
    pub(super) oturum_omru_saniye: i64,
    pub(super) anonim_oturum_omru_saniye: i64,
    pub(super) oturum_sayisi: usize,
    pub(super) anonim_oturum_sayisi: usize,
    pub(super) oran_anahtari_sayisi: usize,
    pub(super) ucnokta_orani: u32,
    pub(super) csrf_orani: u32,
    pub(super) giris_orani: u32,
    pub(super) ucnokta_penceresi_saniye: i64,
    pub(super) csrf_penceresi_saniye: i64,
    pub(super) giris_penceresi_saniye: i64,
}

impl WebSinirlari {
    pub const fn oturum_omru_saniye(self) -> i64 {
        self.oturum_omru_saniye
    }

    pub const fn anonim_oturum_omru_saniye(self) -> i64 {
        self.anonim_oturum_omru_saniye
    }

    pub const fn oturum_sayisi(self) -> usize {
        self.oturum_sayisi
    }

    pub const fn anonim_oturum_sayisi(self) -> usize {
        self.anonim_oturum_sayisi
    }

    pub const fn oran_anahtari_sayisi(self) -> usize {
        self.oran_anahtari_sayisi
    }

    pub const fn ucnokta_orani(self) -> u32 {
        self.ucnokta_orani
    }

    pub const fn csrf_orani(self) -> u32 {
        self.csrf_orani
    }

    pub const fn giris_orani(self) -> u32 {
        self.giris_orani
    }

    pub const fn ucnokta_penceresi_saniye(self) -> i64 {
        self.ucnokta_penceresi_saniye
    }

    pub const fn csrf_penceresi_saniye(self) -> i64 {
        self.csrf_penceresi_saniye
    }

    pub const fn giris_penceresi_saniye(self) -> i64 {
        self.giris_penceresi_saniye
    }
}
