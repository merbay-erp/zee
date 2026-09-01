use super::*;

/// Sonuç dönen işlemin başarı dallarındaki `döndür`leri sarmalama için
/// işaretler (RFC-0008 §4.1). HataDondur zaten Sonuç-hata üretir, dokunulmaz.
pub(super) fn donusleri_sarmala(cumleler: &mut [Cumle]) {
    for cumle in cumleler {
        match cumle {
            Cumle::Dondur { sonuca_sarmala, .. } => *sonuca_sarmala = true,
            Cumle::KezTekrarla { govde, .. }
            | Cumle::AralikDongusu { govde, .. }
            | Cumle::OlduguSurece { govde, .. }
            | Cumle::OlanaKadar { govde, .. }
            | Cumle::HerBiri { govde, .. } => donusleri_sarmala(govde),
            Cumle::Ise { kollar, degilse, .. } => {
                for kol in kollar {
                    donusleri_sarmala(&mut kol.govde);
                }
                if let Some(blok) = degilse {
                    donusleri_sarmala(blok);
                }
            }
            Cumle::Gore { kollar, degilse, .. } => {
                for (_, govde) in kollar {
                    donusleri_sarmala(govde);
                }
                if let Some(blok) = degilse {
                    donusleri_sarmala(blok);
                }
            }
            _ => {}
        }
    }
}


/// Açık dönüş sözleşmesinde değer beklenen bir işlemin hiçbir olağan akışta
/// gövde sonuna düşmediğini muhafazakâr biçimde kanıtlar.
pub(super) fn blok_kesin_sonlanir(cumleler: &[Cumle]) -> bool {
    cumleler.iter().any(cumle_kesin_sonlanir)
}

pub(super) fn cumle_kesin_sonlanir(cumle: &Cumle) -> bool {
    match cumle {
        Cumle::Dondur { .. } | Cumle::HataDondur { .. } | Cumle::ProgramiBitir { .. } => true,
        Cumle::Ise { kollar, degilse, .. } => {
            !kollar.is_empty()
                && kollar.iter().all(|kol| blok_kesin_sonlanir(&kol.govde))
                && degilse.as_deref().is_some_and(blok_kesin_sonlanir)
        }
        Cumle::Gore { kollar, degilse, .. } => {
            !kollar.is_empty()
                && kollar.iter().all(|(_, govde)| blok_kesin_sonlanir(govde))
                && degilse.as_deref().is_some_and(blok_kesin_sonlanir)
        }
        Cumle::IcindeBlogu { govde, yetismezse, .. } => {
            blok_kesin_sonlanir(govde)
                && yetismezse.as_deref().is_some_and(blok_kesin_sonlanir)
        }
        _ => false,
    }
}

/// Dönüş dallarını tek türe birleştirir: {T}→T; {T,Yok}→Seçenek<T>;
/// {T,HataDonusu}→Sonuç<T>; boş→None; tutarsızlık→T018.
pub(super) fn donusleri_birlestir(ad: &str, donusler: &[Tur], satir: usize) -> Result<Option<Tur>, Tani> {
    let mut ayrik: Vec<Tur> = Vec::new();
    for t in donusler {
        if !ayrik.contains(t) {
            ayrik.push(*t);
        }
    }

    if ayrik.contains(&Tur::HataDonusu) {
        let degerler: Vec<Tur> = ayrik
            .iter()
            .copied()
            .filter(|t| *t != Tur::HataDonusu)
            .collect();
        return match degerler.as_slice() {
            [] => Err(Tani::yeni(
                "T018",
                format!("\"{}\" yalnız hata döndürüyor; en az bir dalda değer döndür.", ad),
                satir,
                1,
                1,
            )),
            [tek] => match tek.veri_turu() {
                Some(veri) => Ok(Some(Tur::Sonuc(veri))),
                None => Err(Tani::yeni(
                    "T018",
                    format!("\"{}\" Sonuç içinde {} taşıyamaz (v0).", ad, tek.adi()),
                    satir,
                    1,
                    1,
                )),
            },
            _ => Err(Tani::yeni(
                "T018",
                format!(
                    "\"{}\" hata ile birlikte birden çok değer türü döndürüyor; tek tür seç.",
                    ad
                ),
                satir,
                1,
                1,
            )),
        };
    }

    match ayrik.as_slice() {
        [] => Ok(None),
        [tek] => {
            if *tek == Tur::Yok {
                return Err(Tani::yeni(
                    "T018",
                    format!(
                        "\"{}\" yalnız \"yok\" döndürüyor; Seçenek'in içi belirlenemiyor.",
                        ad
                    ),
                    satir,
                    1,
                    1,
                )
                .onerili("En az bir dalda gerçek bir değer döndür.".into()));
            }
            Ok(Some(*tek))
        }
        [a, b] if *a == Tur::Yok || *b == Tur::Yok => {
            let dolu = if *a == Tur::Yok { *b } else { *a };
            match dolu.veri_turu() {
                Some(veri) => Ok(Some(Tur::Secenek(veri))),
                None => Err(Tani::yeni(
                    "T018",
                    format!("\"{}\" Seçenek içinde {} taşıyamaz (v0).", ad, dolu.adi()),
                    satir,
                    1,
                    1,
                )),
            }
        }
        _ => Err(Tani::yeni(
            "T018",
            format!("\"{}\" farklı türlerde değerler döndürüyor; tek tür seç.", ad),
            satir,
            1,
            1,
        )),
    }
}
