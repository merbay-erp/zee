//! Ad çözümleme + tür denetimi.
//!
//! Ad çözümleme, K-011'deki kuralı uygular: tanımlayıcılara ekler bitişik
//! yazılır ("sayıyı", "toplamı", "sayacı"). Çözüm TAHMİNLE DEĞİL, aday kök
//! üretip kapsamdaki tanımlı adlarla eşleyerek yapılır — birden çok aday
//! eşleşirse bu bir hatadır (determinizm, manifesto 3).
//!
//! Ünsüz yumuşamasının geri çevrimi desteklenir: "sayacı" → "sayac" → "sayaç".

mod cagri;
mod cumle;
mod ifade;

use self::cagri::cagri_denetle;
use self::cumle::blok_denetle;
use self::ifade::ifade_denetle;

use crate::agac::{Cumle, Ifade, Islec, Islem, Ozellik, Program, Yapi};
use crate::intrinsic::{self, IntrinsicTuru};

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
    /// Yapılandırılmış beklenen hata değeri (K-091).
    Hata,
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
            VeriTuru::Hata => "Hata",
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
            VeriTuru::Hata => Tur::Hata,
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
    /// Sonuç<değer, Hata>: değer türü parametreli, hata yapılandırılmıştır.
    Sonuc(VeriTuru),
    /// Yalnız "yok" sabitinin türü; dönüş birleşiminde Seçenek'e erir.
    Yok,
    /// Kullanıcının erişebildiği yapılandırılmış beklenen hata değeri (K-091).
    Hata,
    /// Yalnız "hatasını döndür"ün iç işareti; birleşimde Sonuç'a erir.
    HataDonusu,
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
            Tur::Hata => "Hata".into(),
            Tur::HataDonusu => "hata dönüşü".into(),
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
            Tur::Hata => Some(VeriTuru::Hata),
            _ => None,
        }
    }

    /// TamSayı ya da Ondalık mı? (Karışımda TamSayı, Ondalık'a kayıpsız genişler.)
    fn sayisal(&self) -> bool {
        matches!(self, Tur::TamSayi | Tur::Ondalik)
    }
}

fn intrinsic_turunu_cevir(tur: IntrinsicTuru) -> Tur {
    match tur {
        IntrinsicTuru::Metin => Tur::Metin,
        IntrinsicTuru::Mantiksal => Tur::Mantiksal,
        IntrinsicTuru::AgYaniti => Tur::AgYaniti,
    }
}

/// Çoklu denetim (RFC-0010 §3.1): üst düzey cümle başına hata toplanır;
/// bir cümlenin hatası sonrakilerin denetimini durdurmaz. LSP/denetle --json
/// bu görünümü kullanır; derleme (çalıştır) ilk tanıda durur.
pub fn denetle_coklu(program: &mut Program) -> Vec<Tani> {
    let mut tanilar = Vec::new();
    if let Err(tani) = crate::eylem::denetle(program) {
        tanilar.push(tani);
    }
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
        gezilen_koleksiyonlar: std::collections::HashSet::new(),
    };
    if let Err(tani) = acik_islemleri_denetle(&mut baglam) {
        tanilar.push(tani);
        program.islemler = baglam.islemler;
        return tanilar;
    }
    let mut bas = 0;
    while bas < program.cumleler.len() {
        // Çoklu tanı geçişi normalde cümle cümle toparlanır. Görev bildirimi
        // ile join ise tek sözcüksel kanıttır; ikisini aynı dilimle denetle.
        let mut son = bas + 1;
        if matches!(program.cumleler[bas], Cumle::Eszamanli { .. }) {
            while son < program.cumleler.len() {
                let join = matches!(program.cumleler[son], Cumle::HepsiniBekle { .. });
                son += 1;
                if join {
                    break;
                }
            }
        }
        let onceki_bekleyenler = baglam.bekleyen_gorevler.clone();
        if let Err(tani) =
            blok_denetle(&mut program.cumleler[bas..son], &mut ortam, &mut baglam)
        {
            tanilar.push(tani);
            baglam.bekleyen_gorevler = onceki_bekleyenler;
            if tanilar.len() >= 20 {
                break;
            }
        }
        bas = son;
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
/// Çıkarımlı işlemler ilk çağrı argümanlarıyla; açık imzalı işlemler ise
/// tanım sözleşmesiyle çağrı beklemeden denetlenir (K-083).
pub fn denetle(program: &mut Program) -> Result<(), Tani> {
    crate::eylem::denetle(program)?;
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
        gezilen_koleksiyonlar: std::collections::HashSet::new(),
    };
    let mut sonuc = acik_islemleri_denetle(&mut baglam);
    if sonuc.is_ok() {
        sonuc = blok_denetle(&mut program.cumleler, &mut ortam, &mut baglam);
    }

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

/// Çıkarımlı ya da açık işlem imzası. Yalnız çıkarımlı imza sayısal
/// genişlemeyle değişebilir.
struct Imza {
    parametre_turleri: Vec<Tur>,
    donus: Option<Tur>,
    /// K-083 açık parametre sözleşmesi çağrılarla terfi ettirilemez.
    acik: bool,
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
    /// K-093: gezme boyunca biçimi sabit kalan kaynak listeler/sözlükler.
    /// Aynı kaynağı yeniden bağlama, ekleme, silme ve iç içe gezme T053'tür.
    gezilen_koleksiyonlar: std::collections::HashSet<String>,
}

struct ImzaKaydi {
    ad: String,
    parametre_turleri: Vec<Tur>,
    donusler: Vec<Tur>,
    /// Özyinelemeli kullanımlara verilen tür (son birleşimle doğrulanır).
    verilen_ozyineleme: Option<Tur>,
}

/// Açık imzalı (ve parametresiz) işlemler çağrı beklemeden denetlenir. Böylece
/// kütüphane API'sindeki gövde hatası kullanılmadığı için gizli kalamaz.
fn acik_islemleri_denetle(baglam: &mut Baglam) -> Result<(), Tani> {
    let mut adlar = baglam.islemler.keys().cloned().collect::<Vec<_>>();
    adlar.sort();
    for ad in adlar {
        let (turler, satir) = {
            let islem = baglam.islemler.get(&ad).expect("ad haritadan geldi");
            (
                acik_parametre_turleri(islem, &baglam.yapilar)?,
                islem.satir,
            )
        };
        if let Some(turler) = turler {
            cagri_denetle(&ad, &turler, baglam, satir)?;
        }
    }
    Ok(())
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

/// K-083 parametre tür yazımını çözer. Sembolik generic yerine kontrollü
/// Türkçe kullanılır: `Ondalık listesi`, `Metin sözlüğü`, `Öğrenci`.
fn parametre_turu(yazim: &str, yapilar: &[Yapi]) -> Option<Tur> {
    let basit = |ad: &str| match ad {
        "TamSayı" => Some(Tur::TamSayi),
        "Ondalık" => Some(Tur::Ondalik),
        "Metin" => Some(Tur::Metin),
        "Mantıksal" => Some(Tur::Mantiksal),
        "Tarih" => Some(Tur::Tarih),
        "Saat" => Some(Tur::Saat),
        "Süre" => Some(Tur::Sure),
        "AğYanıtı" => Some(Tur::AgYaniti),
        "Hata" => Some(Tur::Hata),
        _ => yapilar
            .iter()
            .position(|yapi| yapi.ad == ad)
            .map(Tur::Yapi),
    };
    if let Some(kok) = yazim.strip_suffix(" listesi") {
        return basit(kok)
            .and_then(|tur| veri_turu_yap(&tur))
            .map(Tur::Liste);
    }
    if let Some(kok) = yazim.strip_suffix(" sözlüğü") {
        return match basit(kok)? {
            Tur::TamSayi => Some(Tur::Sozluk(SozlukDegerTuru::TamSayi)),
            Tur::Ondalik => Some(Tur::Sozluk(SozlukDegerTuru::Ondalik)),
            Tur::Metin => Some(Tur::Sozluk(SozlukDegerTuru::Metin)),
            _ => None,
        };
    }
    if let Some(kok) = yazim.strip_suffix(" seçeneği") {
        return basit(kok)
            .and_then(|tur| veri_turu_yap(&tur))
            .map(Tur::Secenek);
    }
    if let Some(kok) = yazim.strip_suffix(" sonucu") {
        return basit(kok)
            .and_then(|tur| veri_turu_yap(&tur))
            .map(Tur::Sonuc);
    }
    basit(yazim)
}

/// K-086 dönüş bildirimi: dış `None` bildirim yok, iç `None` değer döndürmez.
fn bildirilmis_donus_turu(
    islem: &Islem,
    yapilar: &[Yapi],
) -> Result<Option<Option<Tur>>, Tani> {
    let Some(yazim) = islem.donus_turu_yazimi.as_deref() else {
        return Ok(None);
    };
    if yazim == "DeğerDöndürmez" {
        return Ok(Some(None));
    }
    parametre_turu(yazim, yapilar)
        .map(|tur| Some(Some(tur)))
        .ok_or_else(|| {
            Tani::yeni(
                "T040",
                format!(
                    "\"{}\" işleminin dönüş türü tanınmadı: \"{}\".",
                    islem.ad, yazim
                ),
                islem.donus_satiri.unwrap_or(islem.satir),
                1,
                1,
            )
            .onerili(
                "Örnekler: `TamSayı döndürür`, `Metin seçeneği döndürür`, `değer döndürmez`."
                    .into(),
            )
        })
}

/// None: bütün parametreler başlangıç yüzeyinde çıkarımlı. Some: açık imza.
fn acik_parametre_turleri(
    islem: &Islem,
    yapilar: &[Yapi],
) -> Result<Option<Vec<Tur>>, Tani> {
    if islem.parametreler.is_empty() {
        return Ok(Some(Vec::new()));
    }
    let acik_sayisi = islem
        .parametreler
        .iter()
        .filter(|parametre| parametre.tur_yazimi.is_some())
        .count();
    if acik_sayisi == 0 {
        if islem.donus_turu_yazimi.is_some() {
            return Err(Tani::yeni(
                "T037",
                format!(
                    "\"{}\" işleminde açık dönüş ile çıkarımlı parametreler karıştırılamaz.",
                    islem.ad
                ),
                islem.donus_satiri.unwrap_or(islem.satir),
                1,
                1,
            )
            .onerili(
                "Dönüş türü yazılıysa bütün parametreleri de `<ad> <Tür> olarak al` biçiminde yaz."
                    .into(),
            ));
        }
        return Ok(None);
    }
    if acik_sayisi != islem.parametreler.len() {
        let satir = islem
            .parametreler
            .iter()
            .find(|parametre| parametre.tur_yazimi.is_none())
            .map(|parametre| parametre.satir)
            .unwrap_or(islem.satir);
        return Err(Tani::yeni(
            "T037",
            format!(
                "\"{}\" işleminde açık ve çıkarımlı parametreler karıştırılamaz.",
                islem.ad
            ),
            satir,
            1,
            1,
        )
        .onerili(
            "İşlemin bütün parametrelerine tür yaz ya da başlangıç yüzeyinde hepsini çıkarımlı bırak."
                .into(),
        ));
    }
    islem
        .parametreler
        .iter()
        .map(|parametre| {
            let yazim = parametre.tur_yazimi.as_deref().expect("hepsi açık");
            parametre_turu(yazim, yapilar).ok_or_else(|| {
                Tani::yeni(
                    "T038",
                    format!(
                        "\"{}\" parametresinin türü tanınmadı: \"{}\".",
                        parametre.ad, yazim
                    ),
                    parametre.satir,
                    1,
                    1,
                )
                .onerili(
                    "Örnekler: TamSayı, Ondalık, Metin, Mantıksal, Ondalık listesi, Metin sözlüğü."
                        .into(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

/// Ham alan yazımını ("adı", "yaşı") yapı tanımındaki yalın ada çözer.
fn alan_cozumle(yapi: &Yapi, ham: &str, satir: usize) -> Result<String, Tani> {
    if yapi.alanlar.iter().any(|(a, _)| a == ham) {
        return Ok(ham.to_string());
    }
    let adaylar = crate::morfoloji::kok_adaylari(ham);
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

fn gezilen_koleksiyonu_degistirme_tanisi(ad: &str, satir: usize) -> Tani {
    Tani::yeni(
        "T053",
        format!(
            "\"{}\" gezilirken koleksiyonun kendisi değiştirilemez.",
            ad
        ),
        satir,
        1,
        1,
    )
    .onerili(
        "Öğeyi döngü adıyla güncelle; ekleme/silme gerekiyorsa değişiklikleri ayrı bir listede topla ve gezme bitince uygula."
            .into(),
    )
}

fn gezilen_hedefi_denetle(ifade: &Ifade, baglam: &Baglam, satir: usize) -> Result<(), Tani> {
    let aday = match ifade {
        Ifade::Degisken {
            cozulmus: Some(ad), ..
        } => baglam.gezilen_koleksiyonlar.get(ad).cloned(),
        Ifade::Degisken { ham, .. } => {
            let kokler = crate::morfoloji::kok_adaylari(ham);
            let mut eslesenler = baglam
                .gezilen_koleksiyonlar
                .iter()
                .filter(|ad| **ad == *ham || kokler.iter().any(|kok| kok == *ad))
                .cloned()
                .collect::<Vec<_>>();
            eslesenler.sort();
            eslesenler.into_iter().next()
        }
        _ => None,
    };
    if let Some(ad) = aday {
        return Err(gezilen_koleksiyonu_degistirme_tanisi(&ad, satir));
    }
    Ok(())
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


fn bekleyen_gorev_olmadigini_denetle(
    baglam: &Baglam,
    giriste_bekleyenler: &std::collections::HashSet<String>,
    satir: usize,
    mesaj: &str,
) -> Result<(), Tani> {
    if baglam
        .bekleyen_gorevler
        .difference(giriste_bekleyenler)
        .next()
        .is_some()
    {
        Err(gorev_kapsami_tanisi(satir, mesaj))
    } else {
        Ok(())
    }
}

fn gorev_kapsami_tanisi(satir: usize, mesaj: &str) -> Tani {
    Tani::yeni("T051", mesaj.into(), satir, 1, 1).onerili(
        "Her `eşzamanlı olarak` grubunu aynı sözcüksel kapsamda tek bir `hepsini bekle` ile kapat."
            .into(),
    )
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

fn cagri_turu_uyumlu(parametre: &Tur, arguman: &Tur) -> bool {
    parametre == arguman
        || matches!((parametre, arguman), (Tur::Ondalik, Tur::TamSayi))
        || matches!(
            (parametre, arguman),
            (
                Tur::Liste(VeriTuru::Ondalik),
                Tur::Liste(VeriTuru::TamSayi)
            )
        )
        || matches!(
            (parametre, arguman),
            (
                Tur::Sozluk(SozlukDegerTuru::Ondalik),
                Tur::Sozluk(SozlukDegerTuru::TamSayi)
            )
        )
        || matches!(
            (parametre, arguman),
            (
                Tur::Secenek(VeriTuru::Ondalik),
                Tur::Secenek(VeriTuru::TamSayi)
            )
        )
        || matches!(
            (parametre, arguman),
            (
                Tur::Sonuc(VeriTuru::Ondalik),
                Tur::Sonuc(VeriTuru::TamSayi)
            )
        )
}

/// Açık dönüş sözleşmesinde değer beklenen bir işlemin hiçbir olağan akışta
/// gövde sonuna düşmediğini muhafazakâr biçimde kanıtlar.
fn blok_kesin_sonlanir(cumleler: &[Cumle]) -> bool {
    cumleler.iter().any(cumle_kesin_sonlanir)
}

fn cumle_kesin_sonlanir(cumle: &Cumle) -> bool {
    match cumle {
        Cumle::Dondur { .. } | Cumle::HataDondur { .. } | Cumle::ProgramiBitir { .. } => true,
        Cumle::Ise { kollar, degilse, .. } => {
            !kollar.is_empty()
                && kollar.iter().all(|kol| blok_kesin_sonlanir(&kol.govde))
                && degilse.as_deref().is_some_and(blok_kesin_sonlanir)
        }
        Cumle::Gore { kollar, degilse, .. } => {
            !kollar.is_empty()
                && kollar.iter().all(|(_, govde)| blok_kesin_sonlanir(govde))
                && degilse.as_deref().is_some_and(blok_kesin_sonlanir)
        }
        Cumle::IcindeBlogu { govde, yetismezse, .. } => {
            blok_kesin_sonlanir(govde)
                && yetismezse.as_deref().is_some_and(blok_kesin_sonlanir)
        }
        _ => false,
    }
}

/// Dönüş dallarını tek türe birleştirir: {T}→T; {T,Yok}→Seçenek<T>;
/// {T,HataDonusu}→Sonuç<T>; boş→None; tutarsızlık→T018.
fn donusleri_birlestir(ad: &str, donusler: &[Tur], satir: usize) -> Result<Option<Tur>, Tani> {
    let mut ayrik: Vec<Tur> = Vec::new();
    for t in donusler {
        if !ayrik.contains(t) {
            ayrik.push(*t);
        }
    }

    if ayrik.contains(&Tur::HataDonusu) {
        let degerler: Vec<Tur> = ayrik
            .iter()
            .copied()
            .filter(|t| *t != Tur::HataDonusu)
            .collect();
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

    let adaylar = crate::morfoloji::kok_adaylari(ham);
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
