//! `gizli` birim testleri (K-410: satır bütçesi için ayrı dosya).

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
    assert!(coz(&anahtar, &bozuk).is_err() || coz(&anahtar, &bozuk).unwrap() != "Gizli:P@ss/1 ✓");
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

/// K-410: kaldırılan eski `zee1:` base64 yazıcı/okuyucusu (3761bcb4).
const ALFABE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn eski_base64_yaz(baytlar: &[u8]) -> String {
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

fn eski_base64_oku(metin: &str) -> Option<Vec<u8>> {
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

/// 0–300 bayt, her bayt değeri; deterministik.
fn esdegerlik_girdileri() -> Vec<Vec<u8>> {
    let mut durum = 0x9e37_79b9u32;
    (0..=300usize)
        .map(|boy| {
            (0..boy)
                .map(|_| {
                    durum ^= durum << 13;
                    durum ^= durum >> 17;
                    durum ^= durum << 5;
                    (durum >> 24) as u8
                })
                .collect()
        })
        .collect()
}

#[test]
fn k410_eski_yazici_ve_okuyucuyla_esdeger() {
    for veri in esdegerlik_girdileri() {
        let yeni = crate::kodlama::base64_kodla(&veri);
        assert_eq!(yeni, eski_base64_yaz(&veri), "{}", veri.len());
        // Kanonik her yazımda iki okuyucu aynı baytları verir.
        assert_eq!(
            crate::kodlama::base64_coz(&yeni, yeni.len()).unwrap(),
            eski_base64_oku(&yeni).unwrap()
        );
    }
    // İki fark: eski okuyucu kanonik olmayan bitli ikinci yazımı ve ortada
    // dolgu taşıyan birleşik dörtlüleri kabul eder; katı çözücü reddeder.
    assert_eq!(eski_base64_oku("Zh==").unwrap(), b"f");
    assert_eq!(eski_base64_oku("Zg==Zg==").unwrap(), b"ff");
    for fark in ["Zh==", "Zg==Zg=="] {
        assert!(crate::kodlama::base64_coz(fark, 16).is_err(), "{fark}");
    }
    // Eski okuyucunun reddettikleri yeni okuyucuda da ret.
    for bozuk in ["abc", "Zg=", "Z===", "=Zg=", "Zm9v\n", "Zm 9v", "Zm9v-_=="] {
        assert!(eski_base64_oku(bozuk).is_none(), "{bozuk}");
        assert!(crate::kodlama::base64_coz(bozuk, 16).is_err(), "{bozuk}");
    }
}
