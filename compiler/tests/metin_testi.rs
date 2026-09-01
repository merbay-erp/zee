//! Metin dalgası (K-053) + JSON serileştirme (K-054): parçala, birleştir,
//! değiştir, kırp, harfler, başlıyor/bitiyor — ve json metni.

use dil::kaynagi_calistir;

#[test]
fn parcala_ve_birlestir() {
    let kaynak = "\
satır \"elma;armut;kiraz\" olsun
parçalar satırın \";\" ile parçaları olsun
parçaların adedi yaz
parçaların ilki yaz
birleşik parçaların \" ve \" ile birleşmişi olsun
birleşik yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["3", "elma", "elma ve armut ve kiraz"]
    );
}

#[test]
fn bos_ayracla_parcalama_harflere_boler() {
    let kaynak = "ad \"zee\" olsun\nparçalar adın \"\" ile parçaları olsun\nparçaların adedi yaz\n";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["3"]);
}

#[test]
fn degistir_ve_kirp() {
    let kaynak = "\
cümle \"  kedi evde  \" olsun
temiz cümlenin kırpılmışı olsun
yeni temizin \"kedi\" yerine \"köpek\" değişmişi olsun
yeni yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["köpek evde"]);
}

#[test]
fn bos_eski_degistirilemez() {
    let kaynak = "veri \"a\" olsun\nyeni verinin \"\" yerine \"b\" değişmişi olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("C004");
    assert_eq!(hata.kod, "C004");
}

#[test]
fn harfler_turkce_dogru() {
    let kaynak = "\
ad \"çiğdem\" olsun
harfler adın harfleri olsun
harflerin adedi yaz
harflerin ilki yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["6", "ç"]);
}

#[test]
fn baslar_biter_kosullari() {
    let kaynak = "\
dosya \"rapor.dil\" olsun
dosya \"rapor\" ile başlıyorsa
    \"rapor dosyası\" yaz
dosya \".dil\" ile bitiyorsa
    \"zee kaynağı\" yaz
dosya \".txt\" ile bitiyorsa
    \"olmaz\" yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["rapor dosyası", "zee kaynağı"]
    );
}

#[test]
fn json_metni_serilestirir() {
    let kaynak = "\
kişi boş sözlük olsun
kişinin \"ad\" değeri \"Zeynep \\\"Eliz\\\"\" olsun
kişinin \"şehir\" değeri \"İzmir\" olsun
kişinin json metni yaz

sayılar 3, 7 listesi olsun
sayıların json metni yaz

oran 3,14 olsun
oranın json metni yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec![
            "{\"ad\":\"Zeynep \\\"Eliz\\\"\",\"şehir\":\"İzmir\"}",
            "[3,7]",
            "3.14"
        ]
    );
}

#[test]
fn metin_dalgasinin_tur_bekcileri() {
    let hata = kaynagi_calistir("veri 5 olsun\nyeni verinin \",\" ile parçaları olsun\n").expect_err("T022");
    assert_eq!(hata.kod, "T022");
    let hata = kaynagi_calistir("sayılar 1, 2 listesi olsun\ny sayıların \",\" ile birleşmişi olsun\n")
        .expect_err("Metin listesi değil");
    assert_eq!(hata.kod, "T022");
}

#[test]
fn json_okuma_sayi_bool_metin_gelir() {
    // K-063: sayı/true/false/null reddedilmez — Metin gelir (nokta → virgül).
    let program = dil::kaynagi_derle(
        "kişi \"k.json\" dosyasından okunan veri olsun\nkişinin \"yaş\" değeri yaz\nkişinin \"boy\" değeri yaz\nkişinin \"üye\" değeri yaz\n",
    )
    .expect("derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    use dil::yorumlayici::GirdiCikti;
    io.dosya_yaz("k.json", "{\"yaş\": 10, \"boy\": 1.35, \"üye\": true}", false).unwrap();
    dil::yorumlayici::calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti, vec!["10", "1,35", "doğru"]);
}

#[test]
fn kuruslusu_para_bicimi() {
    // K-065: daima iki hane; yarımlar sıfırdan uzağa.
    let kaynak = "\
tutar 1824,5 olsun
tutarın kuruşlusu yaz
oran 3,456 olsun
oranın kuruşlusu yaz
tam 5 olsun
tamın kuruşlusu yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("çalışmalı"),
        vec!["1824,50", "3,46", "5,00"]
    );
}
