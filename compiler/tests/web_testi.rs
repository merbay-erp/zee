//! Web dalgası (K-051): örtük "istek" sözlüğü (sorgu + form), yönlendirme,
//! html güvenlisi. Hepsi hermetik — sahte istek kuyruğuyla.

use dil::agac::RotaErisimi;
use dil::yorumlayici::{calistir_io, istek_parcala, EylemHatasi, GirdiCikti, ToplayanIo};

fn sunucuyla(kaynak: &str, istekler: Vec<&str>) -> ToplayanIo {
    let csrf_gerekli = istekler.iter().any(|istek| unsafe_istek(istek));
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    let (oturum, csrf) = if csrf_gerekli {
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
        (oturum, csrf)
    } else {
        (String::new(), String::new())
    };
    let mut kuyruk = Vec::new();
    kuyruk.extend(istekler.into_iter().map(|istek| {
        if unsafe_istek(istek) {
            csrfli_istek_ile(istek, &oturum, &csrf)
        } else {
            istek.to_string()
        }
    }));
    io.istekler = kuyruk.into();
    calistir_io(&program, &mut io).expect("çalışmalı");
    io
}

fn csrfli_bos_io() -> (ToplayanIo, String, String) {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler.push_back("GET /__test-csrf".into());
    assert!(io.istek_al().is_some());
    let csrf = io.csrf_belirteci().expect("CSRF oturumu");
    let (_, oturum) = io.yazilan_cerezler[0].clone();
    io.yanit_gonder("");
    io.istek_islemini_tamamla().expect("oturum commit'i");
    io.sunucu_yanitlari.clear();
    io.sunucu_durumlari.clear();
    io.yazilan_cerezler.clear();
    io.guvenli_cerezler.clear();
    (io, oturum, csrf)
}

#[test]
fn eylem_transaction_hatasi_503_olur_ve_sonraki_istekler_calisir() {
    let kaynak = r#"
8080 kapısında sunucu başlat

eylem kaydet
    değer döndürmez
    "sayac.txt" dosyasına "eylem-gövdesi" ekle

POST "/kaydet" adresine istek geldiğinde
    herkese açık
    kaydet
    "kaydedildi" yanıtını gönder

GET "/saglik" adresine istek geldiğinde
    "ayakta" yanıtını gönder
"#;
    let program = dil::kaynagi_derle(kaynak).expect("web programı derlenmeli");
    let (mut io, oturum, csrf) = csrfli_bos_io();

    let post = csrfli_istek_ile("POST /kaydet", &oturum, &csrf);
    io.istekler.push_back(post.clone());
    io.istekler.push_back("GET /saglik".into());
    io.istekler.push_back(post);
    io.eylem_baslat_sonuclari
        .push_back(Err("bağlantı kaybedildi".into()));

    calistir_io(&program, &mut io).expect("worker transaction hatasından sonra yaşamalı");
    assert_eq!(io.sunucu_durumlari, [503, 200, 200]);
    assert_eq!(
        io.sunucu_yanitlari[0].1,
        "işlem tamamlanamadı; otomatik tekrar yok"
    );
    assert_eq!(io.sunucu_yanitlari[1].1, "ayakta");
    assert_eq!(io.sunucu_yanitlari[2].1, "kaydedildi");
    assert_eq!(
        io.dosyalar.get("sayac.txt").map(String::as_str),
        Some("eylem-gövdesi\n")
    );
}

#[test]
fn dosya_yazma_hatasi_503_olur_ve_sonraki_istekler_calisir() {
    let kaynak = r#"
8080 kapısında sunucu başlat

eylem kaydet
    değer döndürmez
    "medya.txt" dosyasına "içerik" yaz

POST "/kaydet" adresine istek geldiğinde
    herkese açık
    kaydet
    "kaydedildi" yanıtını gönder

GET "/saglik" adresine istek geldiğinde
    "ayakta" yanıtını gönder
"#;
    let program = dil::kaynagi_derle(kaynak).expect("web programı derlenmeli");
    let (mut io, oturum, csrf) = csrfli_bos_io();

    let post = csrfli_istek_ile("POST /kaydet", &oturum, &csrf);
    io.istekler.push_back(post.clone());
    io.istekler.push_back("GET /saglik".into());
    io.istekler.push_back(post);
    io.dosya_yaz_sonuclari
        .push_back(Err("disk kullanılamıyor".into()));

    calistir_io(&program, &mut io).expect("worker dosya hatasından sonra yaşamalı");
    assert_eq!(io.sunucu_durumlari, [503, 200, 200]);
    assert_eq!(
        io.sunucu_yanitlari[0].1,
        "dosya kaydı tamamlanamadı; otomatik tekrar yok"
    );
    assert_eq!(io.sunucu_yanitlari[1].1, "ayakta");
    assert_eq!(io.sunucu_yanitlari[2].1, "kaydedildi");
    assert_eq!(
        io.dosyalar.get("medya.txt").map(String::as_str),
        Some("içerik\n"),
        "başarısız istek otomatik tekrarlanmamalı; yalnız sonraki bağımsız istek yazmalı"
    );
}

#[test]
fn commit_sonucu_belirsizse_retry_yapilmaz_worker_ve_sonraki_istek_yasar() {
    let kaynak = r#"
8080 kapısında sunucu başlat

eylem kaydet
    değer döndürmez
    "sayac.txt" dosyasına "eylem-gövdesi" ekle

POST "/kaydet" adresine istek geldiğinde
    herkese açık
    kaydet
    "kaydedildi" yanıtını gönder

GET "/saglik" adresine istek geldiğinde
    "ayakta" yanıtını gönder
"#;
    let program = dil::kaynagi_derle(kaynak).expect("web programı derlenmeli");
    let (mut io, oturum, csrf) = csrfli_bos_io();

    let post = csrfli_istek_ile("POST /kaydet", &oturum, &csrf);
    io.istekler.push_back(post.clone());
    io.istekler.push_back("GET /saglik".into());
    io.istekler.push_back(post);
    io.eylem_tamamla_sonuclari
        .push_back(Err(EylemHatasi::commit_sonucu_belirsiz(
            "COMMIT cevabı alınamadı",
        )));

    calistir_io(&program, &mut io).expect("worker belirsiz COMMIT sonucundan sonra yaşamalı");
    assert_eq!(io.sunucu_durumlari, [503, 200, 200]);
    assert_eq!(
        io.sunucu_yanitlari[0].1,
        "işlem sonucu belirsiz; otomatik tekrar yok; uzlaştırma gerekli"
    );
    assert_eq!(io.sunucu_yanitlari[1].1, "ayakta");
    assert_eq!(io.sunucu_yanitlari[2].1, "kaydedildi");
    assert_eq!(
        io.dosyalar.get("sayac.txt").map(String::as_str),
        Some("eylem-gövdesi\n"),
        "ilk eylem otomatik tekrarlanmamalı, ikinci bağımsız eylem tam bir kez yazmalı"
    );
}

#[test]
fn commit_sonucu_belirsiz_cli_yolunda_c027_olur() {
    let kaynak = r#"
eylem kaydet
    değer döndürmez
    "sayac.txt" dosyasına "bir" ekle

kaydet
"#;
    let program = dil::kaynagi_derle(kaynak).expect("program derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.eylem_tamamla_sonuclari
        .push_back(Err(EylemHatasi::commit_sonucu_belirsiz(
            "COMMIT cevabı alınamadı",
        )));

    let tani = calistir_io(&program, &mut io).expect_err("belirsizlik yayılmalı");
    assert_eq!(tani.kod, "C027");
    assert!(tani.mesaj.contains("COMMIT cevabı alınamadı"));
    assert!(!io.dosyalar.contains_key("sayac.txt"));
}

fn unsafe_istek(istek: &str) -> bool {
    matches!(
        istek.split_whitespace().next(),
        Some("POST" | "PUT" | "PATCH" | "DELETE")
    )
}

/// ToplayanIo'nun hermetik belirteç üreticisi ilk anonim oturumda 1'i
/// oturum kimliği, 2'yi CSRF olarak 64 haneli hex'e çevirir.
fn csrfli_istek(istek: &str) -> String {
    let oturum = format!("{:064x}", 1);
    let csrf = format!("{:064x}", 2);
    csrfli_istek_ile(istek, &oturum, &csrf)
}

fn csrfli_istek_ile(istek: &str, oturum: &str, csrf: &str) -> String {
    let mut satirlar = istek.lines();
    let ilk = satirlar.next().unwrap_or(istek);
    let mut cerezler = Vec::new();
    let mut govde = Vec::new();
    for satir in satirlar {
        if let Some(cerez) = satir.strip_prefix("çerez ") {
            cerezler.push(cerez.to_string());
        } else {
            govde.push(satir.to_string());
        }
    }
    cerezler.push(format!("__Host-zee-oturum={}", oturum));
    let mut govde = govde.join("\n");
    if !govde.is_empty() {
        govde.push('&');
    }
    govde.push_str(&format!("_csrf={}", csrf));
    format!("{}\nçerez {}\n{}", ilk, cerezler.join("; "), govde)
}

#[test]
fn istek_parcala_sorgu_ve_govde() {
    let (yontem, yol, veriler) =
        istek_parcala("POST /kaydet?k=1\nad=Zeynep+Eliz&not=%C3%A7ok%20iyi")
            .expect("geçerli form çözülmeli");
    assert_eq!((yontem.as_str(), yol.as_str()), ("POST", "/kaydet"));
    let bul = |a: &str| {
        veriler
            .iter()
            .find(|(ad, _)| ad == a)
            .map(|(_, d)| d.as_str())
    };
    assert_eq!(bul("k"), Some("1"));
    assert_eq!(bul("ad"), Some("Zeynep Eliz"));
    assert_eq!(bul("not"), Some("çok iyi"));
    assert_eq!(bul("yöntem"), Some("POST"));
    assert_eq!(bul("yol"), Some("/kaydet"));

    let (yontem, yol, _) = istek_parcala("/durum").expect("yalın GET çözülmeli");
    assert_eq!((yontem.as_str(), yol.as_str()), ("GET", "/durum"));
}

#[test]
fn bozuk_yuzde_kodlamasi_ve_utf8_formu_400_ile_reddedilir() {
    for istek in [
        "GET /form?ad=%",
        "GET /form?ad=%A",
        "GET /form?ad=%GG",
        "GET /form?%GG=deger",
        "GET /form?ad=%C3%28",
        "GET /form?ad=%FF",
        "POST /form\nad=%GG",
    ] {
        let (durum, _) = istek_parcala(istek).expect_err("bozuk form kabul edilmemeli");
        assert_eq!(durum, 400, "{istek}");
    }

    let kaynak = "\
8080 kapısında sunucu başlat
GET \"/form\" adresine istek geldiğinde
    \"rota çalıştı\" yanıtını gönder
";
    let io = sunucuyla(kaynak, vec!["GET /form?ad=%GG"]);
    assert_eq!(io.sunucu_durumlari, vec![400]);
    assert_eq!(
        io.sunucu_yanitlari[0].1,
        "istek formunda geçersiz yüzde kodlaması"
    );
}

#[test]
fn toplayan_io_anonim_oturumu_sonraki_istege_tasir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler.push_back("GET /form".into());
    assert!(io.istek_al().is_some());
    let csrf = io.csrf_belirteci().expect("CSRF");
    let (_, oturum) = io.yazilan_cerezler[0].clone();
    io.yanit_gonder("");
    io.istek_islemini_tamamla().unwrap();
    io.istekler.push_back(format!(
        "POST /kaydet\nçerez __Host-zee-oturum={}\n_csrf={}",
        oturum, csrf
    ));
    assert!(io.istek_al().is_some());
    assert!(io
        .rota_guvenligini_denetle(&RotaErisimi::HerkeseAcik, Some(&csrf), true)
        .is_ok());
    io.istek_islemini_tamamla().unwrap();
}

#[test]
fn csrf_form_rotasindan_durum_degistiren_rotaya_tasinir() {
    let kaynak = "\
8080 kapısında sunucu başlat
GET \"/form\" adresine istek geldiğinde
    csrf belirteci yanıtını gönder
POST \"/kaydet\" adresine istek geldiğinde
    herkese açık
    \"tamam\" yanıtını gönder
";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec!["GET /form".to_string(), csrfli_istek("POST /kaydet")].into();
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.sunucu_durumlari, vec![200, 200]);
    assert_eq!(io.sunucu_yanitlari[1].1, "tamam");
}

#[test]
fn unsafe_rota_politikasiz_derlenmez_ve_onsoz_sirasi_sabittir() {
    let politikasiz = "\
POST \"/kaydet\" adresine istek geldiğinde
    \"tamam\" yanıtını gönder
";
    assert_eq!(
        dil::kaynagi_derle(politikasiz).expect_err("T049").kod,
        "T049"
    );

    let gec = "\
POST \"/kaydet\" adresine istek geldiğinde
    \"hazır\" yaz
    herkese açık
    \"tamam\" yanıtını gönder
";
    assert_eq!(dil::kaynagi_derle(gec).expect_err("T050").kod, "T050");
}

#[test]
fn csrf_ve_zorunlu_alan_kapilari_403_400_ve_200_ayirir() {
    let kaynak = "\
8080 kapısında sunucu başlat
POST \"/kaydet\" adresine istek geldiğinde
    herkese açık
    \"ad\" alanı gerekli
    \"tamam\" yanıtını gönder
";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler.push_back("GET /form".into());
    io.istek_al();
    let csrf = io.csrf_belirteci().expect("csrf");
    let (_, oturum) = io.yazilan_cerezler[0].clone();
    io.yanit_gonder("");
    io.istek_islemini_tamamla().unwrap();
    io.sunucu_yanitlari.clear();
    io.sunucu_durumlari.clear();
    io.istekler = vec![
        "POST /kaydet\nad=Zeynep".to_string(),
        format!(
            "POST /kaydet\nçerez __Host-zee-oturum={}\nad=Zeynep",
            oturum
        ),
        csrfli_istek_ile("POST /kaydet\nad=Zeynep", &oturum, "sahte"),
        csrfli_istek_ile("POST /kaydet", &oturum, &csrf),
        csrfli_istek_ile("POST /kaydet\nad=Zeynep", &oturum, &csrf),
    ]
    .into();
    calistir_io(&program, &mut io).expect("sunucu sürmeli");
    assert_eq!(io.sunucu_durumlari, vec![403, 403, 403, 400, 200]);
    assert!(io.sunucu_yanitlari[3].1.contains("zorunlu istek alanı"));
    assert_eq!(io.sunucu_yanitlari[4].1, "tamam");
}

#[test]
fn yonlendirme_baslik_enjeksiyonu_c022_ile_reddedilir() {
    let kaynak = "\
8080 kapısında sunucu başlat
GET \"/git\" adresine istek geldiğinde
    hedef isteğin \"hedef\" değeri olsun
    hedef adresine yönlendir
";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec!["GET /git?hedef=%2Fiyi%0D%0AX-Sahte%3A+evet".to_string()].into();
    let hata = calistir_io(&program, &mut io).expect_err("enjeksiyon reddedilmeli");
    assert_eq!(hata.kod, "C022");
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
    assert_eq!(
        io.sunucu_yanitlari,
        vec![("/selamla".into(), "Merhaba Zeynep".into())]
    );
}

#[test]
fn form_govdesi_post_ile_gelir() {
    let kaynak = "\
8080 kapısında sunucu başlat

POST \"/kaydet\" adresine istek geldiğinde
    herkese açık
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
    assert_eq!(
        io.sunucu_yanitlari,
        vec![("/eski".into(), "→ /yeni".into())]
    );
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
        csrfli_istek("POST /kaydet-gizli123\nnot=S%C3%BCt+al+%3Cb%3E"),
        "/".to_string(),
    ]
    .into();
    calistir_io(&program, &mut io).expect("çalışmalı");

    assert_eq!(io.sunucu_yanitlari.len(), 4);
    assert!(
        io.sunucu_yanitlari[0].1.contains("İlk notun"),
        "başlangıç notu listede olmalı"
    );
    assert!(io.sunucu_yanitlari[1].1.contains("<form method=post"));
    assert_eq!(
        io.sunucu_yanitlari[2].1, "→ /",
        "kaydet sonrası ana sayfaya yönlendirme"
    );
    let son_liste = &io.sunucu_yanitlari[3].1;
    assert!(
        son_liste.contains("Süt al &lt;b&gt;"),
        "not kaçışlanmış görünmeli: {}",
        son_liste
    );
    assert!(!son_liste.contains("<b>"), "ham HTML sızmamalı");
}

#[test]
fn girisli_panel_oturum_dongusu() {
    // Tam güvenlik akışı: 401 → form/anonim CSRF → yanlış parola →
    // girişte session rotation → rol kapısı → korumalı yazma.
    let kaynak = std::fs::read_to_string("../projeler/girisli-panel.dil").expect("okunmalı");
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec![
        "/yonet".to_string(),
        "/giris".to_string(),
        csrfli_istek("POST /giris-yap\nparola=yanlis"),
        csrfli_istek("POST /giris-yap\nparola=zee2026"),
    ]
    .into();
    calistir_io(&program, &mut io).expect("ilk tur çalışmalı");

    assert_eq!(io.sunucu_durumlari[0], 401);
    assert!(io.sunucu_yanitlari[1].1.contains("name=_csrf"));
    assert_eq!(
        io.sunucu_yanitlari[2].1, "→ /giris",
        "yanlış parola girişe atmalı"
    );
    assert_eq!(
        io.sunucu_yanitlari[3].1, "→ /yonet",
        "doğru parola yönetime almalı"
    );
    assert_eq!(io.yazilan_cerezler.len(), 2, "anonim ve giriş oturumları");
    let (cerez_adi, kimlik) = io.yazilan_cerezler[1].clone();
    assert_eq!(cerez_adi, "__Host-zee-oturum");
    assert_ne!(io.yazilan_cerezler[0].1, kimlik, "session rotation");
    let guvenli = &io.guvenli_cerezler[1];
    assert!(guvenli.secure && guvenli.http_only);
    assert_eq!(guvenli.same_site, "Lax");
    assert_eq!(guvenli.yol, "/");
    assert_eq!(guvenli.azami_omur_saniye, 30 * 60);
    let csrf = format!("{:064x}", 4);

    // Aynı sunucu tarafı oturum deposuyla ikinci istek grubu.
    io.istekler = vec![
        format!("/yonet\nçerez __Host-zee-oturum={}", kimlik),
        csrfli_istek_ile("POST /kaydet\nnot=Gizli+plan", &kimlik, &csrf),
        "/".to_string(),
        csrfli_istek_ile("POST /kaydet\nnot=Korsan", "sahte999", &csrf),
    ]
    .into();
    calistir_io(&program, &mut io).expect("ikinci tur çalışmalı");
    assert!(
        io.sunucu_yanitlari[4].1.contains("<form"),
        "geçerli çerez formu açmalı"
    );
    assert_eq!(
        io.sunucu_yanitlari[5].1, "→ /",
        "kaydet ana sayfaya dönmeli"
    );
    assert!(
        io.sunucu_yanitlari[6].1.contains("Gizli plan"),
        "not listede olmalı"
    );
    assert_eq!(io.sunucu_durumlari[7], 403, "sahte çerez reddedilmeli");

    io.istekler = vec![
        csrfli_istek_ile("POST /cikis", &kimlik, &csrf),
        format!("/yonet\nçerez __Host-zee-oturum={}", kimlik),
    ]
    .into();
    calistir_io(&program, &mut io).expect("çıkış turu");
    assert_eq!(io.sunucu_yanitlari[8].1, "→ /");
    assert_eq!(io.sunucu_durumlari[9], 401, "eski oturum iptal edilmeli");
    assert_eq!(io.guvenli_cerezler.last().unwrap().azami_omur_saniye, 0);
}

#[test]
fn onekli_rota_kuyrugu_yakalar() {
    // K-055: "/yazi/" önekli adrese... — kimlik, yol metninden çıkarılır.
    let kaynak = "\
8080 kapısında sunucu başlat

\"/yazi/\" önekli adrese istek geldiğinde
    yol isteğin \"yol\" değeri olsun
    kimlik yolun \"/yazi/\" yerine \"\" değişmişi olsun
    \"yazı no: \" ile kimlik yanıtını gönder

\"/\" adresine istek geldiğinde
    \"ana sayfa\" yanıtını gönder
";
    let io = {
        let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
        let mut io = ToplayanIo::yeni(Vec::new());
        io.istekler = vec!["/yazi/42".to_string(), "/".to_string()].into();
        calistir_io(&program, &mut io).expect("çalışmalı");
        io
    };
    assert_eq!(io.sunucu_yanitlari[0].1, "yazı no: 42");
    assert_eq!(io.sunucu_yanitlari[1].1, "ana sayfa");
}

#[test]
fn girisli_panel_not_siler() {
    // K-059 pratiği: dosya-satırı silme SAF ZEE (süz + birleştir + yaz).
    let kaynak = std::fs::read_to_string("../projeler/girisli-panel.dil").expect("okunmalı");
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec![
        "/giris".to_string(),
        csrfli_istek("POST /giris-yap\nparola=zee2026"),
    ]
    .into();
    calistir_io(&program, &mut io).expect("giriş turu");
    let (_, kimlik) = io.yazilan_cerezler[1].clone();
    let csrf = format!("{:064x}", 4);

    io.sunucu_yanitlari.clear();
    io.sunucu_durumlari.clear();
    io.istekler = vec![
        csrfli_istek_ile("POST /kaydet\nnot=Silinecek", &kimlik, &csrf),
        csrfli_istek_ile("POST /kaydet\nnot=Kalacak", &kimlik, &csrf),
        format!("/sil?not=Silinecek\nçerez oturum={}", kimlik),
        "/".to_string(),
        csrfli_istek_ile("POST /sil\nnot=Silinecek", &kimlik, &csrf),
        "/".to_string(),
    ]
    .into();
    calistir_io(&program, &mut io).expect("silme turu");
    let get_sonrasi = &io.sunucu_yanitlari[3].1;
    assert!(
        get_sonrasi.contains("Silinecek"),
        "GET durum değiştirmemeli: {}",
        get_sonrasi
    );
    let son = &io.sunucu_yanitlari[5].1;
    assert!(son.contains("Kalacak"), "{}", son);
    assert!(
        !son.contains("Silinecek"),
        "silinen not listede kalmamalı: {}",
        son
    );
}

#[test]
fn cerez_silme_kaydedilir() {
    // K-073: çıkış gerçek silme başlığı üretir.
    let kaynak = "\
8080 kapısında sunucu başlat

POST \"/cikis\" adresine istek geldiğinde
    herkese açık
    \"oturum\" çerezini sil
    \"/\" adresine yönlendir
";
    let io = sunucuyla(kaynak, vec!["POST /cikis"]);
    assert_eq!(
        io.yazilan_cerezler,
        vec![("oturum".into(), "×silindi".into())]
    );
    assert_eq!(io.sunucu_yanitlari[0].1, "→ /");
}

#[test]
fn eylem_ayni_is_kuralini_cli_ve_webden_calistirir() {
    let kaynak = "\
eylem notu kaydet
    notu Metin olarak al
    değer döndürmez
    \"notlar.txt\" dosyasına notu ekle

\"CLI notu\" ile notu kaydet
8080 kapısında sunucu başlat

POST \"/notlar\" adresine istek geldiğinde
    herkese açık
    not isteğin \"not\" değeri olsun
    not ile notu kaydet
    \"kaydedildi\" yanıtını gönder
";
    let io = sunucuyla(kaynak, vec!["POST /notlar\nnot=Web+notu"]);
    assert_eq!(
        io.dosyalar["notlar.txt"], "CLI notu\nWeb notu\n",
        "durum={:?}, yanıt={:?}",
        io.sunucu_durumlari, io.sunucu_yanitlari
    );
    assert_eq!(io.sunucu_yanitlari[0].1, "kaydedildi");
}

#[test]
fn eylem_acik_imza_ister() {
    let kaynak = "\
eylem notu kaydet
    notu al
    \"notlar.txt\" dosyasına notu ekle
";
    let hata = dil::kaynagi_derle(kaynak).expect_err("T043 bekleniyor");
    assert_eq!(hata.kod, "T043");
}

#[test]
fn get_dogrudan_ve_dolayli_yazmayi_reddeder() {
    let dogrudan = "\
GET \"/sil\" adresine istek geldiğinde
    \"durum.txt\" dosyasına \"değişti\" yaz
";
    assert_eq!(dil::kaynagi_derle(dogrudan).expect_err("T045").kod, "T045");

    let dolayli = "\
eylem durumu değiştir
    değer döndürmez
    \"durum.txt\" dosyasına \"değişti\" yaz

GET \"/sil\" adresine istek geldiğinde
    durumu değiştir
";
    assert_eq!(dil::kaynagi_derle(dolayli).expect_err("T045").kod, "T045");
}

#[test]
fn yazan_post_rotasi_mutlaka_eylem_cagirir() {
    let kaynak = "\
POST \"/notlar\" adresine istek geldiğinde
    herkese açık
    \"notlar.txt\" dosyasına \"doğrudan\" ekle
";
    assert_eq!(dil::kaynagi_derle(kaynak).expect_err("T046").kod, "T046");
}

#[test]
fn eylem_http_adaptorune_baglanamaz() {
    let kaynak = "\
eylem yanıt ver
    değer döndürmez
    \"olmaz\" yanıtını gönder
";
    assert_eq!(dil::kaynagi_derle(kaynak).expect_err("T044").kod, "T044");
}

#[test]
fn eylem_geri_alinamayan_ciktiyi_tasimaz() {
    let kaynak = "\
eylem raporu üret
    değer döndürmez
    \"yarım çıktı\" yaz
";
    assert_eq!(dil::kaynagi_derle(kaynak).expect_err("T048").kod, "T048");
}

#[test]
fn yanlis_yontem_405_olmayan_yol_404_doner() {
    let kaynak = "\
8080 kapısında sunucu başlat

POST \"/notlar\" adresine istek geldiğinde
    herkese açık
    \"tamam\" yanıtını gönder
";
    let io = sunucuyla(kaynak, vec!["GET /notlar", "GET /yok"]);
    assert_eq!(io.sunucu_durumlari, vec![405, 404]);
    assert!(io.sunucu_yanitlari[0].1.contains("HTTP yöntemini"));
    assert!(io.sunucu_yanitlari[1].1.contains("aranan sayfa yok"));
}

#[test]
fn fazla_govde_ve_alan_413_doner() {
    let kaynak = "\
8080 kapısında sunucu başlat

POST \"/al\" adresine istek geldiğinde
    herkese açık
    \"tamam\" yanıtını gönder
";
    let buyuk = format!("POST /al\nveri={}", "x".repeat(64 * 1024 + 1));
    let alanlar = (0..101)
        .map(|i| format!("a{}=1", i))
        .collect::<Vec<_>>()
        .join("&");
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec![buyuk, format!("POST /al\n{}", alanlar)].into();
    calistir_io(&program, &mut io).expect("sunucu sürmeli");
    assert_eq!(io.sunucu_durumlari, vec![413, 413]);
}

#[test]
fn put_patch_delete_govdeleri_cozulur() {
    let kaynak = "\
8080 kapısında sunucu başlat

PUT \"/put\" adresine istek geldiğinde
    herkese açık
    isteğin \"değer\" değeri yanıtını gönder
PATCH \"/patch\" adresine istek geldiğinde
    herkese açık
    isteğin \"değer\" değeri yanıtını gönder
DELETE \"/delete\" adresine istek geldiğinde
    herkese açık
    isteğin \"değer\" değeri yanıtını gönder
";
    let io = sunucuyla(
        kaynak,
        vec![
            "PUT /put\ndeğer=bir",
            "PATCH /patch\ndeğer=iki",
            "DELETE /delete\ndeğer=üç",
        ],
    );
    assert_eq!(
        io.sunucu_yanitlari
            .iter()
            .map(|(_, govde)| govde.as_str())
            .collect::<Vec<_>>(),
        vec!["bir", "iki", "üç"]
    );
}

#[test]
fn istek_son_tarihi_504_doner() {
    let kaynak = "\
8080 kapısında sunucu başlat

GET \"/yavaş\" adresine istek geldiğinde
    31 saniye bekle
    \"geç kaldı\" yanıtını gönder
";
    let io = sunucuyla(kaynak, vec!["GET /yavaş"]);
    assert_eq!(io.sunucu_durumlari, vec![504]);
    assert_eq!(
        io.sunucu_yanitlari[0].1,
        "istek 30 saniyelik son tarihini aştı"
    );
}

#[test]
fn zaman_asimi_tamponlu_yaniti_cerezi_ve_oturum_mutasyonunu_birlikte_geri_alir() {
    let kaynak = "\
8080 kapısında sunucu başlat

POST \"/gec\" adresine istek geldiğinde
    herkese açık
    \"Mustafa\" kullanıcısını \"yönetici\" rolüyle oturuma al
    \"erken başarı\" yanıtını gönder
    31 saniye bekle

GET \"/korumali\" adresine istek geldiğinde
    oturum gerekli
    \"oturum sızdı\" yanıtını gönder
";
    let sızmış_kimlik = format!("{:064x}", 3);
    let ikinci = format!("GET /korumali\nçerez __Host-zee-oturum={}", sızmış_kimlik);
    let io = sunucuyla(kaynak, vec!["POST /gec", &ikinci]);

    assert_eq!(io.sunucu_durumlari, vec![504, 401]);
    assert_eq!(
        io.sunucu_yanitlari[0].1, "istek 30 saniyelik son tarihini aştı",
        "tamponlanmış erken başarı timeout'ta yayımlanmamalı"
    );
    assert!(
        io.yazilan_cerezler.is_empty(),
        "geri alınan girişin çerezi sonraki yanıta sızmamalı"
    );
}

#[test]
fn rota_hatasi_tamponlu_yaniti_cerezi_ve_oturum_mutasyonunu_geri_alir() {
    let kaynak = "\
8080 kapısında sunucu başlat

POST \"/bozuk\" adresine istek geldiğinde
    herkese açık
    \"Mustafa\" kullanıcısını \"yönetici\" rolüyle oturuma al
    \"erken başarı\" yanıtını gönder
    sonuç 1 in 0 a bölümü olsun

GET \"/korumali\" adresine istek geldiğinde
    oturum gerekli
    \"oturum sızdı\" yanıtını gönder
";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler.push_back("GET /__test-csrf".into());
    assert!(io.istek_al().is_some());
    let csrf = io.csrf_belirteci().expect("CSRF oturumu");
    let (_, anonim) = io.yazilan_cerezler[0].clone();
    io.yanit_gonder("");
    io.istek_islemini_tamamla().unwrap();
    io.sunucu_yanitlari.clear();
    io.sunucu_durumlari.clear();
    io.yazilan_cerezler.clear();
    io.guvenli_cerezler.clear();

    io.istekler
        .push_back(csrfli_istek_ile("POST /bozuk", &anonim, &csrf));
    let hata = calistir_io(&program, &mut io).expect_err("rota hatası yayılmalı");
    assert_eq!(hata.kod, "C003");
    assert!(io.yazilan_cerezler.is_empty());
    assert!(io.sunucu_yanitlari[0].1.is_empty());

    let sızmış_kimlik = format!("{:064x}", 3);
    io.istekler.push_back(format!(
        "GET /korumali\nçerez __Host-zee-oturum={}",
        sızmış_kimlik
    ));
    calistir_io(&program, &mut io).expect("sunucu yeniden çalışmalı");
    assert_eq!(io.sunucu_durumlari[1], 401);
}

#[test]
fn eylem_calisma_hatasinda_tum_dosyalari_geri_alir() {
    let kaynak = "\
eylem bozuk kaydet
    değer döndürmez
    \"bir.txt\" dosyasına \"bir\" yaz
    \"iki.txt\" dosyasına \"iki\" yaz
    sonuç 1 in 0 a bölümü olsun

bozuk kaydet
";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    let hata = calistir_io(&program, &mut io).expect_err("çalışma hatası");
    assert_eq!(hata.kod, "C003");
    assert!(io.dosyalar.is_empty(), "yarım yazma görünmemeli");
}

#[test]
fn basarisiz_sonuc_ic_savepointi_geri_alir() {
    let kaynak = "\
eylem iç adımı dene
    Metin sonucu döndürür
    \"gunluk.txt\" dosyasına \"iç\" ekle
    başarılı yanlış olsun
    başarılı ise
        \"beklenmeyen başarı\" döndür
    değilse
        \"beklenen hata\" hatasını döndür

eylem dış adımı çalıştır
    değer döndürmez
    \"gunluk.txt\" dosyasına \"önce\" ekle
    sonuç iç adımı dene olsun
    \"gunluk.txt\" dosyasına \"sonra\" ekle

dış adımı çalıştır
";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("dış eylem tamamlanmalı");
    assert_eq!(io.dosyalar["gunluk.txt"], "önce\nsonra\n");
}

#[test]
fn ayni_yontem_ve_yol_iki_kez_baglanamaz() {
    let kaynak = "\
GET \"/x\" adresine istek geldiğinde
    \"bir\" yanıtını gönder
GET \"/x\" adresine istek geldiğinde
    \"iki\" yanıtını gönder
";
    assert_eq!(dil::kaynagi_derle(kaynak).expect_err("T047").kod, "T047");
}
