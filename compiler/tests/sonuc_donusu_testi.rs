//! Kullanıcı işlemlerinden Sonuç dönüşü (RFC-0008 §4.1):
//! `"..." hatasını döndür` + başarı dallarının otomatik sarmalanması.

use dil::kaynagi_calistir;

// Not: "payı al" yazılamazdı — "payı" hem pa+yı hem pay+ı okunur (y-tamponu
// belirsizliği, günlük K-011 bulgusu). Net gövdeli adlar seçildi.
const GUVENLI_BOL: &str = "\
işlem güvenle paylaştır
    bölüneni al
    böleni al

    bölen 0 a eşitse
        \"sıfıra bölünmez\" hatasını döndür
    sonucu bölünenin bölene bölümü olsun
    sonucu döndür
";

#[test]
fn basari_dali_sonuca_sarilir() {
    let kaynak = format!(
        "{}\nbölüm 10 ve 2 ile güvenle paylaştır olsun\nbölüm başarılıysa\n    bölümün değeri yaz\n",
        GUVENLI_BOL
    );
    let cikti = kaynagi_calistir(&kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["5"]);
}

#[test]
fn hata_dali_yakalanir() {
    let kaynak = format!(
        "{}\nbölüm 10 ve 0 ile güvenle paylaştır olsun\nbölüm başarısızsa\n    \"Olmadı: \" ile bölümün hatası yaz\n",
        GUVENLI_BOL
    );
    let cikti = kaynagi_calistir(&kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["Olmadı: sıfıra bölünmez"]);
}

#[test]
fn sonuc_degeri_turu_korunur() {
    // Sonuç<Ondalık>: değeri Ondalık olarak çıkar, aritmetiğe girer.
    let kaynak = "\
işlem yarıla
    sayıyı al
    sayı 0 a eşitse
        \"sıfır yarılanmaz\" hatasını döndür
    sonucu sayının 2 ye bölümü olsun
    sonucu döndür

yarım 5,0 için yarıla olsun
yarım başarılıysa
    değer yarımın değeri olsun
    iki_kati değer ile 2 nin çarpımı olsun
    iki_kati yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["5,0"]);
}

#[test]
fn yanlis_tarafa_erisim_calisma_hatasi() {
    let kaynak = format!(
        "{}\nbölüm 10 ve 2 ile güvenle paylaştır olsun\nbölümün hatası yaz\n",
        GUVENLI_BOL
    );
    let hata = kaynagi_calistir(&kaynak).expect_err("başarılıyken hatası → C009");
    assert_eq!(hata.kod, "C009");
}

#[test]
fn yalniz_hata_donduren_islem_reddedilir() {
    let kaynak = "işlem hep kız\n    \"olmaz\" hatasını döndür\n\nx hep kız olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("T018 bekleniyor");
    assert_eq!(hata.kod, "T018");
    assert!(hata.mesaj.contains("yalnız hata"), "{}", hata.mesaj);
}

#[test]
fn hata_mesaji_metin_olmali() {
    let kaynak = "işlem dene bakalım\n    5 hatasını döndür\n\nx dene bakalım olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("T032 bekleniyor");
    assert_eq!(hata.kod, "T032");
}

#[test]
fn islem_disinda_hata_dondurulemez() {
    let hata = kaynagi_calistir("\"olmaz\" hatasını döndür\n").expect_err("T020");
    assert_eq!(hata.kod, "T020");
}

#[test]
fn yok_ile_hata_karisimi_reddedilir() {
    let kaynak = "\
işlem karışık
    sayıyı al
    sayı 0 a eşitse
        \"olmaz\" hatasını döndür
    sayı 1 e eşitse
        yok döndür
    sayıyı döndür

x 5 için karışık olsun
";
    let hata = kaynagi_calistir(kaynak).expect_err("hata+yok karışımı reddedilmeli");
    assert_eq!(hata.kod, "T018");
}

#[test]
fn sonuc_gecisi_cift_sarilmaz() {
    // İşlem, dene'nin Sonuç'unu olduğu gibi geçirir: tek dönüş türü Sonuç →
    // sarmalama YOK; çağıran başarısızlığı doğrudan görür.
    let kaynak = "\
işlem dosyayı getir
    yolu al
    sonucu yol dosyasını okumayı dene olsun
    sonucu döndür

s \"yok.txt\" için dosyayı getir olsun
s başarısızsa
    \"hata geldi\" yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["hata geldi"]);
}

#[test]
fn hata_donusu_test_dogrulamasiyla() {
    // dil dene ile birlikte: Sonuç dönen işlem test edilebilir.
    let kaynak = format!(
        "{}\ntest \"sıfır bölme yakalanır\"\n    s 1 ve 0 ile güvenle paylaştır olsun\n    s başarısızsa\n        1 1 e eşit olmalı\n",
        GUVENLI_BOL
    );
    let sonuclar = dil::kaynagi_dene(&kaynak).expect("derlenmeli");
    assert!(sonuclar.iter().all(|s| s.hata.is_none()));
}
