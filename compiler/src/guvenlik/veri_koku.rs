//! K-394 (RFC-0045, ADR-170): bildirilen paylaşılan veri kökü.
//!
//! Bir uygulamanın birden çok projesi (ör. ayrı veritabanı rolleriyle koşan
//! alıcı, gönderici ve web hizmetleri) aynı veri alt ağacını paylaşabilir.
//! `proje.dil` yalnız kökün **ortam/credential adını** taşır
//! (`veri_kökü "UYGULAMA_DEPO" olsun`); mutlak yol işletim tarafındadır.
//! Program yolu `veri:` önekiyle adlandırır (`veri:ekler/a-1f…`).
//!
//! Güvenlik sözleşmesi (spec/07 §Paylaşılan veri kökü, ADR-170):
//! - Kök değeri tek satır mutlak yoldur; `.`/`..` ve sembolik bağ içeremez.
//!   Yol `/`'den her bileşen `O_DIRECTORY|O_NOFOLLOW` ile açılarak yürünür.
//! - Kökün üst klasörleri: sahibi root ya da etkin kullanıcı; grup/diğer
//!   yazamaz (yapışkan bitli `/tmp` benzeri klasör hariç).
//! - **Ata değişmezi:** hedefin doğrudan üst klasörü dışındaki her klasör
//!   (kök dahil) root ya da etkin kullanıcıya aittir ve grup/diğer yazamaz.
//!   Böylece başka bir kullanıcı (paylaşılan kökteki başka bir hizmet) hiçbir
//!   ara klasör girdisini sembolik bağla değiştiremez; yolun ara bileşenleri
//!   doğrulama ile kullanım arasında sabittir. Paylaşılan yazma yalnız yaprak
//!   klasörlerde olur.
//! - Yaprak: sembolik bağ izlenmez; okuma `O_NOFOLLOW|O_NONBLOCK` ile açılır,
//!   açılan tanıtıcı normal dosya ve **tek hard-link** olmalıdır (başka
//!   kullanıcının paylaşılan klasöre bağladığı dış dosya okunmaz).
//! - Linux'ta `fs.protected_hardlinks` kapalıysa (`0`) ya da `/proc` bağlı
//!   değilse kök reddedilir; ayrıcalıksız hizmet değeri okuyamıyorsa
//!   (EACCES/EPERM) koruma bilinmiyor kipidir ve kök kabul edilir (K-422).
//! - Yazımlar tek-bağ kipindedir (K-422): kilit, var olan hedefin içerik ve
//!   metadata okuması, eylem/taşıma/silme izi ve EXDEV kopyası açılan
//!   tanıtıcıda tek sert bağ ister ([`crate::kalici_dosya::tek_bag_kipinde`]).
//! - Hata iletileri kökün mutlak yolunu taşımaz; programın `veri:` yolunu taşır.
//! - Windows ve unix dışı hedeflerde veri kökü bu sürümde desteklenmez.

use std::path::{Path, PathBuf};

/// Yolun hangi amaçla çözüldüğü.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Amac {
    /// Var olan normal dosya okunacak.
    Okuma,
    /// Hedef yazılacak/taşınacak/silinecek; yaprak yok olabilir.
    Yazma,
}

/// Doğrulanmış veri kökü. Yalnız [`VeriKoku::ortamdan`] ya da
/// [`VeriKoku::yoldan`] ile kurulur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VeriKoku {
    kok: PathBuf,
}

/// Kök yolunu taşıyan değerin üst sınırı (bayt).
pub const KOK_DEGERI_SINIRI: usize = 4096;

impl VeriKoku {
    /// Kök yolunu ortam değişkeninden ya da aynı adlı systemd credential'dan
    /// okur (K-339 kaynak kuralı) ve doğrular.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn ortamdan(ad: &str) -> Result<Self, String> {
        use super::sir_dizini::{gizli_deger_oku, SatirProfili, SirHatasi};
        let deger =
            gizli_deger_oku(ad, KOK_DEGERI_SINIRI, SatirProfili::TekSatir).map_err(|hata| {
                match hata {
                    SirHatasi::Yok => format!(
                    "veri kökü ayarlanmamış: `{ad}` ortam değişkeni ya da systemd credential'ı yok"
                ),
                    SirHatasi::Gecersiz(neden) => format!("veri kökü `{ad}` okunamadı: {neden}"),
                }
            })?;
        Self::yoldan(Path::new(&deger)).map_err(|neden| format!("veri kökü `{ad}` {neden}"))
    }

    /// Mutlak kök yolunu doğrular.
    pub fn yoldan(kok: &Path) -> Result<Self, String> {
        platform::koku_dogrula(kok)?;
        Ok(Self {
            kok: kok.to_path_buf(),
        })
    }

    /// `veri:` sonrasındaki doğrulanmış bileşenleri kök altında yürür ve
    /// işletim sistemi yolunu döndürür. Bileşenler
    /// [`crate::yetkinlik::veri_yolu_bilesenleri`] ile önceden ayrılmıştır.
    pub fn coz(&self, yol: &str, bilesenler: &[&str], amac: Amac) -> Result<PathBuf, String> {
        platform::yuru(&self.kok, yol, bilesenler, amac)?;
        let mut sonuc = self.kok.clone();
        sonuc.extend(bilesenler);
        Ok(sonuc)
    }
}

/// Veri kökündeki yaprağı okuma için güvenle açar: sembolik bağ izlenmez,
/// FIFO/aygıtta bloklanmaz, normal dosya ve tek hard-link olmalıdır.
pub fn okuma_icin_ac(gercek: &Path, yol: &str) -> Result<std::fs::File, String> {
    platform::okuma_icin_ac(gercek, yol)
}

/// Güvenle açılmış veri kökü yaprağından çağıranın sınırıyla (spec/24) katı
/// UTF-8 metin okur; ileti yalnız programın `veri:` yolunu taşır.
pub fn metin_oku(dosya: std::fs::File, yol: &str, azami: usize) -> std::io::Result<String> {
    use std::io::{Error, ErrorKind, Read};
    let asim = || {
        Error::new(
            ErrorKind::InvalidData,
            format!("`{yol}` {} MiB okuma sınırını aşıyor", azami / 1024 / 1024),
        )
    };
    if dosya.metadata()?.len() > azami as u64 {
        return Err(asim());
    }
    let mut baytlar = Vec::new();
    dosya
        .take(azami.saturating_add(1) as u64)
        .read_to_end(&mut baytlar)?;
    if baytlar.len() > azami {
        return Err(asim());
    }
    String::from_utf8(baytlar).map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("`{yol}` geçerli UTF-8 metin değil"),
        )
    })
}

#[cfg(unix)]
mod platform {
    use super::Amac;
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    const DIZIN_BAYRAKLARI: libc::c_int =
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;

    fn fstat(fd: libc::c_int) -> Option<libc::stat> {
        let mut bilgi = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: fd açık; fstat başarıda yapının tamamını doldurur.
        let sonuc = unsafe { libc::fstat(fd, bilgi.as_mut_ptr()) };
        // SAFETY: yalnız başarıda okunur.
        (sonuc == 0).then(|| unsafe { bilgi.assume_init() })
    }

    fn sahip_kabul(uid: libc::uid_t) -> bool {
        // SAFETY: geteuid yan etkisiz ve her zaman başarılıdır.
        uid == 0 || uid == unsafe { libc::geteuid() }
    }

    fn openat_dizin(ust: &OwnedFd, ad: &[u8]) -> Result<OwnedFd, std::io::Error> {
        let ad = CString::new(ad).map_err(|_| std::io::Error::from_raw_os_error(libc::EINVAL))?;
        // SAFETY: üst descriptor açık; ad NUL sonlu tek bileşen.
        let fd = unsafe { libc::openat(ust.as_raw_fd(), ad.as_ptr(), DIZIN_BAYRAKLARI) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: openat yeni descriptor döndü ve başka sahibi yok.
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }

    fn kok_ac() -> Result<OwnedFd, String> {
        // SAFETY: geçerli NUL sonlu sabit yol.
        let fd = unsafe { libc::open(c"/".as_ptr(), DIZIN_BAYRAKLARI) };
        if fd < 0 {
            return Err("kök dizin açılamadı".into());
        }
        // SAFETY: fd yeni açıldı ve başka sahibi yok.
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }

    /// Kökü `/`'den yürür; son descriptor veri köküdür.
    fn koke_yuru(kok: &Path) -> Result<OwnedFd, String> {
        let baytlar = kok.as_os_str().as_bytes();
        if baytlar.first() != Some(&b'/') {
            return Err("mutlak yol olmalı (`/` ile başlar)".into());
        }
        let parcalar: Vec<&[u8]> = baytlar
            .split(|c| *c == b'/')
            .filter(|p| !p.is_empty())
            .collect();
        if parcalar.is_empty() {
            return Err("dosya sisteminin kökü (`/`) olamaz".into());
        }
        let mut sahip = kok_ac()?;
        for (sira, parca) in parcalar.iter().enumerate() {
            if *parca == b"." || *parca == b".." {
                return Err("`.` ya da `..` bileşeni taşıyamaz".into());
            }
            sahip = openat_dizin(&sahip, parca).map_err(|hata| match hata.raw_os_error() {
                Some(libc::ENOENT) => "klasörü yok".to_string(),
                Some(libc::ELOOP) | Some(libc::ENOTDIR) => {
                    "sembolik bağ içeremez ve klasör olmalı; gerçek (kanonik) yolu yazın"
                        .to_string()
                }
                _ => format!("açılamadı ({hata})"),
            })?;
            let bilgi = fstat(sahip.as_raw_fd()).ok_or("bilgisi okunamadı")?;
            let son = sira + 1 == parcalar.len();
            if !sahip_kabul(bilgi.st_uid) {
                return Err(if son {
                    "klasörünün sahibi root ya da hizmetin etkin kullanıcısı olmalı".into()
                } else {
                    "üst klasörlerinin sahibi root ya da hizmetin etkin kullanıcısı olmalı".into()
                });
            }
            let baskasi_yazar = bilgi.st_mode & 0o022 != 0;
            let yapiskan = bilgi.st_mode & libc::S_ISVTX as libc::mode_t != 0;
            if !son && baskasi_yazar && !yapiskan {
                return Err(
                    "üst klasörlerinden biri grup ya da diğer kullanıcılarca yazılabilir".into(),
                );
            }
        }
        Ok(sahip)
    }

    /// K-422: `fs.protected_hardlinks` okumasının kararı. Değer `0` ise
    /// çekirdek koruması kapalıdır ve kök reddedilir. Dosya yoksa (`/proc`
    /// bağlı değil) ya da başka bir okuma hatasında kök gerçek nedenle
    /// reddedilir. Ayrıcalıksız hizmetin okuma izni yoksa (EACCES/EPERM;
    /// Ubuntu 22.04'te dosya `0600 root`) koruma **bilinmiyor** kipidir: kök
    /// kabul edilir, çünkü veri kökünün her içerik okuyan açılışı zaten
    /// tanıtıcıda tek sert bağ ister (okuma ailesi, ekleme/eylem/taşıma/silme
    /// izi, metadata, EXDEV kopyası, kilit; ADR-170 eki).
    #[cfg(any(target_os = "linux", test))]
    pub(super) fn koruma_karari(okuma: std::io::Result<String>) -> Result<(), String> {
        match okuma {
            Ok(deger) if deger.trim() != "0" => Ok(()),
            Ok(_) => {
                Err("için `fs.protected_hardlinks=1` gerekir (çekirdek koruması kapalı)".into())
            }
            Err(hata) => match hata.raw_os_error() {
                Some(libc::EACCES) | Some(libc::EPERM) => Ok(()),
                Some(libc::ENOENT) | Some(libc::ENOTDIR) => Err(
                    "için `fs.protected_hardlinks` okunamadı: /proc bağlı değil (dosya yok)".into(),
                ),
                _ => Err(format!("için `fs.protected_hardlinks` okunamadı ({hata})")),
            },
        }
    }

    #[cfg(target_os = "linux")]
    fn hard_link_korumasi() -> Result<(), String> {
        #[cfg(test)]
        if let Some(kaynak) = koruma_kancasi::al() {
            return koruma_karari(kaynak());
        }
        koruma_karari(std::fs::read_to_string("/proc/sys/fs/protected_hardlinks"))
    }

    /// Test derlemesinde `/proc` okumasının yerine geçen iş parçacığı kancası.
    #[cfg(all(test, target_os = "linux"))]
    pub(super) mod koruma_kancasi {
        use std::cell::Cell;
        type Kaynak = fn() -> std::io::Result<String>;
        thread_local! {
            static KANCA: Cell<Option<Kaynak>> = const { Cell::new(None) };
        }
        pub(crate) fn al() -> Option<Kaynak> {
            KANCA.with(Cell::get)
        }
        /// Kancayı kurar; dönen değer düşünce kaldırılır.
        pub(crate) fn kur(kaynak: Kaynak) -> impl Drop {
            struct Kaldir;
            impl Drop for Kaldir {
                fn drop(&mut self) {
                    KANCA.with(|k| k.set(None));
                }
            }
            KANCA.with(|k| k.set(Some(kaynak)));
            Kaldir
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn hard_link_korumasi() -> Result<(), String> {
        Ok(())
    }

    pub(super) fn koku_dogrula(kok: &Path) -> Result<(), String> {
        koke_yuru(kok)?;
        hard_link_korumasi()
    }

    /// Ata değişmezi: hedefin doğrudan üst klasörü dışındaki klasörler
    /// başka kullanıcıca değiştirilemez olmalı.
    fn ata_denetle(fd: &OwnedFd, yol: &str) -> Result<(), String> {
        let bilgi = fstat(fd.as_raw_fd())
            .ok_or_else(|| format!("\"{yol}\" veri kökü klasör bilgisi okunamadı"))?;
        if !sahip_kabul(bilgi.st_uid) || bilgi.st_mode & 0o022 != 0 {
            return Err(format!(
                "\"{yol}\" reddedildi: veri kökünde dosya yalnız sahibi root ya da hizmet olan ve grup/diğer kullanıcılarca yazılamayan klasörlerin altındaki klasörde durabilir (paylaşılan yazma yalnız son klasörde)"
            ));
        }
        Ok(())
    }

    pub(super) fn yuru(
        kok: &Path,
        yol: &str,
        bilesenler: &[&str],
        amac: Amac,
    ) -> Result<(), String> {
        let (yaprak, klasorler) = bilesenler
            .split_last()
            .ok_or_else(|| format!("\"{yol}\" veri kökü yolu boş"))?;
        let mut ust =
            koke_yuru(kok).map_err(|neden| format!("\"{yol}\" için veri kökü {neden}"))?;
        for (sira, klasor) in klasorler.iter().enumerate() {
            // Kök ve bu klasörün altında daha klasör varsa bu klasör atadır.
            ata_denetle(&ust, yol)?;
            ust = openat_dizin(&ust, klasor.as_bytes()).map_err(|hata| {
                let klasor_yolu = klasorler[..=sira].join("/");
                match (hata.raw_os_error(), amac) {
                    (Some(libc::ENOENT), Amac::Yazma) => format!(
                        "\"{yol}\" için hedef klasör \"veri:{klasor_yolu}\" yok; veri kökündeki klasörleri işletme oluşturur"
                    ),
                    (Some(libc::ENOENT), Amac::Okuma) => format!("\"{yol}\" bulunamadı"),
                    (Some(libc::ELOOP), _) | (Some(libc::ENOTDIR), _) => format!(
                        "\"{yol}\" reddedildi: veri kökünde sembolik bağ izlenmez ve \"veri:{klasor_yolu}\" klasör olmalı"
                    ),
                    _ => format!("\"{yol}\" veri kökünde güvenle çözülemedi: {hata}"),
                }
            })?;
        }
        let ad = CString::new(yaprak.as_bytes())
            .map_err(|_| format!("\"{yol}\" veri kökü yolu NUL taşıyamaz"))?;
        let mut bilgi = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: üst descriptor açık; ad NUL sonlu tek bileşen; yalnız başarıda okunur.
        let sonuc = unsafe {
            libc::fstatat(
                ust.as_raw_fd(),
                ad.as_ptr(),
                bilgi.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if sonuc != 0 {
            let hata = std::io::Error::last_os_error();
            return match (hata.raw_os_error(), amac) {
                (Some(libc::ENOENT), Amac::Yazma) => Ok(()),
                (Some(libc::ENOENT), Amac::Okuma) => Err(format!("\"{yol}\" bulunamadı")),
                _ => Err(format!("\"{yol}\" veri kökünde güvenle çözülemedi: {hata}")),
            };
        }
        // SAFETY: fstatat başarılı döndü.
        let bilgi = unsafe { bilgi.assume_init() };
        let tur = bilgi.st_mode & libc::S_IFMT;
        if tur == libc::S_IFLNK {
            return Err(format!(
                "\"{yol}\" reddedildi: veri kökünde sembolik bağ izlenmez"
            ));
        }
        if tur != libc::S_IFREG {
            return Err(format!("\"{yol}\" normal dosya değil"));
        }
        if amac == Amac::Okuma && bilgi.st_nlink != 1 {
            return Err(format!(
                "\"{yol}\" reddedildi: veri kökünde birden çok hard-link taşıyan dosya okunmaz"
            ));
        }
        Ok(())
    }

    pub(super) fn okuma_icin_ac(gercek: &Path, yol: &str) -> Result<std::fs::File, String> {
        use std::os::unix::fs::OpenOptionsExt;
        let dosya = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOCTTY)
            .open(gercek)
            .map_err(|hata| match hata.raw_os_error() {
                Some(libc::ELOOP) => {
                    format!("\"{yol}\" reddedildi: veri kökünde sembolik bağ izlenmez")
                }
                Some(libc::ENOENT) => format!("\"{yol}\" bulunamadı"),
                _ => format!("\"{yol}\" dosyası açılamadı: {hata}"),
            })?;
        let bilgi =
            fstat(dosya.as_raw_fd()).ok_or_else(|| format!("\"{yol}\" dosya bilgisi okunamadı"))?;
        if bilgi.st_mode & libc::S_IFMT != libc::S_IFREG {
            return Err(format!("\"{yol}\" normal dosya değil"));
        }
        if bilgi.st_nlink != 1 {
            return Err(format!(
                "\"{yol}\" reddedildi: veri kökünde birden çok hard-link taşıyan dosya okunmaz"
            ));
        }
        // Okuma bloklayan kipe döner (FIFO/aygıt yukarıda elendi).
        // SAFETY: fd açık; F_GETFL/F_SETFL yalnız durum bayraklarını değiştirir.
        unsafe {
            let bayrak = libc::fcntl(dosya.as_raw_fd(), libc::F_GETFL);
            if bayrak >= 0 {
                libc::fcntl(dosya.as_raw_fd(), libc::F_SETFL, bayrak & !libc::O_NONBLOCK);
            }
        }
        Ok(dosya)
    }
}

#[cfg(not(unix))]
mod platform {
    use super::Amac;
    use std::path::Path;

    const DESTEK_YOK: &str =
        "bu platformda desteklenmez (K-394: veri kökü bu sürümde yalnız unix sistemlerde)";

    pub(super) fn koku_dogrula(_kok: &Path) -> Result<(), String> {
        Err(DESTEK_YOK.into())
    }

    pub(super) fn yuru(
        _kok: &Path,
        yol: &str,
        _bilesenler: &[&str],
        _amac: Amac,
    ) -> Result<(), String> {
        Err(format!("\"{yol}\" veri kökü {DESTEK_YOK}"))
    }

    pub(super) fn okuma_icin_ac(_gercek: &Path, yol: &str) -> Result<std::fs::File, String> {
        Err(format!("\"{yol}\" veri kökü {DESTEK_YOK}"))
    }
}

#[cfg(all(test, unix))]
mod testler;
