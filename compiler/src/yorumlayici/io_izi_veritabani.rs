//! PostgreSQL gözlem sonuçlarının sürümlü IO izi alan kodlaması.

use crate::veritabani_modeli::{VeritabaniHatasi, VeritabaniOkumaSonucu, VeritabaniSatirlari};

fn kanonik_sayi(metin: &str, ad: &str) -> Result<usize, String> {
    let sayi = metin
        .parse::<usize>()
        .map_err(|_| format!("{ad} sayı olmalı"))?;
    if sayi.to_string() != metin {
        return Err(format!("{ad} kanonik onluk biçimde olmalı"));
    }
    Ok(sayi)
}

fn hata_yaz(hata: &VeritabaniHatasi) -> Vec<String> {
    let mut alanlar = vec![
        "hata".into(),
        hata.mesaj.clone(),
        hata.veri.len().to_string(),
    ];
    for (ad, deger) in &hata.veri {
        alanlar.extend([ad.clone(), deger.clone()]);
    }
    alanlar
}

fn hata_coz(alanlar: &[String]) -> Result<VeritabaniHatasi, String> {
    let [etiket, mesaj, adet, kalan @ ..] = alanlar else {
        return Err("veritabanı hatası alanları eksik".into());
    };
    if etiket != "hata" {
        return Err("veritabanı hata etiketi `hata` olmalı".into());
    }
    let adet = kanonik_sayi(adet, "veritabanı hata veri sayısı")?;
    if kalan.len() != adet.saturating_mul(2) {
        return Err("veritabanı hata veri sayısı uyuşmuyor".into());
    }
    Ok(VeritabaniHatasi {
        mesaj: mesaj.clone(),
        veri: kalan
            .chunks_exact(2)
            .map(|cift| (cift[0].clone(), cift[1].clone()))
            .collect(),
    })
}

pub(super) fn okuma_yaz(sonuc: &VeritabaniOkumaSonucu) -> Vec<String> {
    match sonuc {
        Err(hata) => hata_yaz(hata),
        Ok(satirlar) => {
            let mut alanlar = vec!["ok".into(), satirlar.len().to_string()];
            for satir in satirlar {
                alanlar.push(satir.len().to_string());
                for (ad, deger) in satir {
                    alanlar.extend([ad.clone(), deger.clone()]);
                }
            }
            alanlar
        }
    }
}

pub(super) fn okuma_coz(alanlar: &[String]) -> Result<VeritabaniOkumaSonucu, String> {
    if alanlar.first().is_some_and(|etiket| etiket == "hata") {
        return Ok(Err(hata_coz(alanlar)?));
    }
    let [etiket, satir_sayisi, kalan @ ..] = alanlar else {
        return Err("veritabanı okuma sonucu eksik".into());
    };
    if etiket != "ok" {
        return Err("veritabanı okuma etiketi `ok`/`hata` olmalı".into());
    }
    let satir_sayisi = kanonik_sayi(satir_sayisi, "veritabanı satır sayısı")?;
    let mut kalan = kalan;
    let mut satirlar = VeritabaniSatirlari::with_capacity(satir_sayisi);
    for _ in 0..satir_sayisi {
        let (sutun_sayisi, govde) = kalan
            .split_first()
            .ok_or_else(|| "veritabanı satır sütun sayısı eksik".to_string())?;
        let alan_sayisi = kanonik_sayi(sutun_sayisi, "veritabanı sütun sayısı")?.saturating_mul(2);
        if govde.len() < alan_sayisi {
            return Err("veritabanı satır alanları eksik".into());
        }
        let (satir, devam) = govde.split_at(alan_sayisi);
        satirlar.push(
            satir
                .chunks_exact(2)
                .map(|cift| (cift[0].clone(), cift[1].clone()))
                .collect(),
        );
        kalan = devam;
    }
    if !kalan.is_empty() {
        return Err("veritabanı okuma sonucu fazla alan taşıyor".into());
    }
    Ok(Ok(satirlar))
}

pub(super) fn degistirme_yaz(sonuc: &Result<i64, VeritabaniHatasi>) -> Vec<String> {
    match sonuc {
        Ok(adet) => vec!["ok".into(), adet.to_string()],
        Err(hata) => hata_yaz(hata),
    }
}

pub(super) fn degistirme_coz(alanlar: &[String]) -> Result<Result<i64, VeritabaniHatasi>, String> {
    match alanlar {
        [etiket, adet]
            if etiket == "ok"
                && adet
                    .parse::<i64>()
                    .ok()
                    .is_some_and(|s| s.to_string() == *adet) =>
        {
            Ok(Ok(adet.parse().map_err(|_| {
                "etkilenen satır sayısı i64 değil".to_string()
            })?))
        }
        [etiket, ..] if etiket == "hata" => Ok(Err(hata_coz(alanlar)?)),
        _ => Err("veritabanı değişiklik sonucu `ok,<i64>`/`hata,...` olmalı".into()),
    }
}
