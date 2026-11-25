//! K-381 (RFC-0039 A2a): o anki bağlantı görevinin satır kalıpları;
//! K-384 (A2c): blok, bayt, parça, dosya aktarımı ve süreli okuma kalıpları;
//! K-396 (A2d): dosyanın satırlı base64 aktarımı ve kodlanmış aralığı.

use super::*;
use crate::intrinsic::{
    BAGLANTI_BASE64_AKTAR, BAGLANTI_BASE64_ARALIK_AKTAR, BAGLANTI_BAYT_BLOGU, BAGLANTI_BAYT_OKU,
    BAGLANTI_DOSYA_AKTAR, BAGLANTI_NOKTA_BLOGU, BAGLANTI_PARCA_YAZ, BAGLANTI_SATIR_OKU,
    BAGLANTI_SATIR_OKU_SURELI, BAGLANTI_SATIR_YAZ, BAGLANTI_TLS_BASLAT,
};

/// Kalıp: (önek, sonek, intrinsic, aradaki ifade). Sıra önemlidir: süreli
/// satır okuma, argümansız satır okumadan önce denenmez (ara boş olamaz).
type Kalip = (
    &'static [&'static str],
    &'static [&'static str],
    &'static str,
    bool,
);

const KALIPLAR: &[Kalip] = &[
    (
        &["bağlantıdan", "satır", "okumayı", "dene"],
        &[],
        BAGLANTI_SATIR_OKU,
        false,
    ),
    (
        &["bağlantıda", "TLS", "başlatmayı", "dene"],
        &[],
        BAGLANTI_TLS_BASLAT,
        false,
    ),
    // K-384: SMTP DATA gövdesi.
    (
        &[
            "bağlantıdan",
            "nokta",
            "bloğunu",
            "dosyaya",
            "almayı",
            "dene",
        ],
        &[],
        BAGLANTI_NOKTA_BLOGU,
        false,
    ),
    // K-384: kısa süreli satır okuma.
    (
        &["bağlantıdan", "satır", "okumayı"],
        &["saniye", "içinde", "dene"],
        BAGLANTI_SATIR_OKU_SURELI,
        true,
    ),
    // K-384: IMAP literal'i dosyaya, bayt-kesin okuma, parça, aktarım.
    (
        &["bağlantıdan"],
        &["baytı", "dosyaya", "almayı", "dene"],
        BAGLANTI_BAYT_BLOGU,
        true,
    ),
    (
        &["bağlantıdan"],
        &["bayt", "okumayı", "dene"],
        BAGLANTI_BAYT_OKU,
        true,
    ),
    (
        &["bağlantıya"],
        &["parçasını", "yazmayı", "dene"],
        BAGLANTI_PARCA_YAZ,
        true,
    ),
    (
        &["bağlantıya"],
        &["dosyasını", "aktarmayı", "dene"],
        BAGLANTI_DOSYA_AKTAR,
        true,
    ),
    (
        &["bağlantıya"],
        &["satırını", "yazmayı", "dene"],
        BAGLANTI_SATIR_YAZ,
        true,
    ),
    // K-396: `YOL dosyasını base64 satırlarıyla bağlantıya aktarmayı dene`.
    (
        &[],
        &[
            "dosyasını",
            "base64",
            "satırlarıyla",
            "bağlantıya",
            "aktarmayı",
            "dene",
        ],
        BAGLANTI_BASE64_AKTAR,
        true,
    ),
];

/// K-396: `YOL dosyasının base64 satırlarının KONUM baytından N baytını
/// bağlantıya aktarmayı dene` — üç tekil argüman (K-389 aralık biçimi gibi).
fn base64_aralik_kalibi(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    const SABIT: [(usize, &str); 8] = [
        (1, "dosyasının"),
        (2, "base64"),
        (3, "satırlarının"),
        (5, "baytından"),
        (7, "baytını"),
        (8, "bağlantıya"),
        (9, "aktarmayı"),
        (10, "dene"),
    ];
    if tokenlar.len() != 11 || !SABIT.iter().all(|(i, k)| kelime_esit(&tokenlar[*i], k)) {
        return Ok(None);
    }
    Ok(Some(Ifade::Intrinsic {
        kimlik: BAGLANTI_BASE64_ARALIK_AKTAR.into(),
        argumanlar: vec![
            tekil_ifade(tokenlar[0].clone())?,
            tekil_ifade(tokenlar[4].clone())?,
            tekil_ifade(tokenlar[6].clone())?,
        ],
    }))
}

fn kelime_esit(token: &Token, beklenen: &str) -> bool {
    matches!(&token.tur, TokenTur::Kelime(k) if k == beklenen)
}

pub(super) fn baglanti_kalibi(
    tokenlar: &[Token],
    islemler: &[String],
) -> Result<Option<Ifade>, Tani> {
    if let Some(ifade) = base64_aralik_kalibi(tokenlar)? {
        return Ok(Some(ifade));
    }
    let n = tokenlar.len();
    for (onek, sonek, kimlik, arguman) in KALIPLAR {
        let sabit = onek.len() + sonek.len();
        let boyut_uyar = if *arguman { n > sabit } else { n == sabit };
        if !boyut_uyar
            || !onek.iter().zip(tokenlar).all(|(k, t)| kelime_esit(t, k))
            || !sonek
                .iter()
                .zip(&tokenlar[n - sonek.len()..])
                .all(|(k, t)| kelime_esit(t, k))
        {
            continue;
        }
        let argumanlar = if *arguman {
            let ara = &tokenlar[onek.len()..n - sonek.len()];
            vec![ile_ifadesi(ara, tokenlar[0].satir, islemler)?]
        } else {
            Vec::new()
        };
        return Ok(Some(Ifade::Intrinsic {
            kimlik: (*kimlik).into(),
            argumanlar,
        }));
    }
    Ok(None)
}
