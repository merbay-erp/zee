//! `dil` — resmi CLI (master plan bölüm 15).
//!
//! Komutlar Türkçedir: çalıştır, denetle, sürüm.

use std::process::ExitCode;

fn main() -> ExitCode {
    // Derinlik sınırına (C019, 500) kadar özyineleme her platformda doğal
    // yığını taşırmamalı; Windows ana iş parçacığı 1 MB olduğundan iş
    // 32 MB yığınlı bir iş parçacığında koşar (K-040).
    std::thread::Builder::new()
        .name("dil".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(govde)
        .expect("iş parçacığı açılamadı")
        .join()
        .expect("iş parçacığı düştü")
}

fn govde() -> ExitCode {
    let mut argumanlar: Vec<String> = std::env::args().skip(1).collect();
    // Çocuk modu (K-047): --güvenli bayrağı komuttan bağımsız yakalanır.
    let guvenli = argumanlar
        .iter()
        .any(|a| a == "--güvenli" || a == "--guvenli");
    let deneysel_web = argumanlar.iter().any(|a| a == "--deneysel-web");
    argumanlar.retain(|a| a != "--güvenli" && a != "--guvenli" && a != "--deneysel-web");

    match argumanlar.first().map(|s| s.as_str()) {
        Some("çalıştır") | Some("calistir") => {
            if guvenli {
                dosya_ile(&argumanlar, calistir_guvenli_komutu)
            } else if deneysel_web {
                dosya_ile(&argumanlar, calistir_deneysel_web_komutu)
            } else {
                dosya_ile(&argumanlar, calistir_komutu)
            }
        }
        Some("denetle") => denetle_yolu(&argumanlar),
        Some("biçimle") | Some("bicimle") => bicimle_komutu(&argumanlar),
        Some("dene") => dosya_ile(&argumanlar, dene_komutu),
        Some("çıkar") | Some("cikar") => cikar_komutu(&argumanlar),
        Some("ekle") => ekle_komutu(&argumanlar),
        Some("kilitle") => kilitle_komutu(&argumanlar),
        Some("paketler") => paketler_komutu(&argumanlar),
        Some("hata") => hata_komutu(&argumanlar),
        Some("belge") => belge_komutu(&argumanlar),
        Some("yeni") => yeni_komutu(&argumanlar),
        Some("sürüm") | Some("surum") => {
            println!(
                "dil {} — Türkçe programlama dili (bootstrap, Stage 0)",
                env!("CARGO_PKG_VERSION")
            );
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
    eprintln!("  dil çalıştır <dosya|proje> programı çalıştırır");
    eprintln!("  dil çalıştır --güvenli ... çocuk modu: ağ kapalı, dosyalar klasörle sınırlı");
    eprintln!("  dil çalıştır --deneysel-web ... localhost web prototipine açıkça izin verir");
    eprintln!("  dil denetle <dosya|proje>  çalıştırmadan denetler (--json: makine çıktısı)");
    eprintln!("  dil dene <dosya|proje>     test bloklarını koşar");
    eprintln!("  dil biçimle <dosya|proje>  dosyayı ya da bütün projeyi biçimler");
    eprintln!("  dil ekle <yerel-yol> [proje] yerel paketi doğrulayıp ekler ve kilitler");
    eprintln!("  dil çıkar <paket> [proje]    kullanılmayan doğrudan paketi kaldırır");
    eprintln!("  dil kilitle <proje>         yerel bağımlılıkları proje.kilit'e sabitler");
    eprintln!("  dil paketler [proje]        doğrudan/geçişli bağımlılık grafiğini gösterir");
    eprintln!("  dil hata <kod>             bir hata kodunu açıklar (örn. dil hata T001)");
    eprintln!(
        "  dil belge <birim>          bir birimin işlemlerini listeler (örn. dil belge matematik)"
    );
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
    eprintln!(
        "\"{}\" katalogda bulunamadı. Tam katalog: docs/hata-katalogu.md",
        kod
    );
    ExitCode::FAILURE
}

/// Birim belgesi (RFC-0014 §8.2): işlem başlıkları + test sayısı.
/// Önce gömülü kitaplığa, yoksa çalışma klasöründeki dosyaya bakar.
fn belge_komutu(argumanlar: &[String]) -> ExitCode {
    let Some(ad) = argumanlar.get(1) else {
        eprintln!(
            "Bir birim adı belirtmelisin. Gömülü birimler: {}",
            dil::gomulu_birim_adlari().join(", ")
        );
        return ExitCode::from(2);
    };
    let (kaynak, koken) = match dil::gomulu_birim(ad) {
        Some(kaynak) => (kaynak.to_string(), "gömülü kitaplık"),
        None => match std::fs::read_to_string(format!("{}.dil", ad)) {
            Ok(kaynak) => (kaynak, "bu klasör"),
            Err(_) => {
                eprintln!(
                    "\"{}\" birimi bulunamadı. Gömülü birimler: {}",
                    ad,
                    dil::gomulu_birim_adlari().join(", ")
                );
                return ExitCode::FAILURE;
            }
        },
    };
    println!("birim: {} ({})", ad, koken);
    print!("{}", dil::birim_ozeti(&kaynak));
    println!("Kaynağı oku: kitaplik/{}.dil — zee'yle yazılmıştır.", ad);
    ExitCode::SUCCESS
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
        "# {} — ilk programın!\n# Çalıştır: dil çalıştır .\n# Testleri koş: dil dene .\n\n\"Merhaba! Bu {} projesi.\" yaz\n\n\"Adın ne?\" diye sor\n\"Hoş geldin \" ile yanıt yaz\n\ntest \"karşılama hazır\"\n    selam \"Hoş geldin\" olsun\n    selam \"Hoş geldin\" e eşit olmalı\n",
        ad, ad
    );
    let bildirim = format!(
        "# zee proje bildirimi — bu dosya da geçerli zee sözdizimidir.\n\nproje \"{}\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"program.dil\" olsun\nyerel_bağımlılıklar boş liste olsun\n",
        ad
    );
    let beni_oku = format!(
        "# {}\n\nTürkçe programlama diliyle yazılmış bir proje. `proje.dil` giriş dosyasını, sürümü ve yerel bağımlılıkları tanımlar; `proje.kilit` bağımlılık kararını sabitler.\n\n```bash\ndil çalıştır .\n```\n\n```bash\ndil dene .\n```\n\nDenetim: `dil denetle .` · Bütün projeyi biçimle: `dil biçimle .` · Bağımlılıkları sabitle: `dil kilitle .` · Hata açıklama: `dil hata <kod>`\n",
        ad
    );
    let git_yoksay = ".zee-yazma-kilidi\n*.zee-gecici-*\n";
    let sonuc = std::fs::create_dir(klasor)
        .and_then(|_| std::fs::write(klasor.join("program.dil"), program))
        .and_then(|_| std::fs::write(klasor.join("proje.dil"), bildirim))
        .and_then(|_| std::fs::write(klasor.join("BENIOKU.md"), beni_oku))
        .and_then(|_| std::fs::write(klasor.join(".gitignore"), git_yoksay));
    match sonuc {
        Ok(()) => match dil::paket::ProjeGrafigi::cozumle(klasor)
            .map_err(|hata| hata.tani.mesaj)
            .and_then(|grafik| grafik.kilidi_yaz())
        {
            Ok(()) => {
                println!("Oluşturuldu: {}/", ad);
                println!("Başlamak için: dil çalıştır {}", ad);
                ExitCode::SUCCESS
            }
            Err(hata) => {
                eprintln!("Proje kilidi oluşturulamadı: {}", hata);
                ExitCode::FAILURE
            }
        },
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
    let girdi = match girdiyi_oku(std::path::Path::new(yol)) {
        Ok(girdi) => girdi,
        Err(hata) => {
            hata.yazdir(json);
            return hata.cikis_kodu();
        }
    };
    let mut yukleyici = |istek: dil::BirimIstegi<'_>| girdi.birim_yukle(istek);
    // Denetim TÜM tanıları toplar (RFC-0010 §3.1) — çalıştır ilk hatada durur.
    let tanilar =
        dil::kaynagi_tanilari_kokenlerle(&girdi.kaynak, Some(&girdi.koken), &mut yukleyici);
    if tanilar.is_empty() {
        if json {
            println!("{{\"durum\":\"temiz\",\"tanilar\":[]}}");
        } else {
            println!("Denetim temiz: sözdizimi ve türler geçerli.");
        }
        ExitCode::SUCCESS
    } else {
        if json {
            let govde: Vec<String> = tanilar.iter().map(|t| t.json()).collect();
            println!("{{\"durum\":\"hata\",\"tanilar\":[{}]}}", govde.join(","));
        } else {
            for tani in &tanilar {
                eprint!("{}", tani.raporla(&girdi.kaynak));
                eprintln!();
            }
            eprintln!("{} tanı bulundu.", tanilar.len());
        }
        ExitCode::FAILURE
    }
}

fn bicimle_komutu(argumanlar: &[String]) -> ExitCode {
    let yol = match argumanlar.get(1) {
        Some(yol) => std::path::Path::new(yol),
        None => {
            eprintln!("Bir .dil dosyası veya proje klasörü belirtmelisin. Örnek: dil biçimle .");
            return ExitCode::from(2);
        }
    };

    if yol.is_dir() {
        return projeyi_bicimle(yol);
    }
    bir_dosyayi_bicimle(yol)
}

fn kilitle_komutu(argumanlar: &[String]) -> ExitCode {
    let Some(yol) = argumanlar.get(1) else {
        eprintln!("Bir proje klasörü belirtmelisin. Örnek: dil kilitle .");
        return ExitCode::from(2);
    };
    let yol = std::path::Path::new(yol);
    if !yol.is_dir() {
        eprintln!("\"{}\" bir proje klasörü değil.", yol.display());
        return ExitCode::from(2);
    }
    let grafik = match dil::paket::ProjeGrafigi::cozumle(yol) {
        Ok(grafik) => grafik,
        Err(hata) => {
            GirdiHatasi::from(hata).yazdir(false);
            return ExitCode::FAILURE;
        }
    };
    match grafik.kilidi_yaz() {
        Ok(()) => {
            println!(
                "Kilitlendi: {} ({} yerel paket)",
                yol.join(dil::paket::KILIT_DOSYASI).display(),
                grafik.paket_sayisi()
            );
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("{}", hata);
            ExitCode::from(2)
        }
    }
}

fn ekle_komutu(argumanlar: &[String]) -> ExitCode {
    let Some(paket_yolu) = argumanlar.get(1) else {
        eprintln!("Bir yerel paket yolu belirtmelisin. Örnek: dil ekle ../hesap");
        return ExitCode::from(2);
    };
    if argumanlar.len() > 3 {
        eprintln!("Kullanım: dil ekle <yerel-yol> [proje]");
        return ExitCode::from(2);
    }
    let proje_yolu = std::path::Path::new(argumanlar.get(2).map_or(".", String::as_str));
    let proje_koku = match std::fs::canonicalize(proje_yolu) {
        Ok(yol) if yol.is_dir() => yol,
        Ok(_) => {
            eprintln!("\"{}\" bir proje klasörü değil.", proje_yolu.display());
            return ExitCode::from(2);
        }
        Err(hata) => {
            eprintln!(
                "\"{}\" proje klasörü çözülemedi: {}",
                proje_yolu.display(),
                hata
            );
            return ExitCode::from(2);
        }
    };
    let paket_koku = match std::fs::canonicalize(paket_yolu) {
        Ok(yol) if yol.is_dir() => yol,
        Ok(_) => {
            eprintln!("\"{}\" bir paket klasörü değil.", paket_yolu);
            return ExitCode::from(2);
        }
        Err(hata) => {
            eprintln!("\"{}\" yerel paketi çözülemedi: {}", paket_yolu, hata);
            return ExitCode::from(2);
        }
    };

    let bildirim_yolu = proje_koku.join("proje.dil");
    let eski_kaynak = match std::fs::read_to_string(&bildirim_yolu) {
        Ok(kaynak) => kaynak,
        Err(hata) => {
            eprintln!("\"{}\" okunamadı: {}", bildirim_yolu.display(), hata);
            return ExitCode::from(2);
        }
    };
    let bildirim = match dil::proje::bildirimi_oku(&eski_kaynak) {
        Ok(bildirim) => bildirim,
        Err(tani) => {
            GirdiHatasi::Tani {
                tani: Box::new(tani),
                kaynak: eski_kaynak,
                yol: bildirim_yolu,
            }
            .yazdir(false);
            return ExitCode::FAILURE;
        }
    };

    if proje_koku == paket_koku {
        let tani = dil::tani::Tani::yeni(
            "P007",
            "Bir proje kendisini yerel bağımlılık olarak ekleyemez.".into(),
            1,
            1,
            1,
        )
        .onerili("Paylaşılacak kodu ayrı bir zee projesine taşı.".into());
        eprint!("{}", tani.raporla(&eski_kaynak));
        return ExitCode::FAILURE;
    }

    let zaten_var = bildirim.yerel_bagimliliklar.iter().any(|yol| {
        std::fs::canonicalize(proje_koku.join(yol))
            .map(|kanonik| kanonik == paket_koku)
            .unwrap_or(false)
    });
    if zaten_var {
        let grafik = match dil::paket::ProjeGrafigi::cozumle(&proje_koku) {
            Ok(grafik) => grafik,
            Err(hata) => {
                GirdiHatasi::from(hata).yazdir(false);
                return ExitCode::FAILURE;
            }
        };
        return match grafik.kilidi_yaz() {
            Ok(()) => {
                println!(
                    "Paket zaten ekli; kilit yenilendi: {}",
                    paket_koku.display()
                );
                ExitCode::SUCCESS
            }
            Err(hata) => {
                eprintln!("{}", hata);
                ExitCode::from(2)
            }
        };
    }

    let goreli = dil::paket::goreli_yerel_yol(&proje_koku, &paket_koku);
    let mut yollar = bildirim.yerel_bagimliliklar;
    yollar.push(goreli.clone());
    let yeni_kaynak = match dil::proje::yerel_bagimliliklari_guncelle(&eski_kaynak, &yollar) {
        Ok(kaynak) => kaynak,
        Err(tani) => {
            eprint!("{}", tani.raporla(&eski_kaynak));
            return ExitCode::FAILURE;
        }
    };

    // Yeni grafik diske dokunmadan çözülür. Döngü, ad çakışması, bozuk paket
    // ya da güvensiz giriş varsa mevcut bildirim ve kilit aynen kalır.
    let grafik = match dil::paket::ProjeGrafigi::cozumle_bildirimle(&proje_koku, &yeni_kaynak) {
        Ok(grafik) => grafik,
        Err(hata) => {
            GirdiHatasi::from(hata).yazdir(false);
            return ExitCode::FAILURE;
        }
    };
    if let Err(hata) = proje_dosyalarini_guncelle(
        &proje_koku,
        &bildirim_yolu,
        &eski_kaynak,
        &yeni_kaynak,
        &grafik,
    ) {
        eprintln!("{}", hata);
        return ExitCode::from(2);
    }

    let paket_bildirimi = std::fs::read_to_string(paket_koku.join("proje.dil"))
        .ok()
        .and_then(|kaynak| dil::proje::bildirimi_oku(&kaynak).ok());
    match paket_bildirimi {
        Some(paket) => println!(
            "Eklendi: {} {} ({}) — proje.kilit güncellendi.",
            paket.ad, paket.surum, goreli
        ),
        None => println!("Eklendi: {} — proje.kilit güncellendi.", goreli),
    }
    ExitCode::SUCCESS
}

fn paketler_komutu(argumanlar: &[String]) -> ExitCode {
    if argumanlar.len() > 2 {
        eprintln!("Kullanım: dil paketler [proje]");
        return ExitCode::from(2);
    }
    let yol = std::path::Path::new(argumanlar.get(1).map_or(".", String::as_str));
    let grafik = match dil::paket::ProjeGrafigi::cozumle(yol) {
        Ok(grafik) => grafik,
        Err(hata) => {
            GirdiHatasi::from(hata).yazdir(false);
            return ExitCode::FAILURE;
        }
    };
    if let Err(hata) = grafik.kilidi_denetle() {
        GirdiHatasi::from(hata).yazdir(false);
        return ExitCode::FAILURE;
    }
    let bildirim = grafik.ana_bildirim();
    let paketler = grafik.paketler();
    println!(
        "proje: {} {} — {} yerel paket",
        bildirim.ad,
        bildirim.surum,
        paketler.len()
    );
    if paketler.is_empty() {
        println!("Bağımlılık yok.");
        return ExitCode::SUCCESS;
    }
    for paket in paketler {
        println!(
            "{}: {} {} · {} · sha256:{}",
            if paket.dogrudan {
                "doğrudan"
            } else {
                "geçişli"
            },
            paket.ad,
            paket.surum,
            paket.yol,
            paket.ozet
        );
    }
    ExitCode::SUCCESS
}

fn cikar_komutu(argumanlar: &[String]) -> ExitCode {
    let Some(paket_adi) = argumanlar.get(1) else {
        eprintln!("Kaldırılacak paket adını belirtmelisin. Örnek: dil çıkar hesap");
        return ExitCode::from(2);
    };
    if argumanlar.len() > 3 {
        eprintln!("Kullanım: dil çıkar <paket> [proje]");
        return ExitCode::from(2);
    }
    let proje_yolu = std::path::Path::new(argumanlar.get(2).map_or(".", String::as_str));
    let proje_koku = match std::fs::canonicalize(proje_yolu) {
        Ok(yol) if yol.is_dir() => yol,
        Ok(_) => {
            eprintln!("\"{}\" bir proje klasörü değil.", proje_yolu.display());
            return ExitCode::from(2);
        }
        Err(hata) => {
            eprintln!(
                "\"{}\" proje klasörü çözülemedi: {}",
                proje_yolu.display(),
                hata
            );
            return ExitCode::from(2);
        }
    };
    let bildirim_yolu = proje_koku.join("proje.dil");
    let eski_kaynak = match std::fs::read_to_string(&bildirim_yolu) {
        Ok(kaynak) => kaynak,
        Err(hata) => {
            eprintln!("\"{}\" okunamadı: {}", bildirim_yolu.display(), hata);
            return ExitCode::from(2);
        }
    };
    let bildirim = match dil::proje::bildirimi_oku(&eski_kaynak) {
        Ok(bildirim) => bildirim,
        Err(tani) => {
            eprint!("{}", tani.raporla(&eski_kaynak));
            return ExitCode::FAILURE;
        }
    };
    let grafik = match dil::paket::ProjeGrafigi::cozumle(&proje_koku) {
        Ok(grafik) => grafik,
        Err(hata) => {
            GirdiHatasi::from(hata).yazdir(false);
            return ExitCode::FAILURE;
        }
    };
    let Some(hedef_kok) = grafik
        .dogrudan_paket_koku(paket_adi)
        .map(std::path::Path::to_path_buf)
    else {
        eprintln!(
            "\"{}\" doğrudan yerel bağımlılıklar arasında yok.",
            paket_adi
        );
        return ExitCode::FAILURE;
    };
    if let Err(hata) = grafik.kaldirmayi_dogrula(paket_adi) {
        GirdiHatasi::from(hata).yazdir(false);
        return ExitCode::FAILURE;
    }

    let yollar = bildirim
        .yerel_bagimliliklar
        .into_iter()
        .filter(|yol| {
            std::fs::canonicalize(proje_koku.join(yol))
                .map(|kanonik| kanonik != hedef_kok)
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    let yeni_kaynak = match dil::proje::yerel_bagimliliklari_guncelle(&eski_kaynak, &yollar) {
        Ok(kaynak) => kaynak,
        Err(tani) => {
            eprint!("{}", tani.raporla(&eski_kaynak));
            return ExitCode::FAILURE;
        }
    };
    let yeni_grafik = match dil::paket::ProjeGrafigi::cozumle_bildirimle(&proje_koku, &yeni_kaynak)
    {
        Ok(grafik) => grafik,
        Err(hata) => {
            GirdiHatasi::from(hata).yazdir(false);
            return ExitCode::FAILURE;
        }
    };
    if let Err(hata) = proje_dosyalarini_guncelle(
        &proje_koku,
        &bildirim_yolu,
        &eski_kaynak,
        &yeni_kaynak,
        &yeni_grafik,
    ) {
        eprintln!("{}", hata);
        return ExitCode::from(2);
    }
    println!("Çıkarıldı: {} — proje.kilit güncellendi.", paket_adi);
    ExitCode::SUCCESS
}

fn proje_dosyalarini_guncelle(
    proje_koku: &std::path::Path,
    bildirim_yolu: &std::path::Path,
    eski_kaynak: &str,
    yeni_kaynak: &str,
    grafik: &dil::paket::ProjeGrafigi,
) -> Result<(), String> {
    let kilit_yolu = proje_koku.join(dil::paket::KILIT_DOSYASI);
    let eski_kilit = std::fs::read(&kilit_yolu).ok();
    dil::kalici_dosya::atomik_yaz(bildirim_yolu, yeni_kaynak.as_bytes())
        .map_err(|hata| format!("\"{}\" yazılamadı: {}", bildirim_yolu.display(), hata))?;
    if let Err(hata) = grafik.kilidi_yaz() {
        let bildirim_geri =
            dil::kalici_dosya::atomik_yaz(bildirim_yolu, eski_kaynak.as_bytes());
        let kilit_geri = match eski_kilit {
            Some(icerik) => dil::kalici_dosya::atomik_yaz(&kilit_yolu, &icerik),
            None if kilit_yolu.exists() => std::fs::remove_file(&kilit_yolu),
            None => Ok(()),
        };
        let geri_bildirimi = if bildirim_geri.is_err() || kilit_geri.is_err() {
            " Uyarı: önceki proje dosyaları bütünüyle geri yüklenemedi."
        } else {
            " Proje bildirimi ve önceki kilit geri yüklendi."
        };
        return Err(format!("Paket kilitlenemedi: {}{}", hata, geri_bildirimi));
    }
    Ok(())
}

fn bir_dosyayi_bicimle(yol: &std::path::Path) -> ExitCode {
    let kaynak = match std::fs::read_to_string(yol) {
        Ok(kaynak) => kaynak,
        Err(hata) => {
            eprintln!("\"{}\" dosyası okunamadı: {}", yol.display(), hata);
            return ExitCode::from(2);
        }
    };
    match dil::bicimleyici::bicimle(&kaynak) {
        Ok(bicimli) if bicimli == kaynak => {
            println!("Zaten biçimli: {}", yol.display());
            ExitCode::SUCCESS
        }
        Ok(bicimli) => match dil::kalici_dosya::atomik_yaz(yol, bicimli.as_bytes()) {
            Ok(()) => {
                println!("Biçimlendi: {}", yol.display());
                ExitCode::SUCCESS
            }
            Err(hata) => {
                eprintln!("\"{}\" dosyasına yazılamadı: {}", yol.display(), hata);
                ExitCode::from(2)
            }
        },
        Err(tani) => {
            eprint!("{}", tani.raporla(&kaynak));
            ExitCode::FAILURE
        }
    }
}

/// Projedeki bütün .dil kaynaklarını önce bellekte biçimler; tek bir kaynak
/// hatalıysa hiçbir dosyaya dokunmaz. Yazma sırası yol sırasıdır (K-077).
fn projeyi_bicimle(kok: &std::path::Path) -> ExitCode {
    if let Err(hata) = girdiyi_oku(kok) {
        hata.yazdir(false);
        return hata.cikis_kodu();
    }
    let mut yollar = Vec::new();
    if let Err(hata) = dil_dosyalarini_topla(kok, &mut yollar) {
        eprintln!("Proje kaynakları listelenemedi: {}", hata);
        return ExitCode::from(2);
    }
    yollar.sort();

    let mut hazir = Vec::new();
    for yol in &yollar {
        let kaynak = match std::fs::read_to_string(yol) {
            Ok(kaynak) => kaynak,
            Err(hata) => {
                eprintln!("\"{}\" dosyası okunamadı: {}", yol.display(), hata);
                return ExitCode::from(2);
            }
        };
        match dil::bicimleyici::bicimle(&kaynak) {
            Ok(bicimli) => hazir.push((yol, kaynak, bicimli)),
            Err(tani) => {
                eprintln!("Biçimlenemedi: {}", yol.display());
                eprint!("{}", tani.raporla(&kaynak));
                return ExitCode::FAILURE;
            }
        }
    }

    let mut degisen = 0usize;
    for (yol, kaynak, bicimli) in hazir {
        if kaynak != bicimli {
            if let Err(hata) = dil::kalici_dosya::atomik_yaz(yol, bicimli.as_bytes()) {
                eprintln!("\"{}\" dosyasına yazılamadı: {}", yol.display(), hata);
                return ExitCode::from(2);
            }
            degisen += 1;
        }
    }
    println!(
        "{} kaynak denetlendi; {} dosya biçimlendi.",
        yollar.len(),
        degisen
    );
    ExitCode::SUCCESS
}

fn dil_dosyalarini_topla(
    klasor: &std::path::Path,
    sonuc: &mut Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
    let mut girdiler: Vec<std::fs::DirEntry> =
        std::fs::read_dir(klasor)?.collect::<Result<_, _>>()?;
    girdiler.sort_by_key(|girdi| girdi.file_name());
    for girdi in girdiler {
        let tur = girdi.file_type()?;
        if tur.is_symlink() {
            continue;
        }
        let yol = girdi.path();
        if tur.is_dir() {
            let ad = girdi.file_name();
            let ad = ad.to_string_lossy();
            if ad.starts_with('.') || matches!(ad.as_ref(), "target" | "hedef") {
                continue;
            }
            dil_dosyalarini_topla(&yol, sonuc)?;
        } else if tur.is_file() && yol.extension().and_then(|uzanti| uzanti.to_str()) == Some("dil")
        {
            sonuc.push(yol);
        }
    }
    Ok(())
}

fn dosya_ile(argumanlar: &[String], komut: fn(&KaynakGirdisi) -> ExitCode) -> ExitCode {
    match argumanlar.get(1) {
        Some(yol) => match girdiyi_oku(std::path::Path::new(yol)) {
            Ok(girdi) => komut(&girdi),
            Err(hata) => {
                hata.yazdir(false);
                hata.cikis_kodu()
            }
        },
        None => {
            eprintln!("Bir .dil dosyası veya proje klasörü belirtmelisin. Örnek: dil çalıştır .");
            ExitCode::from(2)
        }
    }
}

struct KaynakGirdisi {
    kaynak: String,
    klasor: std::path::PathBuf,
    koken: String,
    proje: Option<dil::paket::ProjeGrafigi>,
}

impl KaynakGirdisi {
    fn birim_yukle(&self, istek: dil::BirimIstegi<'_>) -> Result<dil::YuklenenBirim, String> {
        if let Some(proje) = &self.proje {
            return proje.yukle(istek);
        }
        if istek.tur == dil::agac::KullanimTuru::Paket {
            return Err(
                "paket kullanımı için proje.dil taşıyan bir proje klasörü çalıştırılmalı".into(),
            );
        }
        if istek.ad.contains(['/', '\\', '.']) {
            return Err("birim adı yol içeremez".into());
        }
        let isteyen = istek
            .isteyen
            .map(std::path::Path::new)
            .unwrap_or_else(|| std::path::Path::new(&self.koken));
        let klasor = isteyen.parent().unwrap_or(&self.klasor);
        let yol = klasor.join(format!("{}.dil", istek.ad));
        match std::fs::canonicalize(&yol)
            .and_then(|kanonik| std::fs::read_to_string(&kanonik).map(|kaynak| (kanonik, kaynak)))
        {
            Ok((kanonik, kaynak)) => Ok(dil::YuklenenBirim {
                kaynak,
                koken: kanonik.to_string_lossy().into_owned(),
            }),
            Err(hata) => dil::gomulu_birim(istek.ad)
                .map(|kaynak| dil::YuklenenBirim {
                    kaynak: kaynak.to_string(),
                    koken: format!("gömülü:{}", istek.ad),
                })
                .ok_or_else(|| {
                    format!(
                        "{} ({}); gömülü kitaplıkta da yok (var olanlar: {})",
                        hata,
                        yol.display(),
                        dil::gomulu_birim_adlari().join(", ")
                    )
                }),
        }
    }
}

enum GirdiHatasi {
    Mesaj(String),
    Tani {
        tani: Box<dil::tani::Tani>,
        kaynak: String,
        yol: std::path::PathBuf,
    },
}

impl GirdiHatasi {
    fn cikis_kodu(&self) -> ExitCode {
        match self {
            GirdiHatasi::Mesaj(_) => ExitCode::from(2),
            GirdiHatasi::Tani { .. } => ExitCode::FAILURE,
        }
    }

    fn yazdir(&self, json: bool) {
        match self {
            GirdiHatasi::Mesaj(mesaj) => eprintln!("{}", mesaj),
            GirdiHatasi::Tani {
                tani,
                kaynak: _,
                yol: _,
            } if json => {
                println!("{{\"durum\":\"hata\",\"tanilar\":[{}]}}", tani.json());
            }
            GirdiHatasi::Tani { tani, kaynak, yol } => {
                eprintln!("Proje çözülemedi: {}", yol.display());
                eprint!("{}", tani.raporla(kaynak));
            }
        }
    }
}

impl From<dil::paket::ProjeYuklemeHatasi> for GirdiHatasi {
    fn from(hata: dil::paket::ProjeYuklemeHatasi) -> Self {
        GirdiHatasi::Tani {
            tani: hata.tani,
            kaynak: hata.kaynak,
            yol: hata.yol,
        }
    }
}

/// Bir .dil dosyasını ya da `proje.dil` taşıyan proje klasörünü girişe çözer.
/// Proje giriş yolu bildirimde doğrulandığı için proje kökünün dışına çıkamaz.
fn girdiyi_oku(yol: &std::path::Path) -> Result<KaynakGirdisi, GirdiHatasi> {
    if yol.is_dir() {
        let proje = dil::paket::ProjeGrafigi::cozumle(yol).map_err(GirdiHatasi::from)?;
        proje.kilidi_denetle().map_err(GirdiHatasi::from)?;
        let giris = proje.ana_giris();
        let klasor = proje
            .ana_giris_yolu()
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(yol)
            .to_path_buf();
        Ok(KaynakGirdisi {
            kaynak: giris.kaynak,
            klasor,
            koken: giris.koken,
            proje: Some(proje),
        })
    } else {
        let kanonik = std::fs::canonicalize(yol).map_err(|hata| {
            GirdiHatasi::Mesaj(format!("\"{}\" dosyası okunamadı: {}", yol.display(), hata))
        })?;
        let kaynak = std::fs::read_to_string(&kanonik).map_err(|hata| {
            GirdiHatasi::Mesaj(format!("\"{}\" dosyası okunamadı: {}", yol.display(), hata))
        })?;
        let klasor = kanonik
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();
        Ok(KaynakGirdisi {
            kaynak,
            klasor,
            koken: kanonik.to_string_lossy().into_owned(),
            proje: None,
        })
    }
}

/// Gerçek ekran + klavye IO'su: istem yazılır, cevap stdin'den okunur.
/// Rastgelelik: sistem saatiyle tohumlanan xorshift (bağımlılıksız).
struct GercekIo {
    /// Sonraki yanıtla gönderilecek Set-Cookie başlıkları (K-052).
    bekleyen_cerezler: Vec<(String, String)>,
    bekleyen_silinen_cerezler: Vec<String>,
    /// Göreli dosya yollarının kökü: giriş kaynağının klasörü (K-076).
    kok: std::path::PathBuf,
    tohum: u64,
    argumanlar: Vec<String>,
    baslangic: std::time::Instant,
    /// Üretim sözleşmesi tamamlanmamış localhost TCP yüzeyine açık opt-in.
    deneysel_web: bool,
    dinleyici: Option<std::net::TcpListener>,
    bekleyen_akis: Option<std::net::TcpStream>,
    bekleyen_head: bool,
    /// İç içe eylemler için dosya savepoint'leri: yol → çağrı başındaki içerik.
    eylem_yedekleri:
        Vec<std::collections::HashMap<std::path::PathBuf, EylemDosyaYedegi>>,
}

#[derive(Clone)]
struct EylemDosyaYedegi {
    onceki: Option<Vec<u8>>,
    /// Bu savepoint'in hedefte bıraktığını en son gördüğü bütün içerik.
    beklenen: Option<Vec<u8>>,
}

impl GercekIo {
    fn yeni(kok: &std::path::Path, deneysel_web: bool) -> GercekIo {
        let tohum = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|s| s.as_nanos() as u64)
            .unwrap_or(0x5EED)
            | 1;
        // Kaynak dosyası ya da proje klasöründen sonraki parçalar programa gider.
        // --güvenli kaynak önünde olsa da programa yanlışlıkla sızmaz (K-076).
        let argumanlar = program_argumanlari();
        GercekIo {
            bekleyen_cerezler: Vec::new(),
            bekleyen_silinen_cerezler: Vec::new(),
            kok: kok.to_path_buf(),
            tohum,
            argumanlar,
            baslangic: std::time::Instant::now(),
            deneysel_web,
            dinleyici: None,
            bekleyen_akis: None,
            bekleyen_head: false,
            eylem_yedekleri: Vec::new(),
        }
    }

    fn dosya_yolu(&self, yol: &str) -> std::path::PathBuf {
        let istenen = std::path::Path::new(yol);
        if istenen.is_absolute() {
            istenen.to_path_buf()
        } else {
            self.kok.join(istenen)
        }
    }
}

fn program_argumanlari() -> Vec<String> {
    let mut kaynak_goruldu = false;
    std::env::args()
        .skip(2)
        .filter_map(|arguman| {
            if arguman == "--güvenli" || arguman == "--guvenli" || arguman == "--deneysel-web" {
                None
            } else if kaynak_goruldu {
                Some(arguman)
            } else {
                kaynak_goruldu = true;
                None
            }
        })
        .collect()
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
        std::fs::read_to_string(self.dosya_yolu(yol))
            .map_err(|hata| format!("\"{}\" dosyası okunamadı: {}", yol, hata))
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        let gercek_yol = self.dosya_yolu(yol);
        if self
            .eylem_yedekleri
            .iter()
            .any(|yedek| !yedek.contains_key(&gercek_yol))
        {
            let onceki = match std::fs::read(&gercek_yol) {
                Ok(icerik) => Some(icerik),
                Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => None,
                Err(hata) => return Err(format!("transaction yedeği alınamadı: {}", hata)),
            };
            for yedek in &mut self.eylem_yedekleri {
                yedek.entry(gercek_yol.clone()).or_insert_with(|| {
                    EylemDosyaYedegi {
                        onceki: onceki.clone(),
                        beklenen: onceki.clone(),
                    }
                });
            }
        }
        dil::kalici_dosya::atomik_satir_yaz(&gercek_yol, satir, ekleme)
            .map_err(|hata| format!("\"{}\" dosyasına yazılamadı: {}", yol, hata))?;
        let sonraki = std::fs::read(&gercek_yol)
            .map_err(|hata| format!("transaction yazımı doğrulanamadı: {}", hata))?;
        for yedek in &mut self.eylem_yedekleri {
            if let Some(kayit) = yedek.get_mut(&gercek_yol) {
                kayit.beklenen = Some(sonraki.clone());
            }
        }
        Ok(())
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
        (
            yil,
            ay,
            gun,
            (gun_ici / 3600) as u32,
            ((gun_ici % 3600) / 60) as u32,
        )
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.argumanlar.clone()
    }
    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        use std::io::{Read, Write};
        use std::net::ToSocketAddrs;
        // v0: yalnız http:// (TLS elle yazılmaz — ADR-001; https Faz 5 kararı).
        let kalan = url.strip_prefix("http://").ok_or_else(|| {
            if url.starts_with("https://") {
                "v0 https (TLS) desteklemez; http:// kullan".to_string()
            } else {
                "adres http:// ile başlamalı".to_string()
            }
        })?;
        let (konak, yol) = match kalan.split_once('/') {
            Some((konak, yol)) => (konak.to_string(), format!("/{}", yol)),
            None => (kalan.to_string(), "/".to_string()),
        };
        let adres = if konak.contains(':') {
            konak.clone()
        } else {
            format!("{}:80", konak)
        };
        let baslangic = std::time::Instant::now();
        let kalan_sure = |toplam_ms: i64| -> Result<std::time::Duration, String> {
            let gecen = baslangic.elapsed().as_millis() as i64;
            let kalan = toplam_ms.saturating_sub(gecen);
            if kalan <= 0 {
                Err("son tarih doldu".into())
            } else {
                Ok(std::time::Duration::from_millis(kalan as u64))
            }
        };
        let mut akis = match zaman_asimi_ms {
            Some(kalan) => {
                let adresler: Vec<_> = adres
                    .to_socket_addrs()
                    .map_err(|e| e.to_string())?
                    .collect();
                if adresler.is_empty() {
                    return Err("adres çözülemedi".into());
                }
                let mut baglanti = None;
                let mut son_hata = None;
                for soket in adresler {
                    let sure = kalan_sure(kalan)?;
                    match std::net::TcpStream::connect_timeout(&soket, sure) {
                        Ok(akis) => {
                            baglanti = Some(akis);
                            break;
                        }
                        Err(hata) => son_hata = Some(hata.to_string()),
                    }
                }
                baglanti.ok_or_else(|| {
                    son_hata.unwrap_or_else(|| "sunucuya bağlanılamadı".to_string())
                })?
            }
            None => std::net::TcpStream::connect(&adres).map_err(|e| e.to_string())?,
        };
        if let Some(kalan) = zaman_asimi_ms {
            akis.set_write_timeout(Some(kalan_sure(kalan)?))
                .map_err(|e| e.to_string())?;
        }
        write!(
            akis,
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            yol, konak
        )
        .map_err(|e| e.to_string())?;
        if let Some(kalan) = zaman_asimi_ms {
            akis.set_read_timeout(Some(kalan_sure(kalan)?))
                .map_err(|e| e.to_string())?;
        }
        let mut ham = Vec::new();
        akis.read_to_end(&mut ham).map_err(|e| e.to_string())?;
        let metin = String::from_utf8_lossy(&ham);
        let durum: i64 = metin
            .lines()
            .next()
            .and_then(|satir| satir.split_whitespace().nth(1))
            .and_then(|kod| kod.parse().ok())
            .ok_or("HTTP yanıtı çözülemedi")?;
        let govde = metin
            .split_once("\r\n\r\n")
            .map(|(_, g)| g.to_string())
            .unwrap_or_default();
        Ok((durum, govde))
    }
    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String> {
        if !self.deneysel_web {
            return Err(
                "web yüzeyi üretim kullanımı için hazır değil; localhost prototipini bilinçli açmak için `dil çalıştır --deneysel-web <dosya|proje>` kullan"
                    .into(),
            );
        }
        let dinleyici =
            std::net::TcpListener::bind(("127.0.0.1", kapi as u16)).map_err(|e| e.to_string())?;
        println!("Sunucu dinliyor: http://127.0.0.1:{}", kapi);
        self.dinleyici = Some(dinleyici);
        Ok(())
    }
    fn istek_al(&mut self) -> Option<String> {
        use std::io::Read;
        let dinleyici = self.dinleyici.as_ref()?;
        loop {
            let (mut akis, _) = dinleyici.accept().ok()?;
            // 64 KiB gövde + en çok 16 KiB başlangıç satırı/başlık alanı.
            let mut tampon = [0u8; 80 * 1024];
            let mut okunan = akis.read(&mut tampon).ok()?;
            // Content-Length gövdesi ilk okumaya sığmadıysa tamamla (K-051).
            let baslik_sonu = tampon[..okunan]
                .windows(4)
                .position(|p| p == b"\r\n\r\n")
                .map(|i| i + 4);
            if let Some(govde_basi) = baslik_sonu {
                let basliklar = String::from_utf8_lossy(&tampon[..govde_basi]).to_lowercase();
                let beklenen: usize = basliklar
                    .lines()
                    .find_map(|s| s.strip_prefix("content-length:"))
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
                if beklenen > dil::yorumlayici::AZAMI_ISTEK_GOVDESI {
                    use std::io::Write;
                    let govde = b"istek govdesi 64 KiB sinirini asiyor";
                    let head = basliklar
                        .lines()
                        .next()
                        .and_then(|satir| satir.split_whitespace().next())
                        .is_some_and(|yontem| yontem == "head");
                    let _ = write!(
                        akis,
                        "HTTP/1.1 413 Payload Too Large\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        govde.len()
                    );
                    if !head {
                        let _ = akis.write_all(govde);
                    }
                    continue;
                }
                while okunan < tampon.len() && okunan - govde_basi < beklenen {
                    match akis.read(&mut tampon[okunan..]) {
                        Ok(0) | Err(_) => break,
                        Ok(ek) => okunan += ek,
                    }
                }
            }
            let istek = String::from_utf8_lossy(&tampon[..okunan]).to_string();
            let mut satirlar = istek.lines();
            let ilk = satirlar.next().unwrap_or("");
            let mut parcalar = ilk.split_whitespace();
            let (yontem, hedef) = (parcalar.next(), parcalar.next());
            if let (Some(yontem), Some(hedef)) = (yontem, hedef) {
                let govde = istek.split_once("\r\n\r\n").map(|(_, g)| g).unwrap_or("");
                // Cookie başlığı "çerez ..." satırı olarak taşınır (K-052).
                let cerez = istek
                    .lines()
                    .find_map(|s| {
                        let kucuk = s.to_lowercase();
                        kucuk
                            .strip_prefix("cookie:")
                            .map(|_| s[7..].trim().to_string())
                    })
                    .unwrap_or_default();
                self.bekleyen_akis = Some(akis);
                self.bekleyen_head = yontem.eq_ignore_ascii_case("HEAD");
                let cerez_satiri = if cerez.is_empty() {
                    String::new()
                } else {
                    format!("çerez {}\n", cerez)
                };
                return Some(format!("{} {}\n{}{}", yontem, hedef, cerez_satiri, govde));
            }
        }
    }
    fn cerez_yaz(&mut self, ad: &str, deger: &str) {
        self.bekleyen_cerezler
            .push((ad.to_string(), deger.to_string()));
    }
    fn cerez_sil(&mut self, ad: &str) {
        self.bekleyen_silinen_cerezler.push(ad.to_string());
    }
    fn eylem_baslat(&mut self) -> Result<(), String> {
        self.eylem_yedekleri.push(std::collections::HashMap::new());
        Ok(())
    }
    fn eylem_tamamla(&mut self) -> Result<(), String> {
        self.eylem_yedekleri
            .pop()
            .map(|_| ())
            .ok_or_else(|| "açık eylem transaction'ı yok".into())
    }
    fn eylem_geri_al(&mut self) -> Result<(), String> {
        let yedek = self
            .eylem_yedekleri
            .pop()
            .ok_or_else(|| "açık eylem transaction'ı yok".to_string())?;
        let mut girdiler = yedek.into_iter().collect::<Vec<_>>();
        girdiler.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (yol, kayit) in girdiler {
            dil::kalici_dosya::atomik_karsilastir_ve_geri_al(
                &yol,
                kayit.beklenen.as_deref(),
                kayit.onceki.as_deref(),
            )
            .map_err(|hata| format!("\"{}\" geri yüklenemedi: {}", yol.display(), hata))?;
            // İç savepoint geri alındıysa dış savepoint artık hedefte geri
            // yüklenen içeriği bekler; sonraki dış rollback bunu doğrular.
            for ust in &mut self.eylem_yedekleri {
                if let Some(ust_kayit) = ust.get_mut(&yol) {
                    ust_kayit.beklenen = kayit.onceki.clone();
                }
            }
        }
        Ok(())
    }
    fn yanit_gonder(&mut self, yanit: &str) {
        use std::io::Write;
        if let Some(mut akis) = self.bekleyen_akis.take() {
            let head = std::mem::take(&mut self.bekleyen_head);
            let govde = yanit.as_bytes();
            // Gövde işaretlemeyle başlıyorsa tarayıcıya HTML olarak sun
            // (K-050): zee ile web sayfası servis etmenin önünü açar.
            let tur = if yanit.trim_start().starts_with('<') {
                "text/html"
            } else {
                "text/plain"
            };
            let mut cerez_basliklari = String::new();
            for (ad, deger) in self.bekleyen_cerezler.drain(..) {
                cerez_basliklari.push_str(&format!(
                    "Set-Cookie: {}={}; Path=/; HttpOnly\r\n",
                    ad, deger
                ));
            }
            for ad in self.bekleyen_silinen_cerezler.drain(..) {
                cerez_basliklari.push_str(&format!("Set-Cookie: {}=; Path=/; Max-Age=0\r\n", ad));
            }
            let _ = write!(
                akis,
                "HTTP/1.1 200 OK\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\n{}Connection: close\r\n\r\n",
                tur,
                govde.len(),
                cerez_basliklari
            );
            if !head {
                let _ = akis.write_all(govde);
            }
        }
    }
    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        use std::io::Write;
        let aciklama = match durum {
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            413 => "Payload Too Large",
            431 => "Request Header Fields Too Large",
            504 => "Gateway Timeout",
            _ => "Error",
        };
        self.bekleyen_cerezler.clear();
        self.bekleyen_silinen_cerezler.clear();
        if let Some(mut akis) = self.bekleyen_akis.take() {
            let head = std::mem::take(&mut self.bekleyen_head);
            let govde = yanit.as_bytes();
            let _ = write!(
                akis,
                "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                durum,
                aciklama,
                govde.len()
            );
            if !head {
                let _ = akis.write_all(govde);
            }
        }
    }
    fn yonlendir_gonder(&mut self, adres: &str) {
        use std::io::Write;
        if let Some(mut akis) = self.bekleyen_akis.take() {
            self.bekleyen_head = false;
            let mut cerez_basliklari = String::new();
            for (ad, deger) in self.bekleyen_cerezler.drain(..) {
                cerez_basliklari.push_str(&format!(
                    "Set-Cookie: {}={}; Path=/; HttpOnly\r\n",
                    ad, deger
                ));
            }
            for ad in self.bekleyen_silinen_cerezler.drain(..) {
                cerez_basliklari.push_str(&format!("Set-Cookie: {}=; Path=/; Max-Age=0\r\n", ad));
            }
            let _ = write!(
                akis,
                "HTTP/1.1 303 See Other\r\nLocation: {}\r\nContent-Length: 0\r\n{}Connection: close\r\n\r\n",
                adres,
                cerez_basliklari
            );
        }
    }
    fn sensor_acik_mi(&mut self, _ad: &str) -> bool {
        // Donanım bağlı değil: simülatörde sensörler kapalı okunur (bölüm 17).
        false
    }
    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        println!("[ışık] {} {}", ad, if yansin { "yandı" } else { "söndü" });
    }
    fn bekle_ms(&mut self, milisaniye: i64) {
        std::thread::sleep(std::time::Duration::from_millis(milisaniye.max(0) as u64));
    }
    fn an_ms(&mut self) -> i64 {
        self.baslangic.elapsed().as_millis() as i64
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

fn calistir_komutu(girdi: &KaynakGirdisi) -> ExitCode {
    calistir_io_ile(girdi, &mut GercekIo::yeni(&girdi.klasor, false))
}

/// K-082: localhost web prototipi üretim korkuluğunu yalnız açık opt-in'le geçer.
fn calistir_deneysel_web_komutu(girdi: &KaynakGirdisi) -> ExitCode {
    eprintln!(
        "UYARI: deneysel web yüzeyi yalnız localhost eğitim/prototipi içindir; üretim güvenlik sözleşmesi değildir."
    );
    calistir_io_ile(girdi, &mut GercekIo::yeni(&girdi.klasor, true))
}

/// Çocuk modu (K-047): ağ/sunucu kapalı, dosyalar çalışma klasörüyle sınırlı.
fn calistir_guvenli_komutu(girdi: &KaynakGirdisi) -> ExitCode {
    calistir_io_ile(
        girdi,
        &mut dil::yorumlayici::GuvenliIo::yeni(GercekIo::yeni(&girdi.klasor, false)),
    )
}

fn calistir_io_ile(girdi: &KaynakGirdisi, io: &mut dyn dil::yorumlayici::GirdiCikti) -> ExitCode {
    let mut yukleyici = |istek: dil::BirimIstegi<'_>| girdi.birim_yukle(istek);
    let program =
        match dil::kaynagi_derle_kokenlerle(&girdi.kaynak, Some(&girdi.koken), &mut yukleyici) {
            Ok(program) => program,
            Err(tani) => {
                eprint!("{}", tani.raporla(&girdi.kaynak));
                return ExitCode::FAILURE;
            }
        };
    match dil::yorumlayici::calistir_io_kodla(&program, io) {
        // K-069: `programı N ile bitir` süreç çıkış kodu olur (0–255).
        Ok(kod) => ExitCode::from(kod.clamp(0, 255) as u8),
        Err(tani) => {
            eprint!("{}", tani.raporla(&girdi.kaynak));
            ExitCode::FAILURE
        }
    }
}

fn dene_komutu(girdi: &KaynakGirdisi) -> ExitCode {
    let mut yukleyici = |istek: dil::BirimIstegi<'_>| girdi.birim_yukle(istek);
    let sonuclar =
        match dil::kaynagi_derle_kokenlerle(&girdi.kaynak, Some(&girdi.koken), &mut yukleyici)
            .map(|program| dil::programi_dene(&program))
        {
            Ok(sonuclar) => sonuclar,
            Err(tani) => {
                eprint!("{}", tani.raporla(&girdi.kaynak));
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
                eprint!("{}", tani.raporla(&girdi.kaynak));
            }
        }
    }
    let kalan = sonuclar.len() - gecen;
    println!(
        "\n{} test: {} geçti, {} kaldı",
        sonuclar.len(),
        gecen,
        kalan
    );
    if kalan == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
