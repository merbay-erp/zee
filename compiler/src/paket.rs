//! Yerel proje bağımlılıkları, kaynak kökeni ve deterministik kilit dosyası.
//! Registry/ağ yoktur: K-078 yalnız açıkça bildirilmiş yerel projeleri çözer.

use crate::agac::KullanimTuru;
use crate::proje::{bildirimi_oku, ProjeBildirimi};
use crate::tani::Tani;
use crate::{BirimIstegi, YuklenenBirim};
use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path, PathBuf};

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
}

/// Tek komut boyunca değişmeyen proje grafiği. Kaynaklar kilit denetiminden
/// önce belleğe alınır; denetim ile derleme arasında dosya yeniden okunmaz.
pub struct ProjeGrafigi {
    ana_kok: PathBuf,
    dugumler: BTreeMap<PathBuf, ProjeDugumu>,
}

impl ProjeGrafigi {
    pub fn cozumle(kok: &Path) -> Result<Self, ProjeYuklemeHatasi> {
        Self::cozumle_ic(kok, None)
    }

    /// Henüz diske yazılmamış bir ana `proje.dil` adayıyla grafiği çözer.
    /// `dil ekle` bu sayede bozuk bir grafiği manifesti değiştirmeden reddeder.
    pub fn cozumle_bildirimle(
        kok: &Path,
        bildirim_kaynagi: &str,
    ) -> Result<Self, ProjeYuklemeHatasi> {
        Self::cozumle_ic(kok, Some(bildirim_kaynagi))
    }

    fn cozumle_ic(kok: &Path, bildirim_kaynagi: Option<&str>) -> Result<Self, ProjeYuklemeHatasi> {
        let ana_kok = std::fs::canonicalize(kok).map_err(|hata| {
            yalniz_hata(
                "P006",
                format!("Proje kökü çözülemedi: {}.", hata),
                "Var olan ve okunabilen bir proje klasörü seç.",
                kok.to_path_buf(),
            )
        })?;
        let mut kurucu = GrafikKurucu {
            dugumler: BTreeMap::new(),
            ad_kokleri: HashMap::new(),
            yigin: Vec::new(),
        };
        kurucu.ziyaret_et(&ana_kok, false, None, bildirim_kaynagi)?;
        Ok(Self {
            ana_kok,
            dugumler: kurucu.dugumler,
        })
    }

    pub fn ana_bildirim(&self) -> &ProjeBildirimi {
        &self.ana_dugum().bildirim
    }

    pub fn ana_giris_yolu(&self) -> &Path {
        &self.ana_dugum().giris_yolu
    }

    pub fn ana_giris(&self) -> YuklenenBirim {
        let dugum = self.ana_dugum();
        YuklenenBirim {
            kaynak: dugum
                .kaynaklar
                .get(&dugum.giris_yolu)
                .expect("giriş çözümlemede doğrulandı")
                .clone(),
            koken: yol_metni(&dugum.giris_yolu),
        }
    }

    pub fn bagimlilik_var(&self) -> bool {
        self.dugumler
            .values()
            .any(|dugum| !dugum.bagimliliklar.is_empty())
    }

    pub fn paket_sayisi(&self) -> usize {
        self.dugumler.len().saturating_sub(1)
    }

    /// Var olan kilit dosyası her zaman doğrulanır. Eski, bağımlılıksız
    /// projelerde dosya yoksa geriye uyumluluk için sessizce geçilir.
    pub fn kilidi_denetle(&self) -> Result<(), ProjeYuklemeHatasi> {
        let yol = self.ana_kok.join(KILIT_DOSYASI);
        let beklenen = self.kilit_metni();
        match std::fs::read_to_string(&yol) {
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
        std::fs::write(&yol, self.kilit_metni())
            .map_err(|hata| format!("\"{}\" yazılamadı: {}", yol.display(), hata))
    }

    pub fn kilit_metni(&self) -> String {
        let ana = self.ana_dugum();
        let mut metin = String::from(
            "# zee bağımlılık kilidi — `dil kilitle` üretir; elle düzenleme.\nkilit_sürümü 1\n",
        );
        metin.push_str(&format!(
            "ana \"{}\" \"{}\"\n",
            kacis(&ana.bildirim.ad),
            kacis(&ana.bildirim.surum)
        ));

        let mut paketler: Vec<&ProjeDugumu> = self
            .dugumler
            .values()
            .filter(|dugum| dugum.kok != self.ana_kok)
            .collect();
        paketler.sort_by(|a, b| a.bildirim.ad.cmp(&b.bildirim.ad).then(a.kok.cmp(&b.kok)));
        for paket in paketler {
            metin.push_str(&format!(
                "paket \"{}\" \"{}\" \"{}\" \"sha256:{}\"\n",
                kacis(&paket.bildirim.ad),
                kacis(&paket.bildirim.surum),
                kacis(&goreli_yol(&self.ana_kok, &paket.kok)),
                paket.ozet
            ));
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
                if !gecerli_paket_adi(istek.ad) {
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
                    .expect("paket girişi çözümlemede doğrulandı")
                    .clone();
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
}

impl GrafikKurucu {
    fn ziyaret_et(
        &mut self,
        kok: &Path,
        bagimlilik_mi: bool,
        isteyen: Option<(&Path, &str)>,
        gecici_bildirim: Option<&str>,
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
            None => std::fs::read_to_string(&bildirim_yolu).map_err(|hata| {
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
            })?,
        };
        let bildirim = bildirimi_oku(&bildirim_kaynagi).map_err(|tani| ProjeYuklemeHatasi {
            tani: Box::new(tani),
            kaynak: bildirim_kaynagi.clone(),
            yol: bildirim_yolu.clone(),
        })?;
        if bagimlilik_mi && !gecerli_paket_adi(&bildirim.ad) {
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
            self.ziyaret_et(&hedef, true, Some((kok, &bildirilen)), None)?;
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
            },
        );
        Ok(())
    }
}

fn kaynaklari_oku(kok: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    fn gez(klasor: &Path, sonuc: &mut BTreeMap<PathBuf, String>) -> Result<(), String> {
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
                gez(&yol, sonuc)?;
            } else if tur.is_file()
                && yol.extension().and_then(|uzanti| uzanti.to_str()) == Some("dil")
            {
                let kanonik = std::fs::canonicalize(&yol).map_err(|hata| hata.to_string())?;
                let kaynak = std::fs::read_to_string(&kanonik)
                    .map_err(|hata| format!("\"{}\" okunamadı: {}", yol.display(), hata))?;
                sonuc.insert(kanonik, kaynak);
            }
        }
        Ok(())
    }

    let mut sonuc = BTreeMap::new();
    gez(kok, &mut sonuc)?;
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

fn gecerli_paket_adi(ad: &str) -> bool {
    let mut harfler = ad.chars();
    matches!(harfler.next(), Some(k) if k == '_' || k.is_alphabetic())
        && harfler.all(|k| k == '_' || k.is_alphabetic() || k.is_ascii_digit())
        && ad.chars().all(|k| !k.is_uppercase())
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

// Bağımlılıksız SHA-256 (FIPS 180-4): kilit özeti için kriptografik ve
// platformlar arası aynı içerik kimliği.
fn sha256_hex(girdi: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_uzunlugu = (girdi.len() as u64).wrapping_mul(8);
    let mut veri = girdi.to_vec();
    veri.push(0x80);
    while veri.len() % 64 != 56 {
        veri.push(0);
    }
    veri.extend_from_slice(&bit_uzunlugu.to_be_bytes());

    for blok in veri.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, dortlu) in blok.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(dortlu.try_into().expect("dört bayt"));
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (yer, deger) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *yer = yer.wrapping_add(deger);
        }
    }
    h.iter().map(|deger| format!("{:08x}", deger)).collect()
}

#[cfg(test)]
mod testler {
    use super::sha256_hex;

    #[test]
    fn sha256_bilinen_vektor() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
