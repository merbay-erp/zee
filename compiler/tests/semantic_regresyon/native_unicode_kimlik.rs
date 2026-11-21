//! K-363 (GB-109 Unicode gösterim/rename/registry): paket, birim ve yeniden
//! adlandırma kimliklerinin Unicode biçimi. Her vaka `# senaryo: <ad>`
//! başlığıyla tek arızayı seçer; senaryo verisi çalışma anında üretilir,
//! çünkü büyük/küçük harf yalnız-farklı dosyalar ya da görünmez biçim
//! karakterli adlar depoya güvenle alınamaz (macOS/Windows checkout'u ikinci
//! dosyayı yazamaz; Trojan Source denetimi görünmez karakteri reddeder).
//!
//! - `paket-adi`: spec/07 "küçük harfli tek Türkçe/Latin tanımlayıcı";
//!   eski `is_alphabetic` Kiril/Yunan homoglifini, CJK'yı ve birleştirici
//!   imi kabul ediyordu.
//! - `zep-harf-katlama`: `A.dil`/`a.dil` ya da `K/`/`k/` taşıyan arşiv Linux'ta
//!   kuruluyor, büyük/küçük harf duyarsız dosya sisteminde düşüyordu.
//! - `zep-alm`: ALM (U+061C) bidi biçim karakteri `.zep` yolunda kabul
//!   ediliyordu.
//! - `birim-harf-farki`: `çay birimini kullan` macOS/Windows'ta `Çay.dil`'i
//!   bağlıyor, Linux'ta bulamıyordu; NFD saklanan `çay.dil` Linux'ta
//!   bulunamıyordu. Vaka kaynağı birimin kendisidir.
//! - `lsp-yeni-ad`: LSP yeniden adlandırma lexer'ın reddedeceği adı kabul
//!   edip derlenmeyen kaynak üretiyordu.
//! - `kabul-gerekcesi`: kilide yazılan kabul gerekçesi bidi/sıfır genişlikli
//!   biçim karakteri taşıyabiliyordu (`char::is_control` yalnız Cc).
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

struct Temizle(PathBuf);

impl Drop for Temizle {
    fn drop(&mut self) {
        fn yazilabilir(yol: &Path) {
            let Ok(metadata) = std::fs::symlink_metadata(yol) else {
                return;
            };
            if metadata.is_dir() {
                for girdi in std::fs::read_dir(yol).into_iter().flatten().flatten() {
                    yazilabilir(&girdi.path());
                }
            }
            let mut izin = metadata.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            izin.set_readonly(false);
            let _ = std::fs::set_permissions(yol, izin);
        }
        yazilabilir(&self.0);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn gecici(ad: &str) -> Temizle {
    let kok = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-unicode-{ad}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    std::fs::create_dir_all(&kok).expect("geçici kök");
    Temizle(kok)
}

/// Ham `.zep` v1 baytları; üretici denetimlerinden geçmeyen tüketici girdisi.
fn arsiv(girdiler: &[(&str, &str)]) -> Vec<u8> {
    let mut baytlar = b"ZEEZEP\0\x01".to_vec();
    baytlar.extend_from_slice(&(girdiler.len() as u32).to_be_bytes());
    for (yol, icerik) in girdiler {
        baytlar.extend_from_slice(&(yol.len() as u32).to_be_bytes());
        baytlar.extend_from_slice(yol.as_bytes());
        baytlar.extend_from_slice(&(icerik.len() as u64).to_be_bytes());
        baytlar.extend_from_slice(icerik.as_bytes());
    }
    baytlar
}

fn kur(kok: &Path, ad: &str, girdiler: &[(&str, &str)], ret_parcasi: &str) -> String {
    let arsiv_yolu = kok.join(format!("{ad}.zep"));
    std::fs::write(&arsiv_yolu, arsiv(girdiler)).expect("arşiv");
    let hedef = kok.join("kurulum").join(ad);
    let sonuc = match dil::artefakt_dogrulama::paket_arsivini_kur(&arsiv_yolu, &hedef) {
        Ok(()) => "kuruldu",
        Err(hata) if hata.contains(ret_parcasi) => "ret",
        Err(_) => "dosya sistemi hatası",
    };
    let yazim = if hedef.exists() { "var" } else { "yok" };
    format!("{ad} {sonuc} kurulum {yazim}")
}

fn birim_calistir(kok: &Path, birim_kaynagi: &str, dosya: &str) -> String {
    std::fs::create_dir_all(kok).expect("proje");
    std::fs::write(
        kok.join("proje.dil"),
        "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::write(kok.join("ana.dil"), "çay birimini kullan\nselam ver\n").expect("ana");
    std::fs::write(kok.join(dosya), birim_kaynagi).expect("birim");
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", kok.to_str().expect("utf8 yol")])
        .output()
        .expect("dil çalıştır");
    let stdout = String::from_utf8_lossy(&cikti.stdout);
    let stderr = String::from_utf8_lossy(&cikti.stderr);
    let sonuc = if stdout == "merhaba\n" {
        "merhaba".to_string()
    } else if stderr.contains("A010") && stderr.contains("birebir eşleşmeli") {
        "A010 ad farkı".to_string()
    } else if stderr.contains("A010") {
        "A010 bulunamadı".to_string()
    } else {
        format!("beklenmeyen {}", stderr.lines().next().unwrap_or(""))
    };
    format!("çıkış {} {sonuc}", cikti.status.code().unwrap_or(-1))
}

fn senaryo(kaynak: &str) -> &str {
    kaynak
        .lines()
        .find_map(|satir| satir.strip_prefix("# senaryo: "))
        .expect("vaka `# senaryo:` başlığı taşımalı")
}

pub fn calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    let kaynak = std::fs::read_to_string(kaynak_yolu).expect("vaka kaynağı");
    let satirlar = match senaryo(&kaynak) {
        "paket-adi" => [
            ("kiril", "\u{0440}\u{0430}ket"),
            ("yunan", "\u{03c1}aket"),
            ("cjk", "日本"),
            ("birleşik-im", "a\u{0345}"),
            ("bağ-harfi", "\u{fb01}le"),
            ("türkçe", "çay"),
        ]
        .into_iter()
        .map(|(ad, paket)| {
            let sonuc = dil::proje::UzakBagimlilik::ayristir(&format!("{paket}@1.0.0"));
            format!("{ad} {}", if sonuc.is_ok() { "kabul" } else { "ret" })
        })
        .collect(),
        "zep-harf-katlama" => {
            let kok = gecici("zep-harf");
            let ret = "büyük/küçük harf duyarsız";
            vec![
                kur(&kok.0, "dosya", &[("A.dil", "x"), ("a.dil", "y")], ret),
                kur(&kok.0, "klasör", &[("K/a.dil", "x"), ("k/b.dil", "y")], ret),
                kur(&kok.0, "ayrı", &[("a.dil", "x"), ("b.dil", "y")], ret),
            ]
        }
        "zep-alm" => {
            let kok = gecici("zep-alm");
            vec![
                kur(&kok.0, "alm", &[("kaynak/a\u{061c}b.dil", "x")], "Güvensiz"),
                kur(&kok.0, "rlm", &[("kaynak/a\u{200f}b.dil", "x")], "Güvensiz"),
            ]
        }
        "birim-harf-farki" => {
            let kok = gecici("birim");
            vec![
                format!(
                    "büyük {}",
                    birim_calistir(&kok.0.join("buyuk"), &kaynak, "Çay.dil")
                ),
                format!(
                    "nfd {}",
                    birim_calistir(&kok.0.join("nfd"), &kaynak, "c\u{0327}ay.dil")
                ),
                format!(
                    "tam {}",
                    birim_calistir(&kok.0.join("tam"), &kaynak, "çay.dil")
                ),
            ]
        }
        "lsp-yeni-ad" => {
            let mut sunucu = dil::lsp::Sunucu::yeni();
            let metin = kaynak.replace('\n', "\\n").replace('"', "\\\"");
            sunucu.mesaj_isle(&format!(
                r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///tmp/k363.dil","text":"{metin}"}}}}}}"#
            ));
            let satir = kaynak
                .lines()
                .position(|satir| satir.starts_with("sayaç "))
                .expect("sayaç tanımı");
            [
                ("kiril", "\u{0440}\u{0430}ket"),
                ("rakamla", "3kat"),
                ("cjk", "日本"),
                ("türkçe", "puan"),
            ]
            .into_iter()
            .map(|(ad, yeni)| {
                let yanit = sunucu.mesaj_isle(&format!(
                    r#"{{"jsonrpc":"2.0","id":1,"method":"textDocument/rename","params":{{"textDocument":{{"uri":"file:///tmp/k363.dil"}},"position":{{"line":{satir},"character":1}},"newName":"{yeni}"}}}}"#
                ));
                let govde = &yanit.govdeler[0];
                if govde.contains("\"result\":null") {
                    format!("{ad} ret")
                } else {
                    format!("{ad} düzenleme {}", govde.matches("newText").count())
                }
            })
            .collect()
        }
        "kabul-gerekcesi" => [
            ("rlo", "onay\u{202e}landı"),
            ("sıfır-genişlik", "a\u{200b}b"),
            ("alm", "a\u{061c}b"),
            ("düz", "bilinçli kabul"),
        ]
        .into_iter()
        .map(|(ad, gerekce)| {
            let sonuc =
                dil::paket::RegistryCozumPolitikasi::yeni(false, Some(gerekce.into()), None);
            format!("{ad} {}", if sonuc.is_ok() { "kabul" } else { "ret" })
        })
        .collect(),
        diger => panic!("bilinmeyen K-363 senaryosu: {diger}"),
    };
    (0, satirlar)
}
