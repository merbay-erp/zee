//! Var olan replay HTTP/zaman codec yardımcıları.
use super::*;

pub(super) fn http_getir_oynat(
    io: &mut IzYenidenOynatici,
    url: &str,
    zaman_asimi_ms: Option<i64>,
) -> Result<(i64, String), String> {
    let mut argumanlar = vec![url.into()];
    argumanlar.extend(secenek_yaz(
        zaman_asimi_ms.map(|deger| deger.to_string()).as_deref(),
    ));
    let Some(alanlar) = io.siradaki("http_getir", argumanlar) else {
        return Err(io
            .hata
            .clone()
            .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
    };
    match alanlar.as_slice() {
        [etiket, durum, govde] if etiket == "ok" => match durum.parse() {
            Ok(durum) => Ok((durum, govde.clone())),
            Err(_) => {
                io.bozuk_sonuc("http_getir", "durum kodu sayı değil".into());
                Err(io
                    .hata
                    .clone()
                    .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
            }
        },
        [etiket, hata] if etiket == "hata" => Err(hata.clone()),
        _ => {
            io.bozuk_sonuc("http_getir", "geçersiz HTTP sonuç alanları".into());
            Err(io
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
        }
    }
}

pub(super) fn simdi_oynat(io: &mut IzYenidenOynatici) -> (i64, u32, u32, u32, u32) {
    let sonuc = io.siradaki("simdi", Vec::new());
    let Some(alanlar) = sonuc else {
        return (0, 1, 1, 0, 0);
    };
    let [yil, ay, gun, saat, dakika] = alanlar.as_slice() else {
        io.bozuk_sonuc("simdi", "beş zaman alanı bekleniyor".into());
        return (0, 1, 1, 0, 0);
    };
    match (
        yil.parse(),
        ay.parse(),
        gun.parse(),
        saat.parse(),
        dakika.parse(),
    ) {
        (Ok(yil), Ok(ay), Ok(gun), Ok(saat), Ok(dakika)) => (yil, ay, gun, saat, dakika),
        _ => {
            io.bozuk_sonuc("simdi", "zaman alanlarından biri sayı değil".into());
            (0, 1, 1, 0, 0)
        }
    }
}

pub(super) fn postgresql_oku_oynat(
    io: &mut IzYenidenOynatici,
    sorgu: &str,
    parametreler: &[String],
) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
    let mut argumanlar = vec![sorgu.into()];
    argumanlar.extend(parametreler.iter().cloned());
    let Some(alanlar) = io.siradaki("postgresql_oku", argumanlar) else {
        return Err(VeritabaniHatasi {
            mesaj: io
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()),
            veri: Vec::new(),
        });
    };
    match veritabani_okuma_coz(&alanlar) {
        Ok(sonuc) => sonuc,
        Err(hata) => {
            io.bozuk_sonuc("postgresql_oku", hata);
            Err(VeritabaniHatasi {
                mesaj: io
                    .hata
                    .clone()
                    .unwrap_or_else(|| "IO izi sonucu bozuk".into()),
                veri: Vec::new(),
            })
        }
    }
}

pub(super) fn postgresql_degistir_oynat(
    io: &mut IzYenidenOynatici,
    sorgu: &str,
    parametreler: &[String],
) -> Result<i64, VeritabaniHatasi> {
    let mut argumanlar = vec![sorgu.into()];
    argumanlar.extend(parametreler.iter().cloned());
    let Some(alanlar) = io.siradaki("postgresql_degistir", argumanlar) else {
        return Err(VeritabaniHatasi {
            mesaj: io
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()),
            veri: Vec::new(),
        });
    };
    match veritabani_degistirme_coz(&alanlar) {
        Ok(sonuc) => sonuc,
        Err(hata) => {
            io.bozuk_sonuc("postgresql_degistir", hata);
            Err(VeritabaniHatasi {
                mesaj: io
                    .hata
                    .clone()
                    .unwrap_or_else(|| "IO izi sonucu bozuk".into()),
                veri: Vec::new(),
            })
        }
    }
}
