//! K-397 Dilim 2 (RFC-0044 B3, ADR-174): DKIM imza ilkeli.
//!
//! Kanonikleştirme, başlık seçimi ve `bh=` Zee kodudur (ürünün B2
//! doğrulayıcısıyla aynı kod). Host yalnız iki şey yapar:
//!
//! 1. **İmza kâhini önlemi.** Girdi RFC 6376 §3.7 imza girdisidir:
//!    kanonikleştirilmiş başlıklar ve sonda `b=` boş, satır sonu olmayan
//!    `DKIM-Signature` alanı. Host kanonikleştirmeyi bilmeden yapıyı denetler:
//!    son alan `dkim-signature`; etiketler tekil; `v=1`, `bh=` dolu, `h=`
//!    `from` içerir, `l=` yok, `b=` boş ve son etiket; girdide tam bir
//!    `from` alanı var ve adresinin alanı `d=` ya da alt alanı (gevşek
//!    hizalama); `i=` varsa `d=` altında. `d=`/`s=`/`a=` bildirilmiş anahtarla
//!    birebir eşleşir. Böylece anahtar başka alan adına ya da keyfî bayta
//!    imza atmaz.
//! 2. **Kriptografi.** `rsa-sha256` (RSASSA-PKCS1-v1_5, 2048–4096 bit) ve
//!    `ed25519-sha256` (RFC 8463: SHA-256(veri) üzerinde PureEdDSA); `ring`.
//!
//! Özel anahtar PKCS#8 PEM'dir, açılışta bir kez çözülür; okunan PEM ve DER
//! tamponları sıfırlanır (en iyi çaba: `ring` anahtar çifti süreç belleğinde
//! kalır). Anahtar baytı hiçbir hata metnine girmez; metinler yalnız sabit
//! nedendir, çağıran seçiciyi ekler.
use crate::dkim_kripto::{kanonik_baytlar, DkimAlgoritmasi, DkimHatasi};
use crate::kodlama::{base64_kodla, base64_mime_coz};
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, RsaKeyPair, RSA_PKCS1_SHA256};

/// RFC 8301 §3.2 imzalayan için SHOULD 2048; burada ZORUNLU (RFC-0044 B3).
pub const RSA_IMZA_EN_AZ_BIT: usize = 2048;
/// Kabul edilen en büyük imza girdisi (başlık bloğu).
pub const AZAMI_IMZA_GIRDISI: usize = 256 * 1024;
/// PEM okuma üst sınırı (4096 bit PKCS#8 ≈ 3,3 KiB PEM).
pub const AZAMI_PEM_BAYTI: usize = 16 * 1024;

const PEM_BASI: &str = "-----BEGIN PRIVATE KEY-----";
const PEM_SONU: &str = "-----END PRIVATE KEY-----";

/// İmza girdisinden çıkan ve bildirimle karşılaştırılan etiketler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImzaEtiketleri {
    /// `d=`, küçük harf.
    pub alan: String,
    /// `s=`, küçük harf.
    pub secici: String,
    pub algoritma: DkimAlgoritmasi,
}

fn gecersiz(neden: &'static str) -> DkimHatasi {
    DkimHatasi::GirdiGecersiz(neden)
}

/// ASCII DNS adı: boş olmayan etiketler, harf/rakam/tire/alt çizgi.
fn alan_adi_mi(ad: &str) -> bool {
    !ad.is_empty()
        && ad.len() <= 253
        && ad.split('.').all(|etiket| {
            !etiket.is_empty()
                && etiket.len() <= 63
                && etiket
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
}

fn hizali(alt: &str, ust: &str) -> bool {
    alt == ust || alt.strip_suffix(ust).is_some_and(|on| on.ends_with('.'))
}

/// Başlık alanlarını (ad, ham değer) olarak böler; devam satırı (SP/HT ile
/// başlayan) önceki alana aittir. Boş satır, adsız ya da iki noktasız alan ret.
fn alanlar(veri: &str) -> Result<Vec<(String, String)>, DkimHatasi> {
    let mut sonuc: Vec<(String, String)> = Vec::new();
    for satir in veri.split('\n') {
        if satir.starts_with([' ', '\t']) {
            let (_, deger) = sonuc
                .last_mut()
                .ok_or(gecersiz("DKIM imza girdisi devam satırıyla başlıyor"))?;
            deger.push('\n');
            deger.push_str(satir);
            continue;
        }
        let (ad, deger) = satir.split_once(':').ok_or(gecersiz(
            "DKIM imza girdisinde başlık alanı olmayan satır var",
        ))?;
        let ad = ad.trim_end_matches([' ', '\t']);
        if ad.is_empty() || !ad.bytes().all(|b| (33..=126).contains(&b)) {
            return Err(gecersiz("DKIM imza girdisinde geçersiz başlık adı var"));
        }
        sonuc.push((ad.to_ascii_lowercase(), deger.to_string()));
    }
    Ok(sonuc)
}

/// RFC 6376 §3.2 etiket listesi; son etiket `b=` olmalı (değeri boş).
fn etiket_listesi(deger: &str) -> Result<Vec<(String, String)>, DkimHatasi> {
    let acik: String = deger.chars().filter(|k| *k != '\n').collect();
    let mut etiketler: Vec<(String, String)> = Vec::new();
    for parca in acik.split(';') {
        let (ad, deger) = parca
            .split_once('=')
            .ok_or(gecersiz("DKIM-Signature etiket listesi bozuk"))?;
        let ad = ad.trim_matches([' ', '\t']);
        let gecerli_ad = ad.bytes().next().is_some_and(|b| b.is_ascii_alphabetic())
            && ad.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
        if !gecerli_ad || etiketler.iter().any(|(var, _)| var == ad) {
            return Err(gecersiz(
                "DKIM-Signature etiket adı geçersiz ya da yinelenmiş",
            ));
        }
        etiketler.push((ad.into(), deger.trim_matches([' ', '\t']).into()));
    }
    match etiketler.last() {
        Some((ad, deger)) if ad == "b" && deger.is_empty() => Ok(etiketler),
        _ => Err(gecersiz("DKIM-Signature'ın son etiketi boş `b=` olmalı")),
    }
}

/// `From:` değerinden adres alanı: tırnak ve yorum dışındaki metinde ya tek
/// `<…>` ya da tek çıplak adres; tek `@`. Liste, grup ve belirsiz biçim ret.
fn gonderen_alani(deger: &str) -> Result<String, DkimHatasi> {
    let hata = || gecersiz("From adresi tek ve açık bir posta kutusu değil");
    let mut acik = String::new();
    let (mut tirnak, mut yorum, mut kacis) = (false, 0usize, false);
    for k in deger.chars().filter(|k| *k != '\n') {
        if kacis {
            kacis = false;
        } else if k == '\\' && (tirnak || yorum > 0) {
            kacis = true;
        } else if tirnak {
            tirnak = k != '"';
        } else if k == '"' && yorum == 0 {
            tirnak = true;
        } else if k == '(' {
            yorum += 1;
        } else if k == ')' && yorum > 0 {
            yorum -= 1;
        } else if yorum == 0 {
            acik.push(k);
        }
    }
    if tirnak || yorum > 0 {
        return Err(hata());
    }
    let adres = match (acik.find('<'), acik.rfind('>')) {
        (None, None) => acik.trim(),
        (Some(bas), Some(son)) if bas < son => {
            let ic = &acik[bas + 1..son];
            if ic.contains(['<', '>']) || !acik[son + 1..].trim().is_empty() {
                return Err(hata());
            }
            if acik[..bas].contains(['@', ',', ':', ';']) {
                return Err(hata());
            }
            ic.trim()
        }
        _ => return Err(hata()),
    };
    if adres.contains([',', ';', ':', ' ', '\t']) || adres.matches('@').count() != 1 {
        return Err(hata());
    }
    let alan = adres
        .rsplit_once('@')
        .map(|(_, alan)| alan.trim_end_matches('.').to_ascii_lowercase())
        .unwrap_or_default();
    if !alan_adi_mi(&alan) {
        return Err(hata());
    }
    Ok(alan)
}

/// İmza kâhini önlemi: girdinin yapısını ve From hizalamasını denetler,
/// `d=`/`s=`/`a=` değerlerini döner (saf; anahtar gerektirmez).
pub fn imza_girdisini_denetle(veri: &str) -> Result<ImzaEtiketleri, DkimHatasi> {
    if veri.len() > AZAMI_IMZA_GIRDISI {
        return Err(gecersiz("DKIM imza girdisi 256 KiB sınırını aşıyor"));
    }
    if veri.contains('\r') {
        return Err(DkimHatasi::VeriBozuk);
    }
    if veri.ends_with('\n') {
        return Err(gecersiz(
            "DKIM imza girdisi satır sonuyla bitmemeli (son alan CRLF'siz DKIM-Signature)",
        ));
    }
    let alanlar = alanlar(veri)?;
    let Some(((son_ad, son_deger), onceki)) = alanlar.split_last() else {
        return Err(gecersiz("DKIM imza girdisi boş"));
    };
    if son_ad != "dkim-signature" {
        return Err(gecersiz(
            "DKIM imza girdisinin son alanı DKIM-Signature değil",
        ));
    }
    let etiketler = etiket_listesi(son_deger)?;
    let etiket = |ad: &str| {
        etiketler
            .iter()
            .find(|(var, _)| var == ad)
            .map(|(_, deger)| deger.as_str())
    };
    if etiket("v") != Some("1") {
        return Err(gecersiz("DKIM-Signature `v=1` taşımalı"));
    }
    if etiket("l").is_some() {
        return Err(gecersiz("DKIM imzasında `l=` kullanılmaz (RFC-0044 B3)"));
    }
    if etiket("bh").is_none_or(str::is_empty) {
        return Err(gecersiz("DKIM-Signature `bh=` gövde özetini taşımalı"));
    }
    let imzali_from = etiket("h").is_some_and(|h| {
        h.split(':')
            .any(|ad| ad.trim_matches([' ', '\t']).eq_ignore_ascii_case("from"))
    });
    if !imzali_from {
        return Err(gecersiz("DKIM-Signature `h=` listesi `from` içermeli"));
    }
    let algoritma = DkimAlgoritmasi::ayristir(etiket("a").unwrap_or_default())?;
    let alan = etiket("d").unwrap_or_default().to_ascii_lowercase();
    let secici = etiket("s").unwrap_or_default().to_ascii_lowercase();
    if !alan_adi_mi(&alan) || !alan_adi_mi(&secici) {
        return Err(gecersiz(
            "DKIM-Signature `d=` ya da `s=` geçerli bir DNS adı değil",
        ));
    }
    if let Some(kimlik) = etiket("i") {
        let kimlik_alani = kimlik
            .rsplit_once('@')
            .map(|(_, alan)| alan.to_ascii_lowercase())
            .unwrap_or_default();
        if !hizali(&kimlik_alani, &alan) {
            return Err(gecersiz("DKIM-Signature `i=` alanı `d=` altında değil"));
        }
    }
    let mut gonderenler = onceki.iter().filter(|(ad, _)| ad == "from");
    let (Some((_, from)), None) = (gonderenler.next(), gonderenler.next()) else {
        return Err(gecersiz("DKIM imza girdisinde tam bir From alanı olmalı"));
    };
    if !hizali(&gonderen_alani(from)?, &alan) {
        return Err(gecersiz(
            "From adresinin alanı DKIM `d=` alanı ya da alt alanı değil",
        ));
    }
    Ok(ImzaEtiketleri {
        alan,
        secici,
        algoritma,
    })
}

enum AnahtarCifti {
    Rsa(RsaKeyPair),
    Ed25519(Ed25519KeyPair),
}

/// Açılışta yüklenmiş tek DKIM imza anahtarı.
pub struct DkimImzaAnahtari {
    pub alan: String,
    pub secici: String,
    pub algoritma: DkimAlgoritmasi,
    /// RSA modül biti; Ed25519 için 256.
    pub bit: usize,
    cift: AnahtarCifti,
}

impl std::fmt::Debug for DkimImzaAnahtari {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DkimImzaAnahtari")
            .field("alan", &self.alan)
            .field("secici", &self.secici)
            .field("algoritma", &self.algoritma)
            .field("bit", &self.bit)
            .finish_non_exhaustive()
    }
}

fn sifirla(tampon: &mut [u8]) {
    tampon.fill(0);
    std::hint::black_box(tampon);
}

/// Tek `PRIVATE KEY` bloğu (PKCS#8, şifresiz) → DER. Blok dışında yalnız boşluk.
fn pem_govdesi(pem: &str) -> Result<Vec<u8>, &'static str> {
    let bozuk = "PKCS#8 PEM (`-----BEGIN PRIVATE KEY-----`) tek blok olarak çözülemedi";
    let (once, kalan) = pem.split_once(PEM_BASI).ok_or(bozuk)?;
    let (govde, sonra) = kalan.split_once(PEM_SONU).ok_or(bozuk)?;
    if !once.trim().is_empty() || !sonra.trim().is_empty() || govde.contains("-----") {
        return Err(bozuk);
    }
    base64_mime_coz(govde, AZAMI_PEM_BAYTI).map_err(|_| bozuk)
}

impl DkimImzaAnahtari {
    /// PKCS#8 PEM'i bildirilen algoritmayla çözer. Hata yalnız sabit nedendir.
    pub fn pemden(
        alan: &str,
        secici: &str,
        algoritma: DkimAlgoritmasi,
        pem: &str,
    ) -> Result<Self, &'static str> {
        let mut der = pem_govdesi(pem)?;
        let sonuc = Self::derden(algoritma, &der);
        sifirla(&mut der);
        let (cift, bit) = sonuc?;
        Ok(Self {
            alan: alan.to_ascii_lowercase(),
            secici: secici.to_ascii_lowercase(),
            algoritma,
            bit,
            cift,
        })
    }

    fn derden(
        algoritma: DkimAlgoritmasi,
        der: &[u8],
    ) -> Result<(AnahtarCifti, usize), &'static str> {
        match algoritma {
            DkimAlgoritmasi::Ed25519Sha256 => Ed25519KeyPair::from_pkcs8_maybe_unchecked(der)
                .map(|cift| (AnahtarCifti::Ed25519(cift), 256))
                .map_err(|_| "Ed25519 PKCS#8 özel anahtarı değil"),
            DkimAlgoritmasi::RsaSha256 => {
                let cift = RsaKeyPair::from_pkcs8(der).map_err(|hata| {
                    match hata.to_string().as_str() {
                        "TooSmall" => "RSA anahtarı 2048 bitten kısa (RFC 8301)",
                        "TooLarge" => "RSA anahtarı 4096 bitten uzun",
                        _ => "RSA PKCS#8 özel anahtarı değil",
                    }
                })?;
                let bit = cift.public().modulus_len() * 8;
                if bit < RSA_IMZA_EN_AZ_BIT {
                    return Err("RSA anahtarı 2048 bitten kısa (RFC 8301)");
                }
                if bit > crate::dkim_kripto::RSA_EN_COK_BIT {
                    return Err("RSA anahtarı 4096 bitten uzun");
                }
                Ok((AnahtarCifti::Rsa(cift), bit))
            }
        }
    }

    /// Açık anahtar: RSA'da PKCS#1 `RSAPublicKey` DER'i, Ed25519'da ham 32 bayt
    /// (DNS `p=` ile eşleme ve testler için).
    pub fn acik_anahtar(&self) -> Vec<u8> {
        use ring::signature::KeyPair;
        match &self.cift {
            AnahtarCifti::Rsa(cift) => cift.public().as_ref().to_vec(),
            AnahtarCifti::Ed25519(cift) => cift.public_key().as_ref().to_vec(),
        }
    }

    fn imzala_bayt(&self, veri: &[u8]) -> Result<Vec<u8>, DkimHatasi> {
        match &self.cift {
            AnahtarCifti::Rsa(cift) => {
                let mut imza = vec![0u8; cift.public().modulus_len()];
                cift.sign(&RSA_PKCS1_SHA256, &SystemRandom::new(), veri, &mut imza)
                    .map_err(|_| DkimHatasi::AnahtarBozuk)?;
                Ok(imza)
            }
            AnahtarCifti::Ed25519(cift) => {
                let ozet = ring::digest::digest(&ring::digest::SHA256, veri);
                Ok(cift.sign(ozet.as_ref()).as_ref().to_vec())
            }
        }
    }
}

/// `dkim_anahtarları` bildiriminden yüklenen anahtarlar; anahtar Zee'de
/// credential adıyla anılır.
#[derive(Debug, Default)]
pub struct DkimAnahtarDeposu {
    anahtarlar: Vec<DepoKaydi>,
}

#[derive(Debug)]
struct DepoKaydi {
    credential: String,
    anahtar: DkimImzaAnahtari,
    /// K-413: açılış DNS denetiminde devre dışı bırakıldıysa nedeni.
    devre_disi: Option<DkimHatasi>,
}

impl DkimAnahtarDeposu {
    pub fn ekle(&mut self, credential: &str, anahtar: DkimImzaAnahtari) {
        self.anahtarlar.push(DepoKaydi {
            credential: credential.into(),
            anahtar,
            devre_disi: None,
        });
    }

    pub fn bos_mu(&self) -> bool {
        self.anahtarlar.is_empty()
    }

    /// K-413 (ADR-174 §9): her anahtarın `<s>._domainkey.<d>` TXT kaydını
    /// `txt_sorgula` ile alır ve yayımlanan `p=` ile karşılaştırır. Uyuşmayan
    /// ya da iptal edilmiş anahtar yalnız kendisi için devre dışı kalır;
    /// kayıt yokluğu ve sorgu hatası anahtarı etkin bırakır. Dönüş, açılışta
    /// yazılacak tanı satırlarıdır (anahtar baytı yok).
    pub fn yayimlananlarla_denetle(
        &mut self,
        mut txt_sorgula: impl FnMut(&str) -> Result<Vec<Vec<u8>>, String>,
    ) -> Vec<String> {
        let mut tanilar = Vec::new();
        for kayit in &mut self.anahtarlar {
            let (alan, secici) = (&kayit.anahtar.alan, &kayit.anahtar.secici);
            let kimlik = format!("DKIM anahtarı ({alan} / {secici})");
            let ad = format!("{secici}._domainkey.{alan}");
            let txt = match txt_sorgula(&ad) {
                Ok(txt) => txt,
                Err(neden) => {
                    tanilar.push(format!(
                        "UYARI: {kimlik} DNS'te denetlenemedi ({neden}); anahtar etkin."
                    ));
                    continue;
                }
            };
            let (neden, aciklama) = match dns_kaydini_denetle(&kayit.anahtar, &txt) {
                DnsEslesmesi::Eslesti => continue,
                DnsEslesmesi::KayitYok => {
                    tanilar.push(format!(
                        "UYARI: {kimlik} için `{ad}` TXT kaydı yok; anahtar etkin, alıcılar imzayı doğrulayamaz."
                    ));
                    continue;
                }
                DnsEslesmesi::Iptal => (DkimHatasi::AnahtarIptal, "anahtar iptal edilmiş (boş p=)"),
                DnsEslesmesi::Uyusmuyor => (
                    DkimHatasi::AnahtarDnsUyusmuyor,
                    "yayımlanan açık anahtar (p=) özel anahtara uymuyor",
                ),
            };
            kayit.devre_disi = Some(neden);
            tanilar.push(format!(
                "HATA: {kimlik}: `{ad}` kaydında {aciklama}; bu anahtarla imza atılmaz (C034 `{}`).",
                neden.turu()
            ));
        }
        tanilar
    }

    /// Girdiyi yeniden denetler (savunma derinliği), bildirimle eşler ve
    /// imzalar → `b=` değeri (base64, satır kırma yok).
    pub fn imzala(
        &self,
        credential: &str,
        algoritma: &str,
        veri: &str,
    ) -> Result<String, DkimHatasi> {
        let istenen = DkimAlgoritmasi::ayristir(algoritma)?;
        let kayit = self
            .anahtarlar
            .iter()
            .find(|kayit| kayit.credential == credential)
            .ok_or(DkimHatasi::AnahtarBilinmiyor)?;
        if let Some(neden) = kayit.devre_disi {
            return Err(neden);
        }
        let anahtar = &kayit.anahtar;
        let etiketler = imza_girdisini_denetle(veri)?;
        if istenen != anahtar.algoritma
            || etiketler.algoritma != anahtar.algoritma
            || etiketler.alan != anahtar.alan
            || etiketler.secici != anahtar.secici
        {
            return Err(DkimHatasi::AnahtarUyusmuyor);
        }
        let baytlar = kanonik_baytlar(veri)?;
        Ok(base64_kodla(&anahtar.imzala_bayt(&baytlar)?))
    }
}

/// K-413 (ADR-174 eki): `<s>._domainkey.<d>` TXT kayıtlarının özel
/// anahtarla açılış eşleşmesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsEslesmesi {
    /// Bir DKIM anahtar kaydının `p=`'si özel anahtarın açık parçasıdır.
    Eslesti,
    /// Adda DKIM anahtar kaydı yok (uyarı; süreç başlar).
    KayitYok,
    /// Anahtar iptal edilmiş (boş `p=`, RFC 6376 §3.6.1).
    Iptal,
    /// DKIM kayıtları var ama hiçbiri bu anahtar değil.
    Uyusmuyor,
}

/// RFC 6376 §3.6.1 anahtar kaydı: `v=` varsa ilk etiket ve `DKIM1`, `k=`
/// varsayılanı `rsa`, `p=` zorunlu. Anahtar kaydı değilse `None`.
fn anahtar_kaydi(txt: &[u8]) -> Option<(String, String)> {
    let metin = std::str::from_utf8(txt).ok()?;
    let (mut k, mut p) = ("rsa".to_string(), None);
    for (sira, parca) in metin.split(';').enumerate() {
        let parca = parca.trim_matches([' ', '\t', '\r', '\n']);
        if parca.is_empty() {
            continue;
        }
        let (ad, deger) = parca.split_once('=')?;
        let (ad, deger) = (ad.trim(), deger.trim());
        match ad {
            "v" if sira != 0 || deger != "DKIM1" => return None,
            "k" => k = deger.to_ascii_lowercase(),
            "p" => p = Some(deger.to_string()),
            _ => {}
        }
    }
    Some((k, p?))
}

/// TXT kayıtlarını anahtarla karşılaştırır. Eşleşen kayıt varsa `Eslesti`;
/// yoksa iptal edilmiş kayıt `Iptal`, başka DKIM kaydı `Uyusmuyor`.
pub fn dns_kaydini_denetle(anahtar: &DkimImzaAnahtari, txt: &[Vec<u8>]) -> DnsEslesmesi {
    use crate::dkim_kripto::AcikAnahtar;
    let beklenen = AcikAnahtar::ayristir(anahtar.algoritma, &anahtar.acik_anahtar()).ok();
    let tur = match anahtar.algoritma {
        DkimAlgoritmasi::RsaSha256 => "rsa",
        DkimAlgoritmasi::Ed25519Sha256 => "ed25519",
    };
    let (mut iptal, mut dkim_var) = (false, false);
    for (k, p) in txt.iter().filter_map(|kayit| anahtar_kaydi(kayit)) {
        dkim_var = true;
        if p.chars().all(|c| c.is_ascii_whitespace()) {
            iptal = true;
            continue;
        }
        let yayimlanan = AcikAnahtar::base64ten(anahtar.algoritma, &p).ok();
        if k == tur && yayimlanan.is_some() && yayimlanan == beklenen {
            return DnsEslesmesi::Eslesti;
        }
    }
    match (iptal, dkim_var) {
        (true, _) => DnsEslesmesi::Iptal,
        (false, true) => DnsEslesmesi::Uyusmuyor,
        (false, false) => DnsEslesmesi::KayitYok,
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    const GIRDI: &str = "from:Ali <ali@posta.ornek.test>\nsubject:Selam\ndkim-signature:v=1; a=ed25519-sha256; c=relaxed/relaxed; d=ornek.test; s=z2026a; h=from:subject; bh=47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=; b=";

    #[test]
    fn gecerli_girdi_etiketleri_doner() {
        let etiketler = imza_girdisini_denetle(GIRDI).unwrap();
        assert_eq!(etiketler.alan, "ornek.test");
        assert_eq!(etiketler.secici, "z2026a");
        assert_eq!(etiketler.algoritma, DkimAlgoritmasi::Ed25519Sha256);
        let katlanmis = GIRDI.replace("; d=", ";\n\td=").replace("From:", "from:");
        assert!(imza_girdisini_denetle(&katlanmis).is_ok());
    }

    #[test]
    fn kahin_onlemi_bozuk_girdileri_reddeder() {
        let gecersiz_mi = |veri: &str| {
            matches!(
                imza_girdisini_denetle(veri),
                Err(DkimHatasi::GirdiGecersiz(_))
            )
        };
        for (eski, yeni) in [
            ("ali@posta.ornek.test", "ali@kotu.test"),
            ("ali@posta.ornek.test", "ali@ornek.test.kotu"),
            ("ali@posta.ornek.test", "ali@xornek.test"),
            ("from:Ali <ali@posta.ornek.test>\n", ""),
            ("subject:Selam\n", "from:b@ornek.test\n"),
            ("h=from:subject", "h=subject"),
            ("; b=", "; b=abc"),
            ("; b=", "; b=; x=1"),
            ("v=1; ", ""),
            ("bh=47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=", "bh="),
            ("s=z2026a;", "s=z2026a; l=10;"),
            ("s=z2026a;", "s=z2026a; i=@kotu.test;"),
            ("s=z2026a;", "s=z2026a; s=y;"),
            (
                "Ali <ali@posta.ornek.test>",
                "<a@ornek.test>, <b@kotu.test>",
            ),
            ("Ali <ali@posta.ornek.test>", "b@kotu.test <ali@ornek.test>"),
            ("Ali <ali@posta.ornek.test>", "\"Ali <ali@kotu.test>"),
            ("dkim-signature:", "x-imza:"),
            ("d=ornek.test", "d=ornek..test"),
        ] {
            let veri = GIRDI.replace(eski, yeni);
            assert_ne!(veri, GIRDI, "{eski}");
            assert!(gecersiz_mi(&veri), "{eski} → {yeni}");
        }
        assert!(gecersiz_mi(&format!("{GIRDI}\n")));
        assert!(gecersiz_mi(&format!("\n{GIRDI}")));
        assert!(gecersiz_mi(&format!(" devam\n{GIRDI}")));
        assert_eq!(
            imza_girdisini_denetle(&GIRDI.replace("\nsubject", "\r\nsubject")),
            Err(DkimHatasi::VeriBozuk)
        );
        assert_eq!(
            imza_girdisini_denetle(&GIRDI.replace("ed25519-sha256", "rsa-sha1")),
            Err(DkimHatasi::AlgoritmaReddedildi)
        );
        assert!(gecersiz_mi(&"x".repeat(AZAMI_IMZA_GIRDISI + 1)));
    }

    #[test]
    fn tirnakli_ad_ve_yorum_adresi_bozmaz() {
        for from in [
            "\"Ali, <Veli>\" <ali@ornek.test>",
            "ali@ornek.test (Ali <x@kotu.test>)",
            "ali@ORNEK.test.",
        ] {
            let veri = GIRDI.replace("Ali <ali@posta.ornek.test>", from);
            assert!(imza_girdisini_denetle(&veri).is_ok(), "{from}");
        }
    }

    #[test]
    fn ed25519_imzasi_dogrulayiciyla_tutar_ve_depo_eslemeyi_ister() {
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let pem = format!("{PEM_BASI}\n{}\n{PEM_SONU}\n", base64_kodla(pkcs8.as_ref()));
        let anahtar =
            DkimImzaAnahtari::pemden("Ornek.Test", "z2026a", DkimAlgoritmasi::Ed25519Sha256, &pem)
                .unwrap();
        let p = base64_kodla(&anahtar.acik_anahtar());
        assert!(!format!("{anahtar:?}").contains(&p[..16]));
        let mut depo = DkimAnahtarDeposu::default();
        depo.ekle("zeemail_dkim_anahtari", anahtar);
        let b = depo
            .imzala("zeemail_dkim_anahtari", "ed25519-sha256", GIRDI)
            .unwrap();
        let sonuc =
            crate::dkim_kripto::base64_imzayi_dogrula("ed25519-sha256", &p, GIRDI, &b).unwrap();
        assert!(sonuc.tuttu);
        assert_eq!(
            depo.imzala("baska", "ed25519-sha256", GIRDI),
            Err(DkimHatasi::AnahtarBilinmiyor)
        );
        assert_eq!(
            depo.imzala("zeemail_dkim_anahtari", "rsa-sha256", GIRDI),
            Err(DkimHatasi::AnahtarUyusmuyor)
        );
        for (eski, yeni) in [
            ("s=z2026a", "s=r2026a"),
            ("d=ornek.test", "d=posta.ornek.test"),
        ] {
            assert_eq!(
                depo.imzala(
                    "zeemail_dkim_anahtari",
                    "ed25519-sha256",
                    &GIRDI.replace(eski, yeni)
                ),
                Err(DkimHatasi::AnahtarUyusmuyor),
                "{eski}"
            );
        }
    }

    #[test]
    fn bozuk_pem_ve_yanlis_algoritma_reddedilir() {
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let govde = base64_kodla(pkcs8.as_ref());
        let pem = format!("{PEM_BASI}\n{govde}\n{PEM_SONU}");
        let yukle = |algoritma, pem: &str| DkimImzaAnahtari::pemden("a.test", "s", algoritma, pem);
        assert!(yukle(DkimAlgoritmasi::RsaSha256, &pem).is_err());
        for bozuk in [
            String::new(),
            pem.replace("PRIVATE KEY", "RSA PRIVATE KEY"),
            pem.replace("PRIVATE KEY", "ENCRYPTED PRIVATE KEY"),
            format!("{pem}\n{pem}"),
            format!("x{pem}"),
            pem.replace(&govde[..8], "%%%%%%%%"),
        ] {
            let hata = yukle(DkimAlgoritmasi::Ed25519Sha256, &bozuk).unwrap_err();
            assert!(!hata.contains(&govde[..16]));
        }
    }

    #[test]
    fn dns_kaydi_eslesme_iptal_uyusmazlik_ve_yokluk() {
        let rng = SystemRandom::new();
        let pem = |pkcs8: &[u8]| format!("{PEM_BASI}\n{}\n{PEM_SONU}", base64_kodla(pkcs8));
        let yukle = |pkcs8: &[u8]| {
            DkimImzaAnahtari::pemden(
                "ornek.test",
                "s",
                DkimAlgoritmasi::Ed25519Sha256,
                &pem(pkcs8),
            )
            .unwrap()
        };
        let anahtar = yukle(Ed25519KeyPair::generate_pkcs8(&rng).unwrap().as_ref());
        let baska = yukle(Ed25519KeyPair::generate_pkcs8(&rng).unwrap().as_ref());
        let p = base64_kodla(&anahtar.acik_anahtar());
        let kayit = |m: String| m.into_bytes();
        let denetle = |kayitlar: Vec<String>| {
            dns_kaydini_denetle(
                &anahtar,
                &kayitlar.into_iter().map(kayit).collect::<Vec<_>>(),
            )
        };
        use DnsEslesmesi::*;
        assert_eq!(denetle(vec![format!("v=DKIM1; k=ed25519; p={p}")]), Eslesti);
        // Katlanmış p=, sonda ';', başka kayıtlarla birlikte.
        let katli = format!("{} {}", &p[..10], &p[10..]);
        assert_eq!(
            denetle(vec![
                "v=spf1 -all".into(),
                format!("v=DKIM1;k=ed25519;p={katli};")
            ]),
            Eslesti
        );
        assert_eq!(denetle(vec!["v=DKIM1; k=ed25519; p=".into()]), Iptal);
        let baska_p = base64_kodla(&baska.acik_anahtar());
        assert_eq!(
            denetle(vec![format!("v=DKIM1; k=ed25519; p={baska_p}")]),
            Uyusmuyor
        );
        // k= varsayılanı rsa: Ed25519 anahtarı eşleşmez.
        assert_eq!(denetle(vec![format!("v=DKIM1; p={p}")]), Uyusmuyor);
        assert_eq!(
            denetle(vec![format!("v=DKIM1; k=ed25519; p=%%%")]),
            Uyusmuyor
        );
        // v= ilk değil ya da DKIM1 değil: anahtar kaydı sayılmaz.
        assert_eq!(
            denetle(vec![format!("k=ed25519; v=DKIM1; p={p}")]),
            KayitYok
        );
        assert_eq!(
            denetle(vec![format!("v=DKIM2; k=ed25519; p={p}")]),
            KayitYok
        );
        assert_eq!(denetle(vec!["v=spf1 -all".into()]), KayitYok);
        assert_eq!(denetle(vec![]), KayitYok);
        // Eşleşen kayıt iptal kaydından önce gelir.
        assert_eq!(
            denetle(vec![
                "v=DKIM1; k=ed25519; p=".into(),
                format!("v=DKIM1; k=ed25519; p={p}")
            ]),
            Eslesti
        );
    }
}
