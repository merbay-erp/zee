use crate::agac::Yapi;
use crate::intrinsic::IntrinsicTuru;
use crate::kimlik::YapiId;
use crate::tani::Tani;

use super::baglam::Baglam;

/// Kapsayıcı türlerin (Liste, Seçenek) taşıyabildiği öğe türleri (v0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VeriTuru {
    TamSayi,
    Metin,
    Ondalik,
    /// v0'da örtük olarak Sözlük<Metin, TamSayı> demektir (CSV satırları).
    Sozluk,
    /// Kullanıcı yapısı öğesi (K-060): checker yapı dizinine semantic bağ.
    Yapi(YapiId),
    /// Metin değerli satır sözlüğü (K-062): CSV satırları böyle okunur.
    MetinSozluk,
    /// Yapılandırılmış beklenen hata değeri (K-091).
    Hata,
    /// Boş koleksiyonun henüz belirlenmemiş öğe türü (K-045): ilk eklemede
    /// somutlaşır. Guard'lı yollar dışında ture() çağrılmaz.
    Bilinmeyen,
}

impl VeriTuru {
    pub(super) fn adi(&self) -> &'static str {
        match self {
            VeriTuru::TamSayi => "TamSayı",
            VeriTuru::Metin => "Metin",
            VeriTuru::Ondalik => "Ondalık",
            VeriTuru::Bilinmeyen => "belirsiz",
            VeriTuru::Yapi(_) => "Yapı",
            VeriTuru::Sozluk => "Sözlük",
            VeriTuru::MetinSozluk => "satır",
            VeriTuru::Hata => "Hata",
        }
    }
    pub(super) fn ture(&self) -> Tur {
        match self {
            VeriTuru::TamSayi => Tur::TamSayi,
            VeriTuru::Metin => Tur::Metin,
            VeriTuru::Ondalik => Tur::Ondalik,
            VeriTuru::Bilinmeyen => Tur::Yok, // guard'lar erişimi engeller
            VeriTuru::Yapi(i) => Tur::Yapi(*i),
            VeriTuru::Sozluk => Tur::Sozluk(SozlukDegerTuru::TamSayi),
            VeriTuru::MetinSozluk => Tur::Sozluk(SozlukDegerTuru::Metin),
            VeriTuru::Hata => Tur::Hata,
        }
    }
}

/// Sözlük değerlerinin türü (v0: TamSayı ya da Metin).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SozlukDegerTuru {
    /// Boş sözlüğün henüz belirlenmemiş değer türü (K-045).
    Bilinmeyen,
    /// Para/oran sözlükleri (K-067).
    Ondalik,
    TamSayi,
    Metin,
}

impl SozlukDegerTuru {
    pub(super) fn ture(&self) -> Tur {
        match self {
            SozlukDegerTuru::Bilinmeyen => Tur::Yok, // guard'lar erişimi engeller
            SozlukDegerTuru::Ondalik => Tur::Ondalik,
            SozlukDegerTuru::TamSayi => Tur::TamSayi,
            SozlukDegerTuru::Metin => Tur::Metin,
        }
    }
    pub(super) fn adi(&self) -> &'static str {
        match self {
            SozlukDegerTuru::Bilinmeyen => "belirsiz",
            SozlukDegerTuru::Ondalik => "Ondalık",
            SozlukDegerTuru::TamSayi => "TamSayı",
            SozlukDegerTuru::Metin => "Metin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tur {
    TamSayi,
    /// Onluk tam değerli ondalık sayı (RFC-0013).
    Ondalik,
    Metin,
    Mantiksal,
    Liste(VeriTuru),
    /// Sözlük<Metin, değer türü>; anahtarlar v0'da hep Metin.
    Sozluk(SozlukDegerTuru),
    Secenek(VeriTuru),
    /// Sonuç<değer, Hata>: değer türü parametreli, hata yapılandırılmıştır.
    Sonuc(VeriTuru),
    /// Yalnız "yok" sabitinin türü; dönüş birleşiminde Seçenek'e erir.
    Yok,
    /// Kullanıcının erişebildiği yapılandırılmış beklenen hata değeri (K-091).
    Hata,
    /// Yalnız "hatasını döndür"ün iç işareti; birleşimde Sonuç'a erir.
    HataDonusu,
    /// Kullanıcı yapısı — depolama konumundan bağımsız semantic bağ.
    Yapi(YapiId),
    Tarih,
    Saat,
    /// Milisaniye hassasiyetli süre (RFC-0011/0013).
    Sure,
    /// HTTP yanıtı: durum kodu + gövde (golden 24).
    AgYaniti,
}

impl Tur {
    pub fn adi(&self) -> String {
        match self {
            Tur::TamSayi => "TamSayı".into(),
            Tur::Ondalik => "Ondalık".into(),
            Tur::Metin => "Metin".into(),
            Tur::Mantiksal => "Mantıksal".into(),
            Tur::Liste(e) => format!("Liste<{}>", e.adi()),
            Tur::Sozluk(e) => format!("Sözlük<Metin, {}>", e.adi()),
            Tur::Secenek(e) => format!("Seçenek<{}>", e.adi()),
            Tur::Sonuc(e) => format!("Sonuç<{}>", e.adi()),
            Tur::Yok => "yok".into(),
            Tur::Hata => "Hata".into(),
            Tur::HataDonusu => "hata dönüşü".into(),
            Tur::Yapi(_) => "Yapı".into(),
            Tur::Tarih => "Tarih".into(),
            Tur::Saat => "Saat".into(),
            Tur::Sure => "Süre".into(),
            Tur::AgYaniti => "AğYanıtı".into(),
        }
    }

    pub(super) fn veri_turu(&self) -> Option<VeriTuru> {
        match self {
            Tur::TamSayi => Some(VeriTuru::TamSayi),
            Tur::Metin => Some(VeriTuru::Metin),
            Tur::Ondalik => Some(VeriTuru::Ondalik),
            Tur::Hata => Some(VeriTuru::Hata),
            _ => None,
        }
    }

    /// TamSayı ya da Ondalık mı? (Karışımda TamSayı, Ondalık'a kayıpsız genişler.)
    pub(super) fn sayisal(&self) -> bool {
        matches!(self, Tur::TamSayi | Tur::Ondalik)
    }
}

pub(super) fn intrinsic_turunu_cevir(tur: IntrinsicTuru) -> Tur {
    match tur {
        IntrinsicTuru::Metin => Tur::Metin,
        IntrinsicTuru::Mantiksal => Tur::Mantiksal,
        IntrinsicTuru::AgYaniti => Tur::AgYaniti,
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
    if let Some(kok) = yazim.strip_suffix(" sonucu") {
        return basit(kok)
            .and_then(|tur| veri_turu_yap(&tur))
            .map(Tur::Sonuc);
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
