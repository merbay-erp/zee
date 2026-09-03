//! Runtime'ın veritabanı intrinsic ve hermetik test adaptörü sahipliği.

use super::*;
use crate::veritabani_modeli::VeritabaniOkumaSonucu;

#[derive(Default)]
pub struct ToplayanVeritabani {
    pub okuma_sonuclari: VecDeque<VeritabaniOkumaSonucu>,
    pub degistirme_sonuclari: VecDeque<Result<i64, VeritabaniHatasi>>,
    pub istekler: Vec<(String, Vec<String>, bool)>,
}

impl ToplayanVeritabani {
    pub(super) fn oku(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        self.istekler
            .push((sorgu.into(), parametreler.to_vec(), false));
        self.okuma_sonuclari.pop_front().unwrap_or_else(|| {
            Err(VeritabaniHatasi {
                mesaj: "sahte PostgreSQL okuma sonucu yok".into(),
                veri: Vec::new(),
            })
        })
    }

    pub(super) fn degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<i64, VeritabaniHatasi> {
        self.istekler
            .push((sorgu.into(), parametreler.to_vec(), true));
        self.degistirme_sonuclari.pop_front().unwrap_or_else(|| {
            Err(VeritabaniHatasi {
                mesaj: "sahte PostgreSQL değişiklik sonucu yok".into(),
                veri: Vec::new(),
            })
        })
    }
}

pub(super) fn desteklenmeyen() -> VeritabaniHatasi {
    VeritabaniHatasi {
        mesaj: "IO adaptörü PostgreSQL erişimini desteklemiyor".into(),
        veri: Vec::new(),
    }
}

pub(super) fn intrinsic_degerlendir(
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
    yazma: bool,
) -> Result<Deger, Tani> {
    let [Deger::Metin(sorgu), Deger::Liste(parametreler)] = degerler else {
        return Err(ic_hata(satir));
    };
    let parametreler = parametreler
        .iter()
        .map(|deger| metne_sinirli(deger, satir))
        .collect::<Result<Vec<_>, _>>()?;
    son_tarihi_denetle(io, satir)?;
    if yazma {
        Ok(match io.postgresql_degistir(sorgu, &parametreler) {
            Ok(adet) => Deger::Sonuc {
                basarili: true,
                icerik: Box::new(Deger::TamSayi(adet)),
            },
            Err(hata) => hata_sonucu(hata),
        })
    } else {
        Ok(match io.postgresql_oku(sorgu, &parametreler) {
            Ok(satirlar) => Deger::Sonuc {
                basarili: true,
                icerik: Box::new(Deger::Liste(
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
                )),
            },
            Err(hata) => hata_sonucu(hata),
        })
    }
}

fn hata_sonucu(hata: VeritabaniHatasi) -> Deger {
    Deger::Sonuc {
        basarili: false,
        icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
            kod: "C025".into(),
            mesaj: hata.mesaj,
            neden: None,
            veri: hata
                .veri
                .into_iter()
                .map(|(ad, deger)| (ad, Deger::Metin(deger)))
                .collect(),
        }))),
    }
}
