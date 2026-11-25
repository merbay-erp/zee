//! K-397 Dilim 2 (RFC-0044 B3, ADR-174): giden DKIM imza anahtarlarının
//! bildirimi.
//!
//! `dkim_anahtarları "ornek.test z2026a ed25519-sha256 zeemail_dkim_anahtari"
//! listesi olsun` biçiminde her öğe `<alan> <seçici> <algoritma> <credential>`
//! taşır. Özel anahtar kaynakta ya da proje dosyasında değil, K-339
//! modeliyle adlandırılmış credential/ortam değişkenindedir; bildirim yalnız
//! adı taşır ve yalnız `kriptografi` yetkinliğiyle geçerlidir. Bozuk bildirim
//! P015'tir.
use super::*;

/// Bir projenin bildirebileceği en çok DKIM anahtarı (döndürme + çift imza).
pub const DKIM_ANAHTARI_AZAMI: usize = 16;

const ORNEK: &str = "Örnek: dkim_anahtarları \"ornek.test z2026a ed25519-sha256 zeemail_dkim_anahtari\" listesi olsun";

/// Tek imza anahtarı: alan (`d=`), seçici (`s=`), algoritma (`a=`) ve
/// PKCS#8 PEM'i taşıyan credential/ortam adı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DkimAnahtarBildirimi {
    pub alan: String,
    pub secici: String,
    /// `rsa-sha256` ya da `ed25519-sha256`.
    pub algoritma: String,
    pub credential: String,
}

fn dns_adi(ad: &str) -> bool {
    !ad.is_empty()
        && ad.len() <= 253
        && ad.split('.').all(|etiket| {
            !etiket.is_empty()
                && etiket.len() <= 63
                && !etiket.starts_with('-')
                && !etiket.ends_with('-')
                && etiket
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        })
}

/// K-339 credential kimliği kuralı (`guvenlik::sir_dizini::ad_guvenli` ile
/// aynı; proje katmanı güvenlik adaptörüne bağlanmaz).
fn credential_adi_mi(ad: &str) -> bool {
    let b = ad.as_bytes();
    !b.is_empty()
        && b.len() <= 255
        && (b[0].is_ascii_alphanumeric() || b[0] == b'_')
        && b.iter()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'-'))
}

fn ogeyi_ayristir(yazim: &str) -> Result<DkimAnahtarBildirimi, String> {
    let parcalar: Vec<&str> = yazim.split(' ').collect();
    let [alan, secici, algoritma, credential] = parcalar.as_slice() else {
        return Err("öğe `<alan> <seçici> <algoritma> <credential>` biçiminde olmalı".into());
    };
    if !dns_adi(alan) || !alan.contains('.') {
        return Err(format!("\"{alan}\" küçük harfli bir alan adı değil"));
    }
    if !dns_adi(secici) {
        return Err(format!("\"{secici}\" küçük harfli bir DKIM seçicisi değil"));
    }
    match *algoritma {
        "rsa-sha256" | "ed25519-sha256" => {}
        "rsa-sha1" => return Err("rsa-sha1 ile imza atılmaz (RFC 8301)".into()),
        _ => {
            return Err(format!(
                "\"{algoritma}\" desteklenmiyor; rsa-sha256 ya da ed25519-sha256 yaz"
            ))
        }
    }
    if !credential_adi_mi(credential) {
        return Err(format!(
            "\"{credential}\" credential/ortam adı değil (A-Z, a-z, 0-9, _, . ve -)"
        ));
    }
    Ok(DkimAnahtarBildirimi {
        alan: alan.to_string(),
        secici: secici.to_string(),
        algoritma: algoritma.to_string(),
        credential: credential.to_string(),
    })
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
) -> Result<(Vec<DkimAnahtarBildirimi>, usize), Tani> {
    let (yazimlar, satir) = metin_listesini_al(alanlar, "dkim_anahtarları", "P015")?;
    if yazimlar.is_empty() {
        return Ok((Vec::new(), satir));
    }
    let hata = |mesaj: &str| proje_hatasi("P015", mesaj, satir, ORNEK);
    if !yetkinlik_yazimlari.iter().any(|y| y == "kriptografi") {
        return Err(hata(
            "`dkim_anahtarları` yalnız `kriptografi` yetkinliğiyle kullanılabilir.",
        ));
    }
    if yazimlar.len() > DKIM_ANAHTARI_AZAMI {
        return Err(hata(&format!(
            "{} DKIM anahtarı bildirildi; en çok {DKIM_ANAHTARI_AZAMI} olabilir.",
            yazimlar.len()
        )));
    }
    let mut bildirimler: Vec<DkimAnahtarBildirimi> = Vec::new();
    for yazim in &yazimlar {
        let bildirim = ogeyi_ayristir(yazim)
            .map_err(|neden| hata(&format!("`dkim_anahtarları` öğesi \"{yazim}\": {neden}.")))?;
        let yinelenen = bildirimler.iter().any(|var| {
            (var.alan == bildirim.alan && var.secici == bildirim.secici)
                || var.credential == bildirim.credential
        });
        if yinelenen {
            return Err(hata(&format!(
                "`dkim_anahtarları` öğesi \"{yazim}\": aynı alan+seçici ya da aynı credential iki kez bildirilmiş."
            )));
        }
        bildirimler.push(bildirim);
    }
    Ok((bildirimler, satir))
}

/// DKIM credential'ı başka bir gizli değerin adıyla aynı olamaz: imza
/// kâhini TLS anahtarı ya da veritabanı parolası üzerinde çalışmamalı.
pub(super) fn ad_cakismasini_denetle(
    bildirimler: &[DkimAnahtarBildirimi],
    diger_gizliler: &[&str],
    satir: usize,
) -> Result<(), Tani> {
    match bildirimler
        .iter()
        .find(|b| diger_gizliler.contains(&b.credential.as_str()))
    {
        Some(b) => Err(proje_hatasi(
            "P015",
            &format!(
                "DKIM credential'ı \"{}\" projede başka bir gizli değer için de bildirilmiş.",
                b.credential
            ),
            satir,
            "Her DKIM özel anahtarı için ayrı bir credential adı kullan.",
        )),
        None => Ok(()),
    }
}
