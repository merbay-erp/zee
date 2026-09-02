//! K-088 kriptografik sınırı.
//!
//! Uygulama kodu rastgele sayı üretecini oturum/parola güvenliği için
//! kullanmaz. Belirteçler işletim sisteminin CSPRNG kaynağından, parola
//! özetleri Argon2id'in PHC biçiminden gelir. WASM playground bu üretim
//! capability'lerini bilinçli olarak sunmaz.

/// 256 bit işletim sistemi rastgeleliğini küçük harfli hex metne çevirir.
#[cfg(not(target_arch = "wasm32"))]
pub fn guvenli_belirtec_uret() -> Result<String, String> {
    let mut baytlar = [0u8; 32];
    getrandom::fill(&mut baytlar)
        .map_err(|hata| format!("işletim sistemi güvenli rastgelelik vermedi: {}", hata))?;
    Ok(hex(&baytlar))
}

#[cfg(target_arch = "wasm32")]
pub fn guvenli_belirtec_uret() -> Result<String, String> {
    Err("playground üretim güvenlik belirteci üretemez".into())
}

/// Argon2id v19 ve kitaplığın güncel güvenli varsayımlarıyla PHC özeti.
#[cfg(not(target_arch = "wasm32"))]
pub fn parola_ozeti_uret(parola: &str) -> Result<String, String> {
    use argon2::password_hash::PasswordHasher;
    argon2::Argon2::default()
        .hash_password(parola.as_bytes())
        .map(|ozet| ozet.to_string())
        .map_err(|hata| format!("Argon2id parola özeti üretilemedi: {}", hata))
}

#[cfg(target_arch = "wasm32")]
pub fn parola_ozeti_uret(_parola: &str) -> Result<String, String> {
    Err("playground parola özeti üretemez".into())
}

/// Yalnız Argon2id PHC dizelerini kabul eder. Bozuk/bilinmeyen özet ile yanlış
/// parola aynı `false` sonucunu verir; kimlik doğrulama ayrıntı sızdırmaz.
#[cfg(not(target_arch = "wasm32"))]
pub fn parola_dogrula(parola: &str, ozet: &str) -> bool {
    use argon2::password_hash::{phc::PasswordHash, PasswordVerifier};
    let Ok(ayristirilmis) = PasswordHash::new(ozet) else {
        return false;
    };
    if ayristirilmis.algorithm.as_str() != "argon2id" {
        return false;
    }
    argon2::Argon2::default()
        .verify_password(parola.as_bytes(), &ayristirilmis)
        .is_ok()
}

#[cfg(target_arch = "wasm32")]
pub fn parola_dogrula(_parola: &str, _ozet: &str) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn hex(baytlar: &[u8]) -> String {
    const RAKAMLAR: &[u8; 16] = b"0123456789abcdef";
    let mut sonuc = String::with_capacity(baytlar.len() * 2);
    for &bayt in baytlar {
        sonuc.push(RAKAMLAR[(bayt >> 4) as usize] as char);
        sonuc.push(RAKAMLAR[(bayt & 0x0f) as usize] as char);
    }
    sonuc
}

/// Bağımlılıksız SHA-256 (FIPS 180-4); katmanlardan bağımsız içerik kimliği.
pub(crate) fn sha256_hex(girdi: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_uzunlugu = (girdi.len() as u64).wrapping_mul(8);
    let mut veri = girdi.to_vec();
    veri.push(0x80);
    while veri.len() % 64 != 56 {
        veri.push(0);
    }
    veri.extend_from_slice(&bit_uzunlugu.to_be_bytes());

    for blok in veri.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, dortlu) in blok.chunks_exact(4).enumerate() {
            let mut baytlar = [0; 4];
            baytlar.copy_from_slice(dortlu);
            w[i] = u32::from_be_bytes(baytlar);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (yer, deger) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *yer = yer.wrapping_add(deger);
        }
    }
    h.iter().map(|deger| format!("{deger:08x}")).collect()
}

/// Uzunluğu de karşılaştırmaya katarak eşleşme konumunda erken dönmez.
pub(crate) fn sabit_zamanli_esit(sol: &str, sag: &str) -> bool {
    let sol = sol.as_bytes();
    let sag = sag.as_bytes();
    let mut fark = sol.len() ^ sag.len();
    let uzunluk = sol.len().max(sag.len());
    for i in 0..uzunluk {
        let a = sol.get(i).copied().unwrap_or(0);
        let b = sag.get(i).copied().unwrap_or(0);
        fark |= (a ^ b) as usize;
    }
    fark == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guvenli_belirtecler_256_bit_ve_farklidir() {
        let bir = guvenli_belirtec_uret().expect("CSPRNG");
        let iki = guvenli_belirtec_uret().expect("CSPRNG");
        assert_eq!(bir.len(), 64);
        assert!(bir.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_ne!(bir, iki);
    }

    #[test]
    fn argon2id_ozeti_dogru_parolayi_ayirir() {
        let ozet = parola_ozeti_uret("uzun bir deneme parolası").expect("Argon2id");
        assert!(ozet.starts_with("$argon2id$v=19$"));
        assert!(parola_dogrula("uzun bir deneme parolası", &ozet));
        assert!(!parola_dogrula("yanlış", &ozet));
        assert!(!parola_dogrula("x", "bozuk-özet"));
    }

    #[test]
    fn sabit_zamanli_karsilastirma_uzunlugu_da_denetler() {
        assert!(sabit_zamanli_esit("abc", "abc"));
        assert!(!sabit_zamanli_esit("abc", "abd"));
        assert!(!sabit_zamanli_esit("abc", "abc0"));
    }

    #[test]
    fn sha256_bilinen_vektor() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
