//! B-018 — kaynak, token, parsed AST, bound AST ve yürütme fazları.

use dil::agac::{Cumle, Ifade};
use dil::faz::KaynakMetni;
use dil::{kaynagi_derle, kaynagi_fazli_derle};

#[test]
fn kaynak_token_ast_ve_baglanmis_program_ayri_turlerdir() {
    let kaynak = "değer 1 olsun\ndeğeri yaz\n";
    let tokenlar = KaynakMetni::yeni(kaynak).sozcukle().expect("sözcüklenmeli");
    assert!(!tokenlar.tokenlar().is_empty());

    let ast = tokenlar.ayristir(Vec::new()).expect("ayrıştırılmalı");
    assert_eq!(ast.cumleler().len(), 2);
    let Cumle::Yaz {
        deger:
            Ifade::Degisken {
                sembol_kimligi: None,
                ..
            },
        ..
    } = &ast.cumleler()[1]
    else {
        panic!("parsed AST henüz semantic kimlik taşımamalı")
    };

    let baglanmis = kaynagi_fazli_derle(kaynak).expect("checker bağı kurulmalı");
    let Cumle::Yaz {
        deger:
            Ifade::Degisken {
                sembol_kimligi: Some(_),
                ..
            },
        ..
    } = &baglanmis.cumleler[1]
    else {
        panic!("bound AST semantic kimlik taşımalı")
    };
    assert_eq!(
        dil::yorumlayici::calistir_baglanmis(&baglanmis).expect("yalnız bağlı program yürümeli"),
        vec!["1"]
    );
}

#[test]
fn eski_program_api_fazli_hattan_sonra_uyumluluk_adaptorudur() {
    let kaynak = "\"merhaba\" yaz\n";
    let eski = kaynagi_derle(kaynak).expect("eski API korunmalı");
    let fazli = kaynagi_fazli_derle(kaynak).expect("fazlı API çalışmalı");

    assert_eq!(eski.cumleler.len(), fazli.cumleler.len());
    assert_eq!(
        dil::yorumlayici::calistir(&eski).expect("eski runtime adaptörü"),
        dil::yorumlayici::calistir_baglanmis(&fazli).expect("fazlı runtime")
    );
}
