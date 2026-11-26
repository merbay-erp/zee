//! K-389 (RFC-0043 Dilim 1, ADR-167): dosya ↔ kodlanmış içerik.
//!
//! IO adaptörü yalnız ham bayt okur/yazar; aralık denetimi, katı UTF-8,
//! base64/QP çözümü ve sonuç biçimi burada saf olarak yapılır. Kodlama
//! hataları C032, dosya/aralık hataları C013 başarısız `Sonuç`tur; hata
//! mesajı içerik taşımaz.

use super::*;
use crate::intrinsic::{
    DOSYA_ARALIK_BASE64, DOSYA_ARALIK_METIN, DOSYA_BASE64_EKLE, DOSYA_BASE64_YAZ, DOSYA_QP_EKLE,
    DOSYA_QP_YAZ,
};
use crate::kodlama::{self, KodlamaHatasi, AZAMI_PARCA_BAYTI};

/// K-389 kimlikleri; dağıtıcının kalan kolu buraya gelir, tanınmayan
/// kimlik iç tutarlılık hatasıdır.
pub(super) fn degerlendir(
    kimlik: &str,
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    son_tarihi_denetle(io, satir)?;
    match (kimlik, degerler) {
        (DOSYA_ARALIK_METIN | DOSYA_ARALIK_BASE64, [Deger::Metin(yol), konum, adet]) => {
            let (Deger::TamSayi(konum), Deger::TamSayi(adet)) = (konum, adet) else {
                return Err(ic_hata(satir));
            };
            araligi_oku(io, yol, *konum, *adet, kimlik == DOSYA_ARALIK_METIN, satir)
        }
        (
            DOSYA_BASE64_YAZ | DOSYA_BASE64_EKLE | DOSYA_QP_YAZ | DOSYA_QP_EKLE,
            [Deger::Metin(icerik), Deger::Metin(yol)],
        ) => {
            let cozulmus = if matches!(kimlik, DOSYA_BASE64_YAZ | DOSYA_BASE64_EKLE) {
                kodlama::base64_mime_coz(icerik, AZAMI_PARCA_BAYTI)
            } else {
                kodlama::qp_coz(icerik, AZAMI_PARCA_BAYTI)
            };
            let baytlar = match cozulmus {
                Ok(baytlar) => baytlar,
                Err(hata) => return Ok(kodlama_hatasi(hata)),
            };
            let ekleme = matches!(kimlik, DOSYA_BASE64_EKLE | DOSYA_QP_EKLE);
            Ok(match io.dosyaya_bayt_yaz(yol, &baytlar, ekleme) {
                Ok((sha256, bayt)) => basarili(Deger::Sozluk(vec![
                    ("yol".into(), Deger::Metin(yol.clone())),
                    ("sha256".into(), Deger::Metin(sha256)),
                    ("bayt".into(), Deger::Metin(bayt.to_string())),
                ])),
                Err(mesaj) => dosya_hatasi(mesaj),
            })
        }
        _ => Err(ic_hata(satir)),
    }
}

fn araligi_oku(
    io: &mut dyn GirdiCikti,
    yol: &str,
    konum: i64,
    adet: i64,
    metin: bool,
    satir: usize,
) -> Result<Deger, Tani> {
    if konum < 0 || !(1..=AZAMI_PARCA_BAYTI as i64).contains(&adet) {
        return Ok(dosya_hatasi(format!(
            "geçersiz aralık: konum 0 ya da daha büyük, bayt sayısı 1 ile {AZAMI_PARCA_BAYTI} arasında olmalı; konum {konum}, bayt {adet} verildi"
        )));
    }
    let baytlar = match io.dosya_araligi_oku(yol, konum, adet) {
        Ok(baytlar) if baytlar.len() as i64 <= adet => baytlar,
        Ok(_) => return Err(ic_hata(satir)),
        Err(mesaj) => return Ok(dosya_hatasi(mesaj)),
    };
    let sonuc = if metin {
        match kodlama::utf8_coz(baytlar, konum as u64) {
            Ok(metin) => metin,
            Err(hata) => return Ok(kodlama_hatasi(hata)),
        }
    } else {
        kodlama::base64_kodla(&baytlar)
    };
    metin_sinirini_denetle(sonuc.len(), satir)?;
    Ok(basarili(Deger::Metin(sonuc)))
}

fn basarili(deger: Deger) -> Deger {
    Deger::Sonuc {
        basarili: true,
        icerik: Box::new(deger),
    }
}

fn dosya_hatasi(mesaj: String) -> Deger {
    Deger::Sonuc {
        basarili: false,
        icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
            kod: "C013".into(),
            mesaj,
            neden: None,
            veri: Vec::new(),
        }))),
    }
}

pub(super) fn kodlama_hatasi(hata: KodlamaHatasi) -> Deger {
    let mut veri = vec![("tür".to_string(), Deger::Metin(hata.tur.into()))];
    if let Some(konum) = hata.konum {
        veri.push(("konum".into(), Deger::Metin(konum.to_string())));
    }
    Deger::Sonuc {
        basarili: false,
        icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
            kod: "C032".into(),
            mesaj: hata.mesaj,
            neden: None,
            veri,
        }))),
    }
}
