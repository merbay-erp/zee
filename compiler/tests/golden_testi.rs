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
fn golden_06_donguler() {
    let cikti = kaynagi_calistir(&golden("06-donguler.dil")).expect("06 çalışmalı");
    let mut beklenen: Vec<String> = std::iter::repeat("Merhaba".to_string()).take(10).collect();
    beklenen.extend((1..=100).filter(|s| s % 2 == 0).map(|s| s.to_string()));
    beklenen.extend(["3", "2", "1"].map(String::from));
    assert_eq!(cikti, beklenen);
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
