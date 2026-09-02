//! K-056/057/058: Türkçe sıralama, tersi, arasındaki günler, liste üyeliği,
//! csv metni.

use dil::kaynagi_calistir;
use dil::yorumlayici::{calistir_io, GirdiCikti, ToplayanIo};

#[test]
fn turk_alfabesiyle_siralar() {
    let kaynak = "\
adlar \"şeker\", \"çilek\", \"ırmak\", \"iğne\", \"cam\", \"dam\" listesi olsun
sıralı adların sıralanmışı olsun
birleşik sıralının \" \" ile birleşmişi olsun
birleşik yaz
";
    // TANIMLI sıra: c < ç < d ... ı < i ... ş
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["cam çilek dam ırmak iğne şeker"]
    );
}

#[test]
fn sayilari_siralar_ve_ters_cevirir() {
    let kaynak = "\
sayılar 5, 2, 9, 1 listesi olsun
sıralı sayıların sıralanmışı olsun
ters sıralının tersi olsun
tersin ilki yaz
sıralının ilki yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["9", "1"]);
}

#[test]
fn ondalik_siralama_dogru() {
    let kaynak = "\
fiyatlar 2,50, 2,05, 10,0 listesi olsun
sıralı fiyatların sıralanmışı olsun
sıralının ilki yaz
sıralının sonu yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["2,05", "10,0"]
    );
}

#[test]
fn liste_uyeligi_ve_bekci() {
    let kaynak = "\
adlar \"zeynep\", \"eliz\" listesi olsun
adlarda \"eliz\" varsa
    \"bulundu\" yaz
adlarda \"ali\" yoksa
    \"ali yok\" yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["bulundu", "ali yok"]
    );

    let hata =
        kaynagi_calistir("sayılar 1, 2 listesi olsun\nsayılarda \"a\" varsa\n    \"x\" yaz\n")
            .expect_err("tür bekçisi");
    assert_eq!(hata.kod, "T021");
}

#[test]
fn arasindaki_gunler_isaretli() {
    let kaynak = "\
bugün bugünün tarihi olsun
hedef bugünün 45 gün sonrası olsun
ileri bugün ile hedef arasındaki günler olsun
ileri yaz
geri hedef ile bugün arasındaki günler olsun
geri yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["45", "-45"]
    );
}

#[test]
fn csv_metni_tablo_yazar() {
    // Sahte dünyada CSV oku → csv metni ile geri yaz (başlık sırası korunur).
    let kaynak = "\
tablo \"notlar.csv\" dosyasından okunan tablo olsun
tablonun csv metni yaz
";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosya_yaz("notlar.csv", "vize,final\n70,90\n60,85", false)
        .unwrap();
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti.join("\n"), "vize,final\n70,90\n60,85\n");
}
