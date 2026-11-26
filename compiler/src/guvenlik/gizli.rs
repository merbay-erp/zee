//! K-210 eki 3: gizli değer şifreleme (XChaCha20-Poly1305, 256 bit anahtar).
//!
//! Anahtar uygulama koduna değil ortam değişkenine aittir (`proje.dil`
//! `gizli_anahtar_değişkeni`); şifreli metin `zee1:` önekli base64
//! (24 bayt nonce ‖ şifreli veri ‖ 16 bayt etiket). Kimlik doğrulamalı
//! şifreleme: bozulmuş ya da başka anahtarla üretilmiş metin çözülmez.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};

/// Şifreli metin öneki; biçim sürümü.
pub const ONEK: &str = "zee1:";
const NONCE_BAYT: usize = 24;

/// 64 hex karakterlik anahtar metnini 32 bayta çevirir.
pub fn anahtar_coz(hex: &str) -> Result<[u8; 32], String> {
    let hex = hex.trim();
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(
            "gizli anahtar 64 hex karakter (256 bit) olmalı; `dil gizli-anahtar` ile üret".into(),
        );
    }
    let mut anahtar = [0u8; 32];
    for (i, parca) in hex.as_bytes().chunks(2).enumerate() {
        let metin = std::str::from_utf8(parca).map_err(|_| "bozuk hex")?;
        anahtar[i] = u8::from_str_radix(metin, 16).map_err(|_| "bozuk hex")?;
    }
    Ok(anahtar)
}

/// Yeni 256 bit anahtarı hex olarak üretir (CLI `dil gizli-anahtar`).
#[cfg(not(target_arch = "wasm32"))]
pub fn anahtar_uret() -> Result<String, String> {
    super::guvenli_belirtec_uret()
}

/// Düz metni verilen anahtar ve nonce ile şifreler; çıktı `zee1:` + base64.
pub fn sifrele(anahtar: &[u8; 32], nonce: &[u8; NONCE_BAYT], duz: &str) -> Result<String, String> {
    let sifreleyici = XChaCha20Poly1305::new(&Key::from(*anahtar));
    let sifreli = sifreleyici
        .encrypt(&XNonce::from(*nonce), duz.as_bytes())
        .map_err(|_| "gizli değer şifrelenemedi".to_string())?;
    let mut baytlar = Vec::with_capacity(NONCE_BAYT + sifreli.len());
    baytlar.extend_from_slice(nonce);
    baytlar.extend_from_slice(&sifreli);
    Ok(format!("{ONEK}{}", crate::kodlama::base64_kodla(&baytlar)))
}

/// `zee1:` metnini çözer; önek, uzunluk, base64 ya da etiket hatasında Err.
pub fn coz(anahtar: &[u8; 32], metin: &str) -> Result<String, String> {
    let govde = metin
        .strip_prefix(ONEK)
        .ok_or_else(|| "gizli değer `zee1:` biçiminde değil".to_string())?;
    // K-410: tek katı çözücü (RFC 4648; dolgu zorunlu, kanonik bitler, boşluk
    // yok). Çıktı girdiden kısadır; ayrı tavan gerekmez.
    let baytlar = crate::kodlama::base64_coz(govde, govde.len())
        .map_err(|_| "gizli değer base64 biçimi bozuk".to_string())?;
    if baytlar.len() < NONCE_BAYT + 16 {
        return Err("gizli değer çok kısa".into());
    }
    let (nonce, sifreli) = baytlar.split_at(NONCE_BAYT);
    let sifreleyici = XChaCha20Poly1305::new(&Key::from(*anahtar));
    let duz = sifreleyici
        .decrypt(
            &XNonce::try_from(nonce).map_err(|_| "gizli değer nonce boyutu bozuk")?,
            sifreli,
        )
        .map_err(|_| "gizli değer çözülemedi: anahtar farklı ya da veri bozulmuş".to_string())?;
    String::from_utf8(duz).map_err(|_| "gizli değer geçerli UTF-8 değil".into())
}

/// Şifreli metin mi (önek denetimi; çözmeden).
pub fn sifreli_mi(metin: &str) -> bool {
    metin.starts_with(ONEK)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn sifrele_coz_dongusu_ve_bozulma() {
        let anahtar = [7u8; 32];
        let nonce = [9u8; NONCE_BAYT];
        let sifreli = sifrele(&anahtar, &nonce, "Gizli:P@ss/1 ✓").unwrap();
        // chacha20poly1305 0.10.1 ile üretilmiş sentetik eski zee1 vektörü.
        let legacy =
            "zee1:CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJ+RuY5hWQKSKHYScNSJGbeLOEvw4R4C2ORyfw/ptzk34=";
        assert_eq!(sifreli, legacy);
        assert_eq!(coz(&anahtar, legacy).unwrap(), "Gizli:P@ss/1 ✓");
        assert!(sifreli.starts_with(ONEK));
        assert_eq!(coz(&anahtar, &sifreli).unwrap(), "Gizli:P@ss/1 ✓");
        let mut bozuk = sifreli.clone();
        bozuk.replace_range(sifreli.len() - 3..sifreli.len() - 2, "A");
        assert!(
            coz(&anahtar, &bozuk).is_err() || coz(&anahtar, &bozuk).unwrap() != "Gizli:P@ss/1 ✓"
        );
        assert!(coz(&[8u8; 32], &sifreli).is_err());
        assert!(coz(&anahtar, "düz metin").is_err());
        assert!(anahtar_coz("abc").is_err());
        assert_eq!(anahtar_coz(&"ab".repeat(32)).unwrap()[0], 0xab);
    }

    #[test]
    fn base64_dongusu_ve_kanonik_olmayan_kodlama_reddi() {
        let anahtar = [7u8; 32];
        let nonce = [9u8; NONCE_BAYT];
        for n in 0..12 {
            let duz: String = (0..n).map(|i| char::from(b'a' + i)).collect();
            assert_eq!(
                coz(&anahtar, &sifrele(&anahtar, &nonce, &duz).unwrap()).unwrap(),
                duz
            );
        }
        // K-410: aynı baytlara çözülen kanonik olmayan dolgu bitli biçim artık
        // reddedilir (Zee yalnız kanonik biçim üretir).
        let sifreli = sifrele(&anahtar, &nonce, "a").unwrap();
        let govde = sifreli.strip_prefix(ONEK).unwrap();
        assert!(govde.ends_with('='), "{govde}");
        let son = govde.trim_end_matches('=');
        let dolgu = govde.len() - son.len();
        let (bas, kuyruk) = son.split_at(son.len() - 1);
        const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let deger = A.iter().position(|&c| c == kuyruk.as_bytes()[0]).unwrap();
        let bozuk = format!("{ONEK}{bas}{}{}", A[deger | 1] as char, "=".repeat(dolgu));
        assert_ne!(bozuk, sifreli);
        assert_eq!(
            coz(&anahtar, &bozuk).unwrap_err(),
            "gizli değer base64 biçimi bozuk"
        );
        assert!(coz(&anahtar, "zee1:abc").is_err());
        assert!(coz(&anahtar, &format!("{sifreli}\n")).is_err());
    }
}
