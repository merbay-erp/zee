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
