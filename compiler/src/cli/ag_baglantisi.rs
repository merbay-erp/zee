//! K-381 (RFC-0039 A2a): adlı dinleyici bağlantı reaktörü (Unix, `poll(2)`).
//!
//! Private CLI adapter. Devralınan ve doğrulanan adlı dinleyiciler
//! (K-376) bağlantı bloğu kaydında buraya alınır. Reaktör bütün soketleri
//! engellemeyen kipte tutar; yorumlayıcının bağlantı scheduler'ı ilerleyemeyen
//! görevleri `bekle` ile buraya sorar. Sınırlar burada zorlanır: eşzamanlı ve
//! uzak adres başına bağlantı, satır uzunluğu (ortak `dil::ag_satiri`), okuma
//! ve tıkanan yazma için çağrının başından mutlak boşta süresi, 64 KiB giden
//! tampon. Sınır dolunca bağlantı kabul edilip bildirilen kibar ret satırı tek
//! engellemeyen yazımla gönderilir ve kapatılır; Zee görevi kurulmaz.

use dil::ag_satiri::{giden_satir, BaglantiAyari, SatirOkuyucu, GIDEN_TAMPON_BAYTI, OKUMA_PARCASI};
use dil::yorumlayici::{BaglantiIslemi, BaglantiOlayi, BaglantiSonucu, YeniBaglanti};
use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, Shutdown, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

/// SIGTERM'in fark edilmesi için `poll` beklemesinin üst sınırı.
const AZAMI_POLL_MS: i64 = 250;
/// Kapatılan bağlantının giden tamponunu boşaltma süresi tavanı.
const KAPANIS_BOSALTMA: Duration = Duration::from_secs(5);
/// Tek uyanışta kabul edilen en çok bağlantı (dinleyici başına).
const KABUL_DILIMI: usize = 64;
const RET_GUNLUK_ARALIGI: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ilgi {
    Okuma,
    Yazma,
}

struct Bekleme {
    ilgi: Ilgi,
    son: Instant,
}

struct Baglanti {
    akis: TcpStream,
    dinleyici: String,
    uzak: IpAddr,
    okuyucu: SatirOkuyucu,
    giden: Vec<u8>,
    eof: bool,
    kirik: bool,
    bekleme: Option<Bekleme>,
    zaman_asti: bool,
}

struct Kapanan {
    akis: TcpStream,
    giden: Vec<u8>,
    son: Instant,
}

pub(crate) struct Reaktor {
    ayar: BaglantiAyari,
    baslangic: Instant,
    dinleyiciler: Vec<(String, TcpListener)>,
    baglantilar: BTreeMap<u64, Baglanti>,
    kapananlar: Vec<Kapanan>,
    sonraki: u64,
    kapanis: Option<super::web_kapanis::WebKapanis>,
    kapaniyor: bool,
    hazir_bildirildi: bool,
    ret_sayisi: u64,
    son_ret_gunlugu: Option<Instant>,
}

fn uzak_yazimi(ip: IpAddr) -> String {
    match ip {
        IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map_or_else(|| v6.to_string(), |v4| v4.to_string()),
        IpAddr::V4(v4) => v4.to_string(),
    }
}

/// Giden tamponu engellemeden olabildiğince boşaltır; `false`: bağlantı kırık.
fn bosalt(akis: &mut TcpStream, giden: &mut Vec<u8>) -> bool {
    while !giden.is_empty() {
        match akis.write(giden) {
            Ok(0) => return false,
            Ok(n) => {
                giden.drain(..n);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => return true,
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(_) => return false,
        }
    }
    true
}

fn poll_et(olaylar: &mut [libc::pollfd], zaman_asimi_ms: i64) {
    // SAFETY: poll receives a live, correctly sized slice of initialized
    // pollfd records for the entire synchronous call; every fd is owned by
    // this reactor and stays open during the call.
    let sonuc = unsafe {
        libc::poll(
            olaylar.as_mut_ptr(),
            olaylar.len() as libc::nfds_t,
            zaman_asimi_ms.clamp(0, AZAMI_POLL_MS) as libc::c_int,
        )
    };
    if sonuc < 0 {
        // EINTR (SIGTERM) dahil: çağıran durumu yeniden değerlendirir.
        olaylar.iter_mut().for_each(|o| o.revents = 0);
    }
}

impl Reaktor {
    pub(crate) fn yeni(ayar: BaglantiAyari, baslangic: Instant) -> Self {
        Self {
            ayar,
            baslangic,
            dinleyiciler: Vec::new(),
            baglantilar: BTreeMap::new(),
            kapananlar: Vec::new(),
            sonraki: 1,
            kapanis: None,
            kapaniyor: false,
            hazir_bildirildi: false,
            ret_sayisi: 0,
            son_ret_gunlugu: None,
        }
    }

    fn an(&self, ms: i64) -> Instant {
        self.baslangic + Duration::from_millis(ms.max(0) as u64)
    }

    /// Bağlantı bloğu kaydı: adlı dinleyiciyi devralınanlardan alır.
    pub(crate) fn kaydet(&mut self, ad: &str) -> Result<i64, String> {
        if !self.dinleyiciler.iter().any(|(a, _)| a == ad) {
            let dinleyici = super::adli_dinleyici::al(ad).ok_or_else(|| {
                "systemd'den adla devralınan dinleyiciler arasında yok (proje.dil `dinleyiciler` ve .socket FileDescriptorName= aynı ad olmalı)".to_string()
            })?;
            dinleyici
                .set_nonblocking(true)
                .map_err(|e| format!("dinleyici engellemeyen kipe alınamadı: {e}"))?;
            self.dinleyiciler.push((ad.to_string(), dinleyici));
        }
        if self.kapanis.is_none() {
            self.kapanis = Some(super::web_kapanis::WebKapanis::kur()?);
        }
        Ok(self.ayar.omur_ms)
    }

    fn sigterm_geldi(&self) -> bool {
        self.kapanis.as_ref().is_some_and(|k| k.istendi())
    }

    fn reddet(&mut self, mut akis: TcpStream) {
        if let Some(satir) = &self.ayar.dolu_yaniti {
            // Tek engellemeyen yazım; tamamlanmazsa da bağlantı kapanır.
            let _ = akis.write(format!("{satir}\r\n").as_bytes());
        }
        let _ = akis.shutdown(Shutdown::Both);
        self.ret_sayisi += 1;
        let simdi = Instant::now();
        if self
            .son_ret_gunlugu
            .is_none_or(|son| simdi.duration_since(son) >= RET_GUNLUK_ARALIGI)
        {
            eprintln!(
                "Zee bağlantı: sınır dolu; {} bağlantı kibar retle kapatıldı (bağlantı_sınırı {}, uzak_adres_bağlantı_sınırı {}).",
                self.ret_sayisi, self.ayar.sinir, self.ayar.uzak_adres_siniri
            );
            self.ret_sayisi = 0;
            self.son_ret_gunlugu = Some(simdi);
        }
    }

    fn kabul_et(&mut self, sira: usize, olaylar: &mut Vec<BaglantiOlayi>) {
        for _ in 0..KABUL_DILIMI {
            let (ad, dinleyici) = &self.dinleyiciler[sira];
            let ad = ad.clone();
            let (akis, es) = match dinleyici.accept() {
                Ok(kabul) => kabul,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return,
            };
            if akis.set_nonblocking(true).is_err() {
                continue;
            }
            let uzak = es.ip();
            let acik = self.baglantilar.len() + self.kapananlar.len();
            let ayni_uzak = self.baglantilar.values().filter(|b| b.uzak == uzak).count();
            if acik >= self.ayar.sinir || ayni_uzak >= self.ayar.uzak_adres_siniri {
                self.reddet(akis);
                continue;
            }
            let yerel_kapi = akis.local_addr().map(|a| a.port()).unwrap_or(0);
            let kimlik = self.sonraki;
            self.sonraki += 1;
            olaylar.push(BaglantiOlayi::Yeni(YeniBaglanti {
                kimlik,
                dinleyici: ad.clone(),
                uzak_adres: uzak_yazimi(uzak),
                uzak_kapi: es.port(),
                yerel_kapi,
            }));
            self.baglantilar.insert(
                kimlik,
                Baglanti {
                    akis,
                    dinleyici: ad,
                    uzak,
                    okuyucu: SatirOkuyucu::yeni(self.ayar.satir_siniri),
                    giden: Vec::new(),
                    eof: false,
                    kirik: false,
                    bekleme: None,
                    zaman_asti: false,
                },
            );
        }
    }

    /// Okumaya hazır sokette bir parça okur; satır okuyucunun bellek tavanı korunur.
    fn doldur(b: &mut Baglanti) -> bool {
        let alan = b.okuyucu.bos_alan().min(OKUMA_PARCASI);
        if alan == 0 {
            return false;
        }
        let mut tampon = [0u8; OKUMA_PARCASI];
        loop {
            match b.akis.read(&mut tampon[..alan]) {
                Ok(0) => {
                    b.eof = true;
                    return true;
                }
                Ok(n) => {
                    b.okuyucu.besle(&tampon[..n]);
                    return true;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) if e.kind() == ErrorKind::WouldBlock => return false,
                Err(_) => {
                    b.kirik = true;
                    return true;
                }
            }
        }
    }

    fn ilk_bekleme(&mut self) {
        if !self.hazir_bildirildi {
            self.hazir_bildirildi = true;
            super::systemd_bildirim::hazir();
            eprintln!(
                "Zee bağlantı: {} adlı dinleyicide bağlantı kabul ediliyor.",
                self.dinleyiciler.len()
            );
        }
    }

    /// SIGTERM: kabul durur, bekleyen her görev `kapanıyor` için uyanır.
    fn kapanisi_denetle(&mut self, bekleyenler: &[u64]) -> Option<Vec<BaglantiOlayi>> {
        if self.kapaniyor || !self.sigterm_geldi() {
            return None;
        }
        self.kapaniyor = true;
        super::systemd_bildirim::durduruluyor();
        // Yeni kabul durur; dinleyiciler systemd'de açık kalır.
        self.dinleyiciler.clear();
        let mut olaylar = vec![BaglantiOlayi::Kapanis];
        olaylar.extend(bekleyenler.iter().map(|k| BaglantiOlayi::Hazir(*k)));
        Some(olaylar)
    }

    /// Giden tamponları boşaltır; süresi dolan ya da kırılan bekleyenleri uyandırır.
    fn suresi_dolanlar(&mut self, bekleyenler: &[u64], simdi: Instant) -> Vec<BaglantiOlayi> {
        self.kapananlar.retain_mut(|k| {
            bosalt(&mut k.akis, &mut k.giden) && !k.giden.is_empty() && simdi < k.son
        });
        for b in self.baglantilar.values_mut() {
            if !b.giden.is_empty() && !bosalt(&mut b.akis, &mut b.giden) {
                b.kirik = true;
            }
        }
        let mut olaylar = Vec::new();
        for kimlik in bekleyenler {
            let Some(b) = self.baglantilar.get_mut(kimlik) else {
                olaylar.push(BaglantiOlayi::Hazir(*kimlik));
                continue;
            };
            let doldu = b.bekleme.as_ref().is_none_or(|w| simdi >= w.son);
            if doldu || b.kirik {
                b.zaman_asti = doldu && !b.kirik;
                olaylar.push(BaglantiOlayi::Hazir(*kimlik));
            }
        }
        olaylar
    }

    /// Dinleyiciler, bekleyen bağlantılar ve kapananlar için `pollfd` listesi.
    fn poll_listesi(&self, bekleyenler: &[u64]) -> (Vec<libc::pollfd>, Vec<u64>) {
        let olay = |fd, events| libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        let mut pollfd: Vec<libc::pollfd> = self
            .dinleyiciler
            .iter()
            .map(|(_, d)| olay(d.as_raw_fd(), libc::POLLIN))
            .collect();
        let mut sahip = Vec::new();
        for kimlik in bekleyenler {
            let Some(b) = self.baglantilar.get(kimlik) else {
                continue;
            };
            let okuma = b.bekleme.as_ref().is_none_or(|w| w.ilgi == Ilgi::Okuma);
            let ilgi = if okuma { libc::POLLIN } else { libc::POLLOUT };
            pollfd.push(olay(b.akis.as_raw_fd(), ilgi));
            sahip.push(*kimlik);
        }
        pollfd.extend(
            self.kapananlar
                .iter()
                .map(|k| olay(k.akis.as_raw_fd(), libc::POLLOUT)),
        );
        (pollfd, sahip)
    }

    fn poll_suresi(&self, bekleyenler: &[u64], uyanma: Option<Instant>, simdi: Instant) -> i64 {
        let sinir = bekleyenler
            .iter()
            .filter_map(|k| self.baglantilar.get(k)?.bekleme.as_ref().map(|w| w.son))
            .chain(uyanma)
            .chain(self.kapananlar.iter().map(|k| k.son))
            .fold(
                simdi + Duration::from_millis(AZAMI_POLL_MS as u64),
                Instant::min,
            );
        sinir.saturating_duration_since(simdi).as_millis() as i64
    }

    /// `poll` sonucunu işler: yeni bağlantıları kabul eder, hazır bekleyenleri doldurur.
    fn poll_sonucu(&mut self, pollfd: &[libc::pollfd], sahip: &[u64]) -> Vec<BaglantiOlayi> {
        let mut olaylar = Vec::new();
        let dinleyici_sayisi = self.dinleyiciler.len();
        for (sira, olay) in pollfd[..dinleyici_sayisi].iter().enumerate() {
            if olay.revents != 0 {
                self.kabul_et(sira, &mut olaylar);
            }
        }
        for (olay, kimlik) in pollfd[dinleyici_sayisi..].iter().zip(sahip) {
            if olay.revents == 0 {
                continue;
            }
            if let Some(b) = self.baglantilar.get_mut(kimlik) {
                if b.bekleme.as_ref().is_none_or(|w| w.ilgi == Ilgi::Okuma) {
                    Self::doldur(b);
                } else if !bosalt(&mut b.akis, &mut b.giden) {
                    b.kirik = true;
                }
            }
            olaylar.push(BaglantiOlayi::Hazir(*kimlik));
        }
        olaylar
    }

    pub(crate) fn bekle(
        &mut self,
        bekleyenler: &[u64],
        uyanma_an_ms: Option<i64>,
    ) -> Result<Vec<BaglantiOlayi>, String> {
        self.ilk_bekleme();
        let uyanma = uyanma_an_ms.map(|ms| self.an(ms));
        loop {
            if let Some(olaylar) = self.kapanisi_denetle(bekleyenler) {
                return Ok(olaylar);
            }
            let simdi = Instant::now();
            let olaylar = self.suresi_dolanlar(bekleyenler, simdi);
            if !olaylar.is_empty() || uyanma.is_some_and(|an| simdi >= an) {
                return Ok(olaylar);
            }
            let (mut pollfd, sahip) = self.poll_listesi(bekleyenler);
            poll_et(&mut pollfd, self.poll_suresi(bekleyenler, uyanma, simdi));
            let olaylar = self.poll_sonucu(&pollfd, &sahip);
            if !olaylar.is_empty() {
                return Ok(olaylar);
            }
        }
    }

    fn satir_oku(&mut self, kimlik: u64) -> BaglantiSonucu {
        let bosta = self.ayar.bosta_suresi_ms;
        let kapaniyor = self.kapaniyor;
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        if kapaniyor {
            b.bekleme = None;
            return BaglantiSonucu::hata("kapanıyor", "sunucu kapanıyor");
        }
        loop {
            match b.okuyucu.siradaki() {
                Some(Ok(satir)) => {
                    b.bekleme = None;
                    b.zaman_asti = false;
                    return BaglantiSonucu::Satir(satir);
                }
                Some(Err(hata)) => {
                    b.bekleme = None;
                    b.zaman_asti = false;
                    return BaglantiSonucu::hata(hata.turu(), hata.mesaji());
                }
                None => {}
            }
            if b.eof || b.kirik {
                b.bekleme = None;
                return BaglantiSonucu::hata("kapandı", "karşı taraf bağlantıyı kapattı");
            }
            if b.zaman_asti {
                b.bekleme = None;
                b.zaman_asti = false;
                return BaglantiSonucu::hata(
                    "zaman_aşımı",
                    "satır bağlantı_boşta_süresi içinde tamamlanmadı",
                );
            }
            if !Self::doldur(b) {
                if b.bekleme.is_none() {
                    b.bekleme = Some(Bekleme {
                        ilgi: Ilgi::Okuma,
                        son: Instant::now() + Duration::from_millis(bosta as u64),
                    });
                }
                return BaglantiSonucu::Bekle;
            }
        }
    }

    fn satir_yaz(&mut self, kimlik: u64, metin: &str) -> BaglantiSonucu {
        let bosta = self.ayar.bosta_suresi_ms;
        let sinir = self.ayar.satir_siniri;
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        let bayt = match giden_satir(metin, sinir) {
            Ok(bayt) => bayt,
            Err(neden) => return BaglantiSonucu::hata("yazılamadı", neden),
        };
        if b.kirik {
            b.bekleme = None;
            return BaglantiSonucu::hata("yazılamadı", "karşı taraf bağlantıyı kapattı");
        }
        if b.giden.len() + bayt.len() > GIDEN_TAMPON_BAYTI && !bosalt(&mut b.akis, &mut b.giden) {
            b.kirik = true;
            return BaglantiSonucu::hata("yazılamadı", "karşı taraf bağlantıyı kapattı");
        }
        if b.giden.len() + bayt.len() > GIDEN_TAMPON_BAYTI {
            if b.zaman_asti {
                b.bekleme = None;
                b.zaman_asti = false;
                return BaglantiSonucu::hata(
                    "yazılamadı",
                    "giden tampon bağlantı_boşta_süresi içinde boşalmadı",
                );
            }
            if b.bekleme.is_none() {
                b.bekleme = Some(Bekleme {
                    ilgi: Ilgi::Yazma,
                    son: Instant::now() + Duration::from_millis(bosta as u64),
                });
            }
            return BaglantiSonucu::Bekle;
        }
        b.bekleme = None;
        b.zaman_asti = false;
        b.giden.extend_from_slice(&bayt);
        if !bosalt(&mut b.akis, &mut b.giden) {
            b.kirik = true;
            return BaglantiSonucu::hata("yazılamadı", "karşı taraf bağlantıyı kapattı");
        }
        BaglantiSonucu::Tamam(bayt.len() as i64)
    }

    fn kapat(&mut self, kimlik: u64) -> BaglantiSonucu {
        let Some(mut b) = self.baglantilar.remove(&kimlik) else {
            return BaglantiSonucu::Tamam(0);
        };
        if !b.kirik
            && !b.giden.is_empty()
            && bosalt(&mut b.akis, &mut b.giden)
            && !b.giden.is_empty()
        {
            let bosta = Duration::from_millis(self.ayar.bosta_suresi_ms as u64);
            self.kapananlar.push(Kapanan {
                akis: b.akis,
                giden: b.giden,
                son: Instant::now() + bosta.min(KAPANIS_BOSALTMA),
            });
        } else {
            let _ = b.akis.shutdown(Shutdown::Write);
        }
        BaglantiSonucu::Tamam(0)
    }

    fn engelle(&mut self, kimlik: u64, son_ms: Option<i64>) -> BaglantiSonucu {
        let son = son_ms.map(|ms| self.an(ms));
        let Some(b) = self.baglantilar.get(&kimlik) else {
            return BaglantiSonucu::Tamam(0);
        };
        let ilgi = b.bekleme.as_ref().map_or(Ilgi::Okuma, |w| w.ilgi);
        let bekleme_sonu = b.bekleme.as_ref().map(|w| w.son);
        let fd = b.akis.as_raw_fd();
        let mut sinir = bekleme_sonu
            .unwrap_or_else(|| Instant::now() + Duration::from_millis(AZAMI_POLL_MS as u64));
        if let Some(son) = son {
            sinir = sinir.min(son);
        }
        let hazir = loop {
            let simdi = Instant::now();
            if simdi >= sinir || self.sigterm_geldi() {
                break false;
            }
            let mut olay = [libc::pollfd {
                fd,
                events: if ilgi == Ilgi::Okuma {
                    libc::POLLIN
                } else {
                    libc::POLLOUT
                },
                revents: 0,
            }];
            poll_et(
                &mut olay,
                sinir.saturating_duration_since(simdi).as_millis() as i64,
            );
            if olay[0].revents != 0 {
                break true;
            }
        };
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::Tamam(0);
        };
        if hazir {
            if ilgi == Ilgi::Okuma {
                Self::doldur(b);
            } else if !bosalt(&mut b.akis, &mut b.giden) {
                b.kirik = true;
            }
        } else if bekleme_sonu.is_some_and(|an| Instant::now() >= an) {
            b.zaman_asti = true;
        }
        BaglantiSonucu::Tamam(0)
    }

    pub(crate) fn islem(&mut self, kimlik: u64, islem: &BaglantiIslemi) -> BaglantiSonucu {
        match islem {
            BaglantiIslemi::SatirOku => self.satir_oku(kimlik),
            BaglantiIslemi::SatirYaz(metin) => self.satir_yaz(kimlik, metin),
            BaglantiIslemi::Kapat => self.kapat(kimlik),
            BaglantiIslemi::Engelle(son) => self.engelle(kimlik, *son),
        }
    }

    pub(crate) fn gunlukle(&self, kimlik: u64, neden: &str) {
        let (dinleyici, uzak) = self
            .baglantilar
            .get(&kimlik)
            .map(|b| (b.dinleyici.as_str(), uzak_yazimi(b.uzak)))
            .unwrap_or(("?", "?".into()));
        eprintln!("Zee bağlantı #{kimlik} ({dinleyici}, {uzak}) kapandı: {neden}");
    }
}
