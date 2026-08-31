//! Türkçe programlama dili — bootstrap derleyici (Stage 0).
//!
//! Boru hattı (master plan bölüm 11'in v0 dilimi):
//! kaynak → sözcükleyici → ayrıştırıcı → ad çözümleme + tür denetimi → yorumlayıcı.

pub mod agac;
pub mod ayristirici;
pub mod bicimleyici;
pub mod lsp;
pub mod wasm_api;
pub mod cozumleyici;
pub mod sozcukleyici;
pub mod tani;
pub mod yorumlayici;

use agac::{Cumle, Islem, Program, Test, Yapi};
use std::collections::HashMap;
use tani::Tani;

/// Birim yükleyici: birim adını kaynak metne çevirir (RFC-0009). CLI gerçek
/// dosya sisteminden okur; testler sahte tablodan verir — birim çözümü de
/// determinizm ilkesine uyar.
pub type BirimYukleyici<'a> = dyn FnMut(&str) -> Result<String, String> + 'a;

/// Kaynağı çalıştırılabilir programa derler. Birim kullanmayan kaynaklar için;
/// `kullan` görülürse A010 verir (yükleyici bağlanmamış).
pub fn kaynagi_derle(kaynak: &str) -> Result<Program, Tani> {
    kaynagi_derle_birimlerle(kaynak, &mut |ad: &str| {
        Err(format!("\"{}\" birimi bu bağlamda yüklenemez", ad))
    })
}

/// Kaynağı, birimlerini yükleyerek derler (sözcükle + birim çözümü +
/// tohumlu ayrıştırma + hoist + denetle).
pub fn kaynagi_derle_birimlerle(
    kaynak: &str,
    yukleyici: &mut BirimYukleyici,
) -> Result<Program, Tani> {
    let mut yigin: Vec<String> = Vec::new();
    let (cumleler, islemler, yapilar, testler) =
        dosyayi_coz(kaynak, None, yukleyici, &mut yigin)?;
    let mut program = Program { cumleler, islemler, yapilar, testler };
    cozumleyici::denetle(&mut program)?;
    Ok(program)
}

/// Bir kaynak dosyayı çözer: birimlerini özyinelemeli yükler, kendi
/// tanımlarını içe alınanlarla birleştirir. `birim_adi` None ise ana dosyadır.
#[allow(clippy::type_complexity)]
fn dosyayi_coz(
    kaynak: &str,
    birim_adi: Option<&str>,
    yukleyici: &mut BirimYukleyici,
    yigin: &mut Vec<String>,
) -> Result<(Vec<Cumle>, HashMap<String, Islem>, Vec<Yapi>, Vec<Test>), Tani> {
    let tokenlar = sozcukleyici::sozcukle(kaynak)?;

    // 1) Birimleri önden yükle (çağrı tanıma için işlem adları gerekli).
    let mut islemler: HashMap<String, Islem> = HashMap::new();
    let mut islem_kaynagi: HashMap<String, String> = HashMap::new();
    let mut yapilar: Vec<Yapi> = Vec::new();
    let mut yapi_kaynagi: HashMap<String, String> = HashMap::new();
    let mut testler: Vec<Test> = Vec::new();

    for (ad, satir) in ayristirici::kullanilan_birimler(&tokenlar) {
        if yigin.iter().any(|y| y == &ad) || birim_adi == Some(ad.as_str()) {
            return Err(Tani::yeni(
                "A009",
                format!(
                    "Birimler birbirini döngüsel kullanıyor: {} → {}.",
                    yigin.join(" → "),
                    ad
                ),
                satir,
                1,
                1,
            )
            .onerili("Ortak tanımları üçüncü bir birime taşı.".into()));
        }
        let icerik = yukleyici(&ad).map_err(|hata| {
            Tani::yeni(
                "A010",
                format!("\"{}\" birimi yüklenemedi: {}.", ad, hata),
                satir,
                1,
                1,
            )
            .onerili(format!(
                "Aynı klasörde {}.dil dosyası olmalı (RFC-0009).",
                ad
            ))
        })?;
        yigin.push(ad.clone());
        let (_, birim_islemleri, birim_yapilari, birim_testleri) =
            dosyayi_coz(&icerik, Some(&ad), yukleyici, yigin)?;
        yigin.pop();

        for (islem_adi, islem) in birim_islemleri {
            if let Some(onceki) = islem_kaynagi.get(&islem_adi) {
                return Err(cakisma("işlemi", &islem_adi, onceki, &ad, satir));
            }
            islem_kaynagi.insert(islem_adi.clone(), ad.clone());
            islemler.insert(islem_adi, islem);
        }
        for yapi in birim_yapilari {
            if let Some(onceki) = yapi_kaynagi.get(&yapi.ad) {
                return Err(cakisma("yapısı", &yapi.ad, onceki, &ad, satir));
            }
            yapi_kaynagi.insert(yapi.ad.clone(), ad.clone());
            yapilar.push(yapi);
        }
        // Birimin testleri de görünür olur (RFC-0009 §2); ad birimle önek alır.
        for mut test in birim_testleri {
            if !test.ad.starts_with(&format!("{}: ", ad)) {
                test.ad = format!("{}: {}", ad, test.ad);
            }
            testler.push(test);
        }
    }

    // 2) İçe alınan işlem adlarıyla tohumlanmış ayrıştırma.
    let tohum: Vec<String> = islemler.keys().cloned().collect();
    let cumleler = ayristirici::ayristir_tohumla(tokenlar, tohum)?;

    // 3) Kendi tanımlarını ayıkla ve birleştir.
    let mut kalan = Vec::new();
    for cumle in cumleler {
        match cumle {
            Cumle::Kullan { .. } => {} // 1. adımda çözüldü
            Cumle::IslemTanimi(islem) => {
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
                islemler.insert(islem.ad.clone(), islem);
            }
            Cumle::TestBlogu(test) => testler.push(test),
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
                yapilar.push(yapi);
            }
            baska => kalan.push(baska),
        }
    }

    // Birim olarak yüklenen dosyanın üst düzey cümleleri İÇE ALINMAZ
    // (kapsülleme, RFC-0009 §2): dosya kendi başına çalıştırılabilir kalır,
    // birim olarak yalnız tanımlarını verir.
    if birim_adi.is_some() {
        kalan.clear();
    }

    Ok((kalan, islemler, yapilar, testler))
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
    kaynagi_derle(kaynak).map(|_| ())
}

/// Kaynağı uçtan uca çalıştırır; çıktı satırlarını döndürür.
pub fn kaynagi_calistir(kaynak: &str) -> Result<Vec<String>, Tani> {
    kaynagi_calistir_girdiyle(kaynak, Vec::new())
}

/// Kaynağı hazır girdi satırlarıyla çalıştırır ("diye sor" cevapları sırayla
/// bu listeden gelir); istemler de çıktıya dahildir.
pub fn kaynagi_calistir_girdiyle(kaynak: &str, girdiler: Vec<String>) -> Result<Vec<String>, Tani> {
    let program = kaynagi_derle(kaynak)?;
    let mut io = yorumlayici::ToplayanIo::yeni(girdiler);
    yorumlayici::calistir_io(&program, &mut io)?;
    Ok(io.cikti)
}

/// Bir testin sonucu: `hata` None ise geçti.
pub struct TestSonucu {
    pub ad: String,
    pub hata: Option<Tani>,
}

/// Programın testlerini koşar. v0: her test taze ortamda ve dış dünyaya
/// dokunmayan hermetik IO ile çalışır (gerçek dosya/ağ erişimi yok).
pub fn programi_dene(program: &Program) -> Vec<TestSonucu> {
    program
        .testler
        .iter()
        .map(|test| {
            let mut io = yorumlayici::ToplayanIo::yeni(Vec::new());
            let hata = yorumlayici::test_calistir(program, test, &mut io).err();
            TestSonucu { ad: test.ad.clone(), hata }
        })
        .collect()
}

/// Kaynağı derleyip testlerini koşar.
pub fn kaynagi_dene(kaynak: &str) -> Result<Vec<TestSonucu>, Tani> {
    let program = kaynagi_derle(kaynak)?;
    Ok(programi_dene(&program))
}

/// TÜM tanıları toplar (RFC-0010 §3.1): sözcükleme ilk hatada durur (nadir);
/// ayrıştırma cümle atlayarak, denetim cümle başına sürerek toplar.
/// Boş liste = temiz. `dil denetle` ve LSP bu görünümü kullanır.
pub fn kaynagi_tanilari(kaynak: &str, yukleyici: &mut BirimYukleyici) -> Vec<Tani> {
    let tokenlar = match sozcukleyici::sozcukle(kaynak) {
        Ok(tokenlar) => tokenlar,
        Err(tani) => return vec![tani],
    };

    // Birimleri yükle; birim hataları da listeye girer ama ana dosya denetimi sürer.
    let mut tanilar = Vec::new();
    let mut islemler: HashMap<String, Islem> = HashMap::new();
    let mut yapilar: Vec<Yapi> = Vec::new();
    let mut testler: Vec<Test> = Vec::new();
    let mut yigin: Vec<String> = Vec::new();
    for (ad, satir) in ayristirici::kullanilan_birimler(&tokenlar) {
        match yukleyici(&ad) {
            Ok(icerik) => {
                yigin.push(ad.clone());
                match dosyayi_coz(&icerik, Some(&ad), yukleyici, &mut yigin) {
                    Ok((_, birim_islemleri, birim_yapilari, birim_testleri)) => {
                        islemler.extend(birim_islemleri);
                        yapilar.extend(birim_yapilari);
                        testler.extend(birim_testleri);
                    }
                    Err(tani) => tanilar.push(tani),
                }
                yigin.pop();
            }
            Err(hata) => tanilar.push(
                Tani::yeni(
                    "A010",
                    format!("\"{}\" birimi yüklenemedi: {}.", ad, hata),
                    satir,
                    1,
                    1,
                )
                .onerili(format!("Aynı klasörde {}.dil dosyası olmalı (RFC-0009).", ad)),
            ),
        }
    }

    let tohum: Vec<String> = islemler.keys().cloned().collect();
    let (cumleler, ayristirma_tanilari) = ayristirici::ayristir_kurtarmali(tokenlar, tohum);
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

    let mut program = Program { cumleler: kalan, islemler, yapilar, testler };
    tanilar.extend(cozumleyici::denetle_coklu(&mut program));
    tanilar
}
