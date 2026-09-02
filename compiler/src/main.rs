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

//! `dil` — resmi CLI (master plan bölüm 15).
//!
//! Komutlar Türkçedir: çalıştır, denetle, sürüm.

use dil::yetkinlik::AgHedefi;
use std::process::ExitCode;

#[path = "cli/registry.rs"]
mod dil_registry;

const HTTP_ISTEK_OKUMA_SURESI: std::time::Duration = std::time::Duration::from_secs(
    dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
        .http()
        .istek_okuma_saniyesi(),
);
const WEB_BIND_IP: std::net::Ipv4Addr = std::net::Ipv4Addr::LOCALHOST;

#[derive(Clone, Debug, PartialEq, Eq)]
enum WebModu {
    Kapali,
    Deneysel,
    GuvenliProxy {
        origin: AgHedefi,
        worker_kapi: Option<u16>,
    },
}

fn web_modunu_ayikla(argumanlar: &mut Vec<String>) -> Result<WebModu, String> {
    let worker_yerleri = argumanlar
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--web-worker-port")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if worker_yerleri.len() > 1 {
        return Err("--web-worker-port birden çok kez verilemez".into());
    }
    let worker_kapi = if let Some(yer) = worker_yerleri.first().copied() {
        let deger = argumanlar
            .get(yer + 1)
            .ok_or_else(|| "--web-worker-port ardından 1..65535 kapısı ister".to_string())?
            .parse::<u16>()
            .map_err(|_| "--web-worker-port 1..65535 arasında olmalı".to_string())?;
        if deger == 0 {
            return Err("--web-worker-port 1..65535 arasında olmalı".into());
        }
        argumanlar.remove(yer + 1);
        argumanlar.remove(yer);
        Some(deger)
    } else {
        None
    };
    let deneysel_yerleri = argumanlar
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--deneysel-web")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let proxy_yerleri = argumanlar
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--web-proxy" || *a == "--güvenli-web-proxy")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if deneysel_yerleri.len() > 1 || proxy_yerleri.len() > 1 {
        return Err("web modu bayrağı birden çok kez verilemez".into());
    }
    let deneysel = deneysel_yerleri.first().copied();
    let proxy = proxy_yerleri.first().copied();
    if deneysel.is_some() && proxy.is_some() {
        return Err("--deneysel-web ile --web-proxy birlikte kullanılamaz".into());
    }
    if let Some(yer) = proxy {
        let origin = argumanlar
            .get(yer + 1)
            .ok_or_else(|| "--web-proxy ardından https:// origin'i ister".to_string())?
            .clone();
        argumanlar.remove(yer + 1);
        argumanlar.remove(yer);
        return AgHedefi::https_origininden(&origin).map(|origin| WebModu::GuvenliProxy {
            origin,
            worker_kapi,
        });
    }
    if worker_kapi.is_some() {
        return Err("--web-worker-port yalnız --web-proxy ile kullanılabilir".into());
    }
    if let Some(yer) = deneysel {
        argumanlar.remove(yer);
        Ok(WebModu::Deneysel)
    } else {
        Ok(WebModu::Kapali)
    }
}

fn main() -> ExitCode {
    // Derinlik sınırına (C019, 500) kadar özyineleme her platformda doğal
    // yığını taşırmamalı; Windows ana iş parçacığı 1 MB olduğundan iş
    // 32 MB yığınlı bir iş parçacığında koşar (K-040).
    let is_parcacigi = match std::thread::Builder::new()
        .name("dil".into())
        .stack_size(dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.calistirma_yigin_bayti())
        .spawn(govde)
    {
        Ok(is_parcacigi) => is_parcacigi,
        Err(hata) => {
            eprintln!("dil çalışma iş parçacığı açılamadı: {}", hata);
            return ExitCode::FAILURE;
        }
    };
    match is_parcacigi.join() {
        Ok(kod) => kod,
        Err(_) => {
            eprintln!("dil çalışma iş parçacığı beklenmedik biçimde durdu.");
            ExitCode::FAILURE
        }
    }
}

fn govde() -> ExitCode {
    let mut argumanlar: Vec<String> = std::env::args().skip(1).collect();
    // Çocuk modu (K-047): --güvenli bayrağı komuttan bağımsız yakalanır.
    let guvenli = argumanlar
        .iter()
        .any(|a| a == "--güvenli" || a == "--guvenli");
    let web_modu = match web_modunu_ayikla(&mut argumanlar) {
        Ok(modu) => modu,
        Err(hata) => {
            eprintln!("{}", hata);
            return ExitCode::from(2);
        }
    };
    argumanlar.retain(|a| a != "--güvenli" && a != "--guvenli");
    if web_modu != WebModu::Kapali
        && !matches!(
            argumanlar.first().map(String::as_str),
            Some("çalıştır" | "calistir")
        )
    {
        eprintln!("web modu bayrağı yalnız `dil çalıştır` ile kullanılabilir");
        return ExitCode::from(2);
    }
    if guvenli && web_modu != WebModu::Kapali {
        eprintln!("--güvenli çocuk modu web sunucusuyla birlikte kullanılamaz");
        return ExitCode::from(2);
    }

    match argumanlar.first().map(|s| s.as_str()) {
        Some("çalıştır") | Some("calistir") => {
            if guvenli {
                dosya_ile(&argumanlar, calistir_guvenli_komutu)
            } else {
                match web_modu {
                    WebModu::Kapali => dosya_ile(&argumanlar, calistir_komutu),
                    WebModu::Deneysel => dosya_ile(&argumanlar, calistir_deneysel_web_komutu),
                    WebModu::GuvenliProxy {
                        origin,
                        worker_kapi,
                    } => dosya_ile(&argumanlar, |girdi| {
                        calistir_guvenli_web_komutu(girdi, origin, worker_kapi)
                    }),
                }
            }
        }
        Some("denetle") => denetle_yolu(&argumanlar),
        Some("biçimle") | Some("bicimle") => bicimle_komutu(&argumanlar),
        Some("dene") => dosya_ile(&argumanlar, dene_komutu),
        Some("çıkar") | Some("cikar") => cikar_komutu(&argumanlar),
        Some("ekle") => ekle_komutu(&argumanlar),
        Some("kilitle") => dil_registry::kilitle_komutu(&argumanlar),
        Some("anahtar") => anahtar_komutu(&argumanlar),
        Some("paketle") => paketle_komutu(&argumanlar),
        Some("paketler") => dil_registry::paketler_komutu(&argumanlar),
        Some("hata") => hata_komutu(&argumanlar),
        Some("belge") => belge_komutu(&argumanlar),
        Some("morfoloji") => morfoloji_komutu(&argumanlar),
        Some("iz") => io_izi_komutu(&argumanlar),
        Some("parola-özeti") | Some("parola-ozeti") => parola_ozeti_komutu(&argumanlar),
        Some("yeni") => yeni_komutu(&argumanlar),
        Some("sürüm") | Some("surum") => {
            println!(
                "dil {} — Türkçe programlama dili (bootstrap, Stage 0) — morfoloji {} — IO {}",
                env!("CARGO_PKG_VERSION"),
                dil::morfoloji::MORFOLOJI_PROFILI,
                dil::yorumlayici::DETERMINISTIK_IO_PROFILI,
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
    eprintln!("  dil çalıştır --web-proxy https://host [--web-worker-port N] ... güvenli yerel TLS-proxy profili");
    eprintln!("  dil parola-özeti           parolayı gizli okuyup Argon2id PHC özeti üretir");
    eprintln!("  dil denetle <dosya|proje>  çalıştırmadan denetler (--json: makine çıktısı)");
    eprintln!("  dil dene <dosya|proje>     test bloklarını koşar");
    eprintln!("  dil biçimle <dosya|proje>  dosyayı ya da bütün projeyi biçimler");
    eprintln!("  dil ekle <yerel-yol|ad@X.Y.Z> [proje] paketi doğrulayıp ekler ve kilitler");
    eprintln!("      ilk uzak paket: --registry https://... --kök 1@sha256:<özet>");
    eprintln!("  dil çıkar <paket> [proje]    kullanılmayan doğrudan paketi kaldırır");
    eprintln!("  dil kilitle [proje] [--çevrimdışı] bütün bağımlılıkları sabitler");
    eprintln!("  dil anahtar üret <dosya>    0600 izinli Ed25519 yayıncı anahtarı üretir");
    eprintln!("  dil paketle [proje] --anahtar <dosya> [--çıktı <klasör>]");
    eprintln!("  dil paketler [proje] [--yenile] doğrudan/geçişli grafiği gösterir");
    eprintln!("  dil hata <kod>             bir hata kodunu açıklar (örn. dil hata T001)");
    eprintln!(
        "  dil belge <birim>          bir birimin işlemlerini listeler (örn. dil belge matematik)"
    );
    eprintln!("  dil iz kaydet <iz> <program> [argümanlar] deterministik IO izi kaydeder");
    eprintln!("  dil iz oynat <iz> <program>              IO izini dış dünyasız oynatır");
    eprintln!("  dil morfoloji [kelime|--uyumluluk] profil, çözüm veya immutable kaydı gösterir");
    eprintln!("  dil sürüm                  sürümü gösterir");
}

fn io_izi_komutu(argumanlar: &[String]) -> ExitCode {
    let (eylem, iz_yolu, kaynak_yolu) =
        match (argumanlar.get(1), argumanlar.get(2), argumanlar.get(3)) {
            (Some(eylem), Some(iz_yolu), Some(kaynak_yolu)) => {
                (eylem.as_str(), iz_yolu, kaynak_yolu)
            }
            _ => {
                eprintln!(
                    "Kullanım: dil iz kaydet <iz-dosyası> <dosya|proje> [program argümanları]"
                );
                eprintln!("          dil iz oynat <iz-dosyası> <dosya|proje>");
                return ExitCode::from(2);
            }
        };
    if !matches!(eylem, "kaydet" | "oynat") {
        eprintln!("İz eylemi `kaydet` veya `oynat` olmalı.");
        return ExitCode::from(2);
    }
    if eylem == "oynat" && argumanlar.len() > 4 {
        eprintln!("Replay program argümanlarını izden alır; `oynat` sonuna argüman eklenemez.");
        return ExitCode::from(2);
    }
    let girdi = match girdiyi_oku(std::path::Path::new(kaynak_yolu)) {
        Ok(girdi) => girdi,
        Err(hata) => {
            hata.yazdir(false);
            return hata.cikis_kodu();
        }
    };
    let iz_yolu = std::path::Path::new(iz_yolu);

    if eylem == "kaydet" {
        if std::fs::canonicalize(iz_yolu)
            .ok()
            .as_deref()
            .is_some_and(|iz| iz == std::path::Path::new(&girdi.koken))
        {
            eprintln!("IO izi kaynak program dosyasının üzerine yazılamaz.");
            return ExitCode::from(2);
        }
        eprintln!(
            "UYARI: IO izi program girdisi, dosya/ağ içeriği ve belirteç taşıyabilir; özel artefakt olarak sakla."
        );
        let program_argumanlari = argumanlar[4..].to_vec();
        let politika = girdi.politikasi();
        let taban = girdi.gercek_io(WebModu::Kapali, program_argumanlari, politika.clone());
        let korumali = dil::yorumlayici::PolitikaliIo::politikali(taban, politika.clone());
        let mut io = dil::yorumlayici::IzKaydedenIo::yeni(korumali);
        let cikis = calistir_io_ile(&girdi, &politika, &mut io);
        let olay_sayisi = io.olay_sayisi();
        let iz = match io.iz_metni() {
            Ok(iz) => iz,
            Err(hata) => {
                eprintln!("IO izi yazılamadı: {}", hata);
                return ExitCode::FAILURE;
            }
        };
        if let Err(hata) = dil::kalici_dosya::atomik_yaz(iz_yolu, iz.as_bytes()) {
            eprintln!(
                "IO izi \"{}\" dosyasına yazılamadı: {}",
                iz_yolu.display(),
                hata
            );
            return ExitCode::FAILURE;
        }
        eprintln!(
            "IO izi kaydedildi: {} olay · {}",
            olay_sayisi,
            iz_yolu.display()
        );
        return cikis;
    }

    let boyut = match std::fs::metadata(iz_yolu) {
        Ok(metadata) => metadata.len(),
        Err(hata) => {
            eprintln!("IO izi \"{}\" okunamadı: {}", iz_yolu.display(), hata);
            return ExitCode::FAILURE;
        }
    };
    if boyut > dil::yorumlayici::AZAMI_IO_IZ_BAYTI as u64 {
        eprintln!(
            "IO izi {} MiB sınırını aşıyor.",
            dil::yorumlayici::AZAMI_IO_IZ_BAYTI / 1024 / 1024
        );
        return ExitCode::FAILURE;
    }
    let iz = match std::fs::read_to_string(iz_yolu) {
        Ok(iz) => iz,
        Err(hata) => {
            eprintln!("IO izi \"{}\" okunamadı: {}", iz_yolu.display(), hata);
            return ExitCode::FAILURE;
        }
    };
    let mut io = match dil::yorumlayici::IzYenidenOynatici::yeni(&iz) {
        Ok(io) => io,
        Err(hata) => {
            eprintln!("IO izi geçersiz: {}", hata);
            return ExitCode::FAILURE;
        }
    };
    let cikis = calistir_io_ile(&girdi, &girdi.politikasi(), &mut io);
    let cikti = std::mem::take(&mut io.cikti);
    if let Err(hata) = io.bitir() {
        eprintln!("IO replay uyuşmazlığı: {}", hata);
        return ExitCode::FAILURE;
    }
    for satir in cikti {
        println!("{}", satir);
    }
    cikis
}

fn morfoloji_komutu(argumanlar: &[String]) -> ExitCode {
    if argumanlar.len() > 2 {
        eprintln!("Kullanım: dil morfoloji [kelime|--uyumluluk]");
        return ExitCode::from(2);
    }
    let Some(kelime) = argumanlar.get(1) else {
        print!("{}", dil::morfoloji::profil_dokumu());
        return ExitCode::SUCCESS;
    };
    if kelime == "--uyumluluk" {
        print!("{}", dil::morfoloji::profil_uyumluluk_kaydi());
        return ExitCode::SUCCESS;
    }
    let cozumler = dil::morfoloji::cozumleri_bul(kelime);
    println!(
        "{} — morfoloji {}",
        kelime,
        dil::morfoloji::MORFOLOJI_PROFILI
    );
    if cozumler.is_empty() {
        println!("Ekli kök çözümü yok.");
    } else {
        for cozum in cozumler {
            let ekler = cozum
                .ekler
                .iter()
                .map(|ek| ek.adi())
                .collect::<Vec<_>>()
                .join(" + ");
            println!("- {} + {}", cozum.kok, ekler);
        }
    }
    ExitCode::SUCCESS
}

fn parola_ozeti_komutu(argumanlar: &[String]) -> ExitCode {
    let parola = match argumanlar.get(1).map(String::as_str) {
        None => {
            let ilk = match rpassword::prompt_password("Parola: ") {
                Ok(parola) => parola,
                Err(hata) => {
                    eprintln!("Parola okunamadı: {}", hata);
                    return ExitCode::FAILURE;
                }
            };
            let ikinci = match rpassword::prompt_password("Parola (yeniden): ") {
                Ok(parola) => parola,
                Err(hata) => {
                    eprintln!("Parola doğrulaması okunamadı: {}", hata);
                    return ExitCode::FAILURE;
                }
            };
            if ilk != ikinci {
                eprintln!("Parolalar eşleşmiyor.");
                return ExitCode::from(2);
            }
            ilk
        }
        Some("--stdin") if argumanlar.len() == 2 => {
            use std::io::Read;
            let mut girdi = String::new();
            let azami = dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.dosya_okuma_bayti();
            if let Err(hata) = std::io::stdin()
                .take(azami.saturating_add(1) as u64)
                .read_to_string(&mut girdi)
            {
                eprintln!("Parola stdin'den okunamadı: {}", hata);
                return ExitCode::FAILURE;
            }
            if girdi.len() > azami {
                eprintln!(
                    "Parola stdin girdisi {} MiB sınırını aşıyor.",
                    azami / 1024 / 1024
                );
                return ExitCode::from(2);
            }
            girdi.trim_end_matches(['\r', '\n']).to_string()
        }
        _ => {
            eprintln!("Kullanım: dil parola-özeti [--stdin]");
            return ExitCode::from(2);
        }
    };
    if parola.is_empty() {
        eprintln!("Boş parola için özet üretilmez.");
        return ExitCode::from(2);
    }
    match dil::guvenlik::parola_ozeti_uret(&parola) {
        Ok(ozet) => {
            println!("{}", ozet);
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("{}", hata);
            ExitCode::FAILURE
        }
    }
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
        None => match dil::kaynak_sinirlari::kaynak_dosyasi_oku(std::path::Path::new(&format!(
            "{}.dil",
            ad
        ))) {
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
        "# zee proje bildirimi — bu dosya da geçerli zee sözdizimidir.\n\nproje \"{}\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"{}\" olsun\ngiriş \"program.dil\" olsun\nyetkinlikler boş liste olsun\nağ_hedefleri boş liste olsun\nyerel_bağımlılıklar boş liste olsun\n",
        ad,
        dil::morfoloji::MORFOLOJI_PROFILI,
    );
    let beni_oku = format!(
        "# {}\n\nTürkçe programlama diliyle yazılmış bir proje. `proje.dil` giriş dosyasını, sürümü, morfoloji profilini, dış dünya yetkinliklerini ve yerel bağımlılıkları tanımlar; `proje.kilit` bağımlılık kararını sabitler. Ağ erişimi gerekiyorsa hem `ağ` yetkinliğini hem tam şema+host+port `ağ_hedefleri` listesini açıkça bildir.\n\n```bash\ndil çalıştır .\n```\n\n```bash\ndil dene .\n```\n\nDenetim: `dil denetle .` · Bütün projeyi biçimle: `dil biçimle .` · Bağımlılıkları sabitle: `dil kilitle .` · Hata açıklama: `dil hata <kod>`\n",
        ad
    );
    let git_yoksay = ".zee/\n.zee-yazma-kilidi\n*.zee-gecici-*\n*.zee-anahtar\n*.zee-io-izi\n";
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
    let mut tanilar =
        dil::kaynagi_tanilari_kokenlerle(&girdi.kaynak, Some(&girdi.koken), &mut yukleyici);
    if tanilar.is_empty() {
        let mut yukleyici = |istek: dil::BirimIstegi<'_>| girdi.birim_yukle(istek);
        if let Ok(program) =
            dil::kaynagi_derle_kokenlerle(&girdi.kaynak, Some(&girdi.koken), &mut yukleyici)
        {
            if let Err(tani) =
                dil::cozumleyici::yetkinlikleri_denetle(&program, &girdi.politikasi())
            {
                tanilar.push(tani);
            }
        }
    }
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

fn anahtar_komutu(argumanlar: &[String]) -> ExitCode {
    if argumanlar.len() != 3 || !matches!(argumanlar[1].as_str(), "üret" | "uret") {
        eprintln!("Kullanım: dil anahtar üret <dosya>");
        return ExitCode::from(2);
    }
    let yol = std::path::Path::new(&argumanlar[2]);
    match dil::tedarik::anahtar_uret(yol) {
        Ok(kimlik) => {
            println!("Yayıncı anahtarı üretildi: {}", yol.display());
            println!("Açık anahtar kimliği: {}", kimlik);
            println!("Özel anahtarı paylaşma; güvenli bir yedeğini ayrı yerde sakla.");
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("P012: Anahtar üretilemedi: {}", hata);
            ExitCode::FAILURE
        }
    }
}

fn paketle_komutu(argumanlar: &[String]) -> ExitCode {
    let mut proje: Option<&str> = None;
    let mut anahtar: Option<&str> = None;
    let mut cikti: Option<&str> = None;
    let mut i = 1;
    while i < argumanlar.len() {
        match argumanlar[i].as_str() {
            "--anahtar" => {
                let Some(deger) = argumanlar.get(i + 1) else {
                    eprintln!("--anahtar ardından özel anahtar dosyası ister.");
                    return ExitCode::from(2);
                };
                if anahtar.replace(deger).is_some() {
                    eprintln!("--anahtar birden çok kez verilemez.");
                    return ExitCode::from(2);
                }
                i += 2;
            }
            "--çıktı" | "--cikti" => {
                let Some(deger) = argumanlar.get(i + 1) else {
                    eprintln!("--çıktı ardından klasör ister.");
                    return ExitCode::from(2);
                };
                if cikti.replace(deger).is_some() {
                    eprintln!("--çıktı birden çok kez verilemez.");
                    return ExitCode::from(2);
                }
                i += 2;
            }
            bilinmeyen if bilinmeyen.starts_with('-') => {
                eprintln!("Bilinmeyen paketle seçeneği: {}", bilinmeyen);
                return ExitCode::from(2);
            }
            yol => {
                if proje.replace(yol).is_some() {
                    eprintln!("Kullanım: dil paketle [proje] --anahtar <dosya> [--çıktı <klasör>]");
                    return ExitCode::from(2);
                }
                i += 1;
            }
        }
    }
    let Some(anahtar) = anahtar else {
        eprintln!("Paket imzası için --anahtar <dosya> gerekli.");
        return ExitCode::from(2);
    };
    let proje = std::path::Path::new(proje.unwrap_or("."));
    let varsayilan_cikti = proje.join("hedef").join("paket");
    let cikti = cikti
        .map(std::path::PathBuf::from)
        .unwrap_or(varsayilan_cikti);
    match dil::tedarik::paketle(proje, std::path::Path::new(anahtar), &cikti) {
        Ok(uretim) => {
            println!("Paketlendi: {}", uretim.paket.display());
            println!("SHA-256: {}", uretim.paket_ozeti);
            println!("SBOM: {}", uretim.sbom.display());
            println!("Provenance: {}", uretim.provenance.display());
            println!("İmzalı yayın: {}", uretim.yayin.display());
            println!("Yayıncı: {}", uretim.yayinci_anahtar_kimligi);
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("P012: Paketlenemedi: {}", hata);
            ExitCode::FAILURE
        }
    }
}

fn ekle_komutu(argumanlar: &[String]) -> ExitCode {
    let Some(paket_yolu) = argumanlar.get(1) else {
        eprintln!("Bir yerel yol veya exact paket belirtmelisin. Örnek: dil ekle hesap@1.2.3");
        return ExitCode::from(2);
    };
    if paket_yolu.contains('@') && !std::path::Path::new(paket_yolu).exists() {
        return dil_registry::uzak_ekle_komutu(argumanlar);
    }
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
    let eski_kaynak = match dil::kaynak_sinirlari::kaynak_dosyasi_oku(&bildirim_yolu) {
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

    let paket_bildirimi = dil::kaynak_sinirlari::kaynak_dosyasi_oku(&paket_koku.join("proje.dil"))
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

fn cikar_komutu(argumanlar: &[String]) -> ExitCode {
    let Some(paket_adi) = argumanlar.get(1) else {
        eprintln!("Kaldırılacak paket adını belirtmelisin. Örnek: dil çıkar hesap");
        return ExitCode::from(2);
    };
    if argumanlar.len() > 3 {
        eprintln!("Kullanım: dil çıkar <paket> [proje]");
        return ExitCode::from(2);
    }
    if let Some(kod) = dil_registry::uzak_cikar_komutu(argumanlar) {
        return kod;
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
    let eski_kaynak = match dil::kaynak_sinirlari::kaynak_dosyasi_oku(&bildirim_yolu) {
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
    _proje_koku: &std::path::Path,
    _bildirim_yolu: &std::path::Path,
    eski_kaynak: &str,
    yeni_kaynak: &str,
    grafik: &dil::paket::ProjeGrafigi,
) -> Result<(), String> {
    grafik.bildirim_ve_kilidi_yaz(eski_kaynak, yeni_kaynak)
}

fn bir_dosyayi_bicimle(yol: &std::path::Path) -> ExitCode {
    let kaynak = match dil::kaynak_sinirlari::kaynak_dosyasi_oku(yol) {
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
        let kaynak = match dil::kaynak_sinirlari::kaynak_dosyasi_oku(yol) {
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

fn dosya_ile<F>(argumanlar: &[String], komut: F) -> ExitCode
where
    F: FnOnce(&KaynakGirdisi) -> ExitCode,
{
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
    fn politikasi(&self) -> dil::yetkinlik::YetkinlikPolitikasi {
        self.proje
            .as_ref()
            .map(|proje| proje.ana_bildirim().yetkinlik_politikasi())
            .unwrap_or_else(dil::yetkinlik::YetkinlikPolitikasi::gelistirici)
    }

    fn dosya_siniri_koku(&self) -> &std::path::Path {
        self.proje
            .as_ref()
            .map(dil::paket::ProjeGrafigi::ana_kok)
            .unwrap_or(&self.klasor)
    }

    fn gercek_io(
        &self,
        web_modu: WebModu,
        argumanlar: Vec<String>,
        politika: dil::yetkinlik::YetkinlikPolitikasi,
    ) -> GercekIo {
        GercekIo::yeni_argumanlarla(
            &self.klasor,
            self.dosya_siniri_koku(),
            web_modu,
            argumanlar,
            politika,
        )
    }

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
        match std::fs::canonicalize(&yol).and_then(|kanonik| {
            dil::kaynak_sinirlari::kaynak_dosyasi_oku(&kanonik).map(|kaynak| (kanonik, kaynak))
        }) {
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
        let giris = proje.ana_giris().map_err(GirdiHatasi::Mesaj)?;
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
        let kaynak = dil::kaynak_sinirlari::kaynak_dosyasi_oku(&kanonik).map_err(|hata| {
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
/// Rastgelelik: sistem saatiyle tohumlanan sürümlü `zee-io-1` üreteci.
struct GercekIo {
    /// Sonraki yanıtla gönderilecek Set-Cookie başlıkları (K-052).
    bekleyen_cerezler: Vec<BekleyenCerez>,
    bekleyen_silinen_cerezler: Vec<String>,
    /// Göreli dosya yollarının kökü: giriş kaynağının klasörü (K-076).
    kok: std::path::PathBuf,
    rastgele: dil::yorumlayici::SurumluRastgele,
    argumanlar: Vec<String>,
    baslangic: std::time::Instant,
    /// Üretim sözleşmesi tamamlanmamış localhost TCP yüzeyine açık opt-in.
    web_modu: WebModu,
    dinleyici: Option<std::net::TcpListener>,
    bekleyen_akis: Option<std::net::TcpStream>,
    bekleyen_baglanti_izni: Option<dil::kaynak_sinirlari::BaglantiIzni>,
    bekleyen_head: bool,
    /// İç içe eylemler için dosya savepoint'leri: yol → çağrı başındaki içerik.
    eylem_yedekleri: Vec<std::collections::HashMap<std::path::PathBuf, EylemDosyaYedegi>>,
    web_guvenligi: dil::web_guvenligi::WebGuvenligi,
    web_istek_yedegi: Option<WebIstekYedegi>,
    bekleyen_web_yaniti: Option<WebYanitTaslagi>,
    politika: dil::yetkinlik::YetkinlikPolitikasi,
    dosya_siniri_koku: std::path::PathBuf,
}

#[derive(Clone)]
struct BekleyenCerez {
    ad: String,
    deger: String,
    azami_omur_saniye: Option<i64>,
}

struct WebIstekYedegi {
    bekleyen_cerezler: Vec<BekleyenCerez>,
    bekleyen_silinen_cerezler: Vec<String>,
}

enum WebYanitTaslagi {
    Govde(String),
    Durum(u16, String),
    Yonlendirme(String),
}

#[derive(Clone)]
struct EylemDosyaYedegi {
    onceki: Option<Vec<u8>>,
    /// Bu savepoint'in hedefte bıraktığını en son gördüğü bütün içerik.
    beklenen: Option<Vec<u8>>,
}

impl GercekIo {
    fn yeni_argumanlarla(
        kok: &std::path::Path,
        dosya_siniri_koku: &std::path::Path,
        web_modu: WebModu,
        argumanlar: Vec<String>,
        politika: dil::yetkinlik::YetkinlikPolitikasi,
    ) -> GercekIo {
        let tohum = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|s| s.as_nanos() as u64)
            .unwrap_or(0x5EED)
            | 1;
        let web_guvenligi = if matches!(&web_modu, WebModu::GuvenliProxy { .. }) {
            dil::web_guvenligi::WebGuvenligi::kalici(
                dosya_siniri_koku.join(".zee/web-durumu-v1.json"),
            )
        } else {
            dil::web_guvenligi::WebGuvenligi::yeni()
        };
        GercekIo {
            bekleyen_cerezler: Vec::new(),
            bekleyen_silinen_cerezler: Vec::new(),
            kok: kok.to_path_buf(),
            rastgele: dil::yorumlayici::SurumluRastgele::yeni(tohum),
            argumanlar,
            baslangic: std::time::Instant::now(),
            web_modu,
            dinleyici: None,
            bekleyen_akis: None,
            bekleyen_baglanti_izni: None,
            bekleyen_head: false,
            eylem_yedekleri: Vec::new(),
            web_guvenligi,
            web_istek_yedegi: None,
            bekleyen_web_yaniti: None,
            politika,
            dosya_siniri_koku: std::fs::canonicalize(dosya_siniri_koku)
                .unwrap_or_else(|_| dosya_siniri_koku.to_path_buf()),
        }
    }

    fn dosya_yolu(&self, yol: &str, yazma: bool) -> Result<std::path::PathBuf, String> {
        let istenen = std::path::Path::new(yol);
        let aday = if istenen.is_absolute() {
            istenen.to_path_buf()
        } else {
            self.kok.join(istenen)
        };
        if self.politika.dosya_siniri() == dil::yetkinlik::DosyaSiniri::HerYer {
            return Ok(aday);
        }
        if istenen.is_absolute()
            || istenen
                .components()
                .any(|bilesen| matches!(bilesen, std::path::Component::ParentDir))
        {
            return Err("proje dosya sınırı mutlak veya üst dizine çıkan yolu reddetti".into());
        }
        let denetlenecek = if yazma && !aday.exists() {
            aday.parent().unwrap_or(&self.kok)
        } else {
            aday.as_path()
        };
        let kanonik = std::fs::canonicalize(denetlenecek)
            .map_err(|hata| format!("dosya yolu güvenle çözülemedi: {}", hata))?;
        if !kanonik.starts_with(&self.dosya_siniri_koku) {
            return Err(
                "proje dosya sınırı sembolik bağ üzerinden kök dışına çıkışı reddetti".into(),
            );
        }
        Ok(aday)
    }

    fn guvenli_proxy_origin(&self) -> Option<&AgHedefi> {
        match &self.web_modu {
            WebModu::GuvenliProxy { origin, .. } => Some(origin),
            WebModu::Kapali | WebModu::Deneysel => None,
        }
    }

    fn web_worker_kapisi(&self) -> Option<u16> {
        match &self.web_modu {
            WebModu::GuvenliProxy { worker_kapi, .. } => *worker_kapi,
            WebModu::Kapali | WebModu::Deneysel => None,
        }
    }

    fn oturum_cerez_adi(&self) -> &'static str {
        if self.guvenli_proxy_origin().is_some() {
            "__Host-zee-oturum"
        } else {
            "zee-oturum"
        }
    }

    fn oturum_cerezini_yaz(&mut self, yeni: dil::web_guvenligi::YeniOturum) {
        self.bekleyen_cerezler.push(BekleyenCerez {
            ad: self.oturum_cerez_adi().to_string(),
            deger: yeni.belirtec,
            azami_omur_saniye: Some(yeni.azami_omur_saniye),
        });
    }

    fn guvenlik_basliklari(&self) -> String {
        guvenlik_basliklari(self.guvenli_proxy_origin().is_some())
    }

    fn cerez_basliklarini_al(&mut self) -> String {
        let secure = self.guvenli_proxy_origin().is_some();
        let mut sonuc = String::new();
        for cerez in self.bekleyen_cerezler.drain(..) {
            sonuc.push_str(&format!(
                "Set-Cookie: {}={}; Path=/; HttpOnly; SameSite=Lax{}{}\r\n",
                cerez.ad,
                cerez.deger,
                cerez
                    .azami_omur_saniye
                    .map(|omur| format!("; Max-Age={}", omur))
                    .unwrap_or_default(),
                if secure { "; Secure" } else { "" }
            ));
        }
        for ad in self.bekleyen_silinen_cerezler.drain(..) {
            sonuc.push_str(&format!(
                "Set-Cookie: {}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}\r\n",
                ad,
                if secure { "; Secure" } else { "" }
            ));
        }
        sonuc
    }

    fn web_yanit_taslagini_yaz(&mut self, taslak: WebYanitTaslagi) -> Result<(), String> {
        use std::io::Write;

        let mut akis = self
            .bekleyen_akis
            .take()
            .ok_or_else(|| "yanıt bekleyen web bağlantısı yok".to_string())?;
        let _baglanti_izni = self.bekleyen_baglanti_izni.take();
        let head = std::mem::take(&mut self.bekleyen_head);
        let guvenlik_basliklari = self.guvenlik_basliklari();
        let (baslik, govde) = match taslak {
            WebYanitTaslagi::Govde(govde) => {
                let tur = if govde.trim_start().starts_with('<') {
                    "text/html"
                } else {
                    "text/plain"
                };
                let cerezler = self.cerez_basliklarini_al();
                let baslik = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\n{}{}Connection: close\r\n\r\n",
                    tur,
                    govde.len(),
                    cerezler,
                    guvenlik_basliklari
                );
                (baslik, govde)
            }
            WebYanitTaslagi::Durum(durum, govde) => {
                self.bekleyen_cerezler.clear();
                self.bekleyen_silinen_cerezler.clear();
                let baslik = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n{}Connection: close\r\n\r\n",
                    durum,
                    http_durum_aciklamasi(durum),
                    govde.len(),
                    guvenlik_basliklari
                );
                (baslik, govde)
            }
            WebYanitTaslagi::Yonlendirme(adres) => {
                let cerezler = self.cerez_basliklarini_al();
                let baslik = format!(
                    "HTTP/1.1 303 See Other\r\nLocation: {}\r\nContent-Length: 0\r\n{}{}Connection: close\r\n\r\n",
                    adres, cerezler, guvenlik_basliklari
                );
                (baslik, String::new())
            }
        };
        let mut yanit = baslik.into_bytes();
        if !head {
            yanit.extend_from_slice(govde.as_bytes());
        }
        akis.write_all(&yanit)
            .map_err(|hata| format!("HTTP yanıtı yazılamadı: {}", hata))
    }
}

fn guvenlik_basliklari(https: bool) -> String {
    let mut sonuc = String::from(
        "Cache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'self'; style-src 'self' 'unsafe-inline'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; object-src 'none'\r\n",
    );
    if https {
        sonuc.push_str("Strict-Transport-Security: max-age=31536000\r\n");
    }
    sonuc
}

fn http_baslik_degerleri<'a>(istek: &'a str, ad: &str) -> Vec<&'a str> {
    istek
        .split("\r\n")
        .skip(1)
        .take_while(|satir| !satir.is_empty())
        .filter_map(|satir| satir.split_once(':'))
        .filter(|(gelen, _)| gelen.eq_ignore_ascii_case(ad))
        .map(|(_, deger)| deger.trim())
        .collect()
}

fn tek_http_basligi<'a>(istek: &'a str, ad: &str) -> Option<&'a str> {
    let degerler = http_baslik_degerleri(istek, ad);
    (degerler.len() == 1).then(|| degerler[0])
}

fn guvenilir_proxy_esi_mi(es: std::net::SocketAddr) -> bool {
    es.ip().is_loopback()
}

fn guvenli_proxy_istegini_denetle(
    istek: &str,
    origin: &AgHedefi,
    yontem: &str,
) -> Result<String, (u16, &'static str)> {
    let host = tek_http_basligi(istek, "Host").ok_or((400, "tek bir Host başlığı gerekli"))?;
    if AgHedefi::https_otoritesinden(host).as_ref() != Ok(origin) {
        return Err((400, "Host güvenli web origin'iyle eşleşmiyor"));
    }
    let forwarded_degerleri = http_baslik_degerleri(istek, "Forwarded");
    let forwarded = match forwarded_degerleri.as_slice() {
        [] => return Err((426, "güvenilir Forwarded başlığı gerekli")),
        [deger] => *deger,
        _ => return Err((400, "Forwarded başlığı yinelenemez")),
    };
    if forwarded.contains(',') {
        return Err((400, "Forwarded yalnız tek güvenilir proxy halkası taşımalı"));
    }
    let mut istemci = None;
    let mut proto = None;
    let mut forwarded_host = None;
    for alan in forwarded.split(';') {
        let (ad, deger) = alan
            .trim()
            .split_once('=')
            .ok_or((400, "Forwarded parametresi geçersiz"))?;
        let hedef = if ad.eq_ignore_ascii_case("for") {
            &mut istemci
        } else if ad.eq_ignore_ascii_case("proto") {
            &mut proto
        } else if ad.eq_ignore_ascii_case("host") {
            &mut forwarded_host
        } else {
            continue;
        };
        if hedef.replace(deger.trim()).is_some() {
            return Err((400, "Forwarded parametresi yinelenemez"));
        }
    }
    if !proto.is_some_and(|deger| deger.eq_ignore_ascii_case("https")) {
        return Err((426, "güvenli web profili HTTPS gerektiriyor"));
    }
    let forwarded_host = forwarded_host.map(|host| host.trim_matches('"'));
    if forwarded_host
        .and_then(|host| AgHedefi::https_otoritesinden(host).ok())
        .as_ref()
        != Some(origin)
    {
        return Err((400, "Forwarded host güvenli web origin'iyle eşleşmiyor"));
    }
    let istemci = istemci.ok_or((400, "Forwarded for istemci kimliği gerekli"))?;
    let istemci = istemci.trim_matches('"');
    let istemci = istemci
        .strip_prefix('[')
        .and_then(|deger| deger.strip_suffix(']'))
        .unwrap_or(istemci);
    let istemci = istemci
        .parse::<std::net::IpAddr>()
        .map_err(|_| (400, "Forwarded for kanonik IP adresi olmalı"))?;
    let guvenli_yontem = matches!(
        yontem.to_ascii_uppercase().as_str(),
        "GET" | "HEAD" | "OPTIONS"
    );
    if !guvenli_yontem {
        let gelen = tek_http_basligi(istek, "Origin")
            .ok_or((403, "durum değiştiren istek Origin başlığı istiyor"))?;
        if AgHedefi::https_origininden(gelen).as_ref() != Ok(origin) {
            return Err((403, "Origin güvenli web origin'iyle eşleşmiyor"));
        }
    }
    Ok(istemci.to_string())
}

fn ham_http_hatasi_gonder(
    akis: &mut std::net::TcpStream,
    durum: u16,
    mesaj: &str,
    head: bool,
    https: bool,
) {
    use std::io::Write;
    let aciklama = http_durum_aciklamasi(durum);
    let govde = mesaj.as_bytes();
    let basliklar = guvenlik_basliklari(https);
    let _ = write!(
        akis,
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n{}Connection: close\r\n\r\n",
        durum,
        aciklama,
        govde.len(),
        basliklar
    );
    if !head {
        let _ = akis.write_all(govde);
    }
}

fn http_durum_aciklamasi(durum: u16) -> &'static str {
    match durum {
        400 => "Bad Request",
        408 => "Request Timeout",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        426 => "Upgrade Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Error",
    }
}

fn son_tarihli_soket_oku(
    akis: &mut std::net::TcpStream,
    tampon: &mut [u8],
    son_tarih: std::time::Instant,
) -> std::io::Result<usize> {
    use std::io::Read;
    let kalan = son_tarih
        .checked_duration_since(std::time::Instant::now())
        .filter(|sure| !sure.is_zero())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::TimedOut, "son tarih doldu"))?;
    akis.set_read_timeout(Some(kalan))?;
    akis.read(tampon)
}

fn web_duvar_saati_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|sure| sure.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

fn program_argumanlari() -> Vec<String> {
    let mut kaynak_goruldu = false;
    let mut proxy_originini_atla = false;
    std::env::args()
        .skip(2)
        .filter_map(|arguman| {
            if proxy_originini_atla {
                proxy_originini_atla = false;
                return None;
            }
            if arguman == "--web-proxy"
                || arguman == "--güvenli-web-proxy"
                || arguman == "--web-worker-port"
            {
                proxy_originini_atla = true;
                None
            } else if arguman == "--güvenli"
                || arguman == "--guvenli"
                || arguman == "--deneysel-web"
            {
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
        self.politika
            .gerektir(dil::yetkinlik::Yetkinlik::DosyaOkuma)?;
        dil::kaynak_sinirlari::veri_dosyasi_oku(&self.dosya_yolu(yol, false)?)
            .map_err(|hata| format!("\"{}\" dosyası okunamadı: {}", yol, hata))
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        self.politika
            .gerektir(dil::yetkinlik::Yetkinlik::DosyaYazma)?;
        let gercek_yol = self.dosya_yolu(yol, true)?;
        if self
            .eylem_yedekleri
            .iter()
            .any(|yedek| !yedek.contains_key(&gercek_yol))
        {
            let onceki = match dil::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(&gercek_yol) {
                Ok(icerik) => Some(icerik),
                Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => None,
                Err(hata) => return Err(format!("transaction yedeği alınamadı: {}", hata)),
            };
            for yedek in &mut self.eylem_yedekleri {
                yedek
                    .entry(gercek_yol.clone())
                    .or_insert_with(|| EylemDosyaYedegi {
                        onceki: onceki.clone(),
                        beklenen: onceki.clone(),
                    });
            }
        }
        dil::kalici_dosya::atomik_satir_yaz(&gercek_yol, satir, ekleme)
            .map_err(|hata| format!("\"{}\" dosyasına yazılamadı: {}", yol, hata))?;
        let sonraki = dil::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(&gercek_yol)
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
        dil::ag_istemcisi::getir(url, zaman_asimi_ms, &self.politika)
    }
    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String> {
        self.politika
            .gerektir(dil::yetkinlik::Yetkinlik::AgSunucusu)?;
        if self.web_modu == WebModu::Kapali {
            return Err("web yüzeyi kapalı; prototip için `--deneysel-web`, üretim için `--web-proxy https://host` kullan".into());
        }
        let gercek_kapi = self.web_worker_kapisi().map(i64::from).unwrap_or(kapi);
        let dinleyici = std::net::TcpListener::bind((WEB_BIND_IP, gercek_kapi as u16))
            .map_err(|e| e.to_string())?;
        match &self.web_modu {
            WebModu::Deneysel => println!("Sunucu dinliyor: http://127.0.0.1:{}", kapi),
            WebModu::GuvenliProxy { origin, .. } => println!(
                "Sunucu dinliyor: {} (yerel proxy hedefi http://127.0.0.1:{})",
                origin.yazimi(),
                gercek_kapi
            ),
            WebModu::Kapali => {
                return Err("web yüzeyi kapalıyken sunucu kurulamaz".into());
            }
        }
        self.dinleyici = Some(dinleyici);
        Ok(())
    }
    fn istek_al(&mut self) -> Option<String> {
        let dinleyici = self.dinleyici.as_ref()?;
        'istekler: loop {
            let (mut akis, es) = dinleyici.accept().ok()?;
            let https = self.guvenli_proxy_origin().is_some();
            if https && !guvenilir_proxy_esi_mi(es) {
                ham_http_hatasi_gonder(
                    &mut akis,
                    403,
                    "güvenilir proxy bağlantısı loopback üzerinden gelmeli",
                    false,
                    true,
                );
                continue;
            }
            let baglanti_izni = match dil::kaynak_sinirlari::baglanti_izni_al() {
                Ok(izin) => izin,
                Err(_) => {
                    ham_http_hatasi_gonder(
                        &mut akis,
                        503,
                        "sunucu eşzamanlı bağlantı sınırına ulaştı",
                        false,
                        https,
                    );
                    continue;
                }
            };
            let son_tarih = std::time::Instant::now() + HTTP_ISTEK_OKUMA_SURESI;
            if akis
                .set_write_timeout(Some(HTTP_ISTEK_OKUMA_SURESI))
                .is_err()
            {
                continue;
            }
            let http_siniri = dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.http();
            let baslik_siniri = http_siniri.istek_baslik_bayti();
            let mut tampon =
                Vec::with_capacity(baslik_siniri.saturating_add(http_siniri.istek_govde_bayti()));
            let govde_basi = loop {
                if let Some(yer) = tampon.windows(4).position(|p| p == b"\r\n\r\n") {
                    break yer + 4;
                }
                if tampon.len() >= baslik_siniri {
                    ham_http_hatasi_gonder(
                        &mut akis,
                        431,
                        &format!(
                            "istek başlıkları {} KiB sınırını aşıyor",
                            baslik_siniri / 1024
                        ),
                        false,
                        https,
                    );
                    continue 'istekler;
                }
                let mut parca = [0u8; 4096];
                let sinir = (baslik_siniri - tampon.len()).min(parca.len());
                match son_tarihli_soket_oku(&mut akis, &mut parca[..sinir], son_tarih) {
                    Ok(0) => continue 'istekler,
                    Ok(okunan) => tampon.extend_from_slice(&parca[..okunan]),
                    Err(hata)
                        if matches!(
                            hata.kind(),
                            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                        ) =>
                    {
                        ham_http_hatasi_gonder(
                            &mut akis,
                            408,
                            &format!(
                                "istek {} saniyede tamamlanmadı",
                                HTTP_ISTEK_OKUMA_SURESI.as_secs()
                            ),
                            false,
                            https,
                        );
                        continue 'istekler;
                    }
                    Err(_) => continue 'istekler,
                }
            };
            let Ok(baslik_metni) = std::str::from_utf8(&tampon[..govde_basi]) else {
                ham_http_hatasi_gonder(&mut akis, 400, "HTTP başlıkları UTF-8 değil", false, https);
                continue;
            };
            if !http_baslik_degerleri(baslik_metni, "Transfer-Encoding").is_empty() {
                ham_http_hatasi_gonder(
                    &mut akis,
                    400,
                    "Transfer-Encoding desteklenmiyor",
                    false,
                    https,
                );
                continue;
            }
            let uzunluklar = http_baslik_degerleri(baslik_metni, "Content-Length");
            if uzunluklar.len() > 1 {
                ham_http_hatasi_gonder(
                    &mut akis,
                    400,
                    "birden çok Content-Length başlığı reddedildi",
                    false,
                    https,
                );
                continue;
            }
            let beklenen = match uzunluklar.first() {
                Some(deger) => match deger.parse::<usize>() {
                    Ok(uzunluk) => uzunluk,
                    Err(_) => {
                        ham_http_hatasi_gonder(
                            &mut akis,
                            400,
                            "Content-Length geçersiz",
                            false,
                            https,
                        );
                        continue;
                    }
                },
                None => 0,
            };
            if beklenen > dil::yorumlayici::AZAMI_ISTEK_GOVDESI {
                let head = baslik_metni
                    .split_whitespace()
                    .next()
                    .is_some_and(|yontem| yontem.eq_ignore_ascii_case("HEAD"));
                ham_http_hatasi_gonder(
                    &mut akis,
                    413,
                    &format!(
                        "istek gövdesi {} KiB sınırını aşıyor",
                        http_siniri.istek_govde_bayti() / 1024
                    ),
                    head,
                    https,
                );
                continue;
            }
            let toplam = govde_basi + beklenen;
            while tampon.len() < toplam {
                let onceki = tampon.len();
                tampon.resize(toplam, 0);
                match son_tarihli_soket_oku(&mut akis, &mut tampon[onceki..toplam], son_tarih) {
                    Ok(0) => {
                        ham_http_hatasi_gonder(
                            &mut akis,
                            400,
                            "istek gövdesi Content-Length'ten kısa",
                            false,
                            https,
                        );
                        continue 'istekler;
                    }
                    Ok(okunan) => tampon.truncate(onceki + okunan),
                    Err(hata)
                        if matches!(
                            hata.kind(),
                            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                        ) =>
                    {
                        ham_http_hatasi_gonder(
                            &mut akis,
                            408,
                            &format!(
                                "istek {} saniyede tamamlanmadı",
                                HTTP_ISTEK_OKUMA_SURESI.as_secs()
                            ),
                            false,
                            https,
                        );
                        continue 'istekler;
                    }
                    Err(_) => {
                        ham_http_hatasi_gonder(
                            &mut akis,
                            400,
                            "istek gövdesi Content-Length'ten kısa",
                            false,
                            https,
                        );
                        continue 'istekler;
                    }
                }
            }
            let istek = String::from_utf8_lossy(&tampon[..toplam]).to_string();
            let mut satirlar = istek.lines();
            let ilk = satirlar.next().unwrap_or("");
            let mut parcalar = ilk.split_whitespace();
            let (yontem, hedef, surum) = (parcalar.next(), parcalar.next(), parcalar.next());
            if let (Some(yontem), Some(hedef), Some(surum)) = (yontem, hedef, surum) {
                if parcalar.next().is_some() || !matches!(surum, "HTTP/1.0" | "HTTP/1.1") {
                    ham_http_hatasi_gonder(
                        &mut akis,
                        400,
                        "HTTP istek satırı geçersiz",
                        false,
                        https,
                    );
                    continue;
                }
                let istemci_kimligi = if let Some(origin) = self.guvenli_proxy_origin() {
                    match guvenli_proxy_istegini_denetle(&istek, origin, yontem) {
                        Ok(kimlik) => kimlik,
                        Err((durum, mesaj)) => {
                            ham_http_hatasi_gonder(
                                &mut akis,
                                durum,
                                mesaj,
                                yontem.eq_ignore_ascii_case("HEAD"),
                                true,
                            );
                            continue;
                        }
                    }
                } else {
                    es.ip().to_string()
                };
                let govde = istek.split_once("\r\n\r\n").map(|(_, g)| g).unwrap_or("");
                // Cookie başlığı "çerez ..." satırı olarak taşınır (K-052).
                let cerez_degerleri = http_baslik_degerleri(&istek, "Cookie");
                if cerez_degerleri.len() > 1 {
                    ham_http_hatasi_gonder(
                        &mut akis,
                        400,
                        "birden çok Cookie başlığı reddedildi",
                        yontem.eq_ignore_ascii_case("HEAD"),
                        https,
                    );
                    continue;
                }
                let cerez = cerez_degerleri.first().copied().unwrap_or_default();
                let oturum = cerez
                    .split(';')
                    .filter_map(|parca| parca.trim().split_once('='))
                    .find_map(|(ad, deger)| {
                        (ad == self.oturum_cerez_adi()).then_some(deger.trim())
                    });
                let an = web_duvar_saati_ms();
                self.web_istek_yedegi = Some(WebIstekYedegi {
                    bekleyen_cerezler: self.bekleyen_cerezler.clone(),
                    bekleyen_silinen_cerezler: self.bekleyen_silinen_cerezler.clone(),
                });
                self.bekleyen_web_yaniti = None;
                if let Err(hata) = self.web_guvenligi.istegi_baslat_kimlikle(
                    oturum,
                    &istemci_kimligi,
                    an,
                ) {
                    ham_http_hatasi_gonder(
                        &mut akis,
                        503,
                        &format!("web güvenlik deposuna erişilemedi: {hata}"),
                        yontem.eq_ignore_ascii_case("HEAD"),
                        https,
                    );
                    self.web_istek_yedegi = None;
                    continue;
                }
                let kapsam = format!(
                    "{} {}",
                    yontem.to_ascii_uppercase(),
                    hedef.split('?').next().unwrap_or(hedef)
                );
                match self.web_guvenligi.rate_limit_artir(
                    dil::web_guvenligi::RateLimitTuru::Ucnokta,
                    &kapsam,
                    an,
                ) {
                    Ok(karar) if karar.izinli => {}
                    Ok(_) => {
                        ham_http_hatasi_gonder(
                            &mut akis,
                            429,
                            "uç nokta oran sınırı aşıldı",
                            yontem.eq_ignore_ascii_case("HEAD"),
                            https,
                        );
                        self.web_guvenligi.istegi_geri_al();
                        self.web_istek_yedegi = None;
                        continue;
                    }
                    Err(hata) => {
                        ham_http_hatasi_gonder(
                            &mut akis,
                            503,
                            &format!("oran sınırı deposuna erişilemedi: {hata}"),
                            yontem.eq_ignore_ascii_case("HEAD"),
                            https,
                        );
                        self.web_guvenligi.istegi_geri_al();
                        self.web_istek_yedegi = None;
                        continue;
                    }
                }
                self.bekleyen_akis = Some(akis);
                self.bekleyen_baglanti_izni = Some(baglanti_izni);
                self.bekleyen_head = yontem.eq_ignore_ascii_case("HEAD");
                let cerez_satiri = if cerez.is_empty() {
                    String::new()
                } else {
                    format!("çerez {}\n", cerez)
                };
                return Some(format!("{} {}\n{}{}", yontem, hedef, cerez_satiri, govde));
            }
            ham_http_hatasi_gonder(&mut akis, 400, "HTTP istek satırı eksik", false, https);
        }
    }
    fn istek_islemini_tamamla(&mut self) -> Result<(), String> {
        let Some(yedek) = self.web_istek_yedegi.take() else {
            return Ok(());
        };
        let Some(taslak) = self.bekleyen_web_yaniti.take() else {
            self.web_guvenligi.istegi_geri_al();
            self.bekleyen_cerezler = yedek.bekleyen_cerezler;
            self.bekleyen_silinen_cerezler = yedek.bekleyen_silinen_cerezler;
            self.bekleyen_akis = None;
            self.bekleyen_baglanti_izni = None;
            self.bekleyen_head = false;
            return Ok(());
        };
        let commit = match self.web_guvenligi.istegi_tamamla(web_duvar_saati_ms()) {
            Ok(commit) => commit,
            Err(hata) => {
                eprintln!("Web güvenlik durumu commit edilemedi: {hata}");
                self.web_guvenligi.istegi_geri_al();
                self.bekleyen_cerezler = yedek.bekleyen_cerezler;
                self.bekleyen_silinen_cerezler = yedek.bekleyen_silinen_cerezler;
                return self.web_yanit_taslagini_yaz(WebYanitTaslagi::Durum(
                    503,
                    "web güvenlik durumu güvenle kaydedilemedi".into(),
                ));
            }
        };
        if let Err(hata) = self.web_yanit_taslagini_yaz(taslak) {
            self.bekleyen_cerezler = yedek.bekleyen_cerezler;
            self.bekleyen_silinen_cerezler = yedek.bekleyen_silinen_cerezler;
            return match self.web_guvenligi.commiti_geri_al(commit) {
                Ok(()) => Err(hata),
                Err(geri_alma) => Err(format!(
                    "{hata}; web oturum transaction'ı geri alınamadı: {geri_alma}"
                )),
            };
        }
        Ok(())
    }
    fn istek_islemini_geri_al(&mut self) {
        if let Some(yedek) = self.web_istek_yedegi.take() {
            self.web_guvenligi.istegi_geri_al();
            self.bekleyen_cerezler = yedek.bekleyen_cerezler;
            self.bekleyen_silinen_cerezler = yedek.bekleyen_silinen_cerezler;
        }
        self.bekleyen_web_yaniti = None;
    }
    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        if !dil::web_guvenligi::cerez_adi_gecerli(ad)
            || !dil::web_guvenligi::cerez_degeri_gecerli(deger)
        {
            return Err("çerez adı/değeri HTTP başlığı için güvenli değil".into());
        }
        self.bekleyen_cerezler.push(BekleyenCerez {
            ad: ad.to_string(),
            deger: deger.to_string(),
            azami_omur_saniye: None,
        });
        Ok(())
    }
    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        if !dil::web_guvenligi::cerez_adi_gecerli(ad) {
            return Err("çerez adı HTTP başlığı için güvenli değil".into());
        }
        self.bekleyen_silinen_cerezler.push(ad.to_string());
        Ok(())
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
        if self.web_istek_yedegi.is_some() {
            if self.bekleyen_web_yaniti.is_none() {
                self.bekleyen_web_yaniti = Some(WebYanitTaslagi::Govde(yanit.to_string()));
            }
            return;
        }
        let _ = self.web_yanit_taslagini_yaz(WebYanitTaslagi::Govde(yanit.to_string()));
    }
    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        if self.web_istek_yedegi.is_some() {
            if self.bekleyen_web_yaniti.is_none() {
                self.bekleyen_web_yaniti = Some(WebYanitTaslagi::Durum(durum, yanit.to_string()));
            }
            return;
        }
        let _ = self.web_yanit_taslagini_yaz(WebYanitTaslagi::Durum(durum, yanit.to_string()));
    }
    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        if !dil::web_guvenligi::yerel_yonlendirme_gecerli(adres) {
            return Err(
                "yönlendirme yalnız CR/LF içermeyen yerel `/...` adresine yapılabilir".into(),
            );
        }
        if self.web_istek_yedegi.is_some() {
            if self.bekleyen_web_yaniti.is_none() {
                self.bekleyen_web_yaniti = Some(WebYanitTaslagi::Yonlendirme(adres.to_string()));
            }
            return Ok(());
        }
        self.web_yanit_taslagini_yaz(WebYanitTaslagi::Yonlendirme(adres.to_string()))
    }
    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &dil::agac::RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), dil::web_guvenligi::WebReddi> {
        let an = web_duvar_saati_ms();
        if csrf_gerekli {
            let karar = self
                .web_guvenligi
                .rate_limit_artir(dil::web_guvenligi::RateLimitTuru::Csrf, "csrf", an)
                .map_err(|_| dil::web_guvenligi::WebReddi {
                    durum: 503,
                    mesaj: "CSRF oran sınırı deposuna erişilemedi",
                })?;
            if !karar.izinli {
                return Err(dil::web_guvenligi::WebReddi {
                    durum: 429,
                    mesaj: "çok fazla CSRF doğrulama isteği",
                });
            }
        }
        self.web_guvenligi.denetle(erisim, csrf, csrf_gerekli, an)
    }
    fn csrf_belirteci(&mut self) -> Result<String, String> {
        self.politika
            .gerektir(dil::yetkinlik::Yetkinlik::WebOturumu)?;
        let an = web_duvar_saati_ms();
        let (csrf, yeni) = self
            .web_guvenligi
            .csrf_belirteci(an, dil::guvenlik::guvenli_belirtec_uret)?;
        if let Some(yeni) = yeni {
            self.oturum_cerezini_yaz(yeni);
        }
        Ok(csrf)
    }
    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        self.politika
            .gerektir(dil::yetkinlik::Yetkinlik::WebOturumu)?;
        let an = web_duvar_saati_ms();
        let yeni = self.web_guvenligi.oturum_ac(
            kullanici.to_string(),
            rol.to_string(),
            an,
            dil::guvenlik::guvenli_belirtec_uret,
        )?;
        self.oturum_cerezini_yaz(yeni);
        Ok(())
    }
    fn oturum_kapat(&mut self) -> Result<(), String> {
        self.politika
            .gerektir(dil::yetkinlik::Yetkinlik::WebOturumu)?;
        self.web_guvenligi.oturum_kapat();
        self.bekleyen_silinen_cerezler
            .push(self.oturum_cerez_adi().to_string());
        Ok(())
    }
    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        if !self
            .politika
            .izin_verir(dil::yetkinlik::Yetkinlik::Kriptografi)
        {
            return false;
        }
        if self.web_istek_yedegi.is_none() {
            return dil::guvenlik::parola_dogrula(parola, ozet);
        }
        let karar = self.web_guvenligi.rate_limit_artir(
            dil::web_guvenligi::RateLimitTuru::Giris,
            "parola",
            web_duvar_saati_ms(),
        );
        match karar {
            Ok(karar) if karar.izinli => dil::guvenlik::parola_dogrula(parola, ozet),
            Ok(_) => {
                self.durum_yaniti_gonder(429, "çok fazla giriş denemesi");
                false
            }
            Err(_) => {
                self.durum_yaniti_gonder(503, "giriş oran sınırı deposuna erişilemedi");
                false
            }
        }
    }
    fn sensor_acik_mi(&mut self, _ad: &str) -> bool {
        // Donanım bağlı değil: simülatörde sensörler kapalı okunur (bölüm 17).
        false
    }
    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        if self.politika.izin_verir(dil::yetkinlik::Yetkinlik::Donanim) {
            println!("[ışık] {} {}", ad, if yansin { "yandı" } else { "söndü" });
        }
    }
    fn bekle_ms(&mut self, milisaniye: i64) {
        std::thread::sleep(std::time::Duration::from_millis(milisaniye.max(0) as u64));
    }
    fn an_ms(&mut self) -> i64 {
        self.baslangic.elapsed().as_millis() as i64
    }
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        self.rastgele.aralikta(alt, ust)
    }
}

fn calistir_komutu(girdi: &KaynakGirdisi) -> ExitCode {
    calistir_gercek_io_ile(girdi, WebModu::Kapali, girdi.politikasi())
}

/// K-082: localhost web prototipi üretim korkuluğunu yalnız açık opt-in'le geçer.
fn calistir_deneysel_web_komutu(girdi: &KaynakGirdisi) -> ExitCode {
    eprintln!(
        "UYARI: deneysel web yüzeyi yalnız localhost eğitim/prototipi içindir; üretim güvenlik sözleşmesi değildir."
    );
    calistir_gercek_io_ile(girdi, WebModu::Deneysel, girdi.politikasi())
}

fn calistir_guvenli_web_komutu(
    girdi: &KaynakGirdisi,
    origin: AgHedefi,
    worker_kapi: Option<u16>,
) -> ExitCode {
    eprintln!(
        "Güvenli web profili: yalnız 127.0.0.1 üzerindeki HTTPS reverse proxy güvenilir; origin {}.",
        origin.yazimi()
    );
    calistir_gercek_io_ile(
        girdi,
        WebModu::GuvenliProxy {
            origin,
            worker_kapi,
        },
        girdi.politikasi(),
    )
}

/// Çocuk modu (K-047): ağ/sunucu kapalı, dosyalar çalışma klasörüyle sınırlı.
fn calistir_guvenli_komutu(girdi: &KaynakGirdisi) -> ExitCode {
    calistir_gercek_io_ile(
        girdi,
        WebModu::Kapali,
        dil::yetkinlik::YetkinlikPolitikasi::cocuk(),
    )
}

fn calistir_gercek_io_ile(
    girdi: &KaynakGirdisi,
    web_modu: WebModu,
    politika: dil::yetkinlik::YetkinlikPolitikasi,
) -> ExitCode {
    let taban = girdi.gercek_io(web_modu, program_argumanlari(), politika.clone());
    let mut io = dil::yorumlayici::PolitikaliIo::politikali(taban, politika.clone());
    calistir_io_ile(girdi, &politika, &mut io)
}

fn calistir_io_ile(
    girdi: &KaynakGirdisi,
    politika: &dil::yetkinlik::YetkinlikPolitikasi,
    io: &mut dyn dil::yorumlayici::GirdiCikti,
) -> ExitCode {
    let mut yukleyici = |istek: dil::BirimIstegi<'_>| girdi.birim_yukle(istek);
    let program =
        match dil::kaynagi_derle_kokenlerle(&girdi.kaynak, Some(&girdi.koken), &mut yukleyici) {
            Ok(program) => program,
            Err(tani) => {
                eprint!("{}", tani.raporla(&girdi.kaynak));
                return ExitCode::FAILURE;
            }
        };
    if let Err(tani) = dil::cozumleyici::yetkinlikleri_denetle(&program, politika) {
        eprint!("{}", tani.raporla(&girdi.kaynak));
        return ExitCode::FAILURE;
    }
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
    let program =
        match dil::kaynagi_derle_kokenlerle(&girdi.kaynak, Some(&girdi.koken), &mut yukleyici) {
            Ok(program) => program,
            Err(tani) => {
                eprint!("{}", tani.raporla(&girdi.kaynak));
                return ExitCode::FAILURE;
            }
        };
    if let Err(tani) = dil::cozumleyici::yetkinlikleri_denetle(&program, &girdi.politikasi()) {
        eprint!("{}", tani.raporla(&girdi.kaynak));
        return ExitCode::FAILURE;
    }
    let sonuclar = dil::programi_dene(&program);
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

#[cfg(test)]
mod web_profili_testleri {
    use super::*;
    use dil::yorumlayici::GirdiCikti;
    use std::io::{Read, Write};

    #[test]
    fn guvenli_origin_ortak_ag_hedefiyle_kanoniklenir() {
        let origin = AgHedefi::https_origininden("https://PANEL.Example:8443").expect("origin");
        assert_eq!(origin.otoritesi(), "panel.example:8443");
        assert_eq!(origin.yazimi(), "https://panel.example:8443");
        let ipv6 =
            AgHedefi::https_origininden("https://[2001:db8::1]:8443").expect("IPv6 origin");
        assert_eq!(ipv6.otoritesi(), "[2001:db8::1]:8443");
        for gecersiz in [
            "http://panel.example",
            "https://panel.example/yol",
            "https://kisi@panel.example",
            "https://panel_example",
            "https://[2001:db8::1",
            "https://panel.example:+443",
            "https://panel.example:0",
            "https://panel.example:65536",
        ] {
            assert!(
                AgHedefi::https_origininden(gecersiz).is_err(),
                "{gecersiz}"
            );
        }
    }

    #[test]
    fn web_modu_bayragi_tekrarlanamaz_ve_origin_argumandan_ayiklanir() {
        let mut argumanlar = vec![
            "çalıştır".into(),
            "--web-proxy".into(),
            "https://panel.example".into(),
            "uygulama.dil".into(),
        ];
        assert!(matches!(
            web_modunu_ayikla(&mut argumanlar),
            Ok(WebModu::GuvenliProxy { .. })
        ));
        assert_eq!(argumanlar, vec!["çalıştır", "uygulama.dil"]);

        let mut tekrar = vec![
            "çalıştır".into(),
            "--deneysel-web".into(),
            "--deneysel-web".into(),
            "uygulama.dil".into(),
        ];
        assert!(web_modunu_ayikla(&mut tekrar).is_err());

        let mut worker = vec![
            "çalıştır".into(),
            "--web-worker-port".into(),
            "18092".into(),
            "--web-proxy".into(),
            "https://panel.example".into(),
            "uygulama.dil".into(),
        ];
        assert!(matches!(
            web_modunu_ayikla(&mut worker),
            Ok(WebModu::GuvenliProxy {
                worker_kapi: Some(18092),
                ..
            })
        ));
        assert_eq!(worker, vec!["çalıştır", "uygulama.dil"]);
    }

    #[test]
    fn proxy_host_proto_ve_unsafe_origini_birlikte_dogrular() {
        let origin = AgHedefi::https_origininden("https://panel.example").unwrap();
        let get = "GET / HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.7;proto=https;host=panel.example\r\n\r\n";
        assert_eq!(
            guvenli_proxy_istegini_denetle(get, &origin, "GET").unwrap(),
            "203.0.113.7"
        );

        let post = "POST /kaydet HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=2001:db8::7;proto=https;host=panel.example\r\nOrigin: https://panel.example\r\n\r\n";
        assert!(guvenli_proxy_istegini_denetle(post, &origin, "POST").is_ok());

        let originsiz =
            "POST /kaydet HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.7;proto=https;host=panel.example\r\n\r\n";
        assert_eq!(
            guvenli_proxy_istegini_denetle(originsiz, &origin, "POST")
                .unwrap_err()
                .0,
            403
        );
        let sahte_proto =
            "GET / HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.7;proto=http;host=panel.example\r\n\r\n";
        assert_eq!(
            guvenli_proxy_istegini_denetle(sahte_proto, &origin, "GET")
                .unwrap_err()
                .0,
            426
        );
        let cift_host = "GET / HTTP/1.1\r\nHost: panel.example\r\nHost: saldirgan.example\r\nForwarded: for=203.0.113.7;proto=https;host=panel.example\r\n\r\n";
        assert_eq!(
            guvenli_proxy_istegini_denetle(cift_host, &origin, "GET")
                .unwrap_err()
                .0,
            400
        );
        let sahte_zincir = "GET / HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=198.51.100.1;proto=https;host=panel.example, for=127.0.0.1\r\n\r\n";
        assert_eq!(
            guvenli_proxy_istegini_denetle(sahte_zincir, &origin, "GET")
                .unwrap_err()
                .0,
            400
        );
        let xff_tek_basina = "GET / HTTP/1.1\r\nHost: panel.example\r\nX-Forwarded-For: 203.0.113.7\r\nX-Forwarded-Proto: https\r\n\r\n";
        assert_eq!(
            guvenli_proxy_istegini_denetle(xff_tek_basina, &origin, "GET")
                .unwrap_err()
                .0,
            426
        );
        let cift_forwarded = "GET / HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.7;proto=https;host=panel.example\r\nForwarded: for=198.51.100.2;proto=https;host=panel.example\r\n\r\n";
        assert_eq!(
            guvenli_proxy_istegini_denetle(cift_forwarded, &origin, "GET")
                .unwrap_err()
                .0,
            400
        );

        let kanonik_esdeger = "POST /kaydet HTTP/1.1\r\nHost: PANEL.EXAMPLE:443\r\nForwarded: for=203.0.113.7;proto=HTTPS;host=panel.example:443\r\nOrigin: https://PANEL.EXAMPLE:443/\r\n\r\n";
        assert!(
            guvenli_proxy_istegini_denetle(kanonik_esdeger, &origin, "POST").is_ok()
        );
        let yanlis_kapi = "GET / HTTP/1.1\r\nHost: panel.example:8443\r\nForwarded: for=203.0.113.7;proto=https;host=panel.example:8443\r\n\r\n";
        assert_eq!(
            guvenli_proxy_istegini_denetle(yanlis_kapi, &origin, "GET")
                .unwrap_err()
                .0,
            400
        );

        let ipv6 = AgHedefi::https_origininden("https://[2001:db8::1]:8443").unwrap();
        let ipv6_istegi = "POST /kaydet HTTP/1.1\r\nHost: [2001:0DB8:0:0::1]:8443\r\nForwarded: for=203.0.113.7;proto=https;host=\"[2001:0db8:0:0::1]:8443\"\r\nOrigin: https://[2001:0DB8:0:0::1]:8443\r\n\r\n";
        assert!(guvenli_proxy_istegini_denetle(ipv6_istegi, &ipv6, "POST").is_ok());
    }

    #[test]
    fn guvenilir_proxy_yalniz_loopback_bind_ve_es_kabul_eder() {
        assert!(WEB_BIND_IP.is_loopback());
        for es in ["127.0.0.1:443", "[::1]:443"] {
            assert!(guvenilir_proxy_esi_mi(es.parse().unwrap()), "{es}");
        }
        for es in ["192.0.2.1:443", "[2001:db8::1]:443"] {
            assert!(!guvenilir_proxy_esi_mi(es.parse().unwrap()), "{es}");
        }
    }

    #[test]
    fn https_profili_hsts_ve_tarayici_korkuluklarini_tasir() {
        let basliklar = guvenlik_basliklari(true);
        assert!(basliklar.contains("Strict-Transport-Security"));
        assert!(basliklar.contains("Content-Security-Policy"));
        assert!(basliklar.contains("X-Content-Type-Options: nosniff"));
        assert!(!guvenlik_basliklari(false).contains("Strict-Transport-Security"));
    }

    #[test]
    fn soket_okumasi_mutlak_son_tarihi_gecemez() {
        let dinleyici = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let adres = dinleyici.local_addr().unwrap();
        let mut istemci = std::net::TcpStream::connect(adres).unwrap();
        let (mut sunucu, _) = dinleyici.accept().unwrap();
        sunucu.write_all(b"x").unwrap();
        let gecmis = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_millis(1))
            .unwrap();
        let mut bayt = [0u8; 1];
        let hata = son_tarihli_soket_oku(&mut istemci, &mut bayt, gecmis).unwrap_err();
        assert_eq!(hata.kind(), std::io::ErrorKind::TimedOut);
    }

    #[test]
    fn gercek_web_yaniti_commit_oncesi_yayimlanmaz_rollback_oturumu_siler() {
        let dinleyici = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let adres = dinleyici.local_addr().unwrap();
        let mut istemci = std::net::TcpStream::connect(adres).unwrap();
        let (sunucu, _) = dinleyici.accept().unwrap();
        istemci
            .set_read_timeout(Some(std::time::Duration::from_millis(20)))
            .unwrap();
        let mut io = GercekIo::yeni_argumanlarla(
            std::path::Path::new("."),
            std::path::Path::new("."),
            WebModu::Deneysel,
            Vec::new(),
            dil::yetkinlik::YetkinlikPolitikasi::gelistirici(),
        );
        io.bekleyen_akis = Some(sunucu);
        io.web_istek_yedegi = Some(WebIstekYedegi {
            bekleyen_cerezler: Vec::new(),
            bekleyen_silinen_cerezler: Vec::new(),
        });
        io.oturum_ac("Mustafa", "yönetici").unwrap();
        io.yanit_gonder("erken başarı");

        let mut bayt = [0u8; 1];
        let hata = istemci.read(&mut bayt).unwrap_err();
        assert!(matches!(
            hata.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
        io.istek_islemini_geri_al();
        assert_eq!(io.web_guvenligi.oturum_sayisi().unwrap(), 0);
        assert!(io.bekleyen_cerezler.is_empty());
        io.durum_yaniti_gonder(504, "zaman aşımı");

        istemci.set_read_timeout(None).unwrap();
        let mut yanit = String::new();
        istemci.read_to_string(&mut yanit).unwrap();
        assert!(yanit.starts_with("HTTP/1.1 504 Gateway Timeout"));
        assert!(yanit.ends_with("zaman aşımı"));
        assert!(!yanit.contains("erken başarı"));
    }

    #[test]
    fn web_yaniti_yazilamazsa_oturum_commit_edilmez() {
        let dinleyici = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let adres = dinleyici.local_addr().unwrap();
        let istemci = std::net::TcpStream::connect(adres).unwrap();
        let (sunucu, _) = dinleyici.accept().unwrap();
        sunucu.shutdown(std::net::Shutdown::Both).unwrap();
        drop(istemci);
        let mut io = GercekIo::yeni_argumanlarla(
            std::path::Path::new("."),
            std::path::Path::new("."),
            WebModu::Deneysel,
            Vec::new(),
            dil::yetkinlik::YetkinlikPolitikasi::gelistirici(),
        );
        io.bekleyen_akis = Some(sunucu);
        io.web_istek_yedegi = Some(WebIstekYedegi {
            bekleyen_cerezler: Vec::new(),
            bekleyen_silinen_cerezler: Vec::new(),
        });
        io.oturum_ac("Mustafa", "yönetici").unwrap();
        io.yanit_gonder("başarı");

        assert!(io.istek_islemini_tamamla().is_err());
        assert_eq!(io.web_guvenligi.oturum_sayisi().unwrap(), 0);
        assert!(io.bekleyen_cerezler.is_empty());

        let gecici = std::env::temp_dir().join(format!(
            "zee-web-commit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&gecici).unwrap();
        let origin = AgHedefi::https_origininden("https://panel.example").unwrap();
        let mut io = GercekIo::yeni_argumanlarla(
            &gecici,
            &gecici,
            WebModu::GuvenliProxy {
                origin,
                worker_kapi: None,
            },
            Vec::new(),
            dil::yetkinlik::YetkinlikPolitikasi::gelistirici(),
        );
        io.web_guvenligi
            .istegi_baslat_kimlikle(None, "192.0.2.1", web_duvar_saati_ms())
            .unwrap();
        let dinleyici = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let adres = dinleyici.local_addr().unwrap();
        let mut istemci = std::net::TcpStream::connect(adres).unwrap();
        let (sunucu, _) = dinleyici.accept().unwrap();
        io.bekleyen_akis = Some(sunucu);
        io.web_istek_yedegi = Some(WebIstekYedegi {
            bekleyen_cerezler: Vec::new(),
            bekleyen_silinen_cerezler: Vec::new(),
        });
        io.oturum_ac("Mustafa", "yönetici").unwrap();
        io.yanit_gonder("commit edilmemeli");
        std::fs::write(gecici.join(".zee/web-durumu-v1.json"), b"{bozuk").unwrap();
        io.istek_islemini_tamamla()
            .expect("depo commit hatası kontrollü 503 olmalı");
        let mut yanit = String::new();
        istemci.read_to_string(&mut yanit).unwrap();
        assert!(yanit.starts_with("HTTP/1.1 503 Service Unavailable"));
        assert!(!yanit.contains("commit edilmemeli"));
        std::fs::remove_dir_all(&gecici).unwrap();
    }

    #[test]
    fn http_istemcisi_acik_yerel_ag_izniyle_varsayilan_sureyi_kullanir() {
        let dinleyici = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let adres = dinleyici.local_addr().unwrap();
        let sunucu = std::thread::spawn(move || {
            let (mut akis, _) = dinleyici.accept().unwrap();
            // İstemci başlığını bütünüyle tüketmeden kapanmak macOS'ta okunmamış
            // baytlar yüzünden RST üretip başarılı yanıtı kararsızlaştırabilir.
            let mut istek = Vec::new();
            let mut parca = [0u8; 256];
            while !istek.windows(4).any(|pencere| pencere == b"\r\n\r\n") {
                let okunan = std::io::Read::read(&mut akis, &mut parca).unwrap();
                assert!(okunan > 0, "HTTP isteği başlık sonundan önce kapandı");
                istek.extend_from_slice(&parca[..okunan]);
            }
            akis.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nmerhaba",
            )
            .unwrap();
        });
        let url = format!("http://{}/", adres);
        let politika = dil::yetkinlik::YetkinlikPolitikasi::proje(
            [
                dil::yetkinlik::Yetkinlik::Ag,
                dil::yetkinlik::Yetkinlik::YerelAg,
            ]
            .into_iter()
            .collect(),
            [dil::yetkinlik::AgHedefi::bildirimden(&url).unwrap()]
                .into_iter()
                .collect(),
        );
        let mut io = GercekIo::yeni_argumanlarla(
            std::path::Path::new("."),
            std::path::Path::new("."),
            WebModu::Kapali,
            Vec::new(),
            politika,
        );
        let (durum, govde) = io
            .http_getir(&url, None)
            .expect("varsayılan deadline ile yanıt");
        assert_eq!(durum, 200);
        assert_eq!(govde, "merhaba");
        sunucu.join().unwrap();
    }
}
