use super::*;
use crate::yetkinlik::veri_yolu_bilesenleri;
use std::io::Read;
use std::os::unix::fs::{symlink, PermissionsExt};

const SIR: &[u8] = b"K394_KOK_DISI_SIR";

struct Dizin(PathBuf);
impl Dizin {
    fn yeni(etiket: &str) -> Self {
        let taban = std::env::temp_dir().canonicalize().unwrap();
        let yol = taban.join(format!("zee-k394-{etiket}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&yol);
        std::fs::create_dir_all(yol.join("kok/ekler")).unwrap();
        std::fs::create_dir_all(yol.join("dis")).unwrap();
        std::fs::write(yol.join("dis/sir"), SIR).unwrap();
        izin(&yol, 0o700);
        izin(&yol.join("kok"), 0o755);
        Dizin(yol)
    }
    fn kok(&self) -> PathBuf {
        self.0.join("kok")
    }
    fn veri(&self) -> VeriKoku {
        VeriKoku::yoldan(&self.kok()).unwrap()
    }
}
impl Drop for Dizin {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn izin(yol: &Path, kip: u32) {
    std::fs::set_permissions(yol, std::fs::Permissions::from_mode(kip)).unwrap();
}

fn coz(koku: &VeriKoku, yol: &str, amac: Amac) -> Result<PathBuf, String> {
    let bilesenler = veri_yolu_bilesenleri(yol).expect("veri yolu")?;
    koku.coz(yol, &bilesenler, amac)
}

fn oku(koku: &VeriKoku, yol: &str) -> Result<Vec<u8>, String> {
    let gercek = coz(koku, yol, Amac::Okuma)?;
    let mut dosya = okuma_icin_ac(&gercek, yol)?;
    let mut icerik = Vec::new();
    dosya.read_to_end(&mut icerik).unwrap();
    Ok(icerik)
}

#[test]
fn veri_koku_goreli_yol_ve_bilesen_sozdizimini_reddeder() {
    for yol in [
        "veri:",
        "veri:/etc/passwd",
        "veri:../dis/sir",
        "veri:ekler/../../dis/sir",
        "veri:ekler//x",
        "veri:ekler/",
        "veri:./x",
        "veri:ekler\\x",
        "veri:C:x",
        "veri:ek\u{0}ler",
    ] {
        assert!(
            veri_yolu_bilesenleri(yol).unwrap().is_err(),
            "{yol:?} kabul edilmemeliydi"
        );
    }
    assert_eq!(
        veri_yolu_bilesenleri("veri:ekler/a-1f").unwrap().unwrap(),
        ["ekler", "a-1f"]
    );
    assert!(veri_yolu_bilesenleri("ekler/a").is_none());
}

#[test]
fn veri_koku_mutlak_sembolik_bagsiz_ve_korunan_olmali() {
    let d = Dizin::yeni("kok");
    assert!(VeriKoku::yoldan(Path::new("goreli/kok"))
        .unwrap_err()
        .contains("mutlak"));
    assert!(VeriKoku::yoldan(Path::new("/"))
        .unwrap_err()
        .contains("kökü"));
    assert!(VeriKoku::yoldan(&d.0.join("yok"))
        .unwrap_err()
        .contains("yok"));
    let bag = d.0.join("bag");
    symlink(d.kok(), &bag).unwrap();
    assert!(VeriKoku::yoldan(&bag).unwrap_err().contains("sembolik bağ"));
    let nokta = PathBuf::from(format!("{}/../kok", d.kok().display()));
    assert!(VeriKoku::yoldan(&nokta).unwrap_err().contains("`..`"));
    // Kökün üst klasörü başkalarınca yazılabilirse kök değiştirilebilir.
    izin(&d.0, 0o777);
    assert!(VeriKoku::yoldan(&d.kok())
        .unwrap_err()
        .contains("grup ya da diğer"));
    izin(&d.0, 0o700);
    assert!(VeriKoku::yoldan(&d.kok()).is_ok());
}

#[test]
fn veri_koku_ara_ve_yaprak_sembolik_bagi_izlemez() {
    let d = Dizin::yeni("bag");
    let koku = d.veri();
    symlink(d.0.join("dis"), d.kok().join("disari")).unwrap();
    symlink(d.0.join("dis/sir"), d.kok().join("ekler/sir")).unwrap();
    let ara = oku(&koku, "veri:disari/sir").unwrap_err();
    assert!(ara.contains("sembolik bağ izlenmez"), "{ara}");
    let yaprak = oku(&koku, "veri:ekler/sir").unwrap_err();
    assert!(yaprak.contains("sembolik bağ izlenmez"), "{yaprak}");
    assert!(coz(&koku, "veri:ekler/sir", Amac::Yazma)
        .unwrap_err()
        .contains("sembolik bağ"));
    for hata in [ara, yaprak] {
        assert!(
            !hata.contains(&*d.0.to_string_lossy()),
            "mutlak yol sızdı: {hata}"
        );
    }
}

#[test]
fn veri_koku_yoklugu_amaca_gore_soyler() {
    let d = Dizin::yeni("yok");
    let koku = d.veri();
    assert_eq!(
        oku(&koku, "veri:ekler/yok").unwrap_err(),
        "\"veri:ekler/yok\" bulunamadı"
    );
    assert_eq!(
        coz(&koku, "veri:ekler/yeni", Amac::Yazma).unwrap(),
        d.kok().join("ekler/yeni")
    );
    let klasor = coz(&koku, "veri:arsiv/x", Amac::Yazma).unwrap_err();
    assert!(klasor.contains("\"veri:arsiv\" yok"), "{klasor}");
}

#[test]
fn veri_koku_hard_link_dis_dosyayi_okumaz() {
    let d = Dizin::yeni("hard");
    let koku = d.veri();
    std::fs::hard_link(d.0.join("dis/sir"), d.kok().join("ekler/kopya")).unwrap();
    let hata = oku(&koku, "veri:ekler/kopya").unwrap_err();
    assert!(hata.contains("hard-link"), "{hata}");
    let dogrudan = okuma_icin_ac(&d.kok().join("ekler/kopya"), "veri:ekler/kopya").unwrap_err();
    assert!(dogrudan.contains("hard-link"), "{dogrudan}");
}

#[test]
fn veri_koku_ata_klasoru_baskasi_yazabiliyorsa_reddeder() {
    let d = Dizin::yeni("ata");
    let koku = d.veri();
    std::fs::write(d.kok().join("ekler/a"), b"ek").unwrap();
    std::fs::write(d.kok().join("tek"), b"tek").unwrap();
    // Paylaşılan yazma son klasörde serbesttir.
    izin(&d.kok().join("ekler"), 0o770);
    assert_eq!(oku(&koku, "veri:ekler/a").unwrap(), b"ek");
    // Kök grup yazılabilirse `ekler` girdisi bağla değiştirilebilir: ret.
    izin(&d.kok(), 0o775);
    let hata = oku(&koku, "veri:ekler/a").unwrap_err();
    assert!(
        hata.contains("paylaşılan yazma yalnız son klasörde"),
        "{hata}"
    );
    // Kökün kendisi doğrudan üst klasörse paylaşılabilir.
    assert_eq!(oku(&koku, "veri:tek").unwrap(), b"tek");
    izin(&d.kok(), 0o755);
}

#[test]
fn veri_koku_yaprak_yarisinda_kok_disi_icerik_okunmaz() {
    let d = Dizin::yeni("yaris");
    let koku = d.veri();
    let yaprak = d.kok().join("ekler/ek");
    let gecici = d.kok().join("ekler/.bag");
    std::fs::write(&yaprak, b"ek").unwrap();
    let dur = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let saldirgan = {
        let (dur, yaprak, gecici, sir) = (dur.clone(), yaprak.clone(), gecici, d.0.join("dis/sir"));
        std::thread::spawn(move || {
            let duz = yaprak.with_file_name(".duz");
            while !dur.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = std::fs::remove_file(&gecici);
                let _ = symlink(&sir, &gecici);
                let _ = std::fs::rename(&gecici, &yaprak);
                let _ = std::fs::write(&duz, b"ek");
                let _ = std::fs::rename(&duz, &yaprak);
            }
        })
    };
    let mut basarili = 0;
    for _ in 0..3000 {
        if let Ok(icerik) = oku(&koku, "veri:ekler/ek") {
            assert_ne!(icerik, SIR, "sembolik bağ yarışı kök dışını okuttu");
            basarili += 1;
        }
    }
    dur.store(true, std::sync::atomic::Ordering::Relaxed);
    saldirgan.join().unwrap();
    assert!(basarili > 0, "yarış hiç normal dosya göstermedi");
}

#[test]
fn veri_koku_yazar_yaprak_yarisinda_kok_disi_icerigi_hedefe_kopyalamaz() {
    // Başka yazar yaprağı kök dışı dosyaya bağla değiştirirken ekleme/geri
    // alma iç okuması bağı izlemez; hedefe hiçbir zaman sır yazılmaz.
    let d = Dizin::yeni("yazar");
    let koku = d.veri();
    let yaprak = coz(&koku, "veri:ekler/gunluk", Amac::Yazma).unwrap();
    std::fs::write(&yaprak, b"").unwrap();
    let dur = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let saldirgan = {
        let (dur, yaprak, sir) = (dur.clone(), yaprak.clone(), d.0.join("dis/sir"));
        std::thread::spawn(move || {
            let bag = yaprak.with_file_name(".bag");
            while !dur.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = std::fs::remove_file(&bag);
                let _ = symlink(&sir, &bag);
                let _ = std::fs::rename(&bag, &yaprak);
                std::thread::yield_now();
            }
        })
    };
    for _ in 0..1500 {
        let _ = crate::kalici_dosya::sinirli_sona_ekle(&yaprak, b"x", 1 << 20);
        let _ = crate::kalici_dosya::atomik_satir_yaz(&yaprak, "y", true);
    }
    dur.store(true, std::sync::atomic::Ordering::Relaxed);
    saldirgan.join().unwrap();
    for girdi in std::fs::read_dir(d.kok().join("ekler")).unwrap() {
        let yol = girdi.unwrap().path();
        if std::fs::symlink_metadata(&yol).unwrap().is_file() {
            let icerik = std::fs::read(&yol).unwrap();
            assert!(
                !icerik.windows(SIR.len()).any(|p| p == SIR),
                "{} kök dışı içerik taşıyor",
                yol.display()
            );
        }
    }
    assert_eq!(
        std::fs::read(d.0.join("dis/sir")).unwrap(),
        SIR,
        "dış dosya değişmedi"
    );
}

#[test]
fn veri_koku_cift_degistirme_yarisi_ekleme_ile_sir_kopyalatmaz() {
    // Deterministik çift değiştirme: önceki içerik okunmadan hemen önce
    // yaprak kök dışı dosyaya bağa, metadata yakalanmadan önce yeniden
    // normal dosyaya çevrilir. İç okuma bağı izleseydi sır + yeni satır
    // paylaşılan hedefe yayımlanırdı.
    use crate::kalici_dosya::test_kancasi::{kur, temizle, Enjeksiyon};
    use crate::kalici_dosya::YazimAdimi;
    let d = Dizin::yeni("cift");
    let koku = d.veri();
    let yaprak = coz(&koku, "veri:ekler/gunluk", Amac::Yazma).unwrap();
    std::fs::write(&yaprak, b"eski\n").unwrap();
    let (bag_yaprak, sir) = (yaprak.clone(), d.0.join("dis/sir"));
    kur(
        YazimAdimi::MevcutOkuma,
        Enjeksiyon::Eylem(Box::new(move || {
            std::fs::remove_file(&bag_yaprak)?;
            symlink(&sir, &bag_yaprak)
        })),
    );
    let duz_yaprak = yaprak.clone();
    kur(
        YazimAdimi::Metadata,
        Enjeksiyon::Eylem(Box::new(move || {
            std::fs::remove_file(&duz_yaprak)?;
            std::fs::write(&duz_yaprak, b"eski\n")
        })),
    );
    let sonuc = crate::kalici_dosya::atomik_satir_yaz(&yaprak, "yeni", true);
    temizle();
    // Hedef normal dosyaysa (yayımlanmış içerik) sır taşımamalı; bağ olarak
    // kaldıysa hiçbir şey yayımlanmamıştır.
    let yayimlanan = if std::fs::symlink_metadata(&yaprak).is_ok_and(|bilgi| bilgi.is_file()) {
        std::fs::read(&yaprak).unwrap()
    } else {
        Vec::new()
    };
    assert!(
        !yayimlanan.windows(SIR.len()).any(|p| p == SIR),
        "çift değiştirme yarışı sırrı hedefe kopyaladı: {sonuc:?}"
    );
    assert!(sonuc.is_err(), "bağa dönen yaprak okunmamalı");
}

#[test]
fn veri_koku_cozerek_ekleme_boy_denetimi_sonrasi_baga_donen_yapragi_okumaz() {
    // K-395 çözerek ekleme hedefi `symlink_metadata` ile boylar, sonra okur.
    // İkisi arasında eş yazar yaprağı kök dışı dosyaya bağa çevirirse okuma
    // bağı izlememeli; aksi hâlde sır + yeni baytlar hedefe yayımlanırdı.
    use crate::kalici_dosya::test_kancasi::{kur, temizle, Enjeksiyon};
    use crate::kalici_dosya::YazimAdimi;
    let d = Dizin::yeni("cozerek");
    let koku = d.veri();
    let yaprak = coz(&koku, "veri:ekler/ek", Amac::Yazma).unwrap();
    std::fs::write(&yaprak, b"eski").unwrap();
    let (bag_yaprak, sir) = (yaprak.clone(), d.0.join("dis/sir"));
    kur(
        YazimAdimi::MevcutOkuma,
        Enjeksiyon::Eylem(Box::new(move || {
            std::fs::remove_file(&bag_yaprak)?;
            symlink(&sir, &bag_yaprak)
        })),
    );
    // Yayın öncesi metadata yakalanmadan yaprak yeniden normal dosya olur
    // (çift değiştirme); yalnız okumanın kendisi bağı reddedebilir.
    let duz_yaprak = yaprak.clone();
    kur(
        YazimAdimi::Metadata,
        Enjeksiyon::Eylem(Box::new(move || {
            std::fs::remove_file(&duz_yaprak)?;
            std::fs::write(&duz_yaprak, b"eski")
        })),
    );
    let sonuc = crate::kalici_dosya::sinirli_sona_ekle(&yaprak, b"yeni", 32 << 20);
    temizle();
    let yayimlanan = if std::fs::symlink_metadata(&yaprak).is_ok_and(|bilgi| bilgi.is_file()) {
        std::fs::read(&yaprak).unwrap()
    } else {
        Vec::new()
    };
    assert!(
        !yayimlanan.windows(SIR.len()).any(|p| p == SIR),
        "çözerek ekleme çift değiştirmede sırrı hedefe kopyaladı: {sonuc:?}"
    );
    assert!(sonuc.is_err(), "bağa dönen yaprak okunmamalı: {sonuc:?}");
    assert_eq!(std::fs::read(d.0.join("dis/sir")).unwrap(), SIR);
}

#[test]
fn veri_koku_hard_link_korumasi_okuma_hatalarini_ayirir() {
    // K-422 (ZM-F051): Ubuntu 22.04'te `/proc/sys/fs/protected_hardlinks`
    // `0600 root`; ayrıcalıksız hizmet EACCES alır. İzin yokluğu koruma
    // bilinmiyor kipidir (kök kabul), `/proc` yokluğu ve `0` ret.
    use std::io::Error;
    let karar = platform::koruma_karari;
    assert_eq!(karar(Ok("1\n".into())), Ok(()));
    let kapali = karar(Ok("0\n".into())).unwrap_err();
    assert!(kapali.contains("çekirdek koruması kapalı"), "{kapali}");
    for kod in [libc::EACCES, libc::EPERM] {
        assert_eq!(karar(Err(Error::from_raw_os_error(kod))), Ok(()), "{kod}");
    }
    for kod in [libc::ENOENT, libc::ENOTDIR] {
        let yok = karar(Err(Error::from_raw_os_error(kod))).unwrap_err();
        assert!(yok.contains("/proc bağlı değil"), "{yok}");
    }
    let diger = karar(Err(Error::from_raw_os_error(libc::EIO))).unwrap_err();
    assert!(
        diger.contains("okunamadı (") && !diger.contains("/proc bağlı değil"),
        "gerçek neden söylenmeli: {diger}"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn veri_koku_koruma_okunamazsa_kok_kabul_edilir_kapaliysa_reddedilir() {
    use platform::koruma_kancasi::kur;
    let d = Dizin::yeni("koruma");
    {
        let _k = kur(|| Err(std::io::Error::from_raw_os_error(libc::EACCES)));
        assert!(VeriKoku::yoldan(&d.kok()).is_ok());
    }
    {
        let _k = kur(|| Err(std::io::Error::from_raw_os_error(libc::ENOENT)));
        let hata = VeriKoku::yoldan(&d.kok()).unwrap_err();
        assert!(hata.contains("/proc bağlı değil"), "{hata}");
    }
    {
        let _k = kur(|| Ok("0\n".into()));
        let hata = VeriKoku::yoldan(&d.kok()).unwrap_err();
        assert!(hata.contains("kapalı"), "{hata}");
    }
    {
        let _k = kur(|| Ok("1\n".into()));
        assert!(VeriKoku::yoldan(&d.kok()).is_ok());
    }
}

/// Paylaşılan son klasöre eş hizmetin bağladığı dış dosya ve yazım hedefi.
fn sert_bagli(d: &Dizin, ad: &str) -> PathBuf {
    let yaprak = d.kok().join("ekler").join(ad);
    std::fs::hard_link(d.0.join("dis/sir"), &yaprak).unwrap();
    yaprak
}

fn sir_aklanmadi(d: &Dizin) {
    for girdi in std::fs::read_dir(d.kok().join("ekler")).unwrap() {
        let yol = girdi.unwrap().path();
        let bilgi = std::fs::symlink_metadata(&yol).unwrap();
        if bilgi.is_file() && std::os::unix::fs::MetadataExt::nlink(&bilgi) == 1 {
            let icerik = std::fs::read(&yol).unwrap();
            assert!(
                !icerik.windows(SIR.len()).any(|p| p == SIR),
                "{} dış içeriği tek bağlı dosyaya akladı",
                yol.display()
            );
        }
    }
    assert_eq!(std::fs::read(d.0.join("dis/sir")).unwrap(), SIR);
}

#[test]
fn veri_koku_tek_bag_kipinde_sert_bagli_yaprak_yazima_aklanamaz() {
    // K-422: koruma bilinmiyor kipinde (ya da macOS'ta) eş hizmet yazarın
    // okuyabildiği dış dosyayı paylaşılan klasöre sert bağlayabilir. Ekleme,
    // üzerine yazma, eylem yedeği, taşıma/silme izi o içeriği yeni tek bağlı
    // dosyaya kopyalamamalı; yalnız bağ girdisini kaldıran silme serbesttir.
    use crate::kalici_dosya as k;
    let d = Dizin::yeni("tekbag");
    let gunluk = sert_bagli(&d, "gunluk");
    let hatalar: Vec<std::io::Result<()>> = k::tek_bag_kipinde(|| {
        vec![
            k::atomik_satir_yaz(&gunluk, "ek", true),
            k::atomik_satir_yaz(&gunluk, "ezme", false),
            k::atomik_satir_yaz_izli(&gunluk, "iz", false).map(|_| ()),
            k::sinirli_sona_ekle(&gunluk, b"bayt", 1 << 20).map(|_| ()),
            k::base64_satirlari_sona_ekle(&mut &b"abc"[..], 3, &gunluk, 1 << 20).map(|_| ()),
            k::atomik_sil_izli(&gunluk).map(|_| ()),
            k::atomik_tasi_izli(&gunluk, &d.kok().join("ekler/tasinan")).map(|_| ()),
        ]
    });
    for (sira, sonuc) in hatalar.into_iter().enumerate() {
        let hata = sonuc.expect_err(&format!("{sira}. yazım sert bağı kabul etti"));
        assert!(hata.to_string().contains("sert bağ"), "{sira}: {hata}");
    }
    sir_aklanmadi(&d);
    assert!(!d.kok().join("ekler/tasinan").exists());
    // Bağ girdisini kaldırmak dış dosyayı değiştirmez ve kopyalamaz.
    assert!(k::tek_bag_kipinde(|| k::atomik_sil(&gunluk)).unwrap());
    sir_aklanmadi(&d);
}

#[test]
fn veri_koku_tek_bag_kipinde_kilit_adina_baglanan_dis_dosya_kilit_sayilmaz() {
    use crate::kalici_dosya as k;
    let d = Dizin::yeni("kilit");
    sert_bagli(&d, ".zee-yazma-kilidi");
    let hedef = d.kok().join("ekler/yeni");
    let hata = k::tek_bag_kipinde(|| k::atomik_satir_yaz(&hedef, "x", false)).unwrap_err();
    assert!(hata.to_string().contains("sert bağ"), "{hata}");
    assert!(!hedef.exists());
    sir_aklanmadi(&d);
}

#[test]
fn veri_koku_tek_bag_kipi_yalniz_veri_yazimini_kapsar() {
    // Kip dışında (proje dosyası) sert bağlı hedefe ekleme önceki davranışta
    // kalır: içerik okunup yeni dosya yayımlanır. Kip iş parçacığına özeldir
    // ve kapanınca sızmaz.
    use crate::kalici_dosya as k;
    let d = Dizin::yeni("kapsam");
    let gunluk = sert_bagli(&d, "proje-gunluk");
    k::tek_bag_kipinde(|| ());
    k::atomik_satir_yaz(&gunluk, "ek", true).unwrap();
    let mut beklenen = SIR.to_vec();
    beklenen.extend_from_slice(b"ek\n");
    assert_eq!(std::fs::read(&gunluk).unwrap(), beklenen);
    let okuyucu = crate::kaynak_sinirlari::bagsiz_okuma::kalici_okuyucu_ac;
    let bag = sert_bagli(&d, "ikinci");
    assert!(
        okuyucu(&bag).is_ok(),
        "kip dışında EXDEV okuyucusu eski davranışta"
    );
    let hata = k::tek_bag_kipinde(|| okuyucu(&bag)).unwrap_err();
    assert!(hata.to_string().contains("sert bağ"), "{hata}");
    let sembolik = d.kok().join("ekler/sembolik");
    symlink(d.0.join("dis/sir"), &sembolik).unwrap();
    assert!(k::tek_bag_kipinde(|| okuyucu(&sembolik)).is_err());
    std::fs::write(d.kok().join("ekler/duz"), b"duz").unwrap();
    assert!(k::tek_bag_kipinde(|| okuyucu(&d.kok().join("ekler/duz"))).is_ok());
}
