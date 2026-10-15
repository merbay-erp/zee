//! Aralıklı eşitlemede Unicode konum ve bütün paket atomikliği.
use dil::lsp::Sunucu;
use serde_json::{json, Value};

fn gonder(s: &mut Sunucu, method: &str, params: Value) -> Vec<Value> {
    s.mesaj_isle(&json!({"jsonrpc":"2.0","id":9,"method":method,"params":params}).to_string())
        .govdeler
        .iter()
        .map(|x| serde_json::from_str(x).unwrap())
        .collect()
}
fn ac(s: &mut Sunucu, text: &str) {
    gonder(
        s,
        "textDocument/didOpen",
        json!({"textDocument":{"uri":"untitled:k287","text":text,"version":1}}),
    );
}
fn degistir(s: &mut Sunucu, changes: Value) -> Vec<Value> {
    gonder(
        s,
        "textDocument/didChange",
        json!({"textDocument":{"uri":"untitled:k287","version":2},"contentChanges":changes}),
    )
}
fn aralik(line: usize, a: usize, b: usize, text: &str) -> Value {
    json!({"range":{"start":{"line":line,"character":a},"end":{"line":line,"character":b}},"text":text})
}
fn bicim(s: &mut Sunucu) -> Value {
    gonder(
        s,
        "textDocument/formatting",
        json!({"textDocument":{"uri":"untitled:k287"},"options":{"tabSize":4,"insertSpaces":true}}),
    )
    .pop()
    .unwrap()["result"]
        .clone()
}

#[test]
fn utf16_surrogate_ve_crlf_duzenlemesi_kanonik_bicime_ulasir() {
    let mut s = Sunucu::yeni();
    ac(&mut s, "\"😀x\"   yaz\r\n");
    let out = degistir(&mut s, json!([aralik(0, 3, 4, "ş")]));
    assert!(out.iter().any(|x| x["params"]["diagnostics"] == json!([])));
    assert_eq!(bicim(&mut s)[0]["newText"], "\"😀ş\" yaz\n");
}

#[test]
fn sirali_duzenlemeler_onceki_adayin_konumuna_uygulanir() {
    let mut s = Sunucu::yeni();
    ac(&mut s, "\"ab\"  yaz\n");
    degistir(
        &mut s,
        json!([aralik(0, 1, 2, "xyz"), aralik(0, 4, 5, "ç")]),
    );
    assert_eq!(bicim(&mut s)[0]["newText"], "\"xyzç\" yaz\n");
}

#[test]
fn hatali_paket_ve_surrogate_ortasi_belgeyi_hic_degistirmez() {
    for bad in [
        aralik(0, 2, 2, "x"),
        aralik(9, 0, 0, "x"),
        aralik(0, 4, 1, "x"),
        json!({"range":{"start":{"line":0,"character":1},"end":{"line":0,"character":3}},"rangeLength":1,"text":"x"}),
    ] {
        let mut s = Sunucu::yeni();
        ac(&mut s, "\"😀a\"  yaz\n");
        let out = degistir(&mut s, json!([aralik(0, 3, 4, "b"), bad]));
        assert_eq!(out.last().unwrap()["error"]["code"], -32602);
        assert_eq!(bicim(&mut s)[0]["newText"], "\"😀a\" yaz\n");
    }
}

#[test]
fn tam_metin_ardindan_aralik_ve_bicim_idempotenttir() {
    let mut s = Sunucu::yeni();
    ac(&mut s, "\"eski\" yaz\n");
    degistir(
        &mut s,
        json!([{"text":"\"yeni\" yaz\n"},aralik(0,1,5,"son")]),
    );
    assert_eq!(bicim(&mut s), json!([]));
    let out = gonder(&mut s, "initialize", json!({}));
    assert_eq!(out[0]["result"]["capabilities"]["textDocumentSync"], 2);
    assert_eq!(
        out[0]["result"]["capabilities"]["documentFormattingProvider"],
        true
    );
}

#[test]
fn kapali_belge_aralik_ve_bozuk_kaynak_bicimi_reddedilir() {
    let mut s = Sunucu::yeni();
    assert_eq!(
        degistir(&mut s, json!([aralik(0, 0, 0, "x")]))
            .last()
            .unwrap()["error"]["code"],
        -32602
    );
    ac(&mut s, "\"kapanmadi\n");
    assert_eq!(bicim(&mut s), Value::Null);
}

#[test]
fn lint_optin_uyari_siddeti_ve_hatali_ayar_ayridir() {
    let mut s = Sunucu::yeni();
    let bad = gonder(
        &mut s,
        "initialize",
        json!({"initializationOptions":{"lint":"true"}}),
    );
    assert_eq!(bad[0]["error"]["code"], -32602);
    gonder(
        &mut s,
        "initialize",
        json!({"initializationOptions":{"lint":true}}),
    );
    let out = gonder(
        &mut s,
        "textDocument/didOpen",
        json!({"textDocument":{"uri":"untitled:k287","text":"bos 1 olsun\n"}}),
    );
    let d = &out
        .iter()
        .find(|x| x["method"] == "textDocument/publishDiagnostics")
        .unwrap()["params"]["diagnostics"];
    assert_eq!(d[0]["code"], "A012");
    assert_eq!(d[0]["severity"], 2);
}

struct Proje(std::path::PathBuf);
static PROJE_SIRASI: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
impl Proje {
    fn yeni() -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-lsp-k287-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            PROJE_SIRASI.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&yol).unwrap();
        std::fs::write(
            yol.join("proje.dil"),
            "proje \"editor\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
        )
        .unwrap();
        std::fs::write(yol.join("ana.dil"), "").unwrap();
        Self(yol)
    }
    fn yaz(&self, ad: &str, metin: &str) {
        std::fs::write(self.0.join(ad), metin).unwrap();
    }
    fn ac(&self, s: &mut Sunucu, ad: &str, metin: &str) -> Value {
        let out = gonder(
            s,
            "textDocument/didOpen",
            json!({"textDocument":{"uri":self.uri(ad),"text":metin,"version":1}}),
        );
        out.into_iter()
            .find(|v| v["method"] == "textDocument/publishDiagnostics")
            .unwrap()["params"]["diagnostics"]
            .clone()
    }
    fn uri(&self, ad: &str) -> String {
        format!("file://{}", self.0.join(ad).display())
    }
}
impl Drop for Proje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const YARDIMCI: &str = "işlem değer ver\n    TamSayı döndürür\n    7 döndür\n";
const ANA: &str = "yardimci birimini kullan\nsonuç değer ver olsun\nsonucu yaz\n";

#[test]
fn sicak_ve_soguk_oturum_ayni_taniyi_verir_disk_degisimi_gizlenmez() {
    let p = Proje::yeni();
    p.yaz("yardimci.dil", YARDIMCI);
    let mut sicak = Sunucu::yeni();
    assert_eq!(p.ac(&mut sicak, "ana.dil", ANA), json!([]));
    assert_eq!(p.ac(&mut sicak, "ana.dil", ANA), json!([]));
    p.yaz("yardimci.dil", "\"kapanmayan");
    let bozuk = p.ac(&mut sicak, "ana.dil", ANA);
    assert!(!bozuk.as_array().unwrap().is_empty());
    assert_eq!(bozuk, p.ac(&mut Sunucu::yeni(), "ana.dil", ANA));
    p.yaz("yardimci.dil", YARDIMCI);
    assert_eq!(p.ac(&mut sicak, "ana.dil", ANA), json!([]));
    // İthalat kaldırıldığında eski birim tanıları yaşamaz.
    p.yaz("yardimci.dil", "\"kapanmayan");
    assert_eq!(p.ac(&mut sicak, "ana.dil", "\"tamam\" yaz\n"), json!([]));
}

#[test]
fn kaydedilmemis_yerel_birim_kullanilir_kapaninca_disk_geri_gelir() {
    let p = Proje::yeni();
    p.yaz("yardimci.dil", YARDIMCI);
    let mut s = Sunucu::yeni();
    assert_eq!(p.ac(&mut s, "ana.dil", ANA), json!([]));
    p.ac(&mut s, "yardimci.dil", "\"kapanmayan");
    assert!(!p.ac(&mut s, "ana.dil", ANA).as_array().unwrap().is_empty());
    gonder(
        &mut s,
        "textDocument/didClose",
        json!({"textDocument":{"uri":p.uri("yardimci.dil")}}),
    );
    assert_eq!(p.ac(&mut s, "ana.dil", ANA), json!([]));
}

#[test]
fn celisen_uri_aliaslari_rastgele_overlay_secmez() {
    let p = Proje::yeni();
    p.yaz("yardimci.dil", YARDIMCI);
    let mut s = Sunucu::yeni();
    p.ac(&mut s, "yardimci.dil", "\"kapanmayan");
    assert!(!p.ac(&mut s, "ana.dil", ANA).as_array().unwrap().is_empty());
    let alias = p.uri("%79ardimci.dil");
    gonder(
        &mut s,
        "textDocument/didOpen",
        json!({"textDocument":{"uri":alias,"text":YARDIMCI,"version":1}}),
    );
    assert_eq!(p.ac(&mut s, "ana.dil", ANA), json!([]));
    gonder(
        &mut s,
        "textDocument/didClose",
        json!({"textDocument":{"uri":alias}}),
    );
    assert!(!p.ac(&mut s, "ana.dil", ANA).as_array().unwrap().is_empty());
}

#[test]
fn birim_acma_degistirme_kapatma_acik_tuketiciyi_kendiliginden_yeniler() {
    let p = Proje::yeni();
    p.yaz("yardimci.dil", YARDIMCI);
    let mut s = Sunucu::yeni();
    assert_eq!(p.ac(&mut s, "ana.dil", ANA), json!([]));
    let ana = p.uri("ana.dil");
    let birim = p.uri("yardimci.dil");
    let sonuc = |out: Vec<Value>| {
        out.into_iter()
            .find(|v| v["method"] == "textDocument/publishDiagnostics" && v["params"]["uri"] == ana)
            .unwrap()["params"]["diagnostics"]
            .clone()
    };
    let acilis = gonder(
        &mut s,
        "textDocument/didOpen",
        json!({"textDocument":{"uri":birim,"text":"\"kapanmayan","version":1}}),
    );
    assert!(!sonuc(acilis).as_array().unwrap().is_empty());
    let onarim = gonder(
        &mut s,
        "textDocument/didChange",
        json!({"textDocument":{"uri":birim,"version":2},"contentChanges":[{"text":YARDIMCI}]}),
    );
    assert_eq!(sonuc(onarim), json!([]));
    let bozma = gonder(
        &mut s,
        "textDocument/didChange",
        json!({"textDocument":{"uri":birim,"version":3},"contentChanges":[{"text":"\"kapanmayan"}]}),
    );
    assert!(!sonuc(bozma).as_array().unwrap().is_empty());
    let kapanis = gonder(
        &mut s,
        "textDocument/didClose",
        json!({"textDocument":{"uri":birim}}),
    );
    assert_eq!(sonuc(kapanis), json!([]));
}

#[test]
fn toplu_tani_yayini_tek_yanit_butcesini_asamaz() {
    let mut s = Sunucu::yeni();
    let limit = dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.lsp_yanit_bayti();
    let uzun = "a".repeat(limit / 4);
    let mut son = Vec::new();
    for i in 0..5 {
        son = gonder(
            &mut s,
            "textDocument/didOpen",
            json!({"textDocument":{"uri":format!("untitled:{i}{uzun}"),"text":"","version":1}}),
        );
    }
    assert_eq!(son.len(), 1);
    assert_eq!(son[0]["error"]["code"], -32001);
}

#[test]
fn kilitli_alt_paket_acik_editor_metniyle_degistirilemez() {
    let p = Proje::yeni();
    std::fs::create_dir(p.0.join("bagimli")).unwrap();
    p.yaz(
        "bagimli/proje.dil",
        "proje \"bagimli\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    );
    p.yaz("bagimli/ana.dil", YARDIMCI);
    p.yaz("proje.dil", "proje \"editor\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyerel_bağımlılıklar \"bagimli\" listesi olsun\n");
    dil::paket::ProjeGrafigi::cozumle(&p.0)
        .unwrap()
        .kilidi_yaz()
        .unwrap();
    let ana = "bagimli paketini kullan\nsonuç değer ver olsun\nsonucu yaz\n";
    let mut s = Sunucu::yeni();
    assert_eq!(p.ac(&mut s, "ana.dil", ana), json!([]));
    p.ac(&mut s, "bagimli/ana.dil", "\"kapanmayan");
    assert_eq!(p.ac(&mut s, "ana.dil", ana), json!([]));
    p.yaz("bagimli/ana.dil", "# kilit bozuldu\n");
    p.ac(&mut s, "bagimli/ana.dil", YARDIMCI);
    let tanilar = p.ac(&mut s, "ana.dil", ana);
    assert!(
        tanilar
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["code"] == "P008"),
        "{tanilar}"
    );
}

/// Makine ölçümü temiz exact checkout'ta `--release --ignored --nocapture` ile alınır.
/// Paylaşılan koşucuda 200 ms hard gate uygulanmaz; ham örnekler raporlanır.
#[test]
#[ignore = "K-287: 20 bin satır gerçek dosya grafiği release ölçümü"]
// Ignored ölçüm yanlışlıkla debug kipinde çağrılırsa açıkça reddet.
#[allow(clippy::assertions_on_constants)]
fn yirmi_bin_satir_proje_duzenleme_olcumu() {
    assert!(!cfg!(debug_assertions), "ölçüm release profili ister");
    let p = Proje::yeni();
    let mut ana = String::new();
    let mut toplam_satir = 0;
    for i in 0..40 {
        let mut birim = format!("işlem hesap{i}\n    TamSayı döndürür\n");
        for _ in 0..497 {
            birim.push_str("    ara 7 olsun\n");
        }
        birim.push_str("    arayı döndür\n");
        toplam_satir += birim.lines().count();
        p.yaz(&format!("birim{i}.dil"), &birim);
        ana.push_str(&format!("birim{i} birimini kullan\n"));
    }
    ana.push_str("deger 0 olsun\n");
    for i in 0..40 {
        ana.push_str(&format!("hesap{i} yaz\n"));
    }
    toplam_satir += ana.lines().count();
    assert!(toplam_satir >= 20_000);
    p.yaz("ana.dil", &ana);
    for tum_belgeler_acik in [false, true] {
        let mut s = Sunucu::yeni();
        let soguk = std::time::Instant::now();
        assert_eq!(p.ac(&mut s, "ana.dil", &ana), json!([]));
        if tum_belgeler_acik {
            for i in 0..40 {
                let ad = format!("birim{i}.dil");
                let metin = std::fs::read_to_string(p.0.join(&ad)).unwrap();
                assert_eq!(p.ac(&mut s, &ad, &metin), json!([]));
            }
        }
        let soguk_ms = soguk.elapsed().as_secs_f64() * 1000.0;
        let mut ornekler = Vec::new();
        for i in 0..25 {
            let bas = std::time::Instant::now();
            let out = gonder(
                &mut s,
                "textDocument/didChange",
                json!({
                    "textDocument":{"uri":p.uri("ana.dil"),"version":i+2},
                    "contentChanges":[aralik(40, 6, 7, if i % 2 == 0 { "1" } else { "0" })]
                }),
            );
            assert!(
                out.iter()
                    .filter(|x| x["method"] == "textDocument/publishDiagnostics")
                    .count()
                    == if tum_belgeler_acik { 41 } else { 1 }
                    && out
                        .iter()
                        .filter(|x| x["method"] == "textDocument/publishDiagnostics")
                        .all(|x| x["params"]["diagnostics"] == json!([]))
                    && out.iter().all(|x| x.get("error").is_none()),
                "{out:?}"
            );
            let ms = bas.elapsed().as_secs_f64() * 1000.0;
            if i >= 5 {
                ornekler.push(ms);
            }
        }
        let mut sirali = ornekler.clone();
        sirali.sort_by(f64::total_cmp);
        println!(
            "{}",
            json!({"schema":"zee-lsp-edit-measurement-1","lines":toplam_satir,
        "open_documents":if tum_belgeler_acik {41} else {1},
        "modules":40,"warmup_edits":5,"samples_ms":ornekler,"cold_ms":soguk_ms,
        "p95_ms":sirali[18],"max_ms":sirali[19],"budget_ms":200,
        "scope":"ranged edit + fresh project/lock/loader + token/leaf-parse reuse + fresh binding/type + serialized diagnostics",
        "unmeasured":"editor transport, arbitrary real products, incremental HIR, lint opt-in"})
        );
    }
}

fn konum_istegi(s: &mut Sunucu, method: &str, line: usize, character: usize) -> Value {
    let yanitlar = gonder(
        s,
        method,
        json!({"textDocument":{"uri":"untitled:k287"},
        "position":{"line":line,"character":character},"newName":"ad"}),
    );
    assert_eq!(yanitlar.len(), 1);
    let yanit = &yanitlar[0];
    assert_eq!(yanit["id"], 9);
    assert!(yanit.get("error").is_none(), "{yanit}");
    yanit.get("result").expect("result alanı bulunmalı").clone()
}

#[test]
fn utf16_hover_definition_ve_rename_imleci_bmp_disini_sayar() {
    for eol in ["\n", "\r\n"] {
        let mut s = Sunucu::yeni();
        ac(
            &mut s,
            &format!("isim \"Ayşe\" olsun{eol}\"😀\" ile isim yaz{eol}"),
        );
        assert!(
            konum_istegi(&mut s, "textDocument/hover", 1, 12)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("isim")
        );
        assert_eq!(
            konum_istegi(&mut s, "textDocument/definition", 1, 12)["range"],
            json!({"start":{"line":0,"character":0},"end":{"line":0,"character":4}})
        );
        let kullanim = konum_istegi(&mut s, "textDocument/rename", 1, 12);
        let tanim = konum_istegi(&mut s, "textDocument/rename", 0, 2);
        assert!(!kullanim.is_null());
        assert_eq!(kullanim, tanim);
    }
}

#[test]
fn utf16_rename_araliklari_bagimsiz_uygulamada_ayni_kaynagi_degistirir() {
    for eol in ["\n", "\r\n"] {
        let kaynak = format!("isim \"Ayşe\" olsun{eol}\"😀😀\" ile isim yaz{eol}");
        let mut s = Sunucu::yeni();
        ac(&mut s, &kaynak);
        let sonuc = konum_istegi(&mut s, "textDocument/rename", 0, 2);
        let edits = sonuc["changes"]["untitled:k287"].as_array().unwrap();
        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits[1]["range"],
            json!({"start":{"line":1,"character":11},"end":{"line":1,"character":15}})
        );
        // İstemci oracle'ı doğrudan UTF-16 vektöründe çalışır; sunucu dönüşümü paylaşılmaz.
        let mut satirlar: Vec<Vec<u16>> = kaynak
            .split('\n')
            .map(|x| x.encode_utf16().collect())
            .collect();
        for edit in edits.iter().rev() {
            let start = &edit["range"]["start"];
            let end = &edit["range"]["end"];
            assert_eq!(start["line"], end["line"]);
            let line = start["line"].as_u64().unwrap() as usize;
            let a = start["character"].as_u64().unwrap() as usize;
            let b = end["character"].as_u64().unwrap() as usize;
            satirlar[line].splice(a..b, edit["newText"].as_str().unwrap().encode_utf16());
        }
        let sonuc = satirlar
            .iter()
            .map(|x| String::from_utf16(x).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            sonuc,
            format!("ad \"Ayşe\" olsun{eol}\"😀😀\" ile ad yaz{eol}")
        );
        // Aynı WorkspaceEdit gerçek didChange yolunda da uygulanabilir.
        let changes: Vec<Value> = edits
            .iter()
            .rev()
            .map(|x| json!({"range":x["range"],"text":x["newText"]}))
            .collect();
        let out = degistir(&mut s, json!(changes));
        assert!(out.iter().any(|x| x["params"]["diagnostics"] == json!([])));
        assert!(
            konum_istegi(&mut s, "textDocument/hover", 1, 12)["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("ad")
        );
    }
}

#[test]
fn utf16_tani_bmp_disi_on_ekinden_sonra_exact_araligi_yayimlar() {
    for eol in ["\n", "\r\n"] {
        let mut s = Sunucu::yeni();
        let kaynak = format!("\"😀😀\" ile eksik yaz{eol}");
        let out = gonder(
            &mut s,
            "textDocument/didOpen",
            json!({"textDocument":{"uri":"untitled:k287","text":kaynak}}),
        );
        let tanilar = out
            .iter()
            .find_map(|x| x["params"]["diagnostics"].as_array())
            .unwrap();
        assert_eq!(tanilar.len(), 1, "{tanilar:?}");
        let tani = &tanilar[0];
        let core = dil::kaynagi_derle(&kaynak).unwrap_err();
        assert_eq!(
            (core.satir, core.sutun, core.uzunluk),
            (1, 10, 5),
            "{core:?}"
        );
        assert_eq!(tani["code"], core.kod);
        assert_eq!(
            tani["range"],
            json!({"start":{"line":0,"character":11},"end":{"line":0,"character":16}})
        );
    }
}

#[test]
fn utf16_sorgu_surrogate_ortasi_ve_belge_disi_konumda_tahmin_yapmaz() {
    let mut s = Sunucu::yeni();
    ac(&mut s, "isim \"Ayşe\" olsun\n\"😀\" ile isim yaz\n");
    for method in [
        "textDocument/hover",
        "textDocument/definition",
        "textDocument/rename",
    ] {
        for (line, character) in [(1, 2), (1, 100), (3, 0), (2, 0)] {
            assert_eq!(
                konum_istegi(&mut s, method, line, character),
                Value::Null,
                "{method} {line}:{character}"
            );
        }
    }
}

#[path = "semantic_regresyon/lsp_utf16.rs"]
mod utf16_regresyon;

#[test]
fn utf16_minimal_regresyon_hover_definition_rename_birlikte_gecer() {
    let (exit, cikti) = utf16_regresyon::calistir("isim \"Ayşe\" olsun\n\"😀\" ile isim yaz\n");
    assert_eq!(exit, 0);
    assert_eq!(cikti, ["hover=true;definition=0:0-4;rename=0:0-4,1:9-13"]);
}

#[test]
fn utf16_tani_uzunlugu_bmp_disi_karakterleri_tam_kapsar() {
    let mut s = Sunucu::yeni();
    let out = gonder(
        &mut s,
        "textDocument/didOpen",
        json!({"textDocument":{"uri":"untitled:k287","text":"\"😀😀"}}),
    );
    let tanilar = out
        .iter()
        .find_map(|x| x["params"]["diagnostics"].as_array())
        .unwrap();
    assert_eq!(tanilar.len(), 1);
    assert_eq!(
        tanilar[0]["range"],
        json!({"start":{"line":0,"character":0},"end":{"line":0,"character":5}})
    );
}

#[test]
fn utf16_tek_cr_sessizce_satir_sonu_sayilmaz() {
    let mut s = Sunucu::yeni();
    let out = gonder(
        &mut s,
        "textDocument/didOpen",
        json!({"textDocument":{"uri":"untitled:k287","text":"\"😀\" yaz\r"}}),
    );
    let tanilar = out
        .iter()
        .find_map(|x| x["params"]["diagnostics"].as_array())
        .unwrap();
    assert_eq!(tanilar.len(), 1);
    assert_eq!(tanilar[0]["code"], "S001");
    assert_eq!(
        tanilar[0]["range"],
        json!({"start":{"line":0,"character":8},"end":{"line":0,"character":9}})
    );
    // CR içerik olarak sayılır, LF olmadan satır sonuna dönüştürülmez.
    let out = degistir(&mut s, json!([aralik(0, 8, 9, "")]));
    assert!(out.iter().any(|x| x["params"]["diagnostics"] == json!([])));
}

#[test]
fn yol_dizini_ayni_ve_celisen_uc_alias_icin_lint_esligini_korur() {
    for lint in [false, true] {
        for farkli in 0..3 {
            let p = Proje::yeni();
            p.yaz("yardimci.dil", YARDIMCI);
            let mut s = Sunucu::yeni();
            gonder(
                &mut s,
                "initialize",
                json!({"initializationOptions":{"lint":lint}}),
            );
            let adlar = ["yardimci.dil", "%79ardimci.dil", "y%61rdimci.dil"];
            for ad in adlar {
                p.ac(&mut s, ad, "\"kapanmayan");
            }
            assert!(!p.ac(&mut s, "ana.dil", ANA).as_array().unwrap().is_empty());
            p.ac(&mut s, adlar[farkli], YARDIMCI);
            // Üç alias'tan ikisi aynı bozuk metni taşısa da çatışma kalıcıdır.
            assert_eq!(p.ac(&mut s, "ana.dil", ANA), json!([]));
            gonder(
                &mut s,
                "textDocument/didClose",
                json!({"textDocument":{"uri":p.uri(adlar[farkli])}}),
            );
            assert!(!p.ac(&mut s, "ana.dil", ANA).as_array().unwrap().is_empty());
            // Başarılı strict lint de geçerli overlay'i kullanmalı; disk bozuk.
            p.yaz("yardimci.dil", "\"diskte bozuk");
            for ad in adlar {
                p.ac(&mut s, ad, YARDIMCI);
            }
            let tanilar = p.ac(&mut s, "ana.dil", &format!("{ANA}bos 1 olsun\n"));
            let kodlar: Vec<_> = tanilar
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t["code"].as_str().unwrap())
                .collect();
            assert_eq!(kodlar, if lint { vec!["A012"] } else { vec![] });
        }
    }
}

#[cfg(unix)]
#[test]
fn yol_dizini_sonraki_bildirimde_degisen_sembolik_yolu_yeniden_cozer() {
    for lint in [false, true] {
        let p = Proje::yeni();
        p.yaz("hedef_a.dil", YARDIMCI);
        p.yaz("hedef_b.dil", YARDIMCI);
        let yol = p.0.join("yardimci.dil");
        std::os::unix::fs::symlink(p.0.join("hedef_a.dil"), &yol).unwrap();
        let mut s = Sunucu::yeni();
        gonder(
            &mut s,
            "initialize",
            json!({"initializationOptions":{"lint":lint}}),
        );
        p.ac(&mut s, "yardimci.dil", "\"kapanmayan");
        assert!(!p.ac(&mut s, "ana.dil", ANA).as_array().unwrap().is_empty());
        std::fs::remove_file(&yol).unwrap();
        std::os::unix::fs::symlink(p.0.join("hedef_b.dil"), &yol).unwrap();
        // Aynı açık URI yeni gerçek yola bağlanır; eski dizin saklanamaz.
        assert!(!p.ac(&mut s, "ana.dil", ANA).as_array().unwrap().is_empty());
        gonder(
            &mut s,
            "textDocument/didClose",
            json!({"textDocument":{"uri":p.uri("yardimci.dil")}}),
        );
        assert_eq!(p.ac(&mut s, "ana.dil", ANA), json!([]));
    }
}
