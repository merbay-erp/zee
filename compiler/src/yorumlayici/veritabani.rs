//! Runtime'ın veritabanı intrinsic ve hermetik test adaptörü sahipliği.

use super::*;
use crate::veritabani_modeli::VeritabaniOkumaSonucu;

#[derive(Default)]
pub struct ToplayanVeritabani {
    pub okuma_sonuclari: VecDeque<VeritabaniOkumaSonucu>,
    pub degistirme_sonuclari: VecDeque<Result<i64, VeritabaniHatasi>>,
    pub istekler: Vec<(String, Vec<String>, bool)>,
    /// K-375: adaptöre verilen son tarih bütçeleri (ms), çağrı sırasıyla.
    pub sureler: Vec<Option<i64>>,
    /// K-375: hermetik sunucunun her sorguda harcadığı sanal süre. Bütçe bundan
    /// kısaysa sorgu bütçe kadar sürer ve `57014` son tarih iptaliyle döner.
    pub sorgu_suresi_ms: Option<i64>,
}

/// Gerçek adaptörün `statement_timeout`/`lock_timeout` iptaliyle aynı yapı.
pub(super) fn son_tarih_iptali(sqlstate: &str) -> VeritabaniHatasi {
    VeritabaniHatasi {
        mesaj: "PostgreSQL sorgusu son tarih dolduğu için sunucuda iptal edildi".into(),
        veri: vec![
            ("sqlstate".into(), sqlstate.into()),
            (
                "hata_sinifi".into(),
                crate::veritabani_modeli::SON_TARIH_IPTALI_SINIFI.into(),
            ),
        ],
    }
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
    // K-375/RFC-0042: kalan son tarih adaptöre sorgu bütçesi olarak gider;
    // son tarih yoksa `None` ve adaptör davranışı (round trip sayısı) değişmez.
    let son = son_tarih_kalani(io, satir)?;
    let zaman_asimi_ms = son.map(|(_, kalan)| kalan);
    let sonuc = if yazma {
        io.postgresql_degistir(sorgu, &parametreler, zaman_asimi_ms)
            .map(VeritabaniSonucu::Adet)
    } else {
        io.postgresql_oku(sorgu, &parametreler, zaman_asimi_ms)
            .map(VeritabaniSonucu::Satirlar)
    };
    // Süre dolmuş sorgu değere dönüşmez (HTTP/SMTP modeli): sunucu iptali
    // `db.deadline` ya da dönüşte dolmuş son tarih, sahibine Ç001 olarak gider.
    if let (Err(hata), Some((son, _))) = (&sonuc, son) {
        if hata.son_tarih_iptali_mi() {
            return Err(son_tarih_tanisi(son, satir));
        }
    }
    son_tarihi_denetle(io, satir)?;
    match sonuc {
        Err(hata) => Ok(hata_sonucu(hata)),
        Ok(VeritabaniSonucu::Adet(adet)) => Ok(Deger::Sonuc {
            basarili: true,
            icerik: Box::new(Deger::TamSayi(adet)),
        }),
        Ok(VeritabaniSonucu::Satirlar(satirlar)) => Ok(Deger::Sonuc {
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
        }),
    }
}

enum VeritabaniSonucu {
    Adet(i64),
    Satirlar(Vec<Vec<(String, String)>>),
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
