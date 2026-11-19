//! K-345: çok işçi yöneticisinin sağlık bağlantılarının işçilere miras
//! kalmadığını ve her sağlık yanıtının EOF ile bittiğini gerçek native `dil`
//! yöneticisiyle sınar. 32 işçili grup (üç kez) başlarken sağlık uç noktası altı
//! iş parçacığıyla 4 sn yoklanır; ardından işçi PID'lerinin tuttuğu sağlık
//! kapısı soketleri sayılır (macOS `lsof`, Linux `/proc`). Proje Cargo hedef
//! klasöründe geçici kopyadır. ToplayanIo süreç yönetimini taklit edemediği
//! için toplu replay bu kipi atlar; Unix dışında sahte başarı yoktur.
#![allow(dead_code)]

use std::path::Path;

#[cfg(unix)]
fn saglik_govdesi(kapi: u16) -> Option<String> {
    use std::io::{Read, Write};
    let mut akis = std::net::TcpStream::connect(("127.0.0.1", kapi)).ok()?;
    akis.set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok()?;
    akis.write_all(b"GET /saglik HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .ok()?;
    let mut yanit = String::new();
    akis.read_to_string(&mut yanit).ok()?;
    Some(yanit.split("\r\n\r\n").nth(1)?.to_string())
}

/// Linux: `/proc/<pid>/fd` soket inode'ları `/proc/net/tcp` içinde yerel ya
/// da uzak ucu sağlık kapısı olan bağlantılarla eşlenir.
#[cfg(target_os = "linux")]
fn tutulan_soketler(kapi: u16, pidler: &[u32]) -> usize {
    let hedef = format!(":{kapi:04X}");
    let mut inodelar = std::collections::HashSet::new();
    if let Ok(tablo) = std::fs::read_to_string("/proc/net/tcp") {
        for satir in tablo.lines().skip(1) {
            let alan: Vec<&str> = satir.split_whitespace().collect();
            if alan.len() > 9 && (alan[1].ends_with(&hedef) || alan[2].ends_with(&hedef)) {
                inodelar.insert(alan[9].to_string());
            }
        }
    }
    pidler
        .iter()
        .filter_map(|pid| std::fs::read_dir(format!("/proc/{pid}/fd")).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|g| std::fs::read_link(g.path()).ok())
        .filter_map(|h| {
            h.to_str()?
                .strip_prefix("socket:[")?
                .strip_suffix(']')
                .map(str::to_string)
        })
        .filter(|ino| inodelar.contains(ino))
        .count()
}

/// macOS ve diğer Unix: `lsof` ile sağlık kapısındaki TCP uçlarını tutan
/// işçi PID'leri sayılır (dinleyici hariç).
#[cfg(all(unix, not(target_os = "linux")))]
fn tutulan_soketler(kapi: u16, pidler: &[u32]) -> usize {
    let cikti = std::process::Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{kapi}")])
        .output()
        .expect("lsof çalıştırılamadı (kip için gerekli)");
    String::from_utf8_lossy(&cikti.stdout)
        .lines()
        .skip(1)
        .filter(|s| !s.contains("LISTEN"))
        .filter_map(|s| s.split_whitespace().nth(1)?.parse::<u32>().ok())
        .filter(|pid| pidler.contains(pid))
        .count()
}

/// Ard arda açılan grup sayısı. Düzeltme öncesi macOS'ta 40 grupta 8 miras
/// soket görüldü (grup başına ≈%20); üç grup tek koşuda yakalama olasılığını
/// artırır, süreyi ≈20 sn'de tutar.
const GRUP: usize = 3;

/// `(0, ["miras <n>", "asili <n>"])`: `GRUP` koşu boyunca işçilerin tuttuğu
/// sağlık soketi ve 2 sn içinde EOF ile bitmeyen sağlık isteği toplamı; çıkış
/// bir grup 32 canlı işçiye ulaşmadıysa 1.
pub fn calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    #[cfg(not(unix))]
    {
        let _ = kaynak_yolu;
        panic!("actual Unix worker group capability unavailable; no synthetic pass");
    }
    #[cfg(unix)]
    {
        use std::io::{Read, Write};
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::time::{Duration, Instant};

        let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "zee-k345-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("saat")
                .as_nanos()
        ));
        std::fs::create_dir_all(&proje).expect("proje klasörü");
        struct Temizle(std::path::PathBuf);
        impl Drop for Temizle {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _temizle = Temizle(proje.clone());
        let ana = proje.join("ana.dil");
        std::fs::copy(kaynak_yolu, &ana).expect("kaynak kopyası");
        let (mut miras_toplam, mut asili_toplam, mut eksik_grup) = (0, 0, 0);
        for _ in 0..GRUP {
            let kapi = super::native_web_kopma_fixture::bos_kapi();
            let saglik = super::native_web_kopma_fixture::bos_kapi();
            let mut yonetici = Command::new(env!("CARGO_BIN_EXE_dil"))
                .args([
                    "çalıştır",
                    "--web-proxy",
                    super::native_web_kopma_fixture::PROXY,
                    "--web-worker-port",
                    &kapi.to_string(),
                    "--web-isci-sayisi",
                    "32",
                    "--web-saglik-port",
                    &saglik.to_string(),
                    ana.to_str().expect("UTF-8 kaynak yolu"),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .process_group(0)
                .spawn()
                .expect("yönetici süreci");
            let grup = yonetici.id() as libc::pid_t;
            let asili = Arc::new(AtomicUsize::new(0));
            let bitis = Instant::now() + Duration::from_secs(4);
            let yoklayicilar: Vec<_> = (0..6)
                .map(|_| {
                    let asili = Arc::clone(&asili);
                    std::thread::spawn(move || {
                        while Instant::now() < bitis {
                            let Ok(mut akis) = std::net::TcpStream::connect(("127.0.0.1", saglik))
                            else {
                                std::thread::sleep(Duration::from_millis(2));
                                continue;
                            };
                            let _ = akis.set_read_timeout(Some(Duration::from_secs(2)));
                            if akis
                                .write_all(b"GET /saglik HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
                                .is_err()
                            {
                                continue;
                            }
                            let mut yanit = Vec::new();
                            if let Err(hata) = akis.read_to_end(&mut yanit) {
                                if matches!(
                                    hata.kind(),
                                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                                ) {
                                    asili.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        }
                    })
                })
                .collect();
            for y in yoklayicilar {
                let _ = y.join();
            }
            let son = Instant::now() + Duration::from_secs(60);
            let pidler: Vec<u32> = loop {
                let govde = saglik_govdesi(saglik).unwrap_or_default();
                if govde.contains("\"canli\":32") || Instant::now() >= son {
                    break govde
                        .split("\"surecler\":[")
                        .nth(1)
                        .and_then(|k| k.split(']').next())
                        .unwrap_or_default()
                        .split(',')
                        .filter_map(|p| p.parse().ok())
                        .collect();
                }
                std::thread::sleep(Duration::from_millis(50));
            };
            let miras = tutulan_soketler(saglik, &pidler);
            // SAFETY: SIGTERM, then SIGKILL, to the test-owned manager's group.
            unsafe { libc::kill(-grup, libc::SIGTERM) };
            let son = Instant::now() + Duration::from_secs(45);
            while yonetici.try_wait().ok().flatten().is_none() && Instant::now() < son {
                std::thread::sleep(Duration::from_millis(50));
            }
            unsafe { libc::kill(-grup, libc::SIGKILL) };
            let _ = yonetici.wait();
            miras_toplam += miras;
            asili_toplam += asili.load(Ordering::Relaxed);
            eksik_grup += usize::from(pidler.len() != 32);
        }
        (
            i64::from(eksik_grup != 0),
            vec![
                format!("miras {miras_toplam}"),
                format!("asili {asili_toplam}"),
            ],
        )
    }
}
