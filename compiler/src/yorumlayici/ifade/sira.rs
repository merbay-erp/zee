//! K-386 (RFC-0029): sıra ile öğe okuma. Liste adı ortamdan ödünç alınır,
//! yalnız seçilen öğe kopyalanır (O(1); spec/17 değer semantiği).

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) async fn oge(
    liste: &Ifade,
    sira: &Ifade,
    dene: bool,
    ortam: &HashMap<String, Deger>,
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
    derinlik: usize,
    satir: usize,
) -> Result<Deger, Tani> {
    let sira_konumu = sira.kaynak_araligi().map(|a| a.uclu());
    let sira = match degerlendir_async(sira, ortam, program, io, derinlik, satir).await? {
        Deger::TamSayi(s) => s,
        _ => return Err(ic_hata(satir)),
    };
    // O(1): ad ise liste ortamdan ödünç alınır; yalnız seçilen öğe
    // kopyalanır (spec/17 değer semantiği, RFC-0029).
    let secim = if let Ifade::Degisken { cozulmus, ham, .. } = liste.turu() {
        let ad = program
            .sembol_adi(liste, cozulmus.as_deref())
            .ok_or_else(|| ic_hata(satir))?;
        match ortam.get(&ad) {
            Some(Deger::Liste(ogeler)) => liste_ogesi_sec(ogeler, sira),
            Some(_) => return Err(ic_hata(satir)),
            None => {
                return Err(Tani::yeni(
                    "C001",
                    format!("\"{}\" için değer bulunamadı.", ham),
                    satir,
                    1,
                    1,
                ))
            }
        }
    } else {
        match degerlendir_async(liste, ortam, program, io, derinlik, satir).await? {
            Deger::Liste(ogeler) => liste_ogesi_sec(&ogeler, sira),
            _ => return Err(ic_hata(satir)),
        }
    };
    match (secim, dene) {
        (Ok(oge), false) => Ok(oge),
        (Ok(oge), true) => Ok(Deger::Sonuc {
            basarili: true,
            icerik: Box::new(oge),
        }),
        (Err(adet), true) => Ok(Deger::Sonuc {
            basarili: false,
            icerik: Box::new(Deger::hata(
                "SIRA_DISI",
                format!("listenin {}. öğesi yok ({} öğe)", sira, adet),
            )),
        }),
        (Err(adet), false) => {
            let (s_satir, s_sutun, s_uzunluk) = sira_konumu.unwrap_or((satir, 1, 1));
            Err(Tani::yeni(
                "C031",
                format!("Listenin {}. öğesi yok: liste {} öğeli.", sira, adet),
                s_satir,
                s_sutun,
                s_uzunluk,
            )
            .onerili(
                "Sıra 1 ile listenin adedi arasında olmalı; olmayabilecek öğe için \"… öğesini almayı dene\" kullan."
                    .into(),
            ))
        }
    }
}

/// RFC-0029: 1 tabanlı sıra ile öğe kopyası; aralık dışında listenin adedi döner.
fn liste_ogesi_sec(ogeler: &[Deger], sira: i64) -> Result<Deger, usize> {
    usize::try_from(sira)
        .ok()
        .and_then(|s| s.checked_sub(1))
        .and_then(|i| ogeler.get(i))
        .cloned()
        .ok_or(ogeler.len())
}
