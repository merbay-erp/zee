//! Birim sistemi (RFC-0009): `X birimini kullan` — sahte yükleyiciyle
//! deterministik testler.

use dil::kaynagi_derle_birimlerle;
use std::collections::HashMap;

#[test]
fn farkli_rota_birimlerinin_ayni_satiri_ayri_hir_kapsamidir() {
    let mut yukleyici = |ad: &str| match ad {
        "sol" => Ok(include_str!("../../regression/hir/sol.birim").to_string()),
        "sag" => Ok(include_str!("../../regression/hir/sag.birim").to_string()),
        _ => Err("Birim yok".to_string()),
    };
    let program = dil::kaynagi_fazli_derle_birimlerle(
        include_str!("../../regression/hir/rota-birimlerinde-ayni-satir.dil"),
        &mut yukleyici,
    )
    .expect("aynı satırlı rota birimleri bağlanmalı");
    program
        .invariantleri_dogrula()
        .expect("tekil scope ve SymbolId");
    let kullanımlar = program.hir().sembol_kullanimlari();
    let kapsamlar = kullanımlar
        .iter()
        .map(|k| k.kimlik().kapsam())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(kapsamlar.len(), 2);
    let mut tekrar = |ad: &str| yukleyici(ad);
    let ikinci = dil::kaynagi_fazli_derle_birimlerle(
        include_str!("../../regression/hir/rota-birimlerinde-ayni-satir.dil"),
        &mut tekrar,
    )
    .expect("aynı kaynak yeniden bağlanmalı");
    let ikinci_kapsamlar = ikinci
        .hir()
        .sembol_kullanimlari()
        .iter()
        .map(|k| k.kimlik().kapsam())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(kapsamlar, ikinci_kapsamlar);
}

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
fn parametresiz_public_islem_donus_bildirimi_yoksa_t039dur() {
    let kaynak = "sabitler birimini kullan\n";
    let dosyalar = HashMap::from([("sabitler", "işlem bir ver\n    1 döndür\n")]);
    // Çağrılmayan parametresiz export da bütün birim yüzeyinde denetlenir.
    let hata = calistir(kaynak, dosyalar).expect_err("T039 dönüş bildirimi ister");
    assert_eq!(hata.kod, "T039");
    assert_eq!((hata.satir, hata.sutun, hata.uzunluk), (1, 1, 1));
    assert!(hata.mesaj.contains("bir ver"), "{}", hata.mesaj);
    assert!(hata.mesaj.contains("sabitler"), "{}", hata.mesaj);
    assert!(hata.oneri.as_deref().unwrap().contains("<Tür> döndürür"));

    let tam = HashMap::from([(
        "sabitler",
        "işlem bir ver\n    TamSayı döndürür\n    1 döndür\n",
    )]);
    assert!(calistir(kaynak, tam.clone()).unwrap().is_empty());
    assert_eq!(
        calistir(
            "sabitler birimini kullan\nsonuç bir ver olsun\nsonuç yaz\n",
            tam
        )
        .unwrap(),
        ["1"]
    );
    let void = HashMap::from([(
        "sabitler",
        "işlem selam ver\n    değer döndürmez\n    \"selam\" yaz\n",
    )]);
    assert_eq!(
        calistir("sabitler birimini kullan\nselam ver\n", void).unwrap(),
        ["selam"]
    );
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

fn elmas_argumanli_dosyalari() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        (
            "ortak",
            "işlem kart hazırla\n    xi Metin olarak al\n    yi Metin olarak al\n    Metin döndürür\n    xi ile yi döndür\n",
        ),
        (
            "orta",
            "ortak birimini kullan\n\nişlem orta kart hazırla\n    xi Metin olarak al\n    Metin döndürür\n    s xi ve \"b\" ile kart hazırla olsun\n    s döndür\n",
        ),
    ])
}

#[test]
fn elmas_ice_alim_aracinin_gizli_kopyasi_dogrudan_tanimi_dusurmez() {
    // Aracı birim (`orta`) doğrudan içe alımdan ÖNCE gelirse, onun gizli
    // `kart hazırla` kopyası ana dosyanın ayrıştırma tohumundan düşmemeli;
    // argümanlı çağrı ancak tohumda tanınır (aksi hâlde S015).
    for kaynak in [
        "orta birimini kullan\nortak birimini kullan\n\ny \"a\" ve \"b\" ile kart hazırla olsun\nz \"a\" ile orta kart hazırla olsun\ny yaz\nz yaz\n",
        "ortak birimini kullan\norta birimini kullan\n\ny \"a\" ve \"b\" ile kart hazırla olsun\nz \"a\" ile orta kart hazırla olsun\ny yaz\nz yaz\n",
    ] {
        assert_eq!(
            calistir(kaynak, elmas_argumanli_dosyalari()).expect("içe alım sırasından bağımsız çalışmalı"),
            vec!["ab", "ab"]
        );
        let dosyalar = elmas_argumanli_dosyalari();
        let tanilar = dil::kaynagi_tanilari(kaynak, &mut yukleyici(&dosyalar));
        assert!(tanilar.is_empty(), "denetim de sıradan bağımsız temiz olmalı: {tanilar:?}");
    }
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

#[test]
fn rota_birimi_rotalari_kullan_satirinin_yerine_sirayla_katilir() {
    // K-211: `<ad> rotalarını kullan` — birimin üst düzey rotaları ana programa girer.
    let dosyalar = HashMap::from([
        (
            "panel",
            "yardimci birimini kullan\n\nGET \"/panel\" adresine istek geldiğinde\n    cevap \"panel\" için selam olsun\n    cevap yanıtını gönder\n",
        ),
        (
            "yardimci",
            "işlem selam\n    adı Metin olarak al\n    Metin döndürür\n\n    sonuc \"merhaba \" ile ad olsun\n    sonuc döndür\n",
        ),
    ]);
    let kaynak = "8080 kapısında sunucu başlat\n\nGET \"/\" adresine istek geldiğinde\n    \"ana\" yanıtını gönder\n\npanel rotalarını kullan\n\nGET \"/\" önekli adrese istek geldiğinde\n    \"son\" yanıtını gönder\n";
    let program = kaynagi_derle_birimlerle(kaynak, &mut yukleyici(&dosyalar)).expect("derlenmeli");
    let rotalar: Vec<String> = program
        .cumleler
        .iter()
        .filter_map(|c| match c {
            dil::agac::Cumle::IstekGeldiginde { yol, .. } => Some(format!("{yol:?}")),
            _ => None,
        })
        .collect();
    assert_eq!(rotalar.len(), 3, "{rotalar:?}");
    assert!(rotalar[1].contains("/panel"), "{rotalar:?}");
    assert!(
        program.islemler.contains_key("selam"),
        "rota biriminin tanımları da alınır"
    );

    // `birimini kullan` rotaları almaz (kapsülleme korunur).
    let sade = "panel birimini kullan\n\n8080 kapısında sunucu başlat\n";
    let program = kaynagi_derle_birimlerle(sade, &mut yukleyici(&dosyalar)).expect("derlenmeli");
    assert!(!program
        .cumleler
        .iter()
        .any(|c| matches!(c, dil::agac::Cumle::IstekGeldiginde { .. })));

    // Rotasız birim rota olarak alınamaz.
    let hata = kaynagi_derle_birimlerle("yardimci rotalarını kullan\n", &mut yukleyici(&dosyalar))
        .expect_err("A010 bekleniyordu");
    assert_eq!(hata.kod, "A010");
}

#[test]
fn elmas_ice_alimda_gorunurluk_sira_bagimsizdir_ve_denetim_yolu_rotalari_katar() {
    // K-211 düzeltmesi: `baska` doğrudan alınmış public işlem, onu kendi içinde alan
    // `orta` sonradan gelse de görünür kalır (strict ve tanı/denetim yolları).
    let dosyalar = HashMap::from([
        (
            "baska",
            "işlem baska\n    Metin döndürür\n\n    \"b\" döndür\n",
        ),
        (
            "orta",
            "baska birimini kullan\n\nişlem selam3\n    Metin döndürür\n\n    x baska olsun\n    x döndür\n",
        ),
        (
            "rota_p",
            "baska birimini kullan\n\nGET \"/p\" adresine istek geldiğinde\n    y baska olsun\n    y yanıtını gönder\n",
        ),
    ]);
    for kaynak in [
        "baska birimini kullan\norta birimini kullan\n\ny baska olsun\ny yaz\n",
        "orta birimini kullan\nbaska birimini kullan\n\ny baska olsun\ny yaz\n",
    ] {
        assert_eq!(
            calistir(kaynak, dosyalar.clone()).expect("çalışmalı"),
            vec!["b"]
        );
        let tanilar = dil::kaynagi_tanilari(kaynak, &mut yukleyici(&dosyalar));
        assert!(tanilar.is_empty(), "{tanilar:?}");
    }
    // Denetim yolu rota birimini de katar: rota gövdesindeki tanımsız ad tanı üretir.
    let kirik = HashMap::from([(
        "rota_k",
        "GET \"/k\" adresine istek geldiğinde\n    y bilinmeyen_islem olsun\n    y yanıtını gönder\n",
    )]);
    let tanilar = dil::kaynagi_tanilari(
        "8080 kapısında sunucu başlat\n\nrota_k rotalarını kullan\n",
        &mut yukleyici(&kirik),
    );
    assert!(
        !tanilar.is_empty(),
        "rota birimindeki hata denetimde görünmeli"
    );
    let temiz = dil::kaynagi_tanilari(
        "8080 kapısında sunucu başlat\n\nrota_p rotalarını kullan\n",
        &mut yukleyici(&dosyalar),
    );
    assert!(temiz.is_empty(), "{temiz:?}");
}
