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
