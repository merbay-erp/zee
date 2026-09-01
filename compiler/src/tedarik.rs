//! zee paket tedarik zincirinin çevrimdışı, deterministik çekirdeği.
//!
//! Bir yayın dört değişmez parçadan oluşur: kaynak paketi (`.zep`), SPDX
//! SBOM, SLSA provenance ve bunların byte-byte özetlerini bağlayan Ed25519
//! imzalı yayın bildirimi. Registry ve aynalar bu dosyaları yalnız taşır;
//! güven kararı bu modülün doğrulamasından sonra verilir.

use crate::paket::sha256_hex;
use crate::proje::{bildirimi_oku, ProjeBildirimi};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const PAKET_SIHRI: &[u8; 8] = b"ZEEZEP\0\x01";
const IMZA_ALANI: &[u8] = b"zee-yayin-v1\0";
const ANAHTAR_BASLIGI: &str = "zee-ed25519-private-v1";
const AZAMI_PAKET_BOYUTU: usize = 64 * 1024 * 1024;
const AZAMI_DOSYA_BOYUTU: usize = 16 * 1024 * 1024;
const AZAMI_DOSYA_SAYISI: usize = 10_000;
const AZAMI_YOL_BOYUTU: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaketCiktilari {
    pub paket: PathBuf,
    pub sbom: PathBuf,
    pub provenance: PathBuf,
    pub yayin: PathBuf,
    pub paket_ozeti: String,
    pub yayinci_anahtar_kimligi: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YayinDosyasi {
    pub ad: String,
    pub boyut: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImzaliYayin {
    pub sema: String,
    pub paket: String,
    pub surum: String,
    pub morfoloji: String,
    pub yayinci_anahtari: String,
    pub yayinci_anahtar_kimligi: String,
    pub arsiv: YayinDosyasi,
    pub sbom: YayinDosyasi,
    pub provenance: YayinDosyasi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct YayinImzasi {
    anahtar_kimligi: String,
    ed25519: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct YayinZarfi {
    imzali: ImzaliYayin,
    imzalar: Vec<YayinImzasi>,
}

#[derive(Debug)]
struct KaynakGirdisi {
    yol: String,
    icerik: Vec<u8>,
}

/// Yeni bir Ed25519 yayıncı anahtarı üretir. Var olan hedefi hiçbir koşulda
/// ezmez. Unix'te dosya ilk byte yazılmadan önce yalnız sahibine (0600) açılır.
pub fn anahtar_uret(yol: &Path) -> Result<String, String> {
    let mut gizli = [0u8; 32];
    getrandom::fill(&mut gizli)
        .map_err(|hata| format!("İşletim sistemi rastgeleliği alınamadı: {}.", hata))?;
    let anahtar = SigningKey::from_bytes(&gizli);
    let acik = anahtar.verifying_key().to_bytes();
    let kimlik = anahtar_kimligi(&acik);
    let metin = format!(
        "{}\ngizli {}\naçık {}\nkimlik sha256:{}\n",
        ANAHTAR_BASLIGI,
        hex_yaz(&gizli),
        hex_yaz(&acik),
        kimlik
    );

    let mut secenekler = std::fs::OpenOptions::new();
    secenekler.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        secenekler.mode(0o600);
    }
    let mut dosya = secenekler.open(yol).map_err(|hata| {
        if hata.kind() == std::io::ErrorKind::AlreadyExists {
            format!(
                "\"{}\" zaten var; özel anahtarın üzerine yazılmadı.",
                yol.display()
            )
        } else {
            format!("\"{}\" anahtar dosyası açılamadı: {}.", yol.display(), hata)
        }
    })?;
    if let Err(hata) = dosya
        .write_all(metin.as_bytes())
        .and_then(|()| dosya.sync_all())
    {
        drop(dosya);
        let _ = std::fs::remove_file(yol);
        return Err(format!("Anahtar kalıcı yazılamadı: {}.", hata));
    }
    Ok(format!("sha256:{}", kimlik))
}

/// Geçerli saati kullanan kullanıcı yüzeyi. Tam tekrar üretilebilir bir yayın
/// için `SOURCE_DATE_EPOCH` tanımlanabilir.
pub fn paketle(kok: &Path, anahtar_yolu: &Path, cikti: &Path) -> Result<PaketCiktilari, String> {
    let saniye = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(deger) => deger.parse::<i64>().map_err(|_| {
            "SOURCE_DATE_EPOCH negatif olmayan bir Unix saniyesi olmalı.".to_string()
        })?,
        Err(std::env::VarError::NotPresent) => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Sistem saati Unix başlangıcından önce.".to_string())?
            .as_secs()
            .try_into()
            .map_err(|_| "Sistem saati desteklenen Unix saniyesini aşıyor.".to_string())?,
        Err(hata) => return Err(format!("SOURCE_DATE_EPOCH okunamadı: {}.", hata)),
    };
    if saniye < 0 {
        return Err("SOURCE_DATE_EPOCH negatif olamaz.".into());
    }
    paketle_zamanla(kok, anahtar_yolu, cikti, saniye)
}

/// Paketleyicinin zaman girdisi açık sürümü. Testler ve tekrar üretilebilir
/// derleme sistemleri aynı saniyeyi vererek dört çıktıyı da byte-byte yineler.
pub fn paketle_zamanla(
    kok: &Path,
    anahtar_yolu: &Path,
    cikti: &Path,
    uretim_saniyesi: i64,
) -> Result<PaketCiktilari, String> {
    if uretim_saniyesi < 0 {
        return Err("Üretim zamanı negatif olamaz.".into());
    }
    let kok =
        std::fs::canonicalize(kok).map_err(|hata| format!("Paket kökü çözülemedi: {}.", hata))?;
    if !kok.is_dir() {
        return Err(format!("\"{}\" bir proje klasörü değil.", kok.display()));
    }
    let bildirim_kaynagi = std::fs::read_to_string(kok.join("proje.dil"))
        .map_err(|hata| format!("proje.dil okunamadı: {}.", hata))?;
    let bildirim =
        bildirimi_oku(&bildirim_kaynagi).map_err(|tani| format!("{}: {}", tani.kod, tani.mesaj))?;
    if !bildirim.yerel_bagimliliklar.is_empty() {
        return Err(
            "Yerel yol bağımlılığı yayınlanamaz; yayın paketi taşınabilir ve bağımsız olmalı."
                .into(),
        );
    }

    let grafik = crate::paket::ProjeGrafigi::cozumle(&kok)
        .map_err(|hata| format!("{}: {}", hata.tani.kod, hata.tani.mesaj))?;
    grafik
        .kilidi_denetle()
        .map_err(|hata| format!("{}: {}", hata.tani.kod, hata.tani.mesaj))?;
    let giris = grafik.ana_giris();
    let mut yukleyici = |istek: crate::BirimIstegi<'_>| grafik.yukle(istek);
    crate::kaynagi_derle_kokenlerle(&giris.kaynak, Some(&giris.koken), &mut yukleyici)
        .map_err(|tani| format!("Paket kaynağı {}: {}", tani.kod, tani.mesaj))?;

    let anahtar = anahtari_oku(anahtar_yolu)?;
    let girdiler = kaynaklari_topla(&kok)?;
    let arsiv = arsivi_yaz(&girdiler)?;
    let arsiv_ozeti = sha256_hex(&arsiv);
    let zaman = utc_zamani(uretim_saniyesi);
    let taban = format!("{}-{}", bildirim.ad, bildirim.surum);
    let arsiv_adi = format!("{}.zep", taban);
    let sbom_adi = format!("{}.spdx.json", taban);
    let provenance_adi = format!("{}.intoto.json", taban);
    let yayin_adi = format!("{}.zee-yayin.json", taban);

    let sbom = sbom_uret(&bildirim, &arsiv_adi, &arsiv_ozeti, &zaman)?;
    let provenance = provenance_uret(&bildirim, &arsiv_adi, &arsiv_ozeti)?;
    let acik = anahtar.verifying_key().to_bytes();
    let kimlik = anahtar_kimligi(&acik);
    let imzali = ImzaliYayin {
        sema: "zee-yayin-v1".into(),
        paket: bildirim.ad.clone(),
        surum: bildirim.surum.clone(),
        morfoloji: bildirim.morfoloji.clone(),
        yayinci_anahtari: hex_yaz(&acik),
        yayinci_anahtar_kimligi: format!("sha256:{}", kimlik),
        arsiv: yayin_dosyasi(&arsiv_adi, &arsiv),
        sbom: yayin_dosyasi(&sbom_adi, &sbom),
        provenance: yayin_dosyasi(&provenance_adi, &provenance),
    };
    let imzalanacak = imza_girdisi(&imzali)?;
    let imza = anahtar.sign(&imzalanacak).to_bytes();
    let zarf = YayinZarfi {
        imzali,
        imzalar: vec![YayinImzasi {
            anahtar_kimligi: format!("sha256:{}", kimlik),
            ed25519: hex_yaz(&imza),
        }],
    };
    let mut yayin = serde_json::to_vec_pretty(&zarf)
        .map_err(|hata| format!("Yayın bildirimi üretilemedi: {}.", hata))?;
    yayin.push(b'\n');

    // Kendi ürettiğimiz zinciri diske dokunmadan önce aynı tüketici yoluyla
    // doğrulamak, üretici/doğrulayıcı sözleşme kaymasını yayın anında yakalar.
    yayini_dogrula(&yayin, &arsiv, &sbom, &provenance)?;

    std::fs::create_dir_all(cikti)
        .map_err(|hata| format!("Çıktı klasörü oluşturulamadı: {}.", hata))?;
    let arsiv_yolu = cikti.join(arsiv_adi);
    let sbom_yolu = cikti.join(sbom_adi);
    let provenance_yolu = cikti.join(provenance_adi);
    let yayin_yolu = cikti.join(yayin_adi);
    for (yol, icerik) in [
        (&arsiv_yolu, arsiv.as_slice()),
        (&sbom_yolu, sbom.as_slice()),
        (&provenance_yolu, provenance.as_slice()),
        (&yayin_yolu, yayin.as_slice()),
    ] {
        crate::kalici_dosya::atomik_yaz(yol, icerik)
            .map_err(|hata| format!("\"{}\" yazılamadı: {}.", yol.display(), hata))?;
    }

    Ok(PaketCiktilari {
        paket: arsiv_yolu,
        sbom: sbom_yolu,
        provenance: provenance_yolu,
        yayin: yayin_yolu,
        paket_ozeti: arsiv_ozeti,
        yayinci_anahtar_kimligi: format!("sha256:{}", kimlik),
    })
}

/// Yayıncı imzasını, eşlik eden üç dosyanın boyut/özetini ve metadata içindeki
/// temel çapraz bağları doğrular. Başarıdan önce hiçbir paket içeriği açılmaz.
pub fn yayini_dogrula(
    yayin: &[u8],
    arsiv: &[u8],
    sbom: &[u8],
    provenance: &[u8],
) -> Result<ImzaliYayin, String> {
    if yayin.len() > 1024 * 1024 {
        return Err("Yayın bildirimi 1 MiB sınırını aşıyor.".into());
    }
    let zarf: YayinZarfi = serde_json::from_slice(yayin)
        .map_err(|hata| format!("Yayın bildirimi geçerli/kapalı şema JSON değil: {}.", hata))?;
    if zarf.imzali.sema != "zee-yayin-v1" {
        return Err(format!(
            "Desteklenmeyen yayın şeması: {}.",
            zarf.imzali.sema
        ));
    }
    if zarf.imzalar.len() != 1 {
        return Err("zee-yayin-v1 tam olarak bir yayıncı imzası ister.".into());
    }
    let acik = hex_coz::<32>(&zarf.imzali.yayinci_anahtari, "yayıncı açık anahtarı")?;
    let kimlik = format!("sha256:{}", anahtar_kimligi(&acik));
    if zarf.imzali.yayinci_anahtar_kimligi != kimlik || zarf.imzalar[0].anahtar_kimligi != kimlik {
        return Err("Yayıncı anahtar kimliği açık anahtarın SHA-256 özetiyle uyuşmuyor.".into());
    }
    let anahtar = VerifyingKey::from_bytes(&acik)
        .map_err(|_| "Yayıncı Ed25519 açık anahtarı geçersiz.".to_string())?;
    let imza_baytlari = hex_coz::<64>(&zarf.imzalar[0].ed25519, "Ed25519 imzası")?;
    let imza = Signature::from_bytes(&imza_baytlari);
    anahtar
        .verify_strict(&imza_girdisi(&zarf.imzali)?, &imza)
        .map_err(|_| "Yayıncı Ed25519 imzası doğrulanamadı.".to_string())?;

    dosyayi_dogrula(&zarf.imzali.arsiv, arsiv, ".zep")?;
    dosyayi_dogrula(&zarf.imzali.sbom, sbom, ".spdx.json")?;
    dosyayi_dogrula(&zarf.imzali.provenance, provenance, ".intoto.json")?;
    let girdiler = arsivi_oku(arsiv)?;
    let bildirim_baytlari = girdiler
        .get("proje.dil")
        .ok_or_else(|| "Kaynak paketinde proje.dil yok.".to_string())?;
    let bildirim_kaynagi = std::str::from_utf8(bildirim_baytlari)
        .map_err(|_| "Paket içindeki proje.dil UTF-8 değil.".to_string())?;
    let bildirim = bildirimi_oku(bildirim_kaynagi)
        .map_err(|tani| format!("Paket proje.dil doğrulaması {}: {}", tani.kod, tani.mesaj))?;
    if bildirim.ad != zarf.imzali.paket
        || bildirim.surum != zarf.imzali.surum
        || bildirim.morfoloji != zarf.imzali.morfoloji
    {
        return Err("İmzalı yayın kimliği paket içindeki proje.dil ile uyuşmuyor.".into());
    }
    if !bildirim.yerel_bagimliliklar.is_empty() {
        return Err("Yayın paketi yerel yol bağımlılığı taşıyor.".into());
    }
    sbom_dogrula(sbom, &zarf.imzali, &zarf.imzali.arsiv.sha256)?;
    provenance_dogrula(provenance, &zarf.imzali, &zarf.imzali.arsiv.sha256)?;
    Ok(zarf.imzali)
}

fn anahtari_oku(yol: &Path) -> Result<SigningKey, String> {
    let metin = std::fs::read_to_string(yol)
        .map_err(|hata| format!("\"{}\" anahtarı okunamadı: {}.", yol.display(), hata))?;
    let satirlar = metin.lines().collect::<Vec<_>>();
    if satirlar.len() != 4 || satirlar[0] != ANAHTAR_BASLIGI {
        return Err("Özel anahtar dosyası zee-ed25519-private-v1 biçiminde değil.".into());
    }
    let gizli = satirlar[1]
        .strip_prefix("gizli ")
        .ok_or_else(|| "Anahtar dosyasında gizli alanı eksik.".to_string())?;
    let acik = satirlar[2]
        .strip_prefix("açık ")
        .ok_or_else(|| "Anahtar dosyasında açık alanı eksik.".to_string())?;
    let kimlik = satirlar[3]
        .strip_prefix("kimlik sha256:")
        .ok_or_else(|| "Anahtar dosyasında kimlik alanı eksik.".to_string())?;
    let gizli = hex_coz::<32>(gizli, "özel anahtar")?;
    let beklenen_acik = hex_coz::<32>(acik, "açık anahtar")?;
    let anahtar = SigningKey::from_bytes(&gizli);
    let uretilen_acik = anahtar.verifying_key().to_bytes();
    if uretilen_acik != beklenen_acik || anahtar_kimligi(&uretilen_acik) != kimlik {
        return Err("Özel anahtar dosyasının açık anahtarı veya kimliği uyuşmuyor.".into());
    }
    Ok(anahtar)
}

fn kaynaklari_topla(kok: &Path) -> Result<Vec<KaynakGirdisi>, String> {
    fn gez(kok: &Path, klasor: &Path, sonuc: &mut Vec<KaynakGirdisi>) -> Result<(), String> {
        let mut girdiler = std::fs::read_dir(klasor)
            .map_err(|hata| format!("\"{}\" listelenemedi: {}.", klasor.display(), hata))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|hata| hata.to_string())?;
        girdiler.sort_by_key(|girdi| girdi.file_name());
        for girdi in girdiler {
            let tur = girdi.file_type().map_err(|hata| hata.to_string())?;
            let yol = girdi.path();
            if tur.is_symlink() {
                return Err(format!(
                    "Yayın paketinde sembolik bağ kabul edilmez: {}.",
                    yol.display()
                ));
            }
            if tur.is_dir() {
                let ad = girdi.file_name();
                let ad = ad.to_string_lossy();
                if ad.starts_with('.') || matches!(ad.as_ref(), "target" | "hedef") {
                    continue;
                }
                gez(kok, &yol, sonuc)?;
            } else if tur.is_file()
                && yol.extension().and_then(|uzanti| uzanti.to_str()) == Some("dil")
            {
                let goreli = yol
                    .strip_prefix(kok)
                    .map_err(|_| "Paket kaynağı proje kökünün dışında.".to_string())?;
                let yol_metni = duz_guvenli_yol(goreli)?;
                if yol_metni.len() > AZAMI_YOL_BOYUTU {
                    return Err(format!(
                        "Paket yolu {} byte sınırını aşıyor.",
                        AZAMI_YOL_BOYUTU
                    ));
                }
                let icerik = std::fs::read(&yol)
                    .map_err(|hata| format!("\"{}\" okunamadı: {}.", yol.display(), hata))?;
                if icerik.len() > AZAMI_DOSYA_BOYUTU {
                    return Err(format!(
                        "\"{}\" {} MiB dosya sınırını aşıyor.",
                        yol.display(),
                        AZAMI_DOSYA_BOYUTU / 1024 / 1024
                    ));
                }
                std::str::from_utf8(&icerik)
                    .map_err(|_| format!("\"{}\" geçerli UTF-8 değil.", yol.display()))?;
                sonuc.push(KaynakGirdisi {
                    yol: yol_metni,
                    icerik,
                });
            }
        }
        Ok(())
    }

    let mut sonuc = Vec::new();
    gez(kok, kok, &mut sonuc)?;
    sonuc.sort_by(|a, b| a.yol.cmp(&b.yol));
    if sonuc.len() > AZAMI_DOSYA_SAYISI {
        return Err(format!(
            "Paket {} dosya sınırını aşıyor.",
            AZAMI_DOSYA_SAYISI
        ));
    }
    if !sonuc.iter().any(|girdi| girdi.yol == "proje.dil") {
        return Err("Paket kaynaklarında proje.dil yok.".into());
    }
    Ok(sonuc)
}

fn arsivi_yaz(girdiler: &[KaynakGirdisi]) -> Result<Vec<u8>, String> {
    let mut cikti = Vec::new();
    cikti.extend_from_slice(PAKET_SIHRI);
    cikti.extend_from_slice(&(girdiler.len() as u32).to_be_bytes());
    for girdi in girdiler {
        cikti.extend_from_slice(&(girdi.yol.len() as u32).to_be_bytes());
        cikti.extend_from_slice(girdi.yol.as_bytes());
        cikti.extend_from_slice(&(girdi.icerik.len() as u64).to_be_bytes());
        cikti.extend_from_slice(&girdi.icerik);
        if cikti.len() > AZAMI_PAKET_BOYUTU {
            return Err(format!(
                "Paket {} MiB sınırını aşıyor.",
                AZAMI_PAKET_BOYUTU / 1024 / 1024
            ));
        }
    }
    Ok(cikti)
}

fn arsivi_oku(arsiv: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if arsiv.len() > AZAMI_PAKET_BOYUTU {
        return Err(format!(
            "Paket {} MiB sınırını aşıyor.",
            AZAMI_PAKET_BOYUTU / 1024 / 1024
        ));
    }
    let mut okuyucu = ArsivOkuyucu::yeni(arsiv);
    if okuyucu.al(PAKET_SIHRI.len())? != PAKET_SIHRI {
        return Err("Paket zee .zep v1 sihrini taşımıyor.".into());
    }
    let sayi = okuyucu.u32()? as usize;
    if sayi > AZAMI_DOSYA_SAYISI {
        return Err(format!(
            "Paket {} dosya sınırını aşıyor.",
            AZAMI_DOSYA_SAYISI
        ));
    }
    let mut sonuc = BTreeMap::new();
    let mut onceki: Option<String> = None;
    for _ in 0..sayi {
        let yol_boyu = okuyucu.u32()? as usize;
        if yol_boyu == 0 || yol_boyu > AZAMI_YOL_BOYUTU {
            return Err("Paket girdisi yolu boş ya da sınırın dışında.".into());
        }
        let yol = std::str::from_utf8(okuyucu.al(yol_boyu)?)
            .map_err(|_| "Paket girdisi yolu UTF-8 değil.".to_string())?
            .to_string();
        guvenli_arsiv_yolu(&yol)?;
        if onceki.as_ref().is_some_and(|onceki| onceki >= &yol) {
            return Err("Paket girdileri tekil ve artan yol sırasında değil.".into());
        }
        let boyut = usize::try_from(okuyucu.u64()?)
            .map_err(|_| "Paket girdisi boyutu bu platformda temsil edilemiyor.".to_string())?;
        if boyut > AZAMI_DOSYA_BOYUTU {
            return Err("Paket girdisi dosya sınırını aşıyor.".into());
        }
        let icerik = okuyucu.al(boyut)?.to_vec();
        std::str::from_utf8(&icerik)
            .map_err(|_| format!("Paket girdisi \"{}\" UTF-8 değil.", yol))?;
        onceki = Some(yol.clone());
        sonuc.insert(yol, icerik);
    }
    if !okuyucu.bitti() {
        return Err("Paket son girdiden sonra fazladan byte taşıyor.".into());
    }
    Ok(sonuc)
}

struct ArsivOkuyucu<'a> {
    veri: &'a [u8],
    konum: usize,
}

impl<'a> ArsivOkuyucu<'a> {
    fn yeni(veri: &'a [u8]) -> Self {
        Self { veri, konum: 0 }
    }

    fn al(&mut self, boyut: usize) -> Result<&'a [u8], String> {
        let son = self
            .konum
            .checked_add(boyut)
            .filter(|son| *son <= self.veri.len())
            .ok_or_else(|| "Paket beklenmedik yerde bitti.".to_string())?;
        let parca = &self.veri[self.konum..son];
        self.konum = son;
        Ok(parca)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_be_bytes(
            self.al(4)?.try_into().expect("uzunluk denetlendi"),
        ))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_be_bytes(
            self.al(8)?.try_into().expect("uzunluk denetlendi"),
        ))
    }

    fn bitti(&self) -> bool {
        self.konum == self.veri.len()
    }
}

fn duz_guvenli_yol(yol: &Path) -> Result<String, String> {
    let mut parcalar = Vec::new();
    for parca in yol.components() {
        let Component::Normal(ad) = parca else {
            return Err("Paket yolu yalnız normal göreli bileşenler taşımalı.".into());
        };
        let ad = ad
            .to_str()
            .ok_or_else(|| "Paket yolu UTF-8 olmalı.".to_string())?;
        if ad.is_empty() || ad.chars().any(char::is_control) {
            return Err("Paket yolu boş/denetim karakterli bileşen taşıyor.".into());
        }
        parcalar.push(ad);
    }
    let sonuc = parcalar.join("/");
    guvenli_arsiv_yolu(&sonuc)?;
    Ok(sonuc)
}

fn guvenli_arsiv_yolu(yol: &str) -> Result<(), String> {
    if yol.is_empty()
        || yol.starts_with('/')
        || yol.contains('\\')
        || yol.contains(':')
        || yol.contains('\0')
        || yol.chars().any(char::is_control)
        || yol
            .split('/')
            .any(|parca| parca.is_empty() || matches!(parca, "." | ".."))
    {
        return Err(format!("Güvensiz paket yolu: {:?}.", yol));
    }
    Ok(())
}

fn sbom_uret(
    bildirim: &ProjeBildirimi,
    arsiv_adi: &str,
    arsiv_ozeti: &str,
    zaman: &str,
) -> Result<Vec<u8>, String> {
    let taban = format!(
        "https://zee-lang.dev/spdx/{}/{}/{}",
        bildirim.ad, bildirim.surum, arsiv_ozeti
    );
    let belge = json!({
        "@context": "https://spdx.org/rdf/3.0.1/spdx-context.jsonld",
        "@graph": [
            {
                "type": "CreationInfo",
                "@id": "_:creationinfo",
                "specVersion": "3.0.1",
                "createdBy": [format!("{}/agent/dil", taban)],
                "createdUsing": [format!("{}/tool/dil", taban)],
                "created": zaman
            },
            {
                "type": "Tool",
                "spdxId": format!("{}/tool/dil", taban),
                "creationInfo": "_:creationinfo",
                "name": "dil",
                "comment": format!("zee bootstrap paketleyicisi {}", env!("CARGO_PKG_VERSION"))
            },
            {
                "type": "Organization",
                "spdxId": format!("{}/agent/dil", taban),
                "creationInfo": "_:creationinfo",
                "name": "zee package publisher"
            },
            {
                "type": "SpdxDocument",
                "spdxId": format!("{}/document", taban),
                "creationInfo": "_:creationinfo",
                "profileConformance": ["core", "software"],
                "rootElement": [format!("{}/sbom", taban)],
                "element": [format!("{}/sbom", taban), format!("{}/package", taban)]
            },
            {
                "type": "software_Sbom",
                "spdxId": format!("{}/sbom", taban),
                "creationInfo": "_:creationinfo",
                "profileConformance": ["core", "software"],
                "rootElement": [format!("{}/package", taban)],
                "element": [format!("{}/package", taban)],
                "software_sbomType": ["build"]
            },
            {
                "type": "software_Package",
                "spdxId": format!("{}/package", taban),
                "creationInfo": "_:creationinfo",
                "name": bildirim.ad,
                "software_packageVersion": bildirim.surum,
                "software_downloadLocation": format!("urn:zee-package:{}", arsiv_adi),
                "software_copyrightText": "NOASSERTION",
                "suppliedBy": format!("{}/agent/dil", taban),
                "verifiedUsing": [{"type": "Hash", "algorithm": "sha256", "hashValue": arsiv_ozeti}]
            }
        ]
    });
    json_satirli(&belge, "SPDX SBOM")
}

fn provenance_uret(
    bildirim: &ProjeBildirimi,
    arsiv_adi: &str,
    arsiv_ozeti: &str,
) -> Result<Vec<u8>, String> {
    let belge = json!({
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"name": arsiv_adi, "digest": {"sha256": arsiv_ozeti}}],
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {
            "buildDefinition": {
                "buildType": "https://zee-lang.dev/buildtypes/source-package/v1",
                "externalParameters": {
                    "package": bildirim.ad,
                    "version": bildirim.surum,
                    "morphology": bildirim.morfoloji
                },
                "internalParameters": {},
                "resolvedDependencies": []
            },
            "runDetails": {
                "builder": {
                    "id": "https://zee-lang.dev/builders/dil/local-source-package/v1",
                    "version": {"dil": env!("CARGO_PKG_VERSION")}
                },
                "metadata": {}
            }
        }
    });
    json_satirli(&belge, "SLSA provenance")
}

fn json_satirli(deger: &Value, ad: &str) -> Result<Vec<u8>, String> {
    let mut sonuc = serde_json::to_vec_pretty(deger)
        .map_err(|hata| format!("{} üretilemedi: {}.", ad, hata))?;
    sonuc.push(b'\n');
    Ok(sonuc)
}

fn json_kanonik_dogrula(ham: &[u8], deger: &Value, ad: &str) -> Result<(), String> {
    if ham != json_satirli(deger, ad)? {
        return Err(format!(
            "{} zee'nin deterministik girintili JSON biçiminde değil.",
            ad
        ));
    }
    Ok(())
}

fn sbom_dogrula(sbom: &[u8], yayin: &ImzaliYayin, arsiv_ozeti: &str) -> Result<(), String> {
    let deger: Value = serde_json::from_slice(sbom)
        .map_err(|hata| format!("SPDX SBOM JSON olarak çözülemedi: {}.", hata))?;
    json_kanonik_dogrula(sbom, &deger, "SPDX SBOM")?;
    if deger.get("@context").and_then(Value::as_str)
        != Some("https://spdx.org/rdf/3.0.1/spdx-context.jsonld")
    {
        return Err("SBOM SPDX 3.0.1 JSON-LD bağlamını taşımıyor.".into());
    }
    let paket = deger
        .get("@graph")
        .and_then(Value::as_array)
        .and_then(|graf| {
            graf.iter()
                .find(|oge| oge.get("type").and_then(Value::as_str) == Some("software_Package"))
        })
        .ok_or_else(|| "SBOM software_Package öğesi taşımıyor.".to_string())?;
    if paket.get("name").and_then(Value::as_str) != Some(&yayin.paket)
        || paket.get("software_packageVersion").and_then(Value::as_str) != Some(&yayin.surum)
    {
        return Err("SBOM paket kimliği imzalı yayınla uyuşmuyor.".into());
    }
    let ozet_var = paket
        .get("verifiedUsing")
        .and_then(Value::as_array)
        .is_some_and(|ozetler| {
            ozetler.iter().any(|ozet| {
                ozet.get("algorithm").and_then(Value::as_str) == Some("sha256")
                    && ozet.get("hashValue").and_then(Value::as_str) == Some(arsiv_ozeti)
            })
        });
    if !ozet_var {
        return Err("SBOM yayın paketinin SHA-256 özetini taşımıyor.".into());
    }
    Ok(())
}

fn provenance_dogrula(
    provenance: &[u8],
    yayin: &ImzaliYayin,
    arsiv_ozeti: &str,
) -> Result<(), String> {
    let deger: Value = serde_json::from_slice(provenance)
        .map_err(|hata| format!("SLSA provenance JSON olarak çözülemedi: {}.", hata))?;
    json_kanonik_dogrula(provenance, &deger, "SLSA provenance")?;
    if deger.get("_type").and_then(Value::as_str) != Some("https://in-toto.io/Statement/v1")
        || deger.get("predicateType").and_then(Value::as_str)
            != Some("https://slsa.dev/provenance/v1")
    {
        return Err("Provenance in-toto Statement v1 / SLSA provenance v1 değil.".into());
    }
    let konu = deger
        .get("subject")
        .and_then(Value::as_array)
        .and_then(|konular| konular.first())
        .ok_or_else(|| "Provenance subject taşımıyor.".to_string())?;
    if konu.get("name").and_then(Value::as_str) != Some(&yayin.arsiv.ad)
        || konu
            .get("digest")
            .and_then(|d| d.get("sha256"))
            .and_then(Value::as_str)
            != Some(arsiv_ozeti)
    {
        return Err("Provenance konusu yayın paketinin adı/özetiyle uyuşmuyor.".into());
    }
    Ok(())
}

fn yayin_dosyasi(ad: &str, icerik: &[u8]) -> YayinDosyasi {
    YayinDosyasi {
        ad: ad.to_string(),
        boyut: icerik.len() as u64,
        sha256: sha256_hex(icerik),
    }
}

fn dosyayi_dogrula(bilgi: &YayinDosyasi, icerik: &[u8], uzanti: &str) -> Result<(), String> {
    if bilgi.ad.contains(['/', '\\', ':'])
        || bilgi.ad.chars().any(char::is_control)
        || !bilgi.ad.ends_with(uzanti)
    {
        return Err(format!(
            "İmzalı yayın dosya adı güvenli {} adı değil.",
            uzanti
        ));
    }
    if bilgi.boyut != icerik.len() as u64 || bilgi.sha256 != sha256_hex(icerik) {
        return Err(format!(
            "{} boyutu veya SHA-256 özeti imzalı yayınla uyuşmuyor.",
            bilgi.ad
        ));
    }
    Ok(())
}

fn imza_girdisi(imzali: &ImzaliYayin) -> Result<Vec<u8>, String> {
    let mut sonuc = IMZA_ALANI.to_vec();
    sonuc.extend_from_slice(
        &serde_json::to_vec(imzali)
            .map_err(|hata| format!("İmzalanacak yayın kanonikleştirilemedi: {}.", hata))?,
    );
    Ok(sonuc)
}

fn anahtar_kimligi(acik: &[u8; 32]) -> String {
    sha256_hex(acik)
}

fn hex_yaz(baytlar: &[u8]) -> String {
    const BASAMAK: &[u8; 16] = b"0123456789abcdef";
    let mut sonuc = String::with_capacity(baytlar.len() * 2);
    for bayt in baytlar {
        sonuc.push(BASAMAK[(bayt >> 4) as usize] as char);
        sonuc.push(BASAMAK[(bayt & 0x0f) as usize] as char);
    }
    sonuc
}

fn hex_coz<const N: usize>(metin: &str, ad: &str) -> Result<[u8; N], String> {
    if metin.len() != N * 2
        || !metin
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(format!("{} tam {} byte küçük harfli hex olmalı.", ad, N));
    }
    let mut sonuc = [0u8; N];
    for (i, hedef) in sonuc.iter_mut().enumerate() {
        let basamak = |b: u8| {
            if b.is_ascii_digit() {
                b - b'0'
            } else {
                b - b'a' + 10
            }
        };
        *hedef = (basamak(metin.as_bytes()[i * 2]) << 4) | basamak(metin.as_bytes()[i * 2 + 1]);
    }
    Ok(sonuc)
}

fn utc_zamani(saniye: i64) -> String {
    let gunler = saniye.div_euclid(86_400);
    let gun_ici = saniye.rem_euclid(86_400);
    let (yil, ay, gun) = crate::yorumlayici::gunlerden_tarih_utc(gunler);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        yil,
        ay,
        gun,
        gun_ici / 3_600,
        (gun_ici % 3_600) / 60,
        gun_ici % 60
    )
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn bozuk_arsiv_yolu_ve_fazladan_byte_reddedilir() {
        let girdi = KaynakGirdisi {
            yol: "../kaçış.dil".into(),
            icerik: b"x".to_vec(),
        };
        let arsiv = arsivi_yaz(&[girdi]).expect("ham arşiv yazılır");
        assert!(arsivi_oku(&arsiv).unwrap_err().contains("Güvensiz"));

        let mut bos = arsivi_yaz(&[]).expect("boş arşiv");
        bos.push(0);
        assert!(arsivi_oku(&bos).unwrap_err().contains("fazladan"));
    }

    #[test]
    fn arsiv_sayi_yol_ve_tekillik_sinirlarini_ayirmadan_denetler() {
        let mut cok_dosya = PAKET_SIHRI.to_vec();
        cok_dosya.extend_from_slice(&((AZAMI_DOSYA_SAYISI + 1) as u32).to_be_bytes());
        assert!(arsivi_oku(&cok_dosya)
            .unwrap_err()
            .contains("dosya sınırını"));

        let mut uzun_yol = PAKET_SIHRI.to_vec();
        uzun_yol.extend_from_slice(&1u32.to_be_bytes());
        uzun_yol.extend_from_slice(&((AZAMI_YOL_BOYUTU + 1) as u32).to_be_bytes());
        assert!(arsivi_oku(&uzun_yol).unwrap_err().contains("yolu"));

        let yinelenen = arsivi_yaz(&[
            KaynakGirdisi {
                yol: "aynı.dil".into(),
                icerik: Vec::new(),
            },
            KaynakGirdisi {
                yol: "aynı.dil".into(),
                icerik: Vec::new(),
            },
        ])
        .expect("ham arşiv");
        assert!(arsivi_oku(&yinelenen).unwrap_err().contains("tekil"));
    }

    #[test]
    fn sha_kimligi_ve_utc_zamani_kararlidir() {
        assert_eq!(
            anahtar_kimligi(&[0; 32]),
            "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
        );
        assert_eq!(utc_zamani(0), "1970-01-01T00:00:00Z");
    }
}
