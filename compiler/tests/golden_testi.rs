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
fn bos_liste_ilki_calisma_hatasi() {
    let kaynak = "sayılar boş liste olsun\nsayıların ilki yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("boş listenin ilki hata olmalı");
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
    let mut beklenen: Vec<String> = std::iter::repeat("Merhaba".to_string()).take(10).collect();
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
fn bos_secenegin_degeri_calisma_hatasi() {
    let kaynak = "işlem hiç bulma\n    sayıyı al\n    sayı 0 dan küçükse\n        sayıyı döndür\n    yok döndür\n\nbulunan 5 için hiç bulma olsun\nbulunanın değeri yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("boş Seçenek'in değeri hata olmalı");
    assert_eq!(hata.kod, "C008");
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
fn ozyineleme_v0_reddedilir() {
    let kaynak = "işlem kendini çağır\n    kendini çağır\n\nkendini çağır\n";
    let hata = kaynagi_calistir(kaynak).expect_err("özyineleme v0'da reddedilmeli");
    assert_eq!(hata.kod, "T016");
}

#[test]
fn dondurmeyen_islem_ifadede_reddedilir() {
    let kaynak = "işlem selam ver\n    \"selam\" yaz\n\nx selam ver olsun\nx yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("değer döndürmeyen işlem ifadede hata olmalı");
    assert_eq!(hata.kod, "T019");
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
fn a06_parantezli_blok_reddedilir() {
    // anti-ornekler/A06: girinti yerine süslü parantez.
    let kaynak = "10 kez tekrarla {\n    \"Merhaba\" yaz\n}\n";
    let hata = kaynagi_calistir(kaynak).expect_err("süslü parantez reddedilmeli");
    assert_eq!(hata.kod, "S001");
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
