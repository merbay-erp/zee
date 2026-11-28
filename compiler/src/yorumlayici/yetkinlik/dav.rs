//! RFC-0047 (ADR-182) DAV istemci yüzeyi kapıları: uygulama parolası üretimi
//! `kriptografi` ister; trait yöntemlerini aynı politika ile açar.
macro_rules! yontemler {
    () => {
        fn uygulama_parolasi_uret(
            &mut self,
        ) -> Result<crate::guvenlik::uygulama_parolasi::UretilenParola, String> {
            self.politika.gerektir(Yetkinlik::Kriptografi)?;
            self.ic.uygulama_parolasi_uret()
        }

        fn dav_kimligi(&mut self) -> Option<crate::yorumlayici::DavKimligi> {
            self.ic.dav_kimligi()
        }

        fn dav_ozet_dogrula(&mut self, ozet: Option<&str>) -> bool {
            self.politika.izin_verir(Yetkinlik::Kriptografi) && self.ic.dav_ozet_dogrula(ozet)
        }

        fn dav_kovasi(
            &mut self,
            kova: crate::yorumlayici::DavKova,
            anahtar: &str,
        ) -> Result<(), i64> {
            self.ic.dav_kovasi(kova, anahtar)
        }

        fn dav_kimlik_reddi_gonder(&mut self) {
            if self.politika.izin_verir(Yetkinlik::AgSunucusu) {
                self.ic.dav_kimlik_reddi_gonder();
            }
        }

        fn dav_kova_reddi_gonder(&mut self, saniye: i64) {
            if self.politika.izin_verir(Yetkinlik::AgSunucusu) {
                self.ic.dav_kova_reddi_gonder(saniye);
            }
        }

        fn secenekler_gonder(&mut self, izinli: &str) {
            if self.politika.izin_verir(Yetkinlik::AgSunucusu) {
                self.ic.secenekler_gonder(izinli);
            }
        }
    };
}
pub(super) use yontemler;
