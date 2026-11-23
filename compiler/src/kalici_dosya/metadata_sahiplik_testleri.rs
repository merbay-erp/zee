//! K-376: atomik değiştirmede sahiplik çağrısı yalnız gerektiğinde.
use super::*;

/// K-376: aynı sahip/grupta sahiplik çağrısı yok; ikisinden biri
/// farklıysa eski sahiplik taşınır (davranış korunur).
#[test]
fn ayni_sahipte_sahiplik_cagrisi_yapilmaz() {
    assert!(!sahiplik_degismeli((1000, 1000), (1000, 1000)));
    assert!(sahiplik_degismeli((0, 0), (1000, 1000)));
    assert!(sahiplik_degismeli((1000, 50), (1000, 1000)));
    assert!(sahiplik_degismeli((1001, 1000), (1000, 1000)));
}

/// Gerçek atomik yazımda eski dosya bu sürece aitken sahiplik aynen
/// korunur ve izinler taşınır.
#[test]
fn ayni_sahipli_degistirmede_sahiplik_ve_izin_korunur() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let kok = std::env::temp_dir().join(format!("zk376-sahip-{}", std::process::id()));
    std::fs::create_dir_all(&kok).unwrap();
    let eski_yol = kok.join("eski");
    std::fs::write(&eski_yol, "a").unwrap();
    std::fs::set_permissions(&eski_yol, std::fs::Permissions::from_mode(0o640)).unwrap();
    let gecici_yol = kok.join("gecici");
    let gecici = File::create(&gecici_yol).unwrap();
    let eski = File::open(&eski_yol).unwrap();
    unix_tabanini_koru(&eski, &gecici).unwrap();
    let (e, g) = (eski.metadata().unwrap(), gecici.metadata().unwrap());
    assert_eq!((g.uid(), g.gid()), (e.uid(), e.gid()));
    assert_eq!(g.permissions().mode() & 0o777, 0o640);
    std::fs::remove_dir_all(&kok).unwrap();
}
