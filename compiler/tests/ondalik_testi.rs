//! Ondalık sayılar (RFC-0013) — bitişik virgül kuralı ve onluk tam aritmetik.

use dil::kaynagi_calistir;

#[test]
fn sifir_bir_arti_sifir_iki_tam_olarak_sifir_uc() {
    // Dilin verdiği söz: ikili kayan nokta sürprizi YOK.
    let kaynak = "x 0,1 ile 0,2 nin toplamı olsun\nx yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["0,3"]);
}

#[test]
fn ondalik_sabit_ve_basim() {
    let kaynak = "pi 3,14 olsun\npi yaz\nfiyat 2,50 olsun\nfiyat yaz\ntam 2,0 olsun\ntam yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    // 2,50 → 2,5 (sondaki sıfır atılır); 2,0 → 2,0 (Ondalık kimliği korunur).
    assert_eq!(cikti, vec!["3,14", "2,5", "2,0"]);
}

#[test]
fn bitisik_virgul_liste_ayraci_karismaz() {
    let kaynak = "fiyatlar 2,5, 7,25, 10,0 listesi olsun\nfiyatların adedi yaz\nfiyatların ilki yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["3", "2,5"]);
}

#[test]
fn sayisal_karisim_listede_genisler() {
    // TamSayı + Ondalık karışımı Ondalık'a genişler; öğeler gerçekten dönüşür.
    let kaynak = "fiyatlar 2,5, 10 listesi olsun\nfiyatların sonu yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["10,0"]);
}

#[test]
fn karisik_aritmetik_ve_bolme() {
    let kaynak = "\
kilo 2,5 olsun
birim 19,99 olsun
tutar kilo ile birimin çarpımı olsun
tutar yaz
yarısı tutarın 2 ye bölümü olsun
yarısı yaz
iki_kati tutar ile 2 nin çarpımı olsun
iki_kati yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["49,975", "24,9875", "99,95"]);
}

#[test]
fn tam_bolme_degismedi() {
    // İki TamSayı bölmesi TAM kalır (golden 04/12/14 sözleşmesi).
    let kaynak = "bölüm 10 un 4 e bölümü olsun\nbölüm yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["2"]);
}

#[test]
fn ondalik_bolme_yuvarlama() {
    // 10,0 / 3 → 34 anlamlı hane, yarımlar sıfırdan uzağa.
    let kaynak = "bölüm 10,0 ın 3 e bölümü olsun\nbölüm yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["3,333333333333333333333333333333333"]);
}

#[test]
fn karsilastirma_deger_uzerinden() {
    let kaynak = "\
a 1,50 olsun
a 1,5 e eşitse
    \"eşit\" yaz
b 2,0 olsun
b 2 ye eşitse
    \"tam eşit\" yaz
c 1,5 olsun
c 2 den küçükse
    \"küçük\" yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["eşit", "tam eşit", "küçük"]);
}

#[test]
fn artir_azalt_ondalik() {
    let kaynak = "fiyat 1,5 olsun\nfiyatı 0,25 artır\nfiyat yaz\nfiyatı 1 azalt\nfiyat yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["1,75", "0,75"]);
}

#[test]
fn tam_kismi_ve_yuvarlanmisi() {
    let kaynak = "\
sayı 3,7 olsun
sayının tam kısmı yaz
sayının yuvarlanmışı yaz
para 2,5 olsun
paranın yuvarlanmışı yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    // Kırpma 3; yuvarlama 4; 2,5 yarım → sıfırdan uzağa → 3 (okul kuralı).
    assert_eq!(cikti, vec!["3", "4", "3"]);
}

#[test]
fn yanitin_ondaligi() {
    let kaynak = "\"Kilo?\" diye sor\nkilo yanıtın ondalığı olsun\nkilo ile 2 nin çarpımı yaz\n";
    let cikti = dil::kaynagi_calistir_girdiyle(kaynak, vec!["2,5".into()]).expect("çalışmalı");
    assert_eq!(cikti, vec!["Kilo?", "5,0"]);
}

#[test]
fn tam_hedefe_ondalik_artis_reddedilir() {
    let kaynak = "sayaç 1 olsun\nsayacı 0,5 artır\n";
    let hata = kaynagi_calistir(kaynak).expect_err("T006 bekleniyor");
    assert_eq!(hata.kod, "T006");
    assert!(hata.oneri.as_deref().unwrap_or("").contains("0,0"), "öneri yol göstermeli");
}

#[test]
fn bosluk_virgul_rakam_belirsizligi() {
    let kaynak = "sayılar 3 ,14 listesi olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("S033 bekleniyor");
    assert_eq!(hata.kod, "S033");
}

#[test]
fn nokta_ondalik_yonlendirmesi() {
    let hata = kaynagi_calistir("pi 3.14 olsun\n").expect_err("S001 bekleniyor");
    assert_eq!(hata.kod, "S001");
    assert!(
        hata.oneri.as_deref().unwrap_or("").contains("3,14"),
        "öneri virgüllü yazımı göstermeli: {:?}",
        hata.oneri
    );
}

#[test]
fn keyfi_hassasiyetli_sabit_kayipsizdir() {
    let kaynak = "x 1,123456789012345678901234567890123456789 olsun\nx yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["1,123456789012345678901234567890123456789"]);
}

#[test]
fn buyuk_katsayi_ve_sonlu_bolum_tamdir() {
    let kaynak = "\
a 123456789012345678901234567890,12 olsun
tutar a ile 100,0 ın çarpımı olsun
tutar yaz
tutarın binlikli kuruşlusu yaz
c 1,0 ın 8 e bölümü olsun
c yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "12345678901234567890123456789012,0",
            "12.345.678.901.234.567.890.123.456.789.012,00",
            "0,125",
        ]
    );
}

#[test]
fn cok_kucuk_deger_karsilastirma_ve_jsonda_korunur() {
    let kaynak = "\
a 0,0000000000000000000000000000000001 olsun
oran a ile 2 nin çarpımı olsun
oran yaz
oran a dan büyükse
    \"büyük\" yaz
oranın json metni yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "0,0000000000000000000000000000000002",
            "büyük",
            "0.0000000000000000000000000000000002",
        ]
    );
}

#[test]
fn uzun_negatif_girdi_ondaliga_kayipsiz_cevrilir() {
    let kaynak = "\"Değer?\" diye sor\nx yanıtın ondalığı olsun\nx yaz\n";
    let cikti = dil::kaynagi_calistir_girdiyle(
        kaynak,
        vec!["-987654321098765432109876543210,12345678901234567890".into()],
    )
    .expect("çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "Değer?",
            "-987654321098765432109876543210,1234567890123456789",
        ]
    );
}

#[test]
fn buyuk_ondaligi_tam_sayiya_daraltma_tasma_verir() {
    let kaynak =
        "değer 999999999999999999999999999999,5 olsun\ndeğerin tam kısmı yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("C002 bekleniyor");
    assert_eq!(hata.kod, "C002");
}

#[test]
fn bicimleyici_ondaligi_bolmez() {
    let girdi = "fiyatlar   2,5,7,25 listesi olsun\n";
    let bicimli = dil::bicimleyici::bicimle(girdi).expect("biçimlenmeli");
    // 2,5 ve 7,25 tek token kalır; liste virgülünden sonra boşluk gelir.
    assert_eq!(bicimli, "fiyatlar 2,5, 7,25 listesi olsun\n");
    assert_eq!(dil::bicimleyici::bicimle(&bicimli).expect("idempotent"), bicimli);
}

#[test]
fn ondalik_yapi_alani() {
    let kaynak = "\
yapı Ürün
    ad Metin
    fiyat Ondalık

elma yeni Ürün olsun
elmanın fiyatı 12,75 olsun
elmanın fiyatı yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["12,75"]);
}

#[test]
fn secenek_ondalik_tasiyabilir() {
    let kaynak = "\
işlem yarıya kadar bul
    sayıları al
    her sayı için
        sayı 5 ten küçükse
            sayının 2 ye bölümü döndür
    yok döndür

sayılar 3,0, 9,0 listesi olsun
bulunan sayılar için yarıya kadar bul olsun
bulunan varsa
    bulunanın değeri yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["1,5"]);
}
