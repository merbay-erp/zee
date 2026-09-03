//! PostgreSQL sorgu, sonuç ve migration kaynak bütçeleri.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VeritabaniSinirlari {
    pub(super) sorgu_bayti: usize,
    pub(super) parametre: usize,
    pub(super) parametre_bayti: usize,
    pub(super) satir: usize,
    pub(super) sutun: usize,
    pub(super) sonuc_bayti: usize,
    pub(super) goc_dosyasi_bayti: u64,
    pub(super) goc_toplami_bayti: u64,
}

pub(super) const VARSAYILAN: VeritabaniSinirlari = VeritabaniSinirlari {
    sorgu_bayti: 64 * 1024,
    parametre: 100,
    parametre_bayti: 64 * 1024,
    satir: 10_000,
    sutun: 100,
    sonuc_bayti: 16 * 1024 * 1024,
    goc_dosyasi_bayti: 1024 * 1024,
    goc_toplami_bayti: 8 * 1024 * 1024,
};

impl VeritabaniSinirlari {
    pub const fn sorgu_bayti(self) -> usize {
        self.sorgu_bayti
    }
    pub const fn parametre(self) -> usize {
        self.parametre
    }
    pub const fn parametre_bayti(self) -> usize {
        self.parametre_bayti
    }
    pub const fn satir(self) -> usize {
        self.satir
    }
    pub const fn sutun(self) -> usize {
        self.sutun
    }
    pub const fn sonuc_bayti(self) -> usize {
        self.sonuc_bayti
    }
    pub const fn goc_dosyasi_bayti(self) -> u64 {
        self.goc_dosyasi_bayti
    }
    pub const fn goc_toplami_bayti(self) -> u64 {
        self.goc_toplami_bayti
    }
}
