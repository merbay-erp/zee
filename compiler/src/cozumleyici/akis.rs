use super::*;

/// Koşuldan daraltma çıkarır: 1=dolu Seçenek, 2=boş, 3=başarılı, 4=başarısız.
pub(super) fn daraltma_cikar(kosul: &Ifade) -> Option<(u8, String)> {
    match kosul {
        Ifade::SecenekVar { nesne, olumsuz } => {
            nesne_adi(nesne).map(|ad| (if *olumsuz { 2 } else { 1 }, ad))
        }
        Ifade::SonucBasarili { nesne, olumsuz } => {
            nesne_adi(nesne).map(|ad| (if *olumsuz { 4 } else { 3 }, ad))
        }
        _ => None,
    }
}


pub(super) fn nesne_adi(nesne: &Ifade) -> Option<String> {
    match nesne {
        Ifade::Degisken { cozulmus: Some(ad), .. } => Some(ad.clone()),
        _ => None,
    }
}

pub(super) fn gezilen_koleksiyonu_degistirme_tanisi(ad: &str, satir: usize) -> Tani {
    Tani::yeni(
        "T053",
        format!(
            "\"{}\" gezilirken koleksiyonun kendisi değiştirilemez.",
            ad
        ),
        satir,
        1,
        1,
    )
    .onerili(
        "Öğeyi döngü adıyla güncelle; ekleme/silme gerekiyorsa değişiklikleri ayrı bir listede topla ve gezme bitince uygula."
            .into(),
    )
}

pub(super) fn gezilen_hedefi_denetle(ifade: &Ifade, baglam: &Baglam, satir: usize) -> Result<(), Tani> {
    let aday = match ifade {
        Ifade::Degisken {
            cozulmus: Some(ad), ..
        } => baglam.gezilen_koleksiyonlar.get(ad).cloned(),
        Ifade::Degisken { ham, .. } => {
            let kokler = crate::morfoloji::kok_adaylari(ham);
            let mut eslesenler = baglam
                .gezilen_koleksiyonlar
                .iter()
                .filter(|ad| **ad == *ham || kokler.iter().any(|kok| kok == *ad))
                .cloned()
                .collect::<Vec<_>>();
            eslesenler.sort();
            eslesenler.into_iter().next()
        }
        _ => None,
    };
    if let Some(ad) = aday {
        return Err(gezilen_koleksiyonu_degistirme_tanisi(&ad, satir));
    }
    Ok(())
}

pub(super) fn daraltma_ekle(baglam: &mut Baglam, tur_kodu: u8, ad: &str) {
    match tur_kodu {
        1 => {
            baglam.dolu_secenekler.insert(ad.to_string());
        }
        3 => {
            baglam.basarili_sonuclar.insert(ad.to_string());
        }
        4 => {
            baglam.basarisiz_sonuclar.insert(ad.to_string());
        }
        _ => {}
    }
}

pub(super) fn daraltma_cikar_geri(baglam: &mut Baglam, tur_kodu: u8, ad: &str) {
    match tur_kodu {
        1 => {
            baglam.dolu_secenekler.remove(ad);
        }
        3 => {
            baglam.basarili_sonuclar.remove(ad);
        }
        4 => {
            baglam.basarisiz_sonuclar.remove(ad);
        }
        _ => {}
    }
}


pub(super) fn bekleyen_gorev_olmadigini_denetle(
    baglam: &Baglam,
    giriste_bekleyenler: &std::collections::HashSet<String>,
    satir: usize,
    mesaj: &str,
) -> Result<(), Tani> {
    if baglam
        .bekleyen_gorevler
        .difference(giriste_bekleyenler)
        .next()
        .is_some()
    {
        Err(gorev_kapsami_tanisi(satir, mesaj))
    } else {
        Ok(())
    }
}

pub(super) fn gorev_kapsami_tanisi(satir: usize, mesaj: &str) -> Tani {
    Tani::yeni("T051", mesaj.into(), satir, 1, 1).onerili(
        "Her `eşzamanlı olarak` grubunu aynı sözcüksel kapsamda tek bir `hepsini bekle` ile kapat."
            .into(),
    )
}
