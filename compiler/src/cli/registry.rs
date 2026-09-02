//! Registry bağımlılıkları için açık ağ kullanan CLI komutları. Normal
//! çalıştır/denetle/dene bu modülü çağırmaz ve yalnız offline grafiği açar.

use dil::paket::{ProjeGrafigi, RegistryCozumPolitikasi};
use dil::proje::{bildirimi_oku, uzak_bagimliliklari_guncelle, RegistryBildirimi, UzakBagimlilik};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const EKLE_KULLANIMI: &str = "dil ekle <ad@X.Y.Z> [proje] [--registry https://...] [--kök 1@sha256:...] [--çevrimdışı] [--yanked-kabul gerekçe] [--kritik-kabul gerekçe]";
const KILITLE_KULLANIMI: &str =
    "dil kilitle [proje] [--çevrimdışı] [--yanked-kabul gerekçe] [--kritik-kabul gerekçe]";
const PAKETLER_KULLANIMI: &str =
    "dil paketler [proje] [--yenile] [--yanked-kabul gerekçe] [--kritik-kabul gerekçe]";

pub fn uzak_ekle_komutu(argumanlar: &[String]) -> ExitCode {
    let Some(yazim) = argumanlar.get(1) else {
        return kullanim_hatasi(
            "Uzak paket ad@X.Y.Z biçiminde belirtilmeli.",
            EKLE_KULLANIMI,
        );
    };
    let bagimlilik = match UzakBagimlilik::ayristir(yazim) {
        Ok(bagimlilik) => bagimlilik,
        Err(neden) => return kullanim_hatasi(&format!("P017: {}", neden), EKLE_KULLANIMI),
    };
    let secenekler = match ekleme_seceneklerini_ayristir(&argumanlar[2..]) {
        Ok(secenekler) => secenekler,
        Err(neden) => return kullanim_hatasi(&neden, EKLE_KULLANIMI),
    };
    let (proje_koku, _, eski_kaynak, bildirim) = match projeyi_oku(&secenekler.proje) {
        Ok(proje) => proje,
        Err(kod) => return kod,
    };
    let registry = match registry_sec(&bildirim.registry, &secenekler) {
        Ok(registry) => registry,
        Err(neden) => return kullanim_hatasi(&neden, EKLE_KULLANIMI),
    };
    let mut bagimliliklar = bildirim.uzak_bagimliliklar;
    bagimliliklar.retain(|eski| eski.ad != bagimlilik.ad);
    bagimliliklar.push(bagimlilik.clone());
    let yeni_kaynak =
        match uzak_bagimliliklari_guncelle(&eski_kaynak, Some(&registry), &bagimliliklar) {
            Ok(kaynak) => kaynak,
            Err(tani) => {
                eprint!("{}", tani.raporla(&eski_kaynak));
                return ExitCode::FAILURE;
            }
        };
    let politika = match RegistryCozumPolitikasi::yeni(
        secenekler.cevrimdisi,
        secenekler.yanked_gerekcesi,
        secenekler.kritik_gerekcesi,
    ) {
        Ok(politika) => politika,
        Err(neden) => return kullanim_hatasi(&neden, EKLE_KULLANIMI),
    };
    let grafik =
        match ProjeGrafigi::cozumle_bildirimle_registry(&proje_koku, &yeni_kaynak, &politika) {
            Ok(grafik) => grafik,
            Err(hata) => return proje_hatasini_yaz(hata),
        };
    if let Err(hata) = grafik.bildirim_ve_kilidi_yaz(&eski_kaynak, &yeni_kaynak) {
        eprintln!("{}", hata);
        return ExitCode::from(2);
    }
    println!(
        "Eklendi: {}@{} — proje.dil ve proje.kilit güncellendi{}.",
        bagimlilik.ad,
        bagimlilik.surum,
        if secenekler.cevrimdisi {
            " (çevrimdışı cache)"
        } else {
            ""
        }
    );
    ExitCode::SUCCESS
}

pub fn kilitle_komutu(argumanlar: &[String]) -> ExitCode {
    let secenekler = match cozum_seceneklerini_ayristir(&argumanlar[1..], false) {
        Ok(secenekler) => secenekler,
        Err(neden) => return kullanim_hatasi(&neden, KILITLE_KULLANIMI),
    };
    let politika = match RegistryCozumPolitikasi::yeni(
        secenekler.cevrimdisi,
        secenekler.yanked_gerekcesi,
        secenekler.kritik_gerekcesi,
    ) {
        Ok(politika) => politika,
        Err(neden) => return kullanim_hatasi(&neden, KILITLE_KULLANIMI),
    };
    let grafik = match ProjeGrafigi::cozumle_registry_ile(&secenekler.proje, &politika) {
        Ok(grafik) => grafik,
        Err(hata) => return proje_hatasini_yaz(hata),
    };
    match grafik.kilidi_yaz() {
        Ok(()) => {
            let uzak = grafik
                .paketler()
                .iter()
                .filter(|paket| paket.registry_kok_sha256.is_some())
                .count();
            println!(
                "Kilitlendi: {} ({} paket; {} registry)",
                grafik.ana_kok().join(dil::paket::KILIT_DOSYASI).display(),
                grafik.paket_sayisi(),
                uzak
            );
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("{}", hata);
            ExitCode::from(2)
        }
    }
}

pub fn paketler_komutu(argumanlar: &[String]) -> ExitCode {
    let secenekler = match cozum_seceneklerini_ayristir(&argumanlar[1..], true) {
        Ok(secenekler) => secenekler,
        Err(neden) => return kullanim_hatasi(&neden, PAKETLER_KULLANIMI),
    };
    let politika = match RegistryCozumPolitikasi::yeni(
        secenekler.cevrimdisi,
        secenekler.yanked_gerekcesi,
        secenekler.kritik_gerekcesi,
    ) {
        Ok(politika) => politika,
        Err(neden) => return kullanim_hatasi(&neden, PAKETLER_KULLANIMI),
    };
    let grafik = match ProjeGrafigi::cozumle_registry_ile(&secenekler.proje, &politika) {
        Ok(grafik) => grafik,
        Err(hata) => return proje_hatasini_yaz(hata),
    };
    if secenekler.yenile {
        if let Err(hata) = grafik.kilidi_yaz() {
            eprintln!("{}", hata);
            return ExitCode::from(2);
        }
    } else if let Err(hata) = grafik.kilidi_denetle() {
        return proje_hatasini_yaz(hata);
    }
    let bildirim = grafik.ana_bildirim();
    let paketler = grafik.paketler();
    let uzak = paketler
        .iter()
        .filter(|paket| paket.registry_kok_sha256.is_some())
        .count();
    println!(
        "proje: {} {} — {} paket ({} registry)",
        bildirim.ad,
        bildirim.surum,
        paketler.len(),
        uzak
    );
    if paketler.is_empty() {
        println!("Bağımlılık yok.");
        return ExitCode::SUCCESS;
    }
    for paket in paketler {
        let kapsam = if paket.dogrudan {
            "doğrudan"
        } else {
            "geçişli"
        };
        match (paket.registry_kok_sha256, paket.arsiv_sha256) {
            (Some(kok), Some(arsiv)) => println!(
                "{}: {} {} · morfoloji {} · registry {} · arşiv sha256:{} · kaynak sha256:{}",
                kapsam, paket.ad, paket.surum, paket.morfoloji, kok, arsiv, paket.ozet
            ),
            _ => println!(
                "{}: {} {} · morfoloji {} · {} · sha256:{}",
                kapsam, paket.ad, paket.surum, paket.morfoloji, paket.yol, paket.ozet
            ),
        }
    }
    ExitCode::SUCCESS
}

/// Hedef doğrudan registry bağımlılığıysa kaldırmayı bu modül üstlenir.
pub fn uzak_cikar_komutu(argumanlar: &[String]) -> Option<ExitCode> {
    let paket_adi = argumanlar.get(1)?;
    if argumanlar.len() > 3 {
        return None;
    }
    let proje = PathBuf::from(argumanlar.get(2).map_or(".", String::as_str));
    let (proje_koku, _, eski_kaynak, bildirim) = projeyi_oku(&proje).ok()?;
    if !bildirim
        .uzak_bagimliliklar
        .iter()
        .any(|bagimlilik| bagimlilik.ad == *paket_adi)
    {
        return None;
    }
    let grafik = match ProjeGrafigi::cozumle(&proje_koku) {
        Ok(grafik) => grafik,
        Err(hata) => return Some(proje_hatasini_yaz(hata)),
    };
    if let Err(hata) = grafik.kaldirmayi_dogrula(paket_adi) {
        return Some(proje_hatasini_yaz(hata));
    }
    let kalan = bildirim
        .uzak_bagimliliklar
        .into_iter()
        .filter(|bagimlilik| bagimlilik.ad != *paket_adi)
        .collect::<Vec<_>>();
    let yeni_kaynak =
        match uzak_bagimliliklari_guncelle(&eski_kaynak, bildirim.registry.as_ref(), &kalan) {
            Ok(kaynak) => kaynak,
            Err(tani) => {
                eprint!("{}", tani.raporla(&eski_kaynak));
                return Some(ExitCode::FAILURE);
            }
        };
    let yeni_grafik = match ProjeGrafigi::cozumle_bildirimle(&proje_koku, &yeni_kaynak) {
        Ok(grafik) => grafik,
        Err(hata) => return Some(proje_hatasini_yaz(hata)),
    };
    if let Err(hata) = yeni_grafik.bildirim_ve_kilidi_yaz(&eski_kaynak, &yeni_kaynak) {
        eprintln!("{}", hata);
        return Some(ExitCode::from(2));
    }
    println!(
        "Çıkarıldı: {} — proje.dil ve proje.kilit güncellendi.",
        paket_adi
    );
    Some(ExitCode::SUCCESS)
}

struct EklemeSecenekleri {
    proje: PathBuf,
    registry: Option<String>,
    kok: Option<(u64, String)>,
    cevrimdisi: bool,
    yanked_gerekcesi: Option<String>,
    kritik_gerekcesi: Option<String>,
}

fn ekleme_seceneklerini_ayristir(argumanlar: &[String]) -> Result<EklemeSecenekleri, String> {
    let mut sonuc = EklemeSecenekleri {
        proje: PathBuf::from("."),
        registry: None,
        kok: None,
        cevrimdisi: false,
        yanked_gerekcesi: None,
        kritik_gerekcesi: None,
    };
    let mut proje_goruldu = false;
    let mut i = 0;
    while i < argumanlar.len() {
        match argumanlar[i].as_str() {
            "--registry" => {
                let deger = tek_secenek_degeri(argumanlar, &mut i, "--registry")?;
                if sonuc.registry.replace(deger).is_some() {
                    return Err("--registry birden çok kez verilemez.".into());
                }
            }
            "--kök" | "--kok" => {
                let deger = tek_secenek_degeri(argumanlar, &mut i, "--kök")?;
                if sonuc.kok.replace(kok_pinini_ayristir(&deger)?).is_some() {
                    return Err("--kök birden çok kez verilemez.".into());
                }
            }
            "--çevrimdışı" | "--cevrimdisi" => {
                sonuc.cevrimdisi = true;
                i += 1;
            }
            "--yanked-kabul" => {
                let deger = tek_secenek_degeri(argumanlar, &mut i, "--yanked-kabul")?;
                if sonuc.yanked_gerekcesi.replace(deger).is_some() {
                    return Err("--yanked-kabul birden çok kez verilemez.".into());
                }
            }
            "--kritik-kabul" => {
                let deger = tek_secenek_degeri(argumanlar, &mut i, "--kritik-kabul")?;
                if sonuc.kritik_gerekcesi.replace(deger).is_some() {
                    return Err("--kritik-kabul birden çok kez verilemez.".into());
                }
            }
            bilinmeyen if bilinmeyen.starts_with('-') => {
                return Err(format!("Bilinmeyen registry seçeneği: {}", bilinmeyen));
            }
            proje if !proje_goruldu => {
                sonuc.proje = PathBuf::from(proje);
                proje_goruldu = true;
                i += 1;
            }
            _ => return Err("Uzak ekle komutu en çok bir proje klasörü alır.".into()),
        }
    }
    Ok(sonuc)
}

struct CozumSecenekleri {
    proje: PathBuf,
    cevrimdisi: bool,
    yenile: bool,
    yanked_gerekcesi: Option<String>,
    kritik_gerekcesi: Option<String>,
}

fn cozum_seceneklerini_ayristir(
    argumanlar: &[String],
    paketler_mi: bool,
) -> Result<CozumSecenekleri, String> {
    let mut sonuc = CozumSecenekleri {
        proje: PathBuf::from("."),
        cevrimdisi: paketler_mi,
        yenile: false,
        yanked_gerekcesi: None,
        kritik_gerekcesi: None,
    };
    // `kilitle` varsayılan çevrimiçidir; `paketler` sessiz ağ açmaz.
    let mut cevrimdisi_istendi = false;
    let mut proje_goruldu = false;
    let mut i = 0;
    while i < argumanlar.len() {
        match argumanlar[i].as_str() {
            "--çevrimdışı" | "--cevrimdisi" => {
                if sonuc.yenile || cevrimdisi_istendi {
                    return Err("--çevrimdışı ile --yenile birlikte kullanılamaz.".into());
                }
                sonuc.cevrimdisi = true;
                cevrimdisi_istendi = true;
                i += 1;
            }
            "--yenile" if paketler_mi => {
                if cevrimdisi_istendi || sonuc.yenile {
                    return Err("--yenile birden çok kez verilemez.".into());
                }
                sonuc.yenile = true;
                sonuc.cevrimdisi = false;
                i += 1;
            }
            "--yanked-kabul" => {
                let deger = tek_secenek_degeri(argumanlar, &mut i, "--yanked-kabul")?;
                if sonuc.yanked_gerekcesi.replace(deger).is_some() {
                    return Err("--yanked-kabul birden çok kez verilemez.".into());
                }
            }
            "--kritik-kabul" => {
                let deger = tek_secenek_degeri(argumanlar, &mut i, "--kritik-kabul")?;
                if sonuc.kritik_gerekcesi.replace(deger).is_some() {
                    return Err("--kritik-kabul birden çok kez verilemez.".into());
                }
            }
            bilinmeyen if bilinmeyen.starts_with('-') => {
                return Err(format!("Bilinmeyen registry seçeneği: {}", bilinmeyen));
            }
            proje if !proje_goruldu => {
                sonuc.proje = PathBuf::from(proje);
                proje_goruldu = true;
                i += 1;
            }
            _ => return Err("Komut en çok bir proje klasörü alır.".into()),
        }
    }
    Ok(sonuc)
}

fn registry_sec(
    mevcut: &Option<RegistryBildirimi>,
    secenekler: &EklemeSecenekleri,
) -> Result<RegistryBildirimi, String> {
    let origin = secenekler
        .registry
        .as_deref()
        .or_else(|| mevcut.as_ref().map(|registry| registry.origin.as_str()))
        .ok_or_else(|| "İlk uzak paket için --registry https://... gerekli.".to_string())?;
    let (surum, ozet) = secenekler
        .kok
        .clone()
        .or_else(|| {
            mevcut
                .as_ref()
                .map(|registry| (registry.kok_surumu, registry.kok_sha256.clone()))
        })
        .ok_or_else(|| {
            "İlk uzak paket için --kök <sürüm>@sha256:<64 küçük hex> gerekli.".to_string()
        })?;
    RegistryBildirimi::yeni(origin, surum, &ozet).map_err(|neden| format!("P017: {}", neden))
}

fn kok_pinini_ayristir(yazim: &str) -> Result<(u64, String), String> {
    let (surum, ozet) = yazim
        .split_once('@')
        .ok_or_else(|| "--kök <sürüm>@sha256:<64 küçük hex> biçiminde olmalı.".to_string())?;
    let surum = surum
        .parse::<u64>()
        .ok()
        .filter(|surum| *surum > 0)
        .ok_or_else(|| "--kök sürümü pozitif ondalık sayı olmalı.".to_string())?;
    Ok((surum, ozet.to_string()))
}

fn tek_secenek_degeri(argumanlar: &[String], i: &mut usize, ad: &str) -> Result<String, String> {
    let deger = argumanlar
        .get(*i + 1)
        .filter(|deger| !deger.starts_with('-'))
        .ok_or_else(|| format!("{} ardından değer ister.", ad))?
        .clone();
    *i += 2;
    Ok(deger)
}

fn projeyi_oku(
    proje_yolu: &Path,
) -> Result<(PathBuf, PathBuf, String, dil::proje::ProjeBildirimi), ExitCode> {
    let proje_koku = match std::fs::canonicalize(proje_yolu) {
        Ok(yol) if yol.is_dir() => yol,
        Ok(_) => {
            eprintln!("\"{}\" bir proje klasörü değil.", proje_yolu.display());
            return Err(ExitCode::from(2));
        }
        Err(hata) => {
            eprintln!(
                "\"{}\" proje klasörü çözülemedi: {}",
                proje_yolu.display(),
                hata
            );
            return Err(ExitCode::from(2));
        }
    };
    let bildirim_yolu = proje_koku.join("proje.dil");
    let kaynak = match dil::kaynak_sinirlari::kaynak_dosyasi_oku(&bildirim_yolu) {
        Ok(kaynak) => kaynak,
        Err(hata) => {
            eprintln!("\"{}\" okunamadı: {}", bildirim_yolu.display(), hata);
            return Err(ExitCode::from(2));
        }
    };
    let bildirim = match bildirimi_oku(&kaynak) {
        Ok(bildirim) => bildirim,
        Err(tani) => {
            eprint!("{}", tani.raporla(&kaynak));
            return Err(ExitCode::FAILURE);
        }
    };
    Ok((proje_koku, bildirim_yolu, kaynak, bildirim))
}

fn proje_hatasini_yaz(hata: dil::paket::ProjeYuklemeHatasi) -> ExitCode {
    if !hata.yol.as_os_str().is_empty() {
        eprintln!("{}:", hata.yol.display());
    }
    eprint!("{}", hata.tani.raporla(&hata.kaynak));
    ExitCode::FAILURE
}

fn kullanim_hatasi(mesaj: &str, kullanim: &str) -> ExitCode {
    eprintln!("{}", mesaj);
    eprintln!("Kullanım: {}", kullanim);
    ExitCode::from(2)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn uzak_ekle_secenekleri_exact_pin_ve_gerekceleri_ayirir() {
        let ozet = format!("sha256:{}", "a".repeat(64));
        let argumanlar = vec![
            "proje".into(),
            "--registry".into(),
            "https://registry.example".into(),
            "--kök".into(),
            format!("7@{}", ozet),
            "--yanked-kabul".into(),
            "geçiş planı".into(),
        ];
        let sonuc = ekleme_seceneklerini_ayristir(&argumanlar).expect("seçenekler");
        assert_eq!(sonuc.proje, PathBuf::from("proje"));
        assert_eq!(sonuc.kok, Some((7, ozet)));
        assert_eq!(sonuc.yanked_gerekcesi.as_deref(), Some("geçiş planı"));
    }

    #[test]
    fn paketler_yenile_ve_cevrimdisi_birlikte_reddedilir() {
        for argumanlar in [
            vec!["--yenile".into(), "--çevrimdışı".into()],
            vec!["--çevrimdışı".into(), "--yenile".into()],
        ] {
            assert!(cozum_seceneklerini_ayristir(&argumanlar, true).is_err());
        }
        let varsayilan = cozum_seceneklerini_ayristir(&[], true).expect("varsayılan");
        assert!(varsayilan.cevrimdisi);
        assert!(!varsayilan.yenile);
    }
}
