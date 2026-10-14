//! K-163: PostgreSQL yüzeyinin parametre, Sonuç/Hata ve eylem sınırı.

use dil::kaynagi_derle;
use dil::yorumlayici::{self, ToplayanIo, VeritabaniHatasi};

// K-222/GB-022: Bunlar yalnız aşağıdaki izolasyon koşucusunun sağladığı
// geçici sunucuda koşar. Eksik fixture değişkeni başarı/return olamaz.
// `python3 scripts/postgresql-tls-ci.py` beş vakayı --ignored ile zorlar.
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

#[test]
#[ignore = "scripts/postgresql-tls-ci.py geçici gerçek TLS sunucusunu sağlar"]
fn izole_tls_sorgu_parametre_sonuc_ve_goc_sinirlarini_bir_fazlasinda_reddeder() {
    let mut oturum = tls_olumlu_oturum();
    let sinir = dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.veritabani();
    oturum.eylem_baslat().unwrap();
    let mut sorgu = "SELECT 'ok'::text AS value --".to_string();
    sorgu.extend(std::iter::repeat_n(' ', sinir.sorgu_bayti() - sorgu.len()));
    assert_eq!(oturum.oku(&sorgu, &[]).unwrap()[0][0].1, "ok");
    sorgu.push(' ');
    assert!(oturum
        .oku(&sorgu, &[])
        .unwrap_err()
        .mesaj
        .contains("64 KiB"));

    let parametreler = vec!["x".to_string(); sinir.parametre()];
    let ifadeler = (1..=sinir.parametre())
        .map(|i| format!("${i}::text"))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT concat({ifadeler}) AS value");
    assert_eq!(
        oturum.oku(&sql, &parametreler).unwrap()[0][0].1.len(),
        sinir.parametre()
    );
    let fazla = vec!["x".to_string(); sinir.parametre() + 1];
    assert!(oturum
        .oku(&sql, &fazla)
        .unwrap_err()
        .mesaj
        .contains("100 öğe"));
    let deger = "x".repeat(sinir.parametre_bayti());
    assert_eq!(
        oturum
            .oku("SELECT $1::text AS value", std::slice::from_ref(&deger))
            .unwrap()[0][0]
            .1,
        deger
    );
    assert!(oturum
        .oku("SELECT $1::text AS value", &[format!("{deger}x")])
        .unwrap_err()
        .mesaj
        .contains("64 KiB"));

    assert_eq!(
        oturum
            .oku(
                &format!(
                    "SELECT i::text AS value FROM generate_series(1,{}) i",
                    sinir.satir()
                ),
                &[]
            )
            .unwrap()
            .len(),
        sinir.satir()
    );
    assert!(oturum
        .oku(
            &format!(
                "SELECT i::text AS value FROM generate_series(1,{}) i",
                sinir.satir() + 1
            ),
            &[]
        )
        .unwrap_err()
        .mesaj
        .contains("10.000 satır"));
    let alanlar = (1..=sinir.sutun())
        .map(|i| format!("'v'::text AS c{i}"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        oturum.oku(&format!("SELECT {alanlar}"), &[]).unwrap()[0].len(),
        sinir.sutun()
    );
    assert!(oturum
        .oku(&format!("SELECT {alanlar},'v'::text AS fazla"), &[])
        .unwrap_err()
        .mesaj
        .contains("100 sütun"));
    // Collector counts column-name bytes as well as value bytes: alias x is one byte.
    let sonuc = oturum
        .oku(
            &format!("SELECT repeat('v', {})::text AS x", sinir.sonuc_bayti() - 1),
            &[],
        )
        .unwrap();
    assert_eq!(sonuc[0][0].1.len() + 1, sinir.sonuc_bayti());
    drop(sonuc);
    assert!(oturum
        .oku(
            &format!("SELECT repeat('v', {})::text AS x", sinir.sonuc_bayti()),
            &[]
        )
        .unwrap_err()
        .mesaj
        .contains("16 MiB"));
    assert_eq!(
        oturum.oku("SELECT 'healthy'::text AS value", &[]).unwrap()[0][0].1,
        "healthy"
    );
    oturum.eylem_tamamla().unwrap();

    struct Kok(std::path::PathBuf);
    impl Drop for Kok {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let kok = Kok(std::env::temp_dir().join(format!("zee-goc-sinir-{}", std::process::id())));
    std::fs::create_dir(&kok.0).unwrap();
    std::fs::create_dir(kok.0.join("gocler")).unwrap();
    let mut bildirim = tls_bildirimi("DOGRU");
    bildirim.gocler = "gocler".into();
    let sql = " ".repeat(usize::try_from(sinir.goc_dosyasi_bayti()).unwrap());
    let adet = sinir.goc_toplami_bayti() / sinir.goc_dosyasi_bayti();
    for i in 1..=adet {
        std::fs::write(kok.0.join(format!("gocler/{i:04}_sinir.sql")), &sql).unwrap();
    }
    let plan = dil::postgresql::gocleri_planla(&bildirim, &kok.0).unwrap();
    assert_eq!(plan.adimlar.len(), usize::try_from(adet).unwrap());
    assert_eq!(plan.bekleyen, usize::try_from(adet).unwrap());
    let fazla = kok.0.join(format!("gocler/{:04}_fazla.sql", adet + 1));
    std::fs::write(&fazla, " ").unwrap();
    assert!(dil::postgresql::gocleri_planla(&bildirim, &kok.0)
        .unwrap_err()
        .mesaj
        .contains("toplamı 8 MiB"));
    std::fs::remove_file(&fazla).unwrap();
    std::fs::write(kok.0.join("gocler/0001_sinir.sql"), format!("{sql} ")).unwrap();
    assert!(dil::postgresql::gocleri_planla(&bildirim, &kok.0)
        .unwrap_err()
        .mesaj
        .contains("Tek migration 1 MiB"));
}
