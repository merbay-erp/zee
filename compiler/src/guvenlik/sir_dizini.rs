//! K-339: proje bildirimindeki gizli değerin systemd credential kaynağı.
//!
//! Proje bildirimi bir gizli değeri ortam değişkeni adıyla anar
//! (`gizli_anahtar_değişkeni`, `e_posta_bağlantı_değişkeni`, PostgreSQL
//! bağlantı değişkeni, `http_gizli_değişkenleri`). systemd `LoadCredential=`,
//! `LoadCredentialEncrypted=` ve `SetCredential=` ile verilen değerleri
//! servis ömrü boyunca `$CREDENTIALS_DIRECTORY/<ad>` dosyasında sunar ve
//! bunları alt süreçlere ortam olarak miras bırakmaz. Bu modül aynı adı
//! credential kimliği sayarak dosyayı okur; yeni sözdizimi, ayar alanı veya
//! yetki açmaz. Yetki ve izin listesi denetimi çağıranda kalır.
//!
//! Sözleşme (ADR-105 eki, spec/23 §8):
//! - `CREDENTIALS_DIRECTORY` yoksa davranış eskisiyle birebir aynıdır.
//! - Varsa mutlak olmalı; her bileşen sembolik bağ izlenmeden açılır, dizin
//!   gerçek dizin, sahibi etkin kullanıcı ya da root, grup/diğer yazamaz.
//! - Ad yalnız `[A-Za-z0-9_]` ile başlayan, `[A-Za-z0-9_.-]` devam eden en
//!   fazla 255 baytlık tek bileşendir; uymayan ad dizinde hiç aranmaz.
//! - Dosya nofollow/nonblocking açılır; gerçek düzenli dosya, tek hard-link,
//!   sahibi etkin kullanıcı ya da root, grup/diğer yazamaz, diğer okuyamaz.
//! - Boyut 1..=sınır bayttır; sınır ayırma öncesi uygulanır. Katı UTF-8,
//!   NUL yok; tek satır profilinde CR/LF yok. Kırpma yapılmaz.
//! - Aynı ad hem ortamda hem dizinde varsa öncelik yoktur: ret.
//! - Hata metinleri yalnız sabit nedendir; değer, yol, dizin adı içermez.

use std::ffi::OsString;

/// Credential dizinini taşıyan ortam değişkeni (systemd.exec(5)).
pub const DIZIN_DEGISKENI: &str = "CREDENTIALS_DIRECTORY";

/// Değerin satır biçimi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SatirProfili {
    /// URL, hex anahtar, bearer: CR/LF yasak.
    TekSatir,
    /// PEM gibi çok satırlı metin: CR/LF serbest, NUL yasak.
    CokSatir,
}

/// Okuma sonucu hatası. Hiçbir varyant değer, yol veya dizin adı taşımaz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SirHatasi {
    /// Ne ortamda ne credential dizininde var; çağıran eski iletisini verir.
    Yok,
    /// Kaynak bulundu fakat sözleşmeyi bozuyor; sabit neden.
    Gecersiz(&'static str),
}

impl SirHatasi {
    /// Kullanıcıya gösterilecek sabit neden (Yok için çağıranın iletisi kullanılır).
    pub fn neden(&self) -> &'static str {
        match self {
            SirHatasi::Yok => "gizli değer bulunamadı",
            SirHatasi::Gecersiz(neden) => neden,
        }
    }
}

/// Gizli değeri önce systemd credential dizininden, yoksa ortamdan okur.
pub fn gizli_deger_oku(
    ad: &str,
    azami_bayt: usize,
    profil: SatirProfili,
) -> Result<String, SirHatasi> {
    let ortam_var = std::env::var_os(ad).is_some();
    let ortam = std::env::var(ad).ok();
    let dizin = std::env::var_os(DIZIN_DEGISKENI);
    kaynaklardan_oku(ad, ortam_var, ortam, dizin, azami_bayt, profil)
}

/// Ortamdan bağımsız, test edilebilir çekirdek.
pub fn kaynaklardan_oku(
    ad: &str,
    ortam_var: bool,
    ortam: Option<String>,
    dizin: Option<OsString>,
    azami_bayt: usize,
    profil: SatirProfili,
) -> Result<String, SirHatasi> {
    let Some(dizin) = dizin else {
        return ortam.ok_or(SirHatasi::Yok);
    };
    if !ad_guvenli(ad) {
        return ortam.ok_or(SirHatasi::Yok);
    }
    match dizinden_oku(&dizin, ad, azami_bayt, profil)? {
        Some(_) if ortam_var => Err(SirHatasi::Gecersiz(
            "gizli değer hem ortamda hem credential dizininde var; tek kaynak bırakın",
        )),
        Some(deger) => Ok(deger),
        None => ortam.ok_or(SirHatasi::Yok),
    }
}

/// systemd credential kimliği ve tek yol bileşeni olarak güvenli ad.
pub fn ad_guvenli(ad: &str) -> bool {
    let b = ad.as_bytes();
    !b.is_empty()
        && b.len() <= 255
        && (b[0].is_ascii_alphanumeric() || b[0] == b'_')
        && b.iter()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'-'))
}

#[cfg(not(unix))]
fn dizinden_oku(
    _dizin: &OsString,
    _ad: &str,
    _azami_bayt: usize,
    _profil: SatirProfili,
) -> Result<Option<String>, SirHatasi> {
    Err(SirHatasi::Gecersiz(
        "CREDENTIALS_DIRECTORY bu platformda desteklenmez",
    ))
}

#[cfg(unix)]
const DIZIN_ACILAMADI: &str =
    "CREDENTIALS_DIRECTORY açılamadı (yok, sembolik bağ ya da dizin değil)";

#[cfg(unix)]
fn dizinden_oku(
    dizin: &OsString,
    ad: &str,
    azami_bayt: usize,
    profil: SatirProfili,
) -> Result<Option<String>, SirHatasi> {
    let dizin_fd = dizini_ac(dizin)?;
    let Some(dosya) = dosyayi_ac(&dizin_fd, ad)? else {
        return Ok(None);
    };
    drop(dizin_fd);
    dosyayi_dogrula_oku(dosya, azami_bayt, profil).map(Some)
}

/// `/`'den her bileşeni `O_DIRECTORY|O_NOFOLLOW` ile açar; son dizini denetler.
#[cfg(unix)]
fn dizini_ac(dizin: &OsString) -> Result<std::os::fd::OwnedFd, SirHatasi> {
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::ffi::OsStrExt;

    let yol = dizin.as_bytes();
    if yol.first() != Some(&b'/') {
        return Err(SirHatasi::Gecersiz(
            "CREDENTIALS_DIRECTORY mutlak yol olmalı",
        ));
    }
    let bayraklar = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    // SAFETY: geçerli NUL sonlu sabit yol.
    let fd = unsafe { libc::open(c"/".as_ptr(), bayraklar) };
    if fd < 0 {
        return Err(SirHatasi::Gecersiz(DIZIN_ACILAMADI));
    }
    // SAFETY: fd yeni açıldı ve başka sahibi yok.
    let mut sahip = unsafe { OwnedFd::from_raw_fd(fd) };
    for parca in yol.split(|c| *c == b'/').filter(|p| !p.is_empty()) {
        if parca == b"." || parca == b".." {
            return Err(SirHatasi::Gecersiz(
                "CREDENTIALS_DIRECTORY `.` ya da `..` bileşeni taşıyamaz",
            ));
        }
        let parca = CString::new(parca)
            .map_err(|_| SirHatasi::Gecersiz("CREDENTIALS_DIRECTORY NUL taşıyamaz"))?;
        // SAFETY: üst descriptor sahip ile açık; parca NUL sonlu.
        let alt = unsafe { libc::openat(sahip.as_raw_fd(), parca.as_ptr(), bayraklar) };
        if alt < 0 {
            return Err(SirHatasi::Gecersiz(DIZIN_ACILAMADI));
        }
        // SAFETY: openat yeni descriptor döndü; önceki sahip burada kapanır.
        sahip = unsafe { OwnedFd::from_raw_fd(alt) };
    }
    let bilgi = fstat(sahip.as_raw_fd()).ok_or(SirHatasi::Gecersiz(DIZIN_ACILAMADI))?;
    if bilgi.st_mode & libc::S_IFMT != libc::S_IFDIR {
        return Err(SirHatasi::Gecersiz(DIZIN_ACILAMADI));
    }
    if !sahip_kabul(bilgi.st_uid) {
        return Err(SirHatasi::Gecersiz(
            "CREDENTIALS_DIRECTORY sahibi etkin kullanıcı ya da root olmalı",
        ));
    }
    if bilgi.st_mode & 0o022 != 0 {
        return Err(SirHatasi::Gecersiz(
            "CREDENTIALS_DIRECTORY grup ya da diğer kullanıcılarca yazılabilir",
        ));
    }
    Ok(sahip)
}

/// Dizine bağlı tek bileşeni nofollow/nonblocking açar; yoksa `None`.
#[cfg(unix)]
fn dosyayi_ac(dizin: &std::os::fd::OwnedFd, ad: &str) -> Result<Option<std::fs::File>, SirHatasi> {
    use std::os::fd::{AsRawFd, FromRawFd};

    let dosya_adi = std::ffi::CString::new(ad)
        .map_err(|_| SirHatasi::Gecersiz("credential adı NUL taşıyamaz"))?;
    let bayraklar =
        libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOCTTY;
    // SAFETY: dizin descriptor'ı açık; ad tek bileşen ve NUL sonlu.
    let fd = unsafe { libc::openat(dizin.as_raw_fd(), dosya_adi.as_ptr(), bayraklar) };
    if fd < 0 {
        return match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ENOENT) => Ok(None),
            Some(libc::ELOOP) => Err(SirHatasi::Gecersiz(
                "credential dosyası sembolik bağ olamaz",
            )),
            _ => Err(SirHatasi::Gecersiz("credential dosyası açılamadı")),
        };
    }
    // SAFETY: openat yeni descriptor döndü ve başka sahibi yok.
    Ok(Some(unsafe { std::fs::File::from_raw_fd(fd) }))
}

/// Açılmış descriptor üzerinde tür/link/sahip/mod/boyut denetimi ve sınırlı okuma.
#[cfg(unix)]
fn dosyayi_dogrula_oku(
    dosya: std::fs::File,
    azami_bayt: usize,
    profil: SatirProfili,
) -> Result<String, SirHatasi> {
    use std::io::Read;
    use std::os::fd::AsRawFd;

    let bilgi = fstat(dosya.as_raw_fd())
        .ok_or(SirHatasi::Gecersiz("credential dosyası bilgisi okunamadı"))?;
    let red = if bilgi.st_mode & libc::S_IFMT != libc::S_IFREG {
        Some("credential dosyası düzenli dosya olmalı")
    } else if bilgi.st_nlink != 1 {
        Some("credential dosyası tek hard-link taşımalı")
    } else if !sahip_kabul(bilgi.st_uid) {
        Some("credential dosyasının sahibi etkin kullanıcı ya da root olmalı")
    } else if bilgi.st_mode & 0o026 != 0 {
        Some("credential dosyası grupça yazılabilir ya da herkesçe okunabilir olamaz")
    } else {
        None
    };
    if let Some(neden) = red {
        return Err(SirHatasi::Gecersiz(neden));
    }
    let boyut = u64::try_from(bilgi.st_size).unwrap_or(u64::MAX);
    if boyut == 0 {
        return Err(SirHatasi::Gecersiz("credential dosyası boş"));
    }
    if boyut > azami_bayt as u64 {
        return Err(SirHatasi::Gecersiz(
            "credential dosyası boyut sınırını aşıyor",
        ));
    }
    let mut tampon = Vec::with_capacity(boyut as usize);
    let sonuc = dosya
        .take(azami_bayt as u64 + 1)
        .read_to_end(&mut tampon)
        .map_err(|_| SirHatasi::Gecersiz("credential dosyası okunamadı"))
        .and_then(|_| metne_cevir(&tampon, azami_bayt, profil));
    if sonuc.is_err() {
        sifirla(&mut tampon);
    }
    let () = sonuc?;
    String::from_utf8(tampon)
        .map_err(|_| SirHatasi::Gecersiz("credential dosyası geçerli UTF-8 değil"))
}

#[cfg(unix)]
fn metne_cevir(tampon: &[u8], azami_bayt: usize, profil: SatirProfili) -> Result<(), SirHatasi> {
    if tampon.is_empty() {
        return Err(SirHatasi::Gecersiz("credential dosyası boş"));
    }
    if tampon.len() > azami_bayt {
        return Err(SirHatasi::Gecersiz(
            "credential dosyası boyut sınırını aşıyor",
        ));
    }
    if std::str::from_utf8(tampon).is_err() || tampon.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(SirHatasi::Gecersiz(
            "credential dosyası BOM'suz geçerli UTF-8 olmalı",
        ));
    }
    if tampon.contains(&0) {
        return Err(SirHatasi::Gecersiz("credential dosyası NUL taşıyamaz"));
    }
    if profil == SatirProfili::TekSatir && tampon.iter().any(|c| matches!(c, b'\r' | b'\n')) {
        return Err(SirHatasi::Gecersiz(
            "credential dosyası tek satır olmalı; sondaki satır sonunu kaldırın",
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn sifirla(tampon: &mut [u8]) {
    for bayt in tampon.iter_mut() {
        // SAFETY: geçerli &mut u8; volatile yazım derleyicinin silmesini önler.
        unsafe { std::ptr::write_volatile(bayt, 0) };
    }
}

#[cfg(unix)]
fn fstat(fd: libc::c_int) -> Option<libc::stat> {
    let mut bilgi = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: fd açık; fstat başarıda yapının tamamını doldurur.
    let sonuc = unsafe { libc::fstat(fd, bilgi.as_mut_ptr()) };
    // SAFETY: yalnız başarıda okunur.
    (sonuc == 0).then(|| unsafe { bilgi.assume_init() })
}

#[cfg(unix)]
fn sahip_kabul(uid: libc::uid_t) -> bool {
    // SAFETY: geteuid yan etkisiz ve her zaman başarılıdır.
    uid == 0 || uid == unsafe { libc::geteuid() }
}

#[cfg(all(test, unix))]
mod testler;

#[cfg(all(test, not(unix)))]
mod platform_testleri {
    use super::*;

    #[test]
    fn unix_disi_platformda_credential_dizini_reddedilir() {
        let dizin = Some(OsString::from("C:\\credentials"));
        assert_eq!(
            kaynaklardan_oku("A", false, None, dizin.clone(), 64, SatirProfili::TekSatir),
            Err(SirHatasi::Gecersiz(
                "CREDENTIALS_DIRECTORY bu platformda desteklenmez"
            ))
        );
        assert_eq!(
            kaynaklardan_oku(
                "A",
                true,
                Some("o".into()),
                dizin,
                64,
                SatirProfili::TekSatir
            ),
            Err(SirHatasi::Gecersiz(
                "CREDENTIALS_DIRECTORY bu platformda desteklenmez"
            ))
        );
        assert_eq!(
            kaynaklardan_oku(
                "A",
                true,
                Some("o".into()),
                None,
                64,
                SatirProfili::TekSatir
            ),
            Ok("o".into())
        );
    }
}
