//! K-124/B-011: gözlenebilir `zee-esz-1` scheduler conformance korpusu.

use dil::yorumlayici::{calistir_io_kodla, GirdiCikti, ToplayanIo};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

const KORPUS: &str = include_str!("../../conformance/eszamanlilik/zee-esz-1.json");
const SEMA: &str = include_str!("../../conformance/eszamanlilik/sema-v1.schema.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Korpus {
    sema: String,
    normatif: String,
    profil: Profil,
    vakalar: Vec<Vaka>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Profil {
    kimlik: String,
    sayisal_surum: u16,
    yurutme_modeli: String,
    zaman_profili: String,
    uyumluluk_sozu: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Vaka {
    kimlik: String,
    amac: String,
    kaynak: String,
    beklenen: Beklenen,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Beklenen {
    sonlanma: Sonlanma,
    cikti: Vec<String>,
    sanal_sure_ms: i64,
    dosyalar: Vec<Dosya>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Sonlanma {
    tur: String,
    kod: Kod,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Kod {
    Sayi(i64),
    Metin(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Dosya {
    yol: String,
    icerik: String,
}

#[test]
fn bagimsiz_korpus_schedulerin_gozlenebilir_v1_sozunu_sabitler() {
    let sema: serde_json::Value = serde_json::from_str(SEMA).expect("JSON Schema geçerli olmalı");
    assert_eq!(
        sema.get("$id").and_then(serde_json::Value::as_str),
        Some("https://zee.dev/conformance/eszamanlilik/sema-v1.schema.json")
    );
    let korpus: Korpus = serde_json::from_str(KORPUS).expect("conformance JSON geçerli olmalı");
    assert_eq!(korpus.sema, "zee-eszamanlilik-conformance-v1");
    assert_eq!(korpus.normatif, "RFC-0011/spec-14");
    assert_eq!(korpus.profil.kimlik, "zee-esz-1");
    assert_eq!(korpus.profil.sayisal_surum, 1);
    assert_eq!(korpus.profil.yurutme_modeli, "tek-is-parcacigi-isbirlikli");
    assert_eq!(korpus.profil.zaman_profili, "zee-io-1");
    assert_eq!(
        korpus.profil.uyumluluk_sozu,
        [
            "bildirimde-ortam-snapshot",
            "ilk-poll-birlestirmede",
            "hazir-gorev-kaynak-sirasi",
            "esit-uyanis-kaynak-sirasi",
            "beklemeler-ortusur",
            "sonuclar-kaynak-sirasinda-baglanir",
            "ic-agac-dis-kardese-yol-verir",
            "ilk-hata-kardesleri-iptal-eder",
            "son-tarih-butun-agaca-yayilir",
            "eylem-atomik-scheduler-dilimidir",
            "program-cikisi-kardesleri-iptal-eder",
        ]
    );
    assert_eq!(korpus.vakalar.len(), 10);

    let mut kimlikler = HashSet::new();
    let mut gorulen_sozler = HashSet::new();
    for vaka in korpus.vakalar {
        assert!(
            kimlikler.insert(vaka.kimlik.clone()),
            "yinelenen vaka kimliği"
        );
        assert!(!vaka.amac.trim().is_empty(), "{} amacı boş", vaka.kimlik);
        let program = dil::kaynagi_derle(&vaka.kaynak)
            .unwrap_or_else(|hata| panic!("{} derlenemedi: {}", vaka.kimlik, hata));
        let mut io = ToplayanIo::yeni(Vec::new());
        let sonuc = calistir_io_kodla(&program, &mut io);

        match (&vaka.beklenen.sonlanma.tur[..], &vaka.beklenen.sonlanma.kod) {
            ("başarı", Kod::Sayi(0)) => {
                assert_eq!(sonuc.expect("başarı bekleniyor"), 0, "{}", vaka.kimlik);
            }
            ("çıkış", Kod::Sayi(kod)) => {
                assert_eq!(sonuc.expect("çıkış bekleniyor"), *kod, "{}", vaka.kimlik);
                gorulen_sozler.insert("çıkış");
            }
            ("hata", Kod::Metin(kod)) => {
                assert_eq!(
                    sonuc.expect_err("hata bekleniyor").kod,
                    *kod,
                    "{}",
                    vaka.kimlik
                );
                gorulen_sozler.insert("hata");
            }
            (tur, kod) => panic!("{} geçersiz sonlanma taşıyor: {tur}/{kod:?}", vaka.kimlik),
        }

        assert_eq!(io.cikti, vaka.beklenen.cikti, "{} çıktısı", vaka.kimlik);
        assert_eq!(
            io.an_ms(),
            vaka.beklenen.sanal_sure_ms,
            "{} sanal süresi",
            vaka.kimlik
        );
        let mut beklenen_dosyalar = HashMap::new();
        for dosya in vaka.beklenen.dosyalar {
            assert!(
                beklenen_dosyalar.insert(dosya.yol, dosya.icerik).is_none(),
                "{} yinelenen dosya yolu taşıyor",
                vaka.kimlik
            );
        }
        assert_eq!(
            io.dosyalar, beklenen_dosyalar,
            "{} dosya etkisi",
            vaka.kimlik
        );
    }
    assert_eq!(kimlikler.len(), 10);
    assert_eq!(gorulen_sozler, HashSet::from(["çıkış", "hata"]));
}
