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
/// K-210: `<parola>nın argon2 özeti` — Argon2id PHC parola özeti üretimi.
pub const PAROLA_OZETI: &str = "guvenlik.parola_ozeti";
/// K-210: `yeni totp anahtarı` — 160 bit Base32 TOTP gizli anahtarı.
pub const TOTP_ANAHTARI: &str = "guvenlik.totp_anahtari";
/// K-210: `<kod> <gizli> anahtarıyla doğrulanıyorsa` — RFC 6238 TOTP doğrulaması.
pub const TOTP_DOGRULA: &str = "guvenlik.totp_dogrula";
pub const POSTGRESQL_OKU: &str = "veritabani.postgresql_oku";
pub const POSTGRESQL_DEGISTIR: &str = "veritabani.postgresql_degistir";
pub const DOSYA_ATOMIK_TASI: &str = "dosya.atomik_tasi";
pub const DOSYA_SIL: &str = "dosya.sil";
pub const DOSYALARI_LISTELE: &str = "dosya.listele";
pub const DOSYA_SHA256: &str = "dosya.sha256";
/// K-210: `<alıcı> <konu> <gövde> ile ileti göndermeyi dene` — SMTP e-posta.
pub const EPOSTA_GONDER: &str = "iletisim.eposta_gonder";
/// K-210 eki: `<alıcı> <konu> <gövde> ile <bağlantı> üzerinden ileti göndermeyi dene` —
/// SMTP bağlantısı ortam değişkeninden değil değerden gelir (panelden ayarlanabilir SMTP).
pub const EPOSTA_GONDER_BAGLANTILI: &str = "iletisim.eposta_gonder_baglantili";
/// K-210: `<kaynak> görselini <hedef> dosyasına <N> piksele küçültmeyi dene`.
pub const GORSEL_KUCULT: &str = "gorsel.kucult";
/// K-210 eki 3: `<metin>in gizli şifrelenmişi` — XChaCha20-Poly1305, anahtar `gizli_anahtar_değişkeni`.
pub const GIZLI_SIFRELE: &str = "guvenlik.gizli_sifrele";
/// K-210 eki 3: `<şifreli> gizli çözmeyi dene` → Metin Sonucu.
pub const GIZLI_COZ: &str = "guvenlik.gizli_coz";

pub use crate::yetkinlik::Yetkinlik;

/// Intrinsic imzalarında kullanılabilen çekirdek türlerin kapalı kümesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicTuru {
    Metin,
    TamSayi,
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
    /// Transaction ile geri alınamayan dış etki (e-posta gibi): eylem içinde
    /// T048, rota/CLI adaptöründe serbest.
    GeriAlinamaz,
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
const UC_METIN: &[IntrinsicTuru] = &[
    IntrinsicTuru::Metin,
    IntrinsicTuru::Metin,
    IntrinsicTuru::Metin,
];
const DORT_METIN: &[IntrinsicTuru] = &[
    IntrinsicTuru::Metin,
    IntrinsicTuru::Metin,
    IntrinsicTuru::Metin,
    IntrinsicTuru::Metin,
];
const IKI_METIN_BIR_SAYI: &[IntrinsicTuru] = &[
    IntrinsicTuru::Metin,
    IntrinsicTuru::Metin,
    IntrinsicTuru::TamSayi,
];

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
        kimlik: PAROLA_OZETI,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "Özeti alınacak parola Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: TOTP_ANAHTARI,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "TOTP anahtarı argüman almaz.",
    },
    IntrinsicTanimi {
        kimlik: TOTP_DOGRULA,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: IKI_METIN,
        donus_turu: IntrinsicTuru::Mantiksal,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "TOTP kodu ve gizli anahtar Metin olmalı.",
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
    IntrinsicTanimi {
        kimlik: EPOSTA_GONDER,
        yetkinlik: Yetkinlik::Ag,
        arguman_turleri: UC_METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Alıcı, konu ve gövde Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: EPOSTA_GONDER_BAGLANTILI,
        yetkinlik: Yetkinlik::Ag,
        arguman_turleri: DORT_METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Alıcı, konu, gövde ve bağlantı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: GIZLI_SIFRELE,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "Şifrelenecek gizli değer Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: GIZLI_COZ,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "Çözülecek gizli değer Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: GORSEL_KUCULT,
        yetkinlik: Yetkinlik::DosyaYazma,
        arguman_turleri: IKI_METIN_BIR_SAYI,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "Kaynak ve hedef görsel yolu Metin, azami kenar TamSayı olmalı.",
    },
];

pub fn tanim(kimlik: &str) -> Option<&'static IntrinsicTanimi> {
    TANIMLAR.iter().find(|tanim| tanim.kimlik == kimlik)
}
