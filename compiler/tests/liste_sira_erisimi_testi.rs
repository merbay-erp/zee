//! K-386 / RFC-0029 / ADR-166: listede 1 tabanlı sıra ile öğe erişimi.
//!
//! `<liste>nin <sıra>. öğesi` öğenin kopyasını verir; sınır dışı çalışma
//! hatası C031'dir. `… öğesini almayı dene` Sonuç<T> döndürür ve sınır dışını
//! `SIRA_DISI` kodlu Hata olarak taşır. Sıra TamSayı olmalı (T058), sabit sıra
//! 1'den küçükse derleme hatasıdır (T059); kalıba oturmayan sıra eki S049.

use dil::{bicimleyici::bicimle, kaynagi_calistir, kaynagi_denetle};

fn calistir(kaynak: &str) -> Vec<String> {
    kaynagi_calistir(kaynak).unwrap_or_else(|hata| panic!("çalışmalı: {hata:?}"))
}

fn hata(kaynak: &str) -> dil::tani::Tani {
    kaynagi_calistir(kaynak).expect_err("hata vermeli")
}

const HARFLER: &str = "harfler \"a\", \"b\", \"c\", \"d\", \"e\" listesi olsun\n";

#[test]
fn ilk_son_ve_orta_oge_bir_tabanlidir() {
    let kaynak = format!(
        "{HARFLER}harflerin 1. öğesi yaz\nharflerin 5. öğesi yaz\nsıra 3 olsun\nharflerin sıra. öğesi yaz\n"
    );
    assert_eq!(calistir(&kaynak), vec!["a", "e", "c"]);
}

#[test]
fn sira_ilki_ve_sonuyla_tutarlidir() {
    let kaynak = format!(
        "{HARFLER}adet harflerin adedi olsun\nilk harflerin 1. öğesi olsun\nson harflerin adet. öğesi olsun\nbaş harflerin ilki olsun\nuç harflerin sonu olsun\nilk başa eşitse\n    \"ilk tamam\" yaz\nson uca eşitse\n    \"son tamam\" yaz\n"
    );
    assert_eq!(calistir(&kaynak), vec!["ilk tamam", "son tamam"]);
}

#[test]
fn birlestirme_ve_islem_argumani_icinde_calisir() {
    let kaynak = format!(
        "işlem selamla\n    adı Metin olarak al\n    Metin döndürür\n\n    \"merhaba \" ile ad olsun\n\n{HARFLER}\"ikinci: \" ile harflerin 2. öğesi yaz\nselam harflerin 4. öğesi için selamla olsun\nselam yaz\n"
    );
    let kaynak = kaynak.replace(
        "    \"merhaba \" ile ad olsun\n",
        "    selam \"merhaba \" ile ad olsun\n    selamı döndür\n",
    );
    assert_eq!(calistir(&kaynak), vec!["ikinci: b", "merhaba d"]);
}

#[test]
fn sifir_ve_n_arti_bir_calisma_zamaninda_c031() {
    for (sira, beklenen) in [("0", "0."), ("6", "6."), ("-2", "-2.")] {
        let kaynak = format!("{HARFLER}x {sira} olsun\nharflerin x. öğesi yaz\n");
        let tani = hata(&kaynak);
        assert_eq!(tani.kod, "C031", "sıra {sira}");
        assert!(tani.mesaj.contains(beklenen), "{}", tani.mesaj);
        assert!(tani.mesaj.contains("5 öğeli"), "{}", tani.mesaj);
        assert_eq!((tani.satir, tani.sutun, tani.uzunluk), (3, 11, 1));
        assert!(tani.oneri.as_deref().unwrap_or("").contains("almayı dene"));
    }
}

#[test]
fn bos_liste_calisma_zamaninda_c031_derlemede_t058() {
    // Türü belli ama içi boşalmış liste: derlenir, okuma çalışırken reddedilir.
    let kaynak = "adlar \"a\" listesi olsun\nadlar boş liste olsun\nadlar boşsa\n    \"boş\" yaz\nadlara \"b\" ekle\nadlar boş liste olsun\nx 1 olsun\nadların x. öğesi yaz\n";
    let tani = hata(kaynak);
    assert_eq!(tani.kod, "C031");
    assert!(tani.mesaj.contains("0 öğeli"), "{}", tani.mesaj);

    // Türü hiç somutlaşmamış boş liste derleme hatasıdır.
    let tani = kaynagi_denetle("adlar boş liste olsun\nadların 1. öğesi yaz\n").expect_err("T058");
    assert_eq!(tani.kod, "T058");
    assert_eq!((tani.satir, tani.sutun), (2, 1));
}

#[test]
fn sabit_sifir_ve_negatif_sira_derleme_hatasi_t059() {
    for (sira, sutun, uzunluk) in [("0", 11, 1), ("-1", 11, 2)] {
        let kaynak = format!("{HARFLER}harflerin {sira}. öğesi yaz\n");
        let tani = kaynagi_denetle(&kaynak).expect_err("T059");
        assert_eq!(tani.kod, "T059", "sıra {sira}");
        assert_eq!((tani.satir, tani.sutun, tani.uzunluk), (2, sutun, uzunluk));
        assert!(tani.oneri.as_deref().unwrap_or("").contains("1. öğesi"));
    }
}

#[test]
fn sira_tamsayi_nesne_liste_olmali_t058() {
    let tani =
        kaynagi_denetle(&format!("{HARFLER}harflerin \"1\". öğesi yaz\n")).expect_err("T058");
    assert_eq!(tani.kod, "T058");
    assert!(tani.mesaj.contains("TamSayı"), "{}", tani.mesaj);

    let tani = kaynagi_denetle(&format!("{HARFLER}x 1,5 olsun\nharflerin x. öğesi yaz\n"))
        .expect_err("T058");
    assert_eq!(tani.kod, "T058");
    assert!(tani.mesaj.contains("Ondalık"), "{}", tani.mesaj);

    let tani = kaynagi_denetle("ad \"abc\" olsun\nadın 1. öğesi yaz\n").expect_err("T058");
    assert_eq!(tani.kod, "T058");
    assert!(tani.mesaj.contains("liste ister"), "{}", tani.mesaj);
    assert!(tani.oneri.as_deref().unwrap_or("").contains("harfleri"));
}

#[test]
fn almayi_dene_sonuc_dondurur_ve_sira_disini_hata_olarak_tasir() {
    let kaynak = format!(
        "{HARFLER}iyi harflerin 2. öğesini almayı dene olsun\niyi başarılıysa\n    iyinin değeri yaz\nkötü harflerin 9. öğesini almayı dene olsun\nkötü başarısızsa\n    h kötünün hatası olsun\n    hnin kodu yaz\n    hnin mesajı yaz\n"
    );
    let kaynak = kaynak
        .replace("hnin", "hatanın")
        .replace("    h kötünün", "    hata kötünün");
    assert_eq!(
        calistir(&kaynak),
        vec!["b", "SIRA_DISI", "listenin 9. öğesi yok (5 öğe)"]
    );
}

#[test]
fn almayi_dene_bos_listede_cokmez() {
    let kaynak = "adlar \"a\" listesi olsun\nadlar boş liste olsun\nx 1 olsun\ns adların x. öğesini almayı dene olsun\ns başarısızsa\n    \"yok\" yaz\n";
    assert_eq!(calistir(kaynak), vec!["yok"]);
}

#[test]
fn almayi_dene_sonucu_denetlenmeden_okunamaz() {
    // RFC-0008/T036: Sonuç değeri başarı denetimi olmadan açılamaz.
    let tani = kaynagi_denetle(&format!(
        "{HARFLER}sonuç harflerin 2. öğesini almayı dene olsun\nsonucun değeri yaz\n"
    ))
    .expect_err("T036");
    assert_eq!(tani.kod, "T036");
}

#[test]
fn erisilen_oge_kopyadir_kaynak_liste_degismez() {
    let kaynak = "\
yapı Öğrenci
    ad Metin

ali yeni Öğrenci olsun
alinin adı \"Ali\" olsun
öğrenciler boş liste olsun
öğrencilere ali ekle
seçilen öğrencilerin 1. öğesi olsun
seçilenin adı \"Veli\" olsun
seçilenin adı yaz
ilk öğrencilerin 1. öğesi olsun
ilkin adı yaz
";
    assert_eq!(calistir(kaynak), vec!["Veli", "Ali"]);
}

#[test]
fn ic_ice_liste_ogesi_bagimsiz_kopyadir() {
    let kaynak = "\
tablo boş liste olsun
ham1 \"a,b,c\" olsun
satır1 ham1in \",\" ile parçaları olsun
tabloya satır1 ekle
ham2 \"d,e\" olsun
satır2 ham2nin \",\" ile parçaları olsun
tabloya satır2 ekle
ikinci tablonun 2. öğesi olsun
ikincinin 2. öğesi yaz
ikinciye \"z\" ekle
ikincinin adedi yaz
yine tablonun 2. öğesi olsun
yinenin adedi yaz
";
    assert_eq!(calistir(kaynak), vec!["e", "3", "2"]);
}

#[test]
fn gezme_sirasinda_sira_ile_okuma_t053_degildir() {
    // spec/17 T053: gezilen koleksiyon gezme sırasında DEĞİŞTİRİLEMEZ; sıra ile
    // okumak değiştirme değildir ve aynı listeyi gezerken serbesttir.
    let kaynak = format!(
        "{HARFLER}adet harflerin adedi olsun\nsıra 1 olsun\nher harf için\n    karşı harflerin adet. öğesi olsun\n    harf ile karşı yaz\n    adedi 1 azalt\n"
    );
    assert_eq!(calistir(&kaynak), vec!["ae", "bd", "cc", "db", "ea"]);

    // Gezilen listeye ekleme hâlâ T053'tür; sıra erişimi bunu gevşetmez.
    let tani = kaynagi_denetle(&format!(
        "{HARFLER}her harf için\n    ilk harflerin 1. öğesi olsun\n    harflere ilki ekle\n"
    ))
    .expect_err("T053");
    assert_eq!(tani.kod, "T053");
}

#[test]
fn hatali_sira_eki_kullanimi_s049_ve_s001() {
    for (satir, beklenen) in [
        ("harflerin 1. öğeleri yaz\n", "S049"),
        (
            "i 1 olsun\nharflerin i ile 1 in toplamı. öğesi yaz\n",
            "S049",
        ),
        ("harflerin 1 . öğesi yaz\n", "S001"),
        ("harflerin 1.öğesi yaz\n", "S001"),
        ("x 3.14 olsun\n", "S001"),
        ("\"son.\" yaz\nharflerin sonu.\n", "S001"),
    ] {
        let tani = kaynagi_denetle(&format!("{HARFLER}{satir}")).expect_err(satir);
        assert_eq!(tani.kod, beklenen, "{satir:?}");
    }
    // S049 önerisi doğru kalıbı ve ara ad çözümünü gösterir.
    let tani = kaynagi_denetle(&format!("{HARFLER}harflerin 1. öğeleri yaz\n")).expect_err("S049");
    let oneri = tani.oneri.unwrap_or_default();
    assert!(oneri.contains("harflerin 3. öğesi"), "{oneri}");
    let tani = kaynagi_denetle(&format!(
        "{HARFLER}i 1 olsun\nharflerin i ile 1 in toplamı. öğesi yaz\n"
    ))
    .expect_err("S049");
    assert!(tani
        .oneri
        .unwrap_or_default()
        .contains("konum sıra ile 1 in toplamı olsun"));
}

#[test]
fn sira_ile_yazma_acik_degildir_s049_gezme_yazmasini_onerir() {
    let tani = kaynagi_denetle(&format!("{HARFLER}harflerin 2. öğesi \"x\" olsun\n"))
        .expect_err("yazma yok");
    assert_eq!(tani.kod, "S049");
    assert!(tani.mesaj.contains("YAZMA"), "{}", tani.mesaj);
    assert_eq!((tani.satir, tani.sutun), (2, 12));
    assert!(tani.oneri.unwrap_or_default().contains("gezme yazmasını"));
}

#[test]
fn nokta_metin_icinde_ve_ondalikta_etkilenmez() {
    let kaynak = format!("{HARFLER}\"3. öğe\" yaz\nx 3,5 olsun\nx yaz\n");
    assert_eq!(calistir(&kaynak), vec!["3. öğe", "3,5"]);
}

#[test]
fn bicimleyici_sira_ekini_korur() {
    let kaynak = format!("{HARFLER}sıra   2 olsun\nharflerin  sıra.   öğesi yaz\n");
    let bicimli = bicimle(&kaynak).expect("biçimlenmeli");
    assert!(bicimli.contains("harflerin sıra. öğesi yaz"), "{bicimli}");
    assert_eq!(bicimle(&bicimli).expect("idempotent"), bicimli);
}

#[test]
fn test_blogunda_dogrulama_ile_kullanilir() {
    let kaynak = format!(
        "test \"üçüncü harf\"\n    {}    üçüncü harflerin 3. öğesi olsun\n    üçüncü \"c\" ye eşit olmalı\n",
        HARFLER
    );
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    let sonuclar = dil::programi_dene(&program);
    assert_eq!(sonuclar.len(), 1);
    assert!(sonuclar[0].hata.is_none(), "{:?}", sonuclar[0].hata);
}

#[test]
fn koseli_parantez_indeksi_sira_erisimini_onerir() {
    for satir in ["harfler[0] yaz\n", "x harfler[1] olsun\n"] {
        let tani = kaynagi_denetle(&format!("{HARFLER}{satir}")).expect_err(satir);
        assert_eq!(tani.kod, "S001", "{satir:?}");
        let oneri = tani.oneri.unwrap_or_default();
        assert!(oneri.contains("listenin 1. öğesi"), "{oneri}");
    }
    let tani = kaynagi_denetle(&format!("{HARFLER}harflerin 1 . öğesi yaz\n")).expect_err("S001");
    assert!(tani.oneri.unwrap_or_default().contains("listenin 3. öğesi"));
}

#[test]
fn lsp_sira_hatasini_yayinlar_ogesi_hover_ve_yeniden_adlandirma_calisir() {
    use dil::lsp::Sunucu;
    let mut sunucu = Sunucu::yeni();
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/sira.dil","text":"harfler \"a\", \"b\" listesi olsun\nharflerin 0. öğesi yaz\n"}}}"#);
    let yayin = &cikti.govdeler[0];
    assert!(yayin.contains("\"code\":\"T059\""), "{yayin}");
    assert!(yayin.contains("\"line\":1"), "{yayin}");

    let metin = r#"harfler \"a\", \"b\" listesi olsun\nsıra 2 olsun\nharf harflerin sıra. öğesi olsun\nharf yaz\n"#;
    let cikti = sunucu.mesaj_isle(&format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///tmp/sira2.dil","text":"{metin}"}}}}}}"#
    ));
    assert!(
        cikti.govdeler[0].contains("\"diagnostics\":[]"),
        "{}",
        cikti.govdeler[0]
    );

    // 2. satır, 22. karakter → "öğesi" kalıp kelimesi.
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/sira2.dil"},"position":{"line":2,"character":22}}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("**öğesi**"), "{yanit}");
    assert!(yanit.contains("RFC-0029"), "{yanit}");

    // `sıra` → `konum`: sıra eki konumundaki kullanım da yeniden adlandırılır.
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":4,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/sira2.dil"},"position":{"line":1,"character":1},"newName":"konum"}}"#);
    let yanit = &cikti.govdeler[0];
    assert_eq!(yanit.matches("\"newText\":\"konum\"").count(), 2, "{yanit}");
}
