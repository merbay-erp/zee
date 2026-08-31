//! `dil` — resmi CLI (master plan bölüm 15).
//!
//! Komutlar Türkçedir: çalıştır, denetle, sürüm.

use std::process::ExitCode;

fn main() -> ExitCode {
    let argumanlar: Vec<String> = std::env::args().skip(1).collect();

    match argumanlar.first().map(|s| s.as_str()) {
        Some("çalıştır") | Some("calistir") => dosya_ile(&argumanlar, calistir_komutu),
        Some("denetle") => dosya_ile(&argumanlar, denetle_komutu),
        Some("sürüm") | Some("surum") => {
            println!("dil {} — Türkçe programlama dili (bootstrap, Stage 0)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            kullanim();
            ExitCode::from(2)
        }
    }
}

fn kullanim() {
    eprintln!("Türkçe programlama dili — resmi CLI\n");
    eprintln!("Kullanım:");
    eprintln!("  dil çalıştır <dosya.dil>   programı çalıştırır");
    eprintln!("  dil denetle <dosya.dil>    çalıştırmadan denetler");
    eprintln!("  dil sürüm                  sürümü gösterir");
}

fn dosya_ile(argumanlar: &[String], komut: fn(&str) -> ExitCode) -> ExitCode {
    match argumanlar.get(1) {
        Some(yol) => match std::fs::read_to_string(yol) {
            Ok(kaynak) => komut(&kaynak),
            Err(hata) => {
                eprintln!("\"{}\" dosyası okunamadı: {}", yol, hata);
                ExitCode::from(2)
            }
        },
        None => {
            eprintln!("Bir .dil dosyası belirtmelisin. Örnek: dil çalıştır merhaba.dil");
            ExitCode::from(2)
        }
    }
}

/// Gerçek ekran + klavye IO'su: istem yazılır, cevap stdin'den okunur.
struct GercekIo;

impl dil::yorumlayici::GirdiCikti for GercekIo {
    fn yazdir(&mut self, satir: String) {
        println!("{}", satir);
    }
    fn sor(&mut self, istem: &str) -> Option<String> {
        use std::io::{BufRead, Write};
        println!("{}", istem);
        std::io::stdout().flush().ok();
        let mut cevap = String::new();
        match std::io::stdin().lock().read_line(&mut cevap) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(cevap.trim_end_matches(['\n', '\r']).to_string()),
        }
    }
}

fn calistir_komutu(kaynak: &str) -> ExitCode {
    let program = match dil::kaynagi_derle(kaynak) {
        Ok(program) => program,
        Err(tani) => {
            eprint!("{}", tani.raporla(kaynak));
            return ExitCode::FAILURE;
        }
    };
    match dil::yorumlayici::calistir_io(&program, &mut GercekIo) {
        Ok(()) => ExitCode::SUCCESS,
        Err(tani) => {
            eprint!("{}", tani.raporla(kaynak));
            ExitCode::FAILURE
        }
    }
}

fn denetle_komutu(kaynak: &str) -> ExitCode {
    match dil::kaynagi_denetle(kaynak) {
        Ok(()) => {
            println!("Denetim temiz: sözdizimi ve türler geçerli.");
            ExitCode::SUCCESS
        }
        Err(tani) => {
            eprint!("{}", tani.raporla(kaynak));
            ExitCode::FAILURE
        }
    }
}
