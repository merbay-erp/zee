//! K-342: Windows `ReplaceFileW` penceresi yüklü makinede okuyucunun kısa
//! yeniden deneme bütçesini (K-182, ~50 ms) aşsa da gerçek native `dil`
//! okuyucusu hata yerine yeni bütünü görür. Harness gerçek zee yazarı gibi
//! klasörün `.zee-yazma-kilidi` dosyasını tekil kilitler ve pencereyi
//! deterministik kurar: önce hedef yok, sonra hedef paylaşımsız tutanakla
//! açık; toplam ~300 ms. `calistir` yazarın `.anlik.txt.zee-gecici-*`/
//! `….zee-yedek` izlerini de bırakır; `calistir_izsiz` iz bırakmaz (iki
//! değiştirmenin arası; ilk K-342 sürümünün iz tahmini burada düşüyordu).
//! ToplayanIo dosya sistemi penceresini taklit edemediği için toplu replay bu
//! kipleri atlar.
#![allow(dead_code)]

use std::path::Path;

pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    pencereyle_calistir(kaynak, true)
}

pub fn calistir_izsiz(kaynak: &Path) -> (i64, Vec<String>) {
    pencereyle_calistir(kaynak, false)
}

#[cfg(windows)]
fn pencereyle_calistir(kaynak: &Path, iz: bool) -> (i64, Vec<String>) {
    use std::io::{BufRead, BufReader, Write};
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    #[repr(C)]
    struct Overlapped {
        internal: usize,
        internal_high: usize,
        offset: u32,
        offset_high: u32,
        olay: *mut std::ffi::c_void,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn LockFileEx(
            h: *mut std::ffi::c_void,
            f: u32,
            r: u32,
            d: u32,
            y: u32,
            o: *mut Overlapped,
        ) -> i32;
        fn UnlockFileEx(
            h: *mut std::ffi::c_void,
            r: u32,
            d: u32,
            y: u32,
            o: *mut Overlapped,
        ) -> i32;
    }
    struct Temizle(std::path::PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-okuma-penceresi-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    let ana = proje.join("ana.dil");
    std::fs::copy(kaynak, &ana).expect("kaynak kopyası");
    // Gerçek yazar gibi: değiştirme boyunca klasörün yazma kilidi tekil tutulur.
    let kilit = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(proje.join(".zee-yazma-kilidi"))
        .expect("yazma kilidi");
    let mut ortusen = Overlapped {
        internal: 0,
        internal_high: 0,
        offset: 0,
        offset_high: 0,
        olay: std::ptr::null_mut(),
    };
    // SAFETY: tanıtıcı yaşamda, OVERLAPPED sıfırlı; LOCKFILE_EXCLUSIVE_LOCK = 2.
    let kilitlendi = unsafe {
        LockFileEx(
            kilit.as_raw_handle(),
            2,
            0,
            u32::MAX,
            u32::MAX,
            &mut ortusen,
        )
    };
    assert_ne!(kilitlendi, 0, "yazma kilidi alınamadı");
    let gecici = proje.join(".anlik.txt.zee-gecici-1-1");
    let yedek = proje.join(".anlik.txt.zee-gecici-1-1.zee-yedek");
    if iz {
        std::fs::write(&gecici, b"yeni").expect("yazarın geçici dosyası");
    }

    let mut cocuk = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", ana.to_str().expect("UTF-8 yol")])
        .current_dir(&proje)
        .stdout(Stdio::piped())
        .spawn()
        .expect("dil süreci");
    let mut cikti = BufReader::new(cocuk.stdout.take().expect("stdout"));
    let mut ilk = String::new();
    cikti.read_line(&mut ilk).expect("ilk satır");
    // Pencere 1: ReplaceFileW eski hedefi yedeğe taşıdı, yeni henüz yerinde değil.
    std::thread::sleep(Duration::from_millis(150));
    // Pencere 2: yeni hedef yerinde ama değiştirme bitene dek paylaşımsız açık.
    if iz {
        std::fs::write(&yedek, b"eski").expect("yedek");
    }
    let mut tutanak = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .share_mode(0)
        .open(proje.join("anlik.txt"))
        .expect("paylaşımsız hedef");
    tutanak.write_all(b"yeni").expect("yeni içerik");
    if iz {
        std::fs::remove_file(&gecici).expect("geçici taşındı");
    }
    std::thread::sleep(Duration::from_millis(150));
    drop(tutanak);
    if iz {
        std::fs::remove_file(&yedek).expect("yedek silindi");
    }
    // Değiştirme bitti; gerçek yazar gibi kilit açıkça bırakılır.
    // SAFETY: kilitlemedeki aynı tanıtıcı ve OVERLAPPED.
    unsafe { UnlockFileEx(kilit.as_raw_handle(), 0, u32::MAX, u32::MAX, &mut ortusen) };
    drop(kilit);

    let mut satirlar = vec![ilk.trim_end().to_string()];
    satirlar.extend(cikti.lines().map(|satir| satir.expect("stdout satırı")));
    let durum = cocuk.wait().expect("dil çıkışı");
    (i64::from(durum.code().unwrap_or(-1)), satirlar)
}

#[cfg(not(windows))]
fn pencereyle_calistir(_kaynak: &Path, _iz: bool) -> (i64, Vec<String>) {
    panic!("native-dosya-okuma-penceresi kipleri Windows ReplaceFileW penceresi ister")
}
