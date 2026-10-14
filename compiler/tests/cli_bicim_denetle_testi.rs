//! K-229: biçimle --denetle içerik/izin/mtime yazmadan CI sonucu verir.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static SIRA: AtomicU64 = AtomicU64::new(0);
const KANONIK: &str = "\"merhaba\" yaz\n";
const FARKLI: &str = "\"merhaba\"    yaz\n";
const BILDIRIM: &str =
    "proje \"bicimkabulu\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n";

struct KabulKopyasi(PathBuf);

impl KabulKopyasi {
    fn yeni() -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-k229-bicim-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&yol).unwrap();
        Self(yol)
    }

    fn yaz(&self, ad: &str, kaynak: &str) -> PathBuf {
        let yol = self.0.join(ad);
        std::fs::create_dir_all(yol.parent().unwrap()).unwrap();
        std::fs::write(&yol, kaynak).unwrap();
        yol
    }

    fn komut(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dil"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("biçimleme CLI")
    }

    fn dosyalar(&self) -> Vec<PathBuf> {
        let mut yollar = std::fs::read_dir(&self.0)
            .unwrap()
            .map(|girdi| girdi.unwrap().file_name().into())
            .collect::<Vec<PathBuf>>();
        yollar.sort();
        yollar
    }
}

impl Drop for KabulKopyasi {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("yalnız geçici kabul kopyası silinmeli");
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AnlikGoruntu {
    icerik: Vec<u8>,
    degistirilme: SystemTime,
    salt_okunur: bool,
    #[cfg(unix)]
    izin: u32,
    #[cfg(unix)]
    inode: u64,
}

fn anlik_goruntu(yol: &Path) -> AnlikGoruntu {
    let meta = std::fs::metadata(yol).unwrap();
    #[cfg(unix)]
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    AnlikGoruntu {
        icerik: std::fs::read(yol).unwrap(),
        degistirilme: meta.modified().unwrap(),
        salt_okunur: meta.permissions().readonly(),
        #[cfg(unix)]
        izin: meta.permissions().mode(),
        #[cfg(unix)]
        inode: meta.ino(),
    }
}

fn ayni_kalmali(yollar: &[PathBuf], once: &[AnlikGoruntu]) {
    for (yol, once) in yollar.iter().zip(once) {
        assert_eq!(&anlik_goruntu(yol), once, "{} değişmemeli", yol.display());
    }
}

#[test]
fn kanonik_dosya_iki_bayrak_konumunda_yazmadan_basarilidir() {
    let kopya = KabulKopyasi::yeni();
    let yol = kopya.yaz("kanonik ad.dil", KANONIK);
    let once = anlik_goruntu(&yol);
    let girdiler = kopya.dosyalar();
    for args in [
        ["biçimle", "--denetle", "kanonik ad.dil"],
        ["bicimle", "kanonik ad.dil", "--denetle"],
    ] {
        let sonuc = kopya.komut(&args);
        assert_eq!(sonuc.status.code(), Some(0), "{sonuc:?}");
        assert!(String::from_utf8_lossy(&sonuc.stdout).contains("Zaten biçimli:"));
        assert!(sonuc.stderr.is_empty());
        assert_eq!(anlik_goruntu(&yol), once);
        assert_eq!(kopya.dosyalar(), girdiler);
    }
}

#[test]
fn bicim_farki_salt_okunur_dosyada_bir_ve_ayni_icerik_izin_mtime_verir() {
    let kopya = KabulKopyasi::yeni();
    let yol = kopya.yaz("fark.dil", FARKLI);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&yol, std::fs::Permissions::from_mode(0o440)).unwrap();
    }
    let once = anlik_goruntu(&yol);
    let girdiler = kopya.dosyalar();
    let sonuc = kopya.komut(&["biçimle", "--denetle", "fark.dil"]);
    assert_eq!(sonuc.status.code(), Some(1), "{sonuc:?}");
    assert!(String::from_utf8_lossy(&sonuc.stderr).contains("Biçim farkı: fark.dil"));
    assert!(!String::from_utf8_lossy(&sonuc.stdout).contains("Biçimlendi:"));
    assert_eq!(anlik_goruntu(&yol), once);
    assert_eq!(kopya.dosyalar(), girdiler);
}

#[test]
fn butun_proje_farklari_sirali_raporlanir_hicbir_kaynak_yazilmaz() {
    let kopya = KabulKopyasi::yeni();
    let yollar = vec![
        kopya.yaz("proje.dil", BILDIRIM),
        kopya.yaz("ana.dil", KANONIK),
        kopya.yaz("z.dil", FARKLI),
        kopya.yaz("a.dil", FARKLI),
    ];
    let once = yollar.iter().map(|p| anlik_goruntu(p)).collect::<Vec<_>>();
    let girdiler = kopya.dosyalar();
    let sonuc = kopya.komut(&["biçimle", ".", "--denetle"]);
    assert_eq!(sonuc.status.code(), Some(1), "{sonuc:?}");
    let stderr = String::from_utf8(sonuc.stderr).unwrap();
    let farklar = stderr
        .lines()
        .filter(|s| s.starts_with("Biçim farkı:"))
        .collect::<Vec<_>>();
    assert_eq!(farklar, ["Biçim farkı: ./a.dil", "Biçim farkı: ./z.dil"]);
    assert_eq!(
        String::from_utf8(sonuc.stdout).unwrap().trim(),
        "4 kaynak denetlendi; 2 dosyada biçim farkı."
    );
    ayni_kalmali(&yollar, &once);
    assert_eq!(kopya.dosyalar(), girdiler);
}

#[test]
fn kanonik_proje_sifir_donerek_butun_dosyalari_korur() {
    let kopya = KabulKopyasi::yeni();
    let yollar = vec![
        kopya.yaz("proje.dil", BILDIRIM),
        kopya.yaz("ana.dil", KANONIK),
        kopya.yaz(
            "kaynak/yardimci.dil",
            "işlem birVer\n    TamSayı döndürür\n    1 döndür\n",
        ),
    ];
    let once = yollar.iter().map(|p| anlik_goruntu(p)).collect::<Vec<_>>();
    let sonuc = kopya.komut(&["biçimle", "--denetle", "."]);
    assert_eq!(sonuc.status.code(), Some(0), "{sonuc:?}");
    assert_eq!(
        String::from_utf8(sonuc.stdout).unwrap().trim(),
        "3 kaynak denetlendi; 0 dosyada biçim farkı."
    );
    assert!(sonuc.stderr.is_empty());
    ayni_kalmali(&yollar, &once);
}

#[test]
fn dosya_ve_proje_parse_hatasi_onceki_bicim_farkini_da_yazmaz() {
    let dosya = KabulKopyasi::yeni();
    let bozuk = dosya.yaz("bozuk.dil", "\"kapanmayan\n");
    let once = anlik_goruntu(&bozuk);
    let sonuc = dosya.komut(&["biçimle", "--denetle", "bozuk.dil"]);
    assert_eq!(sonuc.status.code(), Some(1), "{sonuc:?}");
    assert!(String::from_utf8_lossy(&sonuc.stderr).contains("HATA S"));
    assert_eq!(anlik_goruntu(&bozuk), once);

    let proje = KabulKopyasi::yeni();
    let yollar = vec![
        proje.yaz("proje.dil", BILDIRIM),
        proje.yaz("ana.dil", FARKLI),
        proje.yaz("z-bozuk.dil", "\"kapanmayan\n"),
    ];
    let once = yollar.iter().map(|p| anlik_goruntu(p)).collect::<Vec<_>>();
    let girdiler = proje.dosyalar();
    let sonuc = proje.komut(&["biçimle", "--denetle", "."]);
    assert_eq!(sonuc.status.code(), Some(1), "{sonuc:?}");
    assert!(String::from_utf8_lossy(&sonuc.stderr).contains("Biçimlenemedi: ./z-bozuk.dil"));
    ayni_kalmali(&yollar, &once);
    assert_eq!(proje.dosyalar(), girdiler);
}

#[test]
fn json_bilinmeyen_kalir_yol_eksikligi_ve_fazla_yol_yazmadan_iki_verir() {
    let kopya = KabulKopyasi::yeni();
    let yol = kopya.yaz("fark.dil", FARKLI);
    let once = anlik_goruntu(&yol);
    for args in [
        vec!["biçimle", "--json", "fark.dil"],
        vec!["biçimle", "fark.dil", "--json"],
        vec!["biçimle", "--denetle"],
        vec!["biçimle", "--denetle", "fark.dil", "fazla.dil"],
    ] {
        let sonuc = kopya.komut(&args);
        assert_eq!(sonuc.status.code(), Some(2), "{args:?}: {sonuc:?}");
        if args.contains(&"--json") {
            assert!(String::from_utf8_lossy(&sonuc.stderr).contains("Bilinmeyen seçenek: --json"));
        }
        assert_eq!(anlik_goruntu(&yol), once);
    }
}

#[test]
fn normal_bicimle_yazma_davranisini_korur() {
    let kopya = KabulKopyasi::yeni();
    let yol = kopya.yaz("fark.dil", FARKLI);
    let sonuc = kopya.komut(&["biçimle", "fark.dil"]);
    assert_eq!(sonuc.status.code(), Some(0), "{sonuc:?}");
    assert_eq!(std::fs::read_to_string(yol).unwrap(), KANONIK);
    assert!(String::from_utf8_lossy(&sonuc.stdout).contains("Biçimlendi:"));
}

#[cfg(unix)]
#[test]
fn proje_denetimi_symlink_gizli_ve_hedef_dizinlerini_izlemez() {
    use std::os::unix::fs::symlink;
    let kopya = KabulKopyasi::yeni();
    let dis = KabulKopyasi::yeni();
    let bozuk = dis.yaz("dis-bozuk.dil", "\"kapanmayan\n");
    let once = anlik_goruntu(&bozuk);
    kopya.yaz("proje.dil", BILDIRIM);
    kopya.yaz("ana.dil", KANONIK);
    kopya.yaz(".gizli/bozuk.dil", "\"kapanmayan\n");
    kopya.yaz("hedef/bozuk.dil", "\"kapanmayan\n");
    kopya.yaz("target/bozuk.dil", "\"kapanmayan\n");
    symlink(&bozuk, kopya.0.join("bag.dil")).unwrap();
    symlink(&dis.0, kopya.0.join("bag-dizin")).unwrap();
    let sonuc = kopya.komut(&["biçimle", "--denetle", "."]);
    assert_eq!(sonuc.status.code(), Some(0), "{sonuc:?}");
    assert_eq!(
        String::from_utf8(sonuc.stdout).unwrap().trim(),
        "2 kaynak denetlendi; 0 dosyada biçim farkı."
    );
    assert_eq!(anlik_goruntu(&bozuk), once);
    assert!(std::fs::symlink_metadata(kopya.0.join("bag.dil"))
        .unwrap()
        .file_type()
        .is_symlink());
}
