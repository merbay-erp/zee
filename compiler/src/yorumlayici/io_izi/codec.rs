//! IO izi kanonik byte biçimi ve ortak sonuç codec'leri.

use super::*;

fn hex_yaz(metin: &str) -> String {
    const RAKAMLAR: &[u8; 16] = b"0123456789abcdef";
    let mut cikti = String::with_capacity(metin.len().saturating_mul(2));
    for bayt in metin.as_bytes() {
        cikti.push(RAKAMLAR[(bayt >> 4) as usize] as char);
        cikti.push(RAKAMLAR[(bayt & 0x0f) as usize] as char);
    }
    cikti
}

fn hex_rakam(bayt: u8) -> Option<u8> {
    match bayt {
        b'0'..=b'9' => Some(bayt - b'0'),
        b'a'..=b'f' => Some(bayt - b'a' + 10),
        _ => None,
    }
}

fn hex_coz(metin: &str) -> Result<String, String> {
    if !metin.len().is_multiple_of(2) {
        return Err("hex alan çift sayıda karakter taşımalı".into());
    }
    let mut baytlar = Vec::with_capacity(metin.len() / 2);
    for cift in metin.as_bytes().chunks_exact(2) {
        let yuksek = hex_rakam(cift[0]).ok_or_else(|| "hex alan küçük harfli değil".to_string())?;
        let dusuk = hex_rakam(cift[1]).ok_or_else(|| "hex alan küçük harfli değil".to_string())?;
        baytlar.push((yuksek << 4) | dusuk);
    }
    String::from_utf8(baytlar).map_err(|_| "hex alan geçerli UTF-8 değil".into())
}

pub(super) fn olaylari_yaz(olaylar: &[IzOlay]) -> Result<String, String> {
    if olaylar.len() > AZAMI_IO_IZ_OLAYI {
        return Err(format!("IO izi {} olay sınırını aşıyor", AZAMI_IO_IZ_OLAYI));
    }
    let mut cikti = String::from(BASLIK);
    for (indis, olay) in olaylar.iter().enumerate() {
        let beklenen = (indis as u64).saturating_add(1);
        if olay.sira != beklenen || !gecerli_islem(&olay.islem) {
            return Err(format!("IO izi {}. olay kimliği geçersiz", beklenen));
        }
        if olay.argumanlar.len().saturating_add(olay.sonuc.len()) > AZAMI_ALAN_SAYISI {
            return Err(format!("IO izi {}. olay çok fazla alan taşıyor", beklenen));
        }
        let mut alanlar = vec![
            olay.sira.to_string(),
            olay.islem.clone(),
            olay.argumanlar.len().to_string(),
            olay.sonuc.len().to_string(),
        ];
        alanlar.extend(olay.argumanlar.iter().map(|alan| hex_yaz(alan)));
        alanlar.extend(olay.sonuc.iter().map(|alan| hex_yaz(alan)));
        cikti.push_str(&alanlar.join("\t"));
        cikti.push('\n');
        if cikti.len() > AZAMI_IO_IZ_BAYTI {
            return Err(format!(
                "IO izi {} MiB sınırını aşıyor",
                AZAMI_IO_IZ_BAYTI / 1024 / 1024
            ));
        }
    }
    Ok(cikti)
}

fn kanonik_sayi(metin: &str, ad: &str) -> Result<usize, String> {
    let sayi = metin
        .parse::<usize>()
        .map_err(|_| format!("{ad} sayı olmalı"))?;
    if sayi.to_string() != metin {
        return Err(format!("{ad} kanonik onluk biçimde olmalı"));
    }
    Ok(sayi)
}

pub(super) fn olaylari_oku(metin: &str) -> Result<VecDeque<IzOlay>, String> {
    if metin.len() > AZAMI_IO_IZ_BAYTI {
        return Err(format!(
            "IO izi {} MiB sınırını aşıyor",
            AZAMI_IO_IZ_BAYTI / 1024 / 1024
        ));
    }
    let govde = metin
        .strip_prefix(BASLIK)
        .ok_or_else(|| "IO izi başlığı `zee-io-izi\\t1` olmalı".to_string())?;
    if !metin.ends_with('\n') {
        return Err("IO izi son satırı yeni satırla bitmeli".into());
    }
    let mut olaylar = Vec::new();
    for (indis, satir) in govde.lines().enumerate() {
        if satir.is_empty() {
            return Err(format!("IO izi {}. olay satırı boş", indis + 1));
        }
        if olaylar.len() >= AZAMI_IO_IZ_OLAYI {
            return Err(format!("IO izi {} olay sınırını aşıyor", AZAMI_IO_IZ_OLAYI));
        }
        let hucreler = satir.split('\t').collect::<Vec<_>>();
        if hucreler.len() < 4 {
            return Err(format!("IO izi {}. olay başlığı eksik", indis + 1));
        }
        let sira_usize = kanonik_sayi(hucreler[0], "olay sırası")?;
        let beklenen = indis.saturating_add(1);
        if sira_usize != beklenen {
            return Err(format!(
                "IO izi olay sırası {} olmalı, {} bulundu",
                beklenen, sira_usize
            ));
        }
        let islem = hucreler[1];
        if !gecerli_islem(islem) {
            return Err(format!(
                "IO izi {}. olay işlemi bilinmiyor: {}",
                beklenen, islem
            ));
        }
        let arguman_sayisi = kanonik_sayi(hucreler[2], "argüman sayısı")?;
        let sonuc_sayisi = kanonik_sayi(hucreler[3], "sonuç sayısı")?;
        let alan_sayisi = arguman_sayisi.saturating_add(sonuc_sayisi);
        if alan_sayisi > AZAMI_ALAN_SAYISI || hucreler.len() != 4 + alan_sayisi {
            return Err(format!("IO izi {}. olay alan sayısı uyuşmuyor", beklenen));
        }
        let alanlar = hucreler[4..]
            .iter()
            .map(|alan| hex_coz(alan))
            .collect::<Result<Vec<_>, _>>()?;
        let (argumanlar, sonuc) = alanlar.split_at(arguman_sayisi);
        let olay = IzOlay {
            sira: sira_usize as u64,
            islem: islem.to_string(),
            argumanlar: argumanlar.to_vec(),
            sonuc: sonuc.to_vec(),
        };
        olay_semasini_denetle(&olay)
            .map_err(|hata| format!("IO izi {}. olay şeması bozuk: {}", beklenen, hata))?;
        olaylar.push(olay);
    }
    let yeniden = olaylari_yaz(&olaylar)?;
    if yeniden != metin {
        return Err("IO izi kanonik byte biçiminde değil".into());
    }
    Ok(olaylar.into())
}

pub(super) fn bool_yaz(deger: bool) -> String {
    if deger { "1" } else { "0" }.into()
}

pub(super) fn bool_coz(metin: &str) -> Result<bool, String> {
    match metin {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(format!("boolean alan 0/1 olmalı, `{metin}` bulundu")),
    }
}

pub(super) fn secenek_yaz(deger: Option<&str>) -> Vec<String> {
    match deger {
        Some(deger) => vec!["var".into(), deger.into()],
        None => vec!["yok".into()],
    }
}

pub(super) fn secenek_coz(alanlar: &[String]) -> Result<Option<String>, String> {
    match alanlar {
        [etiket] if etiket == "yok" => Ok(None),
        [etiket, deger] if etiket == "var" => Ok(Some(deger.clone())),
        _ => Err("seçenek sonucu `yok` veya `var,<değer>` olmalı".into()),
    }
}

pub(super) fn birim_sonuc_yaz(sonuc: &Result<(), String>) -> Vec<String> {
    match sonuc {
        Ok(()) => vec!["ok".into()],
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

pub(super) fn birim_sonuc_coz(alanlar: &[String]) -> Result<Result<(), String>, String> {
    match alanlar {
        [etiket] if etiket == "ok" => Ok(Ok(())),
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        _ => Err("birim sonuç `ok` veya `hata,<mesaj>` olmalı".into()),
    }
}

pub(super) fn metin_sonuc_yaz(sonuc: &Result<String, String>) -> Vec<String> {
    match sonuc {
        Ok(deger) => vec!["ok".into(), deger.clone()],
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

pub(super) fn metin_sonuc_coz(alanlar: &[String]) -> Result<Result<String, String>, String> {
    match alanlar {
        [etiket, deger] if etiket == "ok" => Ok(Ok(deger.clone())),
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        _ => Err("metin sonuç `ok,<değer>` veya `hata,<mesaj>` olmalı".into()),
    }
}

pub(super) fn tamsayi_sonuc_yaz(sonuc: &Result<i64, String>) -> Vec<String> {
    match sonuc {
        Ok(deger) => vec!["ok".into(), deger.to_string()],
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

pub(super) fn tamsayi_sonuc_coz(alanlar: &[String]) -> Result<Result<i64, String>, String> {
    match alanlar {
        [etiket, deger] if etiket == "ok" => deger
            .parse::<i64>()
            .map(Ok)
            .map_err(|_| "tam sayı sonucu kanonik değil".into()),
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        _ => Err("tam sayı sonuç `ok,<değer>` veya `hata,<mesaj>` olmalı".into()),
    }
}

pub(super) fn metin_listesi_sonuc_yaz(sonuc: &Result<Vec<String>, String>) -> Vec<String> {
    match sonuc {
        Ok(degerler) => {
            let mut alanlar = vec!["ok".into()];
            alanlar.extend(degerler.iter().cloned());
            alanlar
        }
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

pub(super) fn metin_listesi_sonuc_coz(
    alanlar: &[String],
) -> Result<Result<Vec<String>, String>, String> {
    match alanlar {
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        [etiket, degerler @ ..] if etiket == "ok" => Ok(Ok(degerler.to_vec())),
        _ => Err("metin listesi sonucu `ok,...` veya `hata,<mesaj>` olmalı".into()),
    }
}
