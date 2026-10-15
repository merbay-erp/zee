//! CLI-only opt-in coverage report ownership; no ambient environment activation.
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

pub(crate) fn ayikla(args: &mut Vec<String>) -> Result<Option<Rapor>, String> {
    let komut = args.first().map(String::as_str).unwrap_or("");
    let calistir = matches!(komut, "çalıştır" | "calistir");
    let dene = komut == "dene";
    let mut yer = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--kapsam" {
            if (!calistir && !dene) || yer.is_some() || i + 1 >= args.len() {
                return Err("--kapsam seçenekleri geçersiz".into());
            }
            yer = Some(i);
            i += 2;
        } else if args[i].starts_with("--") {
            i += if matches!(
                args[i].as_str(),
                "--ayar"
                    | "--web-suite"
                    | "--web-suite-sha256"
                    | "--web-fixtures"
                    | "--test-python"
                    | "--test-postgres-bin"
                    | "--test-output"
                    | "--profil"
                    | "--web-proxy"
                    | "--güvenli-web-proxy"
                    | "--web-metrik-fd"
                    | "--web-listener-fd"
                    | "--web-ready-fd"
                    | "--web-activate-fd"
                    | "--web-generation"
                    | "--web-worker-port"
                    | "--web-oran-deposu"
                    | "--web-oran-siniri"
                    | "--web-oran-kapasitesi"
                    | "--web-oran-doluluk"
            ) {
                2
            } else {
                1
            };
        } else if calistir {
            break;
        } else {
            i += 1;
        }
    }
    let Some(i) = yer else {
        return Ok(None);
    };
    let rapor = Rapor::yeni(Path::new(&args[i + 1]))?;
    args.drain(i..i + 2);
    Ok(Some(rapor))
}

pub(crate) struct Rapor {
    dosya: RefCell<File>,
    yol: std::path::PathBuf,
    ozet: RefCell<Option<serde_json::Value>>,
}
impl Rapor {
    #[cfg(unix)]
    fn yeni(yol: &Path) -> Result<Self, String> {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        let ret = || "--kapsam fresh canonical owned normal dosya ister".to_string();
        let parent = yol.parent().ok_or_else(ret)?;
        // SAFETY: geteuid has no pointer arguments or preconditions.
        let uid = unsafe { libc::geteuid() };
        if !yol.is_absolute()
            || yol.as_os_str().len() > 4096
            || yol.components().count() > 32
            || parent.canonicalize().map_err(|_| ret())? != parent
            || parent.metadata().map_err(|_| ret())?.uid() != uid
            || uid == 0
        {
            return Err(ret());
        }
        let mut dosya = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(yol)
            .map_err(|_| ret())?;
        dosya.write_all(b"{\"schema\":\"zee-source-coverage-1\",\"complete\":false,\"reason\":\"execution-not-finalized\"}\n")
            .and_then(|_|dosya.flush()).map_err(|_|ret())?;
        Ok(Self {
            dosya: RefCell::new(dosya),
            yol: yol.into(),
            ozet: RefCell::new(None),
        })
    }
    #[cfg(not(unix))]
    fn yeni(_yol: &Path) -> Result<Self, String> {
        Err("--kapsam rapor sahipliği bu platformda doğrulanmadı".into())
    }
    pub(crate) fn ozet(&self) -> Option<serde_json::Value> {
        self.ozet.borrow().clone()
    }
    pub(crate) fn yaz(&self, mut veri: serde_json::Value) -> Result<(), String> {
        veri["compiler_build"] = serde_json::from_str(dil::surum::json())
            .map_err(|_| "kapsam build kimliği geçersiz")?;
        let raw = serde_json::to_vec(&veri).map_err(|_| "kapsam JSON geçersiz")?;
        if raw.len() > dil::test_kapsami::AZAMI_RAPOR {
            return Err("kapsam rapor bütçesi aşıldı".into());
        }
        let mut f = self.dosya.borrow_mut();
        f.seek(SeekFrom::Start(0))
            .and_then(|_| f.write_all(&raw))
            .and_then(|_| f.set_len(raw.len() as u64))
            .and_then(|_| f.flush())
            .map_err(|_| "kapsam raporu yazılamadı".to_string())?;
        let hash = Sha256::digest(&raw)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        *self.ozet.borrow_mut() = Some(
            serde_json::json!({"path":self.yol,"sha256":hash,"complete":veri["complete"],"product":{"payda":veri["product"]["payda"],"vurulan":veri["product"]["vurulan"]},"tests":{"payda":veri["tests"]["payda"],"vurulan":veri["tests"]["vurulan"]}}),
        );
        Ok(())
    }
}

pub(crate) fn kaynak_ekle(
    m: &mut BTreeMap<String, String>,
    koken: &str,
    kaynak: &str,
) -> Result<(), String> {
    if koken.len() > 4096 || (m.len() >= dil::test_kapsami::AZAMI_KAYNAK && !m.contains_key(koken))
    {
        return Err("kapsam kaynak bütçesi aşıldı".into());
    }
    let hash = Sha256::digest(kaynak.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    if m.get(koken).is_some_and(|eski| eski != &hash) {
        return Err("kapsam aynı köken farklı snapshot reddedildi".into());
    }
    m.insert(koken.into(), hash);
    Ok(())
}
