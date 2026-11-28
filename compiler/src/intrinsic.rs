//! Çekirdek AST ile alan adaptörleri arasındaki kararlı intrinsic kayıt katmanı.
//!
//! Kaynak sözdizimi parser'da bu kimliklere indirilir. AST yalnız kimlik ve
//! argüman taşır; tür, yetkinlik ve etki bilgisi bu tek tabloda tutulur.

pub const HTTP_GETIR: &str = "ag.http_getir";
pub const HTTP_JSON: &str = "ag.http_json";
pub const HTTP_ISTEK: &str = "ag.http_istek";
pub const HTTP_GIZLI_ORTAM: &str = "ag.http_gizli_ortam";
pub const SENSOR_ACIK_MI: &str = "donanim.sensor_acik_mi";
pub const ISTEMCI_ADRESI: &str = "web.istemci_adresi";
pub const CSRF_BELIRTECI: &str = "web.csrf_belirteci";
/// K-405: `isteğin ham gövdesi` — yalnız makine uç noktası rota gövdesinde,
/// en çok 8 KiB katı UTF-8 istek gövdesi → Metin.
pub const WEB_HAM_GOVDE: &str = "web.istek_ham_govdesi";
/// K-337: `birincil süreç` — çok işçi grubunun ilk birincil işçisinde ve tek
/// süreçte doğru; ikincil ya da yeniden başlatılan işçide yanlış.
pub const WEB_BIRINCIL_SUREC: &str = "web.birincil_surec";
pub const PAROLA_DOGRULA: &str = "guvenlik.parola_dogrula";
pub const WEB_OTURUMLARINI_LISTELE: &str = "web.oturumlarini_listele";
pub const WEB_OTURUMUNU_IPTAL_ET: &str = "web.oturumunu_iptal_et";
/// K-210: `<parola>nın argon2 özeti` — Argon2id PHC parola özeti üretimi.
pub const PAROLA_OZETI: &str = "guvenlik.parola_ozeti";
/// K-210: `yeni totp anahtarı` — 160 bit Base32 TOTP gizli anahtarı.
pub const TOTP_ANAHTARI: &str = "guvenlik.totp_anahtari";
/// RFC-0047 E (ADR-182 dilim B): `yeni uygulama parolası` — cihaz başına
/// uygulama parolası; `{görünen, önek, özet}` sözlüğü.
pub const UYGULAMA_PAROLASI: &str = "guvenlik.uygulama_parolasi";
/// RFC-0047 B (ADR-182 dilim D): `isteğin "Depth" başlığı` — DAV rotasında.
pub const WEB_ISTEK_DERINLIGI: &str = "web.istek_derinligi";
/// RFC-0047 B (ADR-182 dilim D): `isteğin "If-Match" / "If-None-Match"
/// başlığı` → Liste<Metin> (`*` ayrı değer); DAV rotasında.
pub const WEB_ISTEK_KOSULU: &str = "web.istek_kosulu";
/// K-210: `<kod> <gizli> anahtarıyla doğrulanıyorsa` — RFC 6238 TOTP doğrulaması.
pub const TOTP_DOGRULA: &str = "guvenlik.totp_dogrula";
pub const POSTGRESQL_OKU: &str = "veritabani.postgresql_oku";
pub const POSTGRESQL_DEGISTIR: &str = "veritabani.postgresql_degistir";
pub const DOSYA_ATOMIK_TASI: &str = "dosya.atomik_tasi";
pub const DOSYA_SIL: &str = "dosya.sil";
pub const DOSYALARI_LISTELE: &str = "dosya.listele";
pub const DOSYA_SHA256: &str = "dosya.sha256";
/// K-331: `<yol> dosyasının ilk <N> baytını onaltılık okumayı dene` — 1..=64 baytlık
/// salt-okur içerik imzası (küçük harf hex) → Metin Sonucu.
pub const DOSYA_ILK_BAYTLAR: &str = "dosya.ilk_baytlar";
/// K-331: imza okumasının sabit üst sınırı (bayt).
pub const DOSYA_ILK_BAYT_SINIRI: i64 = 64;
/// K-389 (RFC-0043 Dilim 1): `<yol> dosyasının <konum> baytından <n> baytını
/// metin olarak okumayı dene` — en çok 1 MiB'lık aralık, katı UTF-8 → Metin Sonucu.
pub const DOSYA_ARALIK_METIN: &str = "dosya.aralik_metin";
/// K-389: `<yol> dosyasının <konum> baytından <n> baytını base64 okumayı dene`
/// — ikili aralık RFC 4648 base64 Metin Sonucu olarak.
pub const DOSYA_ARALIK_BASE64: &str = "dosya.aralik_base64";
/// K-389: `<içerik> base64 çözerek <yol> dosyasına yazmayı dene` — MIME base64'ü
/// çözüp yeni dosyaya no-clobber bayt-kesin yazar → `yol`/`sha256`/`bayt` sözlüğü.
pub const DOSYA_BASE64_YAZ: &str = "dosya.base64_cozerek_yaz";
/// K-389: `<içerik> base64 çözerek <yol> dosyasına eklemeyi dene` — akışlı ekleme.
pub const DOSYA_BASE64_EKLE: &str = "dosya.base64_cozerek_ekle";
/// K-389: `<içerik> quoted_printable çözerek <yol> dosyasına yazmayı dene`.
pub const DOSYA_QP_YAZ: &str = "dosya.qp_cozerek_yaz";
/// K-389: `<içerik> quoted_printable çözerek <yol> dosyasına eklemeyi dene`.
pub const DOSYA_QP_EKLE: &str = "dosya.qp_cozerek_ekle";
/// K-408 (ADR-172 eki, ZM-F049): `<kaynak> dosyasını base64 satırlarıyla
/// <hedef> dosyasına eklemeyi dene` — K-396 bağlantı aktarımının dosya
/// hedefli aynası; RFC 2045 satırları akışla hedefin sonuna → TamSayı Sonucu.
pub const DOSYA_BASE64_SATIRLARI_EKLE: &str = "dosya.base64_satirlari_ekle";
/// K-378: `<yol> dosyasını <tür> türünde <ad> adıyla ek olarak göndermeyi dene`
/// — rota yanıtını kayıtlı içerik türüyle `Content-Disposition: attachment`
/// dosya yanıtı yapar → TamSayı Sonucu (bayt).
pub const WEB_DOSYA_YANITI: &str = "web.dosya_yaniti";
/// K-381: `bağlantıdan satır okumayı dene` — o anki bağlantı görevinden tek
/// CRLF satırı → Metin Sonucu (C030 `tür` verili ret).
pub const BAGLANTI_SATIR_OKU: &str = "ag.baglanti_satir_oku";
/// K-381: `bağlantıya <metin> satırını yazmayı dene` → TamSayı Sonucu
/// (CRLF dahil kuyruğa alınan bayt).
pub const BAGLANTI_SATIR_YAZ: &str = "ag.baglanti_satir_yaz";
/// K-381 A2b: `bağlantıda TLS başlatmayı dene` — STARTTLS yükseltmesi →
/// Metin Sonucu (anlaşılan sürüm ve şifre takımı).
pub const BAGLANTI_TLS_BASLAT: &str = "ag.baglanti_tls_baslat";
/// K-384 (RFC-0039 A2c): `bağlantıdan nokta bloğunu dosyaya almayı dene` —
/// SMTP `DATA` gövdesini geçici dosyaya akıtır → Metin sözlüğü Sonucu
/// (`yol`, `sha256`, `bayt`).
pub const BAGLANTI_NOKTA_BLOGU: &str = "ag.baglanti_nokta_blogu";
/// K-384: `bağlantıdan <n> baytı dosyaya almayı dene` — IMAP literal'i.
pub const BAGLANTI_BAYT_BLOGU: &str = "ag.baglanti_bayt_blogu";
/// K-384 (ZM-F031): `bağlantıdan <n> bayt okumayı dene` → Metin Sonucu.
pub const BAGLANTI_BAYT_OKU: &str = "ag.baglanti_bayt_oku";
/// K-384 (ZM-F031): `bağlantıya <metin> parçasını yazmayı dene` — satır
/// sonu eklemeden → TamSayı Sonucu.
pub const BAGLANTI_PARCA_YAZ: &str = "ag.baglanti_parca_yaz";
/// K-384: `bağlantıya <yol> dosyasını aktarmayı dene` → TamSayı Sonucu.
pub const BAGLANTI_DOSYA_AKTAR: &str = "ag.baglanti_dosya_aktar";
/// K-396 (RFC-0039 A2d, ZM-F040): `<yol> dosyasını base64 satırlarıyla
/// bağlantıya aktarmayı dene` — RFC 2045 satırlı base64 akışı → TamSayı Sonucu.
pub const BAGLANTI_BASE64_AKTAR: &str = "ag.baglanti_base64_aktar";
/// K-396: `<yol> dosyasının base64 satırlarının <konum> baytından <n> baytını
/// bağlantıya aktarmayı dene` — kodlanmış çıktı üzerinde aralık (IMAP partial).
pub const BAGLANTI_BASE64_ARALIK_AKTAR: &str = "ag.baglanti_base64_aralik_aktar";

/// K-384/K-396: dosyayı bağlantıya akıtan (dolayısıyla dosyayı da okuyan)
/// intrinsic mi.
/// Tanımındaki yetkinliğe ek olarak dosya okuyan intrinsic'ler (derleme
/// zamanı `dosya-okuma`; `veri:` yolunda `veri-okuma` ile karşılanır).
pub fn dosya_da_okur(kimlik: &str) -> bool {
    baglantiya_dosya_aktarir(kimlik)
        || kimlik == POSTA_DKIM_DOSYA_GOVDE_OZETI
        || kimlik == DOSYA_BASE64_SATIRLARI_EKLE
}

pub fn baglantiya_dosya_aktarir(kimlik: &str) -> bool {
    [
        BAGLANTI_DOSYA_AKTAR,
        BAGLANTI_BASE64_AKTAR,
        BAGLANTI_BASE64_ARALIK_AKTAR,
    ]
    .contains(&kimlik)
}
/// K-384 (ZM-F032): `bağlantıdan satır okumayı <n> saniye içinde dene` —
/// kısa okuma süresi; zaman aşımı oturumu kapatmaz → Metin Sonucu.
pub const BAGLANTI_SATIR_OKU_SURELI: &str = "ag.baglanti_satir_oku_sureli";
/// K-397 (RFC-0044 B4a): `"<ana>:<kapı>" adresine bağlanmayı dene` —
/// `ağ_hedefleri` `tcp://` hedefine giden TCP; başarıda görevin bağlantısı
/// olur → Metin sözlüğü Sonucu (`uzak_adres`, `uzak_kapı`, `yerel_kapı`).
pub const GIDEN_BAGLAN: &str = "ag.giden_baglan";
/// K-397: `bağlantıda TLS istemcisi başlatmayı dene "<ad>" ile` — giden
/// bağlantıda istemci STARTTLS → Metin sözlüğü Sonucu (`doğrulandı`, …).
pub const BAGLANTI_TLS_ISTEMCI: &str = "ag.baglanti_tls_istemci";
/// K-390 (RFC-0044 B1): `<ad> için MX kayıtlarını sorgulamayı dene` →
/// kayıt başına satırlı Metin sözlük listesi Sonucu (C033 `tür` verili ret).
pub const DNS_MX: &str = "dns.mx_sorgula";
/// K-390: `<ad> için TXT kayıtlarını sorgulamayı dene`.
pub const DNS_TXT: &str = "dns.txt_sorgula";
/// K-390: `<ad> için adres kayıtlarını sorgulamayı dene` (A, sonra AAAA).
pub const DNS_ADRES: &str = "dns.adres_sorgula";
/// K-390: `<ad> için TLSA kayıtlarını sorgulamayı dene`.
pub const DNS_TLSA: &str = "dns.tlsa_sorgula";
/// K-390 (RFC-0044 B1b): `<ad> alanının kurumsal alanı` → Metin; proje
/// `kamu_son_ekleri` dosyasıyla RFC 7489 §3.2 (yoksa ya da ad kamu son ekiyse "").
pub const POSTA_KURUMSAL_ALAN: &str = "posta.kurumsal_alan";
/// K-390 (RFC-0044 B2b): `<b> dkim imzasını <veri> verisi için <p>
/// anahtarıyla <a> olarak doğrulamayı dene` → Metin sözlük Sonucu (C034).
pub const POSTA_DKIM_DOGRULA: &str = "posta.dkim_dogrula";
/// K-390: `<x> gövdesinin dkim gövde özeti` — Zee'nin kanonikleştirdiği
/// gövdenin LF→CRLF baytlarının SHA-256'sı, base64 (`bh=` biçimi); CR → C034.
pub const POSTA_DKIM_GOVDE_OZETI: &str = "posta.dkim_govde_ozeti";
/// K-397 Dilim 2 (RFC-0044 B3): `<veri> verisi için <ad> özel anahtarıyla
/// <a> olarak imzalamayı dene` → `b=` Metin Sonucu (C034); anahtar proje
/// `dkim_anahtarları` bildiriminden, girdi kâhin önlemiyle denetlenir.
pub const POSTA_DKIM_IMZALA: &str = "posta.dkim_imzala";
/// K-397 B3 (ZM-F046): `<yol> dosyasının <baş> baytından <kip> kipinde dkim
/// gövde özetini almayı dene` — gövdenin simple/relaxed kanonik akışlı `bh=`
/// özeti (≤ 32 MiB, LF ya da CRLF; LF'siz CR → C034); Metin oluşmaz.
/// `kriptografi` + dosyayı okuma (`veri:` yolunda `veri-okuma`).
pub const POSTA_DKIM_DOSYA_GOVDE_OZETI: &str = "posta.dkim_dosya_govde_ozeti";
/// K-210: `<alıcı> <konu> <gövde> ile ileti göndermeyi dene` — SMTP e-posta.
pub const EPOSTA_GONDER: &str = "iletisim.eposta_gonder";
/// K-210 eki: `<alıcı> <konu> <gövde> ile <bağlantı> üzerinden ileti göndermeyi dene` —
/// SMTP bağlantısı ortam değişkeninden değil değerden gelir (panelden ayarlanabilir SMTP).
pub const EPOSTA_GONDER_BAGLANTILI: &str = "iletisim.eposta_gonder_baglantili";
/// K-232: alıcı/konu/plain/HTML/Reply-To/ek yolları/bağlantı ile ekli ileti.
pub const EPOSTA_MIME_GONDER: &str = "iletisim.eposta_mime_gonder";
/// K-210: `<kaynak> görselini <hedef> dosyasına <N> piksele küçültmeyi dene`.
pub const GORSEL_KUCULT: &str = "gorsel.kucult";
/// K-210 eki 3: `<metin>in gizli şifrelenmişi` — XChaCha20-Poly1305, anahtar `gizli_anahtar_değişkeni`.
pub const GIZLI_SIFRELE: &str = "guvenlik.gizli_sifrele";
/// K-210 eki 3: `<şifreli> gizli çözmeyi dene` → Metin Sonucu.
pub const GIZLI_COZ: &str = "guvenlik.gizli_coz";

pub use crate::yetkinlik::Yetkinlik;

/// K-397: o anki bağlantı üzerinde çalışan kalıplar. Tabloda `ağ-dinleyici`
/// görünürler ama derleme denetimi `ağ-dinleyici` **ya da** `ağ` ile
/// karşılanır: bağlantı ya dinleyiciden kabul edilir ya da `ağ` +
/// `ağ_hedefleri` ile açılır; gerçek kapı bağlantıyı açan adımdır.
pub fn baglanti_islemi_mi(kimlik: &str) -> bool {
    matches!(
        kimlik,
        BAGLANTI_SATIR_OKU
            | BAGLANTI_SATIR_YAZ
            | BAGLANTI_TLS_BASLAT
            | BAGLANTI_TLS_ISTEMCI
            | BAGLANTI_NOKTA_BLOGU
            | BAGLANTI_BAYT_BLOGU
            | BAGLANTI_BAYT_OKU
            | BAGLANTI_PARCA_YAZ
            | BAGLANTI_DOSYA_AKTAR
            | BAGLANTI_BASE64_AKTAR
            | BAGLANTI_BASE64_ARALIK_AKTAR
            | BAGLANTI_SATIR_OKU_SURELI
    )
}

/// Intrinsic imzalarında kullanılabilen çekirdek türlerin kapalı kümesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicTuru {
    Metin,
    TamSayi,
    Mantiksal,
    AgYaniti,
    MetinSozluk,
    MetinSozlukSonucu,
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
const SAYI: &[IntrinsicTuru] = &[IntrinsicTuru::TamSayi];
const METIN_VE_SAYI: &[IntrinsicTuru] = &[IntrinsicTuru::Metin, IntrinsicTuru::TamSayi];
const METIN_VE_IKI_SAYI: &[IntrinsicTuru] = &[
    IntrinsicTuru::Metin,
    IntrinsicTuru::TamSayi,
    IntrinsicTuru::TamSayi,
];
const METIN_VE_METIN_LISTESI: &[IntrinsicTuru] =
    &[IntrinsicTuru::Metin, IntrinsicTuru::MetinListesi];
const METIN_SAYI_METIN: &[IntrinsicTuru] = &[
    IntrinsicTuru::Metin,
    IntrinsicTuru::TamSayi,
    IntrinsicTuru::Metin,
];
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
        kimlik: POSTA_KURUMSAL_ALAN,
        yetkinlik: Yetkinlik::Dns,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Kurumsal alanı bulunacak alan adı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: POSTA_DKIM_DOGRULA,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: DORT_METIN,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "DKIM imzası, veri, açık anahtar ve algoritma Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: POSTA_DKIM_IMZALA,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: UC_METIN,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "DKIM imza girdisi, özel anahtar adı ve algoritma Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: POSTA_DKIM_DOSYA_GOVDE_OZETI,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: METIN_SAYI_METIN,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "DKIM gövde özetinde dosya yolu ve kip Metin, gövde başı TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: POSTA_DKIM_GOVDE_OZETI,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "DKIM gövde özeti alınacak gövde Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DNS_MX,
        yetkinlik: Yetkinlik::Dns,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSozlukListesiSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "MX sorgusunun alan adı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DNS_TXT,
        yetkinlik: Yetkinlik::Dns,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSozlukListesiSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "TXT sorgusunun alan adı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DNS_ADRES,
        yetkinlik: Yetkinlik::Dns,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSozlukListesiSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Adres sorgusunun alan adı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DNS_TLSA,
        yetkinlik: Yetkinlik::Dns,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSozlukListesiSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "TLSA sorgusunun alan adı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_SATIR_OKU,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Bağlantıdan satır okuma argüman almaz.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_SATIR_YAZ,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Bağlantıya yazılacak satır Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_TLS_BASLAT,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Bağlantıda TLS başlatma argüman almaz.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_NOKTA_BLOGU,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Bağlantıdan nokta bloğu alma argüman almaz.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_BAYT_BLOGU,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: SAYI,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Dosyaya alınacak bayt sayısı TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_BAYT_OKU,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: SAYI,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Okunacak bayt sayısı TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_PARCA_YAZ,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Bağlantıya yazılacak parça Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_DOSYA_AKTAR,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Bağlantıya aktarılacak dosya yolu Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_BASE64_AKTAR,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Base64 satırlarıyla aktarılacak dosya yolu Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_BASE64_ARALIK_AKTAR,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: METIN_VE_IKI_SAYI,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Aktarılacak dosya yolu Metin, konum ve bayt sayısı TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_SATIR_OKU_SURELI,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: SAYI,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Okuma süresi saniye cinsinden TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: GIDEN_BAGLAN,
        yetkinlik: Yetkinlik::Ag,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Bağlanılacak adres `\"<ana>:<kapı>\"` biçiminde Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: BAGLANTI_TLS_ISTEMCI,
        yetkinlik: Yetkinlik::AgDinleyici,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "TLS istemcisinin sunucu adı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: EPOSTA_MIME_GONDER,
        yetkinlik: Yetkinlik::Ag,
        arguman_turleri: &[IntrinsicTuru::Metin, IntrinsicTuru::Metin, IntrinsicTuru::Metin, IntrinsicTuru::Metin, IntrinsicTuru::Metin, IntrinsicTuru::MetinListesi, IntrinsicTuru::Metin],
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "Ekli iletinin alıcısı, konusu, gövdeleri, yanıt adresi ve bağlantısı Metin; ekleri Metin listesi olmalı.",
    },
    IntrinsicTanimi {
        kimlik: HTTP_JSON, yetkinlik: Yetkinlik::Ag, arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu, etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "HTTP JSON yanıtı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: HTTP_ISTEK,
        yetkinlik: Yetkinlik::Ag,
        arguman_turleri: &[IntrinsicTuru::Metin, IntrinsicTuru::Metin, IntrinsicTuru::MetinSozluk, IntrinsicTuru::Metin, IntrinsicTuru::TamSayi],
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::GeriAlinamaz,
        tur_hatasi: "HTTP adresi, yöntemi ve gövdesi Metin; başlıkları Metin sözlüğü, zaman aşımı TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: HTTP_GIZLI_ORTAM,
        yetkinlik: Yetkinlik::Ag,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "HTTP gizli ortam değişkeninin adı Metin olmalı.",
    },
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
        kimlik: WEB_HAM_GOVDE,
        yetkinlik: Yetkinlik::AgSunucusu,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "İsteğin ham gövdesi argüman almaz.",
    },
    IntrinsicTanimi {
        kimlik: WEB_ISTEK_DERINLIGI,
        yetkinlik: Yetkinlik::AgSunucusu,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::Metin,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "İsteğin Depth başlığı argüman almaz.",
    },
    IntrinsicTanimi {
        kimlik: WEB_ISTEK_KOSULU,
        yetkinlik: Yetkinlik::AgSunucusu,
        arguman_turleri: METIN,
        donus_turu: IntrinsicTuru::MetinListesi,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "Koşul başlığının adı Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: WEB_BIRINCIL_SUREC,
        yetkinlik: Yetkinlik::AgSunucusu,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::Mantiksal,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "Birincil süreç argüman almaz.",
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
        kimlik: UYGULAMA_PAROLASI,
        yetkinlik: Yetkinlik::Kriptografi,
        arguman_turleri: YOK,
        donus_turu: IntrinsicTuru::MetinSozluk,
        etki: IntrinsicEtkisi::Saf,
        tur_hatasi: "Uygulama parolası argüman almaz.",
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
        kimlik: WEB_DOSYA_YANITI,
        yetkinlik: Yetkinlik::DosyaOkuma,
        arguman_turleri: UC_METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::WebAdaptoru,
        tur_hatasi: "Dosya yolu, içerik türü ve dosya adı Metin olmalı.",
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
        kimlik: DOSYA_ILK_BAYTLAR,
        yetkinlik: Yetkinlik::DosyaOkuma,
        arguman_turleri: METIN_VE_SAYI,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Okunacak dosya yolu Metin, bayt sayısı TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_ARALIK_METIN,
        yetkinlik: Yetkinlik::DosyaOkuma,
        arguman_turleri: METIN_VE_IKI_SAYI,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Okunacak dosya yolu Metin, konum ve bayt sayısı TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_ARALIK_BASE64,
        yetkinlik: Yetkinlik::DosyaOkuma,
        arguman_turleri: METIN_VE_IKI_SAYI,
        donus_turu: IntrinsicTuru::MetinSonucu,
        etki: IntrinsicEtkisi::DisOkuma,
        tur_hatasi: "Okunacak dosya yolu Metin, konum ve bayt sayısı TamSayı olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_BASE64_YAZ,
        yetkinlik: Yetkinlik::DosyaYazma,
        arguman_turleri: IKI_METIN,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "Çözülecek içerik ve hedef dosya yolu Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_BASE64_EKLE,
        yetkinlik: Yetkinlik::DosyaYazma,
        arguman_turleri: IKI_METIN,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "Çözülecek içerik ve hedef dosya yolu Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_QP_YAZ,
        yetkinlik: Yetkinlik::DosyaYazma,
        arguman_turleri: IKI_METIN,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "Çözülecek içerik ve hedef dosya yolu Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_QP_EKLE,
        yetkinlik: Yetkinlik::DosyaYazma,
        arguman_turleri: IKI_METIN,
        donus_turu: IntrinsicTuru::MetinSozlukSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "Çözülecek içerik ve hedef dosya yolu Metin olmalı.",
    },
    IntrinsicTanimi {
        kimlik: DOSYA_BASE64_SATIRLARI_EKLE,
        yetkinlik: Yetkinlik::DosyaYazma,
        arguman_turleri: IKI_METIN,
        donus_turu: IntrinsicTuru::TamSayiSonucu,
        etki: IntrinsicEtkisi::DisYazma,
        tur_hatasi: "Kaynak ve hedef dosya yolları Metin olmalı.",
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
