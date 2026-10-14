//! K-232: SMTP MIME kaynak sınırlarının tek sahibi.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpostaSinirlari {
    ek_adedi: usize,
    ek_bayti: usize,
    toplam_ek_bayti: usize,
    metin_bayti: usize,
    ileti_bayti: usize,
    zaman_asimi_ms: i64,
    konu_bayti: usize,
    ad_bayti: usize,
    tarih_bayti: usize,
    kimlik_bayti: usize,
}

impl EpostaSinirlari {
    pub const fn ek_adedi(self) -> usize {
        self.ek_adedi
    }
    pub const fn ek_bayti(self) -> usize {
        self.ek_bayti
    }
    pub const fn toplam_ek_bayti(self) -> usize {
        self.toplam_ek_bayti
    }
    pub const fn metin_bayti(self) -> usize {
        self.metin_bayti
    }
    pub const fn ileti_bayti(self) -> usize {
        self.ileti_bayti
    }
    pub const fn zaman_asimi_ms(self) -> i64 {
        self.zaman_asimi_ms
    }
    pub const fn konu_bayti(self) -> usize {
        self.konu_bayti
    }
    pub const fn ad_bayti(self) -> usize {
        self.ad_bayti
    }
    pub const fn tarih_bayti(self) -> usize {
        self.tarih_bayti
    }
    pub const fn kimlik_bayti(self) -> usize {
        self.kimlik_bayti
    }
}

pub(super) const VARSAYILAN: EpostaSinirlari = EpostaSinirlari {
    ek_adedi: 8,
    ek_bayti: 8 * 1024 * 1024,
    toplam_ek_bayti: 16 * 1024 * 1024,
    metin_bayti: 512 * 1024,
    ileti_bayti: 24 * 1024 * 1024,
    zaman_asimi_ms: 20_000,
    konu_bayti: 512,
    ad_bayti: 255,
    tarih_bayti: 78,
    kimlik_bayti: 48,
};
