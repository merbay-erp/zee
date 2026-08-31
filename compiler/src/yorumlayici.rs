//! Ağaç-yürüyen yorumlayıcı (ADR-003: ilk execution modeli).
//!
//! Tür denetiminden geçmiş programı çalıştırır. Çıktı satır listesi olarak
//! döner; CLI bunu ekrana basar, testler doğrudan karşılaştırır.

use crate::agac::{AritmetikIslec, Cumle, Ifade, Islec, Islem, Ozellik, Program};
use crate::tani::Tani;
use std::collections::{HashMap, VecDeque};

/// Girdi/çıktı ve rastgelelik soyutlaması: testler deterministik kuyruk
/// kullanır, CLI gerçek klavye/ekran ve gerçek rastgelelik.
pub trait GirdiCikti {
    fn yazdir(&mut self, satir: String);
    /// İstem gösterilir, bir satır cevap beklenir. `None` = girdi tükendi.
    fn sor(&mut self, istem: &str) -> Option<String>;
    /// [alt, ust] aralığında (uçlar dahil) rastgele sayı.
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64;
}

/// Çıktıyı toplayan, girdiyi ve "rastgele" sayıları hazır kuyruktan veren IO
/// (testler ve lib arayüzü — determinizm burada da korunur).
pub struct ToplayanIo {
    pub girdiler: VecDeque<String>,
    pub rastgele_degerler: VecDeque<i64>,
    pub cikti: Vec<String>,
}

impl ToplayanIo {
    pub fn yeni(girdiler: Vec<String>) -> ToplayanIo {
        ToplayanIo {
            girdiler: girdiler.into(),
            rastgele_degerler: VecDeque::new(),
            cikti: Vec::new(),
        }
    }
}

impl GirdiCikti for ToplayanIo {
    fn yazdir(&mut self, satir: String) {
        self.cikti.push(satir);
    }
    fn sor(&mut self, istem: &str) -> Option<String> {
        self.cikti.push(istem.to_string());
        self.girdiler.pop_front()
    }
    fn rastgele(&mut self, alt: i64, _ust: i64) -> i64 {
        self.rastgele_degerler.pop_front().unwrap_or(alt)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Deger {
    TamSayi(i64),
    Metin(String),
    Mantiksal(bool),
    Liste(Vec<Deger>),
}

impl Deger {
    fn metne(&self) -> String {
        match self {
            Deger::TamSayi(s) => s.to_string(),
            Deger::Metin(m) => m.clone(),
            Deger::Mantiksal(b) => if *b { "doğru" } else { "yanlış" }.to_string(),
            Deger::Liste(ogeler) => ogeler
                .iter()
                .map(|o| o.metne())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
}

pub fn calistir(program: &Program) -> Result<Vec<String>, Tani> {
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(program, &mut io)?;
    Ok(io.cikti)
}

pub fn calistir_io(program: &Program, io: &mut dyn GirdiCikti) -> Result<(), Tani> {
    let mut ortam: HashMap<String, Deger> = HashMap::new();
    blok_calistir(&program.cumleler, &mut ortam, &program.islemler, io)?;
    Ok(())
}

/// Blok çalıştırmanın sonucu: normal akış mı, "döndür" ile erken çıkış mı.
pub enum Akis {
    Devam,
    Don(Deger),
}

fn blok_calistir(
    cumleler: &[Cumle],
    ortam: &mut HashMap<String, Deger>,
    islemler: &HashMap<String, Islem>,
    cikti: &mut dyn GirdiCikti,
) -> Result<Akis, Tani> {
    for cumle in cumleler {
        match cumle {
            Cumle::Yaz { deger, satir } => {
                let sonuc = degerlendir(deger, ortam, islemler, cikti, *satir)?;
                cikti.yazdir(sonuc.metne());
            }
            Cumle::Sor { istem, satir } => {
                let istem = degerlendir(istem, ortam, islemler, cikti, *satir)?.metne();
                let cevap = cikti.sor(&istem).ok_or_else(|| {
                    Tani::yeni(
                        "C005",
                        "Soruya verilecek girdi kalmadı.".into(),
                        *satir,
                        1,
                        1,
                    )
                })?;
                ortam.insert("yanıt".to_string(), Deger::Metin(cevap));
            }
            Cumle::Olsun { ad, deger, satir, .. } => {
                let sonuc = degerlendir(deger, ortam, islemler, cikti, *satir)?;
                ortam.insert(ad.clone(), sonuc);
            }
            Cumle::KezTekrarla { adet, govde, satir } => {
                let adet = tam_sayi(degerlendir(adet, ortam, islemler, cikti, *satir)?, *satir)?;
                for _ in 0..adet.max(0) {
                    if let Akis::Don(d) = blok_calistir(govde, ortam, islemler, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::AralikDongusu { ad, bastan, sona, govde, satir } => {
                let bastan = tam_sayi(degerlendir(bastan, ortam, islemler, cikti, *satir)?, *satir)?;
                let sona = tam_sayi(degerlendir(sona, ortam, islemler, cikti, *satir)?, *satir)?;
                for deger in bastan..=sona {
                    ortam.insert(ad.clone(), Deger::TamSayi(deger));
                    if let Akis::Don(d) = blok_calistir(govde, ortam, islemler, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::OlduguSurece { kosul, govde, satir } => {
                loop {
                    let devam = mantiksal(degerlendir(kosul, ortam, islemler, cikti, *satir)?, *satir)?;
                    if !devam {
                        break;
                    }
                    if let Akis::Don(d) = blok_calistir(govde, ortam, islemler, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::OlanaKadar { kosul, govde, satir } => {
                loop {
                    let bitti = mantiksal(degerlendir(kosul, ortam, islemler, cikti, *satir)?, *satir)?;
                    if bitti {
                        break;
                    }
                    if let Akis::Don(d) = blok_calistir(govde, ortam, islemler, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::Ise { kollar, degilse, satir } => {
                let mut islendi = false;
                for kol in kollar {
                    if mantiksal(degerlendir(&kol.kosul, ortam, islemler, cikti, *satir)?, *satir)? {
                        if let Akis::Don(d) = blok_calistir(&kol.govde, ortam, islemler, cikti)? {
                            return Ok(Akis::Don(d));
                        }
                        islendi = true;
                        break;
                    }
                }
                if !islendi {
                    if let Some(blok) = degilse {
                        if let Akis::Don(d) = blok_calistir(blok, ortam, islemler, cikti)? {
                            return Ok(Akis::Don(d));
                        }
                    }
                }
            }
            Cumle::Ekle { hedef, deger, satir } => {
                let ad = match hedef {
                    Ifade::Degisken { cozulmus: Some(ad), .. } => ad.clone(),
                    _ => return Err(ic_hata(*satir)),
                };
                let deger = degerlendir(deger, ortam, islemler, cikti, *satir)?;
                match ortam.get_mut(&ad) {
                    Some(Deger::Liste(ogeler)) => ogeler.push(deger),
                    _ => return Err(ic_hata(*satir)),
                }
            }
            Cumle::HerBiri { ad, kaynak, govde, satir } => {
                let kaynak = kaynak.as_ref().ok_or_else(|| ic_hata(*satir))?;
                let ogeler = match degerlendir(kaynak, ortam, islemler, cikti, *satir)? {
                    Deger::Liste(ogeler) => ogeler,
                    _ => return Err(ic_hata(*satir)),
                };
                for oge in ogeler {
                    ortam.insert(ad.clone(), oge);
                    if let Akis::Don(d) = blok_calistir(govde, ortam, islemler, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::Artir { ifade, miktar, satir } => {
                guncelle(ifade, miktar, ortam, islemler, cikti, *satir, 1)?;
            }
            Cumle::Azalt { ifade, miktar, satir } => {
                guncelle(ifade, miktar, ortam, islemler, cikti, *satir, -1)?;
            }
            Cumle::IslemTanimi(islem) => return Err(ic_hata(islem.satir)),
            Cumle::Dondur { deger, satir } => {
                let sonuc = degerlendir(deger, ortam, islemler, cikti, *satir)?;
                return Ok(Akis::Don(sonuc));
            }
            Cumle::BolVeAta { hedef, pay, payda, satir } => {
                let pay = tam_sayi(degerlendir(pay, ortam, islemler, cikti, *satir)?, *satir)?;
                let payda = tam_sayi(degerlendir(payda, ortam, islemler, cikti, *satir)?, *satir)?;
                if payda == 0 {
                    return Err(Tani::yeni(
                        "C003",
                        "Sıfıra bölme yapılamaz.".into(),
                        *satir,
                        1,
                        1,
                    ));
                }
                ortam.insert(hedef.clone(), Deger::TamSayi(pay / payda));
            }
            Cumle::CagriCumlesi { cagri, satir } => {
                if let Ifade::IslemCagrisi { islem_adi, argumanlar, .. } = cagri {
                    let mut degerler = Vec::new();
                    for arg in argumanlar {
                        degerler.push(degerlendir(arg, ortam, islemler, cikti, *satir)?);
                    }
                    islem_cagir(islem_adi, degerler, islemler, cikti, *satir)?;
                } else {
                    return Err(ic_hata(*satir));
                }
            }
        }
    }
    Ok(Akis::Devam)
}

/// İşlemi taze bir ortamda çalıştırır; "döndür" değeri varsa onu verir.
fn islem_cagir(
    ad: &str,
    argumanlar: Vec<Deger>,
    islemler: &HashMap<String, Islem>,
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Option<Deger>, Tani> {
    let islem = islemler.get(ad).ok_or_else(|| ic_hata(satir))?;
    let mut yerel: HashMap<String, Deger> = HashMap::new();
    for (param, deger) in islem.parametreler.iter().zip(argumanlar) {
        yerel.insert(param.clone(), deger);
    }
    match blok_calistir(&islem.govde, &mut yerel, islemler, io)? {
        Akis::Don(deger) => Ok(Some(deger)),
        Akis::Devam => Ok(None),
    }
}

fn guncelle(
    hedef: &Ifade,
    miktar: &Ifade,
    ortam: &mut HashMap<String, Deger>,
    islemler: &HashMap<String, Islem>,
    io: &mut dyn GirdiCikti,
    satir: usize,
    yon: i64,
) -> Result<(), Tani> {
    let ad = match hedef {
        Ifade::Degisken { cozulmus: Some(ad), .. } => ad.clone(),
        _ => return Err(ic_hata(satir)),
    };
    let miktar = tam_sayi(degerlendir(miktar, ortam, islemler, io, satir)?, satir)?;
    let eski = match ortam.get(&ad) {
        Some(Deger::TamSayi(s)) => *s,
        _ => return Err(ic_hata(satir)),
    };
    let yeni = eski.checked_add(yon * miktar).ok_or_else(|| {
        Tani::yeni(
            "C002",
            format!("\"{}\" değeri taştı: TamSayı sınırı aşıldı.", ad),
            satir,
            1,
            1,
        )
    })?;
    ortam.insert(ad, Deger::TamSayi(yeni));
    Ok(())
}

fn degerlendir(
    ifade: &Ifade,
    ortam: &HashMap<String, Deger>,
    islemler: &HashMap<String, Islem>,
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    match ifade {
        Ifade::MetinSabiti(m) => Ok(Deger::Metin(m.clone())),
        Ifade::SayiSabiti(s) => Ok(Deger::TamSayi(*s)),
        Ifade::MantiksalSabiti(b) => Ok(Deger::Mantiksal(*b)),
        Ifade::BosListe => Ok(Deger::Liste(Vec::new())),
        Ifade::ListeSabiti(ogeler) => {
            let mut degerler = Vec::new();
            for oge in ogeler {
                degerler.push(degerlendir(oge, ortam, islemler, io, satir)?);
            }
            Ok(Deger::Liste(degerler))
        }
        Ifade::Ozellik { nesne, ozellik } => {
            let ogeler = match degerlendir(nesne, ortam, islemler, io, satir)? {
                Deger::Liste(ogeler) => ogeler,
                _ => return Err(ic_hata(satir)),
            };
            match ozellik {
                Ozellik::Adet => Ok(Deger::TamSayi(ogeler.len() as i64)),
                Ozellik::Ilk | Ozellik::Son => {
                    let oge = if *ozellik == Ozellik::Ilk {
                        ogeler.first()
                    } else {
                        ogeler.last()
                    };
                    oge.cloned().ok_or_else(|| {
                        Tani::yeni(
                            "C007",
                            "Liste boş: ilki/sonu alınamaz.".into(),
                            satir,
                            1,
                            1,
                        )
                        .onerili("Önce \"listenin adedi\" ile boş olup olmadığını kontrol et.".into())
                    })
                }
            }
        }
        Ifade::Rastgele { alt, ust } => {
            let alt = tam_sayi(degerlendir(alt, ortam, islemler, io, satir)?, satir)?;
            let ust = tam_sayi(degerlendir(ust, ortam, islemler, io, satir)?, satir)?;
            if alt > ust {
                return Err(Tani::yeni(
                    "C006",
                    format!("Rastgele aralığı ters: {} ile {} arasında sayı üretilemez.", alt, ust),
                    satir,
                    1,
                    1,
                ));
            }
            let deger = io.rastgele(alt, ust).clamp(alt, ust);
            Ok(Deger::TamSayi(deger))
        }
        Ifade::Degisken { cozulmus, ham, .. } => {
            let ad = cozulmus.as_ref().ok_or_else(|| ic_hata(satir))?;
            ortam
                .get(ad)
                .cloned()
                .ok_or_else(|| {
                    Tani::yeni("C001", format!("\"{}\" için değer bulunamadı.", ham), satir, 1, 1)
                })
        }
        Ifade::Birlestir(parcalar) => {
            let mut metin = String::new();
            for parca in parcalar {
                metin.push_str(&degerlendir(parca, ortam, islemler, io, satir)?.metne());
            }
            Ok(Deger::Metin(metin))
        }
        Ifade::Karsilastirma { sol, sag, islec } => {
            let sol = degerlendir(sol, ortam, islemler, io, satir)?;
            let sag = degerlendir(sag, ortam, islemler, io, satir)?;
            let sonuc = match islec {
                Islec::Esit => sol == sag,
                _ => {
                    let sol = tam_sayi(sol, satir)?;
                    let sag = tam_sayi(sag, satir)?;
                    match islec {
                        Islec::Buyuk => sol > sag,
                        Islec::Kucuk => sol < sag,
                        Islec::BuyukEsit => sol >= sag,
                        Islec::KucukEsit => sol <= sag,
                        Islec::Esit => unreachable!(),
                    }
                }
            };
            Ok(Deger::Mantiksal(sonuc))
        }
        Ifade::Cift(ic) => {
            let s = tam_sayi(degerlendir(ic, ortam, islemler, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(s % 2 == 0))
        }
        Ifade::Tek(ic) => {
            let s = tam_sayi(degerlendir(ic, ortam, islemler, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(s % 2 != 0))
        }
        Ifade::Aritmetik { islec, sol, sag } => {
            let sol = tam_sayi(degerlendir(sol, ortam, islemler, io, satir)?, satir)?;
            let sag = tam_sayi(degerlendir(sag, ortam, islemler, io, satir)?, satir)?;
            let sonuc = match islec {
                AritmetikIslec::Topla => sol.checked_add(sag),
                AritmetikIslec::Cikar => sol.checked_sub(sag),
                AritmetikIslec::Carp => sol.checked_mul(sag),
                AritmetikIslec::Bol => {
                    if sag == 0 {
                        return Err(Tani::yeni(
                            "C003",
                            "Sıfıra bölme yapılamaz.".into(),
                            satir,
                            1,
                            1,
                        )
                        .onerili("Bölmeden önce bölenin sıfır olup olmadığını kontrol et.".into()));
                    }
                    sol.checked_div(sag)
                }
            };
            let sonuc = sonuc.ok_or_else(|| {
                Tani::yeni("C002", "İşlem sonucu TamSayı sınırını aştı.".into(), satir, 1, 1)
            })?;
            Ok(Deger::TamSayi(sonuc))
        }
        Ifade::IslemCagrisi { islem_adi, argumanlar, .. } => {
            let mut degerler = Vec::new();
            for arg in argumanlar {
                degerler.push(degerlendir(arg, ortam, islemler, io, satir)?);
            }
            islem_cagir(islem_adi, degerler, islemler, io, satir)?
                .ok_or_else(|| ic_hata(satir))
        }
        Ifade::Sayisi(ic) => {
            let metin = match degerlendir(ic, ortam, islemler, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let kirpilmis = metin.trim();
            kirpilmis.parse::<i64>().map(Deger::TamSayi).map_err(|_| {
                Tani::yeni(
                    "C004",
                    format!("\"{}\" sayıya çevrilemedi.", kirpilmis),
                    satir,
                    1,
                    1,
                )
                .onerili("Sayı yalnız rakamlardan oluşmalı. Örnek: 42".into())
            })
        }
    }
}

fn tam_sayi(deger: Deger, satir: usize) -> Result<i64, Tani> {
    match deger {
        Deger::TamSayi(s) => Ok(s),
        _ => Err(ic_hata(satir)),
    }
}

fn mantiksal(deger: Deger, satir: usize) -> Result<bool, Tani> {
    match deger {
        Deger::Mantiksal(b) => Ok(b),
        _ => Err(ic_hata(satir)),
    }
}

/// Tür denetiminden geçmiş programda görünmemesi gereken durum.
fn ic_hata(satir: usize) -> Tani {
    Tani::yeni(
        "C000",
        "İç tutarlılık hatası: tür denetiminden geçen program çalışırken bozuldu. \
         Bu bir derleyici hatasıdır, lütfen bildir."
            .into(),
        satir,
        1,
        1,
    )
}
