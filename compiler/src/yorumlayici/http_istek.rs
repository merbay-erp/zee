use super::*;

pub(super) fn degerlendir(
    kimlik: &str,
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    son_tarihi_denetle(io, satir)?;
    let sonuc = match (kimlik, degerler) {
        (crate::intrinsic::HTTP_JSON, [Deger::Metin(metin)]) => {
            json_sozlugu(metin).map(Deger::Sozluk)
        }
        (crate::intrinsic::HTTP_GIZLI_ORTAM, [Deger::Metin(ad)]) => {
            io.http_gizli_ortam(ad).map(Deger::Metin)
        }
        (
            crate::intrinsic::HTTP_ISTEK,
            [Deger::Metin(url), Deger::Metin(yontem), Deger::Sozluk(basliklar), Deger::Metin(govde), Deger::TamSayi(sure)],
        ) => {
            let mut basliklar_metin = Vec::new();
            for (ad, deger) in basliklar {
                let Deger::Metin(metin) = deger else {
                    return Err(ic_hata(satir));
                };
                basliklar_metin.push((ad.clone(), metin.clone()));
            }
            let kalan = son_tarih_kalani(io, satir)?.map(|(_, kalan)| kalan);
            let sure = kalan.map_or(*sure, |kalan| (*sure).min(kalan));
            io.http_istek(url, yontem, &basliklar_metin, govde, sure)
                .map(|(durum, govde)| {
                    Deger::Sozluk(vec![
                        ("durum".into(), Deger::Metin(durum.to_string())),
                        ("gövde".into(), Deger::Metin(govde)),
                    ])
                })
        }
        _ => return Err(ic_hata(satir)),
    };
    son_tarihi_denetle(io, satir)?;
    Ok(match sonuc {
        Ok(icerik) => Deger::Sonuc {
            basarili: true,
            icerik: Box::new(icerik),
        },
        Err(_) => Deger::Sonuc {
            basarili: false,
            icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
                kod: "C018".into(),
                mesaj: "HTTP işlemi başarısız; sır içeren ayrıntılar gizlendi".into(),
                neden: None,
                veri: Vec::new(),
            }))),
        },
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn json_sozlugu(metin: &str) -> Result<Vec<(String, Deger)>, String> {
    use serde::de::{MapAccess, Visitor};
    use serde::Deserialize;
    struct Nesne(Vec<(String, serde_json::Value)>);
    impl<'de> Deserialize<'de> for Nesne {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct Ziyaret;
            impl<'de> Visitor<'de> for Ziyaret {
                type Value = Nesne;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("tekil anahtarlı JSON nesnesi")
                }
                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Nesne, A::Error> {
                    let mut keys = std::collections::BTreeSet::new();
                    let mut alanlar = Vec::new();
                    while let Some(key) = map.next_key::<String>()? {
                        if alanlar.len() >= 128 || !keys.insert(key.clone()) {
                            return Err(serde::de::Error::custom("JSON alan sınırı veya tekrar"));
                        }
                        alanlar.push((key, map.next_value()?));
                    }
                    Ok(Nesne(alanlar))
                }
            }
            d.deserialize_map(Ziyaret)
        }
    }
    if metin.len() > 1_048_576 {
        return Err("HTTP JSON boyut sınırı".into());
    }
    let nesne = serde_json::from_str::<Nesne>(metin).map_err(|_| "HTTP JSON geçersiz")?;
    nesne
        .0
        .into_iter()
        .map(|(k, v)| {
            serde_json::to_string(&v)
                .map(|s| (k, Deger::Metin(s)))
                .map_err(|_| "HTTP JSON geçersiz".into())
        })
        .collect()
}
#[cfg(target_arch = "wasm32")]
fn json_sozlugu(_: &str) -> Result<Vec<(String, Deger)>, String> {
    Err("Native HTTP JSON adaptörü desteklenmiyor".into())
}

// K-240: mevcut GET tel/tanı/deadline akışı HTTP handler sahibinde kalır.
pub(super) fn getir(url: &str, io: &mut dyn GirdiCikti, satir: usize) -> Result<Deger, Tani> {
    // Sıra verme sırasında kardeş görev saati ilerletmiş veya
    // üst son tarihi doldurmuş olabilir. Adaptöre eski kalan
    // süreyle girmek yerine tam çağrı öncesi yeniden hesapla.
    let zaman_asimi_ms = son_tarih_kalani(io, satir)?.map(|(_, kalan)| kalan);
    let (durum, govde) = match io.http_getir(url, zaman_asimi_ms) {
        Ok(yanit) => {
            son_tarihi_denetle(io, satir)?;
            yanit
        }
        Err(hata) => {
            son_tarihi_denetle(io, satir)?;
            return Err(Tani::yeni(
                "C018",
                format!("Ağ isteği başarısız: {}.", hata),
                satir,
                1,
                1,
            )
            .onerili(
                "Ağ hatasını yönetmek için yeni HTTP isteğini dene kalıbıyla Sonuç al (RFC-0033)."
                    .into(),
            ));
        }
    };
    Ok(Deger::AgYaniti { durum, govde })
}
