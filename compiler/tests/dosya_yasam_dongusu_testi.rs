//! K-163/F030: binary medya saga'sının dar dosya yaşam döngüsü yüzeyi.

use dil::kaynagi_derle;
use dil::yorumlayici::{self, ToplayanIo};

#[test]
fn atomik_yayin_silme_ve_orphan_listesi_sonuc_tasir() {
    let kaynak = r#"
eylem yüklemeyi yayınla
    TamSayı sonucu döndürür
    ".zee/yuklemeler/a.parca" dosyasını "medya/a.bin" yoluna atomik taşımayı dene döndür

eylem medyayı sil
    TamSayı sonucu döndürür
    "medya/a.bin" dosyasını silmeyi dene döndür

yayın yüklemeyi yayınla olsun
yayın başarılıysa
    yayının değeri yaz
orphanlar ".zee/yuklemeler" yolundaki dosyaları listelemeyi dene olsun
orphanlar başarılıysa
    orphanların değeri yaz
silme medyayı sil olsun
silme başarılıysa
    silmenin değeri yaz
ikinci medyayı sil olsun
ikinci başarılıysa
    ikincinin değeri yaz
"#;
    let program = kaynagi_derle(kaynak).expect("dosya yaşam döngüsü derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar
        .insert(".zee/yuklemeler/a.parca".into(), "\0binary".into());
    io.dosyalar
        .insert(".zee/yuklemeler/orphan.parca".into(), "yarım".into());

    yorumlayici::calistir_io(&program, &mut io).expect("yaşam döngüsü çalışmalı");

    assert_eq!(io.cikti, ["1", ".zee/yuklemeler/orphan.parca", "1", "0"]);
    assert!(!io.dosyalar.contains_key("medya/a.bin"));
}

#[test]
fn var_olan_hedef_ezilmez_ve_kaynak_korunur() {
    let kaynak = r#"
eylem yüklemeyi yayınla
    TamSayı sonucu döndürür
    "gecici.bin" dosyasını "medya.bin" yoluna atomik taşımayı dene döndür

sonuç yüklemeyi yayınla olsun
sonuç başarısızsa
    hata sonucun hatası olsun
    hatanın kodu yaz
"#;
    let program = kaynagi_derle(kaynak).expect("hata sonucu derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert("gecici.bin".into(), "yeni".into());
    io.dosyalar.insert("medya.bin".into(), "eski".into());

    yorumlayici::calistir_io(&program, &mut io).expect("beklenen hata yönetilmeli");

    assert_eq!(io.cikti, ["C013"]);
    assert_eq!(io.dosyalar["gecici.bin"], "yeni");
    assert_eq!(io.dosyalar["medya.bin"], "eski");
}

/// K-331: `ilk N baytını onaltılık okumayı dene` Metin Sonucu taşır; 1..=64
/// dışı sayı ve bulunmayan dosya C013 başarısız Sonuç olur, worker düşmez.
#[test]
fn ilk_baytlar_kucuk_harf_onaltilik_ve_sinir_disi_c013() {
    let kaynak = r#"
işlem imza
    yolu Metin olarak al
    adedi TamSayı olarak al
    Metin döndürür
    sonuç yolu dosyasının ilk adedi baytını onaltılık okumayı dene olsun
    çıktı "" olsun
    sonuç başarılıysa
        çıktı sonucun değeri olsun
    değilse
        hata sonucun hatası olsun
        çıktı hatanın kodu olsun
    çıktı döndür

s1 "cv.pdf" ve 5 ile imza olsun
s1 yaz
s2 "cv.pdf" ve 64 ile imza olsun
s2 yaz
s3 "sahte.pdf" ve 2 ile imza olsun
s3 yaz
s4 "cv.pdf" ve 0 ile imza olsun
s4 yaz
s5 "cv.pdf" ve 65 ile imza olsun
s5 yaz
s6 "cv.pdf" ve -1 ile imza olsun
s6 yaz
s7 "yok.pdf" ve 4 ile imza olsun
s7 yaz
"#;
    let program = kaynagi_derle(kaynak).expect("imza okuma derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert("cv.pdf".into(), "%PDF-1.4".into());
    io.dosyalar.insert("sahte.pdf".into(), "MZ\u{7f}".into());

    yorumlayici::calistir_io(&program, &mut io).expect("hatalar Sonuç olarak yönetilmeli");

    assert_eq!(
        io.cikti,
        [
            "255044462d",
            "255044462d312e34",
            "4d5a",
            "C013",
            "C013",
            "C013",
            "C013"
        ]
    );
}

#[test]
fn ilk_baytlar_tur_hatasi_derlemede_reddedilir() {
    for kaynak in [
        "s \"a.pdf\" dosyasının ilk \"8\" baytını onaltılık okumayı dene olsun\n",
        "s 3 dosyasının ilk 8 baytını onaltılık okumayı dene olsun\n",
    ] {
        let tani = kaynagi_derle(kaynak).expect_err("tür hatası beklenir");
        assert_eq!(tani.kod, "T034", "{kaynak}");
        assert!(
            tani.mesaj.contains("bayt sayısı TamSayı"),
            "{kaynak}: {tani:?}"
        );
    }
}
