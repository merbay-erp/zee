//! K-376 (RFC-0039 Aşama A1): adlı çoklu dinleyici devri.
//!
//! Private CLI adapter. `proje.dil` `dinleyiciler` bildirimi olan proje
//! `dil çalıştır` ile açıldığında, kaynak derlenip yürütülmeden önce systemd
//! soket etkinleştirmesi (`LISTEN_PID`, `LISTEN_FDS`, `LISTEN_FDNAMES`; fd 3'ten
//! ardışık) doğrulanır. Dinleyiciler yalnız ADLA eşlenir: systemd fd'leri
//! `Sockets=` sırasıyla vermez, bu yüzden konum hiçbir zaman karşılaştırılmaz.
//! Her soket bildirimdeki aile/adres/kapıya bağlı, dinleyen TCP olmalı; IPv6
//! soketi `IPV6_V6ONLY` taşımalıdır. Hata çıkış 2'dir; bind'e geri dönüş
//! yoktur. `LISTEN_*` her durumda hemen silinir. Doğrulanan soketler
//! `FD_CLOEXEC` + engellemeyen kipe alınır ve süreç ömrü boyunca tutulur;
//! bağlantı kabul yüzeyi bu aşamada yoktur (RFC-0039 A2).

use dil::proje::{gecerli_dinleyici_adi, DinleyiciBildirimi};

/// Devir hatasının her iletisine eklenen düzeltme önerisi.
pub(crate) const ONERI: &str = "Öneri: her bildirilen dinleyici için Accept=no ve FileDescriptorName=<ad> taşıyan ayrı bir .socket birimi tanımlayıp hizmete Sockets= ile bağlayın; IPv6 birimine BindIPv6Only=ipv6-only yazın ve IPv4'ü ayrı adla bildirin (docs/adli-dinleyici-devri.md).";

/// Bildirim ↔ devir eşlemesi: her bildirim indeksi için devralınan fd.
/// Saf işlev: ortam değerleri parametredir, sıra karşılaştırılmaz.
pub(crate) fn eslestir(
    bildirim: &[DinleyiciBildirimi],
    pid: Option<&str>,
    adet: Option<&str>,
    adlar: Option<&str>,
    bu_pid: u32,
) -> Result<Vec<i32>, String> {
    let (Some(pid), Some(adet)) = (pid, adet) else {
        return Err("systemd soket devri yok (LISTEN_PID/LISTEN_FDS tanımsız)".into());
    };
    if pid.parse::<u32>().ok() != Some(bu_pid) {
        return Err(format!(
            "LISTEN_PID={pid} bu sürece ait değil (soket başka sürece devredilmiş)"
        ));
    }
    let adet = adet
        .parse::<usize>()
        .ok()
        .filter(|n| (1..=dil::proje::DINLEYICI_AZAMI).contains(n))
        .ok_or_else(|| format!("LISTEN_FDS={adet} geçersiz"))?;
    let adlar = adlar
        .ok_or_else(|| "LISTEN_FDNAMES tanımsız; dinleyiciler yalnız adla eşlenir".to_string())?;
    let gelen: Vec<&str> = adlar.split(':').collect();
    if gelen.len() != adet {
        return Err(format!(
            "LISTEN_FDNAMES {} ad taşıyor, LISTEN_FDS={adet}",
            gelen.len()
        ));
    }
    let mut fdler = vec![-1i32; bildirim.len()];
    let mut fazla = Vec::new();
    for (konum, ad) in gelen.iter().enumerate() {
        if !gecerli_dinleyici_adi(ad) {
            return Err(format!(
                "fd {} için \"{ad}\" geçerli bir dinleyici adı değil (FileDescriptorName= eksik olabilir)",
                3 + konum
            ));
        }
        if gelen[..konum].contains(ad) {
            return Err(format!("\"{ad}\" adı LISTEN_FDNAMES içinde yinelendi"));
        }
        match bildirim.iter().position(|d| d.ad == *ad) {
            Some(sira) => fdler[sira] = 3 + konum as i32,
            None => fazla.push(*ad),
        }
    }
    let eksik: Vec<&str> = bildirim
        .iter()
        .zip(&fdler)
        .filter(|(_, fd)| **fd < 0)
        .map(|(d, _)| d.ad.as_str())
        .collect();
    if !eksik.is_empty() || !fazla.is_empty() {
        let mut parcalar = Vec::new();
        if !eksik.is_empty() {
            parcalar.push(format!("devredilmeyen: {}", eksik.join(", ")));
        }
        if !fazla.is_empty() {
            parcalar.push(format!("bildirimde olmayan: {}", fazla.join(", ")));
        }
        return Err(format!(
            "devralınan adlar bildirimle eşleşmiyor ({})",
            parcalar.join("; ")
        ));
    }
    Ok(fdler)
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::net::{SocketAddr, TcpListener};
    use std::sync::Mutex;

    /// Süreç ömrü boyunca tutulan adlı dinleyiciler (bağlantı kabulü A2'de).
    static TUTULAN: Mutex<Vec<(String, TcpListener)>> = Mutex::new(Vec::new());

    fn int_secenek(fd: i32, seviye: i32, ad: i32) -> Option<i32> {
        let mut deger = 0i32;
        let mut boy = std::mem::size_of::<i32>() as libc::socklen_t;
        // SAFETY: borrowed fd, correctly sized writable integer and length.
        let sonuc =
            unsafe { libc::getsockopt(fd, seviye, ad, (&mut deger as *mut i32).cast(), &mut boy) };
        (sonuc == 0 && boy as usize == std::mem::size_of::<i32>()).then_some(deger)
    }

    /// Soketin yerel adresi bildirimle tam eşleşir mi; IPv6'da V6ONLY şart.
    fn adresi_dogrula(fd: i32, beklenen: SocketAddr) -> Result<(), String> {
        // SAFETY: sockaddr_storage is initialized by getsockname; family checked
        // before reinterpretation and the returned length bounds the read.
        let mut depo: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
        let mut boy = std::mem::size_of_val(&depo) as libc::socklen_t;
        if unsafe {
            libc::getsockname(
                fd,
                (&mut depo as *mut libc::sockaddr_storage).cast(),
                &mut boy,
            )
        } != 0
        {
            return Err("yerel adres okunamadı".into());
        }
        let gercek = match i32::from(depo.ss_family) {
            libc::AF_INET if boy as usize >= std::mem::size_of::<libc::sockaddr_in>() => {
                // SAFETY: family is AF_INET and length covers sockaddr_in.
                let a = unsafe {
                    &*(&depo as *const libc::sockaddr_storage).cast::<libc::sockaddr_in>()
                };
                SocketAddr::from((a.sin_addr.s_addr.to_ne_bytes(), u16::from_be(a.sin_port)))
            }
            libc::AF_INET6 if boy as usize >= std::mem::size_of::<libc::sockaddr_in6>() => {
                // SAFETY: family is AF_INET6 and length covers sockaddr_in6.
                let a = unsafe {
                    &*(&depo as *const libc::sockaddr_storage).cast::<libc::sockaddr_in6>()
                };
                SocketAddr::from((
                    std::net::Ipv6Addr::from(a.sin6_addr.s6_addr),
                    u16::from_be(a.sin6_port),
                ))
            }
            _ => return Err("AF_INET/AF_INET6 soketi değil".into()),
        };
        if gercek != beklenen {
            return Err(format!("{gercek} adresine bağlı, bildirim {beklenen}"));
        }
        if beklenen.is_ipv6() && int_secenek(fd, libc::IPPROTO_IPV6, libc::IPV6_V6ONLY) != Some(1) {
            return Err(
                "IPv6 soketinde IPV6_V6ONLY kapalı (BindIPv6Only=ipv6-only gerekli)".into(),
            );
        }
        Ok(())
    }

    /// Ortamı okur ve siler, adla eşler, her soketi doğrular ve sahiplenir.
    pub(crate) fn devral(bildirim: &[DinleyiciBildirimi]) -> Result<(), String> {
        let pid = std::env::var("LISTEN_PID").ok();
        let adet = std::env::var("LISTEN_FDS").ok();
        let adlar = std::env::var("LISTEN_FDNAMES").ok();
        for ad in ["LISTEN_PID", "LISTEN_FDS", "LISTEN_FDNAMES"] {
            std::env::remove_var(ad);
        }
        let fdler = eslestir(
            bildirim,
            pid.as_deref(),
            adet.as_deref(),
            adlar.as_deref(),
            std::process::id(),
        )?;
        for (d, &fd) in bildirim.iter().zip(&fdler) {
            super::super::web_listener::listening_tcp(fd)
                .map_err(|_| format!("\"{}\" (fd {fd}) dinleyen bir TCP soketi değil", d.ad))?;
            adresi_dogrula(fd, d.adres)
                .map_err(|neden| format!("\"{}\" (fd {fd}): {neden}", d.ad))?;
        }
        let mut tutulan = TUTULAN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (d, &fd) in bildirim.iter().zip(&fdler) {
            // SAFETY: flag mutation on a validated inherited descriptor.
            let onceki = unsafe { libc::fcntl(fd, libc::F_GETFD) };
            if onceki < 0
                || unsafe { libc::fcntl(fd, libc::F_SETFD, onceki | libc::FD_CLOEXEC) } != 0
            {
                return Err(format!("\"{}\" (fd {fd}) FD_CLOEXEC alamadı", d.ad));
            }
            use std::os::fd::FromRawFd;
            // SAFETY: validated inherited listener; each fd transfers exactly once
            // because names (and so fds) are unique.
            let dinleyici = unsafe { TcpListener::from_raw_fd(fd) };
            dinleyici
                .set_nonblocking(true)
                .map_err(|_| format!("\"{}\" (fd {fd}) engellemeyen kipe alınamadı", d.ad))?;
            tutulan.push((d.ad.clone(), dinleyici));
        }
        eprintln!(
            "Zee adlı dinleyici devri: {} dinleyici adla eşlendi ({}).",
            bildirim.len(),
            bildirim
                .iter()
                .map(|d| format!("{}={}", d.ad, d.adres))
                .collect::<Vec<_>>()
                .join(", ")
        );
        Ok(())
    }
}

/// `dinleyiciler` bildirimi olan projede `çalıştır` öncesi devir. Boş
/// bildirimde hiçbir şey yapmaz (bugünkü davranış).
pub(crate) fn hazirla(bildirim: &[DinleyiciBildirimi], isci: bool) -> Result<(), String> {
    if bildirim.is_empty() {
        return Ok(());
    }
    let sonuc = if isci {
        Err("adlı dinleyici devri çok işçi grubuyla birlikte kullanılamaz (K-376 açık sınırı); --web-isci-sayisi olmadan tek süreç çalıştırın".to_string())
    } else {
        #[cfg(unix)]
        {
            unix::devral(bildirim)
        }
        #[cfg(not(unix))]
        {
            Err("adlı dinleyici devri Unix (systemd) ister".to_string())
        }
    };
    sonuc.map_err(|neden| format!("dinleyici devri: {neden}. {ONERI}"))
}

#[cfg(test)]
mod testler {
    use super::*;

    fn bildirim(ogeler: &[&str]) -> Vec<DinleyiciBildirimi> {
        ogeler
            .iter()
            .map(|o| DinleyiciBildirimi::ayristir(o).expect("öğe"))
            .collect()
    }

    fn dort() -> Vec<DinleyiciBildirimi> {
        bildirim(&[
            "smtp_v4 0.0.0.0:25",
            "smtp_v6 [::]:25",
            "imap_v4 0.0.0.0:143",
            "imap_v6 [::]:143",
        ])
    }

    #[test]
    fn karisik_sirali_dort_soket_adla_eslenir() {
        // systemd 257'de gözlenen bildirim dışı sıra.
        let fdler = eslestir(
            &dort(),
            Some("77"),
            Some("4"),
            Some("imap_v6:smtp_v4:imap_v4:smtp_v6"),
            77,
        )
        .expect("adla eşleşmeli");
        assert_eq!(fdler, vec![4, 6, 5, 3]);
        let fdler = eslestir(
            &dort(),
            Some("77"),
            Some("4"),
            Some("smtp_v4:smtp_v6:imap_v4:imap_v6"),
            77,
        )
        .expect("bildirim sırası da geçer");
        assert_eq!(fdler, vec![3, 4, 5, 6]);
    }

    #[test]
    fn iki_soket_ters_sirada_eslenir() {
        let b = bildirim(&["web_v4 127.0.0.1:8095", "web_v6 [::1]:8095"]);
        assert_eq!(
            eslestir(&b, Some("9"), Some("2"), Some("web_v6:web_v4"), 9).unwrap(),
            vec![4, 3]
        );
    }

    #[test]
    fn eksik_fazla_yinelenen_ve_bozuk_ad_reddedilir() {
        let b = dort();
        let hata = |adet: &str, adlar: Option<&str>| {
            eslestir(&b, Some("5"), Some(adet), adlar, 5).unwrap_err()
        };
        assert!(hata("3", Some("smtp_v4:smtp_v6:imap_v4")).contains("devredilmeyen: imap_v6"));
        assert!(hata("4", Some("smtp_v4:smtp_v6:imap_v4:pop3_v4"))
            .contains("devredilmeyen: imap_v6; bildirimde olmayan: pop3_v4"));
        assert!(hata("5", Some("smtp_v4:smtp_v6:imap_v4:imap_v6:pop3_v4"))
            .contains("bildirimde olmayan: pop3_v4"));
        assert!(hata("4", Some("smtp_v4:smtp_v4:imap_v4:imap_v6")).contains("yinelendi"));
        assert!(hata("4", Some("smtp_v4:smtp_v6:imap_v4")).contains("3 ad taşıyor"));
        assert!(hata("4", None).contains("LISTEN_FDNAMES tanımsız"));
        assert!(hata("4", Some("smtp_v4:zeemail.socket:imap_v4:imap_v6"))
            .contains("fd 4 için \"zeemail.socket\""));
        assert!(hata("4", Some("smtp_v4::imap_v4:imap_v6")).contains("geçerli bir dinleyici adı"));
        for adet in ["0", "-1", "abc", "65"] {
            assert!(hata(adet, Some("smtp_v4")).contains("geçersiz"), "{adet}");
        }
    }

    #[test]
    fn yanlis_ya_da_eksik_pid_reddedilir() {
        let b = dort();
        let adlar = Some("smtp_v4:smtp_v6:imap_v4:imap_v6");
        assert!(eslestir(&b, Some("1"), Some("4"), adlar, 5)
            .unwrap_err()
            .contains("bu sürece ait değil"));
        assert!(eslestir(&b, None, Some("4"), adlar, 5)
            .unwrap_err()
            .contains("tanımsız"));
        assert!(eslestir(&b, Some("5"), None, adlar, 5)
            .unwrap_err()
            .contains("tanımsız"));
    }

    #[test]
    fn bos_bildirim_hicbir_sey_yapmaz_isci_reddedilir() {
        assert!(hazirla(&[], true).is_ok());
        let hata = hazirla(&dort(), true).unwrap_err();
        assert!(
            hata.contains("çok işçi") && hata.contains("Öneri:"),
            "{hata}"
        );
    }
}
