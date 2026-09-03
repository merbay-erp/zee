use crate::agac::Yapi;
use crate::intrinsic::IntrinsicTuru;
use crate::semantic_model::{SozlukDegerTuru, Tur, VeriTuru};
use crate::tani::Tani;

use super::baglam::Baglam;

pub(super) fn intrinsic_turunu_cevir(tur: IntrinsicTuru) -> Tur {
    match tur {
        IntrinsicTuru::Metin => Tur::Metin,
        IntrinsicTuru::Mantiksal => Tur::Mantiksal,
        IntrinsicTuru::AgYaniti => Tur::AgYaniti,
        IntrinsicTuru::MetinListesi => Tur::Liste(VeriTuru::Metin),
        IntrinsicTuru::MetinSozlukListesiSonucu => Tur::Sonuc(VeriTuru::MetinSozlukListesi),
        IntrinsicTuru::TamSayiSonucu => Tur::Sonuc(VeriTuru::TamSayi),
    }
}

/// Yapı alanı tür yazımını çözer ("TamSayı" → Tur::TamSayi).
pub(super) fn alan_turu(yazim: &str) -> Option<Tur> {
    match yazim {
        "TamSayı" => Some(Tur::TamSayi),
        "Ondalık" => Some(Tur::Ondalik),
        "Metin" => Some(Tur::Metin),
        "Mantıksal" => Some(Tur::Mantiksal),
        _ => None,
    }
}

/// K-083 parametre tür yazımını çözer. Sembolik generic yerine kontrollü
/// Türkçe kullanılır: `Ondalık listesi`, `Metin sözlüğü`, `Öğrenci`.
pub(super) fn parametre_turu(yazim: &str, baglam: &Baglam) -> Option<Tur> {
    let basit = |ad: &str| match ad {
        "TamSayı" => Some(Tur::TamSayi),
        "Ondalık" => Some(Tur::Ondalik),
        "Metin" => Some(Tur::Metin),
        "Mantıksal" => Some(Tur::Mantiksal),
        "Tarih" => Some(Tur::Tarih),
        "Saat" => Some(Tur::Saat),
        "Süre" => Some(Tur::Sure),
        "AğYanıtı" => Some(Tur::AgYaniti),
        "Hata" => Some(Tur::Hata),
        _ => baglam.yapi_kimligi(ad).map(Tur::Yapi),
    };
    if let Some(kok) = yazim.strip_suffix(" sonucu") {
        return parametre_turu(kok, baglam)
            .as_ref()
            .and_then(veri_turu_yap)
            .map(Tur::Sonuc);
    }
    if yazim == "Metin sözlüğü listesi" {
        return Some(Tur::Liste(VeriTuru::MetinSozluk));
    }
    if let Some(kok) = yazim.strip_suffix(" listesi") {
        return basit(kok)
            .and_then(|tur| veri_turu_yap(&tur))
            .map(Tur::Liste);
    }
    if let Some(kok) = yazim.strip_suffix(" sözlüğü") {
        return match basit(kok)? {
            Tur::TamSayi => Some(Tur::Sozluk(SozlukDegerTuru::TamSayi)),
            Tur::Ondalik => Some(Tur::Sozluk(SozlukDegerTuru::Ondalik)),
            Tur::Metin => Some(Tur::Sozluk(SozlukDegerTuru::Metin)),
            _ => None,
        };
    }
    if let Some(kok) = yazim.strip_suffix(" seçeneği") {
        return basit(kok)
            .and_then(|tur| veri_turu_yap(&tur))
            .map(Tur::Secenek);
    }
    basit(yazim)
}

pub(super) fn veri_turu_yap(tur: &Tur) -> Option<VeriTuru> {
    match tur {
        Tur::TamSayi => Some(VeriTuru::TamSayi),
        Tur::Metin => Some(VeriTuru::Metin),
        Tur::Ondalik => Some(VeriTuru::Ondalik),
        Tur::Sozluk(SozlukDegerTuru::TamSayi) => Some(VeriTuru::Sozluk),
        Tur::Sozluk(SozlukDegerTuru::Metin) => Some(VeriTuru::MetinSozluk),
        Tur::Liste(VeriTuru::MetinSozluk) => Some(VeriTuru::MetinSozlukListesi),
        Tur::Yapi(i) => Some(VeriTuru::Yapi(*i)),
        _ => None,
    }
}

/// K-045: biri "henüz boş" (Bilinmeyen) koleksiyonsa somut eşiyle uzlaşır;
/// dönen tür bağlamın yeni türüdür. Uzlaşma yoksa None (T002 yolu).
pub(super) fn bos_koleksiyon_uzlasi(eski: &Tur, yeni: &Tur) -> Option<Tur> {
    match (eski, yeni) {
        (Tur::Liste(VeriTuru::Bilinmeyen), Tur::Liste(_)) => Some(*yeni),
        (Tur::Liste(_), Tur::Liste(VeriTuru::Bilinmeyen)) => Some(*eski),
        (Tur::Sozluk(SozlukDegerTuru::Bilinmeyen), Tur::Sozluk(_)) => Some(*yeni),
        (Tur::Sozluk(_), Tur::Sozluk(SozlukDegerTuru::Bilinmeyen)) => Some(*eski),
        _ => None,
    }
}

pub(super) fn yapi_turu_tanilari(yapilar: &[Yapi]) -> Vec<Tani> {
    let mut tanilar = Vec::new();
    for yapi in yapilar {
        for (alan, tur_yazimi) in &yapi.alanlar {
            if alan_turu(tur_yazimi).is_none() {
                tanilar.push(
                    Tani::yeni(
                        "T027",
                        format!(
                            "\"{}\" yapısındaki \"{}\" alanının türü tanınmadı: \"{}\".",
                            yapi.ad, alan, tur_yazimi
                        ),
                        yapi.satir,
                        1,
                        1,
                    )
                    .onerili(
                        "Kullanılabilir alan türleri: TamSayı, Ondalık, Metin, Mantıksal.".into(),
                    ),
                );
            }
        }
    }
    tanilar
}
