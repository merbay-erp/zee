//! K-390 (RFC-0044 B1): `dns_sorgula` olayının IO izi codec'i.
//!
//! Argümanlar `[ad, tür]`; son tarih bütçesi kaydedilmez (K-375 modeli).
//! Başarı `["ok", durum, doğrulanmış, kanonik_ad, kayıtlar…]`, her kayıt
//! etiketiyle sabit alanlı: `a|aaaa ad ttl onaltılık`, `mx ad ttl öncelik
//! hedef`, `txt ad ttl onaltılık`, `tlsa ad ttl kullanım seçici eşleme
//! onaltılık`, `soa ad ttl birincil`. Hata `["hata", tür, mesaj]`.
use super::codec::{bool_coz, bool_yaz};
use crate::dns_teli::{
    onaltilik, onaltilik_coz, Cozum, DnsHataTuru, DnsHatasi, Durum, Kayit, KayitTuru, KayitVerisi,
};

pub(super) const ISLEM: &str = "dns_sorgula";
/// K-390 B1b: `[ad]` → metin sonucu (kamu son ek listesi izde yok; aynı
/// kurumsal alan replay'de izden gelir).
pub(super) const KURUMSAL_ALAN: &str = "kurumsal_alan";

pub(super) fn argumanlar(ad: &str, tur: KayitTuru) -> Vec<String> {
    vec![ad.into(), tur.yazimi().into()]
}

pub(super) fn sonuc_yaz(sonuc: &Result<Cozum, DnsHatasi>) -> Vec<String> {
    let cozum = match sonuc {
        Ok(cozum) => cozum,
        Err(hata) => return vec!["hata".into(), hata.tur.yazimi().into(), hata.mesaj.clone()],
    };
    let mut alanlar = vec![
        "ok".into(),
        cozum.durum.yazimi().into(),
        bool_yaz(cozum.dogrulanmis),
        cozum.kanonik_ad.clone(),
    ];
    for kayit in &cozum.kayitlar {
        let ortak =
            |etiket: &str| vec![etiket.to_string(), kayit.ad.clone(), kayit.ttl.to_string()];
        let mut satir = match &kayit.veri {
            KayitVerisi::A(adres) => [ortak("a"), vec![onaltilik(adres)]].concat(),
            KayitVerisi::Aaaa(adres) => [ortak("aaaa"), vec![onaltilik(adres)]].concat(),
            KayitVerisi::Mx { oncelik, hedef } => {
                [ortak("mx"), vec![oncelik.to_string(), hedef.clone()]].concat()
            }
            KayitVerisi::Txt(baytlar) => [ortak("txt"), vec![onaltilik(baytlar)]].concat(),
            KayitVerisi::Tlsa {
                kullanim,
                secici,
                esleme,
                veri,
            } => [
                ortak("tlsa"),
                vec![
                    kullanim.to_string(),
                    secici.to_string(),
                    esleme.to_string(),
                    onaltilik(veri),
                ],
            ]
            .concat(),
            KayitVerisi::Soa { birincil } => [ortak("soa"), vec![birincil.clone()]].concat(),
            // Cevapta yalnız sorulan tür bulunur; diğerleri çözümde yer almaz.
            KayitVerisi::Cname(_) | KayitVerisi::Diger => continue,
        };
        alanlar.append(&mut satir);
    }
    alanlar
}

fn sayi<T: std::str::FromStr + ToString>(metin: &str) -> Result<T, String> {
    metin
        .parse::<T>()
        .ok()
        .filter(|deger| deger.to_string() == metin)
        .ok_or_else(|| format!("DNS izi sayı alanı kanonik değil: `{metin}`"))
}

fn bayt_dizisi<const N: usize>(metin: &str) -> Result<[u8; N], String> {
    onaltilik_coz(metin)
        .and_then(|baytlar| baytlar.try_into().ok())
        .ok_or_else(|| "DNS izi adres alanı geçersiz".to_string())
}

pub(super) fn sonuc_coz(alanlar: &[String]) -> Result<Result<Cozum, DnsHatasi>, String> {
    match alanlar {
        [etiket, tur, mesaj] if etiket == "hata" => {
            let tur = DnsHataTuru::ayristir(tur)
                .ok_or_else(|| format!("DNS izi hata türü bilinmiyor: `{tur}`"))?;
            Ok(Err(DnsHatasi::yeni(tur, mesaj.clone())))
        }
        [etiket, durum, dogrulanmis, kanonik_ad, kalan @ ..] if etiket == "ok" => {
            let durum = Durum::ayristir(durum)
                .ok_or_else(|| format!("DNS izi durumu bilinmiyor: `{durum}`"))?;
            let dogrulanmis = bool_coz(dogrulanmis)?;
            let mut kayitlar = Vec::new();
            let mut kalan = kalan;
            while let [tur, ad, ttl, ..] = kalan {
                let ttl = sayi::<u32>(ttl)?;
                let (veri, adet) = match (tur.as_str(), &kalan[3..]) {
                    ("a", [adres, ..]) => (KayitVerisi::A(bayt_dizisi(adres)?), 4),
                    ("aaaa", [adres, ..]) => (KayitVerisi::Aaaa(bayt_dizisi(adres)?), 4),
                    ("mx", [oncelik, hedef, ..]) => (
                        KayitVerisi::Mx {
                            oncelik: sayi(oncelik)?,
                            hedef: hedef.clone(),
                        },
                        5,
                    ),
                    ("txt", [veri, ..]) => (
                        KayitVerisi::Txt(onaltilik_coz(veri).ok_or("DNS izi TXT alanı geçersiz")?),
                        4,
                    ),
                    ("tlsa", [kullanim, secici, esleme, veri, ..]) => (
                        KayitVerisi::Tlsa {
                            kullanim: sayi(kullanim)?,
                            secici: sayi(secici)?,
                            esleme: sayi(esleme)?,
                            veri: onaltilik_coz(veri).ok_or("DNS izi TLSA alanı geçersiz")?,
                        },
                        7,
                    ),
                    ("soa", [birincil, ..]) => (
                        KayitVerisi::Soa {
                            birincil: birincil.clone(),
                        },
                        4,
                    ),
                    _ => return Err(format!("DNS izi kayıt alanları geçersiz: `{tur}`")),
                };
                kayitlar.push(Kayit {
                    ad: ad.clone(),
                    ttl,
                    veri,
                });
                kalan = &kalan[adet..];
            }
            if !kalan.is_empty() {
                return Err("DNS izi kayıt alanları eksik".into());
            }
            if (durum == Durum::Var) == kayitlar.is_empty() {
                return Err("DNS izi durumu kayıt sayısıyla uyuşmuyor".into());
            }
            Ok(Ok(Cozum {
                durum,
                kayitlar,
                dogrulanmis,
                kanonik_ad: kanonik_ad.clone(),
            }))
        }
        _ => Err("DNS izi sonuç şeması geçersiz".into()),
    }
}

/// Şema denetiminin son kolu: `dns_sorgula` (K-390) ya da bilinmeyen işlem.
pub(super) fn sema_ya_da_bilinmeyen(olay: &super::IzOlay) -> Result<(), String> {
    if olay.islem == ISLEM {
        semayi_denetle(&olay.argumanlar, &olay.sonuc)
    } else if olay.islem == KURUMSAL_ALAN {
        super::metin_sonuc_semasi(olay, 1)
    } else {
        Err(format!("bilinmeyen işlem: {}", olay.islem))
    }
}

pub(super) fn semayi_denetle(argumanlar: &[String], sonuc: &[String]) -> Result<(), String> {
    match argumanlar {
        [_, tur] if KayitTuru::ayristir(tur).is_some() => {}
        _ => return Err("DNS izi argümanları `[ad, tür]` olmalı".into()),
    }
    sonuc_coz(sonuc).map(|_| ())
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn dns_izi_gidis_donus() {
        let cozum = Cozum {
            durum: Durum::Var,
            kayitlar: vec![
                Kayit {
                    ad: "ornek.test".into(),
                    ttl: 30,
                    veri: KayitVerisi::Mx {
                        oncelik: 10,
                        hedef: "mx.ornek.test".into(),
                    },
                },
                Kayit {
                    ad: "ornek.test".into(),
                    ttl: 30,
                    veri: KayitVerisi::Txt(b"v=spf1 -all\xff".to_vec()),
                },
                Kayit {
                    ad: "_25._tcp.mx.ornek.test".into(),
                    ttl: 1,
                    veri: KayitVerisi::Tlsa {
                        kullanim: 3,
                        secici: 1,
                        esleme: 1,
                        veri: vec![0xAB; 32],
                    },
                },
                Kayit {
                    ad: "ornek.test".into(),
                    ttl: 0,
                    veri: KayitVerisi::Aaaa([
                        0x20, 1, 0xd, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    ]),
                },
            ],
            dogrulanmis: true,
            kanonik_ad: "ornek.test".into(),
        };
        let alanlar = sonuc_yaz(&Ok(cozum.clone()));
        assert_eq!(sonuc_coz(&alanlar).unwrap(), Ok(cozum));
        let hata = Err(DnsHatasi::yeni(DnsHataTuru::SunucuHatasi, "SERVFAIL"));
        assert_eq!(sonuc_coz(&sonuc_yaz(&hata)).unwrap(), hata);
        let bos = Ok(Cozum {
            durum: Durum::AlanYok,
            kayitlar: Vec::new(),
            dogrulanmis: false,
            kanonik_ad: "yok.test".into(),
        });
        assert_eq!(sonuc_coz(&sonuc_yaz(&bos)).unwrap(), bos);
    }

    #[test]
    fn bozuk_dns_izi_reddedilir() {
        let s = |a: &[&str]| a.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert!(sonuc_coz(&s(&["ok", "var", "1", "a.test"])).is_err());
        assert!(sonuc_coz(&s(&[
            "ok", "alan_yok", "0", "a.test", "a", "a.test", "1", "7f000001"
        ]))
        .is_err());
        assert!(sonuc_coz(&s(&[
            "ok", "var", "1", "a.test", "a", "a.test", "01", "7f000001"
        ]))
        .is_err());
        assert!(sonuc_coz(&s(&[
            "ok", "var", "1", "a.test", "a", "a.test", "1", "7f00"
        ]))
        .is_err());
        assert!(sonuc_coz(&s(&["hata", "uydurma", "x"])).is_err());
        assert!(semayi_denetle(&s(&["a.test", "ptr"]), &s(&["hata", "ret", "x"])).is_err());
    }
}
