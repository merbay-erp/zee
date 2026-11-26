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

/// K-402 (RFC-0043 Dilim 2): tek satır protokol alanı base64'ü (AUTH PLAIN,
/// IMAP AUTHENTICATE). MIME çözümüyle aynı katı kurallar, ek olarak gömülü
/// CR/LF/boşluk/sekme de alfabe dışıdır (RFC 4648 §3.3).
pub fn base64_coz(girdi: &str, azami: usize) -> Result<Vec<u8>, KodlamaHatasi> {
    if let Some(konum) = girdi.bytes().position(mime_boslugu_mu) {
        return Err(KodlamaHatasi::yeni(
            "alfabe",
            Some(konum),
            "tek satır base64 boşluk ya da satır sonu içeremez",
        ));
    }
    base64_mime_coz(girdi, azami)
}

/// K-402: base64 kodlamanın çıktı boyu (ayırmadan önce bütçe denetimi için).
pub fn base64_boyu(girdi_bayti: usize) -> usize {
    girdi_bayti.div_ceil(3).saturating_mul(4)
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

/// İkinci geçiş: ön taramadan geçmiş girdiyi çözer, kanonik bitleri denetler.
fn base64_dortlulerini_coz(baytlar: &[u8], boy: usize) -> Result<Vec<u8>, KodlamaHatasi> {
    let mut cikti = Vec::with_capacity(boy);
    let mut grup = [0u8; 4];
    let mut doluluk = 0usize;
    for (konum, &bayt) in baytlar.iter().enumerate() {
        if mime_boslugu_mu(bayt) {
            continue;
        }
        grup[doluluk] = if bayt == b'=' {
            64
        } else {
            base64_degeri(bayt).unwrap_or(0)
        };
        doluluk += 1;
        if doluluk < 4 {
            continue;
        }
        doluluk = 0;
        let deger = |i: usize| u32::from(grup[i] & 63);
        let n = (deger(0) << 18) | (deger(1) << 12) | (deger(2) << 6) | deger(3);
        cikti.push((n >> 16) as u8);
        match (grup[2], grup[3]) {
            (64, _) if n & 0xffff != 0 => return Err(kanonik_degil(konum)),
            (64, _) => {}
            (_, 64) if n & 0xff != 0 => return Err(kanonik_degil(konum)),
            (_, 64) => cikti.push((n >> 8) as u8),
            _ => cikti.extend_from_slice(&[(n >> 8) as u8, n as u8]),
        }
    }
    Ok(cikti)
}

fn kanonik_degil(konum: usize) -> KodlamaHatasi {
    KodlamaHatasi::yeni(
        "dolgu",
        Some(konum),
        "base64 dolgu bitleri sıfır değil (kanonik olmayan kodlama)",
    )
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

/// K-396 (RFC-0039 A2d, ADR-172): RFC 2045 §6.8 satır düzeni — her satır en
/// çok 76 karakter ve ardından CRLF; son satır da CRLF ile biter.
pub const BASE64_SATIR_KARAKTERI: u64 = 76;
/// Tam bir satırı dolduran ham bayt sayısı (57 · 4 / 3 = 76).
pub const BASE64_SATIR_HAM_BAYTI: u64 = 57;
const BASE64_SATIR_TOPLAMI: u64 = BASE64_SATIR_KARAKTERI + 2;
/// K-408 (ADR-172 eki): `… base64 satırlarıyla … dosyasına eklemeyi dene`
/// kaynak dosya tavanı ve hedefin toplam tavanı (bayt); K-395 çözerek ekleme,
/// K-396 bağlantı aktarımı ve K-401 dosya özeti tavanlarıyla aynı 32 MiB.
pub const BASE64_SATIRLARI_TAVANI: u64 = 32 * 1024 * 1024;

/// `b` ham baytın satırlı base64 çıktısının bayt uzunluğu:
/// e = 4·⌈b/3⌉, toplam = e + 2·⌈e/76⌉ (spec/23 §K-396 normatif formülü).
pub fn base64_satirli_uzunluk(b: u64) -> u64 {
    let e = 4 * b.div_ceil(3);
    e + 2 * e.div_ceil(BASE64_SATIR_KARAKTERI)
}

/// Kodlanmış çıktı üzerindeki `[konum, konum+adet)` penceresinin ham dosya
/// karşılığı. Pencere çıktı sonunda kırpılır; `konum ≥ toplam` boş pencere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base64Penceresi {
    /// Okumaya başlanacak ham bayt (satır başına hizalı, 57'nin katı).
    pub ham_baslangic: u64,
    /// O satırdan üretilen çıktının atlanacak ilk baytları (< 78).
    pub atla: u64,
    /// Yazılacak çıktı baytı: min(adet, max(0, toplam − konum)).
    pub uzunluk: u64,
}

/// `b` ham baytlık dosyanın satırlı base64 çıktısında `konum`dan başlayan en
/// çok `adet` baytlık pencere.
pub fn base64_satir_penceresi(b: u64, konum: u64, adet: u64) -> Base64Penceresi {
    let toplam = base64_satirli_uzunluk(b);
    let bas = konum.min(toplam);
    let satir = bas / BASE64_SATIR_TOPLAMI;
    Base64Penceresi {
        // K-414: son satırı tam 76 karakter olan ama 57'nin katı olmayan
        // dosyada (b ≡ 55, 56 mod 57) `bas = toplam` sonraki satıra düşer;
        // boş pencerenin ham konumu dosya sonunu aşamaz.
        ham_baslangic: (satir * BASE64_SATIR_HAM_BAYTI).min(b),
        atla: bas - satir * BASE64_SATIR_TOPLAMI,
        uzunluk: adet.min(toplam - bas),
    }
}

/// Satır başına hizalı ham parçayı satırlı base64'e kodlar: her 57 bayt bir
/// 76 karakterlik satır, kalan kısa parça dolgulu son satır; her satır CRLF.
/// Ardışık çağrılar, yalnız sonuncusu 57'nin katı olmayan parçalarla
/// yapıldığında tüm dosyanın kodlamasını bayt bayt verir.
pub fn base64_satirlari_kodla(ham: &[u8], cikti: &mut Vec<u8>) {
    for satir in ham.chunks(BASE64_SATIR_HAM_BAYTI as usize) {
        cikti.extend_from_slice(base64_kodla(satir).as_bytes());
        cikti.extend_from_slice(b"\r\n");
    }
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

    /// K-396: bağımsız başvuru — tek parça kodla, 76'lık satırlara böl.
    fn basvuru_satirli(ham: &[u8]) -> Vec<u8> {
        let kod = base64_kodla(ham);
        let mut cikti = Vec::new();
        for satir in kod.as_bytes().chunks(76) {
            cikti.extend_from_slice(satir);
            cikti.extend_from_slice(b"\r\n");
        }
        cikti
    }

    #[test]
    fn satirli_base64_basvuruyla_ve_formulle_birebir() {
        for b in [
            0usize, 1, 2, 3, 56, 57, 58, 113, 114, 115, 171, 1000, 70_000,
        ] {
            let ham: Vec<u8> = (0..b).map(|i| (i * 31 + 7) as u8).collect();
            let beklenen = basvuru_satirli(&ham);
            let mut parcali = Vec::new();
            // 57'nin katı parçalar + kısa son parça tek parçayla aynı çıktı.
            for parca in ham.chunks(57 * 3) {
                base64_satirlari_kodla(parca, &mut parcali);
            }
            assert_eq!(parcali, beklenen, "b={b}");
            assert_eq!(base64_satirli_uzunluk(b as u64), beklenen.len() as u64);
        }
        assert_eq!(base64_satirli_uzunluk(0), 0);
        assert_eq!(base64_satirli_uzunluk(1), 6);
        assert_eq!(base64_satirli_uzunluk(57), 78);
        assert_eq!(base64_satirli_uzunluk(58), 84);
        assert_eq!(base64_satirli_uzunluk(16 * 1024 * 1024), 22_958_300);
    }

    #[test]
    fn satirli_base64_penceresi_her_kenarda_dilimle_esit() {
        let ham: Vec<u8> = (0..200u32).map(|i| (i * 13) as u8).collect();
        let tum = basvuru_satirli(&ham);
        let toplam = tum.len() as u64;
        for konum in [
            0,
            1,
            75,
            76,
            77,
            78,
            79,
            155,
            156,
            toplam - 1,
            toplam,
            toplam + 9,
        ] {
            for adet in [0, 1, 2, 77, 78, 79, 1024, u64::MAX] {
                let p = base64_satir_penceresi(ham.len() as u64, konum, adet);
                let mut cikti = Vec::new();
                base64_satirlari_kodla(&ham[p.ham_baslangic as usize..], &mut cikti);
                let parca = &cikti[p.atla as usize..][..p.uzunluk as usize];
                let bas = konum.min(toplam) as usize;
                let son = konum.saturating_add(adet).min(toplam) as usize;
                assert_eq!(parca, &tum[bas..son], "konum={konum} adet={adet}");
            }
        }
    }

    #[test]
    fn tek_satir_base64_gomulu_boslugu_reddeder() {
        for (duz, kod) in RFC4648 {
            assert_eq!(base64_coz(kod, 64).unwrap(), duz.as_bytes());
            assert_eq!(base64_boyu(duz.len()), kod.len());
        }
        for (girdi, konum) in [("Zm9v\r\n", 4), ("Zm 9v", 2), ("\tZm9v", 0), ("Zg==\n", 4)] {
            let hata = base64_coz(girdi, 64).unwrap_err();
            assert_eq!((hata.tur, hata.konum), ("alfabe", Some(konum)), "{girdi:?}");
        }
        assert_eq!(base64_coz("Zg", 64).unwrap_err().tur, "uzunluk");
        assert_eq!(base64_coz("Zh==", 64).unwrap_err().tur, "dolgu");
        let kod = base64_kodla(&[0u8; 1025]);
        assert_eq!(base64_coz(&kod, 1024).unwrap_err().tur, "boyut_aşımı");
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
    fn satir_penceresi_sinirlari_dosya_icinde_ve_dilimle_esit() {
        // K-414: b mod 57 ∈ {0, 1, 55, 56}; toplamda ve ötesinde boş pencere
        // dosya sonunu aşan ham konum vermez.
        for kat in 0..4u64 {
            for kalan in [0u64, 1, 55, 56] {
                let b = kat * 57 + kalan;
                let ham: Vec<u8> = (0..b).map(|i| i as u8).collect();
                let mut tum = Vec::new();
                base64_satirlari_kodla(&ham, &mut tum);
                let toplam = base64_satirli_uzunluk(b);
                assert_eq!(tum.len() as u64, toplam);
                for konum in [
                    0,
                    1,
                    77,
                    78,
                    toplam.saturating_sub(1),
                    toplam,
                    toplam + 1,
                    u64::MAX,
                ] {
                    let p = base64_satir_penceresi(b, konum, 100);
                    assert!(p.ham_baslangic <= b, "b={b} konum={konum} {p:?}");
                    let mut parca = Vec::new();
                    base64_satirlari_kodla(&ham[p.ham_baslangic as usize..], &mut parca);
                    let bas = konum.min(toplam) as usize;
                    let son = (bas + p.uzunluk as usize).min(tum.len());
                    let atla = p.atla as usize;
                    assert_eq!(
                        &parca[atla..atla + (son - bas)],
                        &tum[bas..son],
                        "b={b} konum={konum}"
                    );
                    if konum >= toplam {
                        assert_eq!(p.uzunluk, 0);
                    }
                }
            }
        }
    }
}
