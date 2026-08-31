//! dillsp çekirdeği — bağımlılıksız LSP sunucusu (master plan bölüm 15).
//!
//! Protokolün asgari dilimi: initialize, didOpen/didChange/didClose →
//! publishDiagnostics (çoklu tanı, RFC-0010), completion (kalıp kelimeleri),
//! shutdown/exit. İkili: `dillsp` (stdio üzerinden JSON-RPC).
//!
//! JSON ayrıştırıcı elle yazılmıştır (ADR-001 sıfır bağımlılık kuralı).

use crate::tani::Tani;
use std::collections::HashMap;

// ---------- mini JSON ----------

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Bos,
    Mantik(bool),
    Sayi(f64),
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
            Json::Metin(m) => Some(m),
            _ => None,
        }
    }
}

/// JSON metnini çözer; bozuk girdi None döner (sunucu mesajı yok sayar).
pub fn json_coz(metin: &str) -> Option<Json> {
    let mut karakterler = metin.chars().peekable();
    let deger = deger_coz(&mut karakterler)?;
    bosluk_atla(&mut karakterler);
    match karakterler.next() {
        None => Some(deger),
        Some(_) => None,
    }
}

type Karakterler<'a> = std::iter::Peekable<std::str::Chars<'a>>;

fn bosluk_atla(k: &mut Karakterler) {
    while matches!(k.peek(), Some(' ' | '\n' | '\r' | '\t')) {
        k.next();
    }
}

fn deger_coz(k: &mut Karakterler) -> Option<Json> {
    bosluk_atla(k);
    match k.peek()? {
        '{' => {
            k.next();
            let mut alanlar = Vec::new();
            bosluk_atla(k);
            if k.peek() == Some(&'}') {
                k.next();
                return Some(Json::Nesne(alanlar));
            }
            loop {
                bosluk_atla(k);
                let ad = metin_coz(k)?;
                bosluk_atla(k);
                if k.next() != Some(':') {
                    return None;
                }
                let deger = deger_coz(k)?;
                alanlar.push((ad, deger));
                bosluk_atla(k);
                match k.next() {
                    Some(',') => continue,
                    Some('}') => return Some(Json::Nesne(alanlar)),
                    _ => return None,
                }
            }
        }
        '[' => {
            k.next();
            let mut ogeler = Vec::new();
            bosluk_atla(k);
            if k.peek() == Some(&']') {
                k.next();
                return Some(Json::Dizi(ogeler));
            }
            loop {
                ogeler.push(deger_coz(k)?);
                bosluk_atla(k);
                match k.next() {
                    Some(',') => continue,
                    Some(']') => return Some(Json::Dizi(ogeler)),
                    _ => return None,
                }
            }
        }
        '"' => Some(Json::Metin(metin_coz(k)?)),
        't' => sabit_coz(k, "true", Json::Mantik(true)),
        'f' => sabit_coz(k, "false", Json::Mantik(false)),
        'n' => sabit_coz(k, "null", Json::Bos),
        _ => sayi_coz(k),
    }
}

fn sabit_coz(k: &mut Karakterler, beklenen: &str, deger: Json) -> Option<Json> {
    for b in beklenen.chars() {
        if k.next() != Some(b) {
            return None;
        }
    }
    Some(deger)
}

fn metin_coz(k: &mut Karakterler) -> Option<String> {
    if k.next() != Some('"') {
        return None;
    }
    let mut metin = String::new();
    loop {
        match k.next()? {
            '"' => return Some(metin),
            '\\' => match k.next()? {
                '"' => metin.push('"'),
                '\\' => metin.push('\\'),
                '/' => metin.push('/'),
                'n' => metin.push('\n'),
                'r' => metin.push('\r'),
                't' => metin.push('\t'),
                'b' => metin.push('\u{0008}'),
                'f' => metin.push('\u{000C}'),
                'u' => {
                    let mut kod = 0u32;
                    for _ in 0..4 {
                        kod = kod * 16 + k.next()?.to_digit(16)?;
                    }
                    // Vekil çiftler (surrogate) BMP dışı için:
                    if (0xD800..0xDC00).contains(&kod) {
                        if k.next()? != '\\' || k.next()? != 'u' {
                            return None;
                        }
                        let mut alt = 0u32;
                        for _ in 0..4 {
                            alt = alt * 16 + k.next()?.to_digit(16)?;
                        }
                        kod = 0x10000 + ((kod - 0xD800) << 10) + (alt - 0xDC00);
                    }
                    metin.push(char::from_u32(kod)?);
                }
                _ => return None,
            },
            b => metin.push(b),
        }
    }
}

fn sayi_coz(k: &mut Karakterler) -> Option<Json> {
    let mut govde = String::new();
    if k.peek() == Some(&'-') {
        govde.push('-');
        k.next();
    }
    while matches!(k.peek(), Some(r) if r.is_ascii_digit() || *r == '.' || *r == 'e' || *r == 'E' || *r == '+' || *r == '-')
    {
        govde.push(*k.peek().unwrap());
        k.next();
    }
    govde.parse::<f64>().ok().map(Json::Sayi)
}

fn json_metin_yaz(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len() + 2);
    cikti.push('"');
    for karakter in metin.chars() {
        match karakter {
            '"' => cikti.push_str("\\\""),
            '\\' => cikti.push_str("\\\\"),
            '\n' => cikti.push_str("\\n"),
            '\r' => cikti.push_str("\\r"),
            '\t' => cikti.push_str("\\t"),
            k if (k as u32) < 0x20 => cikti.push_str(&format!("\\u{:04x}", k as u32)),
            k => cikti.push(k),
        }
    }
    cikti.push('"');
    cikti
}

// ---------- sunucu ----------

/// Tamamlama önerileri: dilin kalıp kelimeleri (kaynağı: ayrıştırıcı yüzeyi).
const KALIP_KELIMELERI: [&str; 40] = [
    "yaz", "olsun", "ise", "değilse", "tekrarla", "için", "kez", "her", "kadar",
    "sürece", "olduğu", "olana", "ile", "ve", "veya", "diye", "sor", "yanıt",
    "işlem", "al", "döndür", "yapı", "test", "olmalı", "ekle", "artır", "azalt",
    "böl", "göre", "kullan", "birimini", "doğru", "yanlış", "yok", "yeni",
    "dene", "bitir", "saniye", "dakika", "hatasını",
];

#[derive(Default)]
pub struct Sunucu {
    belgeler: HashMap<String, String>,
}

/// mesaj_isle çıktısı: gönderilecek gövdeler + sunucu devam edecek mi.
pub struct Ciktilar {
    pub govdeler: Vec<String>,
    pub devam: bool,
}

impl Sunucu {
    pub fn yeni() -> Sunucu {
        Sunucu::default()
    }

    /// Tek bir JSON-RPC gövdesini işler.
    pub fn mesaj_isle(&mut self, govde: &str) -> Ciktilar {
        let mut cikti = Ciktilar { govdeler: Vec::new(), devam: true };
        let Some(mesaj) = json_coz(govde) else {
            return cikti;
        };
        let yontem = mesaj.alan("method").and_then(Json::metin).unwrap_or("");
        let kimlik = mesaj.alan("id");

        match yontem {
            "initialize" => {
                let sonuc = format!(
                    "{{\"capabilities\":{{\"textDocumentSync\":1,\
                     \"completionProvider\":{{}}}},\
                     \"serverInfo\":{{\"name\":\"dillsp\",\"version\":{}}}}}",
                    json_metin_yaz(env!("CARGO_PKG_VERSION"))
                );
                cikti.govdeler.push(yanit(kimlik, &sonuc));
            }
            "textDocument/didOpen" => {
                if let Some((uri, metin)) = ac_parametreleri(&mesaj) {
                    self.belgeler.insert(uri.clone(), metin);
                    cikti.govdeler.push(self.tanilari_yayinla(&uri));
                }
            }
            "textDocument/didChange" => {
                if let Some((uri, metin)) = degisim_parametreleri(&mesaj) {
                    self.belgeler.insert(uri.clone(), metin);
                    cikti.govdeler.push(self.tanilari_yayinla(&uri));
                }
            }
            "textDocument/didClose" => {
                if let Some(uri) = belge_uri(&mesaj) {
                    self.belgeler.remove(&uri);
                    cikti.govdeler.push(bos_tanilar(&uri));
                }
            }
            "textDocument/completion" => {
                let ogeler: Vec<String> = KALIP_KELIMELERI
                    .iter()
                    .map(|k| format!("{{\"label\":{},\"kind\":14}}", json_metin_yaz(k)))
                    .collect();
                cikti
                    .govdeler
                    .push(yanit(kimlik, &format!("[{}]", ogeler.join(","))));
            }
            "shutdown" => cikti.govdeler.push(yanit(kimlik, "null")),
            "exit" => cikti.devam = false,
            _ => {
                // Kimlikli bilinmeyen istekler boş sonuçla yanıtlanır ki istemci beklemede kalmasın.
                if kimlik.is_some() {
                    cikti.govdeler.push(yanit(kimlik, "null"));
                }
            }
        }
        cikti
    }

    fn tanilari_yayinla(&self, uri: &str) -> String {
        let metin = self.belgeler.get(uri).cloned().unwrap_or_default();
        let klasor = uri_klasoru(uri);
        let mut yukleyici = move |ad: &str| -> Result<String, String> {
            let klasor = klasor.clone().ok_or("birim yolu çözülemedi")?;
            if ad.contains(['/', '\\', '.']) {
                return Err("birim adı yol içeremez".into());
            }
            std::fs::read_to_string(klasor.join(format!("{}.dil", ad)))
                .map_err(|hata| hata.to_string())
        };
        let tanilar = crate::kaynagi_tanilari(&metin, &mut yukleyici);
        let govde: Vec<String> = tanilar.iter().map(lsp_tanisi).collect();
        format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\
             \"params\":{{\"uri\":{},\"diagnostics\":[{}]}}}}",
            json_metin_yaz(uri),
            govde.join(",")
        )
    }
}

fn yanit(kimlik: Option<&Json>, sonuc: &str) -> String {
    let kimlik = match kimlik {
        Some(Json::Sayi(s)) => format!("{}", *s as i64),
        Some(Json::Metin(m)) => json_metin_yaz(m),
        _ => "null".to_string(),
    };
    format!("{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{}}}", kimlik, sonuc)
}

fn bos_tanilar(uri: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\
         \"params\":{{\"uri\":{},\"diagnostics\":[]}}}}",
        json_metin_yaz(uri)
    )
}

fn lsp_tanisi(tani: &Tani) -> String {
    // Tani 1 tabanlı karakter konumu; LSP 0 tabanlı (UTF-16 — Türkçe harfler
    // BMP'de tek birim olduğundan karakter sayımıyla örtüşür).
    let satir = tani.satir.saturating_sub(1);
    let bas = tani.sutun.saturating_sub(1);
    let son = bas + tani.uzunluk;
    let mesaj = match &tani.oneri {
        Some(oneri) => format!("{}\nÖneri: {}", tani.mesaj, oneri),
        None => tani.mesaj.clone(),
    };
    format!(
        "{{\"range\":{{\"start\":{{\"line\":{},\"character\":{}}},\
         \"end\":{{\"line\":{},\"character\":{}}}}},\"severity\":1,\
         \"code\":{},\"source\":\"dil\",\"message\":{}}}",
        satir,
        bas,
        satir,
        son,
        json_metin_yaz(&tani.kod),
        json_metin_yaz(&mesaj)
    )
}

fn belge_uri(mesaj: &Json) -> Option<String> {
    mesaj
        .alan("params")?
        .alan("textDocument")?
        .alan("uri")?
        .metin()
        .map(str::to_string)
}

fn ac_parametreleri(mesaj: &Json) -> Option<(String, String)> {
    let belge = mesaj.alan("params")?.alan("textDocument")?;
    Some((
        belge.alan("uri")?.metin()?.to_string(),
        belge.alan("text")?.metin()?.to_string(),
    ))
}

fn degisim_parametreleri(mesaj: &Json) -> Option<(String, String)> {
    let uri = belge_uri(mesaj)?;
    let degisimler = mesaj.alan("params")?.alan("contentChanges")?;
    let Json::Dizi(ogeler) = degisimler else {
        return None;
    };
    // Tam eşitleme (textDocumentSync: 1): son değişiklik tüm metindir.
    let metin = ogeler.last()?.alan("text")?.metin()?.to_string();
    Some((uri, metin))
}

/// file:// URI'sinden klasörü çıkarır (yüzde-kaçışları çözerek).
fn uri_klasoru(uri: &str) -> Option<std::path::PathBuf> {
    let yol = uri.strip_prefix("file://")?;
    let mut cozulmus = String::new();
    let mut karakterler = yol.chars().peekable();
    while let Some(k) = karakterler.next() {
        if k == '%' {
            let yuksek = karakterler.next()?.to_digit(16)?;
            let dusuk = karakterler.next()?.to_digit(16)?;
            cozulmus.push(char::from_u32(yuksek * 16 + dusuk)?);
        } else {
            cozulmus.push(k);
        }
    }
    std::path::Path::new(&cozulmus).parent().map(|p| p.to_path_buf())
}
