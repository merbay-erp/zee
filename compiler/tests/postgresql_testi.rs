//! K-163: PostgreSQL yüzeyinin parametre, Sonuç/Hata ve eylem sınırı.

use dil::kaynagi_derle;
use dil::yorumlayici::{self, ToplayanIo, VeritabaniHatasi};

#[test]
fn okuma_parametreyi_sql_metininden_ayri_tasir() {
    let kaynak = r#"
parametreler "x' OR true --" listesi olsun
satırlar "SELECT baslik::text AS baslik FROM sayfalar WHERE slug = $1" sorgusunu parametreler ile okumayı dene olsun
satırlar başarılıysa
    "ok" yaz
"#;
    let program = kaynagi_derle(kaynak).expect("PostgreSQL okuması derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.postgresql
        .okuma_sonuclari
        .push_back(Ok(vec![vec![("baslik".into(), "güvenli".into())]]));
    yorumlayici::calistir_io(&program, &mut io).expect("okuma çalışmalı");
    assert_eq!(io.cikti, ["ok"]);
    assert_eq!(
        io.postgresql.istekler,
        [(
            "SELECT baslik::text AS baslik FROM sayfalar WHERE slug = $1".into(),
            vec!["x' OR true --".into()],
            false
        )]
    );
}

#[test]
fn veritabani_hatasi_yapilandirilmis_sonuc_verisine_donusur() {
    let kaynak = r#"
eylem kaydı ekle
    TamSayı sonucu döndürür
    parametreler "aynı-slug" listesi olsun
    sonuç "INSERT INTO sayfalar (slug) VALUES ($1)" sorgusunu parametreler ile değiştirmeyi dene olsun
    sonuç döndür

sonuç kaydı ekle olsun
sonuç başarısızsa
    hata sonucun hatası olsun
    hatanın kodu yaz
    bilgi hatanın verisi olsun
    bilginin "sqlstate" değeri yaz
    bilginin "kısıt" değeri yaz
"#;
    let program = kaynagi_derle(kaynak).expect("eylem içindeki DB yazımı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.postgresql
        .degistirme_sonuclari
        .push_back(Err(VeritabaniHatasi {
            mesaj: "yinelenen slug".into(),
            veri: vec![
                ("sqlstate".into(), "23505".into()),
                ("kısıt".into(), "sayfalar_slug_key".into()),
            ],
        }));
    yorumlayici::calistir_io(&program, &mut io).expect("beklenen DB hatası yönetilmeli");
    assert_eq!(io.cikti, ["C025", "23505", "sayfalar_slug_key"]);
}
