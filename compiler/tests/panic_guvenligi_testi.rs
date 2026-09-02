//! K-109/B-014 production panic yüzeyi regresyonları.

use dil::agac::{Cumle, Ifade, Program};
use std::collections::HashMap;

#[test]
fn elle_kurulmus_gecersiz_ast_panik_yerine_tani_uretir() {
    let mut program = Program {
        cumleler: vec![Cumle::Yaz {
            deger: Ifade::Degisken {
                ham: "sıfır".into(),
                cozulmus: None,
                sembol_kimligi: None,
                satir: 0,
                sutun: 0,
                uzunluk: 0,
            },
            satir: 0,
        }],
        islemler: HashMap::new(),
        yapilar: Vec::new(),
        testler: Vec::new(),
    };

    let tani = dil::cozumleyici::denetle(&mut program).expect_err("tanı bekleniyor");
    assert_eq!(tani.kod, "T016");
    assert!(tani.mesaj.contains("kaynak aralığı"));
}

#[test]
fn production_crate_kokleri_panic_lint_kapisini_tasir() {
    let kok = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for goreli in [
        "src/lib.rs",
        "src/main.rs",
        "src/bin/dillsp.rs",
        "src/bin/olcum.rs",
    ] {
        let kaynak = std::fs::read_to_string(kok.join(goreli)).expect("crate kökü okunmalı");
        for lint in [
            "clippy::unwrap_used",
            "clippy::expect_used",
            "clippy::panic",
            "clippy::unreachable",
            "clippy::todo",
            "clippy::unimplemented",
        ] {
            assert!(
                kaynak.contains(lint),
                "{goreli} production kapısında {lint} eksik"
            );
        }
    }
}
