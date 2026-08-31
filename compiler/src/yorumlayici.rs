//! Ağaç-yürüyen yorumlayıcı (ADR-003: ilk execution modeli).
//!
//! Tür denetiminden geçmiş programı çalıştırır. Çıktı satır listesi olarak
//! döner; CLI bunu ekrana basar, testler doğrudan karşılaştırır.

use crate::agac::{AritmetikIslec, Cumle, Ifade, Islec};
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
}

impl Deger {
    fn metne(&self) -> String {
        match self {
            Deger::TamSayi(s) => s.to_string(),
            Deger::Metin(m) => m.clone(),
            Deger::Mantiksal(b) => if *b { "doğru" } else { "yanlış" }.to_string(),
        }
    }
}

pub fn calistir(program: &[Cumle]) -> Result<Vec<String>, Tani> {
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(program, &mut io)?;
    Ok(io.cikti)
}

pub fn calistir_io(program: &[Cumle], io: &mut dyn GirdiCikti) -> Result<(), Tani> {
    let mut ortam: HashMap<String, Deger> = HashMap::new();
    blok_calistir(program, &mut ortam, io)
}

fn blok_calistir(
    cumleler: &[Cumle],
    ortam: &mut HashMap<String, Deger>,
    cikti: &mut dyn GirdiCikti,
) -> Result<(), Tani> {
    for cumle in cumleler {
        match cumle {
            Cumle::Yaz { deger, satir } => {
                let sonuc = degerlendir(deger, ortam, cikti, *satir)?;
                cikti.yazdir(sonuc.metne());
            }
            Cumle::Sor { istem, satir } => {
                let istem = degerlendir(istem, ortam, cikti, *satir)?.metne();
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
                let sonuc = degerlendir(deger, ortam, cikti, *satir)?;
                ortam.insert(ad.clone(), sonuc);
            }
            Cumle::KezTekrarla { adet, govde, satir } => {
                let adet = tam_sayi(degerlendir(adet, ortam, cikti, *satir)?, *satir)?;
                for _ in 0..adet.max(0) {
                    blok_calistir(govde, ortam, cikti)?;
                }
            }
            Cumle::AralikDongusu { ad, bastan, sona, govde, satir } => {
                let bastan = tam_sayi(degerlendir(bastan, ortam, cikti, *satir)?, *satir)?;
                let sona = tam_sayi(degerlendir(sona, ortam, cikti, *satir)?, *satir)?;
                for deger in bastan..=sona {
                    ortam.insert(ad.clone(), Deger::TamSayi(deger));
                    blok_calistir(govde, ortam, cikti)?;
                }
            }
            Cumle::OlduguSurece { kosul, govde, satir } => {
                loop {
                    let devam = mantiksal(degerlendir(kosul, ortam, cikti, *satir)?, *satir)?;
                    if !devam {
                        break;
                    }
                    blok_calistir(govde, ortam, cikti)?;
                }
            }
            Cumle::OlanaKadar { kosul, govde, satir } => {
                loop {
                    let bitti = mantiksal(degerlendir(kosul, ortam, cikti, *satir)?, *satir)?;
                    if bitti {
                        break;
                    }
                    blok_calistir(govde, ortam, cikti)?;
                }
            }
            Cumle::Ise { kollar, degilse, satir } => {
                let mut islendi = false;
                for kol in kollar {
                    if mantiksal(degerlendir(&kol.kosul, ortam, cikti, *satir)?, *satir)? {
                        blok_calistir(&kol.govde, ortam, cikti)?;
                        islendi = true;
                        break;
                    }
                }
                if !islendi {
                    if let Some(blok) = degilse {
                        blok_calistir(blok, ortam, cikti)?;
                    }
                }
            }
            Cumle::Artir { ifade, miktar, satir } => {
                guncelle(ifade, miktar, ortam, cikti, *satir, 1)?;
            }
            Cumle::Azalt { ifade, miktar, satir } => {
                guncelle(ifade, miktar, ortam, cikti, *satir, -1)?;
            }
        }
    }
    Ok(())
}

fn guncelle(
    hedef: &Ifade,
    miktar: &Ifade,
    ortam: &mut HashMap<String, Deger>,
    io: &mut dyn GirdiCikti,
    satir: usize,
    yon: i64,
) -> Result<(), Tani> {
    let ad = match hedef {
        Ifade::Degisken { cozulmus: Some(ad), .. } => ad.clone(),
        _ => return Err(ic_hata(satir)),
    };
    let miktar = tam_sayi(degerlendir(miktar, ortam, io, satir)?, satir)?;
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
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    match ifade {
        Ifade::MetinSabiti(m) => Ok(Deger::Metin(m.clone())),
        Ifade::SayiSabiti(s) => Ok(Deger::TamSayi(*s)),
        Ifade::MantiksalSabiti(b) => Ok(Deger::Mantiksal(*b)),
        Ifade::Rastgele { alt, ust } => {
            let alt = tam_sayi(degerlendir(alt, ortam, io, satir)?, satir)?;
            let ust = tam_sayi(degerlendir(ust, ortam, io, satir)?, satir)?;
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
                metin.push_str(&degerlendir(parca, ortam, io, satir)?.metne());
            }
            Ok(Deger::Metin(metin))
        }
        Ifade::Karsilastirma { sol, sag, islec } => {
            let sol = degerlendir(sol, ortam, io, satir)?;
            let sag = degerlendir(sag, ortam, io, satir)?;
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
            let s = tam_sayi(degerlendir(ic, ortam, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(s % 2 == 0))
        }
        Ifade::Tek(ic) => {
            let s = tam_sayi(degerlendir(ic, ortam, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(s % 2 != 0))
        }
        Ifade::Aritmetik { islec, sol, sag } => {
            let sol = tam_sayi(degerlendir(sol, ortam, io, satir)?, satir)?;
            let sag = tam_sayi(degerlendir(sag, ortam, io, satir)?, satir)?;
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
        Ifade::Sayisi(ic) => {
            let metin = match degerlendir(ic, ortam, io, satir)? {
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
