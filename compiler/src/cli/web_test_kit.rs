//! K286: additive, explicit operator-owned native suite; never suite-selected exec.
#[cfg(unix)]
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[cfg(unix)]
use std::path::{Path, PathBuf};

const FLAGS: [&str; 6] = [
    "--web-suite",
    "--web-suite-sha256",
    "--web-fixtures",
    "--test-python",
    "--test-postgres-bin",
    "--test-output",
];
#[cfg(unix)]
const KIT: &[u8] = include_bytes!("../../../scripts/owned-web-tests.py");
#[cfg(unix)]
const OWNER: &[u8] = include_bytes!("../../../scripts/worker-supervisor.py");

pub(crate) struct Plan {
    values: BTreeMap<String, String>,
}
pub(crate) fn ayikla(args: &mut Vec<String>) -> Result<Option<Plan>, String> {
    let mut values = BTreeMap::new();
    let mut i = 1;
    while i < args.len() {
        if FLAGS.contains(&args[i].as_str()) {
            if args.first().map(String::as_str) != Some("dene")
                || i + 1 >= args.len()
                || values.contains_key(&args[i])
            {
                return Err("native test seçenekleri geçersiz".into());
            }
            values.insert(args[i].clone(), args[i + 1].clone());
            args.drain(i..i + 2);
        } else {
            i += 1;
        }
    }
    if values.is_empty() {
        return Ok(None);
    }
    if values.len() != FLAGS.len() {
        return Err("native test seçenekleri birlikte verilmelidir".into());
    }
    let plan = Plan { values };
    plan.preflight()?;
    Ok(Some(plan))
}
#[cfg(unix)]
fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
#[cfg(unix)]
fn bounded(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let meta = path
        .symlink_metadata()
        .map_err(|_| "native test dosya rolü geçersiz")?;
    if !meta.is_file() || meta.len() > max {
        return Err("native test dosya bütçesi aşıldı".into());
    }
    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(path)
    };
    #[cfg(not(unix))]
    let mut file = std::fs::File::open(path);
    let file = file.as_mut().map_err(|_| "native test dosyası açılamadı")?;
    let opened = file
        .metadata()
        .map_err(|_| "native test dosya rolü geçersiz")?;
    if !opened.is_file() || opened.len() != meta.len() {
        return Err("native test dosyası değişti".into());
    }
    let mut bytes = Vec::new();
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "native test dosyası okunamadı")?;
    if bytes.len() as u64 > max {
        return Err("native test dosya bütçesi aşıldı".into());
    }
    Ok(bytes)
}
impl Plan {
    #[cfg(unix)]
    fn path(&self, key: &str) -> PathBuf {
        PathBuf::from(&self.values[key])
    }
    #[cfg_attr(not(unix), allow(clippy::needless_return))]
    fn preflight(&self) -> Result<(), String> {
        #[cfg(not(unix))]
        {
            let _ = &self.values;
            return Err("native test sahipliği bu platformda doğrulanmadı".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let uid = unsafe { libc::geteuid() };
            let out = self.path("--test-output");
            let parent = out.parent().ok_or("native test çıktı rolü geçersiz")?;
            if uid == 0
                || !out.is_absolute()
                || out.components().count() > 32
                || out.as_os_str().len() > 4096
                || parent.canonicalize().ok().as_deref() != Some(parent)
                || parent
                    .metadata()
                    .map_err(|_| "native test çıktı rolü geçersiz")?
                    .uid()
                    != uid
                || out.symlink_metadata().is_ok()
            {
                return Err("native test fresh owned çıktı ister".into());
            }
            let suite = self.path("--web-suite");
            if !suite.is_absolute()
                || sha(&bounded(&suite, 65536)?) != self.values["--web-suite-sha256"]
            {
                return Err("native test suite hash geçersiz".into());
            }
            let python = self.path("--test-python");
            if !python.is_absolute()
                || !python
                    .file_name()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.starts_with("python3"))
            {
                return Err("native test Python rolü geçersiz".into());
            }
            bounded(
                &python
                    .canonicalize()
                    .map_err(|_| "native test Python rolü geçersiz")?,
                64 * 1024 * 1024,
            )?;
            for name in ["initdb", "postgres", "psql"] {
                bounded(
                    &self.path("--test-postgres-bin").join(name),
                    64 * 1024 * 1024,
                )?;
            }
            let root = self.path("--web-fixtures");
            if !root.is_absolute()
                || !root.is_dir()
                || root.canonicalize().ok().as_deref() != Some(root.as_path())
            {
                return Err("native test fixture rolü geçersiz".into());
            }
            Ok(())
        }
    }
    #[cfg(unix)]
    pub(crate) fn calistir(
        self,
        passed: bool,
        source: &str,
        origin: &str,
        unit_coverage: Option<serde_json::Value>,
    ) -> Result<(), String> {
        use std::io::Write;
        self.preflight()?;
        let out = self.path("--test-output");
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&out)
                .map_err(|_| "native test çıktı oluşturulamadı")?;
        }
        #[cfg(not(unix))]
        {
            let _ = (&source, &origin);
            return Err("native test Unix owned capability ister".into());
        }
        let binary = std::env::current_exe()
            .map_err(|_| "native test binary rolü geçersiz")?
            .canonicalize()
            .map_err(|_| "native test binary rolü geçersiz")?;
        let build: serde_json::Value =
            serde_json::from_str(dil::surum::json()).map_err(|_| "native test Build geçersiz")?;
        let python = self
            .path("--test-python")
            .canonicalize()
            .map_err(|_| "native test Python rolü geçersiz")?;
        let embedded: BTreeMap<String, String> = dil::gomulu_birim_adlari()
            .iter()
            .map(|name| {
                let source =
                    dil::gomulu_birim(name).ok_or("native test gömülü kaynak kaydı eksik")?;
                Ok((format!("gömülü:{name}"), sha(source.as_bytes())))
            })
            .collect::<Result<_, String>>()?;
        let receipt = serde_json::json!({"schema":"zee-dene-unit-before-native-1","passed":passed,"compiler_build":build,"source":origin,"source_sha256":sha(source.as_bytes()),"python_sha256":sha(&bounded(&python,64*1024*1024)?),"native_started":passed,"embedded_sources":embedded,"source_coverage":unit_coverage});
        let unit_bytes =
            serde_json::to_vec(&receipt).map_err(|_| "native test unit kanıtı geçersiz")?;
        if unit_bytes.len() > 16384 {
            return Err("native test unit kanıtı bütçesi aşıldı".into());
        }
        std::fs::File::create(out.join("unit.json"))
            .and_then(|mut f| f.write_all(&unit_bytes))
            .map_err(|_| "native test unit kanıtı yazılamadı")?;
        if !passed {
            return Ok(());
        }
        let runtime = out.join("runtime").join("scripts");
        std::fs::create_dir_all(&runtime).map_err(|_| "native test runtime oluşturulamadı")?;
        for (name, bytes) in [("owned-web-tests.py", KIT), ("worker-supervisor.py", OWNER)] {
            std::fs::File::create(runtime.join(name))
                .and_then(|mut f| f.write_all(bytes))
                .map_err(|_| "native test gömülü runtime yazılamadı")?;
        }
        let mut command = std::process::Command::new(&python);
        command
            .args(["-I", "-B"])
            .arg(runtime.join("owned-web-tests.py"));
        for (flag, key) in [
            ("--fixture-root", "--web-fixtures"),
            ("--suite", "--web-suite"),
            ("--suite-sha256", "--web-suite-sha256"),
            ("--postgres-bin-dir", "--test-postgres-bin"),
        ] {
            command.arg(flag).arg(&self.values[key]);
        }
        command
            .arg("--dil")
            .arg(&binary)
            .arg("--binary-sha256")
            .arg(sha(&bounded(&binary, 256 * 1024 * 1024)?))
            .arg("--expected-source")
            .arg(
                build["source_sha"]
                    .as_str()
                    .ok_or("native test Build source geçersiz")?,
            )
            .arg("--output")
            .arg(out.join("results"))
            .arg("--source-coverage")
            .arg("--unit-receipt")
            .arg(out.join("unit.json"));
        if build["dirty"] == true {
            command.arg("--candidate");
        }
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("LANG", "C.UTF-8")
            .stdin(std::process::Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let _ = command.exec();
            Err("native test sabit Python başlatılamadı".into())
        }
        #[cfg(not(unix))]
        Err("native test Unix owned capability ister".into())
    }
    #[cfg(not(unix))]
    pub(crate) fn calistir(
        self,
        passed: bool,
        source: &str,
        origin: &str,
        unit_coverage: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let _ = (self, passed, source, origin, unit_coverage);
        Err("native test Unix owned capability ister".into())
    }
}
