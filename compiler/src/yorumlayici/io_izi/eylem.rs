use super::{IzOlay, IzYenidenOynatici};
use crate::yorumlayici::{EylemHataSinifi, EylemHatasi};

pub(super) fn sonuc_yaz(sonuc: &Result<(), EylemHatasi>) -> Vec<String> {
    match sonuc {
        Ok(()) => vec!["ok".into()],
        Err(hata) => vec![
            "hata".into(),
            match hata.sinif {
                EylemHataSinifi::Genel => "genel",
                EylemHataSinifi::CommitSonucuBelirsiz => "commit_sonucu_belirsiz",
            }
            .into(),
            hata.mesaj.clone(),
        ],
    }
}

fn sonuc_coz(alanlar: &[String]) -> Result<Result<(), EylemHatasi>, String> {
    match alanlar {
        [etiket] if etiket == "ok" => Ok(Ok(())),
        // Eski iki alanlı v1 genel hata kayıtları geriye dönük okunur.
        [etiket, mesaj] if etiket == "hata" => Ok(Err(EylemHatasi::genel(mesaj.clone()))),
        [etiket, sinif, mesaj] if etiket == "hata" => {
            let hata = match sinif.as_str() {
                "genel" => EylemHatasi::genel(mesaj.clone()),
                "commit_sonucu_belirsiz" => EylemHatasi::commit_sonucu_belirsiz(mesaj.clone()),
                _ => return Err("bilinmeyen eylem hata sınıfı".into()),
            };
            Ok(Err(hata))
        }
        _ => Err("eylem sonucu `ok`, `hata,<mesaj>` veya `hata,<sınıf>,<mesaj>` olmalı".into()),
    }
}

pub(super) fn semayi_denetle(olay: &IzOlay) -> Result<(), String> {
    if !olay.argumanlar.is_empty() || sonuc_coz(&olay.sonuc).is_err() {
        return Err(format!("`{}` eylem sonuç şeması geçersiz", olay.islem));
    }
    Ok(())
}

impl IzYenidenOynatici {
    pub(super) fn eylem_sonuc(
        &mut self,
        islem: &str,
        sonuc: Option<Vec<String>>,
    ) -> Result<(), EylemHatasi> {
        let Some(alanlar) = sonuc else {
            return Err(EylemHatasi::genel(
                self.hata
                    .clone()
                    .unwrap_or_else(|| "IO izi uyuşmazlığı".into()),
            ));
        };
        match sonuc_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc(islem, hata);
                Err(EylemHatasi::genel(
                    self.hata
                        .clone()
                        .unwrap_or_else(|| "IO izi sonucu bozuk".into()),
                ))
            }
        }
    }
}
