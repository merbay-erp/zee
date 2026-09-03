//! Bir web isteğinin rota seçimi ve transaction sınırı içindeki yürütmesi.

use super::*;

/// Bir isteği taze kaynak zarfında çalıştırır. `Some`, rota içindeki
/// `programı bitir` çıkış kodudur. İstek transaction'ını çağıran tamamlar veya
/// geri alır; böylece bu fonksiyondaki her `?` aynı güvenli sona ulaşır.
pub(super) fn web_istegini_calistir(
    program: CalistirmaProgrami<'_>,
    ortam: &HashMap<String, Deger>,
    io: &mut dyn GirdiCikti,
    ham: &str,
) -> Result<Option<i64>, Tani> {
    calistirma_butcesini_yenile();
    gorev_ortami_butcesini_tuket(ortam, 0, 1)?;
    if let Err((durum, mesaj)) = istek_sinirlarini_denetle(ham) {
        io.durum_yaniti_gonder(durum, mesaj);
        return Ok(None);
    }

    let (gelen_yontem, yol, veriler, cerezler) = match istek_parcala_cerezli(ham) {
        Ok(istek) => istek,
        Err((durum, mesaj)) => {
            io.durum_yaniti_gonder(durum, mesaj);
            return Ok(None);
        }
    };
    let istek_sozlugu = Deger::Sozluk(
        veriler
            .iter()
            .cloned()
            .map(|(ad, deger)| (ad, Deger::Metin(deger)))
            .collect(),
    );
    let cerez_sozlugu = Deger::Sozluk(
        cerezler
            .into_iter()
            .map(|(ad, deger)| (ad, Deger::Metin(deger)))
            .collect(),
    );
    let mut eslesti = false;
    let mut yol_eslesti = false;

    for cumle in &program.program().cumleler {
        let Cumle::IstekGeldiginde {
            yontem,
            yol: kayitli,
            onekli,
            govde,
            satir,
        } = cumle
        else {
            continue;
        };
        let mut rota_ortami = HashMap::new();
        let kayitli_degeri = degerlendir(kayitli, &rota_ortami, program, io, 0, *satir)?;
        let kayitli = metne_sinirli(&kayitli_degeri, *satir)?;
        let uydu = if *onekli {
            yol.starts_with(&kayitli)
        } else {
            kayitli == yol
        };
        yol_eslesti |= uydu;
        let beklenen = yontem.unwrap_or(HttpYontemi::Get);
        if !uydu || beklenen.yazimi() != gelen_yontem {
            continue;
        }

        let erisim = match govde.first() {
            Some(Cumle::RotaPolitikasi { erisim, .. }) => erisim.clone(),
            _ => RotaErisimi::HerkeseAcik,
        };
        let csrf = veriler
            .iter()
            .find(|(ad, _)| ad == "_csrf")
            .map(|(_, deger)| deger.as_str());
        if let Err(red) = io.rota_guvenligini_denetle(&erisim, csrf, !beklenen.guvenli()) {
            io.durum_yaniti_gonder(red.durum, red.mesaj);
            eslesti = true;
            break;
        }
        if let Some(ad) = eksik_zorunlu_alan(govde, &veriler) {
            io.durum_yaniti_gonder(400, &format!("zorunlu istek alanı eksik ya da boş: {}", ad));
            eslesti = true;
            break;
        }

        ortama_yazma_butcesini_tuket(&rota_ortami, "istek", &istek_sozlugu, *satir)?;
        rota_ortami.insert("istek".into(), istek_sozlugu.clone());
        ortama_yazma_butcesini_tuket(&rota_ortami, "çerezler", &cerez_sozlugu, *satir)?;
        rota_ortami.insert("çerezler".into(), cerez_sozlugu.clone());
        let zaman_asimi = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .http()
            .calistirma_zaman_asimi_ms();
        let nobetci = SonTarihNobetcisi::yeni(io.an_ms().saturating_add(zaman_asimi));
        let sonuc = blok_calistir(govde, &mut rota_ortami, program, io, 0);
        drop(nobetci);
        match sonuc {
            Err(tani) if tani.kod == "Ç000" => {
                return Ok(Some(tani.mesaj.parse::<i64>().unwrap_or(0)));
            }
            Err(tani) => return Err(tani),
            Ok(_) => {}
        }
        eslesti = true;
        break;
    }

    if !eslesti {
        if yol_eslesti {
            io.durum_yaniti_gonder(405, "bu adres istenen HTTP yöntemini kabul etmiyor");
        } else {
            io.durum_yaniti_gonder(404, &format!("aranan sayfa yok: {}", yol));
        }
    }
    Ok(None)
}

fn eksik_zorunlu_alan<'a>(govde: &'a [Cumle], veriler: &[(String, String)]) -> Option<&'a str> {
    govde.iter().find_map(|cumle| match cumle {
        Cumle::RotaAlaniGerekli { ad, .. }
            if !veriler
                .iter()
                .any(|(gelen, deger)| gelen == ad && !deger.trim().is_empty()) =>
        {
            Some(ad.as_str())
        }
        _ => None,
    })
}
