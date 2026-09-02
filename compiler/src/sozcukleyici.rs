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
    /// Ondalık sabit (RFC-0013): `3,14` → govde="314", olcek=2. Onluk tam değer.
    Ondalik { govde: String, olcek: u32 },
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

/// Tanımlayıcılarda geçerli karakterler: ASCII harf/rakam, alt çizgi,
/// Türkçe harfler ve Türkçe yazımda kullanılan şapkalı ünlüler (kâr, îma).
fn harf_gecerli(k: char) -> bool {
    k.is_ascii_alphanumeric()
        || k == '_'
        || "çÇğĞıİöÖşŞüÜâÂîÎûÛ".contains(k)
}

/// Kaynağı tokenlara ayırır. İlk hatada durur (v0 davranışı).
pub fn sozcukle(kaynak: &str) -> Result<Vec<Token>, Tani> {
    crate::kaynak_sinirlari::kaynak_boyutunu_denetle(kaynak)?;
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
        let onceki = girinti_yigini.last().copied().unwrap_or(0);
        if girinti > onceki {
            girinti_yigini.push(girinti);
            token_ekle(
                &mut tokenlar,
                Token::yeni(TokenTur::Girinti, satir_no, 1, girinti),
            )?;
        } else if girinti < onceki {
            while girinti_yigini.last().copied().unwrap_or(0) > girinti {
                girinti_yigini.pop();
                token_ekle(
                    &mut tokenlar,
                    Token::yeni(TokenTur::Cikinti, satir_no, 1, 1),
                )?;
            }
            if girinti_yigini.last().copied().unwrap_or(0) != girinti {
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
        // Token'lar hiçbir zaman boşlukla bitmez; bu yüzden "önceki karakter
        // boşluktu" bilgisi her turda başlangıç karakterinden türetilebilir.
        let mut bosluktan_sonra = true;
        while let Some(&k) = kalanlar.peek() {
            let bosluk_mu = k == ' ';
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
                while let Some(ic) = kalanlar.next() {
                    sutun += 1;
                    if ic == '"' {
                        kapandi = true;
                        break;
                    }
                    // Kaçışlar (RFC-0002 §6.1): \" tırnak, \\ ters bölü, \n yeni satır.
                    if ic == '\\' {
                        match kalanlar.next() {
                            Some('"') => {
                                sutun += 1;
                                icerik.push('"');
                            }
                            Some('\\') => {
                                sutun += 1;
                                icerik.push('\\');
                            }
                            Some('n') => {
                                sutun += 1;
                                icerik.push('\n');
                            }
                            baska => {
                                return Err(Tani::yeni(
                                    "S040",
                                    format!(
                                        "Bilinmeyen kaçış dizisi: \\{}",
                                        baska.map(String::from).unwrap_or_default()
                                    ),
                                    satir_no,
                                    sutun,
                                    2,
                                )
                                .onerili(
                                    "Metin içinde tırnak için \\\", ters bölü için \\\\, \
                                     yeni satır için \\n kullanılır."
                                        .into(),
                                ));
                            }
                        }
                        continue;
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
                token_ekle(
                    &mut tokenlar,
                    Token::yeni(TokenTur::Metin(icerik), satir_no, baslangic_sutun, uzunluk),
                )?;
            } else if k.is_ascii_digit()
                || (k == '-'
                    && kalanlar.clone().nth(1).map(|r| r.is_ascii_digit()) == Some(true))
            {
                let baslangic_sutun = sutun;
                let mut sayi_metni = String::new();
                if k == '-' {
                    sayi_metni.push('-');
                    kalanlar.next();
                    sutun += 1;
                }
                while let Some(&r) = kalanlar.peek() {
                    if r.is_ascii_digit() {
                        sayi_metni.push(r);
                        kalanlar.next();
                        sutun += 1;
                    } else {
                        break;
                    }
                }
                // Bitişik virgül kuralı (RFC-0013): rakam,rakam → ondalık sabit.
                let ondalik_mi = {
                    let mut ileri = kalanlar.clone();
                    ileri.next() == Some(',')
                        && ileri.peek().map(|r| r.is_ascii_digit()) == Some(true)
                };
                if ondalik_mi {
                    kalanlar.next(); // ','
                    sutun += 1;
                    let mut kesir_metni = String::new();
                    while let Some(&r) = kalanlar.peek() {
                        if r.is_ascii_digit() {
                            kesir_metni.push(r);
                            kalanlar.next();
                            sutun += 1;
                        } else {
                            break;
                        }
                    }
                    let olcek = u32::try_from(kesir_metni.len()).map_err(|_| {
                        Tani::yeni(
                            "S006",
                            "Ondalık sabit işlenemeyecek kadar uzun.".into(),
                            satir_no,
                            baslangic_sutun,
                            sayi_metni.len() + 1 + kesir_metni.len(),
                        )
                    })?;
                    // İşaret-duyarlı metinsel katsayı: boyut yalnız kaynak boyuyla sınırlıdır.
                    let eksi = sayi_metni.starts_with('-');
                    let tam = sayi_metni.trim_start_matches('-').trim_start_matches('0');
                    let govde_rakamlari = format!("{}{}", tam, kesir_metni);
                    let govde_rakamlari = govde_rakamlari.trim_start_matches('0');
                    let govde = if govde_rakamlari.is_empty() {
                        "0".to_string()
                    } else if eksi {
                        format!("-{}", govde_rakamlari)
                    } else {
                        govde_rakamlari.to_string()
                    };
                    token_ekle(&mut tokenlar, Token::yeni(
                        TokenTur::Ondalik { govde, olcek },
                        satir_no,
                        baslangic_sutun,
                        sayi_metni.len() + 1 + kesir_metni.len(),
                    ))?;
                } else {
                    let deger: i64 = sayi_metni.parse().map_err(|_| {
                        Tani::yeni(
                            "S006",
                            format!("\"{}\" sayısı çok büyük.", sayi_metni),
                            satir_no,
                            baslangic_sutun,
                            sayi_metni.len(),
                        )
                    })?;
                    token_ekle(&mut tokenlar, Token::yeni(
                        TokenTur::TamSayi(deger),
                        satir_no,
                        baslangic_sutun,
                        sayi_metni.len(),
                    ))?;
                }
            } else if k.is_alphabetic() || k == '_' {
                let baslangic_sutun = sutun;
                let mut kelime = String::new();
                while let Some(&r) = kalanlar.peek() {
                    if r.is_alphanumeric() || r == '_' {
                        // Homoglyph koruması (A08, bölüm 18): tanımlayıcılar yalnız
                        // Türkçe/Latin harfler taşır. Kiril "а" gibi görünüşte özdeş
                        // karakterler sessizce kabul edilmez, burada yakalanır.
                        if !harf_gecerli(r) {
                            return Err(Tani::yeni(
                                "S028",
                                format!(
                                    "Tanımlayıcıda Türkçe/Latin dışı karakter: \"{}\" (U+{:04X}).",
                                    r, r as u32
                                ),
                                satir_no,
                                sutun,
                                1,
                            )
                            .onerili(
                                "Bu karakter başka bir alfabeden geliyor ve Latin benzerleriyle \
                                 karıştırılabilir. Tanımlayıcılarda yalnız Türkçe/Latin harfler, \
                                 rakamlar ve alt çizgi kullanılabilir."
                                    .into(),
                            ));
                        }
                        kelime.push(r);
                        kalanlar.next();
                        sutun += 1;
                    } else {
                        break;
                    }
                }
                let uzunluk = kelime.chars().count();
                token_ekle(
                    &mut tokenlar,
                    Token::yeni(TokenTur::Kelime(kelime), satir_no, baslangic_sutun, uzunluk),
                )?;
            } else if ('\u{0300}'..='\u{036F}').contains(&k) {
                // Birleştirici imler: v0 kuralı kaynak metnin önceden birleştirilmiş
                // (NFC) karakterlerle yazılmasıdır (RFC-0002).
                return Err(Tani::yeni(
                    "S029",
                    format!("Birleştirici im (U+{:04X}) desteklenmiyor.", k as u32),
                    satir_no,
                    sutun,
                    1,
                )
                .onerili(
                    "Harf ve işaretini ayrı yazmak yerine birleşik karakteri kullan \
                     (örneğin g + ˘ yerine tek karakter ğ)."
                        .into(),
                ));
            } else if k == ',' {
                kalanlar.next();
                // "3 ,14" gibi boşluk-virgül-rakam dizisi: ondalık mı liste mi
                // belirsiz görünür; öğretici tanıyla reddedilir (RFC-0013 §1).
                if bosluktan_sonra
                    && kalanlar.peek().map(|r| r.is_ascii_digit()) == Some(true)
                {
                    return Err(Tani::yeni(
                        "S033",
                        "Virgülden önce boşluk, sonra rakam: ondalık mı liste mi belirsiz.".into(),
                        satir_no,
                        sutun,
                        1,
                    )
                    .onerili(
                        "Ondalık sayıysa bitişik yaz: 3,14 — liste ayracıysa virgülden sonra boşluk bırak: 3, 14"
                            .into(),
                    ));
                }
                token_ekle(
                    &mut tokenlar,
                    Token::yeni(TokenTur::Virgul, satir_no, sutun, 1),
                )?;
                sutun += 1;
            } else {
                let oneri = if k == '.' {
                    "Ondalık ayracı Türkçede virgüldür: 3.14 değil 3,14 yaz (RFC-0013)."
                } else {
                    "Bu dilde noktalama en azdadır: süslü parantez, noktalı virgül ve \
                     sembolik işleçler kullanılmaz. Kalıpları kelimelerle yaz."
                };
                return Err(Tani::yeni(
                    "S001",
                    format!("Beklenmeyen karakter: \"{}\"", k),
                    satir_no,
                    sutun,
                    1,
                )
                .onerili(oneri.into()));
            }
            bosluktan_sonra = bosluk_mu;
        }

        token_ekle(
            &mut tokenlar,
            Token::yeni(TokenTur::SatirSonu, satir_no, sutun, 1),
        )?;
    }

    // Dosya sonunda açık blokları kapat.
    let son_satir = kaynak.lines().count().max(1);
    while girinti_yigini.len() > 1 {
        girinti_yigini.pop();
        token_ekle(
            &mut tokenlar,
            Token::yeni(TokenTur::Cikinti, son_satir, 1, 1),
        )?;
    }
    token_ekle(
        &mut tokenlar,
        Token::yeni(TokenTur::DosyaSonu, son_satir, 1, 1),
    )?;
    Ok(tokenlar)
}

fn token_ekle(tokenlar: &mut Vec<Token>, token: Token) -> Result<(), Tani> {
    crate::kaynak_sinirlari::token_sayisini_denetle(tokenlar.len().saturating_add(1), token.satir)?;
    tokenlar.push(token);
    Ok(())
}
