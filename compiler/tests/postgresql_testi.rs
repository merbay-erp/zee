//! K-163: PostgreSQL yüzeyinin parametre, Sonuç/Hata ve eylem sınırı.

use dil::kaynagi_derle;
use dil::yorumlayici::{self, ToplayanIo, VeritabaniHatasi};

// K-222/GB-022: Bunlar yalnız aşağıdaki izolasyon koşucusunun sağladığı
// geçici sunucuda koşar. Eksik fixture değişkeni başarı/return olamaz.
// `python3 scripts/postgresql-tls-ci.py` dört vakayı --ignored ile zorlar.
fn tls_bildirimi(ad: &str) -> dil::veritabani_modeli::VeritabaniBildirimi {
    let degisken = format!("ZEE_POSTGRES_CI_{ad}_URL");
    let hedef = std::env::var(format!("ZEE_POSTGRES_CI_{ad}_HEDEFI"))
        .expect("test yalnız geçici PostgreSQL TLS koşucusuyla çalıştırılmalı");
    let hedef = dil::veritabani_modeli::VeritabaniHedefi::ayristir(&hedef)
        .expect("izole test hedefi geçerli olmalı");
    assert!(hedef.loopback_mi(), "test yalnız loopback fixture kullanır");
    std::env::var(&degisken).expect("fixture bağlantı URL'si zorunlu");
    std::env::var(format!("{degisken}_TLS_CA_PEM")).expect("fixture kökü zorunlu");
    dil::veritabani_modeli::VeritabaniBildirimi {
        hedef,
        baglanti_degiskeni: degisken,
        gocler: "unused".into(),
        istek_baglami_cozucusu: None,
    }
}

fn tls_olumlu_oturum() -> dil::postgresql::PostgresqlOturumu {
    dil::postgresql::PostgresqlOturumu::baglan(&tls_bildirimi("DOGRU"), 0)
        .expect("doğru CA ve localhost SAN ile gerçek TLS bağlantısı kurulmalı")
}

#[test]
#[ignore = "scripts/postgresql-tls-ci.py geçici gerçek TLS sunucusunu sağlar"]
fn izole_tls_dogru_ca_hostname_ve_tls12_ile_baglanir() {
    let mut oturum = tls_olumlu_oturum();
    let sonuc = oturum
        .oku(
            "SELECT ssl::text AS ssl, version::text AS version FROM pg_stat_ssl WHERE pid = pg_backend_pid()",
            &[],
        )
        .expect("sunucu negotiated TLS kaydı okunmalı");
    assert_eq!(
        sonuc,
        [vec![
            ("ssl".into(), "true".into()),
            ("version".into(), "TLSv1.2".into())
        ]],
        "fixture yalnız TLS 1.2 sunar; plaintext kabul olamaz"
    );
}

#[test]
#[ignore = "scripts/postgresql-tls-ci.py geçici gerçek TLS sunucusunu sağlar"]
fn izole_tls_baska_ca_gercek_sunucuyu_reddeder() {
    let mut olumlu = tls_olumlu_oturum();
    assert!(olumlu.oku("SELECT 'once'::text AS health", &[]).is_ok());
    assert!(
        dil::postgresql::PostgresqlOturumu::baglan(&tls_bildirimi("YANLIS_CA"), 0).is_err(),
        "başka kökün imzaladığı CA bu erişilebilir sunucuyu doğrulayamaz"
    );
    assert!(olumlu.oku("SELECT 'sonra'::text AS health", &[]).is_ok());
}

#[test]
#[ignore = "scripts/postgresql-tls-ci.py geçici gerçek TLS sunucusunu sağlar"]
fn izole_tls_san_disindaki_ip_hostnameini_reddeder() {
    let mut olumlu = tls_olumlu_oturum();
    assert!(olumlu.oku("SELECT 'once'::text AS health", &[]).is_ok());
    assert!(
        dil::postgresql::PostgresqlOturumu::baglan(&tls_bildirimi("YANLIS_HOST"), 0).is_err(),
        "127.0.0.1 aynı sunucudur fakat leaf yalnız DNS:localhost SAN taşır"
    );
    assert!(olumlu.oku("SELECT 'sonra'::text AS health", &[]).is_ok());
}

#[test]
#[ignore = "scripts/postgresql-tls-ci.py geçici gerçek TLS sunucusunu sağlar"]
fn izole_tls_null_ve_metin_disini_reddeder_transaction_sagligi_korunur() {
    let mut oturum = tls_olumlu_oturum();
    oturum.eylem_baslat().expect("test transaction'ı başlamalı");
    for (sql, beklenen) in [
        (
            "SELECT NULL::text AS value",
            "\"value\" sütunu NULL; sorguda COALESCE ile açık metin değeri seç",
        ),
        (
            "SELECT 1::integer AS value",
            "\"value\" sütunu Metin değil; sorguda açık ::text dönüşümü kullan",
        ),
    ] {
        let hata = oturum
            .oku(sql, &[])
            .expect_err("uygunsuz hücre reddedilmeli");
        assert_eq!(hata.mesaj, beklenen);
        assert_eq!(hata.veri, [("sutun".into(), "value".into())]);
        assert_eq!(
            oturum.oku("SELECT 'healthy'::text AS health", &[]).unwrap(),
            [vec![("health".into(), "healthy".into())]],
            "uygulama sonuç reddi transaction'ı kirletmemeli"
        );
    }
    assert_eq!(
        oturum
            .oku(
                "SELECT COALESCE(NULL::text, 'fallback') AS value, 1::text AS cast",
                &[]
            )
            .unwrap(),
        [vec![
            ("value".into(), "fallback".into()),
            ("cast".into(), "1".into())
        ]]
    );
    oturum
        .eylem_tamamla()
        .expect("sağlıklı transaction commit olmalı");
}

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
