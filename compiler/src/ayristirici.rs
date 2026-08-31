//! Ayrıştırıcı (parser) — elle yazılmış recursive descent (ADR-002 adayı).
//!
//! Türkçe yüklem-sonlu olduğu için cümlenin türü satırın SON kelimesinden
//! anlaşılır: "... yaz", "... olsun", "... tekrarla", "... ise". Bu, İngilizce
//! dillerdeki "ilk keyword'e bak" yaklaşımının aynadaki karşılığıdır ve
//! deterministik ayrıştırmayı mümkün kılar.

use crate::agac::{AritmetikIslec, Cumle, Ifade, Islec, Islem, KosulKolu, Ozellik, Test, Yapi};
use crate::sozcukleyici::{Token, TokenTur};
use crate::tani::Tani;

pub fn ayristir(tokenlar: Vec<Token>) -> Result<Vec<Cumle>, Tani> {
    let mut ayristirici = Ayristirici {
        tokenlar,
        konum: 0,
        derinlik: 0,
        islem_adlari: Vec::new(),
    };
    let program = ayristirici.blok_ayristir()?;
    ayristirici.bekle_dosya_sonu()?;
    Ok(program)
}

struct Ayristirici {
    tokenlar: Vec<Token>,
    konum: usize,
    /// Blok derinliği: işlem tanımları yalnız en dış düzeyde.
    derinlik: usize,
    /// Şimdiye dek tanımlanan işlem adları — çağrılar bunlarla eşlenir.
    /// v0 kuralı: işlem, çağrılmadan ÖNCE tanımlanmış olmalı.
    islem_adlari: Vec<String>,
}

/// Sayı/sabit sonrası ayrık yazılan hal ekleri (K-011): "1 den", "100 e", "0 dan".
const AYRIK_EKLER: [&str; 8] = ["den", "dan", "ten", "tan", "e", "a", "ye", "ya"];

impl Ayristirici {
    fn bak(&self) -> &Token {
        &self.tokenlar[self.konum]
    }

    fn ilerle(&mut self) -> Token {
        let token = self.tokenlar[self.konum].clone();
        if self.konum + 1 < self.tokenlar.len() {
            self.konum += 1;
        }
        token
    }

    fn bekle_dosya_sonu(&mut self) -> Result<(), Tani> {
        match self.bak().tur {
            TokenTur::DosyaSonu => Ok(()),
            _ => {
                let t = self.bak().clone();
                Err(Tani::yeni("S004", "Dosya sonunda beklenmeyen içerik.".into(), t.satir, t.sutun, t.uzunluk))
            }
        }
    }

    /// Bir satırın tokenlarını (SatirSonu hariç) toplar.
    fn satir_oku(&mut self) -> Vec<Token> {
        let mut satir = Vec::new();
        loop {
            match self.bak().tur {
                TokenTur::SatirSonu => {
                    self.ilerle();
                    break;
                }
                TokenTur::DosyaSonu => break,
                _ => satir.push(self.ilerle()),
            }
        }
        satir
    }

    /// Girintili alt bloğu ayrıştırır (başındaki Girinti ve sonundaki Cikinti dahil).
    fn alt_blok(&mut self, ana_satir: usize) -> Result<Vec<Cumle>, Tani> {
        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "Bu satırdan sonra girintili bir blok bekleniyor.".into(),
                    ana_satir,
                    1,
                    1,
                )
                .onerili("Alt satırları 4 boşluk içeriden yaz.".into()))
            }
        }
        self.derinlik += 1;
        let govde = self.blok_ayristir()?;
        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }
        Ok(govde)
    }

    /// Aynı girinti düzeyindeki cümleleri ayrıştırır.
    fn blok_ayristir(&mut self) -> Result<Vec<Cumle>, Tani> {
        let mut cumleler = Vec::new();
        loop {
            match self.bak().tur {
                TokenTur::Cikinti | TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu => {
                    self.ilerle();
                }
                _ => {
                    let cumle = self.cumle_ayristir()?;
                    cumleler.push(cumle);
                }
            }
        }
        Ok(cumleler)
    }

    fn cumle_ayristir(&mut self) -> Result<Cumle, Tani> {
        // "işlem ..." ve "yapı ..." satırları ilk kelimesinden tanınır
        // (tanım başlıkları, yüklem değil).
        if let TokenTur::Kelime(k) = &self.bak().tur {
            if k == "işlem" {
                return self.islem_ayristir();
            }
            if k == "yapı" {
                return self.yapi_ayristir();
            }
            if k == "test" {
                return self.test_ayristir();
            }
        }

        let satir_tokenlari = self.satir_oku();
        let satir_no = satir_tokenlari.first().map(|t| t.satir).unwrap_or(1);
        let son_kelime = son_kelime(&satir_tokenlari);

        match son_kelime.as_deref() {
            Some("yaz") => self.yaz_ayristir(satir_tokenlari, satir_no),
            Some("olsun") => self.olsun_ayristir(satir_tokenlari, satir_no),
            Some("tekrarla") => self.tekrarla_ayristir(satir_tokenlari, satir_no),
            Some("için") => self.aralik_ayristir(satir_tokenlari, satir_no),
            Some("sürece") => self.surece_ayristir(satir_tokenlari, satir_no),
            Some("artır") => self.artir_azalt_ayristir(satir_tokenlari, satir_no, true),
            Some("azalt") => self.artir_azalt_ayristir(satir_tokenlari, satir_no, false),
            Some("sor") => self.sor_ayristir(satir_tokenlari, satir_no),
            Some("ekle") => self.ekle_ayristir(satir_tokenlari, satir_no),
            Some("döndür") => self.dondur_ayristir(satir_tokenlari, satir_no),
            Some("böl") => self.bol_ayristir(satir_tokenlari, satir_no),
            Some("göre") => self.gore_ayristir(satir_tokenlari, satir_no),
            Some("olmalı") => self.olmali_ayristir(satir_tokenlari, satir_no),
            Some("bitir") => {
                if satir_tokenlari.len() == 2 && kelime_mi(&satir_tokenlari[0], "programı") {
                    Ok(Cumle::ProgramiBitir { satir: satir_no })
                } else {
                    Err(Tani::yeni(
                        "S027",
                        "Sonlandırma \"programı bitir\" biçiminde yazılır.".into(),
                        satir_no,
                        1,
                        1,
                    ))
                }
            }
            // Satır sonundaki "değilse": olumsuzlanmış koşul başlığı
            // ("x 5 e eşit değilse"). Tek başına "değilse" ise başıboş else'tir.
            Some("değilse") => {
                if satir_tokenlari.len() == 1 {
                    Err(Tani::yeni(
                        "S031",
                        "\"değilse\" tek başına duramaz.".into(),
                        satir_no,
                        1,
                        1,
                    )
                    .onerili("\"değilse\" bir \"... ise\" bloğunun hemen ardından, aynı hizada gelir.".into()))
                } else {
                    self.ise_ayristir(satir_tokenlari, satir_no)
                }
            }
            Some(k) if kosul_kelimesi(k) => self.ise_ayristir(satir_tokenlari, satir_no),
            _ => {
                // Tanımlı bir işlem adına biten satır → çağrı cümlesi.
                if let Some(cagri) =
                    cagri_kalibi(&satir_tokenlari, satir_no, &self.islem_adlari)?
                {
                    return Ok(Cumle::CagriCumlesi { cagri, satir: satir_no });
                }
                let ilk = satir_tokenlari.first().cloned();
                let (sutun, uzunluk) = ilk.map(|t| (t.sutun, t.uzunluk)).unwrap_or((1, 1));
                Err(Tani::yeni(
                    "S004",
                    "Bu cümle tanınmadı: cümleler eylemle biter.".into(),
                    satir_no,
                    sutun,
                    uzunluk,
                )
                .onerili(
                    "Desteklenen kalıplar: \"... yaz\", \"<ad> ... olsun\", \"<n> kez tekrarla\", \
                     \"<a> den <b> e kadar her <ad> için\", \"... olduğu sürece\", \"... ise / değilse\", \
                     \"<ad> <n> artır/azalt\", \"işlem <ad>\", \"... döndür\", işlem çağrısı. \
                     Çağrılan işlem daha önce tanımlanmış olmalı."
                        .into(),
                ))
            }
        }
    }

    /// `işlem <çok kelimeli ad>` + gövde. Gövdenin başındaki "X al" satırları
    /// parametre bildirimidir.
    fn islem_ayristir(&mut self) -> Result<Cumle, Tani> {
        let mut baslik = self.satir_oku();
        let satir = baslik.first().map(|t| t.satir).unwrap_or(1);
        if self.derinlik > 0 {
            return Err(Tani::yeni(
                "S021",
                "İşlem tanımı en dış düzeyde olmalı.".into(),
                satir,
                1,
                1,
            ));
        }
        baslik.remove(0); // "işlem"
        let mut ad_kelimeleri = Vec::new();
        for token in &baslik {
            match &token.tur {
                TokenTur::Kelime(k) => ad_kelimeleri.push(k.clone()),
                _ => {
                    return Err(Tani::yeni(
                        "S022",
                        "İşlem adı yalnız kelimelerden oluşur.".into(),
                        token.satir,
                        token.sutun,
                        token.uzunluk,
                    ))
                }
            }
        }
        if ad_kelimeleri.is_empty() {
            return Err(Tani::yeni(
                "S022",
                "İşlemin bir adı olmalı.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: işlem ortalamayı hesapla".into()));
        }
        let ad = ad_kelimeleri.join(" ");
        self.islem_adlari.push(ad.clone());

        // Gövde.
        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "İşlem başlığından sonra girintili bir gövde bekleniyor.".into(),
                    satir,
                    1,
                    1,
                ))
            }
        }
        self.derinlik += 1;

        // Parametre satırları: tam olarak `<ad> al`.
        let mut parametreler = Vec::new();
        loop {
            let param = match (&self.tokenlar.get(self.konum), &self.tokenlar.get(self.konum + 1)) {
                (Some(a), Some(b)) => match (&a.tur, &b.tur) {
                    (TokenTur::Kelime(ad), TokenTur::Kelime(al)) if al == "al" => {
                        match self.tokenlar.get(self.konum + 2).map(|t| &t.tur) {
                            Some(TokenTur::SatirSonu) => Some(yalin_ad(ad)),
                            _ => None,
                        }
                    }
                    _ => None,
                },
                _ => None,
            };
            match param {
                Some(p) => {
                    parametreler.push(p);
                    self.satir_oku();
                }
                None => break,
            }
        }

        let govde = self.blok_ayristir()?;
        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }

        Ok(Cumle::IslemTanimi(Islem { ad, parametreler, govde, satir }))
    }

    /// `yapı <Ad>` + alan satırları (`ad Metin`, `yaş TamSayı`).
    fn yapi_ayristir(&mut self) -> Result<Cumle, Tani> {
        let baslik = self.satir_oku();
        let satir = baslik.first().map(|t| t.satir).unwrap_or(1);
        if self.derinlik > 0 {
            return Err(Tani::yeni(
                "S021",
                "Yapı tanımı en dış düzeyde olmalı.".into(),
                satir,
                1,
                1,
            ));
        }
        let ad = match baslik.get(1).map(|t| &t.tur) {
            Some(TokenTur::Kelime(ad)) if baslik.len() == 2 => ad.clone(),
            _ => {
                return Err(Tani::yeni(
                    "S025",
                    "Yapı tanımı \"yapı <Ad>\" biçiminde başlar.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: yapı Öğrenci — tür adları büyük harfle başlar.".into()))
            }
        };

        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "Yapı başlığından sonra girintili alan listesi bekleniyor.".into(),
                    satir,
                    1,
                    1,
                ))
            }
        }
        self.derinlik += 1;

        let mut alanlar = Vec::new();
        loop {
            match &self.bak().tur {
                TokenTur::Cikinti | TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu => {
                    self.ilerle();
                }
                _ => {
                    let alan_satiri = self.satir_oku();
                    let alan_no = alan_satiri.first().map(|t| t.satir).unwrap_or(satir);
                    match (alan_satiri.first().map(|t| &t.tur), alan_satiri.get(1).map(|t| &t.tur)) {
                        (Some(TokenTur::Kelime(alan)), Some(TokenTur::Kelime(tur)))
                            if alan_satiri.len() == 2 =>
                        {
                            alanlar.push((alan.clone(), tur.clone()));
                        }
                        _ => {
                            return Err(Tani::yeni(
                                "S025",
                                "Yapı alanı \"<ad> <Tür>\" biçiminde yazılır.".into(),
                                alan_no,
                                1,
                                1,
                            )
                            .onerili("Örnek: yaş TamSayı".into()))
                        }
                    }
                }
            }
        }

        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }
        Ok(Cumle::YapiTanimi(Yapi { ad, alanlar, satir }))
    }

    /// `test "<açıklama>"` + gövde (K-025).
    fn test_ayristir(&mut self) -> Result<Cumle, Tani> {
        let baslik = self.satir_oku();
        let satir = baslik.first().map(|t| t.satir).unwrap_or(1);
        if self.derinlik > 0 {
            return Err(Tani::yeni(
                "S021",
                "Test bloğu en dış düzeyde olmalı.".into(),
                satir,
                1,
                1,
            ));
        }
        let ad = match baslik.get(1).map(|t| &t.tur) {
            Some(TokenTur::Metin(ad)) if baslik.len() == 2 => ad.clone(),
            _ => {
                return Err(Tani::yeni(
                    "S026",
                    "Test bloğu \"test \\\"<açıklama>\\\"\" biçiminde başlar.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: test \"toplama doğru çalışır\"".into()))
            }
        };
        let govde = self.alt_blok(satir)?;
        Ok(Cumle::TestBlogu(Test { ad, govde, satir }))
    }

    /// `<koşul> olmalı` — doğrulama cümlesi (K-025).
    fn olmali_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "olmalı"
        let kosul = kosul_ifadesi(&tokenlar, satir)?;
        Ok(Cumle::Olmali { kosul, satir })
    }

    fn dondur_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "döndür"
        let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Dondur { deger, satir })
    }

    /// `sonucu toplamı sayıların adedine böl`
    fn bol_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "böl"
        if tokenlar.len() < 3 {
            return Err(Tani::yeni(
                "S023",
                "Bölme cümlesi \"<hedef> <pay> <payda> böl\" biçiminde yazılır.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: sonucu toplamı sayıların adedine böl".into()));
        }
        let hedef_token = tokenlar.remove(0);
        let hedef = match hedef_token.tur {
            TokenTur::Kelime(k) => yalin_ad(&k),
            _ => {
                return Err(Tani::yeni(
                    "S023",
                    "Bölmenin hedefi bir ad olmalı.".into(),
                    satir,
                    hedef_token.sutun,
                    hedef_token.uzunluk,
                ))
            }
        };
        let pay = tekil_ifade(tokenlar.remove(0))?;
        let payda = bolge_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::BolVeAta { hedef, pay, payda, satir })
    }

    // ---- cümle türleri ----

    fn yaz_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "yaz"

        // Hedefli yazma (K-019): `"X" dosyasına ... yaz`.
        if tokenlar.len() >= 3 && kelime_mi(&tokenlar[1], "dosyasına") {
            let yol = tekil_ifade(tokenlar[0].clone())?;
            let icerik = ile_ifadesi(&tokenlar[2..], satir, &self.islem_adlari)?;
            return Ok(Cumle::DosyayaYaz { yol, icerik, ekleme: false, satir });
        }

        let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Yaz { deger, satir })
    }

    fn olsun_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "olsun"

        // Sözlüğe yazma: `yaşların "Ayşe" değeri 10 olsun` (K-015).
        if tokenlar.len() >= 4 && kelime_mi(&tokenlar[2], "değeri") {
            let sozluk = tekil_ifade(tokenlar[0].clone())?;
            let anahtar = tekil_ifade(tokenlar[1].clone())?;
            let deger = ile_ifadesi(&tokenlar[3..], satir, &self.islem_adlari)?;
            return Ok(Cumle::SozlukAta { sozluk, anahtar, deger, satir });
        }

        // Alan yazma: `ayşenin adı "Ayşe" olsun` (K-020). Kural: tam üç token,
        // ilki tamlayan ekli bir ad, ikincisi alan adı, üçüncüsü tek değer —
        // AMA kuyruk `yanıtın sayısı` gibi yapılı bir kalıpsa bu normal bir
        // değer tanımıdır ("tahmin yanıtın sayısı olsun"); kalıplar önce gelir.
        if tokenlar.len() == 3 {
            if let (TokenTur::Kelime(nesne), TokenTur::Kelime(alan)) =
                (&tokenlar[0].tur, &tokenlar[1].tur)
            {
                let kuyruk_kalip = yapili_kalip(&tokenlar[1..], &self.islem_adlari)?.is_some();
                if !kuyruk_kalip && tamlayan_ekli(nesne) && alan != "ile" {
                    let nesne = tekil_ifade(tokenlar[0].clone())?;
                    let alan = alan.clone();
                    let deger = tekil_ifade(tokenlar[2].clone())?;
                    return Ok(Cumle::AlanAta { nesne, alan, deger, satir });
                }
            }
        }

        if tokenlar.len() < 2 {
            return Err(Tani::yeni(
                "S008",
                "\"olsun\" ile değer tanımı için ad ve değer gerekir.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: isim \"Ayşe\" olsun".into()));
        }
        let ad_token = tokenlar.remove(0);
        let (ad, sutun, uzunluk) = match ad_token.tur {
            TokenTur::Kelime(k) => (k, ad_token.sutun, ad_token.uzunluk),
            _ => {
                return Err(Tani::yeni(
                    "S008",
                    "Değer tanımı bir adla başlamalı.".into(),
                    satir,
                    ad_token.sutun,
                    ad_token.uzunluk,
                )
                .onerili("Örnek: yaş 10 olsun".into()))
            }
        };
        let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Olsun { ad, deger, satir, sutun, uzunluk })
    }

    fn tekrarla_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "tekrarla"

        // <koşul> olana kadar tekrarla
        let n = tokenlar.len();
        if n >= 3
            && kelime_mi(&tokenlar[n - 1], "kadar")
            && kelime_mi(&tokenlar[n - 2], "olana")
        {
            tokenlar.truncate(n - 2);
            let kosul = kosul_ifadesi(&tokenlar, satir)?;
            let govde = self.alt_blok(satir)?;
            return Ok(Cumle::OlanaKadar { kosul, govde, satir });
        }

        // <ifade> kez tekrarla
        match tokenlar.last() {
            Some(t) if kelime_mi(t, "kez") => {
                tokenlar.pop();
            }
            _ => {
                return Err(Tani::yeni(
                    "S009",
                    "Döngü \"<n> kez tekrarla\" ya da \"<koşul> olana kadar tekrarla\" biçiminde yazılır.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnekler: 10 kez tekrarla · bildi doğru olana kadar tekrarla".into()))
            }
        }
        let adet = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        let govde = self.alt_blok(satir)?;
        Ok(Cumle::KezTekrarla { adet, govde, satir })
    }

    fn aralik_ayristir(&mut self, tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        // <koleksiyon>daki her <ad> için — açık kaynaklı döngü
        if tokenlar.len() == 4 && kelime_mi(&tokenlar[1], "her") {
            if let TokenTur::Kelime(kaynakli) = &tokenlar[0].tur {
                let kok = ["daki", "deki", "taki", "teki"]
                    .iter()
                    .find_map(|ek| kaynakli.strip_suffix(ek));
                if let (Some(kok), TokenTur::Kelime(ad)) = (kok, &tokenlar[2].tur) {
                    let t = &tokenlar[0];
                    let kaynak = Ifade::Degisken {
                        ham: kok.to_string(),
                        cozulmus: None,
                        satir: t.satir,
                        sutun: t.sutun,
                        uzunluk: t.uzunluk,
                    };
                    let ad = ad.clone();
                    let govde = self.alt_blok(satir)?;
                    return Ok(Cumle::HerBiri { ad, kaynak: Some(kaynak), govde, satir });
                }
            }
        }

        // her <ad> için — koleksiyon döngüsü (örtük çoğul, K-013)
        if tokenlar.len() == 3 && kelime_mi(&tokenlar[0], "her") {
            let ad = match &tokenlar[1].tur {
                TokenTur::Kelime(k) => k.clone(),
                _ => {
                    return Err(Tani::yeni(
                        "S010",
                        "\"her ... için\" döngüsünde bir ad bekleniyor.".into(),
                        satir,
                        1,
                        1,
                    ))
                }
            };
            let govde = self.alt_blok(satir)?;
            return Ok(Cumle::HerBiri { ad, kaynak: None, govde, satir });
        }

        // Beklenen biçim: <a> den <b> e kadar her <ad> için
        let hata = || {
            Tani::yeni(
                "S010",
                "Döngü tanınmadı.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnekler: her sayı için · 1 den 100 e kadar her sayı için".into())
        };
        let mut t = tokenlar.into_iter().peekable();

        let bastan = tekil_ifade(t.next().ok_or_else(hata)?)?;
        eslesen_ek(&mut t).ok_or_else(hata)?; // den/dan
        let sona = tekil_ifade(t.next().ok_or_else(hata)?)?;
        eslesen_ek(&mut t).ok_or_else(hata)?; // e/a
        bekle_kelime(&mut t, "kadar").ok_or_else(hata)?;
        bekle_kelime(&mut t, "her").ok_or_else(hata)?;
        let ad = match t.next().map(|x| x.tur) {
            Some(TokenTur::Kelime(k)) => k,
            _ => return Err(hata()),
        };
        bekle_kelime(&mut t, "için").ok_or_else(hata)?;
        if t.next().is_some() {
            return Err(hata());
        }

        let govde = self.alt_blok(satir)?;
        Ok(Cumle::AralikDongusu { ad, bastan, sona, govde, satir })
    }

    fn surece_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "sürece"
        match tokenlar.last() {
            Some(t) if kelime_mi(t, "olduğu") => {
                tokenlar.pop();
            }
            _ => {
                return Err(Tani::yeni(
                    "S011",
                    "Koşullu döngü \"<koşul> olduğu sürece\" biçiminde yazılır.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: sayaç 0 dan büyük olduğu sürece".into()))
            }
        }
        let kosul = kosul_ifadesi(&tokenlar, satir)?;
        let govde = self.alt_blok(satir)?;
        Ok(Cumle::OlduguSurece { kosul, govde, satir })
    }

    fn artir_azalt_ayristir(
        &mut self,
        mut tokenlar: Vec<Token>,
        satir: usize,
        artir: bool,
    ) -> Result<Cumle, Tani> {
        tokenlar.pop(); // eylem
        if tokenlar.len() < 2 {
            return Err(Tani::yeni(
                "S012",
                "Artırma/azaltma için hedef ve miktar gerekir.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: toplamı sayıyla artır — ya da: sayacı 1 azalt".into()));
        }
        let hedef_token = tokenlar.remove(0);
        let ifade = tekil_ifade(hedef_token)?;
        let miktar = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        if artir {
            Ok(Cumle::Artir { ifade, miktar, satir })
        } else {
            Ok(Cumle::Azalt { ifade, miktar, satir })
        }
    }

    fn ekle_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "ekle"

        // Dosyanın sonuna ekleme (K-019): `"X" dosyasına ... ekle`.
        if tokenlar.len() >= 3 && kelime_mi(&tokenlar[1], "dosyasına") {
            let yol = tekil_ifade(tokenlar[0].clone())?;
            let icerik = ile_ifadesi(&tokenlar[2..], satir, &self.islem_adlari)?;
            return Ok(Cumle::DosyayaYaz { yol, icerik, ekleme: true, satir });
        }

        if tokenlar.len() < 2 {
            return Err(Tani::yeni(
                "S018",
                "Ekleme için hedef liste ve değer gerekir.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: sayılara 5 ekle".into()));
        }
        let hedef = tekil_ifade(tokenlar.remove(0))?;
        let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Ekle { hedef, deger, satir })
    }

    /// `<konu> a göre` — desen eşleştirme başlığı (K-021). Gövde kolları
    /// `<değer> ise` başlıklı bloklardır; varsayılan kol `değilse`.
    fn gore_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "göre"
        // Yönelme eki konuya bitişik ("şekle") ya da sabit sonrası ayrık olabilir.
        if let Some(t) = tokenlar.last() {
            if matches!(&t.tur, TokenTur::Kelime(k) if AYRIK_EKLER.contains(&k.as_str())) {
                tokenlar.pop();
            }
        }
        if tokenlar.len() != 1 {
            return Err(Tani::yeni(
                "S024",
                "Eşleştirme \"<konu> a göre\" biçiminde başlar.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: şekle göre".into()));
        }
        let konu = tekil_ifade(tokenlar.pop().unwrap())?;

        // Gövde: kollar.
        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "\"göre\" başlığından sonra girintili kollar bekleniyor.".into(),
                    satir,
                    1,
                    1,
                ))
            }
        }
        self.derinlik += 1;

        let mut kollar = Vec::new();
        let mut degilse = None;
        loop {
            match &self.bak().tur {
                TokenTur::Cikinti | TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu => {
                    self.ilerle();
                }
                _ => {
                    let kol_satiri = self.satir_oku();
                    let kol_no = kol_satiri.first().map(|t| t.satir).unwrap_or(satir);
                    if kol_satiri.len() == 1 && kelime_mi(&kol_satiri[0], "değilse") {
                        degilse = Some(self.alt_blok(kol_no)?);
                        continue;
                    }
                    if kol_satiri.len() == 2 && kelime_mi(&kol_satiri[1], "ise") {
                        let deger = tekil_ifade(kol_satiri[0].clone())?;
                        let govde = self.alt_blok(kol_no)?;
                        kollar.push((deger, govde));
                        continue;
                    }
                    return Err(Tani::yeni(
                        "S024",
                        "Eşleştirme kolu \"<değer> ise\" ya da \"değilse\" olmalı.".into(),
                        kol_no,
                        1,
                        1,
                    )
                    .onerili("Örnek:\n    \"kare\" ise\n        \"4 köşesi var\" yaz".into()));
                }
            }
        }

        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }
        Ok(Cumle::Gore { konu, kollar, degilse, satir })
    }

    fn sor_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "sor"
        match tokenlar.last() {
            Some(t) if kelime_mi(t, "diye") => {
                tokenlar.pop();
            }
            _ => {
                return Err(Tani::yeni(
                    "S017",
                    "Soru \"<metin> diye sor\" biçiminde yazılır.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: \"Adın ne?\" diye sor — cevap \"yanıt\" adıyla kullanılır.".into()))
            }
        }
        let istem = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Sor { istem, satir })
    }

    fn ise_ayristir(&mut self, tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        let kosul = kosul_ifadesi(&tokenlar, satir)?;
        let govde = self.alt_blok(satir)?;
        let mut kollar = vec![KosulKolu { kosul, govde }];
        let mut degilse = None;

        // "değilse" zinciri: aynı düzeyde peş peşe gelebilir.
        loop {
            match &self.bak().tur {
                TokenTur::Kelime(k) if k == "değilse" => {
                    let devam_satiri = self.bak().satir;
                    let mut satir_tokenlari = self.satir_oku();
                    satir_tokenlari.remove(0); // "değilse"
                    if satir_tokenlari.is_empty() {
                        degilse = Some(self.alt_blok(devam_satiri)?);
                        break;
                    } else {
                        let kosul = kosul_ifadesi(&satir_tokenlari, devam_satiri)?;
                        let govde = self.alt_blok(devam_satiri)?;
                        kollar.push(KosulKolu { kosul, govde });
                    }
                }
                _ => break,
            }
        }

        Ok(Cumle::Ise { kollar, degilse, satir })
    }
}

// ---- yardımcılar ----

fn son_kelime(tokenlar: &[Token]) -> Option<String> {
    match tokenlar.last().map(|t| &t.tur) {
        Some(TokenTur::Kelime(k)) => Some(k.clone()),
        _ => None,
    }
}

fn kelime_mi(token: &Token, beklenen: &str) -> bool {
    matches!(&token.tur, TokenTur::Kelime(k) if k == beklenen)
}

/// Satır sonundaki kelime bir koşul yüklemi mi? ("büyükse", "çiftse", ...)
fn kosul_kelimesi(kelime: &str) -> bool {
    matches!(
        kelime,
        "büyükse"
            | "küçükse"
            | "eşitse"
            | "çiftse"
            | "tekse"
            | "varsa"
            | "yoksa"
            | "içeriyorsa"
            | "başarılıysa"
            | "başarısızsa"
            | "boşsa"
            | "doluysa"
    )
}

fn eslesen_ek<I: Iterator<Item = Token>>(t: &mut std::iter::Peekable<I>) -> Option<()> {
    match t.next().map(|x| x.tur) {
        Some(TokenTur::Kelime(k)) if AYRIK_EKLER.contains(&k.as_str()) => Some(()),
        _ => None,
    }
}

fn bekle_kelime<I: Iterator<Item = Token>>(t: &mut std::iter::Peekable<I>, beklenen: &str) -> Option<()> {
    match t.next().map(|x| x.tur) {
        Some(TokenTur::Kelime(k)) if k == beklenen => Some(()),
        _ => None,
    }
}

/// Tek token'dan ifade üretir (sabit ya da değişken).
fn tekil_ifade(token: Token) -> Result<Ifade, Tani> {
    match token.tur {
        TokenTur::Metin(m) => Ok(Ifade::MetinSabiti(m)),
        TokenTur::TamSayi(s) => Ok(Ifade::SayiSabiti(s)),
        TokenTur::Kelime(k) if k == "doğru" => Ok(Ifade::MantiksalSabiti(true)),
        TokenTur::Kelime(k) if k == "yanlış" => Ok(Ifade::MantiksalSabiti(false)),
        TokenTur::Kelime(k) if k == "yok" => Ok(Ifade::YokSabiti),
        TokenTur::Kelime(k) => Ok(Ifade::Degisken {
            ham: k,
            cozulmus: None,
            satir: token.satir,
            sutun: token.sutun,
            uzunluk: token.uzunluk,
        }),
        _ => Err(Tani::yeni(
            "S013",
            "Burada bir değer bekleniyor.".into(),
            token.satir,
            token.sutun,
            token.uzunluk,
        )),
    }
}

/// Tamlayan (genitif) ayrık ekleri: "10 un", "3 ün".
const TAMLAYAN_EKLER: [&str; 8] = ["nın", "nin", "nun", "nün", "ın", "in", "un", "ün"];

/// İfade bölgesi: önce yapılı kalıplar (aritmetik genitif, "…ın sayısı"),
/// bulunamazsa "ile" zinciri.
fn ile_ifadesi(tokenlar: &[Token], satir: usize, islemler: &[String]) -> Result<Ifade, Tani> {
    if tokenlar.is_empty() {
        return Err(Tani::yeni("S013", "Burada bir değer bekleniyor.".into(), satir, 1, 1));
    }

    if let Some(ifade) = yapili_kalip(tokenlar, islemler)? {
        return Ok(ifade);
    }

    // "ile" üzerinden parçalara böl; her parça kendi başına bir bölgedir
    // (tek token ya da yapılı kalıp: "sayıların adedi" gibi).
    let mut parcalar: Vec<Ifade> = Vec::new();
    let mut bolge: Vec<Token> = Vec::new();
    for token in tokenlar {
        if kelime_mi(token, "ile") {
            if bolge.is_empty() {
                return Err(Tani::yeni(
                    "S014",
                    "\"ile\"den önce bir değer olmalı.".into(),
                    token.satir,
                    token.sutun,
                    token.uzunluk,
                ));
            }
            parcalar.push(bolge_ifadesi(&bolge, satir, islemler)?);
            bolge.clear();
        } else {
            bolge.push(token.clone());
        }
    }
    if bolge.is_empty() {
        return Err(Tani::yeni(
            "S014",
            "\"ile\"den sonra bir değer olmalı.".into(),
            satir,
            1,
            1,
        ));
    }
    parcalar.push(bolge_ifadesi(&bolge, satir, islemler)?);

    if parcalar.len() == 1 {
        Ok(parcalar.pop().unwrap())
    } else {
        Ok(Ifade::Birlestir(parcalar))
    }
}

/// Tek "ile" parçası: tek token ya da yapılı kalıp.
fn bolge_ifadesi(tokenlar: &[Token], _satir: usize, islemler: &[String]) -> Result<Ifade, Tani> {
    if tokenlar.len() == 1 {
        return tekil_ifade(tokenlar[0].clone());
    }
    if let Some(ifade) = yapili_kalip(tokenlar, islemler)? {
        return Ok(ifade);
    }
    let ikinci = &tokenlar[1];
    Err(Tani::yeni(
        "S015",
        "İki değer yan yana geldi; aralarına \"ile\" koy ya da bilinen bir kalıp kullan.".into(),
        ikinci.satir,
        ikinci.sutun,
        ikinci.uzunluk,
    )
    .onerili(format!(
        "Örnek: {} ile {} — ya da: sayıların adedi, a ile b nin toplamı",
        goster(&tokenlar[0]),
        goster(ikinci)
    )))
}

/// Koşul ifadesi: önce ve/veya zinciri ayrılır, parçalar atomik koşuldur.
///
/// K-027 kuralı: `A ve B ve C` ya da `A veya B` serbesttir; ve/veya KARIŞIMI
/// parantezsiz belirsiz olduğundan hatadır (S030) — kullanıcı koşulu böler.
/// "veya daha" ikilisi karşılaştırma kalıbına aittir ("90 veya daha büyükse"),
/// zincir ayracı sayılmaz.
fn kosul_ifadesi(tokenlar: &[Token], satir: usize) -> Result<Ifade, Tani> {
    let mut baglac: Option<bool> = None; // true = ve, false = veya
    let mut bolgeler: Vec<&[Token]> = Vec::new();
    let mut baslangic = 0usize;

    for i in 0..tokenlar.len() {
        let bu = match &tokenlar[i].tur {
            TokenTur::Kelime(k) if k == "ve" => Some(true),
            TokenTur::Kelime(k) if k == "veya" => {
                let sonraki_daha = matches!(
                    tokenlar.get(i + 1).map(|t| &t.tur),
                    Some(TokenTur::Kelime(d)) if d == "daha"
                );
                if sonraki_daha {
                    None
                } else {
                    Some(false)
                }
            }
            _ => None,
        };
        if let Some(bu) = bu {
            match baglac {
                Some(onceki) if onceki != bu => {
                    return Err(Tani::yeni(
                        "S030",
                        "\"ve\" ile \"veya\" aynı koşulda karıştırılamaz: hangisinin önce \
                         geleceği belirsiz olur."
                            .into(),
                        tokenlar[i].satir,
                        tokenlar[i].sutun,
                        tokenlar[i].uzunluk,
                    )
                    .onerili(
                        "Koşulu ayrı \"ise\" basamaklarına böl ya da tek tür bağlaç kullan."
                            .into(),
                    ));
                }
                _ => baglac = Some(bu),
            }
            bolgeler.push(&tokenlar[baslangic..i]);
            baslangic = i + 1;
        }
    }
    bolgeler.push(&tokenlar[baslangic..]);

    if bolgeler.len() == 1 {
        return kosul_atomu(tokenlar, satir);
    }
    let mut parcalar = Vec::new();
    for bolge in &bolgeler {
        if bolge.is_empty() {
            return Err(Tani::yeni(
                "S030",
                "Bağlacın iki yanında da bir koşul olmalı.".into(),
                satir,
                1,
                1,
            ));
        }
        parcalar.push(kosul_atomu(bolge, satir)?);
    }
    Ok(Ifade::MantiksalZincir {
        hepsi: baglac.unwrap_or(true),
        parcalar,
    })
}

/// Atomik koşul: yüklem sondadır.
///
/// Desteklenen kalıplar (K-010):
///   X Y veya daha büyükse   → X >= Y
///   X Y veya daha küçükse   → X <= Y
///   X Y den büyükse         → X > Y     (den/dan)
///   X Y den küçükse         → X < Y
///   X Y e eşitse            → X == Y    (e/a/ye/ya)
///   X çiftse / X tekse
///   ... değilse             → olumsuzlama: "x 5 e eşit değilse", "bildi doğru değilse"
/// "olduğu sürece" içinde yüklem çıplak gelir: "büyük", "küçük", "eşit".
fn kosul_atomu(tokenlar: &[Token], satir: usize) -> Result<Ifade, Tani> {
    let hata = || {
        Tani::yeni("S016", "Koşul tanınmadı.".into(), satir, 1, 1).onerili(
            "Örnekler: yaş 8 veya daha büyükse · puan 50 den küçükse · sayı 5 e eşitse · sayı çiftse"
                .into(),
        )
    };

    // "... değilse" olumsuzlaması: içteki koşul olumlu biçimiyle ayrıştırılır.
    // "x 5 e eşit değilse" → içerideki "x 5 e eşit" çıplak yüklem kalıbıdır.
    if tokenlar.len() >= 2 {
        if let TokenTur::Kelime(k) = &tokenlar[tokenlar.len() - 1].tur {
            if k == "değilse" {
                let kalan = &tokenlar[..tokenlar.len() - 1];
                let ic = if kalan.len() == 1 {
                    // "bayrak değilse" — Mantıksal değerin doğrudan olumsuzu.
                    tekil_ifade(kalan[0].clone())?
                } else {
                    kosul_atomu(kalan, satir)?
                };
                return Ok(Ifade::Degil(Box::new(ic)));
            }
        }
    }

    let kelimeler: Vec<Option<&str>> = tokenlar
        .iter()
        .map(|t| match &t.tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        })
        .collect();
    let n = tokenlar.len();
    if n < 2 {
        return Err(hata());
    }

    let yuklem = kelimeler[n - 1].ok_or_else(hata)?;
    let yuklem_koku = yuklem.trim_end_matches("se").trim_end_matches("sa");

    // X çiftse / X tekse
    if n == 2 && (yuklem == "çiftse" || yuklem == "tekse") {
        let islenen = tekil_ifade(tokenlar[0].clone())?;
        return Ok(if yuklem == "çiftse" {
            Ifade::Cift(Box::new(islenen))
        } else {
            Ifade::Tek(Box::new(islenen))
        });
    }

    // X varsa / X yoksa — Seçenek dolu mu.
    if n == 2 && (yuklem == "varsa" || yuklem == "yoksa") {
        return Ok(Ifade::SecenekVar {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            olumsuz: yuklem == "yoksa",
        });
    }

    // X boşsa / doluysa — koleksiyon ya da metin boş mu.
    if n == 2 && (yuklem == "boşsa" || yuklem == "doluysa") {
        return Ok(Ifade::BosMu {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            olumsuz: yuklem == "doluysa",
        });
    }

    // X başarılıysa / başarısızsa — Sonuç durumu.
    if n == 2 && (yuklem == "başarılıysa" || yuklem == "başarısızsa") {
        return Ok(Ifade::SonucBasarili {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            olumsuz: yuklem == "başarısızsa",
        });
    }

    // S de (anahtar) varsa/yoksa — sözlükte anahtar var mı.
    if n == 3 && (yuklem == "varsa" || yuklem == "yoksa") {
        return Ok(Ifade::SozlukteVar {
            sozluk: Box::new(tekil_ifade(tokenlar[0].clone())?),
            anahtar: Box::new(tekil_ifade(tokenlar[1].clone())?),
            olumsuz: yuklem == "yoksa",
        });
    }

    // M (aranan) içeriyorsa.
    if n == 3 && yuklem == "içeriyorsa" {
        return Ok(Ifade::Icerir {
            metin: Box::new(tekil_ifade(tokenlar[0].clone())?),
            aranan: Box::new(tekil_ifade(tokenlar[1].clone())?),
        });
    }

    // X Y veya daha büyükse/küçükse
    if n == 5 && kelimeler[2] == Some("veya") && kelimeler[3] == Some("daha") {
        let islec = match yuklem_koku {
            "büyük" => Islec::BuyukEsit,
            "küçük" => Islec::KucukEsit,
            _ => return Err(hata()),
        };
        let sol = tekil_ifade(tokenlar[0].clone())?;
        let sag = tekil_ifade(tokenlar[1].clone())?;
        return Ok(Ifade::Karsilastirma {
            sol: Box::new(sol),
            sag: Box::new(sag),
            islec,
        });
    }

    // X Y <ek> büyükse/küçükse/eşitse  (ya da çıplak: büyük/küçük/eşit — sürece için)
    if n == 4 {
        let ek = kelimeler[2].ok_or_else(hata)?;
        if AYRIK_EKLER.contains(&ek) {
            let islec = match yuklem_koku {
                "büyük" => Islec::Buyuk,
                "küçük" => Islec::Kucuk,
                "eşit" => Islec::Esit,
                _ => return Err(hata()),
            };
            let sol = tekil_ifade(tokenlar[0].clone())?;
            let sag = tekil_ifade(tokenlar[1].clone())?;
            return Ok(Ifade::Karsilastirma {
                sol: Box::new(sol),
                sag: Box::new(sag),
                islec,
            });
        }
    }

    // X Y-ekli büyükse/küçükse/eşitse — ek ada bitişik: "tahmin gizliden küçükse".
    // Çözümleyici eki ayıklar (K-011).
    if n == 3 {
        let islec = match yuklem_koku {
            "büyük" => Some(Islec::Buyuk),
            "küçük" => Some(Islec::Kucuk),
            "eşit" => Some(Islec::Esit),
            _ => None,
        };
        if let Some(islec) = islec {
            let sol = tekil_ifade(tokenlar[0].clone())?;
            let sag = tekil_ifade(tokenlar[1].clone())?;
            return Ok(Ifade::Karsilastirma {
                sol: Box::new(sol),
                sag: Box::new(sag),
                islec,
            });
        }
    }

    // X Y — düz eşitlik ("bildi doğru olana kadar").
    if n == 2 {
        let sol = tekil_ifade(tokenlar[0].clone())?;
        let sag = tekil_ifade(tokenlar[1].clone())?;
        return Ok(Ifade::Karsilastirma {
            sol: Box::new(sol),
            sag: Box::new(sag),
            islec: Islec::Esit,
        });
    }

    Err(hata())
}

/// Yapılı ifade kalıpları (K-008, K-009). Eşleşme yoksa Ok(None) döner ve
/// bölge "ile" zinciri olarak okunur. Kalıplar deterministiktir: bölgenin
/// TAMAMI eşleşmelidir, kısmi eşleşme zincire düşer.
///
///   X ile Y nin toplamı/farkı/çarpımı   (ekli ad: "ikincinin toplamı" — 4 token)
///   X ile 10 un toplamı                 (ayrık ekli sabit — 5 token)
///   X in Y ye bölümü                    (ekler adlara bitişik — 3 token,
///                                        sabitlerde ayrık — 4 ya da 5 token)
///   W ın sayısı                         ("yanıtın sayısı" — 2 token)
fn yapili_kalip(tokenlar: &[Token], islemler: &[String]) -> Result<Option<Ifade>, Tani> {
    let n = tokenlar.len();
    let kelime = |i: usize| -> Option<&str> {
        match &tokenlar[i].tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        }
    };
    let son = match kelime(n - 1) {
        Some(k) => k,
        None => return Ok(None),
    };

    // İşlem çağrısı ifadesi: bölge tanımlı bir işlem adıyla bitiyorsa.
    let ilk_satir = tokenlar[0].satir;
    if let Some(cagri) = cagri_kalibi(tokenlar, ilk_satir, islemler)? {
        return Ok(Some(cagri));
    }

    // W ın sayısı — ek, ada bitişiktir ("yanıtın"); çözümleyici ayıklar.
    if n == 2 && son == "sayısı" {
        return Ok(Some(Ifade::Sayisi(Box::new(tekil_ifade(tokenlar[0].clone())?))));
    }

    // boş liste / boş sözlük
    if n == 2 && kelime(0) == Some("boş") {
        if son == "liste" {
            return Ok(Some(Ifade::BosListe));
        }
        if son == "sözlük" {
            return Ok(Some(Ifade::BosSozluk));
        }
    }

    // W ın adedi / ilki / sonu / uzunluğu / kelimeleri — özellikler.
    // "aded" öneki ekli biçimleri de yakalar ("adedine", "adediyle").
    if n == 2 {
        let ozellik = if son.starts_with("aded") {
            Some(Ozellik::Adet)
        } else if son == "ilki" {
            Some(Ozellik::Ilk)
        } else if son == "sonu" {
            Some(Ozellik::Son)
        } else if son == "uzunluğu" {
            Some(Ozellik::Uzunluk)
        } else if son == "kelimeleri" {
            Some(Ozellik::Kelimeler)
        } else if son == "yılı" {
            Some(Ozellik::Yil)
        } else {
            None
        };
        if let Some(ozellik) = ozellik {
            return Ok(Some(Ifade::Ozellik {
                nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
                ozellik,
            }));
        }
    }

    // W ın değeri — Seçenek/Sonuç içindeki değer; W ın hatası — Sonuç hatası.
    let deger_kelimesi = matches!(son, "değeri" | "değerini" | "değerine" | "değeriyle");
    if n == 2 && deger_kelimesi {
        return Ok(Some(Ifade::IcDeger(Box::new(tekil_ifade(tokenlar[0].clone())?))));
    }
    if n == 2 && (son == "hatası" || son == "hatasını") {
        return Ok(Some(Ifade::SonucHatasi(Box::new(tekil_ifade(tokenlar[0].clone())?))));
    }

    // S in (anahtar) değeri — sözlükten okuma (ekli biçimler: değeriyle, değerine).
    if n == 3 && deger_kelimesi {
        return Ok(Some(Ifade::SozlukDegeri {
            sozluk: Box::new(tekil_ifade(tokenlar[0].clone())?),
            anahtar: Box::new(tekil_ifade(tokenlar[1].clone())?),
        }));
    }

    // W ın büyük/küçük harflisi — Türkçe harf kurallarıyla.
    if n == 3 && son == "harflisi" {
        let buyuk = match kelime(1) {
            Some("büyük") => Some(true),
            Some("küçük") => Some(false),
            _ => None,
        };
        if let Some(buyuk) = buyuk {
            return Ok(Some(Ifade::MetinDonusum {
                nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
                buyuk,
            }));
        }
    }

    // "..." dosyasını okumayı dene → Sonuç.
    if n == 4
        && kelime(1) == Some("dosyasını")
        && kelime(2) == Some("okumayı")
        && son == "dene"
    {
        return Ok(Some(Ifade::DosyaOkumayiDene(Box::new(tekil_ifade(
            tokenlar[0].clone(),
        )?))));
    }

    // "..." dosyasının satırları → Liste<Metin>.
    if n == 3 && kelime(1) == Some("dosyasının") && son == "satırları" {
        return Ok(Some(Ifade::DosyaSatirlari(Box::new(tekil_ifade(
            tokenlar[0].clone(),
        )?))));
    }

    // "..." dosyasından okunan tablo/veri → CSV tablosu / JSON nesnesi.
    if n == 4 && kelime(1) == Some("dosyasından") && kelime(2) == Some("okunan") {
        if son == "tablo" {
            return Ok(Some(Ifade::TabloOku(Box::new(tekil_ifade(tokenlar[0].clone())?))));
        }
        if son == "veri" {
            return Ok(Some(Ifade::VeriOku(Box::new(tekil_ifade(tokenlar[0].clone())?))));
        }
    }

    // bugünün tarihi / şu anın saati / komut satırından gelenler.
    if n == 2 && kelime(0) == Some("bugünün") && son == "tarihi" {
        return Ok(Some(Ifade::BugununTarihi));
    }
    if n == 3 && kelime(0) == Some("şu") && kelime(1) == Some("anın") && son == "saati" {
        return Ok(Some(Ifade::SuAninSaati));
    }
    if n == 3 && kelime(0) == Some("komut") && kelime(1) == Some("satırından") && son == "gelenler" {
        return Ok(Some(Ifade::KomutArgumanlari));
    }

    // T nin <n> gün sonrası — tarih aritmetiği.
    if n == 4 && kelime(2) == Some("gün") && son == "sonrası" {
        return Ok(Some(Ifade::GunSonrasi {
            tarih: Box::new(tekil_ifade(tokenlar[0].clone())?),
            miktar: Box::new(tekil_ifade(tokenlar[1].clone())?),
        }));
    }

    // yeni <Yapı> — yeni yapı örneği (K-020).
    if n == 2 && kelime(0) == Some("yeni") {
        return Ok(Some(Ifade::YeniYapi { yapi_adi: son.to_string() }));
    }

    // <nesne-in> <alan> — iyelik ekiyle alan okuma (K-020). Adlı kalıplardan
    // SONRA denenir; alan adı "değeri", "adedi" gibi ayrılmış kelimeler olamaz.
    if n == 2 {
        if let (Some(nesne), Some(alan)) = (kelime(0), kelime(1)) {
            if tamlayan_ekli(nesne) && alan != "ile" {
                return Ok(Some(Ifade::AlanErisim {
                    nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
                    alan: alan.to_string(),
                }));
            }
        }
    }

    // e1, e2, ... listesi
    if son == "listesi" && n >= 2 {
        let govde = &tokenlar[..n - 1];
        let mut ogeler = Vec::new();
        let mut deger_sirasi = true;
        for token in govde {
            if deger_sirasi {
                ogeler.push(tekil_ifade(token.clone())?);
                deger_sirasi = false;
            } else {
                if token.tur != TokenTur::Virgul {
                    return Ok(None);
                }
                deger_sirasi = true;
            }
        }
        if deger_sirasi {
            // sonda virgül kaldı
            return Ok(None);
        }
        return Ok(Some(Ifade::ListeSabiti(ogeler)));
    }

    // A ile B arasında rastgele sayı
    if n == 6
        && son == "sayı"
        && kelime(1) == Some("ile")
        && kelime(3) == Some("arasında")
        && kelime(4) == Some("rastgele")
    {
        let alt = tekil_ifade(tokenlar[0].clone())?;
        let ust = tekil_ifade(tokenlar[2].clone())?;
        return Ok(Some(Ifade::Rastgele { alt: Box::new(alt), ust: Box::new(ust) }));
    }

    // X ile Y nin toplamı/farkı/çarpımı
    let toplama_islec = match son {
        "toplamı" => Some(AritmetikIslec::Topla),
        "farkı" => Some(AritmetikIslec::Cikar),
        "çarpımı" => Some(AritmetikIslec::Carp),
        _ => None,
    };
    if let Some(islec) = toplama_islec {
        // [X, ile, Y, op] — Y'nin tamlayan eki bitişik; ya da
        // [X, ile, Y, ek, op] — sabitlerde ek ayrık (K-011).
        let uygun = (n == 4 && kelime(1) == Some("ile"))
            || (n == 5
                && kelime(1) == Some("ile")
                && kelime(3).map(|e| TAMLAYAN_EKLER.contains(&e)).unwrap_or(false));
        if uygun {
            let sol = tekil_ifade(tokenlar[0].clone())?;
            let sag = tekil_ifade(tokenlar[2].clone())?;
            return Ok(Some(Ifade::Aritmetik {
                islec,
                sol: Box::new(sol),
                sag: Box::new(sag),
            }));
        }
        return Ok(None);
    }

    // X in Y ye bölümü
    if son == "bölümü" {
        let govde = &tokenlar[..n - 1];
        let mut i = 0;
        if i >= govde.len() {
            return Ok(None);
        }
        let sol_token = govde[i].clone();
        i += 1;
        // Sabit sonrası ayrık tamlayan eki.
        if i < govde.len() {
            if let TokenTur::Kelime(k) = &govde[i].tur {
                if TAMLAYAN_EKLER.contains(&k.as_str()) {
                    i += 1;
                }
            }
        }
        if i >= govde.len() {
            return Ok(None);
        }
        let sag_token = govde[i].clone();
        i += 1;
        // Sabit sonrası ayrık yönelme eki.
        if i < govde.len() {
            if let TokenTur::Kelime(k) = &govde[i].tur {
                if AYRIK_EKLER.contains(&k.as_str()) {
                    i += 1;
                }
            }
        }
        if i != govde.len() {
            return Ok(None);
        }
        let sol = tekil_ifade(sol_token)?;
        let sag = tekil_ifade(sag_token)?;
        return Ok(Some(Ifade::Aritmetik {
            islec: AritmetikIslec::Bol,
            sol: Box::new(sol),
            sag: Box::new(sag),
        }));
    }

    Ok(None)
}

/// Kelime tamlayan (genitif) ekiyle mi bitiyor? ("ayşenin", "öğrencinin")
fn tamlayan_ekli(kelime: &str) -> bool {
    TAMLAYAN_EKLER.iter().any(|ek| kelime.ends_with(ek)) && kelime.chars().count() > 3
}

/// Belirtme eki almış adı yalın hale getirir ("sayıları"→"sayılar", "adı"→"ad").
/// Tanım anında kapsam olmadığı için yapısaldır: yalnız ek atılır, ünsüz
/// yumuşaması geri çevrilmez ("sonucu"→"sonuc"); çözümleyicinin aday üretimi
/// iki yönü de denediğinden gövde içi başvurular tutarlı çözülür.
fn yalin_ad(ekli: &str) -> String {
    for ek in ["yı", "yi", "yu", "yü", "ı", "i", "u", "ü"] {
        if let Some(kok) = ekli.strip_suffix(ek) {
            if kok.chars().count() >= 2 {
                return kok.to_string();
            }
        }
    }
    ekli.to_string()
}

/// Satır sonu, tanımlı bir işlem adıyla bitiyorsa çağrı üretir (K-016 geçici
/// sözdizimi): `<argümanlar> için/ile <işlem adı>`; argümanlar "ve" ile ayrılır.
/// En uzun ad önce denenir (determinizm).
fn cagri_kalibi(
    tokenlar: &[Token],
    satir: usize,
    islemler: &[String],
) -> Result<Option<Ifade>, Tani> {
    let n = tokenlar.len();
    if n == 0 {
        return Ok(None);
    }

    let mut adaylar: Vec<&String> = islemler.iter().collect();
    adaylar.sort_by_key(|ad| std::cmp::Reverse(ad.split(' ').count()));

    for ad in adaylar {
        let kelimeler: Vec<&str> = ad.split(' ').collect();
        let k = kelimeler.len();
        if k > n {
            continue;
        }
        let kuyruk_uyar = tokenlar[n - k..]
            .iter()
            .zip(&kelimeler)
            .all(|(token, kelime)| matches!(&token.tur, TokenTur::Kelime(t) if t == kelime));
        if !kuyruk_uyar {
            continue;
        }

        let mut arg_bolgesi = &tokenlar[..n - k];
        let mut argumanlar = Vec::new();
        if !arg_bolgesi.is_empty() {
            // Sondaki ayraç: "için" ya da "ile".
            let ayrac_var = matches!(
                &arg_bolgesi[arg_bolgesi.len() - 1].tur,
                TokenTur::Kelime(a) if a == "için" || a == "ile"
            );
            if !ayrac_var {
                return Err(Tani::yeni(
                    "S019",
                    format!(
                        "\"{}\" çağrısında argümanlar \"için\" ya da \"ile\" ile ayrılır.",
                        ad
                    ),
                    satir,
                    1,
                    1,
                )
                .onerili(format!("Örnek: notlar için {}", ad)));
            }
            arg_bolgesi = &arg_bolgesi[..arg_bolgesi.len() - 1];
            let mut bolge: Vec<Token> = Vec::new();
            for token in arg_bolgesi {
                if kelime_mi(token, "ve") {
                    if bolge.len() != 1 {
                        return Err(arg_hatasi(ad, satir));
                    }
                    argumanlar.push(tekil_ifade(bolge.pop().unwrap())?);
                } else {
                    bolge.push(token.clone());
                }
            }
            if bolge.len() != 1 {
                return Err(arg_hatasi(ad, satir));
            }
            argumanlar.push(tekil_ifade(bolge.pop().unwrap())?);
        }

        return Ok(Some(Ifade::IslemCagrisi {
            islem_adi: ad.clone(),
            argumanlar,
            satir,
        }));
    }

    Ok(None)
}

fn arg_hatasi(ad: &str, satir: usize) -> Tani {
    Tani::yeni(
        "S020",
        format!("\"{}\" çağrısında her argüman tek bir değer olmalı; \"ve\" ile ayrılır.", ad),
        satir,
        1,
        1,
    )
    .onerili("Örnek: \"Ayşe\" ve 10 ile selamla".into())
}

fn goster(token: &Token) -> String {
    match &token.tur {
        TokenTur::Metin(m) => format!("\"{}\"", m),
        TokenTur::TamSayi(s) => s.to_string(),
        TokenTur::Kelime(k) => k.clone(),
        _ => "?".into(),
    }
}
