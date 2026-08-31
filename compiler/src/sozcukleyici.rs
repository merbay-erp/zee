//! Sözcükleyici (lexer).
//!
//! Kaynak `.dil` metnini token dizisine çevirir. Girinti blok yapısını belirlediği
//! için Girinti/Cikinti tokenları burada üretilir (RFC-0003 taslağı: 4 boşluk
//! önerilir, sekme yasak, tutarlı girinti zorunlu).
//!
//! Anahtar kelime YOKTUR: "yaz", "olsun" gibi kelimeler burada sıradan
//! `Kelime` tokenıdır; anlamı ayrıştırıcı (parser) verir. Böylece kelimeler
//! bağlama göre serbest kalır (K-014 ayrılmış kelime tartışması).

use crate::tani::Tani;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenTur {
    /// Çift tırnaklı metin sabiti (tırnaklar hariç içerik).
    Metin(String),
    /// Tam sayı sabiti.
    TamSayi(i64),
    /// Tanımlayıcı ya da kalıp kelimesi (yaz, olsun, ise, ile...).
    Kelime(String),
    Virgul,
    SatirSonu,
    Girinti,
    Cikinti,
    DosyaSonu,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tur: TokenTur,
    pub satir: usize,
    pub sutun: usize,
    pub uzunluk: usize,
}

impl Token {
    fn yeni(tur: TokenTur, satir: usize, sutun: usize, uzunluk: usize) -> Token {
        Token { tur, satir, sutun, uzunluk }
    }
}

/// Kaynağı tokenlara ayırır. İlk hatada durur (v0 davranışı).
pub fn sozcukle(kaynak: &str) -> Result<Vec<Token>, Tani> {
    let mut tokenlar = Vec::new();
    let mut girinti_yigini: Vec<usize> = vec![0];

    for (satir_indeksi, ham_satir) in kaynak.lines().enumerate() {
        let satir_no = satir_indeksi + 1;

        // Girinti ölçümü.
        let mut girinti = 0usize;
        let mut karakterler = ham_satir.chars().peekable();
        while let Some(&k) = karakterler.peek() {
            if k == ' ' {
                girinti += 1;
                karakterler.next();
            } else if k == '\t' {
                return Err(Tani::yeni(
                    "S003",
                    "Girinti için sekme (tab) kullanılamaz.".into(),
                    satir_no,
                    girinti + 1,
                    1,
                )
                .onerili("Sekme yerine 4 boşluk kullan.".into()));
            } else {
                break;
            }
        }

        // Boş satır ya da yalnız yorum içeren satır blok yapısını etkilemez.
        let kalan: String = karakterler.clone().collect();
        if kalan.is_empty() || kalan.starts_with('#') {
            continue;
        }

        // Girinti/Cikinti tokenları.
        let onceki = *girinti_yigini.last().unwrap();
        if girinti > onceki {
            girinti_yigini.push(girinti);
            tokenlar.push(Token::yeni(TokenTur::Girinti, satir_no, 1, girinti));
        } else if girinti < onceki {
            while *girinti_yigini.last().unwrap() > girinti {
                girinti_yigini.pop();
                tokenlar.push(Token::yeni(TokenTur::Cikinti, satir_no, 1, 1));
            }
            if *girinti_yigini.last().unwrap() != girinti {
                return Err(Tani::yeni(
                    "S005",
                    "Girinti hizası önceki bloklardan hiçbiriyle uyuşmuyor.".into(),
                    satir_no,
                    1,
                    girinti.max(1),
                )
                .onerili("Blok içindeki satırları aynı hizada tut; bir blok 4 boşluk içeri girer.".into()));
            }
        }

        // Satır içeriği.
        let mut sutun = girinti + 1;
        let mut kalanlar = kalan.chars().peekable();
        while let Some(&k) = kalanlar.peek() {
            if k == ' ' {
                kalanlar.next();
                sutun += 1;
            } else if k == '#' {
                break; // satır sonu yorumu
            } else if k == '"' {
                let baslangic_sutun = sutun;
                kalanlar.next();
                sutun += 1;
                let mut icerik = String::new();
                let mut kapandi = false;
                for ic in kalanlar.by_ref() {
                    sutun += 1;
                    if ic == '"' {
                        kapandi = true;
                        break;
                    }
                    icerik.push(ic);
                }
                if !kapandi {
                    return Err(Tani::yeni(
                        "S002",
                        "Metin sabiti kapanmadan satır bitti.".into(),
                        satir_no,
                        baslangic_sutun,
                        sutun - baslangic_sutun,
                    )
                    .onerili("Metnin sonuna kapatan \" işaretini ekle.".into()));
                }
                let uzunluk = icerik.chars().count() + 2;
                tokenlar.push(Token::yeni(TokenTur::Metin(icerik), satir_no, baslangic_sutun, uzunluk));
            } else if k.is_ascii_digit() {
                let baslangic_sutun = sutun;
                let mut sayi_metni = String::new();
                while let Some(&r) = kalanlar.peek() {
                    if r.is_ascii_digit() {
                        sayi_metni.push(r);
                        kalanlar.next();
                        sutun += 1;
                    } else {
                        break;
                    }
                }
                let deger: i64 = sayi_metni.parse().map_err(|_| {
                    Tani::yeni(
                        "S006",
                        format!("\"{}\" sayısı çok büyük.", sayi_metni),
                        satir_no,
                        baslangic_sutun,
                        sayi_metni.len(),
                    )
                })?;
                tokenlar.push(Token::yeni(
                    TokenTur::TamSayi(deger),
                    satir_no,
                    baslangic_sutun,
                    sayi_metni.len(),
                ));
            } else if k.is_alphabetic() || k == '_' {
                let baslangic_sutun = sutun;
                let mut kelime = String::new();
                while let Some(&r) = kalanlar.peek() {
                    if r.is_alphanumeric() || r == '_' {
                        kelime.push(r);
                        kalanlar.next();
                        sutun += 1;
                    } else {
                        break;
                    }
                }
                let uzunluk = kelime.chars().count();
                tokenlar.push(Token::yeni(TokenTur::Kelime(kelime), satir_no, baslangic_sutun, uzunluk));
            } else if k == ',' {
                kalanlar.next();
                tokenlar.push(Token::yeni(TokenTur::Virgul, satir_no, sutun, 1));
                sutun += 1;
            } else {
                return Err(Tani::yeni(
                    "S001",
                    format!("Beklenmeyen karakter: \"{}\"", k),
                    satir_no,
                    sutun,
                    1,
                )
                .onerili(
                    "Bu dilde noktalama en azdadır: süslü parantez, noktalı virgül ve \
                     sembolik işleçler kullanılmaz. Kalıpları kelimelerle yaz."
                        .into(),
                ));
            }
        }

        tokenlar.push(Token::yeni(TokenTur::SatirSonu, satir_no, sutun, 1));
    }

    // Dosya sonunda açık blokları kapat.
    let son_satir = kaynak.lines().count().max(1);
    while girinti_yigini.len() > 1 {
        girinti_yigini.pop();
        tokenlar.push(Token::yeni(TokenTur::Cikinti, son_satir, 1, 1));
    }
    tokenlar.push(Token::yeni(TokenTur::DosyaSonu, son_satir, 1, 1));
    Ok(tokenlar)
}
