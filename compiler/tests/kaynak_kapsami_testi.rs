use dil::test_kapsami;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn kaynaklar(kaynak: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(
        "fixture.dil".into(),
        Sha256::digest(kaynak.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )])
}
fn izle(kaynak: &str) -> (Vec<String>, Value) {
    let program = dil::kaynagi_derle(kaynak).expect("actual Zee source compiles");
    let n = test_kapsami::baslat(&program, "fixture.dil", kaynaklar(kaynak)).unwrap();
    let cikti = dil::yorumlayici::calistir(&program).expect("actual Zee source runs");
    (cikti, n.bitir(true).unwrap())
}

#[test]
fn kaynak_kapsami_cagrilmayan_islem_paydadan_kaybolmaz() {
    let s="işlem uyuyan\n    1 2 den küçükse\n        \"hiç\" yaz\n1 2 den küçükse\n    \"evet\" yaz\n";
    let (out, r) = izle(s);
    assert_eq!(out, ["evet"]);
    assert_eq!(r["product"]["payda"], 4);
    assert_eq!(r["product"]["vurulan"], 1);
    assert_eq!(r["product"]["eksik"].as_array().unwrap().len(), 3);
    assert!(r["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["sahip"] == "uyuyan")
        .all(|e| e["vurus"] == 0));
}

#[test]
fn kaynak_kapsami_kisa_devre_sag_tarafi_calistirmaz_ve_normal_cikti_ayni() {
    let s = "sol kapalıysa veya sag açıksa\n    \"tek\" yaz\n";
    let p = dil::kaynagi_derle(s).unwrap();
    let mut normal =
        dil::yorumlayici::IzKaydedenIo::yeni(dil::yorumlayici::ToplayanIo::yeni(Vec::new()));
    dil::yorumlayici::calistir_io(&p, &mut normal).unwrap();
    let normal_iz = normal.iz_metni().unwrap();
    let n = test_kapsami::baslat(&p, "fixture.dil", kaynaklar(s)).unwrap();
    let mut traced =
        dil::yorumlayici::IzKaydedenIo::yeni(dil::yorumlayici::ToplayanIo::yeni(Vec::new()));
    dil::yorumlayici::calistir_io(&p, &mut traced).unwrap();
    let traced_iz = traced.iz_metni().unwrap();
    let r = n.bitir(true).unwrap();
    assert_eq!(normal_iz, traced_iz);
    assert_eq!(traced_iz.matches("sensor_acik_mi").count(), 1);
    assert!(!traced_iz.contains("sag"));
    assert_eq!(traced.icine_al().cikti, ["tek"]);
    let logical = r["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["karar"] == "logical-operand")
        .collect::<Vec<_>>();
    assert_eq!(logical.len(), 4);
    assert_eq!(
        logical
            .iter()
            .filter(|e| e["vurus"].as_u64().unwrap() > 0)
            .count(),
        1
    );
}

#[test]
fn kaynak_kapsami_klon_envanter_sonrasinda_sessizce_kabul_edilmez() {
    let s = "1 2 den küçükse\n    \"ok\" yaz\n";
    let p = dil::kaynagi_derle(s).unwrap();
    let n = test_kapsami::baslat(&p, "fixture.dil", kaynaklar(s)).unwrap();
    let clone = p.clone();
    assert_eq!(dil::yorumlayici::calistir(&clone).unwrap_err().kod, "C000");
    assert!(n.bitir(false).is_err());
    let n = test_kapsami::baslat(&clone, "fixture.dil", kaynaklar(s)).unwrap();
    assert_eq!(dil::yorumlayici::calistir(&clone).unwrap(), ["ok"]);
    assert_eq!(n.bitir(true).unwrap()["product"]["vurulan"], 1);
}

#[test]
fn kaynak_kapsami_test_ve_urun_paydalari_ayri_ve_thread_yalitimli() {
    let source="işlem karar\n    1 2 den küçükse\n        \"ürün\" yaz\ntest \"gerçek\"\n    karar\n    1 1 e eşit olmalı\n";
    let p = dil::kaynagi_derle(source).unwrap();
    let n = test_kapsami::baslat(&p, "fixture.dil", kaynaklar(source)).unwrap();
    assert!(test_kapsami::baslat(&p, "fixture.dil", kaynaklar(source)).is_err());
    let other = std::thread::spawn(|| izle("1 2 den küçükse\n    \"öteki\" yaz\n").1)
        .join()
        .unwrap();
    assert_eq!(other["product"]["vurulan"], 1);
    assert!(dil::programi_dene(&p).iter().all(|t| t.hata.is_none()));
    let r = n.bitir(true).unwrap();
    assert_eq!(r["product"]["payda"], 2);
    assert_eq!(r["product"]["vurulan"], 1);
    assert_eq!(r["tests"]["payda"], 0);
}

#[test]
fn kaynak_kapsami_dongu_sifir_govde_ve_normal_bitis_gercek_vurulur() {
    let source = "0 kez tekrarla\n    \"asla\" yaz\n2 kez tekrarla\n    \"iki\" yaz\n";
    let (out, r) = izle(source);
    assert_eq!(out, ["iki", "iki"]);
    assert_eq!(r["product"]["payda"], 4);
    assert_eq!(r["product"]["vurulan"], 3);
    assert_eq!(
        r["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["vurus"].as_u64().unwrap() > 0)
            .count(),
        3
    );
}

#[test]
fn kaynak_kapsami_kaynak_butcesi_ve_drop_normal_yurutmeyi_kirletmez() {
    let s = "1 2 den küçükse\n    \"ok\" yaz\n";
    let p = dil::kaynagi_derle(s).unwrap();
    assert!(test_kapsami::baslat(&p, "fixture.dil", BTreeMap::new()).is_err());
    // Allowed origin bytes can expand sixfold in JSON; reject BEFORE runtime.
    let mut escaped = kaynaklar(s);
    for i in 0..test_kapsami::AZAMI_KAYNAK - 1 {
        escaped.insert(format!("{i}{}", "\u{0001}".repeat(4000)), "0".repeat(64));
    }
    assert!(test_kapsami::baslat(&p, "fixture.dil", escaped).is_err());
    let mut many = kaynaklar(s);
    for i in 0..test_kapsami::AZAMI_KAYNAK {
        many.insert(format!("extra-{i}"), "0".repeat(64));
    }
    assert!(test_kapsami::baslat(&p, "fixture.dil", many).is_err());
    {
        let _scope = test_kapsami::baslat(&p, "fixture.dil", kaynaklar(s)).unwrap();
    }
    assert_eq!(dil::yorumlayici::calistir(&p).unwrap(), ["ok"]);
    let n = test_kapsami::baslat(&p, "fixture.dil", kaynaklar(s)).unwrap();
    assert_eq!(n.bitir(true).unwrap()["product"]["vurulan"], 0);
}

#[test]
fn kaynak_kapsami_tum_dongu_turleri_ve_gore_kisa_devreyi_sayar() {
    let source = "1 den 2 ye kadar her sayı için\n    sayıyı yaz\nsayaç 1 olsun\nsayaç 0 dan büyük olduğu sürece\n    sayacı 1 azalt\nbitti yanlış olsun\nbitti doğru olana kadar tekrarla\n    bitti doğru olsun\nsayılar 3, 7 listesi olsun\nher sayı için\n    sayıyı yaz\nşekil \"kare\" olsun\nşekle göre\n    \"daire\" ise\n        \"hayır\" yaz\n    \"kare\" ise\n        \"evet\" yaz\n    \"üçgen\" ise\n        \"geç\" yaz\n";
    let (out, r) = izle(source);
    assert_eq!(out, ["1", "2", "3", "7", "evet"]);
    assert_eq!(r["product"]["payda"], 14);
    assert_eq!(r["product"]["vurulan"], 10);
    let comparisons = r["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["karar"] == "match-comparison")
        .collect::<Vec<_>>();
    assert_eq!(comparisons.len(), 6);
    assert_eq!(
        comparisons
            .iter()
            .filter(|e| e["vurus"].as_u64().unwrap() > 0)
            .count(),
        2
    );
}

#[test]
fn kaynak_kapsami_sure_normal_ve_timeout_kollari_gercek_io_saatiyle() {
    let source = "1 saniye içinde\n    \"normal\" yaz\nyetişmezse\n    \"asla\" yaz\n1 saniye içinde\n    2 saniye bekle\n    \"asla\" yaz\nyetişmezse\n    \"timeout\" yaz\n";
    let (out, r) = izle(source);
    assert_eq!(out, ["normal", "timeout"]);
    assert_eq!(r["product"]["payda"], 4);
    assert_eq!(r["product"]["vurulan"], 2);
    assert_eq!(r["edges"][0]["vurus"], 1);
    assert_eq!(r["edges"][3]["vurus"], 1);
}

#[test]
fn kaynak_kapsami_kimlikler_ayni_snapshotta_adresten_bagimsizdir() {
    let source = "1 2 den küçükse\n    \"ok\" yaz\n";
    let first = izle(source).1;
    let second = izle(source).1;
    assert_eq!(first, second);
    let mut changed = kaynaklar(source);
    changed.insert("fixture.dil".into(), "NOT_A_HASH".into());
    let p = dil::kaynagi_derle(source).unwrap();
    assert!(test_kapsami::baslat(&p, "fixture.dil", changed).is_err());
    let large = "1 2 den küçükse\n    \"x\" yaz\n".repeat(test_kapsami::AZAMI_DAL / 2 + 1);
    let p = dil::kaynagi_derle(&large).unwrap();
    assert!(test_kapsami::baslat(&p, "fixture.dil", kaynaklar(&large)).is_err());
}

#[test]
#[cfg_attr(
    not(unix),
    ignore = "actual Unix owned HTTP/source coverage unavailable"
)]
fn kaynak_kapsami_gercek_native_session_csrf_ve_modul_dallari() {
    #[cfg(not(unix))]
    panic!("Unix owned native capability unavailable");
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let out = root.join(format!(
            "compiler/target/kaynak-kapsami-native-{}-{nonce}",
            std::process::id()
        ));
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("compiler/tests/fixtures/kaynak-kapsami-native.py"))
            .arg("--framework-root")
            .arg(root)
            .arg("--dil")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("--output")
            .arg(&out)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        let r: Value =
            serde_json::from_slice(&std::fs::read(out.join("proof.json")).unwrap()).unwrap();
        assert_eq!(r["status"], "passed-short-candidate");
        assert_eq!(r["owned_groups_closed"], true);
        assert_eq!(r["cases"].as_array().unwrap().len(), 3);
        assert_eq!(r["cases"][1]["requests"], 11);
        assert_eq!(
            r["cases"][1]["login_true_false_hits"],
            serde_json::json!([1, 1])
        );
    }
}

#[test]
fn kaynak_kapsami_imported_route_kokeni_klonda_korunur_kapaliyken_tahsis_edilmez() {
    let main = "a rotalarını kullan\nb rotalarını kullan\n8080 kapısında sunucu başlat\n";
    let a="GET \"/a\" adresine istek geldiğinde\n    1 2 den küçükse\n        \"a\" yanıtını gönder\n";
    let b="GET \"/b\" adresine istek geldiğinde\n    2 1 den küçükse\n        \"b\" yanıtını gönder\n";
    let compile = || {
        dil::kaynagi_derle_kokenlerle(main, Some("main.dil"), &mut |request: dil::BirimIstegi<
            '_,
        >| {
            Ok(dil::YuklenenBirim {
                kaynak: if request.ad == "a" { a } else { b }.into(),
                koken: format!("{}.dil", request.ad),
            })
        })
        .unwrap()
    };
    let normal = compile();
    assert!(normal
        .cumleler
        .iter()
        .all(|c| !matches!(c, dil::agac::Cumle::IstekGeldiginde { koken: Some(_), .. })));
    let scoped = {
        let _scope = test_kapsami::derleme_baslat(true);
        compile()
    };
    let clone = scoped.clone();
    let mut sources = BTreeMap::new();
    for (origin, source) in [("main.dil", main), ("a.dil", a), ("b.dil", b)] {
        sources.insert(
            origin.into(),
            Sha256::digest(source.as_bytes())
                .iter()
                .map(|x| format!("{x:02x}"))
                .collect(),
        );
    }
    assert!(test_kapsami::baslat(&normal, "main.dil", sources.clone()).is_err());
    let n = test_kapsami::baslat(&clone, "main.dil", sources).unwrap();
    let r = n.bitir(true).unwrap();
    assert_eq!(r["product"]["payda"], 4);
    for origin in ["a.dil", "b.dil"] {
        let edges = r["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kaynak"] == origin)
            .collect::<Vec<_>>();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().all(|e| e["satir"] == 2 && e["vurus"] == 0));
    }
    assert!(compile()
        .cumleler
        .iter()
        .all(|c| !matches!(c, dil::agac::Cumle::IstekGeldiginde { koken: Some(_), .. })));
}
