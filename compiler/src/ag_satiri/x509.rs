//! K-397 (RFC-0044 B4a, ADR-173): X.509 sertifikasından `SubjectPublicKeyInfo`
//! DER diliminin çıkarımı (DANE TLSA seçici 1 özeti için).
//!
//! Kapsam bilinçli olarak dardır: yalnız `Certificate → tbsCertificate`
//! içinde sırayla `[0] version` (isteğe bağlı), `serialNumber`, `signature`,
//! `issuer`, `validity`, `subject` atlanır ve yedinci öğe `subjectPublicKeyInfo`
//! SEQUENCE'ı tam TLV olarak döner. Sertifika bu çağrıdan önce rustls
//! tarafından ayrıştırılmış ve el sıkışma imzası doğrulanmıştır; yine de
//! yürüyücü saldırganın gönderdiği bayt üzerinde çalıştığı için her uzunluk
//! sınır denetimlidir: belirsiz uzunluk yok, uzun biçim en çok 4 bayt,
//! kanonik olmayan (gereksiz uzun) uzunluk ret, girdi en çok 64 KiB,
//! `unsafe` yok, panik yok.

/// Kabul edilen en büyük sertifika DER'i.
pub const AZAMI_SERTIFIKA_BAYTI: usize = 64 * 1024;

const SEQUENCE: u8 = 0x30;
const SURUM: u8 = 0xa0;

/// Bir TLV'nin (etiket, içerik başlangıcı, içerik sonu) konumları.
fn tlv(veri: &[u8], bas: usize) -> Option<(u8, usize, usize)> {
    let etiket = *veri.get(bas)?;
    // Yüksek etiket numarası (0x1f) X.509 iskeletinde yok.
    if etiket & 0x1f == 0x1f {
        return None;
    }
    let ilk = *veri.get(bas.checked_add(1)?)? as usize;
    let (uzunluk, baslik) = if ilk < 0x80 {
        (ilk, 2)
    } else {
        let adet = ilk & 0x7f;
        if adet == 0 || adet > 4 {
            return None; // belirsiz uzunluk ya da 4 GiB üstü
        }
        let mut uzunluk = 0usize;
        for sira in 0..adet {
            let bayt = *veri.get(bas.checked_add(2 + sira)?)? as usize;
            if sira == 0 && bayt == 0 {
                return None; // kanonik olmayan baştaki sıfır
            }
            uzunluk = uzunluk.checked_mul(256)?.checked_add(bayt)?;
        }
        if uzunluk < 0x80 {
            return None; // kısa biçimle yazılmalıydı (DER)
        }
        (uzunluk, 2 + adet)
    };
    let icerik = bas.checked_add(baslik)?;
    let son = icerik.checked_add(uzunluk)?;
    (son <= veri.len()).then_some((etiket, icerik, son))
}

/// Sertifika DER'inden `SubjectPublicKeyInfo` TLV dilimi.
pub fn spki_dilimi(sertifika: &[u8]) -> Option<&[u8]> {
    if sertifika.len() > AZAMI_SERTIFIKA_BAYTI {
        return None;
    }
    let (etiket, ic, son) = tlv(sertifika, 0)?;
    if etiket != SEQUENCE || son != sertifika.len() {
        return None;
    }
    let (etiket, mut konum, tbs_son) = tlv(sertifika, ic)?;
    if etiket != SEQUENCE {
        return None;
    }
    let tbs = &sertifika[..tbs_son];
    if tbs.get(konum) == Some(&SURUM) {
        konum = tlv(tbs, konum)?.2;
    }
    // serialNumber, signature, issuer, validity, subject.
    for _ in 0..5 {
        konum = tlv(tbs, konum)?.2;
    }
    let (etiket, _, spki_son) = tlv(tbs, konum)?;
    (etiket == SEQUENCE).then(|| &tbs[konum..spki_son])
}

#[cfg(test)]
mod testler {
    use super::*;

    fn tlv_yaz(etiket: u8, icerik: &[u8]) -> Vec<u8> {
        let mut cikti = vec![etiket];
        let n = icerik.len();
        if n < 0x80 {
            cikti.push(n as u8);
        } else if n < 0x100 {
            cikti.extend([0x81, n as u8]);
        } else {
            cikti.extend([0x82, (n >> 8) as u8, n as u8]);
        }
        cikti.extend_from_slice(icerik);
        cikti
    }

    fn sertifika(surumlu: bool, spki: &[u8]) -> Vec<u8> {
        let mut tbs = Vec::new();
        if surumlu {
            tbs.extend(tlv_yaz(SURUM, &tlv_yaz(0x02, &[2])));
        }
        tbs.extend(tlv_yaz(0x02, &[0x01, 0x23]));
        tbs.extend(tlv_yaz(SEQUENCE, &tlv_yaz(0x06, &[0x2a])));
        tbs.extend(tlv_yaz(SEQUENCE, b"yayinci"));
        tbs.extend(tlv_yaz(SEQUENCE, b"gecerlilik"));
        tbs.extend(tlv_yaz(SEQUENCE, &[0u8; 200]));
        tbs.extend_from_slice(spki);
        tbs.extend(tlv_yaz(0xa3, b"uzantilar"));
        let mut govde = tlv_yaz(SEQUENCE, &tbs);
        govde.extend(tlv_yaz(SEQUENCE, &tlv_yaz(0x06, &[0x2a])));
        govde.extend(tlv_yaz(0x03, &[0, 1, 2]));
        tlv_yaz(SEQUENCE, &govde)
    }

    #[test]
    fn spki_surumlu_ve_surumsuz_sertifikada_tam_tlv_doner() {
        let spki = tlv_yaz(SEQUENCE, &[0x55; 300]);
        for surumlu in [true, false] {
            let der = sertifika(surumlu, &spki);
            assert_eq!(spki_dilimi(&der), Some(spki.as_slice()), "{surumlu}");
        }
    }

    #[test]
    fn bozuk_kesik_ve_kanonik_olmayan_der_reddedilir() {
        let spki = tlv_yaz(SEQUENCE, &[0x55; 40]);
        let der = sertifika(true, &spki);
        for kes in [0, 1, 2, 10, der.len() / 2, der.len() - 1] {
            assert_eq!(spki_dilimi(&der[..kes]), None, "{kes}");
        }
        let mut fazla = der.clone();
        fazla.push(0);
        assert_eq!(spki_dilimi(&fazla), None, "sonda artık bayt");
        assert_eq!(spki_dilimi(&[0x30, 0x80, 0, 0]), None, "belirsiz uzunluk");
        assert_eq!(
            spki_dilimi(&[0x30, 0x81, 0x05, 1, 2, 3, 4, 5]),
            None,
            "uzun biçim < 128"
        );
        assert_eq!(
            spki_dilimi(&[0x30, 0x82, 0x00, 0x90]),
            None,
            "baştaki sıfır"
        );
        assert_eq!(
            spki_dilimi(&[0x30, 0x85, 1, 1, 1, 1, 1]),
            None,
            "5 baytlık uzunluk"
        );
        assert_eq!(spki_dilimi(&[0x3f, 0x00]), None, "yüksek etiket");
        let spki_degil = sertifika(true, &tlv_yaz(0x04, &[1, 2, 3]));
        assert_eq!(spki_dilimi(&spki_degil), None, "SPKI SEQUENCE değil");
        assert_eq!(spki_dilimi(&vec![0u8; AZAMI_SERTIFIKA_BAYTI + 1]), None);
    }

    #[test]
    fn rastgele_kirpma_ve_bayt_cevirme_paniklemez() {
        let der = sertifika(true, &tlv_yaz(SEQUENCE, &[0x55; 64]));
        let mut durum = 0x2545_f491_u32;
        for _ in 0..5000 {
            durum ^= durum << 13;
            durum ^= durum >> 17;
            durum ^= durum << 5;
            let mut bozuk = der.clone();
            let yer = durum as usize % bozuk.len();
            bozuk[yer] ^= (durum >> 8) as u8 | 1;
            let son = (durum as usize >> 4) % (bozuk.len() + 1);
            let _ = spki_dilimi(&bozuk[..son]);
        }
    }
}
