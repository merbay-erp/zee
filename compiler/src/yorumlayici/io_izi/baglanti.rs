//! K-381: bağlantı olaylarının ve işlemlerinin kanonik iz kodlaması.
//!
//! `baglanti_kaydet [ad] → tamsayı sonucu`, `baglanti_bekle [bekleyenler,
//! uyanma] → ok + olaylar | hata`, `baglanti_islemi [kimlik, işlem, …] →
//! satir|tamam|bekle|hata`, `baglanti_gunluk [kimlik, neden] → ∅`.
use super::*;
use crate::yorumlayici::{
    BaglantiIslemi, BaglantiOlayi, BaglantiSonucu, TlsIstemciKaydi, YeniBaglanti,
};

pub(super) fn bekle_argumanlari(bekleyenler: &[u64], uyanma: Option<i64>) -> Vec<String> {
    vec![
        bekleyenler
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(","),
        uyanma.map_or_else(|| "yok".into(), |an| an.to_string()),
    ]
}

pub(super) fn olaylari_yaz(sonuc: &Result<Vec<BaglantiOlayi>, String>) -> Vec<String> {
    match sonuc {
        Err(hata) => vec!["hata".into(), hata.clone()],
        Ok(olaylar) => {
            let mut alanlar = vec!["ok".to_string()];
            for olay in olaylar {
                match olay {
                    BaglantiOlayi::Yeni(y) => alanlar.extend([
                        "yeni".into(),
                        y.kimlik.to_string(),
                        y.dinleyici.clone(),
                        y.uzak_adres.clone(),
                        y.uzak_kapi.to_string(),
                        y.yerel_kapi.to_string(),
                        bool_yaz(y.tls),
                    ]),
                    BaglantiOlayi::Hazir(kimlik) => {
                        alanlar.extend(["hazir".into(), kimlik.to_string()])
                    }
                    BaglantiOlayi::Kapanis => alanlar.push("kapanis".into()),
                }
            }
            alanlar
        }
    }
}

pub(super) fn olaylari_coz(
    alanlar: &[String],
) -> Result<Result<Vec<BaglantiOlayi>, String>, String> {
    match alanlar {
        [etiket, hata] if etiket == "hata" => return Ok(Err(hata.clone())),
        [etiket, ..] if etiket == "ok" => {}
        _ => return Err("bağlantı beklemesi `ok` ya da `hata` ile başlamalı".into()),
    }
    let sayi = |alan: &str| -> Result<u64, String> {
        alan.parse::<u64>()
            .ok()
            .filter(|n| n.to_string() == alan)
            .ok_or_else(|| "bağlantı kimliği kanonik sayı değil".to_string())
    };
    let kapi = |alan: &str| -> Result<u16, String> {
        alan.parse::<u16>()
            .ok()
            .filter(|n| n.to_string() == alan)
            .ok_or_else(|| "kapı kanonik sayı değil".to_string())
    };
    let mut olaylar = Vec::new();
    let mut kalan = &alanlar[1..];
    while let Some((etiket, geri)) = kalan.split_first() {
        match etiket.as_str() {
            "yeni" if geri.len() >= 6 => {
                olaylar.push(BaglantiOlayi::Yeni(YeniBaglanti {
                    kimlik: sayi(&geri[0])?,
                    dinleyici: geri[1].clone(),
                    uzak_adres: geri[2].clone(),
                    uzak_kapi: kapi(&geri[3])?,
                    yerel_kapi: kapi(&geri[4])?,
                    tls: bool_coz(&geri[5])?,
                }));
                kalan = &geri[6..];
            }
            "hazir" if !geri.is_empty() => {
                olaylar.push(BaglantiOlayi::Hazir(sayi(&geri[0])?));
                kalan = &geri[1..];
            }
            "kapanis" => {
                olaylar.push(BaglantiOlayi::Kapanis);
                kalan = geri;
            }
            _ => return Err("bilinmeyen ya da eksik bağlantı olayı".into()),
        }
    }
    Ok(Ok(olaylar))
}

pub(super) fn islem_argumanlari(kimlik: u64, islem: &BaglantiIslemi) -> Vec<String> {
    let mut alanlar = vec![kimlik.to_string()];
    match islem {
        BaglantiIslemi::SatirOku => alanlar.push("oku".into()),
        BaglantiIslemi::SatirYaz(metin) => alanlar.extend(["yaz".into(), metin.clone()]),
        BaglantiIslemi::Kapat => alanlar.push("kapat".into()),
        BaglantiIslemi::TlsBaslat => alanlar.push("tls".into()),
        BaglantiIslemi::NoktaBlogu => alanlar.push("nokta".into()),
        BaglantiIslemi::BaytBlogu(adet) => alanlar.extend(["blok".into(), adet.to_string()]),
        BaglantiIslemi::BaytOku(adet) => alanlar.extend(["bayt".into(), adet.to_string()]),
        BaglantiIslemi::ParcaYaz(metin) => alanlar.extend(["parca".into(), metin.clone()]),
        BaglantiIslemi::DosyaAktar(yol) => alanlar.extend(["aktar".into(), yol.clone()]),
        BaglantiIslemi::SatirOkuSureli(ms) => alanlar.extend(["oku_sure".into(), ms.to_string()]),
        BaglantiIslemi::Engelle(son) => alanlar.extend([
            "engelle".into(),
            son.map_or_else(|| "yok".into(), |an| an.to_string()),
        ]),
        BaglantiIslemi::GidenBaglan(adres, son) => alanlar.extend([
            "baglan".into(),
            adres.clone(),
            son.map_or_else(|| "yok".into(), |an| an.to_string()),
        ]),
        BaglantiIslemi::TlsIstemci(ad) => alanlar.extend(["tls_istemci".into(), ad.clone()]),
    }
    alanlar
}

pub(super) fn islem_semasi(argumanlar: &[String]) -> Result<(), String> {
    let gecerli = match argumanlar {
        [kimlik, islem] => {
            kanonik_tamsayi::<u64>(kimlik)
                && matches!(islem.as_str(), "oku" | "kapat" | "tls" | "nokta")
        }
        [kimlik, islem, son] if islem == "engelle" => {
            kanonik_tamsayi::<u64>(kimlik) && (son == "yok" || kanonik_tamsayi::<i64>(son))
        }
        [kimlik, islem, _, son] if islem == "baglan" => {
            kimlik == "0" && (son == "yok" || kanonik_tamsayi::<i64>(son))
        }
        [kimlik, islem, _]
            if matches!(islem.as_str(), "yaz" | "parca" | "aktar" | "tls_istemci") =>
        {
            kanonik_tamsayi::<u64>(kimlik)
        }
        [kimlik, islem, sayi] if matches!(islem.as_str(), "blok" | "bayt" | "oku_sure") => {
            kanonik_tamsayi::<u64>(kimlik) && kanonik_tamsayi::<i64>(sayi)
        }
        _ => false,
    };
    if gecerli {
        Ok(())
    } else {
        Err("`baglanti_islemi` argümanları geçersiz".into())
    }
}

pub(super) fn sonuc_yaz(sonuc: &BaglantiSonucu) -> Vec<String> {
    match sonuc {
        BaglantiSonucu::Satir(metin) => vec!["satir".into(), metin.clone()],
        BaglantiSonucu::Tamam(bayt) => vec!["tamam".into(), bayt.to_string()],
        BaglantiSonucu::Bekle => vec!["bekle".into()],
        BaglantiSonucu::Hata(tur, mesaj) => vec!["hata".into(), tur.clone(), mesaj.clone()],
        BaglantiSonucu::Blok { yol, sha256, bayt } => {
            vec!["blok".into(), yol.clone(), sha256.clone(), bayt.to_string()]
        }
        BaglantiSonucu::Giden(y) => vec![
            "giden".into(),
            y.kimlik.to_string(),
            y.uzak_adres.clone(),
            y.uzak_kapi.to_string(),
            y.yerel_kapi.to_string(),
        ],
        BaglantiSonucu::TlsIstemci(k) => vec![
            "tls_istemci".into(),
            bool_yaz(k.dogrulandi),
            k.dogrulama_hatasi.clone(),
            k.surum.clone(),
            k.takim.clone(),
            k.sunucu_adi.clone(),
            k.sertifika_sha256.clone(),
            k.spki_sha256.clone(),
            k.zincir_sha256.join(" "),
            k.zincir_spki_sha256.join(" "),
        ],
    }
}

fn ozet_listesi(alan: &str) -> Vec<String> {
    alan.split(' ')
        .filter(|parca| !parca.is_empty())
        .map(str::to_string)
        .collect()
}

pub(super) fn sonuc_coz(alanlar: &[String]) -> Result<BaglantiSonucu, String> {
    match alanlar {
        [etiket, metin] if etiket == "satir" => Ok(BaglantiSonucu::Satir(metin.clone())),
        [etiket, bayt] if etiket == "tamam" && kanonik_tamsayi::<i64>(bayt) => {
            Ok(BaglantiSonucu::Tamam(bayt.parse().unwrap_or(0)))
        }
        [etiket] if etiket == "bekle" => Ok(BaglantiSonucu::Bekle),
        [etiket, tur, mesaj] if etiket == "hata" => {
            Ok(BaglantiSonucu::Hata(tur.clone(), mesaj.clone()))
        }
        [etiket, yol, sha256, bayt]
            if etiket == "blok"
                && kanonik_tamsayi::<i64>(bayt)
                && sha256.len() == 64
                && sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) =>
        {
            Ok(BaglantiSonucu::Blok {
                yol: yol.clone(),
                sha256: sha256.clone(),
                bayt: bayt.parse().unwrap_or(0),
            })
        }
        [etiket, kimlik, adres, uzak, yerel]
            if etiket == "giden" && kanonik_tamsayi::<u64>(kimlik) && kimlik != "0" =>
        {
            let kapi = |alan: &str| alan.parse::<u16>().ok().filter(|n| n.to_string() == alan);
            match (kapi(uzak), kapi(yerel)) {
                (Some(uzak_kapi), Some(yerel_kapi)) => Ok(BaglantiSonucu::Giden(YeniBaglanti {
                    kimlik: kimlik.parse().unwrap_or(0),
                    dinleyici: String::new(),
                    uzak_adres: adres.clone(),
                    uzak_kapi,
                    yerel_kapi,
                    tls: false,
                })),
                _ => Err("giden bağlantı kapısı kanonik değil".into()),
            }
        }
        [etiket, dogrulandi, hata, surum, takim, ad, sertifika, spki, zincir, zincir_spki]
            if etiket == "tls_istemci" =>
        {
            Ok(BaglantiSonucu::TlsIstemci(Box::new(TlsIstemciKaydi {
                dogrulandi: bool_coz(dogrulandi)?,
                dogrulama_hatasi: hata.clone(),
                surum: surum.clone(),
                takim: takim.clone(),
                sunucu_adi: ad.clone(),
                sertifika_sha256: sertifika.clone(),
                spki_sha256: spki.clone(),
                zincir_sha256: ozet_listesi(zincir),
                zincir_spki_sha256: ozet_listesi(zincir_spki),
            })))
        }
        _ => Err("bağlantı işlem sonucu geçersiz".into()),
    }
}

pub(super) fn semayi_denetle(olay: &IzOlay) -> Result<(), String> {
    match olay.islem.as_str() {
        "baglanti_kaydet" => tamsayi_sonuc_semasi(olay, 1),
        "baglanti_bekle" => {
            let [bekleyenler, uyanma] = olay.argumanlar.as_slice() else {
                return Err("`baglanti_bekle` iki argüman bekler".into());
            };
            let bekleyenler_gecerli =
                bekleyenler.is_empty() || bekleyenler.split(',').all(kanonik_tamsayi::<u64>);
            if !bekleyenler_gecerli || !(uyanma == "yok" || kanonik_tamsayi::<i64>(uyanma)) {
                return Err("`baglanti_bekle` argümanları geçersiz".into());
            }
            olaylari_coz(&olay.sonuc).map(|_| ())
        }
        "baglanti_islemi" => {
            islem_semasi(&olay.argumanlar)?;
            sonuc_coz(&olay.sonuc).map(|_| ())
        }
        "baglanti_gunluk" => {
            alan_sayisi(olay, 2, 0)?;
            if kanonik_tamsayi::<u64>(&olay.argumanlar[0]) {
                Ok(())
            } else {
                Err("`baglanti_gunluk` kimliği kanonik sayı olmalı".into())
            }
        }
        _ => Err(format!("bilinmeyen işlem: {}", olay.islem)),
    }
}

pub(super) fn kaydet_oynat(io: &mut IzYenidenOynatici, ad: &str) -> Result<i64, String> {
    let sonuc = io.siradaki("baglanti_kaydet", vec![ad.into()]);
    io.tamsayi_sonuc("baglanti_kaydet", sonuc)
}

pub(super) fn bekle_oynat(
    io: &mut IzYenidenOynatici,
    bekleyenler: &[u64],
    uyanma: Option<i64>,
) -> Result<Vec<BaglantiOlayi>, String> {
    let Some(alanlar) = io.siradaki("baglanti_bekle", bekle_argumanlari(bekleyenler, uyanma))
    else {
        return Err(io
            .hata
            .clone()
            .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
    };
    olaylari_coz(&alanlar).unwrap_or_else(|hata| {
        io.bozuk_sonuc("baglanti_bekle", hata);
        Err(io
            .hata
            .clone()
            .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
    })
}

pub(super) fn islem_oynat(
    io: &mut IzYenidenOynatici,
    kimlik: u64,
    islem: &BaglantiIslemi,
) -> BaglantiSonucu {
    let Some(alanlar) = io.siradaki("baglanti_islemi", islem_argumanlari(kimlik, islem)) else {
        return BaglantiSonucu::hata(
            "kapandı",
            io.hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()),
        );
    };
    sonuc_coz(&alanlar).unwrap_or_else(|hata| {
        io.bozuk_sonuc("baglanti_islemi", hata);
        BaglantiSonucu::hata("kapandı", "IO izi sonucu bozuk")
    })
}

pub(super) fn gunluk_oynat(io: &mut IzYenidenOynatici, kimlik: u64, neden: &str) {
    let sonuc = io.siradaki("baglanti_gunluk", vec![kimlik.to_string(), neden.into()]);
    io.bos_sonuc("baglanti_gunluk", sonuc);
}
