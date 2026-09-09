//! K-181: TOTP (RFC 6238) — bağımlılıksız SHA-1, HMAC ve Base32.

use super::sabit_zamanli_esit;

/// Bağımlılıksız SHA-1 (FIPS 180-4). Yalnız HMAC-TOTP uyumluluğu içindir;
/// içerik kimliği için `sha256_hex` kullanılır.
pub(crate) fn sha1(girdi: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let bit_uzunlugu = (girdi.len() as u64).wrapping_mul(8);
    let mut veri = girdi.to_vec();
    veri.push(0x80);
    while veri.len() % 64 != 56 {
        veri.push(0);
    }
    veri.extend_from_slice(&bit_uzunlugu.to_be_bytes());
    for blok in veri.chunks_exact(64) {
        let mut w = [0u32; 80];
        for (i, dortlu) in blok.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([dortlu[0], dortlu[1], dortlu[2], dortlu[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        for (yer, deger) in h.iter_mut().zip([a, b, c, d, e]) {
            *yer = yer.wrapping_add(deger);
        }
    }
    let mut cikti = [0u8; 20];
    for (i, deger) in h.iter().enumerate() {
        cikti[i * 4..i * 4 + 4].copy_from_slice(&deger.to_be_bytes());
    }
    cikti
}

/// HMAC-SHA1 (RFC 2104).
pub(crate) fn hmac_sha1(anahtar: &[u8], ileti: &[u8]) -> [u8; 20] {
    let mut blok = [0u8; 64];
    if anahtar.len() > 64 {
        blok[..20].copy_from_slice(&sha1(anahtar));
    } else {
        blok[..anahtar.len()].copy_from_slice(anahtar);
    }
    let mut ic = Vec::with_capacity(64 + ileti.len());
    ic.extend(blok.iter().map(|b| b ^ 0x36));
    ic.extend_from_slice(ileti);
    let ic_ozet = sha1(&ic);
    let mut dis = Vec::with_capacity(84);
    dis.extend(blok.iter().map(|b| b ^ 0x5c));
    dis.extend_from_slice(&ic_ozet);
    sha1(&dis)
}

const BASE32_ALFABESI: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// RFC 4648 Base32 (dolgusuz, büyük harf) — otpauth gizli anahtar biçimi.
pub(crate) fn base32_kodla(baytlar: &[u8]) -> String {
    let mut cikti = String::with_capacity(baytlar.len().div_ceil(5) * 8);
    let mut tampon: u64 = 0;
    let mut bit = 0u32;
    for &bayt in baytlar {
        tampon = (tampon << 8) | u64::from(bayt);
        bit += 8;
        while bit >= 5 {
            bit -= 5;
            cikti.push(BASE32_ALFABESI[((tampon >> bit) & 31) as usize] as char);
        }
    }
    if bit > 0 {
        cikti.push(BASE32_ALFABESI[((tampon << (5 - bit)) & 31) as usize] as char);
    }
    cikti
}

/// Base32 çözümü; boşluk ve `=` dolgusu yok sayılır, harf büyüklüğü serbesttir.
/// Geçersiz karakter `None`.
pub(crate) fn base32_coz(metin: &str) -> Option<Vec<u8>> {
    let mut cikti = Vec::with_capacity(metin.len() * 5 / 8);
    let mut tampon: u64 = 0;
    let mut bit = 0u32;
    for karakter in metin.bytes() {
        if karakter == b'=' || karakter == b' ' || karakter == b'-' {
            continue;
        }
        let deger = BASE32_ALFABESI
            .iter()
            .position(|&a| a == karakter.to_ascii_uppercase())?;
        tampon = (tampon << 5) | deger as u64;
        bit += 5;
        if bit >= 8 {
            bit -= 8;
            cikti.push(((tampon >> bit) & 0xff) as u8);
        }
    }
    Some(cikti)
}

/// 160 bit CSPRNG TOTP gizli anahtarı (Base32).
#[cfg(not(target_arch = "wasm32"))]
pub fn totp_anahtari_uret() -> Result<String, String> {
    let mut baytlar = [0u8; 20];
    getrandom::fill(&mut baytlar)
        .map_err(|hata| format!("işletim sistemi güvenli rastgelelik vermedi: {}", hata))?;
    Ok(base32_kodla(&baytlar))
}

#[cfg(target_arch = "wasm32")]
pub fn totp_anahtari_uret() -> Result<String, String> {
    Err("playground TOTP anahtarı üretemez".into())
}

/// RFC 6238 TOTP: 30 saniyelik adım, 6 hane, HMAC-SHA1.
pub fn totp_kodu(gizli: &[u8], unix_saniye: i64) -> String {
    let adim = (unix_saniye.max(0) / 30) as u64;
    let ozet = hmac_sha1(gizli, &adim.to_be_bytes());
    let konum = (ozet[19] & 0x0f) as usize;
    let kesit = ((u32::from(ozet[konum]) & 0x7f) << 24)
        | (u32::from(ozet[konum + 1]) << 16)
        | (u32::from(ozet[konum + 2]) << 8)
        | u32::from(ozet[konum + 3]);
    format!("{:06}", kesit % 1_000_000)
}

/// Verilen anı merkez alarak ±1 adım (90 saniye) penceresinde kodu doğrular.
/// Bozuk Base32 anahtar ya da 6 haneli olmayan kod `false`tur; sabit zamanlı
/// karşılaştırma kullanılır.
pub fn totp_dogrula(gizli_base32: &str, kod: &str, unix_saniye: i64) -> bool {
    let Some(gizli) = base32_coz(gizli_base32) else {
        return false;
    };
    if gizli.is_empty() {
        return false;
    }
    let kod = kod.trim().replace(' ', "");
    if kod.len() != 6 || !kod.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let mut eslesti = false;
    for kayma in [-30i64, 0, 30] {
        let beklenen = totp_kodu(&gizli, unix_saniye.saturating_add(kayma));
        eslesti |= sabit_zamanli_esit(&beklenen, &kod);
    }
    eslesti
}

#[cfg(test)]
mod totp_tests {
    use super::*;

    #[test]
    fn sha1_ve_hmac_bilinen_vektorler() {
        let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
        assert_eq!(
            hex(&sha1(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(hex(&sha1(b"")), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        // RFC 2202 test 2
        assert_eq!(
            hex(&hmac_sha1(b"Jefe", b"what do ya want for nothing?")),
            "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79"
        );
    }

    #[test]
    fn base32_gidis_donus() {
        assert_eq!(base32_kodla(b"foobar"), "MZXW6YTBOI");
        assert_eq!(
            base32_coz("MZXW6YTBOI======").as_deref(),
            Some(&b"foobar"[..])
        );
        assert_eq!(base32_coz("mzxw 6ytb-oi").as_deref(), Some(&b"foobar"[..]));
        assert!(base32_coz("MZXW6YTB0I").is_none());
    }

    #[test]
    fn totp_rfc6238_sha1_vektorleri() {
        // RFC 6238 Ek B, gizli "12345678901234567890", 8 haneli değerlerin son 6 hanesi.
        let gizli = b"12345678901234567890";
        assert_eq!(totp_kodu(gizli, 59), "287082");
        assert_eq!(totp_kodu(gizli, 1111111109), "081804");
        assert_eq!(totp_kodu(gizli, 1234567890), "005924");
        assert_eq!(totp_kodu(gizli, 20000000000), "353130");
        let b32 = base32_kodla(gizli);
        assert!(totp_dogrula(&b32, "287082", 59));
        assert!(
            totp_dogrula(&b32, "287082", 59 + 30),
            "önceki adım penceresi"
        );
        assert!(!totp_dogrula(&b32, "287082", 59 + 120));
        assert!(!totp_dogrula(&b32, "28708", 59));
        assert!(!totp_dogrula("!!!", "287082", 59));
    }
}
