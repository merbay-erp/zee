//! Ad çözümleme + tür denetimi.
//!
//! Ad çözümleme, K-011'deki kuralı uygular: tanımlayıcılara ekler bitişik
//! yazılır ("sayıyı", "toplamı", "sayacı"). Çözüm TAHMİNLE DEĞİL, aday kök
//! üretip kapsamdaki tanımlı adlarla eşleyerek yapılır — birden çok aday
//! eşleşirse bu bir hatadır (determinizm, manifesto 3).
//!
//! Ünsüz yumuşamasının geri çevrimi desteklenir: "sayacı" → "sayac" → "sayaç".

use crate::agac::{Cumle, Ifade, Islec};

use crate::tani::Tani;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tur {
    TamSayi,
    Metin,
    Mantiksal,
    /// v0: yalnız TamSayı öğeli liste (Liste<TamSayı>).
    Liste,
}

impl Tur {
    pub fn adi(&self) -> &'static str {
        match self {
            Tur::TamSayi => "TamSayı",
            Tur::Metin => "Metin",
            Tur::Mantiksal => "Mantıksal",
            Tur::Liste => "Liste",
        }
    }
}

/// Programı yerinde çözümler ve tür denetiminden geçirir.
pub fn denetle(program: &mut [Cumle]) -> Result<(), Tani> {
    let mut ortam: HashMap<String, Tur> = HashMap::new();
    blok_denetle(program, &mut ortam)
}

fn blok_denetle(cumleler: &mut [Cumle], ortam: &mut HashMap<String, Tur>) -> Result<(), Tani> {
    for cumle in cumleler {
        match cumle {
            Cumle::Yaz { deger, satir } => {
                let satir = *satir;
                ifade_denetle(deger, ortam, satir)?;
            }
            Cumle::Olsun { ad, deger, satir, sutun, uzunluk } => {
                let satir = *satir;
                let tur = ifade_denetle(deger, ortam, satir)?;
                if let Some(eski) = ortam.get(ad.as_str()) {
                    if *eski != tur {
                        return Err(Tani::yeni(
                            "T002",
                            format!(
                                "\"{}\" daha önce {} olarak tanımlandı; şimdi {} verilemez.",
                                ad,
                                eski.adi(),
                                tur.adi()
                            ),
                            satir,
                            *sutun,
                            *uzunluk,
                        )
                        .onerili(format!(
                            "Bir değerin türü sonradan değişemez. Farklı türde bir değer \
                             gerekiyorsa yeni bir ad kullan; ya da \"{}\" değerine yine {} türünde \
                             bir değer ver.",
                            ad,
                            eski.adi()
                        )));
                    }
                }
                ortam.insert(ad.clone(), tur);
            }
            Cumle::KezTekrarla { adet, govde, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(adet, ortam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T003",
                        format!("Tekrar adedi TamSayı olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                blok_denetle(govde, ortam)?;
            }
            Cumle::AralikDongusu { ad, bastan, sona, govde, satir } => {
                let satir = *satir;
                for uc in [&mut *bastan, &mut *sona] {
                    let tur = ifade_denetle(uc, ortam, satir)?;
                    if tur != Tur::TamSayi {
                        return Err(Tani::yeni(
                            "T004",
                            format!("Aralık uçları TamSayı olmalı; burada {} var.", tur.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                // Döngü değişkeni gövde kapsamında tanımlıdır (RFC-0004'e not:
                // v0'da düz kapsam kullanılıyor).
                ortam.insert(ad.clone(), Tur::TamSayi);
                blok_denetle(govde, ortam)?;
            }
            Cumle::OlduguSurece { kosul, govde, satir }
            | Cumle::OlanaKadar { kosul, govde, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(kosul, ortam, satir)?;
                if tur != Tur::Mantiksal {
                    return Err(Tani::yeni(
                        "T005",
                        "Koşullu döngü bir koşul ister.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
                blok_denetle(govde, ortam)?;
            }
            Cumle::Ise { kollar, degilse, satir } => {
                let satir = *satir;
                for kol in kollar.iter_mut() {
                    let tur = ifade_denetle(&mut kol.kosul, ortam, satir)?;
                    if tur != Tur::Mantiksal {
                        return Err(Tani::yeni("T005", "\"ise\" bir koşul ister.".into(), satir, 1, 1));
                    }
                    blok_denetle(&mut kol.govde, ortam)?;
                }
                if let Some(blok) = degilse {
                    blok_denetle(blok, ortam)?;
                }
            }
            Cumle::Ekle { hedef, deger, satir } => {
                let satir = *satir;
                let hedef_tur = ifade_denetle(hedef, ortam, satir)?;
                if hedef_tur != Tur::Liste {
                    return Err(Tani::yeni(
                        "T012",
                        format!("Ekleme bir listeye yapılır; hedef {} türünde.", hedef_tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Önce \"<ad> boş liste olsun\" ya da \"... listesi olsun\" ile liste tanımla.".into()));
                }
                let deger_tur = ifade_denetle(deger, ortam, satir)?;
                if deger_tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T011",
                        format!("v0'da listeler yalnız TamSayı tutar; {} eklenemez.", deger_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::HerBiri { ad, kaynak, govde, satir } => {
                let satir = *satir;
                if kaynak.is_none() {
                    // Örtük çoğul (K-013): "her sayı için" → kapsamda "sayılar" aranır.
                    let adaylar = [format!("{}lar", ad), format!("{}ler", ad)];
                    let bulunanlar: Vec<String> = adaylar
                        .iter()
                        .filter(|aday| ortam.get(aday.as_str()) == Some(&Tur::Liste))
                        .cloned()
                        .collect();
                    match bulunanlar.len() {
                        1 => {
                            let kaynak_adi = bulunanlar.into_iter().next().unwrap();
                            *kaynak = Some(Ifade::Degisken {
                                ham: kaynak_adi.clone(),
                                cozulmus: Some(kaynak_adi),
                                satir,
                                sutun: 1,
                                uzunluk: 1,
                            });
                        }
                        0 => {
                            return Err(Tani::yeni(
                                "A003",
                                format!(
                                    "\"her {} için\" gezilecek listeyi bulamadı: kapsamda \"{}lar\" ya da \"{}ler\" adında bir liste yok.",
                                    ad, ad, ad
                                ),
                                satir,
                                1,
                                1,
                            )
                            .onerili(format!("Önce listeyi tanımla: {}lar 1, 2, 3 listesi olsun", ad)))
                        }
                        _ => {
                            return Err(Tani::yeni(
                                "A002",
                                format!("\"her {} için\" iki listeye birden çözülebiliyor.", ad),
                                satir,
                                1,
                                1,
                            ))
                        }
                    }
                }
                if let Some(k) = kaynak {
                    let tur = ifade_denetle(k, ortam, satir)?;
                    if tur != Tur::Liste {
                        return Err(Tani::yeni(
                            "T013",
                            format!("\"her ... için\" bir liste ister; burada {} var.", tur.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                ortam.insert(ad.clone(), Tur::TamSayi);
                blok_denetle(govde, ortam)?;
            }
            Cumle::Sor { istem, satir } => {
                let satir = *satir;
                ifade_denetle(istem, ortam, satir)?;
                // Son cevap örtük "yanıt" adına Metin olarak bağlanır (K-007).
                ortam.insert("yanıt".to_string(), Tur::Metin);
            }
            Cumle::Artir { ifade, miktar, satir } | Cumle::Azalt { ifade, miktar, satir } => {
                let satir = *satir;
                let hedef_tur = ifade_denetle(ifade, ortam, satir)?;
                if hedef_tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T006",
                        format!("Artırma/azaltma sayı ister; hedef {} türünde.", hedef_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                let miktar_tur = ifade_denetle(miktar, ortam, satir)?;
                if miktar_tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T006",
                        format!("Artırma/azaltma miktarı TamSayı olmalı; burada {} var.", miktar_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
        }
    }
    Ok(())
}

fn ifade_denetle(
    ifade: &mut Ifade,
    ortam: &HashMap<String, Tur>,
    satir: usize,
) -> Result<Tur, Tani> {
    match ifade {
        Ifade::MetinSabiti(_) => Ok(Tur::Metin),
        Ifade::SayiSabiti(_) => Ok(Tur::TamSayi),
        Ifade::MantiksalSabiti(_) => Ok(Tur::Mantiksal),
        Ifade::BosListe => Ok(Tur::Liste),
        Ifade::ListeSabiti(ogeler) => {
            for oge in ogeler {
                let tur = ifade_denetle(oge, ortam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T011",
                        format!("v0'da listeler yalnız TamSayı tutar; öğelerden biri {}.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Ok(Tur::Liste)
        }
        Ifade::Ozellik { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, satir)?;
            if tur != Tur::Liste {
                return Err(Tani::yeni(
                    "T014",
                    format!("adedi/ilki/sonu bir listenin özellikleridir; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::TamSayi)
        }
        Ifade::Rastgele { alt, ust } => {
            for uc in [&mut **alt, &mut **ust] {
                let tur = ifade_denetle(uc, ortam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T010",
                        format!("Rastgele sayının uçları TamSayı olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Ok(Tur::TamSayi)
        }
        Ifade::Degisken { ham, cozulmus, satir, sutun, uzunluk } => {
            let ad = ad_cozumle(ham, ortam, *satir, *sutun, *uzunluk)?;
            let tur = ortam[&ad];
            *cozulmus = Some(ad);
            Ok(tur)
        }
        Ifade::Birlestir(parcalar) => {
            for parca in parcalar {
                ifade_denetle(parca, ortam, satir)?;
            }
            // "ile" zinciri yazım bağlamında metne birleşir (K-004).
            Ok(Tur::Metin)
        }
        Ifade::Karsilastirma { sol, sag, islec } => {
            let sol_tur = ifade_denetle(sol, ortam, satir)?;
            let sag_tur = ifade_denetle(sag, ortam, satir)?;
            let esitlik = *islec == Islec::Esit;
            if !esitlik && (sol_tur != Tur::TamSayi || sag_tur != Tur::TamSayi) {
                let sorunlu = if sol_tur != Tur::TamSayi { sol_tur } else { sag_tur };
                return Err(Tani::yeni(
                    "T001",
                    format!(
                        "Büyüklük karşılaştırması sayılar arasında yapılır; burada {} var.",
                        sorunlu.adi()
                    ),
                    satir,
                    1,
                    1,
                )
                .onerili("Karşılaştırılan iki değerin de sayı olduğundan emin ol.".into()));
            }
            if esitlik && sol_tur != sag_tur {
                return Err(Tani::yeni(
                    "T001",
                    format!(
                        "Eşitlik ancak aynı türden değerler arasında sorgulanır: {} ile {} karşılaştırılamaz.",
                        sol_tur.adi(),
                        sag_tur.adi()
                    ),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::Cift(ic) | Ifade::Tek(ic) => {
            let tur = ifade_denetle(ic, ortam, satir)?;
            if tur != Tur::TamSayi {
                return Err(Tani::yeni(
                    "T007",
                    format!("Çift/tek sorgusu TamSayı ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::Aritmetik { sol, sag, .. } => {
            for taraf in [&mut **sol, &mut **sag] {
                let tur = ifade_denetle(taraf, ortam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T008",
                        format!("Aritmetik işlem sayılar arasında yapılır; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili(
                        "Metni sayıya çevirmek için \"<metnin> sayısı\" kalıbını kullan.".into(),
                    ));
                }
            }
            Ok(Tur::TamSayi)
        }
        Ifade::Sayisi(ic) => {
            let tur = ifade_denetle(ic, ortam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T009",
                    format!("\"sayısı\" kalıbı Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::TamSayi)
        }
    }
}

/// Hal eki almış tanımlayıcıyı kapsamdaki tanımlı adlara karşı çözer.
pub fn ad_cozumle(
    ham: &str,
    ortam: &HashMap<String, Tur>,
    satir: usize,
    sutun: usize,
    uzunluk: usize,
) -> Result<String, Tani> {
    // Doğrudan eşleşme her zaman kazanır.
    if ortam.contains_key(ham) {
        return Ok(ham.to_string());
    }

    let adaylar = kok_adaylari(ham);
    let eslesenler: Vec<String> = adaylar
        .into_iter()
        .filter(|aday| ortam.contains_key(aday))
        .collect();

    match eslesenler.len() {
        1 => Ok(eslesenler.into_iter().next().unwrap()),
        0 => {
            let tanimli: Vec<&str> = ortam.keys().map(|s| s.as_str()).collect();
            let oneri = if tanimli.is_empty() {
                "Bir değeri kullanmadan önce \"<ad> <değer> olsun\" ile tanımla.".to_string()
            } else {
                format!(
                    "Bu ad tanımlı değil. Tanımlı adlar: {}. Önce \"<ad> <değer> olsun\" ile tanımla.",
                    tanimli.join(", ")
                )
            };
            Err(Tani::yeni(
                "A001",
                format!("\"{}\" adı bu kapsamda tanımlı değil.", ham),
                satir,
                sutun,
                uzunluk,
            )
            .onerili(oneri))
        }
        _ => Err(Tani::yeni(
            "A002",
            format!(
                "\"{}\" birden çok ada çözülebiliyor: {}. Hangisini kastettiğin belirsiz.",
                ham,
                eslesenler.join(", ")
            ),
            satir,
            sutun,
            uzunluk,
        )
        .onerili("Adlardan birini değiştir; belirsizlik dilde hata sayılır.".into())),
    }
}

/// Ek ayıklama adayları: yaygın hal/iyelik/araç ekleri + ünsüz yumuşaması geri çevrimi.
/// (K-011: desteklenen ek listesi sürümlemeli grammar'ın parçasıdır.)
fn kok_adaylari(ham: &str) -> Vec<String> {
    const EKLER: [&str; 34] = [
        "yı", "yi", "yu", "yü", // belirtme (ünlüyle biten kök)
        "nın", "nin", "nun", "nün", // tamlayan
        "ın", "in", "un", "ün", // tamlayan (ünsüzle biten kök)
        "ı", "i", "u", "ü", // belirtme
        "yla", "yle", // araç (ünlüyle biten kök)
        "la", "le", // araç
        "dan", "den", "tan", "ten", // ayrılma
        "da", "de", "ta", "te", // bulunma
        "ya", "ye", // yönelme (ünlüyle biten kök)
        "a", "e", // yönelme
        "lara", "lere", // yönelme (çoğul)
    ];

    let mut adaylar = Vec::new();
    for ek in EKLER {
        if let Some(kok) = ham.strip_suffix(ek) {
            if kok.chars().count() < 2 {
                continue;
            }
            adaylar.push(kok.to_string());
            // Ünsüz yumuşaması geri çevrimi: sayacı→sayac→sayaç, kitabı→kitab→kitap,
            // yurdu→yurd→yurt, çocuğu→çocuğ→çocuk.
            let mut karakterler: Vec<char> = kok.chars().collect();
            if let Some(son) = karakterler.last().copied() {
                let sertlesmis = match son {
                    'c' => Some('ç'),
                    'b' => Some('p'),
                    'd' => Some('t'),
                    'ğ' => Some('k'),
                    _ => None,
                };
                if let Some(yeni) = sertlesmis {
                    *karakterler.last_mut().unwrap() = yeni;
                    adaylar.push(karakterler.into_iter().collect());
                }
            }
        }
    }
    adaylar
}
