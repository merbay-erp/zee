//! K-405 (ADR-179): native web adaptörünün makine uç noktası yolu.
//!
//! Makine ucu (`herkese açık makine uç noktası`) Origin/CSRF kapısından
//! muaftır; karşılığında istek çerezi okunmaz, oturum anonim başlar ve yanıt
//! hiçbir koşulda Set-Cookie taşımaz. Gövde form olarak ayrıştırılmaz:
//! yalnız XML/JSON içerik türü kabul edilir (415), 8 KiB tavanı gövde
//! okunmadan uygulanır (413), gövde katı UTF-8 olmalıdır (400).

use super::*;

const MAKINE_TURLERI: [&str; 3] = ["text/xml", "application/xml", "application/json"];

/// Tek Content-Type; tür listede, `charset` varsa yalnız `utf-8`.
fn icerik_turunu_denetle(baslik: &HttpIstekBasligi<'_>) -> Result<(), (u16, &'static str)> {
    let turler = baslik.baslik_degerleri("Content-Type");
    let tur = match turler.as_slice() {
        [tur] => *tur,
        [] => return Err((415, "makine uç noktası Content-Type istiyor")),
        _ => return Err((400, "birden çok Content-Type başlığı reddedildi")),
    };
    let mut parcalar = tur.split(';');
    let medya = parcalar.next().unwrap_or_default().trim();
    if !MAKINE_TURLERI
        .iter()
        .any(|izinli| medya.eq_ignore_ascii_case(izinli))
    {
        return Err((
            415,
            "makine uç noktası yalnız text/xml, application/xml ya da application/json kabul eder",
        ));
    }
    let utf8_disi = parcalar
        .filter_map(|p| p.split_once('='))
        .any(|(ad, deger)| {
            ad.trim().eq_ignore_ascii_case("charset")
                && !deger.trim().trim_matches('"').eq_ignore_ascii_case("utf-8")
        });
    if utf8_disi {
        return Err((415, "makine uç noktası yalnız UTF-8 gövde kabul eder"));
    }
    Ok(())
}

impl GercekIo {
    /// POST ve sorgusuz yol derlenen makine yolları kümesinde.
    pub(super) fn makine_istegi_mi(&self, baslik: &HttpIstekBasligi<'_>) -> bool {
        let yol = baslik.hedef().split('?').next().unwrap_or_default();
        baslik.yontem().eq_ignore_ascii_case("POST")
            && self
                .guvenilir_betikler
                .makine_yollari
                .get()
                .is_some_and(|yollar| yollar.contains(yol))
    }

    /// Makine isteğinin zarfı: istek satırı, açık boş çerez satırı ve ham
    /// gövde. Yanıt yazıldıysa `None`.
    pub(super) fn makine_zarfini_kur(
        &mut self,
        akis: &mut web_metrikleri::Akis,
        baslik: &HttpIstekBasligi<'_>,
        ilk_govde: &[u8],
        es: std::net::SocketAddr,
        son_tarih: std::time::Instant,
        https: bool,
    ) -> Option<(String, Option<web_yukleme::IstekYuklemesi>, usize)> {
        let sinir = dil::agac::MAKINE_GOVDE_SINIRI;
        if let Err((durum, mesaj)) = icerik_turunu_denetle(baslik) {
            ham_http_hatasi_gonder(akis, durum, mesaj, false, https);
            return None;
        }
        let oturum_cerez_adi = self.oturum_cerez_adi().to_string();
        let govde = match istek_govdesini_oku(
            akis,
            &self.kok,
            baslik,
            ilk_govde,
            son_tarih,
            https,
            &oturum_cerez_adi,
            sinir,
        ) {
            Ok((govde, _)) => govde,
            Err(GovdeReddi::Gonderildi) => return None,
            Err(red) => {
                red.ham_gonder(akis, false, https);
                return None;
            }
        };
        self.web_istegini_baslat(akis, baslik, es, https)?;
        let zarf = format!("{} {}\nçerez \n{govde}", baslik.yontem(), baslik.hedef());
        Some((zarf, None, sinir))
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn tur_sonucu(tur: Option<&str>) -> Result<(), (u16, &'static str)> {
        let mut ham = String::from("POST /a HTTP/1.1\r\nHost: x\r\n");
        if let Some(tur) = tur {
            ham.push_str(&format!("Content-Type: {tur}\r\n"));
        }
        ham.push_str("\r\n");
        let baslik = HttpIstekBasligi::ayristir(ham.as_bytes()).expect("başlık");
        icerik_turunu_denetle(&baslik)
    }

    #[test]
    fn makine_ucu_yalniz_xml_json_ve_utf8_icerik_turunu_kabul_eder() {
        for tur in [
            "text/xml",
            "text/xml; charset=utf-8",
            "Application/XML; charset=\"UTF-8\"",
            "application/json",
        ] {
            assert_eq!(tur_sonucu(Some(tur)), Ok(()), "{tur}");
        }
        for tur in [
            "text/plain",
            "application/x-www-form-urlencoded",
            "multipart/form-data; boundary=a",
            "application/octet-stream",
            "text/xml; charset=iso-8859-9",
            "text/html",
        ] {
            assert_eq!(tur_sonucu(Some(tur)).map_err(|h| h.0), Err(415), "{tur}");
        }
        assert_eq!(tur_sonucu(None).map_err(|h| h.0), Err(415));
    }
}
