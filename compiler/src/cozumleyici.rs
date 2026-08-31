//! Ad çözümleme + tür denetimi.
//!
//! Ad çözümleme, K-011'deki kuralı uygular: tanımlayıcılara ekler bitişik
//! yazılır ("sayıyı", "toplamı", "sayacı"). Çözüm TAHMİNLE DEĞİL, aday kök
//! üretip kapsamdaki tanımlı adlarla eşleyerek yapılır — birden çok aday
//! eşleşirse bu bir hatadır (determinizm, manifesto 3).
//!
//! Ünsüz yumuşamasının geri çevrimi desteklenir: "sayacı" → "sayac" → "sayaç".

use crate::agac::{Cumle, Ifade, Islec, Islem, Program};

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
///
/// İşlemler ilk çağrı anında, argüman türleriyle denetlenir (v0 monomorfizmi):
/// imza ilk çağrıda sabitlenir, sonraki çağrılar imzaya uymalıdır.
pub fn denetle(program: &mut Program) -> Result<(), Tani> {
    let mut ortam: HashMap<String, Tur> = HashMap::new();
    let mut baglam = Baglam {
        islemler: std::mem::take(&mut program.islemler),
        imzalar: HashMap::new(),
    };
    let mut donusler = Vec::new();
    let sonuc = blok_denetle(&mut program.cumleler, &mut ortam, &mut baglam, &mut donusler, false);
    program.islemler = baglam.islemler;
    sonuc
}

/// İlk çağrıda sabitlenen işlem imzası.
struct Imza {
    parametre_turleri: Vec<Tur>,
    donus: Option<Tur>,
}

struct Baglam {
    islemler: HashMap<String, Islem>,
    imzalar: HashMap<String, Imza>,
}

fn blok_denetle(
    cumleler: &mut [Cumle],
    ortam: &mut HashMap<String, Tur>,
    baglam: &mut Baglam,
    donusler: &mut Vec<Tur>,
    islem_icinde: bool,
) -> Result<(), Tani> {
    for cumle in cumleler {
        match cumle {
            Cumle::Yaz { deger, satir } => {
                let satir = *satir;
                ifade_denetle(deger, ortam, baglam, satir)?;
            }
            Cumle::Olsun { ad, deger, satir, sutun, uzunluk } => {
                let satir = *satir;
                let tur = ifade_denetle(deger, ortam, baglam, satir)?;
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
                let tur = ifade_denetle(adet, ortam, baglam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T003",
                        format!("Tekrar adedi TamSayı olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                blok_denetle(govde, ortam, baglam, donusler, islem_icinde)?;
            }
            Cumle::AralikDongusu { ad, bastan, sona, govde, satir } => {
                let satir = *satir;
                for uc in [&mut *bastan, &mut *sona] {
                    let tur = ifade_denetle(uc, ortam, baglam, satir)?;
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
                blok_denetle(govde, ortam, baglam, donusler, islem_icinde)?;
            }
            Cumle::OlduguSurece { kosul, govde, satir }
            | Cumle::OlanaKadar { kosul, govde, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(kosul, ortam, baglam, satir)?;
                if tur != Tur::Mantiksal {
                    return Err(Tani::yeni(
                        "T005",
                        "Koşullu döngü bir koşul ister.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
                blok_denetle(govde, ortam, baglam, donusler, islem_icinde)?;
            }
            Cumle::Ise { kollar, degilse, satir } => {
                let satir = *satir;
                for kol in kollar.iter_mut() {
                    let tur = ifade_denetle(&mut kol.kosul, ortam, baglam, satir)?;
                    if tur != Tur::Mantiksal {
                        return Err(Tani::yeni("T005", "\"ise\" bir koşul ister.".into(), satir, 1, 1));
                    }
                    blok_denetle(&mut kol.govde, ortam, baglam, donusler, islem_icinde)?;
                }
                if let Some(blok) = degilse {
                    blok_denetle(blok, ortam, baglam, donusler, islem_icinde)?;
                }
            }
            Cumle::Ekle { hedef, deger, satir } => {
                let satir = *satir;
                let hedef_tur = ifade_denetle(hedef, ortam, baglam, satir)?;
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
                let deger_tur = ifade_denetle(deger, ortam, baglam, satir)?;
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
                    let tur = ifade_denetle(k, ortam, baglam, satir)?;
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
                blok_denetle(govde, ortam, baglam, donusler, islem_icinde)?;
            }
            Cumle::Sor { istem, satir } => {
                let satir = *satir;
                ifade_denetle(istem, ortam, baglam, satir)?;
                // Son cevap örtük "yanıt" adına Metin olarak bağlanır (K-007).
                ortam.insert("yanıt".to_string(), Tur::Metin);
            }
            Cumle::IslemTanimi(islem) => {
                // Hoist sonrası burada görünmemeli.
                return Err(Tani::yeni(
                    "S021",
                    format!("\"{}\" işlem tanımı beklenmeyen yerde.", islem.ad),
                    islem.satir,
                    1,
                    1,
                ));
            }
            Cumle::Dondur { deger, satir } => {
                let satir = *satir;
                if !islem_icinde {
                    return Err(Tani::yeni(
                        "T020",
                        "\"döndür\" yalnız bir işlemin içinde kullanılır.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
                let tur = ifade_denetle(deger, ortam, baglam, satir)?;
                donusler.push(tur);
            }
            Cumle::BolVeAta { hedef, pay, payda, satir } => {
                let satir = *satir;
                for taraf in [&mut *pay, &mut *payda] {
                    let tur = ifade_denetle(taraf, ortam, baglam, satir)?;
                    if tur != Tur::TamSayi {
                        return Err(Tani::yeni(
                            "T008",
                            format!("Bölme sayılar arasında yapılır; burada {} var.", tur.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                if let Some(eski) = ortam.get(hedef.as_str()) {
                    if *eski != Tur::TamSayi {
                        return Err(Tani::yeni(
                            "T002",
                            format!("\"{}\" {} türünde; bölme sonucu verilemez.", hedef, eski.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                ortam.insert(hedef.clone(), Tur::TamSayi);
            }
            Cumle::CagriCumlesi { cagri, satir } => {
                let satir = *satir;
                if let Ifade::IslemCagrisi { islem_adi, argumanlar, .. } = cagri {
                    let mut arg_turleri = Vec::new();
                    for arg in argumanlar.iter_mut() {
                        arg_turleri.push(ifade_denetle(arg, ortam, baglam, satir)?);
                    }
                    // Cümle konumunda dönüş değeri kullanılmaz; Some/None fark etmez.
                    cagri_denetle(islem_adi, &arg_turleri, baglam, satir)?;
                } else {
                    return Err(Tani::yeni("S004", "Geçersiz çağrı cümlesi.".into(), satir, 1, 1));
                }
            }
            Cumle::Artir { ifade, miktar, satir } | Cumle::Azalt { ifade, miktar, satir } => {
                let satir = *satir;
                let hedef_tur = ifade_denetle(ifade, ortam, baglam, satir)?;
                if hedef_tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T006",
                        format!("Artırma/azaltma sayı ister; hedef {} türünde.", hedef_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                let miktar_tur = ifade_denetle(miktar, ortam, baglam, satir)?;
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
    baglam: &mut Baglam,
    satir: usize,
) -> Result<Tur, Tani> {
    match ifade {
        Ifade::MetinSabiti(_) => Ok(Tur::Metin),
        Ifade::SayiSabiti(_) => Ok(Tur::TamSayi),
        Ifade::MantiksalSabiti(_) => Ok(Tur::Mantiksal),
        Ifade::BosListe => Ok(Tur::Liste),
        Ifade::ListeSabiti(ogeler) => {
            for oge in ogeler {
                let tur = ifade_denetle(oge, ortam, baglam, satir)?;
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
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
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
                let tur = ifade_denetle(uc, ortam, baglam, satir)?;
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
                ifade_denetle(parca, ortam, baglam, satir)?;
            }
            // "ile" zinciri yazım bağlamında metne birleşir (K-004).
            Ok(Tur::Metin)
        }
        Ifade::Karsilastirma { sol, sag, islec } => {
            let sol_tur = ifade_denetle(sol, ortam, baglam, satir)?;
            let sag_tur = ifade_denetle(sag, ortam, baglam, satir)?;
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
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
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
                let tur = ifade_denetle(taraf, ortam, baglam, satir)?;
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
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
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
        Ifade::IslemCagrisi { islem_adi, argumanlar, satir: cagri_satiri } => {
            let cagri_satiri = *cagri_satiri;
            let mut arg_turleri = Vec::new();
            for arg in argumanlar.iter_mut() {
                arg_turleri.push(ifade_denetle(arg, ortam, baglam, cagri_satiri)?);
            }
            match cagri_denetle(islem_adi, &arg_turleri, baglam, cagri_satiri)? {
                Some(tur) => Ok(tur),
                None => Err(Tani::yeni(
                    "T019",
                    format!(
                        "\"{}\" bir değer döndürmüyor; burada değer bekleniyor.",
                        islem_adi
                    ),
                    cagri_satiri,
                    1,
                    1,
                )
                .onerili("İşlemin içinde \"... döndür\" ile bir sonuç döndür.".into())),
            }
        }
    }
}

/// İşlem çağrısını denetler. İlk çağrıda gövde, argüman türleriyle denetlenip
/// imza sabitlenir; sonraki çağrılar imzaya uymalıdır. Gövde denetlenirken
/// işlem kayıttan geçici olarak çıkarılır — bu da özyinelemeyi doğal biçimde
/// yakalar (v0'da desteklenmez).
fn cagri_denetle(
    ad: &str,
    arg_turleri: &[Tur],
    baglam: &mut Baglam,
    satir: usize,
) -> Result<Option<Tur>, Tani> {
    if let Some(imza) = baglam.imzalar.get(ad) {
        if imza.parametre_turleri.len() != arg_turleri.len() {
            return Err(Tani::yeni(
                "T015",
                format!(
                    "\"{}\" {} parametre bekler, {} argüman verildi.",
                    ad,
                    imza.parametre_turleri.len(),
                    arg_turleri.len()
                ),
                satir,
                1,
                1,
            ));
        }
        if imza.parametre_turleri != arg_turleri {
            return Err(Tani::yeni(
                "T017",
                format!("\"{}\" çağrısındaki argüman türleri işlemin imzasına uymuyor.", ad),
                satir,
                1,
                1,
            ));
        }
        return Ok(imza.donus);
    }

    let mut islem = baglam.islemler.remove(ad).ok_or_else(|| {
        Tani::yeni(
            "T016",
            format!(
                "\"{}\" işlemi burada çağrılamaz: v0'da bir işlem kendi kendini çağıramaz.",
                ad
            ),
            satir,
            1,
            1,
        )
    })?;

    if islem.parametreler.len() != arg_turleri.len() {
        baglam.islemler.insert(ad.to_string(), islem);
        return Err(Tani::yeni(
            "T015",
            format!(
                "\"{}\" {} parametre bekler, {} argüman verildi.",
                ad,
                baglam.islemler[ad].parametreler.len(),
                arg_turleri.len()
            ),
            satir,
            1,
            1,
        ));
    }

    let mut islem_ortami: HashMap<String, Tur> = HashMap::new();
    for (param, tur) in islem.parametreler.iter().zip(arg_turleri) {
        islem_ortami.insert(param.clone(), *tur);
    }

    let mut donusler = Vec::new();
    let denetim = blok_denetle(&mut islem.govde, &mut islem_ortami, baglam, &mut donusler, true);
    // Gövde her durumda kayda geri konur; hata olsa bile kayıt tutarlı kalır.
    let denetim_sonucu = denetim;
    baglam.islemler.insert(ad.to_string(), islem);
    denetim_sonucu?;

    let donus = match donusler.split_first() {
        None => None,
        Some((ilk, kalan)) => {
            if kalan.iter().any(|t| t != ilk) {
                return Err(Tani::yeni(
                    "T018",
                    format!("\"{}\" farklı türlerde değerler döndürüyor; tek tür seç.", ad),
                    satir,
                    1,
                    1,
                ));
            }
            Some(*ilk)
        }
    };

    baglam.imzalar.insert(
        ad.to_string(),
        Imza { parametre_turleri: arg_turleri.to_vec(), donus },
    );
    Ok(donus)
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
