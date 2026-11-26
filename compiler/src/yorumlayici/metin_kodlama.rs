//! RFC-0043 bellekteki metin ↔ bayt dönüşümleri (ADR-167; K-402 Dilim 2).
//!
//! Hepsi saftır: yetkinlik istemez, IO izine olay yazmaz. Kodlama toplamdır,
//! çözme yakalanabilir C032 `Sonuç`tur; `harfin kod noktası` tek harf
//! olmayan girdide programcı hatası olarak C032 çalışma hatası verir. Hata
//! mesajı girdiyi taşımaz (AUTH PLAIN parolası olabilir).

use super::kodlanmis_dosya::kodlama_hatasi;
use super::*;
use crate::kodlama::{self, KodlamaHatasi, AZAMI_PARCA_BAYTI};

/// Bu modülün değerlendirdiği özellikler (dağıtıcının tek kolu).
pub(super) fn kapsar(ozellik: &Ozellik) -> bool {
    matches!(
        ozellik,
        Ozellik::Sha256Ozeti
            | Ozellik::Utf8BaytSayisi
            | Ozellik::Base64Karsiligi
            | Ozellik::Base64Cozumu
            | Ozellik::KodNoktasi
            | Ozellik::KodNoktasiHarfi
    )
}

pub(super) fn ozellik(ozellik: Ozellik, deger: Deger, satir: usize) -> Result<Deger, Tani> {
    match (ozellik, deger) {
        (Ozellik::Sha256Ozeti, Deger::Metin(m)) => {
            Ok(Deger::Metin(crate::guvenlik::sha256_hex(m.as_bytes())))
        }
        (Ozellik::Utf8BaytSayisi, Deger::Metin(m)) => Ok(Deger::TamSayi(m.len() as i64)),
        (Ozellik::Base64Karsiligi, Deger::Metin(m)) => {
            metin_sinirini_denetle(kodlama::base64_boyu(m.len()), satir)?;
            Ok(Deger::Metin(kodlama::base64_kodla(m.as_bytes())))
        }
        (Ozellik::Base64Cozumu, Deger::Metin(m)) => Ok(base64_coz(&m)),
        (Ozellik::KodNoktasi, Deger::Metin(m)) => {
            let mut harfler = m.chars();
            match (harfler.next(), harfler.next()) {
                (Some(harf), None) => Ok(Deger::TamSayi(i64::from(u32::from(harf)))),
                _ => Err(Tani::yeni(
                    "C032",
                    format!(
                        "`kod noktası` tek harfli metin ister; bu metin {} harfli.",
                        m.chars().count()
                    ),
                    satir,
                    1,
                    1,
                )
                .onerili("Metni önce `harfleri` ile harflerine ayır.".into())),
            }
        }
        (Ozellik::KodNoktasiHarfi, Deger::TamSayi(sayi)) => Ok(kod_noktasi_harfi(sayi)),
        _ => Err(ic_hata(satir)),
    }
}

fn basarili(metin: String) -> Deger {
    Deger::Sonuc {
        basarili: true,
        icerik: Box::new(Deger::Metin(metin)),
    }
}

fn base64_coz(kod: &str) -> Deger {
    let baytlar = match kodlama::base64_coz(kod, AZAMI_PARCA_BAYTI) {
        Ok(baytlar) => baytlar,
        Err(hata) => return kodlama_hatasi(hata),
    };
    match kodlama::utf8_coz(baytlar, 0) {
        Ok(metin) => basarili(metin),
        Err(hata) => kodlama_hatasi(KodlamaHatasi {
            mesaj: "çözülen baytlar geçerli UTF-8 metin değil".into(),
            ..hata
        }),
    }
}

fn kod_noktasi_harfi(sayi: i64) -> Deger {
    match u32::try_from(sayi).ok().and_then(char::from_u32) {
        Some(harf) => basarili(harf.to_string()),
        None => kodlama_hatasi(KodlamaHatasi {
            tur: "kod_noktası",
            konum: None,
            mesaj: "kod noktası 0..=1114111 aralığında ve vekil (55296..=57343) dışında olmalı"
                .into(),
        }),
    }
}
