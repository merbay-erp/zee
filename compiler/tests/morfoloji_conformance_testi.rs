//! K-123/B-009: `zee-tr-1` bağımsız JSON conformance korpusu.

use dil::cozumleyici::{ad_cozumle, Tur};
use dil::morfoloji::{
    cozumleri_bul, ek_zinciri_uydur, profil_uyumluluk_kaydi, SoyutEk, EK_TABLOSU,
    MAKSIMUM_EK_KATMANI, MORFOLOJI_PROFILI, MORFOLOJI_SURUMU,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

const KORPUS: &str = include_str!("../../conformance/morfoloji/zee-tr-1.json");
const SEMA: &str = include_str!("../../conformance/morfoloji/sema-v1.schema.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Korpus {
    sema: String,
    normatif: String,
    profil: Profil,
    cozum_vakalari: Vec<CozumVakasi>,
    uretim_vakalari: Vec<UretimVakasi>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Profil {
    kimlik: String,
    sayisal_surum: u16,
    azami_ek_katmani: usize,
    uyumluluk_sha256: String,
    ekler: Vec<EkKaydi>,
    gecerli_iki_katman: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct EkKaydi {
    kimlik: String,
    yuzeyler: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CozumVakasi {
    kimlik: String,
    yuzey: String,
    kapsam: Vec<String>,
    yapisal: Vec<YapisalCozum>,
    karar: Karar,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct YapisalCozum {
    kok: String,
    ekler: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Karar {
    tur: String,
    deger: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UretimVakasi {
    kimlik: String,
    kok: String,
    ekler: Vec<String>,
    yuzey: Option<String>,
}

fn ek(adi: &str) -> SoyutEk {
    match adi {
        "belirtme" => SoyutEk::Belirtme,
        "iyelik" => SoyutEk::Iyelik,
        "tamlayan" => SoyutEk::Tamlayan,
        "yönelme" => SoyutEk::Yonelme,
        "ayrılma" => SoyutEk::Ayrilma,
        "bulunma" => SoyutEk::Bulunma,
        "araç" => SoyutEk::Arac,
        "çoğul+yönelme" => SoyutEk::CogulYonelme,
        baska => panic!("fixture bilinmeyen ek taşıyor: {baska}"),
    }
}

#[test]
fn bagimsiz_json_korpusu_profil_cozum_karar_ve_uretimi_birebir_tanimlar() {
    let sema: serde_json::Value = serde_json::from_str(SEMA).expect("JSON Schema geçerli olmalı");
    assert_eq!(
        sema.get("$id").and_then(serde_json::Value::as_str),
        Some("https://zee.dev/conformance/morfoloji/sema-v1.schema.json")
    );
    let korpus: Korpus = serde_json::from_str(KORPUS).expect("conformance JSON geçerli olmalı");
    assert_eq!(korpus.sema, "zee-morfoloji-conformance-v1");
    assert_eq!(korpus.normatif, "RFC-0018/spec-13");
    assert_eq!(korpus.profil.kimlik, MORFOLOJI_PROFILI);
    assert_eq!(korpus.profil.sayisal_surum, MORFOLOJI_SURUMU);
    assert_eq!(korpus.profil.azami_ek_katmani, MAKSIMUM_EK_KATMANI);
    assert!(
        profil_uyumluluk_kaydi().contains(&format!("sha256={}\n", korpus.profil.uyumluluk_sha256)),
        "conformance korpusu yanlış profil davranışını bağlıyor"
    );

    let gercek_ekler = EK_TABLOSU
        .iter()
        .map(|tanim| EkKaydi {
            kimlik: tanim.ek.adi().into(),
            yuzeyler: tanim.yuzeyler.iter().map(|yuzey| (*yuzey).into()).collect(),
        })
        .collect::<Vec<_>>();
    assert_eq!(korpus.profil.ekler, gercek_ekler);
    assert_eq!(
        korpus.profil.gecerli_iki_katman,
        ["belirtme", "tamlayan", "yönelme", "ayrılma", "bulunma", "araç"]
            .map(|dis| vec![String::from("iyelik"), String::from(dis)])
    );

    let mut kimlikler = HashSet::new();
    let mut karar_turleri = HashSet::new();
    for vaka in &korpus.cozum_vakalari {
        assert!(
            kimlikler.insert(&vaka.kimlik),
            "yinelenen vaka: {}",
            vaka.kimlik
        );
        let bulunan = cozumleri_bul(&vaka.yuzey)
            .into_iter()
            .map(|cozum| YapisalCozum {
                kok: cozum.kok,
                ekler: cozum.ekler.into_iter().map(|ek| ek.adi().into()).collect(),
            })
            .collect::<Vec<_>>();
        assert_eq!(bulunan, vaka.yapisal, "{} yapısal çözümü", vaka.kimlik);

        let ortam = vaka
            .kapsam
            .iter()
            .cloned()
            .map(|kok| (kok, Tur::TamSayi))
            .collect::<HashMap<_, _>>();
        let karar = ad_cozumle(&vaka.yuzey, &ortam, 1, 1, vaka.yuzey.chars().count());
        karar_turleri.insert(format!("{}:{}", vaka.karar.tur, vaka.karar.deger));
        match vaka.karar.tur.as_str() {
            "kok" => assert_eq!(
                karar.expect("kök çözülmeli"),
                vaka.karar.deger,
                "{}",
                vaka.kimlik
            ),
            "hata" => assert_eq!(
                karar.expect_err("korpus hata bekliyor").kod,
                vaka.karar.deger,
                "{}",
                vaka.kimlik
            ),
            baska => panic!("{} bilinmeyen karar türü taşıyor: {baska}", vaka.kimlik),
        }
    }
    assert_eq!(korpus.cozum_vakalari.len(), 27);
    assert!(karar_turleri.contains("hata:A001"));
    assert!(karar_turleri.contains("hata:A002"));
    assert!(karar_turleri.contains("kok:sayacı"));

    let mut tek_ek_kapsami = HashSet::new();
    let mut iki_ek_kapsami = HashSet::new();
    for vaka in &korpus.uretim_vakalari {
        assert!(
            kimlikler.insert(&vaka.kimlik),
            "yinelenen vaka: {}",
            vaka.kimlik
        );
        let ekler = vaka.ekler.iter().map(|ad| ek(ad)).collect::<Vec<_>>();
        assert_eq!(
            ek_zinciri_uydur(&vaka.kok, &ekler),
            vaka.yuzey,
            "{} üretimi",
            vaka.kimlik
        );
        match vaka.ekler.as_slice() {
            [tek] => {
                tek_ek_kapsami.insert(tek.clone());
            }
            [ic, dis] if ic == "iyelik" => {
                iki_ek_kapsami.insert(dis.clone());
            }
            _ => {}
        }
    }
    assert_eq!(korpus.uretim_vakalari.len(), 21);
    assert_eq!(
        tek_ek_kapsami,
        [
            "belirtme",
            "tamlayan",
            "yönelme",
            "ayrılma",
            "bulunma",
            "araç",
            "çoğul+yönelme"
        ]
        .map(String::from)
        .into()
    );
    assert_eq!(
        iki_ek_kapsami,
        ["belirtme", "tamlayan", "yönelme", "ayrılma", "bulunma", "araç"]
            .map(String::from)
            .into()
    );
}
