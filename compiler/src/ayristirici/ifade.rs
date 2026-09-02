use super::*;

pub(super) fn gecerli_hata_kodu(kod: &str) -> bool {
    let mut harfler = kod.chars();
    matches!(harfler.next(), Some('A'..='Z'))
        && harfler.all(|k| k.is_ascii_uppercase() || k.is_ascii_digit() || k == '_')
}

// ---- yardımcılar ----

pub(super) fn son_kelime(tokenlar: &[Token]) -> Option<String> {
    match tokenlar.last().map(|t| &t.tur) {
        Some(TokenTur::Kelime(k)) => Some(k.clone()),
        _ => None,
    }
}

pub(super) fn kelime_mi(token: &Token, beklenen: &str) -> bool {
    matches!(&token.tur, TokenTur::Kelime(k) if k == beklenen)
}

/// Satır sonundaki kelime bir koşul yüklemi mi? ("büyükse", "çiftse", ...)
pub(super) fn kosul_kelimesi(kelime: &str) -> bool {
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
            | "başlıyorsa"
            | "bitiyorsa"
            | "doğrulanıyorsa"
            | "başarılıysa"
            | "başarısızsa"
            | "boşsa"
            | "doluysa"
            | "açıksa"
            | "kapalıysa"
    )
}

pub(super) fn eslesen_ek<I: Iterator<Item = Token>>(t: &mut std::iter::Peekable<I>) -> Option<()> {
    match t.next().map(|x| x.tur) {
        Some(TokenTur::Kelime(k)) if AYRIK_EKLER.contains(&k.as_str()) => Some(()),
        _ => None,
    }
}

pub(super) fn bekle_kelime<I: Iterator<Item = Token>>(t: &mut std::iter::Peekable<I>, beklenen: &str) -> Option<()> {
    match t.next().map(|x| x.tur) {
        Some(TokenTur::Kelime(k)) if k == beklenen => Some(()),
        _ => None,
    }
}

/// Tek token'dan ifade üretir (sabit ya da değişken).
pub(super) fn tekil_ifade(token: Token) -> Result<Ifade, Tani> {
    let konum = token.clone();
    let ifade = match token.tur {
        TokenTur::Metin(m) => Ifade::MetinSabiti(m),
        TokenTur::TamSayi(s) => Ifade::SayiSabiti(s),
        TokenTur::Ondalik { govde, olcek } => Ifade::OndalikSabiti { govde, olcek },
        TokenTur::Kelime(k) if k == "doğru" => Ifade::MantiksalSabiti(true),
        TokenTur::Kelime(k) if k == "yanlış" => Ifade::MantiksalSabiti(false),
        TokenTur::Kelime(k) if k == "yok" => Ifade::YokSabiti,
        TokenTur::Kelime(k) => Ifade::Degisken {
            ham: k,
            cozulmus: None,
            sembol_kimligi: None,
            satir: token.satir,
            sutun: token.sutun,
            uzunluk: token.uzunluk,
        },
        _ => {
            return Err(Tani::yeni(
                "S013",
                "Burada bir değer bekleniyor.".into(),
                token.satir,
                token.sutun,
                token.uzunluk,
            ));
        }
    };
    konumlu_ifade(ifade, std::slice::from_ref(&konum))
}

/// Tamlayan (genitif) ayrık ekleri: "10 un", "3 ün".
const TAMLAYAN_EKLER: [&str; 8] = ["nın", "nin", "nun", "nün", "ın", "in", "un", "ün"];

/// Değer ifadesinin geçiş girdisi (RFC-0021/spec-20).
///
/// Bugünkü uyumluluk yolu bütün bölgeyi önce yapılandırılmış katmanlarda
/// (erişim/postfix → çağrı → aritmetik) dener; yalnız hiçbir tam kalıp
/// tüketmezse `ile` birleştirmesine düşer. Karşılaştırma ve boolean zincir
/// kendi cümle bağlamında bunun üstündedir. Yeni ifade özelliği gelişigüzel
/// bir üst-düzey dal olarak değil, RFC-0021'deki tek katmana eklenir.
pub(super) fn ile_ifadesi(tokenlar: &[Token], satir: usize, islemler: &[String]) -> Result<Ifade, Tani> {
    konumlu_ifade(ile_ifadesi_ic(tokenlar, satir, islemler)?, tokenlar)
}

fn ile_ifadesi_ic(tokenlar: &[Token], satir: usize, islemler: &[String]) -> Result<Ifade, Tani> {
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

    if let [tek] = parcalar.as_slice() {
        Ok(tek.clone())
    } else {
        Ok(Ifade::Birlestir(parcalar))
    }
}

/// Tek "ile" parçası: tek token ya da yapılı kalıp.
pub(super) fn bolge_ifadesi(tokenlar: &[Token], _satir: usize, islemler: &[String]) -> Result<Ifade, Tani> {
    konumlu_ifade(bolge_ifadesi_ic(tokenlar, _satir, islemler)?, tokenlar)
}

fn bolge_ifadesi_ic(tokenlar: &[Token], _satir: usize, islemler: &[String]) -> Result<Ifade, Tani> {
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

/// Boolean katmanı: önce ve/veya zinciri ayrılır, parçalar karşılaştırmadır.
///
/// K-027 kuralı: `A ve B ve C` ya da `A veya B` serbesttir; ve/veya KARIŞIMI
/// parantezsiz belirsiz olduğundan hatadır (S030) — kullanıcı koşulu böler.
/// "veya daha" ikilisi karşılaştırma kalıbına aittir ("90 veya daha büyükse"),
/// zincir ayracı sayılmaz.
pub(super) fn kosul_ifadesi(tokenlar: &[Token], satir: usize) -> Result<Ifade, Tani> {
    konumlu_ifade(kosul_ifadesi_ic(tokenlar, satir)?, tokenlar)
}

fn kosul_ifadesi_ic(tokenlar: &[Token], satir: usize) -> Result<Ifade, Tani> {
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

/// Karşılaştırma katmanı: yüklem sondadır.
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
pub(super) fn kosul_atomu(tokenlar: &[Token], satir: usize) -> Result<Ifade, Tani> {
    konumlu_ifade(kosul_atomu_ic(tokenlar, satir)?, tokenlar)
}

fn kosul_atomu_ic(tokenlar: &[Token], satir: usize) -> Result<Ifade, Tani> {
    let hata = || {
        Tani::yeni("S016", "Koşul tanınmadı.".into(), satir, 1, 1).onerili(
            "Örnekler: yaş 8 veya daha büyükse · puan 50 den küçükse · sayı 5 e eşitse · sayı çiftse"
                .into(),
        )
    };

    // Saf koşaç ayrı yazılmışsa düşer: "asal ise" → koşul "asal" (K-044).
    if tokenlar.len() >= 2 {
        if let TokenTur::Kelime(k) = &tokenlar[tokenlar.len() - 1].tur {
            if k == "ise" {
                return kosul_atomu(&tokenlar[..tokenlar.len() - 1], satir);
            }
        }
    }

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
    // Tek kelimelik koşul: Mantıksal adın kendisi — "asal ise". Olumsuzu
    // zaten vardı ("asal değilse"); bakışım K-044 ile tamamlandı. Tür
    // bekçisi ifadenin Mantıksal olmasını ayrıca zorlar.
    if n == 1 {
        return tekil_ifade(tokenlar[0].clone());
    }
    if n < 2 {
        return Err(hata());
    }

    let yuklem = kelimeler[n - 1].ok_or_else(hata)?;
    let yuklem_koku = yuklem.trim_end_matches("se").trim_end_matches("sa");

    // <sensör> açıksa / kapalıysa — IoT simülatörü (golden 29).
    if n == 2 && (yuklem == "açıksa" || yuklem == "kapalıysa") {
        if let TokenTur::Kelime(ad) = &tokenlar[0].tur {
            let sensor_adi = konumlu_ifade(
                Ifade::MetinSabiti(ad.clone()),
                std::slice::from_ref(&tokenlar[0]),
            )?;
            let sensor = konumlu_ifade(
                Ifade::Intrinsic {
                    kimlik: SENSOR_ACIK_MI.into(),
                    argumanlar: vec![sensor_adi],
                },
                tokenlar,
            )?;
            return Ok(if yuklem == "kapalıysa" {
                Ifade::Degil(Box::new(sensor))
            } else {
                sensor
            });
        }
    }

    // X çiftse / X tekse
    if n == 2 && (yuklem == "çiftse" || yuklem == "tekse") {
        let islenen = tekil_ifade(tokenlar[0].clone())?;
        return Ok(if yuklem == "çiftse" {
            Ifade::Cift(Box::new(islenen))
        } else {
            Ifade::Tek(Box::new(islenen))
        });
    }

    // <parola> <Argon2id PHC özeti> ile doğrulanıyorsa.
    if n == 4 && kelimeler[2] == Some("ile") && yuklem == "doğrulanıyorsa" {
        return Ok(Ifade::Intrinsic {
            kimlik: PAROLA_DOGRULA.into(),
            argumanlar: vec![
                tekil_ifade(tokenlar[0].clone())?,
                tekil_ifade(tokenlar[1].clone())?,
            ],
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
    // X P ile başlıyorsa / bitiyorsa (K-053).
    if n == 4
        && kelimeler[2] == Some("ile")
        && (yuklem == "başlıyorsa" || yuklem == "bitiyorsa")
    {
        return Ok(Ifade::MetinSinari {
            metin: Box::new(tekil_ifade(tokenlar[0].clone())?),
            parca: Box::new(tekil_ifade(tokenlar[1].clone())?),
            bitis: yuklem == "bitiyorsa",
        });
    }

    // Çıplak boş/dolu (K-071): "liste boş olmamalı", "... boş olduğu sürece".
    if n == 2 && (yuklem == "boş" || yuklem == "dolu") {
        return Ok(Ifade::BosMu {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            olumsuz: yuklem == "dolu",
        });
    }

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

/// Primary + erişim/postfix + çağrı + aritmetik katmanlarının bugünkü
/// uyumluluk gerçekleyicisi (RFC-0021/spec-20). Eşleşme yoksa Ok(None) döner
/// ve bölge `ile` birleştirmesi olarak okunur. Kalıplar deterministiktir:
/// bölgenin TAMAMI eşleşmelidir; kısmi eşleşme başka anlama düşmez.
///
///   X ile Y nin toplamı/farkı/çarpımı   (ekli ad: "ikincinin toplamı" — 4 token)
///   X ile 10 un toplamı                 (ayrık ekli sabit — 5 token)
///   X in Y ye bölümü                    (ekler adlara bitişik — 3 token,
///                                        sabitlerde ayrık — 4 ya da 5 token)
///   W ın sayısı                         ("yanıtın sayısı" — 2 token)
pub(super) fn yapili_kalip(tokenlar: &[Token], islemler: &[String]) -> Result<Option<Ifade>, Tani> {
    yapili_kalip_ic(tokenlar, islemler)?
        .map(|ifade| konumlu_ifade(ifade, tokenlar))
        .transpose()
}

fn yapili_kalip_ic(tokenlar: &[Token], islemler: &[String]) -> Result<Option<Ifade>, Tani> {
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

    // Bütün bölge görünür işlem adıysa sıfır argümanlı çağrıdır. Bu eski ve
    // geçerli çağrı biçimini korur; yalnız işlem-adı KUYRUĞUNUN daha güçlü bir
    // postfix'i gölgelemesi aşağıdaki katman sırasıyla engellenir.
    let ilk_satir = tokenlar[0].satir;
    if let Some(cagri) = sifir_argumanli_cagri(tokenlar, ilk_satir, islemler) {
        return Ok(Some(cagri));
    }

    // W ın sayısı / ondalığı — ek, ada bitişiktir ("yanıtın"); çözümleyici ayıklar.
    if n == 2 && son == "sayısı" {
        return Ok(Some(Ifade::Sayisi(Box::new(tekil_ifade(tokenlar[0].clone())?))));
    }
    if n == 2 && son == "ondalığı" {
        return Ok(Some(Ifade::Ondaligi(Box::new(tekil_ifade(tokenlar[0].clone())?))));
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

    if n == 2 && kelime(0) == Some("csrf") && son == "belirteci" {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: CSRF_BELIRTECI.into(),
            argumanlar: Vec::new(),
        }));
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
        } else if son == "yuvarlanmışı" {
            Some(Ozellik::Yuvarlanmis)
        } else if son == "kırpılmışı" {
            Some(Ozellik::Kirpilmis)
        } else if son == "harfleri" {
            Some(Ozellik::Harfler)
        } else if son == "sıralanmışı" {
            Some(Ozellik::Siralanmis)
        } else if son == "tersi" {
            Some(Ozellik::Ters)
        } else if son == "kuruşlusu" {
            Some(Ozellik::Kuruslu)
        } else if matches!(son, "kodu" | "kodunu" | "koduna" | "koduyla") {
            Some(Ozellik::HataKodu)
        } else if matches!(son, "mesajı" | "mesajını" | "mesajına" | "mesajıyla") {
            Some(Ozellik::HataMesaji)
        } else if matches!(son, "nedeni" | "nedenini" | "nedenine" | "nedeniyle") {
            Some(Ozellik::HataNedeni)
        } else if matches!(son, "verisi" | "verisini" | "verisine" | "verisiyle") {
            Some(Ozellik::HataVerisi)
        } else if son == "metni" {
            Some(Ozellik::Metni)
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

    // X in tam kısmı — ondalığın virgül öncesi (RFC-0013).
    if n == 3 && kelime(1) == Some("tam") && son == "kısmı" {
        return Ok(Some(Ifade::Ozellik {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            ozellik: Ozellik::TamKisim,
        }));
    }

    // S in (anahtar) değeri — sözlükten okuma (ekli biçimler: değeriyle, değerine).
    if n == 3 && deger_kelimesi {
        return Ok(Some(Ifade::SozlukDegeri {
            sozluk: Box::new(tekil_ifade(tokenlar[0].clone())?),
            anahtar: Box::new(tekil_ifade(tokenlar[1].clone())?),
        }));
    }

    // X ile Y arasındaki günler — işaretli tarih farkı (K-057).
    if n == 5
        && son == "günler"
        && kelime(1) == Some("ile")
        && kelime(3) == Some("arasındaki")
    {
        return Ok(Some(Ifade::GunFarki {
            birinci: Box::new(tekil_ifade(tokenlar[0].clone())?),
            ikinci: Box::new(tekil_ifade(tokenlar[2].clone())?),
        }));
    }

    // W ın binlikli kuruşlusu — Türk para yazımı (K-075).
    if n == 3 && son == "kuruşlusu" && kelime(1) == Some("binlikli") {
        return Ok(Some(Ifade::Ozellik {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            ozellik: Ozellik::BinlikliKuruslu,
        }));
    }

    // W ın csv metni — tablo serileştirme (K-058).
    if n == 3 && son == "metni" && kelime(1) == Some("csv") {
        return Ok(Some(Ifade::Ozellik {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            ozellik: Ozellik::CsvMetin,
        }));
    }

    // W ın json metni — serileştirme (K-054).
    if n == 3 && son == "metni" && kelime(1) == Some("json") {
        return Ok(Some(Ifade::Ozellik {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            ozellik: Ozellik::JsonMetin,
        }));
    }

    // W ın X ile parçaları / birleşmişi (K-053).
    if n == 4 && kelime(2) == Some("ile") && (son == "parçaları" || son == "birleşmişi") {
        let sol = Box::new(tekil_ifade(tokenlar[0].clone())?);
        let ayrac = Box::new(tekil_ifade(tokenlar[1].clone())?);
        return Ok(Some(if son == "parçaları" {
            Ifade::Parcala { metin: sol, ayrac }
        } else {
            Ifade::ListeBirlestir { liste: sol, ayrac }
        }));
    }

    // W ın E yerine Y değişmişi (K-053).
    if n == 5 && kelime(2) == Some("yerine") && son == "değişmişi" {
        return Ok(Some(Ifade::Degistir {
            metin: Box::new(tekil_ifade(tokenlar[0].clone())?),
            eski: Box::new(tekil_ifade(tokenlar[1].clone())?),
            yeni: Box::new(tekil_ifade(tokenlar[3].clone())?),
        }));
    }

    // W ın html güvenlisi — HTML'e gömülmeye güvenli kaçışlanmış kopya (K-051).
    if n == 3 && son == "güvenlisi" && kelime(1) == Some("html") {
        return Ok(Some(Ifade::Ozellik {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            ozellik: Ozellik::HtmlGuvenli,
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

    // W ın sayısını/ondalığını almayı dene → Sonuç (RFC-0008 §4.3: mastar + dene).
    if n == 4 && kelime(2) == Some("almayı") && son == "dene" {
        if kelime(1) == Some("sayısını") {
            return Ok(Some(Ifade::SayiyiDene(Box::new(tekil_ifade(tokenlar[0].clone())?))));
        }
        if kelime(1) == Some("ondalığını") {
            return Ok(Some(Ifade::OndaligiDene(Box::new(tekil_ifade(tokenlar[0].clone())?))));
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

    // "..." adresinden gelen yanıt → AğYanıtı (golden 24).
    if n == 4 && kelime(1) == Some("adresinden") && kelime(2) == Some("gelen") && son == "yanıt" {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: HTTP_GETIR.into(),
            argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
        }));
    }

    // W ın durum kodu / gövdesi — AğYanıtı özellikleri.
    if n == 3 && kelime(1) == Some("durum") && (son == "kodu" || son == "kodunu") {
        return Ok(Some(Ifade::DurumKodu(Box::new(tekil_ifade(tokenlar[0].clone())?))));
    }
    if n == 2 && (son == "gövdesi" || son == "gövdesini") {
        return Ok(Some(Ifade::Govde(Box::new(tekil_ifade(tokenlar[0].clone())?))));
    }

    // Süre sabiti: <sayı|ondalık|yarım> saniye/dakika/saat (RFC-0011/0013).
    if n == 2 {
        let katsayi = match son {
            "saniye" => Some(1000i128),
            "dakika" => Some(60_000i128),
            "saat" => Some(3_600_000i128),
            _ => None,
        };
        if let Some(katsayi) = katsayi {
            let milisaniye: Option<i128> = match &tokenlar[0].tur {
                TokenTur::TamSayi(s) if *s >= 0 => Some(*s as i128 * katsayi),
                TokenTur::Ondalik { govde, olcek } => {
                    match Ondalik::govdeden(govde, *olcek) {
                        Some(ondalik) if !ondalik.negatif_mi() => Some(
                            ondalik.katsayiyla_yuvarla_i128(katsayi).ok_or_else(|| {
                                Tani::yeni(
                                    "S006",
                                    "Süre değeri sınırı aşıyor.".into(),
                                    tokenlar[0].satir,
                                    tokenlar[0].sutun,
                                    tokenlar[0].uzunluk,
                                )
                            })?,
                        ),
                        _ => None,
                    }
                }
                TokenTur::Kelime(k) if k == "yarım" => Some(katsayi / 2),
                _ => None,
            };
            if let Some(ms) = milisaniye {
                let ms = i64::try_from(ms).map_err(|_| {
                    Tani::yeni(
                        "S006",
                        "Süre değeri sınırı aşıyor.".into(),
                        tokenlar[0].satir,
                        tokenlar[0].sutun,
                        tokenlar[0].uzunluk,
                    )
                })?;
                return Ok(Some(Ifade::SureSabiti { milisaniye: ms }));
            }
        }
    }

    // yeni <Yapı> — yeni yapı örneği (K-020).
    if n == 2 && kelime(0) == Some("yeni") {
        return Ok(Some(Ifade::YeniYapi {
            yapi_adi: son.to_string(),
            yapi_kimligi: None,
        }));
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

    // Çağrı katmanı primary ve erişim/postfix'ten sonra, aritmetikten önce
    // gelir (RFC-0021/spec-20). Böylece görünür bir `sayısı` işlemi,
    // `metnin sayısı` postfix ifadesini bağlama göre gölgeleyemez.
    if let Some(cagri) = cagri_kalibi(tokenlar, ilk_satir, islemler)? {
        return Ok(Some(cagri));
    }

    // X ile Y nin toplamı/farkı/çarpımı
    let toplama_islec = match son {
        "toplamı" => Some(AritmetikIslec::Topla),
        "farkı" => Some(AritmetikIslec::Cikar),
        "çarpımı" => Some(AritmetikIslec::Carp),
        _ => None,
    };
    if let Some(islec) = toplama_islec {
        // Kalıp SONDAN çözülür: [SOL-BÖLGE, ile, Y, op] ya da
        // [SOL-BÖLGE, ile, Y, tamlayan-ek, op] (sabitlerde ek ayrık, K-011).
        // Sol taraf çok tokenli olabilir ("denemenin değeri ile 2 nin çarpımı").
        let (ile_indeksi, sag_indeksi) = if n >= 4 && kelime(n - 3) == Some("ile") {
            (n - 3, n - 2)
        } else if n >= 5
            && kelime(n - 4) == Some("ile")
            && kelime(n - 2).map(|e| TAMLAYAN_EKLER.contains(&e)).unwrap_or(false)
        {
            (n - 4, n - 3)
        } else {
            return Ok(None);
        };
        let satir = tokenlar[0].satir;
        let sol = bolge_ifadesi(&tokenlar[..ile_indeksi], satir, islemler)?;
        let sag = tekil_ifade(tokenlar[sag_indeksi].clone())?;
        return Ok(Some(Ifade::Aritmetik {
            islec,
            sol: Box::new(sol),
            sag: Box::new(sag),
        }));
    }

    // X in Y ye bölümü — ve kalanı: "X in Y ye bölümünden kalanı" (K-046).
    let kalan_kalibi = son == "kalanı"
        && n >= 4
        && kelime(n - 2) == Some("bölümünden");
    if son == "bölümü" || kalan_kalibi {
        let govde = if kalan_kalibi { &tokenlar[..n - 2] } else { &tokenlar[..n - 1] };
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
            islec: if kalan_kalibi { AritmetikIslec::Kalan } else { AritmetikIslec::Bol },
            sol: Box::new(sol),
            sag: Box::new(sag),
        }));
    }

    Ok(None)
}

/// Kelime tamlayan (genitif) ekiyle mi bitiyor? ("ayşenin", "öğrencinin")
pub(super) fn tamlayan_ekli(kelime: &str) -> bool {
    TAMLAYAN_EKLER.iter().any(|ek| kelime.ends_with(ek)) && kelime.chars().count() > 3
}

/// Belirtme eki almış adı yalın hale getirir ("sayıları"→"sayılar", "adı"→"ad").
/// Tanım anında kapsam olmadığı için yapısaldır: yalnız ek atılır, ünsüz
/// yumuşaması geri çevrilmez ("sonucu"→"sonuc"); çözümleyicinin aday üretimi
/// iki yönü de denediğinden gövde içi başvurular tutarlı çözülür.
pub(super) fn yalin_ad(ekli: &str) -> String {
    for ek in ["yı", "yi", "yu", "yü", "ı", "i", "u", "ü"] {
        if let Some(kok) = ekli.strip_suffix(ek) {
            let harfler: Vec<char> = kok.chars().collect();
            if harfler.len() >= 2 {
                // Ünsüz ikizleşmesi geri çevrimi (K-049): "üssü al" → üs,
                // "affı al" → af. (Belirtme ekiyle ikiz açığa çıkar.)
                let n = harfler.len();
                let unlu = |k: char| "aeıioöuüAEIİOÖUÜ".contains(k);
                if n >= 3 && harfler[n - 1] == harfler[n - 2] && !unlu(harfler[n - 1]) {
                    return harfler[..n - 1].iter().collect();
                }
                return kok.to_string();
            }
        }
    }
    ekli.to_string()
}

/// Satır sonu, tanımlı bir işlem adıyla bitiyorsa çağrı üretir (K-016 geçici
/// sözdizimi): `<argümanlar> için/ile <işlem adı>`; argümanlar "ve" ile ayrılır.
/// En uzun ad önce denenir (determinizm).
pub(super) fn cagri_kalibi(
    tokenlar: &[Token],
    satir: usize,
    islemler: &[String],
) -> Result<Option<Ifade>, Tani> {
    cagri_kalibi_ic(tokenlar, satir, islemler)?
        .map(|ifade| konumlu_ifade(ifade, tokenlar))
        .transpose()
}

fn cagri_kalibi_ic(
    tokenlar: &[Token],
    satir: usize,
    islemler: &[String],
) -> Result<Option<Ifade>, Tani> {
    let n = tokenlar.len();
    if n == 0 {
        return Ok(None);
    }

    let adaylar = sirali_islem_adlari(islemler);

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
            // Her argüman bir BÖLGEDİR (v0.2, S020 esnetildi): tek değer ya da
            // yapılı kalıp ("sayıların adedi", "3,14", "yanıtın sayısı"...).
            let mut bolge: Vec<Token> = Vec::new();
            for token in arg_bolgesi {
                if kelime_mi(token, "ve") {
                    if bolge.is_empty() {
                        return Err(arg_hatasi(ad, satir));
                    }
                    argumanlar.push(bolge_ifadesi(&bolge, satir, islemler)?);
                    bolge.clear();
                } else {
                    bolge.push(token.clone());
                }
            }
            if bolge.is_empty() {
                return Err(arg_hatasi(ad, satir));
            }
            argumanlar.push(bolge_ifadesi(&bolge, satir, islemler)?);
        }

        return Ok(Some(Ifade::IslemCagrisi {
            islem_adi: ad.clone(),
            islem_kimligi: None,
            argumanlar,
            satir,
        }));
    }

    Ok(None)
}

/// Bütün bölge görünür işlem adıysa eski sıfır-argüman çağrısını korur.
/// Suffix tabanlı parametreli çağrı, primary/postfix katmanından sonra denenir.
pub(super) fn sifir_argumanli_cagri(tokenlar: &[Token], satir: usize, islemler: &[String]) -> Option<Ifade> {
    let n = tokenlar.len();
    for ad in sirali_islem_adlari(islemler) {
        let kelimeler: Vec<&str> = ad.split(' ').collect();
        if kelimeler.len() == n
            && tokenlar
                .iter()
                .zip(&kelimeler)
                .all(|(token, kelime)| matches!(&token.tur, TokenTur::Kelime(t) if t == kelime))
        {
            return Some(Ifade::IslemCagrisi {
                islem_adi: ad.clone(),
                islem_kimligi: None,
                argumanlar: Vec::new(),
                satir,
            });
        }
    }
    None
}

pub(super) fn sirali_islem_adlari(islemler: &[String]) -> Vec<&String> {
    let mut adaylar: Vec<&String> = islemler.iter().collect();
    adaylar.sort_by_key(|ad| std::cmp::Reverse(ad.split(' ').count()));
    adaylar
}

pub(super) fn arg_hatasi(ad: &str, satir: usize) -> Tani {
    Tani::yeni(
        "S020",
        format!("\"{}\" çağrısında \"ve\"nin iki yanında da bir argüman olmalı.", ad),
        satir,
        1,
        1,
    )
    .onerili("Örnek: \"Ayşe\" ve 10 ile selamla — argüman bir kalıp da olabilir: sayıların adedi ve 3 ile ...".into())
}

pub(super) fn goster(token: &Token) -> String {
    match &token.tur {
        TokenTur::Metin(m) => format!("\"{}\"", m),
        TokenTur::TamSayi(s) => s.to_string(),
        TokenTur::Kelime(k) => k.clone(),
        _ => "?".into(),
    }
}
