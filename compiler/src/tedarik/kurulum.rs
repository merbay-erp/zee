//! Doğrulanmış `.zep` arşivini görünmez geçicide açıp immutable kaynak kökü
//! olarak atomik yayımlayan kurulum katmanı.

use super::{AZAMI_PAKET_BOYUTU, arsivi_oku};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static GECICI_SAYACI: AtomicU64 = AtomicU64::new(0);

pub fn paket_arsivini_kur(arsiv_yolu: &Path, hedef: &Path) -> Result<(), String> {
    let arsiv = sinirli_oku(arsiv_yolu)?;
    let girdiler = arsivi_oku(&arsiv)?;
    if hedef.exists() {
        kurulumu_dogrula(hedef, &girdiler)?;
        return salt_okunur_yap(hedef, &girdiler);
    }
    let ebeveyn = hedef
        .parent()
        .ok_or_else(|| "Paket kurulum hedefinin üst klasörü yok.".to_string())?;
    std::fs::create_dir_all(ebeveyn)
        .map_err(|hata| format!("Paket cache klasörü oluşturulamadı: {}.", hata))?;
    let gecici = gecici_klasor(ebeveyn)?;
    let sonuc = (|| {
        for (yol, icerik) in &girdiler {
            let hedef_yol = yol
                .split('/')
                .fold(gecici.clone(), |taban, parca| taban.join(parca));
            let ust = hedef_yol
                .parent()
                .ok_or_else(|| "Paket girdisinin üst klasörü yok.".to_string())?;
            std::fs::create_dir_all(ust)
                .map_err(|hata| format!("Paket kaynak klasörü oluşturulamadı: {}.", hata))?;
            let mut secenekler = std::fs::OpenOptions::new();
            secenekler.create_new(true).write(true);
            let mut dosya = secenekler
                .open(&hedef_yol)
                .map_err(|hata| format!("Paket kaynağı oluşturulamadı: {}.", hata))?;
            std::io::Write::write_all(&mut dosya, icerik)
                .and_then(|()| dosya.sync_all())
                .map_err(|hata| format!("Paket kaynağı kalıcı yazılamadı: {}.", hata))?;
        }
        std::fs::rename(&gecici, hedef).map_err(|hata| {
            if hedef.exists() {
                "Paket kurulumu başka bir süreççe yayımlandı.".to_string()
            } else {
                format!("Paket kurulum klasörü atomik yayımlanamadı: {}.", hata)
            }
        })?;
        kurulumu_dogrula(hedef, &girdiler)?;
        salt_okunur_yap(hedef, &girdiler)
    })();
    if gecici.exists() {
        let _ = std::fs::remove_dir_all(&gecici);
    }
    match sonuc {
        Err(mesaj) if hedef.exists() && mesaj.contains("başka bir süreççe") => {
            kurulumu_dogrula(hedef, &girdiler)?;
            salt_okunur_yap(hedef, &girdiler)
        }
        sonuc => sonuc,
    }
}

fn sinirli_oku(yol: &Path) -> Result<Vec<u8>, String> {
    let dosya = std::fs::File::open(yol)
        .map_err(|hata| format!("Doğrulanmış paket arşivi açılamadı: {}.", hata))?;
    let boyut = dosya
        .metadata()
        .map_err(|hata| format!("Paket arşivi boyutu okunamadı: {}.", hata))?
        .len();
    if boyut > AZAMI_PAKET_BOYUTU as u64 {
        return Err("Paket arşivi kurulum sınırını aşıyor.".into());
    }
    let mut baytlar = Vec::with_capacity(boyut as usize);
    dosya
        .take(AZAMI_PAKET_BOYUTU.saturating_add(1) as u64)
        .read_to_end(&mut baytlar)
        .map_err(|hata| format!("Paket arşivi okunamadı: {}.", hata))?;
    if baytlar.len() > AZAMI_PAKET_BOYUTU {
        return Err("Paket arşivi okunurken kurulum sınırını aştı.".into());
    }
    Ok(baytlar)
}

fn gecici_klasor(ebeveyn: &Path) -> Result<PathBuf, String> {
    for _ in 0..32 {
        let sira = GECICI_SAYACI.fetch_add(1, Ordering::Relaxed);
        let aday = ebeveyn.join(format!(".zee-paket-gecici-{}-{}", std::process::id(), sira));
        match std::fs::create_dir(&aday) {
            Ok(()) => return Ok(aday),
            Err(hata) if hata.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(hata) => return Err(format!("Geçici paket klasörü açılamadı: {}.", hata)),
        }
    }
    Err("Benzersiz geçici paket klasörü oluşturulamadı.".into())
}

fn kurulumu_dogrula(kok: &Path, beklenen: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    let mut bulunan = BTreeMap::new();
    kurulum_dosyalarini_oku(kok, kok, beklenen, &mut bulunan)?;
    if &bulunan != beklenen {
        return Err("İçerik-adresli paket kurulumu arşiv byte'larıyla uyuşmuyor.".into());
    }
    Ok(())
}

fn kurulum_dosyalarini_oku(
    kok: &Path,
    klasor: &Path,
    beklenenler: &BTreeMap<String, Vec<u8>>,
    sonuc: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    let mut girdiler = std::fs::read_dir(klasor)
        .map_err(|hata| format!("Paket kurulumu listelenemedi: {}.", hata))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|hata| format!("Paket kurulum girdisi okunamadı: {}.", hata))?;
    girdiler.sort_by_key(|girdi| girdi.file_name());
    for girdi in girdiler {
        let tur = girdi
            .file_type()
            .map_err(|hata| format!("Paket kurulum girdi türü okunamadı: {}.", hata))?;
        if tur.is_symlink() {
            return Err("Paket kurulumunda sembolik bağ bulundu.".into());
        }
        let yol = girdi.path();
        if tur.is_dir() {
            let goreli = yol
                .strip_prefix(kok)
                .map_err(|_| "Paket kurulum klasörü kökün dışında.".to_string())?;
            let onek = format!(
                "{}/",
                goreli
                    .components()
                    .map(|parca| parca.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/")
            );
            if !beklenenler
                .keys()
                .any(|beklenen| beklenen.starts_with(&onek))
            {
                return Err(format!(
                    "Paket kurulumunda fazladan klasör var: {}.",
                    goreli.display()
                ));
            }
            kurulum_dosyalarini_oku(kok, &yol, beklenenler, sonuc)?;
        } else if tur.is_file() {
            let goreli = yol
                .strip_prefix(kok)
                .map_err(|_| "Paket kurulum yolu kökün dışında.".to_string())?;
            let ad = goreli
                .components()
                .map(|parca| parca.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let beklenen = beklenenler
                .get(&ad)
                .ok_or_else(|| format!("Paket kurulumunda fazladan kaynak var: {}.", ad))?;
            let dosya = std::fs::File::open(&yol)
                .map_err(|hata| format!("Paket kurulum kaynağı açılamadı: {}.", hata))?;
            if dosya
                .metadata()
                .map_err(|hata| format!("Paket kurulum boyutu okunamadı: {}.", hata))?
                .len()
                != beklenen.len() as u64
            {
                return Err(format!("Paket kurulum kaynağı boyutu değişmiş: {}.", ad));
            }
            let mut icerik = Vec::with_capacity(beklenen.len());
            dosya
                .take(beklenen.len().saturating_add(1) as u64)
                .read_to_end(&mut icerik)
                .map_err(|hata| format!("Paket kurulum kaynağı okunamadı: {}.", hata))?;
            sonuc.insert(ad, icerik);
        } else {
            return Err("Paket kurulumunda normal dosya olmayan girdi bulundu.".into());
        }
    }
    Ok(())
}

fn salt_okunur_yap(kok: &Path, girdiler: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    for yol in girdiler.keys() {
        let dosya = yol
            .split('/')
            .fold(kok.to_path_buf(), |taban, parca| taban.join(parca));
        let mut izin = std::fs::metadata(&dosya)
            .map_err(|hata| format!("Paket kaynak izni okunamadı: {}.", hata))?
            .permissions();
        izin.set_readonly(true);
        std::fs::set_permissions(&dosya, izin)
            .map_err(|hata| format!("Paket kaynağı salt okunur yapılamadı: {}.", hata))?;
    }
    Ok(())
}
