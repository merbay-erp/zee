//! Türkçe programlama dili — bootstrap derleyici (Stage 0).
//!
//! Boru hattı (master plan bölüm 11'in v0 dilimi):
//! kaynak → sözcükleyici → ayrıştırıcı → ad çözümleme + tür denetimi → yorumlayıcı.

pub mod agac;
pub mod ayristirici;
pub mod bicimleyici;
pub mod kalici_dosya;
pub mod lsp;
pub mod morfoloji;
mod ondalik;
pub mod paket;
pub mod proje;
pub mod wasm_api;
pub mod cozumleyici;
mod eylem;
pub mod guvenlik;
pub mod sozcukleyici;
pub mod tani;
#[cfg(not(target_arch = "wasm32"))]
pub mod tedarik;
pub mod web_guvenligi;
pub mod yorumlayici;

use agac::{Cumle, Islem, KullanimTuru, Program, Test, Yapi};
use std::collections::HashMap;
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
    kaynagi_derle_birimlerle(kaynak, &mut |ad: &str| {
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
    &["matematik", "liste_araclari", "metin_araclari", "sozluk_araclari"]
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
    let mut kokenli = |istek: BirimIstegi<'_>| {
        if istek.tur == KullanimTuru::Paket {
            return Err("paket çözümü için proje bağlamı gerekir".into());
        }
        yukleyici(istek.ad).map(|kaynak| YuklenenBirim {
            kaynak,
            koken: istek.ad.to_string(),
        })
    };
    kaynagi_derle_kokenlerle(kaynak, None, &mut kokenli)
}

/// Kaynağı gerçek dosya/paket kökenini koruyarak derler. CLI ve proje araçları
/// bu yüzeyi kullanır; eski `kaynagi_derle_birimlerle` API'si korunmuştur.
pub fn kaynagi_derle_kokenlerle(
    kaynak: &str,
    koken: Option<&str>,
    yukleyici: &mut KokenliBirimYukleyici,
) -> Result<Program, Tani> {
    let mut yigin: Vec<String> = Vec::new();
    let (cumleler, islemler, yapilar, testler) =
        dosyayi_coz(kaynak, koken, true, yukleyici, &mut yigin)?;
    let mut program = Program { cumleler, islemler, yapilar, testler };
    cozumleyici::denetle(&mut program)?;
    Ok(program)
}

/// Bir kaynak dosyayı çözer: birimlerini özyinelemeli yükler, kendi
/// tanımlarını içe alınanlarla birleştirir. `birim_adi` None ise ana dosyadır.
#[allow(clippy::type_complexity)]
fn dosyayi_coz(
    kaynak: &str,
    kaynak_kokeni: Option<&str>,
    ana_kaynak: bool,
    yukleyici: &mut KokenliBirimYukleyici,
    yigin: &mut Vec<String>,
) -> Result<(Vec<Cumle>, HashMap<String, Islem>, Vec<Yapi>, Vec<Test>), Tani> {
    let tokenlar = sozcukleyici::sozcukle(kaynak)?;

    // 1) Birimleri önden yükle (çağrı tanıma için işlem adları gerekli).
    let mut islemler: HashMap<String, Islem> = HashMap::new();
    let mut islem_kaynagi: HashMap<String, String> = HashMap::new();
    let mut yapilar: Vec<Yapi> = Vec::new();
    let mut yapi_kaynagi: HashMap<String, String> = HashMap::new();
    let mut testler: Vec<Test> = Vec::new();

    for (ad, tur, satir) in ayristirici::kullanilan_birimler(&tokenlar) {
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
        let (_, birim_islemleri, birim_yapilari, birim_testleri) =
            dosyayi_coz(&yuklenen.kaynak, Some(&yuklenen.koken), false, yukleyici, yigin)?;
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
    let tohum: Vec<String> = islemler
        .values()
        .filter(|islem| islem.disari_acik)
        .map(|islem| islem.ad.clone())
        .collect();
    let cumleler = ayristirici::ayristir_tohumla(tokenlar, tohum)?;

    // 3) Kendi tanımlarını ayıkla ve birleştir.
    let mut kalan = Vec::new();
    let mut kendi_islem_adlari = Vec::new();
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
                kendi_islem_adlari.push(islem.ad.clone());
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

    if !ana_kaynak {
        // Alınan işlemler bu dosyanın public API'sine örtük yeniden açılmaz.
        // Gövde/runtime çağrıları için haritada kalır; yalnız bu kaynağın
        // doğrudan tanımları bir üst kaynağın çağrı yüzeyine çıkar.
        for islem in islemler.values_mut() {
            islem.disari_acik = false;
        }
        for ad in &kendi_islem_adlari {
            islemler
                .get_mut(ad)
                .expect("kendi işlemi az önce eklendi")
                .disari_acik = true;
        }
        disari_acik_imzalari_denetle(
            &islemler,
            kaynak_kokeni.unwrap_or("adı bilinmeyen birim"),
        )?;
    }

    // Birim olarak yüklenen dosyanın üst düzey cümleleri İÇE ALINMAZ
    // (kapsülleme, RFC-0009 §2): dosya kendi başına çalıştırılabilir kalır,
    // birim olarak yalnız tanımlarını verir.
    if !ana_kaynak {
        kalan.clear();
    }

    Ok((kalan, islemler, yapilar, testler))
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
        let islem = islemler.get(&ad).expect("ad haritadan geldi");
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
    for (ad, tur, satir) in ayristirici::kullanilan_birimler(&tokenlar) {
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
                    Ok((_, birim_islemleri, birim_yapilari, birim_testleri)) => {
                        islemler.extend(birim_islemleri);
                        yapilar.extend(birim_yapilari);
                        testler.extend(birim_testleri);
                    }
                    Err(tani) => tanilar.push(tani),
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
