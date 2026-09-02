//! Zee'nin sürümlü Türkçe tanımlayıcı morfolojisi.
//!
//! Bu modül çözümleyici ile editör araçlarının ortak, tek kaynak-of-truth
//! tablosudur. Profil değişikliği kaynak anlamını değiştirebildiği için yeni
//! bir profil adı ve ana sürüm sınırı olmadan mevcut tablo değiştirilemez.

mod uyumluluk;

pub use uyumluluk::profil_uyumluluk_kaydi;

/// Zee v1 kaynaklarının sabitlediği morfoloji profili.
pub const MORFOLOJI_PROFILI: &str = "zee-tr-1";

/// Profilin makine tarafından okunabilen sayısal sürümü.
pub const MORFOLOJI_SURUMU: u16 = 1;

/// v1'de bir tanımlayıcıda çözülen en derin ek zinciri.
pub const MAKSIMUM_EK_KATMANI: usize = 2;

/// Yüzeyde farklı ünlü/tamponlarla yazılabilen soyut ek kimliği.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SoyutEk {
    Belirtme,
    Iyelik,
    Tamlayan,
    Yonelme,
    Ayrilma,
    Bulunma,
    Arac,
    CogulYonelme,
}

impl SoyutEk {
    pub const fn adi(self) -> &'static str {
        match self {
            Self::Belirtme => "belirtme",
            Self::Iyelik => "iyelik",
            Self::Tamlayan => "tamlayan",
            Self::Yonelme => "yönelme",
            Self::Ayrilma => "ayrılma",
            Self::Bulunma => "bulunma",
            Self::Arac => "araç",
            Self::CogulYonelme => "çoğul+yönelme",
        }
    }
}

/// Bir soyut ekin v1'de kabul edilen bütün yüzey biçimleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EkTanimi {
    pub ek: SoyutEk,
    pub yuzeyler: &'static [&'static str],
}

const BELIRTME: &[&str] = &[
    "yı", "yi", "yu", "yü", "nı", "ni", "nu", "nü", "ı", "i", "u", "ü",
];
const IYELIK: &[&str] = &["sı", "si", "su", "sü", "ı", "i", "u", "ü"];
const TAMLAYAN: &[&str] = &["nın", "nin", "nun", "nün", "ın", "in", "un", "ün"];
const YONELME: &[&str] = &["ya", "ye", "na", "ne", "a", "e"];
const AYRILMA: &[&str] = &["ndan", "nden", "dan", "den", "tan", "ten"];
const BULUNMA: &[&str] = &["nda", "nde", "da", "de", "ta", "te"];
const ARAC: &[&str] = &["yla", "yle", "la", "le"];
const COGUL_YONELME: &[&str] = &["lara", "lere"];

/// Tablo ve yüzey sırası da profil snapshot'ının parçasıdır; bütün adaylar korunur.
pub const EK_TABLOSU: &[EkTanimi] = &[
    EkTanimi {
        ek: SoyutEk::Belirtme,
        yuzeyler: BELIRTME,
    },
    EkTanimi {
        ek: SoyutEk::Iyelik,
        yuzeyler: IYELIK,
    },
    EkTanimi {
        ek: SoyutEk::Tamlayan,
        yuzeyler: TAMLAYAN,
    },
    EkTanimi {
        ek: SoyutEk::Yonelme,
        yuzeyler: YONELME,
    },
    EkTanimi {
        ek: SoyutEk::Ayrilma,
        yuzeyler: AYRILMA,
    },
    EkTanimi {
        ek: SoyutEk::Bulunma,
        yuzeyler: BULUNMA,
    },
    EkTanimi {
        ek: SoyutEk::Arac,
        yuzeyler: ARAC,
    },
    EkTanimi {
        ek: SoyutEk::CogulYonelme,
        yuzeyler: COGUL_YONELME,
    },
];

/// Bir yüzey kelimesinin olası kök ve ek zinciri çözümü.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MorfolojiCozumu {
    pub kok: String,
    /// İçten dışa ekler: `fiyatıyla` için `[Iyelik, Arac]`.
    pub ekler: Vec<SoyutEk>,
}

/// Tek ve iki katmanlı bütün yapısal çözümleri döndürür.
///
/// Bu işlev kapsamdan bağımsızdır; birden çok sonuç olağandır. Ad çözümleyici
/// yalnız kapsamdaki köklerle eşler ve birden fazlasında A002 üretir.
pub fn cozumleri_bul(kelime: &str) -> Vec<MorfolojiCozumu> {
    // İyelik v1'de yalnız zincirin iç katmanıdır. Tek başına `fiyatı`, ad
    // kullanımında belirtme kabul edilir; `fiyatıyla` içinde iyelik olur.
    let dis_cozumler = tek_katman_cozumleri(kelime)
        .into_iter()
        .filter(|cozum| cozum.ekler[0] != SoyutEk::Iyelik)
        .collect::<Vec<_>>();
    let mut cozumler = dis_cozumler.clone();

    for dis in dis_cozumler {
        for ic in tek_katman_cozumleri(&dis.kok) {
            let ekler = vec![ic.ekler[0], dis.ekler[0]];
            if zincir_gecerli(&ekler) {
                cozum_ekle(&mut cozumler, MorfolojiCozumu { kok: ic.kok, ekler });
            }
        }
    }
    cozumler
}

/// Kapsam eşlemesinin kullandığı benzersiz kök adayları.
pub fn kok_adaylari(kelime: &str) -> Vec<String> {
    let mut adaylar = Vec::new();
    for cozum in cozumleri_bul(kelime) {
        if !adaylar.contains(&cozum.kok) {
            adaylar.push(cozum.kok);
        }
    }
    adaylar
}

/// Belirli bir köke uyan bütün ek zincirlerini döndürür.
pub fn ek_zincirleri_coz(kelime: &str, kok: &str) -> Vec<Vec<SoyutEk>> {
    let mut zincirler = Vec::new();
    for cozum in cozumleri_bul(kelime) {
        if cozum.kok == kok && !zincirler.contains(&cozum.ekler) {
            zincirler.push(cozum.ekler);
        }
    }
    zincirler
}

/// Belirli kök için çözüm tekse zinciri verir; belirsizlikte tahmin etmez.
pub fn ek_zinciri_coz(kelime: &str, kok: &str) -> Option<Vec<SoyutEk>> {
    let zincirler = ek_zincirleri_coz(kelime, kok);
    if zincirler.len() == 1 {
        zincirler.into_iter().next()
    } else {
        None
    }
}

/// Tek eki Türkçe uyum kurallarıyla köke giydirir.
pub fn ek_uydur(kok: &str, ek: SoyutEk) -> String {
    eki_uydur(kok, ek, None)
}

/// Geçerli bir v1 ek zincirinin kanonik yüzeyini üretir.
pub fn ek_zinciri_uydur(kok: &str, ekler: &[SoyutEk]) -> Option<String> {
    if !zincir_gecerli(ekler) {
        return None;
    }
    let mut yuzey = kok.to_string();
    let mut onceki = None;
    for &ek in ekler {
        yuzey = eki_uydur(&yuzey, ek, onceki);
        onceki = Some(ek);
    }
    Some(yuzey)
}

/// Profil tablosunun insan ve snapshot testleri için kararlı dökümü.
pub fn profil_dokumu() -> String {
    let mut satirlar = vec![format!(
        "profil={};sürüm={};azami_katman={}",
        MORFOLOJI_PROFILI, MORFOLOJI_SURUMU, MAKSIMUM_EK_KATMANI
    )];
    for tanim in EK_TABLOSU {
        satirlar.push(format!("{}={}", tanim.ek.adi(), tanim.yuzeyler.join(",")));
    }
    satirlar.push("zincir=iyelik+(belirtme|tamlayan|yönelme|ayrılma|bulunma|araç)".into());
    format!("{}\n", satirlar.join("\n"))
}

fn tek_katman_cozumleri(kelime: &str) -> Vec<MorfolojiCozumu> {
    let mut cozumler = Vec::new();
    for tanim in EK_TABLOSU {
        for &yuzey in tanim.yuzeyler {
            let Some(govde) = kelime.strip_suffix(yuzey) else {
                continue;
            };
            if govde.chars().count() < 2 {
                continue;
            }
            for kok in kok_bicimleri(govde) {
                cozum_ekle(
                    &mut cozumler,
                    MorfolojiCozumu {
                        kok,
                        ekler: vec![tanim.ek],
                    },
                );
            }
        }
    }
    cozumler
}

fn cozum_ekle(cozumler: &mut Vec<MorfolojiCozumu>, cozum: MorfolojiCozumu) {
    if !cozumler.contains(&cozum) {
        cozumler.push(cozum);
    }
}

fn zincir_gecerli(ekler: &[SoyutEk]) -> bool {
    match ekler {
        [_] => true,
        [SoyutEk::Iyelik, dis] => !matches!(dis, SoyutEk::Iyelik | SoyutEk::CogulYonelme),
        _ => false,
    }
}

/// Ek öncesi gövdeden doğrudan ve ses değişimi geri çevrimli kökleri çıkarır.
fn kok_bicimleri(govde: &str) -> Vec<String> {
    let mut adaylar = vec![govde.to_string()];
    let harfler: Vec<char> = govde.chars().collect();
    let unlu = |k: char| "aeıioöuüAEIİOÖUÜ".contains(k);

    if let Some(&son) = harfler.last() {
        let sert = match son {
            'b' => Some('p'),
            'c' => Some('ç'),
            'd' => Some('t'),
            'ğ' | 'g' => Some('k'),
            'B' => Some('P'),
            'C' => Some('Ç'),
            'D' => Some('T'),
            'Ğ' | 'G' => Some('K'),
            _ => None,
        };
        if let Some(sert) = sert {
            let mut aday = harfler.clone();
            if let Some(son) = aday.last_mut() {
                *son = sert;
                benzersiz_ekle(&mut adaylar, aday.into_iter().collect());
            }
        }
    }

    // üssü→üs, affı→af; ardından reddi→red→ret ve tıbbı→tıb→tıp.
    let n = harfler.len();
    if n >= 3 && harfler[n - 1] == harfler[n - 2] && !unlu(harfler[n - 1]) {
        let tekli: String = harfler[..n - 1].iter().collect();
        benzersiz_ekle(&mut adaylar, tekli.clone());
        let mut tekli_harfler: Vec<char> = tekli.chars().collect();
        let sert = match tekli_harfler.last().copied() {
            Some('b') => Some('p'),
            Some('c') => Some('ç'),
            Some('d') => Some('t'),
            Some('ğ') | Some('g') => Some('k'),
            _ => None,
        };
        if let Some(sert) = sert {
            if let Some(son) = tekli_harfler.last_mut() {
                *son = sert;
                benzersiz_ekle(&mut adaylar, tekli_harfler.into_iter().collect());
            }
        }
    }

    // şekle→şekil, burnu→burun, oğlu→oğul.
    if n >= 3 && !unlu(harfler[n - 1]) && !unlu(harfler[n - 2]) {
        if let Some(&onceki) = harfler[..n - 2].iter().rev().find(|&&k| unlu(k)) {
            let dar = dar_unlu(onceki);
            let mut aday = harfler.clone();
            aday.insert(n - 1, dar);
            benzersiz_ekle(&mut adaylar, aday.into_iter().collect());
        }
    }
    adaylar
}

fn benzersiz_ekle<T: PartialEq>(liste: &mut Vec<T>, deger: T) {
    if !liste.contains(&deger) {
        liste.push(deger);
    }
}

fn eki_uydur(kok: &str, ek: SoyutEk, onceki: Option<SoyutEk>) -> String {
    let harfler: Vec<char> = kok.chars().collect();
    let son_unlu = harfler
        .iter()
        .rev()
        .find(|&&k| unludur(k))
        .copied()
        .unwrap_or('e');
    let dar = dar_unlu(son_unlu);
    let genis = genis_unlu(son_unlu);
    let son = *harfler.last().unwrap_or(&'e');
    let unluyle_biter = unludur(son);
    let sert_unsuz = "fstkçşhpFSTKÇŞHP".contains(son);
    let iyelik_ardindan = onceki == Some(SoyutEk::Iyelik) && unluyle_biter;

    match ek {
        SoyutEk::Belirtme if iyelik_ardindan => format!("{}n{}", kok, dar),
        SoyutEk::Yonelme if iyelik_ardindan => format!("{}n{}", kok, genis),
        SoyutEk::Ayrilma if iyelik_ardindan => format!("{}nd{}n", kok, genis),
        SoyutEk::Bulunma if iyelik_ardindan => format!("{}nd{}", kok, genis),
        SoyutEk::Belirtme => {
            if unluyle_biter {
                format!("{}y{}", kok, dar)
            } else {
                format!("{}{}", govde_yumusat(kok), dar)
            }
        }
        SoyutEk::Iyelik => {
            if unluyle_biter {
                format!("{}s{}", kok, dar)
            } else {
                format!("{}{}", govde_yumusat(kok), dar)
            }
        }
        SoyutEk::Tamlayan => {
            if unluyle_biter {
                format!("{}n{}n", kok, dar)
            } else {
                format!("{}{}n", govde_yumusat(kok), dar)
            }
        }
        SoyutEk::Yonelme => {
            if unluyle_biter {
                format!("{}y{}", kok, genis)
            } else {
                format!("{}{}", govde_yumusat(kok), genis)
            }
        }
        SoyutEk::Ayrilma => {
            let bas = if sert_unsuz { 't' } else { 'd' };
            format!("{}{}{}n", kok, bas, genis)
        }
        SoyutEk::Bulunma => {
            let bas = if sert_unsuz { 't' } else { 'd' };
            format!("{}{}{}", kok, bas, genis)
        }
        SoyutEk::Arac => {
            if unluyle_biter {
                format!("{}yl{}", kok, genis)
            } else {
                format!("{}l{}", kok, genis)
            }
        }
        SoyutEk::CogulYonelme => format!("{}l{}r{}", kok, genis, genis),
    }
}

fn govde_yumusat(kok: &str) -> String {
    let mut harfler: Vec<char> = kok.chars().collect();
    let hece = harfler.iter().filter(|&&k| unludur(k)).count();
    let n = harfler.len();
    if n == 0 {
        return kok.to_string();
    }
    let nk = n >= 2 && matches!(harfler[n - 2], 'n' | 'N') && matches!(harfler[n - 1], 'k' | 'K');
    if hece < 2 && !nk {
        return kok.to_string();
    }
    harfler[n - 1] = match harfler[n - 1] {
        'p' => 'b',
        'ç' => 'c',
        't' => 'd',
        'k' if nk => 'g',
        'k' => 'ğ',
        'P' => 'B',
        'Ç' => 'C',
        'T' => 'D',
        'K' if nk => 'G',
        'K' => 'Ğ',
        baska => baska,
    };
    harfler.into_iter().collect()
}

fn unludur(k: char) -> bool {
    "aeıioöuüAEIİOÖUÜ".contains(k)
}

fn dar_unlu(k: char) -> char {
    match k {
        'a' | 'ı' | 'A' | 'I' => 'ı',
        'e' | 'i' | 'E' | 'İ' => 'i',
        'o' | 'u' | 'O' | 'U' => 'u',
        _ => 'ü',
    }
}

fn genis_unlu(k: char) -> char {
    if matches!(k, 'a' | 'ı' | 'o' | 'u' | 'A' | 'I' | 'O' | 'U') {
        'a'
    } else {
        'e'
    }
}
