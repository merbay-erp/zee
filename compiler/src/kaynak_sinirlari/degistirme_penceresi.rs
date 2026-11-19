//! K-342: Windows `ReplaceFileW` okuyucu penceresi (spec/08).
//!
//! `ReplaceFileW` hedefi iki yeniden adlandırmayla değiştirir ve değiştirme
//! bitene kadar yeni hedefi paylaşımsız tutanakla açık tutar. Aradaki anda
//! okuyucu `ERROR_FILE_NOT_FOUND`/`PATH_NOT_FOUND` ya da
//! `ERROR_SHARING_VIOLATION`/`ACCESS_DENIED` alır. Boşta pencere ~1 ms'dir;
//! yüklü makinede yazar değiştirmenin ortasında kesilirse süresi sınırsızdır.
//!
//! Her zee değiştirmesi klasörün `.zee-yazma-kilidi` dosyasını tekil kilitler
//! (K-084/K-183). Geçici hata alan okuyucu bu kilidi paylaşımlı alarak süren
//! değiştirmenin bitmesini bekler; kilidi tutarken yeni değiştirme başlayamaz,
//! dolayısıyla kısa yeniden denemeden sonra kalan hata gerçektir. İlk K-342
//! sürümü pencereyi zaman bütçesi ve geçici dosya iziyle tahmin ediyordu; iz
//! görünmeyen anda (iki değiştirmenin arası) süren paylaşım ihlali yine
//! okuyucuya taşıyordu.

// Üretimde yalnız Windows okuyucusu kullanır; testler her platformda koşar.
#![cfg_attr(not(windows), allow(dead_code))]

use std::cell::Cell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

/// Klasör başına kalıcı zee yazma kilidi dosyası (`kalici_dosya` ile ortak).
pub(crate) const YAZMA_KILIDI_ADI: &str = ".zee-yazma-kilidi";

/// Kilit beklendikten sonra kalan dış etkenler için kısa bütçe (K-182 ölçeği).
pub(crate) const KISA_BUTCE: Duration = Duration::from_millis(50);

thread_local! {
    static TUTULAN_YAZMA_KILIDI: Cell<usize> = const { Cell::new(0) };
}

/// Yazma kilidini tutan iş parçacığını işaretler. O iş parçacığının kilit
/// altındaki okuması (ör. ekleme) kendi tekil kilidini beklemez; bu sırada
/// klasörde başka zee değiştirmesi zaten süremez.
pub(crate) struct YazarIsareti(());

impl YazarIsareti {
    pub(crate) fn koy() -> Self {
        TUTULAN_YAZMA_KILIDI.with(|sayi| sayi.set(sayi.get() + 1));
        Self(())
    }
}

impl Drop for YazarIsareti {
    fn drop(&mut self) {
        TUTULAN_YAZMA_KILIDI.with(|sayi| sayi.set(sayi.get().saturating_sub(1)));
    }
}

/// `ac` geçici pencere koduyla düşerse süren zee değiştirmesini `uzun`
/// bütçeye kadar bekler, sonra `KISA_BUTCE` boyunca yeniden dener. Diğer
/// hatalar hemen döner.
pub(crate) fn pencereyi_bekleyerek<T>(
    hedef: &Path,
    uzun: Duration,
    mut ac: impl FnMut() -> io::Result<T>,
) -> io::Result<T> {
    let hata = match ac() {
        Ok(deger) => return Ok(deger),
        Err(hata) => hata,
    };
    if !pencere_kodu(&hata) {
        return Err(hata);
    }
    let _bariyer = if TUTULAN_YAZMA_KILIDI.with(Cell::get) == 0 {
        degistirmenin_bitmesini_bekle(hedef, uzun)
    } else {
        None
    };
    let baslangic = Instant::now();
    let mut bekleme = Duration::from_millis(1);
    loop {
        let hata = match ac() {
            Ok(deger) => return Ok(deger),
            Err(hata) => hata,
        };
        let gecen = baslangic.elapsed();
        if !pencere_kodu(&hata) || gecen >= KISA_BUTCE {
            return Err(hata);
        }
        std::thread::sleep(bekleme.min(KISA_BUTCE - gecen));
        bekleme = (bekleme * 2).min(Duration::from_millis(16));
    }
}

/// 2/3/5/32: `ACCESS_DENIED` ve `SHARING_VIOLATION` `Uncategorized` sınıfına
/// düştüğü için ham kod eşlenir.
fn pencere_kodu(hata: &io::Error) -> bool {
    matches!(hata.raw_os_error(), Some(2 | 3 | 5 | 32))
}

/// Paylaşımlı kilit tutucusu; düşünce kilit bırakılır.
pub(crate) struct Bariyer(File);

impl Drop for Bariyer {
    fn drop(&mut self) {
        platform::birak(&self.0);
    }
}

/// Hedef klasörün yazma kilidini paylaşımlı alır. Kilit dosyası yoksa orada
/// hiç zee değiştirmesi olmamıştır; bekleme süresi dolarsa `None` döner.
/// Bloklayan çağrı yardımcı iş parçacığındadır ki süre sınırı korunsun.
pub(crate) fn degistirmenin_bitmesini_bekle(hedef: &Path, sure: Duration) -> Option<Bariyer> {
    let klasor = hedef
        .parent()
        .filter(|klasor| !klasor.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let kilit = File::open(klasor.join(YAZMA_KILIDI_ADI)).ok()?;
    let (gonder, al) = std::sync::mpsc::channel::<File>();
    std::thread::Builder::new()
        .name("zee-okuyucu-bariyeri".into())
        .spawn(move || {
            if platform::paylasimli_kilitle(&kilit).is_ok() {
                if let Err(geri) = gonder.send(kilit) {
                    platform::birak(&geri.0);
                }
            }
        })
        .ok()?;
    al.recv_timeout(sure).ok().map(Bariyer)
}

#[cfg(unix)]
mod platform {
    use std::fs::File;
    use std::io;
    use std::os::fd::AsRawFd;

    extern "C" {
        fn flock(fd: i32, islem: i32) -> i32;
    }

    pub fn paylasimli_kilitle(dosya: &File) -> io::Result<()> {
        loop {
            // SAFETY: geçerli File tanıtıcısı; LOCK_SH = 1.
            if unsafe { flock(dosya.as_raw_fd(), 1) } == 0 {
                return Ok(());
            }
            let hata = io::Error::last_os_error();
            if hata.kind() != io::ErrorKind::Interrupted {
                return Err(hata);
            }
        }
    }

    pub fn birak(dosya: &File) {
        // SAFETY: aynı geçerli tanıtıcı; LOCK_UN = 8.
        let _ = unsafe { flock(dosya.as_raw_fd(), 8) };
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::fs::File;
    use std::io;
    use std::os::windows::io::AsRawHandle;

    type Handle = *mut c_void;

    // `kalici_dosya::platform` ile yapısal olarak aynı bildirim.
    #[repr(C)]
    struct Overlapped {
        internal: usize,
        internal_high: usize,
        offset: u32,
        offset_high: u32,
        olay: Handle,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LockFileEx(h: Handle, f: u32, r: u32, d: u32, y: u32, o: *mut Overlapped) -> i32;
        fn UnlockFileEx(h: Handle, r: u32, d: u32, y: u32, o: *mut Overlapped) -> i32;
    }

    fn ortusen() -> Overlapped {
        Overlapped {
            internal: 0,
            internal_high: 0,
            offset: 0,
            offset_high: 0,
            olay: std::ptr::null_mut(),
        }
    }

    /// Bayraksız `LockFileEx` paylaşımlı ve bloklayandır; yazarlarla aynı
    /// tüm-dosya aralığını kilitler (K-183).
    pub fn paylasimli_kilitle(dosya: &File) -> io::Result<()> {
        let mut bilgi = ortusen();
        // SAFETY: File yaşamda, OVERLAPPED çağrı boyunca geçerli ve sıfırlı.
        let sonuc =
            unsafe { LockFileEx(dosya.as_raw_handle(), 0, 0, u32::MAX, u32::MAX, &mut bilgi) };
        match sonuc {
            0 => Err(io::Error::last_os_error()),
            _ => Ok(()),
        }
    }

    pub fn birak(dosya: &File) {
        let mut bilgi = ortusen();
        // SAFETY: kilitlemedeki aynı File ve geçerli OVERLAPPED.
        let _ = unsafe { UnlockFileEx(dosya.as_raw_handle(), 0, u32::MAX, u32::MAX, &mut bilgi) };
    }
}

#[cfg(not(any(unix, windows)))]
mod platform {
    pub fn paylasimli_kilitle(_dosya: &std::fs::File) -> std::io::Result<()> {
        Err(std::io::ErrorKind::Unsupported.into())
    }

    pub fn birak(_dosya: &std::fs::File) {}
}
