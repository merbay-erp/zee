//! K-390 (RFC-0044 B1b, ADR-175): Public Suffix List ayrıştırıcısı ve
//! kurumsal alan (RFC 7489 §3.2) hesabı. Veri derleyiciye gömülü değildir;
//! proje dosyayı exact SHA-256 ile bildirir, host açılışta bir kez ayrıştırır.
//!
//! Algoritma publicsuffix.org "en uzun eşleşen kural"dır: istisna (`!`)
//! kuralı her zaman kazanır, joker (`*.`) bir etiket daha tüketir, hiçbir
//! kural eşleşmezse varsayılan `*` kuralı (tek etiket) geçerlidir. ICANN ve
//! PRIVATE bölümlerinin ikisi de kullanılır (DMARC uygulamalarının yaygın
//! seçimi). ASCII dışı (U-label) kural satırları ilk sürümde yok sayılır;
//! sorgu adı da yalnız ASCII LDH etiketleri kabul eder (spec/31 §2).
use std::collections::HashSet;

/// Kabul edilen en büyük dosya (bugünkü liste ~230 KiB).
pub const AZAMI_BAYT: usize = 4 * 1024 * 1024;
/// Kabul edilen en çok kural (bugünkü liste ~10 bin).
pub const AZAMI_KURAL: usize = 100_000;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KamuSonEkleri {
    kurallar: HashSet<String>,
    /// `*.x` kurallarının `x` kısmı.
    jokerler: HashSet<String>,
    /// `!x` kurallarının `x` kısmı.
    istisnalar: HashSet<String>,
    yok_sayilan: usize,
}

impl KamuSonEkleri {
    /// Dosya içeriğini ayrıştırır. Biçim: satır başına bir kural, `//`
    /// yorum, ilk boşluğa kadar olan kısım kuraldır. Bozuk ASCII kuralı
    /// (boş etiket, LDH dışı karakter, ortada joker) dosyayı reddeder.
    pub fn ayristir(icerik: &[u8]) -> Result<Self, String> {
        if icerik.len() > AZAMI_BAYT {
            return Err(format!("dosya {AZAMI_BAYT} bayttan büyük"));
        }
        let metin = std::str::from_utf8(icerik).map_err(|_| "dosya UTF-8 değil".to_string())?;
        let mut liste = Self::default();
        for (no, satir) in metin.lines().enumerate() {
            let kural = satir.split_whitespace().next().unwrap_or("");
            if kural.is_empty() || kural.starts_with("//") {
                continue;
            }
            if !kural.is_ascii() {
                liste.yok_sayilan += 1;
                continue;
            }
            let kural = kural.to_ascii_lowercase();
            let (kume, govde) = if let Some(govde) = kural.strip_prefix('!') {
                (&mut liste.istisnalar, govde.to_string())
            } else if let Some(govde) = kural.strip_prefix("*.") {
                (&mut liste.jokerler, govde.to_string())
            } else {
                (&mut liste.kurallar, kural.clone())
            };
            if !gecerli_kural_govdesi(&govde) {
                return Err(format!("{}. satırda geçersiz kural", no + 1));
            }
            kume.insert(govde);
            if liste.kural_sayisi() > AZAMI_KURAL {
                return Err(format!("{AZAMI_KURAL} kuraldan fazla"));
            }
        }
        if liste.kural_sayisi() == 0 {
            return Err("dosyada kural yok".into());
        }
        Ok(liste)
    }

    pub fn kural_sayisi(&self) -> usize {
        self.kurallar.len() + self.jokerler.len() + self.istisnalar.len()
    }

    /// Yok sayılan ASCII dışı kural satırı sayısı (açılış günlüğü için).
    pub fn yok_sayilan(&self) -> usize {
        self.yok_sayilan
    }

    /// Geçerli adın kamu son ekinin etiket sayısı (varsayılan kural 1).
    fn son_ek_uzunlugu(&self, etiketler: &[&str]) -> usize {
        let n = etiketler.len();
        // İstisna kuralları önce (en uzun eşleşme ilk bulunur): kuralın
        // kendisi değil, bir etiket kısası kamu son ekidir.
        for i in 0..n {
            if self.istisnalar.contains(&etiketler[i..].join(".")) {
                return n - i - 1;
            }
        }
        let mut en_uzun = 1;
        for i in 0..n {
            let ek = etiketler[i..].join(".");
            let k = n - i;
            if self.kurallar.contains(&ek) {
                en_uzun = en_uzun.max(k);
            }
            if self.jokerler.contains(&ek) {
                // `*.ek` bir etiket daha ister; ad yalnız `ek` ise joker
                // eşleşmez (publicsuffix.org testleri: `mm` → `mm` değil).
                en_uzun = en_uzun.max(if i > 0 { k + 1 } else { k });
            }
        }
        en_uzun
    }

    /// RFC 7489 §3.2 kurumsal alanı: kamu son eki + bir etiket. Ad
    /// geçersizse, kamu son ekinin kendisiyse ya da daha kısaysa boş
    /// Metin (`""`) döner; çağıran (DMARC kodu) bunu hizalanmamış sayar.
    /// Sonuç küçük harfli ve sonda noktasızdır.
    pub fn kurumsal_alan(&self, ad: &str) -> String {
        let Some(ad) = olagan_ad(ad) else {
            return String::new();
        };
        let etiketler: Vec<&str> = ad.split('.').collect();
        let son_ek = self.son_ek_uzunlugu(&etiketler);
        if etiketler.len() <= son_ek {
            return String::new();
        }
        etiketler[etiketler.len() - son_ek - 1..].join(".")
    }
}

fn gecerli_etiket(etiket: &str) -> bool {
    !etiket.is_empty()
        && etiket.len() <= 63
        && etiket
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn gecerli_kural_govdesi(govde: &str) -> bool {
    !govde.is_empty() && govde.len() <= 253 && govde.split('.').all(gecerli_etiket)
}

/// Küçük harfe indirilmiş, sondaki tek nokta atılmış ASCII ad; aksi `None`.
/// Baştaki nokta, boş etiket, 63 oktet üstü etiket, 253 oktet üstü ad,
/// ASCII dışı karakter (U-label) ve LDH/alt çizgi dışı bayt reddedilir.
fn olagan_ad(ad: &str) -> Option<String> {
    let ad = ad.strip_suffix('.').unwrap_or(ad);
    if ad.is_empty() || ad.len() > 253 || !ad.is_ascii() {
        return None;
    }
    let ad = ad.to_ascii_lowercase();
    ad.split('.').all(gecerli_etiket).then_some(ad)
}

#[cfg(test)]
mod testler {
    use super::*;

    /// publicsuffix.org `tests.txt` vakalarının ihtiyaç duyduğu kurallar
    /// (resmî listeden alıntılanmış alt küme; ölçüt algoritmadır).
    const ORNEK: &str = "\
// ===BEGIN ICANN DOMAINS===
com
biz
uk.com
ac
// jp
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
// ===END ICANN DOMAINS===
// ===BEGIN PRIVATE DOMAINS===
blogspot.com
// ===END PRIVATE DOMAINS===
";

    fn liste() -> KamuSonEkleri {
        KamuSonEkleri::ayristir(ORNEK.as_bytes()).unwrap()
    }

    /// `checkPublicSuffix(ad, beklenen)` — beklenen `null` → "".
    #[test]
    fn publicsuffix_org_test_vektorleri() {
        let l = liste();
        let vakalar: &[(&str, &str)] = &[
            // Karışık büyük/küçük harf.
            ("COM", ""),
            ("example.COM", "example.com"),
            ("WwW.example.COM", "example.com"),
            // Baştaki nokta.
            (".com", ""),
            (".example", ""),
            (".example.com", ""),
            (".example.example", ""),
            // Listelenmemiş TLD (varsayılan `*` kuralı).
            ("example", ""),
            ("example.example", "example.example"),
            ("b.example.example", "example.example"),
            ("a.b.example.example", "example.example"),
            // Tek kurallı TLD.
            ("biz", ""),
            ("domain.biz", "domain.biz"),
            ("b.domain.biz", "domain.biz"),
            ("a.b.domain.biz", "domain.biz"),
            // Çok kurallı TLD.
            ("com", ""),
            ("example.com", "example.com"),
            ("b.example.com", "example.com"),
            ("a.b.example.com", "example.com"),
            ("uk.com", ""),
            ("example.uk.com", "example.uk.com"),
            ("b.example.uk.com", "example.uk.com"),
            ("a.b.example.uk.com", "example.uk.com"),
            ("test.ac", "test.ac"),
            // Yalnız joker kurallı TLD.
            ("mm", ""),
            ("c.mm", ""),
            ("b.c.mm", "b.c.mm"),
            ("a.b.c.mm", "b.c.mm"),
            // Daha karmaşık TLD.
            ("jp", ""),
            ("test.jp", "test.jp"),
            ("www.test.jp", "test.jp"),
            ("ac.jp", ""),
            ("test.ac.jp", "test.ac.jp"),
            ("www.test.ac.jp", "test.ac.jp"),
            ("kyoto.jp", ""),
            ("test.kyoto.jp", "test.kyoto.jp"),
            ("ide.kyoto.jp", ""),
            ("b.ide.kyoto.jp", "b.ide.kyoto.jp"),
            ("a.b.ide.kyoto.jp", "b.ide.kyoto.jp"),
            ("c.kobe.jp", ""),
            ("b.c.kobe.jp", "b.c.kobe.jp"),
            ("a.b.c.kobe.jp", "b.c.kobe.jp"),
            ("city.kobe.jp", "city.kobe.jp"),
            ("www.city.kobe.jp", "city.kobe.jp"),
            // Joker + istisna TLD.
            ("ck", ""),
            ("test.ck", ""),
            ("b.test.ck", "b.test.ck"),
            ("a.b.test.ck", "b.test.ck"),
            ("www.ck", "www.ck"),
            ("www.www.ck", "www.ck"),
            // ABD K12.
            ("us", ""),
            ("test.us", "test.us"),
            ("www.test.us", "test.us"),
            ("ak.us", ""),
            ("test.ak.us", "test.ak.us"),
            ("www.test.ak.us", "test.ak.us"),
            ("k12.ak.us", ""),
            ("test.k12.ak.us", "test.k12.ak.us"),
            ("www.test.k12.ak.us", "test.k12.ak.us"),
            // A-label (punycode) adlar ASCII kural olarak eşleşir.
            ("xn--85x722f.com.cn", "xn--85x722f.com.cn"),
            ("xn--55qx5d.cn", ""),
            ("xn--85x722f.xn--55qx5d.cn", "xn--85x722f.xn--55qx5d.cn"),
            ("www.xn--85x722f.xn--55qx5d.cn", "xn--85x722f.xn--55qx5d.cn"),
            ("shishi.xn--55qx5d.cn", "shishi.xn--55qx5d.cn"),
            // PRIVATE bölümü de geçerlidir.
            ("blogspot.com", ""),
            ("foo.blogspot.com", "foo.blogspot.com"),
            ("a.foo.blogspot.com", "foo.blogspot.com"),
        ];
        for (ad, beklenen) in vakalar {
            assert_eq!(l.kurumsal_alan(ad), *beklenen, "{ad}");
        }
    }

    #[test]
    fn u_label_ve_gecersiz_adlar_bos_doner() {
        let l = liste();
        for ad in [
            "",
            ".",
            "食狮.com.cn",
            "a..example.com",
            "örnek.com.tr",
            "a b.com",
            "a\0.com",
            "example.com..",
            &format!("{}.com", "a".repeat(64)),
            &format!("{}com", "a.".repeat(127)),
        ] {
            assert_eq!(l.kurumsal_alan(ad), "", "{ad:?}");
        }
        assert_eq!(l.kurumsal_alan("posta.example.com."), "example.com");
        assert_eq!(l.kurumsal_alan("_dmarc.example.com"), "example.com");
    }

    #[test]
    fn ascii_disi_kural_yok_sayilir_bozuk_kural_reddedilir() {
        let l = liste();
        assert_eq!(l.yok_sayilan(), 1);
        assert_eq!(l.kural_sayisi(), 20);
        for bozuk in ["a..b\n", "*.\n", "!\n", "a.*.b\n", "-x/y\n"] {
            assert!(
                KamuSonEkleri::ayristir(bozuk.as_bytes()).is_err(),
                "{bozuk:?}"
            );
        }
        assert!(KamuSonEkleri::ayristir(b"// yalniz yorum\n").is_err());
        assert!(KamuSonEkleri::ayristir(&[0xff, b'\n']).is_err());
        assert!(KamuSonEkleri::ayristir(&vec![b'a'; AZAMI_BAYT + 1]).is_err());
    }

    /// Rastgele ad ve kural karışımı paniksizdir; sonuç ya boştur ya da
    /// adın nokta sınırında biten soneki.
    #[test]
    fn rastgele_adlar_paniksiz_ve_sonek() {
        let l = liste();
        let parcalar = [
            "a", "com", "ck", "www", "mm", "jp", "kobe", "", "-", "x_y", "ö", "A",
        ];
        let mut durum: u64 = 0x2545_f491_4f6c_dd1d;
        for _ in 0..20_000 {
            durum ^= durum << 13;
            durum ^= durum >> 7;
            durum ^= durum << 17;
            let n = (durum % 6) as usize;
            let ad: Vec<&str> = (0..n)
                .map(|i| parcalar[((durum >> (i * 8)) as usize) % parcalar.len()])
                .collect();
            let ad = ad.join(".");
            let sonuc = l.kurumsal_alan(&ad);
            let kucuk = ad.to_ascii_lowercase();
            let kucuk = kucuk.strip_suffix('.').unwrap_or(&kucuk);
            assert!(
                sonuc.is_empty() || kucuk == sonuc || kucuk.ends_with(&format!(".{sonuc}")),
                "{ad:?} → {sonuc:?}"
            );
        }
    }
}
