//! Çekirdek AST ile alan adaptörleri arasındaki kararlı intrinsic kayıt katmanı.
//!
//! Kaynak sözdizimi parser'da bu kimliklere indirilir. AST yalnız kimlik ve
//! argüman taşır; tür, yetkinlik ve etki bilgisi bu tek tabloda tutulur.

pub const HTTP_GETIR: &str = "ag.http_getir";
pub const SENSOR_ACIK_MI: &str = "donanim.sensor_acik_mi";
pub const ISTEMCI_ADRESI: &str = "web.istemci_adresi";
pub const CSRF_BELIRTECI: &str = "web.csrf_belirteci";
pub const PAROLA_DOGRULA: &str = "guvenlik.parola_dogrula";
pub const WEB_OTURUMLARINI_LISTELE: &str = "web.oturumlarini_listele";
pub const WEB_OTURUMUNU_IPTAL_ET: &str = "web.oturumunu_iptal_et";
pub const POSTGRESQL_OKU: &str = "veritabani.postgresql_oku";
pub const POSTGRESQL_DEGISTIR: &str = "veritabani.postgresql_degistir";
pub const DOSYA_ATOMIK_TASI: &str = "dosya.atomik_tasi";
pub const DOSYA_SIL: &str = "dosya.sil";
pub const DOSYALARI_LISTELE: &str = "dosya.listele";
pub const DOSYA_SHA256: &str = "dosya.sha256";

pub use crate::yetkinlik::Yetkinlik;

/// Intrinsic imzalarında kullanılabilen çekirdek türlerin kapalı kümesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicTuru {
    Metin,
    Mantiksal,
    AgYaniti,
    MetinListesi,
    MetinSozlukListesiSonucu,
    MetinListesiSonucu,
    MetinSonucu,
    TamSayiSonucu,
}

/// Statik etki özeti. `WebAdaptoru`, uygulama eylemlerinin HTTP katmanına
/// bağımlı olmadığını kanıtlayan mevcut T044 sınırına bağlanır.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicEtkisi {
    Saf,
    DisOkuma,
    DisYazma,
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
const METIN_VE_METIN_LISTESI: &[IntrinsicTuru] =
    &[IntrinsicTuru::Metin, IntrinsicTuru::MetinListesi];

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
        kimlik: ISTEMCI_ADRESI,
        yetkinlik: Yetkinlik::AgSunucusu,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "İstemci adresi argüman almaz.",
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
        tur_hatasi: "Parola ve parola özeti Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: WEB_OTURUMLARINI_LISTELE,
        yetkinlik: Yetkinlik::WebOturumu,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSozlukListesiSonucu,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "Oturumları listelenecek kullanıcı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: WEB_OTURUMUNU_IPTAL_ET,
        yetkinlik: Yetkinlik::WebOturumu,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "İptal edilecek web oturum kimliği Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: POSTGRESQL_OKU,
        yetkinlik: Yetkinlik::Veritabani,
        arguman_turleri: METIN_VE_METIN_LISTESI,
        donus_turu: IntrinsicTuru::MetinSozlukListesiSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "PostgreSQL sorgusu Metin, parametreleri Metin listesi olmalı.",
    },
    IntrinsicTanimi {
        kimlik: POSTGRESQL_DEGISTIR,
        yetkinlik: Yetkinlik::Veritabani,
        arguman_turleri: METIN_VE_METIN_LISTESI,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "PostgreSQL değişikliği Metin sorgu ve Metin parametre listesi ister.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_ATOMIK_TASI,
        yetkinlik: Yetkinlik::DosyaYazma,
        arguman_turleri: IKI_METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "Kaynak ve hedef dosya yolları Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_SIL,
        yetkinlik: Yetkinlik::DosyaYazma,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "Silinecek dosya yolu Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYALARI_LISTELE,
        yetkinlik: Yetkinlik::DosyaOkuma,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinListesiSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Listelenecek dizin yolu Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_SHA256,
        yetkinlik: Yetkinlik::DosyaOkuma,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Özetlenecek dosya yolu Metin olmalı.",
    },
];

pub fn tanim(kimlik: &str) -> Option<&'static IntrinsicTanimi> {
    TANIMLAR.iter().find(|tanim| tanim.kimlik == kimlik)
}
