//! K-418 (RFC-0044 B1b, ADR-177): `kamu_son_ekleri` için diferansiyel kâhin
//! (yeni bağımlılık yok).
//!
//! Güvenilmez girdi, e-posta başlığından gelen `<ad> alanının kurumsal
//! alanı` adıdır; DMARC hizalaması bu sonuca dayanır. Kâhin iki katmandır:
//!
//! - **Referans uygulama.** publicsuffix.org "en uzun eşleşen kural"
//!   algoritmasının kural listesi üzerinde etiket etiket yürüyen naif
//!   yazımı (küme/birleştirme yok). Ayrıştırma kabul/ret, kural sayısı,
//!   yok sayılan satır ve her ad için kurumsal alan üretim kodunun
//!   sonucuyla birebir eşit olmalıdır. Joker `*.x` adın kendisi `x` ise
//!   `x`'i kamu son eki sayar (modülün belgelenmiş seçimi; publicsuffix.org
//!   `mm` → null vektörü).
//! - **Sözleşme değişmezleri (spec/31 §2).** Sonuç ya boştur ya da
//!   küçük harfli, sonda noktasız, olağanlaştırılmış adın nokta sınırında
//!   biten sonekidir; geçersiz ad boş verir; kurumsal alanın kurumsal alanı
//!   kendisidir; büyük/küçük harf ve tek son nokta sonucu değiştirmez.
//!
//! Girdi biçimi: ilk bayt bayraktır. Tek ise kalan baytlar ilk `0x00`'a
//! kadar kural metni, sonrası adlardır; çift ise kural listesi aşağıdaki
//! publicsuffix.org alt kümesidir ve kalan baytların tamamı adlardır. Adlar
//! `\n` ile ayrılır (en çok [`AZAMI_AD`]); geçersiz UTF-8 kayıplı çevrilir
//! (U-label ve denetim karakteri yolu sınanır).
//!
//! Aynı dosya `tests/kamu_son_ekleri_fuzz_testi.rs` ve
//! `fuzz/fuzz_targets/psl.rs` tarafından `#[path]` ile kullanılır.

use dil::kamu_son_ekleri::{KamuSonEkleri, AZAMI_BAYT, AZAMI_KURAL};

/// Girdi başına sınanan en çok ad.
pub const AZAMI_AD: usize = 16;

/// publicsuffix.org `tests.txt` vakalarının ihtiyaç duyduğu kurallar
/// (`kamu_son_ekleri.rs` birim testleriyle aynı alt küme; bir ASCII dışı
/// satır yok sayılır).
pub const ORNEK_LISTE: &str = "\
// ===BEGIN ICANN DOMAINS===
com
biz
uk.com
ac
jp
ac.jp
kyoto.jp
ide.kyoto.jp
*.kobe.jp
!city.kobe.jp
*.ck
!www.ck
us
ak.us
k12.ak.us
*.mm
cn
com.cn
公司.cn
xn--55qx5d.cn
tr
com.tr
// ===END ICANN DOMAINS===
// ===BEGIN PRIVATE DOMAINS===
blogspot.com
// ===END PRIVATE DOMAINS===
";

pub fn denetle(baytlar: &[u8]) {
    let Some((&bayrak, kalan)) = baytlar.split_first() else {
        return;
    };
    let (kural_metni, adlar): (&[u8], &[u8]) = if bayrak & 1 == 1 {
        match kalan.iter().position(|&b| b == 0) {
            Some(i) => (&kalan[..i], &kalan[i + 1..]),
            None => (kalan, &[]),
        }
    } else {
        (ORNEK_LISTE.as_bytes(), kalan)
    };

    let uretim = KamuSonEkleri::ayristir(kural_metni);
    let referans = Referans::ayristir(kural_metni);
    let (liste, referans) = match (uretim, referans) {
        (Ok(l), Some(r)) => (l, r),
        (Err(_), None) => return,
        (u, r) => panic!(
            "ayrıştırma kabulü ayrıştı: üretim={:?} referans={}",
            u.map(|l| l.kural_sayisi()),
            r.is_some()
        ),
    };
    assert_eq!(
        liste.kural_sayisi(),
        referans.kural_sayisi(),
        "kural sayısı"
    );
    assert!((1..=AZAMI_KURAL).contains(&liste.kural_sayisi()));
    assert_eq!(liste.yok_sayilan(), referans.yok_sayilan, "yok sayılan");

    for ham in adlar.split(|&b| b == b'\n').take(AZAMI_AD) {
        let ad = String::from_utf8_lossy(ham);
        ad_denetle(&liste, &referans, &ad);
    }
}

fn ad_denetle(liste: &KamuSonEkleri, referans: &Referans, ad: &str) {
    let sonuc = liste.kurumsal_alan(ad);
    assert_eq!(sonuc, referans.kurumsal_alan(ad), "referans: {ad:?}");

    let olagan = olagan_ad(ad);
    match &olagan {
        None => assert_eq!(sonuc, "", "geçersiz ad boş vermeli: {ad:?}"),
        Some(o) if !sonuc.is_empty() => {
            assert!(
                o == &sonuc || o.ends_with(&format!(".{sonuc}")),
                "sonuç adın soneki değil: {ad:?} → {sonuc:?}"
            );
        }
        Some(_) => {}
    }
    if !sonuc.is_empty() {
        assert!(
            !sonuc.ends_with('.') && !sonuc.starts_with('.'),
            "{sonuc:?}"
        );
        assert!(!sonuc.bytes().any(|b| b.is_ascii_uppercase()), "{sonuc:?}");
        assert_eq!(liste.kurumsal_alan(&sonuc), sonuc, "eşgüçlülük: {ad:?}");
    }
    // Büyük/küçük harf ve tek son nokta sonucu değiştirmez.
    assert_eq!(
        liste.kurumsal_alan(&ad.to_ascii_uppercase()),
        sonuc,
        "harf: {ad:?}"
    );
    if !ad.ends_with('.') {
        assert_eq!(
            liste.kurumsal_alan(&format!("{ad}.")),
            sonuc,
            "son nokta: {ad:?}"
        );
    }
}

/// spec/31 §2 ad kuralı, üretim kodundan bağımsız yazım: tek son nokta
/// atılır; 1–253 oktet ASCII; her etiket 1–63 oktet `[a-z0-9_-]`.
fn olagan_ad(ad: &str) -> Option<String> {
    let ad = ad.strip_suffix('.').unwrap_or(ad);
    let bayt = ad.as_bytes();
    if bayt.is_empty() || bayt.len() > 253 {
        return None;
    }
    let mut kucuk = String::with_capacity(bayt.len());
    let mut etiket = 0usize;
    for &b in bayt {
        match b {
            b'.' => {
                if etiket == 0 {
                    return None;
                }
                etiket = 0;
            }
            b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => etiket += 1,
            b'A'..=b'Z' => etiket += 1,
            _ => return None,
        }
        if etiket > 63 {
            return None;
        }
        kucuk.push(char::from(b.to_ascii_lowercase()));
    }
    (etiket > 0).then_some(kucuk)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tur {
    Olagan,
    Joker,
    Istisna,
}

/// Naif referans: kurallar sıralı listede, eşleşme etiket etiket.
struct Referans {
    kurallar: Vec<(Tur, Vec<String>)>,
    yok_sayilan: usize,
}

impl Referans {
    fn ayristir(icerik: &[u8]) -> Option<Self> {
        if icerik.len() > AZAMI_BAYT {
            return None;
        }
        let metin = std::str::from_utf8(icerik).ok()?;
        let mut kurallar: Vec<(Tur, Vec<String>)> = Vec::new();
        let mut yok_sayilan = 0;
        for satir in metin.lines() {
            let Some(belirtec) = satir.split_whitespace().next() else {
                continue;
            };
            if belirtec.starts_with("//") {
                continue;
            }
            if belirtec.bytes().any(|b| b >= 0x80) {
                yok_sayilan += 1;
                continue;
            }
            let belirtec = belirtec.to_ascii_lowercase();
            let (tur, govde) = if let Some(g) = belirtec.strip_prefix('!') {
                (Tur::Istisna, g)
            } else if let Some(g) = belirtec.strip_prefix("*.") {
                (Tur::Joker, g)
            } else {
                (Tur::Olagan, belirtec.as_str())
            };
            if govde.len() > 253 {
                return None;
            }
            let etiketler: Vec<String> = govde.split('.').map(str::to_string).collect();
            let gecerli = etiketler.iter().all(|e| {
                (1..=63).contains(&e.len())
                    && e.bytes()
                        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
            });
            if !gecerli {
                return None;
            }
            if !kurallar.iter().any(|(t, e)| *t == tur && *e == etiketler) {
                kurallar.push((tur, etiketler));
            }
        }
        (!kurallar.is_empty()).then_some(Self {
            kurallar,
            yok_sayilan,
        })
    }

    fn kural_sayisi(&self) -> usize {
        self.kurallar.len()
    }

    fn kurumsal_alan(&self, ad: &str) -> String {
        let Some(ad) = olagan_ad(ad) else {
            return String::new();
        };
        let etiketler: Vec<&str> = ad.split('.').collect();
        let n = etiketler.len();
        let sonek_mi = |kural: &[String]| {
            kural.len() <= n
                && kural
                    .iter()
                    .rev()
                    .zip(etiketler.iter().rev())
                    .all(|(k, e)| k == e)
        };
        // İstisna her zaman kazanır; birden çoksa en uzunu.
        let istisna = self
            .kurallar
            .iter()
            .filter(|(t, k)| *t == Tur::Istisna && sonek_mi(k))
            .map(|(_, k)| k.len())
            .max();
        let son_ek = match istisna {
            Some(m) => m - 1,
            None => {
                let mut en_uzun = 1; // varsayılan `*`
                for (tur, kural) in &self.kurallar {
                    if !sonek_mi(kural) {
                        continue;
                    }
                    let m = kural.len();
                    let uzunluk = match tur {
                        Tur::Olagan => m,
                        Tur::Joker if n > m => m + 1,
                        Tur::Joker => m,
                        Tur::Istisna => continue,
                    };
                    en_uzun = en_uzun.max(uzunluk);
                }
                en_uzun
            }
        };
        if n <= son_ek {
            return String::new();
        }
        etiketler[n - son_ek - 1..].join(".")
    }
}
