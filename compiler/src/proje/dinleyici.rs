//! K-376 (RFC-0039 Aşama A1): adlı dinleyici bildirimi.
//!
//! `dinleyiciler "smtp_v4 0.0.0.0:25", "smtp_v6 [::]:25" listesi olsun`
//! biçiminde her öğe `<ad> <adres>:<kapı>` taşır. Ad systemd
//! `FileDescriptorName=` değeridir; çift yığın aile sonekli iki ayrı adla
//! bildirilir (tek ad yalnız tek aileye bağlanır). Bildirim yalnız dış
//! yöneticiden devralınacak dinleyicilerin sözleşmesidir; süreç kendisi bind
//! etmez ve Zee kodu bu sürümde dinleyiciye erişmez.
use super::*;
use std::net::SocketAddr;

/// Bir projenin bildirebileceği en çok adlı dinleyici.
pub const DINLEYICI_AZAMI: usize = 64;

/// Devralınması beklenen tek bir adlı dinleyici.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DinleyiciBildirimi {
    /// `^[a-z][a-z0-9_]{0,31}$` (systemd `FileDescriptorName=`).
    pub ad: String,
    /// Aile, adres ve kapı tam eşleşir; IPv6'da kapsam/akış etiketi yoktur.
    pub adres: SocketAddr,
}

impl DinleyiciBildirimi {
    /// `<ad> <adres>:<kapı>` yazımını doğrular.
    pub fn ayristir(yazim: &str) -> Result<Self, String> {
        let (ad, adres) = yazim
            .split_once(' ')
            .ok_or_else(|| "öğe `<ad> <adres>:<kapı>` biçiminde olmalı".to_string())?;
        if !gecerli_dinleyici_adi(ad) {
            return Err(format!(
                "\"{ad}\" dinleyici adı küçük harfle başlamalı; yalnız a-z, 0-9 ve _ içermeli, en çok 32 karakter olmalı"
            ));
        }
        let adres: SocketAddr = adres.parse().map_err(|_| {
            format!("\"{adres}\" adres:kapı değil (IPv6 köşeli parantezle yazılır)")
        })?;
        if adres.port() == 0 {
            return Err("kapı 0 olamaz".into());
        }
        if let SocketAddr::V6(v6) = adres {
            if v6.scope_id() != 0 || v6.flowinfo() != 0 {
                return Err("IPv6 adresi kapsam kimliği taşıyamaz".into());
            }
            if v6.ip().to_ipv4_mapped().is_some() {
                return Err(
                    "IPv4 eşlemeli IPv6 adresi kullanılamaz; IPv4 dinleyicisini ayrı adla bildir"
                        .into(),
                );
            }
        }
        Ok(Self {
            ad: ad.to_string(),
            adres,
        })
    }
}

/// systemd `LISTEN_FDNAMES` ayracı `:` olduğundan ad ayraç içeremez.
pub fn gecerli_dinleyici_adi(ad: &str) -> bool {
    let mut baytlar = ad.bytes();
    ad.len() <= 32
        && baytlar.next().is_some_and(|b| b.is_ascii_lowercase())
        && baytlar.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
) -> Result<Vec<DinleyiciBildirimi>, Tani> {
    let (yazimlar, satir) = metin_listesini_al(alanlar, "dinleyiciler", "P018")?;
    if yazimlar.len() > DINLEYICI_AZAMI {
        return Err(proje_hatasi(
            "P018",
            &format!(
                "{} dinleyici bildirildi; en çok {DINLEYICI_AZAMI} olabilir.",
                yazimlar.len()
            ),
            satir,
            "Dinleyicileri ayrı hizmetlere böl.",
        ));
    }
    let mut dinleyiciler: Vec<DinleyiciBildirimi> = Vec::with_capacity(yazimlar.len());
    for yazim in yazimlar {
        let dinleyici = DinleyiciBildirimi::ayristir(&yazim).map_err(|neden| {
            proje_hatasi(
                "P018",
                &format!("Geçersiz dinleyici \"{yazim}\": {neden}."),
                satir,
                "Örnek: dinleyiciler \"smtp_v4 0.0.0.0:25\", \"smtp_v6 [::]:25\" listesi olsun",
            )
        })?;
        if dinleyiciler.iter().any(|d| d.ad == dinleyici.ad) {
            return Err(proje_hatasi(
                "P018",
                &format!("\"{}\" dinleyici adı birden çok kez yazıldı.", dinleyici.ad),
                satir,
                "Çift yığında her aileye ayrı ad ver: smtp_v4 ve smtp_v6.",
            ));
        }
        if dinleyiciler.iter().any(|d| d.adres == dinleyici.adres) {
            return Err(proje_hatasi(
                "P018",
                &format!("{} adresi birden çok dinleyicide yazıldı.", dinleyici.adres),
                satir,
                "Her adres:kapı tek bir dinleyiciye aittir.",
            ));
        }
        dinleyiciler.push(dinleyici);
    }
    Ok(dinleyiciler)
}
