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
    Metin(String),
    Mantiksal(bool),
    Liste(Vec<Deger>),
    /// Ekleme sırası korunur (deterministik gezinme).
    Sozluk(Vec<(String, Deger)>),
    /// Seçenek'in boş hali; dolu hali değerin kendisidir.
    Yok,
    /// v0: Sonuç<Metin, Metin>.
    Sonuc { basarili: bool, icerik: String },
    /// Yapı örneği: yalın alan adı → değer (tanım sırasıyla).
    Yapi(Vec<(String, Deger)>),
    Tarih { yil: i64, ay: u32, gun: u32 },
    Saat { saat: u32, dakika: u32 },
}

const AY_ADLARI: [&str; 12] = [
    "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
    "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık",
];

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
                    icerik.clone()
                } else {
                    format!("hata: {}", icerik)
                }
            }
            Deger::Tarih { yil, ay, gun } => {
                format!("{} {} {}", gun, AY_ADLARI[(*ay as usize).saturating_sub(1) % 12], yil)
            }
            Deger::Saat { saat, dakika } => format!("{:02}:{:02}", saat, dakika),
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
            Cumle::Dondur { deger, satir } => {
                let sonuc = degerlendir(deger, ortam, program, cikti, *satir)?;
                return Ok(Akis::Don(sonuc));
            }
            Cumle::BolVeAta { hedef, pay, payda, satir } => {
                let pay = tam_sayi(degerlendir(pay, ortam, program, cikti, *satir)?, *satir)?;
                let payda = tam_sayi(degerlendir(payda, ortam, program, cikti, *satir)?, *satir)?;
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
    let miktar = tam_sayi(degerlendir(miktar, ortam, program, io, satir)?, satir)?;
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
    program: &Program,
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
                degerler.push(degerlendir(oge, ortam, program, io, satir)?);
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
            Deger::Sonuc { basarili: true, icerik } => Ok(Deger::Metin(icerik)),
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
            Deger::Sonuc { basarili: false, icerik } => Ok(Deger::Metin(icerik)),
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
                Ok(icerik) => Deger::Sonuc { basarili: true, icerik },
                Err(hata) => Deger::Sonuc { basarili: false, icerik: hata },
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
            let s = tam_sayi(degerlendir(ic, ortam, program, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(s % 2 == 0))
        }
        Ifade::Tek(ic) => {
            let s = tam_sayi(degerlendir(ic, ortam, program, io, satir)?, satir)?;
            Ok(Deger::Mantiksal(s % 2 != 0))
        }
        Ifade::Aritmetik { islec, sol, sag } => {
            let sol = tam_sayi(degerlendir(sol, ortam, program, io, satir)?, satir)?;
            let sag = tam_sayi(degerlendir(sag, ortam, program, io, satir)?, satir)?;
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
                degerler.push(degerlendir(arg, ortam, program, io, satir)?);
            }
            islem_cagir(islem_adi, degerler, program, io, satir)?
                .ok_or_else(|| ic_hata(satir))
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
