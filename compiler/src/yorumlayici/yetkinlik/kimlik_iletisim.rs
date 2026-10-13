//! K-210 kimlik/iletişim/görsel kapıları; trait yöntemlerini aynı politika ile açar.
macro_rules! yontemler {
    () => {
        fn parola_ozeti_uret(&mut self, parola: &str) -> Result<String, String> {
            self.politika.gerektir(Yetkinlik::Kriptografi)?;
            self.ic.parola_ozeti_uret(parola)
        }

        fn gizli_sifrele(&mut self, duz: &str) -> Result<String, String> {
            self.politika.gerektir(Yetkinlik::Kriptografi)?;
            self.ic.gizli_sifrele(duz)
        }

        fn gizli_coz(&mut self, sifreli: &str) -> Result<String, String> {
            self.politika.gerektir(Yetkinlik::Kriptografi)?;
            self.ic.gizli_coz(sifreli)
        }

        fn totp_anahtari_uret(&mut self) -> Result<String, String> {
            self.politika.gerektir(Yetkinlik::Kriptografi)?;
            self.ic.totp_anahtari_uret()
        }

        fn totp_dogrula(&mut self, gizli: &str, kod: &str) -> bool {
            self.politika.izin_verir(Yetkinlik::Kriptografi) && self.ic.totp_dogrula(gizli, kod)
        }

        fn eposta_gonder(
            &mut self,
            baglanti: Option<&str>,
            alici: &str,
            konu: &str,
            govde: &str,
        ) -> Result<i64, String> {
            self.politika.gerektir(Yetkinlik::Ag)?;
            self.ic.eposta_gonder(baglanti, alici, konu, govde)
        }

        fn gorsel_kucult(&mut self, kaynak: &str, hedef: &str, azami: i64) -> Result<i64, String> {
            self.dosya_yolunu_denetle(kaynak, Yetkinlik::DosyaOkuma)?;
            self.dosya_yolunu_denetle(hedef, Yetkinlik::DosyaYazma)?;
            self.ic.gorsel_kucult(kaynak, hedef, azami)
        }
    };
}
pub(super) use yontemler;
