//! K-394 (RFC-0045, ADR-170): bildirilen paylaşılan veri kökü (`veri_kökü`,
//! `veri:` öneki, `veri-okuma`/`veri-yazma`). Gerçek `dil` sürecinde: aynı
//! uygulamanın iki projesi aynı kökü paylaşır (yazan hizmet ek dosyasını
//! çözerek yazar, ayrı web projesi yalnız okuyup ek olarak sunar); yazma
//! yetkisiz okuyucu, eksik ortam değişkeni, kaçış denemeleri, sembolik bağ,
//! tek dosya kipi ve proje bildirimi tanıları.
#![cfg(unix)]

use dil::proje::bildirimi_oku;
use dil::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};
use dil::yorumlayici::{self, PolitikaliIo, ToplayanIo};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[path = "semantic_regresyon/native_web_kopma.rs"]
mod kopma;

static SIRA: AtomicU64 = AtomicU64::new(0);
const DEGISKEN: &str = "ZEE_K394_DEPO";
const SIR: &str = "K394_KOK_DISI_SIR";

/// Bir uygulamanın paylaşılan veri kökü ve kök dışında bir "sır".
struct Alan(PathBuf);

impl Alan {
    fn yeni() -> Self {
        let taban = std::env::temp_dir().canonicalize().unwrap();
        let yol = taban.join(format!(
            "zee-k394-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(yol.join("depo/ekler")).unwrap();
        std::fs::create_dir_all(yol.join("dis")).unwrap();
        std::fs::write(yol.join("dis/sir.txt"), SIR).unwrap();
        for (klasor, kip) in [("", 0o700), ("depo", 0o755), ("depo/ekler", 0o770)] {
            std::fs::set_permissions(yol.join(klasor), std::fs::Permissions::from_mode(kip))
                .unwrap();
        }
        Self(yol)
    }

    fn depo(&self) -> PathBuf {
        self.0.join("depo")
    }

    fn proje(&self, ad: &str, yetkinlikler: &str, ana: &str) -> PathBuf {
        let yol = self.0.join(ad);
        std::fs::create_dir_all(&yol).unwrap();
        std::fs::write(
            yol.join("proje.dil"),
            format!(
                "proje \"{ad}\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler {yetkinlikler} listesi olsun\nağ_hedefleri boş liste olsun\nveri_kökü \"{DEGISKEN}\" olsun\n"
            ),
        )
        .unwrap();
        std::fs::write(yol.join("ana.dil"), ana).unwrap();
        yol
    }
}

impl Drop for Alan {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn calistir(hedef: &Path, kok: Option<&Path>) -> (bool, String, String) {
    let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
    komut.args(["çalıştır", hedef.to_str().unwrap()]);
    komut
        .env_remove(DEGISKEN)
        .env_remove("CREDENTIALS_DIRECTORY");
    if let Some(kok) = kok {
        komut.env(DEGISKEN, kok);
    }
    let cikti = komut.output().unwrap();
    (
        cikti.status.success(),
        String::from_utf8_lossy(&cikti.stdout).into_owned(),
        String::from_utf8_lossy(&cikti.stderr).into_owned(),
    )
}

fn web_baslat(proje: &Path, kapi: u16, kok: &Path) -> Child {
    let cocuk = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args([
            "çalıştır",
            "--web-proxy",
            kopma::PROXY,
            "--web-worker-port",
            &kapi.to_string(),
            proje.to_str().unwrap(),
        ])
        .env_remove("CREDENTIALS_DIRECTORY")
        .env(DEGISKEN, kok)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    kopma::dinlemeyi_bekle(kapi);
    cocuk
}

fn istek(kapi: u16, hedef: &str) -> (String, Vec<u8>) {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(5)).unwrap();
    akis.set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    write!(
        akis,
        "GET {hedef} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.9;proto=https;host=panel.example\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut ham = Vec::new();
    akis.read_to_end(&mut ham).unwrap();
    let son = ham.windows(4).position(|p| p == b"\r\n\r\n").unwrap() + 4;
    (
        String::from_utf8_lossy(&ham[..son]).into_owned(),
        ham[son..].to_vec(),
    )
}

fn sha256(veri: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(veri)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

const YAZICI: &str = "\"dosya-okuma\", \"veri-okuma\", \"veri-yazma\"";
const WEB: &str = "\"ağ-sunucusu\", \"veri-okuma\"";

/// "Merhaba Eliz\0\u{ff}" ikili içeriği; base64 `TWVyaGFiYSBFbGl6AP8=`.
const EK: &[u8] = b"Merhaba Eliz\0\xff";
const EK_B64: &str = "TWVyaGFiYSBFbGl6AP8=";

#[test]
fn iki_proje_ayni_veri_kokunu_paylasir_web_yalniz_okuyup_ek_sunar() {
    let alan = Alan::yeni();
    let ozet = sha256(EK);
    let yazici = alan.proje(
        "alici",
        YAZICI,
        &format!(
            r#"sonuc "{EK_B64}" base64 çözerek "veri:ekler/k1-{ozet}" dosyasına yazmayı dene olsun
sonuc başarılıysa
    tutamak sonucun değeri olsun
    tutamağın "yol" değeri yaz
    tutamağın "sha256" değeri yaz
sonuc başarısızsa
    sonucun hatası yaz
ikinci "{EK_B64}" base64 çözerek "veri:ekler/k1-{ozet}" dosyasına yazmayı dene olsun
ikinci başarısızsa
    ikincinin hatası yaz
"#
        ),
    );
    let (tamam, stdout, stderr) = calistir(&yazici, Some(&alan.depo()));
    assert!(tamam, "{stdout}{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert_eq!(satirlar[0], format!("veri:ekler/k1-{ozet}"), "{stdout}");
    assert_eq!(satirlar[1], ozet);
    assert!(satirlar[2].contains("zaten var"), "no-clobber: {stdout}");
    assert_eq!(
        std::fs::read(alan.depo().join(format!("ekler/k1-{ozet}"))).unwrap(),
        EK
    );
    assert!(
        !yazici.join("veri:ekler").exists() && !yazici.join("ekler").exists(),
        "veri: yolu proje köküne yazılmamalı"
    );

    let web = alan.proje(
        "web",
        WEB,
        r#"8080 kapısında sunucu başlat

GET "/ek" adresine istek geldiğinde
    herkese açık
    ad isteğin "ad" değeri olsun
    yol "veri:ekler/" ile ad olsun
    yanit yol dosyasını "application/pdf" türünde "Ek.pdf" adıyla ek olarak göndermeyi dene olsun
    yanit başarısızsa
        hata yanitin hatası olsun
        mesaj hatanın mesajı olsun
        mesaj yanıtını 404 durumuyla gönder
"#,
    );
    std::fs::write(alan.depo().join("ekler/bag"), "yer").unwrap();
    std::fs::remove_file(alan.depo().join("ekler/bag")).unwrap();
    symlink(alan.0.join("dis/sir.txt"), alan.depo().join("ekler/bag")).unwrap();
    std::fs::hard_link(alan.0.join("dis/sir.txt"), alan.depo().join("ekler/kopya")).unwrap();
    let kapi = kopma::bos_kapi();
    let mut sunucu = web_baslat(&web, kapi, &alan.depo());
    let (basliklar, govde) = istek(kapi, &format!("/ek?ad=k1-{ozet}"));
    let (bag_basligi, bag_govdesi) = istek(kapi, "/ek?ad=bag");
    let (kopya_basligi, kopya_govdesi) = istek(kapi, "/ek?ad=kopya");
    let _ = sunucu.kill();
    let _ = sunucu.wait();
    assert!(basliklar.starts_with("HTTP/1.1 200 OK\r\n"), "{basliklar}");
    assert_eq!(govde, EK, "indirilen ek bayt bayt aynı");
    for (baslik, govde) in [(bag_basligi, bag_govdesi), (kopya_basligi, kopya_govdesi)] {
        assert!(baslik.starts_with("HTTP/1.1 404"), "{baslik}");
        let govde = String::from_utf8_lossy(&govde).into_owned();
        assert!(!govde.contains(SIR), "kök dışı içerik sızdı: {govde}");
        assert!(
            govde.contains("sembolik bağ izlenmez") || govde.contains("hard-link"),
            "{govde}"
        );
        assert!(
            !govde.contains(&*alan.0.to_string_lossy()),
            "mutlak yol sızdı: {govde}"
        );
    }
}

#[test]
fn yazma_yetkisi_olmayan_proje_veri_kokune_yazamaz() {
    let alan = Alan::yeni();
    // Statik: dosya yazımı var, ne `dosya-yazma` ne `veri-yazma` açık → T054.
    let salt = alan.proje(
        "salt",
        "\"veri-okuma\"",
        "s \"QUJD\" base64 çözerek \"veri:ekler/x\" dosyasına yazmayı dene olsun\n",
    );
    let (tamam, stdout, stderr) = calistir(&salt, Some(&alan.depo()));
    assert!(!tamam, "{stdout}");
    assert!(
        stderr.contains("T054") || stdout.contains("T054"),
        "{stderr}"
    );
    // Çalışma anı: `dosya-yazma` projeye yazmayı açar, veri köküne değil.
    let karisik = alan.proje(
        "karisik",
        "\"dosya-yazma\", \"veri-okuma\"",
        r#"yazma "QUJD" base64 çözerek "veri:ekler/x" dosyasına yazmayı dene olsun
yazma başarısızsa
    yazmanın hatası yaz
silme "veri:ekler/x" dosyasını silmeyi dene olsun
silme başarısızsa
    silmenin hatası yaz
"#,
    );
    let (tamam, stdout, stderr) = calistir(&karisik, Some(&alan.depo()));
    assert!(tamam, "{stdout}{stderr}");
    assert_eq!(
        stdout
            .matches("`veri-yazma` yetkinliği çalışma politikasında açık değil")
            .count(),
        2,
        "{stdout}"
    );
    assert!(!alan.depo().join("ekler/x").exists());
}

#[test]
fn ortam_degiskeni_yoksa_ya_da_kok_guvensizse_veri_yolu_reddedilir() {
    let alan = Alan::yeni();
    std::fs::write(alan.depo().join("ekler/a"), "ek").unwrap();
    let proje = alan.proje(
        "okur",
        "\"veri-okuma\"",
        r#"okuma "veri:ekler/a" dosyasını okumayı dene olsun
okuma başarılıysa
    okumanın değeri yaz
okuma başarısızsa
    okumanın hatası yaz
"#,
    );
    let (tamam, stdout, _) = calistir(&proje, None);
    assert!(tamam);
    assert!(
        stdout.contains(&format!("veri kökü ayarlanmamış: `{DEGISKEN}`")),
        "{stdout}"
    );
    let (_, stdout, _) = calistir(&proje, Some(Path::new("goreli/depo")));
    assert!(stdout.contains("mutlak yol olmalı"), "{stdout}");
    let bag = alan.0.join("depo-bagi");
    symlink(alan.depo(), &bag).unwrap();
    let (_, stdout, _) = calistir(&proje, Some(&bag));
    assert!(stdout.contains("sembolik bağ içeremez"), "{stdout}");
    let (_, stdout, _) = calistir(&proje, Some(&alan.depo()));
    assert_eq!(stdout.trim(), "ek");
}

#[test]
fn kacis_denemeleri_ve_sembolik_baglar_kok_disini_okutamaz() {
    let alan = Alan::yeni();
    symlink(alan.0.join("dis"), alan.depo().join("disari")).unwrap();
    symlink(alan.0.join("dis/sir.txt"), alan.depo().join("ekler/sir")).unwrap();
    let mut ana = String::new();
    for yol in [
        "veri:../dis/sir.txt",
        "veri:ekler/../../dis/sir.txt",
        "veri:/etc/passwd",
        "veri:disari/sir.txt",
        "veri:ekler/sir",
        "veri:ekler//sir",
        "veri:ekler\\\\sir",
    ] {
        ana.push_str(&format!(
            "okuma \"{yol}\" dosyasını okumayı dene olsun\nokuma başarılıysa\n    okumanın değeri yaz\nokuma başarısızsa\n    okumanın hatası yaz\n"
        ));
    }
    let proje = alan.proje("kacis", "\"veri-okuma\"", &ana);
    let (tamam, stdout, stderr) = calistir(&proje, Some(&alan.depo()));
    assert!(tamam, "{stdout}{stderr}");
    assert!(!stdout.contains(SIR), "kök dışı içerik okundu: {stdout}");
    assert!(
        !stdout.contains(&*alan.0.to_string_lossy()),
        "mutlak yol sızdı: {stdout}"
    );
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert_eq!(satirlar.len(), 7, "{stdout}");
    for satir in &satirlar[..3] {
        assert!(satir.contains("veri kökü yolu reddedildi"), "{satir}");
    }
    for satir in &satirlar[3..5] {
        assert!(satir.contains("sembolik bağ izlenmez"), "{satir}");
    }
    for satir in &satirlar[5..] {
        assert!(satir.contains("veri kökü yolu reddedildi"), "{satir}");
    }
}

#[test]
fn ata_klasoru_paylasilan_yazilabilirse_yol_reddedilir() {
    let alan = Alan::yeni();
    std::fs::write(alan.depo().join("ekler/a"), "ek").unwrap();
    std::fs::set_permissions(alan.depo(), std::fs::Permissions::from_mode(0o775)).unwrap();
    let proje = alan.proje(
        "ata",
        "\"veri-okuma\"",
        "okuma \"veri:ekler/a\" dosyasını okumayı dene olsun\nokuma başarısızsa\n    okumanın hatası yaz\n",
    );
    let (tamam, stdout, _) = calistir(&proje, Some(&alan.depo()));
    assert!(tamam);
    assert!(
        stdout.contains("paylaşılan yazma yalnız son klasörde"),
        "{stdout}"
    );
}

#[test]
fn tek_dosya_kipinde_veri_yolu_yerel_dosya_sayilmaz() {
    let alan = Alan::yeni();
    let kaynak = alan.0.join("tek.dil");
    std::fs::write(
        &kaynak,
        r#""veri:x" dosyasına "a" yaz
"#,
    )
    .unwrap();
    let (tamam, stdout, stderr) = calistir(&kaynak, Some(&alan.depo()));
    assert!(!tamam, "{stdout}");
    assert!(stderr.contains("yalnız proje kipinde"), "{stderr}");
    assert!(!alan.0.join("veri:x").exists());
    assert!(!alan.depo().join("x").exists());
}

#[test]
fn bildirimsiz_projede_veri_yolu_reddedilir() {
    let alan = Alan::yeni();
    let yol = alan.0.join("bildirimsiz");
    std::fs::create_dir_all(&yol).unwrap();
    std::fs::write(
        yol.join("proje.dil"),
        "proje \"b\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"dosya-okuma\" listesi olsun\nağ_hedefleri boş liste olsun\n",
    )
    .unwrap();
    std::fs::write(
        yol.join("ana.dil"),
        "okuma \"veri:ekler/a\" dosyasını okumayı dene olsun\nokuma başarısızsa\n    okumanın hatası yaz\n",
    )
    .unwrap();
    let (tamam, stdout, _) = calistir(&yol, Some(&alan.depo()));
    assert!(tamam);
    assert!(
        stdout.contains("`veri-okuma` yetkinliği çalışma politikasında açık değil"),
        "{stdout}"
    );
}

#[test]
fn proje_bildirimi_veri_koku_ve_yetkinlikleri_birlikte_ister() {
    let taban = "proje \"p\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nağ_hedefleri boş liste olsun\n";
    let tamam = bildirimi_oku(&format!(
        "{taban}yetkinlikler \"veri-okuma\" listesi olsun\nveri_kökü \"UYGULAMA_DEPO\" olsun\n"
    ))
    .unwrap();
    assert_eq!(tamam.veri_koku_degiskeni.as_deref(), Some("UYGULAMA_DEPO"));
    assert!(tamam.yetkinlikler.contains(&Yetkinlik::VeriOkuma));
    // Kök olmadan `veri-*` paket bildiriminde geçerlidir; ana projede grafik
    // denetimi P023 verir (yerel_paketin_veri_yazimi_… testi).
    assert!(bildirimi_oku(&format!(
        "{taban}yetkinlikler \"veri-yazma\" listesi olsun\n"
    ))
    .is_ok());
    for (kotu, beklenen) in [
        (
            "yetkinlikler boş liste olsun\nveri_kökü \"UYGULAMA_DEPO\" olsun\n",
            "yetkinliği açık değil",
        ),
        (
            "yetkinlikler \"veri-okuma\" listesi olsun\nveri_kökü \"/srv/depo\" olsun\n",
            "A-Z, 0-9 ve _",
        ),
        (
            "yetkinlikler \"veri-okuma\" listesi olsun\nveri_kökü \"depo\" olsun\n",
            "A-Z, 0-9 ve _",
        ),
    ] {
        let hata = bildirimi_oku(&format!("{taban}{kotu}")).unwrap_err();
        assert_eq!(hata.kod, "P023", "{kotu}: {hata:?}");
        assert!(hata.mesaj.contains(beklenen), "{kotu}: {}", hata.mesaj);
    }
}

#[test]
fn dene_hermetik_tablosunda_veri_yolu_metin_anahtaridir() {
    let program = dil::kaynagi_derle(
        r#""veri:ekler/t" dosyasına "bir" yaz
okuma "veri:ekler/t" dosyasını okumayı dene olsun
okuma başarılıysa
    okumanın değeri yaz
"#,
    )
    .unwrap();
    let politika = YetkinlikPolitikasi::proje(
        [Yetkinlik::VeriOkuma, Yetkinlik::VeriYazma]
            .into_iter()
            .collect(),
        Default::default(),
    );
    let mut io = PolitikaliIo::politikali(ToplayanIo::yeni(Vec::new()), politika);
    yorumlayici::calistir_io(&program, &mut io).unwrap();
    assert_eq!(io.ic.cikti, ["bir\n"]);
    assert!(io.ic.dosyalar.contains_key("veri:ekler/t"));
    // Veri yetkinliği proje dosyası yazımını açmaz.
    let proje_yazimi = dil::kaynagi_derle("\"yerel.txt\" dosyasına \"a\" yaz\n").unwrap();
    let politika = YetkinlikPolitikasi::proje(
        [Yetkinlik::VeriYazma].into_iter().collect(),
        Default::default(),
    );
    let mut io = PolitikaliIo::politikali(ToplayanIo::yeni(Vec::new()), politika);
    let hata = yorumlayici::calistir_io(&proje_yazimi, &mut io).unwrap_err();
    assert!(hata.mesaj.contains("dosya-yazma"), "{hata:?}");
}

#[test]
fn veri_kokunde_yazma_tasima_silme_ve_okuma_ailesi_calisir() {
    let alan = Alan::yeni();
    std::fs::create_dir_all(alan.depo().join("arsiv")).unwrap();
    let proje = alan.proje(
        "isci",
        "\"dosya-okuma\", \"dosya-yazma\", \"veri-okuma\", \"veri-yazma\"",
        r#""yerel.txt" dosyasına "proje" yaz
"veri:ekler/gunluk" dosyasına "bir" yaz
"veri:ekler/gunluk" dosyasına "iki" ekle
okuma "veri:ekler/gunluk" dosyasını okumayı dene olsun
okuma başarılıysa
    okumanın değeri yaz
okuma başarısızsa
    okumanın hatası yaz
ozet "veri:ekler/gunluk" dosyasının sha256 özetini almayı dene olsun
ozet başarılıysa
    ozetin değeri yaz
ozet başarısızsa
    ozetin hatası yaz
parca "veri:ekler/gunluk" dosyasının 4 baytından 3 baytını metin olarak okumayı dene olsun
parca başarılıysa
    parcanın değeri yaz
parca başarısızsa
    parcanın hatası yaz
imza "veri:ekler/gunluk" dosyasının ilk 3 baytını onaltılık okumayı dene olsun
imza başarılıysa
    imzanın değeri yaz
imza başarısızsa
    imzanın hatası yaz
tasima "yerel.txt" dosyasını "veri:ekler/yerelden" yoluna atomik taşımayı dene olsun
tasima başarılıysa
    tasimanın değeri yaz
tasima başarısızsa
    tasimanın hatası yaz
ikinci "veri:ekler/yerelden" dosyasını "veri:arsiv/kalici" yoluna atomik taşımayı dene olsun
ikinci başarılıysa
    ikincinin değeri yaz
ikinci başarısızsa
    ikincinin hatası yaz
silme "veri:ekler/gunluk" dosyasını silmeyi dene olsun
silme başarılıysa
    silmenin değeri yaz
silme başarısızsa
    silmenin hatası yaz
liste "veri:ekler" yolundaki dosyaları listelemeyi dene olsun
liste başarısızsa
    listenin hatası yaz
"#,
    );
    let (tamam, stdout, stderr) = calistir(&proje, Some(&alan.depo()));
    assert!(tamam, "{stdout}{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        satirlar,
        [
            "bir",
            "iki",
            "",
            sha256(b"bir\niki\n").as_str(),
            "iki",
            "626972",
            "1",
            "1",
            "1",
            "\"veri:ekler\" veri kökünde dizin listeleme bu sürümde desteklenmez",
        ],
        "{stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(alan.depo().join("arsiv/kalici")).unwrap(),
        "proje\n"
    );
    assert!(!proje.join("yerel.txt").exists());
    assert!(!alan.depo().join("ekler/gunluk").exists());
    assert!(!alan.depo().join("ekler/yerelden").exists());
}

/// Yerel bağımlılık paketi (`motor`) ve onu kullanan ana proje.
fn paketli(alan: &Alan, ana_ek: &str, paket_yetkinlik: &str, paket_ek: &str) -> PathBuf {
    let paket = alan.0.join("motor");
    std::fs::create_dir_all(&paket).unwrap();
    std::fs::write(
        paket.join("proje.dil"),
        format!(
            "proje \"motor\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"motor.dil\" olsun\nyetkinlikler {paket_yetkinlik} olsun\nağ_hedefleri boş liste olsun\n{paket_ek}"
        ),
    )
    .unwrap();
    std::fs::write(
        paket.join("motor.dil"),
        r#"işlem depola
    adi Metin olarak al
    Metin döndürür
    hedef "veri:ekler/" ile adi olsun
    yazim "QUJD" base64 çözerek hedef dosyasına yazmayı dene olsun
    yazim başarılıysa
        tutamak yazimin değeri olsun
        tutamağın "yol" değeri döndür
    "ret" döndür
"#,
    )
    .unwrap();
    let ana = alan.0.join("ana");
    std::fs::create_dir_all(&ana).unwrap();
    std::fs::write(
        ana.join("proje.dil"),
        format!(
            "proje \"ana\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nağ_hedefleri boş liste olsun\nyerel_bağımlılıklar \"../motor\" listesi olsun\n{ana_ek}"
        ),
    )
    .unwrap();
    std::fs::write(
        ana.join("ana.dil"),
        "motor paketini kullan\nsonuc \"ek\" ile depola olsun\nsonuc yaz\n",
    )
    .unwrap();
    let kilit = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["kilitle", ana.to_str().unwrap()])
        .output()
        .unwrap();
    // Bildirim/grafik reddi kilitlemede de görünür; sonuç `çalıştır` ile sınanır.
    let _ = kilit;
    ana
}

#[test]
fn yerel_paketin_veri_yazimi_ana_projenin_veri_kokune_cozulur() {
    let alan = Alan::yeni();
    let kok = format!("yetkinlikler \"veri-okuma\", \"veri-yazma\" listesi olsun\nveri_kökü \"{DEGISKEN}\" olsun\n");
    // Paket ihtiyacını kökü olmadan bildirir; kök ana projeden gelir.
    let ana = paketli(&alan, &kok, "\"veri-yazma\" listesi", "");
    let (tamam, stdout, stderr) = calistir(&ana, Some(&alan.depo()));
    assert!(tamam, "{stdout}{stderr}");
    assert_eq!(stdout.trim(), "veri:ekler/ek");
    assert_eq!(std::fs::read(alan.depo().join("ekler/ek")).unwrap(), b"ABC");
    assert!(!alan.0.join("motor/veri:ekler").exists() && !ana.join("veri:ekler").exists());

    // Paket kendi kökünü bildiremez.
    let ana = paketli(
        &alan,
        &kok,
        "\"veri-yazma\" listesi",
        "veri_kökü \"BASKA_DEPO\" olsun\n",
    );
    let (tamam, _, stderr) = calistir(&ana, Some(&alan.depo()));
    assert!(!tamam);
    assert!(
        stderr.contains("P023") && stderr.contains("paketi `veri_kökü` bildiremez"),
        "{stderr}"
    );

    // Ana proje veri-* açıp kök bildirmezse grafik P023 verir.
    let ana = paketli(
        &alan,
        "yetkinlikler \"veri-yazma\" listesi olsun\n",
        "\"veri-yazma\" listesi",
        "",
    );
    let (tamam, _, stderr) = calistir(&ana, Some(&alan.depo()));
    assert!(!tamam);
    assert!(
        stderr.contains("P023") && stderr.contains("bildirilmemiş"),
        "{stderr}"
    );

    // Ana proje yalnız okuma veriyorsa paketin yazma ihtiyacı P015'tir.
    let ana = paketli(
        &alan,
        &format!("yetkinlikler \"veri-okuma\" listesi olsun\nveri_kökü \"{DEGISKEN}\" olsun\n"),
        "\"veri-yazma\" listesi",
        "",
    );
    let (tamam, _, stderr) = calistir(&ana, Some(&alan.depo()));
    assert!(!tamam);
    assert!(
        stderr.contains("P015") && stderr.contains("veri-yazma"),
        "{stderr}"
    );
}
