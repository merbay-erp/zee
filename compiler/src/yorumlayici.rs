//! Ağaç-yürüyen yorumlayıcı (ADR-003: ilk execution modeli).
//!
//! Tür denetiminden geçmiş programı çalıştırır. Çıktı satır listesi olarak
//! döner; CLI bunu ekrana basar, testler doğrudan karşılaştırır.

use crate::agac::{AritmetikIslec, Cumle, Ifade, Islec, Ozellik, Program};
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
    /// Dosya içeriğini okur; hata durumunda Türkçe hata metni döner.
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String>;
    /// Bir satırı dosyaya yazar (ekleme=false: baştan yaz; true: sona ekle).
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String>;
    /// Şimdiki zaman: (yıl, ay, gün, saat, dakika). v0'da UTC.
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32);
    /// Komut satırı argümanları (programa aktarılanlar).
    fn argumanlar(&mut self) -> Vec<String>;
}

/// Çıktıyı toplayan, girdiyi ve "rastgele" sayıları hazır kuyruktan veren IO
/// (testler ve lib arayüzü — determinizm burada da korunur).
pub struct ToplayanIo {
    pub girdiler: VecDeque<String>,
    pub rastgele_degerler: VecDeque<i64>,
    /// Sahte dosya sistemi: yol → içerik (testlerde determinizm).
    pub dosyalar: HashMap<String, String>,
    /// Sabit "şimdi" — testlerde determinizm.
    pub zaman: (i64, u32, u32, u32, u32),
    pub argumanlar: Vec<String>,
    pub cikti: Vec<String>,
}

impl ToplayanIo {
    pub fn yeni(girdiler: Vec<String>) -> ToplayanIo {
        ToplayanIo {
            girdiler: girdiler.into(),
            rastgele_degerler: VecDeque::new(),
            dosyalar: HashMap::new(),
            zaman: (2026, 8, 31, 14, 30),
            argumanlar: Vec::new(),
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
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        self.dosyalar
            .get(yol)
            .cloned()
            .ok_or_else(|| format!("\"{}\" dosyası bulunamadı", yol))
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        let girdi = self.dosyalar.entry(yol.to_string()).or_default();
        if !ekleme {
            girdi.clear();
        }
        girdi.push_str(satir);
        girdi.push('\n');
        Ok(())
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.zaman
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.argumanlar.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Deger {
    TamSayi(i64),
    /// Onluk tam değer: govde/10^olcek; hep normalize saklanır (olcek >= 1,
    /// sondaki sıfırlar atılmış). Böylece 1,50 ve 1,5 aynı değerdir.
    Ondalik { govde: i64, olcek: u32 },
    Metin(String),
    Mantiksal(bool),
    Liste(Vec<Deger>),
    /// Ekleme sırası korunur (deterministik gezinme).
    Sozluk(Vec<(String, Deger)>),
    /// Seçenek'in boş hali; dolu hali değerin kendisidir.
    Yok,
    /// Sonuç: başarılıysa değer, değilse hata metni (Metin) taşır.
    Sonuc { basarili: bool, icerik: Box<Deger> },
    /// Yapı örneği: yalın alan adı → değer (tanım sırasıyla).
    Yapi(Vec<(String, Deger)>),
    Tarih { yil: i64, ay: u32, gun: u32 },
    Saat { saat: u32, dakika: u32 },
    /// Milisaniye cinsinden süre.
    Sure { milisaniye: i64 },
}

const AY_ADLARI: [&str; 12] = [
    "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
    "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık",
];

impl Deger {
    fn metne(&self) -> String {
        match self {
            Deger::TamSayi(s) => s.to_string(),
            Deger::Ondalik { govde, olcek } => {
                let isaret = if *govde < 0 { "-" } else { "" };
                let mutlak = govde.unsigned_abs();
                let carpan = 10u64.pow(*olcek);
                format!(
                    "{}{},{:0genislik$}",
                    isaret,
                    mutlak / carpan,
                    mutlak % carpan,
                    genislik = *olcek as usize
                )
            }
            Deger::Metin(m) => m.clone(),
            Deger::Mantiksal(b) => if *b { "doğru" } else { "yanlış" }.to_string(),
            Deger::Liste(ogeler) => ogeler
                .iter()
                .map(|o| o.metne())
                .collect::<Vec<_>>()
                .join(", "),
            Deger::Sozluk(girdiler) => girdiler
                .iter()
                .map(|(anahtar, deger)| format!("{}: {}", anahtar, deger.metne()))
                .collect::<Vec<_>>()
                .join(", "),
            Deger::Yok => "yok".to_string(),
            Deger::Yapi(alanlar) => alanlar
                .iter()
                .map(|(alan, deger)| format!("{}: {}", alan, deger.metne()))
                .collect::<Vec<_>>()
                .join(", "),
            Deger::Sonuc { basarili, icerik } => {
                if *basarili {
                    icerik.metne()
                } else {
                    format!("hata: {}", icerik.metne())
                }
            }
            Deger::Tarih { yil, ay, gun } => {
                format!("{} {} {}", gun, AY_ADLARI[(*ay as usize).saturating_sub(1) % 12], yil)
            }
            Deger::Saat { saat, dakika } => format!("{:02}:{:02}", saat, dakika),
            Deger::Sure { milisaniye } => {
                let ms = *milisaniye;
                if ms % 3_600_000 == 0 {
                    format!("{} saat", ms / 3_600_000)
                } else if ms % 60_000 == 0 {
                    format!("{} dakika", ms / 60_000)
                } else if ms % 1000 == 0 {
                    format!("{} saniye", ms / 1000)
                } else {
                    // Küsuratlı: saniye cinsinden ondalık basım (1500 → "1,5 saniye").
                    let mut govde = ms;
                    let mut olcek = 3u32;
                    while olcek > 1 && govde % 10 == 0 {
                        govde /= 10;
                        olcek -= 1;
                    }
                    format!("{} saniye", Deger::Ondalik { govde, olcek }.metne())
                }
            }
        }
    }
}

/// CLI'nin sistem saatini çevirmesi için dışa açık sarmalayıcı.
pub fn gunlerden_tarih_utc(gunler: i64) -> (i64, u32, u32) {
    gunlerden_tarih(gunler)
}

/// Gregoryen tarih ↔ gün sayısı (Howard Hinnant'ın algoritmaları; 1970-01-01 = 0).
fn gunlerden_tarih(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let devir = if z >= 0 { z } else { z - 146096 } / 146097;
    let devir_gunu = (z - devir * 146097) as u64;
    let yil_gunu = (devir_gunu - devir_gunu / 1460 + devir_gunu / 36524 - devir_gunu / 146096) / 365;
    let yil = yil_gunu as i64 + devir * 400;
    let yilin_gunu = devir_gunu - (365 * yil_gunu + yil_gunu / 4 - yil_gunu / 100);
    let ay_kaba = (5 * yilin_gunu + 2) / 153;
    let gun = (yilin_gunu - (153 * ay_kaba + 2) / 5 + 1) as u32;
    let ay = if ay_kaba < 10 { ay_kaba + 3 } else { ay_kaba - 9 } as u32;
    (if ay <= 2 { yil + 1 } else { yil }, ay, gun)
}

fn tarihten_gunler(yil: i64, ay: u32, gun: u32) -> i64 {
    let yil = if ay <= 2 { yil - 1 } else { yil };
    let devir = if yil >= 0 { yil } else { yil - 399 } / 400;
    let devir_yili = (yil - devir * 400) as u64;
    let yilin_gunu =
        (153 * (if ay > 2 { ay - 3 } else { ay + 9 }) as u64 + 2) / 5 + gun as u64 - 1;
    let devir_gunu = devir_yili * 365 + devir_yili / 4 - devir_yili / 100 + yilin_gunu;
    devir * 146097 + devir_gunu as i64 - 719468
}

/// Ondalık kurucu: normalize eder (sondaki sıfırlar atılır, olcek >= 1) ve
/// i64 sınırını denetler (C002).
fn ondalik_yap(mut govde: i128, mut olcek: u32, satir: usize) -> Result<Deger, Tani> {
    while olcek > 1 && govde % 10 == 0 {
        govde /= 10;
        olcek -= 1;
    }
    if olcek == 0 {
        govde = govde.checked_mul(10).ok_or_else(|| tasma(satir))?;
        olcek = 1;
    }
    let govde = i64::try_from(govde).map_err(|_| tasma(satir))?;
    Ok(Deger::Ondalik { govde, olcek })
}

fn tasma(satir: usize) -> Tani {
    Tani::yeni("C002", "İşlem sonucu sayı sınırını aştı.".into(), satir, 1, 1)
}

/// Sayısal değeri (govde, olcek) çiftine açar; TamSayı olcek 0 ile gelir.
fn sayisal_ac(deger: &Deger) -> Option<(i128, u32)> {
    match deger {
        Deger::TamSayi(v) => Some((*v as i128, 0)),
        Deger::Ondalik { govde, olcek } => Some((*govde as i128, *olcek)),
        _ => None,
    }
}

/// İki sayıyı ortak ölçeğe hizalar.
fn hizala(a: (i128, u32), b: (i128, u32)) -> (i128, i128, u32) {
    let ortak = a.1.max(b.1);
    let ga = a.0 * 10i128.pow(ortak - a.1);
    let gb = b.0 * 10i128.pow(ortak - b.1);
    (ga, gb, ortak)
}

/// Yarımlar sıfırdan uzağa yuvarlanarak bölme (okul kuralı, RFC-0013 §2).
fn yuvarla_bol(pay: i128, payda: i128) -> i128 {
    let isaret = if (pay < 0) != (payda < 0) { -1 } else { 1 };
    let p = pay.abs();
    let q = payda.abs();
    isaret * ((2 * p + q) / (2 * q))
}

/// Türkçe kurallarla büyük harfe çevirme: i→İ, ı→I (A07 anti-örneğindeki tuzak).
fn turkce_buyuk(metin: &str) -> String {
    metin
        .chars()
        .flat_map(|k| match k {
            'i' => vec!['İ'],
            'ı' => vec!['I'],
            _ => k.to_uppercase().collect(),
        })
        .collect()
}

/// Türkçe kurallarla küçük harfe çevirme: İ→i, I→ı.
fn turkce_kucuk(metin: &str) -> String {
    metin
        .chars()
        .flat_map(|k| match k {
            'İ' => vec!['i'],
            'I' => vec!['ı'],
            _ => k.to_lowercase().collect(),
        })
        .collect()
}

pub fn calistir(program: &Program) -> Result<Vec<String>, Tani> {
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(program, &mut io)?;
    Ok(io.cikti)
}

pub fn calistir_io(program: &Program, io: &mut dyn GirdiCikti) -> Result<(), Tani> {
    let mut ortam: HashMap<String, Deger> = HashMap::new();
    match blok_calistir(&program.cumleler, &mut ortam, program, io) {
        // "programı bitir" olağan bir sonlanmadır (Ç000 iç nöbetçisi).
        Err(tani) if tani.kod == "Ç000" => Ok(()),
        sonuc => sonuc.map(|_| ()),
    }
}

/// Tek bir testi taze ortamda koşar; ilk doğrulama/çalışma hatasında durur.
pub fn test_calistir(
    program: &Program,
    test: &crate::agac::Test,
    io: &mut dyn GirdiCikti,
) -> Result<(), Tani> {
    let mut ortam: HashMap<String, Deger> = HashMap::new();
    match blok_calistir(&test.govde, &mut ortam, program, io) {
        Err(tani) if tani.kod == "Ç000" => Ok(()),
        sonuc => sonuc.map(|_| ()),
    }
}

/// Blok çalıştırmanın sonucu: normal akış mı, "döndür" ile erken çıkış mı.
pub enum Akis {
    Devam,
    Don(Deger),
}

fn blok_calistir(
    cumleler: &[Cumle],
    ortam: &mut HashMap<String, Deger>,
    program: &Program,
    cikti: &mut dyn GirdiCikti,
) -> Result<Akis, Tani> {
    for cumle in cumleler {
        match cumle {
            Cumle::Yaz { deger, satir } => {
                let sonuc = degerlendir(deger, ortam, program, cikti, *satir)?;
                cikti.yazdir(sonuc.metne());
            }
            Cumle::Sor { istem, satir } => {
                let istem = degerlendir(istem, ortam, program, cikti, *satir)?.metne();
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
                let sonuc = degerlendir(deger, ortam, program, cikti, *satir)?;
                ortam.insert(ad.clone(), sonuc);
            }
            Cumle::KezTekrarla { adet, govde, satir } => {
                let adet = tam_sayi(degerlendir(adet, ortam, program, cikti, *satir)?, *satir)?;
                for _ in 0..adet.max(0) {
                    if let Akis::Don(d) = blok_calistir(govde, ortam, program, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::AralikDongusu { ad, bastan, sona, govde, satir } => {
                let bastan = tam_sayi(degerlendir(bastan, ortam, program, cikti, *satir)?, *satir)?;
                let sona = tam_sayi(degerlendir(sona, ortam, program, cikti, *satir)?, *satir)?;
                for deger in bastan..=sona {
                    ortam.insert(ad.clone(), Deger::TamSayi(deger));
                    if let Akis::Don(d) = blok_calistir(govde, ortam, program, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::OlduguSurece { kosul, govde, satir } => {
                loop {
                    let devam = mantiksal(degerlendir(kosul, ortam, program, cikti, *satir)?, *satir)?;
                    if !devam {
                        break;
                    }
                    if let Akis::Don(d) = blok_calistir(govde, ortam, program, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::OlanaKadar { kosul, govde, satir } => {
                loop {
                    let bitti = mantiksal(degerlendir(kosul, ortam, program, cikti, *satir)?, *satir)?;
                    if bitti {
                        break;
                    }
                    if let Akis::Don(d) = blok_calistir(govde, ortam, program, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::Ise { kollar, degilse, satir } => {
                let mut islendi = false;
                for kol in kollar {
                    if mantiksal(degerlendir(&kol.kosul, ortam, program, cikti, *satir)?, *satir)? {
                        if let Akis::Don(d) = blok_calistir(&kol.govde, ortam, program, cikti)? {
                            return Ok(Akis::Don(d));
                        }
                        islendi = true;
                        break;
                    }
                }
                if !islendi {
                    if let Some(blok) = degilse {
                        if let Akis::Don(d) = blok_calistir(blok, ortam, program, cikti)? {
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
                let deger = degerlendir(deger, ortam, program, cikti, *satir)?;
                match ortam.get_mut(&ad) {
                    Some(Deger::Liste(ogeler)) => ogeler.push(deger),
                    _ => return Err(ic_hata(*satir)),
                }
            }
            Cumle::HerBiri { ad, kaynak, govde, satir } => {
                let kaynak = kaynak.as_ref().ok_or_else(|| ic_hata(*satir))?;
                let ogeler = match degerlendir(kaynak, ortam, program, cikti, *satir)? {
                    Deger::Liste(ogeler) => ogeler,
                    // Sözlük üzerinde gezinme anahtarları verir (ekleme sırasıyla).
                    Deger::Sozluk(girdiler) => girdiler
                        .into_iter()
                        .map(|(anahtar, _)| Deger::Metin(anahtar))
                        .collect(),
                    _ => return Err(ic_hata(*satir)),
                };
                for oge in ogeler {
                    ortam.insert(ad.clone(), oge);
                    if let Akis::Don(d) = blok_calistir(govde, ortam, program, cikti)? {
                        return Ok(Akis::Don(d));
                    }
                }
            }
            Cumle::Artir { ifade, miktar, satir } => {
                guncelle(ifade, miktar, ortam, program, cikti, *satir, 1)?;
            }
            Cumle::Azalt { ifade, miktar, satir } => {
                guncelle(ifade, miktar, ortam, program, cikti, *satir, -1)?;
            }
            Cumle::Gore { konu, kollar, degilse, satir } => {
                let konu = degerlendir(konu, ortam, program, cikti, *satir)?;
                let mut eslesti = false;
                for (deger, govde) in kollar {
                    let deger = degerlendir(deger, ortam, program, cikti, *satir)?;
                    if deger == konu {
                        if let Akis::Don(d) = blok_calistir(govde, ortam, program, cikti)? {
                            return Ok(Akis::Don(d));
                        }
                        eslesti = true;
                        break;
                    }
                }
                if !eslesti {
                    if let Some(blok) = degilse {
                        if let Akis::Don(d) = blok_calistir(blok, ortam, program, cikti)? {
                            return Ok(Akis::Don(d));
                        }
                    }
                }
            }
            Cumle::ProgramiBitir { satir } => {
                return Err(Tani::yeni("Ç000", "programı bitir".into(), *satir, 1, 1));
            }
            Cumle::IslemTanimi(islem) => return Err(ic_hata(islem.satir)),
            Cumle::YapiTanimi(yapi) => return Err(ic_hata(yapi.satir)),
            Cumle::TestBlogu(test) => return Err(ic_hata(test.satir)),
            Cumle::Kullan { satir, .. } => return Err(ic_hata(*satir)),
            Cumle::Olmali { kosul, satir } => {
                // Karşılaştırmalarda iki tarafın değeri tanıya yazılır —
                // "beklenen/bulunan" göstermek öğretici hata ilkesinin gereği.
                let (tuttu, detay) = match kosul {
                    Ifade::Karsilastirma { sol, sag, .. } => {
                        let sol_deger = degerlendir(sol, ortam, program, cikti, *satir)?;
                        let sag_deger = degerlendir(sag, ortam, program, cikti, *satir)?;
                        let sonuc =
                            mantiksal(degerlendir(kosul, ortam, program, cikti, *satir)?, *satir)?;
                        (
                            sonuc,
                            format!(
                                " Beklenen: {} — bulunan: {}.",
                                sag_deger.metne(),
                                sol_deger.metne()
                            ),
                        )
                    }
                    _ => (
                        mantiksal(degerlendir(kosul, ortam, program, cikti, *satir)?, *satir)?,
                        String::new(),
                    ),
                };
                if !tuttu {
                    return Err(Tani::yeni(
                        "D001",
                        format!("Doğrulama tutmadı.{}", detay),
                        *satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::AlanAta { nesne, alan, deger, satir } => {
                let ad = match nesne {
                    Ifade::Degisken { cozulmus: Some(ad), .. } => ad.clone(),
                    _ => return Err(ic_hata(*satir)),
                };
                let deger = degerlendir(deger, ortam, program, cikti, *satir)?;
                match ortam.get_mut(&ad) {
                    Some(Deger::Yapi(alanlar)) => {
                        match alanlar.iter_mut().find(|(a, _)| a == alan) {
                            Some((_, eski)) => *eski = deger,
                            None => return Err(ic_hata(*satir)),
                        }
                    }
                    _ => return Err(ic_hata(*satir)),
                }
            }
            Cumle::Dondur { deger, sonuca_sarmala, satir } => {
                let sonuc = degerlendir(deger, ortam, program, cikti, *satir)?;
                let sonuc = if *sonuca_sarmala {
                    Deger::Sonuc { basarili: true, icerik: Box::new(sonuc) }
                } else {
                    sonuc
                };
                return Ok(Akis::Don(sonuc));
            }
            Cumle::HataDondur { mesaj, satir } => {
                let mesaj = degerlendir(mesaj, ortam, program, cikti, *satir)?;
                return Ok(Akis::Don(Deger::Sonuc {
                    basarili: false,
                    icerik: Box::new(mesaj),
                }));
            }
            Cumle::BolVeAta { hedef, pay, payda, satir } => {
                let pay = degerlendir(pay, ortam, program, cikti, *satir)?;
                let payda = degerlendir(payda, ortam, program, cikti, *satir)?;
                let sonuc = sayisal_islem(&AritmetikIslec::Bol, &pay, &payda, *satir)?;
                ortam.insert(hedef.clone(), sonuc);
            }
            Cumle::CagriCumlesi { cagri, satir } => {
                if let Ifade::IslemCagrisi { islem_adi, argumanlar, .. } = cagri {
                    let mut degerler = Vec::new();
                    for arg in argumanlar {
                        degerler.push(degerlendir(arg, ortam, program, cikti, *satir)?);
                    }
                    islem_cagir(islem_adi, degerler, program, cikti, *satir)?;
                } else {
                    return Err(ic_hata(*satir));
                }
            }
            Cumle::DosyayaYaz { yol, icerik, ekleme, satir } => {
                let yol = match degerlendir(yol, ortam, program, cikti, *satir)? {
                    Deger::Metin(m) => m,
                    _ => return Err(ic_hata(*satir)),
                };
                let icerik = degerlendir(icerik, ortam, program, cikti, *satir)?.metne();
                cikti.dosya_yaz(&yol, &icerik, *ekleme).map_err(|hata| {
                    Tani::yeni("C013", format!("Dosyaya yazılamadı: {}.", hata), *satir, 1, 1)
                })?;
            }
            Cumle::SozlukAta { sozluk, anahtar, deger, satir } => {
                let ad = match sozluk {
                    Ifade::Degisken { cozulmus: Some(ad), .. } => ad.clone(),
                    _ => return Err(ic_hata(*satir)),
                };
                let anahtar = match degerlendir(anahtar, ortam, program, cikti, *satir)? {
                    Deger::Metin(m) => m,
                    _ => return Err(ic_hata(*satir)),
                };
                let deger = degerlendir(deger, ortam, program, cikti, *satir)?;
                match ortam.get_mut(&ad) {
                    Some(Deger::Sozluk(girdiler)) => {
                        match girdiler.iter_mut().find(|(a, _)| *a == anahtar) {
                            Some((_, eski)) => *eski = deger,
                            None => girdiler.push((anahtar, deger)),
                        }
                    }
                    _ => return Err(ic_hata(*satir)),
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
    program: &Program,
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Option<Deger>, Tani> {
    let islem = program.islemler.get(ad).ok_or_else(|| ic_hata(satir))?;
    let mut yerel: HashMap<String, Deger> = HashMap::new();
    for (param, deger) in islem.parametreler.iter().zip(argumanlar) {
        yerel.insert(param.clone(), deger);
    }
    match blok_calistir(&islem.govde, &mut yerel, program, io)? {
        Akis::Don(deger) => Ok(Some(deger)),
        Akis::Devam => Ok(None),
    }
}

/// Genitif aritmetiğin sayısal çekirdeği: iki TamSayı → TamSayı (tam bölme);
/// Ondalık karışımı → Ondalık (bölme 9 haneye, yarımlar sıfırdan uzağa).
fn sayisal_islem(
    islec: &AritmetikIslec,
    sol: &Deger,
    sag: &Deger,
    satir: usize,
) -> Result<Deger, Tani> {
    // Süre + Süre (checker yalnız topla/çıkar bırakır).
    if let (Deger::Sure { milisaniye: a }, Deger::Sure { milisaniye: b }) = (sol, sag) {
        let sonuc = match islec {
            AritmetikIslec::Topla => a.checked_add(*b),
            AritmetikIslec::Cikar => a.checked_sub(*b),
            _ => return Err(ic_hata(satir)),
        }
        .ok_or_else(|| tasma(satir))?;
        return Ok(Deger::Sure { milisaniye: sonuc });
    }

    let her_iki_tam = matches!((sol, sag), (Deger::TamSayi(_), Deger::TamSayi(_)));
    let a = sayisal_ac(sol).ok_or_else(|| ic_hata(satir))?;
    let b = sayisal_ac(sag).ok_or_else(|| ic_hata(satir))?;

    if *islec == AritmetikIslec::Bol && b.0 == 0 {
        return Err(Tani::yeni("C003", "Sıfıra bölme yapılamaz.".into(), satir, 1, 1)
            .onerili("Bölmeden önce bölenin sıfır olup olmadığını kontrol et.".into()));
    }

    if her_iki_tam {
        let sonuc = match islec {
            AritmetikIslec::Topla => a.0.checked_add(b.0),
            AritmetikIslec::Cikar => a.0.checked_sub(b.0),
            AritmetikIslec::Carp => a.0.checked_mul(b.0),
            AritmetikIslec::Bol => a.0.checked_div(b.0),
        }
        .ok_or_else(|| tasma(satir))?;
        let sonuc = i64::try_from(sonuc).map_err(|_| tasma(satir))?;
        return Ok(Deger::TamSayi(sonuc));
    }

    match islec {
        AritmetikIslec::Topla | AritmetikIslec::Cikar => {
            let (ga, gb, ortak) = hizala(a, b);
            let sonuc = if *islec == AritmetikIslec::Topla { ga + gb } else { ga - gb };
            ondalik_yap(sonuc, ortak, satir)
        }
        AritmetikIslec::Carp => {
            let mut govde = a.0.checked_mul(b.0).ok_or_else(|| tasma(satir))?;
            let mut olcek = a.1 + b.1;
            if olcek > 9 {
                govde = yuvarla_bol(govde, 10i128.pow(olcek - 9));
                olcek = 9;
            }
            ondalik_yap(govde, olcek, satir)
        }
        AritmetikIslec::Bol => {
            // Hedef ölçek 9: q = ga * 10^(9 - sa + sb) / gb (9 >= sa garantili).
            let ust = 9 - a.1 + b.1;
            let pay = a.0.checked_mul(10i128.pow(ust)).ok_or_else(|| tasma(satir))?;
            ondalik_yap(yuvarla_bol(pay, b.0), 9, satir)
        }
    }
}

fn guncelle(
    hedef: &Ifade,
    miktar: &Ifade,
    ortam: &mut HashMap<String, Deger>,
    program: &Program,
    io: &mut dyn GirdiCikti,
    satir: usize,
    yon: i64,
) -> Result<(), Tani> {
    let ad = match hedef {
        Ifade::Degisken { cozulmus: Some(ad), .. } => ad.clone(),
        _ => return Err(ic_hata(satir)),
    };
    let miktar = degerlendir(miktar, ortam, program, io, satir)?;
    let eski = ortam.get(&ad).cloned().ok_or_else(|| ic_hata(satir))?;
    let islec = if yon > 0 { AritmetikIslec::Topla } else { AritmetikIslec::Cikar };
    let yeni = sayisal_islem(&islec, &eski, &miktar, satir)?;
    ortam.insert(ad, yeni);
    Ok(())
}

fn degerlendir(
    ifade: &Ifade,
    ortam: &HashMap<String, Deger>,
    program: &Program,
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    match ifade {
        Ifade::MetinSabiti(m) => Ok(Deger::Metin(m.clone())),
        Ifade::SayiSabiti(s) => Ok(Deger::TamSayi(*s)),
        Ifade::OndalikSabiti { govde, olcek } => {
            ondalik_yap(*govde as i128, *olcek, satir)
        }
        Ifade::MantiksalSabiti(b) => Ok(Deger::Mantiksal(*b)),
        Ifade::BosListe => Ok(Deger::Liste(Vec::new())),
        Ifade::ListeSabiti(ogeler) => {
            let mut degerler = Vec::new();
            for oge in ogeler {
                degerler.push(degerlendir(oge, ortam, program, io, satir)?);
            }
            // Sayısal karışım Ondalık'a genişler (RFC-0013 §2): öğeler gerçekten
            // dönüştürülür ki listenin türü ile içeriği tutarlı kalsın.
            if degerler.iter().any(|d| matches!(d, Deger::Ondalik { .. })) {
                for deger in degerler.iter_mut() {
                    if let Deger::TamSayi(v) = deger {
                        *deger = ondalik_yap(*v as i128 * 10, 1, satir)?;
                    }
                }
            }
            Ok(Deger::Liste(degerler))
        }
        Ifade::Ozellik { nesne, ozellik } => {
            let nesne = degerlendir(nesne, ortam, program, io, satir)?;
            match (ozellik, nesne) {
                (Ozellik::Adet, Deger::Liste(ogeler)) => Ok(Deger::TamSayi(ogeler.len() as i64)),
                (Ozellik::Ilk, Deger::Liste(ogeler)) | (Ozellik::Son, Deger::Liste(ogeler)) => {
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
                (Ozellik::Uzunluk, Deger::Metin(m)) => {
                    Ok(Deger::TamSayi(m.chars().count() as i64))
                }
                (Ozellik::Kelimeler, Deger::Metin(m)) => Ok(Deger::Liste(
                    m.split_whitespace()
                        .map(|k| Deger::Metin(k.to_string()))
                        .collect(),
                )),
                (Ozellik::Yil, Deger::Tarih { yil, .. }) => Ok(Deger::TamSayi(yil)),
                (Ozellik::TamKisim, Deger::Ondalik { govde, olcek }) => {
                    // Sıfıra doğru kırpma (Rust tam bölmesiyle aynı).
                    Ok(Deger::TamSayi(govde / 10i64.pow(olcek)))
                }
                (Ozellik::Yuvarlanmis, Deger::Ondalik { govde, olcek }) => {
                    let sonuc = yuvarla_bol(govde as i128, 10i128.pow(olcek));
                    Ok(Deger::TamSayi(i64::try_from(sonuc).map_err(|_| tasma(satir))?))
                }
                _ => Err(ic_hata(satir)),
            }
        }
        Ifade::BosSozluk => Ok(Deger::Sozluk(Vec::new())),
        Ifade::SozlukDegeri { sozluk, anahtar } => {
            let girdiler = match degerlendir(sozluk, ortam, program, io, satir)? {
                Deger::Sozluk(girdiler) => girdiler,
                _ => return Err(ic_hata(satir)),
            };
            let anahtar = match degerlendir(anahtar, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            girdiler
                .into_iter()
                .find(|(a, _)| *a == anahtar)
                .map(|(_, d)| d)
                .ok_or_else(|| {
                    Tani::yeni(
                        "C010",
                        format!("Sözlükte \"{}\" anahtarı yok.", anahtar),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Önce \"sözlükte <anahtar> varsa\" ile kontrol et.".into())
                })
        }
        Ifade::SozlukteVar { sozluk, anahtar, olumsuz } => {
            let girdiler = match degerlendir(sozluk, ortam, program, io, satir)? {
                Deger::Sozluk(girdiler) => girdiler,
                _ => return Err(ic_hata(satir)),
            };
            let anahtar = match degerlendir(anahtar, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let var = girdiler.iter().any(|(a, _)| *a == anahtar);
            Ok(Deger::Mantiksal(var != *olumsuz))
        }
        Ifade::MetinDonusum { nesne, buyuk } => {
            let metin = match degerlendir(nesne, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            Ok(Deger::Metin(if *buyuk {
                turkce_buyuk(&metin)
            } else {
                turkce_kucuk(&metin)
            }))
        }
        Ifade::Icerir { metin, aranan } => {
            let metin = match degerlendir(metin, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let aranan = match degerlendir(aranan, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            Ok(Deger::Mantiksal(metin.contains(&aranan)))
        }
        Ifade::YokSabiti => Ok(Deger::Yok),
        Ifade::BugununTarihi => {
            let (yil, ay, gun, _, _) = io.simdi();
            Ok(Deger::Tarih { yil, ay, gun })
        }
        Ifade::SuAninSaati => {
            let (_, _, _, saat, dakika) = io.simdi();
            Ok(Deger::Saat { saat, dakika })
        }
        Ifade::SureSabiti { milisaniye } => Ok(Deger::Sure { milisaniye: *milisaniye }),
        Ifade::KomutArgumanlari => Ok(Deger::Liste(
            io.argumanlar().into_iter().map(Deger::Metin).collect(),
        )),
        Ifade::GunSonrasi { tarih, miktar } => {
            let (yil, ay, gun) = match degerlendir(tarih, ortam, program, io, satir)? {
                Deger::Tarih { yil, ay, gun } => (yil, ay, gun),
                _ => return Err(ic_hata(satir)),
            };
            let miktar = tam_sayi(degerlendir(miktar, ortam, program, io, satir)?, satir)?;
            let (yil, ay, gun) = gunlerden_tarih(tarihten_gunler(yil, ay, gun) + miktar);
            Ok(Deger::Tarih { yil, ay, gun })
        }
        Ifade::BosMu { nesne, olumsuz } => {
            let bos = match degerlendir(nesne, ortam, program, io, satir)? {
                Deger::Liste(ogeler) => ogeler.is_empty(),
                Deger::Sozluk(girdiler) => girdiler.is_empty(),
                Deger::Metin(m) => m.is_empty(),
                _ => return Err(ic_hata(satir)),
            };
            Ok(Deger::Mantiksal(bos != *olumsuz))
        }
        Ifade::YeniYapi { yapi_adi } => {
            let yapi = program
                .yapilar
                .iter()
                .find(|y| y.ad == *yapi_adi)
                .ok_or_else(|| ic_hata(satir))?;
            let alanlar = yapi
                .alanlar
                .iter()
                .map(|(alan, tur)| {
                    let varsayilan = match tur.as_str() {
                        "TamSayı" => Deger::TamSayi(0),
                        "Ondalık" => Deger::Ondalik { govde: 0, olcek: 1 },
                        "Mantıksal" => Deger::Mantiksal(false),
                        _ => Deger::Metin(String::new()),
                    };
                    (alan.clone(), varsayilan)
                })
                .collect();
            Ok(Deger::Yapi(alanlar))
        }
        Ifade::AlanErisim { nesne, alan } => {
            let alanlar = match degerlendir(nesne, ortam, program, io, satir)? {
                Deger::Yapi(alanlar) => alanlar,
                _ => return Err(ic_hata(satir)),
            };
            alanlar
                .into_iter()
                .find(|(a, _)| a == alan)
                .map(|(_, d)| d)
                .ok_or_else(|| ic_hata(satir))
        }
        Ifade::SecenekVar { nesne, olumsuz } => {
            let deger = degerlendir(nesne, ortam, program, io, satir)?;
            let var = deger != Deger::Yok;
            Ok(Deger::Mantiksal(var != *olumsuz))
        }
        Ifade::IcDeger(nesne) => match degerlendir(nesne, ortam, program, io, satir)? {
            Deger::Yok => Err(Tani::yeni(
                "C008",
                "Değer yok: boş Seçenek'in değeri alınamaz.".into(),
                satir,
                1,
                1,
            )
            .onerili("Önce \"... varsa\" ile kontrol et.".into())),
            Deger::Sonuc { basarili: true, icerik } => Ok(*icerik),
            Deger::Sonuc { basarili: false, .. } => Err(Tani::yeni(
                "C009",
                "Sonuç başarısız: değeri yerine hatası var.".into(),
                satir,
                1,
                1,
            )
            .onerili("Önce \"... başarılıysa\" ile kontrol et.".into())),
            dolu => Ok(dolu),
        },
        Ifade::SonucHatasi(nesne) => match degerlendir(nesne, ortam, program, io, satir)? {
            Deger::Sonuc { basarili: false, icerik } => Ok(*icerik),
            Deger::Sonuc { basarili: true, .. } => Err(Tani::yeni(
                "C009",
                "Sonuç başarılı: hatası yok, değeri var.".into(),
                satir,
                1,
                1,
            )),
            _ => Err(ic_hata(satir)),
        },
        Ifade::SonucBasarili { nesne, olumsuz } => {
            match degerlendir(nesne, ortam, program, io, satir)? {
                Deger::Sonuc { basarili, .. } => Ok(Deger::Mantiksal(basarili != *olumsuz)),
                _ => Err(ic_hata(satir)),
            }
        }
        Ifade::DosyaOkumayiDene(yol) => {
            let yol = match degerlendir(yol, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            Ok(match io.dosya_oku(&yol) {
                Ok(icerik) => Deger::Sonuc {
                    basarili: true,
                    icerik: Box::new(Deger::Metin(icerik)),
                },
                Err(hata) => Deger::Sonuc {
                    basarili: false,
                    icerik: Box::new(Deger::Metin(hata)),
                },
            })
        }
        Ifade::TabloOku(yol) => {
            let yol = match degerlendir(yol, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let icerik = io.dosya_oku(&yol).map_err(|hata| {
                Tani::yeni("C012", format!("Dosya okunamadı: {}.", hata), satir, 1, 1)
            })?;
            csv_ayristir(&icerik, satir)
        }
        Ifade::VeriOku(yol) => {
            let yol = match degerlendir(yol, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let icerik = io.dosya_oku(&yol).map_err(|hata| {
                Tani::yeni("C012", format!("Dosya okunamadı: {}.", hata), satir, 1, 1)
            })?;
            json_nesnesi_ayristir(&icerik, satir)
        }
        Ifade::DosyaSatirlari(yol) => {
            let yol = match degerlendir(yol, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let icerik = io.dosya_oku(&yol).map_err(|hata| {
                Tani::yeni("C012", format!("Dosya okunamadı: {}.", hata), satir, 1, 1).onerili(
                    "Hatası yönetilecekse \"... dosyasını okumayı dene\" ile Sonuç al.".into(),
                )
            })?;
            Ok(Deger::Liste(
                icerik
                    .lines()
                    .map(|satir| Deger::Metin(satir.to_string()))
                    .collect(),
            ))
        }
        Ifade::Rastgele { alt, ust } => {
            let alt = tam_sayi(degerlendir(alt, ortam, program, io, satir)?, satir)?;
            let ust = tam_sayi(degerlendir(ust, ortam, program, io, satir)?, satir)?;
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
                metin.push_str(&degerlendir(parca, ortam, program, io, satir)?.metne());
            }
            Ok(Deger::Metin(metin))
        }
        Ifade::Karsilastirma { sol, sag, islec } => {
            let sol = degerlendir(sol, ortam, program, io, satir)?;
            let sag = degerlendir(sag, ortam, program, io, satir)?;
            // Sayısal çift değer üzerinden hizalanarak karşılaştırılır
            // (2 = 2,0 doğrudur; 1,5 < 2 çalışır — RFC-0013 §2).
            if let (Deger::Sure { milisaniye: a }, Deger::Sure { milisaniye: b }) = (&sol, &sag)
            {
                let sonuc = match islec {
                    Islec::Esit => a == b,
                    Islec::Buyuk => a > b,
                    Islec::Kucuk => a < b,
                    Islec::BuyukEsit => a >= b,
                    Islec::KucukEsit => a <= b,
                };
                return Ok(Deger::Mantiksal(sonuc));
            }
            let sonuc = match (sayisal_ac(&sol), sayisal_ac(&sag)) {
                (Some(a), Some(b)) => {
                    let (ga, gb, _) = hizala(a, b);
                    match islec {
                        Islec::Esit => ga == gb,
                        Islec::Buyuk => ga > gb,
                        Islec::Kucuk => ga < gb,
                        Islec::BuyukEsit => ga >= gb,
                        Islec::KucukEsit => ga <= gb,
                    }
                }
                _ => match islec {
                    Islec::Esit => sol == sag,
                    _ => return Err(ic_hata(satir)),
                },
            };
            Ok(Deger::Mantiksal(sonuc))
        }
        Ifade::MantiksalZincir { hepsi, parcalar } => {
            // Kısa devre: VE ilk yanlışta, VEYA ilk doğruda durur.
            for parca in parcalar {
                let deger = mantiksal(degerlendir(parca, ortam, program, io, satir)?, satir)?;
                if deger != *hepsi {
                    return Ok(Deger::Mantiksal(deger));
                }
            }
            Ok(Deger::Mantiksal(*hepsi))
        }
        Ifade::Degil(ic) => {
            let deger = mantiksal(degerlendir(ic, ortam, program, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(!deger))
        }
        Ifade::Cift(ic) => {
            let s = tam_sayi(degerlendir(ic, ortam, program, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(s % 2 == 0))
        }
        Ifade::Tek(ic) => {
            let s = tam_sayi(degerlendir(ic, ortam, program, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(s % 2 != 0))
        }
        Ifade::Aritmetik { islec, sol, sag } => {
            let sol = degerlendir(sol, ortam, program, io, satir)?;
            let sag = degerlendir(sag, ortam, program, io, satir)?;
            sayisal_islem(islec, &sol, &sag, satir)
        }
        Ifade::IslemCagrisi { islem_adi, argumanlar, .. } => {
            let mut degerler = Vec::new();
            for arg in argumanlar {
                degerler.push(degerlendir(arg, ortam, program, io, satir)?);
            }
            islem_cagir(islem_adi, degerler, program, io, satir)?
                .ok_or_else(|| ic_hata(satir))
        }
        Ifade::SayiyiDene(ic) => {
            let metin = match degerlendir(ic, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let kirpilmis = metin.trim();
            Ok(match kirpilmis.parse::<i64>() {
                Ok(sayi) => Deger::Sonuc {
                    basarili: true,
                    icerik: Box::new(Deger::TamSayi(sayi)),
                },
                Err(_) => Deger::Sonuc {
                    basarili: false,
                    icerik: Box::new(Deger::Metin(format!(
                        "\"{}\" sayıya çevrilemedi",
                        kirpilmis
                    ))),
                },
            })
        }
        Ifade::OndaligiDene(ic) => {
            let metin = match degerlendir(ic, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let kirpilmis = metin.trim().to_string();
            let deneme = (|| {
                let (tam, kesir) = match kirpilmis.split_once(',') {
                    Some((tam, kesir)) => (tam, kesir),
                    None => (kirpilmis.as_str(), "0"),
                };
                if tam.is_empty()
                    || kesir.is_empty()
                    || kesir.len() > 9
                    || !tam.chars().all(|k| k.is_ascii_digit())
                    || !kesir.chars().all(|k| k.is_ascii_digit())
                {
                    return None;
                }
                let olcek = kesir.len() as u32;
                let govde = tam
                    .parse::<i128>()
                    .ok()?
                    .checked_mul(10i128.pow(olcek))?
                    .checked_add(kesir.parse::<i128>().ok()?)?;
                ondalik_yap(govde, olcek, satir).ok()
            })();
            Ok(match deneme {
                Some(deger) => Deger::Sonuc { basarili: true, icerik: Box::new(deger) },
                None => Deger::Sonuc {
                    basarili: false,
                    icerik: Box::new(Deger::Metin(format!(
                        "\"{}\" ondalığa çevrilemedi",
                        kirpilmis
                    ))),
                },
            })
        }
        Ifade::Ondaligi(ic) => {
            let metin = match degerlendir(ic, ortam, program, io, satir)? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let kirpilmis = metin.trim();
            let hata = || {
                Tani::yeni(
                    "C004",
                    format!("\"{}\" ondalığa çevrilemedi.", kirpilmis),
                    satir,
                    1,
                    1,
                )
                .onerili("Ondalık, virgülle yazılır. Örnek: 3,14".into())
            };
            let (tam, kesir) = match kirpilmis.split_once(',') {
                Some((tam, kesir)) => (tam, kesir),
                None => (kirpilmis, "0"),
            };
            if tam.is_empty()
                || kesir.is_empty()
                || kesir.len() > 9
                || !tam.chars().all(|k| k.is_ascii_digit())
                || !kesir.chars().all(|k| k.is_ascii_digit())
            {
                return Err(hata());
            }
            let olcek = kesir.len() as u32;
            let govde = (tam.parse::<i128>().map_err(|_| hata())?)
                .checked_mul(10i128.pow(olcek))
                .and_then(|t| t.checked_add(kesir.parse::<i128>().ok()?))
                .ok_or_else(hata)?;
            ondalik_yap(govde, olcek, satir)
        }
        Ifade::Sayisi(ic) => {
            let metin = match degerlendir(ic, ortam, program, io, satir)? {
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

/// CSV ayrıştırma (v0): ilk satır başlıklar, hücreler TamSayı.
/// Ayırıcı virgüldür; hücre içi tırnaklama v0'da desteklenmez.
fn csv_ayristir(icerik: &str, satir: usize) -> Result<Deger, Tani> {
    let mut satirlar = icerik.lines().filter(|s| !s.trim().is_empty());
    let basliklar: Vec<String> = match satirlar.next() {
        Some(baslik) => baslik.split(',').map(|b| b.trim().to_string()).collect(),
        None => {
            return Err(Tani::yeni("C015", "CSV dosyası boş.".into(), satir, 1, 1));
        }
    };
    let mut tablo = Vec::new();
    for (indeks, veri_satiri) in satirlar.enumerate() {
        let hucreler: Vec<&str> = veri_satiri.split(',').map(str::trim).collect();
        if hucreler.len() != basliklar.len() {
            return Err(Tani::yeni(
                "C015",
                format!(
                    "CSV {}. veri satırında {} hücre var; başlıkta {} sütun tanımlı.",
                    indeks + 1,
                    hucreler.len(),
                    basliklar.len()
                ),
                satir,
                1,
                1,
            ));
        }
        let mut kayit = Vec::new();
        for (baslik, hucre) in basliklar.iter().zip(hucreler) {
            let deger: i64 = hucre.parse().map_err(|_| {
                Tani::yeni(
                    "C015",
                    format!(
                        "CSV {}. veri satırındaki \"{}\" hücresi sayı değil (v0'da hücreler TamSayı).",
                        indeks + 1,
                        hucre
                    ),
                    satir,
                    1,
                    1,
                )
            })?;
            kayit.push((baslik.clone(), Deger::TamSayi(deger)));
        }
        tablo.push(Deger::Sozluk(kayit));
    }
    Ok(Deger::Liste(tablo))
}

/// Düz JSON nesnesi ayrıştırma (v0): {"anahtar": "metin", ...}.
/// İç içe nesne/dizi ve metin dışı değerler v0'da desteklenmez.
fn json_nesnesi_ayristir(icerik: &str, satir: usize) -> Result<Deger, Tani> {
    let hata = |mesaj: String| Tani::yeni("C016", mesaj, satir, 1, 1);
    let mut karakterler = icerik.chars().peekable();

    fn bosluk_atla(k: &mut std::iter::Peekable<std::str::Chars>) {
        while matches!(k.peek(), Some(' ' | '\n' | '\r' | '\t')) {
            k.next();
        }
    }
    fn metin_oku(
        k: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<String, String> {
        if k.next() != Some('"') {
            return Err("tırnak bekleniyordu".into());
        }
        let mut metin = String::new();
        loop {
            match k.next() {
                Some('"') => return Ok(metin),
                Some('\\') => match k.next() {
                    Some('"') => metin.push('"'),
                    Some('\\') => metin.push('\\'),
                    Some('n') => metin.push('\n'),
                    Some('t') => metin.push('\t'),
                    _ => return Err("bilinmeyen kaçış".into()),
                },
                Some(c) => metin.push(c),
                None => return Err("metin kapanmadı".into()),
            }
        }
    }

    bosluk_atla(&mut karakterler);
    if karakterler.next() != Some('{') {
        return Err(hata("JSON verisi \"{\" ile başlamalı (v0: düz nesne).".into()));
    }
    let mut girdiler = Vec::new();
    loop {
        bosluk_atla(&mut karakterler);
        if karakterler.peek() == Some(&'}') {
            karakterler.next();
            break;
        }
        let anahtar = metin_oku(&mut karakterler)
            .map_err(|m| hata(format!("JSON anahtarı okunamadı: {}.", m)))?;
        bosluk_atla(&mut karakterler);
        if karakterler.next() != Some(':') {
            return Err(hata(format!("\"{}\" anahtarından sonra \":\" bekleniyor.", anahtar)));
        }
        bosluk_atla(&mut karakterler);
        if karakterler.peek() != Some(&'"') {
            return Err(hata(format!(
                "\"{}\" anahtarının değeri metin değil; v0'da JSON değerleri metin olmalı.",
                anahtar
            )));
        }
        let deger = metin_oku(&mut karakterler)
            .map_err(|m| hata(format!("JSON değeri okunamadı: {}.", m)))?;
        girdiler.push((anahtar, Deger::Metin(deger)));
        bosluk_atla(&mut karakterler);
        match karakterler.next() {
            Some(',') => continue,
            Some('}') => break,
            _ => return Err(hata("JSON nesnesinde \",\" ya da \"}\" bekleniyor.".into())),
        }
    }
    Ok(Deger::Sozluk(girdiler))
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
