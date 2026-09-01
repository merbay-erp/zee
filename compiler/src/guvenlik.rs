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
}
