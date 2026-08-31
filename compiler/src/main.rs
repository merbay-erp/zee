//! `dil` — resmi CLI (master plan bölüm 15).
//!
//! Komutlar Türkçedir: çalıştır, denetle, sürüm.

use std::process::ExitCode;

fn main() -> ExitCode {
    let argumanlar: Vec<String> = std::env::args().skip(1).collect();

    match argumanlar.first().map(|s| s.as_str()) {
        Some("çalıştır") | Some("calistir") => dosya_ile(&argumanlar, calistir_komutu),
        Some("denetle") => dosya_ile(&argumanlar, denetle_komutu),
        Some("biçimle") | Some("bicimle") => bicimle_komutu(&argumanlar),
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
    eprintln!("  dil biçimle <dosya.dil>    dosyayı resmi biçime getirir");
    eprintln!("  dil sürüm                  sürümü gösterir");
}

fn bicimle_komutu(argumanlar: &[String]) -> ExitCode {
    let yol = match argumanlar.get(1) {
        Some(yol) => yol,
        None => {
            eprintln!("Bir .dil dosyası belirtmelisin. Örnek: dil biçimle merhaba.dil");
            return ExitCode::from(2);
        }
    };
    let kaynak = match std::fs::read_to_string(yol) {
        Ok(kaynak) => kaynak,
        Err(hata) => {
            eprintln!("\"{}\" dosyası okunamadı: {}", yol, hata);
            return ExitCode::from(2);
        }
    };
    match dil::bicimleyici::bicimle(&kaynak) {
        Ok(bicimli) if bicimli == kaynak => {
            println!("Zaten biçimli: {}", yol);
            ExitCode::SUCCESS
        }
        Ok(bicimli) => match std::fs::write(yol, &bicimli) {
            Ok(()) => {
                println!("Biçimlendi: {}", yol);
                ExitCode::SUCCESS
            }
            Err(hata) => {
                eprintln!("\"{}\" dosyasına yazılamadı: {}", yol, hata);
                ExitCode::from(2)
            }
        },
        Err(tani) => {
            eprint!("{}", tani.raporla(&kaynak));
            ExitCode::FAILURE
        }
    }
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
/// Rastgelelik: sistem saatiyle tohumlanan xorshift (bağımlılıksız).
struct GercekIo {
    tohum: u64,
}

impl GercekIo {
    fn yeni() -> GercekIo {
        let tohum = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|s| s.as_nanos() as u64)
            .unwrap_or(0x5EED)
            | 1;
        GercekIo { tohum }
    }
}

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
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        std::fs::read_to_string(yol).map_err(|hata| {
            format!("\"{}\" dosyası okunamadı: {}", yol, hata)
        })
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        use std::io::Write;
        let sonuc = std::fs::OpenOptions::new()
            .create(true)
            .append(ekleme)
            .write(true)
            .truncate(!ekleme)
            .open(yol)
            .and_then(|mut dosya| writeln!(dosya, "{}", satir));
        sonuc.map_err(|hata| format!("\"{}\" dosyasına yazılamadı: {}", yol, hata))
    }
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        // xorshift64*
        let mut x = self.tohum;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.tohum = x;
        let genislik = (ust - alt) as u64 + 1;
        alt + (x.wrapping_mul(0x2545F4914F6CDD1D) % genislik) as i64
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
    match dil::yorumlayici::calistir_io(&program, &mut GercekIo::yeni()) {
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
