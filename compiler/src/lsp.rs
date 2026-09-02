//! dillsp çekirdeği — bağımlılıksız LSP sunucusu (master plan bölüm 15).
//!
//! Protokolün asgari dilimi: initialize, didOpen/didChange/didClose →
//! publishDiagnostics (çoklu tanı, RFC-0010), completion (kalıp kelimeleri),
//! shutdown/exit. İkili: `dillsp` (stdio üzerinden JSON-RPC).
//!
//! JSON ayrıştırıcı elle yazılmıştır (ADR-001: dil semantiği çekirdekte kalır).

use crate::tani::Tani;
use std::collections::HashMap;

// ---------- mini JSON ----------

pub const AZAMI_LSP_BASLIK_BAYTI: usize = 8 * 1024;
pub const AZAMI_LSP_GOVDE_BAYTI: usize = 8 * 1024 * 1024;
pub const AZAMI_LSP_JSON_DERINLIGI: usize = 128;
pub const AZAMI_LSP_JSON_DUGUMU: usize = 100_000;

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
    if metin.len() > AZAMI_LSP_GOVDE_BAYTI {
        return None;
    }
    let mut karakterler = metin.chars().peekable();
    let mut butce = JsonButcesi {
        kalan_dugum: AZAMI_LSP_JSON_DUGUMU,
    };
    let deger = deger_coz(&mut karakterler, &mut butce, 0)?;
    bosluk_atla(&mut karakterler);
    match karakterler.next() {
        None => Some(deger),
        Some(_) => None,
    }
}

type Karakterler<'a> = std::iter::Peekable<std::str::Chars<'a>>;

struct JsonButcesi {
    kalan_dugum: usize,
}

impl JsonButcesi {
    fn dugum_al(&mut self) -> Option<()> {
        self.kalan_dugum = self.kalan_dugum.checked_sub(1)?;
        Some(())
    }
}

fn bosluk_atla(k: &mut Karakterler) {
    while matches!(k.peek(), Some(' ' | '\n' | '\r' | '\t')) {
        k.next();
    }
}

fn deger_coz(k: &mut Karakterler, butce: &mut JsonButcesi, derinlik: usize) -> Option<Json> {
    if derinlik > AZAMI_LSP_JSON_DERINLIGI {
        return None;
    }
    butce.dugum_al()?;
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
                let deger = deger_coz(k, butce, derinlik + 1)?;
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
                ogeler.push(deger_coz(k, butce, derinlik + 1)?);
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
                    if (0xD800..=0xDBFF).contains(&kod) {
                        if k.next()? != '\\' || k.next()? != 'u' {
                            return None;
                        }
                        let mut alt = 0u32;
                        for _ in 0..4 {
                            alt = alt * 16 + k.next()?.to_digit(16)?;
                        }
                        if !(0xDC00..=0xDFFF).contains(&alt) {
                            return None;
                        }
                        kod = 0x10000 + ((kod - 0xD800) << 10) + (alt - 0xDC00);
                    } else if (0xDC00..=0xDFFF).contains(&kod) {
                        return None;
                    }
                    metin.push(char::from_u32(kod)?);
                }
                _ => return None,
            },
            b if b <= '\u{001F}' => return None,
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
        if let Some(r) = k.next() {
            govde.push(r);
        }
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
const KALIP_KELIMELERI: [&str; 87] = [
    "yaz",
    "olsun",
    "ise",
    "değilse",
    "tekrarla",
    "için",
    "kez",
    "her",
    "kadar",
    "sürece",
    "olduğu",
    "olana",
    "ile",
    "ve",
    "veya",
    "diye",
    "sor",
    "yanıt",
    "işlem",
    "eylem",
    "al",
    "döndür",
    "yapı",
    "test",
    "olmalı",
    "ekle",
    "artır",
    "azalt",
    "böl",
    "göre",
    "kullan",
    "birimini",
    "paketini",
    "doğru",
    "yanlış",
    "yok",
    "yeni",
    "dene",
    "bitir",
    "saniye",
    "dakika",
    "hatasını",
    "kodlu",
    "nedeniyle",
    "verisiyle",
    "kodu",
    "mesajı",
    "nedeni",
    "verisi",
    "varsa",
    "yoksa",
    "başarılıysa",
    "başarısızsa",
    "sil",
    "yönlendir",
    "adresine",
    "çerezine",
    "sıralanmışı",
    "parçaları",
    "birleşmişi",
    "değişmişi",
    "içermeli",
    "olmamalı",
    "kuruşlusu",
    "metni",
    "harfleri",
    "kırpılmışı",
    "arasındaki",
    "günler",
    "önekli",
    "kalanı",
    "GET",
    "HEAD",
    "POST",
    "PUT",
    "PATCH",
    "DELETE",
    "herkese",
    "açık",
    "oturum",
    "gerekli",
    "yetkisi",
    "rolüyle",
    "alanı",
    "belirteci",
    "doğrulanıyorsa",
    "kullanıcısını",
];

/// Hover açıklamaları: kalıp kelimesi → tek satır Türkçe açıklama + örnek.
/// (Kaynak: spec/02-dizim ve dil turu; kelime kalıbın son ya da ayırt edici
/// parçasıdır.)
const KELIME_ACIKLAMALARI: [(&str, &str); 44] = [
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
    ("işlem", "İşlem tanımı açar; başlangıç parametresi `... al`dır. Birim/paket sözleşmesinde `... <Tür> olarak al` ve ardından `<Tür> döndürür` ya da `değer döndürmez` yazılır; değer `... döndür` ile çıkar.\n\n`işlem karesini hesapla`"),
    ("eylem", "HTTP protokolünden bağımsız, açık imzalı uygulama iş kuralı ve transaction sınırı tanımlar. Web, CLI ya da görevden aynı biçimde çağrılır.\n\n`eylem notu kaydet`"),
    ("al", "İşlem parametresi bildirir. Başlangıç: `sayıyı al`; açık API: `sayıyı Ondalık olarak al`."),
    ("döndür", "İşlemden değer döndürür. `yok döndür` Seçenek, `\"...\" hatasını döndür` Sonuç üretir. Yapılandırılmış biçim: `\"KOD\" kodlu \"mesaj\" hatasını döndür`."),
    ("kodlu", "Kararlı etiketli Hata üretir. Kod A-Z ile başlar; A-Z, 0-9 ve `_` kullanır.\n\n`\"DOSYA_YOK\" kodlu \"Dosya bulunamadı\" hatasını döndür`"),
    ("nedeniyle", "Yeni bir Hata'yı başka bir Hata ile sararak neden zinciri kurar."),
    ("verisiyle", "Hata'ya Metin sözlüğü biçiminde makinece okunabilir bağlam ekler."),
    ("kodu", "Hata'nın kararlı etiketini verir; `göre` ile eşlenebilir.\n\n`hatanın kodu`"),
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
    ("kullan", "Kaynak bağlar: `hesap_araclari birimini kullan` aynı klasördeki dosyayı, `grafik paketini kullan` proje bağımlılığını alır (RFC-0009)."),
    ("bitir", "`programı bitir` — programı o noktada sonlandırır."),
    ("bekle", "`yarım saniye bekle` ya da eşzamanlı bloktan sonra `hepsini bekle`."),
    ("listesi", "Liste sabiti: `3, 7, 1, 9 listesi`. Virgülden sonra boşluk liste ayracıdır."),
    ("sil", "Listeden ilk eşleşen öğeyi ya da sözlükten anahtarı siler; yoksa sessizdir.\n\n`sayılardan 5 i sil`"),
    ("yönlendir", "Web: tarayıcıyı başka adrese gönderir (303).\n\n`\"/liste\" adresine yönlendir`"),
    ("çerezine", "Web: yanıtla Set-Cookie gönderir; okumak için rota içinde `çerezler` sözlüğü hazırdır.\n\n`\"oturum\" çerezine kimlik yaz`"),
    ("herkese", "Durum değiştiren rotanın açık erişim politikasıdır; CSRF yine zorunludur.\n\n`herkese açık`"),
    ("oturum", "Kimliği doğrulanmış kullanıcı isteyen rota politikasıdır.\n\n`oturum gerekli`"),
    ("yetkisi", "Sunucu tarafı rozet denetimi yapan rota politikasıdır.\n\n`\"yönetici\" yetkisi gerekli`"),
    ("belirteci", "Form için sunucu oturumuna bağlı CSRF değeri üretir.\n\n`csrf csrf belirteci olsun`"),
    ("doğrulanıyorsa", "Parolayı Argon2id PHC özetiyle doğrular; düz parola karşılaştırması yapmaz.\n\n`verilen özet ile doğrulanıyorsa`"),
    ("rolüyle", "Başarılı girişte yeni, döndürülmüş sunucu oturumu açar.\n\n`\"Zeynep\" kullanıcısını \"yönetici\" rolüyle oturuma al`"),
];

#[derive(Default)]
pub struct Sunucu {
    belgeler: HashMap<String, String>,
    toplam_belge_bayti: usize,
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
        let mut cikti = Ciktilar {
            govdeler: Vec::new(),
            devam: true,
        };
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
                    match self.belgeyi_guncelle(uri.clone(), metin) {
                        Ok(()) => cikti.govdeler.push(self.tanilari_yayinla(&uri)),
                        Err(hata) => cikti.govdeler.push(kaynak_siniri_bildirimi(&uri, &hata)),
                    }
                }
            }
            "textDocument/didChange" => {
                if let Some((uri, metin)) = degisim_parametreleri(&mesaj) {
                    match self.belgeyi_guncelle(uri.clone(), metin) {
                        Ok(()) => cikti.govdeler.push(self.tanilari_yayinla(&uri)),
                        Err(hata) => cikti.govdeler.push(kaynak_siniri_bildirimi(&uri, &hata)),
                    }
                }
            }
            "textDocument/didClose" => {
                if let Some(uri) = belge_uri(&mesaj) {
                    if let Some(eski) = self.belgeler.remove(&uri) {
                        self.toplam_belge_bayti =
                            self.toplam_belge_bayti.saturating_sub(eski.len());
                    }
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
                let sonuc =
                    yeniden_adlandir(&mesaj, &self.belgeler).unwrap_or_else(|| "null".into());
                cikti.govdeler.push(yanit(kimlik, &sonuc));
            }
            "textDocument/definition" => {
                let sonuc = konum_parametreleri(&mesaj)
                    .and_then(|(uri, satir, sutun)| {
                        let metin = self.belgeler.get(&uri)?;
                        match semantik_tanim_sorgula(&uri, metin, satir, sutun) {
                            Some(Some(aralik)) => Some(tanim_yaniti(&uri, aralik)),
                            // Bağsız konumda ya da hatalı belgede metin tahmini
                            // yapılmaz; tanılar didOpen/didChange ile yayımlanır.
                            Some(None) | None => None,
                        }
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
        ciktilari_sinirla(kimlik, &mut cikti);
        cikti
    }

    fn belgeyi_guncelle(&mut self, uri: String, metin: String) -> Result<(), String> {
        let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI;
        if metin.len() > sinirlar.kaynak_bayti() {
            return Err(format!(
                "Belge güvenli profil {} MiB kaynak sınırını aşıyor.",
                sinirlar.kaynak_bayti() / 1024 / 1024
            ));
        }
        let yeni_belge = !self.belgeler.contains_key(&uri);
        if yeni_belge && self.belgeler.len() >= sinirlar.lsp_acik_belge() {
            return Err(format!(
                "LSP güvenli profil {} açık belge sınırını aşıyor.",
                sinirlar.lsp_acik_belge()
            ));
        }
        let eski_bayt = self.belgeler.get(&uri).map_or(0, String::len);
        let yeni_toplam = self
            .toplam_belge_bayti
            .saturating_sub(eski_bayt)
            .checked_add(metin.len())
            .ok_or_else(|| "LSP belge belleği sayı sınırını aştı.".to_string())?;
        if yeni_toplam > sinirlar.lsp_toplam_belge_bayti() {
            return Err(format!(
                "LSP belgeleri toplam {} MiB bellek sınırını aşıyor.",
                sinirlar.lsp_toplam_belge_bayti() / 1024 / 1024
            ));
        }
        self.belgeler.insert(uri, metin);
        self.toplam_belge_bayti = yeni_toplam;
        Ok(())
    }

    fn tanilari_yayinla(&self, uri: &str) -> String {
        let metin = self.belgeler.get(uri).cloned().unwrap_or_default();
        let belge_yolu = uri_yolu(uri).and_then(|yol| std::fs::canonicalize(yol).ok());

        // Bir proje içindeyse CLI ile aynı kökenli paket grafiğini kullan.
        // Böylece editörde temiz görünen kaynak komut satırında kırılmaz.
        let proje_tanilari = belge_yolu.as_ref().and_then(|belge_yolu| {
            let kok = proje_kokunu_bul(belge_yolu)?;
            let grafik = match crate::paket::ProjeGrafigi::cozumle(&kok) {
                Ok(grafik) => grafik,
                Err(hata) => return Some(vec![*hata.tani]),
            };
            if let Err(hata) = grafik.kilidi_denetle() {
                return Some(vec![*hata.tani]);
            }
            let koken = belge_yolu.to_string_lossy().into_owned();
            let mut yukleyici = |istek: crate::BirimIstegi<'_>| grafik.yukle(istek);
            Some(crate::kaynagi_tanilari_kokenlerle(
                &metin,
                Some(&koken),
                &mut yukleyici,
            ))
        });

        let klasor = uri_klasoru(uri);
        let mut yukleyici = move |ad: &str| -> Result<String, String> {
            let klasor = klasor.clone().ok_or("birim yolu çözülemedi")?;
            if ad.contains(['/', '\\', '.']) {
                return Err("birim adı yol içeremez".into());
            }
            match crate::kaynak_sinirlari::kaynak_dosyasi_oku(
                &klasor.join(format!("{}.dil", ad)),
            ) {
                Ok(kaynak) => Ok(kaynak),
                Err(hata) => crate::gomulu_birim(ad)
                    .map(str::to_string)
                    .ok_or_else(|| hata.to_string()),
            }
        };
        let tanilar =
            proje_tanilari.unwrap_or_else(|| crate::kaynagi_tanilari(&metin, &mut yukleyici));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LspKaynakAraligi {
    satir: usize,
    bas: usize,
    uzunluk: usize,
}

impl LspKaynakAraligi {
    fn icerir(self, satir: usize, sutun: usize) -> bool {
        self.satir == satir && self.bas <= sutun && sutun < self.bas + self.uzunluk
    }
}

fn tanim_yaniti(uri: &str, aralik: LspKaynakAraligi) -> String {
    format!(
        "{{\"uri\":{},\"range\":{{\"start\":{{\"line\":{},\"character\":{}}},\
         \"end\":{{\"line\":{},\"character\":{}}}}}}}",
        json_metin_yaz(uri),
        aralik.satir,
        aralik.bas,
        aralik.satir,
        aralik.bas + aralik.uzunluk
    )
}

/// Kod satırındaki tanımlayıcı aralıkları. Metin sabiti ve yorum içi LSP
/// semantic yüzeyine hiç girmez.
fn satir_kelime_araliklari(satir: &str) -> Vec<(LspKaynakAraligi, String)> {
    let karakterler = satir.chars().collect::<Vec<_>>();
    let mut sonuc = Vec::new();
    let mut i = 0usize;
    let mut tirnakta = false;
    let mut kacis = false;
    while i < karakterler.len() {
        let karakter = karakterler[i];
        if tirnakta {
            if kacis {
                kacis = false;
            } else if karakter == '\\' {
                kacis = true;
            } else if karakter == '"' {
                tirnakta = false;
            }
            i += 1;
            continue;
        }
        if karakter == '"' {
            tirnakta = true;
            i += 1;
            continue;
        }
        if karakter == '#' {
            break;
        }
        if karakter.is_alphanumeric() || karakter == '_' {
            let bas = i;
            while i < karakterler.len()
                && (karakterler[i].is_alphanumeric() || karakterler[i] == '_')
            {
                i += 1;
            }
            sonuc.push((
                LspKaynakAraligi {
                    satir: 0,
                    bas,
                    uzunluk: i - bas,
                },
                karakterler[bas..i].iter().collect(),
            ));
            continue;
        }
        i += 1;
    }
    sonuc
}

fn aralik_metni(metin: &str, aralik: LspKaynakAraligi) -> Option<String> {
    let karakterler = metin.lines().nth(aralik.satir)?.chars().collect::<Vec<_>>();
    let son = aralik.bas.checked_add(aralik.uzunluk)?;
    karakterler
        .get(aralik.bas..son)
        .map(|dilim| dilim.iter().collect())
}

fn kokle_eslesir(yazim: &str, kok: &str) -> bool {
    yazim == kok || crate::morfoloji::ek_zinciri_coz(yazim, kok).is_some()
}

fn hir_araligini_coz(
    metin: &str,
    kok: &str,
    aralik: crate::hir::HirKaynakAraligi,
) -> Option<LspKaynakAraligi> {
    if let Some((satir, sutun, uzunluk)) = aralik.kesin_konumu() {
        let kesin = LspKaynakAraligi {
            satir: satir.checked_sub(1)?,
            bas: sutun.checked_sub(1)?,
            uzunluk,
        };
        let yazim = aralik_metni(metin, kesin)?;
        return kokle_eslesir(&yazim, kok).then_some(kesin);
    }

    let satir = aralik.satiri().checked_sub(1)?;
    let satir_metni = metin.lines().nth(satir)?;
    let kelimeler = satir_kelime_araliklari(satir_metni);
    let (mut bulunan, _) = kelimeler
        .iter()
        .find(|(_, yazim)| yazim == kok)
        .or_else(|| {
            kelimeler
                .iter()
                .find(|(_, yazim)| kokle_eslesir(yazim, kok))
        })?;
    bulunan.satir = satir;
    Some(bulunan)
}

fn satirdaki_ifade_araliklari(metin: &str, satir: usize, ifade: &str) -> Vec<LspKaynakAraligi> {
    let Some(satir_metni) = metin.lines().nth(satir) else {
        return Vec::new();
    };
    let kelimeler = satir_kelime_araliklari(satir_metni);
    let aranan = ifade.split(' ').collect::<Vec<_>>();
    if aranan.is_empty() || aranan.iter().any(|kelime| kelime.is_empty()) {
        return Vec::new();
    }
    let mut sonuc = Vec::new();
    for pencere in kelimeler.windows(aranan.len()) {
        if pencere
            .iter()
            .zip(&aranan)
            .all(|((_, yazim), aranan)| yazim == aranan)
        {
            let ilk = pencere[0].0;
            let son = pencere[pencere.len() - 1].0;
            sonuc.push(LspKaynakAraligi {
                satir,
                bas: ilk.bas,
                uzunluk: son.bas + son.uzunluk - ilk.bas,
            });
        }
    }
    sonuc
}

fn satirdaki_ifade_araligi(metin: &str, satir: usize, ifade: &str) -> Option<LspKaynakAraligi> {
    satirdaki_ifade_araliklari(metin, satir, ifade)
        .into_iter()
        .next()
}

fn yerel_islem_tanimi_araligi(metin: &str, satir: usize, ad: &str) -> Option<LspKaynakAraligi> {
    let kirpik = metin.lines().nth(satir)?.trim_start();
    let tanimli_ad = kirpik
        .strip_prefix("işlem ")
        .or_else(|| kirpik.strip_prefix("eylem "))?;
    if tanimli_ad.trim_end() != ad {
        return None;
    }
    satirdaki_ifade_araligi(metin, satir, ad)
}

fn yerel_yapi_tanimi_araligi(metin: &str, satir: usize, ad: &str) -> Option<LspKaynakAraligi> {
    let kirpik = metin.lines().nth(satir)?.trim_start();
    let tanimli_ad = kirpik.strip_prefix("yapı ")?;
    if tanimli_ad.trim_end() != ad {
        return None;
    }
    satirdaki_ifade_araligi(metin, satir, ad)
}

fn hir_satir_ifadelerini_coz(
    metin: &str,
    ifade: &str,
    kaynak_araligi: crate::hir::HirKaynakAraligi,
) -> Vec<LspKaynakAraligi> {
    let Some(satir) = kaynak_araligi.satiri().checked_sub(1) else {
        return Vec::new();
    };
    let mut adaylar = satirdaki_ifade_araliklari(metin, satir, ifade);
    if let Some((_kesin_satir, sutun, uzunluk)) = kaynak_araligi.kesin_konumu() {
        let Some(bas) = sutun.checked_sub(1) else {
            return Vec::new();
        };
        let Some(son) = bas.checked_add(uzunluk) else {
            return Vec::new();
        };
        adaylar.retain(|aday| {
            aday.satir == satir
                && aday.bas >= bas
                && aday
                    .bas
                    .checked_add(aday.uzunluk)
                    .is_some_and(|aday_sonu| aday_sonu <= son)
        });
        // İşlem adı çağrı ifadesinin, yapı adı da `yeni` ifadesinin
        // kuyruğundadır. Aynı yazım argüman bölgesinde de geçse semantic bağ
        // yalnız kesin zarf içindeki son eşleşmeye aittir.
        return adaylar.into_iter().last().into_iter().collect();
    }
    adaylar
}

fn semantik_program_derle(uri: &str, metin: &str) -> Option<crate::faz::BaglanmisProgram> {
    let belge_yolu = uri_yolu(uri).and_then(|yol| std::fs::canonicalize(yol).ok());
    if let Some((belge_yolu, kok)) = belge_yolu
        .as_ref()
        .and_then(|yol| proje_kokunu_bul(yol).map(|kok| (yol, kok)))
    {
        let grafik = crate::paket::ProjeGrafigi::cozumle(&kok).ok()?;
        grafik.kilidi_denetle().ok()?;
        let koken = belge_yolu.to_string_lossy().into_owned();
        let mut yukleyici = |istek: crate::BirimIstegi<'_>| grafik.yukle(istek);
        return crate::kaynagi_fazli_derle_kokenlerle(metin, Some(&koken), &mut yukleyici).ok();
    }

    let klasor = uri_klasoru(uri);
    let mut yukleyici = move |ad: &str| -> Result<String, String> {
        let klasor = klasor.clone().ok_or("birim yolu çözülemedi")?;
        if ad.contains(['/', '\\', '.']) {
            return Err("birim adı yol içeremez".into());
        }
        match crate::kaynak_sinirlari::kaynak_dosyasi_oku(
            &klasor.join(format!("{}.dil", ad)),
        ) {
            Ok(kaynak) => Ok(kaynak),
            Err(hata) => crate::gomulu_birim(ad)
                .map(str::to_string)
                .ok_or_else(|| hata.to_string()),
        }
    };
    crate::kaynagi_fazli_derle_birimlerle(metin, &mut yukleyici).ok()
}

fn semantik_sembol_bul(
    program: &crate::faz::BaglanmisProgram,
    metin: &str,
    satir: usize,
    sutun: usize,
) -> Option<crate::kimlik::SymbolId> {
    let hir = program.hir();
    hir.sembol_kullanimlari().into_iter().find_map(|kullanim| {
        let kimlik = kullanim.kimlik();
        let kok = hir.sembol_adi(kimlik)?;
        let aralik = hir_araligini_coz(metin, kok, kullanim.kaynak_araligi())?;
        aralik.icerir(satir, sutun).then_some(kimlik)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemantikVarlik {
    Sembol(crate::kimlik::SymbolId),
    Islem(crate::kimlik::IslemId),
    Yapi(crate::kimlik::YapiId),
}

fn semantik_varlik_bul(
    program: &crate::faz::BaglanmisProgram,
    metin: &str,
    satir: usize,
    sutun: usize,
) -> Option<SemantikVarlik> {
    if let Some(kimlik) = semantik_sembol_bul(program, metin, satir, sutun) {
        return Some(SemantikVarlik::Sembol(kimlik));
    }
    let hir = program.hir();

    for kimlik in hir.islem_kimlikleri() {
        let ad = hir.islem_adi(kimlik)?;
        let tanim = hir.islem(kimlik)?;
        if yerel_islem_tanimi_araligi(metin, tanim.satir.checked_sub(1)?, ad)
            .is_some_and(|aralik| aralik.icerir(satir, sutun))
        {
            return Some(SemantikVarlik::Islem(kimlik));
        }
    }
    for (kimlik, kaynak_araligi) in hir.islem_kullanimlari() {
        let ad = hir.islem_adi(kimlik)?;
        if hir_satir_ifadelerini_coz(metin, ad, kaynak_araligi)
            .into_iter()
            .any(|aralik| aralik.icerir(satir, sutun))
        {
            return Some(SemantikVarlik::Islem(kimlik));
        }
    }

    for kimlik in hir.yapi_kimlikleri() {
        let yapi = hir.yapi(kimlik)?;
        if yerel_yapi_tanimi_araligi(metin, yapi.satir.checked_sub(1)?, &yapi.ad)
            .is_some_and(|aralik| aralik.icerir(satir, sutun))
        {
            return Some(SemantikVarlik::Yapi(kimlik));
        }
    }
    for (kimlik, kaynak_araligi) in hir.yapi_kullanimlari() {
        let ad = &hir.yapi(kimlik)?.ad;
        if hir_satir_ifadelerini_coz(metin, ad, kaynak_araligi)
            .into_iter()
            .any(|aralik| aralik.icerir(satir, sutun))
        {
            return Some(SemantikVarlik::Yapi(kimlik));
        }
    }
    None
}

/// Dış `None`: belge semantic olarak derlenemedi, tahmin yasak.
/// İç `None`: belge geçerli fakat konum yerel sembol değildir.
fn semantik_tanim_sorgula(
    uri: &str,
    metin: &str,
    satir: usize,
    sutun: usize,
) -> Option<Option<LspKaynakAraligi>> {
    let program = semantik_program_derle(uri, metin)?;
    let Some(varlik) = semantik_varlik_bul(&program, metin, satir, sutun) else {
        return Some(None);
    };
    let hir = program.hir();
    let aralik = match varlik {
        SemantikVarlik::Sembol(kimlik) => {
            let kok = hir.sembol_adi(kimlik)?;
            hir.sembol_tanimi(kimlik)
                .and_then(|aralik| hir_araligini_coz(metin, kok, aralik))
        }
        SemantikVarlik::Islem(kimlik) => {
            let ad = hir.islem_adi(kimlik)?;
            let tanim = hir.islem(kimlik)?;
            yerel_islem_tanimi_araligi(metin, tanim.satir.checked_sub(1)?, ad)
        }
        SemantikVarlik::Yapi(kimlik) => {
            let yapi = hir.yapi(kimlik)?;
            yerel_yapi_tanimi_araligi(metin, yapi.satir.checked_sub(1)?, &yapi.ad)
        }
    };
    Some(aralik)
}

/// Kelimenin kendisi + morfolojik kök adayları (çözümleyiciyle aynı kurallar).
fn adaylar(kelime: &str) -> Vec<String> {
    let mut liste = vec![kelime.to_string()];
    liste.extend(crate::morfoloji::kok_adaylari(kelime));
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

    // 1) işlem / eylem / yapı başlıkları: başlıktaki HERHANGİ bir kelime aday
    //    kökle eşleşirse başlığa gider (işlem adları çok kelimeli olabilir).
    for (no, satir) in metin.lines().enumerate() {
        let kirpik = satir.trim_start();
        if kirpik.starts_with("işlem ")
            || kirpik.starts_with("eylem ")
            || kirpik.starts_with("yapı ")
        {
            if let Some((sutun, uzunluk)) = kelime_konumu(satir, &adaylar) {
                return Some((no, sutun, uzunluk));
            }
        }
    }
    // 2) Değer tanımı (`<ad> ... olsun`) ya da parametre (`<ad>ı al`):
    //    satırın İLK kelimesi aday kökle eşleşmeli.
    for (no, satir) in metin.lines().enumerate() {
        let kirpik = satir.trim_start();
        let Some(ilk) = kirpik.split(' ').next() else {
            continue;
        };
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

/// SymbolId/HIR bağlı yeniden adlandırma (K-120): yalnız seçilen semantic
/// sembolün okuma/yazma aralıklarını değiştirir. Aynı yazımlı başka kapsam,
/// metin sabiti, yorum ve çözümlenemeyen belge için tahmin yapmaz.
fn yeniden_adlandir(
    mesaj: &Json,
    belgeler: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let (uri, satir, sutun) = konum_parametreleri(mesaj)?;
    let yeni_ad = mesaj.alan("params")?.alan("newName")?.metin()?.to_string();
    let metin = belgeler.get(&uri)?;
    let program = semantik_program_derle(&uri, metin)?;
    let varlik = semantik_varlik_bul(&program, metin, satir, sutun)?;
    let hir = program.hir();

    let tek_ad_gecerli = |ad: &str| {
        !ad.is_empty()
            && ad
                .chars()
                .all(|karakter| karakter.is_alphanumeric() || karakter == '_')
    };
    let islem_adi_gecerli = |ad: &str| {
        !ad.is_empty()
            && ad.split(' ').all(tek_ad_gecerli)
            && ad.split_whitespace().collect::<Vec<_>>().join(" ") == ad
    };

    let mut degisiklikler = match varlik {
        SemantikVarlik::Sembol(kimlik) => {
            if !tek_ad_gecerli(&yeni_ad) {
                return None;
            }
            let kok = hir.sembol_adi(kimlik)?;
            hir.sembol_kullanimlari()
                .into_iter()
                .filter(|kullanim| kullanim.kimlik() == kimlik)
                .filter_map(|kullanim| {
                    let aralik = hir_araligini_coz(metin, kok, kullanim.kaynak_araligi())?;
                    let yazim = aralik_metni(metin, aralik)?;
                    let yeni = if yazim == kok {
                        yeni_ad.clone()
                    } else {
                        let ekler = crate::morfoloji::ek_zinciri_coz(&yazim, kok)?;
                        crate::morfoloji::ek_zinciri_uydur(&yeni_ad, &ekler)?
                    };
                    Some((aralik, yeni))
                })
                .collect::<Vec<_>>()
        }
        SemantikVarlik::Islem(kimlik) => {
            if !islem_adi_gecerli(&yeni_ad) {
                return None;
            }
            let ad = hir.islem_adi(kimlik)?;
            let tanim = hir.islem(kimlik)?;
            let mut araliklar = vec![yerel_islem_tanimi_araligi(
                metin,
                tanim.satir.checked_sub(1)?,
                ad,
            )?];
            araliklar.extend(
                hir.islem_kullanimlari()
                    .into_iter()
                    .filter(|(kullanim_kimligi, _)| *kullanim_kimligi == kimlik)
                    .flat_map(|(_, aralik)| hir_satir_ifadelerini_coz(metin, ad, aralik)),
            );
            araliklar
                .into_iter()
                .map(|aralik| (aralik, yeni_ad.clone()))
                .collect()
        }
        SemantikVarlik::Yapi(kimlik) => {
            if !tek_ad_gecerli(&yeni_ad) {
                return None;
            }
            let yapi = hir.yapi(kimlik)?;
            let mut araliklar = vec![yerel_yapi_tanimi_araligi(
                metin,
                yapi.satir.checked_sub(1)?,
                &yapi.ad,
            )?];
            araliklar.extend(
                hir.yapi_kullanimlari()
                    .into_iter()
                    .filter(|(kullanim_kimligi, _)| *kullanim_kimligi == kimlik)
                    .flat_map(|(_, aralik)| hir_satir_ifadelerini_coz(metin, &yapi.ad, aralik)),
            );
            araliklar
                .into_iter()
                .map(|aralik| (aralik, yeni_ad.clone()))
                .collect()
        }
    };
    degisiklikler.sort_by_key(|(aralik, _)| *aralik);
    degisiklikler.dedup_by(|(sol, _), (sag, _)| sol == sag);
    if degisiklikler.is_empty() {
        return None;
    }

    let duzenlemeler = degisiklikler
        .into_iter()
        .map(|(aralik, yeni)| {
            format!(
                "{{\"range\":{{\"start\":{{\"line\":{},\"character\":{}}},\"end\":{{\"line\":{},\"character\":{}}}}},\"newText\":{}}}",
                aralik.satir,
                aralik.bas,
                aralik.satir,
                aralik.bas + aralik.uzunluk,
                json_metin_yaz(&yeni)
            )
        })
        .collect::<Vec<_>>();
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
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{}}}",
        kimlik, sonuc
    )
}

fn ciktilari_sinirla(kimlik: Option<&Json>, ciktilar: &mut Ciktilar) {
    let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.lsp_yanit_bayti();
    let toplam = ciktilar
        .govdeler
        .iter()
        .try_fold(0usize, |toplam, govde| toplam.checked_add(govde.len()));
    if toplam.is_some_and(|toplam| toplam <= azami) {
        return;
    }
    let mesaj = format!("LSP yanıtı güvenli profil {} MiB sınırını aşıyor.", azami / 1024 / 1024);
    ciktilar.govdeler.clear();
    ciktilar.govdeler.push(match kimlik {
        Some(kimlik) => rpc_hatasi(Some(kimlik), -32001, &mesaj),
        None => format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"window/logMessage\",\"params\":{{\"type\":1,\"message\":{}}}}}",
            json_metin_yaz(&mesaj)
        ),
    });
}

fn rpc_hatasi(kimlik: Option<&Json>, kod: i64, mesaj: &str) -> String {
    let kimlik = match kimlik {
        Some(Json::Sayi(s)) => format!("{}", *s as i64),
        Some(Json::Metin(m)) => json_metin_yaz(m),
        _ => "null".to_string(),
    };
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":{},\"error\":{{\"code\":{},\"message\":{}}}}}",
        kimlik,
        kod,
        json_metin_yaz(mesaj)
    )
}

fn kaynak_siniri_bildirimi(uri: &str, mesaj: &str) -> String {
    let tani = Tani::yeni("S045", mesaj.into(), 1, 1, 1)
        .onerili("Kullanılmayan belgeleri kapat veya kaynağı daha küçük birimlere böl.".into());
    format!(
        "{{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\
         \"params\":{{\"uri\":{},\"diagnostics\":[{}]}}}}",
        json_metin_yaz(uri),
        lsp_tanisi(&tani)
    )
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
    uri_yolu(uri)?.parent().map(|p| p.to_path_buf())
}

fn uri_yolu(uri: &str) -> Option<std::path::PathBuf> {
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
    Some(std::path::PathBuf::from(cozulmus))
}

fn proje_kokunu_bul(yol: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut klasor = yol.parent()?;
    loop {
        if klasor.join("proje.dil").is_file() {
            return Some(klasor.to_path_buf());
        }
        klasor = klasor.parent()?;
    }
}

#[cfg(test)]
mod kaynak_siniri_testleri {
    use super::*;

    #[test]
    fn acik_belge_sayisi_sinirli_ve_reddedilen_belge_saklanmaz() {
        let mut sunucu = Sunucu::yeni();
        let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.lsp_acik_belge();
        for sira in 0..azami {
            sunucu
                .belgeyi_guncelle(format!("file:///{}.dil", sira), String::new())
                .expect("sınır içi belge");
        }
        let hata = sunucu
            .belgeyi_guncelle("file:///fazla.dil".into(), String::new())
            .expect_err("fazla belge reddedilmeli");
        assert!(hata.contains("açık belge"));
        assert_eq!(sunucu.belgeler.len(), azami);
    }

    #[test]
    fn buyuk_lsp_yaniti_json_rpc_hatasina_donusur() {
        let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.lsp_yanit_bayti();
        let mut ciktilar = Ciktilar {
            govdeler: vec!["x".repeat(azami + 1)],
            devam: true,
        };
        ciktilari_sinirla(Some(&Json::Sayi(7.0)), &mut ciktilar);
        assert_eq!(ciktilar.govdeler.len(), 1);
        assert!(ciktilar.govdeler[0].contains("\"code\":-32001"));
        assert!(ciktilar.govdeler[0].contains("\"id\":7"));
    }
}
