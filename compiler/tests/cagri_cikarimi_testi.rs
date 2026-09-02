//! B-007: yerel çağrı-güdümlü imza çıkarımı kaynak sırasından bağımsızdır.

use dil::{kaynagi_calistir, kaynagi_derle};

const KIMLIK_ISLEMI: &str = "\
işlem değeri geçir
    değeri al
    değeri döndür
";

#[test]
fn butun_cagri_kisitlari_ilk_sonuc_turlenmeden_once_birlestirilir() {
    let dar_once = format!(
        "{}\ndar 1 için değeri geçir olsun\ngeniş 1,5 için değeri geçir olsun\ndarı 0,5 artır\ndar yaz\n",
        KIMLIK_ISLEMI
    );
    let genis_once = format!(
        "{}\ngeniş 1,5 için değeri geçir olsun\ndar 1 için değeri geçir olsun\ndarı 0,5 artır\ndar yaz\n",
        KIMLIK_ISLEMI
    );

    assert_eq!(kaynagi_calistir(&dar_once).expect("dar-geniş"), vec!["1,5"]);
    assert_eq!(
        kaynagi_calistir(&genis_once).expect("geniş-dar"),
        vec!["1,5"]
    );
}

#[test]
fn nihai_tur_ic_ice_cagri_grafigine_de_sira_bagimsiz_yayilir() {
    let islemler = format!(
        "{}\n\
işlem değeri sar
    değeri al
    sonuç değer için değeri geçir olsun
    sonucu döndür
",
        KIMLIK_ISLEMI
    );
    let dar_once = format!(
        "{}\ndar 1 için değeri sar olsun\ngeniş 1,5 için değeri geçir olsun\ndarı 0,5 artır\ndar yaz\n",
        islemler
    );
    let genis_once = format!(
        "{}\ngeniş 1,5 için değeri geçir olsun\ndar 1 için değeri sar olsun\ndarı 0,5 artır\ndar yaz\n",
        islemler
    );

    assert_eq!(
        kaynagi_calistir(&dar_once).expect("dar-geniş grafik"),
        vec!["1,5"]
    );
    assert_eq!(
        kaynagi_calistir(&genis_once).expect("geniş-dar grafik"),
        vec!["1,5"]
    );
}

#[test]
fn birlesmeyen_cagri_turleri_iki_sirada_da_t017dir() {
    for cagri in [
        "a 1 için değeri geçir olsun\nb \"iki\" için değeri geçir olsun\n",
        "b \"iki\" için değeri geçir olsun\na 1 için değeri geçir olsun\n",
    ] {
        let kaynak = format!("{}\n{}", KIMLIK_ISLEMI, cagri);
        assert_eq!(
            kaynagi_derle(&kaynak).expect_err("T017 bekleniyor").kod,
            "T017"
        );
    }
}

#[test]
fn liste_kapsayicisi_da_cagri_sirasindan_bagimsiz_genisler() {
    let islem = "\
işlem listeyi geçir
    listeyi al
    listeyi döndür
";
    let dar_once = format!(
        "{}\ndarlar 1, 2 listesi için listeyi geçir olsun\ngenişler 1,5, 2,5 listesi için listeyi geçir olsun\nilk darların ilki olsun\nilki 0,5 artır\nilk yaz\n",
        islem
    );
    let genis_once = format!(
        "{}\ngenişler 1,5, 2,5 listesi için listeyi geçir olsun\ndarlar 1, 2 listesi için listeyi geçir olsun\nilk darların ilki olsun\nilki 0,5 artır\nilk yaz\n",
        islem
    );

    assert_eq!(
        kaynagi_calistir(&dar_once).expect("dar-geniş liste"),
        vec!["1,5"]
    );
    assert_eq!(
        kaynagi_calistir(&genis_once).expect("geniş-dar liste"),
        vec!["1,5"]
    );
}

#[test]
fn her_parametre_kendi_kisit_kumesini_siradan_bagimsiz_birlestirir() {
    let islem = "\
işlem ilk değeri geçir
    birinciyi al
    ikinciyi al
    birinciyi döndür
";
    let dar_once = format!(
        "{}\ndar 1 ve 2,5 ile ilk değeri geçir olsun\ngeniş 1,5 ve 2 ile ilk değeri geçir olsun\ndarı 0,5 artır\ndar yaz\n",
        islem
    );
    let genis_once = format!(
        "{}\ngeniş 1,5 ve 2 ile ilk değeri geçir olsun\ndar 1 ve 2,5 ile ilk değeri geçir olsun\ndarı 0,5 artır\ndar yaz\n",
        islem
    );

    assert_eq!(
        kaynagi_calistir(&dar_once).expect("dar-geniş çift"),
        vec!["1,5"]
    );
    assert_eq!(
        kaynagi_calistir(&genis_once).expect("geniş-dar çift"),
        vec!["1,5"]
    );
}

#[test]
fn ic_bloktaki_sonraki_kisit_onceki_dar_sonuc_hatasina_takilmaz() {
    let dar_once = format!(
        "{}\ndoğru ise\n    dar 1 için değeri geçir olsun\n    darı 0,5 artır\n    geniş 1,5 için değeri geçir olsun\n    dar yaz\n",
        KIMLIK_ISLEMI
    );
    let genis_once = format!(
        "{}\ndoğru ise\n    geniş 1,5 için değeri geçir olsun\n    dar 1 için değeri geçir olsun\n    darı 0,5 artır\n    dar yaz\n",
        KIMLIK_ISLEMI
    );

    assert_eq!(
        kaynagi_calistir(&dar_once).expect("iç blok dar-geniş"),
        vec!["1,5"]
    );
    assert_eq!(
        kaynagi_calistir(&genis_once).expect("iç blok geniş-dar"),
        vec!["1,5"]
    );
}
