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
