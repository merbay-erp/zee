//! Proje modeli (K-076): proje.dil bildirimi ve klasör-temelli CLI akışı.

use dil::proje::{bildirimi_oku, yerel_bagimliliklari_guncelle, ProjeBildirimi};
use std::path::{Path, PathBuf};
use std::process::Command;

struct GeciciKlasor(PathBuf);

impl GeciciKlasor {
    fn yeni() -> Self {
        let benzersiz = format!(
            "zee-proje-testi-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("saat")
                .as_nanos()
        );
        let yol = std::env::temp_dir().join(benzersiz);
        std::fs::create_dir(&yol).expect("geçici klasör");
        Self(yol)
    }

    fn yol(&self) -> &Path {
        &self.0
    }
}

impl Drop for GeciciKlasor {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn bildirim_gecerli_zee_kaynagidir() {
    let kaynak =
        "proje \"stok-paneli\" olsun\nsürüm \"1.2.3\" olsun\ngiriş \"kaynak/ana.dil\" olsun\n";
    assert_eq!(
        bildirimi_oku(kaynak).expect("bildirim geçmeli"),
        ProjeBildirimi {
            ad: "stok-paneli".into(),
            surum: "1.2.3".into(),
            giris: "kaynak/ana.dil".into(),
            yerel_bagimliliklar: Vec::new(),
        }
    );
}

#[test]
fn bildirim_yerel_bagimliliklari_tek_gercek_kaynaktan_alir() {
    let kaynak = "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"../grafik\", \"../ortak\" listesi olsun\n";
    assert_eq!(
        bildirimi_oku(kaynak)
            .expect("yerel bağımlılıklar geçmeli")
            .yerel_bagimliliklar,
        vec!["../grafik", "../ortak"]
    );

    let kotu = "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"/mutlak\" listesi olsun\n";
    assert_eq!(
        bildirimi_oku(kotu).expect_err("mutlak bağımlılık yolu").kod,
        "P005"
    );
}

#[test]
fn bagimlilik_guncellemesi_yorumlari_korur_ve_yollari_siralar() {
    let kaynak = "# özenle korunacak yorum\n\nproje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n";
    let guncel = yerel_bagimliliklari_guncelle(
        kaynak,
        &["../zengin".into(), "../ortak".into(), "../zengin".into()],
    )
    .expect("güncellenmeli");
    assert!(guncel.starts_with("# özenle korunacak yorum\n"));
    assert!(guncel.contains("yerel_bağımlılıklar \"../ortak\", \"../zengin\" listesi olsun\n"));
    assert_eq!(
        bildirimi_oku(&guncel)
            .expect("üretilen bildirim")
            .yerel_bagimliliklar,
        vec!["../ortak", "../zengin"]
    );
}

#[test]
fn bildirim_yalniz_tanimli_alanlari_kabul_eder() {
    let kaynak = "proje \"x\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nlisans \"özel\" olsun\n";
    assert_eq!(
        bildirimi_oku(kaynak).expect_err("alan reddedilmeli").kod,
        "P001"
    );
}

#[test]
fn eksik_alan_ogretici_tanidir() {
    let kaynak = "proje \"x\" olsun\ngiriş \"ana.dil\" olsun\n";
    let hata = bildirimi_oku(kaynak).expect_err("sürüm eksik");
    assert_eq!(hata.kod, "P002");
    assert!(hata.mesaj.contains("sürüm"));
}

#[test]
fn surum_uc_sayili_ve_giris_guvenli_olmali() {
    let kotu_surum = "proje \"x\" olsun\nsürüm \"v1\" olsun\ngiriş \"ana.dil\" olsun\n";
    assert_eq!(
        bildirimi_oku(kotu_surum)
            .expect_err("sürüm reddedilmeli")
            .kod,
        "P003"
    );

    let kotu_giris = "proje \"x\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"../ana.dil\" olsun\n";
    assert_eq!(
        bildirimi_oku(kotu_giris)
            .expect_err("kaçış reddedilmeli")
            .kod,
        "P004"
    );
}

#[test]
fn cli_proje_klasorunu_calistirir_denetler_ve_dener() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.yol().join("proje.dil"),
        "proje \"hesap\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::write(
        gecici.yol().join("yardimci.dil"),
        "işlem iki katını bul\n    sayıyı al\n    sonucu sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n",
    )
    .expect("birim");
    std::fs::write(
        gecici.yol().join("ana.dil"),
        "yardimci birimini kullan\n\n\"veri.txt\" dosyasına \"projenin verisi\" yaz\nsonuç 21 için iki katını bul olsun\nsonucu yaz\n\ntest \"iki katı\"\n    x 3 için iki katını bul olsun\n    x 6 ya eşit olmalı\n",
    )
    .expect("giriş");

    let ikili = env!("CARGO_BIN_EXE_dil");
    let calistir = Command::new(ikili)
        .arg("çalıştır")
        .arg(gecici.yol())
        .output()
        .expect("çalıştır");
    assert!(
        calistir.status.success(),
        "{}",
        String::from_utf8_lossy(&calistir.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&calistir.stdout), "42\n");
    assert_eq!(
        std::fs::read_to_string(gecici.yol().join("veri.txt")).expect("veri proje kökünde"),
        "projenin verisi\n"
    );

    let denetle = Command::new(ikili)
        .arg("denetle")
        .arg(gecici.yol())
        .output()
        .expect("denetle");
    assert!(
        denetle.status.success(),
        "{}",
        String::from_utf8_lossy(&denetle.stderr)
    );

    let dene = Command::new(ikili)
        .arg("dene")
        .arg(gecici.yol())
        .output()
        .expect("dene");
    assert!(
        dene.status.success(),
        "{}",
        String::from_utf8_lossy(&dene.stderr)
    );
    assert!(String::from_utf8_lossy(&dene.stdout).contains("1 test: 1 geçti, 0 kaldı"));
}

#[test]
fn yeni_komutu_proje_bildirimi_uretir() {
    let gecici = GeciciKlasor::yeni();
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .current_dir(gecici.yol())
        .args(["yeni", "ilk-projem"])
        .output()
        .expect("yeni");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );

    let proje = gecici.yol().join("ilk-projem");
    let bildirim = std::fs::read_to_string(proje.join("proje.dil")).expect("proje.dil");
    assert_eq!(
        bildirimi_oku(&bildirim).expect("üretilen bildirim").ad,
        "ilk-projem"
    );
    assert!(bildirim.contains("yerel_bağımlılıklar boş liste olsun"));
    assert!(
        proje.join("proje.kilit").is_file(),
        "iskelet kilitli başlamalı"
    );

    let dene = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("dene")
        .arg(&proje)
        .output()
        .expect("üretilen projeyi dene");
    assert!(
        dene.status.success(),
        "{}",
        String::from_utf8_lossy(&dene.stderr)
    );
}

#[test]
fn yerel_paketler_kokenli_yuklenir_ve_icerikle_kilitlenir() {
    let gecici = GeciciKlasor::yeni();
    let temel = gecici.yol().join("temel");
    let hesap = gecici.yol().join("hesap");
    let uygulama = gecici.yol().join("uygulama");
    for klasor in [&temel, &hesap, &uygulama] {
        std::fs::create_dir(klasor).expect("proje klasörü");
    }

    std::fs::write(
        temel.join("proje.dil"),
        "proje \"temel\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"temel.dil\" olsun\n",
    )
    .expect("temel bildirim");
    std::fs::write(temel.join("temel.dil"), "işlem üç ver\n    3 döndür\n").expect("temel kaynak");

    std::fs::write(
        hesap.join("proje.dil"),
        "proje \"hesap\" olsun\nsürüm \"2.1.0\" olsun\ngiriş \"paket.dil\" olsun\nyerel_bağımlılıklar \"../temel\" listesi olsun\n",
    )
    .expect("hesap bildirim");
    std::fs::write(hesap.join("yardimci.dil"), "işlem iki ver\n    2 döndür\n")
        .expect("paket içi birim");
    std::fs::write(
        hesap.join("paket.dil"),
        "yardimci birimini kullan\ntemel paketini kullan\n\n\"paketin üst düzeyi çalışmamalı\" yaz\n\nişlem toplam ver\n    a iki ver olsun\n    b üç ver olsun\n    toplam a ile b nin toplamı olsun\n    toplamı döndür\n",
    )
    .expect("hesap giriş");

    std::fs::write(
        uygulama.join("proje.dil"),
        "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"../hesap\" listesi olsun\n",
    )
    .expect("uygulama bildirim");
    std::fs::write(
        uygulama.join("ana.dil"),
        "hesap paketini kullan\n\nsonuç toplam ver olsun\nsonucu yaz\n",
    )
    .expect("uygulama giriş");

    let ikili = env!("CARGO_BIN_EXE_dil");
    let kilitsiz = Command::new(ikili)
        .args(["çalıştır", uygulama.to_str().expect("utf8")])
        .output()
        .expect("kilitsiz çalıştır");
    assert!(!kilitsiz.status.success());
    assert!(String::from_utf8_lossy(&kilitsiz.stderr).contains("P008"));

    let kilitle = Command::new(ikili)
        .args(["kilitle", uygulama.to_str().expect("utf8")])
        .output()
        .expect("kilitle");
    assert!(
        kilitle.status.success(),
        "{}",
        String::from_utf8_lossy(&kilitle.stderr)
    );
    let ilk_kilit = std::fs::read_to_string(uygulama.join("proje.kilit")).expect("kilit");
    assert!(ilk_kilit.contains("paket \"hesap\" \"2.1.0\""));
    assert!(ilk_kilit.contains("paket \"temel\" \"1.0.0\""));
    assert!(ilk_kilit.contains("sha256:"));
    assert!(
        !ilk_kilit.contains(&gecici.yol().to_string_lossy().into_owned()),
        "kilit makineye özgü mutlak yol taşımamalı"
    );
    let paketler = Command::new(ikili)
        .args(["paketler", uygulama.to_str().expect("utf8")])
        .output()
        .expect("paket grafiği");
    assert!(paketler.status.success());
    let paket_ciktisi = String::from_utf8_lossy(&paketler.stdout);
    assert!(paket_ciktisi.contains("doğrudan: hesap 2.1.0"));
    assert!(paket_ciktisi.contains("geçişli: temel 1.0.0"));

    let calistir = Command::new(ikili)
        .args(["çalıştır", uygulama.to_str().expect("utf8")])
        .output()
        .expect("paketli çalıştır");
    assert!(
        calistir.status.success(),
        "{}",
        String::from_utf8_lossy(&calistir.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&calistir.stdout), "5\n");

    // Paket içeriği değişince eski kilit sessizce kabul edilmez.
    std::fs::write(hesap.join("yardimci.dil"), "işlem iki ver\n    4 döndür\n")
        .expect("paket değişikliği");
    let bayat = Command::new(ikili)
        .args(["denetle", uygulama.to_str().expect("utf8")])
        .output()
        .expect("bayat kilit");
    assert!(!bayat.status.success());
    assert!(String::from_utf8_lossy(&bayat.stderr).contains("P008"));

    let yeniden = Command::new(ikili)
        .args(["kilitle", uygulama.to_str().expect("utf8")])
        .output()
        .expect("yeniden kilitle");
    assert!(yeniden.status.success());
    let ikinci_kilit = std::fs::read_to_string(uygulama.join("proje.kilit")).expect("yeni kilit");
    assert_ne!(ilk_kilit, ikinci_kilit, "içerik özeti değişmeli");

    let tekrar = Command::new(ikili)
        .args(["kilitle", uygulama.to_str().expect("utf8")])
        .output()
        .expect("deterministik kilitle");
    assert!(tekrar.status.success());
    assert_eq!(
        ikinci_kilit,
        std::fs::read_to_string(uygulama.join("proje.kilit")).expect("aynı kilit")
    );

    // Geçişli paket grafikte bulunsa da yalnız doğrudan bağımlılık alınabilir.
    std::fs::write(
        uygulama.join("ana.dil"),
        "temel paketini kullan\n\nsonuç üç ver olsun\nsonucu yaz\n",
    )
    .expect("doğrudan sınır");
    let gecisli = Command::new(ikili)
        .args(["denetle", uygulama.to_str().expect("utf8")])
        .output()
        .expect("geçişli bağımlılık denetimi");
    assert!(!gecisli.status.success());
    assert!(String::from_utf8_lossy(&gecisli.stderr).contains("A011"));
}

#[test]
fn yerel_bagimlilik_dongusu_kilitlenmez() {
    let gecici = GeciciKlasor::yeni();
    let a = gecici.yol().join("a");
    let b = gecici.yol().join("b");
    std::fs::create_dir(&a).expect("a");
    std::fs::create_dir(&b).expect("b");
    std::fs::write(
        a.join("proje.dil"),
        "proje \"a\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"../b\" listesi olsun\n",
    )
    .expect("a bildirim");
    std::fs::write(a.join("ana.dil"), "\"a\" yaz\n").expect("a kaynak");
    std::fs::write(
        b.join("proje.dil"),
        "proje \"b\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"../a\" listesi olsun\n",
    )
    .expect("b bildirim");
    std::fs::write(b.join("ana.dil"), "\"b\" yaz\n").expect("b kaynak");

    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["kilitle", a.to_str().expect("utf8")])
        .output()
        .expect("döngü kilitle");
    assert!(!cikti.status.success());
    assert!(String::from_utf8_lossy(&cikti.stderr).contains("P007"));
    assert!(!a.join("proje.kilit").exists());
}

#[test]
fn ekle_komutu_once_dogrular_sonra_bildirimi_ve_kilidi_gunceller() {
    let gecici = GeciciKlasor::yeni();
    let paket = gecici.yol().join("hesap");
    let uygulama = gecici.yol().join("uygulama");
    std::fs::create_dir(&paket).expect("paket");
    std::fs::create_dir(&uygulama).expect("uygulama");
    std::fs::write(
        paket.join("proje.dil"),
        "proje \"hesap\" olsun\nsürüm \"1.3.0\" olsun\ngiriş \"paket.dil\" olsun\n",
    )
    .expect("paket bildirim");
    std::fs::write(paket.join("paket.dil"), "işlem yedi ver\n    7 döndür\n")
        .expect("paket kaynak");
    std::fs::write(
        uygulama.join("proje.dil"),
        "# bu yorum kaybolmamalı\n\nproje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .expect("uygulama bildirim");
    std::fs::write(
        uygulama.join("ana.dil"),
        "hesap paketini kullan\n\nsonuç yedi ver olsun\nsonucu yaz\n",
    )
    .expect("uygulama kaynak");

    let ikili = env!("CARGO_BIN_EXE_dil");
    let ekle = Command::new(ikili)
        .current_dir(gecici.yol())
        .args(["ekle", "hesap", "uygulama"])
        .output()
        .expect("paket ekle");
    assert!(
        ekle.status.success(),
        "{}",
        String::from_utf8_lossy(&ekle.stderr)
    );
    assert!(String::from_utf8_lossy(&ekle.stdout).contains("hesap 1.3.0"));
    let bildirim = std::fs::read_to_string(uygulama.join("proje.dil")).expect("bildirim");
    assert!(bildirim.starts_with("# bu yorum kaybolmamalı\n"));
    assert!(bildirim.contains("yerel_bağımlılıklar \"../hesap\" listesi olsun"));
    assert!(uygulama.join("proje.kilit").is_file());

    let calistir = Command::new(ikili)
        .args(["çalıştır", uygulama.to_str().expect("utf8")])
        .output()
        .expect("eklenen paketle çalıştır");
    assert!(calistir.status.success());
    assert_eq!(String::from_utf8_lossy(&calistir.stdout), "7\n");

    // Aynı gerçek kök farklı yazımla verilse de ikinci kez eklenmez.
    let yine = Command::new(ikili)
        .current_dir(gecici.yol())
        .args(["ekle", "./hesap", "./uygulama"])
        .output()
        .expect("yinelenen ekle");
    assert!(yine.status.success());
    assert!(String::from_utf8_lossy(&yine.stdout).contains("zaten ekli"));
    assert_eq!(
        bildirim,
        std::fs::read_to_string(uygulama.join("proje.dil")).expect("aynı bildirim")
    );

    // Kendisini eklemek döngüdür; doğrulama yazmadan önce yapıldığı için iki
    // proje dosyası da byte-byte aynı kalır.
    let onceki_kilit = std::fs::read(uygulama.join("proje.kilit")).expect("önceki kilit");
    let dongu = Command::new(ikili)
        .current_dir(gecici.yol())
        .args(["ekle", "uygulama", "uygulama"])
        .output()
        .expect("döngülü ekle");
    assert!(!dongu.status.success());
    assert!(
        String::from_utf8_lossy(&dongu.stderr).contains("P007"),
        "{}",
        String::from_utf8_lossy(&dongu.stderr)
    );
    assert_eq!(
        bildirim,
        std::fs::read_to_string(uygulama.join("proje.dil")).expect("geri alınmış bildirim")
    );
    assert_eq!(
        onceki_kilit,
        std::fs::read(uygulama.join("proje.kilit")).expect("aynı kilit")
    );

    // Kaynakta kullanım varken güvenli kaldırma durur ve dosyaları korur.
    let kullanilirken = Command::new(ikili)
        .current_dir(gecici.yol())
        .args(["çıkar", "hesap", "uygulama"])
        .output()
        .expect("kullanılanı çıkar");
    assert!(!kullanilirken.status.success());
    assert!(String::from_utf8_lossy(&kullanilirken.stderr).contains("P010"));
    assert_eq!(
        bildirim,
        std::fs::read_to_string(uygulama.join("proje.dil")).expect("korunan bildirim")
    );

    std::fs::write(uygulama.join("ana.dil"), "\"paketsiz\" yaz\n")
        .expect("paket kullanımını kaldır");
    let cikar = Command::new(ikili)
        .current_dir(gecici.yol())
        .args(["çıkar", "hesap", "uygulama"])
        .output()
        .expect("paketi çıkar");
    assert!(
        cikar.status.success(),
        "{}",
        String::from_utf8_lossy(&cikar.stderr)
    );
    let son_bildirim =
        std::fs::read_to_string(uygulama.join("proje.dil")).expect("paketsiz bildirim");
    assert!(son_bildirim.contains("yerel_bağımlılıklar boş liste olsun"));
    assert!(!son_bildirim.contains("../hesap"));
    let bos_liste = Command::new(ikili)
        .args(["paketler", uygulama.to_str().expect("utf8")])
        .output()
        .expect("boş paket grafiği");
    assert!(bos_liste.status.success());
    assert!(String::from_utf8_lossy(&bos_liste.stdout).contains("Bağımlılık yok"));
}

#[test]
fn guvenli_bayragi_program_argumanlarina_sizmaz() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.yol().join("proje.dil"),
        "proje \"arguman\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::write(
        gecici.yol().join("ana.dil"),
        "gelenler komut satırından gelenler olsun\nher gelen için\n    geleni yaz\n",
    )
    .expect("giriş");

    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(gecici.yol())
        .arg("--güvenli")
        .arg("merhaba")
        .output()
        .expect("güvenli çalıştır");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&cikti.stdout), "merhaba\n");
}

#[test]
fn bicimle_komutu_projenin_butun_kaynaklarini_bicimler() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.yol().join("proje.dil"),
        "proje    \"biçim\"    olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::write(gecici.yol().join("ana.dil"), "\"merhaba\"    yaz\n").expect("giriş");
    std::fs::create_dir(gecici.yol().join("kaynak")).expect("alt klasör");
    std::fs::write(
        gecici.yol().join("kaynak/yardimci.dil"),
        "işlem  bir ver\n    1   döndür\n",
    )
    .expect("birim");

    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("biçimle")
        .arg(gecici.yol())
        .output()
        .expect("biçimle");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    assert!(
        String::from_utf8_lossy(&cikti.stdout).contains("3 kaynak denetlendi; 3 dosya biçimlendi")
    );
    assert_eq!(
        std::fs::read_to_string(gecici.yol().join("ana.dil")).expect("giriş"),
        "\"merhaba\" yaz\n"
    );
    assert_eq!(
        std::fs::read_to_string(gecici.yol().join("kaynak/yardimci.dil")).expect("birim"),
        "işlem bir ver\n    1 döndür\n"
    );
}
