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

    /// Bağlantı işlemi `ağ-dinleyici` ya da `ağ` ister (tanı türü "kapandı");
    /// K-397 (ADR-173): giden bağlanma yalnız `ağ` ister (tür "hedef_yasak";
    /// hedef ve IP politikası host'ta, tek çözümlemeyle). K-394: dosyadan
    /// bağlantıya aktarımın yolu (proje dosyası ya da `veri:`) aynı merkezî
    /// yol/yetkinlik denetiminden geçer (tanı türü "dosya").
    pub(super) fn baglanti_islemini_denetle(
        &self,
        islem: &super::super::BaglantiIslemi,
    ) -> Result<(), (&'static str, String)> {
        if matches!(islem, super::super::BaglantiIslemi::GidenBaglan(..)) {
            return self
                .gerektir(Yetkinlik::Ag)
                .map_err(|hata| ("hedef_yasak", hata));
        }
        if !self.politika.izin_verir(Yetkinlik::Ag) {
            self.gerektir(Yetkinlik::AgDinleyici)
                .map_err(|hata| ("kapandı", hata))?;
        }
        match islem {
            super::super::BaglantiIslemi::DosyaAktar(yol)
            | super::super::BaglantiIslemi::Base64Aktar { yol, .. } => self
                .dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)
                .map_err(|hata| ("dosya", hata)),
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
