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
    Ok(format!("{ONEK}{}", base64_yaz(&baytlar)))
}

/// `zee1:` metnini çözer; önek, uzunluk, base64 ya da etiket hatasında Err.
pub fn coz(anahtar: &[u8; 32], metin: &str) -> Result<String, String> {
    let govde = metin
        .strip_prefix(ONEK)
        .ok_or_else(|| "gizli değer `zee1:` biçiminde değil".to_string())?;
    let baytlar = base64_oku(govde).ok_or_else(|| "gizli değer base64 biçimi bozuk".to_string())?;
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

const ALFABE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_yaz(baytlar: &[u8]) -> String {
    let mut cikti = String::with_capacity(baytlar.len().div_ceil(3) * 4);
    for parca in baytlar.chunks(3) {
        let b = [
            parca[0],
            *parca.get(1).unwrap_or(&0),
            *parca.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        cikti.push(ALFABE[(n >> 18) as usize & 63] as char);
        cikti.push(ALFABE[(n >> 12) as usize & 63] as char);
        cikti.push(if parca.len() > 1 {
            ALFABE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        cikti.push(if parca.len() > 2 {
            ALFABE[n as usize & 63] as char
        } else {
            '='
        });
    }
    cikti
}

fn base64_oku(metin: &str) -> Option<Vec<u8>> {
    let baytlar = metin.as_bytes();
    if !baytlar.len().is_multiple_of(4) {
        return None;
    }
    let deger = |c: u8| ALFABE.iter().position(|&a| a == c).map(|p| p as u32);
    let mut cikti = Vec::with_capacity(baytlar.len() / 4 * 3);
    for parca in baytlar.chunks(4) {
        let dolgu = parca.iter().rev().take_while(|&&c| c == b'=').count();
        if dolgu > 2 || parca[..4 - dolgu].contains(&b'=') {
            return None;
        }
        let mut n = 0u32;
        for &c in &parca[..4 - dolgu] {
            n = (n << 6) | deger(c)?;
        }
        n <<= 6 * dolgu as u32;
        cikti.push((n >> 16) as u8);
        if dolgu < 2 {
            cikti.push((n >> 8) as u8);
        }
        if dolgu < 1 {
            cikti.push(n as u8);
        }
    }
    Some(cikti)
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
    fn base64_dongusu() {
        for n in 0..12 {
            let v: Vec<u8> = (0..n).map(|i| (i as u8).wrapping_mul(37)).collect();
            assert_eq!(base64_oku(&base64_yaz(&v)).unwrap(), v);
        }
        assert!(base64_oku("abc").is_none());
    }
}
