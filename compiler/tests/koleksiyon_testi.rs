//! K-045: boş koleksiyonun öğe/değer türü ilk eklemeyle somutlaşır;
//! K-044: Mantıksal ad tek başına koşuldur ("asal ise" / "asal değilse").

use dil::kaynagi_calistir;

#[test]
fn metin_listesi_ilk_eklemeden_cikarilir() {
    let kaynak = "\
adlar boş liste olsun
adlara \"Zeynep\" ekle
adlara \"Eliz\" ekle
her ad için
    adı yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["Zeynep", "Eliz"]);
}

#[test]
fn metin_sozlugu_ilk_atamadan_cikarilir() {
    let kaynak = "\
defter boş sözlük olsun
defterin \"merhaba\" değeri \"zumzum\" olsun
defterin \"merhaba\" değeri yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["zumzum"]);
}

#[test]
fn cikarimdan_sonra_karisik_tur_reddedilir() {
    let kaynak = "adlar boş liste olsun\nadlara \"a\" ekle\nadlara 3 ekle\n";
    let hata = kaynagi_calistir(kaynak).expect_err("T011");
    assert_eq!(hata.kod, "T011");

    let kaynak = "defter boş sözlük olsun\ndefterin \"x\" değeri \"a\" olsun\ndefterin \"y\" değeri 3 olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("T021");
    assert_eq!(hata.kod, "T021");
}

#[test]
fn belirsiz_koleksiyon_okumalari_derleme_hatasi() {
    let hata = kaynagi_calistir("x boş liste olsun\nher öğe için\n    öğeyi yaz\n")
        .expect_err("belirsiz gezme");
    assert_eq!(hata.kod, "A003"); // örtük çoğul "öğeler" yok — önce o yakalanır

    let hata = kaynagi_calistir("öğeler boş liste olsun\nher öğe için\n    öğeyi yaz\n")
        .expect_err("belirsiz gezme T013");
    assert_eq!(hata.kod, "T013");

    let hata = kaynagi_calistir("defter boş sözlük olsun\ndefterin \"x\" değeri yaz\n")
        .expect_err("belirsiz okuma T021");
    assert_eq!(hata.kod, "T021");
}

#[test]
fn bos_ve_somut_liste_uzlasir() {
    // Boşla başla → somutla yeniden ata (ve tersi): T002 DEĞİL.
    let kaynak = "kutu boş liste olsun\nkutu 3, 5 listesi olsun\nkutu boş liste olsun\nkutunun adedi yaz\n";
    assert_eq!(kaynagi_calistir(kaynak).expect("uzlaşmalı"), vec!["0"]);
}

#[test]
fn mantiksal_ad_tek_basina_kosuldur() {
    let kaynak = "\
hazır doğru olsun
hazır ise
    \"başla\" yaz
hazır değilse
    \"bekle\" yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["başla"]);
}

#[test]
fn mantiksal_olmayan_ad_kosul_olamaz() {
    let hata = kaynagi_calistir("x 5 olsun\nx ise\n    \"a\" yaz\n").expect_err("T005");
    assert_eq!(hata.kod, "T005");
}

#[test]
fn kalan_kalibi_okul_kurali() {
    // K-046: "X in Y ye bölümünden kalanı" — kalan daima negatif değildir.
    let kaynak = "\
k 17 nin 5 e bölümünden kalanı olsun
k yaz
eksi 0 ile 7 nin farkı olsun
eksinin 3 e bölümünden kalanı yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["2", "2"]);

    let hata = kaynagi_calistir("x 10 un 0 a bölümünden kalanı olsun\n").expect_err("C003");
    assert_eq!(hata.kod, "C003");

    let hata = kaynagi_calistir("x 2,5 un 2 ye bölümünden kalanı olsun\n").expect_err("T008");
    assert_eq!(hata.kod, "T008");
}

#[test]
fn ikizlesme_geri_cevrimi() {
    // K-049: üs→üssü, af→affı; sertleşmeyle birleşik: ret→reddi.
    let kaynak = "\
üs 2 olsun
üssü yaz
ret 9 olsun
reddi yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["2", "9"]);
}

#[test]
fn silme_liste_ve_sozluk() {
    // K-059: ilk eşleşen öğe/anahtar silinir; yoksa sessizce hiçbir şey olmaz.
    let kaynak = "\
sayılar 3, 5, 5, 7 listesi olsun
sayılardan 5 i sil
sayılardan 99 u sil
sayıların adedi yaz
defter boş sözlük olsun
defterin \"elma\" değeri \"kırmızı\" olsun
defterin \"muz\" değeri \"sarı\" olsun
defterden \"elma\" yı sil
defterdeki her ad için
    adı yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["3", "muz"]);
}

#[test]
fn silme_tur_bekcileri() {
    let hata = kaynagi_calistir("sayılar 1, 2 listesi olsun\nsayılardan \"a\" yı sil\n")
        .expect_err("T011");
    assert_eq!(hata.kod, "T011");
    let hata = kaynagi_calistir("veri 5 olsun\nveriden 1 i sil\n").expect_err("T012");
    assert_eq!(hata.kod, "T012");
}

#[test]
fn yapi_listesi_kayit_tablosu() {
    // K-060: Liste<Yapı> — kayıt tabloları. Ekleme türü somutlar, gezmede
    // alan erişimi çalışır, JSON nesne listesi üretir.
    let kaynak = "\
yapı Kitap
    ad Metin
    fiyat Ondalık

kitaplar boş liste olsun

birinci yeni Kitap olsun
birincinin adı \"Masallar\" olsun
birincinin fiyatı 45,50 olsun
kitaplara birinciyi ekle

ikinci yeni Kitap olsun
ikincinin adı \"Şiirler\" olsun
kitaplara ikinciyi ekle

toplam 0,0 olsun
her kitap için
    şimdiki kitabın fiyatı olsun
    toplamı şimdikiyle artır
toplam yaz
kitapların json metni yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti[0], "45,5");
    assert_eq!(
        cikti[1],
        "[{\"ad\":\"Masallar\",\"fiyat\":45.5},{\"ad\":\"Şiirler\",\"fiyat\":0.0}]"
    );
}

#[test]
fn yapi_listesine_yanlis_yapi_giremez() {
    let kaynak = "\
yapı Kedi
    ad Metin

yapı Köpek
    ad Metin

kediler boş liste olsun
tekir yeni Kedi olsun
kedilere tekiri ekle
karabaş yeni Köpek olsun
kedilere karabaşı ekle
";
    let hata = kaynagi_calistir(kaynak).expect_err("T011");
    assert_eq!(hata.kod, "T011");
}

#[test]
fn ozellik_kelimesiyle_cakisan_alan_okunur() {
    // K-064: "ürünün adedi" — alan adı özellik kelimesiyle çakışsa da alan kazanır.
    let kaynak = "\
yapı Ürün
    adet TamSayı

kalem yeni Ürün olsun
kalemin adedi 7 olsun
kalemin adedi yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["7"]);
}

#[test]
fn islemden_liste_donusu() {
    let kaynak = "\
işlem çiftleri topla
    sayıları al

    çiftler boş liste olsun
    her sayı için
        sayı çiftse
            çiftlere sayıyı ekle
    çiftleri döndür

hepsi 1, 2, 3, 4, 5, 6 listesi olsun
seçilen hepsi için çiftleri topla olsun
seçilenin adedi yaz
seçilenin sonu yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["3", "6"]);
}

#[test]
fn ondalik_degerli_sozluk() {
    // K-067: para sözlükleri.
    let kaynak = "\
fiyatlar boş sözlük olsun
fiyatların \"çay\" değeri 45,50 olsun
fiyatların \"un\" değeri 28,75 olsun

toplam 0,0 olsun
fiyatlardaki her ürün için
    toplamı fiyatların ürün değeriyle artır
toplamın kuruşlusu yaz
fiyatların json metni yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["74,25", "{\"çay\":45.5,\"un\":28.75}"]
    );
}
