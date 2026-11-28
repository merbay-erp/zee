//! RFC-0047 B/C (ADR-182 dilim D): DAV istek başlığı okuma ifadesi ve ETag
//! yanıt başlığı cümlesi.
//!
//! - `isteğin "<ad>" başlığı`: ad kaynakta görünen tırnaklı literaldir ve
//!   kapalı listeyle (büyük/küçük harf duyarsız) eşleşir: `Depth` → Metin,
//!   `If-Match` / `If-None-Match` → Liste<Metin>. Güven başlıkları
//!   (`Authorization`, `Cookie`, `Forwarded`, `X-Forwarded-For`, `Host`,
//!   `Origin`, `X-Zee-CSRF`) S052; liste dışı ya da hesaplanmış ad S051.
//! - `yanıta "ETag" başlığını <ifade> olarak ekle`: başlık adı yalnız `ETag`
//!   literalidir (S053); keyfi başlık yazımı yoktur.

use super::ifade::{ile_ifadesi, kelime_mi, tekil_ifade};
use super::*;
use crate::intrinsic::{WEB_ISTEK_DERINLIGI, WEB_ISTEK_KOSULU};

const GUVEN_BASLIKLARI: [&str; 7] = [
    "authorization",
    "cookie",
    "forwarded",
    "x-forwarded-for",
    "host",
    "origin",
    "x-zee-csrf",
];

fn liste_disi(satir: usize, mesaj: String) -> Tani {
    Tani::yeni("S051", mesaj, satir, 1, 1).onerili(
        "Okunabilen istek başlıkları yalnız \"Depth\", \"If-Match\" ve \"If-None-Match\"tır; ad tırnaklı literal olmalı: `derinlik isteğin \"Depth\" başlığı olsun`."
            .into(),
    )
}

/// `isteğin "<ad>" başlığı`; `None`, ifadenin bu kalıp olmadığını söyler.
pub(super) fn istek_basligi(tokenlar: &[Token], satir: usize) -> Option<Result<Ifade, Tani>> {
    let [ilk, ad_tokeni, son] = tokenlar else {
        return None;
    };
    if !kelime_mi(ilk, "isteğin") || !kelime_mi(son, "başlığı") {
        return None;
    }
    let TokenTur::Metin(ad) = &ad_tokeni.tur else {
        return Some(Err(liste_disi(
            satir,
            "İstek başlığının adı kaynakta görünen tırnaklı bir literal olmalı.".into(),
        )));
    };
    let kucuk = ad.to_ascii_lowercase();
    let (kimlik, arguman) = match kucuk.as_str() {
        "depth" => (WEB_ISTEK_DERINLIGI, false),
        "if-match" | "if-none-match" => (WEB_ISTEK_KOSULU, true),
        _ if GUVEN_BASLIKLARI.contains(&kucuk.as_str()) => {
            return Some(Err(Tani::yeni(
                "S052",
                format!("\"{ad}\" bir güven başlığıdır; Zee kodu bu başlığı okuyamaz."),
                satir,
                1,
                1,
            )
            .onerili(
                "Kimliği rotanın ilk satırına `dav istemcisi gerekli` yazarak çalışma zamanından al ve `oturumun \"kullanıcı\" değeri` ile oku; Origin ve Host denetimini çalışma zamanı yapar."
                    .into(),
            )))
        }
        _ => {
            return Some(Err(liste_disi(
                satir,
                format!("\"{ad}\" okunabilen istek başlıkları listesinde değil."),
            )))
        }
    };
    Some(Ok(Ifade::Intrinsic {
        kimlik: kimlik.into(),
        // Ad argümanı kaynaktaki literal tokendir (kaynak aralığı korunur);
        // çalışma zamanı adı büyük/küçük harf duyarsız karşılaştırır.
        argumanlar: match arguman {
            true => match tekil_ifade(ad_tokeni.clone()) {
                Ok(ifade) => vec![ifade],
                Err(tani) => return Some(Err(tani)),
            },
            false => Vec::new(),
        },
    }))
}

/// `yanıta "ETag" başlığını <ifade> olarak ekle` (son `ekle` atılmış).
pub(super) fn etag_cumlesi(
    tokenlar: &[Token],
    satir: usize,
    islem_adlari: &[String],
) -> Option<Result<Cumle, Tani>> {
    let n = tokenlar.len();
    if n < 5
        || !kelime_mi(&tokenlar[0], "yanıta")
        || !kelime_mi(&tokenlar[2], "başlığını")
        || !kelime_mi(&tokenlar[n - 1], "olarak")
    {
        return None;
    }
    let ad_dogru =
        matches!(&tokenlar[1].tur, TokenTur::Metin(ad) if ad.eq_ignore_ascii_case("etag"));
    if !ad_dogru {
        return Some(Err(Tani::yeni(
            "S053",
            "Yanıta yalnız `ETag` başlığı yazılabilir; keyfi başlık yazımı yoktur.".into(),
            satir,
            1,
            1,
        )
        .onerili("Örnek: `yanıta \"ETag\" başlığını etiket olarak ekle`; öteki başlıkları çalışma zamanı yazar.".into())));
    }
    Some(
        ile_ifadesi(&tokenlar[3..n - 1], satir, islem_adlari)
            .map(|deger| Cumle::YanitEtagi { deger, satir }),
    )
}
