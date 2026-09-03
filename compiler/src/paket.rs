//! Yerel ve exact registry proje bağımlılıkları, kaynak kökeni ve deterministik
//! kilit dosyası. Normal derleme ağ açmadan doğrulanmış cache kullanır.

use crate::agac::KullanimTuru;
use crate::guvenlik::sha256_hex;
use crate::proje::{bildirimi_oku, ProjeBildirimi};
use crate::tani::Tani;
use crate::{BirimIstegi, YuklenenBirim};
use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
mod uzak;
#[cfg(target_arch = "wasm32")]
#[path = "paket/uzak_wasm.rs"]
mod uzak;

pub use crate::paket_modeli::PaketBilgisi;
pub use uzak::RegistryCozumPolitikasi;
use uzak::{paketleri_hazirla, UzakPaketCozumleri, UzakPaketKilidi, UzakPaketKimligi};

pub const KILIT_DOSYASI: &str = "proje.kilit";

#[derive(Debug)]
pub struct ProjeYuklemeHatasi {
    pub tani: Box<Tani>,
    pub kaynak: String,
    pub yol: PathBuf,
}

struct ProjeDugumu {
    kok: PathBuf,
    bildirim: ProjeBildirimi,
    bildirim_kaynagi: String,
    bildirim_yolu: PathBuf,
    giris_yolu: PathBuf,
    kaynaklar: BTreeMap<PathBuf, String>,
    bagimliliklar: BTreeMap<String, PathBuf>,
    ozet: String,
    uzak_kilit: Option<UzakPaketKilidi>,
}

/// Tek komut boyunca değişmeyen proje grafiği. Kaynaklar kilit denetiminden
/// önce belleğe alınır; denetim ile derleme arasında dosya yeniden okunmaz.
pub struct ProjeGrafigi {
    ana_kok: PathBuf,
    dugumler: BTreeMap<PathBuf, ProjeDugumu>,
}

impl ProjeGrafigi {
    pub fn cozumle(kok: &Path) -> Result<Self, ProjeYuklemeHatasi> {
        Self::cozumle_ic(kok, None, &RegistryCozumPolitikasi::default())
    }

    /// Yalnız açık paket CLI komutlarının kullandığı çevrimiçi çözüm. Yerel
    /// projelerde ağ açılmaz; exact uzak bağımlılıklar doğrulanıp cache'lenir.
    pub fn cozumle_registry_ile(
        kok: &Path,
        politika: &RegistryCozumPolitikasi,
    ) -> Result<Self, ProjeYuklemeHatasi> {
        Self::cozumle_ic(kok, None, politika)
    }

    /// Henüz diske yazılmamış bir ana `proje.dil` adayıyla grafiği çözer.
    /// `dil ekle` bu sayede bozuk bir grafiği manifesti değiştirmeden reddeder.
    pub fn cozumle_bildirimle(
        kok: &Path,
        bildirim_kaynagi: &str,
    ) -> Result<Self, ProjeYuklemeHatasi> {
        Self::cozumle_ic(
            kok,
            Some(bildirim_kaynagi),
            &RegistryCozumPolitikasi::default(),
        )
    }

    pub fn cozumle_bildirimle_registry(
        kok: &Path,
        bildirim_kaynagi: &str,
        politika: &RegistryCozumPolitikasi,
    ) -> Result<Self, ProjeYuklemeHatasi> {
        Self::cozumle_ic(kok, Some(bildirim_kaynagi), politika)
    }

    fn cozumle_ic(
        kok: &Path,
        bildirim_kaynagi: Option<&str>,
        politika: &RegistryCozumPolitikasi,
    ) -> Result<Self, ProjeYuklemeHatasi> {
        let ana_kok = std::fs::canonicalize(kok).map_err(|hata| {
            yalniz_hata(
                "P006",
                format!("Proje kökü çözülemedi: {}.", hata),
                "Var olan ve okunabilen bir proje klasörü seç.",
                kok.to_path_buf(),
            )
        })?;
        let uzak_cozumler = paketleri_hazirla(&ana_kok, bildirim_kaynagi, politika)?;
        let mut kurucu = GrafikKurucu {
            dugumler: BTreeMap::new(),
            ad_kokleri: HashMap::new(),
            yigin: Vec::new(),
            uzak_cozumler,
        };
        kurucu.ziyaret_et(&ana_kok, false, None, bildirim_kaynagi, None)?;
        let grafik = Self {
            ana_kok,
            dugumler: kurucu.dugumler,
        };
        grafik.paket_yetkinliklerini_denetle()?;
        Ok(grafik)
    }

    pub fn ana_bildirim(&self) -> &ProjeBildirimi {
        &self.ana_dugum().bildirim
    }

    pub fn ana_kok(&self) -> &Path {
        &self.ana_kok
    }

    pub fn ana_giris_yolu(&self) -> &Path {
        &self.ana_dugum().giris_yolu
    }

    pub fn ana_giris(&self) -> Result<YuklenenBirim, String> {
        let dugum = self.ana_dugum();
        let kaynak = dugum
            .kaynaklar
            .get(&dugum.giris_yolu)
            .cloned()
            .ok_or_else(|| "Proje giriş kaynağı doğrulanmış grafikte bulunamadı.".to_string())?;
        Ok(YuklenenBirim {
            kaynak,
            koken: yol_metni(&dugum.giris_yolu),
        })
    }

    pub fn bagimlilik_var(&self) -> bool {
        self.dugumler
            .values()
            .any(|dugum| !dugum.bagimliliklar.is_empty())
    }

    pub fn paket_sayisi(&self) -> usize {
        self.dugumler.len().saturating_sub(1)
    }

    /// Kullanıcıya gösterilecek kararlı paket görünümü: doğrudan/geçişli
    /// ayrımı, taşınabilir yol ve tam içerik özeti.
    pub fn paketler(&self) -> Vec<PaketBilgisi> {
        let dogrudan_kokler: std::collections::HashSet<&PathBuf> =
            self.ana_dugum().bagimliliklar.values().collect();
        let mut paketler = self
            .dugumler
            .values()
            .filter(|dugum| dugum.kok != self.ana_kok)
            .map(|dugum| PaketBilgisi {
                ad: dugum.bildirim.ad.clone(),
                surum: dugum.bildirim.surum.clone(),
                morfoloji: dugum.bildirim.morfoloji.clone(),
                yol: goreli_yol(&self.ana_kok, &dugum.kok),
                ozet: dugum.ozet.clone(),
                dogrudan: dogrudan_kokler.contains(&dugum.kok),
                registry_kok_sha256: dugum
                    .uzak_kilit
                    .as_ref()
                    .map(|kilit| kilit.registry_kok_sha256.clone()),
                arsiv_sha256: dugum
                    .uzak_kilit
                    .as_ref()
                    .map(|kilit| kilit.arsiv_sha256.clone()),
            })
            .collect::<Vec<_>>();
        paketler.sort_by(|a, b| a.ad.cmp(&b.ad).then(a.yol.cmp(&b.yol)));
        paketler
    }

    pub fn dogrudan_paket_koku(&self, ad: &str) -> Option<&Path> {
        self.ana_dugum().bagimliliklar.get(ad).map(PathBuf::as_path)
    }

    /// Paketin doğrudan kenarı kaldırılmadan önce ana projenin bütün gerçek
    /// kaynaklarını tarar. Tek kullanım bile varsa sessiz kırılma yerine P010.
    pub fn kaldirmayi_dogrula(&self, ad: &str) -> Result<(), ProjeYuklemeHatasi> {
        let ana = self.ana_dugum();
        for (yol, kaynak) in &ana.kaynaklar {
            let tokenlar =
                crate::sozcukleyici::sozcukle(kaynak).map_err(|tani| ProjeYuklemeHatasi {
                    tani: Box::new(tani),
                    kaynak: kaynak.clone(),
                    yol: yol.clone(),
                })?;
            if let Some((_, _, satir)) = crate::ayristirici::kullanilan_birimler(&tokenlar)
                .into_iter()
                .find(|(kullanilan, tur, _)| kullanilan == ad && *tur == KullanimTuru::Paket)
            {
                return Err(ProjeYuklemeHatasi {
                    tani: Box::new(
                        Tani::yeni(
                            "P010",
                            format!("\"{}\" paketi bu kaynakta hâlâ kullanılıyor.", ad),
                            satir,
                            1,
                            1,
                        )
                        .onerili(format!(
                            "Önce `{} paketini kullan` satırını ve ona bağlı çağrıları kaldır.",
                            ad
                        )),
                    ),
                    kaynak: kaynak.clone(),
                    yol: yol.clone(),
                });
            }
        }
        Ok(())
    }

    /// Var olan kilit dosyası her zaman doğrulanır. Eski, bağımlılıksız
    /// projelerde dosya yoksa geriye uyumluluk için sessizce geçilir.
    pub fn kilidi_denetle(&self) -> Result<(), ProjeYuklemeHatasi> {
        let yol = self.ana_kok.join(KILIT_DOSYASI);
        let beklenen = self.kilit_metni();
        match crate::kaynak_sinirlari::kaynak_dosyasi_oku(&yol) {
            Ok(bulunan) if bulunan == beklenen => Ok(()),
            Ok(_) => Err(self.kilit_hatasi(
                "Proje kilidi bildirimlerle veya bağımlılık kaynaklarıyla uyuşmuyor.",
            )),
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound && !self.bagimlilik_var() => {
                Ok(())
            }
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => {
                Err(self.kilit_hatasi("Yerel bağımlılık kullanan projede proje.kilit bulunamadı."))
            }
            Err(hata) => Err(self.kilit_hatasi(&format!("proje.kilit okunamadı: {}.", hata))),
        }
    }

    pub fn kilidi_yaz(&self) -> Result<(), String> {
        let yol = self.ana_kok.join(KILIT_DOSYASI);
        crate::kalici_dosya::atomik_yaz(&yol, self.kilit_metni().as_bytes())
            .map_err(|hata| format!("\"{}\" yazılamadı: {}", yol.display(), hata))
    }

    /// Doğrulanmış aday bildirim ile bu grafiğin kilidini birlikte günceller;
    /// kilit yazılamazsa iki dosyayı da önceki byte'larına geri döndürür.
    pub fn bildirim_ve_kilidi_yaz(
        &self,
        eski_bildirim: &str,
        yeni_bildirim: &str,
    ) -> Result<(), String> {
        let bildirim_yolu = self.ana_kok.join("proje.dil");
        let kilit_yolu = self.ana_kok.join(KILIT_DOSYASI);
        let eski_kilit = match crate::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(&kilit_yolu) {
            Ok(icerik) => Some(icerik),
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => None,
            Err(hata) => return Err(format!("Önceki proje.kilit okunamadı: {}.", hata)),
        };
        crate::kalici_dosya::atomik_yaz(&bildirim_yolu, yeni_bildirim.as_bytes())
            .map_err(|hata| format!("\"{}\" yazılamadı: {}", bildirim_yolu.display(), hata))?;
        if let Err(hata) = self.kilidi_yaz() {
            let bildirim_geri =
                crate::kalici_dosya::atomik_yaz(&bildirim_yolu, eski_bildirim.as_bytes());
            let kilit_geri = match eski_kilit {
                Some(icerik) => crate::kalici_dosya::atomik_yaz(&kilit_yolu, &icerik),
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

    pub fn kilit_metni(&self) -> String {
        let ana = self.ana_dugum();
        let mut metin = String::from(
            "# zee bağımlılık kilidi — `dil kilitle` üretir; elle düzenleme.\nkilit_sürümü 3\n",
        );
        metin.push_str(&format!(
            "ana \"{}\" \"{}\" \"{}\"\n",
            kacis(&ana.bildirim.ad),
            kacis(&ana.bildirim.surum),
            kacis(&ana.bildirim.morfoloji)
        ));

        let mut paketler: Vec<&ProjeDugumu> = self
            .dugumler
            .values()
            .filter(|dugum| dugum.kok != self.ana_kok)
            .collect();
        paketler.sort_by(|a, b| a.bildirim.ad.cmp(&b.bildirim.ad).then(a.kok.cmp(&b.kok)));
        for paket in paketler {
            match &paket.uzak_kilit {
                None => metin.push_str(&format!(
                    "paket \"{}\" \"{}\" \"{}\" \"{}\" \"sha256:{}\"\n",
                    kacis(&paket.bildirim.ad),
                    kacis(&paket.bildirim.surum),
                    kacis(&paket.bildirim.morfoloji),
                    kacis(&goreli_yol(&self.ana_kok, &paket.kok)),
                    paket.ozet
                )),
                Some(kilit) => {
                    metin.push_str(&format!(
                        concat!(
                            "uzak \"{}\" \"{}\" \"{}\" \"sha256:{}\" ",
                            "\"{}\" \"{}\" \"{}\" ",
                            "\"{}\" \"{}\" \"{}\" \"{}\" \"{}\" \"{}\" ",
                            "\"{}\" \"{}\" \"{}\" \"{}\" \"{}\" \"{}\" \"{}\"\n"
                        ),
                        kacis(&paket.bildirim.ad),
                        kacis(&paket.bildirim.surum),
                        kacis(&paket.bildirim.morfoloji),
                        paket.ozet,
                        kilit.registry_kok_surumu,
                        kilit.registry_kok_sha256,
                        kilit.metadata.root,
                        kilit.metadata.timestamp,
                        kilit.metadata.timestamp_sha256,
                        kilit.metadata.snapshot,
                        kilit.metadata.snapshot_sha256,
                        kilit.metadata.targets,
                        kilit.metadata.targets_sha256,
                        kilit.yayinci_anahtar_kimligi,
                        kilit.arsiv_sha256,
                        kilit.sbom_sha256,
                        kilit.provenance_sha256,
                        kilit.yayin_sha256,
                        if kilit.yanked { "evet" } else { "hayır" },
                        kilit.kritik_duyurular.join(",")
                    ));
                    if let Some(gerekce) = &kilit.yanked_kabul_gerekcesi {
                        metin.push_str(&format!(
                            "yanked_kabul \"{}\" \"{}\"\n",
                            kacis(&kilit.yanked_politika_anahtari),
                            kacis(gerekce)
                        ));
                    }
                    if let Some(gerekce) = &kilit.kritik_duyuru_kabul_gerekcesi {
                        metin.push_str(&format!(
                            "kritik_kabul \"{}\" \"{}\"\n",
                            kacis(&kilit.kritik_politika_anahtari),
                            kacis(gerekce)
                        ));
                    }
                }
            }
        }

        let mut kenarlar = Vec::new();
        for dugum in self.dugumler.values() {
            for (ad, hedef) in &dugum.bagimliliklar {
                let hedef_adi = &self.dugumler[hedef].bildirim.ad;
                kenarlar.push((dugum.bildirim.ad.clone(), ad.clone(), hedef_adi.clone()));
            }
        }
        kenarlar.sort();
        for (kaynak, kullanim, hedef) in kenarlar {
            metin.push_str(&format!(
                "bağ \"{}\" \"{}\" \"{}\"\n",
                kacis(&kaynak),
                kacis(&kullanim),
                kacis(&hedef)
            ));
        }
        metin
    }

    /// Derleyicinin kaynak-kökenli yükleyici yüzeyi.
    pub fn yukle(&self, istek: BirimIstegi<'_>) -> Result<YuklenenBirim, String> {
        let isteyen = istek
            .isteyen
            .map(Path::new)
            .ok_or_else(|| "kullanan kaynağın kökeni bilinmiyor".to_string())?;
        let sahip = self
            .sahip_dugum(isteyen)
            .ok_or_else(|| format!("\"{}\" proje grafiğinin dışında", isteyen.display()))?;

        match istek.tur {
            KullanimTuru::Birim => {
                if !crate::proje::gecerli_paket_adi(istek.ad) {
                    return Err("birim adı tek bir Türkçe/Latin tanımlayıcı olmalı".into());
                }
                let aday = isteyen
                    .parent()
                    .unwrap_or(&sahip.kok)
                    .join(format!("{}.dil", istek.ad));
                if let Ok(kanonik) = std::fs::canonicalize(&aday) {
                    if !kanonik.starts_with(&sahip.kok) {
                        return Err("birim sembolik bağ üzerinden proje dışına çıkamaz".into());
                    }
                    if let Some(kaynak) = sahip.kaynaklar.get(&kanonik) {
                        return Ok(YuklenenBirim {
                            kaynak: kaynak.clone(),
                            koken: yol_metni(&kanonik),
                        });
                    }
                }
                crate::gomulu_birim(istek.ad)
                    .map(|kaynak| YuklenenBirim {
                        kaynak: kaynak.to_string(),
                        koken: format!("gömülü:{}", istek.ad),
                    })
                    .ok_or_else(|| format!("\"{}\" bulunamadı", aday.display()))
            }
            KullanimTuru::Paket => {
                let hedef = sahip.bagimliliklar.get(istek.ad).ok_or_else(|| {
                    format!(
                        "\"{}\" projesinin doğrudan bağımlılıkları arasında yok",
                        sahip.bildirim.ad
                    )
                })?;
                let paket = &self.dugumler[hedef];
                let kaynak = paket
                    .kaynaklar
                    .get(&paket.giris_yolu)
                    .cloned()
                    .ok_or_else(|| {
                        "Paket giriş kaynağı doğrulanmış grafikte bulunamadı.".to_string()
                    })?;
                Ok(YuklenenBirim {
                    kaynak,
                    koken: yol_metni(&paket.giris_yolu),
                })
            }
        }
    }

    fn ana_dugum(&self) -> &ProjeDugumu {
        &self.dugumler[&self.ana_kok]
    }

    fn paket_yetkinliklerini_denetle(&self) -> Result<(), ProjeYuklemeHatasi> {
        let ana_politika = self.ana_dugum().bildirim.yetkinlik_politikasi();
        for dugum in self
            .dugumler
            .values()
            .filter(|dugum| dugum.kok != self.ana_kok)
        {
            if let Err(neden) =
                ana_politika.alt_politikayi_denetle(&dugum.bildirim.yetkinlik_politikasi())
            {
                return Err(proje_hatasi(
                    "P015",
                    &format!(
                        "\"{}\" paketi üst projenin vermediği bir yetkinlik istiyor: {}.",
                        dugum.bildirim.ad, neden
                    ),
                    "Paketi kullanmadan önce yetkinliği ve ağ hedefini ana proje.dil bildiriminde açıkça onayla.",
                    dugum.bildirim_yolu.clone(),
                    dugum.bildirim_kaynagi.clone(),
                ));
            }
        }
        Ok(())
    }

    fn sahip_dugum(&self, kaynak: &Path) -> Option<&ProjeDugumu> {
        self.dugumler
            .values()
            .filter(|dugum| kaynak.starts_with(&dugum.kok))
            .max_by_key(|dugum| dugum.kok.components().count())
    }

    fn kilit_hatasi(&self, mesaj: &str) -> ProjeYuklemeHatasi {
        let ana = self.ana_dugum();
        proje_hatasi(
            "P008",
            mesaj,
            "Bağımlılıkları bilinçli olarak sabitlemek için `dil kilitle .` çalıştır.",
            ana.bildirim_yolu.clone(),
            ana.bildirim_kaynagi.clone(),
        )
    }
}

struct GrafikKurucu {
    dugumler: BTreeMap<PathBuf, ProjeDugumu>,
    ad_kokleri: HashMap<String, PathBuf>,
    yigin: Vec<PathBuf>,
    uzak_cozumler: UzakPaketCozumleri,
}

impl GrafikKurucu {
    fn ziyaret_et(
        &mut self,
        kok: &Path,
        bagimlilik_mi: bool,
        isteyen: Option<(&Path, &str)>,
        gecici_bildirim: Option<&str>,
        uzak_kilit: Option<UzakPaketKilidi>,
    ) -> Result<(), ProjeYuklemeHatasi> {
        if self.yigin.iter().any(|yol| yol == kok) {
            let (yol, kaynak) = isteyen
                .and_then(|(yol, _)| self.dugumler.get(yol))
                .map(|d| (d.bildirim_yolu.clone(), d.bildirim_kaynagi.clone()))
                .unwrap_or_else(|| (kok.join("proje.dil"), String::new()));
            return Err(proje_hatasi(
                "P007",
                &format!("Yerel bağımlılık döngüsü oluştu: {}.", kok.display()),
                "Ortak kodu döngünün dışında üçüncü bir projeye taşı.",
                yol,
                kaynak,
            ));
        }
        if self.dugumler.contains_key(kok) {
            return Ok(());
        }

        let bildirim_yolu = kok.join("proje.dil");
        let bildirim_kaynagi = match gecici_bildirim {
            Some(kaynak) => kaynak.to_string(),
            None => {
                crate::kaynak_sinirlari::kaynak_dosyasi_oku(&bildirim_yolu).map_err(|hata| {
                    proje_hatasi(
                        "P006",
                        &format!(
                            "Yerel bağımlılığın proje.dil bildirimi okunamadı: {}.",
                            hata
                        ),
                        "Yolun bir zee proje klasörünü gösterdiğini doğrula.",
                        bildirim_yolu.clone(),
                        String::new(),
                    )
                })?
            }
        };
        let bildirim = bildirimi_oku(&bildirim_kaynagi).map_err(|tani| ProjeYuklemeHatasi {
            tani: Box::new(tani),
            kaynak: bildirim_kaynagi.clone(),
            yol: bildirim_yolu.clone(),
        })?;
        if bagimlilik_mi && !crate::proje::gecerli_paket_adi(&bildirim.ad) {
            return Err(proje_hatasi(
                "P007",
                &format!(
                    "\"{}\" paket adı kaynakta tek kelime olarak kullanılamaz.",
                    bildirim.ad
                ),
                "Yerel bağımlılığın proje adını küçük harfli tek bir tanımlayıcı yap; örnek: grafik_araclari.",
                bildirim_yolu,
                bildirim_kaynagi,
            ));
        }
        if let Some(onceki) = self.ad_kokleri.get(&bildirim.ad) {
            if onceki != kok {
                return Err(proje_hatasi(
                    "P007",
                    &format!(
                        "\"{}\" paket adı iki ayrı projede kullanılıyor: {} ve {}.",
                        bildirim.ad,
                        onceki.display(),
                        kok.display()
                    ),
                    "Paket adlarını benzersiz yap; sessiz seçim yapılmaz.",
                    bildirim_yolu,
                    bildirim_kaynagi,
                ));
            }
        }
        self.ad_kokleri
            .insert(bildirim.ad.clone(), kok.to_path_buf());

        self.yigin.push(kok.to_path_buf());
        let mut bagimliliklar = BTreeMap::new();
        let mut yollar = bildirim.yerel_bagimliliklar.clone();
        yollar.sort();
        for bildirilen in yollar {
            let hedef = std::fs::canonicalize(kok.join(&bildirilen)).map_err(|hata| {
                proje_hatasi(
                    "P006",
                    &format!("\"{}\" yerel bağımlılığı çözülemedi: {}.", bildirilen, hata),
                    "Yolu proje.dil dosyasının bulunduğu klasöre göre düzelt.",
                    bildirim_yolu.clone(),
                    bildirim_kaynagi.clone(),
                )
            })?;
            self.ziyaret_et(&hedef, true, Some((kok, &bildirilen)), None, None)?;
            let hedef_adi = self.dugumler[&hedef].bildirim.ad.clone();
            if bagimliliklar.insert(hedef_adi.clone(), hedef).is_some() {
                return Err(proje_hatasi(
                    "P007",
                    &format!("\"{}\" doğrudan bağımlılığı iki kez çözüldü.", hedef_adi),
                    "Aynı adlı bağımlılıklardan birini kaldır.",
                    bildirim_yolu.clone(),
                    bildirim_kaynagi.clone(),
                ));
            }
        }
        let registry = bildirim.registry.as_ref();
        let mut uzak_bagimliliklar = bildirim.uzak_bagimliliklar.clone();
        uzak_bagimliliklar.sort();
        for uzak in uzak_bagimliliklar {
            let Some(registry) = registry else {
                return Err(proje_hatasi(
                    "P017",
                    "Uzak bağımlılık registry sabitlemesi olmadan çözülemez.",
                    "Registry origin'i, root sürümü ve SHA-256 özetini birlikte bildir.",
                    bildirim_yolu.clone(),
                    bildirim_kaynagi.clone(),
                ));
            };
            let kimlik = UzakPaketKimligi::yeni(registry, &uzak);
            let cozum = self.uzak_cozumler.get(&kimlik).cloned().ok_or_else(|| {
                proje_hatasi(
                    "P016",
                    &format!(
                        "{}@{} doğrulanmış registry cache'inde yok.",
                        uzak.ad, uzak.surum
                    ),
                    "Açık bir paket komutuyla cache'i güncelle veya --çevrimdışı için önce indir.",
                    bildirim_yolu.clone(),
                    bildirim_kaynagi.clone(),
                )
            })?;
            let hedef = std::fs::canonicalize(&cozum.kok).map_err(|hata| {
                proje_hatasi(
                    "P016",
                    &format!("Registry paket kurulumu çözülemedi: {}.", hata),
                    "İçerik-adresli kurulumu doğrulanmış cache'den yeniden oluştur.",
                    bildirim_yolu.clone(),
                    bildirim_kaynagi.clone(),
                )
            })?;
            self.ziyaret_et(&hedef, true, Some((kok, &uzak.ad)), None, Some(cozum.kilit))?;
            let hedef_adi = self.dugumler[&hedef].bildirim.ad.clone();
            if bagimliliklar.insert(hedef_adi.clone(), hedef).is_some() {
                return Err(proje_hatasi(
                    "P007",
                    &format!("\"{}\" doğrudan bağımlılığı iki kez çözüldü.", hedef_adi),
                    "Yerel ve uzak listelerde aynı paket adını birlikte kullanma.",
                    bildirim_yolu.clone(),
                    bildirim_kaynagi.clone(),
                ));
            }
        }
        self.yigin.pop();

        let kaynaklar = kaynaklari_oku(kok).map_err(|mesaj| {
            proje_hatasi(
                "P009",
                &mesaj,
                "Proje kaynaklarının okunabildiğini ve sembolik bağ olmadığını doğrula.",
                bildirim_yolu.clone(),
                bildirim_kaynagi.clone(),
            )
        })?;
        let giris_yolu = std::fs::canonicalize(kok.join(&bildirim.giris)).map_err(|hata| {
            proje_hatasi(
                "P009",
                &format!("Proje giriş kaynağı çözülemedi: {}.", hata),
                "Bildirimdeki giriş yolunu var olan bir .dil dosyasına yönelt.",
                bildirim_yolu.clone(),
                bildirim_kaynagi.clone(),
            )
        })?;
        if !giris_yolu.starts_with(kok) || !kaynaklar.contains_key(&giris_yolu) {
            return Err(proje_hatasi(
                "P009",
                "Proje girişi proje ağacında gerçek, sembolik bağ olmayan bir .dil dosyası değil.",
                "Girişi proje kökü içindeki gerçek bir .dil kaynağına yönelt.",
                bildirim_yolu.clone(),
                bildirim_kaynagi.clone(),
            ));
        }
        let ozet = kaynak_ozeti(kok, &kaynaklar);
        self.dugumler.insert(
            kok.to_path_buf(),
            ProjeDugumu {
                kok: kok.to_path_buf(),
                bildirim,
                bildirim_kaynagi,
                bildirim_yolu,
                giris_yolu,
                kaynaklar,
                bagimliliklar,
                ozet,
                uzak_kilit,
            },
        );
        Ok(())
    }
}

fn kaynaklari_oku(kok: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    fn gez(
        klasor: &Path,
        sonuc: &mut BTreeMap<PathBuf, String>,
        toplam_bayt: &mut usize,
    ) -> Result<(), String> {
        let mut girdiler: Vec<std::fs::DirEntry> = std::fs::read_dir(klasor)
            .map_err(|hata| format!("\"{}\" listelenemedi: {}", klasor.display(), hata))?
            .collect::<Result<_, _>>()
            .map_err(|hata| hata.to_string())?;
        girdiler.sort_by_key(|girdi| girdi.file_name());
        for girdi in girdiler {
            let tur = girdi.file_type().map_err(|hata| hata.to_string())?;
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
                gez(&yol, sonuc, toplam_bayt)?;
            } else if tur.is_file()
                && yol.extension().and_then(|uzanti| uzanti.to_str()) == Some("dil")
            {
                let kanonik = std::fs::canonicalize(&yol).map_err(|hata| hata.to_string())?;
                let kaynak = crate::kaynak_sinirlari::kaynak_dosyasi_oku(&kanonik)
                    .map_err(|hata| format!("\"{}\" okunamadı: {}", yol.display(), hata))?;
                let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI;
                if sonuc.len() >= sinirlar.kaynak_dosyasi() {
                    return Err(format!(
                        "Proje {} kaynak dosyası sınırını aşıyor.",
                        sinirlar.kaynak_dosyasi()
                    ));
                }
                *toplam_bayt = toplam_bayt
                    .checked_add(kaynak.len())
                    .ok_or_else(|| "Proje kaynak boyutu sayı sınırını aştı.".to_string())?;
                if *toplam_bayt > sinirlar.toplam_kaynak_bayti() {
                    return Err(format!(
                        "Proje kaynakları toplam {} MiB sınırını aşıyor.",
                        sinirlar.toplam_kaynak_bayti() / 1024 / 1024
                    ));
                }
                sonuc.insert(kanonik, kaynak);
            }
        }
        Ok(())
    }

    let mut sonuc = BTreeMap::new();
    let mut toplam_bayt = 0usize;
    gez(kok, &mut sonuc, &mut toplam_bayt)?;
    Ok(sonuc)
}

fn kaynak_ozeti(kok: &Path, kaynaklar: &BTreeMap<PathBuf, String>) -> String {
    let mut veri = Vec::new();
    for (yol, kaynak) in kaynaklar {
        let goreli = yol.strip_prefix(kok).unwrap_or(yol);
        let yol = duz_yol(goreli);
        veri.extend_from_slice(&(yol.len() as u64).to_be_bytes());
        veri.extend_from_slice(yol.as_bytes());
        veri.extend_from_slice(&(kaynak.len() as u64).to_be_bytes());
        veri.extend_from_slice(kaynak.as_bytes());
    }
    sha256_hex(&veri)
}

fn proje_hatasi(
    kod: &str,
    mesaj: &str,
    oneri: &str,
    yol: PathBuf,
    kaynak: String,
) -> ProjeYuklemeHatasi {
    ProjeYuklemeHatasi {
        tani: Box::new(Tani::yeni(kod, mesaj.into(), 1, 1, 1).onerili(oneri.into())),
        kaynak,
        yol,
    }
}

fn yalniz_hata(kod: &str, mesaj: String, oneri: &str, yol: PathBuf) -> ProjeYuklemeHatasi {
    proje_hatasi(kod, &mesaj, oneri, yol, String::new())
}

fn yol_metni(yol: &Path) -> String {
    yol.to_string_lossy().into_owned()
}

fn duz_yol(yol: &Path) -> String {
    yol.components()
        .filter_map(|bilesen| match bilesen {
            Component::Normal(ad) => Some(ad.to_string_lossy().into_owned()),
            Component::ParentDir => Some("..".into()),
            Component::CurDir => None,
            Component::RootDir => Some(String::new()),
            Component::Prefix(on) => Some(on.as_os_str().to_string_lossy().into_owned()),
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn goreli_yol(kok: &Path, hedef: &Path) -> String {
    let kok_bilesenleri: Vec<_> = kok.components().collect();
    let hedef_bilesenleri: Vec<_> = hedef.components().collect();
    let ortak = kok_bilesenleri
        .iter()
        .zip(&hedef_bilesenleri)
        .take_while(|(a, b)| a == b)
        .count();
    if ortak == 0 {
        return duz_yol(hedef);
    }
    let mut parcalar = vec!["..".to_string(); kok_bilesenleri.len() - ortak];
    parcalar.extend(hedef_bilesenleri[ortak..].iter().filter_map(|b| match b {
        Component::Normal(ad) => Some(ad.to_string_lossy().into_owned()),
        _ => None,
    }));
    if parcalar.is_empty() {
        ".".into()
    } else {
        parcalar.join("/")
    }
}

/// İki kanonik klasör arasında manifestte taşınabilir `/` ayraçlı yol üretir.
pub fn goreli_yerel_yol(kok: &Path, hedef: &Path) -> String {
    goreli_yol(kok, hedef)
}

fn kacis(metin: &str) -> String {
    metin
        .chars()
        .flat_map(|k| match k {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect(),
            '\n' => "\\n".chars().collect(),
            _ => vec![k],
        })
        .collect()
}
