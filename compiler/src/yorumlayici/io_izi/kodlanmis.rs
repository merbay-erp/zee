//! K-389 (RFC-0043 Dilim 1, ADR-167): aralıklı okuma ve çözerek yazma
//! olaylarının içeriksiz iz kodlaması. İz içerik değil uzunluk + SHA-256
//! taşır; bu yüzden başarılı aralıklı okuma yeniden oynatılamaz (fail-closed).
//!
//! - `dosya_araligi_oku [yol, konum, adet] → ok,<bayt>,<sha256> | hata,<mesaj>`
//! - `dosyaya_bayt_yaz [yol, yaz|ekle, <bayt>, <sha256>] → ok,<sha256>,<toplam> | hata,<mesaj>`
use super::*;

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
}
