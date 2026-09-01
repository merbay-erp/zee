//! Çekirdek AST ile alan adaptörleri arasındaki kararlı intrinsic kayıt katmanı.
//!
//! Kaynak sözdizimi parser'da bu kimliklere indirilir. AST yalnız kimlik ve
//! argüman taşır; tür, yetkinlik ve etki bilgisi bu tek tabloda tutulur.

pub const HTTP_GETIR: &str = "ag.http_getir";
pub const SENSOR_ACIK_MI: &str = "donanim.sensor_acik_mi";
pub const CSRF_BELIRTECI: &str = "web.csrf_belirteci";
pub const PAROLA_DOGRULA: &str = "guvenlik.parola_dogrula";

/// Bir intrinsic'in çalışmak için ihtiyaç duyduğu dış dünya sınırı.
///
/// Bu sınıflandırma bugün denetleyici ve etki çözümlemesinin ortak sözlüğüdür;
/// kullanıcı/paket bazlı izin denetimi B-023'te bunun üstüne kurulacaktır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Yetkinlik {
    Ag,
    Donanim,
    WebOturumu,
    Kriptografi,
}

/// Intrinsic imzalarında kullanılabilen çekirdek türlerin kapalı kümesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicTuru {
    Metin,
    Mantiksal,
    AgYaniti,
}

/// Statik etki özeti. `WebAdaptoru`, uygulama eylemlerinin HTTP katmanına
/// bağımlı olmadığını kanıtlayan mevcut T044 sınırına bağlanır.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicEtkisi {
    Saf,
    DisOkuma,
    WebAdaptoru,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntrinsicTanimi {
    pub kimlik: &'static str,
    pub yetkinlik: Yetkinlik,
    pub arguman_turleri: &'static [IntrinsicTuru],
    pub donus_turu: IntrinsicTuru,
    pub etki: IntrinsicEtkisi,
    /// Kaynak türü imzaya uymadığında gösterilecek kararlı Türkçe açıklama.
    pub tur_hatasi: &'static str,
}

const METIN: &[IntrinsicTuru] = &[IntrinsicTuru::Metin];
const IKI_METIN: &[IntrinsicTuru] = &[IntrinsicTuru::Metin, IntrinsicTuru::Metin];
const YOK: &[IntrinsicTuru] = &[];

pub const TANIMLAR: &[IntrinsicTanimi] = &[
    IntrinsicTanimi {
        kimlik: HTTP_GETIR,
        yetkinlik: Yetkinlik::Ag,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::AgYaniti,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Adres Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: SENSOR_ACIK_MI,
        yetkinlik: Yetkinlik::Donanim,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::Mantiksal,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Sensör adı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: CSRF_BELIRTECI,
        yetkinlik: Yetkinlik::WebOturumu,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "CSRF belirteci argüman almaz.",
    },
    IntrinsicTanimi {
        kimlik: PAROLA_DOGRULA,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: IKI_METIN,
        donus_turu: IntrinsicTuru::Mantiksal,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "Parola ve Argon2id özeti Metin olmalı.",
    },
];

pub fn tanim(kimlik: &str) -> Option<&'static IntrinsicTanimi> {
    TANIMLAR.iter().find(|tanim| tanim.kimlik == kimlik)
}
