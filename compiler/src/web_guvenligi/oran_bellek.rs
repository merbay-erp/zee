//! Süreç başına atomik ve sınırlı oran deposu (K-218/K-219).
//! En eski uç nokta penceresi yalnız explicit seçimde tahliye edilir;
//! giriş/CSRF kayıtları dolana kadar korunur.

use super::{OranKaydi, RateLimitKarari, RateLimitTuru};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OranDoluluk {
    #[default]
    Reddet,
    EnEski,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellekOranAyari {
    ucnokta_siniri: u32,
    kapasite: usize,
    doluluk: OranDoluluk,
}

impl Default for BellekOranAyari {
    fn default() -> Self {
        let web = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.web();
        Self {
            ucnokta_siniri: web.ucnokta_orani(),
            kapasite: web.oran_anahtari_sayisi(),
            doluluk: OranDoluluk::Reddet,
        }
    }
}

impl BellekOranAyari {
    pub fn yeni(
        ucnokta_siniri: u32,
        kapasite: usize,
        doluluk: OranDoluluk,
    ) -> Result<Self, String> {
        if !(1..=1_000_000).contains(&ucnokta_siniri) {
            return Err("--web-oran-siniri 1..1000000 arasında olmalı".into());
        }
        let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .web()
            .oran_anahtari_sayisi();
        if !(1..=azami).contains(&kapasite) {
            return Err(format!("--web-oran-kapasitesi 1..{azami} arasında olmalı"));
        }
        Ok(Self {
            ucnokta_siniri,
            kapasite,
            doluluk,
        })
    }

    pub const fn doluluk(self) -> OranDoluluk {
        self.doluluk
    }
}

#[derive(Clone, Default)]
pub(super) struct OranBellek {
    durum: Arc<Mutex<Durum>>,
    ayar: BellekOranAyari,
}

#[derive(Default)]
struct Durum {
    kayitlar: BTreeMap<String, OranKaydi>,
    sonlar: BTreeSet<(i64, String)>,
    tahliyeler: BTreeSet<(i64, String)>,
}

impl Durum {
    fn sil(&mut self, anahtar: &str, son: i64) {
        self.kayitlar.remove(anahtar);
        self.sonlar.remove(&(son, anahtar.to_owned()));
        self.tahliyeler.remove(&(son, anahtar.to_owned()));
    }
}

impl OranBellek {
    pub(super) fn yeni(ayar: BellekOranAyari) -> Self {
        Self {
            ayar,
            ..Self::default()
        }
    }

    pub(super) fn artir(
        &self,
        anahtar: String,
        an_ms: i64,
        pencere_ms: i64,
        sinir: u32,
        tur: RateLimitTuru,
    ) -> Result<RateLimitKarari, String> {
        let mut durum = self
            .durum
            .lock()
            .map_err(|_| "bellek oran deposu kilidi zehirlendi")?;
        while let Some((son, dolan)) = durum
            .sonlar
            .first()
            .filter(|(son, _)| *son <= an_ms)
            .cloned()
        {
            durum.sil(&dolan, son);
        }
        if !durum.kayitlar.contains_key(&anahtar) {
            if durum.kayitlar.len() >= self.ayar.kapasite {
                let aday = if self.ayar.doluluk == OranDoluluk::EnEski {
                    durum.tahliyeler.first().cloned()
                } else {
                    None
                };
                if let Some((son, eski)) = aday {
                    durum.sil(&eski, son);
                } else {
                    return Err("web oran sınırı anahtar kapasitesi dolu; korunmuş yeni kimlik fail-closed reddedildi".into());
                }
            }
            let son = an_ms.saturating_add(pencere_ms);
            durum.sonlar.insert((son, anahtar.clone()));
            if tur == RateLimitTuru::Ucnokta && self.ayar.doluluk == OranDoluluk::EnEski {
                durum.tahliyeler.insert((son, anahtar.clone()));
            }
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
        let sinir = if tur == RateLimitTuru::Ucnokta {
            self.ayar.ucnokta_siniri
        } else {
            sinir
        };
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
                depo.artir(i.to_string(), 0, 60_000, 100, RateLimitTuru::Ucnokta)
                    .unwrap()
                    .izinli
            );
        }
        assert!(depo
            .artir("fazla".into(), 1, 60_000, 100, RateLimitTuru::Ucnokta)
            .is_err());
        for _ in 0..100 {
            depo.artir("0".into(), 1, 60_000, 100, RateLimitTuru::Ucnokta)
                .unwrap();
        }
        assert!(
            !depo
                .artir("0".into(), 1, 60_000, 100, RateLimitTuru::Ucnokta)
                .unwrap()
                .izinli
        );
        assert_eq!(depo.durum.lock().unwrap().sonlar.len(), 32_768);
        assert!(
            depo.artir("fazla".into(), 60_000, 60_000, 100, RateLimitTuru::Ucnokta)
                .unwrap()
                .izinli
        );
        let durum = depo.durum.lock().unwrap();
        assert_eq!(durum.kayitlar.len(), 1);
        assert_eq!(durum.sonlar.len(), 1);
        assert_eq!(durum.tahliyeler.len(), 0);
    }

    #[test]
    fn ayarli_clone_yarisi_kotayi_deldirmez() {
        let depo = OranBellek::yeni(BellekOranAyari::yeni(7, 2, OranDoluluk::EnEski).unwrap());
        let isler = (0..8)
            .map(|_| {
                let clone = depo.clone();
                std::thread::spawn(move || {
                    (0..10)
                        .filter(|_| {
                            clone
                                .artir("aynı".into(), 0, 60_000, 100, RateLimitTuru::Ucnokta)
                                .unwrap()
                                .izinli
                        })
                        .count()
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            isler
                .into_iter()
                .map(|is| is.join().unwrap())
                .sum::<usize>(),
            7
        );
        let durum = depo.durum.lock().unwrap();
        assert_eq!(durum.kayitlar.len(), 1);
        assert_eq!(durum.sonlar.len(), 1);
        assert_eq!(durum.tahliyeler.len(), 1);
    }

    #[test]
    fn oran_ayari_sifir_tasma_ve_sinirsiz_kapasiteyi_reddeder() {
        for (sinir, kapasite) in [(0, 1), (1_000_001, 1), (1, 0), (1, 32_769)] {
            assert!(BellekOranAyari::yeni(sinir, kapasite, OranDoluluk::EnEski).is_err());
        }
        assert!(BellekOranAyari::yeni(1, 1, OranDoluluk::EnEski).is_ok());
        assert!(BellekOranAyari::yeni(1_000_000, 32_768, OranDoluluk::Reddet).is_ok());
    }

    #[test]
    fn en_eski_pencere_tahliye_edilir_erisim_omru_uzatmaz() {
        let depo = OranBellek::yeni(BellekOranAyari::yeni(2, 2, OranDoluluk::EnEski).unwrap());
        let artir = |key: &str, an| {
            depo.artir(key.into(), an, 60_000, 100, RateLimitTuru::Ucnokta)
                .unwrap()
        };
        assert!(artir("a", 0).izinli);
        assert!(artir("b", 1).izinli);
        assert!(artir("a", 2).izinli);
        let red = artir("a", 3);
        assert!(!red.izinli);
        assert_eq!(red.yeniden_dene_saniye, 60);
        assert!(artir("c", 4).izinli);
        assert!(!depo.durum.lock().unwrap().kayitlar.contains_key("a"));
        assert!(artir("b", 5).izinli);
        assert!(!artir("b", 6).izinli);
        // Explicit tahliyenin sınırı: atılan uç nokta yeniden gelince yeni kota alır.
        assert!(artir("a", 7).izinli);
        let durum = depo.durum.lock().unwrap();
        assert_eq!(durum.kayitlar.len(), 2);
        assert_eq!(durum.sonlar.len(), 2);
        assert_eq!(durum.tahliyeler.len(), 2);
    }

    #[test]
    fn anahtar_churn_giris_ve_csrf_kotasini_sifirlayamaz() {
        let depo = OranBellek::yeni(BellekOranAyari::yeni(999, 3, OranDoluluk::EnEski).unwrap());
        for (key, tur, sinir) in [
            ("g", RateLimitTuru::Giris, 5),
            ("c", RateLimitTuru::Csrf, 60),
        ] {
            for _ in 0..sinir {
                assert!(
                    depo.artir(key.into(), 0, 300_000, sinir, tur)
                        .unwrap()
                        .izinli
                );
            }
            assert!(
                !depo
                    .artir(key.into(), 0, 300_000, sinir, tur)
                    .unwrap()
                    .izinli
            );
        }
        for i in 0..10_000 {
            assert!(
                depo.artir(format!("u/{i}"), 1, 60_000, 100, RateLimitTuru::Ucnokta)
                    .unwrap()
                    .izinli
            );
        }
        for (key, tur, sinir) in [
            ("g", RateLimitTuru::Giris, 5),
            ("c", RateLimitTuru::Csrf, 60),
        ] {
            assert!(
                !depo
                    .artir(key.into(), 1, 300_000, sinir, tur)
                    .unwrap()
                    .izinli
            );
        }
        assert!(
            depo.artir("yeni-g".into(), 1, 300_000, 5, RateLimitTuru::Giris)
                .unwrap()
                .izinli
        );
        // Tüm kayıtlar korunmuşsa yeni kimlik yine reddedilir; sınırsız büyüme yok.
        assert!(depo
            .artir("fazla".into(), 1, 60_000, 100, RateLimitTuru::Ucnokta)
            .is_err());
        assert!(depo
            .artir("fazla-g".into(), 1, 300_000, 5, RateLimitTuru::Giris)
            .is_err());
        let durum = depo.durum.lock().unwrap();
        assert_eq!(durum.kayitlar.len(), 3);
        assert_eq!(durum.sonlar.len(), 3);
        assert!(durum.tahliyeler.is_empty());
        drop(durum);
        assert!(
            depo.artir("fazla".into(), 300_000, 60_000, 100, RateLimitTuru::Ucnokta)
                .unwrap()
                .izinli
        );
    }
}
