//! K-210/ADR-083 handler'ı: parola özeti, TOTP, e-posta ve görsel küçültme
//! intrinsic'leri. HTTP/adaptör kararları IO trait'inde kalır; burada yalnız
//! değer/`Sonuç` kurulumu ve tanı kimlikleri vardır.

use super::*;

pub(super) fn intrinsic_degerlendir(
    kimlik: &str,
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    son_tarihi_denetle(io, satir)?;
    match (kimlik, degerler) {
        (PAROLA_OZETI, [parola]) => io
            .parola_ozeti_uret(&metne_sinirli(parola, satir)?)
            .map(Deger::Metin)
            .map_err(|hata| {
                Tani::yeni(
                    "C028",
                    format!("Parola özeti üretilemedi: {}.", hata),
                    satir,
                    1,
                    1,
                )
            }),
        (TOTP_ANAHTARI, []) => io.totp_anahtari_uret().map(Deger::Metin).map_err(|hata| {
            Tani::yeni(
                "C028",
                format!("TOTP anahtarı üretilemedi: {}.", hata),
                satir,
                1,
                1,
            )
        }),
        (TOTP_DOGRULA, [kod, gizli]) => Ok(Deger::Mantiksal(
            io.totp_dogrula(&metne_sinirli(gizli, satir)?, &metne_sinirli(kod, satir)?),
        )),
        (EPOSTA_GONDER, [alici, konu, govde]) => {
            let sonuc = io.eposta_gonder(
                None,
                &metne_sinirli(alici, satir)?,
                &metne_sinirli(konu, satir)?,
                &metne_sinirli(govde, satir)?,
            );
            Ok(eposta_sonucu(sonuc))
        }
        (EPOSTA_GONDER_BAGLANTILI, [alici, konu, govde, baglanti]) => {
            let sonuc = io.eposta_gonder(
                Some(&metne_sinirli(baglanti, satir)?),
                &metne_sinirli(alici, satir)?,
                &metne_sinirli(konu, satir)?,
                &metne_sinirli(govde, satir)?,
            );
            Ok(eposta_sonucu(sonuc))
        }
        (EPOSTA_MIME_GONDER, [alici, konu, govde, html, yanit, Deger::Liste(ekler), baglanti]) => {
            let ekler: Vec<String> = ekler
                .iter()
                .map(|ek| match ek {
                    Deger::Metin(yol) => Ok(yol.clone()),
                    _ => Err(ic_hata(satir)),
                })
                .collect::<Result<_, _>>()?;
            let baglanti = metne_sinirli(baglanti, satir)?;
            let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
                .eposta()
                .zaman_asimi_ms();
            let zaman_asimi_ms =
                son_tarih_kalani(io, satir)?.map_or(azami, |(_, kalan)| kalan.min(azami));
            let sonuc = io.eposta_mime_gonder(
                (!baglanti.is_empty()).then_some(baglanti.as_str()),
                &metne_sinirli(alici, satir)?,
                &metne_sinirli(konu, satir)?,
                &metne_sinirli(govde, satir)?,
                &crate::eposta::MimeGonderim {
                    html: &metne_sinirli(html, satir)?,
                    yanit_adresi: &metne_sinirli(yanit, satir)?,
                    ekler: &ekler,
                    zaman_asimi_ms,
                },
            );
            son_tarihi_denetle(io, satir)?;
            Ok(eposta_sonucu(sonuc.map_err(|_| "MIME e-posta gönderilemedi; sır içeren ayrıntılar gizlendi; otomatik tekrar yok".into())))
        }
        (GIZLI_SIFRELE, [duz]) => io
            .gizli_sifrele(&metne_sinirli(duz, satir)?)
            .map(Deger::Metin)
            .map_err(|hata| {
                Tani::yeni(
                    "C028",
                    format!("Gizli değer şifrelenemedi: {}.", hata),
                    satir,
                    1,
                    1,
                )
            }),
        (GIZLI_COZ, [sifreli]) => Ok(match io.gizli_coz(&metne_sinirli(sifreli, satir)?) {
            Ok(duz) => Deger::Sonuc {
                basarili: true,
                icerik: Box::new(Deger::Metin(duz)),
            },
            Err(mesaj) => Deger::Sonuc {
                basarili: false,
                icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
                    kod: "C028".into(),
                    mesaj,
                    neden: None,
                    veri: Vec::new(),
                }))),
            },
        }),
        (GORSEL_KUCULT, [kaynak, hedef, Deger::TamSayi(azami)]) => {
            let sonuc = io.gorsel_kucult(
                &metne_sinirli(kaynak, satir)?,
                &metne_sinirli(hedef, satir)?,
                *azami,
            );
            Ok(match sonuc {
                Ok(deger) => basarili(deger),
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
        _ => Err(ic_hata(satir)),
    }
}

/// E-posta gönderimi `Sonuç` değeri: başarıda 1, hatada C029.
fn eposta_sonucu(sonuc: Result<i64, String>) -> Deger {
    match sonuc {
        Ok(deger) => basarili(deger),
        Err(mesaj) => Deger::Sonuc {
            basarili: false,
            icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
                kod: "C029".into(),
                mesaj,
                neden: None,
                veri: Vec::new(),
            }))),
        },
    }
}

fn basarili(deger: i64) -> Deger {
    Deger::Sonuc {
        basarili: true,
        icerik: Box::new(Deger::TamSayi(deger)),
    }
}
