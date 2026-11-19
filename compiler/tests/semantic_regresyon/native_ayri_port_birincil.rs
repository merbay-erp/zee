//! K-341: elle kurulan ayrı portlu `--web-worker-port` süreçlerinin aynı
//! kökte yalnız birini `birincil süreç` saydığını gerçek native `dil`
//! süreçleriyle sınar. Süreç 1 ve 2 açılır, süreç 1 öldürülüp aynı kapıda
//! yeniden açılır (süreç 3). Proje Cargo hedef klasöründe geçici kopyadır
//! (`.zee` süreç kilitleri depo ağacına yazılmaz). ToplayanIo süreç
//! yönetimini taklit edemediği için toplu replay bu kipi atlar.
#![allow(dead_code)]

use super::native_web_kopma_fixture::{bos_kapi, tam_istek, worker_baslat};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};

struct Temizle(PathBuf);

impl Drop for Temizle {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Surec(Child);

impl Drop for Surec {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn rol(kapi: u16) -> String {
    tam_istek(kapi, "GET /rol", "198.51.100.7")
        .and_then(|yanit| yanit.split("\r\n\r\n").nth(1).map(str::to_string))
        .unwrap_or_else(|| "-".into())
}

/// `(0, ["1 <rol>", "2 <rol>", "3 <rol>"])` döndürür; rol, sürecin ilk
/// `GET /rol` isteğinde gördüğü `birincil süreç` değeridir.
pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    let ek = format!(
        "zee-k341-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    );
    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(ek);
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    let ana = proje.join("ana.dil");
    std::fs::copy(kaynak, &ana).expect("kaynak kopyası");
    let (kapi1, kapi2) = (bos_kapi(), bos_kapi());
    let mut bir = Surec(worker_baslat(&ana, kapi1, Stdio::null));
    let rol1 = rol(kapi1);
    let iki = Surec(worker_baslat(&ana, kapi2, Stdio::null));
    let rol2 = rol(kapi2);
    let _ = bir.0.kill();
    let _ = bir.0.wait();
    drop(bir);
    let uc = Surec(worker_baslat(&ana, kapi1, Stdio::null));
    let rol3 = rol(kapi1);
    drop((iki, uc));
    (
        0,
        vec![
            format!("1 {rol1}"),
            format!("2 {rol2}"),
            format!("3 {rol3}"),
        ],
    )
}
