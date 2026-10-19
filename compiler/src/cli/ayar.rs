//! K-234: Sınırlı, sürümlü, güvenlikte fail-closed proje runtime ayarı.
//! CLI adaptörüdür; gramer ve proje yetkinlikleri değişmez.
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub(super) const SEMA: &str = "zee-ayar-1";
const AZAMI_BAYT: u64 = 65_536;
const AZAMI_YOL_BAYTI: usize = 4096;
const AZAMI_YOL_PARCASI: usize = 64;
const OLAGAN_UYARI: &str = "zee.ayar: bilinmeyen olağan ayarlar atlandı; varsayılanlar korundu.";
const GUVENLIK_ALANLARI: &[&str] = &[
    "cocuk-modu",
    "web-modu",
    "proxy-origin",
    "worker-port",
    "oran-deposu",
    "oran-siniri",
    "oran-kapasitesi",
    "oran-doluluk",
];

// Serde Value'ın last-key-wins davranışı yerine her derinlikte yinelenen
// çözümlenmiş anahtar (Unicode kaçışı dahil) reddedilir.
struct TekilJson(Value);
impl<'de> Deserialize<'de> for TekilJson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct TekilZiyaretci;
        impl<'de> Visitor<'de> for TekilZiyaretci {
            type Value = TekilJson;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("tekil anahtarlı JSON")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(TekilJson(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(TekilJson(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(TekilJson(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| TekilJson(Value::Number(n)))
                    .ok_or_else(|| E::custom("geçersiz sayı"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(TekilJson(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(TekilJson(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(TekilJson(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut v = Vec::new();
                while let Some(TekilJson(x)) = a.next_element()? {
                    v.push(x);
                }
                Ok(TekilJson(Value::Array(v)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut v = Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if v.contains_key(&k) {
                        return Err(de::Error::custom("yinelenen JSON anahtarı"));
                    }
                    let TekilJson(x) = a.next_value()?;
                    v.insert(k, x);
                }
                Ok(TekilJson(Value::Object(v)))
            }
        }
        d.deserialize_any(TekilZiyaretci)
    }
}
fn ret(mesaj: &str) -> String {
    format!("zee.ayar: {mesaj}")
}
fn nesne<'a>(v: &'a Value, alan: &str) -> Result<&'a Map<String, Value>, String> {
    v.get(alan)
        .and_then(Value::as_object)
        .ok_or_else(|| ret("zarf nesne alanları eksik veya geçersiz"))
}
fn metin<'a>(v: &'a Value, alan: &str) -> Result<Option<&'a str>, String> {
    v.get(alan)
        .map(|x| {
            x.as_str()
                .ok_or_else(|| ret("güvenlik alanının türü geçersiz"))
        })
        .transpose()
}
fn sayi(v: &Value, alan: &str, ust: u64) -> Result<Option<u64>, String> {
    v.get(alan)
        .map(|x| {
            x.as_u64()
                .filter(|n| *n > 0 && *n <= ust)
                .ok_or_else(|| ret("güvenlik sayısı türü veya sınırı geçersiz"))
        })
        .transpose()
}
type Bayraklar = Vec<(String, Option<String>)>;

fn cozumle(baytlar: &[u8]) -> Result<(Bayraklar, bool), String> {
    let TekilJson(v) = serde_json::from_slice(baytlar)
        .map_err(|_| ret("JSON geçersiz veya yinelenen anahtar var"))?;
    let zarf = v
        .as_object()
        .ok_or_else(|| ret("sürümlü zarf nesne olmalı"))?;
    if zarf.len() != 4
        || zarf
            .keys()
            .any(|k| !["sema", "surum", "ayarlar", "guvenlik"].contains(&k.as_str()))
    {
        return Err(ret("zarf alanları eksik veya bilinmiyor"));
    }
    if v.get("sema").and_then(Value::as_str) != Some(SEMA)
        || v.get("surum").and_then(Value::as_u64) != Some(1)
    {
        return Err(ret(
            "şema veya sürüm desteklenmiyor; zee-ayar-1/surum 1 gerekir",
        ));
    }
    let olagan = nesne(&v, "ayarlar")?;
    let guvenlik = nesne(&v, "guvenlik")?;
    if guvenlik
        .keys()
        .any(|k| !GUVENLIK_ALANLARI.contains(&k.as_str()))
    {
        return Err(ret("bilinmeyen güvenlik alanı reddedildi"));
    }
    let g = &v["guvenlik"];
    let mut bayraklar: Vec<(String, Option<String>)> = Vec::new();
    if let Some(cocuk) = g.get("cocuk-modu") {
        match cocuk.as_bool() {
            Some(true) => bayraklar.push(("--güvenli".into(), None)),
            Some(false) => {}
            None => return Err(ret("güvenlik alanının türü geçersiz")),
        }
    }
    let modu = metin(g, "web-modu")?.unwrap_or("kapali");
    let origin = metin(g, "proxy-origin")?;
    match modu {
        "kapali" => {
            if origin.is_some() {
                return Err(ret("proxy-origin yalnız proxy modunda kullanılabilir"));
            }
        }
        "deneysel" => {
            if origin.is_some() {
                return Err(ret("proxy-origin yalnız proxy modunda kullanılabilir"));
            }
            bayraklar.push(("--deneysel-web".into(), None));
        }
        "proxy" => {
            let origin = origin.ok_or_else(|| ret("proxy modu proxy-origin ister"))?;
            dil::yetkinlik::AgHedefi::https_origininden(origin)
                .map_err(|_| ret("proxy-origin geçerli HTTPS origin olmalı"))?;
            bayraklar.push(("--web-proxy".into(), Some(origin.into())));
        }
        _ => return Err(ret("web-modu kapali, deneysel veya proxy olmalı")),
    }
    if let Some(n) = sayi(g, "worker-port", 65535)? {
        bayraklar.push(("--web-worker-port".into(), Some(n.to_string())));
    }
    if let Some(depo) = metin(g, "oran-deposu")? {
        if depo != "bellek" {
            return Err(ret("oran-deposu yalnız bellek olabilir"));
        }
        bayraklar.push(("--web-oran-deposu".into(), Some(depo.into())));
    }
    for (alan, bayrak, ust) in [
        ("oran-siniri", "--web-oran-siniri", 1_000_000),
        ("oran-kapasitesi", "--web-oran-kapasitesi", 32768),
    ] {
        if let Some(n) = sayi(g, alan, ust)? {
            bayraklar.push((bayrak.into(), Some(n.to_string())));
        }
    }
    if let Some(doluluk) = metin(g, "oran-doluluk")? {
        if !["reddet", "en-eski"].contains(&doluluk) {
            return Err(ret("oran-doluluk reddet veya en-eski olmalı"));
        }
        bayraklar.push(("--web-oran-doluluk".into(), Some(doluluk.into())));
    }
    // Açık CLI seçeneği bozuk config'i geçerli hale getiremez.
    let mut deneme = vec!["çalıştır".into()];
    for (bayrak, deger) in &bayraklar {
        deneme.push(bayrak.clone());
        deneme.extend(deger.clone());
    }
    let cocuk = deneme.iter().any(|a| a == "--güvenli");
    deneme.retain(|a| a != "--güvenli");
    let web = super::web_modunu_ayikla(&mut deneme)
        .map_err(|_| ret("güvenlik ayarları birlikte geçersiz"))?;
    if cocuk && web != super::WebModu::Kapali {
        return Err(ret("çocuk modu web modu ile birleştirilemez"));
    }
    Ok((bayraklar, !olagan.is_empty()))
}
fn kaynak_yeri(a: &[String]) -> Option<usize> {
    if !matches!(a.first().map(String::as_str), Some("çalıştır" | "calistir")) {
        return None;
    }
    let mut i = 1;
    while let Some(s) = a.get(i) {
        if !s.starts_with("--") {
            return Some(i);
        }
        i += match s.as_str() {
            "--web-proxy"
            | "--web-metrik-fd"
            | "--web-listener-fd"
            | "--web-ready-fd"
            | "--web-activate-fd"
            | "--web-generation"
            | "--güvenli-web-proxy"
            | "--web-worker-port"
            | "--web-oran-deposu"
            | "--web-oran-siniri"
            | "--web-oran-kapasitesi"
            | "--web-oran-doluluk"
            | "--kapsam"
            | "--web-suite"
            | "--web-suite-sha256"
            | "--web-fixtures"
            | "--test-python"
            | "--test-postgres-bin"
            | "--test-output" => 2,
            "--güvenli" | "--guvenli" | "--deneysel-web" => 1,
            _ => return None,
        };
    }
    None
}
fn kesin_kok(yol: &Path) -> Result<PathBuf, String> {
    if yol.as_os_str().len() > AZAMI_YOL_BAYTI || yol.components().count() > AZAMI_YOL_PARCASI {
        return Err(ret("proje yolu sınırı aşıldı"));
    }
    let mut kesin = if yol.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().map_err(|_| ret("çalışma kökü çözülemedi"))?
    };
    for p in yol.components() {
        match p {
            Component::ParentDir => {
                return Err(ret("ayar taşıyan proje yolunda üst dizin geçişi yasak"))
            }
            Component::CurDir => {}
            _ => kesin.push(p.as_os_str()),
        }
    }
    if kesin.as_os_str().len() > AZAMI_YOL_BAYTI || kesin.components().count() > AZAMI_YOL_PARCASI {
        return Err(ret("proje yolu sınırı aşıldı"));
    }
    let mut biriken = PathBuf::new();
    for p in kesin.components() {
        biriken.push(p.as_os_str());
        // Windows sürücü/UNC öneki (`C:`, `\\?\C:`) ve kök ayırıcı tek başına dosya sistemi
        // girdisi değildir; sembolik bağ denetimi gerçek dizin adlarında yapılır.
        if matches!(p, Component::Prefix(_) | Component::RootDir) {
            continue;
        }
        let m = fs::symlink_metadata(&biriken).map_err(|_| ret("proje yolu çözülemedi"))?;
        if m.file_type().is_symlink() {
            return Err(ret("ayar taşıyan proje yolunda sembolik bağ yasak"));
        }
    }
    let kok = fs::canonicalize(&kesin).map_err(|_| ret("proje kökü çözülemedi"))?;
    let bildirim = fs::symlink_metadata(kok.join("proje.dil"))
        .map_err(|_| ret("ayar yalnız proje.dil taşıyan kökte okunur"))?;
    if !bildirim.is_file() || bildirim.file_type().is_symlink() {
        return Err(ret("proje.dil normal dosya olmalı"));
    }
    Ok(kok)
}
fn dosyayi_oku(yol: &Path) -> Result<Vec<u8>, String> {
    let once = fs::symlink_metadata(yol).map_err(|_| ret("ayar dosyası okunamadı"))?;
    if !once.is_file()
        || once.file_type().is_symlink()
        || once.len() == 0
        || once.len() > AZAMI_BAYT
    {
        return Err(ret(
            "ayar normal, boş olmayan ve en fazla 65536 bayt dosya olmalı",
        ));
    }
    let mut secenek = OpenOptions::new();
    secenek.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        secenek.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        secenek.custom_flags(0x00200000);
    }
    let dosya = secenek
        .open(yol)
        .map_err(|_| ret("ayar dosyası güvenle açılamadı"))?;
    let acilan = dosya
        .metadata()
        .map_err(|_| ret("ayar dosyası okunamadı"))?;
    if !acilan.is_file() || acilan.file_type().is_symlink() || acilan.len() != once.len() {
        return Err(ret("ayar dosyası açılırken değişti"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if acilan.dev() != once.dev() || acilan.ino() != once.ino() {
            return Err(ret("ayar dosyası açılırken değişti"));
        }
    }
    let mut baytlar = Vec::new();
    (&dosya)
        .take(AZAMI_BAYT + 1)
        .read_to_end(&mut baytlar)
        .map_err(|_| ret("ayar dosyası okunamadı"))?;
    if baytlar.len() as u64 != acilan.len() || baytlar.len() as u64 > AZAMI_BAYT {
        return Err(ret("ayar boyutu okuma sırasında değişti veya sınırı aştı"));
    }
    let sonra = dosya
        .metadata()
        .map_err(|_| ret("ayar dosyası okunamadı"))?;
    let yerinde =
        fs::symlink_metadata(yol).map_err(|_| ret("ayar dosyası okuma sırasında değişti"))?;
    if !yerinde.is_file()
        || yerinde.file_type().is_symlink()
        || sonra.len() != acilan.len()
        || sonra.modified().ok() != acilan.modified().ok()
    {
        return Err(ret("ayar dosyası okuma sırasında değişti"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if yerinde.dev() != acilan.dev() || yerinde.ino() != acilan.ino() {
            return Err(ret("ayar dosyası okuma sırasında değişti"));
        }
    }
    Ok(baytlar)
}
/// CWD/ev/üst dizin araması yok; kaynak sonrası uygulama argv'sine bakılmaz.
pub(super) fn uygula(a: &mut Vec<String>) -> Result<(), String> {
    let Some(yer) = kaynak_yeri(a) else {
        return Ok(());
    };
    let yol = Path::new(&a[yer]);
    if !yol.is_dir() {
        return Ok(());
    }
    match fs::symlink_metadata(yol.join("zee.ayar")) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(ret("ayar dosyası bulunamadı")),
        Ok(_) => {}
    }
    let kok = kesin_kok(yol)?;
    let kok_once = fs::metadata(&kok).map_err(|_| ret("proje kökü okunamadı"))?;
    let (bayraklar, uyari) = cozumle(&dosyayi_oku(&kok.join("zee.ayar"))?)?;
    if kesin_kok(yol)? != kok {
        return Err(ret("proje kökü okuma sırasında değişti"));
    }
    let kok_sonra = fs::metadata(&kok).map_err(|_| ret("proje kökü okunamadı"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if kok_once.dev() != kok_sonra.dev() || kok_once.ino() != kok_sonra.ino() {
            return Err(ret("proje kökü okuma sırasında değişti"));
        }
    }
    #[cfg(not(unix))]
    if kok_once.modified().ok() != kok_sonra.modified().ok() {
        return Err(ret("proje kökü okuma sırasında değişti"));
    }
    let acik = &a[1..yer];
    let web_acik = acik
        .iter()
        .any(|s| ["--web-proxy", "--güvenli-web-proxy", "--deneysel-web"].contains(&s.as_str()));
    let mut ek = Vec::new();
    for (bayrak, deger) in bayraklar {
        let ayni = acik.iter().any(|s| {
            s == &bayrak
                || (bayrak == "--güvenli" && s == "--guvenli")
                || (bayrak == "--web-proxy" && s == "--güvenli-web-proxy")
        });
        let web = bayrak == "--web-proxy" || bayrak == "--deneysel-web";
        if !(ayni || web && web_acik) {
            ek.push(bayrak);
            ek.extend(deger);
        }
    }
    a.splice(yer..yer, ek);
    if uyari {
        eprintln!("{OLAGAN_UYARI}");
    }
    Ok(())
}
pub(super) fn yetenek_json() -> String {
    serde_json::json!({"schema":"zee-config-capability-1", "config_schemas":[SEMA], "unknown_security":"reject", "version":env!("CARGO_PKG_VERSION")}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn zarf(g: Value) -> Vec<u8> {
        serde_json::to_vec(&json!({"sema":SEMA,"surum":1,"ayarlar":{},"guvenlik":g})).unwrap()
    }
    #[test]
    fn guvenlik_tek_exact_allowlist_ve_kapali_varsayilan() {
        assert_eq!(
            GUVENLIK_ALANLARI,
            [
                "cocuk-modu",
                "web-modu",
                "proxy-origin",
                "worker-port",
                "oran-deposu",
                "oran-siniri",
                "oran-kapasitesi",
                "oran-doluluk"
            ]
        );
        assert_eq!(cozumle(&zarf(json!({}))).unwrap(), (vec![], false));
        for ad in ["debug", "web", "tls", "renk", "proxy_origin", "Web-Modu"] {
            assert!(cozumle(&zarf(json!({ad:false})))
                .unwrap_err()
                .contains("bilinmeyen güvenlik"));
        }
    }
    #[test]
    fn mevcut_bayraklara_esleme_ve_sinir_degerleri_exact() {
        let (a, uyari) = cozumle(&zarf(json!({"web-modu":"proxy","proxy-origin":"https://example.invalid","worker-port":65535,
            "oran-deposu":"bellek","oran-siniri":1000000,"oran-kapasitesi":32768,"oran-doluluk":"en-eski"}))).unwrap();
        assert!(!uyari);
        assert_eq!(
            a,
            vec![
                ("--web-proxy".into(), Some("https://example.invalid".into())),
                ("--web-worker-port".into(), Some("65535".into())),
                ("--web-oran-deposu".into(), Some("bellek".into())),
                ("--web-oran-siniri".into(), Some("1000000".into())),
                ("--web-oran-kapasitesi".into(), Some("32768".into())),
                ("--web-oran-doluluk".into(), Some("en-eski".into())),
            ]
        );
    }
    #[test]
    fn bilinen_guvenlik_alaninda_null_ve_yanlis_tur_ret_edilir() {
        for ad in GUVENLIK_ALANLARI {
            for v in [Value::Null, json!([]), json!({})] {
                assert!(cozumle(&zarf(json!({*ad:v}))).is_err(), "{ad}");
            }
        }
    }
    #[test]
    fn yol_bayt_ve_bilesen_sinirlari_dosya_acmadan_ret_edilir() {
        assert!(kesin_kok(Path::new(&"x".repeat(4097)))
            .unwrap_err()
            .contains("yolu sınırı"));
        assert!(kesin_kok(Path::new(&vec!["x"; 65].join("/")))
            .unwrap_err()
            .contains("yolu sınırı"));
    }
    #[test]
    fn cli_onceligi_gercek_loader_ve_web_parser_son_degerlerini_korur() {
        let kok = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!("zee-k234-loader-{}", std::process::id()));
        fs::create_dir_all(&kok).unwrap();
        fs::write(kok.join("proje.dil"), "unused marker").unwrap();
        fs::write(kok.join("zee.ayar"),zarf(json!({"web-modu":"proxy","proxy-origin":"https://config.invalid","worker-port":18123,
            "oran-deposu":"bellek","oran-siniri":25,"oran-kapasitesi":120,"oran-doluluk":"reddet"}))).unwrap();
        let mut a = [
            "çalıştır",
            "--güvenli-web-proxy",
            "https://cli.invalid",
            "--web-worker-port",
            "18124",
            "--web-oran-siniri",
            "20",
            "--web-oran-kapasitesi",
            "100",
            "--web-oran-doluluk",
            "en-eski",
        ]
        .map(str::to_owned)
        .to_vec();
        a.push(kok.to_string_lossy().into_owned());
        a.push("--uygulama-secimi".into());
        uygula(&mut a).unwrap();
        let web = super::super::web_modunu_ayikla(&mut a).unwrap();
        fs::remove_dir_all(&kok).unwrap();
        assert_eq!(
            web,
            super::super::WebModu::GuvenliProxy {
                origin: dil::yetkinlik::AgHedefi::https_origininden("https://cli.invalid").unwrap(),
                worker_kapi: Some(18124),
                listener_profile: None,
                oran_ayari: Some(
                    dil::web_guvenligi::BellekOranAyari::yeni(
                        20,
                        100,
                        dil::web_guvenligi::OranDoluluk::EnEski
                    )
                    .unwrap()
                )
            }
        );
        assert_eq!(
            a,
            vec![
                "çalıştır".to_string(),
                kok.to_string_lossy().into_owned(),
                "--uygulama-secimi".to_string()
            ]
        );
    }
}
