//! `dil` — resmi CLI (master plan bölüm 15).
//!
//! Komutlar Türkçedir: çalıştır, denetle, sürüm.

use std::process::ExitCode;

fn main() -> ExitCode {
    let argumanlar: Vec<String> = std::env::args().skip(1).collect();

    match argumanlar.first().map(|s| s.as_str()) {
        Some("çalıştır") | Some("calistir") => dosya_ile(&argumanlar, calistir_komutu),
        Some("denetle") => denetle_yolu(&argumanlar),
        Some("biçimle") | Some("bicimle") => bicimle_komutu(&argumanlar),
        Some("dene") => dosya_ile(&argumanlar, dene_komutu),
        Some("hata") => hata_komutu(&argumanlar),
        Some("yeni") => yeni_komutu(&argumanlar),
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
    eprintln!("  dil yeni <ad>              yeni proje klasörü oluşturur");
    eprintln!("  dil çalıştır <dosya.dil>   programı çalıştırır");
    eprintln!("  dil denetle <dosya.dil>    çalıştırmadan denetler (--json: makine çıktısı)");
    eprintln!("  dil dene <dosya.dil>       test bloklarını koşar");
    eprintln!("  dil biçimle <dosya.dil>    dosyayı resmi biçime getirir");
    eprintln!("  dil hata <kod>             bir hata kodunu açıklar (örn. dil hata T001)");
    eprintln!("  dil sürüm                  sürümü gösterir");
}

/// Hata kataloğu ikiliye gömülüdür: çevrimdışı sınıfta da `dil hata T001` çalışır.
const HATA_KATALOGU: &str = include_str!("../../docs/hata-katalogu.md");

fn hata_komutu(argumanlar: &[String]) -> ExitCode {
    let kod = match argumanlar.get(1) {
        Some(kod) => kod.to_uppercase(),
        None => {
            eprintln!("Bir hata kodu belirtmelisin. Örnek: dil hata T001");
            return ExitCode::from(2);
        }
    };
    let onek = format!("| {} ", kod);
    for satir in HATA_KATALOGU.lines() {
        if satir.starts_with(&onek) {
            let hucreler: Vec<&str> = satir.split('|').map(str::trim).collect();
            // Biçim: | kod | ne oldu | çözüm |
            if hucreler.len() >= 4 {
                println!("HATA {}", kod);
                println!("\nNe oldu:\n{}", hucreler[2]);
                if hucreler[3] != "—" && !hucreler[3].is_empty() {
                    println!("\nÇözüm:\n{}", hucreler[3]);
                }
                return ExitCode::SUCCESS;
            }
        }
    }
    eprintln!("\"{}\" katalogda bulunamadı. Tam katalog: docs/hata-katalogu.md", kod);
    ExitCode::FAILURE
}

fn yeni_komutu(argumanlar: &[String]) -> ExitCode {
    let ad = match argumanlar.get(1) {
        Some(ad) if !ad.is_empty() && !ad.contains(['/', '\\', '.']) => ad,
        _ => {
            eprintln!("Geçerli bir proje adı belirtmelisin. Örnek: dil yeni uzay-oyunum");
            return ExitCode::from(2);
        }
    };
    let klasor = std::path::Path::new(ad);
    if klasor.exists() {
        eprintln!("\"{}\" zaten var; üzerine yazılmadı.", ad);
        return ExitCode::FAILURE;
    }
    let program = format!(
        "# {} — ilk programın!\n# Çalıştır: dil çalıştır program.dil\n# Testleri koş: dil dene program.dil\n\n\"Merhaba! Bu {} projesi.\" yaz\n\n\"Adın ne?\" diye sor\n\"Hoş geldin \" ile yanıt yaz\n\ntest \"karşılama hazır\"\n    selam \"Hoş geldin\" olsun\n    selam \"Hoş geldin\" e eşit olmalı\n",
        ad, ad
    );
    let beni_oku = format!(
        "# {}\n\nTürkçe programlama diliyle yazılmış bir proje.\n\n```bash\ndil çalıştır program.dil\n```\n\n```bash\ndil dene program.dil\n```\n\nBiçim düzeltme: `dil biçimle program.dil` · Hata açıklama: `dil hata <kod>`\n",
        ad
    );
    let sonuc = std::fs::create_dir(klasor)
        .and_then(|_| std::fs::write(klasor.join("program.dil"), program))
        .and_then(|_| std::fs::write(klasor.join("BENIOKU.md"), beni_oku));
    match sonuc {
        Ok(()) => {
            println!("Oluşturuldu: {}/", ad);
            println!("Başlamak için: dil çalıştır {}/program.dil", ad);
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("Proje oluşturulamadı: {}", hata);
            ExitCode::FAILURE
        }
    }
}

fn denetle_yolu(argumanlar: &[String]) -> ExitCode {
    let json = argumanlar.iter().any(|a| a == "--json");
    let yol = match argumanlar.iter().skip(1).find(|a| !a.starts_with("--")) {
        Some(yol) => yol,
        None => {
            eprintln!("Bir .dil dosyası belirtmelisin. Örnek: dil denetle merhaba.dil");
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
    let klasor = std::path::Path::new(yol)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let mut yukleyici = birim_yukleyici(&klasor);
    match dil::kaynagi_derle_birimlerle(&kaynak, &mut yukleyici).map(|_| ()) {
        Ok(()) => {
            if json {
                println!("{{\"durum\":\"temiz\",\"tanilar\":[]}}");
            } else {
                println!("Denetim temiz: sözdizimi ve türler geçerli.");
            }
            ExitCode::SUCCESS
        }
        Err(tani) => {
            if json {
                println!("{{\"durum\":\"hata\",\"tanilar\":[{}]}}", tani.json());
            } else {
                eprint!("{}", tani.raporla(&kaynak));
            }
            ExitCode::FAILURE
        }
    }
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

fn dosya_ile(argumanlar: &[String], komut: fn(&str, &std::path::Path) -> ExitCode) -> ExitCode {
    match argumanlar.get(1) {
        Some(yol) => match std::fs::read_to_string(yol) {
            Ok(kaynak) => {
                let klasor = std::path::Path::new(yol)
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .to_path_buf();
                komut(&kaynak, &klasor)
            }
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

/// Ana dosyanın klasöründen birim yükler: `<klasör>/<ad>.dil` (RFC-0009 §4:
/// başka arama yolu yoktur). Ad, tanımlayıcı kurallarına uymalıdır.
fn birim_yukleyici(klasor: &std::path::Path) -> impl FnMut(&str) -> Result<String, String> + '_ {
    move |ad: &str| {
        if ad.contains(['/', '\\', '.']) {
            return Err("birim adı yol içeremez".into());
        }
        let yol = klasor.join(format!("{}.dil", ad));
        std::fs::read_to_string(&yol).map_err(|hata| format!("{} ({})", hata, yol.display()))
    }
}

/// Gerçek ekran + klavye IO'su: istem yazılır, cevap stdin'den okunur.
/// Rastgelelik: sistem saatiyle tohumlanan xorshift (bağımlılıksız).
struct GercekIo {
    tohum: u64,
    argumanlar: Vec<String>,
}

impl GercekIo {
    fn yeni() -> GercekIo {
        let tohum = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|s| s.as_nanos() as u64)
            .unwrap_or(0x5EED)
            | 1;
        // `dil çalıştır program.dil selam dünya` → programa ["selam", "dünya"] gider.
        let argumanlar = std::env::args().skip(3).collect();
        GercekIo { tohum, argumanlar }
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
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        // v0: UTC. Yerel saat dilimi desteği stdlib Zaman modülüyle gelecek.
        let saniye = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|s| s.as_secs() as i64)
            .unwrap_or(0);
        let gunler = saniye.div_euclid(86400);
        let gun_ici = saniye.rem_euclid(86400);
        let (yil, ay, gun) = dil::yorumlayici::gunlerden_tarih_utc(gunler);
        (yil, ay, gun, (gun_ici / 3600) as u32, ((gun_ici % 3600) / 60) as u32)
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.argumanlar.clone()
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

fn calistir_komutu(kaynak: &str, klasor: &std::path::Path) -> ExitCode {
    let mut yukleyici = birim_yukleyici(klasor);
    let program = match dil::kaynagi_derle_birimlerle(kaynak, &mut yukleyici) {
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

fn dene_komutu(kaynak: &str, klasor: &std::path::Path) -> ExitCode {
    let mut yukleyici = birim_yukleyici(klasor);
    let sonuclar = match dil::kaynagi_derle_birimlerle(kaynak, &mut yukleyici)
        .map(|program| dil::programi_dene(&program))
    {
        Ok(sonuclar) => sonuclar,
        Err(tani) => {
            eprint!("{}", tani.raporla(kaynak));
            return ExitCode::FAILURE;
        }
    };
    if sonuclar.is_empty() {
        println!("Bu dosyada test yok. Test eklemek için: test \"açıklama\"");
        return ExitCode::SUCCESS;
    }
    let mut gecen = 0usize;
    for sonuc in &sonuclar {
        match &sonuc.hata {
            None => {
                println!("✓ {}", sonuc.ad);
                gecen += 1;
            }
            Some(tani) => {
                println!("✗ {}", sonuc.ad);
                eprint!("{}", tani.raporla(kaynak));
            }
        }
    }
    let kalan = sonuclar.len() - gecen;
    println!("\n{} test: {} geçti, {} kaldı", sonuclar.len(), gecen, kalan);
    if kalan == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

