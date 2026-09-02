//! Proje, derleyici ve runtime için tek merkezî dış dünya yetkinlik modeli.

use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

mod origin;
use origin::ayristir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Yetkinlik {
    DosyaOkuma,
    DosyaYazma,
    Ag,
    YerelAg,
    AgSunucusu,
    Donanim,
    WebOturumu,
    Kriptografi,
}

impl Yetkinlik {
    pub const TUMU: [Self; 8] = [
        Self::DosyaOkuma,
        Self::DosyaYazma,
        Self::Ag,
        Self::YerelAg,
        Self::AgSunucusu,
        Self::Donanim,
        Self::WebOturumu,
        Self::Kriptografi,
    ];

    pub const fn yazimi(self) -> &'static str {
        match self {
            Self::DosyaOkuma => "dosya-okuma",
            Self::DosyaYazma => "dosya-yazma",
            Self::Ag => "ağ",
            Self::YerelAg => "yerel-ağ",
            Self::AgSunucusu => "ağ-sunucusu",
            Self::Donanim => "donanım",
            Self::WebOturumu => "web-oturumu",
            Self::Kriptografi => "kriptografi",
        }
    }

    pub fn ayristir(yazim: &str) -> Option<Self> {
        Self::TUMU
            .into_iter()
            .find(|yetkinlik| yetkinlik.yazimi() == yazim)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DosyaSiniri {
    HerYer,
    ProjeKoku,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgSemasi {
    Http,
    Https,
}

impl AgSemasi {
    pub const fn yazimi(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }

    const fn varsayilan_kapi(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https => 443,
        }
    }
}

/// Bir outbound izninin tam şema + host + port kimliği. Yol ve sorgu izin
/// kimliğine katılmaz; tek origin altındaki bütün uçlar aynı capability'dir.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgHedefi {
    sema: AgSemasi,
    konak: String,
    kapi: u16,
}

impl AgHedefi {
    pub fn bildirimden(yazim: &str) -> Result<Self, String> {
        let (hedef, kuyruk) = ayristir(yazim)?;
        if !kuyruk.is_empty() && kuyruk != "/" {
            return Err("ağ hedefi yol, sorgu veya parça taşıyamaz; yalnız origin yaz".into());
        }
        Ok(hedef)
    }

    pub fn istekten(yazim: &str) -> Result<Self, String> {
        let (hedef, _) = ayristir(yazim)?;
        Ok(hedef)
    }

    /// HTTPS origin bekleyen CLI/protokol sınırları için ortak kurucu.
    pub fn https_origininden(yazim: &str) -> Result<Self, String> {
        let hedef = Self::bildirimden(yazim)?;
        if hedef.semasi() != AgSemasi::Https {
            return Err("güvenli origin https:// şeması taşımalı".into());
        }
        Ok(hedef)
    }

    /// Host ve Forwarded host alanını aynı origin parser'ıyla doğrular.
    pub fn https_otoritesinden(yazim: &str) -> Result<Self, String> {
        if yazim.is_empty() || yazim.contains(['/', '?', '#', '\\']) {
            return Err("HTTPS otoritesi yalnız host ve isteğe bağlı port taşımalı".into());
        }
        Self::https_origininden(&format!("https://{yazim}"))
    }

    pub const fn semasi(&self) -> AgSemasi {
        self.sema
    }

    pub fn konagi(&self) -> &str {
        &self.konak
    }

    pub const fn kapisi(&self) -> u16 {
        self.kapi
    }

    pub fn yazimi(&self) -> String {
        format!("{}://{}", self.sema.yazimi(), self.otoritesi())
    }

    pub fn otoritesi(&self) -> String {
        let konak = if self.konak.contains(':') {
            format!("[{}]", self.konak)
        } else {
            self.konak.clone()
        };
        if self.kapi == self.sema.varsayilan_kapi() {
            konak
        } else {
            format!("{}:{}", konak, self.kapi)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AgHedefKapsami {
    Kapali,
    Yalniz(BTreeSet<AgHedefi>),
    GenelInternet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YetkinlikPolitikasi {
    izinler: BTreeSet<Yetkinlik>,
    ag_hedefleri: AgHedefKapsami,
    dosya_siniri: DosyaSiniri,
}

impl YetkinlikPolitikasi {
    pub fn kapali() -> Self {
        Self {
            izinler: BTreeSet::new(),
            ag_hedefleri: AgHedefKapsami::Kapali,
            dosya_siniri: DosyaSiniri::ProjeKoku,
        }
    }

    /// Bildirimsiz tek dosya geliştirici akışı. Dış dünya yetkinlikleri açık
    /// kalır; outbound ağ yine yalnız public HTTPS hedefine çıkabilir.
    pub fn gelistirici() -> Self {
        let izinler = Yetkinlik::TUMU
            .into_iter()
            .filter(|yetkinlik| *yetkinlik != Yetkinlik::YerelAg)
            .collect();
        Self {
            izinler,
            ag_hedefleri: AgHedefKapsami::GenelInternet,
            dosya_siniri: DosyaSiniri::HerYer,
        }
    }

    /// Çocuk modu yerel proje dosyalarını korur; ağ, sunucu ve donanımı açmaz.
    pub fn cocuk() -> Self {
        Self {
            izinler: [Yetkinlik::DosyaOkuma, Yetkinlik::DosyaYazma]
                .into_iter()
                .collect(),
            ag_hedefleri: AgHedefKapsami::Kapali,
            dosya_siniri: DosyaSiniri::ProjeKoku,
        }
    }

    pub fn proje(izinler: BTreeSet<Yetkinlik>, hedefler: BTreeSet<AgHedefi>) -> Self {
        let ag_hedefleri = if izinler.contains(&Yetkinlik::Ag) {
            AgHedefKapsami::Yalniz(hedefler)
        } else {
            AgHedefKapsami::Kapali
        };
        Self {
            izinler,
            ag_hedefleri,
            dosya_siniri: DosyaSiniri::ProjeKoku,
        }
    }

    pub fn izin_verir(&self, yetkinlik: Yetkinlik) -> bool {
        self.izinler.contains(&yetkinlik)
    }

    pub fn izinler(&self) -> impl Iterator<Item = Yetkinlik> + '_ {
        self.izinler.iter().copied()
    }

    pub const fn dosya_siniri(&self) -> DosyaSiniri {
        self.dosya_siniri
    }

    pub fn ag_hedefleri(&self) -> impl Iterator<Item = &AgHedefi> {
        match &self.ag_hedefleri {
            AgHedefKapsami::Yalniz(hedefler) => Some(hedefler.iter()),
            AgHedefKapsami::Kapali | AgHedefKapsami::GenelInternet => None,
        }
        .into_iter()
        .flatten()
    }

    pub fn gerektir(&self, yetkinlik: Yetkinlik) -> Result<(), String> {
        if self.izin_verir(yetkinlik) {
            Ok(())
        } else {
            Err(format!(
                "`{}` yetkinliği çalışma politikasında açık değil",
                yetkinlik.yazimi()
            ))
        }
    }

    pub fn ag_istegini_denetle(&self, url: &str) -> Result<AgHedefi, String> {
        self.gerektir(Yetkinlik::Ag)?;
        let hedef = AgHedefi::istekten(url)?;
        match &self.ag_hedefleri {
            AgHedefKapsami::Kapali => Err("outbound ağ hedefleri kapalı".into()),
            AgHedefKapsami::Yalniz(hedefler) if hedefler.contains(&hedef) => Ok(hedef),
            AgHedefKapsami::Yalniz(_) => Err(format!(
                "{} proje bildirimindeki `ağ_hedefleri` listesinde yok",
                hedef.yazimi()
            )),
            AgHedefKapsami::GenelInternet if hedef.semasi() == AgSemasi::Https => Ok(hedef),
            AgHedefKapsami::GenelInternet => {
                Err("bildirimsiz geliştirici ağında yalnız https:// kullanılabilir".into())
            }
        }
    }

    pub fn alt_politikayi_denetle(&self, alt: &Self) -> Result<(), String> {
        for yetkinlik in alt.izinler() {
            self.gerektir(yetkinlik)?;
        }
        for hedef in alt.ag_hedefleri() {
            match &self.ag_hedefleri {
                AgHedefKapsami::Yalniz(hedefler) if hedefler.contains(hedef) => {}
                AgHedefKapsami::GenelInternet if hedef.semasi() == AgSemasi::Https => {}
                _ => {
                    return Err(format!(
                        "paketin {} hedefi üst projenin ağ hedefleri içinde değil",
                        hedef.yazimi()
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn cozulmus_ipyi_denetle(&self, ip: IpAddr) -> Result<(), String> {
        if metadata_ip_mi(ip) {
            return Err(format!(
                "{} metadata/link-local hedefi her profilde yasak",
                ip
            ));
        }
        if public_ip_mi(ip) || self.izin_verir(Yetkinlik::YerelAg) && yerel_ip_mi(ip) {
            Ok(())
        } else {
            Err(format!(
                "{} public internet adresi değil; açık `yerel-ağ` yetkinliği gerekir",
                ip
            ))
        }
    }
}

fn metadata_ip_mi(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.octets()[0..2] == [169, 254],
        IpAddr::V6(ip) => {
            let bolum = ip.segments();
            bolum[0] & 0xffc0 == 0xfe80 || bolum == [0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254]
        }
    }
}

fn yerel_ip_mi(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_loopback()
                || ip.is_private()
                || ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1])
        }
        IpAddr::V6(ip) => ip.is_loopback() || ip.segments()[0] & 0xfe00 == 0xfc00,
    }
}

fn public_ip_mi(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => public_ipv4_mi(ip),
        IpAddr::V6(ip) => public_ipv6_mi(ip),
    }
}

fn public_ipv4_mi(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(a == 0
        || a == 10
        || a == 127
        || a >= 224
        || a == 100 && (64..=127).contains(&b)
        || a == 169 && b == 254
        || a == 172 && (16..=31).contains(&b)
        || a == 192 && b == 168
        || a == 192 && b == 0 && c == 0
        || a == 192 && b == 0 && c == 2
        || a == 192 && b == 88 && c == 99
        || a == 198 && (b == 18 || b == 19)
        || a == 198 && b == 51 && c == 100
        || a == 203 && b == 0 && c == 113)
}

fn public_ipv6_mi(ip: Ipv6Addr) -> bool {
    let bolum = ip.segments();
    if let Some(ipv4) = ip.to_ipv4_mapped() {
        return public_ipv4_mi(ipv4);
    }
    // Fail-closed: IANA'nın ayrılmış üst alanlarını ve iç IPv4'e geçiş
    // sağlayabilen özel önekleri public internet sayma.
    bolum[0] & 0xe000 == 0x2000
        && !(ip.is_loopback()
            || ip.is_multicast()
            || bolum[0] & 0xfe00 == 0xfc00
            || bolum[0] & 0xffc0 == 0xfe80
            || bolum[0] == 0x2001 && bolum[1] <= 0x01ff
            || bolum[0] == 0x2001 && bolum[1] == 0x0db8
            || bolum[0] == 0x2002
            || bolum[0] == 0x3fff && bolum[1] & 0xf000 == 0)
}

#[cfg(test)]
mod testler {
    use super::*;

    fn proje_politikasi(hedef: &str, yerel: bool) -> YetkinlikPolitikasi {
        let mut izinler = BTreeSet::from([Yetkinlik::Ag]);
        if yerel {
            izinler.insert(Yetkinlik::YerelAg);
        }
        YetkinlikPolitikasi::proje(
            izinler,
            BTreeSet::from([AgHedefi::bildirimden(hedef).expect("hedef")]),
        )
    }

    #[test]
    fn ag_hedefi_origini_kanonikler_ve_yolu_ayirir() {
        let hedef = AgHedefi::bildirimden("https://API.Example:443/").expect("origin");
        assert_eq!(hedef.yazimi(), "https://api.example");
        assert_eq!(hedef.otoritesi(), "api.example");
        assert_eq!(
            AgHedefi::https_origininden("https://[2001:0DB8:0:0::1]:443")
                .unwrap()
                .yazimi(),
            "https://[2001:db8::1]"
        );
        assert_eq!(
            AgHedefi::https_origininden("https://API.Example:443/").unwrap(),
            hedef
        );
        assert_eq!(
            AgHedefi::https_otoritesinden("API.Example:443").unwrap(),
            hedef
        );
        assert_eq!(
            AgHedefi::istekten("https://api.example/v1?q=1").unwrap(),
            hedef
        );
        assert!(AgHedefi::bildirimden("https://api.example/v1").is_err());
        assert!(AgHedefi::https_origininden("http://api.example").is_err());
    }

    #[test]
    fn ag_adresi_kullanici_parca_ters_bolu_ve_gecersiz_portu_reddeder() {
        for adres in [
            "https://kisi@api.example",
            "https://api.example/#parca",
            "https://api.example\\hedef",
            "ftp://api.example",
            "https://api.example:+443",
            "https://api.example:0",
            "https://[::1",
        ] {
            assert!(AgHedefi::istekten(adres).is_err(), "{adres}");
        }
    }

    #[test]
    fn proje_ag_izni_tam_originle_eslesir() {
        let politika = proje_politikasi("https://api.example:8443", false);
        assert!(politika
            .ag_istegini_denetle("https://api.example:8443/veri")
            .is_ok());
        assert!(politika
            .ag_istegini_denetle("https://api.example/veri")
            .is_err());
        assert!(politika
            .ag_istegini_denetle("http://api.example:8443/veri")
            .is_err());
    }

    #[test]
    fn dns_sonrasi_private_ve_metadata_fail_closed_kalir() {
        let public = proje_politikasi("https://api.example", false);
        assert!(public
            .cozulmus_ipyi_denetle("8.8.8.8".parse().unwrap())
            .is_ok());
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "100.64.0.1",
            "::1",
            "64:ff9b::a9fe:a9fe",
            "2001::1",
            "2002:a9fe:a9fe::1",
            "3fff::1",
            "5f00::1",
            "fd00::1",
        ] {
            assert!(
                public.cozulmus_ipyi_denetle(ip.parse().unwrap()).is_err(),
                "{ip}"
            );
        }

        let yerel = proje_politikasi("http://127.0.0.1:8080", true);
        assert!(yerel
            .cozulmus_ipyi_denetle("127.0.0.1".parse().unwrap())
            .is_ok());
        for metadata in ["169.254.169.254", "fe80::1", "fd00:ec2::254"] {
            assert!(
                yerel
                    .cozulmus_ipyi_denetle(metadata.parse().unwrap())
                    .is_err(),
                "{metadata}"
            );
        }
    }
}
