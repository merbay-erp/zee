//! Proje modeli (K-076): proje.dil bildirimi ve klasör-temelli CLI akışı.

use dil::proje::{
    bildirimi_oku, uzak_bagimliliklari_guncelle, yerel_bagimliliklari_guncelle, ProjeBildirimi,
    RegistryBildirimi, UzakBagimlilik,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static GECICI_KLASOR_SIRASI: AtomicU64 = AtomicU64::new(0);

struct GeciciKlasor(PathBuf);

impl GeciciKlasor {
    fn yeni() -> Self {
        let benzersiz = format!(
            "zee-proje-testi-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("saat")
                .as_nanos(),
            GECICI_KLASOR_SIRASI.fetch_add(1, Ordering::Relaxed)
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
fn gercek_io_eylem_hatasinda_dosyalari_geri_alir() {
    let gecici = GeciciKlasor::yeni();
    let kaynak = gecici.yol().join("geri-al.dil");
    std::fs::write(gecici.yol().join("bir.txt"), "eski\n").expect("ilk durum");
    std::fs::write(
        &kaynak,
        "eylem bozuk kaydet\n    değer döndürmez\n    \"bir.txt\" dosyasına \"yeni\" yaz\n    \"iki.txt\" dosyasına \"yarım\" yaz\n    sonuç 1 in 0 a bölümü olsun\n\nbozuk kaydet\n",
    )
    .expect("kaynak");

    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", kaynak.to_str().expect("utf8")])
        .output()
        .expect("CLI çalışmalı");
    assert!(!cikti.status.success(), "çalışma hatası süreçte görünmeli");
    assert!(String::from_utf8_lossy(&cikti.stderr).contains("C003"));
    assert_eq!(
        std::fs::read_to_string(gecici.yol().join("bir.txt")).unwrap(),
        "eski\n"
    );
    assert!(
        !gecici.yol().join("iki.txt").exists(),
        "eylemin yarım oluşturduğu dosya kalmamalı"
    );
}

#[test]
fn gercek_io_iki_surecte_ekleme_kaybetmez() {
    let gecici = GeciciKlasor::yeni();
    let ikili = env!("CARGO_BIN_EXE_dil");
    let mut kaynaklar = Vec::new();
    for onek in ["a", "b"] {
        let yol = gecici.yol().join(format!("yazar-{}.dil", onek));
        std::fs::write(
            &yol,
            format!(
                "1 den 40 a kadar her sayı için\n    satır \"{}-\" ile sayının metni olsun\n    \"olaylar.txt\" dosyasına satır ekle\n",
                onek
            ),
        )
        .expect("yazar kaynağı");
        kaynaklar.push(yol);
    }

    let mut yazarlar = kaynaklar
        .iter()
        .map(|kaynak| {
            Command::new(ikili)
                .args(["çalıştır", kaynak.to_str().expect("utf8")])
                .spawn()
                .expect("yazar süreç")
        })
        .collect::<Vec<_>>();
    for yazar in &mut yazarlar {
        assert!(yazar.wait().expect("yazar sonucu").success());
    }

    let satirlar = std::fs::read_to_string(gecici.yol().join("olaylar.txt"))
        .expect("olaylar")
        .lines()
        .map(str::to_string)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(satirlar.len(), 80);
    for onek in ["a", "b"] {
        for sayi in 1..=40 {
            assert!(satirlar.contains(&format!("{}-{}", onek, sayi)));
        }
    }
}

#[test]
fn gercek_web_sunucusu_acik_opt_in_ister() {
    let gecici = GeciciKlasor::yeni();
    let kaynak = gecici.yol().join("sunucu.dil");
    std::fs::write(
        &kaynak,
        "0 kapısında sunucu başlat\nargümanlar komut satırından gelenler olsun\nher argüman için\n    argümanı yaz\nprogramı bitir\n",
    )
    .expect("web kaynağı");
    let ikili = env!("CARGO_BIN_EXE_dil");

    let korumali = Command::new(ikili)
        .args(["çalıştır", kaynak.to_str().expect("utf8")])
        .output()
        .expect("korumalı çalıştırma");
    assert!(!korumali.status.success());
    let korumali_hata = String::from_utf8_lossy(&korumali.stderr);
    assert!(korumali_hata.contains("C017"), "{}", korumali_hata);
    assert!(
        korumali_hata.contains("--deneysel-web"),
        "{}",
        korumali_hata
    );

    let acik = Command::new(ikili)
        .args([
            "çalıştır",
            "--deneysel-web",
            kaynak.to_str().expect("utf8"),
            "yalnız-programa",
        ])
        .output()
        .expect("opt-in çalıştırma");
    assert!(
        acik.status.success(),
        "{}",
        String::from_utf8_lossy(&acik.stderr)
    );
    assert!(
        String::from_utf8_lossy(&acik.stderr).contains("yalnız localhost"),
        "uyarı görünür olmalı"
    );
    assert_eq!(
        String::from_utf8_lossy(&acik.stdout),
        "Sunucu dinliyor: http://127.0.0.1:0\nyalnız-programa\n",
        "opt-in bayrağı program argümanlarına sızmamalı"
    );

    let guvenli_web = Command::new(ikili)
        .args([
            "çalıştır",
            "--web-proxy",
            "https://panel.example",
            kaynak.to_str().expect("utf8"),
            "yalnız-programa",
        ])
        .output()
        .expect("güvenli web profili");
    assert!(guvenli_web.status.success());
    assert!(String::from_utf8_lossy(&guvenli_web.stderr)
        .contains("yalnız 127.0.0.1 üzerindeki HTTPS reverse proxy"));
    assert_eq!(
        String::from_utf8_lossy(&guvenli_web.stdout),
        "Sunucu dinliyor: https://panel.example (yerel proxy hedefi http://127.0.0.1:0)\nyalnız-programa\n"
    );
}

#[test]
fn parola_ozeti_komutu_argon2id_phc_uretir() {
    use std::io::Write;
    use std::process::Stdio;

    let mut cocuk = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["parola-özeti", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("parola özeti süreci");
    cocuk
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"uzun deneme parolasi\n")
        .expect("parola yazılmalı");
    let cikti = cocuk.wait_with_output().expect("süreç sonucu");
    assert!(cikti.status.success());
    let ozet = String::from_utf8(cikti.stdout).expect("utf8");
    let ozet = ozet.trim();
    assert!(ozet.starts_with("$argon2id$v=19$"));
    assert!(dil::guvenlik::parola_dogrula("uzun deneme parolasi", ozet));
    assert!(!dil::guvenlik::parola_dogrula("yanlis", ozet));
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
            morfoloji: "zee-tr-1".into(),
            giris: "kaynak/ana.dil".into(),
            yetkinlikler: std::collections::BTreeSet::new(),
            ag_hedefleri: std::collections::BTreeSet::new(),
            yerel_bagimliliklar: Vec::new(),
            registry: None,
            uzak_bagimliliklar: Vec::new(),
            veritabani: None,
            eposta_baglanti_degiskeni: None,
            gizli_anahtar_degiskeni: None,
            http_gizli_degiskenleri: Vec::new(),
            web_oturumu: None,
        }
    );
}

#[test]
fn postgresql_bildirimi_sirri_kaynaga_almadan_kesin_hedefi_kilitler() {
    let kaynak = "proje \"uygulama\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"veritabanı\" listesi olsun\nağ_hedefleri boş liste olsun\nveritabanı_hedefi \"postgresql://127.0.0.1:5432/uygulama\" olsun\nveritabanı_bağlantı_değişkeni \"UYGULAMA_DATABASE_URL\" olsun\nveritabanı_göçleri \"göçler\" olsun\nveritabanı_istek_bağlamı \"platform.session_context\" olsun\n";
    let bildirim = bildirimi_oku(kaynak).expect("PostgreSQL bildirimi geçmeli");
    let veritabani = bildirim.veritabani.expect("veritabanı ayarı");
    assert_eq!(veritabani.hedef.konak, "127.0.0.1");
    assert_eq!(veritabani.hedef.kapi, 5432);
    assert_eq!(veritabani.hedef.veritabani, "uygulama");
    assert_eq!(veritabani.baglanti_degiskeni, "UYGULAMA_DATABASE_URL");
    assert_eq!(veritabani.gocler, "göçler");
    assert_eq!(
        veritabani.istek_baglami_cozucusu.as_deref(),
        Some("platform.session_context")
    );

    for (eski, yeni) in [
        (
            "postgresql://127.0.0.1:5432/uygulama",
            "postgresql://kullanici:parola@127.0.0.1:5432/uygulama",
        ),
        ("UYGULAMA_DATABASE_URL", "postgresql://sır"),
        ("\"göçler\"", "\"../göçler\""),
    ] {
        let bozuk = kaynak.replacen(eski, yeni, 1);
        let hata = bildirimi_oku(&bozuk).expect_err("güvensiz DB bildirimi");
        assert_eq!(hata.kod, "P015", "{eski} → {yeni}: {}", hata.mesaj);
    }

    let production = kaynak.replacen("127.0.0.1", "db.example", 1);
    let production = bildirimi_oku(&production).expect("kanonik DNS hedefi bildirilebilmeli");
    assert_eq!(production.veritabani.unwrap().hedef.konak, "db.example");

    for gecersiz in ["DB.example", "999.999.999.999", "-db.example"] {
        let bozuk = kaynak.replacen("127.0.0.1", gecersiz, 1);
        let hata = bildirimi_oku(&bozuk).expect_err("kanonik olmayan konak reddedilmeli");
        assert_eq!(hata.kod, "P015", "{gecersiz}: {}", hata.mesaj);
    }

    for gecersiz in [
        "session_context",
        "Platform.session_context",
        "platform.session-context",
        "platform.session_context.extra",
    ] {
        let bozuk = kaynak.replacen("platform.session_context", gecersiz, 1);
        let hata = bildirimi_oku(&bozuk).expect_err("SQL işlev adı kapalı profilde olmalı");
        assert_eq!(hata.kod, "P015", "{gecersiz}: {}", hata.mesaj);
    }
}

#[test]
fn bildirim_morfoloji_profilini_sabitler_ve_bilinmeyeni_reddeder() {
    let acik = "proje \"uygulama\" olsun\nsürüm \"1.0.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"ana.dil\" olsun\n";
    assert_eq!(
        bildirimi_oku(acik).expect("profil desteklenmeli").morfoloji,
        "zee-tr-1"
    );

    let gelecek = acik.replace("zee-tr-1", "zee-tr-2");
    let hata = bildirimi_oku(&gelecek).expect_err("bilinmeyen profil fail-closed olmalı");
    assert_eq!(hata.kod, "P011");
    assert!(hata.mesaj.contains("zee-tr-2"), "{}", hata.mesaj);
}

#[test]
fn bildirim_yetkinlikleri_ve_tam_ag_originlerini_dogrular() {
    let kaynak = "proje \"uygulama\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ\", \"yerel-ağ\" listesi olsun\nağ_hedefleri \"https://api.example\", \"http://127.0.0.1:8080\" listesi olsun\n";
    let bildirim = bildirimi_oku(kaynak).expect("açık yetkinlikler geçmeli");
    assert!(bildirim
        .yetkinlikler
        .contains(&dil::yetkinlik::Yetkinlik::Ag));
    assert!(bildirim
        .yetkinlikler
        .contains(&dil::yetkinlik::Yetkinlik::YerelAg));
    assert_eq!(bildirim.ag_hedefleri.len(), 2);

    for kotu in [
        kaynak.replace("\"ağ\", \"yerel-ağ\"", "\"yerel-ağ\""),
        kaynak.replace(
            "ağ_hedefleri \"https://api.example\", \"http://127.0.0.1:8080\" listesi",
            "ağ_hedefleri boş liste",
        ),
        kaynak.replace("\"ağ\", \"yerel-ağ\"", "\"ağ\""),
        kaynak.replace("https://api.example", "https://api.example/yol"),
    ] {
        assert_eq!(
            bildirimi_oku(&kotu)
                .expect_err("bozuk yetkinlik bildirimi")
                .kod,
            "P015"
        );
    }
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
fn uzak_bagimlilik_exact_registry_root_kimligine_baglanir() {
    let ozet = format!("sha256:{}", "a".repeat(64));
    let kaynak = format!(
        concat!(
            "proje \"uygulama\" olsun\n",
            "sürüm \"1.0.0\" olsun\n",
            "giriş \"ana.dil\" olsun\n",
            "registry \"https://registry.example\" olsun\n",
            "registry_kök_sürümü \"1\" olsun\n",
            "registry_kök_özeti \"{}\" olsun\n",
            "uzak_bağımlılıklar \"miras@1.2.3\" listesi olsun\n"
        ),
        ozet
    );
    let bildirim = bildirimi_oku(&kaynak).expect("exact registry bildirimi");
    assert_eq!(
        bildirim.registry,
        Some(RegistryBildirimi {
            origin: "https://registry.example".into(),
            kok_surumu: 1,
            kok_sha256: ozet,
        })
    );
    assert_eq!(
        bildirim.uzak_bagimliliklar,
        vec![UzakBagimlilik {
            ad: "miras".into(),
            surum: "1.2.3".into(),
        }]
    );

    for bozuk in [
        kaynak.replace("https://", "http://"),
        kaynak.replace("miras@1.2.3", "miras@^1"),
        kaynak.replace("registry_kök_özeti", "# registry_kök_özeti"),
    ] {
        assert_eq!(
            bildirimi_oku(&bozuk).expect_err("P017 beklenir").kod,
            "P017"
        );
    }
}

#[test]
fn uzak_bagimlilik_guncellemesi_yorumu_korur_siralar_ve_surumu_degistirir() {
    let kaynak =
        "# miras\nproje \"uygulama\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\n";
    let registry = RegistryBildirimi::yeni(
        "https://registry.example",
        7,
        &format!("sha256:{}", "b".repeat(64)),
    )
    .expect("registry");
    let guncel = uzak_bagimliliklari_guncelle(
        kaynak,
        Some(&registry),
        &[
            UzakBagimlilik::ayristir("zaman@2.0.0").expect("zaman"),
            UzakBagimlilik::ayristir("miras@1.4.0").expect("miras"),
        ],
    )
    .expect("güncellenmeli");
    assert!(guncel.starts_with("# miras\n"));
    assert!(guncel.contains("registry_kök_sürümü \"7\" olsun"));
    assert!(guncel.contains("uzak_bağımlılıklar \"miras@1.4.0\", \"zaman@2.0.0\" listesi olsun"));
    let bildirim = bildirimi_oku(&guncel).expect("yeniden okunmalı");
    assert_eq!(bildirim.uzak_bagimliliklar.len(), 2);
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
fn paket_ana_projenin_vermedigi_yetkinligi_genisletemez() {
    let gecici = GeciciKlasor::yeni();
    let paket = gecici.yol().join("okuyucu");
    let uygulama = gecici.yol().join("uygulama");
    std::fs::create_dir(&paket).unwrap();
    std::fs::create_dir(&uygulama).unwrap();
    std::fs::write(
        paket.join("proje.dil"),
        "proje \"okuyucu\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"paket.dil\" olsun\nyetkinlikler \"dosya-okuma\" listesi olsun\n",
    )
    .unwrap();
    std::fs::write(paket.join("paket.dil"), "\"paket\" yaz\n").unwrap();
    let uygulama_bildirimi = "proje \"uygulama\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"../okuyucu\" listesi olsun\n";
    std::fs::write(uygulama.join("proje.dil"), uygulama_bildirimi).unwrap();
    std::fs::write(uygulama.join("ana.dil"), "okuyucu paketini kullan\n").unwrap();

    let hata = dil::paket::ProjeGrafigi::cozumle(&uygulama)
        .err()
        .expect("paket yetkinlik yükseltememeli");
    assert_eq!(hata.tani.kod, "P015");
    assert!(hata.tani.mesaj.contains("dosya-okuma"));

    std::fs::write(
        uygulama.join("proje.dil"),
        uygulama_bildirimi.replace(
            "giriş \"ana.dil\" olsun\n",
            "giriş \"ana.dil\" olsun\nyetkinlikler \"dosya-okuma\" listesi olsun\n",
        ),
    )
    .unwrap();
    dil::paket::ProjeGrafigi::cozumle(&uygulama).expect("üst proje açıkça onayladı");
}

#[cfg(unix)]
#[test]
fn proje_dosya_siniri_sembolik_bag_kacisini_reddeder() {
    let gecici = GeciciKlasor::yeni();
    let uygulama = gecici.yol().join("uygulama");
    std::fs::create_dir(&uygulama).unwrap();
    std::fs::write(gecici.yol().join("gizli.txt"), "proje dışı\n").unwrap();
    std::os::unix::fs::symlink("../gizli.txt", uygulama.join("kacis.txt")).unwrap();
    std::fs::write(
        uygulama.join("proje.dil"),
        "proje \"uygulama\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"dosya-okuma\" listesi olsun\n",
    )
    .unwrap();
    std::fs::write(
        uygulama.join("ana.dil"),
        "satırlar \"kacis.txt\" dosyasının satırları olsun\nsatırların ilki yaz\n",
    )
    .unwrap();

    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", uygulama.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!cikti.status.success());
    let hata = String::from_utf8_lossy(&cikti.stderr);
    assert!(hata.contains("sembolik bağ"), "{hata}");
}

#[test]
fn cli_proje_klasorunu_calistirir_denetler_ve_dener() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.yol().join("proje.dil"),
        "proje \"hesap\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"dosya-yazma\" listesi olsun\n",
    )
    .expect("bildirim");
    std::fs::write(
        gecici.yol().join("yardimci.dil"),
        "işlem iki katını bul\n    sayıyı TamSayı olarak al\n    TamSayı döndürür\n    sonucu sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n",
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
    assert!(bildirim.contains("morfoloji \"zee-tr-1\" olsun"));
    assert!(bildirim.contains("yetkinlikler boş liste olsun"));
    assert!(bildirim.contains("ağ_hedefleri boş liste olsun"));
    assert!(bildirim.contains("yerel_bağımlılıklar boş liste olsun"));
    assert!(
        proje.join("proje.kilit").is_file(),
        "iskelet kilitli başlamalı"
    );
    let git_yoksay = std::fs::read_to_string(proje.join(".gitignore")).expect("gitignore");
    assert!(git_yoksay.contains(".zee/"));
    assert!(git_yoksay.contains(".zee-yazma-kilidi"));
    assert!(git_yoksay.contains("*.zee-gecici-*"));
    assert!(git_yoksay.contains("*.zee-anahtar"));
    assert!(git_yoksay.contains("*.zee-io-izi"));

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
fn surum_komutu_etkin_morfoloji_profilini_gosterir() {
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("sürüm")
        .output()
        .expect("sürüm");
    assert!(cikti.status.success());
    let stdout = String::from_utf8(cikti.stdout).expect("utf8");
    assert!(stdout.contains("morfoloji zee-tr-1"), "{}", stdout);
    assert!(stdout.contains("IO zee-io-1"), "{}", stdout);
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
    std::fs::write(
        temel.join("temel.dil"),
        "işlem üç ver\n    TamSayı döndürür\n    3 döndür\n",
    )
    .expect("temel kaynak");

    std::fs::write(
        hesap.join("proje.dil"),
        "proje \"hesap\" olsun\nsürüm \"2.1.0\" olsun\ngiriş \"paket.dil\" olsun\nyerel_bağımlılıklar \"../temel\" listesi olsun\n",
    )
    .expect("hesap bildirim");
    std::fs::write(
        hesap.join("yardimci.dil"),
        "işlem iki ver\n    TamSayı döndürür\n    2 döndür\n",
    )
    .expect("paket içi birim");
    std::fs::write(
        hesap.join("paket.dil"),
        "yardimci birimini kullan\ntemel paketini kullan\n\n\"paketin üst düzeyi çalışmamalı\" yaz\n\nişlem toplam ver\n    TamSayı döndürür\n    a iki ver olsun\n    b üç ver olsun\n    toplam a ile b nin toplamı olsun\n    toplamı döndür\n",
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
    assert!(ilk_kilit.contains("kilit_sürümü 3"));
    assert!(ilk_kilit.contains("ana \"uygulama\" \"0.1.0\" \"zee-tr-1\""));
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
    assert!(paket_ciktisi.matches("morfoloji zee-tr-1").count() >= 2);

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
    std::fs::write(
        hesap.join("yardimci.dil"),
        "işlem iki ver\n    TamSayı döndürür\n    4 döndür\n",
    )
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
fn paket_islemi_eksik_public_sozlesmeyle_alinamaz() {
    let gecici = GeciciKlasor::yeni();
    let paket = gecici.yol().join("hesap");
    let uygulama = gecici.yol().join("uygulama");
    std::fs::create_dir(&paket).expect("paket");
    std::fs::create_dir(&uygulama).expect("uygulama");
    std::fs::write(
        paket.join("proje.dil"),
        "proje \"hesap\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"paket.dil\" olsun\n",
    )
    .expect("paket bildirimi");
    std::fs::write(
        paket.join("paket.dil"),
        "işlem iki katını bul\n    sayıyı al\n    sonuç sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n",
    )
    .expect("eksik public imza");
    std::fs::write(
        uygulama.join("proje.dil"),
        "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"../hesap\" listesi olsun\n",
    )
    .expect("uygulama bildirimi");
    std::fs::write(
        uygulama.join("ana.dil"),
        "hesap paketini kullan\n\nx 5 için iki katını bul olsun\nx yaz\n",
    )
    .expect("uygulama kaynağı");

    let ikili = env!("CARGO_BIN_EXE_dil");
    let kilitle = Command::new(ikili)
        .args(["kilitle", uygulama.to_str().expect("utf8")])
        .output()
        .expect("kilitle");
    assert!(kilitle.status.success());
    let denetle = Command::new(ikili)
        .args(["denetle", uygulama.to_str().expect("utf8")])
        .output()
        .expect("denetle");
    assert!(!denetle.status.success());
    let hata = String::from_utf8_lossy(&denetle.stderr);
    assert!(hata.contains("T039"), "{}", hata);
    assert!(hata.contains("iki katını bul"), "{}", hata);
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
    std::fs::write(
        paket.join("paket.dil"),
        "işlem yedi ver\n    TamSayı döndürür\n    7 döndür\n",
    )
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

const WEB_OTURUMU_BASI: &str = "proje \"erp\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ-sunucusu\", \"web-oturumu\" listesi olsun\n";

#[test]
fn oturum_omru_ve_kayar_bildirimi_okunur_sinirlar_runtime_ile_ayni() {
    let bildirim = bildirimi_oku(&format!(
        "{WEB_OTURUMU_BASI}oturum_ömrü_saniye 2592000 olsun\noturum_kayar doğru olsun\n"
    ))
    .expect("oturum ayarı geçmeli");
    assert_eq!(
        bildirim.web_oturumu,
        Some(dil::proje::WebOturumuBildirimi {
            omur_saniye: 2_592_000,
            kayar: true,
        })
    );
    let yalniz_kayar = bildirimi_oku(&format!("{WEB_OTURUMU_BASI}oturum_kayar doğru olsun\n"))
        .expect("yalnız kayar geçmeli")
        .web_oturumu
        .unwrap();
    assert_eq!(yalniz_kayar.omur_saniye, 30 * 60);
    let yalniz_omur = bildirimi_oku(&format!("{WEB_OTURUMU_BASI}oturum_ömrü_saniye 300 olsun\n"))
        .expect("alt sınır geçmeli")
        .web_oturumu
        .unwrap();
    assert!(!yalniz_omur.kayar);
    assert_eq!(bildirimi_oku(WEB_OTURUMU_BASI).unwrap().web_oturumu, None);

    assert_eq!(
        dil::proje::OTURUM_OMRU_ALT_SANIYE,
        dil::web_guvenligi::OTURUM_OMRU_ALT_SANIYE
    );
    assert_eq!(
        dil::proje::OTURUM_OMRU_UST_SANIYE,
        dil::web_guvenligi::OTURUM_MUTLAK_TAVAN_SANIYE
    );
    assert_eq!(
        dil::proje::OTURUM_OMRU_VARSAYILAN_SANIYE,
        dil::web_guvenligi::OTURUM_OMRU_SANIYE
    );
}

#[test]
fn oturum_omru_sinir_disi_tur_ve_yetkinlik_hatalari_tani_verir() {
    for (ek, kod, parca) in [
        ("oturum_ömrü_saniye 299 olsun\n", "P015", "300 ile 2592000"),
        (
            "oturum_ömrü_saniye 2592001 olsun\n",
            "P015",
            "300 ile 2592000",
        ),
        ("oturum_ömrü_saniye 0 olsun\n", "P015", "300 ile 2592000"),
        ("oturum_ömrü_saniye \"1800\" olsun\n", "P001", "sabit Sayı"),
        (
            "oturum_kayar \"evet\" olsun\n",
            "P001",
            "doğru ya da yanlış",
        ),
    ] {
        let tani = bildirimi_oku(&format!("{WEB_OTURUMU_BASI}{ek}")).unwrap_err();
        assert_eq!(tani.kod, kod, "{ek}");
        assert!(tani.mesaj.contains(parca), "{ek}: {}", tani.mesaj);
        assert_eq!(tani.satir, 5, "{ek}");
    }
    let tani = bildirimi_oku(
        "proje \"erp\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\noturum_kayar doğru olsun\n",
    )
    .unwrap_err();
    assert_eq!(tani.kod, "P015");
    assert!(tani.mesaj.contains("web-oturumu"));
}

/// K-330/ADR-158 eki: hermetik `dil dene` adaptörü proje bildirimindeki
/// kimlikli oturum ömrünü gerçek runtime ile aynı uygular; ayarsız proje
/// varsayılan 30 dakikada kalır.
#[test]
fn dene_hermetik_adaptoru_proje_oturum_omrunu_uygular() {
    fn dene(bildirim_eki: &str, bekleme_sn: i64, beklenen_adet: i64) -> std::process::Output {
        let gecici = GeciciKlasor::yeni();
        std::fs::write(
            gecici.yol().join("proje.dil"),
            format!("{WEB_OTURUMU_BASI}{bildirim_eki}"),
        )
        .unwrap();
        std::fs::write(
            gecici.yol().join("ana.dil"),
            r#"8093 kapısında sunucu başlat

GET "/" adresine istek geldiğinde
    "a" yanıtını gönder

test "oturum ömrü"
    "m" kullanıcısını "k" rolüyle oturuma al
    BEKLEME saniye bekle
    oturumlar "m" ile web oturumlarını listelemeyi dene olsun
    adet -1 olsun
    oturumlar başarılıysa
        liste oturumların değeri olsun
        adet listenin adedi olsun
    adet BEKLENEN eşit olmalı
"#
            .replace("BEKLEME", &bekleme_sn.to_string())
            .replace("BEKLENEN", if beklenen_adet == 0 { "0 a" } else { "1 e" }),
        )
        .unwrap();
        Command::new(env!("CARGO_BIN_EXE_dil"))
            .arg("dene")
            .arg(gecici.yol())
            .output()
            .expect("dil dene")
    }
    let gecmeli = |ek: &str, bekleme: i64, adet: i64| {
        let cikti = dene(ek, bekleme, adet);
        assert!(
            cikti.status.success(),
            "{ek} / {bekleme} sn / {adet}: {}{}",
            String::from_utf8_lossy(&cikti.stdout),
            String::from_utf8_lossy(&cikti.stderr)
        );
    };
    let omur = "oturum_ömrü_saniye 300 olsun\n";
    let kayar = "oturum_ömrü_saniye 300 olsun\noturum_kayar doğru olsun\n";
    // Olumlu: bildirilen 300 sn ömür hermetik adaptörde de geçerlidir.
    gecmeli(omur, 299, 1);
    gecmeli(omur, 301, 0);
    gecmeli(kayar, 301, 0);
    // Ayarsız proje: varsayılan 30 dakika mutlak.
    gecmeli("", 301, 1);
    gecmeli("", 1801, 0);
    // Olumsuz: düzeltme öncesi davranış (ayar yok sayılıp oturum yaşar) D001 ile kalır.
    let eski = dene(omur, 301, 1);
    assert!(!eski.status.success());
    let stderr = String::from_utf8_lossy(&eski.stderr);
    assert!(stderr.contains("D001"), "{stderr}");
    assert!(stderr.contains("Beklenen: 1"), "{stderr}");
}

/// K-331: gerçek IO adaptörü ilk baytları ikili dosyadan okur; proje dosya
/// sınırı, üst sınır ve yetkinlik `sha256 özetini almayı dene` ile aynıdır.
#[cfg(unix)]
#[test]
fn dosya_imzasi_gercek_ikili_dosyada_sinirli_ve_proje_kokunde_okunur() {
    let gecici = GeciciKlasor::yeni();
    let uygulama = gecici.yol().join("uygulama");
    std::fs::create_dir_all(uygulama.join(".zee/yuklemeler")).unwrap();
    std::fs::write(gecici.yol().join("gizli.bin"), b"%PDF-gizli").unwrap();
    std::os::unix::fs::symlink("../gizli.bin", uygulama.join("kacis.pdf")).unwrap();
    std::fs::write(
        uygulama.join(".zee/yuklemeler/cv.parca"),
        b"%PDF-1.7\n\xff\xfe\x00binary",
    )
    .unwrap();
    std::fs::write(uygulama.join("kisa.bin"), [0xd0u8, 0xcf]).unwrap();
    std::fs::write(uygulama.join("buyuk.bin"), vec![0xabu8; 200]).unwrap();
    let bildirim = |yetkinlik: &str| {
        format!("proje \"uygulama\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler {yetkinlik} listesi olsun\n")
    };
    std::fs::write(uygulama.join("proje.dil"), bildirim("\"dosya-okuma\"")).unwrap();
    std::fs::write(
        uygulama.join("ana.dil"),
        r#"işlem imzaYaz
    yolu Metin olarak al
    adedi TamSayı olarak al
    değer döndürmez
    sonuç yolu dosyasının ilk adedi baytını onaltılık okumayı dene olsun
    sonuç başarılıysa
        sonucun değeri yaz
    değilse
        hata sonucun hatası olsun
        hatanın kodu yaz
        hatanın mesajı yaz

".zee/yuklemeler/cv.parca" ve 8 ile imzaYaz
".zee/yuklemeler/cv.parca" ve 12 ile imzaYaz
"kisa.bin" ve 8 ile imzaYaz
"buyuk.bin" ve 64 ile imzaYaz
"buyuk.bin" ve 65 ile imzaYaz
"buyuk.bin" ve 0 ile imzaYaz
"../gizli.bin" ve 4 ile imzaYaz
"kacis.pdf" ve 4 ile imzaYaz
"yok.pdf" ve 4 ile imzaYaz
".zee" ve 4 ile imzaYaz
"#,
    )
    .unwrap();

    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", uygulama.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&cikti.stdout);
    assert!(
        cikti.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert_eq!(satirlar[0], "255044462d312e37");
    assert_eq!(satirlar[1], "255044462d312e370afffe00");
    assert_eq!(satirlar[2], "d0cf", "kısa dosyada olan kadar");
    assert_eq!(satirlar[3], "ab".repeat(64));
    assert_eq!(satirlar[4], "C013");
    assert!(satirlar[5].contains("1 ile 64"), "{stdout}");
    assert_eq!(satirlar[6], "C013");
    assert!(satirlar[7].contains("1 ile 64"), "{stdout}");
    assert_eq!(satirlar[8], "C013");
    assert!(satirlar[9].contains("kök içinde kalan"), "{stdout}");
    assert_eq!(satirlar[10], "C013");
    assert!(satirlar[11].contains("sembolik bağ"), "{stdout}");
    assert_eq!(satirlar[12], "C013");
    assert!(satirlar[13].contains("çözülemedi"), "{stdout}");
    assert_eq!(satirlar[14], "C013");
    assert!(satirlar[15].contains("normal dosya değil"), "{stdout}");
    assert_eq!(satirlar.len(), 16, "{stdout}");

    std::fs::write(uygulama.join("proje.dil"), bildirim("\"dosya-yazma\"")).unwrap();
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", uygulama.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!cikti.status.success());
    assert!(
        String::from_utf8_lossy(&cikti.stderr).contains("T054"),
        "dosya-okuma olmadan derleme reddi"
    );
}

/// K-347: `dil ekle/çıkar` proje.dil'i okuduktan sonra başka bir komut
/// bildirimi değiştirirse yazım karşılaştırmayla reddedilir; diğer komutun
/// eklediği bağımlılık ezilmez, kilit de dokunulmadan kalır.
#[test]
fn bildirim_yazimi_araya_giren_komutun_bagimliligini_ezmez() {
    let gecici = GeciciKlasor::yeni();
    let uygulama = gecici.yol().join("uygulama");
    std::fs::create_dir(&uygulama).expect("uygulama");
    for ad in ["hesap", "takvim"] {
        let paket = gecici.yol().join(ad);
        std::fs::create_dir(&paket).expect("paket");
        std::fs::write(
            paket.join("proje.dil"),
            format!("proje \"{ad}\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"paket.dil\" olsun\n"),
        )
        .expect("paket bildirim");
        std::fs::write(
            paket.join("paket.dil"),
            "işlem bir ver\n    TamSayı döndürür\n    1 döndür\n",
        )
        .expect("paket kaynak");
    }
    let bildirim = uygulama.join("proje.dil");
    let ilk = "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n";
    std::fs::write(&bildirim, ilk).expect("uygulama bildirim");
    std::fs::write(uygulama.join("ana.dil"), "\"tamam\" yaz\n").expect("ana");

    let aday = |yol: &str| {
        dil::proje::yerel_bagimliliklari_guncelle(ilk, &[yol.to_string()]).expect("aday bildirim")
    };
    let (a_kaynak, b_kaynak) = (aday("../hesap"), aday("../takvim"));
    let a = dil::paket::ProjeGrafigi::cozumle_bildirimle(&uygulama, &a_kaynak).expect("a grafiği");
    let b = dil::paket::ProjeGrafigi::cozumle_bildirimle(&uygulama, &b_kaynak).expect("b grafiği");

    // İki komut da `ilk`i okudu; b önce yazar, a sonra.
    b.bildirim_ve_kilidi_yaz(ilk, &b_kaynak).expect("b yazımı");
    let b_kilidi = std::fs::read(uygulama.join("proje.kilit")).expect("b kilidi");
    let hata = a.bildirim_ve_kilidi_yaz(ilk, &a_kaynak).unwrap_err();
    assert!(hata.contains("başka bir süreç"), "{hata}");
    assert_eq!(std::fs::read_to_string(&bildirim).unwrap(), b_kaynak);
    assert_eq!(
        std::fs::read(uygulama.join("proje.kilit")).unwrap(),
        b_kilidi
    );
}

/// K-347: proje kaynakları toplanırken `proje.dil` başka bir sürecin atomik
/// değiştirmesinin ortasında olabilir (eşzamanlı `dil ekle`). Kaynak toplayıcı
/// dosyayı değiştirme penceresini bekleyen okuyucudan başka bir yolla
/// açmamalıdır; Windows'ta `canonicalize` açılışı ReplaceFileW arasında
/// düşüyordu.
#[test]
fn kaynak_toplama_proje_dil_degistirilirken_dusmez() {
    let gecici = GeciciKlasor::yeni();
    let uygulama = gecici.yol().join("uygulama");
    std::fs::create_dir(&uygulama).expect("uygulama");
    let bildirim = uygulama.join("proje.dil");
    let ilk = "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n";
    std::fs::write(&bildirim, ilk).expect("bildirim");
    std::fs::write(uygulama.join("ana.dil"), "\"tamam\" yaz\n").expect("ana");
    let bitti = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let yazar = {
        let (bildirim, bitti) = (bildirim.clone(), std::sync::Arc::clone(&bitti));
        std::thread::spawn(move || {
            let mut sira = 0u32;
            while !bitti.load(std::sync::atomic::Ordering::Acquire) {
                sira += 1;
                let icerik = format!("{ilk}# yazım {sira}\n");
                dil::kalici_dosya::atomik_yaz(&bildirim, icerik.as_bytes()).expect("yazım");
            }
            sira
        })
    };
    let mut hatalar = Vec::new();
    for _ in 0..300 {
        if let Err(hata) = dil::paket::ProjeGrafigi::cozumle_bildirimle(&uygulama, ilk) {
            hatalar.push(format!("{hata:?}"));
        }
    }
    bitti.store(true, std::sync::atomic::Ordering::Release);
    let yazim = yazar.join().expect("yazar");
    assert!(yazim > 0);
    assert!(
        hatalar.is_empty(),
        "{} çözüm düştü: {:?}",
        hatalar.len(),
        hatalar.first()
    );
}

/// K-363 (GB-109 Unicode gösterim/registry): paket adı spec/07 gereği küçük
/// harfli tek Türkçe/Latin tanımlayıcıdır. Önceki `is_alphabetic` denetimi
/// Kiril/Yunan homoglifini (`раket` görünüşte `paket`), CJK'yı, bağ harfini,
/// unvan harfini ve birleştirici imi (U+0345) kabul ediyordu.
#[test]
fn paket_adi_yalniz_kucuk_turkce_latin_tanimlayicidir() {
    for ad in ["paket", "çay", "kâr", "_iç", "ağ2", "a_b_1"] {
        assert!(
            UzakBagimlilik::ayristir(&format!("{ad}@1.0.0")).is_ok(),
            "{ad} kabul edilmeli"
        );
    }
    for ad in [
        "\u{0440}\u{0430}ket", // Kiril р, а
        "\u{03c1}aket",        // Yunan ρ
        "c\u{0327}ay",         // NFD ç
        "a\u{0345}",           // birleştirici Yunan ypogegrammeni
        "\u{fb01}le",          // ﬁ bağ harfi
        "\u{01c6}",            // ǆ
        "日本",
        "\u{2170}", // ⅰ
        "\u{00aa}", // ª
        "a\u{202e}b",
        "a\u{200d}b",
        "Paket",
        "\u{0130}", // İ
        "1paket",
        "",
    ] {
        let hata = UzakBagimlilik::ayristir(&format!("{ad}@1.0.0"))
            .expect_err(&format!("{ad:?} reddedilmeli"));
        assert!(
            hata.contains("küçük harfli tek tanımlayıcı"),
            "{ad:?}: {hata}"
        );
    }
}

/// K-363: homoglif adlı yerel bağımlılık grafiğe girmez (P007).
#[test]
fn homoglif_adli_yerel_bagimlilik_p007_ile_reddedilir() {
    let gecici = GeciciKlasor::yeni();
    let uygulama = gecici.yol().join("uygulama");
    let paket = gecici.yol().join("paket");
    std::fs::create_dir(&uygulama).unwrap();
    std::fs::create_dir(&paket).unwrap();
    std::fs::write(
        uygulama.join("proje.dil"),
        "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"../paket\" listesi olsun\n",
    )
    .unwrap();
    std::fs::write(uygulama.join("ana.dil"), "1 yaz\n").unwrap();
    std::fs::write(
        paket.join("proje.dil"),
        "proje \"\u{0440}\u{0430}ket\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"paket.dil\" olsun\n",
    )
    .unwrap();
    std::fs::write(
        paket.join("paket.dil"),
        "işlem yedi ver\n    TamSayı döndürür\n    7 döndür\n",
    )
    .unwrap();
    let hata = match dil::paket::ProjeGrafigi::cozumle(&uygulama) {
        Ok(_) => panic!("homoglif paket adı grafiğe girmemeli"),
        Err(hata) => hata,
    };
    assert_eq!(hata.tani.kod, "P007");
    assert!(hata.tani.mesaj.contains("tek kelime olarak kullanılamaz"));
}

fn birim_projesi(gecici: &GeciciKlasor, birim_dosyasi: &str) -> PathBuf {
    let kok = gecici.yol().join("uygulama");
    std::fs::create_dir(&kok).unwrap();
    std::fs::write(
        kok.join("proje.dil"),
        "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .unwrap();
    std::fs::write(kok.join("ana.dil"), "çay birimini kullan\nselam ver\n").unwrap();
    std::fs::write(
        kok.join(birim_dosyasi),
        "işlem selam ver\n    değer döndürmez\n    \"merhaba\" yaz\n",
    )
    .unwrap();
    kok
}

/// K-363: `çay birimini kullan` yalnız NFC adı `çay.dil` olan dosyayı
/// bağlar. Önceden büyük/küçük harf duyarsız dosya sisteminde (macOS APFS,
/// Windows NTFS) `Çay.dil` yükleniyor, Linux'ta birim bulunamıyordu; aynı
/// proje platforma göre farklı anlam taşıyordu. Artık her platformda aynı
/// A010 tanısıdır. NFD saklanan ad (`c` + U+0327) aynı kimliktir ve her
/// platformda bağlanır.
#[test]
fn birim_adi_dosya_adiyla_nfc_birebir_eslesir() {
    let gecici = GeciciKlasor::yeni();
    let kok = birim_projesi(&gecici, "Çay.dil");
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", kok.to_str().expect("utf8")])
        .output()
        .expect("dil");
    assert_eq!(cikti.status.code(), Some(1));
    assert!(cikti.stdout.is_empty());
    let hata = String::from_utf8_lossy(&cikti.stderr);
    assert!(hata.contains("HATA A010"), "{hata}");
    assert!(
        hata.contains(
            "\"çay\" birimi \"çay.dil\" dosyasıyla birebir eşleşmeli; diskteki \"Çay.dil\" yalnız büyük/küçük harf ya da Unicode biçimiyle farklı"
        ),
        "{hata}"
    );

    let gecici = GeciciKlasor::yeni();
    let kok = birim_projesi(&gecici, "c\u{0327}ay.dil");
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", kok.to_str().expect("utf8")])
        .output()
        .expect("dil");
    assert_eq!(
        String::from_utf8_lossy(&cikti.stdout),
        "merhaba\n",
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    assert!(cikti.status.success());
}

/// K-363: `--yanked-kabul`/`--kritik-kabul` gerekçesi `proje.kilit`e yazılır
/// ve incelemede okunur; bidi/sıfır genişlikli biçim karakteri (Trojan
/// Source) Cc denetim karakteri gibi reddedilir.
#[test]
fn kabul_gerekcesi_gorunmez_bicim_karakteri_tasiyamaz() {
    use dil::paket::RegistryCozumPolitikasi;
    assert!(RegistryCozumPolitikasi::yeni(false, Some("bilinçli kabul".into()), None).is_ok());
    for gerekce in [
        "onay\u{202e}lanmadı",
        "a\u{200b}b",
        "a\u{061c}b",
        "a\u{2066}b",
        "a\nb",
    ] {
        let hata =
            RegistryCozumPolitikasi::yeni(false, Some(gerekce.into()), None).expect_err(gerekce);
        assert!(hata.contains("denetim karakterli"), "{gerekce:?}: {hata}");
        let hata =
            RegistryCozumPolitikasi::yeni(false, None, Some(gerekce.into())).expect_err(gerekce);
        assert!(hata.contains("kritik duyuru kabul"), "{gerekce:?}: {hata}");
    }
}
