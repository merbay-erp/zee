//! dillsp — LSP sunucusu (stdio). Editör yapılandırmaları: editors/README.md.

use dil::lsp::Sunucu;
use std::io::{Read, Write};

fn main() {
    let mut sunucu = Sunucu::yeni();
    let stdin = std::io::stdin();
    let mut girdi = stdin.lock();
    let stdout = std::io::stdout();
    let mut cikti = stdout.lock();

    loop {
        let Some(govde) = mesaj_oku(&mut girdi) else {
            break; // stdin kapandı
        };
        let sonuc = sunucu.mesaj_isle(&govde);
        for yanit in &sonuc.govdeler {
            let _ = write!(cikti, "Content-Length: {}\r\n\r\n{}", yanit.len(), yanit);
            let _ = cikti.flush();
        }
        if !sonuc.devam {
            break;
        }
    }
}

/// Content-Length çerçeveli bir JSON-RPC gövdesi okur.
fn mesaj_oku(girdi: &mut impl Read) -> Option<String> {
    let mut baslik = Vec::new();
    let mut son_dort = [0u8; 4];
    loop {
        let mut bayt = [0u8; 1];
        if girdi.read_exact(&mut bayt).is_err() {
            return None;
        }
        baslik.push(bayt[0]);
        son_dort.rotate_left(1);
        son_dort[3] = bayt[0];
        if &son_dort == b"\r\n\r\n" {
            break;
        }
    }
    let baslik = String::from_utf8_lossy(&baslik);
    let uzunluk: usize = baslik
        .lines()
        .find_map(|satir| satir.strip_prefix("Content-Length:"))
        .and_then(|deger| deger.trim().parse().ok())?;
    let mut govde = vec![0u8; uzunluk];
    girdi.read_exact(&mut govde).ok()?;
    String::from_utf8(govde).ok()
}
