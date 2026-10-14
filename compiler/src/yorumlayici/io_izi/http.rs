//! K-220 HTTP yöntemleri; mevcut trait/politika ve iz sınırının tek sahibi.
macro_rules! yontemler {
    () => {
        fn http_getir(
            &mut self,
            url: &str,
            zaman_asimi_ms: Option<i64>,
        ) -> Result<(i64, String), String> {
            let sonuc = self.ic.http_getir(url, zaman_asimi_ms);
            let mut argumanlar = vec![url.into()];
            argumanlar.extend(secenek_yaz(
                zaman_asimi_ms.map(|deger| deger.to_string()).as_deref(),
            ));
            let iz_sonucu = match &sonuc {
                Ok((durum, govde)) => vec!["ok".into(), durum.to_string(), govde.clone()],
                Err(hata) => vec!["hata".into(), hata.clone()],
            };
            self.kaydet("http_getir", argumanlar, iz_sonucu);
            sonuc
        }

        // K-220: gizli HTTP/ortam işlemi iz kipinde yürütülmez; export da kapanır.
        // Replay trait varsayılanıyla fail-closed kalır; başarı uydurulmaz.
        fn http_istek(
            &mut self,
            url: &str,
            yontem: &str,
            basliklar: &[(String, String)],
            govde: &str,
            zaman_asimi_ms: i64,
        ) -> Result<(i64, String), String> {
            let _ = (url, yontem, basliklar, govde, zaman_asimi_ms);
            self.hata = Some("Sır taşıyabilen HTTP işlemi IO iz kaydında kapalı".into());
            Err("HTTP işlemi IO iz kaydında kapalı".into())
        }
        fn http_gizli_ortam(&mut self, ad: &str) -> Result<String, String> {
            let _ = ad;
            self.hata = Some("Gizli ortam erişimi IO iz kaydında kapalı".into());
            Err("Gizli ortam erişimi IO iz kaydında kapalı".into())
        }
    };
}
pub(super) use yontemler;
