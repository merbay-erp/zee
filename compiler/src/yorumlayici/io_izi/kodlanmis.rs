//! K-389 (RFC-0043 Dilim 1, ADR-167): aralıklı okuma ve çözerek yazma
//! olaylarının içeriksiz iz kodlaması. İz içerik değil uzunluk + SHA-256
//! taşır; bu yüzden başarılı aralıklı okuma yeniden oynatılamaz (fail-closed).
//!
//! - `dosya_araligi_oku [yol, konum, adet] → ok,<bayt>,<sha256> | hata,<mesaj>`
//! - `dosyaya_bayt_yaz [yol, yaz|ekle, <bayt>, <sha256>] → ok,<sha256>,<toplam> | hata,<mesaj>`
//! - K-395 `dosyadan_base64_ekle [kaynak, konum, adet, hedef] → ok,<sha256>,<toplam>,<eklenen>
//!   | hata,<mesaj> | kodlama,<tür>,<konum>,<mesaj>` — içerik yok, tamamen oynatılabilir.
//! - K-395 `dosyada_bayt_ara [yol, konum, <iğne bayt>, <iğne sha256>] → ok,<konum> | hata,<mesaj>`
use super::*;
use crate::kalici_dosya::AkisliEkleme;
use crate::kodlama::{AkisHatasi, KodlamaHatasi};

pub(super) fn akis_argumanlari(kaynak: &str, konum: i64, adet: i64, hedef: &str) -> Vec<String> {
    vec![
        kaynak.into(),
        konum.to_string(),
        adet.to_string(),
        hedef.into(),
    ]
}

pub(super) fn arama_argumanlari(yol: &str, konum: i64, igne: &[u8]) -> Vec<String> {
    vec![
        yol.into(),
        konum.to_string(),
        igne.len().to_string(),
        sha256_hex(igne),
    ]
}

pub(super) fn akis_sonuc_yaz(sonuc: &Result<AkisliEkleme, AkisHatasi>) -> Vec<String> {
    match sonuc {
        Ok(ek) => vec![
            "ok".into(),
            ek.sha256.clone(),
            ek.toplam.to_string(),
            ek.eklenen.to_string(),
        ],
        Err(AkisHatasi::Dosya(hata)) => vec!["hata".into(), hata.clone()],
        Err(AkisHatasi::Kodlama(hata)) => vec![
            "kodlama".into(),
            hata.tur.into(),
            hata.konum.map_or_else(|| "-".into(), |k| k.to_string()),
            hata.mesaj.clone(),
        ],
    }
}

fn base64_hata_turu(tur: &str) -> Option<&'static str> {
    ["alfabe", "dolgu", "uzunluk"]
        .into_iter()
        .find(|bilinen| *bilinen == tur)
}

fn akis_sonuc_coz(alanlar: &[String]) -> Result<Result<AkisliEkleme, AkisHatasi>, String> {
    let gecersiz = || {
        "akış sonucu `ok,<sha256>,<toplam>,<eklenen>`, `hata,<mesaj>` veya `kodlama,<tür>,<konum>,<mesaj>` olmalı".to_string()
    };
    match alanlar {
        [etiket, ozet, toplam, eklenen] if etiket == "ok" && ozet_mi(ozet) => {
            let (toplam, eklenen) = (kanonik_sayi(toplam)?, kanonik_sayi(eklenen)?);
            if eklenen > toplam {
                return Err(gecersiz());
            }
            Ok(Ok(AkisliEkleme {
                sha256: ozet.clone(),
                toplam,
                eklenen,
            }))
        }
        [etiket, hata] if etiket == "hata" => Ok(Err(AkisHatasi::Dosya(hata.clone()))),
        [etiket, tur, konum, mesaj] if etiket == "kodlama" => {
            let tur = base64_hata_turu(tur).ok_or_else(gecersiz)?;
            let konum = match konum.as_str() {
                "-" => None,
                sayi => Some(kanonik_sayi(sayi)?),
            };
            Ok(Err(AkisHatasi::Kodlama(KodlamaHatasi {
                tur,
                konum,
                mesaj: mesaj.clone(),
            })))
        }
        _ => Err(gecersiz()),
    }
}

pub(super) fn aralik_argumanlari(yol: &str, konum: i64, adet: i64) -> Vec<String> {
    vec![yol.into(), konum.to_string(), adet.to_string()]
}

pub(super) fn aralik_sonuc_yaz(sonuc: &Result<Vec<u8>, String>) -> Vec<String> {
    match sonuc {
        Ok(baytlar) => vec!["ok".into(), baytlar.len().to_string(), sha256_hex(baytlar)],
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

pub(super) fn yazim_argumanlari(yol: &str, baytlar: &[u8], ekleme: bool) -> Vec<String> {
    vec![
        yol.into(),
        if ekleme { "ekle" } else { "yaz" }.into(),
        baytlar.len().to_string(),
        sha256_hex(baytlar),
    ]
}

pub(super) fn yazim_sonuc_yaz(sonuc: &Result<(String, u64), String>) -> Vec<String> {
    match sonuc {
        Ok((ozet, toplam)) => vec!["ok".into(), ozet.clone(), toplam.to_string()],
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

fn ozet_mi(alan: &str) -> bool {
    alan.len() == 64 && alan.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn kanonik_sayi(alan: &str) -> Result<u64, String> {
    alan.parse::<u64>()
        .ok()
        .filter(|n| n.to_string() == alan)
        .ok_or_else(|| "bayt sayısı kanonik sayı değil".to_string())
}

fn yazim_sonuc_coz(alanlar: &[String]) -> Result<Result<(String, u64), String>, String> {
    match alanlar {
        [etiket, ozet, toplam] if etiket == "ok" && ozet_mi(ozet) => {
            Ok(Ok((ozet.clone(), kanonik_sayi(toplam)?)))
        }
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        _ => Err("yazım sonucu `ok,<sha256>,<bayt>` veya `hata,<mesaj>` olmalı".into()),
    }
}

pub(super) fn semayi_denetle(olay: &IzOlay) -> Result<(), String> {
    let gecersiz = || format!("`{}` içeriksiz olay şeması geçersiz", olay.islem);
    match (olay.islem.as_str(), olay.argumanlar.as_slice()) {
        // K-333 imza okuması: aynı dosya-bayt olay ailesi.
        ("dosya_ilk_baytlari", _) => {
            metin_sonuc_semasi(olay, 2)?;
            olay.argumanlar[1]
                .parse::<i64>()
                .map(|_| ())
                .map_err(|_| "`dosya_ilk_baytlari` bayt sayısı tam sayı olmalı".into())
        }
        ("dosya_araligi_oku", [_, konum, adet]) => {
            konum.parse::<i64>().map_err(|_| gecersiz())?;
            adet.parse::<i64>().map_err(|_| gecersiz())?;
            match olay.sonuc.as_slice() {
                [etiket, bayt, ozet] if etiket == "ok" && ozet_mi(ozet) => {
                    kanonik_sayi(bayt).map(|_| ())
                }
                [etiket, _] if etiket == "hata" => Ok(()),
                _ => Err(gecersiz()),
            }
        }
        ("dosyadan_base64_ekle", [_, konum, adet, _]) => {
            konum.parse::<i64>().map_err(|_| gecersiz())?;
            adet.parse::<i64>().map_err(|_| gecersiz())?;
            akis_sonuc_coz(&olay.sonuc).map(|_| ())
        }
        ("dosyada_bayt_ara", [_, konum, bayt, ozet]) if ozet_mi(ozet) => {
            konum.parse::<i64>().map_err(|_| gecersiz())?;
            kanonik_sayi(bayt)?;
            tamsayi_sonuc_coz(&olay.sonuc)
                .map(|_| ())
                .map_err(|_| gecersiz())
        }
        ("dosyaya_bayt_yaz", [_, kip, bayt, ozet])
            if matches!(kip.as_str(), "yaz" | "ekle") && ozet_mi(ozet) =>
        {
            kanonik_sayi(bayt)?;
            yazim_sonuc_coz(&olay.sonuc).map(|_| ())
        }
        _ => Err(gecersiz()),
    }
}

impl IzYenidenOynatici {
    pub(super) fn aralik_oynat(
        &mut self,
        yol: &str,
        konum: i64,
        adet: i64,
    ) -> Result<Vec<u8>, String> {
        let islem = "dosya_araligi_oku";
        let Some(alanlar) = self.siradaki(islem, aralik_argumanlari(yol, konum, adet)) else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        match alanlar.as_slice() {
            [etiket, hata] if etiket == "hata" => Err(hata.clone()),
            _ => {
                self.hata_yaz(format!(
                    "IO izi `{islem}` içerik taşımaz; başarılı aralıklı okuma yeniden oynatılamaz (ADR-167)"
                ));
                Err(self.hata.clone().unwrap_or_default())
            }
        }
    }

    pub(super) fn yazim_oynat(
        &mut self,
        yol: &str,
        baytlar: &[u8],
        ekleme: bool,
    ) -> Result<(String, u64), String> {
        let islem = "dosyaya_bayt_yaz";
        let Some(alanlar) = self.siradaki(islem, yazim_argumanlari(yol, baytlar, ekleme)) else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        yazim_sonuc_coz(&alanlar).unwrap_or_else(|hata| {
            self.bozuk_sonuc(islem, hata);
            Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
        })
    }

    pub(super) fn akis_oynat(
        &mut self,
        kaynak: &str,
        konum: i64,
        adet: i64,
        hedef: &str,
    ) -> Result<AkisliEkleme, AkisHatasi> {
        let islem = "dosyadan_base64_ekle";
        let kayip = |oynatici: &Self, varsayilan: &str| {
            AkisHatasi::Dosya(oynatici.hata.clone().unwrap_or_else(|| varsayilan.into()))
        };
        let Some(alanlar) = self.siradaki(islem, akis_argumanlari(kaynak, konum, adet, hedef))
        else {
            return Err(kayip(self, "IO izi uyuşmazlığı"));
        };
        akis_sonuc_coz(&alanlar).unwrap_or_else(|hata| {
            self.bozuk_sonuc(islem, hata);
            Err(kayip(self, "IO izi sonucu bozuk"))
        })
    }
}
