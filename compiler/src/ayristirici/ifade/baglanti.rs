//! K-381 (RFC-0039 A2a): o anki bağlantı görevinin satır kalıpları;
//! K-384 (A2c): blok, bayt, parça, dosya aktarımı ve süreli okuma kalıpları.

use super::*;
use crate::intrinsic::{
    BAGLANTI_BAYT_BLOGU, BAGLANTI_BAYT_OKU, BAGLANTI_DOSYA_AKTAR, BAGLANTI_NOKTA_BLOGU,
    BAGLANTI_PARCA_YAZ, BAGLANTI_SATIR_OKU, BAGLANTI_SATIR_OKU_SURELI, BAGLANTI_SATIR_YAZ,
    BAGLANTI_TLS_BASLAT, BAGLANTI_TLS_ISTEMCI, GIDEN_BAGLAN,
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
    // K-397 (RFC-0044 B4a): giden TCP ve istemci STARTTLS.
    (&[], &["adresine", "bağlanmayı", "dene"], GIDEN_BAGLAN, true),
    (
        &["bağlantıda", "TLS", "istemcisi", "başlatmayı", "dene"],
        &["ile"],
        BAGLANTI_TLS_ISTEMCI,
        true,
    ),
];

fn kelime_esit(token: &Token, beklenen: &str) -> bool {
    matches!(&token.tur, TokenTur::Kelime(k) if k == beklenen)
}

pub(super) fn baglanti_kalibi(
    tokenlar: &[Token],
    islemler: &[String],
) -> Result<Option<Ifade>, Tani> {
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
