//! Birim sistemi (RFC-0009): `X birimini kullan` — sahte yükleyiciyle
//! deterministik testler.

use dil::kaynagi_derle_birimlerle;
use std::collections::HashMap;

fn yukleyici<'a>(
    dosyalar: &'a HashMap<&'static str, &'static str>,
) -> impl FnMut(&str) -> Result<String, String> + 'a {
    move |ad: &str| {
        dosyalar
            .get(ad)
            .map(|icerik| icerik.to_string())
            .ok_or_else(|| "dosya yok".to_string())
    }
}

fn calistir(kaynak: &str, dosyalar: HashMap<&'static str, &'static str>) -> Result<Vec<String>, dil::tani::Tani> {
    let program = kaynagi_derle_birimlerle(kaynak, &mut yukleyici(&dosyalar))?;
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    dil::yorumlayici::calistir_io(&program, &mut io)?;
    Ok(io.cikti)
}

#[test]
fn birimden_islem_kullanma() {
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem ikiyle çarp\n    sayıyı al\n    sonucu sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n\nx 5 için ikiyle çarp olsun\nx yaz\n";
    assert_eq!(calistir(kaynak, dosyalar).expect("çalışmalı"), vec!["10"]);
}

#[test]
fn birimden_yapi_kullanma() {
    let dosyalar = HashMap::from([(
        "kayitlar",
        "yapı Öğrenci\n    ad Metin\n    yaş TamSayı\n",
    )]);
    let kaynak = "kayitlar birimini kullan\n\nayşe yeni Öğrenci olsun\nayşenin yaşı 10 olsun\nayşenin yaşı yaz\n";
    assert_eq!(calistir(kaynak, dosyalar).expect("çalışmalı"), vec!["10"]);
}

#[test]
fn birim_zinciri() {
    // ana → üst → alt: birimler kendi birimlerini kullanabilir.
    let dosyalar = HashMap::from([
        (
            "alt",
            "işlem bir ver\n    1 döndür\n",
        ),
        (
            "ust",
            "alt birimini kullan\n\nişlem iki ver\n    x bir ver olsun\n    sonucu x ile 2 nin çarpımı olsun\n    sonucu döndür\n",
        ),
    ]);
    let kaynak = "ust birimini kullan\n\ny iki ver olsun\ny yaz\n";
    assert_eq!(calistir(kaynak, dosyalar).expect("çalışmalı"), vec!["2"]);
}

#[test]
fn birimin_ust_duzey_cumleleri_calismaz() {
    // Kapsülleme (RFC-0009 §2): birim olarak alınan dosyanın üst düzey
    // cümleleri koşulmaz; dosya kendi başına çalıştırılabilir kalır.
    let dosyalar = HashMap::from([(
        "selamci",
        "\"birimden selam\" yaz\n\nişlem selam ver\n    \"işlemden selam\" yaz\n",
    )]);
    let kaynak = "selamci birimini kullan\n\nselam ver\n";
    assert_eq!(
        calistir(kaynak, dosyalar).expect("çalışmalı"),
        vec!["işlemden selam"],
        "birimin üst düzey yaz'ı görünmemeli"
    );
}

#[test]
fn dongusel_kullanim_reddedilir() {
    let dosyalar = HashMap::from([
        ("a", "b birimini kullan\n\nişlem a ver\n    1 döndür\n"),
        ("b", "a birimini kullan\n\nişlem b ver\n    2 döndür\n"),
    ]);
    let kaynak = "a birimini kullan\n\n\"olmaz\" yaz\n";
    let hata = calistir(kaynak, dosyalar).expect_err("döngü reddedilmeli");
    assert_eq!(hata.kod, "A009");
}

#[test]
fn ad_cakismasi_sessiz_golgelenmez() {
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem topla\n    1 döndür\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n\nişlem topla\n    2 döndür\n\nx topla olsun\nx yaz\n";
    let hata = calistir(kaynak, dosyalar).expect_err("çakışma hata olmalı");
    assert_eq!(hata.kod, "A008");
    assert!(hata.mesaj.contains("hesaplar"), "kaynaklar sayılmalı: {}", hata.mesaj);
}

#[test]
fn bulunamayan_birim() {
    let kaynak = "kayip birimini kullan\n";
    let hata = calistir(kaynak, HashMap::new()).expect_err("A010 bekleniyor");
    assert_eq!(hata.kod, "A010");
    assert!(hata.oneri.as_deref().unwrap_or("").contains("kayip.dil"));
}

#[test]
fn birim_testleri_dene_kapsaminda() {
    // RFC-0009 §2: birimin test tanımları da görünür — dil dene hepsini koşar.
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem ikiyle çarp\n    sayıyı al\n    sonucu sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n\ntest \"çarpım doğru\"\n    x 3 için ikiyle çarp olsun\n    x 6 ya eşit olmalı\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n\ntest \"ana test\"\n    1 1 e eşit olmalı\n";
    let program = kaynagi_derle_birimlerle(kaynak, &mut yukleyici(&dosyalar)).expect("derlenmeli");
    let sonuclar = dil::programi_dene(&program);
    let adlar: Vec<&str> = sonuclar.iter().map(|s| s.ad.as_str()).collect();
    assert!(adlar.contains(&"hesaplar: çarpım doğru"), "birim testi öneklenmeli: {:?}", adlar);
    assert!(adlar.contains(&"ana test"));
    assert!(sonuclar.iter().all(|s| s.hata.is_none()), "hepsi geçmeli");
}

#[test]
fn kullan_islem_icinde_yasak() {
    let kaynak = "işlem deneme\n    hesaplar birimini kullan\n    1 döndür\n";
    let hata = calistir(kaynak, HashMap::new()).expect_err("S021 bekleniyor");
    assert_eq!(hata.kod, "S021");
}
