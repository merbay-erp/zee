//! RFC-0047 E (ADR-182 dilim B, K-364): uygulama parolası HMAC anahtarları.
//!
//! `uygulama_parolası_anahtarları "k2026 zeemail_up_anahtari" listesi olsun`
//! biçiminde her öğe `<anahtar kimliği> <credential>` taşır. İlk öğe etkin
//! anahtardır (yeni parola onunla üretilir); sonrakiler yalnız doğrulamadır
//! ve döndürmede ona bağlı en az bir kayıt kaldıkça tutulur (süreyle silinmez,
//! ADR-182 soru 2). Anahtar baytı (64 onaltılık karakter) K-339 modeliyle
//! credential/ortam değişkenindedir; şifreleme anahtarından türetilmez.
//! Bildirim yalnız `kriptografi` yetkinliğiyle geçerlidir; bozuk bildirim P015.
use super::*;

/// Bir projenin bildirebileceği en çok uygulama parolası anahtarı.
pub const UYGULAMA_PAROLASI_ANAHTARI_AZAMI: usize = 8;

const ORNEK: &str =
    "Örnek: uygulama_parolası_anahtarları \"k2026 zeemail_up_anahtari\" listesi olsun";

/// Tek HMAC anahtarı: özetin taşıdığı kimlik ve anahtarı taşıyan credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UygulamaParolasiAnahtari {
    pub kimlik: String,
    pub credential: String,
}

fn ogeyi_ayristir(yazim: &str) -> Result<UygulamaParolasiAnahtari, String> {
    let parcalar: Vec<&str> = yazim.split(' ').collect();
    let [kimlik, credential] = parcalar.as_slice() else {
        return Err("öğe `<anahtar kimliği> <credential>` biçiminde olmalı".into());
    };
    // `guvenlik::uygulama_parolasi::anahtar_kimligi_gecerli` ile aynı kural;
    // proje katmanı güvenlik adaptörüne bağlanmaz.
    let kimlik_gecerli = !kimlik.is_empty()
        && kimlik.len() <= 16
        && kimlik
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit());
    if !kimlik_gecerli {
        return Err(format!(
            "\"{kimlik}\" anahtar kimliği 1–16 küçük harf ya da rakam olmalı"
        ));
    }
    if !dkim::credential_adi_mi(credential) {
        return Err(format!(
            "\"{credential}\" credential/ortam adı değil (A-Z, a-z, 0-9, _, . ve -)"
        ));
    }
    Ok(UygulamaParolasiAnahtari {
        kimlik: kimlik.to_string(),
        credential: credential.to_string(),
    })
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
) -> Result<(Vec<UygulamaParolasiAnahtari>, usize, super::DavBildirimi), Tani> {
    let (anahtarlar, satir) = anahtarlari_al(alanlar, yetkinlik_yazimlari)?;
    // RFC-0047 dilim C: DAV bildirimi anahtar listesine bağlıdır.
    let dav = super::dav::alanlari_al(alanlar, yetkinlik_yazimlari, !anahtarlar.is_empty())?;
    Ok((anahtarlar, satir, dav))
}

fn anahtarlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
) -> Result<(Vec<UygulamaParolasiAnahtari>, usize), Tani> {
    let (yazimlar, satir) = metin_listesini_al(alanlar, "uygulama_parolası_anahtarları", "P015")?;
    if yazimlar.is_empty() {
        return Ok((Vec::new(), satir));
    }
    let hata = |mesaj: &str| proje_hatasi("P015", mesaj, satir, ORNEK);
    if !yetkinlik_yazimlari.iter().any(|y| y == "kriptografi") {
        return Err(hata(
            "`uygulama_parolası_anahtarları` yalnız `kriptografi` yetkinliğiyle kullanılabilir.",
        ));
    }
    if yazimlar.len() > UYGULAMA_PAROLASI_ANAHTARI_AZAMI {
        return Err(hata(&format!(
            "{} uygulama parolası anahtarı bildirildi; en çok {UYGULAMA_PAROLASI_ANAHTARI_AZAMI} olabilir.",
            yazimlar.len()
        )));
    }
    let mut bildirimler: Vec<UygulamaParolasiAnahtari> = Vec::new();
    for yazim in &yazimlar {
        let bildirim = ogeyi_ayristir(yazim).map_err(|neden| {
            hata(&format!(
                "`uygulama_parolası_anahtarları` öğesi \"{yazim}\": {neden}."
            ))
        })?;
        if bildirimler
            .iter()
            .any(|var| var.kimlik == bildirim.kimlik || var.credential == bildirim.credential)
        {
            return Err(hata(&format!(
                "`uygulama_parolası_anahtarları` öğesi \"{yazim}\": aynı anahtar kimliği ya da aynı credential iki kez bildirilmiş."
            )));
        }
        bildirimler.push(bildirim);
    }
    Ok((bildirimler, satir))
}

/// HMAC anahtarının credential'ı başka bir gizli değerin (şifreleme anahtarı,
/// DKIM, TLS, veritabanı…) adıyla aynı olamaz: anahtar ayrımı ilkesi.
pub(super) fn ad_cakismasini_denetle(bildirim: &ProjeBildirimi, satir: usize) -> Result<(), Tani> {
    let mut gizliler = dkim::gizli_adlari(bildirim);
    gizliler.extend(
        bildirim
            .dkim_anahtarlari
            .iter()
            .map(|b| b.credential.as_str()),
    );
    match bildirim
        .uygulama_parolasi_anahtarlari
        .iter()
        .find(|b| gizliler.contains(&b.credential.as_str()))
    {
        Some(b) => Err(proje_hatasi(
            "P015",
            &format!(
                "Uygulama parolası credential'ı \"{}\" projede başka bir gizli değer için de bildirilmiş.",
                b.credential
            ),
            satir,
            "HMAC anahtarı için ayrı bir credential adı kullan; şifreleme anahtarından türetilmez.",
        )),
        None => Ok(()),
    }
}
