//! Proje bildirimi (`proje.dil`) — Faz 3 proje/paket temelinin ilk katmanı.
//!
//! Bildirim ayrı bir veri dili öğretmez; sıradan zee değer tanımlarından oluşur:
//!
//! ```text
//! proje "benim-projem" olsun
//! sürüm "0.1.0" olsun
//! morfoloji "zee-tr-1" olsun
//! giriş "program.dil" olsun
//! yetkinlikler boş liste olsun
//! ağ_hedefleri boş liste olsun
//! yerel_bağımlılıklar "../ortak" listesi olsun
//! ```

use crate::agac::{Cumle, Ifade};
use crate::tani::Tani;
use crate::yetkinlik::{AgHedefi, Yetkinlik, YetkinlikPolitikasi};
use std::collections::{BTreeSet, HashMap};
use std::path::{Component, Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjeBildirimi {
    pub ad: String,
    pub surum: String,
    /// Kaynak adlarının hangi sürümlü Türkçe ek tablosuyla çözüleceği.
    pub morfoloji: String,
    pub giris: String,
    /// Dış dünya erişimleri kaynakta değil proje sahibinin bildiriminde açılır.
    pub yetkinlikler: BTreeSet<Yetkinlik>,
    /// Outbound ağ için tam şema+host+port allowlist'i.
    pub ag_hedefleri: BTreeSet<AgHedefi>,
    /// Her yol, kendi `proje.dil` bildirimi olan yerel bir projedir. Paket adı
    /// ve sürümü bağımlı projenin bildiriminden gelir; iki yerde tekrarlanmaz.
    pub yerel_bagimliliklar: Vec<String>,
}

/// `proje.dil` kaynağını doğrular ve proje sözleşmesine çevirir.
///
/// Bildirim geçerli zee kaynağı olmak zorundadır; bunun üstüne yalnız tanımlı
/// alanları kabul eden, yan etkisiz ve deterministik proje biçimi uygulanır.
pub fn bildirimi_oku(kaynak: &str) -> Result<ProjeBildirimi, Tani> {
    let program = crate::kaynagi_derle(kaynak)?;
    if !program.islemler.is_empty() || !program.yapilar.is_empty() || !program.testler.is_empty() {
        return Err(proje_hatasi(
            "P001",
            "Proje bildiriminde işlem, eylem, yapı veya test tanımlanamaz.",
            1,
            "proje.dil yalnız proje, sürüm ve giriş metinlerini tanımlar.",
        ));
    }

    let mut alanlar: HashMap<String, (Ifade, usize)> = HashMap::new();
    for cumle in program.cumleler {
        let Cumle::Olsun {
            ad, deger, satir, ..
        } = cumle
        else {
            return Err(proje_hatasi(
                "P001",
                "Proje bildiriminde yalnız değer tanımları kullanılabilir.",
                cumle_satiri(&cumle),
                "Örnek: giriş \"program.dil\" olsun",
            ));
        };
        if !matches!(
            ad.as_str(),
            "proje"
                | "sürüm"
                | "morfoloji"
                | "giriş"
                | "yetkinlikler"
                | "ağ_hedefleri"
                | "yerel_bağımlılıklar"
        ) {
            return Err(proje_hatasi(
                "P001",
                &format!("\"{}\" proje bildirimi alanı değil.", ad),
                satir,
                "Geçerli alanlar: proje, sürüm, morfoloji, giriş, yetkinlikler, ağ_hedefleri, yerel_bağımlılıklar.",
            ));
        }
        if alanlar.insert(ad.clone(), (deger, satir)).is_some() {
            return Err(proje_hatasi(
                "P001",
                &format!("\"{}\" alanı birden çok kez tanımlandı.", ad),
                satir,
                "Her proje alanını yalnız bir kez tanımla.",
            ));
        }
    }

    let (ad, ad_satiri) = gerekli_metni_al(&mut alanlar, "proje")?;
    let (surum, surum_satiri) = gerekli_metni_al(&mut alanlar, "sürüm")?;
    let (morfoloji, morfoloji_satiri) = istege_bagli_metni_al(
        &mut alanlar,
        "morfoloji",
        crate::morfoloji::MORFOLOJI_PROFILI,
    )?;
    let (giris, giris_satiri) = gerekli_metni_al(&mut alanlar, "giriş")?;
    let yerel_bagimliliklar = bagimliliklari_al(&mut alanlar)?;
    let (yetkinlik_yazimlari, yetkinlik_satiri) =
        metin_listesini_al(&mut alanlar, "yetkinlikler", "P015")?;
    let (ag_hedef_yazimlari, ag_hedef_satiri) =
        metin_listesini_al(&mut alanlar, "ağ_hedefleri", "P015")?;
    let mut yetkinlikler = BTreeSet::new();
    for yazim in yetkinlik_yazimlari {
        let Some(yetkinlik) = Yetkinlik::ayristir(&yazim) else {
            return Err(proje_hatasi(
                "P015",
                &format!("\"{}\" bilinen bir proje yetkinliği değil.", yazim),
                yetkinlik_satiri,
                &format!(
                    "Geçerli yetkinlikler: {}.",
                    Yetkinlik::TUMU
                        .into_iter()
                        .map(Yetkinlik::yazimi)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        };
        if !yetkinlikler.insert(yetkinlik) {
            return Err(proje_hatasi(
                "P015",
                &format!("\"{}\" yetkinliği birden çok kez yazıldı.", yazim),
                yetkinlik_satiri,
                "Her yetkinliği yalnız bir kez bildir.",
            ));
        }
    }
    let mut ag_hedefleri = BTreeSet::new();
    for yazim in ag_hedef_yazimlari {
        let hedef = AgHedefi::bildirimden(&yazim).map_err(|neden| {
            proje_hatasi(
                "P015",
                &format!("Geçersiz ağ hedefi \"{}\": {}.", yazim, neden),
                ag_hedef_satiri,
                "Tam origin yaz; örnek: https://api.example.com veya http://127.0.0.1:8080",
            )
        })?;
        if !ag_hedefleri.insert(hedef) {
            return Err(proje_hatasi(
                "P015",
                &format!("\"{}\" ağ hedefi birden çok kez yazıldı.", yazim),
                ag_hedef_satiri,
                "Her şema+host+port hedefini yalnız bir kez bildir.",
            ));
        }
    }
    if yetkinlikler.contains(&Yetkinlik::YerelAg) && !yetkinlikler.contains(&Yetkinlik::Ag) {
        return Err(proje_hatasi(
            "P015",
            "`yerel-ağ` yetkinliği tek başına kullanılamaz.",
            yetkinlik_satiri,
            "Yetkinliklere `ağ`ı da ekle ve hedefleri açıkça bildir.",
        ));
    }
    if yetkinlikler.contains(&Yetkinlik::WebOturumu)
        && !yetkinlikler.contains(&Yetkinlik::AgSunucusu)
    {
        return Err(proje_hatasi(
            "P015",
            "`web-oturumu` yetkinliği tek başına kullanılamaz.",
            yetkinlik_satiri,
            "Yetkinliklere `ağ-sunucusu`nu da ekle.",
        ));
    }
    if yetkinlikler.contains(&Yetkinlik::Ag) == ag_hedefleri.is_empty() {
        return Err(proje_hatasi(
            "P015",
            "`ağ` yetkinliği ile `ağ_hedefleri` birlikte ve boş olmayan biçimde bildirilmelidir.",
            if yetkinlikler.contains(&Yetkinlik::Ag) {
                ag_hedef_satiri
            } else {
                yetkinlik_satiri
            },
            "Ağ gerekmiyorsa ikisini de boş bırak; gerekiyorsa `ağ` ve en az bir tam origin yaz.",
        ));
    }
    if ag_hedefleri
        .iter()
        .any(|hedef| hedef.semasi() == crate::yetkinlik::AgSemasi::Http)
        && !yetkinlikler.contains(&Yetkinlik::YerelAg)
    {
        return Err(proje_hatasi(
            "P015",
            "Düz http:// hedefi yalnız açık `yerel-ağ` yetkinliğiyle kullanılabilir.",
            ag_hedef_satiri,
            "Public ağ için https:// kullan; yerel geliştirme gerekiyorsa `yerel-ağ`ı ayrıca bildir.",
        ));
    }

    if ad.trim().is_empty() || ad.chars().any(char::is_control) {
        return Err(proje_hatasi(
            "P003",
            "Proje adı boş ya da denetim karakterli olamaz.",
            ad_satiri,
            "Kısa ve ayırt edici bir proje adı yaz.",
        ));
    }
    if !gecerli_surum(&surum) {
        return Err(proje_hatasi(
            "P003",
            &format!("\"{}\" geçerli bir proje sürümü değil.", surum),
            surum_satiri,
            "Sürümü üç sayıyla yaz: 0.1.0",
        ));
    }
    if morfoloji != crate::morfoloji::MORFOLOJI_PROFILI {
        return Err(proje_hatasi(
            "P011",
            &format!("\"{}\" morfoloji profili bu derleyicide desteklenmiyor.", morfoloji),
            morfoloji_satiri,
            &format!(
                "Bu sürüm için `morfoloji \"{}\" olsun` yaz.",
                crate::morfoloji::MORFOLOJI_PROFILI
            ),
        ));
    }
    if let Err(neden) = girisi_dogrula(&giris) {
        return Err(proje_hatasi(
            "P004",
            &format!("Geçersiz giriş dosyası \"{}\": {}.", giris, neden),
            giris_satiri,
            "Giriş proje içinde kalan göreli bir .dil yolu olmalı; örnek: program.dil",
        ));
    }

    Ok(ProjeBildirimi {
        ad,
        surum,
        morfoloji,
        giris,
        yetkinlikler,
        ag_hedefleri,
        yerel_bagimliliklar,
    })
}

impl ProjeBildirimi {
    pub fn yetkinlik_politikasi(&self) -> YetkinlikPolitikasi {
        YetkinlikPolitikasi::proje(self.yetkinlikler.clone(), self.ag_hedefleri.clone())
    }
}

fn istege_bagli_metni_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    ad: &str,
    varsayilan: &str,
) -> Result<(String, usize), Tani> {
    let Some((ifade, satir)) = alanlar.remove(ad) else {
        return Ok((varsayilan.to_string(), 1));
    };
    match ifade.turu() {
        Ifade::MetinSabiti(deger) => Ok((deger.clone(), satir)),
        _ => Err(proje_hatasi(
            "P001",
            &format!("\"{}\" alanı Metin olmalı.", ad),
            satir,
            &format!("Örnek: {} \"...\" olsun", ad),
        )),
    }
}

/// Yerel bağımlılık alanını resmî biçimde günceller. Önce eski bildirim,
/// sonra üretilen bildirim doğrulanır; hata varsa çağıran hiçbir şey yazmaz.
/// Yorumlar ve diğer alanlar korunur, yollar sıralanıp tekilleştirilir.
pub fn yerel_bagimliliklari_guncelle(
    kaynak: &str,
    yollar: &[String],
) -> Result<String, Tani> {
    bildirimi_oku(kaynak)?;
    let mut yollar = yollar.to_vec();
    yollar.sort();
    yollar.dedup();
    for yol in &yollar {
        if let Err(neden) = bagimlilik_yolunu_dogrula(yol) {
            return Err(proje_hatasi(
                "P005",
                &format!("Geçersiz yerel bağımlılık yolu \"{}\": {}.", yol, neden),
                1,
                "Göreli bir proje klasörü yaz; örnek: ../ortak",
            ));
        }
    }

    let ifade = if yollar.is_empty() {
        "boş liste".to_string()
    } else {
        let ogeler = yollar
            .iter()
            .map(|yol| format!("\"{}\"", metni_kacir(yol)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} listesi", ogeler)
    };
    let yeni_satir = format!("yerel_bağımlılıklar {} olsun", ifade);
    let mut satirlar = Vec::new();
    let mut degisti = false;
    for satir in kaynak.lines() {
        let kirpilmis = satir.trim_start();
        if !kirpilmis.starts_with('#')
            && kirpilmis.split_whitespace().next() == Some("yerel_bağımlılıklar")
        {
            satirlar.push(yeni_satir.clone());
            degisti = true;
        } else {
            satirlar.push(satir.to_string());
        }
    }
    if !degisti {
        if satirlar.last().is_some_and(|satir| !satir.is_empty()) {
            satirlar.push(String::new());
        }
        satirlar.push(yeni_satir);
    }
    let aday = format!("{}\n", satirlar.join("\n"));
    bildirimi_oku(&aday)?;
    crate::bicimleyici::bicimle(&aday)
}

fn gerekli_metni_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    ad: &str,
) -> Result<(String, usize), Tani> {
    let (ifade, satir) = alanlar.remove(ad).ok_or_else(|| {
        proje_hatasi(
            "P002",
            &format!("Proje bildiriminde \"{}\" alanı eksik.", ad),
            1,
            &format!("{} \"...\" olsun satırını ekle.", ad),
        )
    })?;
    match ifade.turu() {
        Ifade::MetinSabiti(deger) => Ok((deger.clone(), satir)),
        _ => Err(proje_hatasi(
            "P001",
            &format!("\"{}\" alanı Metin olmalı.", ad),
            satir,
            &format!("Örnek: {} \"...\" olsun", ad),
        )),
    }
}

fn bagimliliklari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
) -> Result<Vec<String>, Tani> {
    let Some((ifade, satir)) = alanlar.remove("yerel_bağımlılıklar") else {
        return Ok(Vec::new());
    };
    let yollar = match ifade.turu() {
        Ifade::BosListe => Vec::new(),
        Ifade::ListeSabiti(ogeler) => ogeler
            .iter()
            .map(|oge| match oge.turu() {
                Ifade::MetinSabiti(yol) => Ok(yol.clone()),
                _ => Err(proje_hatasi(
                    "P005",
                    "Yerel bağımlılıkların her biri Metin yol olmalı.",
                    satir,
                    "Örnek: yerel_bağımlılıklar \"../ortak\", \"../grafik\" listesi olsun",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(proje_hatasi(
                "P005",
                "\"yerel_bağımlılıklar\" bir Metin listesi olmalı.",
                satir,
                "Örnek: yerel_bağımlılıklar \"../ortak\" listesi olsun",
            ));
        }
    };

    let mut gorulen = std::collections::HashSet::new();
    for yol in &yollar {
        if let Err(neden) = bagimlilik_yolunu_dogrula(yol) {
            return Err(proje_hatasi(
                "P005",
                &format!("Geçersiz yerel bağımlılık yolu \"{}\": {}.", yol, neden),
                satir,
                "Göreli bir proje klasörü yaz; örnek: ../ortak",
            ));
        }
        if !gorulen.insert(yol) {
            return Err(proje_hatasi(
                "P005",
                &format!("\"{}\" yerel bağımlılığı birden çok kez yazıldı.", yol),
                satir,
                "Her yerel proje yolunu yalnız bir kez yaz.",
            ));
        }
    }
    Ok(yollar)
}

fn metin_listesini_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    ad: &str,
    kod: &str,
) -> Result<(Vec<String>, usize), Tani> {
    let Some((ifade, satir)) = alanlar.remove(ad) else {
        return Ok((Vec::new(), 1));
    };
    let yazimlar = match ifade.turu() {
        Ifade::BosListe => Vec::new(),
        Ifade::ListeSabiti(ogeler) => ogeler
            .iter()
            .map(|oge| match oge.turu() {
                Ifade::MetinSabiti(yazim) => Ok(yazim.clone()),
                _ => Err(proje_hatasi(
                    kod,
                    &format!("\"{}\" alanındaki her öğe Metin olmalı.", ad),
                    satir,
                    &format!("Örnek: {} \"değer\" listesi olsun", ad),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(proje_hatasi(
                kod,
                &format!("\"{}\" bir Metin listesi olmalı.", ad),
                satir,
                &format!("Örnek: {} boş liste olsun", ad),
            ));
        }
    };
    Ok((yazimlar, satir))
}

fn gecerli_surum(surum: &str) -> bool {
    let parcalar: Vec<&str> = surum.split('.').collect();
    parcalar.len() == 3
        && parcalar.iter().all(|p| {
            !p.is_empty()
                && p.chars().all(|k| k.is_ascii_digit())
                && (p == &"0" || !p.starts_with('0'))
        })
}

fn girisi_dogrula(giris: &str) -> Result<(), &'static str> {
    if giris.is_empty() {
        return Err("yol boş");
    }
    if giris.contains(['\\', ':']) {
        return Err("platforma bağlı yol işareti kullanılamaz");
    }
    let yol = Path::new(giris);
    if yol.is_absolute() {
        return Err("mutlak yol kullanılamaz");
    }
    if yol.extension().and_then(|e| e.to_str()) != Some("dil") {
        return Err("uzantı .dil olmalı");
    }
    if yol.components().any(|b| !matches!(b, Component::Normal(_))) {
        return Err("yol proje dışına çıkamaz");
    }
    Ok(())
}

fn bagimlilik_yolunu_dogrula(yol: &str) -> Result<(), &'static str> {
    if yol.is_empty() || yol.chars().any(char::is_control) {
        return Err("yol boş ya da denetim karakterli");
    }
    if yol.contains(['\\', ':']) {
        return Err("platforma bağlı yol işareti kullanılamaz");
    }
    let yol = Path::new(yol);
    if yol.is_absolute() {
        return Err("mutlak yol kullanılamaz");
    }
    if yol
        .components()
        .any(|b| matches!(b, Component::RootDir | Component::Prefix(_)))
    {
        return Err("yol göreli olmalı");
    }
    if !yol
        .components()
        .any(|b| matches!(b, Component::Normal(_) | Component::ParentDir))
    {
        return Err("yol bir proje klasörü göstermeli");
    }
    Ok(())
}

fn metni_kacir(metin: &str) -> String {
    metin.replace('\\', "\\\\").replace('"', "\\\"")
}

fn cumle_satiri(cumle: &Cumle) -> usize {
    match cumle {
        Cumle::Yaz { satir, .. }
        | Cumle::Olsun { satir, .. }
        | Cumle::KezTekrarla { satir, .. }
        | Cumle::AralikDongusu { satir, .. }
        | Cumle::OlduguSurece { satir, .. }
        | Cumle::OlanaKadar { satir, .. }
        | Cumle::Ise { satir, .. }
        | Cumle::Artir { satir, .. }
        | Cumle::Azalt { satir, .. }
        | Cumle::Sor { satir, .. }
        | Cumle::Ekle { satir, .. }
        | Cumle::HerBiri { satir, .. }
        | Cumle::Gore { satir, .. }
        | Cumle::Kullan { satir, .. }
        | Cumle::Olmali { satir, .. }
        | Cumle::AlanAta { satir, .. }
        | Cumle::Dondur { satir, .. }
        | Cumle::HataDondur { satir, .. }
        | Cumle::BolVeAta { satir, .. }
        | Cumle::CagriCumlesi { satir, .. }
        | Cumle::ProgramiBitir { satir, .. }
        | Cumle::SunucuBaslat { satir, .. }
        | Cumle::IstekGeldiginde { satir, .. }
        | Cumle::YanitGonder { satir, .. }
        | Cumle::Yonlendir { satir, .. }
        | Cumle::CerezYaz { satir, .. }
        | Cumle::Sil { satir, .. }
        | Cumle::CerezSil { satir, .. }
        | Cumle::RotaPolitikasi { satir, .. }
        | Cumle::RotaAlaniGerekli { satir, .. }
        | Cumle::OturumAc { satir, .. }
        | Cumle::OturumKapat { satir, .. }
        | Cumle::Eszamanli { satir, .. }
        | Cumle::HepsiniBekle { satir, .. }
        | Cumle::IcindeBlogu { satir, .. }
        | Cumle::IsikAyarla { satir, .. }
        | Cumle::Bekle { satir, .. }
        | Cumle::SozlukAta { satir, .. }
        | Cumle::DosyayaYaz { satir, .. } => *satir,
        Cumle::IslemTanimi(islem) => islem.satir,
        Cumle::YapiTanimi(yapi) => yapi.satir,
        Cumle::TestBlogu(test) => test.satir,
    }
}

fn proje_hatasi(kod: &str, mesaj: &str, satir: usize, oneri: &str) -> Tani {
    Tani::yeni(kod, mesaj.into(), satir, 1, 1).onerili(oneri.into())
}
