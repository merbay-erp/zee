//! ADR-011 intrinsic/yetkinlik lowering sınırı.
//!
//! Kullanıcı yüzeyi değişmeden kalırken alan işlemlerinin çekirdek AST'de
//! ayrı varyantlar değil, merkezi kayda bağlı tek bir düğüm olmasını korur.

use dil::agac::{Cumle, Ifade};
use dil::intrinsic::{
    self, IntrinsicEtkisi, Yetkinlik, CSRF_BELIRTECI, DOSYALARI_LISTELE, DOSYA_ATOMIK_TASI,
    HTTP_GETIR, PAROLA_DOGRULA, POSTGRESQL_DEGISTIR, POSTGRESQL_OKU, SENSOR_ACIK_MI,
    WEB_OTURUMLARINI_LISTELE, WEB_OTURUMUNU_IPTAL_ET,
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
    assert_eq!(kimlikler.len(), 55);
    for kimlik in [
        intrinsic::BAGLANTI_SATIR_OKU,
        intrinsic::BAGLANTI_SATIR_YAZ,
        intrinsic::BAGLANTI_TLS_BASLAT,
        intrinsic::BAGLANTI_NOKTA_BLOGU,
        intrinsic::BAGLANTI_BAYT_BLOGU,
        intrinsic::BAGLANTI_BAYT_OKU,
        intrinsic::BAGLANTI_PARCA_YAZ,
        intrinsic::BAGLANTI_DOSYA_AKTAR,
        intrinsic::BAGLANTI_SATIR_OKU_SURELI,
        intrinsic::BAGLANTI_TLS_ISTEMCI,
        intrinsic::BAGLANTI_BASE64_AKTAR,
        intrinsic::BAGLANTI_BASE64_ARALIK_AKTAR,
    ] {
        let kayit = intrinsic::tanim(kimlik).expect("K-381 bağlantı kaydı");
        assert_eq!(kayit.yetkinlik, Yetkinlik::AgDinleyici);
        // K-397: bağlantı işlemleri `ağ-dinleyici` ya da `ağ` ile derlenir.
        assert!(intrinsic::baglanti_islemi_mi(kimlik));
        // Ağ beklemesi transaction tutamaz: eylem içinde T048.
        assert_eq!(kayit.etki, IntrinsicEtkisi::GeriAlinamaz);
    }
    // K-397: giden bağlanma yalnız `ağ` ister; bağlantı işlemi sayılmaz.
    let giden = intrinsic::tanim(intrinsic::GIDEN_BAGLAN).expect("K-397 kaydı");
    assert_eq!(giden.yetkinlik, Yetkinlik::Ag);
    assert_eq!(giden.etki, IntrinsicEtkisi::GeriAlinamaz);
    assert!(!intrinsic::baglanti_islemi_mi(intrinsic::GIDEN_BAGLAN));
    let dosya_yaniti = intrinsic::tanim(intrinsic::WEB_DOSYA_YANITI).expect("K-378 kaydı");
    assert_eq!(dosya_yaniti.yetkinlik, Yetkinlik::DosyaOkuma);
    assert_eq!(dosya_yaniti.etki, IntrinsicEtkisi::WebAdaptoru);
    assert_eq!(dosya_yaniti.arguman_turleri.len(), 3);
    let imza = intrinsic::tanim(intrinsic::DOSYA_ILK_BAYTLAR).expect("K-333 imza kaydı");
    assert_eq!(imza.yetkinlik, Yetkinlik::DosyaOkuma);
    assert_eq!(imza.etki, IntrinsicEtkisi::DisOkuma);
    assert_eq!(intrinsic::DOSYA_ILK_BAYT_SINIRI, 64);
    // K-389: aralıklı okuma dosya-okuma/DisOkuma, çözerek yazma
    // dosya-yazma/DisYazma (eylem savepoint'i ile, T048 değil).
    for kimlik in [
        intrinsic::DOSYA_ARALIK_METIN,
        intrinsic::DOSYA_ARALIK_BASE64,
    ] {
        let kayit = intrinsic::tanim(kimlik).expect("K-389 okuma kaydı");
        assert_eq!(kayit.yetkinlik, Yetkinlik::DosyaOkuma);
        assert_eq!(kayit.etki, IntrinsicEtkisi::DisOkuma);
    }
    for kimlik in [
        intrinsic::DOSYA_BASE64_YAZ,
        intrinsic::DOSYA_BASE64_EKLE,
        intrinsic::DOSYA_QP_YAZ,
        intrinsic::DOSYA_QP_EKLE,
    ] {
        let kayit = intrinsic::tanim(kimlik).expect("K-389 yazma kaydı");
        assert_eq!(kayit.yetkinlik, Yetkinlik::DosyaYazma);
        assert_eq!(kayit.etki, IntrinsicEtkisi::DisYazma);
    }

    for (kimlik, etki) in [
        (intrinsic::HTTP_ISTEK, IntrinsicEtkisi::GeriAlinamaz),
        (intrinsic::EPOSTA_MIME_GONDER, IntrinsicEtkisi::GeriAlinamaz),
        (intrinsic::HTTP_GIZLI_ORTAM, IntrinsicEtkisi::DisOkuma),
        (intrinsic::HTTP_JSON, IntrinsicEtkisi::Saf),
    ] {
        let kayit = intrinsic::tanim(kimlik).expect("K-220 HTTP kaydı");
        assert_eq!(kayit.yetkinlik, Yetkinlik::Ag);
        assert_eq!(kayit.etki, etki);
    }

    let http = intrinsic::tanim(HTTP_GETIR).expect("HTTP kaydı");
    assert_eq!(http.yetkinlik, Yetkinlik::Ag);
    assert_eq!(http.etki, IntrinsicEtkisi::DisOkuma);

    let ip = intrinsic::tanim("web.istemci_adresi").expect("istemci adresi kaydı");
    assert_eq!(ip.yetkinlik, Yetkinlik::AgSunucusu);
    assert_eq!(ip.etki, IntrinsicEtkisi::WebAdaptoru);
    assert!(ip.arguman_turleri.is_empty());
    let birincil = intrinsic::tanim("web.birincil_surec").expect("K-337 birincil süreç kaydı");
    assert_eq!(birincil.yetkinlik, Yetkinlik::AgSunucusu);
    assert_eq!(birincil.etki, IntrinsicEtkisi::WebAdaptoru);
    assert!(birincil.arguman_turleri.is_empty());
    let csrf = intrinsic::tanim(CSRF_BELIRTECI).expect("CSRF kaydı");
    assert_eq!(csrf.yetkinlik, Yetkinlik::WebOturumu);
    assert_eq!(csrf.etki, IntrinsicEtkisi::WebAdaptoru);
    for kimlik in [WEB_OTURUMLARINI_LISTELE, WEB_OTURUMUNU_IPTAL_ET] {
        let oturum = intrinsic::tanim(kimlik).expect("web oturum yönetimi kaydı");
        assert_eq!(oturum.yetkinlik, Yetkinlik::WebOturumu);
        assert_eq!(oturum.etki, IntrinsicEtkisi::WebAdaptoru);
    }

    let postgresql = intrinsic::tanim(POSTGRESQL_DEGISTIR).expect("PostgreSQL kaydı");
    assert_eq!(postgresql.yetkinlik, Yetkinlik::Veritabani);
    assert_eq!(postgresql.etki, IntrinsicEtkisi::DisYazma);

    let yayin = intrinsic::tanim(DOSYA_ATOMIK_TASI).expect("atomik yayın kaydı");
    assert_eq!(yayin.yetkinlik, Yetkinlik::DosyaYazma);
    assert_eq!(yayin.etki, IntrinsicEtkisi::DisYazma);
    let tarama = intrinsic::tanim(DOSYALARI_LISTELE).expect("dizin tarama kaydı");
    assert_eq!(tarama.yetkinlik, Yetkinlik::DosyaOkuma);
    assert_eq!(tarama.etki, IntrinsicEtkisi::DisOkuma);
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
    assert!(hata.mesaj.contains("Parola ve parola özeti Metin olmalı"));
}
