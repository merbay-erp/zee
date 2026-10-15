//! K-297: UTF-16 imleç ve yanıt aralıkları gerçek LSP mesaj işleyicisinden gelir.
//! Süreç başlatılmaz; normalleştirilen sonuç ayrı manifest beklentisiyle karşılaştırılır.
use dil::lsp::Sunucu;
use serde_json::{json, Value};

const URI: &str = "untitled:k297-utf16";

fn istek(sunucu: &mut Sunucu, id: u64, yontem: &str, params: Value) -> Value {
    let cikti = sunucu
        .mesaj_isle(&json!({"jsonrpc":"2.0","id":id,"method":yontem,"params":params}).to_string());
    assert!(cikti.devam, "sorgu sunucuyu kapatmamalı");
    assert_eq!(cikti.govdeler.len(), 1, "sorgu tek yanıt üretmeli");
    let yanit: Value = serde_json::from_str(&cikti.govdeler[0]).expect("geçerli JSON yanıtı");
    assert_eq!(yanit["jsonrpc"], "2.0");
    assert_eq!(yanit["id"], id);
    assert!(yanit.get("error").is_none(), "sorgu hatası: {yanit}");
    yanit.get("result").expect("sonuç alanı").clone()
}

fn aralik(aralik: &Value) -> String {
    let satir = aralik["start"]["line"].as_u64().expect("başlangıç satırı");
    let bas = aralik["start"]["character"]
        .as_u64()
        .expect("başlangıç sütunu");
    let son_satir = aralik["end"]["line"].as_u64().expect("bitiş satırı");
    let son = aralik["end"]["character"].as_u64().expect("bitiş sütunu");
    assert_eq!(satir, son_satir, "fixture adı tek satırda kalmalı");
    format!("{satir}:{bas}-{son}")
}

pub(super) fn calistir(kaynak: &str) -> (i64, Vec<String>) {
    let mut sunucu = Sunucu::yeni();
    let acilis = sunucu.mesaj_isle(
        &json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
            "textDocument":{"uri":URI,"languageId":"dil","version":1,"text":kaynak}
        }})
        .to_string(),
    );
    assert!(acilis.devam);
    assert_eq!(acilis.govdeler.len(), 1, "tek açık belge tanısı");
    let tani: Value = serde_json::from_str(&acilis.govdeler[0]).expect("tanı JSON");
    assert_eq!(tani["method"], "textDocument/publishDiagnostics");
    assert_eq!(tani["params"]["uri"], URI);
    assert_eq!(tani["params"]["diagnostics"], json!([]));

    let konum = json!({"textDocument":{"uri":URI},"position":{"line":1,"character":12}});
    let hover = istek(&mut sunucu, 1, "textDocument/hover", konum.clone());
    let hover_dogru = hover["contents"]["kind"] == "markdown"
        && hover["contents"]["value"]
            .as_str()
            .is_some_and(|metin| metin.contains("isim \"Ayşe\" olsun"));
    let tanim = istek(&mut sunucu, 2, "textDocument/definition", konum.clone());
    assert_eq!(tanim["uri"], URI);

    let mut rename_konumu = konum;
    rename_konumu["newName"] = json!("ad");
    let rename = istek(&mut sunucu, 3, "textDocument/rename", rename_konumu);
    let tanimdan_rename = istek(
        &mut sunucu,
        4,
        "textDocument/rename",
        json!({"textDocument":{"uri":URI},"position":{"line":0,"character":2},"newName":"ad"}),
    );
    assert_eq!(
        rename, tanimdan_rename,
        "iki imleç aynı semantic düzenlemeyi seçmeli"
    );
    let degisiklikler = rename["changes"]
        .as_object()
        .expect("WorkspaceEdit değişiklikleri");
    assert_eq!(degisiklikler.len(), 1, "yalnız açık belge değişmeli");
    let duzenlemeler = degisiklikler[URI].as_array().expect("düzenleme dizisi");
    let araliklar = duzenlemeler
        .iter()
        .map(|duzenleme| {
            assert_eq!(duzenleme["newText"], "ad");
            aralik(&duzenleme["range"])
        })
        .collect::<Vec<_>>()
        .join(",");
    (
        0,
        vec![format!(
            "hover={hover_dogru};definition={};rename={araliklar}",
            aralik(&tanim["range"])
        )],
    )
}
