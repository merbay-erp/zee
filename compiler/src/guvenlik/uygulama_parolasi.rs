//! RFC-0047 E (ADR-182 dilim B, K-364): cihaz başına uygulama parolası.
//!
//! Görünen değer `zua1-` sürüm etiketi + 8 Base32 karakterlik **arama öneki**
//! (40 bit, gizli değil) + 32 Base32 karakterlik **gizli kısım** (160 bit
//! CSPRNG) taşır; gösterimde dörtlü gruplara bölünür. Depoya yalnız önek ve
//! `zuah1:<anahtar kimliği>:<base64url HMAC-SHA256>` özeti girer; gizli kısım
//! ve görünen değer saklanmaz. HMAC anahtarı ayrı credential'dır
//! (`uygulama_parolası_anahtarları`), şifreleme anahtarından türetilmez.
//!
//! Doğrulama ilkelleri (Basic başlığı çözme, biçim kapısı, sabit zamanlı özet
//! karşılaştırması, ASCII kullanıcı adı katlaması) dilim C'deki DAV kimlik
//! kararının parçalarıdır; burada saf ve test edilebilir tutulur.

use super::{sabit_zamanli_esit, sha256_hex};

/// Görünen değerin sürüm etiketi.
pub const SURUM_ETIKETI: &str = "zua1";
/// Özet biçiminin sürüm etiketi.
pub const OZET_ETIKETI: &str = "zuah1";
/// Arama önekinin Base32 karakter sayısı (40 bit).
pub const ONEK_KARAKTERI: usize = 8;
/// Gizli kısmın Base32 karakter sayısı (160 bit).
pub const GIZLI_KARAKTERI: usize = 32;
/// Üretimin istediği rastgele bayt sayısı: 5 (önek) + 20 (gizli).
pub const RASTGELE_BAYT: usize = 25;
/// HMAC girdisinin alan ayracı.
const ALAN_AYRACI: &[u8] = b"zee-uygulama-parolasi-1\0";
/// Basic çözülmüş değerinin ve kullanıcı adının bayt tavanları.
pub const BASIC_AZAMI_BAYT: usize = 512;
pub const KULLANICI_AZAMI_BAYT: usize = 254;
/// Anahtar kimliği tavanı.
pub const ANAHTAR_KIMLIGI_AZAMI: usize = 16;

/// Üretilen parola: görünen (yalnız bir kez gösterilir), önek, özet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UretilenParola {
    pub gorunen: String,
    pub onek: String,
    pub ozet: String,
}

/// Anahtar kimliği: 1–16 küçük ASCII harf ya da rakam.
pub fn anahtar_kimligi_gecerli(kimlik: &str) -> bool {
    !kimlik.is_empty()
        && kimlik.len() <= ANAHTAR_KIMLIGI_AZAMI
        && kimlik
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

/// Bağımlılıksız SHA-256'nın (`sha256_hex`) 32 baytlık özeti.
fn sha256(girdi: &[u8]) -> [u8; 32] {
    let hex = sha256_hex(girdi);
    let mut cikti = [0u8; 32];
    for (i, bayt) in cikti.iter_mut().enumerate() {
        *bayt = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap_or(0);
    }
    cikti
}

/// HMAC-SHA256 (RFC 2104) bağımlılıksız SHA-256 üstünde.
pub fn hmac_sha256(anahtar: &[u8], ileti: &[u8]) -> [u8; 32] {
    let mut blok = [0u8; 64];
    if anahtar.len() > 64 {
        blok[..32].copy_from_slice(&sha256(anahtar));
    } else {
        blok[..anahtar.len()].copy_from_slice(anahtar);
    }
    let mut ic = Vec::with_capacity(64 + ileti.len());
    ic.extend(blok.iter().map(|b| b ^ 0x36));
    ic.extend_from_slice(ileti);
    let ic_ozet = sha256(&ic);
    let mut dis = Vec::with_capacity(96);
    dis.extend(blok.iter().map(|b| b ^ 0x5c));
    dis.extend_from_slice(&ic_ozet);
    sha256(&dis)
}

/// RFC 4648 §5 base64url, dolgusuz.
fn base64url(veri: &[u8]) -> String {
    const ALFABE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut cikti = String::with_capacity(veri.len().div_ceil(3) * 4);
    for grup in veri.chunks(3) {
        let b = [
            grup[0],
            *grup.get(1).unwrap_or(&0),
            *grup.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..=grup.len() {
            cikti.push(ALFABE[((n >> (18 - 6 * i)) & 63) as usize] as char);
        }
    }
    cikti
}

fn ozet_hesapla(anahtar: &[u8; 32], kimlik: &str, onek: &str, gizli: &str) -> String {
    let mut ileti = Vec::with_capacity(ALAN_AYRACI.len() + onek.len() + gizli.len());
    ileti.extend_from_slice(ALAN_AYRACI);
    ileti.extend_from_slice(onek.as_bytes());
    ileti.extend_from_slice(gizli.as_bytes());
    format!(
        "{OZET_ETIKETI}:{kimlik}:{}",
        base64url(&hmac_sha256(anahtar, &ileti))
    )
}

/// Rastgele 25 bayttan parola kurar; `kimlik` etkin anahtarın kimliğidir.
pub fn uret(
    anahtar: &[u8; 32],
    kimlik: &str,
    rastgele: &[u8; RASTGELE_BAYT],
) -> Result<UretilenParola, String> {
    if !anahtar_kimligi_gecerli(kimlik) {
        return Err("uygulama parolası anahtar kimliği geçersiz".into());
    }
    let onek = super::base32_kodla(&rastgele[..5]);
    let gizli = super::base32_kodla(&rastgele[5..]);
    debug_assert_eq!(onek.len(), ONEK_KARAKTERI);
    debug_assert_eq!(gizli.len(), GIZLI_KARAKTERI);
    let govde = format!("{onek}{gizli}");
    let gruplar: Vec<&str> = (0..govde.len())
        .step_by(4)
        .map(|i| &govde[i..i + 4])
        .collect();
    Ok(UretilenParola {
        gorunen: format!("{SURUM_ETIKETI}-{}", gruplar.join("-")),
        ozet: ozet_hesapla(anahtar, kimlik, &onek, &gizli),
        onek,
    })
}

/// Biçim kapısı: `zua1-` ile başlayan, `-` ve boşluklar atıldıktan sonra tam
/// 40 Base32 karakteri olan değer (büyük/küçük harf duyarsız) → (önek,
/// gizli). Hesap parolası dahil başka her değer `None`dır.
pub fn parolayi_coz(parola: &str) -> Option<(String, String)> {
    let (etiket, govde) = parola.split_once('-')?;
    if !etiket.eq_ignore_ascii_case(SURUM_ETIKETI) || parola.len() > 64 {
        return None;
    }
    let govde: String = govde
        .chars()
        .filter(|c| *c != '-' && *c != ' ')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let gecerli = govde.len() == ONEK_KARAKTERI + GIZLI_KARAKTERI
        && govde
            .bytes()
            .all(|b| b.is_ascii_uppercase() || (b'2'..=b'7').contains(&b));
    gecerli.then(|| {
        (
            govde[..ONEK_KARAKTERI].to_string(),
            govde[ONEK_KARAKTERI..].to_string(),
        )
    })
}

/// Özetin anahtar kimliği; biçimsiz özet `None`.
pub fn ozet_anahtar_kimligi(ozet: &str) -> Option<&str> {
    let mut parca = ozet.splitn(3, ':');
    match (parca.next(), parca.next(), parca.next()) {
        (Some(OZET_ETIKETI), Some(kimlik), Some(_)) if anahtar_kimligi_gecerli(kimlik) => {
            Some(kimlik)
        }
        _ => None,
    }
}

/// Kayıttaki özeti sabit zamanlı karşılaştırır. Anahtar kimliği bildirilen
/// anahtarlardan birinde yoksa ya da kayıt yoksa (`ozet` = `None`) aynı
/// maliyette sahte karşılaştırma yapılır ve sonuç `false`tur.
pub fn ozet_dogrula(
    anahtarlar: &[(String, [u8; 32])],
    onek: &str,
    gizli: &str,
    ozet: Option<&str>,
) -> bool {
    let sahte = [0u8; 32];
    let (anahtar, kimlik, beklenen, gercek) = match ozet
        .and_then(|ozet| ozet_anahtar_kimligi(ozet).map(|kimlik| (ozet, kimlik)))
        .and_then(|(ozet, kimlik)| {
            anahtarlar
                .iter()
                .find(|(k, _)| k == kimlik)
                .map(|(k, a)| (a, k.as_str(), ozet))
        }) {
        Some((anahtar, kimlik, ozet)) => (anahtar, kimlik, ozet, true),
        None => (&sahte, "sahte", "", false),
    };
    let hesap = ozet_hesapla(anahtar, kimlik, onek, gizli);
    let esit = sabit_zamanli_esit(&hesap, beklenen);
    esit && gercek
}

/// `Authorization` başlık değerini Basic olarak çözer (RFC 7617): şema
/// büyük/küçük harf duyarsız, tek boşluk, katı dolgulu base64, çözülmüş
/// değer ≤ 512 bayt katı UTF-8, tam bir `:` (kullanıcı adında `:` olamaz),
/// boş olmayan ≤ 254 baytlık kullanıcı adı.
pub fn basic_coz(deger: &str) -> Option<(String, String)> {
    let (sema, kodlu) = deger.split_once(' ')?;
    if !sema.eq_ignore_ascii_case("basic") || kodlu.is_empty() {
        return None;
    }
    let baytlar = kati_base64_coz(kodlu)?;
    if baytlar.len() > BASIC_AZAMI_BAYT {
        return None;
    }
    let metin = String::from_utf8(baytlar).ok()?;
    let (kullanici, parola) = metin.split_once(':')?;
    if kullanici.is_empty()
        || kullanici.len() > KULLANICI_AZAMI_BAYT
        || metin.chars().any(char::is_control)
    {
        return None;
    }
    Some((kullanici.to_string(), parola.to_string()))
}

fn kati_base64_coz(metin: &str) -> Option<Vec<u8>> {
    let b = metin.as_bytes();
    if b.is_empty() || !b.len().is_multiple_of(4) {
        return None;
    }
    let deger = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let mut cikti = Vec::with_capacity(b.len() / 4 * 3);
    for (i, grup) in b.chunks(4).enumerate() {
        let son = i + 1 == b.len() / 4;
        let dolgu = grup.iter().rev().take_while(|&&c| c == b'=').count();
        if dolgu > 2 || (dolgu > 0 && !son) || grup[..4 - dolgu].contains(&b'=') {
            return None;
        }
        let mut n = 0u32;
        for (j, &c) in grup.iter().enumerate() {
            let v = if j < 4 - dolgu { deger(c)? } else { 0 };
            n |= v << (18 - 6 * j);
        }
        let parca = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        // Kanonik olmayan dolgu bitleri reddedilir.
        if (dolgu == 1 && parca[2] != 0) || (dolgu == 2 && (parca[1] != 0 || parca[2] != 0)) {
            return None;
        }
        cikti.extend_from_slice(&parca[..3 - dolgu]);
    }
    Some(cikti)
}

/// Kullanıcı adı karşılaştırması ASCII küçük harfe katlanmış biçimle yapılır
/// (ürün kararı, ADR-182 soru 1); ASCII dışı baytlar olduğu gibi kalır.
pub fn kullanici_katla(ad: &str) -> String {
    ad.to_ascii_lowercase()
}

#[cfg(test)]
mod testler {
    use super::*;

    const ANAHTAR: [u8; 32] = [7u8; 32];

    #[test]
    fn hmac_sha256_rfc4231_vektorleri() {
        let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
        assert_eq!(
            hex(&hmac_sha256(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(
            hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        // 64 bayttan uzun anahtar önce özetlenir (RFC 4231 §4.6).
        assert_eq!(
            hex(&hmac_sha256(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn uretim_bicimi_onek_gizli_ve_ozet() {
        let rastgele: [u8; RASTGELE_BAYT] = core::array::from_fn(|i| i as u8);
        let p = uret(&ANAHTAR, "k2026", &rastgele).expect("üretim");
        assert!(p.gorunen.starts_with("zua1-"), "{}", p.gorunen);
        assert_eq!(p.gorunen.len(), 5 + 40 + 9);
        assert_eq!(p.gorunen.split('-').count(), 11);
        assert_eq!(p.onek.len(), ONEK_KARAKTERI);
        assert!(p.ozet.starts_with("zuah1:k2026:"), "{}", p.ozet);
        assert_eq!(ozet_anahtar_kimligi(&p.ozet), Some("k2026"));
        let (onek, gizli) = parolayi_coz(&p.gorunen).expect("biçim");
        assert_eq!(onek, p.onek);
        assert!(!p.ozet.contains(&gizli), "özet gizliyi taşımaz");
        // Aynı rastgelelik, aynı anahtar: deterministik; başka anahtar: başka özet.
        assert_eq!(uret(&ANAHTAR, "k2026", &rastgele).unwrap(), p);
        assert_ne!(uret(&[8u8; 32], "k2026", &rastgele).unwrap().ozet, p.ozet);
        assert!(uret(&ANAHTAR, "K-1", &rastgele).is_err());
        assert!(uret(&ANAHTAR, "", &rastgele).is_err());
        assert!(uret(&ANAHTAR, &"a".repeat(17), &rastgele).is_err());
    }

    #[test]
    fn dogrulama_dogru_yanlis_eski_anahtar_ve_sahte_karsilastirma() {
        let rastgele = [0x5au8; RASTGELE_BAYT];
        let p = uret(&ANAHTAR, "eski", &rastgele).unwrap();
        let (onek, gizli) = parolayi_coz(&p.gorunen).unwrap();
        let anahtarlar = vec![
            ("yeni".to_string(), [9u8; 32]),
            ("eski".to_string(), ANAHTAR),
        ];
        // Döndürme: eski anahtar kimliği bildirildikçe doğrular.
        assert!(ozet_dogrula(&anahtarlar, &onek, &gizli, Some(&p.ozet)));
        // Küçük harfle ve grupsuz yazılmış aynı parola kabul.
        let duz = p.gorunen.to_ascii_lowercase().replace('-', "");
        let duz = format!("zua1-{}", &duz[4..]);
        let (o2, g2) = parolayi_coz(&duz).unwrap();
        assert!(ozet_dogrula(&anahtarlar, &o2, &g2, Some(&p.ozet)));
        // Yanlış gizli, bilinmeyen anahtar kimliği, kayıtsız: hepsi false.
        let mut yanlis = gizli.clone();
        yanlis.replace_range(0..1, if &gizli[0..1] == "A" { "B" } else { "A" });
        assert!(!ozet_dogrula(&anahtarlar, &onek, &yanlis, Some(&p.ozet)));
        assert!(!ozet_dogrula(
            &anahtarlar[..1],
            &onek,
            &gizli,
            Some(&p.ozet)
        ));
        assert!(!ozet_dogrula(&anahtarlar, &onek, &gizli, None));
        assert!(!ozet_dogrula(
            &anahtarlar,
            &onek,
            &gizli,
            Some("zuah1:eski:")
        ));
        assert!(!ozet_dogrula(&anahtarlar, &onek, &gizli, Some("bozuk")));
    }

    #[test]
    fn bicim_kapisi_hesap_parolasini_ve_bozugu_reddeder() {
        for kotu in [
            "",
            "hesap-parolasi-123",
            "Parola123!",
            "zua2-AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDDEEEEEEEE",
            "zua1-AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDDEEEEEEE",
            "zua1-AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDDEEEEEEEEE",
            "zua1-AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDDEEEEEEE1",
            "zua1-AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDDEEEEEEE=",
            "zua1AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDDEEEEEEEE",
        ] {
            assert!(parolayi_coz(kotu).is_none(), "{kotu}");
        }
        assert!(parolayi_coz("ZUA1-aaaa aaaa-BBBB-bbbbCCCCCCCCDDDDDDDDEEEEEEEE").is_some());
    }

    #[test]
    fn basic_cozumu_kati() {
        // "ali:zua1-x" → YWxpOnp1YTEteA==
        assert_eq!(
            basic_coz("Basic YWxpOnp1YTEteA=="),
            Some(("ali".into(), "zua1-x".into()))
        );
        assert_eq!(
            basic_coz("bAsIc YWxpOnp1YTEteA=="),
            Some(("ali".into(), "zua1-x".into()))
        );
        // Parolada `:` olabilir: "ali:a:b" → YWxpOmE6Yg==
        assert_eq!(
            basic_coz("Basic YWxpOmE6Yg=="),
            Some(("ali".into(), "a:b".into()))
        );
        for kotu in [
            "",
            "Basic",
            "Basic ",
            "Bearer YWxpOnp1YTEteA==",
            "Basic  YWxpOnp1YTEteA==",
            "Basic YWxpOnp1YTEteA",       // dolgu eksik
            "Basic YWxpOnp1YTEteA==\r\n", // boşluk/denetim
            "Basic YWxp OnA=",            // iç boşluk
            "Basic YWxp",                 // ":" yok ("ali")
            "Basic OnA=",                 // boş kullanıcı (":p")
            "Basic YWxpOnp1YTEteB==",     // kanonik olmayan dolgu bitleri
            "Basic /w==",                 // geçersiz UTF-8 (0xFF)
        ] {
            assert!(basic_coz(kotu).is_none(), "{kotu:?}");
        }
        let uzun = format!("{}:p", "a".repeat(255));
        let kodlu = crate::kodlama::base64_kodla(uzun.as_bytes());
        assert!(basic_coz(&format!("Basic {kodlu}")).is_none());
    }

    #[test]
    fn kullanici_adi_ascii_katlanir_unicode_katlanmaz() {
        assert_eq!(kullanici_katla("Ali@Ornek.TEST"), "ali@ornek.test");
        assert_eq!(kullanici_katla("İLKER"), "İlker");
        assert_eq!(kullanici_katla("ılık"), "ılık");
    }
}
