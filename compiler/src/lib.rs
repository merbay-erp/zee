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

//! Türkçe programlama dili — bootstrap derleyici (Stage 0).
//!
//! Boru hattı (master plan bölüm 11'in v0 dilimi):
//! kaynak → sözcükleyici → ayrıştırıcı → ad çözümleme + tür denetimi → yorumlayıcı.

pub mod api;

#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub mod ag_istemcisi;
#[doc(hidden)]
pub mod agac;
#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub mod artefakt_dogrulama;
#[doc(hidden)]
pub mod ayristirici;
#[doc(hidden)]
pub mod bicimleyici;
#[doc(hidden)]
pub mod cozumleyici;
#[doc(hidden)]
pub mod faz;
#[doc(hidden)]
pub mod guvenlik;
#[doc(hidden)]
pub mod hir;
#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub mod http_istegi;
#[doc(hidden)]
pub mod intrinsic;
#[doc(hidden)]
pub mod invariant;
#[doc(hidden)]
pub mod kalici_dosya;
#[doc(hidden)]
pub mod kaynak_sinirlari;
#[doc(hidden)]
pub mod kimlik;
#[doc(hidden)]
pub mod lsp;
#[doc(hidden)]
pub mod morfoloji;
mod ondalik;
#[doc(hidden)]
pub mod paket;
#[doc(hidden)]
pub mod paket_modeli;
#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub mod postgresql;
#[doc(hidden)]
pub mod proje;
#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub mod registry;
mod semantic_model;
#[doc(hidden)]
pub mod sozcukleyici;
#[doc(hidden)]
pub mod tani;
mod tani_politikasi;
#[doc(hidden)]
pub mod veritabani_modeli;
#[doc(hidden)]
pub mod wasm_api;
#[doc(hidden)]
pub mod web_guvenligi;
#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub mod yayin;
#[doc(hidden)]
pub mod yetkinlik;
#[doc(hidden)]
pub mod yorumlayici;
#[doc(hidden)]
pub mod zaman;

use agac::{Cumle, Islem, KullanimTuru, Program, Test, Yapi};
use faz::{BaglanmamisProgram, BaglanmisProgram, KaynakMetni};
use std::collections::{BTreeSet, HashMap};
use tani::Tani;

/// Birim yükleyici: birim adını kaynak metne çevirir (RFC-0009). CLI gerçek
/// dosya sisteminden okur; testler sahte tablodan verir — birim çözümü de
/// determinizm ilkesine uyar.
pub type BirimYukleyici<'a> = dyn FnMut(&str) -> Result<String, String> + 'a;

/// Kaynak kökenini koruyan yükleme isteği. `isteyen`, kullanımı yapan gerçek
/// dosyanın yükleyici tarafından verilmiş kimliğidir; böylece iç içe birimler
/// kendi klasörlerinden çözülür ve paket sınırları kaybolmaz (K-078).
pub struct BirimIstegi<'a> {
    pub ad: &'a str,
    pub tur: KullanimTuru,
    pub isteyen: Option<&'a str>,
}

/// Yüklenen kaynakla birlikte bir sonraki çözümün dayanacağı kararlı köken.
pub struct YuklenenBirim {
    pub kaynak: String,
    pub koken: String,
}

pub type KokenliBirimYukleyici<'a> =
    dyn for<'b> FnMut(BirimIstegi<'b>) -> Result<YuklenenBirim, String> + 'a;

/// Kaynağı çalıştırılabilir programa derler. Birim kullanmayan kaynaklar için;
/// `kullan` görülürse A010 verir (yükleyici bağlanmamış).
pub fn kaynagi_derle(kaynak: &str) -> Result<Program, Tani> {
    kaynagi_fazli_derle(kaynak).map(BaglanmisProgram::into_program)
}

/// Kaynağı faz bilgisini silmeden derler. Yeni derleyici/runtime kodu bu
/// yüzeyi tercih eder; `kaynagi_derle` geriye uyum için `Program` döndürür.
pub fn kaynagi_fazli_derle(kaynak: &str) -> Result<BaglanmisProgram, Tani> {
    kaynagi_fazli_derle_birimlerle(kaynak, &mut |ad: &str| {
        Err(format!("\"{}\" birimi bu bağlamda yüklenemez", ad))
    })
}

/// Kaynağı, birimlerini yükleyerek derler (sözcükle + birim çözümü +
/// tohumlu ayrıştırma + hoist + denetle).
/// Gömülü standart kitaplık (RFC-0014, deneysel): standart birimler ikiliye
/// gömülüdür — kurulumsuz ve internetsiz çalışır, playground dahil.
/// Çözüm sırası: önce yerel klasör, bulunamazsa buradaki gömülü kaynak.
pub fn gomulu_birim(ad: &str) -> Option<&'static str> {
    match ad {
        "matematik" => Some(include_str!("../../kitaplik/matematik.dil")),
        "liste_araclari" => Some(include_str!("../../kitaplik/liste_araclari.dil")),
        "metin_araclari" => Some(include_str!("../../kitaplik/metin_araclari.dil")),
        "sozluk_araclari" => Some(include_str!("../../kitaplik/sozluk_araclari.dil")),
        _ => None,
    }
}

/// Gömülü birim adları (A010 tanısında listelenir).
pub fn gomulu_birim_adlari() -> &'static [&'static str] {
    &[
        "matematik",
        "liste_araclari",
        "metin_araclari",
        "sozluk_araclari",
    ]
}

/// Bir birimin insan-okur özeti: işlem/eylem başlıkları (parametreleriyle) ve
/// test sayısı. `dil belge <birim>` bunun üstüne kuruludur (RFC-0014 §8.2).
pub fn birim_ozeti(kaynak: &str) -> String {
    let mut cikti = String::new();
    let mut test_sayisi = 0usize;
    let satirlar: Vec<&str> = kaynak.lines().collect();
    for (i, satir) in satirlar.iter().enumerate() {
        if let Some((tur, ad)) = satir
            .strip_prefix("işlem ")
            .map(|ad| ("işlem", ad))
            .or_else(|| satir.strip_prefix("eylem ").map(|ad| ("eylem", ad)))
        {
            // K-066 eki: işlemin hemen üstündeki # satırları açıklamadır.
            let mut aciklama = Vec::new();
            for onceki in satirlar[..i].iter().rev() {
                match onceki.strip_prefix('#') {
                    Some(metin) => aciklama.push(metin.trim().to_string()),
                    None => break,
                }
            }
            aciklama.reverse();
            let mut parametreler = Vec::new();
            let mut donus = None;
            for devam in satirlar.iter().skip(i + 1) {
                let kirpik = devam.trim();
                if kirpik.is_empty() {
                    continue;
                }
                if devam.starts_with("    ") && kirpik.ends_with(" al") {
                    parametreler.push(kirpik.to_string());
                    continue;
                }
                if kirpik == "değer döndürmez" {
                    donus = Some("değer döndürmez".to_string());
                } else if let Some(tur) = kirpik.strip_suffix(" döndürür") {
                    donus = Some(tur.to_string());
                }
                break;
            }
            cikti.push_str(&format!("  {} {}", tur, ad));
            if !parametreler.is_empty() {
                cikti.push_str(&format!("  ({})", parametreler.join(", ")));
            }
            if let Some(donus) = donus {
                cikti.push_str(&format!(" → {}", donus));
            }
            cikti.push('\n');
            for satir in &aciklama {
                cikti.push_str(&format!("      # {}\n", satir));
            }
        } else if satir.starts_with("test ") {
            test_sayisi += 1;
        }
    }
    cikti.push_str(&format!("  testler: {}\n", test_sayisi));
    cikti
}

pub fn kaynagi_derle_birimlerle(
    kaynak: &str,
    yukleyici: &mut BirimYukleyici,
) -> Result<Program, Tani> {
    kaynagi_fazli_derle_birimlerle(kaynak, yukleyici).map(BaglanmisProgram::into_program)
}

pub fn kaynagi_fazli_derle_birimlerle(
    kaynak: &str,
    yukleyici: &mut BirimYukleyici,
) -> Result<BaglanmisProgram, Tani> {
    let mut kokenli = |istek: BirimIstegi<'_>| {
        if istek.tur == KullanimTuru::Paket {
            return Err("paket çözümü için proje bağlamı gerekir".into());
        }
        yukleyici(istek.ad).map(|kaynak| YuklenenBirim {
            kaynak,
            koken: istek.ad.to_string(),
        })
    };
    kaynagi_fazli_derle_kokenlerle(kaynak, None, &mut kokenli)
}

/// Kaynağı gerçek dosya/paket kökenini koruyarak derler. CLI ve proje araçları
/// bu yüzeyi kullanır; eski `kaynagi_derle_birimlerle` API'si korunmuştur.
pub fn kaynagi_derle_kokenlerle(
    kaynak: &str,
    koken: Option<&str>,
    yukleyici: &mut KokenliBirimYukleyici,
) -> Result<Program, Tani> {
    kaynagi_fazli_derle_kokenlerle(kaynak, koken, yukleyici).map(BaglanmisProgram::into_program)
}

pub fn kaynagi_fazli_derle_kokenlerle(
    kaynak: &str,
    koken: Option<&str>,
    yukleyici: &mut KokenliBirimYukleyici,
) -> Result<BaglanmisProgram, Tani> {
    let mut yigin: Vec<String> = Vec::new();
    let cozulmus = dosyayi_coz(kaynak, koken, true, yukleyici, &mut yigin)?;
    BaglanmamisProgram::yeni(Program {
        cumleler: cozulmus.cumleler,
        islemler: cozulmus.islemler,
        yapilar: cozulmus.yapilar,
        testler: cozulmus.testler,
    })
    .denetle()
}

/// Bir kaynak dosyanın çözülmüş tanımları. `islemler`/`yapilar` çalışma
/// için gereken geçişli tanımları da taşır; `*_kokenleri` her adın
/// TANIMLANDIĞI kaynağı söyler ki elmas içe alım çakışma sayılmasın
/// (spec/07, K-164/ADR-072). `alinan_kokenler` bu düzeyde birleşen bütün
/// kaynak kökenleridir; birim testleri kökeni başına bir kez alınır.
struct CozulmusDosya {
    cumleler: Vec<Cumle>,
    islemler: HashMap<String, Islem>,
    islem_kokenleri: HashMap<String, String>,
    yapilar: Vec<Yapi>,
    yapi_kokenleri: HashMap<String, String>,
    testler: Vec<Test>,
    alinan_kokenler: BTreeSet<String>,
}

/// Birimden dönen tanıyı, henüz kökeni yoksa o birimin kökeniyle etiketler.
fn birim_kokeniyle(tani: Tani, koken: &str) -> Tani {
    tani.kokenle(koken)
}

/// Bir kaynak dosyayı çözer: birimlerini özyinelemeli yükler, kendi
/// tanımlarını içe alınanlarla birleştirir. `birim_adi` None ise ana dosyadır.
fn dosyayi_coz(
    kaynak: &str,
    kaynak_kokeni: Option<&str>,
    ana_kaynak: bool,
    yukleyici: &mut KokenliBirimYukleyici,
    yigin: &mut Vec<String>,
) -> Result<CozulmusDosya, Tani> {
    let tokenlar = KaynakMetni::yeni(kaynak).sozcukle()?;
    let bu_koken = kaynak_kokeni.unwrap_or("").to_string();

    // 1) Birimleri önden yükle (çağrı tanıma için işlem adları gerekli).
    let mut islemler: HashMap<String, Islem> = HashMap::new();
    let mut islem_kaynagi: HashMap<String, String> = HashMap::new();
    let mut islem_kokenleri: HashMap<String, String> = HashMap::new();
    let mut yapilar: Vec<Yapi> = Vec::new();
    let mut yapi_kaynagi: HashMap<String, String> = HashMap::new();
    let mut yapi_kokenleri: HashMap<String, String> = HashMap::new();
    let mut testler: Vec<Test> = Vec::new();
    let mut alinan_kokenler: BTreeSet<String> = BTreeSet::new();

    for (ad, tur, satir) in ayristirici::kullanilan_birimler(tokenlar.tokenlar()) {
        let yuklenen = yukleyici(BirimIstegi {
            ad: &ad,
            tur,
            isteyen: kaynak_kokeni,
        })
        .map_err(|hata| yukleme_hatasi(&ad, tur, &hata, satir))?;
        if yigin.iter().any(|y| y == &yuklenen.koken)
            || kaynak_kokeni == Some(yuklenen.koken.as_str())
        {
            return Err(Tani::yeni(
                "A009",
                format!(
                    "Kaynaklar birbirini döngüsel kullanıyor: {} → {}.",
                    yigin.join(" → "),
                    yuklenen.koken
                ),
                satir,
                1,
                1,
            )
            .onerili("Ortak tanımları üçüncü bir birime ya da pakete taşı.".into()));
        }
        yigin.push(yuklenen.koken.clone());
        let birim = dosyayi_coz(
            &yuklenen.kaynak,
            Some(&yuklenen.koken),
            false,
            yukleyici,
            yigin,
        )
        .map_err(|tani| birim_kokeniyle(tani, &yuklenen.koken));
        yigin.pop();
        let birim = birim?;

        birim_tanimlarini_birlestir(
            &ad,
            satir,
            birim,
            &mut islemler,
            &mut islem_kaynagi,
            &mut islem_kokenleri,
            &mut yapilar,
            &mut yapi_kaynagi,
            &mut yapi_kokenleri,
            &mut testler,
            &mut alinan_kokenler,
        )?;
    }

    // 2) İçe alınan işlem adlarıyla tohumlanmış ayrıştırma.
    let tohum: Vec<String> = islemler
        .values()
        .filter(|islem| islem.disari_acik)
        .map(|islem| islem.ad.clone())
        .collect();
    let cumleler = tokenlar.ayristir(tohum)?.into_cumleler();

    // 3) Kendi tanımlarını ayıkla ve birleştir.
    let (kalan, kendi_islem_adlari) = kendi_tanimlarini_ayikla(
        cumleler,
        ana_kaynak,
        kaynak_kokeni,
        &bu_koken,
        &islem_kaynagi,
        &yapi_kaynagi,
        &mut islemler,
        &mut islem_kokenleri,
        &mut yapilar,
        &mut yapi_kokenleri,
        &mut testler,
    )?;
    let mut kalan = kalan;
    alinan_kokenler.insert(bu_koken);

    if !ana_kaynak {
        // Alınan işlemler bu dosyanın public API'sine örtük yeniden açılmaz.
        // Gövde/runtime çağrıları için haritada kalır; yalnız bu kaynağın
        // doğrudan tanımları bir üst kaynağın çağrı yüzeyine çıkar.
        for islem in islemler.values_mut() {
            islem.disari_acik = false;
        }
        for ad in &kendi_islem_adlari {
            let islem = islemler.get_mut(ad).ok_or_else(|| {
                Tani::yeni(
                    "T016",
                    format!("\"{}\" işlemi birim tablosundan kayboldu — derleyici iç hatası olabilir, bildir.", ad),
                    1,
                    1,
                    1,
                )
            })?;
            islem.disari_acik = true;
        }
        disari_acik_imzalari_denetle(&islemler, kaynak_kokeni.unwrap_or("adı bilinmeyen birim"))?;
    }

    // Birim olarak yüklenen dosyanın üst düzey cümleleri İÇE ALINMAZ
    // (kapsülleme, RFC-0009 §2): dosya kendi başına çalıştırılabilir kalır,
    // birim olarak yalnız tanımlarını verir.
    if !ana_kaynak {
        kalan.clear();
    }

    Ok(CozulmusDosya {
        cumleler: kalan,
        islemler,
        islem_kokenleri,
        yapilar,
        yapi_kokenleri,
        testler,
        alinan_kokenler,
    })
}

/// Dosyanın kendi işlem/yapı/test tanımlarını içe alınanlarla birleştirir;
/// üst düzey cümleleri ve kendi işlem adlarını verir. Birim olarak yüklenen
/// dosyanın tanımları kökeniyle etiketlenir (K-164/ADR-072).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn kendi_tanimlarini_ayikla(
    cumleler: Vec<Cumle>,
    ana_kaynak: bool,
    kaynak_kokeni: Option<&str>,
    bu_koken: &str,
    islem_kaynagi: &HashMap<String, String>,
    yapi_kaynagi: &HashMap<String, String>,
    islemler: &mut HashMap<String, Islem>,
    islem_kokenleri: &mut HashMap<String, String>,
    yapilar: &mut Vec<Yapi>,
    yapi_kokenleri: &mut HashMap<String, String>,
    testler: &mut Vec<Test>,
) -> Result<(Vec<Cumle>, Vec<String>), Tani> {
    let mut kalan = Vec::new();
    let mut kendi_islem_adlari = Vec::new();
    for cumle in cumleler {
        match cumle {
            Cumle::Kullan { .. } => {} // 1. adımda çözüldü
            Cumle::IslemTanimi(mut islem) => {
                if let Some(birim) = islem_kaynagi.get(&islem.ad) {
                    return Err(cakisma("işlemi", &islem.ad, birim, "bu dosya", islem.satir));
                }
                if islemler.contains_key(&islem.ad) {
                    return Err(Tani::yeni(
                        "A005",
                        format!("\"{}\" işlemi birden çok kez tanımlandı.", islem.ad),
                        islem.satir,
                        1,
                        1,
                    ));
                }
                if !ana_kaynak {
                    islem.koken = kaynak_kokeni.map(str::to_string);
                }
                kendi_islem_adlari.push(islem.ad.clone());
                islem_kokenleri.insert(islem.ad.clone(), bu_koken.to_string());
                islemler.insert(islem.ad.clone(), islem);
            }
            Cumle::TestBlogu(mut test) => {
                if !ana_kaynak {
                    test.koken = kaynak_kokeni.map(str::to_string);
                }
                testler.push(test);
            }
            Cumle::YapiTanimi(yapi) => {
                if let Some(birim) = yapi_kaynagi.get(&yapi.ad) {
                    return Err(cakisma("yapısı", &yapi.ad, birim, "bu dosya", yapi.satir));
                }
                if yapilar.iter().any(|y| y.ad == yapi.ad) {
                    return Err(Tani::yeni(
                        "A006",
                        format!("\"{}\" yapısı birden çok kez tanımlandı.", yapi.ad),
                        yapi.satir,
                        1,
                        1,
                    ));
                }
                yapi_kokenleri.insert(yapi.ad.clone(), bu_koken.to_string());
                yapilar.push(yapi);
            }
            baska => kalan.push(baska),
        }
    }
    Ok((kalan, kendi_islem_adlari))
}

/// `ad` birimiyle gelen tanımları üst dosyaya katar. Aynı ad daha önce
/// AYNI kökenden geldiyse (elmas içe alım) sessizce atlanır; farklı kökenden
/// geldiyse A008'dir. Birim testleri kökeni başına yalnız bir kez alınır.
#[allow(clippy::too_many_arguments)]
fn birim_tanimlarini_birlestir(
    ad: &str,
    satir: usize,
    birim: CozulmusDosya,
    islemler: &mut HashMap<String, Islem>,
    islem_kaynagi: &mut HashMap<String, String>,
    islem_kokenleri: &mut HashMap<String, String>,
    yapilar: &mut Vec<Yapi>,
    yapi_kaynagi: &mut HashMap<String, String>,
    yapi_kokenleri: &mut HashMap<String, String>,
    testler: &mut Vec<Test>,
    alinan_kokenler: &mut BTreeSet<String>,
) -> Result<(), Tani> {
    for (islem_adi, islem) in birim.islemler {
        let koken = birim
            .islem_kokenleri
            .get(&islem_adi)
            .cloned()
            .unwrap_or_default();
        if let Some(onceki) = islem_kaynagi.get(&islem_adi) {
            if islem_kokenleri.get(&islem_adi) == Some(&koken) {
                continue;
            }
            return Err(cakisma("işlemi", &islem_adi, onceki, ad, satir));
        }
        islem_kaynagi.insert(islem_adi.clone(), ad.to_string());
        islem_kokenleri.insert(islem_adi.clone(), koken);
        islemler.insert(islem_adi, islem);
    }
    for yapi in birim.yapilar {
        let koken = birim
            .yapi_kokenleri
            .get(&yapi.ad)
            .cloned()
            .unwrap_or_default();
        if let Some(onceki) = yapi_kaynagi.get(&yapi.ad) {
            if yapi_kokenleri.get(&yapi.ad) == Some(&koken) {
                continue;
            }
            return Err(cakisma("yapısı", &yapi.ad, onceki, ad, satir));
        }
        yapi_kaynagi.insert(yapi.ad.clone(), ad.to_string());
        yapi_kokenleri.insert(yapi.ad.clone(), koken);
        yapilar.push(yapi);
    }
    // Birimin testleri de görünür olur (RFC-0009 §2); ad birimle önek alır.
    // Elmas yoldan ikinci kez gelen köken testlerini yinelemez.
    for mut test in birim.testler {
        let test_kokeni = test.koken.clone().unwrap_or_default();
        if alinan_kokenler.contains(&test_kokeni) {
            continue;
        }
        if !test.ad.starts_with(&format!("{}: ", ad)) {
            test.ad = format!("{}: {}", ad, test.ad);
        }
        testler.push(test);
    }
    alinan_kokenler.extend(birim.alinan_kokenler);
    Ok(())
}

/// K-086: Bir dosya birim ya da paket olarak alındığında içindeki bütün
/// işlemler public yüzeye çıkar. Bu sınırda çağrı-güdümlü çıkarım yasaktır;
/// parametreler ve dönüş kaynakta okunabilir, monomorfik bir sözleşme taşır.
fn disari_acik_imzalari_denetle(
    islemler: &HashMap<String, Islem>,
    kaynak: &str,
) -> Result<(), Tani> {
    let mut adlar = islemler.keys().cloned().collect::<Vec<_>>();
    adlar.sort();
    for ad in adlar {
        let Some(islem) = islemler.get(&ad) else {
            continue;
        };
        if !islem.disari_acik {
            continue;
        }
        let parametreler_acik = islem
            .parametreler
            .iter()
            .all(|parametre| parametre.tur_yazimi.is_some());
        if !parametreler_acik || islem.donus_turu_yazimi.is_none() {
            return Err(Tani::yeni(
                "T039",
                format!(
                    "\"{}\" işlemi \"{}\" dışa açık yüzeyinde tam tür sözleşmesi taşımıyor.",
                    islem.ad, kaynak
                ),
                islem.satir,
                1,
                1,
            )
            .onerili(
                "Her parametreyi `<ad> <Tür> olarak al` yaz; ardından `<Tür> döndürür` ya da `değer döndürmez` ekle."
                    .into(),
            ));
        }
    }
    Ok(())
}

fn yukleme_hatasi(ad: &str, tur: KullanimTuru, hata: &str, satir: usize) -> Tani {
    match tur {
        KullanimTuru::Birim => Tani::yeni(
            "A010",
            format!("\"{}\" birimi yüklenemedi: {}.", ad, hata),
            satir,
            1,
            1,
        )
        .onerili(format!(
            "Kullanan kaynakla aynı klasörde {}.dil dosyası olmalı (RFC-0009).",
            ad
        )),
        KullanimTuru::Paket => Tani::yeni(
            "A011",
            format!("\"{}\" paketi yüklenemedi: {}.", ad, hata),
            satir,
            1,
            1,
        )
        .onerili(format!(
            "{} projesini yerel_bağımlılıklar listesine ekle ve `dil kilitle .` çalıştır.",
            ad
        )),
    }
}

fn cakisma(tur: &str, ad: &str, birinci: &str, ikinci: &str, satir: usize) -> Tani {
    Tani::yeni(
        "A008",
        format!(
            "\"{}\" {} iki kaynaktan geliyor: \"{}\" ve \"{}\". Sessiz gölgeleme yoktur.",
            ad, tur, birinci, ikinci
        ),
        satir,
        1,
        1,
    )
    .onerili("Adlardan birini değiştir ya da tek kaynakta topla (RFC-0009 §2.3).".into())
}

/// Kaynağı denetler, çalıştırmaz.
pub fn kaynagi_denetle(kaynak: &str) -> Result<(), Tani> {
    kaynagi_fazli_derle(kaynak).map(|_| ())
}

/// Kaynağı uçtan uca çalıştırır; çıktı satırlarını döndürür.
pub fn kaynagi_calistir(kaynak: &str) -> Result<Vec<String>, Tani> {
    kaynagi_calistir_girdiyle(kaynak, Vec::new())
}

/// Kaynağı hazır girdi satırlarıyla çalıştırır ("diye sor" cevapları sırayla
/// bu listeden gelir); istemler de çıktıya dahildir.
pub fn kaynagi_calistir_girdiyle(kaynak: &str, girdiler: Vec<String>) -> Result<Vec<String>, Tani> {
    let program = kaynagi_fazli_derle(kaynak)?;
    let mut io = yorumlayici::ToplayanIo::yeni(girdiler);
    yorumlayici::calistir_baglanmis_io(&program, &mut io)?;
    Ok(io.cikti)
}

/// Bir testin sonucu: `hata` None ise geçti.
pub struct TestSonucu {
    pub ad: String,
    pub hata: Option<Tani>,
}

/// Birimden alınan testin tanısını o birimin kökeniyle etiketler.
fn test_kokeniyle(tani: Tani, test: &Test) -> Tani {
    match &test.koken {
        Some(koken) => tani.kokenle(koken),
        None => tani,
    }
}

/// Programın testlerini koşar. v0: her test taze ortamda ve dış dünyaya
/// dokunmayan hermetik IO ile çalışır (gerçek dosya/ağ erişimi yok).
pub fn programi_dene(program: &Program) -> Vec<TestSonucu> {
    program
        .testler
        .iter()
        .map(|test| {
            let mut io = yorumlayici::ToplayanIo::yeni(Vec::new());
            let hata = yorumlayici::test_calistir(program, test, &mut io)
                .map_err(|tani| test_kokeniyle(tani, test))
                .err();
            TestSonucu {
                ad: test.ad.clone(),
                hata,
            }
        })
        .collect()
}

/// Faz bilgisini koruyan programın testlerini typed HIR bağlarıyla koşar.
pub fn programi_dene_baglanmis(program: &BaglanmisProgram) -> Vec<TestSonucu> {
    program
        .testler
        .iter()
        .map(|test| {
            let mut io = yorumlayici::ToplayanIo::yeni(Vec::new());
            let hata = yorumlayici::test_calistir_baglanmis(program, test, &mut io)
                .map_err(|tani| test_kokeniyle(tani, test))
                .err();
            TestSonucu {
                ad: test.ad.clone(),
                hata,
            }
        })
        .collect()
}

/// Kaynağı derleyip testlerini koşar.
pub fn kaynagi_dene(kaynak: &str) -> Result<Vec<TestSonucu>, Tani> {
    let program = kaynagi_fazli_derle(kaynak)?;
    Ok(programi_dene_baglanmis(&program))
}

/// TÜM tanıları toplar (RFC-0010 §2.1): sözcükleme ilk hatada durur (nadir);
/// ayrıştırma cümle+girinti sınırında, denetim cümle başına sürer. Tanılar
/// kaynak konumunda sıralanır ve belge başına 20 kayıtla sınırlanır.
/// Boş liste = temiz. `dil denetle` ve LSP bu görünümü kullanır.
pub fn kaynagi_tanilari(kaynak: &str, yukleyici: &mut BirimYukleyici) -> Vec<Tani> {
    let mut kokenli = |istek: BirimIstegi<'_>| {
        if istek.tur == KullanimTuru::Paket {
            return Err("paket çözümü için proje bağlamı gerekir".into());
        }
        yukleyici(istek.ad).map(|kaynak| YuklenenBirim {
            kaynak,
            koken: istek.ad.to_string(),
        })
    };
    kaynagi_tanilari_kokenlerle(kaynak, None, &mut kokenli)
}

/// Çoklu tanı görünümünün kaynak-kökenli karşılığı.
pub fn kaynagi_tanilari_kokenlerle(
    kaynak: &str,
    koken: Option<&str>,
    yukleyici: &mut KokenliBirimYukleyici,
) -> Vec<Tani> {
    let tokenlar = match KaynakMetni::yeni(kaynak).sozcukle() {
        Ok(tokenlar) => tokenlar,
        Err(tani) => return vec![tani],
    };

    // Birimleri yükle; birim hataları da listeye girer ama ana dosya denetimi sürer.
    let mut tanilar = Vec::new();
    let mut islemler: HashMap<String, Islem> = HashMap::new();
    let mut yapilar: Vec<Yapi> = Vec::new();
    let mut testler: Vec<Test> = Vec::new();
    let mut yigin: Vec<String> = Vec::new();
    for (ad, tur, satir) in ayristirici::kullanilan_birimler(tokenlar.tokenlar()) {
        match yukleyici(BirimIstegi {
            ad: &ad,
            tur,
            isteyen: koken,
        }) {
            Ok(yuklenen) => {
                yigin.push(yuklenen.koken.clone());
                match dosyayi_coz(
                    &yuklenen.kaynak,
                    Some(&yuklenen.koken),
                    false,
                    yukleyici,
                    &mut yigin,
                ) {
                    Ok(birim) => {
                        islemler.extend(birim.islemler);
                        yapilar.extend(birim.yapilar);
                        testler.extend(birim.testler);
                    }
                    Err(tani) => tanilar.push(birim_kokeniyle(tani, &yuklenen.koken)),
                }
                yigin.pop();
            }
            Err(hata) => tanilar.push(yukleme_hatasi(&ad, tur, &hata, satir)),
        }
    }

    let tohum: Vec<String> = islemler
        .values()
        .filter(|islem| islem.disari_acik)
        .map(|islem| islem.ad.clone())
        .collect();
    let on_taranan_islemler = ayristirici::islem_adlarini_tara(tokenlar.tokenlar());
    let (ast, ayristirma_tanilari) = tokenlar.ayristir_kurtarmali(tohum);
    let cumleler = ast.into_cumleler();
    tanilar.extend(ayristirma_tanilari);

    // Hoist (çakışmalar tanı olur, tanımlar yine de alınır ki devamı denetlensin).
    let mut kalan = Vec::new();
    for cumle in cumleler {
        match cumle {
            Cumle::Kullan { .. } => {}
            Cumle::IslemTanimi(islem) => {
                if islemler.contains_key(&islem.ad) {
                    tanilar.push(Tani::yeni(
                        "A005",
                        format!("\"{}\" işlemi birden çok kez tanımlandı.", islem.ad),
                        islem.satir,
                        1,
                        1,
                    ));
                }
                islemler.insert(islem.ad.clone(), islem);
            }
            Cumle::TestBlogu(test) => testler.push(test),
            Cumle::YapiTanimi(yapi) => {
                if yapilar.iter().any(|y| y.ad == yapi.ad) {
                    tanilar.push(Tani::yeni(
                        "A006",
                        format!("\"{}\" yapısı birden çok kez tanımlandı.", yapi.ad),
                        yapi.satir,
                        1,
                        1,
                    ));
                } else {
                    yapilar.push(yapi);
                }
            }
            baska => kalan.push(baska),
        }
    }

    let mut program = Program {
        cumleler: kalan,
        islemler,
        yapilar,
        testler,
    };
    let mut kok_nedeni_raporlanan = dusen_satirlarin_bas_sozleri(kaynak, &tanilar);
    if tanilar.len() < tani::AZAMI_TANI_SAYISI {
        let kalan = tani::AZAMI_TANI_SAYISI - tanilar.len();
        let (checker_tanilari, hatali_tanimlar) =
            cozumleyici::denetle_coklu_hatali_tanimlarla(&mut program);
        kok_nedeni_raporlanan.extend(hatali_tanimlar);
        tanilar.extend(checker_tanilari.into_iter().take(kalan));
    }
    tanimlanamayan_islem_cagrilarini_ayikla(&mut tanilar, &on_taranan_islemler, &program);
    tani::tanimsiz_ad_tekrarlarini_ayikla(&mut tanilar, &kok_nedeni_raporlanan);
    tani::tanilari_sirala_ve_sinirla(&mut tanilar);
    tanilar
}

/// Parser'ın düşürdüğü satırın baş sözü (`isim "Ayşe"` → `isim`) kök nedeni
/// raporlanmış ad sayılır; sonraki A001 tekrarları gürültüdür (K-166, ADR-024 §8).
fn dusen_satirlarin_bas_sozleri(kaynak: &str, tanilar: &[Tani]) -> Vec<String> {
    tanilar
        .iter()
        .filter_map(|tani| kaynak.lines().nth(tani.satir.checked_sub(1)?))
        .filter_map(|satir| satir.split_whitespace().next())
        .filter(|soz| soz.chars().all(|k| k.is_alphabetic() || k == '_'))
        .map(str::to_string)
        .collect()
}

/// Başlığı bozuk olduğu için tanımlanamayan işlemin çağrıları kök nedeni
/// (parser tanısı) tekrar etmez; T016 iç tutarlılık mesajı kullanıcıya
/// yansımaz (K-166, ADR-024 §8).
fn tanimlanamayan_islem_cagrilarini_ayikla(
    tanilar: &mut Vec<Tani>,
    on_taranan_islemler: &[String],
    program: &Program,
) {
    let tanimlanamayan = on_taranan_islemler
        .iter()
        .filter(|ad| !program.islemler.contains_key(*ad))
        .collect::<Vec<_>>();
    tanilar.retain(|tani| {
        tani.kod != "T016"
            || !tanimlanamayan
                .iter()
                .any(|ad| tani.mesaj.starts_with(&format!("\"{ad}\" işleminin")))
    });
}
