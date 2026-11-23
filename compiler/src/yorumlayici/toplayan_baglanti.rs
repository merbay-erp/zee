//! K-381: hermetik bağlantı adaptörü. Betiklenmiş istemciler gerçek
//! reaktörle aynı satır okuyucusu ve sınırlarla sunulur; her `parça` ayrı
//! bir ağ teslimidir, böylece eşzamanlı oturumların sırası deterministiktir.

use super::{BaglantiIslemi, BaglantiOlayi, BaglantiSonucu, YeniBaglanti};
use crate::ag_satiri::{giden_satir, BaglantiAyari, SatirOkuyucu};
use std::collections::VecDeque;

/// Bir betiklenmiş istemci bağlantısı.
#[derive(Debug, Clone)]
pub struct SahteBaglanti {
    pub dinleyici: String,
    pub uzak_adres: String,
    /// Sırayla teslim edilen ham bayt parçaları.
    pub parcalar: VecDeque<Vec<u8>>,
    /// Parçalar bitince karşı taraf kapatmaz, susar (okuma zaman aşımına düşer).
    pub sessiz_kalir: bool,
    /// Sunucunun yazdığı satırlar (CRLF'siz).
    pub giden: Vec<String>,
    pub kapandi: bool,
    /// Implicit TLS dinleyicisinden gelir ya da STARTTLS ile yükseltildi.
    pub tls: bool,
    okuyucu: Option<SatirOkuyucu>,
    sustu: bool,
}

impl SahteBaglanti {
    pub fn yeni(dinleyici: &str, uzak_adres: &str, parcalar: &[&[u8]]) -> Self {
        Self {
            dinleyici: dinleyici.into(),
            uzak_adres: uzak_adres.into(),
            parcalar: parcalar.iter().map(|p| p.to_vec()).collect(),
            sessiz_kalir: false,
            giden: Vec::new(),
            kapandi: false,
            tls: false,
            okuyucu: None,
            sustu: false,
        }
    }

    /// Her satırı CRLF ile tek parça olarak gönderen istemci.
    pub fn satirlarla(dinleyici: &str, uzak_adres: &str, satirlar: &[&str]) -> Self {
        let parcalar: Vec<Vec<u8>> = satirlar
            .iter()
            .map(|satir| format!("{satir}\r\n").into_bytes())
            .collect();
        let parcalar: Vec<&[u8]> = parcalar.iter().map(Vec::as_slice).collect();
        Self::yeni(dinleyici, uzak_adres, &parcalar)
    }

    fn teslim_et(&mut self) -> bool {
        match (self.parcalar.pop_front(), self.okuyucu.as_mut()) {
            (Some(parca), Some(okuyucu)) => {
                okuyucu.besle(&parca);
                true
            }
            _ => false,
        }
    }
}

/// Hermetik bağlantı durumu.
#[derive(Debug, Clone, Default)]
pub struct ToplayanBaglanti {
    pub ayar: BaglantiAyari,
    /// `None`: her dinleyici adı kabul edilir (kütüphane/test kullanımı).
    pub dinleyici_adlari: Option<Vec<String>>,
    pub bekleyenler: VecDeque<SahteBaglanti>,
    /// Kabul edilen bağlantılar; kimlik = sıra + 1.
    pub baglantilar: Vec<SahteBaglanti>,
    /// Sınır dolunca kibar retle kapatılanlar: (uzak adres, yazılan satır).
    pub reddedilenler: Vec<(String, Option<String>)>,
    pub gunluk: Vec<(u64, String)>,
    kapanis_bildirildi: bool,
}

impl ToplayanBaglanti {
    pub(super) fn kaydet(&self, ad: &str) -> Result<i64, String> {
        match &self.dinleyici_adlari {
            Some(adlar) if !adlar.iter().any(|a| a == ad) => {
                Err("proje.dil `dinleyiciler` bildiriminde yok".into())
            }
            _ => Ok(self.ayar.omur_ms),
        }
    }

    fn acik_sayisi(&self, uzak: Option<&str>) -> usize {
        self.baglantilar
            .iter()
            .filter(|b| !b.kapandi && uzak.is_none_or(|u| b.uzak_adres == u))
            .count()
    }

    /// Dönen ikinci değer: sanal saatin ilerletileceği mutlak an; üçüncü:
    /// susan istemci için eklenecek süre.
    pub(super) fn bekle(
        &mut self,
        bekleyenler: &[u64],
        uyanma: Option<i64>,
    ) -> (Vec<BaglantiOlayi>, Option<i64>, i64) {
        let mut olaylar = Vec::new();
        if !self.kapanis_bildirildi {
            while let Some(mut yeni) = self.bekleyenler.pop_front() {
                if self.acik_sayisi(None) >= self.ayar.sinir
                    || self.acik_sayisi(Some(&yeni.uzak_adres)) >= self.ayar.uzak_adres_siniri
                {
                    self.reddedilenler
                        .push((yeni.uzak_adres, self.ayar.dolu_yaniti.clone()));
                    continue;
                }
                yeni.okuyucu = Some(SatirOkuyucu::yeni(self.ayar.satir_siniri));
                let kimlik = self.baglantilar.len() as u64 + 1;
                olaylar.push(BaglantiOlayi::Yeni(YeniBaglanti {
                    kimlik,
                    dinleyici: yeni.dinleyici.clone(),
                    uzak_adres: yeni.uzak_adres.clone(),
                    uzak_kapi: 40000 + kimlik as u16,
                    yerel_kapi: 2525,
                    tls: yeni.tls,
                }));
                self.baglantilar.push(yeni);
            }
        }
        let mut ekle = 0;
        for kimlik in bekleyenler {
            let Some(b) = self.baglantilar.get_mut((*kimlik as usize).wrapping_sub(1)) else {
                continue;
            };
            if !b.teslim_et() && b.sessiz_kalir {
                // Susan istemci: okuma boşta süresinin sonunda zaman aşımına düşer.
                b.sustu = true;
                ekle = self.ayar.bosta_suresi_ms;
            }
            olaylar.push(BaglantiOlayi::Hazir(*kimlik));
        }
        if olaylar.is_empty() {
            if let Some(an) = uyanma {
                return (olaylar, Some(an), 0);
            }
            if !self.kapanis_bildirildi {
                self.kapanis_bildirildi = true;
                olaylar.push(BaglantiOlayi::Kapanis);
            }
        }
        (olaylar, None, ekle)
    }

    pub(super) fn islem(&mut self, kimlik: u64, islem: &BaglantiIslemi) -> BaglantiSonucu {
        let satir_siniri = self.ayar.satir_siniri;
        let kapaniyor = self.kapanis_bildirildi;
        let tls_var = self.ayar.tls.is_some();
        let Some(b) = self.baglantilar.get_mut((kimlik as usize).wrapping_sub(1)) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı yok");
        };
        match islem {
            BaglantiIslemi::SatirOku => {
                if b.kapandi {
                    return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
                }
                if kapaniyor {
                    return BaglantiSonucu::hata("kapanıyor", "sunucu kapanıyor");
                }
                let Some(okuyucu) = b.okuyucu.as_mut() else {
                    return BaglantiSonucu::hata("kapandı", "bağlantı yok");
                };
                match okuyucu.siradaki() {
                    Some(Ok(satir)) => BaglantiSonucu::Satir(satir),
                    Some(Err(hata)) => BaglantiSonucu::hata(hata.turu(), hata.mesaji()),
                    None if b.sustu => {
                        b.sustu = false;
                        BaglantiSonucu::hata("zaman_aşımı", "satır boşta süresi içinde gelmedi")
                    }
                    None if b.parcalar.is_empty() && b.sessiz_kalir => BaglantiSonucu::Bekle,
                    None if b.parcalar.is_empty() => {
                        BaglantiSonucu::hata("kapandı", "karşı taraf bağlantıyı kapattı")
                    }
                    None => BaglantiSonucu::Bekle,
                }
            }
            BaglantiIslemi::SatirYaz(metin) => {
                if b.kapandi {
                    return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
                }
                match giden_satir(metin, satir_siniri) {
                    Ok(bayt) => {
                        b.giden.push(metin.clone());
                        BaglantiSonucu::Tamam(bayt.len() as i64)
                    }
                    Err(neden) => BaglantiSonucu::hata("yazılamadı", neden),
                }
            }
            BaglantiIslemi::Kapat => {
                b.kapandi = true;
                BaglantiSonucu::Tamam(0)
            }
            BaglantiIslemi::TlsBaslat => {
                if b.tls {
                    return BaglantiSonucu::hata("tls", crate::ag_satiri::ZATEN_TLS);
                }
                if !tls_var {
                    return BaglantiSonucu::hata("tls", crate::ag_satiri::TLS_KIMLIGI_YOK);
                }
                let okuyucu = b.okuyucu.as_mut();
                if okuyucu.as_ref().is_some_and(|o| o.tamponlu_bayt() > 0) {
                    b.kapandi = true;
                    return BaglantiSonucu::hata("tls", crate::ag_satiri::STARTTLS_ENJEKSIYONU);
                }
                okuyucu.into_iter().for_each(SatirOkuyucu::sifirla);
                b.tls = true;
                BaglantiSonucu::Satir("TLSv1_3 TLS13_AES_128_GCM_SHA256".into())
            }
            BaglantiIslemi::Engelle(_) => {
                b.teslim_et();
                BaglantiSonucu::Tamam(0)
            }
        }
    }
}
