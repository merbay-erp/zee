//! K-397 (RFC-0044 B4a, ADR-173): giden TCP hedefinin politika denetimi.
//!
//! Saf kural: hedef `ağ_hedefleri`ndeki `tcp://<konak>:<kapı>` tam hedefiyle
//! ya da yalnız 25 kapısındaki `tcp://*:25` jokeriyle eşleşmeli. Ad ise
//! verilen çözücüyle **bir kez** çözülür; çözülen adreslerden biri bile
//! izinsizse (fail-closed, HTTP istemcisinin `PolitikaliCozucu` kuralıyla
//! aynı) hedef reddedilir. Dönen `SocketAddr` bağlantının açılacağı tek
//! adrestir: host adı ikinci kez çözmez (DNS yeniden bağlama penceresi yok).

use super::{
    origin::ayristir, public_ip_mi, AgHedefKapsami, AgHedefi, AgSemasi, Yetkinlik,
    YetkinlikPolitikasi,
};
use std::collections::BTreeSet;
use std::net::{IpAddr, SocketAddr};

/// K-397 (ADR-173): yalnız bildirimde geçerli iki dar joker. `tcp://*:25`
/// her public IP'nin 25 kapısı; `https://mta-sts.*` RFC 8461 ilke sunucusu.
pub(super) const TCP_JOKERI: &str = "*";
pub(super) const MTA_STS_JOKERI: &str = "mta-sts.*";
/// Joker TCP hedefinin tek izinli kapısı (SMTP MX teslimi).
pub(super) const TCP_JOKER_KAPISI: u16 = 25;

/// `*` ya da `mta-sts.*` içeren otorite: yalnız iki dar biçim kabul edilir.
pub(super) fn joker(
    sema: AgSemasi,
    yetki: &str,
    serbest: bool,
) -> Result<Option<AgHedefi>, String> {
    if sema == AgSemasi::Tcp && (yetki.ends_with(']') || !yetki.contains(':')) {
        return Err("tcp:// hedefi kapıyı açıkça yazmalı (ör. tcp://mx.ornek:25)".into());
    }
    if !yetki.contains('*') {
        return Ok(None);
    }
    if !serbest {
        return Err("istek adresi joker (`*`) taşıyamaz".into());
    }
    match (sema, yetki) {
        (AgSemasi::Tcp, "*:25") => Ok(Some(AgHedefi {
            sema,
            konak: TCP_JOKERI.into(),
            kapi: TCP_JOKER_KAPISI,
        })),
        (AgSemasi::Https, "mta-sts.*" | "mta-sts.*:443") => Ok(Some(AgHedefi {
            sema,
            konak: MTA_STS_JOKERI.into(),
            kapi: 443,
        })),
        _ => Err(
            "joker yalnız `tcp://*:25` (public IP'lerin SMTP kapısı) ve `https://mta-sts.*` (RFC 8461 ilke sunucusu) biçiminde yazılabilir"
                .into(),
        ),
    }
}

/// RFC 8461 §3.2 ilke yolu; `https://mta-sts.*` jokeri yalnız bu yolu açar.
pub const MTA_STS_YOLU: &str = "/.well-known/mta-sts.txt";
/// Bir addan değerlendirilen en çok adres (saldırganın denetlediği DNS yanıtı).
const AZAMI_ADRES: usize = 16;

/// Giden hedef reddi: `tür` Zee'ye C030 verisi olarak gider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GidenHedefHatasi {
    pub tur: &'static str,
    pub mesaj: String,
}

impl GidenHedefHatasi {
    fn yeni(tur: &'static str, mesaj: impl Into<String>) -> Self {
        Self {
            tur,
            mesaj: mesaj.into(),
        }
    }
}

/// `https://mta-sts.<alan>/.well-known/mta-sts.txt` isteği jokerle mi izinli?
fn mta_sts_jokeri_eslesir(hedefler: &BTreeSet<AgHedefi>, hedef: &AgHedefi, kuyruk: &str) -> bool {
    let joker = AgHedefi {
        sema: AgSemasi::Https,
        konak: MTA_STS_JOKERI.into(),
        kapi: 443,
    };
    hedef.sema == AgSemasi::Https
        && hedef.kapi == 443
        && kuyruk == MTA_STS_YOLU
        && hedef.konak.parse::<IpAddr>().is_err()
        && hedef
            .konak
            .strip_prefix("mta-sts.")
            .is_some_and(|alan| alan.contains('.') || !alan.is_empty())
        && hedefler.contains(&joker)
}

impl YetkinlikPolitikasi {
    /// HTTP(S) isteği denetimi; ikinci değer isteğin yalnız `https://mta-sts.*`
    /// jokeriyle izinli olduğunu söyler (K-397): o zaman çözülen IP `yerel-ağ`
    /// açık olsa bile yalnız public olabilir.
    pub fn ag_istegini_ayrintili_denetle(&self, url: &str) -> Result<(AgHedefi, bool), String> {
        self.gerektir(Yetkinlik::Ag)?;
        let (hedef, kuyruk) = ayristir(url, false)?;
        if hedef.semasi() == AgSemasi::Tcp {
            return Err("tcp:// hedefi HTTP isteğinde kullanılamaz".into());
        }
        match &self.ag_hedefleri {
            AgHedefKapsami::Kapali => Err("outbound ağ hedefleri kapalı".into()),
            AgHedefKapsami::Yalniz(hedefler) if hedefler.contains(&hedef) => Ok((hedef, false)),
            AgHedefKapsami::Yalniz(hedefler)
                if mta_sts_jokeri_eslesir(hedefler, &hedef, kuyruk) =>
            {
                Ok((hedef, true))
            }
            AgHedefKapsami::Yalniz(_) => Err(format!(
                "{} proje bildirimindeki `ağ_hedefleri` listesinde yok",
                hedef.yazimi()
            )),
            AgHedefKapsami::GenelInternet if hedef.semasi().sifreli() => Ok((hedef, false)),
            AgHedefKapsami::GenelInternet => Err(
                "bildirimsiz geliştirici ağında yalnız https:// ve smtps:// kullanılabilir".into(),
            ),
        }
    }

    /// K-397: joker eşleşmesinde yalnız public adres; aksi hâlde ortak kural.
    pub fn cozulmus_ipyi_kipli_denetle(
        &self,
        ip: IpAddr,
        yalniz_public: bool,
    ) -> Result<(), String> {
        if yalniz_public && !public_ip_mi(ip) {
            return Err(format!(
                "{} public internet adresi değil; joker hedef `yerel-ağ` ile genişlemez",
                ip
            ));
        }
        self.cozulmus_ipyi_denetle(ip)
    }

    /// `"<ana>:<kapı>"` giden hedefini denetler ve bağlanılacak tek adresi
    /// verir. `cozucu` yalnız ad için ve en çok bir kez çağrılır.
    pub fn giden_tcp_hedefini_coz(
        &self,
        adres: &str,
        cozucu: impl FnOnce(&str, u16) -> std::io::Result<Vec<SocketAddr>>,
    ) -> Result<SocketAddr, GidenHedefHatasi> {
        if !self.izin_verir(Yetkinlik::Ag) {
            return Err(GidenHedefHatasi::yeni(
                "hedef_yasak",
                "`ağ` yetkinliği çalışma politikasında açık değil",
            ));
        }
        if adres.contains("://") {
            return Err(GidenHedefHatasi::yeni(
                "geçersiz",
                "giden adres şemasız `<ana>:<kapı>` olmalı",
            ));
        }
        let hedef = AgHedefi::istekten(&format!("tcp://{adres}"))
            .map_err(|neden| GidenHedefHatasi::yeni("geçersiz", neden))?;
        let joker = match &self.ag_hedefleri {
            AgHedefKapsami::Yalniz(hedefler) if hedefler.contains(&hedef) => false,
            AgHedefKapsami::Yalniz(hedefler)
                if hedef.kapi == TCP_JOKER_KAPISI
                    && hedefler.contains(&AgHedefi {
                        sema: AgSemasi::Tcp,
                        konak: TCP_JOKERI.into(),
                        kapi: TCP_JOKER_KAPISI,
                    }) =>
            {
                true
            }
            AgHedefKapsami::Yalniz(_) => {
                return Err(GidenHedefHatasi::yeni(
                    "hedef_yasak",
                    format!(
                        "{} proje bildirimindeki `ağ_hedefleri` listesinde yok",
                        hedef.yazimi()
                    ),
                ))
            }
            AgHedefKapsami::Kapali | AgHedefKapsami::GenelInternet => {
                return Err(GidenHedefHatasi::yeni(
                    "hedef_yasak",
                    "giden TCP yalnız proje.dil `ağ_hedefleri` içindeki tcp:// hedefiyle açılır",
                ))
            }
        };
        let adresler = match hedef.konak.parse::<IpAddr>() {
            Ok(ip) => vec![SocketAddr::new(ip, hedef.kapi)],
            Err(_) => cozucu(&hedef.konak, hedef.kapi).map_err(|_| {
                GidenHedefHatasi::yeni("çözülemedi", format!("{} çözülemedi", hedef.konak))
            })?,
        };
        if adresler.is_empty() {
            return Err(GidenHedefHatasi::yeni(
                "çözülemedi",
                format!("{} adres vermedi", hedef.konak),
            ));
        }
        for adres in adresler.iter().take(AZAMI_ADRES) {
            giden_ipyi_denetle(self, adres.ip(), joker)
                .map_err(|neden| GidenHedefHatasi::yeni("hedef_yasak", neden))?;
        }
        Ok(SocketAddr::new(adresler[0].ip(), hedef.kapi))
    }
}

/// Çözülen adresin giden TCP kuralı: IPv4-mapped IPv6 her profilde ret,
/// joker yalnız public, tam hedef spec/23 kuralı.
fn giden_ipyi_denetle(
    politika: &YetkinlikPolitikasi,
    ip: IpAddr,
    joker: bool,
) -> Result<(), String> {
    if let IpAddr::V6(v6) = ip {
        if v6.to_ipv4_mapped().is_some() {
            return Err(format!(
                "{ip} IPv4-mapped IPv6 adresi giden TCP hedefi olamaz"
            ));
        }
    }
    politika.cozulmus_ipyi_kipli_denetle(ip, joker)
}

#[cfg(test)]
mod testler {
    use super::*;

    fn politika(hedefler: &[&str], yerel: bool) -> YetkinlikPolitikasi {
        let mut izinler = BTreeSet::from([Yetkinlik::Ag]);
        if yerel {
            izinler.insert(Yetkinlik::YerelAg);
        }
        YetkinlikPolitikasi::proje(
            izinler,
            hedefler
                .iter()
                .map(|h| AgHedefi::bildirimden(h).expect("hedef"))
                .collect(),
        )
    }

    fn cozulmez(_: &str, _: u16) -> std::io::Result<Vec<SocketAddr>> {
        panic!("IP hedefinde çözücü çağrılmamalı")
    }

    #[test]
    fn joker_yalniz_25_ve_yalniz_bildirimde() {
        assert!(AgHedefi::bildirimden("tcp://*:25").is_ok());
        for yanlis in [
            "tcp://*:26",
            "tcp://*:2525",
            "tcp://*",
            "tcp://*.ornek:25",
            "https://*.ornek",
            "https://mta-sts.*:8443",
            "smtp://*:25",
        ] {
            assert!(AgHedefi::bildirimden(yanlis).is_err(), "{yanlis}");
        }
        assert!(AgHedefi::istekten("tcp://*:25").is_err());
        assert!(AgHedefi::istekten("https://mta-sts.*/.well-known/mta-sts.txt").is_err());
        assert!(
            AgHedefi::bildirimden("tcp://mx.ornek").is_err(),
            "kapı zorunlu"
        );
        assert!(AgHedefi::bildirimden("tcp://[2001:db8::1]").is_err());
        assert_eq!(
            AgHedefi::bildirimden("tcp://MX.Ornek:25").unwrap().yazimi(),
            "tcp://mx.ornek:25"
        );
    }

    #[test]
    fn joker_ozel_araliklari_yerel_ag_acikken_bile_reddeder() {
        let p = politika(&["tcp://*:25"], true);
        assert_eq!(
            p.giden_tcp_hedefini_coz("8.8.8.8:25", cozulmez).unwrap(),
            "8.8.8.8:25".parse().unwrap()
        );
        assert!(p
            .giden_tcp_hedefini_coz("[2a00:1450::1]:25", cozulmez)
            .is_ok());
        for ip in [
            "0.0.0.1",
            "10.1.2.3",
            "100.64.0.1",
            "100.127.255.254",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.0.1",
            "172.31.255.255",
            "192.0.0.8",
            "192.0.2.1",
            "192.88.99.1",
            "192.168.1.1",
            "198.18.0.1",
            "198.51.100.7",
            "203.0.113.7",
            "224.0.0.1",
            "239.255.255.250",
            "240.0.0.1",
            "255.255.255.255",
            "[::]",
            "[::1]",
            "[::ffff:8.8.8.8]",
            "[::ffff:127.0.0.1]",
            "[64:ff9b::808:808]",
            "[fc00::1]",
            "[fd00:ec2::254]",
            "[fe80::1]",
            "[ff02::1]",
            "[2001::1]",
            "[2001:db8::1]",
            "[2002:808:808::1]",
            "[3fff::1]",
            "[5f00::1]",
        ] {
            let sonuc = p.giden_tcp_hedefini_coz(&format!("{ip}:25"), cozulmez);
            assert_eq!(sonuc.unwrap_err().tur, "hedef_yasak", "{ip}");
        }
    }

    #[test]
    fn kapi_ve_hedef_listesi_disi_reddedilir() {
        let p = politika(&["tcp://*:25"], false);
        for adres in ["8.8.8.8:587", "8.8.8.8:22", "8.8.8.8:6379"] {
            assert_eq!(
                p.giden_tcp_hedefini_coz(adres, cozulmez).unwrap_err().tur,
                "hedef_yasak",
                "{adres}"
            );
        }
        for adres in [
            "8.8.8.8",
            "8.8.8.8:0",
            "tcp://8.8.8.8:25",
            "*:25",
            "a b:25",
            "",
        ] {
            assert_eq!(
                p.giden_tcp_hedefini_coz(adres, cozulmez).unwrap_err().tur,
                "geçersiz",
                "{adres}"
            );
        }
        let kapali = YetkinlikPolitikasi::gelistirici_agli();
        assert_eq!(
            kapali
                .giden_tcp_hedefini_coz("8.8.8.8:25", cozulmez)
                .unwrap_err()
                .tur,
            "hedef_yasak"
        );
        let agsiz = YetkinlikPolitikasi::proje(
            BTreeSet::new(),
            BTreeSet::from([AgHedefi::bildirimden("tcp://*:25").unwrap()]),
        );
        assert_eq!(
            agsiz
                .giden_tcp_hedefini_coz("8.8.8.8:25", cozulmez)
                .unwrap_err()
                .tur,
            "hedef_yasak"
        );
    }

    #[test]
    fn tam_hedef_yerel_adrese_yalniz_yerel_agla_izin_verir() {
        let yerel = politika(&["tcp://127.0.0.1:2525"], true);
        assert!(yerel
            .giden_tcp_hedefini_coz("127.0.0.1:2525", cozulmez)
            .is_ok());
        assert_eq!(
            yerel
                .giden_tcp_hedefini_coz("127.0.0.1:2526", cozulmez)
                .unwrap_err()
                .tur,
            "hedef_yasak"
        );
        let public = politika(&["tcp://127.0.0.1:2525"], false);
        assert_eq!(
            public
                .giden_tcp_hedefini_coz("127.0.0.1:2525", cozulmez)
                .unwrap_err()
                .tur,
            "hedef_yasak"
        );
        let metadata = politika(&["tcp://169.254.169.254:80"], true);
        assert_eq!(
            metadata
                .giden_tcp_hedefini_coz("169.254.169.254:80", cozulmez)
                .unwrap_err()
                .tur,
            "hedef_yasak"
        );
    }

    #[test]
    fn ad_bir_kez_cozulur_ve_denetlenen_adres_doner() {
        let p = politika(&["tcp://*:25"], true);
        let mut cagri = 0;
        let adres = p
            .giden_tcp_hedefini_coz("mx.ornek:25", |ad, kapi| {
                cagri += 1;
                assert_eq!((ad, kapi), ("mx.ornek", 25));
                Ok(vec!["93.184.216.34:9".parse().unwrap()])
            })
            .unwrap();
        assert_eq!(cagri, 1);
        // Çözücünün kapısı yok sayılır; bağlantı hedefin kapısına açılır.
        assert_eq!(adres, "93.184.216.34:25".parse().unwrap());
    }

    #[test]
    fn yeniden_baglama_karisik_ya_da_ozel_yanit_fail_closed() {
        let p = politika(&["tcp://*:25"], true);
        for yanit in [
            vec!["127.0.0.1:25"],
            vec!["93.184.216.34:25", "10.0.0.5:25"],
            vec!["[::ffff:10.0.0.5]:25"],
        ] {
            let yanit: Vec<SocketAddr> = yanit.iter().map(|a| a.parse().unwrap()).collect();
            let hata = p
                .giden_tcp_hedefini_coz("kotu.ornek:25", move |_, _| Ok(yanit))
                .unwrap_err();
            assert_eq!(hata.tur, "hedef_yasak");
        }
        let bos = p.giden_tcp_hedefini_coz("yok.ornek:25", |_, _| Ok(Vec::new()));
        assert_eq!(bos.unwrap_err().tur, "çözülemedi");
        let hata = p.giden_tcp_hedefini_coz("yok.ornek:25", |_, _| {
            Err(std::io::ErrorKind::NotFound.into())
        });
        assert_eq!(hata.unwrap_err().tur, "çözülemedi");
    }

    #[test]
    fn mta_sts_jokeri_yol_kapi_ve_onek_ister() {
        let p = politika(&["https://mta-sts.*"], true);
        let (_, joker) = p
            .ag_istegini_ayrintili_denetle("https://mta-sts.alici.ornek/.well-known/mta-sts.txt")
            .unwrap();
        assert!(joker);
        for url in [
            "https://mta-sts.alici.ornek/",
            "https://mta-sts.alici.ornek/.well-known/mta-sts.txt?x=1",
            "https://mta-sts.alici.ornek:8443/.well-known/mta-sts.txt",
            "https://alici.ornek/.well-known/mta-sts.txt",
            "https://mta-stsx.alici.ornek/.well-known/mta-sts.txt",
            "http://mta-sts.alici.ornek/.well-known/mta-sts.txt",
            "tcp://mta-sts.alici.ornek:443",
        ] {
            assert!(p.ag_istegini_ayrintili_denetle(url).is_err(), "{url}");
        }
        assert!(p
            .cozulmus_ipyi_kipli_denetle("127.0.0.1".parse().unwrap(), true)
            .is_err());
        assert!(p
            .cozulmus_ipyi_kipli_denetle("127.0.0.1".parse().unwrap(), false)
            .is_ok());
    }
}
