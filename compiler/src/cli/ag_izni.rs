//! K-387: bildirimsiz tek dosya kipinde dış ağ yalnız açık `--izin ağ` ile açılır.
//!
//! Bayrak yalnız dosya işleneninden önce okunur; sonrası programın kendi
//! argümanıdır ve yetki genişletemez. Proje kipinde yetki `proje.dil`
//! `yetkinlikler` listesinden gelir; orada bayrak reddedilir (sessiz no-op yok).

use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) const IZIN_BAYRAGI: &str = "--izin";
pub(crate) const PROJE_RET: &str = "--izin ağ yalnız bildirimsiz tek dosya kipinde geçerlidir; projede ağ, proje.dil `yetkinlikler` ve `ağ_hedefleri` listeleriyle açılır";

/// Süreç başına tek CLI çağrısı: bayrak `govde` başında bir kez ayıklanır,
/// kaynak girdisi okunurken (`girdiyi_oku`) uygulanır.
static VERILDI: AtomicBool = AtomicBool::new(false);

/// `--izin ağ` bayrağını ayıklar ve süreç için kaydeder; `--güvenli` ile çakışır.
pub(crate) fn ayikla(argumanlar: &mut Vec<String>, guvenli: bool) -> Result<(), String> {
    let izin = bayragi_ayikla(argumanlar)?;
    if izin && guvenli {
        return Err("--güvenli çocuk modu --izin ağ ile birlikte kullanılamaz".into());
    }
    VERILDI.store(izin, Ordering::Relaxed);
    Ok(())
}

/// Bu süreçte açık `--izin ağ` verildi mi.
pub(crate) fn verildi() -> bool {
    VERILDI.load(Ordering::Relaxed)
}

/// `--izin ağ` bayrağını ayıklar. Bulunursa `true` döner ve argümanlardan siler.
fn bayragi_ayikla(argumanlar: &mut Vec<String>) -> Result<bool, String> {
    let komut = argumanlar.first().cloned().unwrap_or_default();
    let komut = komut.as_str();
    let ilk = match komut {
        // `iz kaydet|oynat` eylemi işlenenden önce gelir.
        "iz" => 2,
        _ => 1,
    };
    let mut yer = ilk;
    let mut bulundu = false;
    while yer < argumanlar.len() && argumanlar[yer].starts_with("--") {
        if argumanlar[yer] != IZIN_BAYRAGI {
            yer += 1;
            continue;
        }
        if bulundu {
            return Err("--izin birden çok kez verilemez".into());
        }
        match argumanlar.get(yer + 1).map(String::as_str) {
            Some("ağ") => {}
            Some(deger) if !deger.starts_with("--") => {
                return Err(format!(
                    "--izin yalnız `ağ` değerini kabul eder (verilen: `{deger}`); diğer yetkinlikler proje.dil `yetkinlikler` listesiyle açılır"
                ));
            }
            _ => return Err("--izin ardından `ağ` değeri ister".into()),
        }
        argumanlar.drain(yer..yer + 2);
        bulundu = true;
    }
    if bulundu {
        let eylem = argumanlar.get(1).map(String::as_str);
        let uygun = match komut {
            "çalıştır" | "calistir" | "denetle" => true,
            "iz" => eylem == Some("kaydet"),
            _ => false,
        };
        if !uygun {
            return Err(
                "--izin ağ yalnız `dil çalıştır`, `dil denetle` ve `dil iz kaydet` ile kullanılabilir"
                    .into(),
            );
        }
    }
    Ok(bulundu)
}

#[cfg(test)]
mod testler {
    use super::bayragi_ayikla as ayikla;

    fn args(dizi: &[&str]) -> Vec<String> {
        dizi.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn izin_ag_dosyadan_once_ayiklanir_sonrasi_program_argumanidir() {
        let mut a = args(&["çalıştır", "--izin", "ağ", "x.dil", "--izin", "ağ"]);
        assert_eq!(ayikla(&mut a), Ok(true));
        assert_eq!(a, args(&["çalıştır", "x.dil", "--izin", "ağ"]));
        let mut a = args(&["çalıştır", "x.dil", "--izin", "ağ"]);
        assert_eq!(ayikla(&mut a), Ok(false));
        assert_eq!(a.len(), 4);
        let mut a = args(&["iz", "kaydet", "--izin", "ağ", "iz.json", "x.dil"]);
        assert_eq!(ayikla(&mut a), Ok(true));
        assert_eq!(a, args(&["iz", "kaydet", "iz.json", "x.dil"]));
        let mut a = args(&["denetle", "--json", "--izin", "ağ", "x.dil"]);
        assert_eq!(ayikla(&mut a), Ok(true));
        assert_eq!(a, args(&["denetle", "--json", "x.dil"]));
    }

    #[test]
    fn izin_yanlis_deger_tekrar_ve_uygunsuz_komut_reddedilir() {
        for dizi in [
            &["çalıştır", "--izin", "dosya-yazma", "x.dil"][..],
            &["çalıştır", "--izin"][..],
            &["çalıştır", "--izin", "--json", "x.dil"][..],
            &["çalıştır", "--izin", "ağ", "--izin", "ağ", "x.dil"][..],
            &["dene", "--izin", "ağ", "x.dil"][..],
            &["iz", "oynat", "--izin", "ağ", "iz.json", "x.dil"][..],
        ] {
            assert!(ayikla(&mut args(dizi)).is_err(), "{dizi:?}");
        }
    }
}
