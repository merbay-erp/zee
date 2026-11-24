//! K-386 (RFC-0029): sıra ile öğe erişiminin tür denetimi (T058, T059).

use super::*;

/// RFC-0029 (K-386): `<liste>nin <sıra>. öğesi` öğe türü. Liste öğe türü
/// belli olmalı, sıra TamSayı olmalı; sabit sıra 1'den küçükse derleme
/// zamanında reddedilir (çalışma zamanı sınırı C031 / `SIRA_DISI`).
pub(super) fn liste_ogesi_turu(
    liste: &mut Ifade,
    sira: &mut Ifade,
    ortam: &SembolTablosu,
    baglam: &mut Baglam,
    satir: usize,
) -> Result<VeriTuru, Tani> {
    let konum = |ifade: &Ifade| {
        ifade
            .kaynak_araligi()
            .map(|aralik| aralik.uclu())
            .unwrap_or((satir, 1, 1))
    };
    let (l_satir, l_sutun, l_uzunluk) = konum(liste);
    let oge = match ifade_denetle(liste, ortam, baglam, satir)? {
        Tur::Liste(VeriTuru::Bilinmeyen) => {
            return Err(Tani::yeni(
                "T058",
                "Bu liste henüz boş: sıradaki öğenin türü belli değil.".into(),
                l_satir,
                l_sutun,
                l_uzunluk,
            )
            .onerili(
                "Önce listeye öğe ekle; boş kalabilecek listede \"… öğesini almayı dene\" kullan."
                    .into(),
            ));
        }
        Tur::Liste(oge) => oge,
        baska => {
            return Err(Tani::yeni(
                "T058",
                format!(
                    "Sıra ile öğe erişimi bir liste ister; burada {} var.",
                    baska.adi()
                ),
                l_satir,
                l_sutun,
                l_uzunluk,
            )
            .onerili(
                "Metnin harfleri için önce \"metnin harfleri\" listesini al; sözlükte \"sözlüğün anahtar değeri\" ile oku."
                    .into(),
            ));
        }
    };
    let (s_satir, s_sutun, s_uzunluk) = konum(sira);
    let sira_turu = ifade_denetle(sira, ortam, baglam, satir)?;
    if sira_turu != Tur::TamSayi {
        return Err(Tani::yeni(
            "T058",
            format!("Sıra TamSayı olmalı; burada {} var.", sira_turu.adi()),
            s_satir,
            s_sutun,
            s_uzunluk,
        )
        .onerili(
            "Sıra 1, 2, 3 gibi bir tam sayıdır; metinden geliyorsa önce \"metnin sayısı\" ile çevir."
                .into(),
        ));
    }
    if let Ifade::SayiSabiti(deger) = sira.turu() {
        if *deger < 1 {
            return Err(Tani::yeni(
                "T059",
                format!("Sıra 1'den başlar; {}. öğe yoktur.", deger),
                s_satir,
                s_sutun,
                s_uzunluk,
            )
            .onerili(
                "İlk öğe \"listenin 1. öğesi\" (ya da \"listenin ilki\"), son öğe \"listenin sonu\"dur."
                    .into(),
            ));
        }
    }
    Ok(oge)
}
