//! K-419 (RFC-0044 B2b, ADR-177): `dkim_anahtari` fuzz hedefinin ve kararlı
//! oynatma testinin ortak kâhini. DKIM `p=` açık anahtarını (SPKI ya da çıplak
//! PKCS#1 `RSAPublicKey`, Ed25519 ham 32 bayt) ayrı yazılmış bir referans
//! X.690 DER okuyucusuyla çözer ve `dkim_kripto::AcikAnahtar` sonucunu birebir
//! karşılaştırır: kabul/ret, ret türü, `anahtar_biti` ve saklanan PKCS#1
//! baytları. Yeni bağımlılık yoktur.
//!
//! Girdi biçimi: ilk bayt kiptir. Bit 0 algoritmadır (0 `rsa-sha256`, 1
//! `ed25519-sha256`). Bit 1 sıfırsa kalan baytlar çözülmüş anahtardır
//! (`AcikAnahtar::ayristir`); birse kalan baytlar `p=` etiketinin metnidir
//! (`AcikAnahtar::base64ten`, geçersiz UTF-8 atlanır).
//!
//! RSA bileşen kuralları, imzayı doğrulayan ring 0.17.14'ün
//! `RSA_PKCS1_1024_8192_SHA256_FOR_LEGACY_USE_ONLY` anahtar kabulüdür
//! (`rsa/public_exponent.rs`, `arithmetic/bigint/modulusvalue.rs`): modül ve
//! üs pozitif, en kısa DER INTEGER'dır; modül tektir; üs tek ve
//! 3 ≤ e ≤ 2^33 − 1'dir. ring'in çözemediği anahtar spec/31 §3 gereği
//! kriptografik işlemden önce `anahtar_bozuk` rettir (K-419 bulgusu).

use dil::dkim_kripto::{AcikAnahtar, DkimAlgoritmasi, DkimHatasi, RSA_EN_AZ_BIT, RSA_EN_COK_BIT};

/// `base64ten`'in çözülmüş `p=` tavanı (4096 bit SPKI ≈ 550 bayt).
const P_AZAMI_BAYT: usize = 2048;
/// ring `PublicExponent::MAX`.
const US_AZAMI: u64 = (1 << 33) - 1;
/// rsaEncryption 1.2.840.113549.1.1.1 (RFC 8017 Ek C).
const RSA_ENCRYPTION: [u8; 9] = [0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01];

/// Tam TLV: (etiket, içerik, TLV'den sonra kalan). X.690 §8.1: kısa ya da
/// en çok 4 oktetli uzun biçim uzunluk; uzun biçim en kısa olmalı (baştaki
/// sıfır oktet ve 128'den küçük değer yasak), belirsiz uzunluk yok.
/// 65 535 bayttan uzun içerik reddedilir: üretim okuyucusu en çok iki uzunluk
/// okteti okur ve en uzun kabul edilebilir anahtar (4096 bit SPKI) ~550
/// bayttır.
fn tlv(veri: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    if veri.len() < 2 {
        return None;
    }
    let etiket = veri[0];
    let (baslik, boy) = if veri[1] < 0x80 {
        (2, usize::from(veri[1]))
    } else {
        let oktet = usize::from(veri[1] - 0x80);
        if oktet == 0 || oktet > 4 || veri.len() < 2 + oktet || veri[2] == 0 {
            return None;
        }
        let mut boy = 0u64;
        for b in &veri[2..2 + oktet] {
            boy = boy * 256 + u64::from(*b);
        }
        if !(0x80..=0xffff).contains(&boy) {
            return None;
        }
        (2 + oktet, boy as usize)
    };
    if veri.len() - baslik < boy {
        return None;
    }
    Some((etiket, &veri[baslik..baslik + boy], &veri[baslik + boy..]))
}

/// Pozitif, en kısa INTEGER içeriği → baştaki işaret sıfırı atılmış büyüklük.
/// Sıfır değeri (`00`) de reddedilir.
fn pozitif(icerik: &[u8]) -> Option<&[u8]> {
    match icerik {
        [] => None,
        [ilk, ..] if *ilk >= 0x80 => None,
        [0, ikinci, ..] if *ikinci >= 0x80 => Some(&icerik[1..]),
        [0, ..] => None,
        _ => Some(icerik),
    }
}

fn bit_boyu(buyukluk: &[u8]) -> usize {
    let ilk = buyukluk[0];
    (buyukluk.len() - 1) * 8 + (8 - ilk.leading_zeros() as usize)
}

/// Çıplak `RSAPublicKey ::= SEQUENCE { modulus INTEGER, publicExponent INTEGER }`
/// TLV'si; sonda bayt yok. Dönen: modül bit boyu.
fn pkcs1_referansi(tlv_baytlari: &[u8]) -> Option<usize> {
    let (0x30, govde, []) = tlv(tlv_baytlari)? else {
        return None;
    };
    let (0x02, n, kalan) = tlv(govde)? else {
        return None;
    };
    let (0x02, e, []) = tlv(kalan)? else {
        return None;
    };
    let n = pozitif(n)?;
    let e = pozitif(e)?;
    if n[n.len() - 1] & 1 == 0 || e.len() > 5 {
        return None;
    }
    let us = e.iter().fold(0u64, |t, b| (t << 8) | u64::from(*b));
    if !(3..=US_AZAMI).contains(&us) || us & 1 == 0 {
        return None;
    }
    Some(bit_boyu(n))
}

/// Referans `p=` çözümü: (bit, saklanacak anahtar baytları).
pub fn referans(algoritma: DkimAlgoritmasi, bayt: &[u8]) -> Result<(usize, Vec<u8>), DkimHatasi> {
    if algoritma == DkimAlgoritmasi::Ed25519Sha256 {
        // RFC 8463 §4: `p=` ham 32 baytlık Ed25519 açık anahtarıdır.
        return if bayt.len() == 32 {
            Ok((256, bayt.to_vec()))
        } else {
            Err(DkimHatasi::AnahtarBozuk)
        };
    }
    let (0x30, govde, []) = tlv(bayt).ok_or(DkimHatasi::AnahtarBozuk)? else {
        return Err(DkimHatasi::AnahtarBozuk);
    };
    let pkcs1: &[u8] = match tlv(govde).ok_or(DkimHatasi::AnahtarBozuk)? {
        // İlk öğe INTEGER: çıplak PKCS#1 (birçok yayıncı bunu kullanır).
        (0x02, _, _) => bayt,
        // RFC 5280 §4.1.2.7 SubjectPublicKeyInfo; parametre NULL ya da yok.
        (0x30, algoritma_kimligi, kalan) => {
            let (0x06, oid, parametre) = tlv(algoritma_kimligi).ok_or(DkimHatasi::AnahtarBozuk)?
            else {
                return Err(DkimHatasi::AnahtarBozuk);
            };
            if oid != RSA_ENCRYPTION || !matches!(parametre, [] | [0x05, 0x00]) {
                return Err(DkimHatasi::AnahtarBozuk);
            }
            let (0x03, bit_dizisi, []) = tlv(kalan).ok_or(DkimHatasi::AnahtarBozuk)? else {
                return Err(DkimHatasi::AnahtarBozuk);
            };
            match bit_dizisi {
                [0, ic @ ..] => ic,
                _ => return Err(DkimHatasi::AnahtarBozuk),
            }
        }
        _ => return Err(DkimHatasi::AnahtarBozuk),
    };
    let bit = pkcs1_referansi(pkcs1).ok_or(DkimHatasi::AnahtarBozuk)?;
    if bit < 1024 {
        return Err(DkimHatasi::AnahtarKisa);
    }
    if bit > 4096 {
        return Err(DkimHatasi::AnahtarUzun);
    }
    Ok((bit, pkcs1.to_vec()))
}

/// PKCS#1 TLV'sini RFC 5280 SPKI'ye sarar (NULL parametreli).
fn spki_sar(pkcs1: &[u8]) -> Vec<u8> {
    fn tlv_yaz(etiket: u8, icerik: &[u8], cikti: &mut Vec<u8>) {
        cikti.push(etiket);
        match icerik.len() {
            n @ 0..=0x7f => cikti.push(n as u8),
            n @ 0x80..=0xff => cikti.extend_from_slice(&[0x81, n as u8]),
            n => cikti.extend_from_slice(&[0x82, (n >> 8) as u8, n as u8]),
        }
        cikti.extend_from_slice(icerik);
    }
    let mut algoritma = Vec::new();
    tlv_yaz(0x06, &RSA_ENCRYPTION, &mut algoritma);
    algoritma.extend_from_slice(&[0x05, 0x00]);
    let mut bit_dizisi = vec![0];
    bit_dizisi.extend_from_slice(pkcs1);
    let mut govde = Vec::new();
    tlv_yaz(0x30, &algoritma, &mut govde);
    tlv_yaz(0x03, &bit_dizisi, &mut govde);
    let mut spki = Vec::new();
    tlv_yaz(0x30, &govde, &mut spki);
    spki
}

fn karsilastir(
    algoritma: DkimAlgoritmasi,
    zee: Result<AcikAnahtar, DkimHatasi>,
    beklenen: Result<(usize, Vec<u8>), DkimHatasi>,
) {
    match (zee, beklenen) {
        (Err(z), Err(r)) => assert_eq!(z, r, "ret türü referanstan ayrılıyor"),
        (Ok(anahtar), Ok((bit, baytlar))) => {
            assert_eq!(anahtar.algoritma, algoritma);
            assert_eq!(anahtar.bit, bit, "anahtar_biti referanstan ayrılıyor");
            // Saklanan anahtar alanı özeldir; türetilmiş Debug yazımı onu
            // gösterir (yalnız açık anahtar, gizli değer değil).
            assert_eq!(
                format!("{anahtar:?}"),
                format!(
                    "AcikAnahtar {{ algoritma: {algoritma:?}, bit: {bit}, anahtar: {baytlar:?} }}"
                ),
                "saklanan anahtar baytları referanstan ayrılıyor"
            );
            match algoritma {
                DkimAlgoritmasi::Ed25519Sha256 => assert_eq!(bit, 256),
                DkimAlgoritmasi::RsaSha256 => {
                    assert!((RSA_EN_AZ_BIT..=RSA_EN_COK_BIT).contains(&bit));
                    // Aynı PKCS#1 SPKI zarfıyla ve çıplak verildiğinde aynı
                    // anahtar çıkar (iki biçim eşdeğer, RFC 6376 §3.6.1).
                    let spki = spki_sar(&baytlar);
                    assert_eq!(
                        AcikAnahtar::ayristir(algoritma, &spki).as_ref(),
                        Ok(&anahtar)
                    );
                    assert_eq!(
                        AcikAnahtar::ayristir(algoritma, &baytlar).as_ref(),
                        Ok(&anahtar)
                    );
                }
            }
            // Kabul edilmiş anahtar sahte imzayla panik yapmadan `fail` verir.
            let boy = bit.div_ceil(8);
            assert!(!anahtar.dogrula(b"x", &vec![0x5a; boy]));
        }
        (z, r) => panic!("kabul/ret referanstan ayrılıyor: zee {z:?}, referans {r:?}"),
    }
}

pub fn denetle(baytlar: &[u8]) {
    let Some((&kip, govde)) = baytlar.split_first() else {
        return;
    };
    let algoritma = if kip & 1 == 0 {
        DkimAlgoritmasi::RsaSha256
    } else {
        DkimAlgoritmasi::Ed25519Sha256
    };
    if kip & 2 == 0 {
        karsilastir(
            algoritma,
            AcikAnahtar::ayristir(algoritma, govde),
            referans(algoritma, govde),
        );
        return;
    }
    let Ok(p) = std::str::from_utf8(govde) else {
        return;
    };
    // `p=` boş ya da yalnız boşluksa anahtar iptaldir (RFC 6376 §3.6.1);
    // base64 katı MIME çözücüsüyledir (kendi kâhini `kodlama` hedefinde).
    let beklenen = if p.chars().all(|k| k.is_ascii_whitespace()) {
        Err(DkimHatasi::AnahtarIptal)
    } else {
        match dil::kodlama::base64_mime_coz(p, P_AZAMI_BAYT) {
            Ok(cozulmus) => referans(algoritma, &cozulmus),
            Err(_) => Err(DkimHatasi::AnahtarBozuk),
        }
    };
    karsilastir(algoritma, AcikAnahtar::base64ten(algoritma, p), beklenen);
}
