//! Bütçeli değer metni, JSON ve CSV üretimi.

use super::*;

pub(super) fn metne_sinirli(deger: &Deger, satir: usize) -> Result<String, Tani> {
    let mut cikti = String::new();
    deger_metnini_ekle(&mut cikti, deger, satir)?;
    Ok(cikti)
}

pub(super) fn dogrulama_detayi(sol: &Deger, sag: &Deger, satir: usize) -> Result<String, Tani> {
    let mut cikti = String::new();
    ekle(&mut cikti, " Beklenen: ", satir)?;
    deger_metnini_ekle(&mut cikti, sag, satir)?;
    ekle(&mut cikti, " — bulunan: ", satir)?;
    deger_metnini_ekle(&mut cikti, sol, satir)?;
    ekle(&mut cikti, ".", satir)?;
    Ok(cikti)
}

fn deger_metnini_ekle(cikti: &mut String, deger: &Deger, satir: usize) -> Result<(), Tani> {
    match deger {
        Deger::TamSayi(s) => ekle(cikti, &s.to_string(), satir),
        Deger::Ondalik(ondalik) => {
            metin_sinirini_denetle(ondalik.metin_bayti_ust_siniri(), satir)?;
            ekle(cikti, &ondalik.metne(), satir)
        }
        Deger::Metin(m) => ekle(cikti, m, satir),
        Deger::Mantiksal(b) => ekle(cikti, if *b { "doğru" } else { "yanlış" }, satir),
        Deger::Liste(ogeler) => {
            for (sira, oge) in ogeler.iter().enumerate() {
                if sira > 0 {
                    ekle(cikti, ", ", satir)?;
                }
                deger_metnini_ekle(cikti, oge, satir)?;
            }
            Ok(())
        }
        Deger::Sozluk(girdiler) | Deger::Yapi(girdiler) => {
            for (sira, (anahtar, deger)) in girdiler.iter().enumerate() {
                if sira > 0 {
                    ekle(cikti, ", ", satir)?;
                }
                ekle(cikti, anahtar, satir)?;
                ekle(cikti, ": ", satir)?;
                deger_metnini_ekle(cikti, deger, satir)?;
            }
            Ok(())
        }
        Deger::Yok => ekle(cikti, "yok", satir),
        Deger::Sonuc { basarili, icerik } => {
            if !basarili {
                ekle(cikti, "hata: ", satir)?;
            }
            deger_metnini_ekle(cikti, icerik, satir)
        }
        Deger::Hata(hata) => ekle(cikti, &hata.mesaj, satir),
        Deger::Tarih { yil, ay, gun } => ekle(
            cikti,
            &format!(
                "{} {} {}",
                gun,
                AY_ADLARI[(*ay as usize).saturating_sub(1) % 12],
                yil
            ),
            satir,
        ),
        Deger::Saat { saat, dakika } => ekle(cikti, &format!("{:02}:{:02}", saat, dakika), satir),
        Deger::Sure { milisaniye } => {
            let ms = *milisaniye;
            let metin = if ms % 3_600_000 == 0 {
                format!("{} saat", ms / 3_600_000)
            } else if ms % 60_000 == 0 {
                format!("{} dakika", ms / 60_000)
            } else if ms % 1000 == 0 {
                format!("{} saniye", ms / 1000)
            } else {
                let ondalik = Ondalik::katsayidan(ms, 3);
                format!("{} saniye", ondalik.metne())
            };
            ekle(cikti, &metin, satir)
        }
        Deger::AgYaniti { durum, govde } => {
            ekle(cikti, &format!("[{}] ", durum), satir)?;
            ekle(cikti, govde, satir)
        }
    }
}

/// Satır sözlükleri listesini CSV metnine çevirir (K-058).
pub(super) fn csv_yaz(satirlar: &[Deger], satir_no: usize) -> Result<String, Tani> {
    let mut cikti = String::new();
    let Some(Deger::Sozluk(ilk)) = satirlar.first() else {
        return Ok(cikti);
    };
    let basliklar: Vec<&String> = ilk.iter().map(|(a, _)| a).collect();
    for (sira, baslik) in basliklar.iter().enumerate() {
        if sira > 0 {
            ekle(&mut cikti, ",", satir_no)?;
        }
        csv_hucresi_ekle(&mut cikti, baslik, satir_no)?;
    }
    ekle(&mut cikti, "\n", satir_no)?;
    for satir in satirlar {
        if let Deger::Sozluk(girdiler) = satir {
            for (sira, baslik) in basliklar.iter().enumerate() {
                if sira > 0 {
                    ekle(&mut cikti, ",", satir_no)?;
                }
                let metin = match girdiler.iter().find(|(ad, _)| ad == *baslik) {
                    Some((_, deger)) => metne_sinirli(deger, satir_no)?,
                    None => String::new(),
                };
                csv_hucresi_ekle(&mut cikti, &metin, satir_no)?;
            }
            ekle(&mut cikti, "\n", satir_no)?;
        }
    }
    Ok(cikti)
}

fn csv_hucresi_ekle(cikti: &mut String, metin: &str, satir: usize) -> Result<(), Tani> {
    if !metin.contains([',', '"', '\n']) {
        return ekle(cikti, metin, satir);
    }
    ekle(cikti, "\"", satir)?;
    for parca in metin.split_inclusive('"') {
        ekle(cikti, parca, satir)?;
        if parca.ends_with('"') {
            ekle(cikti, "\"", satir)?;
        }
    }
    ekle(cikti, "\"", satir)
}

/// Değeri JSON metnine serileştirir; sözlük anahtar sırası korunur.
pub(super) fn json_yaz(deger: &Deger, satir: usize) -> Result<String, Tani> {
    let mut cikti = String::new();
    json_degerini_ekle(&mut cikti, deger, satir)?;
    Ok(cikti)
}

fn json_degerini_ekle(cikti: &mut String, deger: &Deger, satir: usize) -> Result<(), Tani> {
    match deger {
        Deger::TamSayi(s) => ekle(cikti, &s.to_string(), satir),
        Deger::Ondalik(ondalik) => {
            metin_sinirini_denetle(ondalik.metin_bayti_ust_siniri(), satir)?;
            ekle(cikti, &ondalik.json_metni(), satir)
        }
        Deger::Mantiksal(b) => ekle(cikti, if *b { "true" } else { "false" }, satir),
        Deger::Metin(metin) => json_metin_ekle(cikti, metin, satir),
        Deger::Liste(ogeler) => {
            ekle(cikti, "[", satir)?;
            for (sira, oge) in ogeler.iter().enumerate() {
                if sira > 0 {
                    ekle(cikti, ",", satir)?;
                }
                json_degerini_ekle(cikti, oge, satir)?;
            }
            ekle(cikti, "]", satir)
        }
        Deger::Yapi(alanlar) | Deger::Sozluk(alanlar) => {
            json_alanlarini_ekle(cikti, alanlar, satir)
        }
        Deger::Hata(hata) => json_hatayi_ekle(cikti, hata, satir),
        baska => json_metin_ekle(cikti, &metne_sinirli(baska, satir)?, satir),
    }
}

fn json_alanlarini_ekle(
    cikti: &mut String,
    alanlar: &[(String, Deger)],
    satir: usize,
) -> Result<(), Tani> {
    ekle(cikti, "{", satir)?;
    for (sira, (ad, deger)) in alanlar.iter().enumerate() {
        if sira > 0 {
            ekle(cikti, ",", satir)?;
        }
        json_metin_ekle(cikti, ad, satir)?;
        ekle(cikti, ":", satir)?;
        json_degerini_ekle(cikti, deger, satir)?;
    }
    ekle(cikti, "}", satir)
}

fn json_hatayi_ekle(cikti: &mut String, hata: &HataDegeri, satir: usize) -> Result<(), Tani> {
    ekle(cikti, "{\"kod\":", satir)?;
    json_metin_ekle(cikti, &hata.kod, satir)?;
    ekle(cikti, ",\"mesaj\":", satir)?;
    json_metin_ekle(cikti, &hata.mesaj, satir)?;
    ekle(cikti, ",\"neden\":", satir)?;
    match hata.neden.as_deref() {
        Some(neden) => json_hatayi_ekle(cikti, neden, satir)?,
        None => ekle(cikti, "null", satir)?,
    }
    ekle(cikti, ",\"veri\":", satir)?;
    json_alanlarini_ekle(cikti, &hata.veri, satir)?;
    ekle(cikti, "}", satir)
}

fn json_metin_ekle(cikti: &mut String, metin: &str, satir: usize) -> Result<(), Tani> {
    ekle(cikti, "\"", satir)?;
    for karakter in metin.chars() {
        match karakter {
            '"' => ekle(cikti, "\\\"", satir)?,
            '\\' => ekle(cikti, "\\\\", satir)?,
            '\n' => ekle(cikti, "\\n", satir)?,
            '\r' => ekle(cikti, "\\r", satir)?,
            '\t' => ekle(cikti, "\\t", satir)?,
            k if (k as u32) < 0x20 => ekle(cikti, &format!("\\u{:04x}", k as u32), satir)?,
            k => {
                let mut tampon = [0u8; 4];
                ekle(cikti, k.encode_utf8(&mut tampon), satir)?;
            }
        }
    }
    ekle(cikti, "\"", satir)
}

fn ekle(cikti: &mut String, parca: &str, satir: usize) -> Result<(), Tani> {
    metin_parcasi_ekle(cikti, parca, satir)
}
