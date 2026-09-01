use super::*;

/// Açık imzalı (ve parametresiz) işlemler çağrı beklemeden denetlenir. Böylece
/// kütüphane API'sindeki gövde hatası kullanılmadığı için gizli kalamaz.
pub(super) fn acik_islemleri_denetle(baglam: &mut Baglam) -> Result<(), Tani> {
    let mut adlar = baglam.islemler.keys().cloned().collect::<Vec<_>>();
    adlar.sort();
    for ad in adlar {
        let Some(islem) = baglam.islemler.get(&ad).cloned() else {
            continue;
        };
        let turler = acik_parametre_turleri(&islem, baglam)?;
        let satir = islem.satir;
        if let Some(turler) = turler {
            cagri_denetle(&ad, &turler, baglam, satir)?;
        }
    }
    Ok(())
}


/// K-086 dönüş bildirimi: dış `None` bildirim yok, iç `None` değer döndürmez.
pub(super) fn bildirilmis_donus_turu(
    islem: &Islem,
    baglam: &Baglam,
) -> Result<Option<Option<Tur>>, Tani> {
    let Some(yazim) = islem.donus_turu_yazimi.as_deref() else {
        return Ok(None);
    };
    if yazim == "DeğerDöndürmez" {
        return Ok(Some(None));
    }
    parametre_turu(yazim, baglam)
        .map(|tur| Some(Some(tur)))
        .ok_or_else(|| {
            Tani::yeni(
                "T040",
                format!(
                    "\"{}\" işleminin dönüş türü tanınmadı: \"{}\".",
                    islem.ad, yazim
                ),
                islem.donus_satiri.unwrap_or(islem.satir),
                1,
                1,
            )
            .onerili(
                "Örnekler: `TamSayı döndürür`, `Metin seçeneği döndürür`, `değer döndürmez`."
                    .into(),
            )
        })
}

/// None: bütün parametreler başlangıç yüzeyinde çıkarımlı. Some: açık imza.
pub(super) fn acik_parametre_turleri(
    islem: &Islem,
    baglam: &Baglam,
) -> Result<Option<Vec<Tur>>, Tani> {
    if islem.parametreler.is_empty() {
        return Ok(Some(Vec::new()));
    }
    let acik_sayisi = islem
        .parametreler
        .iter()
        .filter(|parametre| parametre.tur_yazimi.is_some())
        .count();
    if acik_sayisi == 0 {
        if islem.donus_turu_yazimi.is_some() {
            return Err(Tani::yeni(
                "T037",
                format!(
                    "\"{}\" işleminde açık dönüş ile çıkarımlı parametreler karıştırılamaz.",
                    islem.ad
                ),
                islem.donus_satiri.unwrap_or(islem.satir),
                1,
                1,
            )
            .onerili(
                "Dönüş türü yazılıysa bütün parametreleri de `<ad> <Tür> olarak al` biçiminde yaz."
                    .into(),
            ));
        }
        return Ok(None);
    }
    if acik_sayisi != islem.parametreler.len() {
        let satir = islem
            .parametreler
            .iter()
            .find(|parametre| parametre.tur_yazimi.is_none())
            .map(|parametre| parametre.satir)
            .unwrap_or(islem.satir);
        return Err(Tani::yeni(
            "T037",
            format!(
                "\"{}\" işleminde açık ve çıkarımlı parametreler karıştırılamaz.",
                islem.ad
            ),
            satir,
            1,
            1,
        )
        .onerili(
            "İşlemin bütün parametrelerine tür yaz ya da başlangıç yüzeyinde hepsini çıkarımlı bırak."
                .into(),
        ));
    }
    islem
        .parametreler
        .iter()
        .map(|parametre| {
            let yazim = parametre.tur_yazimi.as_deref().ok_or_else(|| {
                ic_tutarlilik_hatasi("Açık parametrenin tür yazımı kayboldu", parametre.satir)
            })?;
            parametre_turu(yazim, baglam).ok_or_else(|| {
                Tani::yeni(
                    "T038",
                    format!(
                        "\"{}\" parametresinin türü tanınmadı: \"{}\".",
                        parametre.ad, yazim
                    ),
                    parametre.satir,
                    1,
                    1,
                )
                .onerili(
                    "Örnekler: TamSayı, Ondalık, Metin, Mantıksal, Ondalık listesi, Metin sözlüğü."
                        .into(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
