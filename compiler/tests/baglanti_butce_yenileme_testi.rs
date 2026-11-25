//! K-388 (ZM-F035, spec/12 §9, spec/24): bağlantı görevinde çalışma bütçesi
//! protokol birimi sınırında yenilenir. Sınır, girdi tüketen ya da süresini
//! dolduran her okuma dönüşüdür (başarı, `zaman_aşımı`, `satır_uzun`,
//! `satır_sonu`, `kodlama`, nokta bloğunda `boyut_aşımı`). Anında dönen
//! `kapandı` ve okumadan dönen `boyut_aşımı` sınır değildir; tek birimdeki
//! 64 MiB zarf aynen geçerlidir. K-393: birim içinde düşen değer iade edilir,
//! canlı birikim zarfı tüketir.

use dil::yorumlayici::{calistir_io_kodla, SahteBaglanti, ToplayanIo};

/// Her birim ~22 MiB canlı değer üretir ve bağlantı boyunca tutar
/// (`tutulan`). K-393'ten beri atılan değer zarfa iade edildiğinden yenileme
/// geçici çöple değil canlı birikimle sınanır: iki birim tek bütçeyle 64 MiB
/// zarfını aşar, birim başına yenilenen bütçeyle aşmaz (birim sınırını aşan
/// değer yeni zarfa yeniden sayılmaz — K-381 sınırı).
const CALKALA: &str = r#"
işlem çalkala
    adı al
    blok "0123456789" olsun
    7 kez tekrarla
        blok blok ile blok olsun
    parçalar boş liste olsun
    parçalara ad ekle
    17000 kez tekrarla
        parçalara blok ekle
    parçaları döndür
"#;

fn calistir(govde: &str, io: &mut ToplayanIo) {
    let kaynak = format!("{CALKALA}\n{govde}");
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    calistir_io_kodla(&program, io).expect("çalışmalı");
}

fn gunluk(io: &ToplayanIo) -> Vec<String> {
    io.baglanti.gunluk.iter().map(|(_, n)| n.clone()).collect()
}

fn c024_var(io: &ToplayanIo) -> bool {
    gunluk(io).iter().any(|n| n.starts_with("C024"))
}

#[test]
fn idle_yoklamasi_zaman_asiminda_butceyi_yeniler() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let mut b = SahteBaglanti::satirlarla("imap_v4", "192.0.2.24", &["A1 IDLE"]);
    b.sessiz_kalir = true;
    io.baglanti.bekleyenler.push_back(b);
    calistir(
        r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    tutulan boş liste olsun
    ilk bağlantıdan satır okumayı dene olsun
    4 kez tekrarla
        yoklama bağlantıdan satır okumayı 1 saniye içinde dene olsun
        t "x" için çalkala olsun
        tutulana t ekle
    bağlantıya "A1 OK bitti" satırını yaz
"#,
        &mut io,
    );
    assert_eq!(gunluk(&io), Vec::<String>::new());
    assert_eq!(io.baglanti.baglantilar[0].giden, vec!["A1 OK bitti"]);
}

#[test]
fn girdi_tuketen_ret_birim_sinirdir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.satir_siniri = 64;
    let uzun = "u".repeat(200);
    let satirlar = [uzun.as_str(), uzun.as_str(), uzun.as_str(), "QUIT"];
    io.baglanti.bekleyenler.push_back(SahteBaglanti::satirlarla(
        "smtp_v4",
        "192.0.2.10",
        &satirlar,
    ));
    calistir(
        r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    tutulan boş liste olsun
    açık doğru olsun
    açık olduğu sürece
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            açık yanlış olsun
        değilse
            t "x" için çalkala olsun
            tutulana t ekle
            bağlantıya "500 uzun" satırını yaz
    bağlantıya "221 hoşça kal" satırını yaz
"#,
        &mut io,
    );
    assert_eq!(gunluk(&io), Vec::<String>::new());
    assert_eq!(
        io.baglanti.baglantilar[0].giden,
        vec!["500 uzun", "500 uzun", "500 uzun", "221 hoşça kal"]
    );
}

#[test]
fn cok_komutlu_oturum_basarili_okumada_yenilenir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.bekleyenler.push_back(SahteBaglanti::satirlarla(
        "smtp_v4",
        "192.0.2.10",
        &["NOOP", "NOOP", "NOOP", "NOOP"],
    ));
    calistir(
        r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    tutulan boş liste olsun
    açık doğru olsun
    açık olduğu sürece
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            t "x" için çalkala olsun
            tutulana t ekle
            bağlantıya "250 tamam" satırını yaz
        değilse
            açık yanlış olsun
"#,
        &mut io,
    );
    assert_eq!(gunluk(&io), Vec::<String>::new());
    assert_eq!(io.baglanti.baglantilar[0].giden.len(), 4);
}

#[test]
fn tek_birimde_zarf_asimi_yine_c024() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.bekleyenler.push_back(SahteBaglanti::satirlarla(
        "smtp_v4",
        "192.0.2.10",
        &["DATA"],
    ));
    calistir(
        r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    tutulan boş liste olsun
    komut bağlantıdan satır okumayı dene olsun
    3 kez tekrarla
        t "x" için çalkala olsun
        tutulana t ekle
    bağlantıya "250 tamam" satırını yaz
"#,
        &mut io,
    );
    assert!(c024_var(&io), "{:?}", gunluk(&io));
    assert!(io.baglanti.baglantilar[0].giden.is_empty());
    assert!(io.baglanti.baglantilar[0].kapandi);
}

#[test]
fn kapandi_donusu_butceyi_yenilemez() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla("smtp_v4", "192.0.2.10", &[]));
    calistir(
        r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    tutulan boş liste olsun
    3 kez tekrarla
        komut bağlantıdan satır okumayı dene olsun
        t "x" için çalkala olsun
        tutulana t ekle
    bağlantıya "yanlış: kapandı döngüsü bitti" satırını yaz
"#,
        &mut io,
    );
    assert!(c024_var(&io), "{:?}", gunluk(&io));
    assert!(io.baglanti.baglantilar[0].giden.is_empty());
}

#[test]
fn okumadan_donen_boyut_asimi_butceyi_yenilemez() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.blok_siniri = 1024;
    let mut b = SahteBaglanti::satirlarla("imap_v4", "192.0.2.24", &["A1 APPEND"]);
    b.sessiz_kalir = true;
    io.baglanti.bekleyenler.push_back(b);
    calistir(
        r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    tutulan boş liste olsun
    ilk bağlantıdan satır okumayı dene olsun
    3 kez tekrarla
        ek bağlantıdan 4096 baytı dosyaya almayı dene olsun
        t "x" için çalkala olsun
        tutulana t ekle
    bağlantıya "yanlış: boyut döngüsü bitti" satırını yaz
"#,
        &mut io,
    );
    assert!(c024_var(&io), "{:?}", gunluk(&io));
    assert!(io.baglanti.baglantilar[0].giden.is_empty());
}
