//! K-362 / GB-109: zee native HTTP istek ayrıştırıcısı için bağımsız kâhin.
//!
//! Kâhin `httparse` (yalnız dev/fuzz bağımlılığı; derleyici ikilisine girmez)
//! ile RFC 9112 §6.3 gövde çerçevesidir. Sözleşme tek yönlüdür:
//!
//! * zee kabul ederse kâhin de exact aynı başlık sonunu, request-line'ı,
//!   başlık listesini ve Content-Length gövdesini görmelidir; aksi zee
//!   hatasıdır (kâhinin reddettiğini zee kabul edemez).
//! * zee reddedip kâhin kabul ederse fark [`BilincliRet`] kataloğundaki
//!   belgelenmiş katı kurallardan en az birine dayanmalıdır; dayanmıyorsa
//!   belgelenmemiş ayrılıktır ve test/fuzz düşer.
//!
//! Aynı dosya `tests/http_kahini_testi.rs` ve `fuzz/fuzz_targets/http_istegi.rs`
//! tarafından `#[path]` ile kullanılır.

use dil::http_istegi::{baslik_sonunu_bul, HttpIstegi};

/// Kâhinin kabul ettiği ama zee'nin bilerek reddettiği sınıflar
/// (`docs/urunlesme-acik-incelemeler.md` K-362 tablosu).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BilincliRet {
    /// RFC 9112 §2.2 robustness: istek satırından önce boş satır.
    OnBosSatir,
    /// Yalın LF satır sonu (zee yalnız CRLF; kâhin LF'yi kabul eder).
    YalinLf,
    /// origin-form dışı hedef (`*`, absolute-form) ya da `#`/ASCII dışı bayt.
    OriginFormuDisi,
    /// Başlık değerinde obs-text (0x80-0xFF); zee yalnız görünür ASCII + SP/HTAB.
    AsciiDisiBaslikDegeri,
    /// Transfer-Encoding hiç desteklenmez (TE+CL kaçakçılığı dahil).
    TransferEncoding,
    /// Birden çok Content-Length (eşit değerli olsa bile).
    CokluContentLength,
    /// Content-Length boş, rakam dışı ya da `usize` taşması.
    GecersizContentLength,
    /// Başlık sonrası bayt sayısı Content-Length'e eşit değil (pipelining yok).
    GovdeCercevesi,
    /// Gövde NUL ya da geçersiz UTF-8 taşıyor.
    GovdeMetni,
}

/// Bir girdinin iki ayrıştırıcıdaki sonucu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Karar {
    IkisiKabul,
    IkisiRet,
    ZeeDahaKati(Vec<BilincliRet>),
}

struct KahinIstegi<'a> {
    baslik_sonu: usize,
    yontem: &'a str,
    hedef: &'a str,
    surum: u8,
    basliklar: Vec<(&'a str, &'a [u8])>,
}

fn kahin(baytlar: &[u8]) -> Option<KahinIstegi<'_>> {
    // En kısa başlık satırı `a:\n` 3 bayttır; dizi her girdiye yeter, kâhin
    // `TooManyHeaders` ile kapasite yüzünden reddetmez.
    let mut basliklar = vec![httparse::EMPTY_HEADER; baytlar.len() / 3 + 1];
    let mut istek = httparse::Request::new(&mut basliklar);
    let Ok(httparse::Status::Complete(baslik_sonu)) = istek.parse(baytlar) else {
        return None;
    };
    Some(KahinIstegi {
        baslik_sonu,
        yontem: istek.method?,
        hedef: istek.path?,
        surum: istek.version?,
        basliklar: istek
            .headers
            .iter()
            .map(|baslik| (baslik.name, kirp(baslik.value)))
            .collect(),
    })
}

fn kirp(deger: &[u8]) -> &[u8] {
    let bas = deger
        .iter()
        .position(|b| !matches!(b, b' ' | b'\t'))
        .unwrap_or(deger.len());
    let son = deger
        .iter()
        .rposition(|b| !matches!(b, b' ' | b'\t'))
        .map_or(bas, |sira| sira + 1);
    &deger[bas..son]
}

fn degerler<'a>(istek: &KahinIstegi<'a>, ad: &str) -> Vec<&'a [u8]> {
    istek
        .basliklar
        .iter()
        .filter(|(gelen, _)| gelen.eq_ignore_ascii_case(ad))
        .map(|(_, deger)| *deger)
        .collect()
}

/// RFC 9112 §6.3: TE yoksa ve tek geçerli Content-Length varsa gövde uzunluğu.
fn rfc_govde_uzunlugu(istek: &KahinIstegi<'_>) -> Result<usize, BilincliRet> {
    if !degerler(istek, "Transfer-Encoding").is_empty() {
        return Err(BilincliRet::TransferEncoding);
    }
    match degerler(istek, "Content-Length").as_slice() {
        [] => Ok(0),
        [deger] => std::str::from_utf8(deger)
            .ok()
            .filter(|metin| !metin.is_empty() && metin.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|metin| metin.parse::<usize>().ok())
            .ok_or(BilincliRet::GecersizContentLength),
        _ => Err(BilincliRet::CokluContentLength),
    }
}

/// İki ayrıştırıcıyı karşılaştırır; `Err` belgelenmemiş bir ayrılıktır.
pub fn karsilastir(baytlar: &[u8]) -> Result<Karar, String> {
    let zee = HttpIstegi::ayristir(baytlar);
    let kahin = kahin(baytlar);
    match (zee, kahin) {
        (Ok(zee), Some(kahin)) => {
            let zee_sonu = baslik_sonunu_bul(baytlar)
                .ok()
                .flatten()
                .ok_or("zee kabul etti ama başlık sonu bulunamadı")?;
            let mut farklar = Vec::new();
            if zee_sonu != kahin.baslik_sonu {
                farklar.push(format!(
                    "başlık sonu zee {zee_sonu}, kâhin {}",
                    kahin.baslik_sonu
                ));
            }
            if zee.baslik.yontem() != kahin.yontem || zee.baslik.hedef() != kahin.hedef {
                farklar.push(format!(
                    "request-line zee {} {}, kâhin {} {}",
                    zee.baslik.yontem(),
                    zee.baslik.hedef(),
                    kahin.yontem,
                    kahin.hedef
                ));
            }
            if zee.baslik.surum() != format!("HTTP/1.{}", kahin.surum) {
                farklar.push(format!(
                    "sürüm zee {}, kâhin 1.{}",
                    zee.baslik.surum(),
                    kahin.surum
                ));
            }
            for (ad, _) in &kahin.basliklar {
                let zee_degerleri: Vec<&[u8]> = zee
                    .baslik
                    .baslik_degerleri(ad)
                    .into_iter()
                    .map(str::as_bytes)
                    .collect();
                if zee_degerleri != degerler(&kahin, ad) {
                    farklar.push(format!("`{ad}` değerleri farklı"));
                }
            }
            match rfc_govde_uzunlugu(&kahin) {
                Ok(uzunluk) if uzunluk == zee.baslik.govde_uzunlugu() => {}
                Ok(uzunluk) => farklar.push(format!(
                    "gövde uzunluğu zee {}, RFC {uzunluk}",
                    zee.baslik.govde_uzunlugu()
                )),
                Err(sinif) => farklar.push(format!("zee RFC'nin reddettiği {sinif:?} kabul etti")),
            }
            if zee.govde.as_bytes() != &baytlar[kahin.baslik_sonu..] {
                farklar.push("gövde baytları farklı".into());
            }
            if farklar.is_empty() {
                Ok(Karar::IkisiKabul)
            } else {
                Err(format!("zee/kâhin kabul ayrılığı: {}", farklar.join("; ")))
            }
        }
        (Ok(_), None) => Err("zee kabul etti, bağımsız kâhin reddetti".into()),
        (Err(_), None) => Ok(Karar::IkisiRet),
        (Err(hata), Some(kahin)) => {
            let siniflar = bilincli_retler(baytlar, &kahin);
            if siniflar.is_empty() {
                Err(format!(
                    "zee `{hata}` ile reddetti, kâhin kabul etti; belgelenmiş katı kural yok"
                ))
            } else {
                Ok(Karar::ZeeDahaKati(siniflar))
            }
        }
    }
}

fn bilincli_retler(baytlar: &[u8], kahin: &KahinIstegi<'_>) -> Vec<BilincliRet> {
    let mut siniflar = Vec::new();
    let baslik = &baytlar[..kahin.baslik_sonu];
    if baslik.starts_with(b"\r\n") || baslik.starts_with(b"\n") {
        siniflar.push(BilincliRet::OnBosSatir);
    }
    if baslik
        .iter()
        .enumerate()
        .any(|(sira, b)| *b == b'\n' && (sira == 0 || baslik[sira - 1] != b'\r'))
    {
        siniflar.push(BilincliRet::YalinLf);
    }
    if !kahin.hedef.starts_with('/') || kahin.hedef.bytes().any(|b| b == b'#' || b >= 0x80) {
        siniflar.push(BilincliRet::OriginFormuDisi);
    }
    if kahin
        .basliklar
        .iter()
        .any(|(_, deger)| deger.iter().any(|b| *b >= 0x80))
    {
        siniflar.push(BilincliRet::AsciiDisiBaslikDegeri);
    }
    match rfc_govde_uzunlugu(kahin) {
        Ok(uzunluk) => {
            let govde = &baytlar[kahin.baslik_sonu..];
            if govde.len() != uzunluk {
                siniflar.push(BilincliRet::GovdeCercevesi);
            } else if govde.contains(&0) || std::str::from_utf8(govde).is_err() {
                siniflar.push(BilincliRet::GovdeMetni);
            }
        }
        Err(sinif) => siniflar.push(sinif),
    }
    siniflar
}
