//! K-248: yüzde UTF-8 dosya URI'si gerçek stdio LSP ve disk birimiyle sınanır.
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const AZAMI_GIRDI_BAYTI: usize = 4096;
const AZAMI_CIKTI_BAYTI: usize = 262_144;

struct SahipliCocuk {
    cocuk: Child,
    isciler: Vec<JoinHandle<()>>,
}

impl SahipliCocuk {
    fn okuyucu<R: Read + Send + 'static>(&mut self, akis: R) -> Receiver<std::io::Result<Vec<u8>>> {
        let (gonder, al) = mpsc::channel();
        self.isciler.push(std::thread::spawn(move || {
            let mut baytlar = Vec::new();
            let sonuc = akis
                .take((AZAMI_CIKTI_BAYTI + 1) as u64)
                .read_to_end(&mut baytlar)
                .map(|_| baytlar);
            let _ = gonder.send(sonuc);
        }));
        al
    }

    fn iscileri_topla(&mut self) {
        while let Some(isci) = self.isciler.pop() {
            isci.join().expect("sahipli stdio işçisi");
        }
    }
}

impl Drop for SahipliCocuk {
    fn drop(&mut self) {
        // Setup/write/wait panic'i dahil sahipli süreç ve pipe işçileri kalmaz.
        let _ = self.cocuk.kill();
        let _ = self.cocuk.wait();
        while let Some(isci) = self.isciler.pop() {
            let _ = isci.join();
        }
    }
}

fn kalan_sure(son: Instant) -> Duration {
    son.checked_duration_since(Instant::now())
        .expect("LSP stdio mutlak 10 saniyelik deadline'ı aştı")
}

struct GeciciKlasor(PathBuf);

impl Drop for GeciciKlasor {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(super) fn calistir(kaynak: &str, birim: &Path) -> (i64, Vec<String>) {
    assert!(
        kaynak.len() <= AZAMI_GIRDI_BAYTI,
        "küçük fixture kaynak sınırı"
    );
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
    assert!(
        girdi.len() <= AZAMI_GIRDI_BAYTI,
        "küçük fixture girdi sınırı"
    );
    let son = Instant::now() + Duration::from_secs(10);
    let cocuk = Command::new(env!("CARGO_BIN_EXE_dillsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("gerçek dillsp");
    let mut sahip = SahipliCocuk {
        cocuk,
        isciler: Vec::new(),
    };
    let cikti_akisi = sahip.cocuk.stdout.take().expect("LSP stdout pipe");
    let hata_akisi = sahip.cocuk.stderr.take().expect("LSP stderr pipe");
    let cikti_al = sahip.okuyucu(cikti_akisi);
    let hata_al = sahip.okuyucu(hata_akisi);
    let mut stdin = sahip.cocuk.stdin.take().expect("LSP stdin pipe");
    let (yazildi_gonder, yazildi_al) = mpsc::channel();
    sahip.isciler.push(std::thread::spawn(move || {
        let sonuc = stdin.write_all(&girdi);
        drop(stdin); // EOF; wait ederken açık writer handle'ı bırakılmaz.
        let _ = yazildi_gonder.send(sonuc);
    }));
    yazildi_al
        .recv_timeout(kalan_sure(son))
        .expect("LSP stdin deadline/işçi")
        .expect("stdio mesajları");
    let durum = loop {
        if let Some(durum) = sahip.cocuk.try_wait().expect("LSP wait") {
            break durum;
        }
        std::thread::sleep(kalan_sure(son).min(Duration::from_millis(10)));
    };
    let cikti = cikti_al
        .recv_timeout(kalan_sure(son))
        .expect("LSP stdout deadline/işçi")
        .expect("LSP stdout okuması");
    let hata = hata_al
        .recv_timeout(kalan_sure(son))
        .expect("LSP stderr deadline/işçi")
        .expect("LSP stderr okuması");
    sahip.iscileri_topla();
    assert!(hata.is_empty(), "LSP stderr boş olmalı");
    assert!(cikti.len() <= AZAMI_CIKTI_BAYTI, "fixture çıktı sınırı");
    let mut kalan = cikti.as_slice();
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
    assert!(yanitlar[0].get("error").is_none());
    assert!(yanitlar[0]["result"]["capabilities"].is_object());
    assert_eq!(yanitlar[2]["id"], 2);
    assert!(yanitlar[2].get("error").is_none());
    assert_eq!(yanitlar[2].get("result"), Some(&serde_json::Value::Null));
    assert_eq!(yanitlar[1]["method"], "textDocument/publishDiagnostics");
    assert_eq!(yanitlar[1]["params"]["uri"], uri);
    let kodlar = yanitlar[1]["params"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tani| tani["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    (
        i64::from(durum.code().expect("normal LSP exit")),
        vec![serde_json::to_string(&kodlar).unwrap()],
    )
}
