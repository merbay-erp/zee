//! K-338: `--web-isci-sayisi` yöneticisinin art arda hızlı çıkış sınırını
//! gerçek native `dil` yönetici ve işçi süreçleriyle sınar. Korpus kipi
//! `native-isci-grubu`; ToplayanIo süreç yönetimini taklit edemediği için
//! toplu replay bu kipi atlar. Unix dışında sahte başarı yoktur. K-341:
//! proje Cargo hedef klasöründe koşu başına geçici kopyadır; birincil işçinin
//! `.zee` süreç kilitleri depo ağacına yazılmaz ve eşzamanlı iki koşu aynı
//! kökte birbirini canlı katılımcı görmez.
#![allow(dead_code)]

use std::path::Path;

/// `(yönetici çıkışı, satırlar)` döndürür: `yuva 2 başlatma <n>` (ikinci
/// yuvanın kaç kez başlatıldığı) ve `durma <n>` (günlükteki art arda çıkış
/// sayısı). Kaynağın kapısı yok sayılır; yönetici boş bir kapı alır.
pub fn calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    #[cfg(not(unix))]
    {
        let _ = kaynak_yolu;
        panic!("actual Unix inherited listener capability unavailable; no synthetic pass");
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};

        let proje = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "zee-k338-{}-{}",
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
        let kapi = super::native_web_kopma_fixture::bos_kapi();
        let mut cocuk = Command::new(env!("CARGO_BIN_EXE_dil"))
            .args([
                "çalıştır",
                "--web-proxy",
                super::native_web_kopma_fixture::PROXY,
                "--web-worker-port",
                &kapi.to_string(),
                "--web-isci-sayisi",
                "2",
                ana.to_str().expect("UTF-8 kaynak yolu"),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .expect("yönetici süreci");
        let mut boru = cocuk.stderr.take().expect("yönetici stderr borusu");
        let (gonder, al) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            use std::io::Read;
            let mut metin = String::new();
            let _ = boru.read_to_string(&mut metin);
            let _ = gonder.send(metin);
        });
        let grup = cocuk.id() as libc::pid_t;
        let son = Instant::now() + Duration::from_secs(90);
        let durum = loop {
            if let Some(durum) = cocuk.try_wait().expect("yönetici durumu") {
                break durum;
            }
            if Instant::now() >= son {
                let _ = cocuk.kill();
                break cocuk.wait().expect("yönetici sonucu");
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        // Yönetici çıktı; geride kalan işçi borunun yazma ucunu tutamasın.
        // SAFETY: SIGKILL to the reaped manager's own process group; PID'i
        // biçildiğinden yalnız aynı gruptaki sahipsiz işçilere ulaşır.
        unsafe { libc::kill(-grup, libc::SIGKILL) };
        let metin = al
            .recv_timeout(Duration::from_secs(15))
            .expect("yönetici stderr borusu 15 sn içinde kapanmadı");
        let baslatma = metin.matches("zee-web işçi 2 başladı").count();
        let durma = metin
            .split("zee-web işçi 2 art arda ")
            .nth(1)
            .and_then(|k| k.split(' ').next())
            .unwrap_or("-")
            .to_string();
        (
            i64::from(durum.code().unwrap_or(-1)),
            vec![
                format!("yuva 2 başlatma {baslatma}"),
                format!("durma {durma}"),
            ],
        )
    }
}
