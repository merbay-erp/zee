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
const KALIP_KELIMELERI: [&str; 52] = [
    "yaz", "olsun", "ise", "değilse", "tekrarla", "için", "kez", "her", "kadar",
    "sürece", "olduğu", "olana", "ile", "ve", "veya", "diye", "sor", "yanıt",
    "işlem", "al", "döndür", "yapı", "test", "olmalı", "ekle", "artır", "azalt",
    "böl", "göre", "kullan", "birimini", "doğru", "yanlış", "yok", "yeni",
    "dene", "bitir", "saniye", "dakika", "hatasını",
    "varsa", "yoksa", "başarılıysa", "başarısızsa", "sil", "yönlendir",
    "adresine", "çerezine", "sıralanmışı", "parçaları", "birleşmişi", "değişmişi",
];

/// Hover açıklamaları: kalıp kelimesi → tek satır Türkçe açıklama + örnek.
/// (Kaynak: spec/02-dizim ve dil turu; kelime kalıbın son ya da ayırt edici
/// parçasıdır.)
const KELIME_ACIKLAMALARI: [(&str, &str); 33] = [
    ("yaz", "Cümleyi bitirir: değeri ekrana (ya da `X dosyasına`) yazar.\n\n`\"Merhaba\" ile isim yaz`"),
    ("olsun", "Ad tanımlar ya da var olan ada atar; tür ilk değerden çıkar ve sonra değişmez.\n\n`yaş 10 olsun`"),
    ("ise", "Koşul dalı açar; koşul yüklem-sonludur (`...se/...sa`).\n\n`yaş 8 veya daha büyükse`"),
    ("değilse", "Bir `... ise` bloğunun aksi dalı; `değilse <koşul>` ile zincirlenir.\n\n`değilse tahmin gizliden büyükse`"),
    ("tekrarla", "Döngü açar: `10 kez tekrarla` ya da `bildi doğru olana kadar tekrarla`."),
    ("için", "İki iş görür: `her sayı için` (gezinme) ve `4 için karesini hesapla` (işlem çağrısı)."),
    ("sürece", "Koşullu döngü: `sayaç 0 dan büyük olduğu sürece`."),
    ("sor", "Kullanıcıya sorar; cevap `yanıt` adına gelir.\n\n`\"Adın ne?\" diye sor`"),
    ("yanıt", "Son `diye sor` cevabı. Sayı gerekiyorsa: `yanıtın sayısı`."),
    ("ile", "Değerleri birleştirir: metinde ekleme, aritmetik kalıpta ilk terim, çağrıda ayraç."),
    ("işlem", "İşlem tanımı açar; parametreler gövde başında `... al`, dönüş `... döndür`.\n\n`işlem karesini hesapla`"),
    ("al", "İşlem parametresi bildirir (gövdenin başında).\n\n`sayıyı al`"),
    ("döndür", "İşlemden değer döndürür. `yok döndür` Seçenek, `\"...\" hatasını döndür` Sonuç üretir."),
    ("yapı", "Alanları türleriyle bildirilen kayıt türü tanımlar; `yeni <Ad>` ile kurulur."),
    ("yeni", "Bir yapıdan değer oluşturur.\n\n`ayşe yeni Öğrenci olsun`"),
    ("test", "Test bloğu açar; `dil dene` (ve playground) koşar. Doğrulama: `... olmalı`."),
    ("olmalı", "Test doğrulaması: `kare 16 ya eşit olmalı`. Tutmazsa D001 beklenen/bulunanı gösterir."),
    ("ekle", "Listeye öğe ekler ya da dosya sonuna satır ekler.\n\n`sayılara 5 ekle`"),
    ("artır", "Sayıyı yerinde artırır.\n\n`toplamı notla artır`"),
    ("azalt", "Sayıyı yerinde azaltır.\n\n`sayacı 1 azalt`"),
    ("böl", "Bölme cümlesi: `ortalamayı toplamı adede böl`."),
    ("göre", "Desen eşleştirme açar; kollar `\"kare\" ise`, varsayılan `değilse`."),
    ("dene", "Başarabilir işi Sonuç'a çevirir: `dosyasını okumayı dene`, `sayısını almayı dene`."),
    ("varsa", "Seçenek sorgusu; bu dalda `değeri` erişimi güvenlidir (T036 daraltması)."),
    ("başarılıysa", "Sonuç sorgusu; bu dalda `değeri` güvenlidir, `değilse` dalında `hatası`."),
    ("yok", "Değerin yokluğu (Seçenek). Koleksiyon boşluğu ayrıdır: `boşsa`."),
    ("kullan", "Birim bağlar: `hesap_araclari birimini kullan` — dosya = birim (RFC-0009)."),
    ("bitir", "`programı bitir` — programı o noktada sonlandırır."),
    ("bekle", "`yarım saniye bekle` ya da eşzamanlı bloktan sonra `hepsini bekle`."),
    ("listesi", "Liste sabiti: `3, 7, 1, 9 listesi`. Virgülden sonra boşluk liste ayracıdır."),
    ("sil", "Listeden ilk eşleşen öğeyi ya da sözlükten anahtarı siler; yoksa sessizdir.\n\n`sayılardan 5 i sil`"),
    ("yönlendir", "Web: tarayıcıyı başka adrese gönderir (303).\n\n`\"/liste\" adresine yönlendir`"),
    ("çerezine", "Web: yanıtla Set-Cookie gönderir; okumak için rota içinde `çerezler` sözlüğü hazırdır.\n\n`\"oturum\" çerezine kimlik yaz`"),
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
                     \"completionProvider\":{{}},\
                     \"hoverProvider\":true,\
                     \"definitionProvider\":true,\
                     \"renameProvider\":true}},\
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
            "textDocument/hover" => {
                let sonuc = konum_parametreleri(&mesaj)
                    .and_then(|(uri, satir, sutun)| {
                        let metin = self.belgeler.get(&uri)?;
                        let kelime = konumdaki_kelime(metin, satir, sutun)?;
                        aciklama_uret(metin, &kelime)
                    })
                    .map(|aciklama| {
                        format!(
                            "{{\"contents\":{{\"kind\":\"markdown\",\"value\":{}}}}}",
                            json_metin_yaz(&aciklama)
                        )
                    })
                    .unwrap_or_else(|| "null".into());
                cikti.govdeler.push(yanit(kimlik, &sonuc));
            }
            "textDocument/rename" => {
                let sonuc = yeniden_adlandir(&mesaj, &self.belgeler)
                    .unwrap_or_else(|| "null".into());
                cikti.govdeler.push(yanit(kimlik, &sonuc));
            }
            "textDocument/definition" => {
                let sonuc = konum_parametreleri(&mesaj)
                    .and_then(|(uri, satir, sutun)| {
                        let metin = self.belgeler.get(&uri)?;
                        let kelime = konumdaki_kelime(metin, satir, sutun)?;
                        let (tanim_satiri, bas, uzunluk) = tanimi_bul(metin, &kelime)?;
                        Some(format!(
                            "{{\"uri\":{},\"range\":{{\"start\":{{\"line\":{},\"character\":{}}},\
                             \"end\":{{\"line\":{},\"character\":{}}}}}}}",
                            json_metin_yaz(&uri),
                            tanim_satiri,
                            bas,
                            tanim_satiri,
                            bas + uzunluk
                        ))
                    })
                    .unwrap_or_else(|| "null".into());
                cikti.govdeler.push(yanit(kimlik, &sonuc));
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
            match std::fs::read_to_string(klasor.join(format!("{}.dil", ad))) {
                Ok(kaynak) => Ok(kaynak),
                Err(hata) => crate::gomulu_birim(ad)
                    .map(str::to_string)
                    .ok_or_else(|| hata.to_string()),
            }
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

/// textDocument + position parametrelerini söker (satır/sütun 0 tabanlı).
fn konum_parametreleri(mesaj: &Json) -> Option<(String, usize, usize)> {
    let parametreler = mesaj.alan("params")?;
    let uri = parametreler
        .alan("textDocument")?
        .alan("uri")?
        .metin()?
        .to_string();
    let konum = parametreler.alan("position")?;
    let sayi = |alan: &str| -> Option<usize> {
        match konum.alan(alan)? {
            Json::Sayi(s) => Some(*s as usize),
            _ => None,
        }
    };
    Some((uri, sayi("line")?, sayi("character")?))
}

/// Konumdaki kelimeyi döner. Sütun UTF-16 birimidir; dilin alfabesi BMP
/// içinde kaldığından karakter sayımıyla birebirdir.
fn konumdaki_kelime(metin: &str, satir: usize, sutun: usize) -> Option<String> {
    let satir_metni = metin.lines().nth(satir)?;
    let karakterler: Vec<char> = satir_metni.chars().collect();
    let kelime_harfi = |k: char| k.is_alphanumeric() || k == '_';
    if sutun >= karakterler.len() || !kelime_harfi(karakterler[sutun]) {
        return None;
    }
    let mut bas = sutun;
    while bas > 0 && kelime_harfi(karakterler[bas - 1]) {
        bas -= 1;
    }
    let mut son = sutun;
    while son < karakterler.len() && kelime_harfi(karakterler[son]) {
        son += 1;
    }
    Some(karakterler[bas..son].iter().collect())
}

/// Kelimenin kendisi + morfolojik kök adayları (çözümleyiciyle aynı kurallar).
fn adaylar(kelime: &str) -> Vec<String> {
    let mut liste = vec![kelime.to_string()];
    liste.extend(crate::cozumleyici::kok_adaylari(kelime));
    liste
}

/// Hover içeriği: önce kalıp kelimesi açıklaması, yoksa belgedeki tanım satırı.
fn aciklama_uret(metin: &str, kelime: &str) -> Option<String> {
    for (kalip, aciklama) in KELIME_ACIKLAMALARI {
        if kelime == kalip {
            return Some(format!("**{}** — {}", kalip, aciklama));
        }
    }
    let (satir, _, _) = tanimi_bul(metin, kelime)?;
    let tanim = metin.lines().nth(satir)?.trim();
    Some(format!("Tanım (satır {}):\n```\n{}\n```", satir + 1, tanim))
}

/// Kelimenin tanımlandığı yeri arar: işlem/yapı başlığı, `... olsun` ya da
/// `... al` satırı. Dönen: (satır, sütun, uzunluk) — hepsi karakter cinsinden.
fn tanimi_bul(metin: &str, kelime: &str) -> Option<(usize, usize, usize)> {
    let adaylar = adaylar(kelime);
    let kelime_konumu = |satir: &str, hedefler: &[String]| -> Option<(usize, usize)> {
        let mut sutun = 0usize;
        for parca in satir.split(' ') {
            let temiz = parca.trim();
            if hedefler.iter().any(|h| h == temiz) {
                return Some((sutun, temiz.chars().count()));
            }
            sutun += parca.chars().count() + 1;
        }
        None
    };

    // 1) işlem / yapı başlıkları: başlıktaki HERHANGİ bir kelime aday
    //    kökle eşleşirse başlığa gider (işlem adları çok kelimeli olabilir).
    for (no, satir) in metin.lines().enumerate() {
        let kirpik = satir.trim_start();
        if kirpik.starts_with("işlem ") || kirpik.starts_with("yapı ") {
            if let Some((sutun, uzunluk)) = kelime_konumu(satir, &adaylar) {
                return Some((no, sutun, uzunluk));
            }
        }
    }
    // 2) Değer tanımı (`<ad> ... olsun`) ya da parametre (`<ad>ı al`):
    //    satırın İLK kelimesi aday kökle eşleşmeli.
    for (no, satir) in metin.lines().enumerate() {
        let kirpik = satir.trim_start();
        let Some(ilk) = kirpik.split(' ').next() else { continue };
        let ilk_adaylar = adaylar_ile_kesisir(ilk, &adaylar);
        if !ilk_adaylar {
            continue;
        }
        if kirpik.ends_with(" olsun") || kirpik.ends_with(" al") {
            let girinti = satir.chars().count() - kirpik.chars().count();
            return Some((no, girinti, ilk.chars().count()));
        }
    }
    None
}

/// İlk kelimenin kendi kök adayları, aranan adaylarla kesişiyor mu?
/// (Kullanımdaki ek ile tanımdaki ek farklı olabilir: `sayacı` ↔ `sayaç`.)
fn adaylar_ile_kesisir(ilk: &str, aranan: &[String]) -> bool {
    let ilk_kokler = adaylar(ilk);
    ilk_kokler.iter().any(|k| aranan.iter().any(|a| a == k))
}

/// Morfoloji-farkındalıklı yeniden adlandırma (K-072): kökü bul, belgedeki
/// bütün ekli/eksiz kullanımları yeni köke Türkçe uyumla giydirerek değiştir.
/// Metin sabitleri ve # yorumları dokunulmaz.
fn yeniden_adlandir(
    mesaj: &Json,
    belgeler: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let (uri, satir, sutun) = konum_parametreleri(mesaj)?;
    let yeni_ad = mesaj.alan("params")?.alan("newName")?.metin()?.to_string();
    if yeni_ad.is_empty() || !yeni_ad.chars().all(|k| k.is_alphanumeric() || k == '_') {
        return None;
    }
    let metin = belgeler.get(&uri)?;
    let kelime = konumdaki_kelime(metin, satir, sutun)?;
    // Kök: belgede tanımlı ada çöz (tanım satırından yalın ad).
    let (tanim_satiri, tanim_sutunu, tanim_uzunlugu) = tanimi_bul(metin, &kelime)?;
    let kok: String = metin
        .lines()
        .nth(tanim_satiri)?
        .chars()
        .skip(tanim_sutunu)
        .take(tanim_uzunlugu)
        .collect();

    let mut duzenlemeler = Vec::new();
    for (satir_no, satir_metni) in metin.lines().enumerate() {
        let karakterler: Vec<char> = satir_metni.chars().collect();
        let mut i = 0usize;
        let mut tirnakta = false;
        while i < karakterler.len() {
            let k = karakterler[i];
            if k == '"' {
                tirnakta = !tirnakta;
                i += 1;
                continue;
            }
            if !tirnakta && k == '#' {
                break;
            }
            let kelime_harfi = |k: char| k.is_alphanumeric() || k == '_';
            if !tirnakta && kelime_harfi(k) {
                let bas = i;
                while i < karakterler.len() && kelime_harfi(karakterler[i]) {
                    i += 1;
                }
                let soz: String = karakterler[bas..i].iter().collect();
                let yeni = if soz == kok {
                    Some(yeni_ad.clone())
                } else {
                    crate::cozumleyici::ek_coz(&soz, &kok)
                        .map(|ek| crate::cozumleyici::ek_uydur(&yeni_ad, ek))
                };
                if let Some(yeni) = yeni {
                    duzenlemeler.push(format!(
                        "{{\"range\":{{\"start\":{{\"line\":{},\"character\":{}}},\"end\":{{\"line\":{},\"character\":{}}}}},\"newText\":{}}}",
                        satir_no, bas, satir_no, i, json_metin_yaz(&yeni)
                    ));
                }
                continue;
            }
            i += 1;
        }
    }
    if duzenlemeler.is_empty() {
        return None;
    }
    Some(format!(
        "{{\"changes\":{{{}:[{}]}}}}",
        json_metin_yaz(&uri),
        duzenlemeler.join(",")
    ))
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
