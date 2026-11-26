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
//!
//! K-384 (A2c): nokta bloğu ve literal `0600` geçici `.parca` dosyasına
//! SHA-256 aynı akışta hesaplanarak akar; yarım dosya her hatada silinir,
//! tamamlanan dosya bir sonraki blokta ya da bağlantı bitince silinir (Zee
//! taşımadıysa). Dosyadan bağlantıya aktarım giden tamponu geri basınçla
//! doldurur. İkisinin de süresi çağrının başından mutlak
//! `boşta + bayt / AKIS_ASGARI_HIZI` bütçesidir.
//!
//! K-397 (RFC-0044 B4a, ADR-173): giden bağlantılar aynı tabloya girer.
//! Hedef ve IP politikası `YetkinlikPolitikasi::giden_tcp_hedefini_coz` ile
//! tek çözümlemede denetlenir, soket yalnız dönen adrese açılır (yeniden
//! bağlama yok). Bağlanma `min(Zee son tarihi, 30 sn)`; süreç başına en çok
//! `AZAMI_GIDEN_BAGLANTI` açık giden bağlantı. İstemci STARTTLS'i sunucu
//! yükseltmesiyle aynı tampon kuralını izler.

use super::ag_tls::Kanal;
use super::web_yukleme::{
    yetim_parcalari_supur_dizinde, AkanYukleme, IstekYuklemesi, BAGLANTI_BLOK_DIZINI,
    YETIM_SUPURME_ARALIGI,
};
use dil::ag_satiri::{
    akis_butcesi_ms, bayt_metni, giden_parca, giden_satir, BaglantiAyari, BlokOkuyucu,
    SatirOkuyucu, BAYT_OKUMA_SINIRI, BLOK_SINIRI_ASILDI, GIDEN_TAMPON_BAYTI, OKUMA_PARCASI,
};
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
/// Tek blok/aktarım çağrısının bir turda işlediği en çok bayt; sonra görev
/// scheduler'a döner (hızlı istemci kardeş bağlantıları aç bırakmaz).
const BLOK_DILIMI: usize = 256 * 1024;
/// Bağlantı blok dosyasının yetim sayılması için `bağlantı_ömrü`ne eklenen pay.
const BLOK_YETIM_PAYI: Duration = Duration::from_secs(15 * 60);

/// Sürmekte olan blok okuması.
struct AcikBlok {
    okuyucu: BlokOkuyucu,
    yukleme: AkanYukleme,
    baslangic: Instant,
    /// Bütçe tabanı: literal'de bildirilen adet, nokta bloğunda 0.
    taban: u64,
    /// Bağlantıdan tüketilen ham bayt.
    alinan: u64,
}

impl AcikBlok {
    fn son(&self, bosta_ms: i64) -> Instant {
        let butce = akis_butcesi_ms(bosta_ms, self.taban.max(self.alinan));
        self.baslangic + Duration::from_millis(butce.max(0) as u64)
    }
}

/// Sürmekte olan dosyadan bağlantıya aktarım.
struct Aktarim {
    dosya: std::fs::File,
    /// Dosyadan okunacak ham bayt.
    kalan: u64,
    /// Zee'ye dönen bayt: ham aktarımda dosya boyu, base64'te çıktı penceresi.
    toplam: u64,
    son: Instant,
    /// K-396: RFC 2045 satırlı base64 akışı; `None` ham aktarım.
    kodlu: Option<KodluAktarim>,
}

/// K-396: kodlanmış çıktı penceresinin akış durumu. Bellekte en çok bir ham
/// parçanın (57·256 bayt) kodlaması tutulur; `Metin` üretilmez.
struct KodluAktarim {
    bekleyen: Vec<u8>,
    bas: usize,
    /// İlk satırdan atlanacak çıktı baytı (pencere satır ortasında başlar).
    atla: u64,
    cikti_kalan: u64,
}

/// K-396: bir okumada kodlanan ham bayt; 57'nin katı, satır hizası korunur.
const KODLU_HAM_PARCA: u64 = dil::kodlama::BASE64_SATIR_HAM_BAYTI * 256;

impl Aktarim {
    fn bitti(&self) -> bool {
        match &self.kodlu {
            Some(k) => k.cikti_kalan == 0,
            None => self.kalan == 0,
        }
    }
}

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
    kanal: Kanal,
    dinleyici: String,
    uzak: IpAddr,
    okuyucu: SatirOkuyucu,
    giden: Vec<u8>,
    eof: bool,
    kirik: bool,
    bekleme: Option<Bekleme>,
    zaman_asti: bool,
    /// Implicit TLS el sıkışması sürüyor: (son an, uzak kapı, yerel kapı).
    /// Bitmeden Zee görevi kurulmaz.
    el_sikisma: Option<(Instant, u16, u16)>,
    /// STARTTLS el sıkışması sürüyor.
    yukseltiliyor: bool,
    /// K-397: giden bağlantı (dinleyiciden kabul edilmedi).
    giden_mi: bool,
    /// K-384: sürmekte olan blok; düşünce yarım dosya silinir.
    blok: Option<AcikBlok>,
    /// K-384: son tamamlanan blok dosyası; sonraki blokta ya da bağlantı
    /// bitince silinir (Zee taşıdıysa etkisiz).
    blok_dosyasi: Option<IstekYuklemesi>,
    /// K-384: yarıda kalan blok/literal akışı eşzamanlamayı bozdu; sonraki
    /// okumalar `kapandı` döner, yazma ve kapatma çalışır.
    okuma_kapali: bool,
    aktarim: Option<Aktarim>,
}

struct Kapanan {
    kanal: Kanal,
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
    /// Implicit TLS dinleyici adları (proje.dil `tls_dinleyicileri`).
    implicit_tls: Vec<String>,
    kapaniyor: bool,
    hazir_bildirildi: bool,
    ret_sayisi: u64,
    son_ret_gunlugu: Option<Instant>,
    /// Göreli IO kökü (blok dosyaları `kök/.zee/baglanti-bloklari`).
    kok: std::path::PathBuf,
    son_supurme: Option<Instant>,
}

fn uzak_yazimi(ip: IpAddr) -> String {
    match ip {
        IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map_or_else(|| v6.to_string(), |v4| v4.to_string()),
        IpAddr::V4(v4) => v4.to_string(),
    }
}

/// Giden tamponu (ve bekleyen TLS kayıtlarını) engellemeden olabildiğince
/// boşaltır; `false`: bağlantı kırık.
fn bosalt(kanal: &mut Kanal, giden: &mut Vec<u8>) -> bool {
    while !giden.is_empty() {
        match kanal.write(giden) {
            Ok(0) => return false,
            Ok(n) => {
                giden.drain(..n);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => return true,
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(_) => return false,
        }
    }
    kanal.tls_bosalt().is_ok()
}

/// Kapanmamış giden veri (düz tampon ya da TLS kaydı) var mı.
fn bekleyen_cikti(kanal: &Kanal, giden: &[u8]) -> bool {
    !giden.is_empty() || kanal.yazma_bekliyor()
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
    pub(crate) fn yeni(ayar: BaglantiAyari, baslangic: Instant, kok: std::path::PathBuf) -> Self {
        let implicit_tls = ayar
            .tls
            .as_ref()
            .map(|t| t.dinleyiciler.clone())
            .unwrap_or_default();
        Self {
            ayar,
            baslangic,
            dinleyiciler: Vec::new(),
            baglantilar: BTreeMap::new(),
            kapananlar: Vec::new(),
            sonraki: 1,
            kapanis: None,
            implicit_tls,
            kapaniyor: false,
            hazir_bildirildi: false,
            ret_sayisi: 0,
            son_ret_gunlugu: None,
            kok,
            son_supurme: None,
        }
    }

    /// K-384: süreç ölümünden kalan blok dosyalarını süpürür. Eşik en uzun
    /// bağlantı ömrünü aştığı için canlı oturumun dosyasına dokunulmaz.
    fn yetimleri_supur(&mut self) {
        if self
            .son_supurme
            .is_some_and(|an| an.elapsed() < YETIM_SUPURME_ARALIGI)
        {
            return;
        }
        self.son_supurme = Some(Instant::now());
        let yas = Duration::from_millis(self.ayar.omur_ms.max(0) as u64) + BLOK_YETIM_PAYI;
        let silinen = yetim_parcalari_supur_dizinde(
            &self.kok,
            BAGLANTI_BLOK_DIZINI,
            std::time::SystemTime::now(),
            yas,
        );
        if silinen > 0 {
            eprintln!("Zee bağlantı: {silinen} yetim blok geçici dosyası silindi.");
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
            let acik =
                self.baglantilar.values().filter(|b| !b.giden_mi).count() + self.kapananlar.len();
            let ayni_uzak = self.baglantilar.values().filter(|b| b.uzak == uzak).count();
            if acik >= self.ayar.sinir || ayni_uzak >= self.ayar.uzak_adres_siniri {
                self.reddet(akis);
                continue;
            }
            let yerel_kapi = akis.local_addr().map(|a| a.port()).unwrap_or(0);
            let mut kanal = Kanal::duz(akis);
            let implicit = self.implicit_tls.contains(&ad);
            if implicit && kanal.tls_ac().is_err() {
                continue;
            }
            let kimlik = self.sonraki;
            self.sonraki += 1;
            let el_sikisma = implicit.then(|| {
                let son = Instant::now() + Duration::from_millis(self.ayar.bosta_suresi_ms as u64);
                (son, es.port(), yerel_kapi)
            });
            if !implicit {
                olaylar.push(BaglantiOlayi::Yeni(YeniBaglanti {
                    kimlik,
                    dinleyici: ad.clone(),
                    uzak_adres: uzak_yazimi(uzak),
                    uzak_kapi: es.port(),
                    yerel_kapi,
                    tls: false,
                }));
            }
            self.baglantilar.insert(
                kimlik,
                Baglanti {
                    kanal,
                    dinleyici: ad,
                    uzak,
                    okuyucu: SatirOkuyucu::yeni(self.ayar.satir_siniri),
                    giden: Vec::new(),
                    giden_mi: false,
                    eof: false,
                    kirik: false,
                    bekleme: None,
                    zaman_asti: false,
                    el_sikisma,
                    yukseltiliyor: false,
                    blok: None,
                    blok_dosyasi: None,
                    okuma_kapali: false,
                    aktarim: None,
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
            match b.kanal.read(&mut tampon[..alan]) {
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
            bosalt(&mut k.kanal, &mut k.giden)
                && bekleyen_cikti(&k.kanal, &k.giden)
                && simdi < k.son
        });
        for b in self.baglantilar.values_mut() {
            if bekleyen_cikti(&b.kanal, &b.giden) && !bosalt(&mut b.kanal, &mut b.giden) {
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
            pollfd.push(olay(b.kanal.akis.as_raw_fd(), Self::ilgi(b)));
            sahip.push(*kimlik);
        }
        for (kimlik, b) in &self.baglantilar {
            if b.el_sikisma.is_some() {
                pollfd.push(olay(b.kanal.akis.as_raw_fd(), Self::ilgi(b)));
                sahip.push(*kimlik);
            }
        }
        pollfd.extend(
            self.kapananlar
                .iter()
                .map(|k| olay(k.kanal.akis.as_raw_fd(), libc::POLLOUT)),
        );
        (pollfd, sahip)
    }

    fn ilgi(b: &Baglanti) -> libc::c_short {
        let okuma = b.bekleme.as_ref().is_none_or(|w| w.ilgi == Ilgi::Okuma);
        let temel = if okuma { libc::POLLIN } else { libc::POLLOUT };
        if b.kanal.yazma_bekliyor() {
            temel | libc::POLLOUT
        } else {
            temel
        }
    }

    /// Implicit TLS el sıkışmalarını ilerletir; biten `Yeni` olur, süresi
    /// dolan ya da bozulan Zee görevi kurulmadan kapanır.
    fn el_sikismalari(&mut self, simdi: Instant, olaylar: &mut Vec<BaglantiOlayi>) {
        let mut bitenler = Vec::new();
        for (kimlik, b) in &mut self.baglantilar {
            let Some((son, uzak_kapi, yerel_kapi)) = b.el_sikisma else {
                continue;
            };
            match b.kanal.el_sikis() {
                Ok(true) => {
                    b.el_sikisma = None;
                    olaylar.push(BaglantiOlayi::Yeni(YeniBaglanti {
                        kimlik: *kimlik,
                        dinleyici: b.dinleyici.clone(),
                        uzak_adres: uzak_yazimi(b.uzak),
                        uzak_kapi,
                        yerel_kapi,
                        tls: true,
                    }));
                }
                Ok(false) if simdi < son => {}
                _ => bitenler.push(*kimlik),
            }
        }
        for kimlik in bitenler {
            self.baglantilar.remove(&kimlik);
        }
    }

    fn poll_suresi(&self, bekleyenler: &[u64], uyanma: Option<Instant>, simdi: Instant) -> i64 {
        let sinir = bekleyenler
            .iter()
            .filter_map(|k| self.baglantilar.get(k)?.bekleme.as_ref().map(|w| w.son))
            .chain(uyanma)
            .chain(self.kapananlar.iter().map(|k| k.son))
            .chain(
                self.baglantilar
                    .values()
                    .filter_map(|b| b.el_sikisma.map(|e| e.0)),
            )
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
                if b.el_sikisma.is_some() {
                    continue;
                }
                if b.bekleme.as_ref().is_none_or(|w| w.ilgi == Ilgi::Okuma) {
                    Self::doldur(b);
                } else if !bosalt(&mut b.kanal, &mut b.giden) {
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
        self.yetimleri_supur();
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
            let mut olaylar = self.poll_sonucu(&pollfd, &sahip);
            self.el_sikismalari(Instant::now(), &mut olaylar);
            if !olaylar.is_empty() {
                return Ok(olaylar);
            }
        }
    }

    /// `sure_ms`: okumanın çağrı başından mutlak süresi (boşta süresi ya da
    /// K-384 kısa süre); zaman aşımı bağlantıyı kapatmaz, yarım satır
    /// tamponda kalır ve sonraki okuma onu tamamlar.
    fn satir_oku(&mut self, kimlik: u64, sure_ms: i64) -> BaglantiSonucu {
        let bosta = sure_ms.clamp(1, self.ayar.bosta_suresi_ms.max(1));
        let kapaniyor = self.kapaniyor;
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        if b.okuma_kapali {
            return BaglantiSonucu::hata("kapandı", dil::ag_satiri::OKUMA_KAPALI);
        }
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
                return BaglantiSonucu::hata("zaman_aşımı", "satır süresi içinde tamamlanmadı");
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

    /// Satır (CRLF eklenir) ya da K-384 parça (satır sonu eklenmez) yazımı.
    fn satir_yaz(&mut self, kimlik: u64, metin: &str, satir_sonu: bool) -> BaglantiSonucu {
        let bosta = self.ayar.bosta_suresi_ms;
        let sinir = self.ayar.satir_siniri;
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        let bayt = if satir_sonu {
            giden_satir(metin, sinir)
        } else {
            giden_parca(metin, sinir)
        };
        let bayt = match bayt {
            Ok(bayt) => bayt,
            Err(neden) => return BaglantiSonucu::hata("yazılamadı", neden),
        };
        if b.kirik {
            b.bekleme = None;
            return BaglantiSonucu::hata("yazılamadı", "karşı taraf bağlantıyı kapattı");
        }
        if b.giden.len() + bayt.len() > GIDEN_TAMPON_BAYTI && !bosalt(&mut b.kanal, &mut b.giden) {
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
        if !bosalt(&mut b.kanal, &mut b.giden) {
            b.kirik = true;
            return BaglantiSonucu::hata("yazılamadı", "karşı taraf bağlantıyı kapattı");
        }
        BaglantiSonucu::Tamam(bayt.len() as i64)
    }

    fn kapat(&mut self, kimlik: u64) -> BaglantiSonucu {
        let Some(mut b) = self.baglantilar.remove(&kimlik) else {
            return BaglantiSonucu::Tamam(0);
        };
        b.kanal.kapanis_bildir();
        if !b.kirik && bosalt(&mut b.kanal, &mut b.giden) && bekleyen_cikti(&b.kanal, &b.giden) {
            let bosta = Duration::from_millis(self.ayar.bosta_suresi_ms as u64);
            self.kapananlar.push(Kapanan {
                kanal: b.kanal,
                giden: b.giden,
                son: Instant::now() + bosta.min(KAPANIS_BOSALTMA),
            });
        } else {
            let _ = b.kanal.akis.shutdown(Shutdown::Write);
        }
        BaglantiSonucu::Tamam(0)
    }

    /// STARTTLS: ara bellek boş olmalı (aksi hâlde bağlantı kapanır), düz
    /// giden tampon önce boşaltılır, sonra el sıkışma süresi içinde biter.
    /// K-397: `istemci` doluysa giden bağlantıda istemci oturumu (SNI).
    fn tls_baslat(&mut self, kimlik: u64, istemci: Option<&str>) -> BaglantiSonucu {
        let bosta = self.ayar.bosta_suresi_ms;
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        if !b.yukseltiliyor {
            if let Some(ret) = tls_on_kosulu(b, istemci) {
                return ret;
            }
            if b.okuyucu.tamponlu_bayt() > 0 {
                // CVE-2011-0411 (sunucu) / CVE-2011-1575 (istemci) sınıfı:
                // STARTTLS ile aynı pakette gelen düz metin TLS sonrasına
                // taşınamaz; tampon atılır, bağlantı kapanır.
                b.okuyucu.sifirla();
                b.kirik = true;
                self.kapat(kimlik);
                let neden = if istemci.is_some() {
                    dil::ag_satiri::STARTTLS_YANIT_ENJEKSIYONU
                } else {
                    dil::ag_satiri::STARTTLS_ENJEKSIYONU
                };
                return BaglantiSonucu::hata("tls", neden);
            }
            if !bosalt(&mut b.kanal, &mut b.giden) {
                b.kirik = true;
                return BaglantiSonucu::hata("yazılamadı", "karşı taraf bağlantıyı kapattı");
            }
            if !b.giden.is_empty() {
                b.bekleme.get_or_insert(Bekleme {
                    ilgi: Ilgi::Yazma,
                    son: Instant::now() + Duration::from_millis(bosta as u64),
                });
                if b.zaman_asti {
                    b.kirik = true;
                    self.kapat(kimlik);
                    return BaglantiSonucu::hata("tls", "STARTTLS öncesi giden tampon boşalmadı");
                }
                return BaglantiSonucu::Bekle;
            }
            let acildi = match istemci {
                Some(ad) => b.kanal.tls_istemci_ac(ad),
                None => b.kanal.tls_ac(),
            };
            if let Err(neden) = acildi {
                return BaglantiSonucu::hata("tls", neden);
            }
            b.okuyucu.sifirla();
            b.yukseltiliyor = true;
            b.bekleme = None;
            b.zaman_asti = false;
        }
        match b.kanal.el_sikis() {
            Ok(true) => {
                b.yukseltiliyor = false;
                b.bekleme = None;
                if b.giden_mi {
                    BaglantiSonucu::TlsIstemci(Box::new(b.kanal.istemci_kaydi()))
                } else {
                    BaglantiSonucu::Satir(b.kanal.tls_bilgisi())
                }
            }
            Ok(false) if !b.zaman_asti => {
                b.bekleme.get_or_insert(Bekleme {
                    ilgi: Ilgi::Okuma,
                    son: Instant::now() + Duration::from_millis(bosta as u64),
                });
                BaglantiSonucu::Bekle
            }
            _ => {
                b.kirik = true;
                self.kapat(kimlik);
                BaglantiSonucu::hata("tls", "TLS el sıkışması başarısız ya da süresinde bitmedi")
            }
        }
    }

    fn engelle(&mut self, kimlik: u64, son_ms: Option<i64>) -> BaglantiSonucu {
        let son = son_ms.map(|ms| self.an(ms));
        let Some(b) = self.baglantilar.get(&kimlik) else {
            return BaglantiSonucu::Tamam(0);
        };
        let ilgi = b.bekleme.as_ref().map_or(Ilgi::Okuma, |w| w.ilgi);
        let bekleme_sonu = b.bekleme.as_ref().map(|w| w.son);
        let fd = b.kanal.akis.as_raw_fd();
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
            } else if !bosalt(&mut b.kanal, &mut b.giden) {
                b.kirik = true;
            }
        } else if bekleme_sonu.is_some_and(|an| Instant::now() >= an) {
            b.zaman_asti = true;
        }
        BaglantiSonucu::Tamam(0)
    }

    /// K-384: bayt-kesin okuma (en çok satır sınırı, bellekte).
    fn bayt_oku(&mut self, kimlik: u64, adet: i64) -> BaglantiSonucu {
        let bosta = self.ayar.bosta_suresi_ms;
        let sinir = self.ayar.satir_siniri;
        let kapaniyor = self.kapaniyor;
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        if b.okuma_kapali {
            return BaglantiSonucu::hata("kapandı", dil::ag_satiri::OKUMA_KAPALI);
        }
        if kapaniyor {
            b.bekleme = None;
            return BaglantiSonucu::hata("kapanıyor", "sunucu kapanıyor");
        }
        let adet = adet.max(0) as usize;
        if adet > sinir {
            return BaglantiSonucu::hata("boyut_aşımı", BAYT_OKUMA_SINIRI);
        }
        loop {
            if let Some(bayt) = b.okuyucu.bayt_al(adet) {
                b.bekleme = None;
                b.zaman_asti = false;
                return match bayt_metni(bayt) {
                    Ok(metin) => BaglantiSonucu::Satir(metin),
                    Err(hata) => BaglantiSonucu::hata(hata.turu(), hata.mesaji()),
                };
            }
            if b.eof || b.kirik {
                b.bekleme = None;
                b.okuma_kapali = true;
                return BaglantiSonucu::hata("kapandı", "karşı taraf baytlar bitmeden kapattı");
            }
            if b.zaman_asti {
                b.bekleme = None;
                b.zaman_asti = false;
                b.okuma_kapali = true;
                return BaglantiSonucu::hata("zaman_aşımı", "baytlar süresi içinde gelmedi");
            }
            if !Self::doldur(b) {
                b.bekleme.get_or_insert(Bekleme {
                    ilgi: Ilgi::Okuma,
                    son: Instant::now()
                        + Duration::from_millis(akis_butcesi_ms(bosta, adet as u64) as u64),
                });
                return BaglantiSonucu::Bekle;
            }
        }
    }

    /// K-384: yarıda kalan bloğu bırakır (yarım dosya düşerken silinir) ve
    /// okuma yönünü kapatır: eşzamanı bozuk akışın kalanı komut olamaz.
    fn blok_kes(b: &mut Baglanti, tur: &str, mesaj: &str) -> BaglantiSonucu {
        b.blok = None;
        b.bekleme = None;
        b.zaman_asti = false;
        b.okuma_kapali = true;
        BaglantiSonucu::hata(tur, mesaj)
    }

    fn blok_kesintisi(b: &Baglanti, kapaniyor: bool) -> Option<(&'static str, &'static str)> {
        if kapaniyor {
            Some(("kapanıyor", "sunucu kapanıyor; blok atıldı"))
        } else if b.eof || b.kirik {
            Some(("kapandı", "karşı taraf blok bitmeden kapattı"))
        } else if b.zaman_asti {
            Some(("zaman_aşımı", "blok süresi içinde tamamlanmadı; atıldı"))
        } else {
            None
        }
    }

    /// Yeni blok: sınır denetimi, önceki taşınmamış dosyanın bırakılması
    /// (bağlantı başına en çok bir dosya) ve `0600` geçici dosya.
    fn blok_baslat(
        b: &mut Baglanti,
        kok: &std::path::Path,
        adet: Option<i64>,
        blok_siniri: u64,
    ) -> Result<(), BaglantiSonucu> {
        let adet = adet.map(|a| a.max(0) as u64);
        if adet.is_some_and(|a| a > blok_siniri) {
            return Err(BaglantiSonucu::hata("boyut_aşımı", BLOK_SINIRI_ASILDI));
        }
        b.blok_dosyasi = None;
        let yukleme = AkanYukleme::dizinde(kok, BAGLANTI_BLOK_DIZINI)
            .map_err(|neden| BaglantiSonucu::hata("dosya", neden))?;
        b.blok = Some(AcikBlok {
            okuyucu: adet.map_or_else(|| BlokOkuyucu::nokta(blok_siniri), BlokOkuyucu::bayt),
            yukleme,
            baslangic: Instant::now(),
            taban: adet.unwrap_or(0),
            alinan: 0,
        });
        b.bekleme = None;
        b.zaman_asti = false;
        Ok(())
    }

    /// Gelen tampondaki baytları çözücüye verir, çıktıyı dosyaya yazar.
    fn blok_tamponu_isle(b: &mut Baglanti, cikti: &mut Vec<u8>) -> Result<usize, String> {
        let Some(acik) = b.blok.as_mut() else {
            return Ok(0);
        };
        cikti.clear();
        let tuketilen = acik.okuyucu.besle(b.okuyucu.tampon(), cikti);
        b.okuyucu.tuket_bayt(tuketilen);
        acik.alinan += tuketilen as u64;
        if !cikti.is_empty() && acik.okuyucu.hata().is_none() {
            acik.yukleme.yaz(cikti)?;
        }
        Ok(tuketilen)
    }

    /// Biten blok: ret ya da `fsync` edilmiş dosyanın künyesi.
    fn blok_bitir(b: &mut Baglanti) -> BaglantiSonucu {
        b.bekleme = None;
        b.zaman_asti = false;
        let Some(acik) = b.blok.take() else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        if let Some(hata) = acik.okuyucu.hata() {
            return BaglantiSonucu::hata(hata.turu(), hata.mesaji());
        }
        match acik.yukleme.tamamla() {
            Ok((bilgi, koruma)) => {
                b.blok_dosyasi = Some(koruma);
                BaglantiSonucu::Blok {
                    yol: bilgi.goreli_yol,
                    sha256: bilgi.sha256,
                    bayt: bilgi.bayt as i64,
                }
            }
            Err(neden) => BaglantiSonucu::hata("dosya", neden),
        }
    }

    /// K-384: nokta bloğu (`adet` yok) ya da tam `adet` baytlık literal'i
    /// geçici dosyaya akıtır.
    fn blok_al(&mut self, kimlik: u64, adet: Option<i64>) -> BaglantiSonucu {
        let bosta = self.ayar.bosta_suresi_ms;
        let blok_siniri = self.ayar.blok_siniri;
        let kapaniyor = self.kapaniyor;
        let kok = self.kok.clone();
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        if b.okuma_kapali {
            return BaglantiSonucu::hata("kapandı", dil::ag_satiri::OKUMA_KAPALI);
        }
        if b.blok.is_none() {
            if kapaniyor {
                return BaglantiSonucu::hata("kapanıyor", "sunucu kapanıyor");
            }
            if let Err(ret) = Self::blok_baslat(b, &kok, adet, blok_siniri) {
                return ret;
            }
        }
        let mut islenen = 0usize;
        let mut cikti = Vec::with_capacity(OKUMA_PARCASI + 2);
        loop {
            match Self::blok_tamponu_isle(b, &mut cikti) {
                Ok(tuketilen) => islenen += tuketilen,
                Err(neden) => return Self::blok_kes(b, "dosya", &neden),
            }
            let Some(acik) = b.blok.as_ref() else {
                return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
            };
            if acik.okuyucu.bitti() {
                return Self::blok_bitir(b);
            }
            if let Some((tur, mesaj)) = Self::blok_kesintisi(b, kapaniyor) {
                return Self::blok_kes(b, tur, mesaj);
            }
            // Süre her turda çağrı başından yeniden hesaplanır; damla damla
            // gelen bayt hız tabanının altında bütçeyi uzatamaz.
            let son = acik.son(bosta);
            if islenen >= BLOK_DILIMI || !Self::doldur(b) {
                b.bekleme = Some(Bekleme {
                    ilgi: Ilgi::Okuma,
                    son,
                });
                if Instant::now() >= son {
                    b.zaman_asti = true;
                    continue;
                }
                return BaglantiSonucu::Bekle;
            }
        }
    }

    fn aktarim_ac(
        yol: &std::path::Path,
        veri: Option<&str>,
        (blok_siniri, bosta): (u64, i64),
        kodlu: Option<(u64, u64)>,
    ) -> Result<Aktarim, BaglantiSonucu> {
        // K-394: `veri:` yaprağı bağ izlenmeden, tek hard-link olarak açılır.
        let dosya = match veri {
            Some(veri_yolu) => dil::guvenlik::veri_koku::okuma_icin_ac(yol, veri_yolu)
                .map_err(|neden| BaglantiSonucu::hata("dosya", neden))?,
            None => std::fs::File::open(yol).map_err(|hata| {
                BaglantiSonucu::hata("dosya", format!("dosya açılamadı: {hata}"))
            })?,
        };
        let bilgi = match dosya.metadata() {
            Ok(bilgi) if bilgi.is_file() => bilgi,
            _ => return Err(BaglantiSonucu::hata("dosya", "normal dosya değil")),
        };
        if bilgi.len() > blok_siniri {
            return Err(BaglantiSonucu::hata("boyut_aşımı", BLOK_SINIRI_ASILDI));
        }
        if kodlu.is_some() && bilgi.len() > dil::ag_satiri::BASE64_KAYNAK_TAVANI {
            return Err(BaglantiSonucu::hata(
                "boyut_aşımı",
                dil::ag_satiri::BASE64_KAYNAK_TAVANI_ASILDI,
            ));
        }
        let mut aktarim = Aktarim {
            dosya,
            kalan: bilgi.len(),
            toplam: bilgi.len(),
            son: Instant::now(),
            kodlu: None,
        };
        if let Some((konum, adet)) = kodlu {
            Self::kodlu_pencereyi_ac(&mut aktarim, konum, adet)?;
        }
        // Süre bütçesi gerçekten yazılacak bayta göre (K-372 hız tabanı).
        let butce = akis_butcesi_ms(bosta, aktarim.toplam);
        aktarim.son += Duration::from_millis(butce.max(0) as u64);
        Ok(aktarim)
    }

    /// K-396: kodlanmış çıktı penceresinin ilk satırına konumlanır.
    fn kodlu_pencereyi_ac(
        aktarim: &mut Aktarim,
        konum: u64,
        adet: u64,
    ) -> Result<(), BaglantiSonucu> {
        use std::io::{Seek, SeekFrom};
        let pencere = dil::kodlama::base64_satir_penceresi(aktarim.kalan, konum, adet);
        aktarim
            .dosya
            .seek(SeekFrom::Start(pencere.ham_baslangic))
            .map_err(|hata| {
                BaglantiSonucu::hata("dosya", format!("dosya konumlanamadı: {hata}"))
            })?;
        // K-414: pencere ham konumu dosya içinde kalır; ikinci savunma hattı
        // olarak çıkarma doymalıdır (istemci aralığı sarma üretemez).
        aktarim.kalan = aktarim.kalan.saturating_sub(pencere.ham_baslangic);
        aktarim.toplam = pencere.uzunluk;
        aktarim.kodlu = Some(KodluAktarim {
            bekleyen: Vec::new(),
            bas: 0,
            atla: pencere.atla,
            cikti_kalan: pencere.uzunluk,
        });
        Ok(())
    }

    /// K-396: giden tampona en çok `butce` kodlanmış bayt; (eklenen, kısaldı).
    fn kodlu_doldur(
        dosya: &mut std::fs::File,
        kalan: &mut u64,
        k: &mut KodluAktarim,
        giden: &mut Vec<u8>,
        butce: usize,
    ) -> (usize, bool) {
        let mut eklenen = 0usize;
        while k.cikti_kalan > 0 && giden.len() < GIDEN_TAMPON_BAYTI && eklenen < butce {
            if k.bas == k.bekleyen.len() {
                if !Self::kodlu_parca_oku(dosya, kalan, k) {
                    return (eklenen, true);
                }
                continue;
            }
            let n = (k.bekleyen.len() - k.bas)
                .min(GIDEN_TAMPON_BAYTI - giden.len())
                .min(usize::try_from(k.cikti_kalan).unwrap_or(usize::MAX));
            giden.extend_from_slice(&k.bekleyen[k.bas..k.bas + n]);
            k.bas += n;
            k.cikti_kalan -= n as u64;
            eklenen += n;
        }
        (eklenen, false)
    }

    /// Sıradaki ham parçayı okuyup satırlı kodlar; dosya kısaldıysa `false`.
    fn kodlu_parca_oku(dosya: &mut std::fs::File, kalan: &mut u64, k: &mut KodluAktarim) -> bool {
        let istek = KODLU_HAM_PARCA.min(*kalan) as usize;
        let mut ham = vec![0u8; istek];
        if istek == 0 || dosya.read_exact(&mut ham).is_err() {
            return false;
        }
        *kalan -= istek as u64;
        k.bekleyen.clear();
        dil::kodlama::base64_satirlari_kodla(&ham, &mut k.bekleyen);
        let atla = usize::try_from(k.atla)
            .unwrap_or(usize::MAX)
            .min(k.bekleyen.len());
        k.bas = atla;
        k.atla -= atla as u64;
        true
    }

    /// Dosyadan giden tampona en çok `butce` bayt; (eklenen, dosya kısaldı).
    fn aktarim_doldur(aktarim: &mut Aktarim, giden: &mut Vec<u8>, butce: usize) -> (usize, bool) {
        if let Some(k) = aktarim.kodlu.as_mut() {
            return Self::kodlu_doldur(&mut aktarim.dosya, &mut aktarim.kalan, k, giden, butce);
        }
        let mut parca = [0u8; 16 * 1024];
        let mut eklenen = 0usize;
        while aktarim.kalan > 0 && giden.len() < GIDEN_TAMPON_BAYTI && eklenen < butce {
            let alan = (GIDEN_TAMPON_BAYTI - giden.len())
                .min(parca.len())
                .min(aktarim.kalan as usize);
            match aktarim.dosya.read(&mut parca[..alan]) {
                Ok(n) if n > 0 => {
                    giden.extend_from_slice(&parca[..n]);
                    aktarim.kalan -= n as u64;
                    eklenen += n;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                _ => return (eklenen, true),
            }
        }
        (eklenen, false)
    }

    /// Bildirilen uzunluk tutmaz: bağlantı eşzamanı bozuldu, kırık sayılır.
    fn aktarim_kes(b: &mut Baglanti, neden: &str) -> BaglantiSonucu {
        b.aktarim = None;
        b.bekleme = None;
        b.zaman_asti = false;
        b.kirik = true;
        BaglantiSonucu::hata("yazılamadı", neden)
    }

    /// K-384: çözülmüş yoldaki dosyayı giden tampona geri basınçla akıtır;
    /// K-396: `kodlu` verilirse dosyanın satırlı base64 çıktısının
    /// (konum, adet) penceresini akıtır.
    pub(crate) fn dosya_aktar(
        &mut self,
        kimlik: u64,
        yol: &std::path::Path,
        veri: Option<&str>,
        kodlu: Option<(u64, u64)>,
    ) -> BaglantiSonucu {
        let bosta = self.ayar.bosta_suresi_ms;
        let blok_siniri = self.ayar.blok_siniri;
        let Some(b) = self.baglantilar.get_mut(&kimlik) else {
            return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
        };
        if b.kirik {
            b.aktarim = None;
            return BaglantiSonucu::hata("yazılamadı", "karşı taraf bağlantıyı kapattı");
        }
        if b.aktarim.is_none() {
            match Self::aktarim_ac(yol, veri, (blok_siniri, bosta), kodlu) {
                Ok(aktarim) => b.aktarim = Some(aktarim),
                Err(ret) => return ret,
            }
            b.bekleme = None;
            b.zaman_asti = false;
        }
        let mut islenen = 0usize;
        loop {
            let Some(aktarim) = b.aktarim.as_mut() else {
                return BaglantiSonucu::hata("kapandı", "bağlantı kapatıldı");
            };
            let (eklenen, kisaldi) =
                Self::aktarim_doldur(aktarim, &mut b.giden, BLOK_DILIMI.saturating_sub(islenen));
            islenen += eklenen;
            let (bitti, toplam, son) = (aktarim.bitti(), aktarim.toplam, aktarim.son);
            if !bosalt(&mut b.kanal, &mut b.giden) {
                return Self::aktarim_kes(b, "karşı taraf bağlantıyı kapattı");
            }
            if bitti {
                b.aktarim = None;
                b.bekleme = None;
                b.zaman_asti = false;
                return BaglantiSonucu::Tamam(toplam as i64);
            }
            if kisaldi {
                return Self::aktarim_kes(b, "dosya aktarım sırasında kısaldı");
            }
            if b.zaman_asti {
                return Self::aktarim_kes(b, "dosya süresi içinde aktarılamadı");
            }
            if Instant::now() >= son {
                b.zaman_asti = true;
                continue;
            }
            if islenen >= BLOK_DILIMI || b.giden.len() >= GIDEN_TAMPON_BAYTI {
                // Dilim bitti ya da tampon dolu: görev scheduler'a döner,
                // yazmaya hazır sokette sürer.
                b.bekleme = Some(Bekleme {
                    ilgi: Ilgi::Yazma,
                    son,
                });
                return BaglantiSonucu::Bekle;
            }
        }
    }

    pub(crate) fn islem(&mut self, kimlik: u64, islem: &BaglantiIslemi) -> BaglantiSonucu {
        match islem {
            BaglantiIslemi::SatirOku => self.satir_oku(kimlik, self.ayar.bosta_suresi_ms),
            BaglantiIslemi::SatirOkuSureli(ms) => self.satir_oku(kimlik, *ms),
            BaglantiIslemi::SatirYaz(metin) => self.satir_yaz(kimlik, metin, true),
            BaglantiIslemi::ParcaYaz(metin) => self.satir_yaz(kimlik, metin, false),
            BaglantiIslemi::NoktaBlogu => self.blok_al(kimlik, None),
            BaglantiIslemi::BaytBlogu(adet) => self.blok_al(kimlik, Some(*adet)),
            BaglantiIslemi::BaytOku(adet) => self.bayt_oku(kimlik, *adet),
            // Yol çözümü ve `dosya-okuma` denetimi adaptörde (main.rs) yapılır.
            BaglantiIslemi::DosyaAktar(_) | BaglantiIslemi::Base64Aktar { .. } => {
                BaglantiSonucu::hata("dosya", "dosya yolu adaptörde çözülmedi")
            }
            BaglantiIslemi::Kapat => self.kapat(kimlik),
            BaglantiIslemi::TlsBaslat => self.tls_baslat(kimlik, None),
            BaglantiIslemi::TlsIstemci(ad) => self.tls_baslat(kimlik, Some(ad)),
            BaglantiIslemi::Engelle(son) => self.engelle(kimlik, *son),
            // Politika gerektirir; main.rs `giden_baglan` ile yönlendirir.
            BaglantiIslemi::GidenBaglan(..) => {
                BaglantiSonucu::hata("hedef_yasak", "giden bağlantı politikasız açılamaz")
            }
        }
    }

    /// K-397: kapatılan giden bağlantıların tamponlarını ilerletir (giden
    /// yalnız programda dinleyici beklemesi hiç koşmaz).
    fn kapananlari_ilerlet(&mut self) {
        let simdi = Instant::now();
        self.kapananlar.retain_mut(|k| {
            bosalt(&mut k.kanal, &mut k.giden)
                && bekleyen_cikti(&k.kanal, &k.giden)
                && simdi < k.son
        });
    }

    /// K-397: `"<ana>:<kapı>"` giden hedefine bağlanır. Politika ve IP
    /// denetimi tek çözümlemede; soket yalnız denetlenen adrese açılır.
    pub(crate) fn giden_baglan(
        &mut self,
        politika: &dil::yetkinlik::YetkinlikPolitikasi,
        adres: &str,
        son_ms: Option<i64>,
    ) -> BaglantiSonucu {
        self.kapananlari_ilerlet();
        let acik = self.baglantilar.values().filter(|b| b.giden_mi).count();
        if acik >= dil::ag_satiri::AZAMI_GIDEN_BAGLANTI {
            return BaglantiSonucu::hata("sınır", "süreç başına giden bağlantı sınırı dolu");
        }
        let hedef = match politika.giden_tcp_hedefini_coz(adres, |ad, kapi| {
            use std::net::ToSocketAddrs;
            (ad, kapi).to_socket_addrs().map(Iterator::collect)
        }) {
            Ok(hedef) => hedef,
            Err(hata) => return BaglantiSonucu::hata(hata.tur, hata.mesaj),
        };
        let mut sure = Duration::from_millis(dil::ag_satiri::GIDEN_BAGLANMA_MS as u64);
        if let Some(son) = son_ms {
            let kalan = self.an(son).saturating_duration_since(Instant::now());
            if kalan.is_zero() {
                return BaglantiSonucu::hata("zaman_aşımı", "son tarih bağlanmadan doldu");
            }
            sure = sure.min(kalan);
        }
        let akis = match TcpStream::connect_timeout(&hedef, sure) {
            Ok(akis) => akis,
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                return BaglantiSonucu::hata(
                    "zaman_aşımı",
                    format!("{hedef} bağlanma süresi doldu"),
                )
            }
            Err(_) => {
                return BaglantiSonucu::hata(
                    "bağlanamadı",
                    format!("{hedef} bağlantıyı kabul etmedi"),
                )
            }
        };
        if akis.set_nonblocking(true).is_err() {
            return BaglantiSonucu::hata("bağlanamadı", "soket engellemeyen kipe alınamadı");
        }
        let _ = akis.set_nodelay(true);
        let yerel_kapi = akis.local_addr().map(|a| a.port()).unwrap_or(0);
        let kimlik = self.sonraki;
        self.sonraki += 1;
        self.baglantilar.insert(
            kimlik,
            Baglanti {
                kanal: Kanal::duz(akis),
                dinleyici: "giden".into(),
                uzak: hedef.ip(),
                okuyucu: SatirOkuyucu::yeni(self.ayar.satir_siniri),
                giden: Vec::new(),
                giden_mi: true,
                eof: false,
                kirik: false,
                bekleme: None,
                zaman_asti: false,
                el_sikisma: None,
                yukseltiliyor: false,
                blok: None,
                blok_dosyasi: None,
                okuma_kapali: false,
                aktarim: None,
            },
        );
        BaglantiSonucu::Giden(YeniBaglanti {
            kimlik,
            dinleyici: String::new(),
            uzak_adres: uzak_yazimi(hedef.ip()),
            uzak_kapi: hedef.port(),
            yerel_kapi,
            tls: false,
        })
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

/// K-397: STARTTLS'in yön (sunucu/istemci) ve kimlik ön koşulları.
fn tls_on_kosulu(b: &Baglanti, istemci: Option<&str>) -> Option<BaglantiSonucu> {
    if b.kanal.tls_mi() {
        return Some(BaglantiSonucu::hata("tls", dil::ag_satiri::ZATEN_TLS));
    }
    let ret = match istemci {
        Some(_) if !b.giden_mi => BaglantiSonucu::hata(
            "geçersiz",
            "TLS istemcisi yalnız giden bağlantıda başlatılır",
        ),
        Some(ad) if !dil::ag_satiri::tls_sunucu_adi_gecerli(ad) => {
            BaglantiSonucu::hata("geçersiz", "TLS sunucu adı ASCII DNS adı olmalı")
        }
        None if b.giden_mi => BaglantiSonucu::hata(
            "geçersiz",
            "giden bağlantıda `bağlantıda TLS istemcisi başlatmayı dene \"<ad>\" ile` kullan",
        ),
        None if super::ag_tls::yapilandirma().is_none() => {
            BaglantiSonucu::hata("tls", dil::ag_satiri::TLS_KIMLIGI_YOK)
        }
        _ => return None,
    };
    Some(ret)
}
