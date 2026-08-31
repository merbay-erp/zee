//! Ayrıştırıcı (parser) — elle yazılmış recursive descent (ADR-002 adayı).
//!
//! Türkçe yüklem-sonlu olduğu için cümlenin türü satırın SON kelimesinden
//! anlaşılır: "... yaz", "... olsun", "... tekrarla", "... ise". Bu, İngilizce
//! dillerdeki "ilk keyword'e bak" yaklaşımının aynadaki karşılığıdır ve
//! deterministik ayrıştırmayı mümkün kılar.

use crate::agac::{AritmetikIslec, Cumle, Ifade, Islec, KosulKolu, Ozellik};
use crate::sozcukleyici::{Token, TokenTur};
use crate::tani::Tani;

pub fn ayristir(tokenlar: Vec<Token>) -> Result<Vec<Cumle>, Tani> {
    let mut ayristirici = Ayristirici { tokenlar, konum: 0 };
    let program = ayristirici.blok_ayristir()?;
    ayristirici.bekle_dosya_sonu()?;
    Ok(program)
}

struct Ayristirici {
    tokenlar: Vec<Token>,
    konum: usize,
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
        let govde = self.blok_ayristir()?;
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
            Some(k) if kosul_kelimesi(k) => self.ise_ayristir(satir_tokenlari, satir_no),
            _ => {
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
                     \"<ad> <n> artır/azalt\"."
                        .into(),
                ))
            }
        }
    }

    // ---- cümle türleri ----

    fn yaz_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "yaz"
        let deger = ile_ifadesi(&tokenlar, satir)?;
        Ok(Cumle::Yaz { deger, satir })
    }

    fn olsun_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "olsun"
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
        let deger = ile_ifadesi(&tokenlar, satir)?;
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
        let adet = ile_ifadesi(&tokenlar, satir)?;
        let govde = self.alt_blok(satir)?;
        Ok(Cumle::KezTekrarla { adet, govde, satir })
    }

    fn aralik_ayristir(&mut self, tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
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
        let miktar = ile_ifadesi(&tokenlar, satir)?;
        if artir {
            Ok(Cumle::Artir { ifade, miktar, satir })
        } else {
            Ok(Cumle::Azalt { ifade, miktar, satir })
        }
    }

    fn ekle_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "ekle"
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
        let deger = ile_ifadesi(&tokenlar, satir)?;
        Ok(Cumle::Ekle { hedef, deger, satir })
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
        let istem = ile_ifadesi(&tokenlar, satir)?;
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
        "büyükse" | "küçükse" | "eşitse" | "çiftse" | "tekse"
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
fn ile_ifadesi(tokenlar: &[Token], satir: usize) -> Result<Ifade, Tani> {
    if tokenlar.is_empty() {
        return Err(Tani::yeni("S013", "Burada bir değer bekleniyor.".into(), satir, 1, 1));
    }

    if let Some(ifade) = yapili_kalip(tokenlar)? {
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
            parcalar.push(bolge_ifadesi(&bolge, satir)?);
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
    parcalar.push(bolge_ifadesi(&bolge, satir)?);

    if parcalar.len() == 1 {
        Ok(parcalar.pop().unwrap())
    } else {
        Ok(Ifade::Birlestir(parcalar))
    }
}

/// Tek "ile" parçası: tek token ya da yapılı kalıp.
fn bolge_ifadesi(tokenlar: &[Token], _satir: usize) -> Result<Ifade, Tani> {
    if tokenlar.len() == 1 {
        return tekil_ifade(tokenlar[0].clone());
    }
    if let Some(ifade) = yapili_kalip(tokenlar)? {
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

/// Koşul ifadesi: yüklem satır sonundadır.
///
/// Desteklenen kalıplar (K-010):
///   X Y veya daha büyükse   → X >= Y
///   X Y veya daha küçükse   → X <= Y
///   X Y den büyükse         → X > Y     (den/dan)
///   X Y den küçükse         → X < Y
///   X Y e eşitse            → X == Y    (e/a/ye/ya)
///   X çiftse / X tekse
/// "olduğu sürece" içinde yüklem çıplak gelir: "büyük", "küçük", "eşit".
fn kosul_ifadesi(tokenlar: &[Token], satir: usize) -> Result<Ifade, Tani> {
    let hata = || {
        Tani::yeni("S016", "Koşul tanınmadı.".into(), satir, 1, 1).onerili(
            "Örnekler: yaş 8 veya daha büyükse · puan 50 den küçükse · sayı 5 e eşitse · sayı çiftse"
                .into(),
        )
    };

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
fn yapili_kalip(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
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

    // W ın sayısı — ek, ada bitişiktir ("yanıtın"); çözümleyici ayıklar.
    if n == 2 && son == "sayısı" {
        return Ok(Some(Ifade::Sayisi(Box::new(tekil_ifade(tokenlar[0].clone())?))));
    }

    // boş liste
    if n == 2 && kelime(0) == Some("boş") && son == "liste" {
        return Ok(Some(Ifade::BosListe));
    }

    // W ın adedi / ilki / sonu — liste özellikleri. "aded" öneki, ekli
    // biçimleri de yakalar ("adedine", "adediyle").
    if n == 2 {
        let ozellik = if son.starts_with("aded") {
            Some(Ozellik::Adet)
        } else if son == "ilki" {
            Some(Ozellik::Ilk)
        } else if son == "sonu" {
            Some(Ozellik::Son)
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

fn goster(token: &Token) -> String {
    match &token.tur {
        TokenTur::Metin(m) => format!("\"{}\"", m),
        TokenTur::TamSayi(s) => s.to_string(),
        TokenTur::Kelime(k) => k.clone(),
        _ => "?".into(),
    }
}
