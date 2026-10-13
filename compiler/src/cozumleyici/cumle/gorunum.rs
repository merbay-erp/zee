use super::*;

/// K-212: görünüm düğümlerinin ifadeleri Metin (metin/ham/öznitelik) ya da Mantıksal (öznitelik) olmalı.
pub(super) fn gorunum_dugumlerini_denetle(
    dugumler: &mut [crate::agac::GorunumDugumu],
    ortam: &SembolTablosu,
    baglam: &mut Baglam,
) -> Result<(), Tani> {
    use crate::agac::GorunumDugumu;
    for dugum in dugumler.iter_mut() {
        match dugum {
            GorunumDugumu::Oge {
                oznitelikler,
                cocuklar,
                satir,
                ..
            } => {
                for (ad, ifade) in oznitelikler.iter_mut() {
                    let tur = ifade_denetle(ifade, ortam, baglam, *satir)?;
                    if tur != Tur::Metin && tur != Tur::Mantiksal && tur != Tur::TamSayi {
                        return Err(Tani::yeni(
                            "T055",
                            format!(
                                "\"{}\" özniteliği Metin, TamSayı ya da Mantıksal ister; {} verildi.",
                                ad,
                                tur.adi()
                            ),
                            *satir,
                            1,
                            1,
                        )
                        .onerili("Değeri metne çevir ya da `doğru`/`yanlış` ver (bayrak öznitelik).".into()));
                    }
                }
                gorunum_dugumlerini_denetle(cocuklar, ortam, baglam)?;
            }
            GorunumDugumu::Metin { ifade, satir } | GorunumDugumu::Ham { ifade, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(ifade, ortam, baglam, satir)?;
                if tur != Tur::Metin {
                    return Err(Tani::yeni(
                        "T055",
                        format!("Görünüm metni Metin ister; {} verildi.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Sayıyı `\"\" ile sayi` biçiminde metne çevir.".into()));
                }
            }
        }
    }
    Ok(())
}

pub(super) fn hedefi_denetle(
    cumle: &mut Cumle,
    ortam: &mut SembolTablosu,
    baglam: &mut Baglam,
) -> Result<(), Tani> {
    let Cumle::Gorunum {
        ad,
        dugumler,
        satir,
        sutun,
        uzunluk,
    } = cumle
    else {
        return Err(ic_tutarlilik_hatasi(
            "Görünüm handler'ına başka cümle geldi",
            1,
        ));
    };
    let satir = *satir;
    if baglam.gezilen_koleksiyonlar.contains(ad) {
        return Err(gezilen_koleksiyonu_degistirme_tanisi(ad, satir));
    }
    gorunum_dugumlerini_denetle(dugumler, ortam, baglam)?;
    if let Some(eski) = ortam.get(ad.as_str()) {
        if *eski != Tur::Metin {
            return Err(Tani::yeni(
                "T002",
                format!(
                    "\"{}\" daha önce {} olarak tanımlandı; görünüm Metin verir.",
                    ad,
                    eski.adi()
                ),
                satir,
                *sutun,
                *uzunluk,
            ));
        }
    }
    let kimlik = ortam.insert(ad.clone(), Tur::Metin);
    let kaynak_araligi = crate::hir::HirKaynakAraligi::kesin(satir, *sutun, *uzunluk)
        .ok_or_else(|| hir_kaynak_hatasi(satir))?;
    sembol_yazimi_kaydet(baglam, kimlik, ad, kaynak_araligi);
    Ok(())
}
