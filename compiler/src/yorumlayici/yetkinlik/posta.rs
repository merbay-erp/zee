//! K-390 DNS/kurumsal alan ve K-397 DKIM kapıları (RFC-0044); trait
//! yöntemlerini aynı politika ile açar.
macro_rules! yontemler {
    () => {
        fn dns_sorgula(
            &mut self,
            ad: &str,
            tur: crate::dns_teli::KayitTuru,
            ms: i64,
        ) -> crate::dns_teli::DnsSonucu {
            use crate::dns_teli::{DnsHataTuru, DnsHatasi};
            self.gerektir(Yetkinlik::Dns)
                .map_err(|m| DnsHatasi::yeni(DnsHataTuru::Ag, m))?;
            self.ic.dns_sorgula(ad, tur, ms)
        }

        fn kurumsal_alan(&mut self, ad: &str) -> Result<String, String> {
            self.gerektir(Yetkinlik::Dns)?;
            self.ic.kurumsal_alan(ad)
        }

        /// K-397 B3: dosya gövde özeti `kriptografi` + dosyayı okuma ister.
        fn dosya_dkim_govde_ozeti(
            &mut self,
            yol: &str,
            bas: i64,
            kip: &str,
        ) -> Result<String, String> {
            self.politika.gerektir(Yetkinlik::Kriptografi)?;
            self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)?;
            self.ic.dosya_dkim_govde_ozeti(yol, bas, kip)
        }

        /// K-397 B3: DKIM imzası `kriptografi` ister (anahtar bildirimi adaptörde).
        fn dkim_imzala(
            &mut self,
            anahtar: &str,
            algoritma: &str,
            veri: &str,
        ) -> Result<String, String> {
            self.politika.gerektir(Yetkinlik::Kriptografi)?;
            self.ic.dkim_imzala(anahtar, algoritma, veri)
        }
    };
}
pub(super) use yontemler;
