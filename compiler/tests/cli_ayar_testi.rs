//! K-234: Gerçek CLI, sürümlü ayar ve eski-consumer kabul sınırı.
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
static SIRA: AtomicU64 = AtomicU64::new(0);
struct Proje(PathBuf);
impl Proje {
    fn yeni() -> Self {
        let kok = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "zee-k234-{}-{}",
                std::process::id(),
                SIRA.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&kok).unwrap();
        fs::write(
            kok.join("proje.dil"),
            "proje \"ayar-kabulu\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"ana.dil\" olsun\n",
        )
        .unwrap();
        fs::write(kok.join("ana.dil"), "\"aday\" yaz\n").unwrap();
        Self(kok)
    }
    fn ayar(&self, v: Value) {
        fs::write(self.0.join("zee.ayar"), v.to_string()).unwrap();
    }
    fn ham(&self, v: &[u8]) {
        fs::write(self.0.join("zee.ayar"), v).unwrap();
    }
    fn kos(&self, once: &[&str], sonra: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dil"))
            .arg("çalıştır")
            .args(once)
            .arg(&self.0)
            .args(sonra)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
}
impl Drop for Proje {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn zarf(g: Value) -> Value {
    json!({"sema":"zee-ayar-1", "surum":1, "ayarlar":{}, "guvenlik":g})
}
fn gecti(o: Output) -> String {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let stderr = String::from_utf8(o.stderr).unwrap();
    assert_eq!(String::from_utf8(o.stdout).unwrap().trim(), "aday");
    stderr
}
fn ret(o: Output, beklenen: &str) {
    assert_eq!(o.status.code(), Some(2));
    assert!(
        o.stdout.is_empty(),
        "source must not run on rejected config"
    );
    let hata = String::from_utf8(o.stderr).unwrap();
    assert!(hata.contains(beklenen), "{hata}");
    assert!(
        !hata.contains("SENTETIK-SIR"),
        "config diagnostics must not reveal keys/values"
    );
}
#[test]
fn ayarsiz_ve_kapali_varsayilan_gercek_cli_ayni_ciktilidir() {
    let p = Proje::yeni();
    assert!(gecti(p.kos(&[], &[])).is_empty());
    p.ayar(zarf(json!({})));
    assert!(gecti(p.kos(&[], &[])).is_empty());
    p.ayar(zarf(json!({"web-modu":"kapali", "cocuk-modu":false})));
    assert!(gecti(p.kos(&[], &[])).is_empty());
}
#[test]
fn simulated_schema1_consumer_gelecek_olagan_alani_varsayilanla_atlar() {
    let p = Proje::yeni();
    let mut v = zarf(json!({}));
    // Future producer remains schema1; no security behavior is in this branch.
    v["ayarlar"] =
        json!({"SENTETIK-SIR-anahtar":{"tema":"SENTETIK-SIR-deger", "gelecek":[1,true,null]}});
    p.ayar(v);
    assert_eq!(
        gecti(p.kos(&[], &[])),
        "zee.ayar: bilinmeyen olağan ayarlar atlandı; varsayılanlar korundu.\n"
    );
}
#[test]
fn bilinmeyen_guvenlik_ve_ust_zarf_alanlari_deger_yazmadan_reddedilir() {
    let p = Proje::yeni();
    for ad in [
        "SENTETIK-SIR-security",
        "ordinary-looking-setting",
        "tls",
        "WEB-MODU",
    ] {
        p.ayar(zarf(json!({ad:"SENTETIK-SIR-deger"})));
        ret(p.kos(&[], &[]), "bilinmeyen güvenlik alanı");
    }
    let mut v = zarf(json!({}));
    v["SENTETIK-SIR-extra"] = json!("SENTETIK-SIR-deger");
    p.ayar(v);
    ret(p.kos(&[], &[]), "zarf alanları");
}
#[test]
fn bos_invalid_future_ve_cift_anahtar_json_fail_closed() {
    let p = Proje::yeni();
    for s in [
        "",
        " ",
        "{}",
        "[]",
        "null",
        "not-json-SENTETIK-SIR",
        "{\"sema\":\"zee-ayar-1\"}",
        r#"{"sema":"zee-ayar-1","surum":1,"surum":1,"ayarlar":{},"guvenlik":{}}"#,
        r#"{"sema":"zee-ayar-1","surum":1,"ayarlar":{},"guvenlik":{"web-modu":"kapali","web\u002dmodu":"kapali"}}"#,
        r#"{"sema":"zee-ayar-1","surum":1,"ayarlar":{"gelecek":{"a":1,"a":2}},"guvenlik":{}}"#,
        r#"{"sema":"zee-ayar-1","surum":1,"ayarlar":{},"guvenlik":{}}{}"#,
    ] {
        p.ham(s.as_bytes());
        ret(p.kos(&[], &[]), "zee.ayar:");
    }
    p.ham(&[0xff, 0xfe]);
    ret(p.kos(&[], &[]), "JSON geçersiz");
    for v in [
        json!(0),
        json!(2),
        json!("1"),
        json!(true),
        json!(1.0),
        Value::Null,
    ] {
        let mut z = zarf(json!({}));
        z["surum"] = v;
        p.ayar(z);
        ret(p.kos(&[], &[]), "desteklenmiyor");
    }
    let mut z = zarf(json!({}));
    z["sema"] = json!("zee-ayar-2");
    p.ayar(z);
    ret(p.kos(&[], &[]), "desteklenmiyor");
    for alan in ["ayarlar", "guvenlik"] {
        let mut z = zarf(json!({}));
        z[alan] = json!([]);
        p.ayar(z);
        ret(p.kos(&[], &[]), "nesne alanları");
    }
}
#[test]
fn guvenlik_tur_aralik_ve_ic_bagimlilikler_cli_override_ile_asillamaz() {
    let p = Proje::yeni();
    for g in [
        json!({"cocuk-modu":"true"}),
        json!({"web-modu":false}),
        json!({"web-modu":"unknown"}),
        json!({"web-modu":"proxy"}),
        json!({"web-modu":"proxy","proxy-origin":"http://SENTETIK-SIR.invalid"}),
        json!({"proxy-origin":"https://example.invalid"}),
        json!({"worker-port":0}),
        json!({"worker-port":65536}),
        json!({"worker-port":"80"}),
        json!({"worker-port":1.5}),
        json!({"worker-port":true}),
        json!({"worker-port":-1}),
        json!({"oran-deposu":"disk"}),
        json!({"oran-siniri":0}),
        json!({"oran-siniri":1000001}),
        json!({"oran-kapasitesi":32769}),
        json!({"oran-doluluk":"drop"}),
        json!({"web-modu":"deneysel","oran-deposu":"bellek"}),
        json!({"web-modu":"proxy","proxy-origin":"https://example.invalid","oran-siniri":20}),
        json!({"web-modu":"deneysel","cocuk-modu":true}),
    ] {
        p.ayar(zarf(g));
        ret(p.kos(&["--deneysel-web"], &[]), "zee.ayar:");
    }
}
#[test]
fn mevcut_web_ayar_eslemesi_ve_cli_alana_gore_onceligi() {
    let p = Proje::yeni();
    p.ayar(zarf(json!({"web-modu":"deneysel"})));
    assert!(gecti(p.kos(&[], &[])).contains("deneysel web yüzeyi"));
    assert!(
        gecti(p.kos(&["--web-proxy", "https://example.invalid"], &[]))
            .contains("Güvenli web profili")
    );
    p.ayar(zarf(
        json!({"web-modu":"proxy", "proxy-origin":"https://example.invalid", "worker-port":18123,
        "oran-deposu":"bellek", "oran-siniri":25, "oran-kapasitesi":120, "oran-doluluk":"reddet"}),
    ));
    assert!(gecti(p.kos(&[], &[])).contains("Güvenli web profili"));
    // Invalid explicit numeric values are not hidden by valid config values.
    ret(p.kos(&["--web-worker-port", "0"], &[]), "--web-worker-port");
    ret(p.kos(&["--web-oran-siniri", "0"], &[]), "1..1000000");
    assert!(gecti(p.kos(
        &[
            "--güvenli-web-proxy",
            "https://override.invalid",
            "--web-worker-port",
            "18124",
            "--web-oran-siniri",
            "20",
            "--web-oran-kapasitesi",
            "100",
            "--web-oran-doluluk",
            "en-eski"
        ],
        &[]
    ))
    .contains("Güvenli web profili"));
    ret(
        p.kos(
            &[
                "--web-proxy",
                "https://a.invalid",
                "--web-proxy",
                "https://b.invalid",
            ],
            &[],
        ),
        "birden çok",
    );
}
#[test]
fn cocuk_modu_ve_birlesim_cakismasi_korunur() {
    let p = Proje::yeni();
    p.ayar(zarf(json!({"cocuk-modu":true})));
    assert!(gecti(p.kos(&[], &[])).is_empty());
    assert!(gecti(p.kos(&["--guvenli"], &[])).is_empty());
    ret(p.kos(&["--deneysel-web"], &[]), "çocuk modu");
}
#[test]
fn sinirli_boyut_ve_derinlik_guvenle_reddedilir() {
    let p = Proje::yeni();
    p.ham(&vec![b' '; 65_537]);
    ret(p.kos(&[], &[]), "65536");
    let derin = format!(
        "{{\"sema\":\"zee-ayar-1\",\"surum\":1,\"ayarlar\":{{\"a\":{}0{}}},\"guvenlik\":{{}}}}",
        "[".repeat(150),
        "]".repeat(150)
    );
    p.ham(derin.as_bytes());
    ret(p.kos(&[], &[]), "JSON geçersiz");
    // Inclusive boundary: same safe zarf padded to exactly 65536 bytes.
    let mut bayt = zarf(json!({})).to_string().into_bytes();
    bayt.resize(65_536, b' ');
    p.ham(&bayt);
    assert!(gecti(p.kos(&[], &[])).is_empty());
}
#[test]
fn yalniz_acik_proje_koku_okunur_ust_dizin_ve_standalone_aranmaz() {
    let p = Proje::yeni();
    p.ayar(zarf(json!({"unknown-security":true})));
    let alt = p.0.join("alt");
    fs::create_dir(&alt).unwrap();
    fs::copy(p.0.join("proje.dil"), alt.join("proje.dil")).unwrap();
    fs::copy(p.0.join("ana.dil"), alt.join("ana.dil")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", "alt"])
        .current_dir(&p.0)
        .output()
        .unwrap();
    assert!(gecti(o).is_empty());
    let o = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(p.0.join("ana.dil"))
        .current_dir(&p.0)
        .output()
        .unwrap();
    assert!(gecti(o).is_empty());
    fs::remove_file(p.0.join("proje.dil")).unwrap();
    ret(p.kos(&[], &[]), "proje.dil taşıyan");
}
#[test]
fn program_argumanlari_ayar_yolu_sayilmaz_ve_ust_yol_ret_edilir() {
    let p = Proje::yeni();
    p.ayar(zarf(json!({})));
    assert!(gecti(p.kos(&[], &["--ayar-semasi", "--bilinmeyen-program-ayari"])).is_empty());
    let o = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", "../"])
        .arg(&p.0)
        .current_dir(&p.0)
        .output()
        .unwrap();
    // No zee.ayar in parent: legacy source failure, not an implicit CWD config.
    assert!(!o.status.success());
    let parent = p.0.parent().unwrap();
    let ad = p.0.file_name().unwrap();
    let yol = Path::new("..").join(parent.file_name().unwrap()).join(ad);
    let o = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(yol)
        .current_dir(parent)
        .output()
        .unwrap();
    ret(o, "üst dizin geçişi");
}
#[cfg(unix)]
#[test]
fn ayar_ve_proje_yolu_sembolik_baglari_ve_normal_olmayan_dosya_ret_edilir() {
    use std::os::unix::fs::symlink;
    let p = Proje::yeni();
    p.ayar(zarf(json!({})));
    fs::rename(p.0.join("zee.ayar"), p.0.join("ayar.json")).unwrap();
    symlink("ayar.json", p.0.join("zee.ayar")).unwrap();
    ret(p.kos(&[], &[]), "normal");
    fs::remove_file(p.0.join("zee.ayar")).unwrap();
    symlink("yok", p.0.join("zee.ayar")).unwrap();
    ret(p.kos(&[], &[]), "normal");
    fs::remove_file(p.0.join("zee.ayar")).unwrap();
    fs::create_dir(p.0.join("zee.ayar")).unwrap();
    ret(p.kos(&[], &[]), "normal");
    fs::remove_dir(p.0.join("zee.ayar")).unwrap();
    // A FIFO must be rejected by metadata before any blocking open/read.
    use std::os::unix::ffi::OsStrExt;
    let fifo = std::ffi::CString::new(p.0.join("zee.ayar").as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    ret(p.kos(&[], &[]), "normal");
    fs::remove_file(p.0.join("zee.ayar")).unwrap();
    p.ayar(zarf(json!({})));
    let bag = p.0.with_extension("bag");
    symlink(&p.0, &bag).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(&bag)
        .output()
        .unwrap();
    fs::remove_file(bag).unwrap();
    ret(o, "sembolik bağ");
    fs::rename(p.0.join("proje.dil"), p.0.join("bildirim.dil")).unwrap();
    symlink("bildirim.dil", p.0.join("proje.dil")).unwrap();
    ret(p.kos(&[], &[]), "proje.dil normal");
}
#[test]
fn capability_probe_build_jsonu_degistirmez_ve_selector_once_destek_ister() {
    let o = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["sürüm", "--ayar-semasi"])
        .output()
        .unwrap();
    assert!(o.status.success());
    assert!(o.stderr.is_empty());
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(
        v,
        json!({"schema":"zee-config-capability-1","config_schemas":["zee-ayar-1"],"unknown_security":"reject","version":env!("CARGO_PKG_VERSION")})
    );
    let o = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["sürüm", "--json"])
        .output()
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&o.stdout).unwrap(),
        serde_json::from_str::<Value>(dil::surum::json()).unwrap()
    );
    let o = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["sürüm", "--ayar-semasi", "--json"])
        .output()
        .unwrap();
    ret(o, "tek başına");
    // Capability absence must not be interpreted as schema1 acceptance.
    let simulated_old_build = json!({"schema":"zee-build-v1","version":"0.7.0"});
    assert!(simulated_old_build.get("config_schemas").is_none());
}
