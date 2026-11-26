//! K-390 (RFC-0044 B2b, ADR-177): DKIM imza doğrulama ilkeli. Yalnız
//! kriptografik çekirdek host'tadır: açık anahtar (`p=`) ayrıştırma, bit
//! sınırı ve imza doğrulama. Kanonikleştirme, `h=` seçimi, `bh=` karşılaştırma,
//! `x=`/`t=` ve DMARC politikası Zee kodudur (merkez kararı (b), dogfood-minimal).
//!
//! - `rsa-sha256` (RFC 6376 §3.3.1): RSASSA-PKCS1-v1_5 + SHA-256, ring.
//!   Anahtar 1024–4096 bit (RFC 8301 §3.2 alt sınır; üst sınır CPU DoS).
//! - `ed25519-sha256` (RFC 8463): imza SHA-256(veri) üzerinde PureEdDSA;
//!   `p=` ham 32 bayt.
//! - `rsa-sha1` RFC 8301 §3.1 gereği reddedilir (ayrı tür, ayrı tanı metni).
//!
//! İmza yüzeyi (K-397, B3) aynı algoritma adlarını ve sınırları kullanır.
use crate::kodlama::base64_mime_coz;
use ring::signature::{self, UnparsedPublicKey};

/// RFC 8301 §3.2: doğrulayıcı 1024 bitten kısa RSA anahtarını kabul etmez.
pub const RSA_EN_AZ_BIT: usize = 1024;
/// 4096 bit üstü anahtar CPU DoS yüzeyidir (RFC-0044 §Değerlendirici sınırları).
pub const RSA_EN_COK_BIT: usize = 4096;
/// Çözülmüş `p=`/`b=` için üst sınır (4096 bit SPKI ≈ 550 bayt).
const AZAMI_BAYT: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DkimAlgoritmasi {
    RsaSha256,
    Ed25519Sha256,
}

impl DkimAlgoritmasi {
    /// `a=` etiketinin değeri (büyük/küçük harf duyarsız, RFC 6376 §3.5).
    pub fn ayristir(ad: &str) -> Result<Self, DkimHatasi> {
        match ad.trim().to_ascii_lowercase().as_str() {
            "rsa-sha256" => Ok(Self::RsaSha256),
            "ed25519-sha256" => Ok(Self::Ed25519Sha256),
            "rsa-sha1" => Err(DkimHatasi::AlgoritmaReddedildi),
            _ => Err(DkimHatasi::AlgoritmaBilinmiyor),
        }
    }

    pub const fn adi(self) -> &'static str {
        match self {
            Self::RsaSha256 => "rsa-sha256",
            Self::Ed25519Sha256 => "ed25519-sha256",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DkimHatasi {
    /// `rsa-sha1` (RFC 8301 §3.1).
    AlgoritmaReddedildi,
    AlgoritmaBilinmiyor,
    /// Boş `p=`: anahtar iptal edilmiş (RFC 6376 §3.6.1).
    AnahtarIptal,
    AnahtarBozuk,
    AnahtarKisa,
    AnahtarUzun,
    /// `b=` base64 değil ya da boyu algoritmaya uymuyor.
    ImzaBozuk,
    /// Zee verisinde CR var (satır sonu LF ile yazılır, CRLF host'ta kurulur).
    VeriBozuk,
    /// K-397 imza: Zee'nin verdiği ad `dkim_anahtarları` içinde yok.
    AnahtarBilinmiyor,
    /// K-397 imza: girdinin `d=`/`s=`/`a=` değeri bildirilmiş anahtara uymuyor.
    AnahtarUyusmuyor,
    /// K-413 imza: DNS'te yayımlanan `p=` özel anahtara uymadığı için anahtar
    /// açılışta devre dışı bırakıldı.
    AnahtarDnsUyusmuyor,
    /// K-397 imza: imza girdisi kâhin önlemini geçmedi (sabit neden).
    GirdiGecersiz(&'static str),
}

impl DkimHatasi {
    /// Zee Hata verisindeki `tür` değeri (C034).
    pub const fn turu(self) -> &'static str {
        match self {
            Self::AlgoritmaReddedildi => "algoritma_reddedildi",
            Self::AlgoritmaBilinmiyor => "algoritma_bilinmiyor",
            Self::AnahtarIptal => "anahtar_iptal",
            Self::AnahtarBozuk => "anahtar_bozuk",
            Self::AnahtarKisa => "anahtar_kısa",
            Self::AnahtarUzun => "anahtar_uzun",
            Self::ImzaBozuk => "imza_bozuk",
            Self::VeriBozuk => "veri_bozuk",
            Self::AnahtarBilinmiyor => "anahtar_bilinmiyor",
            Self::AnahtarUyusmuyor => "anahtar_uyuşmuyor",
            Self::AnahtarDnsUyusmuyor => "anahtar_dns_uyuşmuyor",
            Self::GirdiGecersiz(_) => "geçersiz",
        }
    }

    pub const fn mesaji(self) -> &'static str {
        match self {
            Self::AlgoritmaReddedildi => {
                "rsa-sha1 DKIM imzası kabul edilmez (RFC 8301); imza geçersiz (permerror) sayılmalı"
            }
            Self::AlgoritmaBilinmiyor => {
                "DKIM algoritması desteklenmiyor; yalnız rsa-sha256 ve ed25519-sha256"
            }
            Self::AnahtarIptal => "DKIM anahtarı iptal edilmiş (boş p=)",
            Self::AnahtarBozuk => "DKIM açık anahtarı çözülemedi",
            Self::AnahtarKisa => "DKIM RSA anahtarı 1024 bitten kısa (RFC 8301)",
            Self::AnahtarUzun => "DKIM RSA anahtarı 4096 bitten uzun",
            Self::ImzaBozuk => "DKIM imzası (b=) çözülemedi",
            Self::VeriBozuk => {
                "DKIM verisinde CR var; satır sonunu \\n ile yaz (imza girdisinde CRLF olur)"
            }
            Self::AnahtarBilinmiyor => {
                "DKIM özel anahtarı proje `dkim_anahtarları` bildiriminde yok"
            }
            Self::AnahtarUyusmuyor => {
                "DKIM imza girdisinin d=/s=/a= değeri bildirilmiş anahtara uymuyor"
            }
            Self::AnahtarDnsUyusmuyor => {
                "DKIM özel anahtarı DNS'te yayımlanan açık anahtara (p=) uymuyor; anahtar açılışta devre dışı bırakıldı"
            }
            Self::GirdiGecersiz(neden) => neden,
        }
    }

    /// IO izinde ve adaptör sınırında taşınan `tür` değerinden geri çevirir.
    pub fn turden(tur: &str) -> Option<Self> {
        [
            Self::AlgoritmaReddedildi,
            Self::AlgoritmaBilinmiyor,
            Self::AnahtarIptal,
            Self::AnahtarBozuk,
            Self::AnahtarKisa,
            Self::AnahtarUzun,
            Self::ImzaBozuk,
            Self::VeriBozuk,
            Self::AnahtarBilinmiyor,
            Self::AnahtarUyusmuyor,
            Self::AnahtarDnsUyusmuyor,
            Self::GirdiGecersiz("DKIM imza girdisi geçersiz"),
        ]
        .into_iter()
        .find(|hata| hata.turu() == tur)
    }
}

/// Doğrulamaya hazır açık anahtar. RSA'da ring'in beklediği PKCS#1
/// `RSAPublicKey` DER'i, Ed25519'da ham 32 bayt tutulur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcikAnahtar {
    pub algoritma: DkimAlgoritmasi,
    /// RSA modülünün bit uzunluğu; Ed25519 için 256.
    pub bit: usize,
    anahtar: Vec<u8>,
}

impl AcikAnahtar {
    /// `p=` etiketinin base64 değeri (FWS/boşluk atlanır).
    pub fn base64ten(algoritma: DkimAlgoritmasi, p: &str) -> Result<Self, DkimHatasi> {
        if p.chars().all(|k| k.is_ascii_whitespace()) {
            return Err(DkimHatasi::AnahtarIptal);
        }
        let bayt = base64_mime_coz(p, AZAMI_BAYT).map_err(|_| DkimHatasi::AnahtarBozuk)?;
        Self::ayristir(algoritma, &bayt)
    }

    pub fn ayristir(algoritma: DkimAlgoritmasi, bayt: &[u8]) -> Result<Self, DkimHatasi> {
        match algoritma {
            DkimAlgoritmasi::Ed25519Sha256 => {
                if bayt.len() != 32 {
                    return Err(DkimHatasi::AnahtarBozuk);
                }
                Ok(Self {
                    algoritma,
                    bit: 256,
                    anahtar: bayt.to_vec(),
                })
            }
            DkimAlgoritmasi::RsaSha256 => {
                let (pkcs1, bit) = rsa_acik_anahtari(bayt).ok_or(DkimHatasi::AnahtarBozuk)?;
                if bit < RSA_EN_AZ_BIT {
                    return Err(DkimHatasi::AnahtarKisa);
                }
                if bit > RSA_EN_COK_BIT {
                    return Err(DkimHatasi::AnahtarUzun);
                }
                Ok(Self {
                    algoritma,
                    bit,
                    anahtar: pkcs1.to_vec(),
                })
            }
        }
    }

    /// `veri` Zee'nin kanonikleştirdiği imza girdisidir (RFC 6376 §3.7:
    /// `h=` başlıkları + `b=` değeri boşaltılmış DKIM-Signature alanı).
    /// İmza tutmazsa `false`; bu bir hata değil, DKIM `fail` sonucudur.
    pub fn dogrula(&self, veri: &[u8], imza: &[u8]) -> bool {
        match self.algoritma {
            DkimAlgoritmasi::RsaSha256 => UnparsedPublicKey::new(
                &signature::RSA_PKCS1_1024_8192_SHA256_FOR_LEGACY_USE_ONLY,
                &self.anahtar,
            )
            .verify(veri, imza)
            .is_ok(),
            DkimAlgoritmasi::Ed25519Sha256 => {
                let ozet = ring::digest::digest(&ring::digest::SHA256, veri);
                UnparsedPublicKey::new(&signature::ED25519, &self.anahtar)
                    .verify(ozet.as_ref(), imza)
                    .is_ok()
            }
        }
    }
}

/// Zee Metni → kanonik DKIM baytları: her LF CRLF olur (RFC 6376 §3.4
/// kanonik biçimi CRLF'dir; Zee metin sabitinde `\r` yoktur ve satır
/// okuyucular CRLF'yi atar). Veride CR varsa ret: çift dönüşüm ve çıplak CR
/// belirsizliği yapısal olarak kapalıdır. K-397 imza tarafı da bunu kullanır.
pub fn kanonik_baytlar(veri: &str) -> Result<Vec<u8>, DkimHatasi> {
    if veri.contains('\r') {
        return Err(DkimHatasi::VeriBozuk);
    }
    Ok(veri.replace('\n', "\r\n").into_bytes())
}

/// Tek doğrulamanın sonucu; `tuttu == false` DKIM `fail`tir, hata değil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DkimSonucu {
    pub algoritma: DkimAlgoritmasi,
    pub bit: usize,
    pub tuttu: bool,
}

/// `a=`, `p=` ve `b=` değerlerini çözer, imzayı doğrular (Zee yüzeyinin
/// tek çağrısı). Algoritma, anahtar ve imza hataları imzadan önce elenir.
pub fn base64_imzayi_dogrula(
    algoritma: &str,
    p: &str,
    veri: &str,
    b: &str,
) -> Result<DkimSonucu, DkimHatasi> {
    let algoritma = DkimAlgoritmasi::ayristir(algoritma)?;
    let veri = kanonik_baytlar(veri)?;
    let anahtar = AcikAnahtar::base64ten(algoritma, p)?;
    let imza = base64_mime_coz(b, AZAMI_BAYT).map_err(|_| DkimHatasi::ImzaBozuk)?;
    let boy_uygun = match algoritma {
        DkimAlgoritmasi::Ed25519Sha256 => imza.len() == 64,
        DkimAlgoritmasi::RsaSha256 => imza.len() == anahtar.bit.div_ceil(8),
    };
    if !boy_uygun {
        return Err(DkimHatasi::ImzaBozuk);
    }
    Ok(DkimSonucu {
        algoritma,
        bit: anahtar.bit,
        tuttu: anahtar.dogrula(&veri, &imza),
    })
}

/// `bh=` biçiminde DKIM gövde özeti (RFC 6376 §3.7): Zee'nin
/// kanonikleştirdiği gövde (simple/relaxed kuralları Zee'de) `kanonik_baytlar`
/// ile CRLF'ye çevrilir, SHA-256'sı base64 yazılır. İmza tarafı (K-397) da
/// bunu kullanır; boş simple gövde Zee'de `"\n"`dir.
pub fn govde_ozeti(govde: &str) -> Result<String, DkimHatasi> {
    let baytlar = kanonik_baytlar(govde)?;
    let ozet = ring::digest::digest(&ring::digest::SHA256, &baytlar);
    Ok(crate::kodlama::base64_kodla(ozet.as_ref()))
}

/// Dosyadan gövde özetinin işlenen gövde bayt tavanı (32 MiB; dosya
/// SHA-256'sı ve K-395 çözerek yazma tavanıyla aynı).
pub const DOSYA_GOVDE_OZETI_SINIRI: usize = 32 * 1024 * 1024;

/// Adaptörlerin ortak akış döngüsü: okuyucu gövde başına konumlanmış
/// olmalı. Hata metni IO sınırının sözleşmesidir: tam `veri_bozuk`,
/// `sınır: …` ya da okuma hatası (`dosya`).
pub fn govde_ozetini_akit(
    okuyucu: &mut dyn std::io::Read,
    kip: GovdeKipi,
    yol: &str,
) -> Result<String, String> {
    let mut ozetleyici = KanonikGovdeOzetleyici::yeni(kip);
    let mut parca = [0u8; 16 * 1024];
    loop {
        let okunan = okuyucu
            .read(&mut parca)
            .map_err(|hata| format!("\"{yol}\" dosyası okunamadı: {}", hata.kind()))?;
        if okunan == 0 {
            return ozetleyici.bitir().map_err(|hata| hata.turu().to_string());
        }
        if ozetleyici.bayt + okunan as u64 > DOSYA_GOVDE_OZETI_SINIRI as u64 {
            return Err("sınır: DKIM gövdesi 32 MiB'ı aşıyor".into());
        }
        ozetleyici
            .ekle(&parca[..okunan])
            .map_err(|hata| hata.turu().to_string())?;
    }
}

/// RFC 6376 §3.4 `c=` gövde kipi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovdeKipi {
    /// §3.4.3: sondaki boş satırlar atılır; boş gövde tek CRLF'dir.
    Simple,
    /// §3.4.4: satır sonu boşlukları atılır, satır içi WSP dizileri tek SP
    /// olur, sondaki boş satırlar atılır; boş gövde boştur.
    Relaxed,
}

impl GovdeKipi {
    pub fn ayristir(ad: &str) -> Option<Self> {
        match ad {
            "simple" => Some(Self::Simple),
            "relaxed" => Some(Self::Relaxed),
            _ => None,
        }
    }
}

/// K-397 B3 (ADR-174 §8, ZM-F046): dosyadan akışlı kanonik gövde özeti.
/// Gövde kanonikleştirmesi (yalnız gövde; başlıklar Zee'de) burada yapılır.
/// Satır sonu LF ya da CRLF olabilir; LF'siz CR `VeriBozuk`tur. Özet
/// kanonik CRLF baytları üzerinde alınır; gövde bellekte tutulmaz.
pub struct KanonikGovdeOzetleyici {
    kip: GovdeKipi,
    ozet: ring::digest::Context,
    tampon: Vec<u8>,
    /// Henüz yazılmamış boş satırlar (sonda kalırsa atılır).
    bekleyen_bos: u64,
    satir_dolu: bool,
    bosluk: bool,
    cr: bool,
    cikti_var: bool,
    /// İşlenen girdi baytı.
    pub bayt: u64,
}

impl KanonikGovdeOzetleyici {
    pub fn yeni(kip: GovdeKipi) -> Self {
        Self {
            kip,
            ozet: ring::digest::Context::new(&ring::digest::SHA256),
            tampon: Vec::with_capacity(16 * 1024),
            bekleyen_bos: 0,
            satir_dolu: false,
            bosluk: false,
            cr: false,
            cikti_var: false,
            bayt: 0,
        }
    }

    fn yaz(&mut self, veri: &[u8]) {
        self.cikti_var = true;
        self.tampon.extend_from_slice(veri);
        if self.tampon.len() >= 16 * 1024 {
            self.ozet.update(&self.tampon);
            self.tampon.clear();
        }
    }

    fn satir_sonu(&mut self) {
        if self.satir_dolu {
            self.yaz(b"\r\n");
        } else {
            self.bekleyen_bos += 1;
        }
        self.satir_dolu = false;
        self.bosluk = false;
    }

    pub fn ekle(&mut self, parca: &[u8]) -> Result<(), DkimHatasi> {
        for &b in parca {
            if self.cr {
                if b != b'\n' {
                    return Err(DkimHatasi::VeriBozuk);
                }
                self.cr = false;
                self.satir_sonu();
                continue;
            }
            match b {
                b'\r' => self.cr = true,
                b'\n' => self.satir_sonu(),
                b' ' | b'\t' if self.kip == GovdeKipi::Relaxed => self.bosluk = true,
                _ => {
                    if !self.satir_dolu {
                        while self.bekleyen_bos > 0 {
                            self.bekleyen_bos -= 1;
                            self.yaz(b"\r\n");
                        }
                        self.satir_dolu = true;
                    }
                    if self.bosluk {
                        self.bosluk = false;
                        self.yaz(b" ");
                    }
                    self.yaz(&[b]);
                }
            }
        }
        self.bayt += parca.len() as u64;
        Ok(())
    }

    /// base64 SHA-256 (`bh=` biçimi).
    pub fn bitir(mut self) -> Result<String, DkimHatasi> {
        if self.cr {
            return Err(DkimHatasi::VeriBozuk);
        }
        if self.satir_dolu {
            self.yaz(b"\r\n");
        }
        if self.kip == GovdeKipi::Simple && !self.cikti_var {
            self.yaz(b"\r\n");
        }
        self.ozet.update(&self.tampon);
        Ok(crate::kodlama::base64_kodla(self.ozet.finish().as_ref()))
    }
}

// ---- en küçük DER okuyucusu (yalnız kesin uzunluklu, DER-kanonik) ----

/// `(etiket, içerik, kalan)`; uzun biçim uzunlukta kanonik olmayan
/// kodlama (gereksiz baştaki sıfır, 128'den küçük değer) reddedilir.
fn der_oge(girdi: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&etiket, girdi) = girdi.split_first()?;
    let (&ilk, mut girdi) = girdi.split_first()?;
    let boy = if ilk < 0x80 {
        usize::from(ilk)
    } else {
        let n = usize::from(ilk & 0x7f);
        if n == 0 || n > 2 || girdi.len() < n || girdi[0] == 0 {
            return None;
        }
        let boy = girdi[..n]
            .iter()
            .fold(0usize, |t, b| (t << 8) | usize::from(*b));
        girdi = &girdi[n..];
        if boy < 0x80 {
            return None;
        }
        boy
    };
    (girdi.len() >= boy).then(|| (etiket, &girdi[..boy], &girdi[boy..]))
}

/// rsaEncryption OID'i 1.2.840.113549.1.1.1.
const RSA_OID: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01];

/// SPKI (RFC 6376 §3.6.1 `p=`) ya da çıplak PKCS#1 `RSAPublicKey` kabul
/// eder; PKCS#1 DER'ini ve modül bit uzunluğunu döner.
fn rsa_acik_anahtari(bayt: &[u8]) -> Option<(&[u8], usize)> {
    let (0x30, govde, []) = der_oge(bayt)? else {
        return None;
    };
    let pkcs1 = match der_oge(govde)? {
        (0x02, _, _) => bayt,
        (0x30, algoritma, kalan) => {
            let (0x06, oid, parametre) = der_oge(algoritma)? else {
                return None;
            };
            if oid != RSA_OID || !(parametre.is_empty() || parametre == [0x05, 0x00]) {
                return None;
            }
            let (0x03, bit_dizisi, []) = der_oge(kalan)? else {
                return None;
            };
            let (&0, ic) = bit_dizisi.split_first()? else {
                return None;
            };
            ic
        }
        _ => return None,
    };
    let (0x30, anahtar, []) = der_oge(pkcs1)? else {
        return None;
    };
    let (0x02, modul, kalan) = der_oge(anahtar)? else {
        return None;
    };
    let (0x02, ust, []) = der_oge(kalan)? else {
        return None;
    };
    if ust.is_empty() || modul.is_empty() || modul[0] & 0x80 != 0 {
        return None;
    }
    let modul = match modul {
        [0, kalan @ ..] if kalan.first().is_some_and(|b| b & 0x80 != 0) => kalan,
        [0, ..] => return None,
        _ => modul,
    };
    let bit = modul.len() * 8 - modul[0].leading_zeros() as usize;
    Some((pkcs1, bit))
}

#[cfg(test)]
mod testler {
    use super::*;

    fn kanonik(kip: GovdeKipi, parcalar: &[&[u8]]) -> Result<String, DkimHatasi> {
        let mut ozetleyici = KanonikGovdeOzetleyici::yeni(kip);
        for parca in parcalar {
            ozetleyici.ekle(parca)?;
        }
        ozetleyici.bitir()
    }

    fn sha(bayt: &[u8]) -> String {
        crate::kodlama::base64_kodla(ring::digest::digest(&ring::digest::SHA256, bayt).as_ref())
    }

    #[test]
    fn kanonik_govde_rfc6376_ornekleri_ve_kenarlari() {
        use GovdeKipi::*;
        // RFC 6376 §3.4.5 örneği.
        let ornek: &[u8] = b" C \r\nD \t E\r\n\r\n\r\n";
        assert_eq!(kanonik(Relaxed, &[ornek]).unwrap(), sha(b" C\r\nD E\r\n"));
        assert_eq!(
            kanonik(Simple, &[ornek]).unwrap(),
            sha(b" C \r\nD \t E\r\n")
        );
        // Boş gövde (§3.4.3/§3.4.4) ve yalnız boş satırlar.
        for bos in [&b""[..], b"\r\n", b"\n\n\r\n"] {
            assert_eq!(
                kanonik(Simple, &[bos]).unwrap(),
                "frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY="
            );
            assert_eq!(
                kanonik(Relaxed, &[bos]).unwrap(),
                "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="
            );
        }
        // Relaxed'de yalnız boşluklu satır boştur; simple'da değildir.
        assert_eq!(kanonik(Relaxed, &[b"a\n \t\n"]).unwrap(), sha(b"a\r\n"));
        assert_eq!(
            kanonik(Simple, &[b"a\n \t\n"]).unwrap(),
            sha(b"a\r\n \t\r\n")
        );
        // Ara boş satırlar korunur, sonda satır sonu eklenir.
        assert_eq!(
            kanonik(Simple, &[b"a\n\nb"]).unwrap(),
            sha(b"a\r\n\r\nb\r\n")
        );
        assert_eq!(kanonik(Relaxed, &[b"  a  b  "]).unwrap(), sha(b" a b\r\n"));
    }

    #[test]
    fn kanonik_govde_lf_crlf_ve_parca_siniri_esdegerdir() {
        let crlf = b"Satir 1 \t x\r\n\r\n  ikinci\r\n\r\n";
        let lf = b"Satir 1 \t x\n\n  ikinci\n\n";
        for kip in [GovdeKipi::Simple, GovdeKipi::Relaxed] {
            let beklenen = kanonik(kip, &[crlf]).unwrap();
            assert_eq!(kanonik(kip, &[lf]).unwrap(), beklenen);
            for kes in 0..=crlf.len() {
                assert_eq!(
                    kanonik(kip, &[&crlf[..kes], &crlf[kes..]]).unwrap(),
                    beklenen,
                    "{kes}"
                );
            }
        }
        // Simple + LF: K-390 Metin ilkesiyle aynı (Zee'nin kanonik gövdesi).
        let govde = "Merhaba\n\nDünya\n";
        assert_eq!(
            kanonik(GovdeKipi::Simple, &[govde.as_bytes()]).unwrap(),
            govde_ozeti(govde).unwrap()
        );
        for bozuk in [&b"a\rb\n"[..], b"a\r", b"\r\r\n"] {
            assert_eq!(
                kanonik(GovdeKipi::Simple, &[bozuk]),
                Err(DkimHatasi::VeriBozuk)
            );
        }
        assert_eq!(
            kanonik(GovdeKipi::Relaxed, &[b"a\r", b"\nb"]).unwrap(),
            sha(b"a\r\nb\r\n")
        );
        assert_eq!(GovdeKipi::ayristir("Relaxed"), None);
    }

    #[test]
    fn algoritma_adlari() {
        assert_eq!(
            DkimAlgoritmasi::ayristir("RSA-SHA256"),
            Ok(DkimAlgoritmasi::RsaSha256)
        );
        assert_eq!(
            DkimAlgoritmasi::ayristir("ed25519-sha256"),
            Ok(DkimAlgoritmasi::Ed25519Sha256)
        );
        assert_eq!(
            DkimAlgoritmasi::ayristir("rsa-sha1"),
            Err(DkimHatasi::AlgoritmaReddedildi)
        );
        assert_eq!(
            DkimAlgoritmasi::ayristir("rsa-sha512"),
            Err(DkimHatasi::AlgoritmaBilinmiyor)
        );
    }

    #[test]
    fn der_okuyucusu_kanonik_olmayan_ve_kesik_girdiyi_reddeder() {
        assert!(der_oge(&[0x30]).is_none());
        assert!(der_oge(&[0x30, 0x05, 0x00]).is_none());
        assert!(der_oge(&[0x30, 0x81, 0x05, 0, 0, 0, 0, 0]).is_none());
        assert!(der_oge(&[0x30, 0x82, 0x00, 0x80]).is_none());
        assert!(der_oge(&[0x30, 0x80]).is_none());
        assert!(rsa_acik_anahtari(&[0x30, 0x00]).is_none());
        assert!(rsa_acik_anahtari(&[]).is_none());
    }

    #[test]
    fn bos_p_iptal_ed25519_boyu_ve_bozuk_base64() {
        assert_eq!(
            AcikAnahtar::base64ten(DkimAlgoritmasi::RsaSha256, " \r\n "),
            Err(DkimHatasi::AnahtarIptal)
        );
        assert_eq!(
            AcikAnahtar::base64ten(DkimAlgoritmasi::Ed25519Sha256, "AAAA"),
            Err(DkimHatasi::AnahtarBozuk)
        );
        assert_eq!(
            AcikAnahtar::base64ten(DkimAlgoritmasi::RsaSha256, "%%%"),
            Err(DkimHatasi::AnahtarBozuk)
        );
        // RFC 6376 §3.4.3/§3.4.4 boş gövde ve Ek A.2 örnek gövdesi.
        assert_eq!(
            govde_ozeti("\n").unwrap(),
            "frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY="
        );
        assert_eq!(
            govde_ozeti("").unwrap(),
            "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="
        );
        assert_eq!(
            govde_ozeti("Hi.\n\nWe lost the game. Are you hungry yet?\n\nJoe.\n").unwrap(),
            "2jUSOH9NhtVGCQWNr9BrIAPreKQjO6Sn7XIkfJVOzv8="
        );
        assert_eq!(govde_ozeti("a\r\n"), Err(DkimHatasi::VeriBozuk));
        assert_eq!(kanonik_baytlar("a:b\nc:d").unwrap(), b"a:b\r\nc:d");
        assert_eq!(kanonik_baytlar("a\r\n"), Err(DkimHatasi::VeriBozuk));
    }

    /// Ed25519: ring doğrulamasını bağımsız `ed25519-dalek` imzasıyla sınar
    /// (diferansiyel; RFC 8463 PureEdDSA, SHA-256 özeti üzerinde).
    #[test]
    fn ed25519_dalek_imzasi_dogrulanir_degisiklik_reddedilir() {
        use ed25519_dalek::{Signer, SigningKey};
        let anahtar = SigningKey::from_bytes(&[7u8; 32]);
        let veri = b"from:Zee <a@example.com>\r\ndkim-signature:v=1; a=ed25519-sha256; b=";
        let ozet = ring::digest::digest(&ring::digest::SHA256, veri);
        let imza = anahtar.sign(ozet.as_ref()).to_bytes();
        let acik = AcikAnahtar::ayristir(
            DkimAlgoritmasi::Ed25519Sha256,
            anahtar.verifying_key().as_bytes(),
        )
        .unwrap();
        assert!(acik.dogrula(veri, &imza));
        let mut degisik = veri.to_vec();
        degisik[5] ^= 1;
        assert!(!acik.dogrula(&degisik, &imza));
        let mut bozuk = imza;
        bozuk[0] ^= 1;
        assert!(!acik.dogrula(veri, &bozuk));
        // Özetsiz (ham veri) imza RFC 8463'e uymaz → fail.
        let ham = anahtar.sign(veri).to_bytes();
        assert!(!acik.dogrula(veri, &ham));
    }

    /// Geçerli SPKI'nin 20 000 deterministik bozulmuş kopyası paniksiz
    /// ayrıştırılır; kabul edilen her anahtar 1024–4096 bit sınırındadır.
    #[test]
    fn bozulmus_spki_paniksiz_ve_sinir_icinde() {
        let mut spki = vec![0x30, 0x81, 0x9f, 0x30, 0x0d, 0x06, 0x09];
        spki.extend_from_slice(RSA_OID);
        spki.extend_from_slice(&[0x05, 0x00, 0x03, 0x81, 0x8d, 0x00]);
        spki.extend_from_slice(&[0x30, 0x81, 0x89, 0x02, 0x81, 0x81, 0x00]);
        spki.extend((0..128).map(|i| 0x80 | i as u8));
        spki.extend_from_slice(&[0x02, 0x03, 0x01, 0x00, 0x01]);
        let (_, bit) = rsa_acik_anahtari(&spki).expect("taban geçerli");
        assert_eq!(bit, 1024);
        let mut durum: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut sonraki = || {
            durum ^= durum << 13;
            durum ^= durum >> 7;
            durum ^= durum << 17;
            durum
        };
        for _ in 0..20_000 {
            let mut girdi = spki.clone();
            for _ in 0..(sonraki() % 4 + 1) {
                let konum = (sonraki() as usize) % girdi.len();
                match sonraki() % 3 {
                    0 => girdi[konum] = sonraki() as u8,
                    1 => girdi.truncate(konum),
                    _ => girdi.insert(konum, sonraki() as u8),
                }
                if girdi.is_empty() {
                    break;
                }
            }
            if let Ok(anahtar) = AcikAnahtar::ayristir(DkimAlgoritmasi::RsaSha256, &girdi) {
                assert!((RSA_EN_AZ_BIT..=RSA_EN_COK_BIT).contains(&anahtar.bit));
                assert!(!anahtar.dogrula(b"x", &[0u8; 128]));
            }
        }
    }
}
