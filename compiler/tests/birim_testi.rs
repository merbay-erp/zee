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

fn calistir(
    kaynak: &str,
    dosyalar: HashMap<&'static str, &'static str>,
) -> Result<Vec<String>, dil::tani::Tani> {
    let program = kaynagi_derle_birimlerle(kaynak, &mut yukleyici(&dosyalar))?;
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    dil::yorumlayici::calistir_io(&program, &mut io)?;
    Ok(io.cikti)
}

#[test]
fn birimden_islem_kullanma() {
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem ikiyle çarp\n    sayıyı TamSayı olarak al\n    TamSayı döndürür\n    sonucu sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n\nx 5 için ikiyle çarp olsun\nx yaz\n";
    assert_eq!(calistir(kaynak, dosyalar).expect("çalışmalı"), vec!["10"]);
}

#[test]
fn disari_acik_islem_tam_sozlesme_ister() {
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem ikiyle çarp\n    sayıyı al\n    sonucu sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n";
    let hata = calistir(kaynak, dosyalar).expect_err("T039 bekleniyor");
    assert_eq!(hata.kod, "T039");
    assert!(hata.mesaj.contains("hesaplar"), "{}", hata.mesaj);
}

#[test]
fn public_kapsayici_ve_ozyineleme_cagri_sirasindan_bagimsizdir() {
    let birim = "\
işlem toplamını hesapla
    sayıları Ondalık listesi olarak al
    Ondalık döndürür
    toplam 0,0 olsun
    her sayı için
        toplamı sayıyla artır
    toplamı döndür

işlem faktöriyelini hesapla
    sayıyı TamSayı olarak al
    TamSayı döndürür
    sayı 1 den küçükse
        1 döndür
    önceki sayı ile 1 in farkı olsun
    alt önceki için faktöriyelini hesapla olsun
    sonuç sayı ile altın çarpımı olsun
    sonucu döndür
";
    let dar_once = "hesaplar birimini kullan\n\na 3, 7 listesi için toplamını hesapla olsun\nb 1,5, 2,5 listesi için toplamını hesapla olsun\nf 5 için faktöriyelini hesapla olsun\na yaz\nb yaz\nf yaz\n";
    let genis_once = "hesaplar birimini kullan\n\nb 1,5, 2,5 listesi için toplamını hesapla olsun\na 3, 7 listesi için toplamını hesapla olsun\nf 5 için faktöriyelini hesapla olsun\na yaz\nb yaz\nf yaz\n";
    let dosyalar = HashMap::from([("hesaplar", birim)]);
    assert_eq!(
        calistir(dar_once, dosyalar.clone()).expect("dar-geniş"),
        vec!["10,0", "4,0", "120"]
    );
    assert_eq!(
        calistir(genis_once, dosyalar).expect("geniş-dar"),
        vec!["10,0", "4,0", "120"]
    );
}

#[test]
fn birimden_yapi_kullanma() {
    let dosyalar = HashMap::from([("kayitlar", "yapı Öğrenci\n    ad Metin\n    yaş TamSayı\n")]);
    let kaynak = "kayitlar birimini kullan\n\nayşe yeni Öğrenci olsun\nayşenin yaşı 10 olsun\nayşenin yaşı yaz\n";
    assert_eq!(calistir(kaynak, dosyalar).expect("çalışmalı"), vec!["10"]);
}

#[test]
fn birim_zinciri() {
    // ana → üst → alt: birimler kendi birimlerini kullanabilir.
    let dosyalar = HashMap::from([
        (
            "alt",
            "işlem bir ver\n    TamSayı döndürür\n    1 döndür\n",
        ),
        (
            "ust",
            "alt birimini kullan\n\nişlem iki ver\n    TamSayı döndürür\n    x bir ver olsun\n    sonucu x ile 2 nin çarpımı olsun\n    sonucu döndür\n",
        ),
    ]);
    let kaynak = "ust birimini kullan\n\ny iki ver olsun\ny yaz\n";
    assert_eq!(calistir(kaynak, dosyalar).expect("çalışmalı"), vec!["2"]);
}

#[test]
fn birimin_aldigi_islem_ortuk_yeniden_acilmaz() {
    let dosyalar = HashMap::from([
        (
            "alt",
            "işlem gizli değeri ver\n    TamSayı döndürür\n    1 döndür\n",
        ),
        (
            "ust",
            "alt birimini kullan\n\nişlem açık değeri ver\n    TamSayı döndürür\n    x gizli değeri ver olsun\n    x döndür\n",
        ),
    ]);
    let kaynak = "ust birimini kullan\n\nx gizli değeri ver olsun\nx yaz\n";
    let hata = calistir(kaynak, dosyalar).expect_err("geçişli işlem görünmemeli");
    assert_eq!(hata.kod, "A001");
}

#[test]
fn birimin_ust_duzey_cumleleri_calismaz() {
    // Kapsülleme (RFC-0009 §2): birim olarak alınan dosyanın üst düzey
    // cümleleri koşulmaz; dosya kendi başına çalıştırılabilir kalır.
    let dosyalar = HashMap::from([(
        "selamci",
        "\"birimden selam\" yaz\n\nişlem selam ver\n    değer döndürmez\n    \"işlemden selam\" yaz\n",
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
        (
            "a",
            "b birimini kullan\n\nişlem a ver\n    TamSayı döndürür\n    1 döndür\n",
        ),
        (
            "b",
            "a birimini kullan\n\nişlem b ver\n    TamSayı döndürür\n    2 döndür\n",
        ),
    ]);
    let kaynak = "a birimini kullan\n\n\"olmaz\" yaz\n";
    let hata = calistir(kaynak, dosyalar).expect_err("döngü reddedilmeli");
    assert_eq!(hata.kod, "A009");
}

#[test]
fn ad_cakismasi_sessiz_golgelenmez() {
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem topla\n    TamSayı döndürür\n    1 döndür\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n\nişlem topla\n    2 döndür\n\nx topla olsun\nx yaz\n";
    let hata = calistir(kaynak, dosyalar).expect_err("çakışma hata olmalı");
    assert_eq!(hata.kod, "A008");
    assert!(
        hata.mesaj.contains("hesaplar"),
        "kaynaklar sayılmalı: {}",
        hata.mesaj
    );
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
        "işlem ikiyle çarp\n    sayıyı TamSayı olarak al\n    TamSayı döndürür\n    sonucu sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n\ntest \"çarpım doğru\"\n    x 3 için ikiyle çarp olsun\n    x 6 ya eşit olmalı\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n\ntest \"ana test\"\n    1 1 e eşit olmalı\n";
    let program = kaynagi_derle_birimlerle(kaynak, &mut yukleyici(&dosyalar)).expect("derlenmeli");
    let sonuclar = dil::programi_dene(&program);
    let adlar: Vec<&str> = sonuclar.iter().map(|s| s.ad.as_str()).collect();
    assert!(
        adlar.contains(&"hesaplar: çarpım doğru"),
        "birim testi öneklenmeli: {:?}",
        adlar
    );
    assert!(adlar.contains(&"ana test"));
    assert!(sonuclar.iter().all(|s| s.hata.is_none()), "hepsi geçmeli");
}

#[test]
fn kullan_islem_icinde_yasak() {
    let kaynak = "işlem deneme\n    hesaplar birimini kullan\n    1 döndür\n";
    let hata = calistir(kaynak, HashMap::new()).expect_err("S021 bekleniyor");
    assert_eq!(hata.kod, "S021");
}

// --- K-164/ADR-072: elmas içe alım ve birim kökenli tanılar ---

fn elmas_dosyalari() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        (
            "a",
            "işlem bir ver\n    TamSayı döndürür\n    1 döndür\n\ntest \"a testi\"\n    x bir ver olsun\n    x 1 e eşit olmalı\n",
        ),
        (
            "b",
            "a birimini kullan\n\nişlem iki ver\n    TamSayı döndürür\n    x bir ver olsun\n    sonucu x ile 2 nin çarpımı olsun\n    sonucu döndür\n",
        ),
    ])
}

#[test]
fn elmas_ice_alim_ayni_kokenli_tanimi_cakisma_saymaz() {
    // c → a ve c → b → a: `bir ver` iki yoldan gelir ama tek tanımdır (spec/07).
    let kaynak = "a birimini kullan\nb birimini kullan\n\nx bir ver olsun\ny iki ver olsun\nx ile \" \" ile y yaz\n";
    assert_eq!(
        calistir(kaynak, elmas_dosyalari()).expect("elmas içe alım çalışmalı"),
        vec!["1 2"]
    );
}

#[test]
fn elmas_ice_alimda_birim_testleri_bir_kez_alinir() {
    let kaynak = "a birimini kullan\nb birimini kullan\n\ny iki ver olsun\n";
    let dosyalar = elmas_dosyalari();
    let program = kaynagi_derle_birimlerle(kaynak, &mut yukleyici(&dosyalar)).expect("derlenmeli");
    let adlar: Vec<&str> = program.testler.iter().map(|t| t.ad.as_str()).collect();
    assert_eq!(
        adlar,
        vec!["a: a testi"],
        "a'nın testi tek sefer alınmalı: {adlar:?}"
    );
    let sonuclar = dil::programi_dene(&program);
    assert!(sonuclar.iter().all(|s| s.hata.is_none()));
}

#[test]
fn farkli_kokenli_ayni_ad_elmas_sayilmaz_a008_kalir() {
    let mut dosyalar = elmas_dosyalari();
    dosyalar.insert("d", "işlem bir ver\n    TamSayı döndürür\n    9 döndür\n");
    let kaynak = "a birimini kullan\nd birimini kullan\n\nx bir ver olsun\n";
    let hata = calistir(kaynak, dosyalar).expect_err("iki farklı tanım çakışmalı");
    assert_eq!(hata.kod, "A008");
    assert!(
        hata.mesaj.contains("\"a\"") && hata.mesaj.contains("\"d\""),
        "{}",
        hata.mesaj
    );
    assert_eq!(hata.satir, 2, "çakışma ikinci kullan satırında bildirilir");
    assert!(hata.koken.is_none(), "ana kaynaktaki çakışma kökensizdir");
}

#[test]
fn birim_ayristirma_tanisi_birim_kokenini_ve_satirini_tasir() {
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem selamla\n    adı Metin olarak al\n    Metin döndürür\n\n    x adın adedi 3 e eşit olmalı\n    adı döndür\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n\n\"Zeynep\" için selamla yaz\n";
    let hata = calistir(kaynak, dosyalar).expect_err("birimdeki S016 yayılmalı");
    assert_eq!(hata.kod, "S016");
    assert_eq!(hata.koken.as_deref(), Some("hesaplar"));
    assert_eq!(hata.satir, 5, "satır birimin kendi metnine göredir");
}

#[test]
fn birim_govdesindeki_tur_tanisi_birim_kokenini_tasir() {
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem selamla\n    adı Metin olarak al\n    Metin döndürür\n\n    toplam 1 ile \"x\" nin toplamı olsun\n    adı döndür\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n\n\"Zeynep\" için selamla yaz\n";
    let hata = calistir(kaynak, dosyalar).expect_err("birimdeki T008 yayılmalı");
    assert_eq!(hata.kod, "T008");
    assert_eq!(hata.koken.as_deref(), Some("hesaplar"));
    assert_eq!(hata.satir, 5);
}

#[test]
fn birim_govdesindeki_calisma_tanisi_birim_kokenini_tasir() {
    let dosyalar = HashMap::from([(
        "okuyucu",
        "işlem satırları getir\n    Metin listesi döndürür\n\n    satırlar \"yok.txt\" dosyasının satırları olsun\n    satırları döndür\n",
    )]);
    let kaynak = "okuyucu birimini kullan\n\nx satırları getir olsun\nx yaz\n";
    let hata = calistir(kaynak, dosyalar).expect_err("dosya yokken çalışma tanısı");
    assert!(hata.kod.starts_with('C'), "{}", hata.kod);
    assert_eq!(hata.koken.as_deref(), Some("okuyucu"));
    assert_eq!(hata.satir, 4);
}

#[test]
fn birim_testinin_tanisi_birim_kokenini_tasir() {
    let dosyalar = HashMap::from([(
        "hesaplar",
        "işlem bir ver\n    TamSayı döndürür\n    1 döndür\n\ntest \"yanlış beklenti\"\n    x bir ver olsun\n    x 2 ye eşit olmalı\n",
    )]);
    let kaynak = "hesaplar birimini kullan\n";
    let program = kaynagi_derle_birimlerle(kaynak, &mut yukleyici(&dosyalar)).expect("derlenmeli");
    let sonuclar = dil::programi_dene(&program);
    let hata = sonuclar[0].hata.as_ref().expect("D001 beklenir");
    assert_eq!(hata.kod, "D001");
    assert_eq!(hata.koken.as_deref(), Some("hesaplar"));
    assert_eq!(hata.satir, 7);
}

#[test]
fn kokenli_tani_raporu_ana_kaynaktan_alinti_yapmaz_birim_metniyle_yapar() {
    let tani =
        dil::tani::Tani::yeni("T008", "Aritmetik hata.".into(), 5, 12, 3).kokenle("hesaplar");
    let ana = "hesaplar birimini kullan\n\n\"Zeynep\" için selamla yaz\n";
    let rapor = tani.raporla(ana);
    assert!(rapor.contains("Birim: hesaplar (satır 5)"), "{rapor}");
    assert!(
        !rapor.contains(" | "),
        "ana kaynaktan alıntı yapılmamalı: {rapor}"
    );
    let birim = "işlem selamla\n    adı Metin olarak al\n    Metin döndürür\n\n    toplam 1 ile \"x\" nin toplamı olsun\n";
    let birimli = tani.raporla_birimle(birim);
    assert!(
        birimli.contains("5 |     toplam 1 ile \"x\" nin toplamı olsun"),
        "{birimli}"
    );
    assert!(birimli.contains("^^^"), "{birimli}");
    assert!(
        tani.json().contains("\"koken\":\"hesaplar\""),
        "{}",
        tani.json()
    );
    let kokensiz = dil::tani::Tani::yeni("T008", "x".into(), 1, 1, 1);
    assert!(!kokensiz.json().contains("koken"), "kökensiz JSON değişmez");
    let ikinci = tani.clone().kokenle("dış");
    assert_eq!(
        ikinci.koken.as_deref(),
        Some("hesaplar"),
        "en içteki köken kalır"
    );
}
