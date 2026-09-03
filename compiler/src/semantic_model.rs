//! Checker ve HIR'ın birlikte kullandığı bağımsız semantic tür modeli.

use crate::kimlik::YapiId;

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
    /// PostgreSQL okumasının metin değerli satır listesi.
    MetinSozlukListesi,
    /// Yapılandırılmış beklenen hata değeri (K-091).
    Hata,
    /// Boş koleksiyonun henüz belirlenmemiş öğe türü (K-045): ilk eklemede
    /// somutlaşır. Guard'lı yollar dışında `ture` çağrılmaz.
    Bilinmeyen,
}

/// Sözlük değerlerinin türü (v0: TamSayı, Ondalık ya da Metin).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SozlukDegerTuru {
    /// Boş sözlüğün henüz belirlenmemiş değer türü (K-045).
    Bilinmeyen,
    /// Para/oran sözlükleri (K-067).
    Ondalik,
    TamSayi,
    Metin,
}

/// Zee ifadelerinin checker ile HIR arasında taşınan semantic türü.
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

impl VeriTuru {
    pub(crate) fn adi(&self) -> &'static str {
        match self {
            VeriTuru::TamSayi => "TamSayı",
            VeriTuru::Metin => "Metin",
            VeriTuru::Ondalik => "Ondalık",
            VeriTuru::Bilinmeyen => "belirsiz",
            VeriTuru::Yapi(_) => "Yapı",
            VeriTuru::Sozluk => "Sözlük",
            VeriTuru::MetinSozluk => "satır",
            VeriTuru::MetinSozlukListesi => "Metin sözlüğü listesi",
            VeriTuru::Hata => "Hata",
        }
    }

    pub(crate) fn ture(&self) -> Tur {
        match self {
            VeriTuru::TamSayi => Tur::TamSayi,
            VeriTuru::Metin => Tur::Metin,
            VeriTuru::Ondalik => Tur::Ondalik,
            VeriTuru::Bilinmeyen => Tur::Yok, // guard'lar erişimi engeller
            VeriTuru::Yapi(i) => Tur::Yapi(*i),
            VeriTuru::Sozluk => Tur::Sozluk(SozlukDegerTuru::TamSayi),
            VeriTuru::MetinSozluk => Tur::Sozluk(SozlukDegerTuru::Metin),
            VeriTuru::MetinSozlukListesi => Tur::Liste(VeriTuru::MetinSozluk),
            VeriTuru::Hata => Tur::Hata,
        }
    }
}

impl SozlukDegerTuru {
    pub(crate) fn ture(&self) -> Tur {
        match self {
            SozlukDegerTuru::Bilinmeyen => Tur::Yok, // guard'lar erişimi engeller
            SozlukDegerTuru::Ondalik => Tur::Ondalik,
            SozlukDegerTuru::TamSayi => Tur::TamSayi,
            SozlukDegerTuru::Metin => Tur::Metin,
        }
    }

    pub(crate) fn adi(&self) -> &'static str {
        match self {
            SozlukDegerTuru::Bilinmeyen => "belirsiz",
            SozlukDegerTuru::Ondalik => "Ondalık",
            SozlukDegerTuru::TamSayi => "TamSayı",
            SozlukDegerTuru::Metin => "Metin",
        }
    }
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

    pub(crate) fn veri_turu(&self) -> Option<VeriTuru> {
        match self {
            Tur::TamSayi => Some(VeriTuru::TamSayi),
            Tur::Metin => Some(VeriTuru::Metin),
            Tur::Ondalik => Some(VeriTuru::Ondalik),
            Tur::Hata => Some(VeriTuru::Hata),
            _ => None,
        }
    }

    /// TamSayı ya da Ondalık mı? (Karışımda TamSayı, Ondalık'a kayıpsız genişler.)
    pub(crate) fn sayisal(&self) -> bool {
        matches!(self, Tur::TamSayi | Tur::Ondalik)
    }
}
