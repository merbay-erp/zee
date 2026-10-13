//! K-212 — görünüm bloğu: girintili HTML ağacı, varsayılan kaçış, bayrak öznitelik,
//! boş öğe, ham parça ve biçim tanıları.

use dil::yorumlayici::{calistir_io, ToplayanIo};

fn calistir(kaynak: &str) -> Vec<String> {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("çalışmalı");
    io.cikti
}

#[test]
fn gorunum_html_agacini_kacisli_yazar() {
    let kaynak = "\
ad \"<Ali> & \\\"Veli\\\"\" olsun
adres \"/yonet/kaydet?a=1&b=2\" olsun
zorunlu doğru olsun
gizli yanlış olsun
parca \"<b>hazır</b>\" olsun
form_html görünüm olsun
    form method \"post\" action adres class \"editor-card\"
        input type \"hidden\" name \"_csrf\" value ad
        label
            \"E-posta\"
            input type \"email\" name \"eposta\" required zorunlu hidden gizli
        p class \"muted\"
            metin ad
        ham parca
form_html yaz
";
    let cikti = calistir(kaynak);
    assert_eq!(
        cikti,
        vec![concat!(
            "<form method=\"post\" action=\"/yonet/kaydet?a=1&amp;b=2\" class=\"editor-card\">",
            "<input type=\"hidden\" name=\"_csrf\" value=\"&lt;Ali&gt; &amp; &quot;Veli&quot;\">",
            "<label>E-posta<input type=\"email\" name=\"eposta\" required></label>",
            "<p class=\"muted\">&lt;Ali&gt; &amp; &quot;Veli&quot;</p>",
            "<b>hazır</b>",
            "</form>"
        )
        .to_string()]
    );
}

#[test]
fn gorunum_islem_icinde_ve_tirnakli_oznitelik_adiyla_calisir() {
    let kaynak = "\
işlem kart
    adı Metin olarak al
    Metin döndürür

    html görünüm olsun
        a \"data-modal\" doğru href \"/x\" \"aria-label\" ad
            metin ad
    html döndür

sonuc \"Düzenle\" için kart olsun
sonuc yaz
";
    assert_eq!(
        calistir(kaynak),
        vec!["<a data-modal href=\"/x\" aria-label=\"Düzenle\">Düzenle</a>".to_string()]
    );
}

#[test]
fn gorunum_bicim_ve_tur_tanilari() {
    let tek_deger =
        dil::kaynagi_derle("x görünüm olsun\n    input type \"a\" name\n").expect_err("S046");
    assert_eq!(tek_deger.kod, "S046");
    let bos = dil::kaynagi_derle("x görünüm olsun\ny 1 olsun\n").expect_err("S007 ya da S046");
    assert!(bos.kod == "S007" || bos.kod == "S046", "{bos:?}");
    let sayi_metin = dil::kaynagi_derle("n 5 olsun\nx görünüm olsun\n    p\n        metin n\n")
        .expect_err("T055");
    assert_eq!(sayi_metin.kod, "T055");
    let cocuklu_metin =
        dil::kaynagi_derle("x görünüm olsun\n    \"a\"\n        b\n").expect_err("S046");
    assert_eq!(cocuklu_metin.kod, "S046");
    let tur_degisimi = dil::kaynagi_derle("x 3 olsun\nx görünüm olsun\n    p\n").expect_err("T002");
    assert_eq!(tur_degisimi.kod, "T002");
}

#[test]
fn gorunum_tanimi_hir_definition_ve_yeniden_atama_kaydini_korur() {
    let kaynak = "kart görünüm olsun\n    p\nkart görünüm olsun\n    b\nkart yaz\n";
    let program = dil::kaynagi_fazli_derle(kaynak).expect("HIR kurulmalı");
    program
        .invariantleri_dogrula()
        .expect("görünüm bağı geçerli");
    let kullanımlar = program.hir().sembol_kullanimlari();
    assert_eq!(kullanımlar.len(), 3);
    let kimlik = kullanımlar[0].kimlik();
    assert!(kullanımlar.iter().all(|k| k.kimlik() == kimlik));
    assert_eq!(program.hir().sembol_tanimi(kimlik).unwrap().satiri(), 1);
    assert_eq!(calistir(kaynak), ["<b></b>"]);
}
