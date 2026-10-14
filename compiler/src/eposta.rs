//! K-210: bağımlılıksız SMTP istemcisi (RFC 5321) — implicit TLS (`smtps://`,
//! 465) ve STARTTLS (`smtp://`, 587). TLS `native-tls` ile; düz metin yalnız
//! loopback hedefte kabul edilir. Kimlik doğrulama AUTH PLAIN/LOGIN. İleti
//! `text/plain; charset=utf-8`, konu RFC 2047 UTF-8 Base64. Gönderim geri
//! alınamaz dış etkidir; eylem içinde değil rota/CLI adaptöründe kullanılır.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// `smtps://kullanici:parola@sunucu:465?from=info@alan&name=Ad` biçimindeki
/// bağlantı URL'sinden çözülen gönderim ayarı. Sır kaynakta değil ortam
/// değişkenindedir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpostaAyari {
    pub implicit_tls: bool,
    pub konak: String,
    pub kapi: u16,
    pub kullanici: String,
    pub parola: String,
    pub gonderen: String,
    pub gonderen_adi: String,
}

fn yuzde_coz(metin: &str) -> Result<String, String> {
    let baytlar = metin.as_bytes();
    let mut cikti = Vec::with_capacity(baytlar.len());
    let mut i = 0;
    while i < baytlar.len() {
        if baytlar[i] == b'%' {
            let hex = baytlar.get(i + 1..i + 3).ok_or("eksik yüzde kaçışı")?;
            let deger = u8::from_str_radix(
                std::str::from_utf8(hex).map_err(|_| "bozuk yüzde kaçışı")?,
                16,
            )
            .map_err(|_| "bozuk yüzde kaçışı")?;
            cikti.push(deger);
            i += 3;
        } else {
            cikti.push(baytlar[i]);
            i += 1;
        }
    }
    String::from_utf8(cikti).map_err(|_| "yüzde kaçışı geçerli UTF-8 vermedi".into())
}

impl EpostaAyari {
    pub fn ayristir(url: &str) -> Result<Self, String> {
        let (sema, kalan) = url
            .split_once("://")
            .ok_or("e-posta bağlantısı smtp:// ya da smtps:// ile başlamalı")?;
        let implicit_tls = match sema {
            "smtps" => true,
            "smtp" => false,
            _ => return Err("e-posta bağlantı şeması smtp ya da smtps olmalı".into()),
        };
        let (otorite, sorgu) = kalan.split_once('?').unwrap_or((kalan, ""));
        let (otorite, _) = otorite.split_once('/').unwrap_or((otorite, ""));
        let (kimlik, konak_kapi) = otorite
            .rsplit_once('@')
            .ok_or("e-posta bağlantısı kullanıcı:parola@sunucu taşımalı")?;
        let (kullanici, parola) = kimlik
            .split_once(':')
            .ok_or("e-posta bağlantısı kullanıcı:parola taşımalı")?;
        let (konak, kapi) = match konak_kapi.rsplit_once(':') {
            Some((k, p)) if !k.contains(']') || k.ends_with(']') => (
                k.trim_matches(|c| c == '[' || c == ']').to_string(),
                p.parse::<u16>()
                    .map_err(|_| "e-posta sunucu portu geçersiz")?,
            ),
            _ => (
                konak_kapi
                    .trim_matches(|c| c == '[' || c == ']')
                    .to_string(),
                if implicit_tls { 465 } else { 587 },
            ),
        };
        if konak.is_empty() {
            return Err("e-posta sunucu adı boş".into());
        }
        let mut gonderen = String::new();
        let mut gonderen_adi = String::new();
        for cift in sorgu.split('&').filter(|c| !c.is_empty()) {
            let (ad, deger) = cift.split_once('=').unwrap_or((cift, ""));
            match ad {
                "from" => gonderen = yuzde_coz(deger)?,
                "name" => gonderen_adi = yuzde_coz(&deger.replace('+', " "))?,
                _ => return Err(format!("e-posta bağlantısında bilinmeyen alan: {ad}")),
            }
        }
        if !adres_gecerli(&gonderen) {
            return Err("e-posta bağlantısı ?from=gonderen@alan taşımalı".into());
        }
        Ok(Self {
            implicit_tls,
            konak,
            kapi,
            kullanici: yuzde_coz(kullanici)?,
            parola: yuzde_coz(parola)?,
            gonderen,
            gonderen_adi,
        })
    }
}

/// Basit ama fail-closed adres denetimi: tek `@`, boşluk/denetim/CRLF yok,
/// `<>` yok, 254 bayt sınırı.
pub fn adres_gecerli(adres: &str) -> bool {
    let baytlar = adres.as_bytes();
    if baytlar.is_empty() || baytlar.len() > 254 {
        return false;
    }
    if baytlar.iter().any(|b| {
        b.is_ascii_control() || matches!(b, b' ' | b'<' | b'>' | b',' | b';' | b'"' | b'\\')
    }) {
        return false;
    }
    let Some((yerel, alan)) = adres.rsplit_once('@') else {
        return false;
    };
    !yerel.is_empty() && alan.contains('.') && !alan.starts_with('.') && !alan.ends_with('.')
}

fn base64(baytlar: &[u8]) -> String {
    const ALFABE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut cikti = String::with_capacity(baytlar.len().div_ceil(3) * 4);
    for parca in baytlar.chunks(3) {
        let b = [
            parca[0],
            *parca.get(1).unwrap_or(&0),
            *parca.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        cikti.push(ALFABE[(n >> 18) as usize & 63] as char);
        cikti.push(ALFABE[(n >> 12) as usize & 63] as char);
        cikti.push(if parca.len() > 1 {
            ALFABE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        cikti.push(if parca.len() > 2 {
            ALFABE[n as usize & 63] as char
        } else {
            '='
        });
    }
    cikti
}

/// RFC 2047 UTF-8/B kodlu başlık sözcüğü (76 karakterlik parçalar).
fn baslik_kodla(metin: &str) -> String {
    if metin.is_ascii() && !metin.bytes().any(|b| b.is_ascii_control()) {
        return metin.to_string();
    }
    let mut parcalar = Vec::new();
    let mut tampon = Vec::new();
    for karakter in metin.chars() {
        let mut gecici = [0u8; 4];
        let baytlar = karakter.encode_utf8(&mut gecici).as_bytes();
        if tampon.len() + baytlar.len() > 45 {
            parcalar.push(format!("=?UTF-8?B?{}?=", base64(&tampon)));
            tampon.clear();
        }
        tampon.extend_from_slice(baytlar);
    }
    if !tampon.is_empty() {
        parcalar.push(format!("=?UTF-8?B?{}?=", base64(&tampon)));
    }
    parcalar.join("\r\n ")
}

/// Gövdeyi 76 sütunlu Base64 satırlarına böler (CRLF).
fn govde_kodla(govde: &str) -> String {
    let b64 = base64(govde.as_bytes());
    let mut cikti = String::with_capacity(b64.len() + b64.len() / 76 * 2 + 2);
    for parca in b64.as_bytes().chunks(76) {
        if let Ok(satir) = std::str::from_utf8(parca) {
            cikti.push_str(satir);
        }
        cikti.push_str("\r\n");
    }
    cikti
}

/// RFC 2822 tarih başlığı (UTC).
pub fn rfc2822_tarih(unix_saniye: i64) -> String {
    let gunler = unix_saniye.div_euclid(86_400);
    let kalan = unix_saniye.rem_euclid(86_400);
    let (yil, ay, gun) = crate::zaman::gunlerden_tarih_utc(gunler);
    const GUNLER: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    const AYLAR: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} +0000",
        GUNLER[gunler.rem_euclid(7) as usize],
        gun,
        AYLAR[(ay.clamp(1, 12) - 1) as usize],
        yil,
        kalan / 3600,
        (kalan % 3600) / 60,
        kalan % 60
    )
}

/// RFC 5322 ileti; Message-ID gönderen alanı ve verilen kimlikle kurulur.
/// Gönderilecek tek ileti: alıcı, konu, gövde ve Message-ID/Date parçaları.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ileti<'a> {
    pub alici: &'a str,
    pub konu: &'a str,
    pub govde: &'a str,
    pub kimlik: &'a str,
    pub tarih: &'a str,
}

/// K-232: MIME metadata ve host tarafından sınırlandırılan ek yolları.
/// Native olmayan adaptör bu gönderimi bilinçli olarak desteklemez.
pub struct MimeGonderim<'a> {
    pub html: &'a str,
    pub yanit_adresi: &'a str,
    pub ekler: &'a [String],
    pub zaman_asimi_ms: i64,
}

pub fn ileti_kur(ayar: &EpostaAyari, ileti: &Ileti<'_>) -> String {
    let Ileti {
        alici,
        konu,
        govde,
        kimlik,
        tarih,
    } = *ileti;
    let gonderen = if ayar.gonderen_adi.is_empty() {
        ayar.gonderen.clone()
    } else {
        format!("{} <{}>", baslik_kodla(&ayar.gonderen_adi), ayar.gonderen)
    };
    let alan = ayar
        .gonderen
        .rsplit_once('@')
        .map(|(_, a)| a)
        .unwrap_or("localhost");
    format!(
        "From: {gonderen}\r\nTo: {alici}\r\nSubject: {}\r\nDate: {tarih}\r\nMessage-ID: <{kimlik}@{alan}>\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\n{}",
        baslik_kodla(konu),
        govde_kodla(govde)
    )
}

#[derive(Debug)]
struct SonTarihliTcp {
    tcp: TcpStream,
    son: Instant,
}
impl SonTarihliTcp {
    fn kalan(&self) -> io::Result<Duration> {
        self.son
            .checked_duration_since(Instant::now())
            .filter(|sure| !sure.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "SMTP son tarihi doldu"))
    }
}
impl Read for SonTarihliTcp {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        self.tcp.set_read_timeout(Some(self.kalan()?))?;
        self.tcp.read(b)
    }
}
impl Write for SonTarihliTcp {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.tcp.set_write_timeout(Some(self.kalan()?))?;
        self.tcp.write(b)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.kalan()?;
        self.tcp.flush()
    }
}

trait Akis: Read + Write {}
impl<T: Read + Write> Akis for T {}

struct Oturum {
    okuyucu: BufReader<Box<dyn Akis>>,
}

impl Oturum {
    fn yanit(&mut self) -> Result<(u16, String), String> {
        const SINIR: usize = 16 * 1024;
        let mut satirlar = String::with_capacity(SINIR + 1);
        loop {
            let baslangic = satirlar.len();
            // Tek taşma baytı, LF/EOF gelmeden ve UTF-8 hata dönüşünden önce
            // sınırı kanıtlar. BufReader'ın sabit ön okuması bundan ayrıdır.
            let mut sinirli = (&mut self.okuyucu).take((SINIR - baslangic + 1) as u64);
            let okunan = sinirli.read_line(&mut satirlar);
            if sinirli.limit() == 0 {
                return Err("SMTP yanıtı 16 KiB sınırını aşıyor".into());
            }
            let okunan = okunan.map_err(|hata| format!("SMTP yanıtı okunamadı: {hata}"))?;
            if okunan == 0 {
                return Err("SMTP sunucusu bağlantıyı kapattı".into());
            }
            let satir = &satirlar[baslangic..];
            if satir.len() >= 4 && satir.as_bytes()[3] == b' ' || satir.len() < 4 {
                break;
            }
        }
        let kod = satirlar
            .get(..3)
            .and_then(|k| k.parse::<u16>().ok())
            .ok_or_else(|| format!("SMTP yanıtı çözülemedi: {}", satirlar.trim()))?;
        Ok((kod, satirlar))
    }

    fn komut(&mut self, komut: &str, beklenen: &[u16]) -> Result<String, String> {
        let akis = self.okuyucu.get_mut();
        akis.write_all(komut.as_bytes())
            .and_then(|_| akis.write_all(b"\r\n"))
            .and_then(|_| akis.flush())
            .map_err(|hata| format!("SMTP komutu gönderilemedi: {hata}"))?;
        let (kod, metin) = self.yanit()?;
        if !beklenen.contains(&kod) {
            let gizli = komut.split_whitespace().next().unwrap_or("?");
            return Err(format!(
                "SMTP {gizli} {kod}: {}",
                metin.lines().next().unwrap_or("").trim()
            ));
        }
        Ok(metin)
    }
}

fn adres_coz(
    konak: &str,
    kapi: u16,
    ip_denetimi: &dyn Fn(IpAddr) -> Result<(), String>,
) -> Result<SocketAddr, String> {
    let adresler: Vec<SocketAddr> = (konak, kapi)
        .to_socket_addrs()
        .map_err(|hata| format!("SMTP sunucusu çözülemedi: {hata}"))?
        .collect();
    if adresler.is_empty() {
        return Err("SMTP sunucusu için adres bulunamadı".into());
    }
    for adres in &adresler {
        ip_denetimi(adres.ip())?;
    }
    Ok(adresler[0])
}

/// İletiyi gönderir. `ip_denetimi` çözülen her IP için yetkinlik politikasını
/// uygular (K-127 SSRF kapısı). Düz `smtp://` yalnız loopback hedefte
/// STARTTLS'siz kabul edilir; başka her hedefte STARTTLS zorunludur.
#[cfg(not(target_arch = "wasm32"))]
pub fn gonder(
    ayar: &EpostaAyari,
    ileti: &Ileti<'_>,
    zaman_asimi: Duration,
    ip_denetimi: &dyn Fn(IpAddr) -> Result<(), String>,
) -> Result<(), String> {
    gonder_ic(ayar, ileti, zaman_asimi, ip_denetimi).map_err(|_| GONDERIM_HATASI.into())
}

/// K-236: server echo, URL/credential ve adaptör ayrıntısı dışarı çıkmaz.
pub(crate) const GONDERIM_HATASI: &str = "E-posta gönderilemedi; ayar geçersiz veya gönderim sonucu belirsiz; ayrıntılar gizlendi; otomatik tekrar yok";

#[cfg(not(target_arch = "wasm32"))]
fn gonder_ic(
    ayar: &EpostaAyari,
    ileti: &Ileti<'_>,
    zaman_asimi: Duration,
    ip_denetimi: &dyn Fn(IpAddr) -> Result<(), String>,
) -> Result<(), String> {
    if zaman_asimi.is_zero() {
        return Err("SMTP bütçesi sıfır".into());
    }
    let son = Instant::now()
        .checked_add(zaman_asimi)
        .ok_or("SMTP bütçesi geçersiz")?;
    let (alici, konu, govde) = (ileti.alici, ileti.konu, ileti.govde);
    if !adres_gecerli(alici) {
        return Err("alıcı e-posta adresi geçersiz".into());
    }
    if konu.bytes().any(|b| b == b'\r' || b == b'\n') {
        return Err("e-posta konusu satır sonu taşıyamaz".into());
    }
    if govde.len() > 512 * 1024 {
        return Err("e-posta gövdesi 512 KiB sınırını aşıyor".into());
    }
    let adres = adres_coz(&ayar.konak, ayar.kapi, ip_denetimi)?;
    let kalan = son
        .checked_duration_since(Instant::now())
        .filter(|sure| !sure.is_zero())
        .ok_or("SMTP son tarihi doldu")?;
    let tcp = TcpStream::connect_timeout(&adres, kalan)
        .map_err(|hata| format!("SMTP sunucusuna bağlanılamadı: {hata}"))?;
    let tcp = SonTarihliTcp { tcp, son };
    let loopback = adres.ip().is_loopback();
    let tls = || {
        native_tls::TlsConnector::builder()
            .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
            .build()
            .map_err(|hata| format!("TLS kurulamadı: {hata}"))
    };
    let mut oturum = if ayar.implicit_tls {
        let akis = tls()?
            .connect(&ayar.konak, tcp)
            .map_err(|hata| format!("SMTPS TLS el sıkışması başarısız: {hata}"))?;
        Oturum {
            okuyucu: BufReader::new(Box::new(akis)),
        }
    } else {
        Oturum {
            okuyucu: BufReader::new(Box::new(tcp)),
        }
    };
    let (kod, _) = oturum.yanit()?;
    if kod != 220 {
        return Err(format!("SMTP karşılama {kod}"));
    }
    let ehlo = oturum.komut("EHLO zee.invalid", &[250])?;
    if !ayar.implicit_tls {
        if ehlo.to_ascii_uppercase().contains("STARTTLS") {
            oturum.komut("STARTTLS", &[220])?;
            let Oturum { okuyucu } = oturum;
            let ic = okuyucu.into_inner();
            // Box<dyn Akis> içindeki TcpStream'i geri alamayız; STARTTLS için
            // yeni bağlantı yerine ham akışı sarmalayacak yapı gerekir.
            let akis = tls()?
                .connect(&ayar.konak, HamAkis(ic))
                .map_err(|hata| format!("STARTTLS el sıkışması başarısız: {hata}"))?;
            oturum = Oturum {
                okuyucu: BufReader::new(Box::new(akis)),
            };
            oturum.komut("EHLO zee.invalid", &[250])?;
        } else if !loopback {
            return Err(
                "SMTP sunucusu STARTTLS sunmuyor; düz metin yalnız loopback hedefte kabul edilir"
                    .into(),
            );
        }
    }
    if !ayar.kullanici.is_empty() {
        let plain = base64(format!("\0{}\0{}", ayar.kullanici, ayar.parola).as_bytes());
        oturum.komut(&format!("AUTH PLAIN {plain}"), &[235])?;
    }
    oturum.komut(&format!("MAIL FROM:<{}>", ayar.gonderen), &[250])?;
    oturum.komut(&format!("RCPT TO:<{alici}>"), &[250, 251])?;
    oturum.komut("DATA", &[354])?;
    let ileti_metni = ileti_kur(ayar, ileti);
    let mut veri = String::with_capacity(ileti_metni.len() + 8);
    for satir in ileti_metni.split("\r\n") {
        if satir.starts_with('.') {
            veri.push('.');
        }
        veri.push_str(satir);
        veri.push_str("\r\n");
    }
    veri.push('.');
    oturum.komut(&veri, &[250])?;
    oturum.komut("QUIT", &[221]).ok();
    Ok(())
}

/// STARTTLS öncesi düz akışı TLS'e devretmek için `Read + Write` sarmalayıcı.
struct HamAkis(Box<dyn Akis>);
impl std::fmt::Debug for HamAkis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HamAkis")
    }
}
impl Read for HamAkis {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}
impl Write for HamAkis {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

#[cfg(test)]
#[path = "eposta/testler.rs"]
mod tests;

// K-232 candidate: additive MIME implementation.
#[cfg(not(target_arch = "wasm32"))]
#[path = "eposta/mime.rs"]
pub mod mime;
