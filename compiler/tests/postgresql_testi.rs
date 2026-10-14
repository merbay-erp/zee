//! K-163: PostgreSQL yüzeyinin parametre, Sonuç/Hata ve eylem sınırı.

use dil::kaynagi_derle;
use dil::yorumlayici::{self, ToplayanIo, VeritabaniHatasi};

// K-222/GB-022: Bunlar yalnız aşağıdaki izolasyon koşucusunun sağladığı
// geçici sunucuda koşar. Eksik fixture değişkeni başarı/return olamaz.
// `python3 scripts/postgresql-tls-ci.py` yedi vakayı --ignored ile zorlar.
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

// K-250: statement-time faults on stock PostgreSQL; no COMMIT classification rule.
fn fault_admin() -> postgres::Client {
    let mut config: postgres::Config = std::env::var("ZEE_POSTGRES_CI_DOGRU_URL")
        .expect("owned fixture URL required")
        .parse()
        .unwrap();
    config.connect_timeout(std::time::Duration::from_secs(5));
    config.options(
        "-c statement_timeout=15000 -c lock_timeout=12000 -c application_name=zee_k250_observer",
    );
    let ca = std::env::var("ZEE_POSTGRES_CI_DOGRU_URL_TLS_CA_PEM").unwrap();
    let mut tls = native_tls::TlsConnector::builder();
    tls.disable_built_in_roots(true);
    tls.add_root_certificate(native_tls::Certificate::from_pem(ca.as_bytes()).unwrap());
    tls.min_protocol_version(Some(native_tls::Protocol::Tlsv12));
    config
        .connect(postgres_native_tls::MakeTlsConnector::new(
            tls.build().unwrap(),
        ))
        .expect("independent owned TLS observer")
}

fn fault_oturum(ad: &str) -> dil::postgresql::PostgresqlOturumu {
    dil::postgresql::PostgresqlOturumu::baglan(&tls_bildirimi(ad), 0)
        .expect("controlled owned fault session")
}

fn fault_pid(oturum: &mut dil::postgresql::PostgresqlOturumu) -> i32 {
    oturum
        .oku("SELECT pg_backend_pid()::text AS pid", &[])
        .unwrap()[0][0]
        .1
        .parse()
        .unwrap()
}

fn fault_sqlstate(hata: &VeritabaniHatasi) -> &str {
    assert!(
        !hata
            .veri
            .iter()
            .any(|(ad, deger)| ad == "hata_sinifi" && deger == "db.commit_unknown"),
        "statement failure is not a COMMIT attempt"
    );
    hata.veri
        .iter()
        .find(|(ad, _)| ad == "sqlstate")
        .expect("actual server SQLSTATE")
        .1
        .as_str()
}

fn fault_fresh_health(old_pids: &[i32]) {
    let mut fresh = tls_olumlu_oturum();
    assert_eq!(
        fresh.oku("SELECT 'fresh'::text AS health", &[]).unwrap()[0][0].1,
        "fresh"
    );
    assert!(
        !old_pids.contains(&fault_pid(&mut fresh)),
        "new pool uses an independent backend"
    );
}

#[test]
#[ignore = "scripts/postgresql-tls-ci.py creates the owned TLS statement-fault fixture"]
fn izole_tls_deadlock_40p01_tekrarsiz_rollback_ve_yeni_oturum_sagligi() {
    use std::sync::mpsc;
    use std::time::Duration;
    let mut admin = fault_admin();
    admin.batch_execute("CREATE TABLE zee_k250_deadlock(id bigint PRIMARY KEY,value bigint NOT NULL); INSERT INTO zee_k250_deadlock VALUES(1,0),(2,0); CREATE SEQUENCE zee_k250_deadlock_attempts; CREATE FUNCTION zee_k250_deadlock_attempt() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM nextval('zee_k250_deadlock_attempts'); RETURN NULL; END $$; CREATE TRIGGER zee_k250_deadlock_attempt BEFORE UPDATE ON zee_k250_deadlock FOR EACH STATEMENT EXECUTE FUNCTION zee_k250_deadlock_attempt();").unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    let mut handles = Vec::new();
    let mut gates = Vec::new();
    for first in [1, 2] {
        let (go_tx, go_rx) = mpsc::sync_channel(1);
        gates.push(go_tx);
        let ready = ready_tx.clone();
        handles.push(std::thread::spawn(move || {
            let mut oturum = fault_oturum("DEADLOCK");
            oturum.eylem_baslat().unwrap();
            let pid = fault_pid(&mut oturum);
            let sql = "UPDATE zee_k250_deadlock SET value=value+1 WHERE id=$1::bigint";
            assert_eq!(oturum.degistir(sql, &[first.to_string()]).unwrap(), 1);
            ready.send(pid).unwrap();
            go_rx
                .recv_timeout(Duration::from_secs(20))
                .expect("bounded control barrier");
            let second = oturum.degistir(sql, &[(3 - first).to_string()]);
            let health = oturum.oku("SELECT 'alive'::text AS health", &[]);
            let rollback = oturum.eylem_geri_al();
            (second, health, rollback)
        }));
    }
    drop(ready_tx);
    let pids: Vec<_> = (0..2)
        .map(|_| {
            ready_rx
                .recv_timeout(Duration::from_secs(20))
                .expect("both real row locks acknowledged")
        })
        .collect();
    assert_ne!(pids[0], pids[1], "two independent PostgreSQL sessions");
    for gate in gates {
        gate.send(()).unwrap();
    }
    // Join both before asserting either result: a rejected actor explicitly releases
    // its outer row lock so the other actor can complete, then also rolls back.
    let joined: Vec<_> = handles.into_iter().map(|handle| handle.join()).collect();
    let results: Vec<_> = joined.into_iter().map(Result::unwrap).collect();
    let mut rejected = 0;
    let mut accepted = 0;
    for (second, health, rollback) in results {
        match second {
            Ok(1) => accepted += 1,
            Err(hata) => {
                assert_eq!(fault_sqlstate(&hata), "40P01");
                rejected += 1;
            }
            other => panic!("unexpected controlled deadlock result: {other:?}"),
        }
        assert_eq!(
            health.unwrap()[0][0].1,
            "alive",
            "intrinsic savepoint recovered"
        );
        rollback.expect("explicit outer ROLLBACK acknowledged");
    }
    assert_eq!((rejected, accepted), (1, 1));
    let attempts: i64 = admin
        .query_one("SELECT last_value FROM zee_k250_deadlock_attempts", &[])
        .unwrap()
        .get(0);
    assert_eq!(
        attempts, 4,
        "two writes per actor; nontransactional statement observer exposes any replay"
    );
    let durable_sum: i64 = admin
        .query_one("SELECT sum(value)::bigint FROM zee_k250_deadlock", &[])
        .unwrap()
        .get(0);
    assert_eq!(
        durable_sum, 0,
        "independent durable observer sees both explicit rollbacks"
    );
    fault_fresh_health(&pids);
    eprintln!("K250 deadlock sqlstate=40P01 attempts=4 rejected=1 accepted=1 durable_sum=0 fresh_health=true");
}

#[test]
#[ignore = "scripts/postgresql-tls-ci.py creates the owned TLS statement-fault fixture"]
fn izole_tls_serialization_40001_tekrarsiz_rollback_ve_yeni_oturum_sagligi() {
    let mut admin = fault_admin();
    admin.batch_execute("CREATE TABLE zee_k250_serial(id bigint PRIMARY KEY,value bigint NOT NULL); INSERT INTO zee_k250_serial VALUES(1,0),(2,0); CREATE SEQUENCE zee_k250_serial_attempts; CREATE FUNCTION zee_k250_serial_attempt() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF current_setting('application_name')='zee_k250_serialization' THEN PERFORM nextval('zee_k250_serial_attempts'); END IF; RETURN NULL; END $$; CREATE TRIGGER zee_k250_serial_attempt BEFORE UPDATE ON zee_k250_serial FOR EACH STATEMENT EXECUTE FUNCTION zee_k250_serial_attempt();").unwrap();
    let mut victim = fault_oturum("SERIALIZABLE");
    victim.eylem_baslat().unwrap();
    assert_eq!(
        victim
            .oku(
                "SELECT current_setting('transaction_isolation')::text AS isolation",
                &[]
            )
            .unwrap()[0][0]
            .1,
        "serializable"
    );
    let pid = fault_pid(&mut victim);
    assert_eq!(
        victim
            .oku("SELECT value::text FROM zee_k250_serial WHERE id=1", &[])
            .unwrap()[0][0]
            .1,
        "0",
        "snapshot barrier is an acknowledged read"
    );
    assert_eq!(
        victim
            .degistir("UPDATE zee_k250_serial SET value=value+10 WHERE id=2", &[])
            .unwrap(),
        1
    );
    let invisible: i64 = admin
        .query_one("SELECT value FROM zee_k250_serial WHERE id=2", &[])
        .unwrap()
        .get(0);
    assert_eq!(invisible, 0, "victim marker is still uncommitted");
    admin
        .batch_execute("BEGIN; UPDATE zee_k250_serial SET value=value+1 WHERE id=1; COMMIT")
        .expect("other session commits after victim snapshot");
    let hata = victim
        .degistir("UPDATE zee_k250_serial SET value=value+10 WHERE id=1", &[])
        .expect_err("stock concurrent update causes real serialization rejection");
    assert_eq!(fault_sqlstate(&hata), "40001");
    assert_eq!(
        victim.oku("SELECT 'alive'::text AS health", &[]).unwrap()[0][0].1,
        "alive"
    );
    victim
        .eylem_geri_al()
        .expect("explicit outer ROLLBACK acknowledged");
    let attempts: i64 = admin
        .query_one("SELECT last_value FROM zee_k250_serial_attempts", &[])
        .unwrap()
        .get(0);
    assert_eq!(
        attempts, 2,
        "prior write and failing statement only; no automatic replay"
    );
    let values: Vec<i64> = admin
        .query("SELECT value FROM zee_k250_serial ORDER BY id", &[])
        .unwrap()
        .into_iter()
        .map(|row| row.get(0))
        .collect();
    assert_eq!(
        values,
        [1, 0],
        "other commit survives; victim marker and failed update do not"
    );
    let idle: bool = admin
        .query_one(
            "SELECT state='idle' AND xact_start IS NULL FROM pg_stat_activity WHERE pid=$1",
            &[&pid],
        )
        .unwrap()
        .get(0);
    assert!(
        idle,
        "server observed no open transaction after acknowledged rollback"
    );
    fault_fresh_health(&[pid]);
    eprintln!("K250 serialization sqlstate=40001 attempts=2 other_value=1 victim_marker=0 fresh_health=true");
}
