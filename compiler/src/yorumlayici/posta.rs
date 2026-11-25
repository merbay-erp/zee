//! K-390 (RFC-0044 B1b/B2b, ADR-175): posta kimlik doğrulama ilkellerinin
//! yorumlayıcı sınırı. Kurumsal alan host'un açılışta yüklediği kamu son ek
//! listesinden (IO izinde `kurumsal_alan`), DKIM doğrulaması saf kriptografidir
//! ve gövde özeti saf kriptografidir (iz olayı yok: aynı girdi aynı
//! sonucu verir). Hata kodu C034.
use super::*;
use crate::intrinsic::{POSTA_DKIM_DOGRULA, POSTA_DKIM_GOVDE_OZETI, POSTA_KURUMSAL_ALAN};

pub(super) fn degerlendir(
    kimlik: &str,
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    match (kimlik, degerler) {
        (POSTA_KURUMSAL_ALAN, [Deger::Metin(ad)]) => {
            io.kurumsal_alan(ad).map(Deger::Metin).map_err(|neden| {
                Tani::yeni(
                    "C034",
                    format!("Kurumsal alan bulunamadı: {neden}."),
                    satir,
                    1,
                    1,
                )
            })
        }
        (POSTA_DKIM_GOVDE_OZETI, [Deger::Metin(govde)]) => govde_ozeti(govde)
            .map(Deger::Metin)
            .map_err(|mesaj| Tani::yeni("C034", mesaj, satir, 1, 1)),
        (
            POSTA_DKIM_DOGRULA,
            [Deger::Metin(b), Deger::Metin(veri), Deger::Metin(p), Deger::Metin(a)],
        ) => Ok(dkim_dogrula(a, p, veri, b)),
        _ => Err(ic_hata(satir)),
    }
}

/// CR içeren gövde yürütmeyi durdurur: SMTP alıcısı çıplak CR/LF'yi zaten
/// reddettiği için bu ürün kanonikleştirme hatasıdır, saldırgan girdisi değil.
#[cfg(not(target_arch = "wasm32"))]
fn govde_ozeti(govde: &str) -> Result<String, String> {
    crate::dkim_kripto::govde_ozeti(govde).map_err(|hata| hata.mesaji().to_string())
}

#[cfg(target_arch = "wasm32")]
fn govde_ozeti(_govde: &str) -> Result<String, String> {
    Err("DKIM gövde özeti tarayıcı oyun alanında yok".into())
}

fn metin(deger: impl Into<String>) -> Deger {
    Deger::Metin(deger.into())
}

/// Başarı: `doğrulandı` ("doğru"/"yanlış"), `algoritma`, `anahtar_biti`.
/// İmzanın tutmaması başarılı Sonuç'tur (DKIM `fail`); ayrıştırılamayan
/// ya da politika dışı girdi C034 `tür` verili ret (DKIM `permerror`).
#[cfg(not(target_arch = "wasm32"))]
fn dkim_dogrula(algoritma: &str, p: &str, veri: &str, b: &str) -> Deger {
    match crate::dkim_kripto::base64_imzayi_dogrula(algoritma, p, veri, b) {
        Ok(sonuc) => Deger::Sonuc {
            basarili: true,
            icerik: Box::new(Deger::Sozluk(vec![
                (
                    "doğrulandı".into(),
                    metin(if sonuc.tuttu { "doğru" } else { "yanlış" }),
                ),
                ("algoritma".into(), metin(sonuc.algoritma.adi())),
                ("anahtar_biti".into(), metin(sonuc.bit.to_string())),
            ])),
        },
        Err(hata) => ret(hata.turu(), hata.mesaji()),
    }
}

#[cfg(target_arch = "wasm32")]
fn dkim_dogrula(_algoritma: &str, _p: &str, _veri: &str, _b: &str) -> Deger {
    ret(
        "desteklenmiyor",
        "DKIM doğrulaması tarayıcı oyun alanında yok",
    )
}

fn ret(tur: &str, mesaj: &str) -> Deger {
    Deger::Sonuc {
        basarili: false,
        icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
            kod: "C034".into(),
            mesaj: mesaj.into(),
            neden: None,
            veri: vec![("tür".into(), metin(tur))],
        }))),
    }
}
