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
    let (yontem, yol, veriler) =
        istek_parcala("POST /kaydet?k=1\nad=Zeynep+Eliz&not=%C3%A7ok%20iyi");
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
        "POST /kaydet-gizli123\nnot=S%C3%BCt+al+%3Cb%3E".to_string(),
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

    assert_eq!(
        io.sunucu_yanitlari[0].1, "→ /giris",
        "çerezsiz yönetim girişe atmalı"
    );
    assert_eq!(
        io.sunucu_yanitlari[1].1, "→ /giris",
        "yanlış parola girişe atmalı"
    );
    assert_eq!(
        io.sunucu_yanitlari[2].1, "→ /yonet",
        "doğru parola yönetime almalı"
    );
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
    assert!(
        io2.sunucu_yanitlari[0].1.contains("<form"),
        "geçerli çerez formu açmalı"
    );
    assert_eq!(
        io2.sunucu_yanitlari[1].1, "→ /",
        "kaydet ana sayfaya dönmeli"
    );
    assert!(
        io2.sunucu_yanitlari[2].1.contains("Gizli plan"),
        "not listede olmalı"
    );
    assert_eq!(
        io2.sunucu_yanitlari[3].1, "→ /giris",
        "sahte çerez reddedilmeli"
    );
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
    io.istekler = vec!["POST /giris-yap\nparola=zee2026".to_string()].into();
    calistir_io(&program, &mut io).expect("giriş turu");
    let (_, kimlik) = io.yazilan_cerezler[0].clone();

    let mut io2 = ToplayanIo::yeni(Vec::new());
    io2.dosyalar = io.dosyalar.clone();
    io2.istekler = vec![
        format!("POST /kaydet\nçerez oturum={}\nnot=Silinecek", kimlik),
        format!("POST /kaydet\nçerez oturum={}\nnot=Kalacak", kimlik),
        format!("/sil?not=Silinecek\nçerez oturum={}", kimlik),
        "/".to_string(),
        format!("POST /sil\nçerez oturum={}\nnot=Silinecek", kimlik),
        "/".to_string(),
    ]
    .into();
    calistir_io(&program, &mut io2).expect("silme turu");
    let get_sonrasi = &io2.sunucu_yanitlari[3].1;
    assert!(
        get_sonrasi.contains("Silinecek"),
        "GET durum değiştirmemeli: {}",
        get_sonrasi
    );
    let son = &io2.sunucu_yanitlari[5].1;
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
    not isteğin \"not\" değeri olsun
    not ile notu kaydet
    \"kaydedildi\" yanıtını gönder
";
    let program = dil::kaynagi_derle(kaynak).expect("eylem derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec!["POST /notlar\nnot=Web+notu".to_string()].into();
    calistir_io(&program, &mut io).expect("iki adaptör de çalışmalı");
    assert_eq!(io.dosyalar["notlar.txt"], "CLI notu\nWeb notu\n");
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
    isteğin \"değer\" değeri yanıtını gönder
PATCH \"/patch\" adresine istek geldiğinde
    isteğin \"değer\" değeri yanıtını gönder
DELETE \"/delete\" adresine istek geldiğinde
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
    assert_eq!(io.sunucu_yanitlari[0].1, "istek 30 saniyelik son tarihini aştı");
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
