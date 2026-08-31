//! Golden korpus regression testleri (master plan bölüm 5).
//!
//! Kural: desteklenen her golden program bu dosyada beklenen çıktısıyla
//! sabitlenir. Syntax değişikliği önce burada kırılır.

use dil::{kaynagi_calistir, kaynagi_calistir_girdiyle};

fn golden(ad: &str) -> String {
    let yol = format!("{}/../golden/{}", env!("CARGO_MANIFEST_DIR"), ad);
    std::fs::read_to_string(&yol).unwrap_or_else(|_| panic!("golden dosyası bulunamadı: {}", yol))
}

#[test]
fn golden_01_merhaba_dunya() {
    let cikti = kaynagi_calistir(&golden("01-merhaba-dunya.dil")).expect("01 çalışmalı");
    assert_eq!(cikti, vec!["Dünyaya merhaba"]);
}

#[test]
fn golden_02_degiskenler() {
    let cikti = kaynagi_calistir(&golden("02-degiskenler.dil")).expect("02 çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "Benim adım Ayşe",
            "Ayşe 10 yaşında",
            "Ayşe İzmir şehrinde yaşıyor",
        ]
    );
}

#[test]
fn golden_03_girdi_alma() {
    let cikti = kaynagi_calistir_girdiyle(&golden("03-girdi-alma.dil"), vec!["Zeynep".into()])
        .expect("03 çalışmalı");
    assert_eq!(cikti, vec!["Adın ne?", "Merhaba Zeynep"]);
}

#[test]
fn golden_04_hesap_makinesi() {
    let cikti = kaynagi_calistir_girdiyle(
        &golden("04-hesap-makinesi.dil"),
        vec!["7".into(), "2".into()],
    )
    .expect("04 çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "Birinci sayı?",
            "İkinci sayı?",
            "Toplam: 9",
            "Fark: 5",
            "Çarpım: 14",
            "Bölüm: 3",
        ]
    );
}

#[test]
fn sifira_bolme_calisma_hatasi() {
    let kaynak = "bölüm 10 un 0 a bölümü olsun\nbölümü yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("sıfıra bölme hata olmalı");
    assert_eq!(hata.kod, "C003");
}

#[test]
fn sayiya_cevrilemeyen_girdi() {
    let kaynak = "\"Sayı?\" diye sor\nx yanıtın sayısı olsun\nx yaz\n";
    let hata = kaynagi_calistir_girdiyle(kaynak, vec!["elma".into()])
        .expect_err("çevrilemeyen girdi hata olmalı");
    assert_eq!(hata.kod, "C004");
}

#[test]
fn golden_07_sayi_tahmini() {
    // Determinizm: "rastgele" sayı testte sabit kuyruktan gelir (gizli = 42).
    let program = dil::kaynagi_derle(&golden("07-sayi-tahmini.dil")).expect("07 derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(vec!["50".into(), "30".into(), "42".into()]);
    io.rastgele_degerler.push_back(42);
    dil::yorumlayici::calistir_io(&program, &mut io).expect("07 çalışmalı");
    assert_eq!(
        io.cikti,
        vec![
            "Tahminin?",
            "Daha küçük söyle",
            "Tahminin?",
            "Daha büyük söyle",
            "Tahminin?",
            "Bildin!",
        ]
    );
}

#[test]
fn golden_05_kosullar() {
    let cikti = kaynagi_calistir(&golden("05-kosullar.dil")).expect("05 çalışmalı");
    assert_eq!(cikti, vec!["İyi"]);
}

#[test]
fn golden_08_listeler() {
    let cikti = kaynagi_calistir(&golden("08-listeler.dil")).expect("08 çalışmalı");
    assert_eq!(
        cikti,
        vec!["Adet: 5", "İlk: 3", "Son: 5", "3", "7", "1", "9", "5"]
    );
}

#[test]
fn golden_09_liste_isleme() {
    let cikti = kaynagi_calistir(&golden("09-liste-isleme.dil")).expect("09 çalışmalı");
    assert_eq!(cikti, vec!["Toplam: 345", "Geçen sayısı: 3"]);
}

#[test]
fn bos_liste_ilki_derleme_hatasi() {
    // K-045: hiç eklenmemiş listenin ilki artık DERLEME hatası (öğe türü belirsiz).
    let kaynak = "sayılar boş liste olsun\nsayıların ilki yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("belirsiz listenin ilki T014");
    assert_eq!(hata.kod, "T014");
}

#[test]
fn kosullu_bos_kalan_listenin_ilki_c007() {
    // Tür eklemeyle somutlaştı ama çalışma anında liste boş kaldı → C007 yaşıyor.
    let kaynak = "\
sayılar boş liste olsun
1 2 den büyükse
    sayılara 5 ekle
sayıların ilki yaz
";
    let hata = kaynagi_calistir(kaynak).expect_err("boş kalan listenin ilki C007");
    assert_eq!(hata.kod, "C007");
}

#[test]
fn ortuk_cogul_bulunamazsa_hata() {
    let kaynak = "her elma için\n    elmayı yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("kapsamda elmalar yoksa hata");
    assert_eq!(hata.kod, "A003");
}

#[test]
fn golden_06_donguler() {
    let cikti = kaynagi_calistir(&golden("06-donguler.dil")).expect("06 çalışmalı");
    let mut beklenen: Vec<String> = std::iter::repeat_n("Merhaba".to_string(), 10).collect();
    beklenen.extend((1..=100).filter(|s| s % 2 == 0).map(|s| s.to_string()));
    beklenen.extend(["3", "2", "1"].map(String::from));
    assert_eq!(cikti, beklenen);
}

#[test]
fn golden_10_sozlukler() {
    let cikti = kaynagi_calistir(&golden("10-sozlukler.dil")).expect("10 çalışmalı");
    assert_eq!(cikti, vec!["Ayşe 10 yaşında", "Ayşe: 10", "Ali: 12"]);
}

#[test]
fn golden_11_metin_islemleri() {
    let cikti = kaynagi_calistir(&golden("11-metin-islemleri.dil")).expect("11 çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "Uzunluk: 24",
            "TÜRKÇE DÜŞÜN, TÜRKÇE YAZ",
            "türkçe düşün, türkçe yaz",
            "Geçiyor",
            "Türkçe",
            "düşün,",
            "Türkçe",
            "yaz",
        ]
    );
}

#[test]
fn golden_15_secenek_turu() {
    let cikti = kaynagi_calistir(&golden("15-secenek-turu.dil")).expect("15 çalışmalı");
    assert_eq!(cikti, vec!["Bulundu: 8"]);
}

#[test]
fn golden_16_sonuc_ve_hata() {
    // Dosya varsa: değer okunur.
    let program = dil::kaynagi_derle(&golden("16-sonuc-ve-hata.dil")).expect("16 derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert("veriler.txt".into(), "merhaba veri".into());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("16 çalışmalı");
    assert_eq!(io.cikti, vec!["merhaba veri"]);

    // Dosya yoksa: hata dalı çalışır, program ÇÖKMEZ.
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("hata dalı da çalışmalı");
    assert_eq!(io.cikti, vec!["Okunamadı: \"veriler.txt\" dosyası bulunamadı"]);
}

#[test]
fn golden_17_dosya_okuma() {
    let program = dil::kaynagi_derle(&golden("17-dosya-okuma.dil")).expect("17 derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert("siir.txt".into(), "bir\niki\nüç\n".into());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("17 çalışmalı");
    assert_eq!(io.cikti, vec!["bir", "iki", "üç", "Toplam 3 satır"]);
}

#[test]
fn golden_18_dosya_yazma() {
    let program = dil::kaynagi_derle(&golden("18-dosya-yazma.dil")).expect("18 derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("18 çalışmalı");
    assert_eq!(io.cikti, vec!["Günlük kaydedildi"]);
    assert_eq!(
        io.dosyalar.get("günlük.txt").map(String::as_str),
        Some("Bugün hava güzeldi\nYarın da güzel olsun\n")
    );
}

#[test]
fn olmayan_dosyanin_satirlari_turkce_hata() {
    let kaynak = "satırlar \"yok.txt\" dosyasının satırları olsun\nsatırların adedi yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("olmayan dosya çalışma hatası olmalı");
    assert_eq!(hata.kod, "C012");
    assert!(hata.oneri.as_deref().unwrap_or("").contains("dene"));
}

#[test]
fn turkce_buyuk_kucuk_harf_i_kurali() {
    // A07 anti-örneğindeki tuzak: i→İ, ı→I (İngilizce i→I DEĞİL).
    let kaynak = "ad \"izmir ılık\" olsun\nadın büyük harflisi yaz\nbaslik \"İZMİR ILIK\" olsun\nbasliğin küçük harflisi yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("Türkçe harf dönüşümü çalışmalı");
    assert_eq!(cikti, vec!["İZMİR ILIK", "izmir ılık"]);
}

#[test]
fn korumasiz_secenek_erisimi_derleme_hatasi() {
    // v0.2 akış-duyarlı daraltma (RFC-0008 §4.2): eski C008 çalışma hatası
    // artık derleme zamanında T036 olarak yakalanır — hata daha erken, daha iyi.
    let kaynak = "işlem hiç bulma\n    sayıyı al\n    sayı 0 dan küçükse\n        sayıyı döndür\n    yok döndür\n\nbulunan 5 için hiç bulma olsun\nbulunanın değeri yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("korumasız erişim T036");
    assert_eq!(hata.kod, "T036");
    assert!(hata.oneri.as_deref().unwrap_or("").contains("varsa"));
}

#[test]
fn golden_21_tarih_ve_saat() {
    // Sabit saat: 31 Ağustos 2026, 14:30 (ToplayanIo varsayılanı).
    let program = dil::kaynagi_derle(&golden("21-tarih-ve-saat.dil")).expect("21 derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("21 çalışmalı");
    assert_eq!(
        io.cikti,
        vec![
            "Bugün: 31 Ağustos 2026",
            "Yıl: 2026",
            "Saat: 14:30",
            "Yarın: 1 Eylül 2026",
        ]
    );
}

#[test]
fn golden_28_cli_araci() {
    let program = dil::kaynagi_derle(&golden("28-cli-araci.dil")).expect("28 derlenmeli");

    // Argümanlarla: her birine selam.
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    io.argumanlar = vec!["Zeynep".into(), "Eliz".into()];
    dil::yorumlayici::calistir_io(&program, &mut io).expect("28 çalışmalı");
    assert_eq!(io.cikti, vec!["Merhaba Zeynep", "Merhaba Eliz"]);

    // Argümansız: kullanım yazısı + programı bitir (çökme yok, döngü koşulmaz).
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("bitir olağan sonlanmadır");
    assert_eq!(io.cikti, vec!["Kullanım: selamla <isim> <isim> ..."]);
}

#[test]
fn yil_donumu_gecisi() {
    let kaynak = "bugün bugünün tarihi olsun\nsonra bugünün 130 gün sonrası olsun\nsonrayı yaz\n";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("çalışmalı");
    // 31 Ağu 2026 + 130 gün = 8 Ocak 2027.
    assert_eq!(io.cikti, vec!["8 Ocak 2027"]);
}

#[test]
fn golden_22_yapilar() {
    let cikti = kaynagi_calistir(&golden("22-yapilar.dil")).expect("22 çalışmalı");
    assert_eq!(cikti, vec!["Ayşe 10 yaşında"]);
}

#[test]
fn yapida_olmayan_alan_turkce_hata() {
    let kaynak = "yapı Kedi\n    ad Metin\n\nkedi yeni Kedi olsun\nkedinin kuyruğu 5 olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("olmayan alan hata olmalı");
    assert_eq!(hata.kod, "T028");
    assert!(hata.mesaj.contains("ad"), "hata alan listesini saymalı: {}", hata.mesaj);
}

#[test]
fn golden_23_desen_eslestirme() {
    let cikti = kaynagi_calistir(&golden("23-desen-eslestirme.dil")).expect("23 çalışmalı");
    assert_eq!(cikti, vec!["4 köşesi var"]);
}

#[test]
fn unlu_dusmesi_geri_cevrimi() {
    // "şekle göre" → şekil; "burnu yaz" → burun.
    let kaynak = "burun 5 olsun\nburnu yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("burnu → burun çözülmeli");
    assert_eq!(cikti, vec!["5"]);
}

#[test]
fn golden_12_islem_tanimi() {
    let cikti = kaynagi_calistir(&golden("12-islem-tanimi.dil")).expect("12 çalışmalı");
    assert_eq!(cikti, vec!["90"]);
}

#[test]
fn golden_13_islem_parametreleri() {
    let cikti = kaynagi_calistir(&golden("13-islem-parametreleri.dil")).expect("13 çalışmalı");
    assert_eq!(cikti, vec!["Merhaba genç Ayşe", "Merhaba Mustafa"]);
}

#[test]
fn golden_14_not_ortalamasi() {
    let cikti = kaynagi_calistir(&golden("14-not-ortalamasi.dil")).expect("14 çalışmalı");
    assert_eq!(cikti, vec!["Ortalama: 76", "Geçtin"]);
}

#[test]
fn temel_durumsuz_ozyineleme_reddedilir() {
    // v0.2: özyineleme serbest ama temel durum ÖNCE gelmeli (T035).
    let kaynak = "işlem kendini çağır\n    kendini çağır\n\nkendini çağır\n";
    let hata = kaynagi_calistir(kaynak).expect_err("temel durumsuz özyineleme T035");
    assert_eq!(hata.kod, "T035");
}

#[test]
fn dondurmeyen_islem_ifadede_reddedilir() {
    let kaynak = "işlem selam ver\n    \"selam\" yaz\n\nx selam ver olsun\nx yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("değer döndürmeyen işlem ifadede hata olmalı");
    assert_eq!(hata.kod, "T019");
}

#[test]
fn golden_19_csv_analizi() {
    let program = dil::kaynagi_derle(&golden("19-csv-analizi.dil")).expect("19 derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    io.dosyalar
        .insert("notlar.csv".into(), "not\n45\n90\n72\n38\n100\n".into());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("19 çalışmalı");
    assert_eq!(io.cikti, vec!["Ortalama: 69"]);
}

#[test]
fn golden_20_json_verisi() {
    let program = dil::kaynagi_derle(&golden("20-json-verisi.dil")).expect("20 derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert(
        "kisi.json".into(),
        "{\n  \"ad\": \"Ayşe\",\n  \"şehir\": \"İzmir\"\n}\n".into(),
    );
    dil::yorumlayici::calistir_io(&program, &mut io).expect("20 çalışmalı");
    assert_eq!(io.cikti, vec!["Ad: Ayşe", "Şehir: İzmir"]);
}

#[test]
fn csv_sayi_olmayan_hucre_turkce_hata() {
    let kaynak = "tablo \"t.csv\" dosyasından okunan tablo olsun\ntablonun adedi yaz\n";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert("t.csv".into(), "ad,not\nAyşe,90\n".into());
    let hata = dil::yorumlayici::calistir_io(&program, &mut io).expect_err("hücre hatası");
    assert_eq!(hata.kod, "C015");
    assert!(hata.mesaj.contains("Ayşe"), "sorunlu hücre gösterilmeli: {}", hata.mesaj);
}

#[test]
fn golden_30_testler() {
    let sonuclar = dil::kaynagi_dene(&golden("30-testler.dil")).expect("30 derlenmeli");
    assert_eq!(sonuclar.len(), 2);
    for sonuc in &sonuclar {
        assert!(sonuc.hata.is_none(), "test geçmeli: {} — {:?}", sonuc.ad, sonuc.hata);
    }
}

#[test]
fn kalan_test_beklenen_bulunan_gosterir() {
    let kaynak = "test \"bilerek kalan\"\n    x 2 ile 2 nin toplamı olsun\n    x 5 e eşit olmalı\n";
    let sonuclar = dil::kaynagi_dene(kaynak).expect("derlenmeli");
    let hata = sonuclar[0].hata.as_ref().expect("test kalmalı");
    assert_eq!(hata.kod, "D001");
    assert!(
        hata.mesaj.contains("Beklenen: 5") && hata.mesaj.contains("bulunan: 4"),
        "beklenen/bulunan gösterilmeli: {}",
        hata.mesaj
    );
}

#[test]
fn calistir_testleri_atlar() {
    let kaynak = "\"program\" yaz\n\ntest \"ayrı dünya\"\n    1 1 e eşit olmalı\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["program"], "test blokları çalıştırmada koşulmamalı");
}

#[test]
fn coklu_tani_toplanir() {
    // Üç bağımsız hata tek geçişte raporlanır (RFC-0010 §3.1).
    let kaynak = "\
bilinmeyeni yaz
x \"a\" olsun
x 5 den büyükse
    x yaz
tanınmayan bir şey
son 1 olsun
sonu yaz
";
    let mut yukleyici = |_: &str| Err("yok".to_string());
    let tanilar = dil::kaynagi_tanilari(kaynak, &mut yukleyici);
    let kodlar: Vec<&str> = tanilar.iter().map(|t| t.kod.as_str()).collect();
    assert!(kodlar.len() >= 3, "en az 3 tanı: {:?}", kodlar);
    assert!(kodlar.contains(&"A001"), "{:?}", kodlar);
    assert!(kodlar.contains(&"T001"), "{:?}", kodlar);
    assert!(kodlar.contains(&"S004"), "{:?}", kodlar);
}

#[test]
fn coklu_tani_temiz_dosyada_bos() {
    let mut yukleyici = |_: &str| Err("yok".to_string());
    let tanilar = dil::kaynagi_tanilari("\"selam\" yaz\n", &mut yukleyici);
    assert!(tanilar.is_empty(), "{:?}", tanilar.first().map(|t| &t.mesaj));
}

// ---- compile-fail: anti-örnekler ve tanı kalitesi ----

#[test]
fn tur_hatasi_turkce_ve_kodlu() {
    let kaynak = "toplam \"Mustafa\" olsun\ntoplam 10 dan büyükse\n    \"olmaz\" yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("tür hatası bekleniyor");
    assert_eq!(hata.kod, "T001");
    assert!(hata.mesaj.contains("Metin"), "mesaj Türkçe tür adı içermeli: {}", hata.mesaj);
}

#[test]
fn tanimsiz_ad_hatasi() {
    let kaynak = "sayıyı yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("tanımsız ad hatası bekleniyor");
    assert_eq!(hata.kod, "A001");
}

#[test]
fn tur_degisimi_reddedilir() {
    let kaynak = "x 5 olsun\nx \"metin\" olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("tür değişimi reddedilmeli");
    assert_eq!(hata.kod, "T002");
}

#[test]
fn sekme_girinti_reddedilir() {
    let kaynak = "5 kez tekrarla\n\t\"a\" yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("sekme reddedilmeli");
    assert_eq!(hata.kod, "S003");
}

#[test]
fn ve_zinciri_ve_kisa_devre() {
    let kaynak = "yaş 16 olsun\nyaş 8 veya daha büyükse ve yaş 18 den küçükse\n    \"genç\" yaz\ndeğilse\n    \"değil\" yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("ve zinciri çalışmalı");
    assert_eq!(cikti, vec!["genç"]);
}

#[test]
fn veya_zinciri() {
    let kaynak = "gün 7 olsun\ngün 6 ya eşitse veya gün 7 ye eşitse\n    \"hafta sonu\" yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("veya zinciri çalışmalı");
    assert_eq!(cikti, vec!["hafta sonu"]);
}

#[test]
fn ve_veya_karisimi_reddedilir() {
    let kaynak = "x 1 olsun\nx 1 e eşitse ve x 2 ye eşitse veya x 3 e eşitse\n    \"olmaz\" yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("karışım belirsizdir");
    assert_eq!(hata.kod, "S030");
}

#[test]
fn a03_dogrusu_calisir() {
    // anti-ornekler/A03'ün "doğrusu" bölümü: sembolsüz mantık, değilse olumsuzlaması.
    let kaynak = "a 5 olsun\nb 7 olsun\nc 9 olsun\nd 3 olsun\ne yanlış olsun\n\na b ye eşit değilse ve c d den küçük değilse\n    \"birinci\" yaz\ndeğilse e doğru değilse\n    \"ikinci\" yaz\ndeğilse\n    \"üçüncü\" yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("A03 doğrusu çalışmalı");
    assert_eq!(cikti, vec!["birinci"]);
}

#[test]
fn mantiksal_degerin_olumsuzu() {
    let kaynak = "bayrak yanlış olsun\nbayrak değilse\n    \"kapalı\" yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("tekil olumsuzlama çalışmalı");
    assert_eq!(cikti, vec!["kapalı"]);
}

#[test]
fn a08_homoglyph_reddedilir() {
    // anti-ornekler/A08: Kiril "а" (U+0430) Latin "a" ile görünüşte özdeş.
    let kaynak = "s\u{0430}yı 5 olsun\nsayıyı yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("homoglyph reddedilmeli");
    assert_eq!(hata.kod, "S028");
    assert!(hata.mesaj.contains("U+0430"), "kod noktası gösterilmeli: {}", hata.mesaj);
}

#[test]
fn birlestirici_im_reddedilir() {
    // "ğ" yerine g + U+0306 (breve): NFC zorunluluğu (RFC-0002).
    let kaynak = "dag\u{0306} 5 olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("birleştirici im reddedilmeli");
    assert_eq!(hata.kod, "S029");
}

#[test]
fn sapkali_unluler_kabul_edilir() {
    // Türkçe yazımdaki şapkalı ünlüler (kâr) tanımlayıcıda geçerlidir.
    let kaynak = "kâr 100 olsun\nkârı yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("şapkalı ünlü çalışmalı");
    assert_eq!(cikti, vec!["100"]);
}

#[test]
fn a06_parantezli_blok_reddedilir() {
    // anti-ornekler/A06: girinti yerine süslü parantez.
    let kaynak = "10 kez tekrarla {\n    \"Merhaba\" yaz\n}\n";
    let hata = kaynagi_calistir(kaynak).expect_err("süslü parantez reddedilmeli");
    assert_eq!(hata.kod, "S001");
}

#[test]
fn tani_json_cikti_ve_kacis() {
    // Mesajlarda çift tırnak geçer ("x" adı...): JSON kaçışı doğru olmalı.
    let hata = kaynagi_calistir("bilinmeyeni yaz\n").expect_err("A001 bekleniyor");
    let json = hata.json();
    assert!(json.starts_with("{\"kod\":\"A001\""), "kod alanı: {}", json);
    assert!(json.contains("\\\"bilinmeyeni\\\""), "tırnaklar kaçışlanmalı: {}", json);
    assert!(json.contains("\"satir\":1"), "konum alanları: {}", json);
    assert!(json.ends_with("}"), "geçerli nesne: {}", json);
}

#[test]
fn rapor_ayrinti_satiri_icerir() {
    let kaynak = "x \"a\" olsun\nx 5 den büyükse\n    x yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("T001");
    let rapor = hata.raporla(kaynak);
    assert!(
        rapor.contains("Ayrıntı için: dil hata T001"),
        "rapor katalog bağlantısı içermeli: {}",
        rapor
    );
}

// ---- ad çözümleme: ek ayıklama (K-011) ----

#[test]
fn ek_ayiklama_unsuz_yumusamasi() {
    // "sayacı" → sayac → ünsüz sertleşmesi geri çevrimi → "sayaç"
    let kaynak = "sayaç 5 olsun\nsayacı yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("sayacı → sayaç çözülmeli");
    assert_eq!(cikti, vec!["5"]);
}

#[test]
fn belirsiz_ad_reddedilir() {
    // Hem "sayı" hem "sayıyı" tanımlıysa "sayıyı" doğrudan eşleşir (belirsizlik yok);
    // ama hem "sayaç" hem "sayac" tanımlıyken "sayacı" iki köke çözülür → A002.
    let kaynak = "sayaç 1 olsun\nsayac 2 olsun\nsayacı yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("belirsizlik hata olmalı");
    assert_eq!(hata.kod, "A002");
}
