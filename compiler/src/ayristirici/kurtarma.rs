use super::*;
use crate::tani::{tanilari_sirala_ve_sinirla, AZAMI_TANI_SAYISI};

impl Ayristirici {
    /// Kök cümlelerini dosya sonuna veya tanı bütçesine kadar ayrıştırır.
    pub(super) fn koku_ayristir(&mut self) -> Vec<Cumle> {
        let mut cumleler = Vec::new();
        while !self.tani_limiti_doldu() {
            match self.bak().tur {
                TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu | TokenTur::Cikinti => {
                    self.ilerle();
                }
                TokenTur::Girinti => self.dengeyi_atla(),
                _ => {
                    if let Some(cumle) = self.cumleyi_kurtararak_ayristir() {
                        cumleler.push(cumle);
                    }
                }
            }
        }
        cumleler
    }

    /// Aynı girinti düzeyindeki cümleleri, kardeşleri koruyarak ayrıştırır.
    pub(super) fn blok_ayristir(&mut self) -> Vec<Cumle> {
        let mut cumleler = Vec::new();
        while !self.tani_limiti_doldu() {
            match self.bak().tur {
                TokenTur::Cikinti | TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu => {
                    self.ilerle();
                }
                TokenTur::Girinti => self.dengeyi_atla(),
                _ => {
                    if let Some(cumle) = self.cumleyi_kurtararak_ayristir() {
                        cumleler.push(cumle);
                    }
                }
            }
        }
        cumleler
    }

    fn cumleyi_kurtararak_ayristir(&mut self) -> Option<Cumle> {
        let baslangic = self.konum;
        match self.cumle_ayristir() {
            Ok(cumle) => Some(cumle),
            Err(tani) => {
                let hata_satiri = tani.satir;
                self.tani_kaydet(tani);
                self.cumle_sinirina_senkronla(baslangic, hata_satiri);
                None
            }
        }
    }

    /// Hatalı satırın kalanını ve yalnız ona ait dengeli alt gövdeyi atlar.
    fn cumle_sinirina_senkronla(&mut self, baslangic: usize, hata_satiri: usize) {
        if self.konum == baslangic || self.bak().satir <= hata_satiri {
            loop {
                match self.bak().tur {
                    TokenTur::SatirSonu => {
                        self.ilerle();
                        break;
                    }
                    TokenTur::Cikinti | TokenTur::DosyaSonu | TokenTur::Girinti => break,
                    _ => {
                        self.ilerle();
                    }
                }
            }
        }
        self.bekleyen_govdeyi_atla();
    }

    /// İçi zaten tüketilmiş hatalı başlık/kolun varsa alt gövdesini atlar.
    pub(super) fn bekleyen_govdeyi_atla(&mut self) {
        if matches!(self.bak().tur, TokenTur::Girinti) {
            self.dengeyi_atla();
        }
    }

    /// Girinti..Çıkıntı dengeli bölgesini, iç içe gövdelerle birlikte atlar.
    pub(super) fn dengeyi_atla(&mut self) {
        if !matches!(self.bak().tur, TokenTur::Girinti) {
            return;
        }
        self.ilerle();
        let mut derinlik = 1usize;
        loop {
            match self.bak().tur {
                TokenTur::Girinti => {
                    derinlik += 1;
                    self.ilerle();
                }
                TokenTur::Cikinti => {
                    derinlik -= 1;
                    self.ilerle();
                    if derinlik == 0 {
                        return;
                    }
                }
                TokenTur::DosyaSonu => return,
                _ => {
                    self.ilerle();
                }
            }
        }
    }

    pub(super) fn tani_kaydet(&mut self, tani: Tani) {
        if !self.tani_limiti_doldu() {
            self.tanilar.push(tani);
        }
    }

    pub(super) fn tanilari_kaydet(&mut self, tanilar: Vec<Tani>) {
        for tani in tanilar {
            self.tani_kaydet(tani);
        }
    }

    pub(super) fn tani_limiti_doldu(&self) -> bool {
        self.tanilar.len() >= AZAMI_TANI_SAYISI
    }

    pub(super) fn tanilari_sirala(&mut self) {
        tanilari_sirala_ve_sinirla(&mut self.tanilar);
    }
}
