//! `GirdiCikti` çağrılarını merkezî yetkinlik politikasında fail-closed tutar.
mod dosya;

use super::{EylemHatasi, GirdiCikti, VeritabaniHatasi};
use crate::agac::RotaErisimi;
use crate::web_guvenligi::WebReddi;
use crate::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};

pub struct PolitikaliIo<T: GirdiCikti> {
    pub ic: T,
    politika: YetkinlikPolitikasi,
    cocuk_modu: bool,
}

pub type GuvenliIo<T> = PolitikaliIo<T>;
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

    fn dosya_yolunu_denetle(&self, yol: &str, yetkinlik: Yetkinlik) -> Result<(), String> {
        dosya::yolu_denetle(&self.politika, self.cocuk_modu, yol, yetkinlik)
    }

    fn web_gerektir(&self) -> Result<(), String> {
        self.gerektir(Yetkinlik::AgSunucusu)
    }

    fn web_oturumu_gerektir(&self) -> Result<(), String> {
        self.web_gerektir()?;
        self.gerektir(Yetkinlik::WebOturumu)
    }

    fn veritabani_gerektir(&self) -> Result<(), VeritabaniHatasi> {
        self.gerektir(Yetkinlik::Veritabani)
            .map_err(super::yetkinlik_hatasi::veritabani)
    }

    fn gerektir(&self, yetkinlik: Yetkinlik) -> Result<(), String> {
        self.politika
            .gerektir(yetkinlik)
            .map_err(|hata| super::yetkinlik_hatasi::mesaj(hata, self.cocuk_modu, yetkinlik))
    }
}

impl<T: GirdiCikti> GirdiCikti for PolitikaliIo<T> {
    fn yazdir(&mut self, satir: String) {
        self.ic.yazdir(satir);
    }

    fn sor(&mut self, istem: &str) -> Option<String> {
        self.ic.sor(istem)
    }

    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        self.ic.rastgele(alt, ust)
    }

    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)?;
        self.ic.dosya_oku(yol)
    }

    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaYazma)?;
        self.ic.dosya_yaz(yol, satir, ekleme)
    }

    fn dosya_atomik_tasi(&mut self, kaynak: &str, hedef: &str) -> Result<i64, String> {
        self.dosya_yolunu_denetle(kaynak, Yetkinlik::DosyaYazma)?;
        self.dosya_yolunu_denetle(hedef, Yetkinlik::DosyaYazma)?;
        self.ic.dosya_atomik_tasi(kaynak, hedef)
    }

    fn dosya_sil(&mut self, yol: &str) -> Result<i64, String> {
        self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaYazma)?;
        self.ic.dosya_sil(yol)
    }

    fn dosyalari_listele(&mut self, dizin: &str) -> Result<Vec<String>, String> {
        self.dosya_yolunu_denetle(dizin, Yetkinlik::DosyaOkuma)?;
        self.ic.dosyalari_listele(dizin)
    }

    fn dosya_sha256(&mut self, yol: &str) -> Result<String, String> {
        self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)?;
        self.ic.dosya_sha256(yol)
    }

    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.ic.simdi()
    }

    fn argumanlar(&mut self) -> Vec<String> {
        self.ic.argumanlar()
    }

    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        self.gerektir(Yetkinlik::Ag)?;
        self.politika.ag_istegini_denetle(url)?;
        self.ic.http_getir(url, zaman_asimi_ms)
    }

    fn postgresql_oku(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        self.veritabani_gerektir()?;
        self.ic.postgresql_oku(sorgu, parametreler)
    }

    fn postgresql_degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<i64, VeritabaniHatasi> {
        self.veritabani_gerektir()?;
        self.ic.postgresql_degistir(sorgu, parametreler)
    }

    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String> {
        self.web_gerektir()?;
        self.ic.sunucu_kur(kapi)
    }

    fn istek_al(&mut self) -> Option<String> {
        self.politika
            .izin_verir(Yetkinlik::AgSunucusu)
            .then(|| self.ic.istek_al())
            .flatten()
    }

    fn istek_islemini_tamamla(&mut self) -> Result<(), String> {
        self.ic.istek_islemini_tamamla()
    }

    fn istek_islemini_geri_al(&mut self) {
        self.ic.istek_islemini_geri_al();
    }

    fn yanit_gonder(&mut self, yanit: &str) {
        if self.politika.izin_verir(Yetkinlik::AgSunucusu) {
            self.ic.yanit_gonder(yanit);
        }
    }

    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        if self.politika.izin_verir(Yetkinlik::AgSunucusu) {
            self.ic.durum_yaniti_gonder(durum, yanit);
        }
    }

    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        self.web_gerektir()?;
        self.ic.yonlendir_gonder(adres)
    }

    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        self.web_oturumu_gerektir()?;
        self.ic.cerez_yaz(ad, deger)
    }

    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        self.web_oturumu_gerektir()?;
        self.ic.cerez_sil(ad)
    }

    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        self.web_gerektir().map_err(|_| WebReddi {
            durum: 503,
            mesaj: "çalışma politikası ağ sunucusunu açmıyor",
        })?;
        self.ic.rota_guvenligini_denetle(erisim, csrf, csrf_gerekli)
    }

    fn csrf_belirteci(&mut self) -> Result<String, String> {
        self.web_oturumu_gerektir()?;
        self.ic.csrf_belirteci()
    }

    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        self.web_oturumu_gerektir()?;
        self.ic.oturum_ac(kullanici, rol)
    }

    fn oturum_kapat(&mut self) -> Result<(), String> {
        self.web_oturumu_gerektir()?;
        self.ic.oturum_kapat()
    }

    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        self.politika.izin_verir(Yetkinlik::Kriptografi) && self.ic.parola_dogrula(parola, ozet)
    }

    fn eylem_baslat(&mut self) -> Result<(), EylemHatasi> {
        self.ic.eylem_baslat()
    }

    fn eylem_tamamla(&mut self) -> Result<(), EylemHatasi> {
        self.ic.eylem_tamamla()
    }

    fn eylem_geri_al(&mut self) -> Result<(), EylemHatasi> {
        self.ic.eylem_geri_al()
    }

    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        self.politika.izin_verir(Yetkinlik::Donanim) && self.ic.sensor_acik_mi(ad)
    }

    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        if self.politika.izin_verir(Yetkinlik::Donanim) {
            self.ic.isik_ayarla(ad, yansin);
        }
    }

    fn bekle_ms(&mut self, milisaniye: i64) {
        self.ic.bekle_ms(milisaniye);
    }

    fn an_ms(&mut self) -> i64 {
        self.ic.an_ms()
    }
}
