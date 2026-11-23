//! K-210 kimlik/iletişim/görsel ve K-378 dosya yanıtı (ek indirme) kapıları;
//! trait yöntemlerini aynı politika ile açar.
macro_rules! yontemler {
    () => {
        fn dosya_yaniti_gonder(&mut self, yol: &str, tur: &str, ad: &str) -> Result<i64, String> {
            self.web_gerektir()?;
            self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)?;
            self.ic.dosya_yaniti_gonder(yol, tur, ad)
        }

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
            zaman_asimi_ms: i64,
        ) -> Result<i64, String> {
            self.politika.gerektir(Yetkinlik::Ag)?;
            self.ic
                .eposta_gonder(baglanti, alici, konu, govde, zaman_asimi_ms)
        }

        fn gorsel_kucult(&mut self, kaynak: &str, hedef: &str, azami: i64) -> Result<i64, String> {
            self.dosya_yolunu_denetle(kaynak, Yetkinlik::DosyaOkuma)?;
            self.dosya_yolunu_denetle(hedef, Yetkinlik::DosyaYazma)?;
            self.ic.gorsel_kucult(kaynak, hedef, azami)
        }
        fn eposta_mime_gonder(
            &mut self,
            b: Option<&str>,
            a: &str,
            k: &str,
            g: &str,
            s: &crate::eposta::MimeGonderim<'_>,
        ) -> Result<i64, String> {
            self.politika.gerektir(Yetkinlik::Ag)?;
            for yol in s.ekler {
                self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)?;
            }
            self.ic.eposta_mime_gonder(b, a, k, g, s)
        }
    };
}
pub(super) use yontemler;
