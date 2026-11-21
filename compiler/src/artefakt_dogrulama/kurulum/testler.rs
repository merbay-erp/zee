//! K-363 (GB-109 paket/cache yayın süresi): içerik-adresli paket kurulumunun
//! yayın penceresi. Kurulum görünmez kardeş geçicide yazılıp tek `rename` ile
//! yayımlanır; yarış kaybeden yalnız doğrulanmış kazananı kabul eder, yarım
//! kalan geçici ya da bozulmuş yayın sessizce onarılmaz.

use super::super::{arsivi_yaz, KaynakGirdisi};
use super::paket_arsivini_kur;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};

static SAYAC: AtomicU64 = AtomicU64::new(0);

struct Gecici(PathBuf);

impl Gecici {
    fn yeni() -> Self {
        let sira = SAYAC.fetch_add(1, Ordering::Relaxed);
        let yol = std::env::temp_dir().join(format!("zee-kurulum-{}-{}", std::process::id(), sira));
        std::fs::create_dir(&yol).expect("geçici klasör");
        Self(yol)
    }
}

impl Drop for Gecici {
    fn drop(&mut self) {
        yazilabilir_yap(&self.0);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn yazilabilir_yap(yol: &Path) {
    let Ok(metadata) = std::fs::symlink_metadata(yol) else {
        return;
    };
    if metadata.is_dir() {
        for girdi in std::fs::read_dir(yol).into_iter().flatten().flatten() {
            yazilabilir_yap(&girdi.path());
        }
    }
    let mut izin = metadata.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        izin.set_mode(izin.mode() | 0o200);
    }
    #[cfg(not(unix))]
    #[allow(clippy::permissions_set_readonly_false)]
    izin.set_readonly(false);
    let _ = std::fs::set_permissions(yol, izin);
}

const GIRDILER: &[(&str, &str)] = &[
    (
        "kaynak/yardımcı.dil",
        "işlem iki ver\n    TamSayı döndürür\n    2 döndür\n",
    ),
    (
        "paket.dil",
        "işlem yedi ver\n    TamSayı döndürür\n    7 döndür\n",
    ),
    (
        "proje.dil",
        "proje \"miras\" olsun\nsürüm \"1.2.3\" olsun\ngiriş \"paket.dil\" olsun\n",
    ),
];

fn arsiv(gecici: &Gecici, girdiler: &[(&str, &str)]) -> PathBuf {
    let girdiler = girdiler
        .iter()
        .map(|(yol, icerik)| KaynakGirdisi {
            yol: (*yol).into(),
            icerik: icerik.as_bytes().to_vec(),
        })
        .collect::<Vec<_>>();
    let yol = gecici
        .0
        .join(format!("{}.zep", SAYAC.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&yol, arsivi_yaz(&girdiler).expect("ham arşiv")).expect("arşiv yaz");
    yol
}

/// Kurulum ağacını `/` ayraçlı göreli yol → içerik olarak okur; klasörler
/// ayrıca `yol/` anahtarıyla görünür.
fn agac(kok: &Path) -> Vec<(String, Option<String>)> {
    fn gez(kok: &Path, klasor: &Path, sonuc: &mut Vec<(String, Option<String>)>) {
        let mut girdiler = std::fs::read_dir(klasor)
            .expect("listele")
            .collect::<Result<Vec<_>, _>>()
            .expect("girdi");
        girdiler.sort_by_key(|girdi| girdi.file_name());
        for girdi in girdiler {
            let yol = girdi.path();
            let goreli = yol
                .strip_prefix(kok)
                .expect("kök içinde")
                .to_string_lossy()
                .replace('\\', "/");
            if girdi.file_type().expect("tür").is_dir() {
                sonuc.push((format!("{goreli}/"), None));
                gez(kok, &yol, sonuc);
            } else {
                sonuc.push((goreli, Some(std::fs::read_to_string(&yol).expect("oku"))));
            }
        }
    }
    let mut sonuc = Vec::new();
    gez(kok, kok, &mut sonuc);
    sonuc
}

fn beklenen_agac() -> Vec<(String, Option<String>)> {
    vec![
        ("kaynak/".into(), None),
        ("kaynak/yardımcı.dil".into(), Some(GIRDILER[0].1.into())),
        ("paket.dil".into(), Some(GIRDILER[1].1.into())),
        ("proje.dil".into(), Some(GIRDILER[2].1.into())),
    ]
}

fn kardesler(hedef: &Path) -> Vec<String> {
    let mut adlar = std::fs::read_dir(hedef.parent().expect("üst"))
        .expect("üst klasör")
        .map(|girdi| {
            girdi
                .expect("girdi")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    adlar.sort();
    adlar
}

fn salt_okunur(kok: &Path) -> bool {
    GIRDILER.iter().all(|(yol, _)| {
        std::fs::metadata(
            yol.split('/')
                .fold(kok.to_path_buf(), |taban, parca| taban.join(parca)),
        )
        .expect("izin")
        .permissions()
        .readonly()
    })
}

/// Sekiz iş parçacığı aynı içerik adresine aynı anda kurar. Hepsi başarılı
/// döner; tek exact ağaç yayımlanır, her dosya salt okunurdur ve hiçbir
/// görünür ya da geçici kardeş kalmaz (kaybedenler kendi geçicisini siler).
#[test]
fn eszamanli_kurulum_tek_exact_agac_yayimlar_ve_kaybeden_kazanani_dogrular() {
    let gecici = Gecici::yeni();
    let arsiv = arsiv(&gecici, GIRDILER);
    let hedef = gecici.0.join("paketler").join("sha256").join("ozet");
    for tur in 0..4 {
        let engel = Arc::new(Barrier::new(8));
        let sonuclar = (0..8)
            .map(|_| {
                let (arsiv, hedef, engel) = (arsiv.clone(), hedef.clone(), engel.clone());
                std::thread::spawn(move || {
                    engel.wait();
                    paket_arsivini_kur(&arsiv, &hedef)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|is| is.join().expect("kurulum iş parçacığı"))
            .collect::<Vec<_>>();
        assert!(
            sonuclar.iter().all(Result::is_ok),
            "tur {tur}: yarış kaybeden kazananı kabul etmeli: {sonuclar:?}"
        );
        assert_eq!(agac(&hedef), beklenen_agac(), "tur {tur}");
        assert!(salt_okunur(&hedef), "tur {tur}");
        assert_eq!(kardesler(&hedef), vec!["ozet".to_string()], "tur {tur}");
        yazilabilir_yap(&hedef);
        std::fs::remove_dir_all(&hedef).expect("sonraki tur boş başlar");
    }
}

/// Çöken bir sürecin yarım geçicisi (görünmez `.zee-paket-gecici-*` kardeşi)
/// yayın adına hiç taşınmadığından kurulumu etkilemez ve kaynak olarak
/// okunmaz. Belgelenmiş sınır: yabancı geçici temizlenmez.
#[test]
fn yarim_kalan_gecici_kurulumu_etkilemez_ve_okunmaz() {
    let gecici = Gecici::yeni();
    let arsiv = arsiv(&gecici, GIRDILER);
    let ust = gecici.0.join("sha256");
    let yarim = ust.join(".zee-paket-gecici-999999-0");
    std::fs::create_dir_all(yarim.join("kaynak")).expect("yarım geçici");
    std::fs::write(yarim.join("paket.dil"), "işlem ya").expect("yarım dosya");
    let hedef = ust.join("ozet");
    paket_arsivini_kur(&arsiv, &hedef).expect("kurulum");
    assert_eq!(agac(&hedef), beklenen_agac());
    assert_eq!(
        kardesler(&hedef),
        vec![".zee-paket-gecici-999999-0".to_string(), "ozet".to_string()]
    );
    assert_eq!(
        std::fs::read_to_string(yarim.join("paket.dil")).expect("dokunulmadı"),
        "işlem ya"
    );
}

/// Yayın adında yarım ya da sonradan bozulmuş bir ağaç (eksik, kesik,
/// fazladan dosya/klasör) bir sonraki kurulumda arşivle yeniden doğrulanır;
/// fark tanıyla reddedilir ve ağaç sessizce onarılmaz.
#[test]
fn yarim_yayimlanmis_kurulum_sessizce_onarilmaz() {
    type Bozma = fn(&Path);
    let durumlar: [(&str, Bozma, &str); 5] = [
        (
            "eksik",
            |kok| std::fs::remove_file(kok.join("paket.dil")).expect("sil"),
            "arşiv byte'larıyla uyuşmuyor",
        ),
        (
            "kesik",
            |kok| std::fs::write(kok.join("paket.dil"), "işlem").expect("kes"),
            "boyutu değişmiş: paket.dil",
        ),
        (
            "ayni-boy",
            |kok| {
                let mut icerik = std::fs::read(kok.join("paket.dil")).expect("oku");
                icerik[0] ^= 1;
                std::fs::write(kok.join("paket.dil"), icerik).expect("değiştir");
            },
            "arşiv byte'larıyla uyuşmuyor",
        ),
        (
            "fazladan-dosya",
            |kok| std::fs::write(kok.join("ek.dil"), "x").expect("ekle"),
            "fazladan kaynak var: ek.dil",
        ),
        (
            "fazladan-klasor",
            |kok| std::fs::create_dir(kok.join("gizli")).expect("klasör"),
            "fazladan klasör var: gizli",
        ),
    ];
    for (ad, boz, beklenen) in durumlar {
        let gecici = Gecici::yeni();
        let arsiv = arsiv(&gecici, GIRDILER);
        let hedef = gecici.0.join("ozet");
        paket_arsivini_kur(&arsiv, &hedef).expect("ilk kurulum");
        yazilabilir_yap(&hedef);
        boz(&hedef);
        let once = agac(&hedef);
        let hata = paket_arsivini_kur(&arsiv, &hedef).expect_err(ad);
        assert!(hata.contains(beklenen), "{ad}: {hata}");
        assert_eq!(agac(&hedef), once, "{ad}: bozuk ağaç onarılmamalı");
    }
}

/// K-363: `A.dil`/`a.dil` ya da `K/`/`k/` taşıyan arşiv dosya sistemine
/// dokunmadan, her platformda aynı tanıyla reddedilir. Önceden Linux'ta
/// kuruluyor, macOS/Windows'ta ham `File exists` ya da "fazladan kaynak"
/// hatasıyla düşüyordu.
#[test]
fn harf_katlama_cakismasi_kurulumdan_once_her_platformda_reddedilir() {
    for (ad, girdiler, beklenen) in [
        (
            "dosya",
            &[("A.dil", "x"), ("a.dil", "y")][..],
            "\"A.dil\" ve \"a.dil\"",
        ),
        (
            "klasor",
            &[("K/a.dil", "x"), ("k/b.dil", "y")][..],
            "\"K\" ve \"k\"",
        ),
    ] {
        let gecici = Gecici::yeni();
        let arsiv = arsiv(&gecici, girdiler);
        let hedef = gecici.0.join("paketler").join("ozet");
        let hata = paket_arsivini_kur(&arsiv, &hedef).expect_err(ad);
        assert_eq!(
            hata,
            format!(
                "Paket yolları büyük/küçük harf duyarsız dosya sisteminde çakışıyor: {beklenen}."
            ),
            "{ad}"
        );
        assert!(
            !gecici.0.join("paketler").exists(),
            "{ad}: hiçbir şey yazılmamalı"
        );
    }
}
