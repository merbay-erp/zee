//! Ad çözümleme + tür denetimi.
//!
//! Ad çözümleme, K-011'deki kuralı uygular: tanımlayıcılara ekler bitişik
//! yazılır ("sayıyı", "toplamı", "sayacı"). Çözüm TAHMİNLE DEĞİL, aday kök
//! üretip kapsamdaki tanımlı adlarla eşleyerek yapılır — birden çok aday
//! eşleşirse bu bir hatadır (determinizm, manifesto 3).
//!
//! Ünsüz yumuşamasının geri çevrimi desteklenir: "sayacı" → "sayac" → "sayaç".

use crate::agac::{Cumle, Ifade, Islec, Islem, Ozellik, Program, Yapi};

use crate::tani::Tani;
use std::collections::HashMap;

/// Kapsayıcı türlerin (Liste, Seçenek) taşıyabildiği öğe türleri (v0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VeriTuru {
    TamSayi,
    Metin,
    Ondalik,
    /// v0'da örtük olarak Sözlük<Metin, TamSayı> demektir (CSV satırları).
    Sozluk,
    /// Kullanıcı yapısı öğesi (K-060): Program.yapilar'a indeks.
    Yapi(usize),
    /// Metin değerli satır sözlüğü (K-062): CSV satırları böyle okunur.
    MetinSozluk,
    /// Boş koleksiyonun henüz belirlenmemiş öğe türü (K-045): ilk eklemede
    /// somutlaşır. Guard'lı yollar dışında ture() çağrılmaz.
    Bilinmeyen,
}

impl VeriTuru {
    fn adi(&self) -> &'static str {
        match self {
            VeriTuru::TamSayi => "TamSayı",
            VeriTuru::Metin => "Metin",
            VeriTuru::Ondalik => "Ondalık",
            VeriTuru::Bilinmeyen => "belirsiz",
            VeriTuru::Yapi(_) => "Yapı",
            VeriTuru::Sozluk => "Sözlük",
            VeriTuru::MetinSozluk => "satır",
        }
    }
    fn ture(&self) -> Tur {
        match self {
            VeriTuru::TamSayi => Tur::TamSayi,
            VeriTuru::Metin => Tur::Metin,
            VeriTuru::Ondalik => Tur::Ondalik,
            VeriTuru::Bilinmeyen => Tur::Yok, // guard'lar erişimi engeller
            VeriTuru::Yapi(i) => Tur::Yapi(*i),
            VeriTuru::Sozluk => Tur::Sozluk(SozlukDegerTuru::TamSayi),
            VeriTuru::MetinSozluk => Tur::Sozluk(SozlukDegerTuru::Metin),
        }
    }
}

/// Sözlük değerlerinin türü (v0: TamSayı ya da Metin).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SozlukDegerTuru {
    /// Boş sözlüğün henüz belirlenmemiş değer türü (K-045).
    Bilinmeyen,
    /// Para/oran sözlükleri (K-067).
    Ondalik,
    TamSayi,
    Metin,
}

impl SozlukDegerTuru {
    fn ture(&self) -> Tur {
        match self {
            SozlukDegerTuru::Bilinmeyen => Tur::Yok, // guard'lar erişimi engeller
            SozlukDegerTuru::Ondalik => Tur::Ondalik,
            SozlukDegerTuru::TamSayi => Tur::TamSayi,
            SozlukDegerTuru::Metin => Tur::Metin,
        }
    }
    fn adi(&self) -> &'static str {
        match self {
            SozlukDegerTuru::Bilinmeyen => "belirsiz",
            SozlukDegerTuru::Ondalik => "Ondalık",
            SozlukDegerTuru::TamSayi => "TamSayı",
            SozlukDegerTuru::Metin => "Metin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tur {
    TamSayi,
    /// Onluk tam değerli ondalık sayı (RFC-0013).
    Ondalik,
    Metin,
    Mantiksal,
    Liste(VeriTuru),
    /// Sözlük<Metin, değer türü>; anahtarlar v0'da hep Metin.
    Sozluk(SozlukDegerTuru),
    Secenek(VeriTuru),
    /// Sonuç<değer, Metin>: değer türü parametreli, hata hep Metin (v0).
    Sonuc(VeriTuru),
    /// Yalnız "yok" sabitinin türü; dönüş birleşiminde Seçenek'e erir.
    Yok,
    /// Yalnız "hatasını döndür"ün iç işareti; birleşimde Sonuç'a erir.
    Hata,
    /// Kullanıcı yapısı — Program.yapilar'a indeks.
    Yapi(usize),
    Tarih,
    Saat,
    /// Milisaniye hassasiyetli süre (RFC-0011/0013).
    Sure,
    /// HTTP yanıtı: durum kodu + gövde (golden 24).
    AgYaniti,
}

impl Tur {
    pub fn adi(&self) -> String {
        match self {
            Tur::TamSayi => "TamSayı".into(),
            Tur::Ondalik => "Ondalık".into(),
            Tur::Metin => "Metin".into(),
            Tur::Mantiksal => "Mantıksal".into(),
            Tur::Liste(e) => format!("Liste<{}>", e.adi()),
            Tur::Sozluk(e) => format!("Sözlük<Metin, {}>", e.adi()),
            Tur::Secenek(e) => format!("Seçenek<{}>", e.adi()),
            Tur::Sonuc(e) => format!("Sonuç<{}>", e.adi()),
            Tur::Yok => "yok".into(),
            Tur::Hata => "hata".into(),
            Tur::Yapi(_) => "Yapı".into(),
            Tur::Tarih => "Tarih".into(),
            Tur::Saat => "Saat".into(),
            Tur::Sure => "Süre".into(),
            Tur::AgYaniti => "AğYanıtı".into(),
        }
    }

    fn veri_turu(&self) -> Option<VeriTuru> {
        match self {
            Tur::TamSayi => Some(VeriTuru::TamSayi),
            Tur::Metin => Some(VeriTuru::Metin),
            Tur::Ondalik => Some(VeriTuru::Ondalik),
            _ => None,
        }
    }

    /// TamSayı ya da Ondalık mı? (Karışımda TamSayı, Ondalık'a kayıpsız genişler.)
    fn sayisal(&self) -> bool {
        matches!(self, Tur::TamSayi | Tur::Ondalik)
    }
}

/// Çoklu denetim (RFC-0010 §3.1): üst düzey cümle başına hata toplanır;
/// bir cümlenin hatası sonrakilerin denetimini durdurmaz. LSP/denetle --json
/// bu görünümü kullanır; derleme (çalıştır) ilk tanıda durur.
pub fn denetle_coklu(program: &mut Program) -> Vec<Tani> {
    let mut tanilar = Vec::new();
    let mut ortam: HashMap<String, Tur> = HashMap::new();
    for yapi in &program.yapilar {
        for (alan, tur_yazimi) in &yapi.alanlar {
            if alan_turu(tur_yazimi).is_none() {
                tanilar.push(
                    Tani::yeni(
                        "T027",
                        format!(
                            "\"{}\" yapısındaki \"{}\" alanının türü tanınmadı: \"{}\".",
                            yapi.ad, alan, tur_yazimi
                        ),
                        yapi.satir,
                        1,
                        1,
                    )
                    .onerili("Kullanılabilir alan türleri: TamSayı, Ondalık, Metin, Mantıksal.".into()),
                );
            }
        }
    }
    let mut baglam = Baglam {
        islemler: std::mem::take(&mut program.islemler),
        imzalar: HashMap::new(),
        yapilar: program.yapilar.clone(),
        bekleyen_gorevler: std::collections::HashSet::new(),
        denetim_yigini: Vec::new(),
        dolu_secenekler: std::collections::HashSet::new(),
        basarili_sonuclar: std::collections::HashSet::new(),
        basarisiz_sonuclar: std::collections::HashSet::new(),
    };
    for cumle in program.cumleler.iter_mut() {
        if let Err(tani) = blok_denetle(std::slice::from_mut(cumle), &mut ortam, &mut baglam) {
            tanilar.push(tani);
            if tanilar.len() >= 20 {
                break;
            }
        }
    }
    for test in program.testler.iter_mut() {
        let mut test_ortami: HashMap<String, Tur> = HashMap::new();
        if let Err(tani) = blok_denetle(&mut test.govde, &mut test_ortami, &mut baglam) {
            tanilar.push(tani);
            if tanilar.len() >= 20 {
                break;
            }
        }
    }
    program.islemler = baglam.islemler;
    tanilar
}

/// Programı yerinde çözümler ve tür denetiminden geçirir.
///
/// İşlemler ilk çağrı anında, argüman türleriyle denetlenir (v0 monomorfizmi):
/// imza ilk çağrıda sabitlenir, sonraki çağrılar imzaya uymalıdır.
pub fn denetle(program: &mut Program) -> Result<(), Tani> {
    let mut ortam: HashMap<String, Tur> = HashMap::new();
    for yapi in &program.yapilar {
        for (alan, tur_yazimi) in &yapi.alanlar {
            if alan_turu(tur_yazimi).is_none() {
                return Err(Tani::yeni(
                    "T027",
                    format!(
                        "\"{}\" yapısındaki \"{}\" alanının türü tanınmadı: \"{}\".",
                        yapi.ad, alan, tur_yazimi
                    ),
                    yapi.satir,
                    1,
                    1,
                )
                .onerili("Kullanılabilir alan türleri: TamSayı, Ondalık, Metin, Mantıksal.".into()));
            }
        }
    }
    let mut baglam = Baglam {
        islemler: std::mem::take(&mut program.islemler),
        imzalar: HashMap::new(),
        yapilar: program.yapilar.clone(),
        bekleyen_gorevler: std::collections::HashSet::new(),
        denetim_yigini: Vec::new(),
        dolu_secenekler: std::collections::HashSet::new(),
        basarili_sonuclar: std::collections::HashSet::new(),
        basarisiz_sonuclar: std::collections::HashSet::new(),
    };
    let mut sonuc = blok_denetle(&mut program.cumleler, &mut ortam, &mut baglam);

    // Testler ana programdan bağımsız, taze ortamda denetlenir.
    if sonuc.is_ok() {
        for test in program.testler.iter_mut() {
            let mut test_ortami: HashMap<String, Tur> = HashMap::new();
            sonuc = blok_denetle(&mut test.govde, &mut test_ortami, &mut baglam);
            if sonuc.is_err() {
                break;
            }
        }
    }

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
    yapilar: Vec<Yapi>,
    /// `eşzamanlı olarak` görev adları; `hepsini bekle`ye dek erişilemez (T033).
    bekleyen_gorevler: std::collections::HashSet<String>,
    /// Denetimi süren işlemler (özyineleme desteği): ad + parametre türleri +
    /// o ana dek görülen dönüş türleri. `döndür` en üsttekine yazar.
    denetim_yigini: Vec<ImzaKaydi>,
    /// Akış-duyarlı daraltma (RFC-0008 §4.2): "X varsa" dalında X'in değeri
    /// güvenlidir; "X başarılıysa" dalında değeri, "başarısızsa" dalında hatası.
    dolu_secenekler: std::collections::HashSet<String>,
    basarili_sonuclar: std::collections::HashSet<String>,
    basarisiz_sonuclar: std::collections::HashSet<String>,
}

struct ImzaKaydi {
    ad: String,
    parametre_turleri: Vec<Tur>,
    donusler: Vec<Tur>,
    /// Özyinelemeli kullanımlara verilen tür (son birleşimle doğrulanır).
    verilen_ozyineleme: Option<Tur>,
}

/// Yapı alanı tür yazımını çözer ("TamSayı" → Tur::TamSayi).
fn alan_turu(yazim: &str) -> Option<Tur> {
    match yazim {
        "TamSayı" => Some(Tur::TamSayi),
        "Ondalık" => Some(Tur::Ondalik),
        "Metin" => Some(Tur::Metin),
        "Mantıksal" => Some(Tur::Mantiksal),
        _ => None,
    }
}

/// Ham alan yazımını ("adı", "yaşı") yapı tanımındaki yalın ada çözer.
fn alan_cozumle(yapi: &Yapi, ham: &str, satir: usize) -> Result<String, Tani> {
    if yapi.alanlar.iter().any(|(a, _)| a == ham) {
        return Ok(ham.to_string());
    }
    let adaylar = kok_adaylari(ham);
    let eslesenler: Vec<&String> = yapi
        .alanlar
        .iter()
        .map(|(a, _)| a)
        .filter(|a| adaylar.iter().any(|aday| aday == *a))
        .collect();
    match eslesenler.len() {
        1 => Ok(eslesenler[0].clone()),
        _ => Err(Tani::yeni(
            "T028",
            format!(
                "\"{}\" yapısında \"{}\" diye bir alan yok. Alanlar: {}.",
                yapi.ad,
                ham,
                yapi.alanlar
                    .iter()
                    .map(|(a, _)| a.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            satir,
            1,
            1,
        )),
    }
}

/// Blok kapsamı (RFC-0004 kararı, v0.2): gövdeye girerken ad kümesi alınır,
/// çıkarken gövdede DOĞAN adlar düşer. Dıştaki ada atama kalıcıdır; içerde
/// aynı adla yeniden tanım diye bir şey yoktur (gölgeleme yapısal olarak yok).
fn kapsam_baslat(ortam: &HashMap<String, Tur>) -> std::collections::HashSet<String> {
    ortam.keys().cloned().collect()
}

fn kapsam_bitir(ortam: &mut HashMap<String, Tur>, kapsam: &std::collections::HashSet<String>) {
    ortam.retain(|ad, _| kapsam.contains(ad));
}

/// Koşuldan daraltma çıkarır: 1=dolu Seçenek, 2=boş, 3=başarılı, 4=başarısız.
fn daraltma_cikar(kosul: &Ifade) -> Option<(u8, String)> {
    match kosul {
        Ifade::SecenekVar { nesne, olumsuz } => {
            nesne_adi(nesne).map(|ad| (if *olumsuz { 2 } else { 1 }, ad))
        }
        Ifade::SonucBasarili { nesne, olumsuz } => {
            nesne_adi(nesne).map(|ad| (if *olumsuz { 4 } else { 3 }, ad))
        }
        _ => None,
    }
}

/// K-045: derleyici türünü VeriTuru'ya indirger (liste öğesi çıkarımı için).
fn veri_turu_yap(tur: &Tur) -> Option<VeriTuru> {
    match tur {
        Tur::TamSayi => Some(VeriTuru::TamSayi),
        Tur::Metin => Some(VeriTuru::Metin),
        Tur::Ondalik => Some(VeriTuru::Ondalik),
        Tur::Sozluk(SozlukDegerTuru::TamSayi) => Some(VeriTuru::Sozluk),
        Tur::Sozluk(SozlukDegerTuru::Metin) => Some(VeriTuru::MetinSozluk),
        Tur::Yapi(i) => Some(VeriTuru::Yapi(*i)),
        _ => None,
    }
}

/// K-045: biri "henüz boş" (Bilinmeyen) koleksiyonsa somut eşiyle uzlaşır;
/// dönen tür bağlamın yeni türüdür. Uzlaşma yoksa None (T002 yolu).
fn bos_koleksiyon_uzlasi(eski: &Tur, yeni: &Tur) -> Option<Tur> {
    match (eski, yeni) {
        (Tur::Liste(VeriTuru::Bilinmeyen), Tur::Liste(_)) => Some(*yeni),
        (Tur::Liste(_), Tur::Liste(VeriTuru::Bilinmeyen)) => Some(*eski),
        (Tur::Sozluk(SozlukDegerTuru::Bilinmeyen), Tur::Sozluk(_)) => Some(*yeni),
        (Tur::Sozluk(_), Tur::Sozluk(SozlukDegerTuru::Bilinmeyen)) => Some(*eski),
        _ => None,
    }
}

fn nesne_adi(nesne: &Ifade) -> Option<String> {
    match nesne {
        Ifade::Degisken { cozulmus: Some(ad), .. } => Some(ad.clone()),
        _ => None,
    }
}

fn daraltma_ekle(baglam: &mut Baglam, tur_kodu: u8, ad: &str) {
    match tur_kodu {
        1 => {
            baglam.dolu_secenekler.insert(ad.to_string());
        }
        3 => {
            baglam.basarili_sonuclar.insert(ad.to_string());
        }
        4 => {
            baglam.basarisiz_sonuclar.insert(ad.to_string());
        }
        _ => {}
    }
}

fn daraltma_cikar_geri(baglam: &mut Baglam, tur_kodu: u8, ad: &str) {
    match tur_kodu {
        1 => {
            baglam.dolu_secenekler.remove(ad);
        }
        3 => {
            baglam.basarili_sonuclar.remove(ad);
        }
        4 => {
            baglam.basarisiz_sonuclar.remove(ad);
        }
        _ => {}
    }
}

fn blok_denetle(
    cumleler: &mut [Cumle],
    ortam: &mut HashMap<String, Tur>,
    baglam: &mut Baglam,
) -> Result<(), Tani> {
    for cumle in cumleler {
        match cumle {
            Cumle::Yaz { deger, satir } => {
                let satir = *satir;
                ifade_denetle(deger, ortam, baglam, satir)?;
            }
            Cumle::Olsun { ad, deger, satir, sutun, uzunluk } => {
                let satir = *satir;
                let mut tur = ifade_denetle(deger, ortam, baglam, satir)?;
                if let Some(eski) = ortam.get(ad.as_str()) {
                    // K-045: boş koleksiyon somut eşiyle iki yönde uzlaşır —
                    // "x boş liste olsun" sonrası somut liste (ve tersi) T002 değildir.
                    if let Some(uzlasi) = bos_koleksiyon_uzlasi(eski, &tur) {
                        tur = uzlasi;
                    } else if *eski != tur {
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
                let kapsam = kapsam_baslat(ortam);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
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
                // Döngü değişkeni gövde kapsamındadır ve gövdeyle ölür (RFC-0004).
                let kapsam = kapsam_baslat(ortam);
                ortam.insert(ad.clone(), Tur::TamSayi);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
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
                let kapsam = kapsam_baslat(ortam);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::Ise { kollar, degilse, satir } => {
                let satir = *satir;
                let mut son_kol_daraltmasi: Option<(u8, String)> = None;
                for kol in kollar.iter_mut() {
                    let tur = ifade_denetle(&mut kol.kosul, ortam, baglam, satir)?;
                    if tur != Tur::Mantiksal {
                        return Err(Tani::yeni("T005", "\"ise\" bir koşul ister.".into(), satir, 1, 1));
                    }
                    // Daraltma çıkarımı (RFC-0008 §4.2).
                    let daraltma = daraltma_cikar(&kol.kosul);
                    son_kol_daraltmasi = daraltma.clone();
                    if let Some((tur_kodu, ad)) = &daraltma {
                        daraltma_ekle(baglam, *tur_kodu, ad);
                    }
                    let kapsam = kapsam_baslat(ortam);
                    let sonuc = blok_denetle(&mut kol.govde, ortam, baglam);
                    if let Some((tur_kodu, ad)) = &daraltma {
                        daraltma_cikar_geri(baglam, *tur_kodu, ad);
                    }
                    sonuc?;
                    kapsam_bitir(ortam, &kapsam);
                }
                if let Some(blok) = degilse {
                    // "X varsa ... değilse" dalında X boştur; "X yoksa ... değilse"
                    // ve "başarısızsa ... değilse" dallarında TERSİ daraltılır.
                    let ters = son_kol_daraltmasi
                        .filter(|_| kollar.len() == 1)
                        .and_then(|(kod, ad)| match kod {
                            1 => None,             // varsa'nın değilse'si: boş
                            2 => Some((1, ad)),    // yoksa'nın değilse'si: dolu
                            3 => Some((4, ad)),    // başarılıysa'nın değilse'si: başarısız
                            4 => Some((3, ad)),    // başarısızsa'nın değilse'si: başarılı
                            _ => None,
                        });
                    if let Some((tur_kodu, ad)) = &ters {
                        daraltma_ekle(baglam, *tur_kodu, ad);
                    }
                    let kapsam = kapsam_baslat(ortam);
                    let sonuc = blok_denetle(blok, ortam, baglam);
                    if let Some((tur_kodu, ad)) = &ters {
                        daraltma_cikar_geri(baglam, *tur_kodu, ad);
                    }
                    sonuc?;
                    kapsam_bitir(ortam, &kapsam);
                }
            }
            Cumle::Ekle { hedef, deger, satir } => {
                let satir = *satir;
                let hedef_tur = ifade_denetle(hedef, ortam, baglam, satir)?;
                let oge = match hedef_tur {
                    // K-045: boş listenin öğe türü ilk eklemeyle somutlaşır.
                    Tur::Liste(VeriTuru::Bilinmeyen) => {
                        let deger_tur = ifade_denetle(deger, ortam, baglam, satir)?;
                        let Some(yeni_oge) = veri_turu_yap(&deger_tur) else {
                            return Err(Tani::yeni(
                                "T011",
                                format!("Liste öğesi {} olamaz.", deger_tur.adi()),
                                satir,
                                1,
                                1,
                            )
                            .onerili("v0'da liste öğesi TamSayı, Ondalık, Metin ya da satır (Sözlük) olabilir.".into()));
                        };
                        if let Some(ad) = nesne_adi(hedef) {
                            ortam.insert(ad, Tur::Liste(yeni_oge));
                        }
                        yeni_oge
                    }
                    Tur::Liste(oge) => oge,
                    baska => {
                        return Err(Tani::yeni(
                            "T012",
                            format!("Ekleme bir listeye yapılır; hedef {} türünde.", baska.adi()),
                            satir,
                            1,
                            1,
                        )
                        .onerili("Önce \"<ad> boş liste olsun\" ya da \"... listesi olsun\" ile liste tanımla.".into()));
                    }
                };
                let deger_tur = ifade_denetle(deger, ortam, baglam, satir)?;
                if deger_tur != oge.ture() {
                    return Err(Tani::yeni(
                        "T011",
                        format!(
                            "{} listesine {} eklenemez.",
                            oge.adi(),
                            deger_tur.adi()
                        ),
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
                        .filter(|aday| {
                            matches!(
                                ortam.get(aday.as_str()),
                                Some(Tur::Liste(_)) | Some(Tur::Sozluk(_))
                            )
                        })
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
                let oge_turu = match kaynak {
                    Some(k) => match ifade_denetle(k, ortam, baglam, satir)? {
                        Tur::Liste(VeriTuru::Bilinmeyen) => {
                            return Err(Tani::yeni(
                                "T013",
                                "Bu liste henüz boş: öğe türü belli değil.".into(),
                                satir,
                                1,
                                1,
                            )
                            .onerili("Gezmeden önce listeye en az bir öğe ekle.".into()));
                        }
                        Tur::Liste(oge) => oge.ture(),
                        // Sözlük üzerinde gezinme anahtarları (Metin) verir.
                        Tur::Sozluk(_) => Tur::Metin,
                        baska => {
                            return Err(Tani::yeni(
                                "T013",
                                format!(
                                    "\"her ... için\" bir liste ya da sözlük ister; burada {} var.",
                                    baska.adi()
                                ),
                                satir,
                                1,
                                1,
                            ));
                        }
                    },
                    None => unreachable!("örtük çoğul yukarıda dolduruldu"),
                };
                let kapsam = kapsam_baslat(ortam);
                ortam.insert(ad.clone(), oge_turu);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::ProgramiBitir { kod, satir } => {
                if let Some(kod) = kod {
                    let tur = ifade_denetle(kod, ortam, baglam, *satir)?;
                    if tur != Tur::TamSayi {
                        return Err(Tani::yeni(
                            "T034",
                            format!("Çıkış kodu TamSayı olmalı; burada {} var.", tur.adi()),
                            *satir,
                            1,
                            1,
                        ));
                    }
                }
            }
            Cumle::SunucuBaslat { kapi, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(kapi, ortam, baglam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T034",
                        format!("Kapı numarası TamSayı olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::IstekGeldiginde { yol, onekli: _, govde, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(yol, ortam, baglam, satir)?;
                if tur != Tur::Metin {
                    return Err(Tani::yeni(
                        "T034",
                        format!("İstek yolu Metin olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                // Her istek taze ortamda işlenir (kapsülleme); form ve sorgu
                // verisi örtük "istek" sözlüğünde gelir (K-051).
                let mut istek_ortami: HashMap<String, Tur> = HashMap::new();
                istek_ortami.insert("istek".into(), Tur::Sozluk(SozlukDegerTuru::Metin));
                istek_ortami.insert("çerezler".into(), Tur::Sozluk(SozlukDegerTuru::Metin));
                blok_denetle(govde, &mut istek_ortami, baglam)?;
            }
            Cumle::YanitGonder { deger, satir } => {
                let satir = *satir;
                ifade_denetle(deger, ortam, baglam, satir)?;
            }
            Cumle::Sil { kap, deger, satir } => {
                let satir = *satir;
                let kap_turu = ifade_denetle(kap, ortam, baglam, satir)?;
                let deger_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                match kap_turu {
                    Tur::Liste(oge) if oge != VeriTuru::Bilinmeyen => {
                        if deger_turu != oge.ture() {
                            return Err(Tani::yeni(
                                "T011",
                                format!("{} listesinden {} silinemez.", oge.adi(), deger_turu.adi()),
                                satir,
                                1,
                                1,
                            ));
                        }
                    }
                    Tur::Sozluk(_) => {
                        if deger_turu != Tur::Metin {
                            return Err(Tani::yeni(
                                "T021",
                                "Sözlükten silme anahtarla (Metin) yapılır.".into(),
                                satir,
                                1,
                                1,
                            ));
                        }
                    }
                    baska => {
                        return Err(Tani::yeni(
                            "T012",
                            format!("Silme bir liste ya da sözlük ister; hedef {} türünde.", baska.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
            }
            Cumle::CerezSil { ad, satir } => {
                let satir = *satir;
                if ifade_denetle(ad, ortam, baglam, satir)? != Tur::Metin {
                    return Err(Tani::yeni(
                        "T034",
                        "Çerez adı Metin olmalı.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::CerezYaz { ad, deger, satir } => {
                let satir = *satir;
                let ad_turu = ifade_denetle(ad, ortam, baglam, satir)?;
                let deger_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                if ad_turu != Tur::Metin || deger_turu != Tur::Metin {
                    return Err(Tani::yeni(
                        "T034",
                        "Çerez adı ve değeri Metin olmalı.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::Yonlendir { adres, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(adres, ortam, baglam, satir)?;
                if tur != Tur::Metin {
                    return Err(Tani::yeni(
                        "T034",
                        format!("Yönlendirme adresi Metin olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::Eszamanli { gorevler, satir } => {
                let satir = *satir;
                for (ad, deger, gorev_satiri) in gorevler.iter_mut() {
                    let tur = ifade_denetle(deger, ortam, baglam, *gorev_satiri)?;
                    if let Some(eski) = ortam.get(ad.as_str()) {
                        if *eski != tur {
                            return Err(Tani::yeni(
                                "T002",
                                format!("\"{}\" daha önce {} türündeydi.", ad, eski.adi()),
                                satir,
                                1,
                                1,
                            ));
                        }
                    }
                    ortam.insert(ad.clone(), tur);
                    baglam.bekleyen_gorevler.insert(ad.clone());
                }
            }
            Cumle::HepsiniBekle { .. } => {
                baglam.bekleyen_gorevler.clear();
            }
            Cumle::IcindeBlogu { sure, govde, yetismezse, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(sure, ortam, baglam, satir)?;
                if tur != Tur::Sure {
                    return Err(Tani::yeni(
                        "T034",
                        format!("\"içinde\" bir Süre ister; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Örnek: 5 saniye içinde".into()));
                }
                let kapsam = kapsam_baslat(ortam);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
                if let Some(blok) = yetismezse {
                    let kapsam = kapsam_baslat(ortam);
                    blok_denetle(blok, ortam, baglam)?;
                    kapsam_bitir(ortam, &kapsam);
                }
            }
            Cumle::IsikAyarla { .. } => {}
            Cumle::Bekle { sure, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(sure, ortam, baglam, satir)?;
                if tur != Tur::Sure {
                    return Err(Tani::yeni(
                        "T034",
                        format!("\"bekle\" bir Süre ister; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Örnek: yarım saniye bekle".into()));
                }
            }
            Cumle::Sor { istem, satir } => {
                let satir = *satir;
                ifade_denetle(istem, ortam, baglam, satir)?;
                // Son cevap örtük "yanıt" adına Metin olarak bağlanır (K-007).
                ortam.insert("yanıt".to_string(), Tur::Metin);
            }
            Cumle::Gore { konu, kollar, degilse, satir } => {
                let satir = *satir;
                let konu_turu = ifade_denetle(konu, ortam, baglam, satir)?;
                if konu_turu.veri_turu().is_none() {
                    return Err(Tani::yeni(
                        "T026",
                        format!("\"göre\" eşleştirmesi TamSayı ya da Metin ister; burada {} var.", konu_turu.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                for (deger, govde) in kollar.iter_mut() {
                    let kol_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                    if kol_turu != konu_turu {
                        return Err(Tani::yeni(
                            "T026",
                            format!(
                                "Eşleştirme kolu {} olmalı ({} ile karşılaştırılıyor); burada {} var.",
                                konu_turu.adi(),
                                konu_turu.adi(),
                                kol_turu.adi()
                            ),
                            satir,
                            1,
                            1,
                        ));
                    }
                    let kapsam = kapsam_baslat(ortam);
                    blok_denetle(govde, ortam, baglam)?;
                    kapsam_bitir(ortam, &kapsam);
                }
                if let Some(blok) = degilse {
                    let kapsam = kapsam_baslat(ortam);
                    blok_denetle(blok, ortam, baglam)?;
                    kapsam_bitir(ortam, &kapsam);
                }
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
            Cumle::YapiTanimi(yapi) => {
                return Err(Tani::yeni(
                    "S021",
                    format!("\"{}\" yapı tanımı beklenmeyen yerde.", yapi.ad),
                    yapi.satir,
                    1,
                    1,
                ));
            }
            Cumle::Kullan { birim, satir } => {
                // Birimler derleme öncesi çözülüp hoist'te düşürülür.
                return Err(Tani::yeni(
                    "S021",
                    format!("\"{}\" birim kullanımı beklenmeyen yerde.", birim),
                    *satir,
                    1,
                    1,
                ));
            }
            Cumle::TestBlogu(test) => {
                return Err(Tani::yeni(
                    "S021",
                    format!("\"{}\" test bloğu beklenmeyen yerde.", test.ad),
                    test.satir,
                    1,
                    1,
                ));
            }
            Cumle::Olmali { kosul, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(kosul, ortam, baglam, satir)?;
                if tur != Tur::Mantiksal {
                    return Err(Tani::yeni(
                        "T005",
                        "\"olmalı\" bir koşul ister.".into(),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Örnek: kare 16 ya eşit olmalı".into()));
                }
            }
            Cumle::AlanAta { nesne, alan, deger, satir } => {
                let satir = *satir;
                let nesne_turu = ifade_denetle(nesne, ortam, baglam, satir)?;
                let yapi_indeksi = match nesne_turu {
                    Tur::Yapi(i) => i,
                    baska => {
                        return Err(Tani::yeni(
                            "T028",
                            format!("Alan yazma bir yapı ister; burada {} var.", baska.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                };
                let yapi = baglam.yapilar[yapi_indeksi].clone();
                let yalin = alan_cozumle(&yapi, alan, satir)?;
                let beklenen = yapi
                    .alanlar
                    .iter()
                    .find(|(a, _)| *a == yalin)
                    .and_then(|(_, t)| alan_turu(t))
                    .expect("alan türü doğrulandı");
                let deger_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                if deger_turu != beklenen {
                    return Err(Tani::yeni(
                        "T028",
                        format!(
                            "\"{}\" alanı {} türünde; {} verilemez.",
                            yalin,
                            beklenen.adi(),
                            deger_turu.adi()
                        ),
                        satir,
                        1,
                        1,
                    ));
                }
                *alan = yalin;
            }
            Cumle::Dondur { deger, satir, .. } => {
                let satir = *satir;
                let tur = ifade_denetle(deger, ortam, baglam, satir)?;
                match baglam.denetim_yigini.last_mut() {
                    Some(kayit) => kayit.donusler.push(tur),
                    None => {
                        return Err(Tani::yeni(
                            "T020",
                            "\"döndür\" yalnız bir işlemin içinde kullanılır.".into(),
                            satir,
                            1,
                            1,
                        ))
                    }
                }
            }
            Cumle::HataDondur { mesaj, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(mesaj, ortam, baglam, satir)?;
                if tur != Tur::Metin {
                    return Err(Tani::yeni(
                        "T032",
                        format!("Hata mesajı Metin olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Örnek: \"sıfıra bölünmez\" hatasını döndür".into()));
                }
                match baglam.denetim_yigini.last_mut() {
                    Some(kayit) => kayit.donusler.push(Tur::Hata),
                    None => {
                        return Err(Tani::yeni(
                            "T020",
                            "\"hatasını döndür\" yalnız bir işlemin içinde kullanılır.".into(),
                            satir,
                            1,
                            1,
                        ))
                    }
                }
            }
            Cumle::BolVeAta { hedef, pay, payda, satir } => {
                let satir = *satir;
                let mut ondalik_var = false;
                for taraf in [&mut *pay, &mut *payda] {
                    let tur = ifade_denetle(taraf, ortam, baglam, satir)?;
                    if !tur.sayisal() {
                        return Err(Tani::yeni(
                            "T008",
                            format!("Bölme sayılar arasında yapılır; burada {} var.", tur.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                    ondalik_var |= tur == Tur::Ondalik;
                }
                // İki TamSayı → tam bölme (mevcut davranış); Ondalık karışımı → Ondalık.
                let sonuc_turu = if ondalik_var { Tur::Ondalik } else { Tur::TamSayi };
                if let Some(eski) = ortam.get(hedef.as_str()) {
                    if *eski != sonuc_turu {
                        return Err(Tani::yeni(
                            "T002",
                            format!("\"{}\" {} türünde; {} bölme sonucu verilemez.", hedef, eski.adi(), sonuc_turu.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                ortam.insert(hedef.clone(), sonuc_turu);
            }
            Cumle::SozlukAta { sozluk, anahtar, deger, satir } => {
                let satir = *satir;
                let sozluk_turu = ifade_denetle(sozluk, ortam, baglam, satir)?;
                let beklenen_deger = match sozluk_turu {
                    // K-045: boş sözlüğün değer türü ilk atamayla somutlaşır.
                    Tur::Sozluk(SozlukDegerTuru::Bilinmeyen) => {
                        let deger_tur = ifade_denetle(deger, ortam, baglam, satir)?;
                        let yeni_deger = match deger_tur {
                            Tur::TamSayi => SozlukDegerTuru::TamSayi,
                            Tur::Ondalik => SozlukDegerTuru::Ondalik,
                            Tur::Metin => SozlukDegerTuru::Metin,
                            baska => {
                                return Err(Tani::yeni(
                                    "T021",
                                    format!("Sözlük değeri {} olamaz.", baska.adi()),
                                    satir,
                                    1,
                                    1,
                                )
                                .onerili("Sözlük değeri TamSayı, Ondalık ya da Metin olabilir.".into()));
                            }
                        };
                        if let Some(ad) = nesne_adi(sozluk) {
                            ortam.insert(ad, Tur::Sozluk(yeni_deger));
                        }
                        yeni_deger.ture()
                    }
                    Tur::Sozluk(e) => e.ture(),
                    baska => {
                        return Err(Tani::yeni(
                            "T021",
                            format!("\"değeri ... olsun\" bir sözlük ister; hedef {} türünde.", baska.adi()),
                            satir,
                            1,
                            1,
                        )
                        .onerili("Önce \"<ad> boş sözlük olsun\" ile sözlük tanımla.".into()));
                    }
                };
                let anahtar_turu = ifade_denetle(anahtar, ortam, baglam, satir)?;
                if anahtar_turu != Tur::Metin {
                    return Err(Tani::yeni(
                        "T021",
                        "v0'da sözlük anahtarı Metin olmalı.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
                let deger_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                if deger_turu != beklenen_deger {
                    return Err(Tani::yeni(
                        "T021",
                        format!(
                            "Bu sözlük {} değerler taşıyor; {} verilemez.",
                            beklenen_deger.adi(),
                            deger_turu.adi()
                        ),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::DosyayaYaz { yol, icerik, satir, .. } => {
                let satir = *satir;
                let yol_turu = ifade_denetle(yol, ortam, baglam, satir)?;
                if yol_turu != Tur::Metin {
                    return Err(Tani::yeni(
                        "T025",
                        format!("Dosya yolu Metin olmalı; burada {} var.", yol_turu.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                ifade_denetle(icerik, ortam, baglam, satir)?;
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
                if !hedef_tur.sayisal() {
                    return Err(Tani::yeni(
                        "T006",
                        format!("Artırma/azaltma sayı ister; hedef {} türünde.", hedef_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                let miktar_tur = ifade_denetle(miktar, ortam, baglam, satir)?;
                if !miktar_tur.sayisal() {
                    return Err(Tani::yeni(
                        "T006",
                        format!("Artırma/azaltma miktarı sayı olmalı; burada {} var.", miktar_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                if hedef_tur == Tur::TamSayi && miktar_tur == Tur::Ondalik {
                    return Err(Tani::yeni(
                        "T006",
                        "TamSayı hedefe Ondalık miktar eklenemez: sonuç tam sayı kalamaz.".into(),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Hedefi ondalık başlat (örn. 0,0 olsun) ya da miktarı tam sayı yap.".into()));
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
        Ifade::OndalikSabiti { .. } => Ok(Tur::Ondalik),
        Ifade::MantiksalSabiti(_) => Ok(Tur::Mantiksal),
        // Boş listenin öğe türü v0'da TamSayı varsayılır (tür çıkarımı RFC-0007).
        // K-045: öğe türü ilk eklemede somutlaşır.
        Ifade::BosListe => Ok(Tur::Liste(VeriTuru::Bilinmeyen)),
        Ifade::ListeSabiti(ogeler) => {
            let mut oge_turu: Option<VeriTuru> = None;
            for oge in ogeler {
                let tur = ifade_denetle(oge, ortam, baglam, satir)?;
                let veri = tur.veri_turu().ok_or_else(|| {
                    Tani::yeni(
                        "T011",
                        format!("Liste öğesi TamSayı ya da Metin olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                })?;
                match oge_turu {
                    None => oge_turu = Some(veri),
                    Some(onceki) if onceki != veri => {
                        // TamSayı + Ondalık karışımı Ondalık'a genişler (RFC-0013 §2);
                        // yorumlayıcı öğeleri gerçekten genişletir.
                        let sayisal_karisim = matches!(
                            (onceki, veri),
                            (VeriTuru::TamSayi, VeriTuru::Ondalik)
                                | (VeriTuru::Ondalik, VeriTuru::TamSayi)
                        );
                        if sayisal_karisim {
                            oge_turu = Some(VeriTuru::Ondalik);
                        } else {
                            return Err(Tani::yeni(
                                "T011",
                                "Bir listenin bütün öğeleri aynı türden olmalı.".into(),
                                satir,
                                1,
                                1,
                            ));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Tur::Liste(oge_turu.unwrap_or(VeriTuru::TamSayi)))
        }
        Ifade::Ozellik { nesne, ozellik } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            // K-064: nesne bir yapıysa ve özellik kelimesi bir ALANA çözülüyorsa
            // bu aslında alan erişimidir ("ürünün adedi" → alan "adet").
            // Örtük-çoğul emsalindeki gibi ifade yeniden yazılır.
            if let Tur::Yapi(yapi_indeksi) = tur {
                let soz = match ozellik {
                    Ozellik::Adet => "adedi",
                    Ozellik::Ilk => "ilki",
                    Ozellik::Son => "sonu",
                    Ozellik::Uzunluk => "uzunluğu",
                    Ozellik::Kelimeler => "kelimeleri",
                    Ozellik::Yil => "yılı",
                    Ozellik::TamKisim => "kısmı",
                    Ozellik::Yuvarlanmis => "yuvarlanmışı",
                    Ozellik::HtmlGuvenli => "güvenlisi",
                    Ozellik::Kirpilmis => "kırpılmışı",
                    Ozellik::Harfler => "harfleri",
                    Ozellik::JsonMetin => "metni",
                    Ozellik::Siralanmis => "sıralanmışı",
                    Ozellik::Ters => "tersi",
                    Ozellik::CsvMetin => "metni",
                    Ozellik::Kuruslu => "kuruşlusu",
                    Ozellik::Metni => "metni",
                    Ozellik::BinlikliKuruslu => "kuruşlusu",
                };
                let yapi = &baglam.yapilar[yapi_indeksi];
                if let Ok(alan) = alan_cozumle(yapi, soz, satir) {
                    let yeni = Ifade::AlanErisim {
                        nesne: nesne.clone(),
                        alan: alan.clone(),
                    };
                    *ifade = yeni;
                    return ifade_denetle(ifade, ortam, baglam, satir);
                }
            }
            match (ozellik, tur) {
                (Ozellik::Adet, Tur::Liste(_)) => Ok(Tur::TamSayi),
                (Ozellik::Ilk, Tur::Liste(VeriTuru::Bilinmeyen))
                | (Ozellik::Son, Tur::Liste(VeriTuru::Bilinmeyen)) => Err(Tani::yeni(
                    "T014",
                    "Bu liste henüz boş: ilki/sonu yok.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Önce listeye öğe ekle.".into())),
                (Ozellik::Ilk, Tur::Liste(e)) | (Ozellik::Son, Tur::Liste(e)) => Ok(e.ture()),
                (Ozellik::Uzunluk, Tur::Metin) => Ok(Tur::TamSayi),
                (Ozellik::HtmlGuvenli, Tur::Metin) => Ok(Tur::Metin),
                (Ozellik::Kirpilmis, Tur::Metin) => Ok(Tur::Metin),
                (Ozellik::Siralanmis, Tur::Liste(oge))
                    if matches!(oge, VeriTuru::TamSayi | VeriTuru::Ondalik | VeriTuru::Metin) =>
                {
                    Ok(Tur::Liste(oge))
                }
                (Ozellik::Ters, Tur::Liste(oge)) if oge != VeriTuru::Bilinmeyen => {
                    Ok(Tur::Liste(oge))
                }
                (Ozellik::Kuruslu, Tur::Ondalik)
                | (Ozellik::Kuruslu, Tur::TamSayi)
                | (Ozellik::BinlikliKuruslu, Tur::Ondalik)
                | (Ozellik::BinlikliKuruslu, Tur::TamSayi) => Ok(Tur::Metin),
                // K-066: her değerin resmî metin hali (yaz ile aynı temsil).
                (Ozellik::Metni, _) => Ok(Tur::Metin),
                (Ozellik::CsvMetin, Tur::Liste(VeriTuru::Sozluk))
                | (Ozellik::CsvMetin, Tur::Liste(VeriTuru::MetinSozluk)) => Ok(Tur::Metin),
                (Ozellik::Harfler, Tur::Metin) => Ok(Tur::Liste(VeriTuru::Metin)),
                (Ozellik::JsonMetin, Tur::Sozluk(_))
                | (Ozellik::JsonMetin, Tur::Liste(_))
                | (Ozellik::JsonMetin, Tur::Metin)
                | (Ozellik::JsonMetin, Tur::TamSayi)
                | (Ozellik::JsonMetin, Tur::Ondalik)
                | (Ozellik::JsonMetin, Tur::Mantiksal) => Ok(Tur::Metin),
                (Ozellik::Kelimeler, Tur::Metin) => Ok(Tur::Liste(VeriTuru::Metin)),
                (Ozellik::Yil, Tur::Tarih) => Ok(Tur::TamSayi),
                (Ozellik::TamKisim, Tur::Ondalik) | (Ozellik::Yuvarlanmis, Tur::Ondalik) => {
                    Ok(Tur::TamSayi)
                }
                (_, baska) => Err(Tani::yeni(
                    "T014",
                    format!("Bu özellik {} türüne uygulanamaz.", baska.adi()),
                    satir,
                    1,
                    1,
                )
                .onerili(
                    "adedi/ilki/sonu listeler, uzunluğu/kelimeleri metinler, tam kısmı/yuvarlanmışı ondalıklar içindir.".into(),
                )),
            }
        }
        // K-045: değer türü ilk atamada somutlaşır (eski varsayım TamSayı idi).
        Ifade::BosSozluk => Ok(Tur::Sozluk(SozlukDegerTuru::Bilinmeyen)),
        Ifade::SozlukDegeri { sozluk, anahtar } => {
            let sozluk_turu = ifade_denetle(sozluk, ortam, baglam, satir)?;
            let deger_turu = match sozluk_turu {
                Tur::Sozluk(SozlukDegerTuru::Bilinmeyen) => {
                    return Err(Tani::yeni(
                        "T021",
                        "Bu sözlük henüz boş: değer türü belli değil.".into(),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Önce bir değer ata: sözlüğün \"anahtar\" değeri ... olsun".into()));
                }
                Tur::Sozluk(e) => e,
                baska => {
                    return Err(Tani::yeni(
                        "T021",
                        format!("\"değeri\" ile okuma bir sözlük ister; burada {} var.", baska.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            };
            let anahtar_turu = ifade_denetle(anahtar, ortam, baglam, satir)?;
            if anahtar_turu != Tur::Metin {
                return Err(Tani::yeni(
                    "T021",
                    format!("v0'da sözlük anahtarı Metin olmalı; burada {} var.", anahtar_turu.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(deger_turu.ture())
        }
        Ifade::SozlukteVar { sozluk, anahtar, .. } => {
            let sozluk_turu = ifade_denetle(sozluk, ortam, baglam, satir)?;
            // K-058: aynı yüzey listede üyelik de sorar: "sayılarda 5 varsa".
            if let Tur::Liste(oge) = sozluk_turu {
                if !matches!(oge, VeriTuru::TamSayi | VeriTuru::Ondalik | VeriTuru::Metin) {
                    return Err(Tani::yeni(
                        "T021",
                        "Listede üyelik yalnız sayı/metin listelerinde sorulur.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
                let aranan = ifade_denetle(anahtar, ortam, baglam, satir)?;
                if aranan != oge.ture() {
                    return Err(Tani::yeni(
                        "T021",
                        format!("{} listesinde {} aranamaz.", oge.adi(), aranan.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                return Ok(Tur::Mantiksal);
            }
            if !matches!(sozluk_turu, Tur::Sozluk(_)) {
                return Err(Tani::yeni(
                    "T021",
                    format!("\"varsa\" sorgusu burada bir sözlük ister; {} var.", sozluk_turu.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            let anahtar_turu = ifade_denetle(anahtar, ortam, baglam, satir)?;
            if anahtar_turu != Tur::Metin {
                return Err(Tani::yeni(
                    "T021",
                    "v0'da sözlük anahtarı Metin olmalı.".into(),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::MetinDonusum { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T022",
                    format!("büyük/küçük harfli dönüşümü Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Metin)
        }
        Ifade::Icerir { metin, aranan } => {
            for taraf in [&mut **metin, &mut **aranan] {
                let tur = ifade_denetle(taraf, ortam, baglam, satir)?;
                if tur != Tur::Metin {
                    return Err(Tani::yeni(
                        "T022",
                        format!("\"içeriyorsa\" metinler arasında sorgulanır; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::YokSabiti => Ok(Tur::Yok),
        Ifade::BugununTarihi => Ok(Tur::Tarih),
        Ifade::SuAninSaati => Ok(Tur::Saat),
        Ifade::KomutArgumanlari => Ok(Tur::Liste(VeriTuru::Metin)),
        Ifade::SureSabiti { .. } => Ok(Tur::Sure),
        Ifade::HttpGetir(url) => {
            let tur = ifade_denetle(url, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T034",
                    format!("Adres Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::AgYaniti)
        }
        Ifade::DurumKodu(nesne) => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if tur != Tur::AgYaniti {
                return Err(Tani::yeni(
                    "T034",
                    format!("\"durum kodu\" bir ağ yanıtı ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::TamSayi)
        }
        Ifade::Govde(nesne) => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if tur != Tur::AgYaniti {
                return Err(Tani::yeni(
                    "T034",
                    format!("\"gövdesi\" bir ağ yanıtı ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Metin)
        }
        Ifade::SensorAcik { .. } => Ok(Tur::Mantiksal),
        Ifade::GunSonrasi { tarih, miktar } => {
            let tarih_turu = ifade_denetle(tarih, ortam, baglam, satir)?;
            if tarih_turu != Tur::Tarih {
                return Err(Tani::yeni(
                    "T029",
                    format!("\"gün sonrası\" bir Tarih ister; burada {} var.", tarih_turu.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            let miktar_turu = ifade_denetle(miktar, ortam, baglam, satir)?;
            if miktar_turu != Tur::TamSayi {
                return Err(Tani::yeni(
                    "T029",
                    format!("Gün sayısı TamSayı olmalı; burada {} var.", miktar_turu.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Tarih)
        }
        Ifade::BosMu { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            match tur {
                Tur::Liste(_) | Tur::Sozluk(_) | Tur::Metin => Ok(Tur::Mantiksal),
                baska => Err(Tani::yeni(
                    "T030",
                    format!("\"boşsa\" liste, sözlük ya da metin ister; burada {} var.", baska.adi()),
                    satir,
                    1,
                    1,
                )),
            }
        }
        Ifade::YeniYapi { yapi_adi } => {
            match baglam.yapilar.iter().position(|y| y.ad == *yapi_adi) {
                Some(i) => Ok(Tur::Yapi(i)),
                None => Err(Tani::yeni(
                    "A007",
                    format!("\"{}\" adında bir yapı tanımlı değil.", yapi_adi),
                    satir,
                    1,
                    1,
                )
                .onerili("Önce \"yapı <Ad>\" ile tanımla; yapı, kullanımından önce gelmeli.".into())),
            }
        }
        Ifade::AlanErisim { nesne, alan } => {
            let nesne_turu = ifade_denetle(nesne, ortam, baglam, satir)?;
            let yapi_indeksi = match nesne_turu {
                Tur::Yapi(i) => i,
                baska => {
                    return Err(Tani::yeni(
                        "T028",
                        format!("Alan okuma bir yapı ister; burada {} var.", baska.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            };
            let yapi = baglam.yapilar[yapi_indeksi].clone();
            let yalin = alan_cozumle(&yapi, alan, satir)?;
            let tur = yapi
                .alanlar
                .iter()
                .find(|(a, _)| *a == yalin)
                .and_then(|(_, t)| alan_turu(t))
                .expect("alan türü doğrulandı");
            *alan = yalin;
            Ok(tur)
        }
        Ifade::SecenekVar { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            match tur {
                Tur::Secenek(_) => Ok(Tur::Mantiksal),
                baska => Err(Tani::yeni(
                    "T023",
                    format!("\"varsa\" sorgusu bir Seçenek ister; burada {} var.", baska.adi()),
                    satir,
                    1,
                    1,
                )
                .onerili("Seçenek, değer döndüren bir işlemin \"yok döndür\" ile karışık dönüşünden doğar.".into())),
            }
        }
        Ifade::IcDeger(nesne) => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            let ad = nesne_adi(nesne);
            match tur {
                Tur::Secenek(e) => {
                    if let Some(ad) = ad {
                        if !baglam.dolu_secenekler.contains(&ad) {
                            return Err(Tani::yeni(
                                "T036",
                                format!(
                                    "\"{}\" boş olabilir: değeri ancak \"varsa\" dalında alınır.",
                                    ad
                                ),
                                satir,
                                1,
                                1,
                            )
                            .onerili(format!("Önce kontrol et: {} varsa", ad)));
                        }
                    }
                    Ok(e.ture())
                }
                Tur::Sonuc(e) => {
                    if let Some(ad) = ad {
                        if !baglam.basarili_sonuclar.contains(&ad) {
                            return Err(Tani::yeni(
                                "T036",
                                format!(
                                    "\"{}\" başarısız olabilir: değeri ancak \"başarılıysa\" dalında alınır.",
                                    ad
                                ),
                                satir,
                                1,
                                1,
                            )
                            .onerili(format!("Önce kontrol et: {} başarılıysa", ad)));
                        }
                    }
                    Ok(e.ture())
                }
                baska => Err(Tani::yeni(
                    "T024",
                    format!("\"değeri\" bir Seçenek ya da Sonuç ister; burada {} var.", baska.adi()),
                    satir,
                    1,
                    1,
                )),
            }
        }
        Ifade::SonucHatasi(nesne) => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if matches!(tur, Tur::Sonuc(_)) {
                if let Some(ad) = nesne_adi(nesne) {
                    if !baglam.basarisiz_sonuclar.contains(&ad) {
                        return Err(Tani::yeni(
                            "T036",
                            format!(
                                "\"{}\" başarılı olabilir: hatası ancak \"başarısızsa\" dalında okunur.",
                                ad
                            ),
                            satir,
                            1,
                            1,
                        )
                        .onerili(format!("Önce kontrol et: {} başarısızsa", ad)));
                    }
                }
            }
            if !matches!(tur, Tur::Sonuc(_)) {
                return Err(Tani::yeni(
                    "T024",
                    format!("\"hatası\" bir Sonuç ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Metin)
        }
        Ifade::SonucBasarili { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if !matches!(tur, Tur::Sonuc(_)) {
                return Err(Tani::yeni(
                    "T024",
                    format!("\"başarılıysa\" bir Sonuç ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::DosyaOkumayiDene(yol) => {
            let tur = ifade_denetle(yol, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T025",
                    format!("Dosya yolu Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Sonuc(VeriTuru::Metin))
        }
        Ifade::DosyaSatirlari(yol) => {
            let tur = ifade_denetle(yol, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T025",
                    format!("Dosya yolu Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Liste(VeriTuru::Metin))
        }
        Ifade::TabloOku(yol) => {
            let tur = ifade_denetle(yol, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T025",
                    format!("Dosya yolu Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            // K-062: hücreler Metin okunur; sayı gerekirse `değerin sayısı`.
            Ok(Tur::Liste(VeriTuru::MetinSozluk))
        }
        Ifade::VeriOku(yol) => {
            let tur = ifade_denetle(yol, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T025",
                    format!("Dosya yolu Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Sozluk(SozlukDegerTuru::Metin))
        }
        Ifade::GunFarki { birinci, ikinci } => {
            let b = ifade_denetle(birinci, ortam, baglam, satir)?;
            let i = ifade_denetle(ikinci, ortam, baglam, satir)?;
            if b != Tur::Tarih || i != Tur::Tarih {
                return Err(Tani::yeni(
                    "T029",
                    format!("\"arasındaki günler\" iki Tarih ister; burada {} ile {} var.", b.adi(), i.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::TamSayi)
        }
        Ifade::Parcala { metin, ayrac } => {
            let m = ifade_denetle(metin, ortam, baglam, satir)?;
            let a = ifade_denetle(ayrac, ortam, baglam, satir)?;
            if m != Tur::Metin || a != Tur::Metin {
                return Err(Tani::yeni("T022", "\"parçaları\" iki Metin ister: metnin ayraçla parçaları.".into(), satir, 1, 1));
            }
            Ok(Tur::Liste(VeriTuru::Metin))
        }
        Ifade::ListeBirlestir { liste, ayrac } => {
            let l = ifade_denetle(liste, ortam, baglam, satir)?;
            let a = ifade_denetle(ayrac, ortam, baglam, satir)?;
            if l != Tur::Liste(VeriTuru::Metin) || a != Tur::Metin {
                return Err(Tani::yeni("T022", format!("\"birleşmişi\" Metin listesi ile Metin ayraç ister; burada {} ile {} var.", l.adi(), a.adi()), satir, 1, 1));
            }
            Ok(Tur::Metin)
        }
        Ifade::Degistir { metin, eski, yeni } => {
            for parca in [metin, eski, yeni] {
                if ifade_denetle(parca, ortam, baglam, satir)? != Tur::Metin {
                    return Err(Tani::yeni("T022", "\"değişmişi\" üç Metin ister: metnin eski yerine yeni değişmişi.".into(), satir, 1, 1));
                }
            }
            Ok(Tur::Metin)
        }
        Ifade::MetinSinari { metin, parca, .. } => {
            let m = ifade_denetle(metin, ortam, baglam, satir)?;
            let p = ifade_denetle(parca, ortam, baglam, satir)?;
            if m != Tur::Metin || p != Tur::Metin {
                return Err(Tani::yeni("T022", "başlıyorsa/bitiyorsa iki Metin ister.".into(), satir, 1, 1));
            }
            Ok(Tur::Mantiksal)
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
            // RFC-0011 §1: görev sonucuna "hepsini bekle"den önce erişilemez.
            if baglam.bekleyen_gorevler.contains(&ad) {
                return Err(Tani::yeni(
                    "T033",
                    format!(
                        "\"{}\" bir eşzamanlı görev: sonucuna \"hepsini bekle\"den önce erişilemez.",
                        ad
                    ),
                    *satir,
                    *sutun,
                    *uzunluk,
                ));
            }
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
            let iki_sure = sol_tur == Tur::Sure && sag_tur == Tur::Sure;
            if !esitlik && !iki_sure && (!sol_tur.sayisal() || !sag_tur.sayisal()) {
                let sorunlu = if !sol_tur.sayisal() { sol_tur } else { sag_tur };
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
            if esitlik && sol_tur != sag_tur && !(sol_tur.sayisal() && sag_tur.sayisal()) {
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
        Ifade::MantiksalZincir { parcalar, .. } => {
            for parca in parcalar {
                let tur = ifade_denetle(parca, ortam, baglam, satir)?;
                if tur != Tur::Mantiksal {
                    return Err(Tani::yeni(
                        "T031",
                        format!("ve/veya zincirinin her parçası koşul olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::Degil(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Mantiksal {
                return Err(Tani::yeni(
                    "T031",
                    format!("\"değilse\" bir koşulu olumsuzlar; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnekler: x 5 e eşit değilse · bildi doğru değilse".into()));
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
        Ifade::Aritmetik { islec, sol, sag } => {
            // K-046: kalan yalnız TamSayılar arasında (okul kavramı tam bölmeye ait).
            if matches!(islec, crate::agac::AritmetikIslec::Kalan) {
                let sol_tur = ifade_denetle(sol, ortam, baglam, satir)?;
                let sag_tur = ifade_denetle(sag, ortam, baglam, satir)?;
                if sol_tur != Tur::TamSayi || sag_tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T008",
                        format!(
                            "Kalan iki TamSayı ister; burada {} ile {} var.",
                            sol_tur.adi(),
                            sag_tur.adi()
                        ),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Ondalık için önce tam kısmını al.".into()));
                }
                return Ok(Tur::TamSayi);
            }
            // Süre + Süre: yalnız toplama/çıkarma (RFC-0011).
            {
                let sol_on = ifade_denetle(sol, ortam, baglam, satir)?;
                let sag_on = ifade_denetle(sag, ortam, baglam, satir)?;
                if sol_on == Tur::Sure || sag_on == Tur::Sure {
                    if sol_on != Tur::Sure || sag_on != Tur::Sure {
                        return Err(Tani::yeni(
                            "T008",
                            format!(
                                "Süre yalnız süreyle toplanıp çıkarılır; burada {} ile {} var.",
                                sol_on.adi(),
                                sag_on.adi()
                            ),
                            satir,
                            1,
                            1,
                        ));
                    }
                    return match islec {
                        crate::agac::AritmetikIslec::Topla
                        | crate::agac::AritmetikIslec::Cikar => Ok(Tur::Sure),
                        _ => Err(Tani::yeni(
                            "T008",
                            "Süre çarpılamaz ve bölünemez (v0).".into(),
                            satir,
                            1,
                            1,
                        )),
                    };
                }
            }
            let mut ondalik_var = false;
            for taraf in [&mut **sol, &mut **sag] {
                let tur = ifade_denetle(taraf, ortam, baglam, satir)?;
                if !tur.sayisal() {
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
                ondalik_var |= tur == Tur::Ondalik;
            }
            // TamSayı → Ondalık genişlemesi kayıpsızdır (RFC-0013 §2).
            Ok(if ondalik_var { Tur::Ondalik } else { Tur::TamSayi })
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
        Ifade::SayiyiDene(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T009",
                    format!("\"almayı dene\" kalıbı Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Sonuc(VeriTuru::TamSayi))
        }
        Ifade::OndaligiDene(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T009",
                    format!("\"almayı dene\" kalıbı Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Sonuc(VeriTuru::Ondalik))
        }
        Ifade::Ondaligi(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T009",
                    format!("\"ondalığı\" kalıbı Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Ondalik)
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

/// Sonuç dönen işlemin başarı dallarındaki `döndür`leri sarmalama için
/// işaretler (RFC-0008 §4.1). HataDondur zaten Sonuç-hata üretir, dokunulmaz.
fn donusleri_sarmala(cumleler: &mut [Cumle]) {
    for cumle in cumleler {
        match cumle {
            Cumle::Dondur { sonuca_sarmala, .. } => *sonuca_sarmala = true,
            Cumle::KezTekrarla { govde, .. }
            | Cumle::AralikDongusu { govde, .. }
            | Cumle::OlduguSurece { govde, .. }
            | Cumle::OlanaKadar { govde, .. }
            | Cumle::HerBiri { govde, .. } => donusleri_sarmala(govde),
            Cumle::Ise { kollar, degilse, .. } => {
                for kol in kollar {
                    donusleri_sarmala(&mut kol.govde);
                }
                if let Some(blok) = degilse {
                    donusleri_sarmala(blok);
                }
            }
            Cumle::Gore { kollar, degilse, .. } => {
                for (_, govde) in kollar {
                    donusleri_sarmala(govde);
                }
                if let Some(blok) = degilse {
                    donusleri_sarmala(blok);
                }
            }
            _ => {}
        }
    }
}

/// İşlem çağrısını denetler. İlk çağrıda gövde, argüman türleriyle denetlenip
/// imza sabitlenir (v0 monomorfizmi); sonraki çağrılar imzaya uymalıdır.
///
/// ÖZYİNELEME (v0.2): denetimi süren bir işlem kendini (ya da karşılıklı
/// olarak birbirini) çağırabilir. Özyinelemeli çağrının türü, o ana dek
/// görülen dönüş dallarından çıkarılır — bu yüzden TEMEL DURUM ÖNCE yazılır
/// (T035): hem tür çıkarımı hem sonsuz döngüye karşı pedagojik korkuluk.
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
        // K-067: çağrıda genişleme — TamSayı argüman Ondalık parametreye,
        // Liste<TamSayı> argüman Liste<Ondalık> parametreye uyar (skaler
        // genişleme kuralının doğal uzantısı; ters yön yine bilinçli değil).
        let uyumlu = imza.parametre_turleri.len() == arg_turleri.len()
            && imza
                .parametre_turleri
                .iter()
                .zip(arg_turleri.iter())
                .all(|(param, arg)| {
                    param == arg
                        || matches!((param, arg), (Tur::Ondalik, Tur::TamSayi))
                        || matches!(
                            (param, arg),
                            (
                                Tur::Liste(VeriTuru::Ondalik),
                                Tur::Liste(VeriTuru::TamSayi)
                            )
                        )
                });
        if !uyumlu {
            // K-067 imza terfisi: uyumsuzluk YALNIZ ters-genişlemeyse
            // (param TamSayı[-listesi], arg Ondalık[-listesi]) imza kaldırılır
            // ve gövde geniş türlerle ilk-çağrı gibi yeniden denetlenir —
            // sonuç, çağrı sırasından bağımsız en geniş imzadır. Gövde geniş
            // türle geçerli değilse doğal tanısı çıkar. Özyineleme denetimi
            // sürerken terfi yapılmaz (T017 kalır).
            let yalniz_ters_genisleme = imza
                .parametre_turleri
                .iter()
                .zip(arg_turleri.iter())
                .all(|(param, arg)| {
                    param == arg
                        || matches!((param, arg), (Tur::Ondalik, Tur::TamSayi))
                        || matches!(
                            (param, arg),
                            (Tur::Liste(VeriTuru::Ondalik), Tur::Liste(VeriTuru::TamSayi))
                        )
                        || matches!((param, arg), (Tur::TamSayi, Tur::Ondalik))
                        || matches!(
                            (param, arg),
                            (Tur::Liste(VeriTuru::TamSayi), Tur::Liste(VeriTuru::Ondalik))
                        )
                });
            let ozyinelemede = baglam.denetim_yigini.iter().any(|k| k.ad == ad);
            if yalniz_ters_genisleme && !ozyinelemede {
                baglam.imzalar.remove(ad);
                return cagri_denetle(ad, arg_turleri, baglam, satir);
            }
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

    // Özyinelemeli (ya da karşılıklı özyinelemeli) çağrı: denetim yığınında.
    if let Some(indeks) = baglam.denetim_yigini.iter().position(|k| k.ad == ad) {
        if baglam.denetim_yigini[indeks].parametre_turleri != arg_turleri {
            return Err(Tani::yeni(
                "T017",
                format!(
                    "\"{}\" özyinelemeli çağrısındaki argüman türleri ilk çağrıyla uyuşmuyor.",
                    ad
                ),
                satir,
                1,
                1,
            ));
        }
        let tahmin =
            donusleri_birlestir(ad, &baglam.denetim_yigini[indeks].donusler, satir)?;
        let tahmin = match tahmin {
            Some(tur) => tur,
            None => {
                return Err(Tani::yeni(
                    "T035",
                    format!(
                        "\"{}\" özyinelemeli çağrıdan ÖNCE en az bir dalda değer döndürmeli.",
                        ad
                    ),
                    satir,
                    1,
                    1,
                )
                .onerili(
                    "Temel durumu üste yaz: önce \"n 1 e eşitse\" gibi bir dalda döndür, \
                     sonra özyinelemeli adım. Bu, sonsuz döngüye karşı da ilk korkuluktur."
                        .into(),
                ));
            }
        };
        let kayit = &mut baglam.denetim_yigini[indeks];
        match kayit.verilen_ozyineleme {
            None => kayit.verilen_ozyineleme = Some(tahmin),
            Some(onceki) if onceki != tahmin => {
                return Err(Tani::yeni(
                    "T018",
                    format!("\"{}\" özyinelemeli kullanımları farklı türlere çıkıyor.", ad),
                    satir,
                    1,
                    1,
                ));
            }
            _ => {}
        }
        return Ok(Some(tahmin));
    }

    let mut islem = baglam.islemler.remove(ad).ok_or_else(|| {
        Tani::yeni(
            "T016",
            format!(
                "\"{}\" işleminin kaydı bulunamadı — derleyici iç hatası olabilir, bildir.",
                ad
            ),
            satir,
            1,
            1,
        )
    })?;

    if islem.parametreler.len() != arg_turleri.len() {
        let beklenen = islem.parametreler.len();
        baglam.islemler.insert(ad.to_string(), islem);
        return Err(Tani::yeni(
            "T015",
            format!(
                "\"{}\" {} parametre bekler, {} argüman verildi.",
                ad,
                beklenen,
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

    baglam.denetim_yigini.push(ImzaKaydi {
        ad: ad.to_string(),
        parametre_turleri: arg_turleri.to_vec(),
        donusler: Vec::new(),
        verilen_ozyineleme: None,
    });
    let denetim = blok_denetle(&mut islem.govde, &mut islem_ortami, baglam);
    let kayit = baglam.denetim_yigini.pop().expect("kayıt az önce eklendi");
    // Gövde her durumda kayda geri konur; hata olsa bile kayıt tutarlı kalır.
    if denetim.is_ok() && kayit.donusler.contains(&Tur::Hata) {
        donusleri_sarmala(&mut islem.govde);
    }
    baglam.islemler.insert(ad.to_string(), islem);
    denetim?;

    let donus = donusleri_birlestir(ad, &kayit.donusler, satir)?;

    // Özyinelemeli kullanıma verilen tür, son birleşimle çelişmemeli
    // (örn. temel durum TamSayı verip sonradan "yok döndür" eklemek).
    if let Some(verilen) = kayit.verilen_ozyineleme {
        if donus != Some(verilen) {
            return Err(Tani::yeni(
                "T018",
                format!(
                    "\"{}\" işleminde özyinelemeli kullanım {} sayılmıştı ama dönüş birleşimi {} çıktı.",
                    ad,
                    verilen.adi(),
                    donus.map(|d| d.adi()).unwrap_or_else(|| "dönüşsüz".into())
                ),
                satir,
                1,
                1,
            )
            .onerili(
                "yok/hatasını döndür dallarını özyinelemeli adımdan ÖNCE, temel durumla \
                 birlikte en üste yaz."
                    .into(),
            ));
        }
    }

    baglam.imzalar.insert(
        ad.to_string(),
        Imza { parametre_turleri: arg_turleri.to_vec(), donus },
    );
    Ok(donus)
}

/// Dönüş dallarını tek türe birleştirir: {T}→T; {T,Yok}→Seçenek<T>;
/// {T,Hata}→Sonuç<T>; boş→None; tutarsızlık→T018.
fn donusleri_birlestir(ad: &str, donusler: &[Tur], satir: usize) -> Result<Option<Tur>, Tani> {
    let mut ayrik: Vec<Tur> = Vec::new();
    for t in donusler {
        if !ayrik.contains(t) {
            ayrik.push(*t);
        }
    }

    if ayrik.contains(&Tur::Hata) {
        let degerler: Vec<Tur> = ayrik.iter().copied().filter(|t| *t != Tur::Hata).collect();
        return match degerler.as_slice() {
            [] => Err(Tani::yeni(
                "T018",
                format!("\"{}\" yalnız hata döndürüyor; en az bir dalda değer döndür.", ad),
                satir,
                1,
                1,
            )),
            [tek] => match tek.veri_turu() {
                Some(veri) => Ok(Some(Tur::Sonuc(veri))),
                None => Err(Tani::yeni(
                    "T018",
                    format!("\"{}\" Sonuç içinde {} taşıyamaz (v0).", ad, tek.adi()),
                    satir,
                    1,
                    1,
                )),
            },
            _ => Err(Tani::yeni(
                "T018",
                format!(
                    "\"{}\" hata ile birlikte birden çok değer türü döndürüyor; tek tür seç.",
                    ad
                ),
                satir,
                1,
                1,
            )),
        };
    }

    match ayrik.as_slice() {
        [] => Ok(None),
        [tek] => {
            if *tek == Tur::Yok {
                return Err(Tani::yeni(
                    "T018",
                    format!(
                        "\"{}\" yalnız \"yok\" döndürüyor; Seçenek'in içi belirlenemiyor.",
                        ad
                    ),
                    satir,
                    1,
                    1,
                )
                .onerili("En az bir dalda gerçek bir değer döndür.".into()));
            }
            Ok(Some(*tek))
        }
        [a, b] if *a == Tur::Yok || *b == Tur::Yok => {
            let dolu = if *a == Tur::Yok { *b } else { *a };
            match dolu.veri_turu() {
                Some(veri) => Ok(Some(Tur::Secenek(veri))),
                None => Err(Tani::yeni(
                    "T018",
                    format!("\"{}\" Seçenek içinde {} taşıyamaz (v0).", ad, dolu.adi()),
                    satir,
                    1,
                    1,
                )),
            }
        }
        _ => Err(Tani::yeni(
            "T018",
            format!("\"{}\" farklı türlerde değerler döndürüyor; tek tür seç.", ad),
            satir,
            1,
            1,
        )),
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


// ---------- İleri morfoloji: ek uydurma (K-072, yeniden adlandırma) ----------

/// Soyut ek türü — çözümlenen yüzey ekinin dilbilgisel kimliği.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SoyutEk {
    Belirtme,
    Tamlayan,
    Yonelme,
    Ayrilma,
    Bulunma,
    Arac,
    CogulYonelme,
}

/// Kelimeyi verilen köke göre çözer: hangi soyut ek? (Yalnız tek katman;
/// yumuşama/ikizleşme/ünlü düşmesi geri çevrimleri hesaba katılır.)
pub(crate) fn ek_coz(kelime: &str, kok: &str) -> Option<SoyutEk> {
    if kelime == kok {
        return None;
    }
    let dene = |ekler: &[&str], tur: SoyutEk| -> Option<SoyutEk> {
        for ek in ekler {
            if let Some(govde) = kelime.strip_suffix(ek) {
                if govde == kok {
                    return Some(tur);
                }
                // Yumuşama: kitabı → kitab ≠ kitap; sertleştir ve kıyasla.
                let harfler: Vec<char> = govde.chars().collect();
                if let Some(&son) = harfler.last() {
                    let sert = match son {
                        'b' => Some('p'),
                        'c' => Some('ç'),
                        'd' => Some('t'),
                        'ğ' => Some('k'),
                        _ => None,
                    };
                    if let Some(sert) = sert {
                        let mut aday: Vec<char> = harfler.clone();
                        *aday.last_mut().unwrap() = sert;
                        let aday: String = aday.iter().collect();
                        if aday == kok {
                            return Some(tur);
                        }
                        // nk→ng özel durumu: rengi → reng → renk.
                        if son == 'g' {
                        }
                    }
                    if son == 'g' {
                        let mut aday: Vec<char> = harfler.clone();
                        *aday.last_mut().unwrap() = 'k';
                        let aday: String = aday.iter().collect();
                        if aday == kok {
                            return Some(tur);
                        }
                    }
                    // İkizleşme: üssü → üss → üs.
                    let n = harfler.len();
                    if n >= 2 && harfler[n - 1] == harfler[n - 2] {
                        let aday: String = harfler[..n - 1].iter().collect();
                        if aday == kok {
                            return Some(tur);
                        }
                    }
                }
                // Ünlü düşmesi: şekli → şekl → şekil.
                let unlu = |k: char| "aeıioöuü".contains(k);
                let n = harfler.len();
                if n >= 2 && !unlu(harfler[n - 1]) && !unlu(harfler[n - 2]) {
                    if let Some(&onceki) = harfler[..n - 1].iter().rev().find(|&&k| unlu(k)) {
                        let dar = match onceki {
                            'a' | 'ı' => 'ı',
                            'e' | 'i' => 'i',
                            'o' | 'u' => 'u',
                            _ => 'ü',
                        };
                        let mut aday = harfler.clone();
                        aday.insert(n - 1, dar);
                        let aday: String = aday.into_iter().collect();
                        if aday == kok {
                            return Some(tur);
                        }
                    }
                }
            }
        }
        None
    };
    dene(&["lara", "lere"], SoyutEk::CogulYonelme)
        .or_else(|| dene(&["nın", "nin", "nun", "nün", "ın", "in", "un", "ün"], SoyutEk::Tamlayan))
        .or_else(|| dene(&["yla", "yle", "la", "le"], SoyutEk::Arac))
        .or_else(|| dene(&["dan", "den", "tan", "ten"], SoyutEk::Ayrilma))
        .or_else(|| dene(&["da", "de", "ta", "te"], SoyutEk::Bulunma))
        .or_else(|| dene(&["yı", "yi", "yu", "yü", "ı", "i", "u", "ü"], SoyutEk::Belirtme))
        .or_else(|| dene(&["ya", "ye", "a", "e"], SoyutEk::Yonelme))
}

/// Yeni köke soyut eki Türkçe uyum kurallarıyla giydirir (K-072).
/// Yumuşama: çok heceli p/ç/t/k + ünlüyle başlayan ek (renk→rengi özel).
pub(crate) fn ek_uydur(kok: &str, ek: SoyutEk) -> String {
    let harfler: Vec<char> = kok.chars().collect();
    let unlu = |k: char| "aeıioöuüAEIİOÖUÜ".contains(k);
    let son_unlu = harfler.iter().rev().find(|&&k| unlu(k)).copied().unwrap_or('e');
    let dar = match son_unlu {
        'a' | 'ı' | 'A' | 'I' => 'ı',
        'e' | 'i' | 'E' | 'İ' => 'i',
        'o' | 'u' | 'O' | 'U' => 'u',
        _ => 'ü',
    };
    let genis = matches!(son_unlu, 'a' | 'ı' | 'o' | 'u' | 'A' | 'I' | 'O' | 'U');
    let son = *harfler.last().unwrap_or(&'e');
    let unluyle_biter = unlu(son);
    let hece = harfler.iter().filter(|&&k| unlu(k)).count();
    let sert_unsuz = "fstkçşhpFSTKÇŞHP".contains(son);

    // Ünlüyle başlayan ek gelirken yumuşama (çok hecelide).
    let govde_yumusat = |ek_unluyle_baslar: bool| -> String {
        if !ek_unluyle_baslar || unluyle_biter {
            return kok.to_string();
        }
        // nk→ng her hecede güvenilir (renk→rengi); diğer yumuşamalar çok hecede.
        let nk = harfler.len() >= 2
            && harfler[harfler.len() - 1] == 'k'
            && harfler[harfler.len() - 2] == 'n';
        if hece < 2 && !nk {
            return kok.to_string();
        }
        let mut h = harfler.clone();
        let n = h.len();
        match h[n - 1] {
            'p' => h[n - 1] = 'b',
            'ç' => h[n - 1] = 'c',
            't' => h[n - 1] = 'd',
            'k' => {
                if n >= 2 && h[n - 2] == 'n' {
                    h[n - 1] = 'g';
                } else {
                    h[n - 1] = 'ğ';
                }
            }
            _ => {}
        }
        h.into_iter().collect()
    };

    match ek {
        SoyutEk::Belirtme => {
            if unluyle_biter {
                format!("{}y{}", kok, dar)
            } else {
                format!("{}{}", govde_yumusat(true), dar)
            }
        }
        SoyutEk::Tamlayan => {
            if unluyle_biter {
                format!("{}n{}n", kok, dar)
            } else {
                format!("{}{}n", govde_yumusat(true), dar)
            }
        }
        SoyutEk::Yonelme => {
            let sesli = if genis { 'a' } else { 'e' };
            if unluyle_biter {
                format!("{}y{}", kok, sesli)
            } else {
                format!("{}{}", govde_yumusat(true), sesli)
            }
        }
        SoyutEk::Ayrilma => {
            let sesli = if genis { 'a' } else { 'e' };
            let baslangic = if sert_unsuz { 't' } else { 'd' };
            format!("{}{}{}n", kok, baslangic, sesli)
        }
        SoyutEk::Bulunma => {
            let sesli = if genis { 'a' } else { 'e' };
            let baslangic = if sert_unsuz { 't' } else { 'd' };
            format!("{}{}{}", kok, baslangic, sesli)
        }
        SoyutEk::Arac => {
            let sesli = if genis { 'a' } else { 'e' };
            if unluyle_biter {
                format!("{}yl{}", kok, sesli)
            } else {
                format!("{}l{}", kok, sesli)
            }
        }
        SoyutEk::CogulYonelme => {
            let sesli = if genis { 'a' } else { 'e' };
            format!("{}l{}r{}", kok, sesli, sesli)
        }
    }
}

/// Ek ayıklama adayları: yaygın hal/iyelik/araç ekleri + ünsüz yumuşaması geri çevrimi.
/// (K-011: desteklenen ek listesi sürümlemeli grammar'ın parçasıdır.)
pub(crate) fn kok_adaylari(ham: &str) -> Vec<String> {
    // K-061: iki katmanlı ek zinciri (iyelik + hâl/araç): "fiyatıyla" →
    // fiyatı → fiyat. Birinci katman adayları ikinci turdan da geçer;
    // belirsizlik güvenliği değişmez (çözüm kapsam/alan eşleşmesine bakar,
    // çoklu eşleşme A002/T028'dir).
    let birinci = kok_adaylari_tek_katman(ham);
    let mut hepsi = birinci.clone();
    for aday in &birinci {
        for ikinci in kok_adaylari_tek_katman(aday) {
            if !hepsi.contains(&ikinci) {
                hepsi.push(ikinci);
            }
        }
    }
    hepsi
}

fn kok_adaylari_tek_katman(ham: &str) -> Vec<String> {
    const EKLER: [&str; 44] = [
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
        // Zamir n'si (K-041): iyelikli kökten sonra hâl eki n ile bağlanır —
        // zarı+ndan, zarı+nda, zarı+na, zarı+nı (bilgisayarın_zarından).
        "ndan", "nden", "nda", "nde", // ayrılma/bulunma (iyelikli kök)
        "na", "ne", // yönelme (iyelikli kök)
        "nı", "ni", "nu", "nü", // belirtme (iyelikli kök)
    ];

    let mut adaylar = Vec::new();
    for ek in EKLER {
        if let Some(kok) = ham.strip_suffix(ek) {
            if kok.chars().count() < 2 {
                continue;
            }
            adaylar.push(kok.to_string());

            let karakterler: Vec<char> = kok.chars().collect();

            // Ünsüz yumuşaması geri çevrimi: sayacı→sayac→sayaç, kitabı→kitab→kitap,
            // yurdu→yurd→yurt, çocuğu→çocuğ→çocuk.
            if let Some(son) = karakterler.last().copied() {
                let sertlesmis = match son {
                    'c' => Some('ç'),
                    'b' => Some('p'),
                    'd' => Some('t'),
                    'ğ' => Some('k'),
                    _ => None,
                };
                if let Some(yeni) = sertlesmis {
                    let mut aday = karakterler.clone();
                    *aday.last_mut().unwrap() = yeni;
                    adaylar.push(aday.into_iter().collect());
                }
            }

            // Ünsüz ikizleşmesi geri çevrimi (K-049): üssü→üss→üs, affı→aff→af,
            // zammı→zamm→zam. Sertleşmeyle birleşik: reddi→redd→red→ret,
            // tıbbı→tıbb→tıb→tıp.
            let n_ikiz = karakterler.len();
            if n_ikiz >= 3 {
                let son = karakterler[n_ikiz - 1];
                let unlu = |k: char| "aeıioöuüAEIİOÖUÜ".contains(k);
                if son == karakterler[n_ikiz - 2] && !unlu(son) {
                    let tekli: String = karakterler[..n_ikiz - 1].iter().collect();
                    adaylar.push(tekli.clone());
                    let sertlesmis = match son {
                        'c' => Some('ç'),
                        'b' => Some('p'),
                        'd' => Some('t'),
                        'ğ' => Some('k'),
                        _ => None,
                    };
                    if let Some(yeni) = sertlesmis {
                        let mut sert: Vec<char> = tekli.chars().collect();
                        *sert.last_mut().unwrap() = yeni;
                        adaylar.push(sert.into_iter().collect());
                    }
                }
            }

            // Ünlü düşmesi geri çevrimi: şekle→şekl→şekil, burnu→burn→burun,
            // oğlu→oğl→oğul. Son iki harf ünsüzse araya uyumlu dar ünlü girer.
            let n = karakterler.len();
            if n >= 3 {
                let unlu = |k: char| "aeıioöuüAEIİOÖUÜ".contains(k);
                if !unlu(karakterler[n - 1]) && !unlu(karakterler[n - 2]) {
                    if let Some(&onceki_unlu) =
                        karakterler[..n - 2].iter().rev().find(|&&k| unlu(k))
                    {
                        let dar = match onceki_unlu {
                            'a' | 'ı' => 'ı',
                            'e' | 'i' => 'i',
                            'o' | 'u' => 'u',
                            'ö' | 'ü' => 'ü',
                            _ => 'i',
                        };
                        let mut aday = karakterler.clone();
                        aday.insert(n - 1, dar);
                        adaylar.push(aday.into_iter().collect());
                    }
                }
            }
        }
    }
    adaylar
}
