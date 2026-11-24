//! K-390 (RFC-0044 B1, ADR-169): DNS sorgu intrinsic'lerinin yorumlayıcı
//! sınırı. Başarı kayıt başına bir Metin sözlüğüdür (boş liste = kayıt yok
//! ya da alan yok); başarısız Sonuç yalnız geçici hatadır (C031, `tür`).
use super::*;
use crate::dns_teli::{Cozum, DnsHatasi, KayitTuru, KayitVerisi};
use crate::intrinsic::{DNS_ADRES, DNS_MX, DNS_TLSA, DNS_TXT};

/// Tek sorgunun Zee tarafındaki en uzun bütçesi (adaptör de 5 sn ile keser).
const SORGU_MS: i64 = 5_000;

pub(super) fn intrinsic_degerlendir(
    kimlik: &str,
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    let [Deger::Metin(ad)] = degerler else {
        return Err(ic_hata(satir));
    };
    let turler: &[KayitTuru] = match kimlik {
        DNS_MX => &[KayitTuru::Mx],
        DNS_TXT => &[KayitTuru::Txt],
        DNS_ADRES => &[KayitTuru::A, KayitTuru::Aaaa],
        DNS_TLSA => &[KayitTuru::Tlsa],
        _ => return Err(ic_hata(satir)),
    };
    let mut satirlar = Vec::new();
    for tur in turler {
        let sure = son_tarih_kalani(io, satir)?.map_or(SORGU_MS, |(_, kalan)| kalan.min(SORGU_MS));
        let sonuc = io.dns_sorgula(ad, *tur, sure);
        son_tarihi_denetle(io, satir)?;
        match sonuc {
            Ok(cozum) => satirlar.extend(satirlari(&cozum)),
            Err(hata) => return Ok(hata_sonucu(hata)),
        }
    }
    Ok(Deger::Sonuc {
        basarili: true,
        icerik: Box::new(Deger::Liste(satirlar)),
    })
}

fn metin(deger: impl Into<String>) -> Deger {
    Deger::Metin(deger.into())
}

fn ad_yazimi(ad: &str) -> String {
    if ad.is_empty() {
        ".".into()
    } else {
        ad.into()
    }
}

/// Her satırda `ad`, `ttl`, `doğrulanmış` ("doğru"/"yanlış") ve türe özgü alanlar.
fn satirlari(cozum: &Cozum) -> Vec<Deger> {
    let dogrulanmis = if cozum.dogrulanmis {
        "doğru"
    } else {
        "yanlış"
    };
    cozum
        .kayitlar
        .iter()
        .filter_map(|kayit| {
            let mut alanlar = vec![
                ("ad".to_string(), metin(ad_yazimi(&kayit.ad))),
                ("ttl".to_string(), metin(kayit.ttl.to_string())),
                ("doğrulanmış".to_string(), metin(dogrulanmis)),
            ];
            match &kayit.veri {
                KayitVerisi::A(adres) => {
                    alanlar.push(("aile".into(), metin("ipv4")));
                    alanlar.push((
                        "adres".into(),
                        metin(std::net::Ipv4Addr::from(*adres).to_string()),
                    ));
                }
                KayitVerisi::Aaaa(adres) => {
                    alanlar.push(("aile".into(), metin("ipv6")));
                    alanlar.push((
                        "adres".into(),
                        metin(std::net::Ipv6Addr::from(*adres).to_string()),
                    ));
                }
                KayitVerisi::Mx { oncelik, hedef } => {
                    alanlar.push(("öncelik".into(), metin(oncelik.to_string())));
                    alanlar.push(("hedef".into(), metin(ad_yazimi(hedef))));
                }
                // TXT baytları SPF/DKIM için ASCII'dir; UTF-8 olmayan bayt U+FFFD olur.
                KayitVerisi::Txt(baytlar) => {
                    alanlar.push(("metin".into(), metin(String::from_utf8_lossy(baytlar))));
                }
                KayitVerisi::Tlsa {
                    kullanim,
                    secici,
                    esleme,
                    veri,
                } => {
                    alanlar.push(("kullanım".into(), metin(kullanim.to_string())));
                    alanlar.push(("seçici".into(), metin(secici.to_string())));
                    alanlar.push(("eşleme".into(), metin(esleme.to_string())));
                    alanlar.push(("veri".into(), metin(crate::dns_teli::onaltilik(veri))));
                }
                KayitVerisi::Soa { .. } | KayitVerisi::Cname(_) | KayitVerisi::Diger => {
                    return None
                }
            }
            Some(Deger::Sozluk(alanlar))
        })
        .collect()
}

fn hata_sonucu(hata: DnsHatasi) -> Deger {
    Deger::Sonuc {
        basarili: false,
        icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
            kod: "C031".into(),
            mesaj: format!("DNS sorgusu geçici olarak başarısız: {}", hata.mesaj),
            neden: None,
            veri: vec![("tür".into(), metin(hata.tur.yazimi()))],
        }))),
    }
}
