// Only verification is reachable from Zee. Bound work before calling bcrypt;
// use normal verify for Node interoperability, with our own byte limit to avoid
// silently truncating. non_truncating_verify counts its NUL terminator at 72.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn dogrula(parola: &str, ozet: &str) -> bool {
    let bytes = ozet.as_bytes();
    if parola.len() > 72
        || bytes.len() != 60
        || !ozet.is_ascii()
        || !matches!(bytes.get(..4), Some(b"$2a$" | b"$2b$" | b"$2y$"))
        || !bytes[4..6].iter().all(u8::is_ascii_digit)
        || bytes[6] != b'$'
        || !bytes[7..]
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'/'))
    {
        return false;
    }
    let cost = (bytes[4] - b'0') * 10 + bytes[5] - b'0';
    if !(4..=14).contains(&cost) {
        return false;
    }
    // Upstream compares the computed digest with subtle::ConstantTimeEq.
    bcrypt::verify(parola.as_bytes(), ozet).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::super::{parola_dogrula, parola_ozeti_uret};

    #[test]
    fn bcrypt_node_legacy_surumleri_yalniz_dogru_parolayi_dogrular() {
        let hash = "$2b$11$ps3248yUh0B5w2jKirP8QufvI/UGGxj8vc8YKmJKwj0plgNdg5jLy";
        for version in ["2a", "2b", "2y"] {
            let hash = format!("${version}{}", &hash[3..]);
            assert!(parola_dogrula("dogru-parola", &hash));
            assert!(!parola_dogrula("yanlis-parola", &hash));
        }
        let produced = parola_ozeti_uret("dogru-parola").unwrap();
        assert!(produced.starts_with("$argon2id$"));
    }

    #[test]
    fn bcrypt_72_bayt_utf8_siniri_kirpma_yapmaz() {
        assert!(parola_dogrula(
            &"x".repeat(72),
            "$2b$04$GIXDOrrzdGb1OyxFyoEH6.Xpip0zBlHblFtaKPpTrjyGl241EPbW."
        ));
        assert!(!parola_dogrula(
            &"x".repeat(73),
            "$2b$04$GIXDOrrzdGb1OyxFyoEH6.Xpip0zBlHblFtaKPpTrjyGl241EPbW."
        ));
        assert!(parola_dogrula(
            &"é".repeat(36),
            "$2b$04$V7cCT.0keO7mClAb9DaLv.qiuA27F8F9cZKgVIXRskVxYNayW1XU."
        ));
        assert!(!parola_dogrula(
            &"é".repeat(37),
            "$2b$04$V7cCT.0keO7mClAb9DaLv.qiuA27F8F9cZKgVIXRskVxYNayW1XU."
        ));
    }

    #[test]
    fn bcrypt_maliyet_ve_bozuk_bicim_is_yapmadan_reddedilir() {
        let hash = "$2b$11$ps3248yUh0B5w2jKirP8QufvI/UGGxj8vc8YKmJKwj0plgNdg5jLy";
        assert!(parola_dogrula(
            "dogru-parola",
            "$2b$14$vGeMUmkR4hXrISq.18F08uR81dSkkGZV4UXK4hObIAK3.g1omENLq"
        ));
        for cost in ["00", "03", "15", "31", "99", "ab"] {
            let malformed = format!("$2b${cost}${}", &hash[7..]);
            assert!(!parola_dogrula("dogru-parola", &malformed));
        }
        for malformed in ["", "$2b$", "$2x$11$", "$2$11$", "bozuk"] {
            assert!(!parola_dogrula("dogru-parola", malformed));
        }
        assert!(!parola_dogrula(
            "dogru-parola",
            &hash.replace("$2b$", "$2x$")
        ));
        assert!(!parola_dogrula("dogru-parola", &format!("{hash}x")));
        for position in 7..60 {
            let mut bytes = hash.as_bytes().to_vec();
            bytes[position] = b'!';
            assert!(!parola_dogrula(
                "dogru-parola",
                &String::from_utf8(bytes).unwrap()
            ));
        }
    }
}
