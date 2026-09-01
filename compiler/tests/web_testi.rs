//! Web dalgası (K-051): örtük "istek" sözlüğü (sorgu + form), yönlendirme,
//! html güvenlisi. Hepsi hermetik — sahte istek kuyruğuyla.

use dil::yorumlayici::{calistir_io, istek_parcala, ToplayanIo};

fn sunucuyla(kaynak: &str, istekler: Vec<&str>) -> ToplayanIo {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = istekler.into_iter().map(str::to_string).collect();
    calistir_io(&program, &mut io).expect("çalışmalı");
    io
}

#[test]
fn istek_parcala_sorgu_ve_govde() {
    let (yontem, yol, veriler) = istek_parcala("POST /kaydet?k=1\nad=Zeynep+Eliz&not=%C3%A7ok%20iyi");
    assert_eq!((yontem.as_str(), yol.as_str()), ("POST", "/kaydet"));
    let bul = |a: &str| veriler.iter().find(|(ad, _)| ad == a).map(|(_, d)| d.as_str());
    assert_eq!(bul("k"), Some("1"));
    assert_eq!(bul("ad"), Some("Zeynep Eliz"));
    assert_eq!(bul("not"), Some("çok iyi"));
    assert_eq!(bul("yöntem"), Some("POST"));
    assert_eq!(bul("yol"), Some("/kaydet"));

    let (yontem, yol, _) = istek_parcala("/durum");
    assert_eq!((yontem.as_str(), yol.as_str()), ("GET", "/durum"));
}

#[test]
fn sorgu_verisi_rotada_okunur() {
    let kaynak = "\
8080 kapısında sunucu başlat

\"/selamla\" adresine istek geldiğinde
    ad isteğin \"ad\" değeri olsun
    \"Merhaba \" ile ad yanıtını gönder
";
    let io = sunucuyla(kaynak, vec!["/selamla?ad=Zeynep"]);
    assert_eq!(io.sunucu_yanitlari, vec![("/selamla".into(), "Merhaba Zeynep".into())]);
}

#[test]
fn form_govdesi_post_ile_gelir() {
    let kaynak = "\
8080 kapısında sunucu başlat

\"/kaydet\" adresine istek geldiğinde
    istekte \"not\" varsa
        \"alındı: \" ile isteğin \"not\" değeri yanıtını gönder
";
    let io = sunucuyla(kaynak, vec!["POST /kaydet\nnot=s%C3%BCt+al"]);
    assert_eq!(io.sunucu_yanitlari[0].1, "alındı: süt al");
}

#[test]
fn yonlendirme_kaydedilir() {
    let kaynak = "\
8080 kapısında sunucu başlat

\"/eski\" adresine istek geldiğinde
    \"/yeni\" adresine yönlendir
";
    let io = sunucuyla(kaynak, vec!["/eski"]);
    assert_eq!(io.sunucu_yanitlari, vec![("/eski".into(), "→ /yeni".into())]);
}

#[test]
fn html_guvenlisi_kacislar() {
    let kaynak = "\
tehlikeli \"<script>alert(1)</script> & \\\"tırnak\\\"\" olsun
tehlikelinin html güvenlisi yaz
";
    let cikti = dil::kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(
        cikti,
        vec!["&lt;script&gt;alert(1)&lt;/script&gt; &amp; &quot;tırnak&quot;"]
    );
}

#[test]
fn yonlendirme_bicim_hatasi_s041() {
    let hata = dil::kaynagi_calistir("\"/x\" yönlendir\n").expect_err("S041");
    assert_eq!(hata.kod, "S041");
}

#[test]
fn istek_sozlugu_yalniz_rotada_var() {
    // Rota dışında "istek" tanımsızdır (A001) — örtük ad sızmaz.
    let hata = dil::kaynagi_calistir("isteğin \"a\" değeri yaz\n").expect_err("A001");
    assert_eq!(hata.kod, "A001");
}

#[test]
fn panel_not_defteri_tam_dongu() {
    // GERÇEK uygulama akışı, hermetik: listele → form → kaydet → yönlendir →
    // yeni not listede (sahte dosya sistemi koşu boyunca yaşar).
    let kaynak = std::fs::read_to_string("../projeler/panel-not-defteri.dil").expect("okunmalı");
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec![
        "/".to_string(),
        "/yonet-gizli123".to_string(),
        "POST /kaydet-gizli123\nnot=S%C3%BCt+al+%3Cb%3E".to_string(),
        "/".to_string(),
    ]
    .into();
    calistir_io(&program, &mut io).expect("çalışmalı");

    assert_eq!(io.sunucu_yanitlari.len(), 4);
    assert!(io.sunucu_yanitlari[0].1.contains("İlk notun"), "başlangıç notu listede olmalı");
    assert!(io.sunucu_yanitlari[1].1.contains("<form method=post"));
    assert_eq!(io.sunucu_yanitlari[2].1, "→ /", "kaydet sonrası ana sayfaya yönlendirme");
    let son_liste = &io.sunucu_yanitlari[3].1;
    assert!(son_liste.contains("Süt al &lt;b&gt;"), "not kaçışlanmış görünmeli: {}", son_liste);
    assert!(!son_liste.contains("<b>"), "ham HTML sızmamalı");
}

#[test]
fn girisli_panel_oturum_dongusu() {
    // Tam güvenlik akışı, hermetik: çerezsiz yönet → girişe; yanlış parola →
    // girişe; doğru parola → çerez + yönet; çerezle kaydet → not listede.
    let kaynak = std::fs::read_to_string("../projeler/girisli-panel.dil").expect("okunmalı");
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec![
        "/yonet".to_string(),
        "POST /giris-yap\nparola=yanlis".to_string(),
        "POST /giris-yap\nparola=zee2026".to_string(),
    ]
    .into();
    calistir_io(&program, &mut io).expect("ilk tur çalışmalı");

    assert_eq!(io.sunucu_yanitlari[0].1, "→ /giris", "çerezsiz yönetim girişe atmalı");
    assert_eq!(io.sunucu_yanitlari[1].1, "→ /giris", "yanlış parola girişe atmalı");
    assert_eq!(io.sunucu_yanitlari[2].1, "→ /yonet", "doğru parola yönetime almalı");
    assert_eq!(io.yazilan_cerezler.len(), 1, "oturum çerezi yazılmalı");
    let (cerez_adi, kimlik) = io.yazilan_cerezler[0].clone();
    assert_eq!(cerez_adi, "oturum");

    // İkinci tur: aynı sahte dünyada (dosyalar taşınır) çerezle korumalı işlemler.
    let mut io2 = ToplayanIo::yeni(Vec::new());
    io2.dosyalar = io.dosyalar.clone();
    io2.istekler = vec![
        format!("/yonet\nçerez oturum={}", kimlik),
        format!("POST /kaydet\nçerez oturum={}\nnot=Gizli+plan", kimlik),
        "/".to_string(),
        "POST /kaydet\nçerez oturum=sahte999\nnot=Korsan".to_string(),
    ]
    .into();
    let _ = kimlik;
    calistir_io(&program, &mut io2).expect("ikinci tur çalışmalı");
    assert!(io2.sunucu_yanitlari[0].1.contains("<form"), "geçerli çerez formu açmalı");
    assert_eq!(io2.sunucu_yanitlari[1].1, "→ /", "kaydet ana sayfaya dönmeli");
    assert!(io2.sunucu_yanitlari[2].1.contains("Gizli plan"), "not listede olmalı");
    assert_eq!(io2.sunucu_yanitlari[3].1, "→ /giris", "sahte çerez reddedilmeli");
}
