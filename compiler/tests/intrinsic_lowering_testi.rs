//! ADR-011 intrinsic/yetkinlik lowering sınırı.
//!
//! Kullanıcı yüzeyi değişmeden kalırken alan işlemlerinin çekirdek AST'de
//! ayrı varyantlar değil, merkezi kayda bağlı tek bir düğüm olmasını korur.

use dil::agac::{Cumle, Ifade};
use dil::intrinsic::{
    self, IntrinsicEtkisi, Yetkinlik, CSRF_BELIRTECI, HTTP_GETIR, PAROLA_DOGRULA,
    POSTGRESQL_DEGISTIR, POSTGRESQL_OKU, SENSOR_ACIK_MI,
};
use dil::kaynagi_derle;
use std::collections::HashSet;

fn atama_degeri(cumle: &Cumle) -> &Ifade {
    let Cumle::Olsun { deger, .. } = cumle else {
        panic!("atama bekleniyordu")
    };
    deger
}

#[test]
fn intrinsic_kaydi_kimlik_yetkinlik_ve_etkiyi_tekillestirir() {
    let kimlikler = intrinsic::TANIMLAR
        .iter()
        .map(|tanim| tanim.kimlik)
        .collect::<HashSet<_>>();
    assert_eq!(kimlikler.len(), intrinsic::TANIMLAR.len());
    assert_eq!(kimlikler.len(), 6);

    let http = intrinsic::tanim(HTTP_GETIR).expect("HTTP kaydı");
    assert_eq!(http.yetkinlik, Yetkinlik::Ag);
    assert_eq!(http.etki, IntrinsicEtkisi::DisOkuma);

    let csrf = intrinsic::tanim(CSRF_BELIRTECI).expect("CSRF kaydı");
    assert_eq!(csrf.yetkinlik, Yetkinlik::WebOturumu);
    assert_eq!(csrf.etki, IntrinsicEtkisi::WebAdaptoru);

    let postgresql = intrinsic::tanim(POSTGRESQL_DEGISTIR).expect("PostgreSQL kaydı");
    assert_eq!(postgresql.yetkinlik, Yetkinlik::Veritabani);
    assert_eq!(postgresql.etki, IntrinsicEtkisi::DisYazma);
}

#[test]
fn postgresql_yuzeyi_sorgu_ve_parametreleri_ayri_intrinsice_indirir() {
    let kaynak = r#"
parametreler "x' OR true --" listesi olsun
satırlar "SELECT ad::text AS ad FROM sayfalar WHERE slug = $1" sorgusunu parametreler ile okumayı dene olsun
etkilenen "UPDATE sayfalar SET ad = $1 WHERE slug = $2" sorgusunu parametreler ile değiştirmeyi dene olsun
"#;
    let program = kaynagi_derle(kaynak).expect("PostgreSQL ifadeleri derlenmeli");
    for (sira, kimlik) in [POSTGRESQL_OKU, POSTGRESQL_DEGISTIR]
        .into_iter()
        .enumerate()
    {
        let Ifade::Intrinsic {
            kimlik: bulunan,
            argumanlar,
        } = atama_degeri(&program.cumleler[sira + 1]).turu()
        else {
            panic!("PostgreSQL intrinsic bekleniyordu")
        };
        assert_eq!(bulunan, kimlik);
        assert_eq!(argumanlar.len(), 2);
    }
}

#[test]
fn http_yuzeyi_generic_intrinsice_indirilir() {
    let program = kaynagi_derle("cevap \"https://ornek.dev/veri\" adresinden gelen yanıt olsun\n")
        .expect("derlenmeli");
    let Ifade::Intrinsic { kimlik, argumanlar } = atama_degeri(&program.cumleler[0]).turu() else {
        panic!("intrinsic bekleniyordu")
    };
    assert_eq!(kimlik, HTTP_GETIR);
    assert!(
        matches!(argumanlar.as_slice(), [ifade] if matches!(ifade.turu(), Ifade::MetinSabiti(_)))
    );
}

#[test]
fn csrf_yuzeyi_argumansiz_generic_intrinsice_indirilir() {
    let program = kaynagi_derle("belirteç csrf belirteci olsun\n").expect("derlenmeli");
    let Ifade::Intrinsic { kimlik, argumanlar } = atama_degeri(&program.cumleler[0]).turu() else {
        panic!("intrinsic bekleniyordu")
    };
    assert_eq!(kimlik, CSRF_BELIRTECI);
    assert!(argumanlar.is_empty());
}

#[test]
fn parola_yuzeyi_iki_argumanli_generic_intrinsice_indirilir() {
    let kaynak = r#"
verilen "gizli" olsun
özet "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$YWJj" olsun
verilen özet ile doğrulanıyorsa
    "doğru" yaz
"#;
    let program = kaynagi_derle(kaynak).expect("derlenmeli");
    let Cumle::Ise { kollar, .. } = &program.cumleler[2] else {
        panic!("koşul bekleniyordu")
    };
    let Ifade::Intrinsic { kimlik, argumanlar } = kollar[0].kosul.turu() else {
        panic!("intrinsic bekleniyordu")
    };
    assert_eq!(kimlik, PAROLA_DOGRULA);
    assert_eq!(argumanlar.len(), 2);
}

#[test]
fn sensor_yuzeyi_tek_intrinsic_ve_genel_olumsuzlama_kullanir() {
    let kaynak = r#"
kapı açıksa
    "açık" yaz
kapı kapalıysa
    "kapalı" yaz
"#;
    let program = kaynagi_derle(kaynak).expect("derlenmeli");
    let Cumle::Ise { kollar: acik, .. } = &program.cumleler[0] else {
        panic!("ilk koşul bekleniyordu")
    };
    let Ifade::Intrinsic { kimlik, argumanlar } = acik[0].kosul.turu() else {
        panic!("sensor intrinsic bekleniyordu")
    };
    assert_eq!(kimlik, SENSOR_ACIK_MI);
    assert!(
        matches!(argumanlar.as_slice(), [ifade] if matches!(ifade.turu(), Ifade::MetinSabiti(ad) if ad == "kapı"))
    );

    let Cumle::Ise { kollar: kapali, .. } = &program.cumleler[1] else {
        panic!("ikinci koşul bekleniyordu")
    };
    let Ifade::Degil(ic) = kapali[0].kosul.turu() else {
        panic!("genel olumsuzlama bekleniyordu")
    };
    assert!(matches!(ic.turu(), Ifade::Intrinsic { kimlik, .. } if kimlik == SENSOR_ACIK_MI));
}

#[test]
fn merkezi_http_imzasi_yanlis_arguman_turunu_reddeder() {
    let hata =
        kaynagi_derle("cevap 42 adresinden gelen yanıt olsun\n").expect_err("adres Metin olmalı");
    assert_eq!(hata.kod, "T034");
    assert!(hata.mesaj.contains("Adres Metin olmalı"));
}

#[test]
fn merkezi_parola_imzasi_yanlis_arguman_turunu_reddeder() {
    let kaynak = r#"
verilen 42 olsun
özet "PHC" olsun
verilen özet ile doğrulanıyorsa
    "ulaşılmaz" yaz
"#;
    let hata = kaynagi_derle(kaynak).expect_err("parola Metin olmalı");
    assert_eq!(hata.kod, "T034");
    assert!(hata.mesaj.contains("Parola ve Argon2id özeti Metin olmalı"));
}
