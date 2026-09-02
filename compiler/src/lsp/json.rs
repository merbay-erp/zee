//! RFC 8259 uyumlu, kaynak bütçeli JSON ayrıştırıcı.

use std::collections::HashSet;

use super::{AZAMI_LSP_GOVDE_BAYTI, AZAMI_LSP_JSON_DERINLIGI, AZAMI_LSP_JSON_DUGUMU};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonSayisi {
    ham: String,
}

impl JsonSayisi {
    pub fn ham(&self) -> &str {
        &self.ham
    }

    pub fn dogal_sayi(&self) -> Option<usize> {
        if self.ham.bytes().all(|bayt| bayt.is_ascii_digit()) {
            self.ham.parse().ok()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Json {
    Bos,
    Mantik(bool),
    Sayi(JsonSayisi),
    Metin(String),
    Dizi(Vec<Json>),
    Nesne(Vec<(String, Json)>),
}

impl Json {
    pub fn alan(&self, ad: &str) -> Option<&Json> {
        match self {
            Json::Nesne(alanlar) => alanlar.iter().find(|(a, _)| a == ad).map(|(_, d)| d),
            _ => None,
        }
    }

    pub fn metin(&self) -> Option<&str> {
        match self {
            Json::Metin(metin) => Some(metin),
            _ => None,
        }
    }
}

/// Tek bir RFC 8259 JSON değerini çözer. Bozuk, fazla derin veya fazla büyük
/// girdi hiçbir kısmi değer döndürmez.
pub fn json_coz(metin: &str) -> Option<Json> {
    if metin.len() > AZAMI_LSP_GOVDE_BAYTI {
        return None;
    }
    let mut cozer = Cozer {
        metin,
        konum: 0,
        kalan_dugum: AZAMI_LSP_JSON_DUGUMU,
    };
    let deger = cozer.deger(0)?;
    cozer.bosluk_atla();
    (cozer.konum == metin.len()).then_some(deger)
}

struct Cozer<'a> {
    metin: &'a str,
    konum: usize,
    kalan_dugum: usize,
}

impl Cozer<'_> {
    fn deger(&mut self, derinlik: usize) -> Option<Json> {
        if derinlik > AZAMI_LSP_JSON_DERINLIGI {
            return None;
        }
        self.kalan_dugum = self.kalan_dugum.checked_sub(1)?;
        self.bosluk_atla();
        match self.siradaki_bayt()? {
            b'{' => self.nesne(derinlik),
            b'[' => self.dizi(derinlik),
            b'"' => self.metin().map(Json::Metin),
            b't' => self.sabit(b"true", Json::Mantik(true)),
            b'f' => self.sabit(b"false", Json::Mantik(false)),
            b'n' => self.sabit(b"null", Json::Bos),
            b'-' | b'0'..=b'9' => self.sayi(),
            _ => None,
        }
    }

    fn nesne(&mut self, derinlik: usize) -> Option<Json> {
        self.bayt_al(b'{')?;
        let mut alanlar = Vec::new();
        let mut adlar = HashSet::new();
        self.bosluk_atla();
        if self.bayt_al(b'}').is_some() {
            return Some(Json::Nesne(alanlar));
        }
        loop {
            self.bosluk_atla();
            let ad = self.metin()?;
            if !adlar.insert(ad.clone()) {
                return None;
            }
            self.bosluk_atla();
            self.bayt_al(b':')?;
            let deger = self.deger(derinlik + 1)?;
            alanlar.push((ad, deger));
            self.bosluk_atla();
            if self.bayt_al(b'}').is_some() {
                return Some(Json::Nesne(alanlar));
            }
            self.bayt_al(b',')?;
        }
    }

    fn dizi(&mut self, derinlik: usize) -> Option<Json> {
        self.bayt_al(b'[')?;
        let mut ogeler = Vec::new();
        self.bosluk_atla();
        if self.bayt_al(b']').is_some() {
            return Some(Json::Dizi(ogeler));
        }
        loop {
            ogeler.push(self.deger(derinlik + 1)?);
            self.bosluk_atla();
            if self.bayt_al(b']').is_some() {
                return Some(Json::Dizi(ogeler));
            }
            self.bayt_al(b',')?;
        }
    }

    fn metin(&mut self) -> Option<String> {
        self.bayt_al(b'"')?;
        let mut sonuc = String::new();
        loop {
            match self.karakter_al()? {
                '"' => return Some(sonuc),
                '\\' => match self.karakter_al()? {
                    '"' => sonuc.push('"'),
                    '\\' => sonuc.push('\\'),
                    '/' => sonuc.push('/'),
                    'b' => sonuc.push('\u{0008}'),
                    'f' => sonuc.push('\u{000C}'),
                    'n' => sonuc.push('\n'),
                    'r' => sonuc.push('\r'),
                    't' => sonuc.push('\t'),
                    'u' => sonuc.push(self.unicode_kacisi()?),
                    _ => return None,
                },
                karakter if karakter <= '\u{001F}' => return None,
                karakter => sonuc.push(karakter),
            }
        }
    }

    fn unicode_kacisi(&mut self) -> Option<char> {
        let ust = self.dort_onaltilik()?;
        let kod = if (0xD800..=0xDBFF).contains(&ust) {
            self.bayt_al(b'\\')?;
            self.bayt_al(b'u')?;
            let alt = self.dort_onaltilik()?;
            if !(0xDC00..=0xDFFF).contains(&alt) {
                return None;
            }
            0x10000 + ((ust - 0xD800) << 10) + (alt - 0xDC00)
        } else if (0xDC00..=0xDFFF).contains(&ust) {
            return None;
        } else {
            ust
        };
        char::from_u32(kod)
    }

    fn dort_onaltilik(&mut self) -> Option<u32> {
        let mut kod = 0;
        for _ in 0..4 {
            kod = kod * 16 + (self.karakter_al()?.to_digit(16)?);
        }
        Some(kod)
    }

    fn sayi(&mut self) -> Option<Json> {
        let baslangic = self.konum;
        self.bayt_al(b'-');
        match self.siradaki_bayt()? {
            b'0' => self.konum += 1,
            b'1'..=b'9' => {
                self.konum += 1;
                self.rakamlar();
            }
            _ => return None,
        }
        if self.bayt_al(b'.').is_some() {
            self.en_az_bir_rakam()?;
        }
        if matches!(self.siradaki_bayt(), Some(b'e' | b'E')) {
            self.konum += 1;
            if matches!(self.siradaki_bayt(), Some(b'+' | b'-')) {
                self.konum += 1;
            }
            self.en_az_bir_rakam()?;
        }
        Some(Json::Sayi(JsonSayisi {
            ham: self.metin[baslangic..self.konum].to_string(),
        }))
    }

    fn en_az_bir_rakam(&mut self) -> Option<()> {
        if !matches!(self.siradaki_bayt(), Some(b'0'..=b'9')) {
            return None;
        }
        self.rakamlar();
        Some(())
    }

    fn rakamlar(&mut self) {
        while matches!(self.siradaki_bayt(), Some(b'0'..=b'9')) {
            self.konum += 1;
        }
    }

    fn sabit(&mut self, beklenen: &[u8], deger: Json) -> Option<Json> {
        let son = self.konum.checked_add(beklenen.len())?;
        if self.metin.as_bytes().get(self.konum..son)? != beklenen {
            return None;
        }
        self.konum = son;
        Some(deger)
    }

    fn bosluk_atla(&mut self) {
        while matches!(self.siradaki_bayt(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.konum += 1;
        }
    }

    fn siradaki_bayt(&self) -> Option<u8> {
        self.metin.as_bytes().get(self.konum).copied()
    }

    fn bayt_al(&mut self, beklenen: u8) -> Option<()> {
        if self.siradaki_bayt()? != beklenen {
            return None;
        }
        self.konum += 1;
        Some(())
    }

    fn karakter_al(&mut self) -> Option<char> {
        let karakter = self.metin.get(self.konum..)?.chars().next()?;
        self.konum += karakter.len_utf8();
        Some(karakter)
    }
}
