//! Beklenen dosya yaşam döngüsü hatalarını `Sonuç` olarak taşıyan intrinsic'ler.

use super::*;

pub(super) fn intrinsic_degerlendir(
    kimlik: &str,
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    son_tarihi_denetle(io, satir)?;
    let sonuc = match (kimlik, degerler) {
        (DOSYA_ATOMIK_TASI, [Deger::Metin(kaynak), Deger::Metin(hedef)]) => {
            io.dosya_atomik_tasi(kaynak, hedef).map(Deger::TamSayi)
        }
        (DOSYA_SIL, [Deger::Metin(yol)]) => io.dosya_sil(yol).map(Deger::TamSayi),
        (DOSYALARI_LISTELE, [Deger::Metin(dizin)]) => io
            .dosyalari_listele(dizin)
            .map(|yollar| Deger::Liste(yollar.into_iter().map(Deger::Metin).collect())),
        (DOSYA_SHA256, [Deger::Metin(yol)]) => io.dosya_sha256(yol).map(Deger::Metin),
        _ => return Err(ic_hata(satir)),
    };
    Ok(match sonuc {
        Ok(deger) => Deger::Sonuc {
            basarili: true,
            icerik: Box::new(deger),
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
