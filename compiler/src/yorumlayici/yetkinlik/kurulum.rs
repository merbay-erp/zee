//! `PolitikaliIo` kurulumu ve ortak yetkinlik denetim yardımcıları.
use super::*;

impl<T: GirdiCikti> PolitikaliIo<T> {
    pub fn yeni(ic: T) -> Self {
        Self {
            ic,
            politika: YetkinlikPolitikasi::cocuk(),
            cocuk_modu: true,
        }
    }

    pub fn politikali(ic: T, politika: YetkinlikPolitikasi) -> Self {
        Self {
            ic,
            politika,
            cocuk_modu: false,
        }
    }

    pub fn politikasi(&self) -> &YetkinlikPolitikasi {
        &self.politika
    }

    pub(super) fn dosya_yolunu_denetle(
        &self,
        yol: &str,
        yetkinlik: Yetkinlik,
    ) -> Result<(), String> {
        dosya::yolu_denetle(&self.politika, self.cocuk_modu, yol, yetkinlik)
    }

    /// K-394: dosyadan bağlantıya aktarımın yolu (proje dosyası ya da
    /// `veri:`) aynı merkezî yol/yetkinlik denetiminden geçer.
    pub(super) fn aktarim_yolunu_denetle(
        &self,
        islem: &super::super::BaglantiIslemi,
    ) -> Result<(), String> {
        match islem {
            super::super::BaglantiIslemi::DosyaAktar(yol)
            | super::super::BaglantiIslemi::Base64Aktar { yol, .. } => {
                self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)
            }
            _ => Ok(()),
        }
    }

    pub(super) fn web_gerektir(&self) -> Result<(), String> {
        self.gerektir(Yetkinlik::AgSunucusu)
    }

    pub(super) fn web_oturumu_gerektir(&self) -> Result<(), String> {
        self.web_gerektir()?;
        self.gerektir(Yetkinlik::WebOturumu)
    }

    pub(super) fn veritabani_gerektir(&self) -> Result<(), VeritabaniHatasi> {
        self.gerektir(Yetkinlik::Veritabani)
            .map_err(super::super::yetkinlik_hatasi::veritabani)
    }

    pub(super) fn gerektir(&self, yetkinlik: Yetkinlik) -> Result<(), String> {
        self.politika
            .gerektir(yetkinlik)
            .map_err(|hata| super::super::yetkinlik_hatasi::mesaj(hata, self.cocuk_modu, yetkinlik))
    }
}
