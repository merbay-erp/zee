//! K-336 resmî çok işçi web profili (W2) ve K-337 açılış birincilliği (W4).
//!
//! Private CLI adapter: `dil çalıştır --web-proxy … --web-worker-port P
//! --web-isci-sayisi N` yönetici süreci tek loopback dinleyiciyi bağlar ve
//! N işçi sürecine aynı açık dinleyiciyi devreder (pre-fork; SO_REUSEPORT
//! ya da proxy aktarımı yok). Yalnız Unix: Windows'ta açık sınır. Yönetici Zee
//! programı yürütmez; işçiler bugünkü tek süreç web çalışma zamanının aynısıdır.
//! İlk açılışta yalnız birincil işçi `birincil süreç` için doğru görür ve
//! ikincil işçiler birincil `sunucu başlat`a ulaşmadan başlatılmaz.

use std::process::ExitCode;

pub(crate) const ISCI_BAYRAGI: &str = "--web-isci-sayisi";
pub(crate) const SAGLIK_BAYRAGI: &str = "--web-saglik-port";
pub(crate) const AZAMI_ISCI: u16 = 64;

const ORTAM_DINLEYICI: &str = "ZEE_WEB_ISCI_DINLEYICI_FD";
const ORTAM_HAZIR: &str = "ZEE_WEB_ISCI_HAZIR_FD";
const ORTAM_ROL: &str = "ZEE_WEB_ISCI_ROLU";
const ORTAM_EBEVEYN: &str = "ZEE_WEB_ISCI_EBEVEYN";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ayar {
    pub isci: u16,
    pub saglik_kapisi: Option<u16>,
}

/// Kaynaktan önceki seçenek bölgesinden iki bayrağı çıkarır. Kaynak sonrası
/// aynı sözcükler uygulama argv verisidir ve dokunulmaz.
pub(crate) fn ayikla(args: &mut Vec<String>) -> Result<Option<Ayar>, String> {
    let mut isci: Option<String> = None;
    let mut saglik: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == ISCI_BAYRAGI || arg == SAGLIK_BAYRAGI {
            let hedef = if arg == ISCI_BAYRAGI {
                &mut isci
            } else {
                &mut saglik
            };
            if hedef.is_some() {
                return Err(format!("{arg} birden çok kez verilemez"));
            }
            let deger = args
                .get(i + 1)
                .ok_or_else(|| format!("{arg} ardından değer ister"))?
                .clone();
            *hedef = Some(deger);
            args.drain(i..i + 2);
            continue;
        }
        if !arg.starts_with('-') {
            break;
        }
        i += if super::web_listener::option_has_value(arg) {
            2
        } else {
            1
        };
    }
    let Some(isci) = isci else {
        if saglik.is_some() {
            return Err(format!(
                "{SAGLIK_BAYRAGI} yalnız {ISCI_BAYRAGI} ile kullanılabilir"
            ));
        }
        return Ok(None);
    };
    let isci = isci
        .parse::<u16>()
        .ok()
        .filter(|n| (1..=AZAMI_ISCI).contains(n))
        .ok_or_else(|| format!("{ISCI_BAYRAGI} 1..{AZAMI_ISCI} arasında olmalı"))?;
    let saglik_kapisi = saglik
        .map(|deger| {
            deger
                .parse::<u16>()
                .ok()
                .filter(|kapi| *kapi != 0)
                .ok_or_else(|| format!("{SAGLIK_BAYRAGI} 1..65535 arasında olmalı"))
        })
        .transpose()?;
    Ok(Some(Ayar {
        isci,
        saglik_kapisi,
    }))
}

/// `govde` başında: önce yönetici devri ortamını (varsa) alır ve siler, sonra
/// seçenekleri ayıklar. İşçi süreci yönetici değildir: yönetici çocuk
/// argv'sinden kendi bayraklarını çıkardığından işçide görülen işçi
/// seçeneği yalnız yeniden okunan `zee.ayar`dan (K-340) gelir ve atlanır.
pub(crate) fn hazirla(args: &mut Vec<String>) -> Result<Option<Ayar>, String> {
    let cocuk = cocuk_ortamini_al()?;
    let ayar = ayikla(args)?;
    Ok(if cocuk { None } else { ayar })
}

/// Yönetici çocuğa yalnız bu ortam kayıtlarıyla rol ve descriptor verir.
/// İşçi bunları ilk iş olarak okur, ortamdan siler ve descriptor'ları
/// FD_CLOEXEC yapar; Zee programının açtığı alt süreçlere sızmaz.
#[derive(Debug, Default)]
struct CocukDurumu {
    rol: Option<Rol>,
    dinleyici: Option<i32>,
    #[cfg_attr(not(unix), allow(dead_code))]
    hazir: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rol {
    Birincil,
    Ikincil,
}

static COCUK: std::sync::Mutex<CocukDurumu> = std::sync::Mutex::new(CocukDurumu {
    rol: None,
    dinleyici: None,
    hazir: None,
});
static EBEVEYN: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

fn cocuk() -> std::sync::MutexGuard<'static, CocukDurumu> {
    COCUK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// `govde` başında bir kez çağrılır. Yönetici dışında (ortam yoksa) hiçbir şey
/// yapmaz; tek süreç davranışı bugünküyle aynıdır. Grup işçisiyse `true`.
fn cocuk_ortamini_al() -> Result<bool, String> {
    let rol = std::env::var_os(ORTAM_ROL);
    let dinleyici = std::env::var_os(ORTAM_DINLEYICI);
    let hazir = std::env::var_os(ORTAM_HAZIR);
    let ebeveyn = std::env::var_os(ORTAM_EBEVEYN);
    for ad in [ORTAM_ROL, ORTAM_DINLEYICI, ORTAM_HAZIR, ORTAM_EBEVEYN] {
        std::env::remove_var(ad);
    }
    if rol.is_none() && dinleyici.is_none() && hazir.is_none() && ebeveyn.is_none() {
        return Ok(false);
    }
    let ret = || "web işçi grubu ortamı geçersiz".to_string();
    let rol = match rol.as_ref().and_then(|r| r.to_str()) {
        Some("birincil") => Rol::Birincil,
        Some("ikincil") => Rol::Ikincil,
        _ => return Err(ret()),
    };
    let fd = |deger: Option<&std::ffi::OsStr>| -> Result<Option<i32>, String> {
        deger
            .map(|d| {
                d.to_str()
                    .and_then(|d| d.parse::<i32>().ok())
                    .filter(|fd| *fd > 2)
                    .ok_or_else(ret)
            })
            .transpose()
    };
    let dinleyici = fd(dinleyici.as_deref())?.ok_or_else(ret)?;
    let hazir = fd(hazir.as_deref())?;
    if (rol == Rol::Birincil) != hazir.is_some() || hazir == Some(dinleyici) {
        return Err(ret());
    }
    let ebeveyn = ebeveyn
        .as_ref()
        .and_then(|e| e.to_str())
        .and_then(|e| e.parse::<i64>().ok())
        .filter(|pid| *pid > 1)
        .ok_or_else(ret)?;
    #[cfg(unix)]
    {
        for fd in std::iter::once(dinleyici).chain(hazir) {
            // SAFETY: flags mutation on a borrowed descriptor; ownership moves later.
            let onceki = unsafe { libc::fcntl(fd, libc::F_GETFD) };
            if onceki < 0
                || unsafe { libc::fcntl(fd, libc::F_SETFD, onceki | libc::FD_CLOEXEC) } != 0
            {
                return Err(ret());
            }
        }
        EBEVEYN.store(ebeveyn, std::sync::atomic::Ordering::Relaxed);
    }
    #[cfg(not(unix))]
    {
        let _ = ebeveyn;
        return Err("web işçi grubu yalnız Unix'te desteklenir".into());
    }
    #[allow(unreachable_code)]
    {
        *cocuk() = CocukDurumu {
            rol: Some(rol),
            dinleyici: Some(dinleyici),
            hazir,
        };
        Ok(true)
    }
}

/// W4 (K-337): tek süreçte ve grubun ilk birincil işçisinde doğru; ikincil
/// ve yeniden başlatılan işçilerde yanlış.
///
/// K-341: `--web-worker-port` süreci (grup işçisi ya da elle kurulan ayrı
/// portlu süreç) ayrıca `kok/.zee` altındaki süreç kilitleriyle karar verir;
/// aynı kökte canlı başka katılımcı varken kimse birincil olamaz. `kok` yoksa
/// (tek süreç, `--deneysel-web`) davranış K-337 ile aynıdır.
pub(crate) fn birincil_mi(kok: Option<&std::path::Path>) -> bool {
    let rol_birincil = cocuk().rol != Some(Rol::Ikincil);
    let Some(kok) = kok else {
        return rol_birincil;
    };
    let mut kilit = kilit_durumu();
    if let Some(karar) = kilit.karar {
        return karar;
    }
    let karar = match birincilligi_kilitle(kok, rol_birincil) {
        Ok((karar, acilis, canli)) => {
            // `sunucu başlat` sonrası ilk kez sorulduysa açılış zaten bitti.
            kilit.acilis = acilis.filter(|_| !kilit.hazir);
            kilit.canli = Some(canli);
            karar
        }
        Err(hata) => {
            // Belirlenemeyen birincillik açılış işini iki kez koşturabilir;
            // güvenli yön ikincil kalmaktır.
            eprintln!(
                "zee-web birincillik belirlenemedi (.zee süreç kilidi: {hata}); süreç ikincil sayılır"
            );
            false
        }
    };
    kilit.karar = Some(karar);
    karar
}

/// K-341 süreç kilitleri. `canli`: her katılımcı ömrü boyunca paylaşımlı
/// tutar. `acilis`: kararı sıraya koyar; birincil `sunucu başlat`a dek tutar.
#[derive(Debug, Default)]
struct KilitDurumu {
    karar: Option<bool>,
    hazir: bool,
    acilis: Option<std::fs::File>,
    canli: Option<std::fs::File>,
}

static KILIT: std::sync::Mutex<KilitDurumu> = std::sync::Mutex::new(KilitDurumu {
    karar: None,
    hazir: false,
    acilis: None,
    canli: None,
});

fn kilit_durumu() -> std::sync::MutexGuard<'static, KilitDurumu> {
    KILIT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

const ACILIS_KILIDI: &str = "birincil-acilis.kilit";
const CANLI_KILIDI: &str = "birincil-canli.kilit";

type KilitKarari = (bool, Option<std::fs::File>, std::fs::File);

/// Açılış kilidi altında: canlı kilit dışlayıcı alınabiliyorsa kökte başka
/// katılımcı yoktur. Her katılımcı canlı kilidi yalnız açılış kilidini
/// tutarken sınadığından "yalnızım" kararı ile paylaşımlı kayıt arasına
/// başka süreç giremez. İkincil açılış kilidini hemen bırakır; birincil
/// `hazir_bildir`e dek tutar, böylece sonra gelen süreç birincilin açılış
/// işi bitmeden istek kabul etmez.
fn birincilligi_kilitle(kok: &std::path::Path, rol_birincil: bool) -> std::io::Result<KilitKarari> {
    let dizin = kok.join(".zee");
    std::fs::create_dir_all(&dizin)?;
    let acilis = kilit_dosyasi(&dizin.join(ACILIS_KILIDI))?;
    let canli = kilit_dosyasi(&dizin.join(CANLI_KILIDI))?;
    match acilis.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => {
            eprintln!(
                "zee-web aynı kökte birincil sürecin açılışı bekleniyor (.zee/{ACILIS_KILIDI})"
            );
            acilis.lock()?;
        }
        Err(std::fs::TryLockError::Error(hata)) => return Err(hata),
    }
    let yalniz = match canli.try_lock() {
        Ok(()) => {
            canli.unlock()?;
            true
        }
        Err(std::fs::TryLockError::WouldBlock) => false,
        Err(std::fs::TryLockError::Error(hata)) => return Err(hata),
    };
    canli.lock_shared()?;
    let karar = yalniz && rol_birincil;
    Ok((karar, karar.then_some(acilis), canli))
}

fn kilit_dosyasi(yol: &std::path::Path) -> std::io::Result<std::fs::File> {
    if std::fs::symlink_metadata(yol).is_ok_and(|m| !m.is_file()) {
        return Err(std::io::Error::other("kilit normal dosya olmalı"));
    }
    let mut secenek = std::fs::OpenOptions::new();
    secenek.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        secenek.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    secenek.open(yol)
}

/// Devralınan dinleyici descriptor'ı bir kez verilir.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn devralinan_dinleyici() -> Option<i32> {
    cocuk().dinleyici.take()
}

/// Birincil işçi `sunucu başlat`a ulaştı: yönetici ikincilleri başlatabilir.
pub(crate) fn hazir_bildir() {
    // K-341: birincil açılış işi bitti; bekleyen ayrı portlu süreçler sürer.
    let mut kilit = kilit_durumu();
    kilit.hazir = true;
    drop(kilit.acilis.take());
    drop(kilit);
    #[cfg(unix)]
    if let Some(fd) = cocuk().hazir.take() {
        use std::io::Write;
        use std::os::fd::FromRawFd;
        // SAFETY: validated inherited write end, transferred exactly once.
        let mut kanal = unsafe { std::fs::File::from_raw_fd(fd) };
        let _ = kanal.write_all(b"h");
    }
}

/// Yönetici kaybolduysa (SIGKILL) işçi yeni istek kabul etmeyi bırakır;
/// sahipsiz işçi paylaşılan dinleyiciyi tutmaya devam etmez.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn ebeveyn_kayboldu() -> bool {
    let beklenen = EBEVEYN.load(std::sync::atomic::Ordering::Relaxed);
    if beklenen == 0 {
        return false;
    }
    #[cfg(unix)]
    {
        // SAFETY: getppid has no preconditions.
        i64::from(unsafe { libc::getppid() }) != beklenen
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(not(unix))]
pub(crate) fn yonet(_: Ayar, _: u16, _: Vec<String>) -> ExitCode {
    eprintln!(
        "{ISCI_BAYRAGI} Unix (Linux/macOS) ister: Windows'ta paylaşılan dinleyici devri yok. \
         Her işçiyi ayrı --web-worker-port ile başlatıp önlerindeki proxy'de dengeleyin."
    );
    ExitCode::from(2)
}

#[cfg(unix)]
pub(crate) use unix::yonet;

#[cfg(unix)]
mod unix {
    use super::*;
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::process::{Child, Command};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    static SINYAL: AtomicBool = AtomicBool::new(false);
    extern "C" fn sinyal(_: libc::c_int) {
        SINYAL.store(true, Ordering::Relaxed);
    }
    const ADIM: Duration = Duration::from_millis(100);
    /// 30 sn istek son tarihi + 10 sn yanıt yazımı payı.
    const KAPANIS_SURESI: Duration = Duration::from_secs(40);
    const HIZLI_COKUS: Duration = Duration::from_secs(10);
    const AZAMI_ARDISIK_COKUS: u32 = 10;

    #[derive(Default)]
    struct Saglik {
        hazir: bool,
        kapaniyor: bool,
        canli: Vec<u32>,
        yeniden_baslatma: u64,
    }

    struct Yuva {
        cocuk: Option<Child>,
        baslangic: Instant,
        ardisik: u32,
        sonraki: Option<Instant>,
    }

    fn sinyalleri_kur() -> Result<(), String> {
        for sig in [libc::SIGTERM, libc::SIGINT] {
            // SAFETY: zeroed sigaction is valid; handler only stores an atomic.
            let mut yeni: libc::sigaction = unsafe { std::mem::zeroed() };
            yeni.sa_sigaction = sinyal as *const () as libc::sighandler_t;
            yeni.sa_flags = libc::SA_RESTART;
            let sonuc = unsafe {
                libc::sigemptyset(&mut yeni.sa_mask);
                libc::sigaction(sig, &yeni, std::ptr::null_mut())
            };
            if sonuc != 0 {
                return Err("web işçi grubu sinyal yönetimi kurulamadı".into());
            }
        }
        Ok(())
    }

    fn kapanis(cocuklar: &mut [Yuva], saglik: &Mutex<Saglik>) -> ExitCode {
        if let Ok(mut s) = saglik.lock() {
            s.kapaniyor = true;
        }
        for yuva in cocuklar.iter_mut() {
            if let Some(c) = &yuva.cocuk {
                // SAFETY: signal to our own unreaped child PID.
                unsafe { libc::kill(c.id() as libc::pid_t, libc::SIGTERM) };
            }
        }
        let son = Instant::now() + KAPANIS_SURESI;
        loop {
            let mut kalan = 0;
            for yuva in cocuklar.iter_mut() {
                if let Some(c) = &mut yuva.cocuk {
                    match c.try_wait() {
                        Ok(Some(_)) | Err(_) => yuva.cocuk = None,
                        Ok(None) => kalan += 1,
                    }
                }
            }
            if kalan == 0 {
                return ExitCode::SUCCESS;
            }
            if Instant::now() >= son {
                for yuva in cocuklar.iter_mut() {
                    if let Some(mut c) = yuva.cocuk.take() {
                        let _ = c.kill();
                        let _ = c.wait();
                    }
                }
                eprintln!("zee-web işçi grubu: kapanış süresi aşıldı; kalan işçiler durduruldu");
                return ExitCode::FAILURE;
            }
            std::thread::sleep(ADIM);
        }
    }

    fn baslat(
        args: &[String],
        dinleyici: i32,
        hazir: Option<i32>,
        sira: usize,
    ) -> std::io::Result<Child> {
        let exe = std::env::current_exe()?;
        let mut komut = Command::new(exe);
        komut
            .args(args)
            .env(ORTAM_DINLEYICI, dinleyici.to_string())
            .env(
                ORTAM_ROL,
                if hazir.is_some() {
                    "birincil"
                } else {
                    "ikincil"
                },
            )
            .env(ORTAM_EBEVEYN, std::process::id().to_string());
        if let Some(fd) = hazir {
            komut.env(ORTAM_HAZIR, fd.to_string());
        }
        let cocuk = komut.spawn()?;
        eprintln!(
            "zee-web işçi {} başladı: pid={} rol={}",
            sira + 1,
            cocuk.id(),
            if hazir.is_some() {
                "birincil"
            } else {
                "ikincil"
            }
        );
        Ok(cocuk)
    }

    fn saglik_sunucusu(kapi: u16, isci: u16, saglik: Arc<Mutex<Saglik>>) -> Result<(), String> {
        let dinleyici = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, kapi))
            .map_err(|e| format!("sağlık portu açılamadı: {e}"))?;
        std::thread::Builder::new()
            .name("zee-web-saglik".into())
            .spawn(move || {
                for akis in dinleyici.incoming().flatten() {
                    saglik_yaniti(akis, isci, &saglik);
                }
            })
            .map_err(|e| format!("sağlık iş parçacığı açılamadı: {e}"))?;
        Ok(())
    }

    fn saglik_yaniti(mut akis: std::net::TcpStream, isci: u16, saglik: &Mutex<Saglik>) {
        let _ = akis.set_read_timeout(Some(Duration::from_secs(2)));
        let _ = akis.set_write_timeout(Some(Duration::from_secs(2)));
        let mut tampon = [0u8; 1024];
        let mut dolu = 0;
        while dolu < tampon.len() {
            match akis.read(&mut tampon[dolu..]) {
                Ok(0) | Err(_) => break,
                Ok(n) => dolu += n,
            }
            if tampon[..dolu].windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
        let satir = tampon[..dolu]
            .split(|b| *b == b'\r' || *b == b'\n')
            .next()
            .unwrap_or_default();
        let head = satir.starts_with(b"HEAD ");
        let (kod, govde) = if satir == b"GET /saglik HTTP/1.1"
            || satir == b"GET /saglik HTTP/1.0"
            || satir == b"HEAD /saglik HTTP/1.1"
            || satir == b"HEAD /saglik HTTP/1.0"
        {
            let s = saglik
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let canli = s.canli.len();
            let durum = if s.kapaniyor {
                "kapaniyor"
            } else if !s.hazir {
                "basliyor"
            } else if canli == usize::from(isci) {
                "hazir"
            } else {
                "eksik"
            };
            let pidler = s
                .canli
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",");
            (
                if durum == "hazir" { 200 } else { 503 },
                format!(
                    "{{\"durum\":\"{durum}\",\"isci\":{isci},\"canli\":{canli},\"yeniden_baslatma\":{},\"surecler\":[{pidler}]}}\n",
                    s.yeniden_baslatma
                ),
            )
        } else {
            (404, "{\"durum\":\"bulunamadi\"}\n".to_string())
        };
        let neden = match kod {
            200 => "OK",
            503 => "Service Unavailable",
            _ => "Not Found",
        };
        let mut yanit = format!(
            "HTTP/1.1 {kod} {neden}\r\nContent-Type: application/json; charset=utf-8\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            govde.len()
        );
        if !head {
            yanit.push_str(&govde);
        }
        let _ = akis.write_all(yanit.as_bytes());
    }

    fn saglik_guncelle(saglik: &Mutex<Saglik>, yuvalar: &[Yuva]) {
        if let Ok(mut s) = saglik.lock() {
            s.canli = yuvalar
                .iter()
                .filter_map(|y| y.cocuk.as_ref().map(Child::id))
                .collect();
        }
    }

    /// Birincilin hazır baytını bekler. `Ok(true)`: hazır; `Ok(false)`:
    /// sinyal geldi; `Err(kod)`: birincil hazır olmadan çıktı.
    fn birincili_bekle(okuma: &mut std::fs::File, cocuk: &mut Child) -> Result<bool, ExitCode> {
        loop {
            if SINYAL.load(Ordering::Relaxed) {
                return Ok(false);
            }
            let mut olay = libc::pollfd {
                fd: okuma.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: one live pollfd for the synchronous call.
            let sonuc = unsafe { libc::poll(&mut olay, 1, 100) };
            if sonuc > 0 {
                let mut bayt = [0u8; 1];
                match okuma.read(&mut bayt) {
                    Ok(1) if bayt[0] == b'h' => return Ok(true),
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    _ => {
                        // EOF: birincil kanalı hazır demeden kapattı (çıktı).
                        let durum = cocuk.wait().ok();
                        eprintln!(
                            "zee-web işçi grubu: birincil işçi sunucu başlatmadan çıktı ({})",
                            durum.map_or_else(|| "bilinmiyor".into(), |d| d.to_string())
                        );
                        return Err(durum
                            .and_then(|d| d.code())
                            .filter(|k| *k != 0)
                            .map_or(ExitCode::FAILURE, |k| ExitCode::from(k as u8)));
                    }
                }
            }
        }
    }

    fn yeni_yuva(cocuk: Option<Child>) -> Yuva {
        Yuva {
            sonraki: cocuk
                .is_none()
                .then(|| Instant::now() + Duration::from_secs(1)),
            cocuk,
            baslangic: Instant::now(),
            ardisik: 0,
        }
    }

    /// Ortak dinleyiciyi bağlar ve miras kopyasını döndürür.
    fn kur(ayar: Ayar, kapi: u16, saglik: &Arc<Mutex<Saglik>>) -> Result<i32, String> {
        sinyalleri_kur()?;
        let dinleyici = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, kapi))
            .map_err(|hata| format!("web işçi grubu 127.0.0.1:{kapi} dinleyemedi: {hata}"))?;
        // dup(2) FD_CLOEXEC taşımaz: yalnız bu kopya işçilere miras kalır.
        // SAFETY: duplicate of a live owned listener; it stays open until exit.
        let miras = unsafe { libc::dup(dinleyici.as_raw_fd()) };
        if miras < 0 {
            return Err("web işçi grubu dinleyiciyi devredemedi".into());
        }
        // Miras kopyası soketi açık tutar; yönetici kendisi kabul yapmaz.
        drop(dinleyici);
        if let Some(saglik_kapisi) = ayar.saglik_kapisi {
            saglik_sunucusu(saglik_kapisi, ayar.isci, Arc::clone(saglik))?;
        }
        eprintln!(
            "Zee web işçi grubu: {} işçi, ortak dinleyici 127.0.0.1:{kapi}{}; açılış birincil işçide koşar.",
            ayar.isci,
            ayar.saglik_kapisi
                .map(|k| format!(", sağlık http://127.0.0.1:{k}/saglik"))
                .unwrap_or_default()
        );
        Ok(miras)
    }

    /// Birincili hazır kanalıyla başlatır ve `sunucu başlat`a ulaşmasını bekler.
    fn birincili_baslat(
        args: &[String],
        miras: i32,
        saglik: &Mutex<Saglik>,
    ) -> Result<Child, ExitCode> {
        let mut uclar = [0i32; 2];
        // SAFETY: two-int output array for pipe(2).
        if unsafe { libc::pipe(uclar.as_mut_ptr()) } != 0 {
            eprintln!("web işçi grubu hazır kanalı açılamadı");
            return Err(ExitCode::FAILURE);
        }
        let [okuma_fd, yazma_fd] = uclar;
        use std::os::fd::FromRawFd;
        // SAFETY: fresh pipe read end owned exactly once; kept out of children.
        let mut okuma = unsafe {
            libc::fcntl(okuma_fd, libc::F_SETFD, libc::FD_CLOEXEC);
            std::fs::File::from_raw_fd(okuma_fd)
        };
        let birincil = baslat(args, miras, Some(yazma_fd), 0);
        // SAFETY: parent copy of the write end; only the primary keeps one.
        unsafe { libc::close(yazma_fd) };
        let mut birincil = birincil.map_err(|hata| {
            eprintln!("web işçisi başlatılamadı: {hata}");
            ExitCode::FAILURE
        })?;
        match birincili_bekle(&mut okuma, &mut birincil)? {
            true => Ok(birincil),
            false => Err(kapanis(&mut [yeni_yuva(Some(birincil))], saglik)),
        }
    }

    /// Çıkan işçiyi kaydeder; art arda hızlı çıkış sınırı aşılırsa `false`.
    fn cikisi_isle(yuva: &mut Yuva, sira: usize, simdi: Instant) -> bool {
        let Some(c) = &mut yuva.cocuk else {
            return true;
        };
        let durum = match c.try_wait() {
            Ok(Some(d)) => d.to_string(),
            Ok(None) => return true,
            Err(e) => e.to_string(),
        };
        let pid = c.id();
        yuva.cocuk = None;
        let hizli = simdi.duration_since(yuva.baslangic) < HIZLI_COKUS;
        yuva.ardisik = if hizli { yuva.ardisik + 1 } else { 1 };
        // K-338: 10 sn dolmadan art arda AZAMI_ARDISIK_COKUS'uncu çıkış grubu
        // durdurur (spec/12); sayaç bu çıkışı da içerir.
        if hizli && yuva.ardisik >= AZAMI_ARDISIK_COKUS {
            eprintln!(
                "zee-web işçi {} art arda {} kez hızlı çıktı; grup durduruluyor",
                sira + 1,
                yuva.ardisik
            );
            return false;
        }
        let gecikme = Duration::from_millis(250)
            .saturating_mul(1 << (yuva.ardisik - 1).min(5))
            .min(Duration::from_secs(5));
        eprintln!(
            "zee-web işçi {} (pid {pid}) çıktı: {durum}; {} ms sonra ikincil olarak yeniden başlatılıyor",
            sira + 1,
            gecikme.as_millis()
        );
        yuva.sonraki = Some(simdi + gecikme);
        true
    }

    /// Zamanı gelen boş yuvaya ikincil işçi başlatır.
    fn yeniden_baslat(
        yuva: &mut Yuva,
        sira: usize,
        simdi: Instant,
        args: &[String],
        miras: i32,
        saglik: &Mutex<Saglik>,
    ) {
        if yuva.cocuk.is_some() || yuva.sonraki.is_none_or(|z| z > simdi) {
            return;
        }
        yuva.sonraki = None;
        match baslat(args, miras, None, sira) {
            Ok(c) => {
                yuva.cocuk = Some(c);
                yuva.baslangic = Instant::now();
                if let Ok(mut s) = saglik.lock() {
                    s.yeniden_baslatma += 1;
                }
            }
            Err(hata) => {
                eprintln!("web işçisi {} başlatılamadı: {hata}", sira + 1);
                yuva.sonraki = Some(simdi + Duration::from_secs(1));
            }
        }
    }

    pub(crate) fn yonet(ayar: Ayar, kapi: u16, args: Vec<String>) -> ExitCode {
        let saglik = Arc::new(Mutex::new(Saglik::default()));
        let miras = match kur(ayar, kapi, &saglik) {
            Ok(miras) => miras,
            Err(hata) => {
                eprintln!("{hata}");
                return ExitCode::FAILURE;
            }
        };
        let birincil = match birincili_baslat(&args, miras, &saglik) {
            Ok(c) => c,
            Err(kod) => return kod,
        };
        let mut yuvalar = vec![yeni_yuva(Some(birincil))];
        for sira in 1..usize::from(ayar.isci) {
            let cocuk = baslat(&args, miras, None, sira)
                .map_err(|hata| eprintln!("web işçisi {} başlatılamadı: {hata}", sira + 1))
                .ok();
            yuvalar.push(yeni_yuva(cocuk));
        }
        if let Ok(mut s) = saglik.lock() {
            s.hazir = true;
        }
        loop {
            saglik_guncelle(&saglik, &yuvalar);
            if SINYAL.load(Ordering::Relaxed) {
                return kapanis(&mut yuvalar, &saglik);
            }
            let simdi = Instant::now();
            for sira in 0..yuvalar.len() {
                if !cikisi_isle(&mut yuvalar[sira], sira, simdi) {
                    let _ = kapanis(&mut yuvalar, &saglik);
                    return ExitCode::FAILURE;
                }
                yeniden_baslat(&mut yuvalar[sira], sira, simdi, &args, miras, &saglik);
            }
            std::thread::sleep(ADIM);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn isci_bayraklari_yalniz_kaynak_oncesinden_ayiklanir() {
        let mut args = a(&[
            "çalıştır",
            "--web-proxy",
            "https://panel.example",
            "--web-isci-sayisi",
            "4",
            "--web-worker-port",
            "18091",
            "--web-saglik-port",
            "18099",
            "app.dil",
            "--web-isci-sayisi",
            "9",
        ]);
        let ayar = ayikla(&mut args).unwrap().unwrap();
        assert_eq!(
            ayar,
            Ayar {
                isci: 4,
                saglik_kapisi: Some(18099)
            }
        );
        assert_eq!(
            args,
            a(&[
                "çalıştır",
                "--web-proxy",
                "https://panel.example",
                "--web-worker-port",
                "18091",
                "app.dil",
                "--web-isci-sayisi",
                "9",
            ])
        );
    }

    #[test]
    fn isci_bayragi_gecersiz_degerleri_reddeder() {
        for kotu in [
            vec!["çalıştır", "--web-isci-sayisi"],
            vec!["çalıştır", "--web-isci-sayisi", "0", "a.dil"],
            vec!["çalıştır", "--web-isci-sayisi", "65", "a.dil"],
            vec!["çalıştır", "--web-isci-sayisi", "iki", "a.dil"],
            vec![
                "çalıştır",
                "--web-isci-sayisi",
                "2",
                "--web-isci-sayisi",
                "2",
                "a.dil",
            ],
            vec!["çalıştır", "--web-saglik-port", "9000", "a.dil"],
            vec![
                "çalıştır",
                "--web-isci-sayisi",
                "2",
                "--web-saglik-port",
                "0",
                "a.dil",
            ],
        ] {
            assert!(ayikla(&mut a(&kotu)).is_err(), "{kotu:?}");
        }
        assert_eq!(ayikla(&mut a(&["çalıştır", "a.dil"])).unwrap(), None);
    }

    #[test]
    fn tek_surecte_birincil_dogrudur() {
        assert!(birincil_mi(None));
        assert!(!ebeveyn_kayboldu());
        assert_eq!(devralinan_dinleyici(), None);
    }

    fn gecici_kok(ad: &str) -> std::path::PathBuf {
        let kok = std::env::temp_dir().join(format!("zee-k341-{ad}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        std::fs::create_dir_all(&kok).unwrap();
        kok
    }

    #[test]
    fn birincillik_kilidi_canli_katilimci_varken_ikincil_verir() {
        let kok = gecici_kok("canli");
        let (karar, acilis, canli) = birincilligi_kilitle(&kok, true).unwrap();
        assert!(karar);
        let acilis = acilis.expect("birincil açılış kilidini hazır olana dek tutar");
        // Açılış kilidi tutulurken yeni katılımcı karar veremez.
        let bekleyen = kilit_dosyasi(&kok.join(".zee").join(ACILIS_KILIDI)).unwrap();
        assert!(matches!(
            bekleyen.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        drop(bekleyen);
        drop(acilis);
        // Birincil yaşarken gelen süreç ikincildir ve açılış kilidini bırakır.
        let (karar2, acilis2, canli2) = birincilligi_kilitle(&kok, true).unwrap();
        assert!(!karar2);
        assert!(acilis2.is_none());
        // Birincil ölür, ikincil yaşar: yeni gelen yine ikincil.
        drop(canli);
        let (karar3, _, canli3) = birincilligi_kilitle(&kok, true).unwrap();
        assert!(!karar3);
        // Kökte kimse kalmayınca yeniden birincil seçilir.
        drop((canli2, canli3));
        let (karar4, _, _canli4) = birincilligi_kilitle(&kok, true).unwrap();
        assert!(karar4);
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn birincillik_kilidi_grup_ikincil_rolunu_asamaz_ve_bag_reddeder() {
        let kok = gecici_kok("rol");
        let (karar, acilis, _canli) = birincilligi_kilitle(&kok, false).unwrap();
        assert!(!karar, "grup ikincili kökte yalnız olsa da birincil olamaz");
        assert!(acilis.is_none());
        #[cfg(unix)]
        {
            let bag = gecici_kok("bag");
            std::fs::create_dir_all(bag.join(".zee")).unwrap();
            std::os::unix::fs::symlink(
                kok.join(".zee").join(ACILIS_KILIDI),
                bag.join(".zee").join(ACILIS_KILIDI),
            )
            .unwrap();
            assert!(birincilligi_kilitle(&bag, true).is_err());
            let _ = std::fs::remove_dir_all(&bag);
        }
        let _ = std::fs::remove_dir_all(&kok);
    }
}
