//! Native web adaptörü için fail-closed HTTP/1.x istek ayrıştırıcısı.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpIstekHatasi(&'static str);

impl HttpIstekHatasi {
    const fn yeni(mesaj: &'static str) -> Self {
        Self(mesaj)
    }
}

impl fmt::Display for HttpIstekHatasi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for HttpIstekHatasi {}

/// Tam CRLF başlık sonunu döndürür. Eksik parça `Ok(None)`, kesin bozuk satır
/// sonu veya NUL `Err` olur; böylece socket okuyucusu bare-LF için beklemez.
pub fn baslik_sonunu_bul(baytlar: &[u8]) -> Result<Option<usize>, HttpIstekHatasi> {
    let mut sira = 0;
    while sira < baytlar.len() {
        match baytlar[sira] {
            0 => return Err(HttpIstekHatasi::yeni("HTTP başlığı NUL baytı taşıyamaz")),
            b'\n' => {
                return Err(HttpIstekHatasi::yeni(
                    "HTTP satırları yalnız CRLF ile bitmeli",
                ));
            }
            b'\r' => {
                let Some(&sonraki) = baytlar.get(sira + 1) else {
                    return Ok(None);
                };
                if sonraki != b'\n' {
                    return Err(HttpIstekHatasi::yeni(
                        "HTTP satırları yalnız CRLF ile bitmeli",
                    ));
                }
                if baytlar.get(sira..sira + 4) == Some(b"\r\n\r\n") {
                    return Ok(Some(sira + 4));
                }
                sira += 2;
            }
            _ => sira += 1,
        }
    }
    Ok(None)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpIstekBasligi<'a> {
    ham: &'a str,
    yontem: &'a str,
    hedef: &'a str,
    surum: &'a str,
    govde_uzunlugu: usize,
}

impl<'a> HttpIstekBasligi<'a> {
    pub fn ayristir(baytlar: &'a [u8]) -> Result<Self, HttpIstekHatasi> {
        if baslik_sonunu_bul(baytlar)? != Some(baytlar.len()) {
            return Err(HttpIstekHatasi::yeni("HTTP başlığı tam CRLF ile bitmeli"));
        }
        let ham = std::str::from_utf8(baytlar)
            .map_err(|_| HttpIstekHatasi::yeni("HTTP başlığı geçerli UTF-8 değil"))?;
        let mut satirlar = ham[..ham.len() - 4].split("\r\n");
        let ilk = satirlar
            .next()
            .ok_or_else(|| HttpIstekHatasi::yeni("HTTP istek satırı eksik"))?;
        let (yontem, hedef, surum) = istek_satirini_ayristir(ilk)?;
        let mut govde_uzunlugu = None;
        for satir in satirlar {
            basligi_dogrula(satir, &mut govde_uzunlugu)?;
        }
        Ok(Self {
            ham,
            yontem,
            hedef,
            surum,
            govde_uzunlugu: govde_uzunlugu.unwrap_or(0),
        })
    }

    pub const fn yontem(&self) -> &'a str {
        self.yontem
    }

    pub const fn hedef(&self) -> &'a str {
        self.hedef
    }

    pub const fn surum(&self) -> &'a str {
        self.surum
    }

    pub const fn govde_uzunlugu(&self) -> usize {
        self.govde_uzunlugu
    }

    pub fn baslik_degerleri(&self, ad: &str) -> Vec<&'a str> {
        self.ham[..self.ham.len() - 4]
            .split("\r\n")
            .skip(1)
            .filter_map(|satir| satir.split_once(':'))
            .filter(|(gelen, _)| gelen.eq_ignore_ascii_case(ad))
            .map(|(_, deger)| deger.trim_matches([' ', '\t']))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpIstegi<'a> {
    pub baslik: HttpIstekBasligi<'a>,
    pub govde: &'a str,
}

impl<'a> HttpIstegi<'a> {
    /// Fuzz/conformance için bütün bir bağlantı isteğini exact uzunlukla çözer.
    pub fn ayristir(baytlar: &'a [u8]) -> Result<Self, HttpIstekHatasi> {
        let govde_basi = baslik_sonunu_bul(baytlar)?
            .ok_or_else(|| HttpIstekHatasi::yeni("HTTP başlık sonu eksik"))?;
        let baslik = HttpIstekBasligi::ayristir(&baytlar[..govde_basi])?;
        let govde_baytlari = &baytlar[govde_basi..];
        if govde_baytlari.len() != baslik.govde_uzunlugu() {
            return Err(HttpIstekHatasi::yeni(
                "HTTP gövdesi Content-Length ile aynı uzunlukta olmalı",
            ));
        }
        let govde = govde_metni(govde_baytlari)?;
        Ok(Self { baslik, govde })
    }
}

pub fn govde_metni(baytlar: &[u8]) -> Result<&str, HttpIstekHatasi> {
    if baytlar.contains(&0) {
        return Err(HttpIstekHatasi::yeni("HTTP gövdesi NUL baytı taşıyamaz"));
    }
    std::str::from_utf8(baytlar)
        .map_err(|_| HttpIstekHatasi::yeni("HTTP gövdesi geçerli UTF-8 değil"))
}

fn istek_satirini_ayristir(satir: &str) -> Result<(&str, &str, &str), HttpIstekHatasi> {
    let mut parcalar = satir.split(' ');
    let (yontem, hedef, surum) = (parcalar.next(), parcalar.next(), parcalar.next());
    let (Some(yontem), Some(hedef), Some(surum)) = (yontem, hedef, surum) else {
        return Err(HttpIstekHatasi::yeni("HTTP istek satırı eksik"));
    };
    if parcalar.next().is_some()
        || yontem.is_empty()
        || !yontem.bytes().all(token_bayti_mi)
        || !gecerli_origin_formu(hedef)
        || !matches!(surum, "HTTP/1.0" | "HTTP/1.1")
    {
        return Err(HttpIstekHatasi::yeni("HTTP istek satırı geçersiz"));
    }
    Ok((yontem, hedef, surum))
}

fn basligi_dogrula(satir: &str, govde_uzunlugu: &mut Option<usize>) -> Result<(), HttpIstekHatasi> {
    if satir.starts_with([' ', '\t']) {
        return Err(HttpIstekHatasi::yeni(
            "HTTP obs-fold başlığı desteklenmiyor",
        ));
    }
    let (ad, ham_deger) = satir
        .split_once(':')
        .ok_or_else(|| HttpIstekHatasi::yeni("HTTP başlık satırı geçersiz"))?;
    if ad.is_empty() || !ad.bytes().all(token_bayti_mi) {
        return Err(HttpIstekHatasi::yeni("HTTP başlık adı geçersiz"));
    }
    if !ham_deger
        .bytes()
        .all(|bayt| bayt == b'\t' || (b' '..=b'~').contains(&bayt))
    {
        return Err(HttpIstekHatasi::yeni("HTTP başlık değeri geçersiz"));
    }
    let deger = ham_deger.trim_matches([' ', '\t']);
    if ad.eq_ignore_ascii_case("Transfer-Encoding") {
        return Err(HttpIstekHatasi::yeni("Transfer-Encoding desteklenmiyor"));
    }
    if ad.eq_ignore_ascii_case("Content-Length") {
        if govde_uzunlugu.is_some() {
            return Err(HttpIstekHatasi::yeni(
                "birden çok Content-Length başlığı reddedildi",
            ));
        }
        if deger.is_empty() || !deger.bytes().all(|bayt| bayt.is_ascii_digit()) {
            return Err(HttpIstekHatasi::yeni("Content-Length geçersiz"));
        }
        *govde_uzunlugu = Some(
            deger
                .parse::<usize>()
                .map_err(|_| HttpIstekHatasi::yeni("Content-Length geçersiz"))?,
        );
    }
    Ok(())
}

fn gecerli_origin_formu(hedef: &str) -> bool {
    hedef.starts_with('/')
        && hedef
            .bytes()
            .all(|bayt| (b'!'..=b'~').contains(&bayt) && bayt != b'#')
}

fn token_bayti_mi(bayt: u8) -> bool {
    bayt.is_ascii_alphanumeric()
        || matches!(
            bayt,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}
