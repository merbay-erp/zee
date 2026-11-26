//! K-228: own isolated PostgreSQL only; status/dry plan, checksum and roles.

use dil::postgresql::{
    gocleri_planla, gocleri_planla_secenekli, gocleri_uygula, gocleri_uygula_secenekli, GocRaporu,
};
use dil::veritabani_modeli::{VeritabaniBildirimi, VeritabaniHedefi};
use postgres::{Client, NoTls};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

struct Proje(PathBuf);
impl Drop for Proje {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("own test project cleanup");
    }
}

fn bildirim(ad: &str) -> VeritabaniBildirimi {
    let hedef =
        std::env::var("ZEE_POSTGRES_GOC_CI_HEDEFI").expect("strict isolated fixture required");
    let hedef = VeritabaniHedefi::ayristir(&hedef).unwrap();
    assert!(hedef.loopback_mi());
    let baglanti_degiskeni = format!("ZEE_POSTGRES_GOC_CI_{ad}_URL");
    std::env::var(&baglanti_degiskeni).expect("strict own role URL required");
    VeritabaniBildirimi {
        hedef,
        baglanti_degiskeni,
        gocler: "gocler".into(),
        istek_baglami_cozucusu: None,
    }
}

fn metadata(istemci: &mut Client) -> Vec<(i64, String, String, String)> {
    istemci
        .query(
            "SELECT surum, ad, sha256, uygulandi::text FROM _zee_gocleri ORDER BY surum",
            &[],
        )
        .unwrap()
        .iter()
        .map(|r| (r.get(0), r.get(1), r.get(2), r.get(3)))
        .collect()
}

fn degerler(istemci: &mut Client) -> Vec<String> {
    istemci
        .query("SELECT value FROM migration_probe ORDER BY value", &[])
        .unwrap()
        .iter()
        .map(|r| r.get(0))
        .collect()
}

fn tablo_var(istemci: &mut Client, ad: &str) -> bool {
    istemci
        .query_one("SELECT to_regclass($1::text) IS NOT NULL", &[&ad])
        .unwrap()
        .get(0)
}

fn dosya(goc: &Path, ad: &str, sql: &str) {
    std::fs::write(goc.join(ad), sql).unwrap();
}

#[test]
#[ignore = "scripts/postgresql-goc-ci.py creates its own empty database and roles"]
fn izole_goc_durum_kuru_checksum_rol_ve_atomik_uygulama_sinirini_korur() {
    let owner = bildirim("OWNER");
    let runtime = bildirim("RUNTIME");
    let url = std::env::var("ZEE_POSTGRES_GOC_CI_OWNER_URL").unwrap();
    let mut gozlem = Client::connect(&url, NoTls).unwrap();
    gozlem
        .batch_execute(
            "REVOKE ALL ON SCHEMA public FROM PUBLIC; GRANT USAGE ON SCHEMA public TO zee_runtime;",
        )
        .unwrap();
    let proje = Proje(std::env::temp_dir().join(format!(
            "zee-goc-plan-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    let goc = proje.0.join("gocler");
    std::fs::create_dir_all(&goc).unwrap();
    let ilk_sql =
        "CREATE TABLE migration_probe(value TEXT); INSERT INTO migration_probe VALUES ('one');";
    // Creation order differs from migration order.
    dosya(
        &goc,
        "0002_ikinci.sql",
        "INSERT INTO migration_probe VALUES ('two');",
    );
    dosya(&goc, "0001_ilk.sql", ilk_sql);
    for rol in [&owner, &runtime] {
        let plan = gocleri_planla(rol, &proje.0).unwrap();
        assert_eq!((plan.uygulanmis, plan.bekleyen), (0, 2));
        assert_eq!(
            plan.adimlar.iter().map(|a| a.surum).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            plan.adimlar[0].sha256,
            Sha256::digest(ilk_sql.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
    assert!(!tablo_var(&mut gozlem, "_zee_gocleri"));
    assert!(!tablo_var(&mut gozlem, "migration_probe"));
    let rol_reddi = gocleri_uygula(&runtime, &proje.0).unwrap_err();
    assert!(rol_reddi
        .veri
        .contains(&("sqlstate".into(), "42501".into())));
    assert!(!tablo_var(&mut gozlem, "_zee_gocleri"));
    assert!(!tablo_var(&mut gozlem, "migration_probe"));
    assert_eq!(
        gocleri_uygula(&owner, &proje.0).unwrap(),
        GocRaporu {
            uygulanan: 2,
            atlanan: 0,
            ileri: vec![],
        }
    );
    gozlem
        .batch_execute("GRANT SELECT ON _zee_gocleri, migration_probe TO zee_runtime;")
        .unwrap();
    let kayitlar = metadata(&mut gozlem);
    assert_eq!(degerler(&mut gozlem), ["one", "two"]);
    for rol in [&owner, &runtime] {
        let plan = gocleri_planla(rol, &proje.0).unwrap();
        assert_eq!((plan.uygulanmis, plan.bekleyen), (2, 0));
        assert!(plan.adimlar.iter().all(|a| a.uygulanmis));
    }
    assert_eq!(
        gocleri_uygula(&owner, &proje.0).unwrap(),
        GocRaporu {
            uygulanan: 0,
            atlanan: 2,
            ileri: vec![],
        }
    );
    assert_eq!(metadata(&mut gozlem), kayitlar);
    dosya(
        &goc,
        "0003_ucuncu.sql",
        "INSERT INTO migration_probe VALUES ('three');",
    );
    let plan = gocleri_planla(&runtime, &proje.0).unwrap();
    assert_eq!((plan.uygulanmis, plan.bekleyen), (2, 1));
    assert!(gocleri_uygula(&runtime, &proje.0).is_err());
    assert_eq!(degerler(&mut gozlem), ["one", "two"]);
    assert_eq!(metadata(&mut gozlem), kayitlar);
    // A changed comment is still a changed byte hash; neither mode may hide it.
    dosya(&goc, "0001_ilk.sql", &format!("{ilk_sql}\n-- changed\n"));
    for rol in [&owner, &runtime] {
        assert_eq!(
            gocleri_planla(rol, &proje.0).unwrap_err().veri,
            [("surum".into(), "1".into())]
        );
    }
    assert_eq!(
        gocleri_uygula(&owner, &proje.0).unwrap_err().veri,
        [("surum".into(), "1".into())]
    );
    dosya(&goc, "0001_ilk.sql", ilk_sql);
    std::fs::rename(goc.join("0001_ilk.sql"), goc.join("0001_yeni.sql")).unwrap();
    assert!(gocleri_planla(&owner, &proje.0).is_err());
    assert!(gocleri_uygula(&owner, &proje.0).is_err());
    std::fs::remove_file(goc.join("0001_yeni.sql")).unwrap();
    assert!(gocleri_planla(&runtime, &proje.0)
        .unwrap_err()
        .mesaj
        .contains("proje ağacında yok"));
    assert!(gocleri_uygula(&owner, &proje.0).is_err());
    assert_eq!(degerler(&mut gozlem), ["one", "two"]);
    assert_eq!(metadata(&mut gozlem), kayitlar);
    dosya(&goc, "0001_ilk.sql", ilk_sql);
    dosya(
        &goc,
        "0004_bozuk.sql",
        "INSERT INTO migration_probe VALUES ('before_bad'); SELECT 1/0;",
    );
    let sql_reddi = gocleri_uygula(&owner, &proje.0).unwrap_err();
    assert!(sql_reddi
        .veri
        .contains(&("sqlstate".into(), "22012".into())));
    // Both the earlier pending migration and the failed migration roll back.
    assert_eq!(degerler(&mut gozlem), ["one", "two"]);
    assert_eq!(metadata(&mut gozlem), kayitlar);
    std::fs::remove_file(goc.join("0004_bozuk.sql")).unwrap();
    assert_eq!(
        gocleri_uygula(&owner, &proje.0).unwrap(),
        GocRaporu {
            uygulanan: 1,
            atlanan: 2,
            ileri: vec![],
        }
    );
    assert_eq!(degerler(&mut gozlem), ["one", "three", "two"]);
    dosya(
        &goc,
        "0004_paralel.sql",
        "INSERT INTO migration_probe VALUES ('parallel');",
    );
    let (sol, sag) = std::thread::scope(|scope| {
        let sol = scope.spawn(|| gocleri_uygula(&owner, &proje.0));
        let sag = scope.spawn(|| gocleri_uygula(&owner, &proje.0));
        (sol.join().unwrap().unwrap(), sag.join().unwrap().unwrap())
    });
    assert_eq!(sol.uygulanan + sag.uygulanan, 1);
    assert_eq!(sol.atlanan + sag.atlanan, 7);
    assert_eq!(degerler(&mut gozlem), ["one", "parallel", "three", "two"]);
    assert_eq!(metadata(&mut gozlem).len(), 4);
    // K-407: A/B kök geri dönüşü — yeni dilim 0006'yı uygulamış, eski ağaçta yok.
    gozlem
        .batch_execute(
            "INSERT INTO _zee_gocleri (surum, ad, sha256) VALUES (6, '0006_yeni_dilim.sql', 'fff');",
        )
        .unwrap();
    let ileri_kayitlar = metadata(&mut gozlem);
    assert!(gocleri_planla(&runtime, &proje.0)
        .unwrap_err()
        .mesaj
        .contains("proje ağacında yok"));
    assert!(gocleri_uygula(&owner, &proje.0).is_err());
    for rol in [&owner, &runtime] {
        let plan = gocleri_planla_secenekli(rol, &proje.0, true).unwrap();
        assert_eq!((plan.uygulanmis, plan.bekleyen), (4, 0));
        assert_eq!(plan.ileri.iter().map(|a| a.surum).collect::<Vec<_>>(), [6]);
    }
    let rapor = gocleri_uygula_secenekli(&owner, &proje.0, true).unwrap();
    assert_eq!(
        (rapor.uygulanan, rapor.atlanan, rapor.ileri.len()),
        (0, 4, 1)
    );
    assert_eq!(degerler(&mut gozlem), ["one", "parallel", "three", "two"]);
    assert_eq!(metadata(&mut gozlem), ileri_kayitlar);
    // Araya giren uygulanmamış göç ileri kayıtla gizlenemez; SQL yürümez.
    dosya(
        &goc,
        "0005_ara.sql",
        "INSERT INTO migration_probe VALUES ('ara');",
    );
    assert!(gocleri_planla_secenekli(&runtime, &proje.0, true).is_err());
    assert!(gocleri_uygula_secenekli(&owner, &proje.0, true).is_err());
    std::fs::remove_file(goc.join("0005_ara.sql")).unwrap();
    dosya(
        &goc,
        "0003_ucuncu.sql",
        "INSERT INTO migration_probe VALUES ('three'); -- değişti",
    );
    assert!(gocleri_uygula_secenekli(&owner, &proje.0, true).is_err());
    dosya(
        &goc,
        "0003_ucuncu.sql",
        "INSERT INTO migration_probe VALUES ('three');",
    );
    assert!(gocleri_planla_secenekli(&owner, &proje.0, true).is_ok());
    assert_eq!(degerler(&mut gozlem), ["one", "parallel", "three", "two"]);
    assert_eq!(metadata(&mut gozlem), ileri_kayitlar);
    gozlem
        .batch_execute("DELETE FROM _zee_gocleri WHERE surum = 6;")
        .unwrap();
    gozlem
        .batch_execute("REVOKE SELECT ON _zee_gocleri FROM zee_runtime;")
        .unwrap();
    let ret = gocleri_planla(&runtime, &proje.0).unwrap_err();
    assert!(ret.veri.contains(&("sqlstate".into(), "42501".into())));
    urun_i_defter_geri_donusu(&owner, &mut gozlem);
}

/// K-407: `dogfood/urun-i` ZeeOS defter aynasının KOKEN a–d adımları; küme N
/// 0001'i, küme N+1 0001+0002'yi taşır. Ayna dosyaları yalnız okunur.
fn urun_i_defter_geri_donusu(owner: &VeritabaniBildirimi, gozlem: &mut Client) {
    gozlem.batch_execute("DROP TABLE _zee_gocleri;").unwrap();
    let kok = Path::new(env!("CARGO_MANIFEST_DIR")).join("../dogfood/urun-i/defter-geri-donus");
    let (n, n1) = (kok.join("n"), kok.join("n1"));
    let rapor = |uygulanan, atlanan| GocRaporu {
        uygulanan,
        atlanan,
        ileri: vec![],
    };
    assert_eq!(gocleri_uygula(owner, &n).unwrap(), rapor(1, 0));
    assert_eq!(gocleri_uygula(owner, &n1).unwrap(), rapor(1, 1));
    let kayitlar = metadata(gozlem);
    assert!(gocleri_uygula(owner, &n)
        .unwrap_err()
        .mesaj
        .contains("proje ağacında yok"));
    assert!(gocleri_planla(owner, &n)
        .unwrap_err()
        .mesaj
        .contains("proje ağacında yok"));
    let plan = gocleri_planla_secenekli(owner, &n, true).unwrap();
    assert_eq!((plan.uygulanmis, plan.bekleyen), (1, 0));
    assert_eq!(
        plan.ileri
            .iter()
            .map(|a| (a.surum, a.ad.as_str()))
            .collect::<Vec<_>>(),
        [(2, "0002_etiket.sql")]
    );
    let geri = gocleri_uygula_secenekli(owner, &n, true).unwrap();
    assert_eq!((geri.uygulanan, geri.atlanan), (0, 1));
    assert_eq!(geri.ileri, plan.ileri);
    assert_eq!(metadata(gozlem), kayitlar);
    // N+1'e yeniden geçiş kayıtları aynen bulur.
    assert_eq!(gocleri_uygula(owner, &n1).unwrap(), rapor(0, 2));
    assert_eq!(metadata(gozlem), kayitlar);
}
