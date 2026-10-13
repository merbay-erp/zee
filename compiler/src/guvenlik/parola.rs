//! K-210: hermetik profil için sabit tuzlu Argon2id üretimi.

/// K-210: `zee-io-1` deterministik profilinde sahte adaptörün ürettiği sabit
/// tuzlu Argon2id PHC özeti. Aynı tohum aynı özeti verir; üretim adaptörü bu
/// işlevi kullanmaz, her zaman CSPRNG tuzu alır.
#[cfg(not(target_arch = "wasm32"))]
pub fn parola_ozeti_uret_tuzlu(parola: &str, tuz: &[u8; 16]) -> Result<String, String> {
    use argon2::password_hash::CustomizedPasswordHasher;
    argon2::Argon2::default()
        .hash_password_customized(
            parola.as_bytes(),
            tuz,
            None,
            None,
            argon2::Params::default(),
        )
        .map(|ozet| ozet.to_string())
        .map_err(|hata| format!("Argon2id parola özeti üretilemedi: {}", hata))
}

#[cfg(target_arch = "wasm32")]
pub fn parola_ozeti_uret_tuzlu(_parola: &str, _tuz: &[u8; 16]) -> Result<String, String> {
    Err("playground parola özeti üretemez".into())
}
