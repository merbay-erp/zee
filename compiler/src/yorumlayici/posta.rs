//! K-390 (RFC-0044 B1b/B2b, ADR-177): posta kimlik doğrulama ilkellerinin
//! yorumlayıcı sınırı. Kurumsal alan host'un açılışta yüklediği kamu son ek
//! listesinden (IO izinde `kurumsal_alan`), DKIM doğrulaması saf kriptografidir
//! ve gövde özeti saf kriptografidir (iz olayı yok: aynı girdi aynı
//! sonucu verir). K-397 B3 imzası proje anahtarını kullanır (IO izinde
//! `dkim_imzala`: anahtar adı, algoritma, girdi SHA-256'sı). Hata kodu C034.
use super::*;
use crate::intrinsic::{
    POSTA_DKIM_DOGRULA, POSTA_DKIM_DOSYA_GOVDE_OZETI, POSTA_DKIM_GOVDE_OZETI, POSTA_DKIM_IMZALA,
    POSTA_KURUMSAL_ALAN,
};

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
        (
            POSTA_DKIM_DOSYA_GOVDE_OZETI,
            [Deger::Metin(yol), Deger::TamSayi(bas), Deger::Metin(kip)],
        ) => Ok(dosya_govde_ozeti(io, yol, *bas, kip)),
        (POSTA_DKIM_IMZALA, [Deger::Metin(veri), Deger::Metin(anahtar), Deger::Metin(a)]) => {
            Ok(dkim_imzala(io, veri, anahtar, a))
        }
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

/// K-397 B3 (ADR-174): imza kâhini önlemi saf olarak burada (anlamlı tanı
/// için) ve adaptörde yeniden (savunma derinliği) uygulanır; `a=` Zee'nin
/// verdiği algoritmayla aynı olmalı. Başarı `b=` Metni; ret C034 `tür`.
#[cfg(not(target_arch = "wasm32"))]
fn dkim_imzala(io: &mut dyn GirdiCikti, veri: &str, anahtar: &str, algoritma: &str) -> Deger {
    use crate::dkim_kripto::{DkimAlgoritmasi, DkimHatasi};
    let on_denetim = DkimAlgoritmasi::ayristir(algoritma).and_then(|istenen| {
        let etiketler = crate::dkim_imza::imza_girdisini_denetle(veri)?;
        if etiketler.algoritma != istenen {
            return Err(DkimHatasi::GirdiGecersiz(
                "DKIM-Signature `a=` değeri imzalama algoritmasıyla aynı değil",
            ));
        }
        Ok(istenen)
    });
    let istenen = match on_denetim {
        Ok(istenen) => istenen,
        Err(hata) => return ret(hata.turu(), hata.mesaji()),
    };
    match io.dkim_imzala(anahtar, istenen.adi(), veri) {
        Ok(b) => Deger::Sonuc {
            basarili: true,
            icerik: Box::new(metin(b)),
        },
        Err(tur) => match DkimHatasi::turden(&tur) {
            Some(hata) => ret(hata.turu(), hata.mesaji()),
            None => ret("yetkinlik", &tur),
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn dkim_imzala(_io: &mut dyn GirdiCikti, _veri: &str, _anahtar: &str, _a: &str) -> Deger {
    ret("desteklenmiyor", "DKIM imzası tarayıcı oyun alanında yok")
}

/// K-397 B3 (ADR-174 §8, ZM-F046): dosyanın `bas` baytından başlayan
/// gövdesinin simple/relaxed kanonik akışlı `bh=` özeti. Dosya dış veridir:
/// CR ret Sonucudur, duruş değildir. `tür`: `geçersiz` (kip ya da baş),
/// `veri_bozuk` (LF'siz CR), `sınır` (32 MiB), `dosya` (okuma/izin).
fn dosya_govde_ozeti(io: &mut dyn GirdiCikti, yol: &str, bas: i64, kip: &str) -> Deger {
    if !matches!(kip, "simple" | "relaxed") {
        return ret(
            "geçersiz",
            "DKIM gövde kipi yalnız \"simple\" ya da \"relaxed\" olabilir",
        );
    }
    if bas < 0 {
        return ret("geçersiz", "DKIM gövde başı negatif olamaz");
    }
    match io.dosya_dkim_govde_ozeti(yol, bas, kip) {
        Ok(bh) => Deger::Sonuc {
            basarili: true,
            icerik: Box::new(metin(bh)),
        },
        Err(hata) if hata == "veri_bozuk" => ret(
            "veri_bozuk",
            "DKIM gövdesinde LF'siz CR var; satır sonu LF ya da CRLF olmalı",
        ),
        Err(hata) if hata.starts_with("sınır") => ret("sınır", &hata),
        Err(hata) if hata.starts_with("aralık") => ret("geçersiz", &hata),
        Err(hata) => ret("dosya", &hata),
    }
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
