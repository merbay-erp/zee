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
