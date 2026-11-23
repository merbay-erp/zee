use super::*;

pub(super) fn oturum_yonetimini_degerlendir(
    kimlik: &str,
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    let [Deger::Metin(arguman)] = degerler else {
        return Err(ic_hata(satir));
    };
    son_tarihi_denetle(io, satir)?;
    let sonuc = if kimlik == WEB_OTURUMLARINI_LISTELE {
        io.kullanici_oturumlarini_listele(arguman).map(|satirlar| {
            Deger::Liste(
                satirlar
                    .into_iter()
                    .map(|satir| {
                        Deger::Sozluk(
                            satir
                                .into_iter()
                                .map(|(ad, deger)| (ad, Deger::Metin(deger)))
                                .collect(),
                        )
                    })
                    .collect(),
            )
        })
    } else {
        io.oturum_kimligini_iptal_et(arguman).map(Deger::TamSayi)
    };
    Ok(match sonuc {
        Ok(deger) => Deger::Sonuc {
            basarili: true,
            icerik: Box::new(deger),
        },
        Err(mesaj) => Deger::Sonuc {
            basarili: false,
            icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
                kod: "C022".into(),
                mesaj,
                neden: None,
                veri: Vec::new(),
            }))),
        },
    })
}

/// K-370: rota içindeki `csrf belirteci` ve `oturuma al` hatası. Oturum
/// deposunun yeni kaydı reddetmesi (kapasite, depo IO, kilit) kullanıcı/program
/// hatası değil kaynak durumudur: `Ç003` iç nöbetçisi istek döngüsünde commit
/// yoluyla aynı 503 yanıtına çevrilir. Web isteği dışında C022 kalır.
pub(in crate::yorumlayici) fn web_oturumu_tanisi(baslik: &str, hata: &str, satir: usize) -> Tani {
    match hata.strip_prefix(crate::web_guvenligi::OTURUM_DEPOSU_REDDI_ONEKI) {
        Some(neden) => Tani::yeni("Ç003", format!("{baslik}: {neden}."), satir, 1, 1),
        None => Tani::yeni("C022", format!("{baslik}: {hata}."), satir, 1, 1),
    }
}

/// K-378: rota dosya yanıtı. Beklenen dosya/tür/ad hatası C013 `Sonuç`'tur;
/// başarıda yanıt taslağı kurulur ve gönderilecek bayt sayısı döner.
pub(super) fn dosya_yanitini_degerlendir(
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    let [Deger::Metin(yol), Deger::Metin(tur), Deger::Metin(ad)] = degerler else {
        return Err(ic_hata(satir));
    };
    son_tarihi_denetle(io, satir)?;
    Ok(match io.dosya_yaniti_gonder(yol, tur, ad) {
        Ok(bayt) => Deger::Sonuc {
            basarili: true,
            icerik: Box::new(Deger::TamSayi(bayt)),
        },
        Err(mesaj) => Deger::Sonuc {
            basarili: false,
            icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
                kod: "C013".into(),
                mesaj,
                neden: None,
                veri: Vec::new(),
            }))),
        },
    })
}
