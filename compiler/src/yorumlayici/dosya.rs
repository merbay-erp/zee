//! Beklenen dosya yaşam döngüsü hatalarını `Sonuç` olarak taşıyan intrinsic'ler.

use super::*;

/// Bu modülün değerlendirdiği dosya intrinsic'leri (ifade dağıtıcısı buna bakar).
pub(super) const INTRINSICLER: &[&str] = &[
    DOSYA_ATOMIK_TASI,
    DOSYA_SIL,
    DOSYALARI_LISTELE,
    DOSYA_SHA256,
    DOSYA_ILK_BAYTLAR,
    WEB_DOSYA_YANITI,
    DOSYA_BASE64_SATIRLARI_EKLE,
];

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
        (DOSYA_BASE64_SATIRLARI_EKLE, [Deger::Metin(kaynak), Deger::Metin(hedef)]) => io
            .dosyaya_base64_satirlari_ekle(kaynak, hedef)
            .map(Deger::TamSayi),
        (DOSYA_SIL, [Deger::Metin(yol)]) => io.dosya_sil(yol).map(Deger::TamSayi),
        (DOSYALARI_LISTELE, [Deger::Metin(dizin)]) => io
            .dosyalari_listele(dizin)
            .map(|yollar| Deger::Liste(yollar.into_iter().map(Deger::Metin).collect())),
        (DOSYA_SHA256, [Deger::Metin(yol)]) => io.dosya_sha256(yol).map(Deger::Metin),
        (DOSYA_ILK_BAYTLAR, [Deger::Metin(yol), Deger::TamSayi(adet)]) => {
            if (1..=DOSYA_ILK_BAYT_SINIRI).contains(adet) {
                io.dosya_ilk_baytlari(yol, *adet).map(Deger::Metin)
            } else {
                Err(format!(
                    "okunacak bayt sayısı 1 ile {DOSYA_ILK_BAYT_SINIRI} arasında olmalı; {adet} verildi"
                ))
            }
        }
        // K-378: rota dosya yanıtı; ret yanıt yazılmadan C013 Sonuç'tur.
        (WEB_DOSYA_YANITI, [Deger::Metin(yol), Deger::Metin(tur), Deger::Metin(ad)]) => {
            io.dosya_yaniti_gonder(yol, tur, ad).map(Deger::TamSayi)
        }
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
