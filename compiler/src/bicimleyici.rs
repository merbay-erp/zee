//! Biçimleyici (dilfmt çekirdeği) — tek resmi biçim, idempotent.
//!
//! Token temelli çalışır (AST gerekmez): satır içi boşluklar tekleştirilir,
//! virgülden önce boşluk kalkar sonra bir boşluk konur, girinti düzeyi başına
//! 4 boşluk basılır. Yorumlar ve boş satırlar korunur; yorum satırı bir
//! sonraki kod satırının hizasına çekilir.
//!
//! Güvence: biçimlenmiş çıktının token dizisi girdiyle birebir aynı olmalıdır;
//! değilse biçimleyici sonucu YAZMAZ ve iç hata bildirir (determinizm).

use crate::sozcukleyici::{sozcukle, TokenTur};
use crate::tani::Tani;

enum SatirTuru {
    Bos,
    /// Yorum metni ("#" sonrası, kırpılmış).
    Yorum(String),
    /// Token metinleri + varsa satır sonu yorumu.
    Kod(Vec<String>, Option<String>),
}

struct Satir {
    girinti: usize,
    tur: SatirTuru,
    /// Kod satırları için hesaplanan blok düzeyi.
    duzey: usize,
}

pub fn bicimle(kaynak: &str) -> Result<String, Tani> {
    let mut satirlar = Vec::new();

    for (indeks, ham) in kaynak.lines().enumerate() {
        let satir_no = indeks + 1;
        let mut girinti = 0usize;
        let mut karakterler = ham.chars().peekable();
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
        let icerik: String = karakterler.collect();
        let icerik = icerik.trim_end();

        let tur = if icerik.is_empty() {
            SatirTuru::Bos
        } else if let Some(yorum) = icerik.strip_prefix('#') {
            SatirTuru::Yorum(yorum.trim().to_string())
        } else {
            let (tokenlar, yorum) = satiri_parcala(icerik, satir_no)?;
            SatirTuru::Kod(tokenlar, yorum)
        };
        satirlar.push(Satir { girinti, tur, duzey: 0 });
    }

    // Kod satırlarının blok düzeyleri (sözcükleyicidekiyle aynı kural).
    let mut yigin: Vec<usize> = vec![0];
    for satir in satirlar.iter_mut() {
        if let SatirTuru::Kod(..) = satir.tur {
            let onceki = *yigin.last().unwrap();
            if satir.girinti > onceki {
                yigin.push(satir.girinti);
            } else if satir.girinti < onceki {
                while *yigin.last().unwrap() > satir.girinti {
                    yigin.pop();
                }
            }
            satir.duzey = yigin.len() - 1;
        }
    }

    // Yorum ve boş satırlar bir sonraki kod satırının hizasını alır.
    let mut sonraki_duzey = 0usize;
    for satir in satirlar.iter_mut().rev() {
        match satir.tur {
            SatirTuru::Kod(..) => sonraki_duzey = satir.duzey,
            _ => satir.duzey = sonraki_duzey,
        }
    }

    // Basım.
    let mut cikti = String::new();
    for satir in &satirlar {
        match &satir.tur {
            SatirTuru::Bos => cikti.push('\n'),
            SatirTuru::Yorum(yorum) => {
                cikti.push_str(&"    ".repeat(satir.duzey));
                if yorum.is_empty() {
                    cikti.push_str("#\n");
                } else {
                    cikti.push_str(&format!("# {}\n", yorum));
                }
            }
            SatirTuru::Kod(tokenlar, yorum) => {
                cikti.push_str(&"    ".repeat(satir.duzey));
                let mut ilk = true;
                for token in tokenlar {
                    if token == "," {
                        cikti.push(',');
                    } else {
                        if !ilk {
                            cikti.push(' ');
                        }
                        cikti.push_str(token);
                    }
                    ilk = false;
                }
                if let Some(yorum) = yorum {
                    cikti.push_str(&format!("  # {}", yorum));
                }
                cikti.push('\n');
            }
        }
    }

    // Sondaki fazla boş satırları tek satır sonuna indir.
    while cikti.ends_with("\n\n") {
        cikti.pop();
    }
    if !cikti.is_empty() && !cikti.ends_with('\n') {
        cikti.push('\n');
    }

    // Güvence: token dizisi değişmemiş olmalı.
    let once = token_turleri(kaynak)?;
    let sonra = token_turleri(&cikti)?;
    if once != sonra {
        return Err(Tani::yeni(
            "C011",
            "Biçimleyici iç tutarlılık hatası: biçim, programın tokenlarını değiştirecekti. \
             Dosya YAZILMADI; lütfen bildir."
                .into(),
            1,
            1,
            1,
        ));
    }

    Ok(cikti)
}

/// Bir kod satırını token metinlerine ve varsa satır sonu yorumuna ayırır.
fn satiri_parcala(icerik: &str, satir_no: usize) -> Result<(Vec<String>, Option<String>), Tani> {
    let mut tokenlar = Vec::new();
    let mut karakterler = icerik.chars().peekable();
    let mut sutun = 0usize;

    while let Some(&k) = karakterler.peek() {
        sutun += 1;
        if k == ' ' {
            karakterler.next();
        } else if k == '#' {
            karakterler.next();
            let yorum: String = karakterler.collect();
            return Ok((tokenlar, Some(yorum.trim().to_string())));
        } else if k == '"' {
            karakterler.next();
            let mut metin = String::from("\"");
            let mut kapandi = false;
            for ic in karakterler.by_ref() {
                metin.push(ic);
                if ic == '"' {
                    kapandi = true;
                    break;
                }
            }
            if !kapandi {
                return Err(Tani::yeni(
                    "S002",
                    "Metin sabiti kapanmadan satır bitti.".into(),
                    satir_no,
                    sutun,
                    1,
                ));
            }
            tokenlar.push(metin);
        } else if k == ',' {
            karakterler.next();
            // Bitişik virgül kuralı (RFC-0013): rakam,rakam tek ondalık tokendir.
            // Bir ondalık tokenda en çok bir virgül olur: "2,5,7" zinciri
            // "2,5" + ayraç + "7..." biçiminde ayrışmalıdır (sözcükleyiciyle aynı).
            let onceki_rakamla_bitiyor = tokenlar
                .last()
                .map(|t| {
                    t.chars().last().is_some_and(|s| s.is_ascii_digit()) && !t.contains(',')
                })
                == Some(true);
            let sonraki_rakam = karakterler.peek().map(|r| r.is_ascii_digit()) == Some(true);
            if onceki_rakamla_bitiyor && sonraki_rakam {
                let son = tokenlar.last_mut().expect("önceki token var");
                son.push(',');
                while let Some(&r) = karakterler.peek() {
                    if r.is_ascii_digit() {
                        son.push(r);
                        karakterler.next();
                    } else {
                        break;
                    }
                }
            } else {
                tokenlar.push(",".to_string());
            }
        } else {
            // Kelime ya da sayı: boşluk, virgül ve # dışındaki her şey.
            let mut parca = String::new();
            while let Some(&r) = karakterler.peek() {
                if r == ' ' || r == ',' || r == '#' || r == '"' {
                    break;
                }
                parca.push(r);
                karakterler.next();
            }
            tokenlar.push(parca);
        }
    }
    Ok((tokenlar, None))
}

fn token_turleri(kaynak: &str) -> Result<Vec<TokenTur>, Tani> {
    Ok(sozcukle(kaynak)?
        .into_iter()
        .map(|t| t.tur)
        .filter(|t| !matches!(t, TokenTur::SatirSonu))
        .collect())
}
