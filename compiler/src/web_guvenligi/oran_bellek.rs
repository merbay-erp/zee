//! K-218: süreç başına, clone'lar arasında atomik ve boyutu sınırlı oran deposu.
//!
//! Canlı kayıt düşürülmez. Son kullanım indeksi yalnız ilk eklemede büyür;
//! her istekte bütün anahtarları taramak yerine yalnız dolan kayıtlar çıkarılır.

use super::{OranKaydi, RateLimitKarari};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub(super) struct OranBellek(Arc<Mutex<Durum>>);

#[derive(Default)]
struct Durum {
    kayitlar: BTreeMap<String, OranKaydi>,
    sonlar: BTreeSet<(i64, String)>,
}

impl OranBellek {
    pub(super) fn artir(
        &self,
        anahtar: String,
        an_ms: i64,
        pencere_ms: i64,
        sinir: u32,
        azami: usize,
    ) -> Result<RateLimitKarari, String> {
        let mut durum = self
            .0
            .lock()
            .map_err(|_| "bellek oran deposu kilidi zehirlendi")?;
        while durum.sonlar.first().is_some_and(|(son, _)| *son <= an_ms) {
            let Some((_, dolan)) = durum.sonlar.pop_first() else {
                return Err("bellek oran son-tarih indeksi tutarsız".into());
            };
            durum.kayitlar.remove(&dolan);
        }
        if !durum.kayitlar.contains_key(&anahtar) {
            if durum.kayitlar.len() >= azami {
                return Err(
                    "web oran sınırı anahtar kapasitesi dolu; yeni kimlik fail-closed reddedildi"
                        .into(),
                );
            }
            let son = an_ms.saturating_add(pencere_ms);
            durum.sonlar.insert((son, anahtar.clone()));
            durum.kayitlar.insert(
                anahtar.clone(),
                OranKaydi {
                    pencere_sonu_ms: son,
                    sayi: 0,
                },
            );
        }
        let kayit = durum
            .kayitlar
            .get_mut(&anahtar)
            .ok_or("bellek oran kayıt indeksi tutarsız")?;
        kayit.sayi = kayit.sayi.saturating_add(1);
        Ok(RateLimitKarari {
            izinli: kayit.sayi <= sinir,
            yeniden_dene_saniye: kayit
                .pencere_sonu_ms
                .saturating_sub(an_ms)
                .saturating_add(999)
                / 1_000,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bellek_oran_kapasitesi_canli_kotayi_dusurmez_ve_sonlar_sinirli_kalir() {
        let depo = OranBellek::default();
        for i in 0..32_768 {
            assert!(
                depo.artir(i.to_string(), 0, 60_000, 100, 32_768)
                    .unwrap()
                    .izinli
            );
        }
        assert!(depo.artir("fazla".into(), 1, 60_000, 100, 32_768).is_err());
        for _ in 0..100 {
            depo.artir("0".into(), 1, 60_000, 100, 32_768).unwrap();
        }
        assert!(
            !depo
                .artir("0".into(), 1, 60_000, 100, 32_768)
                .unwrap()
                .izinli
        );
        assert_eq!(depo.0.lock().unwrap().sonlar.len(), 32_768);
        assert!(
            depo.artir("fazla".into(), 60_000, 60_000, 100, 32_768)
                .unwrap()
                .izinli
        );
        let durum = depo.0.lock().unwrap();
        assert_eq!(durum.kayitlar.len(), 1);
        assert_eq!(durum.sonlar.len(), 1);
    }
}
