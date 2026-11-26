//! K-403: rota gövdesindeki `yönlendir`/`yanıtını gönder` rotayı bitirir.
//! Yetki koruması fail-closed olur; sonraki cümleler (çağrılan işlem, döngü
//! ve iç bloklar dahil) koşmaz. Rota dışında yanıt akışı bitirmez.

use dil::yorumlayici::{calistir_io, GirdiCikti, ToplayanIo};

fn calistir(kaynak: &str, istekler: &[&str]) -> ToplayanIo {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = istekler.iter().map(|i| i.to_string()).collect();
    calistir_io(&program, &mut io).expect("çalışmalı");
    io
}

/// POST istekleri için hermetik anonim oturum + CSRF belirteci kurar
/// (web_testi'ndeki önyüklemeyle aynı).
fn postla(kaynak: &str, govdeler: &[&str]) -> ToplayanIo {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler.push_back("GET /__test-csrf".into());
    assert!(io.istek_al().is_some());
    let csrf = io.csrf_belirteci().expect("CSRF oturumu");
    let (_, oturum) = io.yazilan_cerezler[0].clone();
    io.yanit_gonder("");
    io.istek_islemini_tamamla().unwrap();
    io.sunucu_yanitlari.clear();
    io.sunucu_durumlari.clear();
    io.yazilan_cerezler.clear();
    io.guvenli_cerezler.clear();
    io.istekler = govdeler
        .iter()
        .map(|g| {
            let ayrac = if g.is_empty() { "" } else { "&" };
            format!("POST /sil\nçerez __Host-zee-oturum={oturum}\n{g}{ayrac}_csrf={csrf}")
        })
        .collect();
    calistir_io(&program, &mut io).expect("çalışmalı");
    io
}

#[test]
fn koruma_yonlendirmesinden_sonra_cerez_cikti_ve_ikinci_yanit_yok() {
    let kaynak = r#"
8080 kapısında sunucu başlat

POST "/sil" adresine istek geldiğinde
    herkese açık
    onay isteğin "onay" değeri olsun
    onay boşsa
        "/giris" adresine yönlendir
    "silindi" çerezine "evet" yaz
    "hassas iş" yaz
    "silindi" yanıtını gönder
"#;
    let io = postla(kaynak, &["onay=", "onay=evet"]);
    assert_eq!(io.sunucu_durumlari, [200, 200]);
    assert_eq!(io.sunucu_yanitlari[0].1, "→ /giris");
    assert_eq!(io.sunucu_yanitlari[1].1, "silindi");
    // Korumayı geçemeyen ilk istekte çerez ve çıktı yok; ikincide bir kez.
    assert_eq!(io.cikti, ["hassas iş"]);
    let silindi: Vec<_> = io
        .yazilan_cerezler
        .iter()
        .filter(|(ad, _)| ad == "silindi")
        .collect();
    assert_eq!(silindi.len(), 1, "{:?}", io.yazilan_cerezler);
}

#[test]
fn islemdeki_ve_dongudeki_yanit_rotayi_bitirir() {
    let kaynak = r#"
8080 kapısında sunucu başlat

işlem reddet
    değer döndürmez
    "yasak" yanıtını 403 durumuyla gönder

GET "/islem" adresine istek geldiğinde
    reddet
    "işlemden sonra" yaz

GET "/dongu" adresine istek geldiğinde
    3 kez tekrarla
        "turda" yaz
        "ilk" yanıtını gönder
    "döngüden sonra" yaz
"#;
    let io = calistir(kaynak, &["GET /islem", "GET /dongu"]);
    assert_eq!(io.sunucu_durumlari, [403, 200]);
    assert_eq!(io.sunucu_yanitlari[0].1, "yasak");
    assert_eq!(io.sunucu_yanitlari[1].1, "ilk");
    assert_eq!(io.cikti, ["turda"]);
}

#[test]
fn yanittan_once_olusan_hata_ve_yanitsiz_rota_degismez() {
    let kaynak = r#"
8080 kapısında sunucu başlat

GET "/hata" adresine istek geldiğinde
    girdi "abc" olsun
    sayı girdinin sayısı olsun
    sayıyı yaz
    "olmamalı" yanıtını gönder

GET "/sonra" adresine istek geldiğinde
    "ikinci" yanıtını gönder
"#;
    let io = calistir(kaynak, &["GET /hata", "GET /sonra"]);
    assert_eq!(io.sunucu_durumlari, [500, 200]);
    assert_eq!(io.sunucu_yanitlari[1].1, "ikinci");
}

#[test]
fn rota_disinda_yanit_akisi_bitirmez() {
    let kaynak = "\"üst\" yanıtını gönder\n\"sürer\" yaz\n";
    let io = calistir(kaynak, &[]);
    assert_eq!(io.cikti, ["sürer"]);
}
