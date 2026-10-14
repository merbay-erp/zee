//! K-220 HTTP yöntemleri; mevcut trait/politika ve iz sınırının tek sahibi.
macro_rules! yontemler {
    () => {
        fn http_getir(
            &mut self,
            url: &str,
            zaman_asimi_ms: Option<i64>,
        ) -> Result<(i64, String), String> {
            self.gerektir(Yetkinlik::Ag)?;
            self.politika.ag_istegini_denetle(url)?;
            self.ic.http_getir(url, zaman_asimi_ms)
        }

        fn http_istek(
            &mut self,
            url: &str,
            yontem: &str,
            basliklar: &[(String, String)],
            govde: &str,
            zaman_asimi_ms: i64,
        ) -> Result<(i64, String), String> {
            self.gerektir(Yetkinlik::Ag)
                .map_err(|_| "HTTP ağ yetkinliği kapalı".to_string())?;
            self.politika
                .ag_istegini_denetle(url)
                .map_err(|_| "HTTP hedefi izinli değil".to_string())?;
            self.ic
                .http_istek(url, yontem, basliklar, govde, zaman_asimi_ms)
        }
        fn http_gizli_ortam(&mut self, ad: &str) -> Result<String, String> {
            self.gerektir(Yetkinlik::Ag)
                .map_err(|_| "HTTP ağ yetkinliği kapalı".to_string())?;
            self.ic.http_gizli_ortam(ad)
        }
    };
}
pub(super) use yontemler;
