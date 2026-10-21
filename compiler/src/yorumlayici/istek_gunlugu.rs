//! İstek düzeyine çevrilen rota tanısının operatör günlüğü satırı (K-324).

use crate::tani::Tani;

/// Bir günlük satırının azami bayt uzunluğu; uzun mesaj kırpılır.
const AZAMI_SATIR_BAYTI: usize = 4096;

/// Tek satırlık `anahtar=değer` kaydı kurar. Yol sorgusuz yazılır; yol ve
/// mesajdaki tırnak, ters bölü ve denetim karakterleri kaçışlanır, böylece
/// istemci verisi sahte günlük satırı üretemez.
pub fn istek_tani_gunluk_satiri(durum: u16, tani: &Tani, istek: Option<(&str, &str)>) -> String {
    let mut satir = format!(
        "zee-web istek-hatası durum={durum} kod={} satır={}",
        tani.kod, tani.satir
    );
    istek_alanlarini_ekle(&mut satir, istek);
    satir.push_str(&format!(" mesaj=\"{}\"", kacisla(&tani.mesaj)));
    kirp(satir)
}

/// İstemci yanıtın yazımı sırasında koptuğunda (W1) yazılan tek satırlık kayıt.
pub fn yanit_kopmasi_gunluk_satiri(durum: u16, hata: &str, istek: Option<(&str, &str)>) -> String {
    let mut satir = format!("zee-web istemci yanıt sırasında koptu: durum={durum}");
    istek_alanlarini_ekle(&mut satir, istek);
    satir.push_str(&format!(" hata=\"{}\"", kacisla(hata)));
    kirp(satir)
}

fn istek_alanlarini_ekle(satir: &mut String, istek: Option<(&str, &str)>) {
    if let Some((yontem, hedef)) = istek {
        let yol = hedef.split(['?', '#']).next().unwrap_or("");
        let (yontem, yol) = (kacisla(yontem), kacisla(yol));
        satir.push_str(&format!(" yöntem={yontem} yol=\"{yol}\""));
    }
}

fn kirp(mut satir: String) -> String {
    if satir.len() > AZAMI_SATIR_BAYTI {
        satir.truncate(satir.floor_char_boundary(AZAMI_SATIR_BAYTI - 4));
        satir.push_str("…\"");
    }
    satir
}

fn kacisla(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len());
    for karakter in metin.chars() {
        match karakter {
            '"' => cikti.push_str("\\\""),
            '\\' => cikti.push_str("\\\\"),
            '\n' => cikti.push_str("\\n"),
            '\r' => cikti.push_str("\\r"),
            '\t' => cikti.push_str("\\t"),
            k if k.is_control() => cikti.push_str(&format!("\\u{{{:x}}}", k as u32)),
            k => cikti.push(k),
        }
    }
    cikti
}
