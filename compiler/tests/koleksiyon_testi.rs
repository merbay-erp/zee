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
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["Zeynep", "Eliz"]
    );
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
    let kaynak =
        "kutu boş liste olsun\nkutu 3, 5 listesi olsun\nkutu boş liste olsun\nkutunun adedi yaz\n";
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
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["3", "muz"]
    );
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

#[test]
fn aralik_geri_sayar() {
    // K-068: aralık iki yönde çalışır.
    let kaynak = "3 ten 1 e kadar her sayı için\n    sayıyı yaz\n\"ateşle\" yaz\n";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["3", "2", "1", "ateşle"]
    );
}

#[test]
fn cikis_kodu_tasinir() {
    // K-069: `programı N ile bitir` → calistir_io_kodla N döner.
    let program =
        dil::kaynagi_derle("\"a\" yaz\nprogramı 7 ile bitir\n\"b\" yaz\n").expect("derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    let kod = dil::yorumlayici::calistir_io_kodla(&program, &mut io).expect("çalışmalı");
    assert_eq!(kod, 7);
    assert_eq!(io.cikti, vec!["a"]);

    let program = dil::kaynagi_derle("\"a\" yaz\n").expect("derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    assert_eq!(
        dil::yorumlayici::calistir_io_kodla(&program, &mut io).unwrap(),
        0
    );

    let hata = kaynagi_calistir("programı 999 ile bitir\n").expect_err("C020");
    assert_eq!(hata.kod, "C020");
    let hata = kaynagi_calistir("programı \"üç\" ile bitir\n").expect_err("T034");
    assert_eq!(hata.kod, "T034");
}

#[test]
fn roket_geri_sayar() {
    let kaynak = std::fs::read_to_string("../projeler/roket.dil").expect("okunmalı");
    let cikti = kaynagi_calistir(&kaynak).expect("çalışmalı");
    assert_eq!(cikti.first().unwrap(), "Fırlatmaya hazırlanın!");
    assert_eq!(cikti.last().unwrap(), "🚀 ATEŞLE!");
    assert!(cikti.contains(&"5".to_string()) && cikti.contains(&"1".to_string()));
}

#[test]
fn yeni_dogrulamalar_icermeli_olmamali() {
    // K-071: içermeli + olmamalı (genel olumsuz) + çıplak boş atomu.
    let kaynak = "\
test \"zengin doğrulamalar\"
    cümle \"zeytin dalı\" olsun
    cümle \"zeytin\" içermeli
    sayılar 1, 2 listesi olsun
    sayılar boş olmamalı
    kare 16 olsun
    kare 17 ye eşit olmamalı
";
    let sonuclar = dil::kaynagi_dene(kaynak).expect("derlenmeli");
    assert!(sonuclar[0].hata.is_none());

    // Başarısızlık yolu D001 verir.
    let kaynak = "test \"düşer\"\n    cümle \"a\" olsun\n    cümle \"yok\" içermeli\n";
    let sonuclar = dil::kaynagi_dene(kaynak).expect("derlenmeli");
    assert!(sonuclar[0].hata.is_some());
}

#[test]
fn gezmede_oge_degisikligi_listeye_yansir() {
    // K-074: kaynak bir adsa, döngü değişkenine yazım GERİ YAZILIR.
    let kaynak = "\
yapı Kutu
    adet TamSayı

kutular boş liste olsun
bir yeni Kutu olsun
birin adedi 1 olsun
kutulara biri ekle
iki yeni Kutu olsun
ikinin adedi 2 olsun
kutulara ikiyi ekle

her kutu için
    eski kutunun adedi olsun
    yeni eski ile 10 un çarpımı olsun
    kutunun adedi yeni olsun

her kutu için
    kutunun adedi yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["10", "20"]
    );
}

#[test]
fn gezmede_yeniden_baglama_listeye_yansir() {
    // K-093: döngü adı değer-sonuç imlecidir; alan yazma gibi doğrudan
    // yeniden bağlama da turun aynı sırasına geri yazılır.
    let kaynak = "\
sayılar 1, 2, 3 listesi olsun
her sayı için
    sayı 7 olsun
sayıların json metni yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["[7,7,7]"]
    );
}

#[test]
fn gezmede_deger_kopyalari_alias_olusturmaz() {
    // K-093: yapı/liste ataması derin değer kopyasıdır. Gezme yalnız kaynak
    // listenin öğesini günceller; önceden alınmış kopya değişmez.
    let kaynak = "\
yapı Kutu
    adet TamSayı

asıllar boş liste olsun
bir yeni Kutu olsun
birin adedi 1 olsun
asıllara biri ekle
kopyalar asıllar olsun

asıllardaki her kutu için
    kutunun adedi 9 olsun
    kopyalara kutuyu ekle

asılların json metni yaz
kopyaların json metni yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["[{\"adet\":9}]", "[{\"adet\":1},{\"adet\":9}]"]
    );
}

#[test]
fn gezilen_koleksiyonun_bicimi_sabittir() {
    // K-093: sıra kayması/lost update yerine öğretici ve statik T053.
    let kaynaklar = [
        "sayılar 1, 2 listesi olsun\nher sayı için\n    sayılara 3 ekle\n",
        "sayılar 1, 2 listesi olsun\nher sayı için\n    sayılardan sayıyı sil\n",
        "sayılar 1, 2 listesi olsun\nher sayı için\n    sayılar 3, 4 listesi olsun\n",
        "değerler 1, 2 listesi olsun\ndeğerlerdeki her sayı için\n    değerlerdeki her öteki için\n        ötekiyi yaz\n",
        "defter boş sözlük olsun\ndefterin \"a\" değeri 1 olsun\ndefterdeki her ad için\n    defterin \"b\" değeri 2 olsun\n",
    ];
    for (sira, kaynak) in kaynaklar.into_iter().enumerate() {
        let hata = kaynagi_calistir(kaynak).expect_err("T053 vermeli");
        assert_eq!(hata.kod, "T053", "senaryo {}", sira + 1);
        assert!(hata.mesaj.contains("gezilirken"));
        assert!(hata
            .oneri
            .as_deref()
            .unwrap_or_default()
            .contains("ayrı bir listede"));
    }
}

#[test]
fn gezmede_deger_sonuc_ozelligi_uzunluktan_bagimsizdir() {
    // Küçük property korpusu: boş olmayan farklı liste uzunluklarında her
    // sıra tam bir kez ve aynı değer-sonuç kuralıyla değiştirilir.
    for adet in 1..=24 {
        let ogeler = (1..=adet)
            .map(|sayi| sayi.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let beklenen = std::iter::repeat_n("7", adet).collect::<Vec<_>>().join(",");
        let kaynak = format!(
            "sayılar {} listesi olsun\nher sayı için\n    sayı 7 olsun\nsayıların json metni yaz\n",
            ogeler
        );
        assert_eq!(
            kaynagi_calistir(&kaynak).expect("çalışmalı"),
            vec![format!("[{}]", beklenen)]
        );
    }
}

#[test]
fn binlikli_kuruslusu_turk_yazimi() {
    // K-075: binlik ayraç NOKTA, ondalık VİRGÜL — "1.234.567,89".
    let kaynak = "\
tutar 1234567,891 olsun
tutarın binlikli kuruşlusu yaz
kucuk 42,5 olsun
kucuğun binlikli kuruşlusu yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["1.234.567,89", "42,50"]
    );
}
