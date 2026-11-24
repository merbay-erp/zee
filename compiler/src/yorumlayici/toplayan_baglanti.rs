//! K-381: hermetik bağlantı adaptörü. Betiklenmiş istemciler gerçek
//! reaktörle aynı satır okuyucusu ve sınırlarla sunulur; her `parça` ayrı
//! bir ağ teslimidir, böylece eşzamanlı oturumların sırası deterministiktir.
//!
//! K-384: blok okuma gerçek reaktörle aynı `BlokOkuyucu`yu kullanır; dosya
//! yerine bellekte tutulur (`bloklar`, `dosyalar`), yol deterministiktir.

use super::{BaglantiIslemi, BaglantiOlayi, BaglantiSonucu, YeniBaglanti};
use crate::ag_satiri::{
    akis_butcesi_ms, bayt_metni, giden_parca, giden_satir, BaglantiAyari, BlokOkuyucu, SatirOkuyucu,
};
use std::collections::{HashMap, VecDeque};

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
    /// K-384: sunucunun yazdığı bütün baytlar (satır CRLF'si, parça, aktarım).
    pub ham_giden: Vec<u8>,
    /// K-384: tamamlanan blokların çözülmüş içeriği.
    pub bloklar: Vec<Vec<u8>>,
    okuyucu: Option<SatirOkuyucu>,
    sustu: bool,
    blok: Option<(BlokOkuyucu, Vec<u8>)>,
    /// Yarıda kalan blok/literal okuma yönünü kapatır.
    okuma_kapali: bool,
    /// Bekleyen işlemin süresi (susan istemcinin sanal saat artışı).
    bekleme_ms: Option<i64>,
}

/// K-396: hermetik taraf aynı saf pencereyle (`kodlama`) üretir.
fn base64_penceresi(icerik: &[u8], aralik: Option<(u64, u64)>) -> Vec<u8> {
    let (konum, adet) = aralik.unwrap_or((0, u64::MAX));
    let p = crate::kodlama::base64_satir_penceresi(icerik.len() as u64, konum, adet);
    let mut cikti = Vec::new();
    crate::kodlama::base64_satirlari_kodla(&icerik[p.ham_baslangic as usize..], &mut cikti);
    cikti.drain(..p.atla as usize);
    cikti.truncate(p.uzunluk as usize);
    cikti
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
            ham_giden: Vec::new(),
            bloklar: Vec::new(),
            okuyucu: None,
            sustu: false,
            blok: None,
            okuma_kapali: false,
            bekleme_ms: None,
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
    /// K-384: bağlantı tarafının sanal dosyaları (alınan bloklar ve
    /// aktarılacak dosyalar; yol → bayt).
    pub dosyalar: HashMap<String, Vec<u8>>,
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
                ekle = b.bekleme_ms.unwrap_or(self.ayar.bosta_suresi_ms);
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
        let ayar = self.ayar.clone();
        let kapaniyor = self.kapanis_bildirildi;
        let blok_sirasi = self.dosyalar.len();
        let Some(b) = self.baglantilar.get_mut((kimlik as usize).wrapping_sub(1)) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı yok");
        };
        let okuma = matches!(
            islem,
            BaglantiIslemi::SatirOku
                | BaglantiIslemi::SatirOkuSureli(_)
                | BaglantiIslemi::BaytOku(_)
                | BaglantiIslemi::NoktaBlogu
                | BaglantiIslemi::BaytBlogu(_)
        );
        let kapatma = matches!(islem, BaglantiIslemi::Kapat | BaglantiIslemi::Engelle(_));
        if (b.kapandi && !kapatma) || (okuma && b.okuma_kapali) {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        }
        if okuma && kapaniyor && b.blok.is_none() {
            return BaglantiSonucu::hata("kapanıyor", "sunucu kapanıyor");
        }
        match islem {
            BaglantiIslemi::SatirOku => b.satir_oku(None, ayar.bosta_suresi_ms),
            BaglantiIslemi::SatirOkuSureli(ms) => b.satir_oku(Some(*ms), ayar.bosta_suresi_ms),
            BaglantiIslemi::BaytOku(adet) => b.bayt_oku(*adet, &ayar),
            BaglantiIslemi::NoktaBlogu | BaglantiIslemi::BaytBlogu(_) => {
                let sonuc = b.blok_al(islem, &ayar, kapaniyor, blok_sirasi);
                if let BaglantiSonucu::Blok { yol, .. } = &sonuc {
                    let icerik = b.bloklar.last().cloned().unwrap_or_default();
                    self.dosyalar.insert(yol.clone(), icerik);
                }
                sonuc
            }
            BaglantiIslemi::SatirYaz(metin) => {
                b.yaz(metin, giden_satir(metin, ayar.satir_siniri), true)
            }
            BaglantiIslemi::ParcaYaz(metin) => {
                b.yaz(metin, giden_parca(metin, ayar.satir_siniri), false)
            }
            BaglantiIslemi::DosyaAktar(yol) => match self.dosyalar.get(yol) {
                None => BaglantiSonucu::hata("dosya", format!("\"{yol}\" dosyası yok")),
                Some(icerik) if icerik.len() as u64 > ayar.blok_siniri => {
                    BaglantiSonucu::hata("boyut_aşımı", crate::ag_satiri::BLOK_SINIRI_ASILDI)
                }
                Some(icerik) => {
                    b.ham_giden.extend_from_slice(icerik);
                    BaglantiSonucu::Tamam(icerik.len() as i64)
                }
            },
            BaglantiIslemi::Base64Aktar { yol, aralik } => match self.dosyalar.get(yol) {
                None => BaglantiSonucu::hata("dosya", format!("\"{yol}\" dosyası yok")),
                Some(icerik) if icerik.len() as u64 > ayar.blok_siniri => {
                    BaglantiSonucu::hata("boyut_aşımı", crate::ag_satiri::BLOK_SINIRI_ASILDI)
                }
                Some(icerik) => {
                    let parca = base64_penceresi(icerik, *aralik);
                    b.ham_giden.extend_from_slice(&parca);
                    BaglantiSonucu::Tamam(parca.len() as i64)
                }
            },
            BaglantiIslemi::Kapat => {
                b.kapandi = true;
                b.blok = None;
                BaglantiSonucu::Tamam(0)
            }
            BaglantiIslemi::TlsBaslat => b.tls_baslat(ayar.tls.is_some()),
            BaglantiIslemi::Engelle(_) => {
                b.teslim_et();
                BaglantiSonucu::Tamam(0)
            }
        }
    }
}

impl SahteBaglanti {
    /// Veri yokken: susan istemci zaman aşımı, kapanmış istemci `kapandı`.
    fn veri_yok(&mut self, tur_mesaji: &str, okuma_kapanir: bool) -> BaglantiSonucu {
        if self.sustu {
            // Satır okumasında zaman aşımı bağlantıyı kapatmaz (K-384, ZM-F032).
            self.sustu = false;
            self.okuma_kapali |= okuma_kapanir;
            return BaglantiSonucu::hata("zaman_aşımı", tur_mesaji);
        }
        if self.parcalar.is_empty() && !self.sessiz_kalir {
            self.okuma_kapali |= okuma_kapanir;
            return BaglantiSonucu::hata("kapandı", "karşı taraf bağlantıyı kapattı");
        }
        BaglantiSonucu::Bekle
    }

    fn satir_oku(&mut self, sure_ms: Option<i64>, bosta: i64) -> BaglantiSonucu {
        self.bekleme_ms = sure_ms.map(|ms| ms.min(bosta));
        let Some(okuyucu) = self.okuyucu.as_mut() else {
            return BaglantiSonucu::hata("kapandı", "bağlantı yok");
        };
        match okuyucu.siradaki() {
            Some(Ok(satir)) => BaglantiSonucu::Satir(satir),
            Some(Err(hata)) => BaglantiSonucu::hata(hata.turu(), hata.mesaji()),
            None => self.veri_yok("satır süresi içinde gelmedi", false),
        }
    }

    fn bayt_oku(&mut self, adet: i64, ayar: &BaglantiAyari) -> BaglantiSonucu {
        if adet as u64 > ayar.satir_siniri as u64 {
            return BaglantiSonucu::hata("boyut_aşımı", crate::ag_satiri::BAYT_OKUMA_SINIRI);
        }
        self.bekleme_ms = Some(akis_butcesi_ms(ayar.bosta_suresi_ms, adet as u64));
        let Some(okuyucu) = self.okuyucu.as_mut() else {
            return BaglantiSonucu::hata("kapandı", "bağlantı yok");
        };
        match okuyucu.bayt_al(adet as usize) {
            Some(bayt) => match bayt_metni(bayt) {
                Ok(metin) => BaglantiSonucu::Satir(metin),
                Err(hata) => BaglantiSonucu::hata(hata.turu(), hata.mesaji()),
            },
            None => self.veri_yok("baytlar süresi içinde gelmedi", true),
        }
    }

    fn blok_al(
        &mut self,
        islem: &BaglantiIslemi,
        ayar: &BaglantiAyari,
        kapaniyor: bool,
        sira: usize,
    ) -> BaglantiSonucu {
        if self.blok.is_none() {
            let (okuyucu, taban) = match islem {
                BaglantiIslemi::BaytBlogu(adet) if *adet as u64 > ayar.blok_siniri => {
                    return BaglantiSonucu::hata(
                        "boyut_aşımı",
                        crate::ag_satiri::BLOK_SINIRI_ASILDI,
                    );
                }
                BaglantiIslemi::BaytBlogu(adet) => (BlokOkuyucu::bayt(*adet as u64), *adet as u64),
                _ => (BlokOkuyucu::nokta(ayar.blok_siniri), ayar.blok_siniri),
            };
            self.bekleme_ms = Some(akis_butcesi_ms(ayar.bosta_suresi_ms, taban));
            self.blok = Some((okuyucu, Vec::new()));
        }
        let (Some((blok, icerik)), Some(okuyucu)) = (self.blok.as_mut(), self.okuyucu.as_mut())
        else {
            return BaglantiSonucu::hata("kapandı", "bağlantı yok");
        };
        let tuketilen = blok.besle(okuyucu.tampon(), icerik);
        okuyucu.tuket_bayt(tuketilen);
        if blok.bitti() {
            let hata = blok.hata();
            let icerik = std::mem::take(icerik);
            self.blok = None;
            if let Some(hata) = hata {
                return BaglantiSonucu::hata(hata.turu(), hata.mesaji());
            }
            let sha256 = crate::guvenlik::sha256_hex(&icerik);
            let bayt = icerik.len() as i64;
            self.bloklar.push(icerik);
            return BaglantiSonucu::Blok {
                yol: format!(".zee/baglanti-bloklari/{sira:064x}.parca"),
                sha256,
                bayt,
            };
        }
        if kapaniyor {
            self.blok = None;
            self.okuma_kapali = true;
            return BaglantiSonucu::hata("kapanıyor", "sunucu kapanıyor; blok atıldı");
        }
        let sonuc = self.veri_yok("blok süresi içinde tamamlanmadı; atıldı", true);
        if sonuc != BaglantiSonucu::Bekle {
            self.blok = None;
        }
        sonuc
    }

    fn yaz(
        &mut self,
        metin: &str,
        bayt: Result<Vec<u8>, &'static str>,
        satir: bool,
    ) -> BaglantiSonucu {
        match bayt {
            Ok(bayt) => {
                if satir {
                    self.giden.push(metin.to_string());
                }
                self.ham_giden.extend_from_slice(&bayt);
                BaglantiSonucu::Tamam(bayt.len() as i64)
            }
            Err(neden) => BaglantiSonucu::hata("yazılamadı", neden),
        }
    }

    fn tls_baslat(&mut self, tls_var: bool) -> BaglantiSonucu {
        if self.tls {
            return BaglantiSonucu::hata("tls", crate::ag_satiri::ZATEN_TLS);
        }
        if !tls_var {
            return BaglantiSonucu::hata("tls", crate::ag_satiri::TLS_KIMLIGI_YOK);
        }
        let okuyucu = self.okuyucu.as_mut();
        if okuyucu.as_ref().is_some_and(|o| o.tamponlu_bayt() > 0) {
            self.kapandi = true;
            return BaglantiSonucu::hata("tls", crate::ag_satiri::STARTTLS_ENJEKSIYONU);
        }
        okuyucu.into_iter().for_each(SatirOkuyucu::sifirla);
        self.tls = true;
        BaglantiSonucu::Satir("TLSv1_3 TLS13_AES_128_GCM_SHA256".into())
    }
}
