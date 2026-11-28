//! Bir web isteğinin rota seçimi ve transaction sınırı içindeki yürütmesi.

use super::istek_siniri::eksik_zorunlu_alan;
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
    let (ham, ham_govde) = match makine_istegi::ayir(program.program(), ham) {
        Ok(Some((zarf, govde))) => (std::borrow::Cow::Owned(zarf), Some(govde)),
        Ok(None) => (std::borrow::Cow::Borrowed(ham), None),
        Err((durum, mesaj)) => {
            io.durum_yaniti_gonder(durum, mesaj);
            return Ok(None);
        }
    };
    if let Err((durum, mesaj)) = istek_sinirlarini_denetle(&ham) {
        io.durum_yaniti_gonder(durum, mesaj);
        return Ok(None);
    }

    let (gelen_yontem, yol, veriler, cerezler) =
        match istek_parcala_cerezli(&ham).and_then(yontem_reddi::kayitli_yontem) {
            Ok(istek) => istek,
            Err((durum, mesaj)) => {
                io.durum_yaniti_gonder(durum, mesaj);
                return Ok(None);
            }
        };
    let (cerezler, oturum_bilgileri) = match oturum_cerezlerini_ayir(cerezler) {
        Ok(deger) => deger,
        Err((durum, mesaj)) => {
            io.durum_yaniti_gonder(durum, mesaj);
            return Ok(None);
        }
    };
    let istek_sozlugu = metin_sozlugu(veriler.iter().cloned());
    let cerez_sozlugu = metin_sozlugu(cerezler);
    let oturum_sozlugu = metin_sozlugu(oturum_bilgileri);
    let mut eslesti = false;
    let mut izinli = Vec::new();

    for cumle in &program.program().cumleler {
        let Cumle::IstekGeldiginde {
            yontem,
            yol: kayitli,
            onekli,
            govde,
            satir,
            ..
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
        let beklenen = yontem.unwrap_or(HttpYontemi::Get);
        if uydu {
            izinli.push(beklenen);
        }
        if !uydu || beklenen.yazimi() != gelen_yontem {
            continue;
        }

        let (erisim, kip) = match govde.first() {
            Some(Cumle::RotaPolitikasi { erisim, kip, .. }) => (erisim.clone(), *kip),
            _ => (RotaErisimi::HerkeseAcik, RotaKipi::Olagan),
        };
        let makine = kip == RotaKipi::Makine;
        let csrf = veriler
            .iter()
            .find(|(ad, _)| ad == "_csrf")
            .map(|(_, deger)| deger.as_str());
        if let Err(red) = io.rota_guvenligini_denetle(&erisim, csrf, !beklenen.guvenli() && !makine)
        {
            io.durum_yaniti_gonder(red.durum, red.mesaj);
            eslesti = true;
            break;
        }
        if let Some(ad) = eksik_zorunlu_alan(govde, &veriler) {
            io.durum_yaniti_gonder(400, &format!("zorunlu istek alanı eksik ya da boş: {}", ad));
            eslesti = true;
            break;
        }

        // K-335/K-405: önbelleklenebilir ve makine rotası oturum/çerez görmez;
        // T057/T061 statik kapısını aşan bir yol olsa bile yanıt kimliğe bağlanamaz.
        if let RotaKipi::Onbellekli(saniye) = kip {
            io.yanit_onbellegine_izin_ver(saniye);
        }
        let (cerez_sozlugu, oturum_sozlugu) = if kip != RotaKipi::Olagan {
            (Deger::Sozluk(Vec::new()), Deger::Sozluk(Vec::new()))
        } else {
            (cerez_sozlugu.clone(), oturum_sozlugu.clone())
        };
        makine_istegi::ortama_ekle(&mut rota_ortami, makine, &ham_govde, *satir)?;
        ortama_yazma_butcesini_tuket(&rota_ortami, "istek", &istek_sozlugu, *satir)?;
        rota_ortami.insert("istek".into(), istek_sozlugu.clone());
        ortama_yazma_butcesini_tuket(&rota_ortami, "çerezler", &cerez_sozlugu, *satir)?;
        rota_ortami.insert("çerezler".into(), cerez_sozlugu);
        ortama_yazma_butcesini_tuket(&rota_ortami, "oturum", &oturum_sozlugu, *satir)?;
        rota_ortami.insert("oturum".into(), oturum_sozlugu);
        let zaman_asimi = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .http()
            .calistirma_zaman_asimi_ms();
        let nobetci = SonTarihNobetcisi::yeni(io.an_ms().saturating_add(zaman_asimi));
        let sonuc = rota_sonu::kos(|| blok_calistir(govde, &mut rota_ortami, program, io, 0));
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
        yontem_reddi::eslesmeyen_istek(io, &izinli, &yol);
    }
    Ok(None)
}

/// İstek-yerel hata sınırı (spec/11 §5, K-324/ADR-157): rota tanısı yalnız o
/// isteği ayrıntı sızdırmayan sabit gövdeyle bitirir, worker sonraki isteği
/// alır. Geçici/kaynak sınıfı 503, diğer tanılar 500'dür. Ret kümesi yalnız
/// iç tutarlılıktır (C000, Ç nöbetçileri): `None` süreci tanıyla bitirir.
pub fn istek_hatasi_yaniti(kod: &str) -> Option<(u16, &'static str)> {
    if kod == "C000" || kod.starts_with('Ç') {
        return None;
    }
    let gecici = eylem_hatasi::web_mesaji(kod).or_else(|| {
        // Adım/çıktı (C023) ve heap (C024) zarfı istek başınadır (spec/24).
        matches!(kod, "C023" | "C024")
            .then_some("istek güvenli kaynak bütçesini aştı; otomatik tekrar yok")
    });
    Some(match gecici {
        Some(mesaj) => (503, mesaj),
        None => (500, "istek işlenemedi; sunucu hatası"),
    })
}

fn metin_sozlugu(ciftler: impl IntoIterator<Item = (String, String)>) -> Deger {
    Deger::Sozluk(
        ciftler
            .into_iter()
            .map(|(ad, deger)| (ad, Deger::Metin(deger)))
            .collect(),
    )
}
