//! K-232: sınırlı MIME kodlayıcı ve native SMTP adaptörü.

use super::{base64, baslik_kodla, EpostaAyari, HamAkis, Ileti, Oturum, SonTarihliTcp};
use std::io::{BufReader, Read};
use std::net::{IpAddr, TcpStream};
use std::time::{Duration, Instant};
#[path = "mime/dosya.rs"]
mod dosya;
pub use dosya::dosyayi_guvenle_ac;

pub const MIME_PROFILI: &str = "zee-smtp-mime-1";
const SINIRLAR: crate::kaynak_sinirlari::EpostaSinirlari =
    crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.eposta();
pub const AZAMI_EK_ADEDI: usize = SINIRLAR.ek_adedi();
pub const AZAMI_EK_BAYTI: usize = SINIRLAR.ek_bayti();
pub const AZAMI_TOPLAM_EK_BAYTI: usize = SINIRLAR.toplam_ek_bayti();
pub const AZAMI_METIN_BAYTI: usize = SINIRLAR.metin_bayti();
pub const AZAMI_ILETI_BAYTI: usize = SINIRLAR.ileti_bayti();

/// SMTP eki türleri: `icerik_turu` kaydının SMTP adayı satırları (K-378).
/// Varsayılan küme (`zee-smtp-mime-1`, bildirimsiz proje) ilk dördüdür.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EkTuru {
    Pdf,
    Png,
    Jpeg,
    Webp,
    Txt,
    Csv,
    Ics,
    Docx,
    Xlsx,
    Pptx,
    Zip,
}

impl EkTuru {
    pub const TUMU: [EkTuru; 11] = [
        Self::Pdf,
        Self::Png,
        Self::Jpeg,
        Self::Webp,
        Self::Txt,
        Self::Csv,
        Self::Ics,
        Self::Docx,
        Self::Xlsx,
        Self::Pptx,
        Self::Zip,
    ];

    /// `icerik_turu` kaydındaki kanonik kısa ad.
    pub fn ad(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
            Self::Txt => "txt",
            Self::Csv => "csv",
            Self::Ics => "ics",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
            Self::Zip => "zip",
        }
    }

    fn kayit(self) -> Option<&'static crate::icerik_turu::IcerikTuru> {
        crate::icerik_turu::adla(self.ad()).filter(|tur| tur.smtp_aday)
    }

    fn mime(self) -> String {
        self.kayit()
            .map(crate::icerik_turu::baslik_degeri)
            .unwrap_or_else(|| "application/octet-stream".into())
    }

    fn gorsel(self) -> bool {
        matches!(self, Self::Png | Self::Jpeg | Self::Webp)
    }

    /// Ad (kanonik kısa ad) ile tür.
    pub fn adla(ad: &str) -> Option<Self> {
        Self::TUMU.into_iter().find(|tur| tur.ad() == ad)
    }

    /// Dosya adının son uzantısından aday tür (izin kararı vermez).
    pub fn uzantidan(ad: &str) -> Option<Self> {
        crate::icerik_turu::uzantidan(ad)
            .filter(|tur| tur.smtp_aday)
            .and_then(|tur| Self::adla(tur.ad))
    }

    /// Bildirimsiz projenin dört tür kuralı (`zee-smtp-mime-1`).
    pub fn dosya_adindan(ad: &str) -> Result<Self, String> {
        Self::dosya_adindan_izinle(ad, &EkIzni::default())
    }

    pub fn dosya_adindan_izinle(ad: &str, izin: &EkIzni) -> Result<Self, String> {
        Self::uzantidan(ad)
            .filter(|tur| izin.izinli_mi(*tur))
            .ok_or_else(|| format!("MIME ek türü izinli değil; izinli türler: {}", izin.adlar()))
    }
}

/// K-378: projenin SMTP ek türü kümesi. Varsayılan dört türdür; proje
/// `e_posta_ek_türleri` bildirimi kümeyi tam olarak belirler (varsayılanlar
/// örtük eklenmez) ve yalnız kayıttaki SMTP adaylarından seçebilir.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EkIzni {
    turler: Vec<EkTuru>,
}

impl Default for EkIzni {
    fn default() -> Self {
        Self {
            turler: EkTuru::TUMU
                .into_iter()
                .filter(|tur| tur.kayit().is_some_and(|k| k.smtp_varsayilan))
                .collect(),
        }
    }
}

impl EkIzni {
    /// Bildirimdeki kanonik adları doğrular: boş liste, bilinmeyen ya da
    /// yinelenen ad reddedilir.
    pub fn adlardan(adlar: &[String]) -> Result<Self, String> {
        if adlar.is_empty() {
            return Err("ek türü listesi boş olamaz".into());
        }
        let mut turler = Vec::new();
        for ad in adlar {
            let tur = EkTuru::adla(ad).filter(|tur| tur.kayit().is_some()).ok_or_else(|| {
                format!("\"{ad}\" SMTP ek türü kaydında yok (makro, yürütülebilir ve betik türleri açılamaz)")
            })?;
            if turler.contains(&tur) {
                return Err(format!("\"{ad}\" ek türü birden çok kez yazıldı"));
            }
            turler.push(tur);
        }
        Ok(Self { turler })
    }

    pub fn izinli_mi(&self, tur: EkTuru) -> bool {
        self.turler.contains(&tur)
    }

    fn adlar(&self) -> String {
        self.turler
            .iter()
            .map(|tur| tur.ad())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Clone, Debug)]
pub struct EkDosya<'a> {
    pub ad: &'a str,
    pub tur: EkTuru,
    pub baytlar: &'a [u8],
}

#[derive(Clone, Default, Debug)]
pub struct MimeSecenekleri<'a> {
    pub yanit_adresi: Option<&'a str>,
    pub html: Option<&'a str>,
    pub ekler: &'a [EkDosya<'a>],
}

#[derive(Debug)]
pub struct YetkiliEk {
    pub ad: String,
    pub tur: EkTuru,
    pub baytlar: Vec<u8>,
}

/// Bildirimsiz projenin dört tür kümesiyle [`ekleri_izinle_oku`].
pub fn ekleri_oku(
    yollar: &[String],
    yetkili_ac: &mut dyn FnMut(&str) -> Result<std::fs::File, String>,
) -> Result<Vec<YetkiliEk>, String> {
    ekleri_izinle_oku(yollar, &EkIzni::default(), yetkili_ac)
}

/// The host must grant DosyaOkuma and resolve its file boundary before opening.
/// Only read handles enter this bounded loader; it never creates/writes files.
/// K-378: tür projenin `izin` kümesinden seçilir; imzalı türlerde (OOXML/zip)
/// ilk baytlar, metin türlerinde UTF-8 strict denetlenir. Bu yalnız uzantı
/// sahteciliğini daraltır; içerik/antivirüs güvenliği değildir. Bütün
/// hatalar sır taşımayan sabit metinlerdir ve ağa çıkmadan döner.
pub fn ekleri_izinle_oku(
    yollar: &[String],
    izin: &EkIzni,
    yetkili_ac: &mut dyn FnMut(&str) -> Result<std::fs::File, String>,
) -> Result<Vec<YetkiliEk>, String> {
    if yollar.len() > AZAMI_EK_ADEDI {
        return Err("MIME ek sayısı 8 sınırını aşıyor".into());
    }
    let mut ekler = Vec::new();
    let mut toplam = 0usize;
    for yol in yollar {
        let ad = std::path::Path::new(yol)
            .file_name()
            .and_then(|ad| ad.to_str())
            .ok_or("MIME ek adı geçersiz")?;
        ek_adini_denetle(ad)?;
        let tur = EkTuru::dosya_adindan_izinle(ad, izin)?;
        let mut dosya = yetkili_ac(yol).map_err(|_| "MIME ek erişimi reddedildi")?;
        let meta = dosya.metadata().map_err(|_| "MIME ek bilgisi okunamadı")?;
        if !meta.is_file() || meta.len() > AZAMI_EK_BAYTI as u64 {
            return Err("MIME ek düzenli ve en fazla 8 MiB dosya olmalı".into());
        }
        let sinir = AZAMI_EK_BAYTI.min(AZAMI_TOPLAM_EK_BAYTI.saturating_sub(toplam));
        let mut baytlar = Vec::new();
        (&mut dosya)
            .take(sinir as u64 + 1)
            .read_to_end(&mut baytlar)
            .map_err(|_| "MIME ek okunamadı")?;
        if baytlar.len() > sinir {
            return Err("MIME ek byte sınırı aşıldı".into());
        }
        let sonra = dosya.metadata().map_err(|_| "MIME ek bilgisi okunamadı")?;
        if baytlar.len() as u64 != meta.len()
            || sonra.len() != meta.len()
            || sonra.modified().map_err(|_| "MIME ek tarihi okunamadı")?
                != meta.modified().map_err(|_| "MIME ek tarihi okunamadı")?
        {
            return Err("MIME ek okunurken değişti".into());
        }
        icerigi_denetle(tur, &baytlar)?;
        toplam += baytlar.len();
        ekler.push(YetkiliEk {
            ad: ad.into(),
            tur,
            baytlar,
        });
    }
    Ok(ekler)
}

/// K-378: imzalı türde ilk baytlar, metin türünde UTF-8 strict (NUL yok).
fn icerigi_denetle(tur: EkTuru, baytlar: &[u8]) -> Result<(), String> {
    let Some(kayit) = tur.kayit() else {
        return Err("MIME ek türü kayıtta yok".into());
    };
    if kayit.imza.is_some_and(|imza| !baytlar.starts_with(imza)) {
        return Err("MIME ek içeriği uzantısındaki türle uyuşmuyor".into());
    }
    if kayit.metin && (std::str::from_utf8(baytlar).is_err() || baytlar.contains(&0)) {
        return Err("MIME metin eki geçerli UTF-8 değil".into());
    }
    Ok(())
}

fn baslik_denetle(deger: &str, azami: usize) -> Result<(), String> {
    if deger.len() > azami || deger.chars().any(char::is_control) {
        return Err("MIME başlığı denetim karakteri veya boyut sınırı taşıyor".into());
    }
    Ok(())
}

fn adres_denetle(adres: &str) -> bool {
    // SMTPUTF8 and quoted/multiple-address envelopes are outside this candidate.
    adres.is_ascii()
        && adres.bytes().filter(|b| *b == b'@').count() == 1
        && super::adres_gecerli(adres)
        && adres
            .rsplit_once('@')
            .is_some_and(|(yerel, _)| yerel.len() <= 64)
}

pub fn ek_adini_denetle(ad: &str) -> Result<(), String> {
    baslik_denetle(ad, SINIRLAR.ad_bayti())?;
    if ad.is_empty() || matches!(ad, "." | "..") || ad.contains(['/', '\\']) {
        return Err("MIME ek adı tek, boş olmayan dosya adı olmalı".into());
    }
    Ok(())
}

/// RFC 8187 kodlaması web `Content-Disposition` ile ortaktır (K-378).
fn yuzde_kodla(ad: &str) -> String {
    crate::icerik_turu::rfc8187_kodla(ad)
}

fn bayt_govdesi(baytlar: &[u8]) -> String {
    let metin = base64(baytlar);
    let mut cikti = String::with_capacity(metin.len() + metin.len() / 76 * 2 + 2);
    for satir in metin.as_bytes().chunks(76) {
        cikti.extend(satir.iter().copied().map(char::from));
        cikti.push_str("\r\n");
    }
    cikti
}

fn metin_parcasi(cikti: &mut String, tur: &str, metin: &str) {
    cikti.push_str(&format!(
        "Content-Type: {tur}; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\n"
    ));
    let kanonik = metin
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    cikti.push_str(&bayt_govdesi(kanonik.as_bytes()));
}

fn govde_kur(cikti: &mut String, ileti: &Ileti<'_>, secenek: &MimeSecenekleri<'_>, sinir: &str) {
    if let Some(html) = secenek.html {
        cikti.push_str(&format!(
            "Content-Type: multipart/alternative; boundary=\"{sinir}\"\r\n\r\n--{sinir}\r\n"
        ));
        metin_parcasi(cikti, "text/plain", ileti.govde);
        cikti.push_str(&format!("--{sinir}\r\n"));
        metin_parcasi(cikti, "text/html", html);
        cikti.push_str(&format!("--{sinir}--\r\n"));
    } else {
        metin_parcasi(cikti, "text/plain", ileti.govde);
    }
}

/// HTML'nin `cid:<ad>` ile başvurduğu görsel ek iletiye gömülür (multipart/related, Content-ID <ad>).
/// Ad yalnız ASCII harf/rakam ve `.` `_` `-` taşıyorsa uygundur; görsel olmayan ve başvurulmayan ekler eklenti kalır.
fn gomulu_mu(ek: &EkDosya<'_>, html: Option<&str>) -> bool {
    ek.tur.gorsel()
        && !ek.ad.is_empty()
        && ek
            .ad
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        && html.is_some_and(|h| {
            let anahtar = format!("cid:{}", ek.ad);
            h.match_indices(&anahtar).any(|(i, _)| {
                // Tam ad: "cid:logo.png" başvurusu "cid:logo.png2" gibi daha uzun bir adla eşleşmez.
                !h[i + anahtar.len()..]
                    .bytes()
                    .next()
                    .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
            })
        })
}

fn ilgili_kur(
    cikti: &mut String,
    ileti: &Ileti<'_>,
    secenek: &MimeSecenekleri<'_>,
    gomulu: &[&EkDosya<'_>],
    related: &str,
    alternative: &str,
) {
    if gomulu.is_empty() {
        govde_kur(cikti, ileti, secenek, alternative);
        return;
    }
    cikti.push_str(&format!(
        "Content-Type: multipart/related; type=\"multipart/alternative\"; boundary=\"{related}\"\r\n\r\n--{related}\r\n"
    ));
    govde_kur(cikti, ileti, secenek, alternative);
    for ek in gomulu {
        cikti.push_str(&format!("--{related}\r\nContent-Type: {}\r\nContent-ID: <{}>\r\nContent-Disposition: inline;\r\n filename*=UTF-8''{}\r\nContent-Transfer-Encoding: base64\r\n\r\n", ek.tur.mime(), ek.ad, yuzde_kodla(ek.ad)));
        cikti.push_str(&bayt_govdesi(ek.baytlar));
    }
    cikti.push_str(&format!("--{related}--\r\n"));
}

/// Only already-authorized bytes enter the codec; this function opens no file.
pub fn ileti_kur(
    ayar: &EpostaAyari,
    ileti: &Ileti<'_>,
    secenek: &MimeSecenekleri<'_>,
) -> Result<String, String> {
    if !adres_denetle(ileti.alici) || !adres_denetle(&ayar.gonderen) {
        return Err("MIME alıcı veya gönderen adresi geçersiz".into());
    }
    if let Some(adres) = secenek.yanit_adresi {
        if !adres_denetle(adres) {
            return Err("MIME yanıt adresi geçersiz".into());
        }
    }
    baslik_denetle(ileti.konu, SINIRLAR.konu_bayti())?;
    baslik_denetle(&ayar.gonderen_adi, SINIRLAR.ad_bayti())?;
    baslik_denetle(ileti.tarih, SINIRLAR.tarih_bayti())?;
    if ileti.kimlik.is_empty()
        || ileti.kimlik.len() > SINIRLAR.kimlik_bayti()
        || !ileti
            .kimlik
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
    {
        return Err("MIME ileti kimliği geçersiz".into());
    }
    if ileti.govde.len() > AZAMI_METIN_BAYTI
        || secenek
            .html
            .is_some_and(|html| html.len() > AZAMI_METIN_BAYTI)
    {
        return Err("MIME metin veya HTML gövdesi 512 KiB sınırını aşıyor".into());
    }
    if secenek.ekler.len() > AZAMI_EK_ADEDI {
        return Err("MIME ek sayısı 8 sınırını aşıyor".into());
    }
    let mut toplam = 0usize;
    for ek in secenek.ekler {
        ek_adini_denetle(ek.ad)?;
        if EkTuru::uzantidan(ek.ad) != Some(ek.tur) {
            return Err("MIME ek adı ve türü uyuşmuyor".into());
        }
        if ek.baytlar.len() > AZAMI_EK_BAYTI {
            return Err("MIME tek ek 8 MiB sınırını aşıyor".into());
        }
        toplam = toplam
            .checked_add(ek.baytlar.len())
            .ok_or("MIME ek boyutu taştı")?;
        if toplam > AZAMI_TOPLAM_EK_BAYTI {
            return Err("MIME toplam ek 16 MiB sınırını aşıyor".into());
        }
    }
    if secenek.html.is_none() && secenek.ekler.is_empty() && secenek.yanit_adresi.is_none() {
        return Ok(super::ileti_kur(ayar, ileti));
    }
    let gonderen = if ayar.gonderen_adi.is_empty() {
        ayar.gonderen.clone()
    } else {
        let ad = if ayar.gonderen_adi.is_ascii() {
            format!(
                "\"{}\"",
                ayar.gonderen_adi.replace('\\', "\\\\").replace('"', "\\\"")
            )
        } else {
            baslik_kodla(&ayar.gonderen_adi)
        };
        format!("{ad} <{}>", ayar.gonderen)
    };
    let alan = ayar
        .gonderen
        .rsplit_once('@')
        .ok_or("MIME gönderen geçersiz")?
        .1;
    let mut cikti = format!("From: {gonderen}\r\nTo: {}\r\nSubject: {}\r\nDate: {}\r\nMessage-ID: <{}@{alan}>\r\nMIME-Version: 1.0\r\n", ileti.alici, baslik_kodla(ileti.konu), ileti.tarih, ileti.kimlik);
    if let Some(adres) = secenek.yanit_adresi {
        cikti.push_str(&format!("Reply-To: {adres}\r\n"));
    }
    // '-' never appears in base64 payload; these boundaries cannot collide with parts.
    let mixed = format!("zee-{}-mixed", ileti.kimlik);
    let alternative = format!("zee-{}-alt", ileti.kimlik);
    let related = format!("zee-{}-rel", ileti.kimlik);
    let (gomulu, ekli): (Vec<&EkDosya<'_>>, Vec<&EkDosya<'_>>) = secenek
        .ekler
        .iter()
        .partition(|ek| gomulu_mu(ek, secenek.html));
    if ekli.is_empty() {
        ilgili_kur(&mut cikti, ileti, secenek, &gomulu, &related, &alternative);
    } else {
        cikti.push_str(&format!(
            "Content-Type: multipart/mixed; boundary=\"{mixed}\"\r\n\r\n--{mixed}\r\n"
        ));
        ilgili_kur(&mut cikti, ileti, secenek, &gomulu, &related, &alternative);
        for ek in ekli {
            cikti.push_str(&format!("--{mixed}\r\nContent-Type: {}\r\nContent-Disposition: attachment;\r\n filename*=UTF-8''{}\r\nContent-Transfer-Encoding: base64\r\n\r\n", ek.tur.mime(), yuzde_kodla(ek.ad)));
            cikti.push_str(&bayt_govdesi(ek.baytlar));
        }
        cikti.push_str(&format!("--{mixed}--\r\n"));
    }
    if cikti.len() > AZAMI_ILETI_BAYTI {
        return Err("MIME kodlu ileti 24 MiB sınırını aşıyor".into());
    }
    if cikti.split("\r\n").any(|satir| satir.len() > 998) {
        return Err("MIME SMTP satırı 998 bayt sınırını aşıyor".into());
    }
    Ok(cikti)
}

/// Redacted, no retry. Deadline covers connect/SMTP/TLS IO; system DNS is not cancellable.
pub fn gonder(
    ayar: &EpostaAyari,
    ileti: &Ileti<'_>,
    secenek: &MimeSecenekleri<'_>,
    butce: Duration,
    ip_denetimi: &dyn Fn(IpAddr) -> Result<(), String>,
) -> Result<(), String> {
    let baslangic = Instant::now();
    let wire = ileti_kur(ayar, ileti, secenek)?;
    let tls = native_tls::TlsConnector::builder()
        .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
        .build()
        .map_err(|_| "SMTP MIME TLS hazırlanamadı")?;
    let kalan = butce
        .checked_sub(baslangic.elapsed())
        .ok_or("SMTP MIME son tarihi doldu")?;
    gonder_ic(ayar, ileti.alici, &wire, kalan, ip_denetimi, &tls).map_err(|_| {
        "SMTP MIME gönderimi başarısız; sır içeren ayrıntılar gizlendi; otomatik tekrar yok".into()
    })
}

fn gonder_ic(
    ayar: &EpostaAyari,
    alici: &str,
    wire: &str,
    butce: Duration,
    ip_denetimi: &dyn Fn(IpAddr) -> Result<(), String>,
    tls: &native_tls::TlsConnector,
) -> Result<(), String> {
    if butce.is_zero() {
        return Err("SMTP bütçesi sıfır".into());
    }
    let son = Instant::now()
        .checked_add(butce)
        .ok_or("SMTP bütçesi geçersiz")?;
    let _baglanti_izni = crate::kaynak_sinirlari::baglanti_izni_al()?;
    let adres = super::adres_coz(&ayar.konak, ayar.kapi, ip_denetimi)?;
    let kalan = son
        .checked_duration_since(Instant::now())
        .filter(|sure| !sure.is_zero())
        .ok_or("SMTP son tarihi doldu")?;
    let tcp = TcpStream::connect_timeout(&adres, kalan).map_err(|_| "SMTP bağlantısı başarısız")?;
    let tcp = SonTarihliTcp { tcp, son };
    let mut oturum = if ayar.implicit_tls {
        let akis = tls
            .connect(&ayar.konak, tcp)
            .map_err(|hata| format!("SMTPS TLS başarısız: {hata}"))?;
        Oturum {
            okuyucu: BufReader::new(Box::new(akis)),
        }
    } else {
        Oturum {
            okuyucu: BufReader::new(Box::new(tcp)),
        }
    };
    if oturum.yanit()?.0 != 220 {
        return Err("SMTP karşılama geçersiz".into());
    }
    let ehlo = oturum.komut("EHLO zee.invalid", &[250])?;
    if !ayar.implicit_tls {
        if ehlo.to_ascii_uppercase().contains("STARTTLS") {
            oturum.komut("STARTTLS", &[220])?;
            let Oturum { okuyucu } = oturum;
            if !okuyucu.buffer().is_empty() {
                return Err("STARTTLS öncesi fazla açık veri".into());
            }
            let akis = tls
                .connect(&ayar.konak, HamAkis(okuyucu.into_inner()))
                .map_err(|hata| format!("STARTTLS başarısız: {hata}"))?;
            oturum = Oturum {
                okuyucu: BufReader::new(Box::new(akis)),
            };
            oturum.komut("EHLO zee.invalid", &[250])?;
        } else if !adres.ip().is_loopback() {
            return Err("STARTTLS zorunlu".into());
        }
    }
    if !ayar.kullanici.is_empty() {
        let plain = base64(format!("\0{}\0{}", ayar.kullanici, ayar.parola).as_bytes());
        oturum.komut(&format!("AUTH PLAIN {plain}"), &[235])?;
    }
    oturum.komut(&format!("MAIL FROM:<{}>", ayar.gonderen), &[250])?;
    oturum.komut(&format!("RCPT TO:<{alici}>"), &[250, 251])?;
    oturum.komut("DATA", &[354])?;
    let veri = smtp_verisi(wire);
    oturum.komut(&veri, &[250])?;
    oturum.komut("QUIT", &[221]).ok();
    Ok(())
}

fn smtp_verisi(wire: &str) -> String {
    let mut veri = String::with_capacity(wire.len() + 8);
    for satir in wire.split("\r\n") {
        if satir.starts_with('.') {
            veri.push('.');
        }
        veri.push_str(satir);
        veri.push_str("\r\n");
    }
    veri.push('.');
    veri
}

#[cfg(test)]
#[path = "mime/tests.rs"]
mod tests;
