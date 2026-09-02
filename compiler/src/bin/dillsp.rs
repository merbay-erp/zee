#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]

//! dillsp — LSP sunucusu (stdio). Editör yapılandırmaları: editors/README.md.

use dil::lsp::{Sunucu, AZAMI_LSP_BASLIK_BAYTI, AZAMI_LSP_GOVDE_BAYTI};
use std::io::{Read, Write};

fn main() {
    let mut sunucu = Sunucu::yeni();
    let stdin = std::io::stdin();
    let mut girdi = stdin.lock();
    let stdout = std::io::stdout();
    let mut cikti = stdout.lock();

    loop {
        let govde = match mesaj_oku(&mut girdi) {
            Ok(Some(govde)) => govde,
            Ok(None) => break,
            Err(hata) => {
                eprintln!("dillsp girdi hatası: {}", hata);
                break;
            }
        };
        let sonuc = sunucu.mesaj_baytlari_isle(&govde);
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
fn mesaj_oku(girdi: &mut impl Read) -> Result<Option<Vec<u8>>, String> {
    let mut baslik = Vec::new();
    let mut son_dort = [0u8; 4];
    loop {
        if baslik.len() >= AZAMI_LSP_BASLIK_BAYTI {
            return Err(format!(
                "LSP başlığı {} KiB sınırını aşıyor",
                AZAMI_LSP_BASLIK_BAYTI / 1024
            ));
        }
        let mut bayt = [0u8; 1];
        if let Err(hata) = girdi.read_exact(&mut bayt) {
            if baslik.is_empty() && hata.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err("LSP başlığı tamamlanmadan girdi bitti".into());
        }
        baslik.push(bayt[0]);
        son_dort.rotate_left(1);
        son_dort[3] = bayt[0];
        if &son_dort == b"\r\n\r\n" {
            break;
        }
    }
    let baslik = std::str::from_utf8(&baslik).map_err(|_| "LSP başlığı UTF-8 değil")?;
    let mut uzunluk = None;
    for satir in baslik.split("\r\n").filter(|satir| !satir.is_empty()) {
        let (ad, deger) = satir
            .split_once(':')
            .ok_or_else(|| "LSP başlık satırı geçersiz".to_string())?;
        if ad.eq_ignore_ascii_case("Content-Length") {
            if uzunluk.is_some() {
                return Err("birden çok Content-Length başlığı reddedildi".into());
            }
            uzunluk = Some(
                deger
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Content-Length geçersiz")?,
            );
        }
    }
    let uzunluk = uzunluk.ok_or_else(|| "Content-Length başlığı eksik".to_string())?;
    if uzunluk > AZAMI_LSP_GOVDE_BAYTI {
        return Err(format!(
            "LSP gövdesi {} MiB sınırını aşıyor",
            AZAMI_LSP_GOVDE_BAYTI / (1024 * 1024)
        ));
    }
    let mut govde = vec![0u8; uzunluk];
    girdi
        .read_exact(&mut govde)
        .map_err(|_| "LSP gövdesi tamamlanmadan girdi bitti")?;
    Ok(Some(govde))
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn kucuk_content_length_cercevesini_okur() {
        let girdi = b"Content-Length: 2\r\n\r\n{}";
        let mut girdi = std::io::Cursor::new(girdi);
        assert_eq!(mesaj_oku(&mut girdi).unwrap(), Some(b"{}".to_vec()));
    }

    #[test]
    fn baslik_ve_govde_sinirini_tahsis_oncesi_uygular() {
        let uzun_baslik = vec![b'x'; AZAMI_LSP_BASLIK_BAYTI];
        let mut uzun_baslik = std::io::Cursor::new(uzun_baslik);
        assert!(mesaj_oku(&mut uzun_baslik).unwrap_err().contains("başlığı"));

        let girdi = format!("Content-Length: {}\r\n\r\n", AZAMI_LSP_GOVDE_BAYTI + 1);
        let mut girdi = std::io::Cursor::new(girdi);
        assert!(mesaj_oku(&mut girdi).unwrap_err().contains("gövdesi"));
    }

    #[test]
    fn eksik_tekrarli_ve_gecersiz_content_length_reddedilir() {
        for girdi in [
            "Content-Type: application/json\r\n\r\n",
            "Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}",
            "Content-Length: iki\r\n\r\n{}",
        ] {
            assert!(mesaj_oku(&mut std::io::Cursor::new(girdi)).is_err());
        }
    }

    #[test]
    fn utf8_disini_cerceveler_ve_sonraki_mesaja_devam_eder() {
        let ikinci = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let mut girdi = Vec::new();
        girdi.extend_from_slice(b"Content-Length: 2\r\n\r\n\xff\xfe");
        girdi.extend_from_slice(format!("Content-Length: {}\r\n\r\n", ikinci.len()).as_bytes());
        girdi.extend_from_slice(ikinci);
        let mut girdi = std::io::Cursor::new(girdi);
        let mut sunucu = Sunucu::yeni();

        let ilk = mesaj_oku(&mut girdi).unwrap().expect("ilk çerçeve");
        let ilk_cikti = sunucu.mesaj_baytlari_isle(&ilk);
        assert!(ilk_cikti.govdeler[0].contains("\"code\":-32700"));
        assert!(ilk_cikti.devam);

        let ikinci = mesaj_oku(&mut girdi).unwrap().expect("ikinci çerçeve");
        let ikinci_cikti = sunucu.mesaj_baytlari_isle(&ikinci);
        assert!(ikinci_cikti.govdeler[0].contains("\"id\":1"));
        assert!(ikinci_cikti.govdeler[0].contains("\"result\""));
    }
}
