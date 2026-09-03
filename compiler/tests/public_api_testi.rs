use dil::api::v1;
use std::collections::BTreeSet;

#[test]
fn v1_facade_derleme_calistirma_tani_ve_bicim_yuzeyini_sunar() {
    let program = v1::kaynagi_derle("\"merhaba\" yaz\n").expect("kaynak derlenmeli");
    assert!(v1::programi_dene(&program).is_empty());
    assert_eq!(
        v1::kaynagi_calistir("\"merhaba\" yaz\n").expect("kaynak çalışmalı"),
        ["merhaba"]
    );
    assert!(v1::kaynagi_denetle("1 + yaz\n").is_err());
    assert_eq!(
        v1::bicimle("\"merhaba\"   yaz\n").expect("biçim"),
        "\"merhaba\" yaz\n"
    );
}

#[test]
fn kokte_yalniz_api_modulu_desteklenen_yuzeydir() {
    let kok = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let kaynak = std::fs::read_to_string(kok.join("src/lib.rs")).expect("lib.rs okunmalı");
    assert!(kaynak.contains("pub mod api;"));

    let satirlar = kaynak.lines().collect::<Vec<_>>();
    for (sira, satir) in satirlar
        .iter()
        .enumerate()
        .filter(|(_, satir)| satir.starts_with("pub mod "))
    {
        if *satir == "pub mod api;" {
            continue;
        }
        assert!(
            satirlar[sira.saturating_sub(2)..sira].contains(&"#[doc(hidden)]"),
            "internal kök modül facade dışına açıkça sınıflanmamış: {satir}"
        );
    }
}

#[test]
fn v1_facade_exact_allowlist_disinda_yeni_ihracat_kabul_etmez() {
    let kok = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let api = std::fs::read_to_string(kok.join("src/api.rs")).expect("api.rs okunmalı");
    let beklenen = BTreeSet::from([
        "Program",
        "bicimle",
        "BaglanmisProgram",
        "Tani",
        "birim_ozeti",
        "gomulu_birim",
        "gomulu_birim_adlari",
        "kaynagi_calistir",
        "kaynagi_calistir_girdiyle",
        "kaynagi_dene",
        "kaynagi_denetle",
        "kaynagi_derle",
        "kaynagi_derle_birimlerle",
        "kaynagi_derle_kokenlerle",
        "kaynagi_fazli_derle",
        "kaynagi_fazli_derle_birimlerle",
        "kaynagi_fazli_derle_kokenlerle",
        "kaynagi_tanilari",
        "kaynagi_tanilari_kokenlerle",
        "programi_dene",
        "programi_dene_baglanmis",
        "BirimIstegi",
        "BirimYukleyici",
        "KokenliBirimYukleyici",
        "TestSonucu",
        "YuklenenBirim",
    ]);
    let mut bulunan = BTreeSet::new();
    for ifade in api.split(';') {
        let Some(kullanim) = ifade.split("pub use ").nth(1) else {
            continue;
        };
        if let Some((_, grup)) = kullanim.split_once('{') {
            for ad in grup.trim_end_matches('}').split(',').map(str::trim) {
                if !ad.is_empty() {
                    bulunan.insert(ad);
                }
            }
        } else if let Some(ad) = kullanim.trim().rsplit("::").next() {
            bulunan.insert(ad);
        }
    }
    assert_eq!(bulunan, beklenen, "V1 facade allowlist dışına çıktı");
}
