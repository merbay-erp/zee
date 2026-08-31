//! Türkçe programlama dili — bootstrap derleyici (Stage 0).
//!
//! Boru hattı (master plan bölüm 11'in v0 dilimi):
//! kaynak → sözcükleyici → ayrıştırıcı → ad çözümleme + tür denetimi → yorumlayıcı.

pub mod agac;
pub mod ayristirici;
pub mod cozumleyici;
pub mod sozcukleyici;
pub mod tani;
pub mod yorumlayici;

use agac::{Cumle, Program};
use tani::Tani;

/// Kaynağı çalıştırılabilir programa derler (sözcükle + ayrıştır + denetle).
/// İşlem tanımları Program.islemler'e kaldırılır (hoist).
pub fn kaynagi_derle(kaynak: &str) -> Result<Program, Tani> {
    let tokenlar = sozcukleyici::sozcukle(kaynak)?;
    let cumleler = ayristirici::ayristir(tokenlar)?;

    let mut islemler = std::collections::HashMap::new();
    let mut kalan = Vec::new();
    for cumle in cumleler {
        match cumle {
            Cumle::IslemTanimi(islem) => {
                if islemler.contains_key(&islem.ad) {
                    return Err(Tani::yeni(
                        "A005",
                        format!("\"{}\" işlemi birden çok kez tanımlandı.", islem.ad),
                        islem.satir,
                        1,
                        1,
                    ));
                }
                islemler.insert(islem.ad.clone(), islem);
            }
            baska => kalan.push(baska),
        }
    }

    let mut program = Program { cumleler: kalan, islemler };
    cozumleyici::denetle(&mut program)?;
    Ok(program)
}

/// Kaynağı denetler, çalıştırmaz.
pub fn kaynagi_denetle(kaynak: &str) -> Result<(), Tani> {
    kaynagi_derle(kaynak).map(|_| ())
}

/// Kaynağı uçtan uca çalıştırır; çıktı satırlarını döndürür.
pub fn kaynagi_calistir(kaynak: &str) -> Result<Vec<String>, Tani> {
    kaynagi_calistir_girdiyle(kaynak, Vec::new())
}

/// Kaynağı hazır girdi satırlarıyla çalıştırır ("diye sor" cevapları sırayla
/// bu listeden gelir); istemler de çıktıya dahildir.
pub fn kaynagi_calistir_girdiyle(kaynak: &str, girdiler: Vec<String>) -> Result<Vec<String>, Tani> {
    let program = kaynagi_derle(kaynak)?;
    let mut io = yorumlayici::ToplayanIo::yeni(girdiler);
    yorumlayici::calistir_io(&program, &mut io)?;
    Ok(io.cikti)
}
