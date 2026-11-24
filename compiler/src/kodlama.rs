//! K-389 (RFC-0043 Dilim 1, ADR-167): kodlanmış içeriğin katı ve sınırlı
//! çözümü. Saf, IO'suz ve yetkinliksizdir; yorumlayıcı dosyaya bayt-kesin
//! yazmadan ve aralıklı okumayı `Metin`e çevirmeden önce burayı kullanır.
//!
//! - MIME base64 (RFC 4648 §4 alfabesi, RFC 2045 §6.8): yalnız CR, LF, boşluk
//!   ve sekme atlanır; dolgu zorunlu ve yalnız sonda; kanonik olmayan dolgu
//!   bitleri reddedilir.
//! - Quoted-printable (RFC 2045 §6.7): `=XY` (büyük/küçük onaltılık), yumuşak
//!   satır sonu, satır sonu boşluklarının atılması; bozuk `=` dizisi, çıplak
//!   CR/LF ve yazdırılamaz karakter katı ret.
//! - UTF-8: onarım yok (`U+FFFD` YASAK); ilk geçersiz baytın konumu verilir.
//!
//! Hata hiçbir zaman girdinin kendisini taşımaz (girdi gizli olabilir).

/// Tek çağrıda çözülen içeriğin ve aralıklı okumanın üst sınırı (1 MiB).
pub const AZAMI_PARCA_BAYTI: usize = 1024 * 1024;
/// K-395: çözerek yazma/eklemede hedef dosyanın toplam tavanı (32 MiB; 25 MB
/// iletinin tek ~18 MB eki sığar, GB satırı DoS gerekçesini taşır).
pub const AZAMI_HEDEF_BAYTI: u64 = 32 * 1024 * 1024;
/// K-395: dosyadan dosyaya çözmede kaynak aralığının üst sınırı (48 MiB:
/// 32 MiB çıktının 76 karakterlik satırlı base64'ü ~43,8 MiB'tır).
pub const AZAMI_AKIS_ARALIGI: u64 = 48 * 1024 * 1024;
/// K-395: bayt aramada konumdan itibaren taranan en çok bayt (64 MiB).
pub const AZAMI_TARAMA_BAYTI: u64 = 64 * 1024 * 1024;
/// K-395: aranan bayt dizisinin (iğne) üst sınırı.
pub const AZAMI_IGNE_BAYTI: usize = 1024;

/// Kodlama hatası: C032 `Hata` verisinin `tür` ve `konum` alanları.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KodlamaHatasi {
    pub tur: &'static str,
    /// Girdideki (ya da dosyadaki) ilk hatalı baytın sıfır tabanlı konumu.
    pub konum: Option<u64>,
    pub mesaj: String,
}

impl KodlamaHatasi {
    fn yeni(tur: &'static str, konum: Option<usize>, mesaj: impl Into<String>) -> Self {
        Self {
            tur,
            konum: konum.map(|k| k as u64),
            mesaj: mesaj.into(),
        }
    }

    fn konumlu(tur: &'static str, konum: u64, mesaj: &str) -> Self {
        Self {
            tur,
            konum: Some(konum),
            mesaj: mesaj.into(),
        }
    }
}

/// K-395: dosyadan dosyaya akışın hatası — dosya/aralık/hedef (C013) ya da
/// kodlama (C032, kaynak dosyadaki mutlak konumla).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AkisHatasi {
    Dosya(String),
    Kodlama(KodlamaHatasi),
}

fn boyut_asimi(azami: usize) -> KodlamaHatasi {
    KodlamaHatasi::yeni(
        "boyut_aşımı",
        None,
        format!("çözülen içerik {azami} bayt sınırını aşıyor"),
    )
}

fn base64_degeri(bayt: u8) -> Option<u8> {
    match bayt {
        b'A'..=b'Z' => Some(bayt - b'A'),
        b'a'..=b'z' => Some(bayt - b'a' + 26),
        b'0'..=b'9' => Some(bayt - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// RFC 4648 §4 standart alfabeyle, dolgulu kodlama (testler ve iz için).
pub fn base64_kodla(veri: &[u8]) -> String {
    const ALFABE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut cikti = String::with_capacity(veri.len().div_ceil(3) * 4);
    for grup in veri.chunks(3) {
        let b = [
            grup[0],
            *grup.get(1).unwrap_or(&0),
            *grup.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= grup.len() {
                cikti.push(ALFABE[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                cikti.push('=');
            }
        }
    }
    cikti
}

/// MIME gövdesindeki base64'ü çözer. CR, LF, boşluk ve sekme her yerde
/// atlanır; başka her alfabe dışı karakter, ortada ya da fazla dolgu, 4'e
/// bölünmeyen uzunluk ve kanonik olmayan son bitler ret. Çıktı `azami` baytı
/// ayırmadan önce kestirilip denetlenir.
pub fn base64_mime_coz(girdi: &str, azami: usize) -> Result<Vec<u8>, KodlamaHatasi> {
    let (anlamli, dolgu) = base64_on_tara(girdi.as_bytes())?;
    let cikti_boyu = anlamli / 4 * 3 - dolgu;
    if cikti_boyu > azami {
        return Err(boyut_asimi(azami));
    }
    base64_dortlulerini_coz(girdi.as_bytes(), cikti_boyu)
}

fn mime_boslugu_mu(bayt: u8) -> bool {
    matches!(bayt, b'\r' | b'\n' | b' ' | b'\t')
}

/// İlk geçiş: alfabe ve dolgu yerleşimini denetler; (anlamlı karakter,
/// dolgu) sayısını verir. Hiç ayırma yapmaz.
fn base64_on_tara(baytlar: &[u8]) -> Result<(usize, usize), KodlamaHatasi> {
    let mut anlamli = 0usize;
    let mut dolgu = 0usize;
    for (konum, &bayt) in baytlar.iter().enumerate() {
        if mime_boslugu_mu(bayt) {
            continue;
        }
        if bayt == b'=' {
            dolgu += 1;
            if dolgu > 2 || anlamli % 4 < 2 {
                return Err(KodlamaHatasi::yeni(
                    "dolgu",
                    Some(konum),
                    "base64 dolgusu yalnız son dörtlünün 3. ya da 4. karakteri olabilir",
                ));
            }
        } else if base64_degeri(bayt).is_none() {
            return Err(KodlamaHatasi::yeni(
                "alfabe",
                Some(konum),
                "base64 alfabesi dışında karakter",
            ));
        } else if dolgu > 0 {
            return Err(KodlamaHatasi::yeni(
                "dolgu",
                Some(konum),
                "base64 dolgusundan sonra veri gelemez",
            ));
        }
        anlamli += 1;
    }
    if !anlamli.is_multiple_of(4) {
        return Err(KodlamaHatasi::yeni(
            "uzunluk",
            Some(baytlar.len()),
            "base64 dörtlüsü eksik; dolgu (=) zorunludur",
        ));
    }
    Ok((anlamli, dolgu))
}

/// İkinci geçiş: ön taramadan geçmiş girdiyi akış çözücüsüyle çözer.
fn base64_dortlulerini_coz(baytlar: &[u8], boy: usize) -> Result<Vec<u8>, KodlamaHatasi> {
    let mut cikti = Vec::with_capacity(boy);
    let mut cozucu = Base64AkisCozucu::default();
    cozucu.besle(baytlar, 0, &mut cikti)?;
    cozucu.bitir(baytlar.len() as u64)?;
    Ok(cikti)
}

/// K-395 (ADR-171): MIME base64 akış çözücüsü. `base64_mime_coz` ile aynı
/// katı kurallar; girdi parça parça beslenir, dörtlü ve dolgu durumu parça
/// sınırını aşar. Hata konumu mutlaktır (`taban` + parça içi indeks).
#[derive(Debug, Default)]
pub struct Base64AkisCozucu {
    grup: [u8; 4],
    doluluk: usize,
    anlamli: u64,
    dolgu: u8,
}

impl Base64AkisCozucu {
    /// `parca`yı çözüp tamamlanan dörtlülerin baytlarını `cikti`ya ekler.
    pub fn besle(
        &mut self,
        parca: &[u8],
        taban: u64,
        cikti: &mut Vec<u8>,
    ) -> Result<(), KodlamaHatasi> {
        for (sira, &bayt) in parca.iter().enumerate() {
            let konum = taban.saturating_add(sira as u64);
            if mime_boslugu_mu(bayt) {
                continue;
            }
            let deger = if bayt == b'=' {
                self.dolgu += 1;
                if self.dolgu > 2 || self.anlamli % 4 < 2 {
                    return Err(KodlamaHatasi::konumlu(
                        "dolgu",
                        konum,
                        "base64 dolgusu yalnız son dörtlünün 3. ya da 4. karakteri olabilir",
                    ));
                }
                64
            } else if let Some(deger) = base64_degeri(bayt) {
                if self.dolgu > 0 {
                    return Err(KodlamaHatasi::konumlu(
                        "dolgu",
                        konum,
                        "base64 dolgusundan sonra veri gelemez",
                    ));
                }
                deger
            } else {
                return Err(KodlamaHatasi::konumlu(
                    "alfabe",
                    konum,
                    "base64 alfabesi dışında karakter",
                ));
            };
            self.anlamli += 1;
            self.grup[self.doluluk] = deger;
            self.doluluk += 1;
            if self.doluluk == 4 {
                self.doluluk = 0;
                dortluyu_coz(&self.grup, konum, cikti)?;
            }
        }
        Ok(())
    }

    /// Girdi bitti: yarım dörtlü `uzunluk` hatasıdır (`son` = girdi sonu).
    pub fn bitir(&self, son: u64) -> Result<(), KodlamaHatasi> {
        if self.doluluk == 0 {
            return Ok(());
        }
        Err(KodlamaHatasi::konumlu(
            "uzunluk",
            son,
            "base64 dörtlüsü eksik; dolgu (=) zorunludur",
        ))
    }
}

fn dortluyu_coz(grup: &[u8; 4], konum: u64, cikti: &mut Vec<u8>) -> Result<(), KodlamaHatasi> {
    let deger = |i: usize| u32::from(grup[i] & 63);
    let n = (deger(0) << 18) | (deger(1) << 12) | (deger(2) << 6) | deger(3);
    cikti.push((n >> 16) as u8);
    match (grup[2], grup[3]) {
        (64, _) if n & 0xffff != 0 => Err(kanonik_degil(konum)),
        (64, _) => Ok(()),
        (_, 64) if n & 0xff != 0 => Err(kanonik_degil(konum)),
        (_, 64) => {
            cikti.push((n >> 8) as u8);
            Ok(())
        }
        _ => {
            cikti.extend_from_slice(&[(n >> 8) as u8, n as u8]);
            Ok(())
        }
    }
}

fn kanonik_degil(konum: u64) -> KodlamaHatasi {
    KodlamaHatasi::konumlu(
        "dolgu",
        konum,
        "base64 dolgu bitleri sıfır değil (kanonik olmayan kodlama)",
    )
}

/// K-395 (ADR-171): akışlı bayt arama (Knuth–Morris–Pratt). Parça sınırını
/// aşan eşleşmeyi de bulur; bellek iğne boyuyla sınırlıdır, süre doğrusaldır
/// (kötü niyetli girdi karesel tarama üretemez).
#[derive(Debug)]
pub struct BaytArayici {
    igne: Vec<u8>,
    geri: Vec<usize>,
    eslesen: usize,
    islenen: u64,
}

impl BaytArayici {
    /// Boş olmayan `igne` için arayıcı kurar.
    pub fn yeni(igne: &[u8]) -> Self {
        let mut geri = vec![0usize; igne.len()];
        let mut k = 0usize;
        for i in 1..igne.len() {
            while k > 0 && igne[i] != igne[k] {
                k = geri[k - 1];
            }
            if igne[i] == igne[k] {
                k += 1;
            }
            geri[i] = k;
        }
        Self {
            igne: igne.to_vec(),
            geri,
            eslesen: 0,
            islenen: 0,
        }
    }

    /// Sıradaki parçayı tarar; ilk eşleşmenin taramanın başına göre
    /// başlangıç konumunu verir.
    pub fn besle(&mut self, parca: &[u8]) -> Option<u64> {
        for &bayt in parca {
            self.islenen += 1;
            while self.eslesen > 0 && self.igne[self.eslesen] != bayt {
                self.eslesen = self.geri[self.eslesen - 1];
            }
            if self.igne[self.eslesen] == bayt {
                self.eslesen += 1;
            }
            if self.eslesen == self.igne.len() {
                return Some(self.islenen - self.igne.len() as u64);
            }
        }
        None
    }
}

fn onaltilik_degeri(bayt: u8) -> Option<u8> {
    match bayt {
        b'0'..=b'9' => Some(bayt - b'0'),
        b'a'..=b'f' => Some(bayt - b'a' + 10),
        b'A'..=b'F' => Some(bayt - b'A' + 10),
        _ => None,
    }
}

/// Quoted-printable gövdeyi çözer (RFC 2045 §6.7). Satır sonu yalnız CRLF'dir;
/// sert satır sonu aynen yazılır, `=` + CRLF yumuşak satır sonudur, satır
/// sonundaki boşluk/sekme atılır. Bozuk `=` dizisi (`kaçış`), çıplak CR/LF
/// (`satır_sonu`) ve yazdırılabilir ASCII dışı karakter (`alfabe`) ret.
pub fn qp_coz(girdi: &str, azami: usize) -> Result<Vec<u8>, KodlamaHatasi> {
    let baytlar = girdi.as_bytes();
    let mut cikti = Vec::with_capacity(baytlar.len().min(azami));
    let mut bas = 0usize;
    while bas < baytlar.len() {
        let (son, sonraki, sert) = satir_siniri(baytlar, bas)?;
        let mut satir_sonu = son;
        while satir_sonu > bas && matches!(baytlar[satir_sonu - 1], b' ' | b'\t') {
            satir_sonu -= 1;
        }
        let yumusak = satir_sonu > bas && baytlar[satir_sonu - 1] == b'=';
        let icerik_sonu = if yumusak { satir_sonu - 1 } else { satir_sonu };
        qp_satiri_coz(baytlar, bas, icerik_sonu, &mut cikti, azami)?;
        if sert && !yumusak {
            if cikti.len() + 2 > azami {
                return Err(boyut_asimi(azami));
            }
            cikti.extend_from_slice(b"\r\n");
        }
        bas = sonraki;
    }
    Ok(cikti)
}

/// `bas`tan başlayan satırın içerik sonunu, sonraki satır başını ve satırın
/// CRLF ile bitip bitmediğini verir; çıplak CR ya da LF ret.
fn satir_siniri(baytlar: &[u8], bas: usize) -> Result<(usize, usize, bool), KodlamaHatasi> {
    let mut i = bas;
    while i < baytlar.len() {
        match baytlar[i] {
            b'\r' if baytlar.get(i + 1) == Some(&b'\n') => return Ok((i, i + 2, true)),
            b'\r' | b'\n' => {
                return Err(KodlamaHatasi::yeni(
                    "satır_sonu",
                    Some(i),
                    "quoted-printable satır sonu yalnız CRLF olabilir",
                ))
            }
            _ => i += 1,
        }
    }
    Ok((baytlar.len(), baytlar.len(), false))
}

fn qp_satiri_coz(
    baytlar: &[u8],
    bas: usize,
    son: usize,
    cikti: &mut Vec<u8>,
    azami: usize,
) -> Result<(), KodlamaHatasi> {
    let mut i = bas;
    while i < son {
        let bayt = baytlar[i];
        let (deger, adim) = if bayt == b'=' {
            let ust = baytlar.get(i + 1).copied().filter(|_| i + 1 < son);
            let alt = baytlar.get(i + 2).copied().filter(|_| i + 2 < son);
            match (
                ust.and_then(onaltilik_degeri),
                alt.and_then(onaltilik_degeri),
            ) {
                (Some(u), Some(a)) => ((u << 4) | a, 3),
                _ => {
                    return Err(KodlamaHatasi::yeni(
                        "kaçış",
                        Some(i),
                        "quoted-printable `=` ardından iki onaltılık hane gelmeli",
                    ))
                }
            }
        } else if bayt == b' ' || bayt == b'\t' || (33..=126).contains(&bayt) {
            (bayt, 1)
        } else {
            return Err(KodlamaHatasi::yeni(
                "alfabe",
                Some(i),
                "quoted-printable gövdede yazdırılabilir ASCII dışı karakter",
            ));
        };
        if cikti.len() >= azami {
            return Err(boyut_asimi(azami));
        }
        cikti.push(deger);
        i += adim;
    }
    Ok(())
}

/// Katı UTF-8 çözümü; `taban` hata konumuna eklenir (dosya içi mutlak konum).
pub fn utf8_coz(baytlar: Vec<u8>, taban: u64) -> Result<String, KodlamaHatasi> {
    String::from_utf8(baytlar).map_err(|hata| {
        let gecerli = hata.utf8_error().valid_up_to() as u64;
        KodlamaHatasi {
            tur: "utf8",
            konum: Some(taban.saturating_add(gecerli)),
            mesaj: "içerik geçerli UTF-8 değil; ikili içerik için `base64 okumayı dene` kullanın"
                .into(),
        }
    })
}

#[cfg(test)]
mod testler {
    use super::*;

    const RFC4648: [(&str, &str); 7] = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];

    #[test]
    fn rfc4648_vektorleri_gidis_donus() {
        for (duz, kod) in RFC4648 {
            assert_eq!(base64_kodla(duz.as_bytes()), kod);
            assert_eq!(base64_mime_coz(kod, 64).unwrap(), duz.as_bytes());
        }
        let ikili: Vec<u8> = (0..=255).collect();
        let kod = base64_kodla(&ikili);
        let satirli: String = kod
            .as_bytes()
            .chunks(76)
            .map(|s| format!("{}\r\n", std::str::from_utf8(s).unwrap()))
            .collect();
        assert_eq!(base64_mime_coz(&satirli, 1024).unwrap(), ikili);
    }

    #[test]
    fn base64_gevsek_girdiyi_konumla_reddeder() {
        let tur = |girdi: &str| base64_mime_coz(girdi, 64).unwrap_err();
        assert_eq!(tur("Zg").tur, "uzunluk");
        assert_eq!(tur("Zg=").tur, "uzunluk");
        let hata = tur("Zm9v!A==");
        assert_eq!((hata.tur, hata.konum), ("alfabe", Some(4)));
        assert_eq!(tur("Zg==Zg==").tur, "dolgu");
        assert_eq!(tur("Z===").tur, "dolgu");
        assert_eq!(tur("=Zg=").tur, "dolgu");
        // RFC 4648 §3.5: "Zh==" ve "Zm9=" kanonik değil.
        assert_eq!(tur("Zh==").tur, "dolgu");
        assert_eq!(tur("Zm9=").tur, "dolgu");
        assert_eq!(tur("Zm9vYmFy-_").tur, "alfabe");
        assert!(!tur("Zm9v\u{e7}").mesaj.contains('\u{e7}'));
    }

    #[test]
    fn base64_cikti_sinirini_ayirmadan_once_uygular() {
        let kod = base64_kodla(&[7u8; 300]);
        assert_eq!(base64_mime_coz(&kod, 300).unwrap().len(), 300);
        assert_eq!(base64_mime_coz(&kod, 299).unwrap_err().tur, "boyut_aşımı");
    }

    #[test]
    fn qp_yumusak_sert_satir_ve_bosluklari_cozer() {
        let girdi = "Merhaba =C3=A7ok   \r\nuzun sat=\r\nir=3D1\r\nson=   \r\n";
        let cikti = qp_coz(girdi, 1024).unwrap();
        assert_eq!(cikti, "Merhaba çok\r\nuzun satir=1\r\nson".as_bytes());
        assert_eq!(qp_coz("=ff=0A=00", 8).unwrap(), vec![0xff, 0x0a, 0x00]);
        assert_eq!(qp_coz("bitti=", 8).unwrap(), b"bitti");
        assert_eq!(qp_coz("", 8).unwrap(), b"");
    }

    #[test]
    fn qp_bozuk_diziyi_konumla_reddeder() {
        let hata = qp_coz("ab=G1", 64).unwrap_err();
        assert_eq!((hata.tur, hata.konum), ("kaçış", Some(2)));
        assert_eq!(qp_coz("ab=4", 64).unwrap_err().tur, "kaçış");
        assert_eq!(qp_coz("ab=\r\n=4\r\n", 64).unwrap_err().tur, "kaçış");
        let hata = qp_coz("a\nb", 64).unwrap_err();
        assert_eq!((hata.tur, hata.konum), ("satır_sonu", Some(1)));
        assert_eq!(qp_coz("a\rb", 64).unwrap_err().tur, "satır_sonu");
        assert_eq!(qp_coz("çay", 64).unwrap_err().tur, "alfabe");
        assert_eq!(qp_coz("a\u{7}", 64).unwrap_err().tur, "alfabe");
        assert_eq!(qp_coz("abcd", 3).unwrap_err().tur, "boyut_aşımı");
    }

    #[test]
    fn utf8_onarmaz_ve_mutlak_konum_verir() {
        assert_eq!(utf8_coz("çay".as_bytes().to_vec(), 0).unwrap(), "çay");
        for (bozuk, gecerli) in [
            (vec![b'a', 0xC0, 0xAF], 1),
            (vec![0xED, 0xA0, 0x80], 0),
            (vec![0xF4, 0x90, 0x80, 0x80], 0),
            (vec![b'x', b'y', 0xC3], 2),
        ] {
            let hata = utf8_coz(bozuk, 100).unwrap_err();
            assert_eq!((hata.tur, hata.konum), ("utf8", Some(100 + gecerli)));
        }
    }

    #[test]
    fn akis_cozucu_her_parca_sinirinda_ayni_sonucu_verir() {
        let veri: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
        let kod: String = base64_kodla(&veri)
            .as_bytes()
            .chunks(76)
            .map(|s| format!("{}\r\n", std::str::from_utf8(s).unwrap()))
            .collect();
        for boy in [1, 2, 3, 5, 77, 78, 4096] {
            let mut cozucu = Base64AkisCozucu::default();
            let mut cikti = Vec::new();
            for (i, parca) in kod.as_bytes().chunks(boy).enumerate() {
                cozucu.besle(parca, (i * boy) as u64, &mut cikti).unwrap();
            }
            cozucu.bitir(kod.len() as u64).unwrap();
            assert_eq!(cikti, veri, "parça boyu {boy}");
        }
    }

    #[test]
    fn akis_cozucu_hata_konumu_mutlaktir() {
        let mut cozucu = Base64AkisCozucu::default();
        let mut cikti = Vec::new();
        cozucu.besle(b"Zm9v", 1000, &mut cikti).unwrap();
        let hata = cozucu.besle(b"Ym!", 1004, &mut cikti).unwrap_err();
        assert_eq!((hata.tur, hata.konum), ("alfabe", Some(1006)));
        let mut cozucu = Base64AkisCozucu::default();
        cozucu.besle(b"Zm9vY", 7, &mut cikti).unwrap();
        let hata = cozucu.bitir(12).unwrap_err();
        assert_eq!((hata.tur, hata.konum), ("uzunluk", Some(12)));
        let mut cozucu = Base64AkisCozucu::default();
        cozucu.besle(b"Zg=", 0, &mut cikti).unwrap();
        let hata = cozucu.besle(b"=Zg==", 3, &mut cikti).unwrap_err();
        assert_eq!((hata.tur, hata.konum), ("dolgu", Some(4)));
    }

    #[test]
    fn bayt_arayici_parca_sinirini_asar_ve_kmp_geri_doner() {
        let mut arayici = BaytArayici::yeni(b"--sinir");
        assert_eq!(arayici.besle(b"abc--si"), None);
        assert_eq!(arayici.besle(b"nir"), Some(3));
        let mut arayici = BaytArayici::yeni(b"aab");
        assert_eq!(arayici.besle(b"aaaab"), Some(2));
        let mut arayici = BaytArayici::yeni("ç".as_bytes());
        assert_eq!(arayici.besle(&"aç".as_bytes()[..2]), None);
        assert_eq!(arayici.besle(&"aç".as_bytes()[2..]), Some(1));
    }
}
