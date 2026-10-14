//! K-248: yüzde UTF-8 dosya URI'si gerçek stdio LSP ve disk birimiyle sınanır.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct GeciciKlasor(PathBuf);

impl Drop for GeciciKlasor {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(super) fn calistir(kaynak: &str, birim: &Path) -> (i64, Vec<String>) {
    let gecici = GeciciKlasor(std::env::temp_dir().join(format!(
        "zee-lsp-uri-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    )));
    std::fs::create_dir(&gecici.0).expect("tekil geçici kök");
    let dizin = gecici.0.join("Türkçe ö 😀 [1]+%");
    std::fs::create_dir(&dizin).expect("UTF-8 klasör");
    std::fs::write(dizin.join("ana.dil"), kaynak).expect("ana kaynak");
    std::fs::copy(birim, dizin.join("yardimci.dil")).expect("hermetik birim");
    let yol = dizin.join("ana.dil").to_str().unwrap().replace('\\', "/");
    let mut uri = String::from("file://");
    for bayt in yol.bytes() {
        if bayt.is_ascii_alphanumeric() || b"/-._~:".contains(&bayt) {
            uri.push(char::from(bayt));
        } else {
            use std::fmt::Write;
            write!(&mut uri, "%{bayt:02X}").unwrap();
        }
    }
    let mesajlar = [
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        serde_json::json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"dil","version":1,"text":kaynak}}}),
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"shutdown"}),
        serde_json::json!({"jsonrpc":"2.0","method":"exit"}),
    ];
    let mut girdi = Vec::new();
    for mesaj in mesajlar {
        let govde = serde_json::to_vec(&mesaj).unwrap();
        write!(&mut girdi, "Content-Length: {}\r\n\r\n", govde.len()).unwrap();
        girdi.extend(govde);
    }
    let mut cocuk = Command::new(env!("CARGO_BIN_EXE_dillsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("gerçek dillsp");
    cocuk
        .stdin
        .take()
        .unwrap()
        .write_all(&girdi)
        .expect("stdio mesajları");
    let son = Instant::now() + Duration::from_secs(10);
    while cocuk.try_wait().expect("LSP wait").is_none() {
        if Instant::now() >= son {
            let _ = cocuk.kill();
            let _ = cocuk.wait();
            panic!("LSP regresyonu 10 saniyede kapanmadı");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let cikti = cocuk.wait_with_output().expect("LSP çıkışı");
    assert!(cikti.stderr.is_empty(), "LSP stderr boş olmalı");
    assert!(cikti.stdout.len() <= 262_144, "fixture çıktı sınırı");
    let mut kalan = cikti.stdout.as_slice();
    let mut yanitlar = Vec::new();
    while !kalan.is_empty() {
        let ayrac = kalan
            .windows(4)
            .position(|p| p == b"\r\n\r\n")
            .expect("LSP başlığı");
        let baslik = std::str::from_utf8(&kalan[..ayrac]).unwrap();
        let boy = baslik
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        kalan = &kalan[ayrac + 4..];
        assert!(boy <= kalan.len(), "tam LSP gövdesi");
        yanitlar.push(serde_json::from_slice::<serde_json::Value>(&kalan[..boy]).unwrap());
        kalan = &kalan[boy..];
    }
    assert_eq!(yanitlar.len(), 3, "initialize, tanı, shutdown");
    assert_eq!(yanitlar[0]["id"], 1);
    assert_eq!(yanitlar[2]["id"], 2);
    assert!(yanitlar[2]["result"].is_null());
    assert_eq!(yanitlar[1]["method"], "textDocument/publishDiagnostics");
    assert_eq!(yanitlar[1]["params"]["uri"], uri);
    let kodlar = yanitlar[1]["params"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tani| tani["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    (
        i64::from(cikti.status.code().expect("normal LSP exit")),
        vec![serde_json::to_string(&kodlar).unwrap()],
    )
}
