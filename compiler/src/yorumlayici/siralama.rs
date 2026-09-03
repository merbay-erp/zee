//! Değer sıralama ve Türk alfabesi karşılaştırma sahipliği.

use super::*;

pub(super) fn deger_sirasi(a: &Deger, b: &Deger) -> std::cmp::Ordering {
    match (a, b) {
        (Deger::TamSayi(x), Deger::TamSayi(y)) => x.cmp(y),
        (Deger::Ondalik(_), _) | (_, Deger::Ondalik(_)) => match (sayisal_ac(a), sayisal_ac(b)) {
            (Some(sol), Some(sag)) => sol.karsilastir(&sag),
            _ => std::cmp::Ordering::Equal,
        },
        (Deger::Metin(x), Deger::Metin(y)) => turkce_karsilastir(x, y),
        _ => std::cmp::Ordering::Equal,
    }
}

fn turkce_karsilastir(a: &str, b: &str) -> std::cmp::Ordering {
    a.chars()
        .map(turkce_harf_sirasi)
        .cmp(b.chars().map(turkce_harf_sirasi))
}

fn turkce_harf_sirasi(k: char) -> (u8, u32) {
    const ALFABE: &str = "abcçdefgğhıijklmnoöprsştuüvyz";
    let kucuk = match k {
        'İ' => 'i',
        'I' => 'ı',
        _ => k.to_lowercase().next().unwrap_or(k),
    };
    match ALFABE.chars().position(|a| a == kucuk) {
        Some(sira) => (0, sira as u32),
        None => (1, k as u32),
    }
}
