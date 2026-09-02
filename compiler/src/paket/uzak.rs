//! Exact registry bağımlılıklarını doğrulanmış cache'den kaynak köklerine
//! çevirir. Normal derleme yalnız çevrimdışı; ağ ancak açık CLI güncellemesidir.

use super::{proje_hatasi, ProjeYuklemeHatasi};
use crate::proje::{bildirimi_oku, RegistryBildirimi, UzakBagimlilik};
use crate::registry::{
    HedefPolitikasi, HttpsRegistryTasiyici, MetadataSurumleri, RegistryIstemcisi,
    RegistryPaketCiktisi,
};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};

mod politika;

use politika::{
    kilit_politikalarini_oku, kritik_politika_anahtari, kritik_politika_on_eki, politika_anahtari,
    yanked_politika_anahtari,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryCozumPolitikasi {
    pub cevrimdisi: bool,
    pub yanked_kabul_gerekcesi: Option<String>,
    pub kritik_duyuru_kabul_gerekcesi: Option<String>,
}

impl Default for RegistryCozumPolitikasi {
    fn default() -> Self {
        Self {
            cevrimdisi: true,
            yanked_kabul_gerekcesi: None,
            kritik_duyuru_kabul_gerekcesi: None,
        }
    }
}

impl RegistryCozumPolitikasi {
    pub fn yeni(
        cevrimdisi: bool,
        yanked_kabul_gerekcesi: Option<String>,
        kritik_duyuru_kabul_gerekcesi: Option<String>,
    ) -> Result<Self, String> {
        for (ad, gerekce) in [
            ("yanked kabul", yanked_kabul_gerekcesi.as_deref()),
            (
                "kritik duyuru kabul",
                kritik_duyuru_kabul_gerekcesi.as_deref(),
            ),
        ] {
            if gerekce.is_some_and(|gerekce| {
                gerekce.trim().is_empty()
                    || gerekce.len() > 1024
                    || gerekce.chars().any(char::is_control)
            }) {
                return Err(format!(
                    "{} gerekçesi boş, denetim karakterli veya 1024 byte'tan uzun olamaz.",
                    ad
                ));
            }
        }
        Ok(Self {
            cevrimdisi,
            yanked_kabul_gerekcesi,
            kritik_duyuru_kabul_gerekcesi,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct UzakPaketKimligi {
    registry: RegistryBildirimi,
    bagimlilik: UzakBagimlilik,
}

#[derive(Debug, Clone)]
pub(super) struct UzakPaketCozumu {
    pub kok: PathBuf,
    pub kilit: UzakPaketKilidi,
}

#[derive(Debug, Clone)]
pub(super) struct UzakPaketKilidi {
    pub registry_kok_surumu: u64,
    pub registry_kok_sha256: String,
    pub metadata: MetadataSurumleri,
    pub yayinci_anahtar_kimligi: String,
    pub arsiv_sha256: String,
    pub sbom_sha256: String,
    pub provenance_sha256: String,
    pub yayin_sha256: String,
    pub yanked: bool,
    pub kritik_duyurular: Vec<String>,
    pub yanked_kabul_gerekcesi: Option<String>,
    pub kritik_duyuru_kabul_gerekcesi: Option<String>,
    pub yanked_politika_anahtari: String,
    pub kritik_politika_anahtari: String,
}

pub(super) type UzakPaketCozumleri = BTreeMap<UzakPaketKimligi, UzakPaketCozumu>;

impl UzakPaketKimligi {
    pub(super) fn yeni(registry: &RegistryBildirimi, bagimlilik: &UzakBagimlilik) -> Self {
        Self {
            registry: registry.clone(),
            bagimlilik: bagimlilik.clone(),
        }
    }
}

pub(super) fn paketleri_hazirla(
    ana_kok: &Path,
    ana_bildirim_kaynagi: Option<&str>,
    politika: &RegistryCozumPolitikasi,
) -> Result<UzakPaketCozumleri, ProjeYuklemeHatasi> {
    let ana_bildirim_yolu = ana_kok.join("proje.dil");
    let ana_bildirim_kaynagi = match ana_bildirim_kaynagi {
        Some(kaynak) => kaynak.to_string(),
        None => {
            crate::kaynak_sinirlari::kaynak_dosyasi_oku(&ana_bildirim_yolu).map_err(|hata| {
                proje_hatasi(
                    "P006",
                    &format!("Proje bildirimi okunamadı: {}.", hata),
                    "Var olan ve okunabilen bir proje.dil kullan.",
                    ana_bildirim_yolu.clone(),
                    String::new(),
                )
            })?
        }
    };
    let ana_bildirim = bildirimi_oku(&ana_bildirim_kaynagi).map_err(|tani| ProjeYuklemeHatasi {
        tani: Box::new(tani),
        kaynak: ana_bildirim_kaynagi.clone(),
        yol: ana_bildirim_yolu.clone(),
    })?;
    if ana_bildirim.uzak_bagimliliklar.is_empty() {
        return Ok(BTreeMap::new());
    }

    let kilit_politikalari = kilit_politikalarini_oku(ana_kok, &ana_bildirim_kaynagi)?;
    let mut kuyruk = VecDeque::new();
    bagimliliklari_kuyruga_ekle_kokenli(
        &ana_bildirim,
        &ana_bildirim_yolu,
        &ana_bildirim_kaynagi,
        &mut kuyruk,
    );
    let mut cozumler = BTreeMap::new();
    while let Some((registry, bagimlilik, bildirim_yolu, bildirim_kaynagi)) = kuyruk.pop_front() {
        let kimlik = UzakPaketKimligi::yeni(&registry, &bagimlilik);
        if cozumler.contains_key(&kimlik) {
            continue;
        }
        let politika_anahtari = politika_anahtari(&registry, &bagimlilik);
        let yanked_politika_anahtari = yanked_politika_anahtari(&politika_anahtari);
        let onceki_yanked = kilit_politikalari
            .get(&yanked_politika_anahtari)
            .cloned()
            .unwrap_or_default();
        let yanked_gerekcesi = politika
            .yanked_kabul_gerekcesi
            .clone()
            .or(onceki_yanked.yanked);
        let kritik_onceki_kabul = kilit_politikalari
            .range(kritik_politika_on_eki(&politika_anahtari)..)
            .take_while(|(anahtar, _)| {
                anahtar.starts_with(&kritik_politika_on_eki(&politika_anahtari))
            })
            .any(|(_, gerekceler)| gerekceler.kritik.is_some());
        let hedef_politikasi = HedefPolitikasi {
            yanked_kabul: yanked_gerekcesi.is_some(),
            kritik_duyuru_kabul: politika.kritik_duyuru_kabul_gerekcesi.is_some()
                || kritik_onceki_kabul,
        };
        let cikti = registryden_al(
            ana_kok,
            &registry,
            &bagimlilik,
            politika.cevrimdisi,
            hedef_politikasi,
        )
        .map_err(|hata| {
            proje_hatasi(
                hata.kod,
                &hata.mesaj,
                "Registry pinini, exact sürümü ve doğrulanmış cache'i denetle.",
                bildirim_yolu.clone(),
                bildirim_kaynagi.clone(),
            )
        })?;
        let kritik_politika_anahtari =
            kritik_politika_anahtari(&politika_anahtari, &cikti.kritik_duyurular);
        let onceki_kritik = kilit_politikalari
            .get(&kritik_politika_anahtari)
            .cloned()
            .unwrap_or_default();
        let kritik_gerekcesi = politika
            .kritik_duyuru_kabul_gerekcesi
            .clone()
            .or(onceki_kritik.kritik);
        if !cikti.kritik_duyurular.is_empty() && kritik_gerekcesi.is_none() {
            return Err(proje_hatasi(
                "P014",
                "Etkin kritik duyuru kümesi önceki kabul kaydından farklı; yeni açık gerekçe gerekli.",
                "Güncel duyuruları incele ve yalnız bilinçli kabulde --kritik-kabul gerekçesi ver.",
                bildirim_yolu,
                bildirim_kaynagi,
            ));
        }
        let paket_koku = paket_koku(ana_kok, &cikti.arsiv_sha256).map_err(|mesaj| {
            proje_hatasi(
                "P016",
                &mesaj,
                "Bozuk içerik-adresli paket kurulumunu kullanma.",
                bildirim_yolu.clone(),
                bildirim_kaynagi.clone(),
            )
        })?;
        crate::tedarik::paket_arsivini_kur(&cikti.arsiv, &paket_koku).map_err(|mesaj| {
            proje_hatasi(
                "P016",
                &format!("Doğrulanmış paket kaynak cache'ine kurulamadı: {}", mesaj),
                "Cache izinlerini denetle; bozuk kurulumu kullanma.",
                bildirim_yolu.clone(),
                bildirim_kaynagi.clone(),
            )
        })?;
        let paket_bildirim_yolu = paket_koku.join("proje.dil");
        let paket_bildirim_kaynagi =
            crate::kaynak_sinirlari::kaynak_dosyasi_oku(&paket_bildirim_yolu).map_err(|hata| {
                proje_hatasi(
                    "P016",
                    &format!("Kurulu paket bildirimi okunamadı: {}.", hata),
                    "İçerik-adresli kurulumu doğrulanmış arşivden yeniden oluştur.",
                    paket_bildirim_yolu.clone(),
                    String::new(),
                )
            })?;
        let paket_bildirimi =
            bildirimi_oku(&paket_bildirim_kaynagi).map_err(|tani| ProjeYuklemeHatasi {
                tani: Box::new(tani),
                kaynak: paket_bildirim_kaynagi.clone(),
                yol: paket_bildirim_yolu.clone(),
            })?;
        if paket_bildirimi.ad != bagimlilik.ad || paket_bildirimi.surum != bagimlilik.surum {
            return Err(proje_hatasi(
                "P014",
                "Kurulu paketin proje kimliği exact registry isteğiyle uyuşmuyor.",
                "Registry targets ve yayıncı zincirini denetle.",
                paket_bildirim_yolu,
                paket_bildirim_kaynagi,
            ));
        }
        bagimliliklari_kuyruga_ekle_kokenli(
            &paket_bildirimi,
            &paket_koku.join("proje.dil"),
            &paket_bildirim_kaynagi,
            &mut kuyruk,
        );
        let kritik_var = !cikti.kritik_duyurular.is_empty();
        let kilit = UzakPaketKilidi {
            registry_kok_surumu: registry.kok_surumu,
            registry_kok_sha256: registry.kok_sha256.clone(),
            metadata: cikti.metadata_surumleri,
            yayinci_anahtar_kimligi: cikti.yayinci_anahtar_kimligi,
            arsiv_sha256: cikti.arsiv_sha256,
            sbom_sha256: cikti.sbom_sha256,
            provenance_sha256: cikti.provenance_sha256,
            yayin_sha256: cikti.yayin_sha256,
            yanked: cikti.yanked,
            kritik_duyurular: cikti.kritik_duyurular,
            yanked_kabul_gerekcesi: cikti.yanked.then_some(yanked_gerekcesi).flatten(),
            kritik_duyuru_kabul_gerekcesi: kritik_var.then_some(kritik_gerekcesi).flatten(),
            yanked_politika_anahtari,
            kritik_politika_anahtari,
        };
        cozumler.insert(
            kimlik,
            UzakPaketCozumu {
                kok: paket_koku,
                kilit,
            },
        );
    }
    Ok(cozumler)
}

fn registryden_al(
    ana_kok: &Path,
    registry: &RegistryBildirimi,
    bagimlilik: &UzakBagimlilik,
    cevrimdisi: bool,
    politika: HedefPolitikasi,
) -> Result<RegistryPaketCiktisi, crate::registry::RegistryHatasi> {
    let cache = registry_cache_koku(ana_kok, &registry.kok_sha256)?;
    if cevrimdisi {
        return RegistryIstemcisi::mevcut_cache(cache, &registry.kok_sha256).paketi_cevrimdisi_al(
            &bagimlilik.ad,
            &bagimlilik.surum,
            politika,
        );
    }
    let mut tasiyici = HttpsRegistryTasiyici::yeni(&registry.origin)?;
    let sabit_kok = RegistryIstemcisi::sabit_koku_getir(&mut tasiyici, registry.kok_surumu)?;
    RegistryIstemcisi::yeni(cache, sabit_kok, &registry.kok_sha256).paketi_guncelle(
        &mut tasiyici,
        &bagimlilik.ad,
        &bagimlilik.surum,
        politika,
    )
}

fn registry_cache_koku(
    ana_kok: &Path,
    kok_sha256: &str,
) -> Result<PathBuf, crate::registry::RegistryHatasi> {
    let ham =
        kok_sha256
            .strip_prefix("sha256:")
            .ok_or_else(|| crate::registry::RegistryHatasi {
                kod: "P016",
                mesaj: "Registry root cache anahtarı sha256: öneki taşımıyor.".into(),
            })?;
    Ok(ana_kok.join(".zee").join("registry").join(ham))
}

fn paket_koku(ana_kok: &Path, arsiv_sha256: &str) -> Result<PathBuf, String> {
    if arsiv_sha256.len() != 64
        || !arsiv_sha256
            .bytes()
            .all(|bayt| bayt.is_ascii_digit() || (b'a'..=b'f').contains(&bayt))
    {
        return Err("Registry arşiv özeti güvenli cache anahtarı değil.".into());
    }
    Ok(ana_kok
        .join(".zee")
        .join("paketler")
        .join("sha256")
        .join(arsiv_sha256))
}

fn bagimliliklari_kuyruga_ekle_kokenli(
    bildirim: &crate::proje::ProjeBildirimi,
    bildirim_yolu: &Path,
    bildirim_kaynagi: &str,
    kuyruk: &mut VecDeque<(RegistryBildirimi, UzakBagimlilik, PathBuf, String)>,
) {
    let Some(registry) = &bildirim.registry else {
        return;
    };
    let mut bagimliliklar = bildirim.uzak_bagimliliklar.clone();
    bagimliliklar.sort();
    for bagimlilik in bagimliliklar {
        kuyruk.push_back((
            registry.clone(),
            bagimlilik,
            bildirim_yolu.to_path_buf(),
            bildirim_kaynagi.to_string(),
        ));
    }
}
