//! K-381 (RFC-0039 A2a): adlı dinleyici bağlantı yüzeyi — hermetik adaptörle
//! satır akışı, sınırlar, eşzamanlı oturum sırası, son tarih, yalıtım ve IO izi.

use dil::yorumlayici::{
    calistir_io, calistir_io_kodla, IzKaydedenIo, IzYenidenOynatici, SahteBaglanti, ToplayanIo,
};

const YANKI: &str = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    bağlantıya "220 hazır" satırını yaz
    açık doğru olsun
    açık olduğu sürece
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            metin komutun değeri olsun
            metin "QUIT" e eşitse
                bağlantıya "221 hoşça kal" satırını yaz
                açık yanlış olsun
            değilse
                bağlantıya "250 " ile metin satırını yaz
        değilse
            hata komutun hatası olsun
            veri hatanın verisi olsun
            tür verinin "tür" değeri olsun
            tür "kapandı" e eşitse
                açık yanlış olsun
            değilse
                bağlantıya "500 " ile tür satırını yaz
"#;

fn calistir(kaynak: &str, io: &mut ToplayanIo) -> i64 {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    calistir_io_kodla(&program, io).expect("çalışmalı")
}

fn giden(io: &ToplayanIo, sira: usize) -> Vec<String> {
    io.baglanti.baglantilar[sira].giden.clone()
}

#[test]
fn satirlar_okunur_yanitlanir_ve_quit_kapatir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.bekleyenler.push_back(SahteBaglanti::satirlarla(
        "smtp_v4",
        "192.0.2.10",
        &["EHLO istemci.test", "NOOP", "QUIT", "SONRA"],
    ));
    assert_eq!(calistir(YANKI, &mut io), 0);
    assert_eq!(
        giden(&io, 0),
        vec![
            "220 hazır",
            "250 EHLO istemci.test",
            "250 NOOP",
            "221 hoşça kal"
        ]
    );
    assert!(io.baglanti.baglantilar[0].kapandi);
}

#[test]
fn ciplak_lf_cr_uzun_satir_ve_kodlama_siniflandirilir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.satir_siniri = 64;
    let uzun = format!("{}\r\n", "A".repeat(80));
    io.baglanti.bekleyenler.push_back(SahteBaglanti::yeni(
        "smtp_v4",
        "192.0.2.11",
        &[
            b"MAIL FROM:<a@b>\n",
            b"x\r.\rRCPT\r\n",
            uzun.as_bytes(),
            b"\xff\r\n",
            b"QUIT\r\n",
        ],
    ));
    calistir(YANKI, &mut io);
    assert_eq!(
        giden(&io, 0),
        vec![
            "220 hazır",
            "500 satır_sonu",
            "500 satır_sonu",
            "500 satır_uzun",
            "500 kodlama",
            "221 hoşça kal"
        ]
    );
}

#[test]
fn eszamanli_oturumlar_parca_parca_ilerler() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let kaynak = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    ad bağlantının "uzak_adres" değeri olsun
    3 kez tekrarla
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            ad ile " " ile komutun değeri yaz
"#;
    for (uzak, onek) in [("192.0.2.1", "a"), ("192.0.2.2", "b")] {
        let satirlar: Vec<String> = (1..=3).map(|i| format!("{onek}{i}")).collect();
        let satirlar: Vec<&str> = satirlar.iter().map(String::as_str).collect();
        io.baglanti
            .bekleyenler
            .push_back(SahteBaglanti::satirlarla("smtp_v4", uzak, &satirlar));
    }
    calistir(kaynak, &mut io);
    // Bir bağlantının beklemesi diğerini durdurmaz: satırlar sırayla iç içe.
    assert_eq!(
        io.cikti,
        vec![
            "192.0.2.1 a1",
            "192.0.2.2 b1",
            "192.0.2.1 a2",
            "192.0.2.2 b2",
            "192.0.2.1 a3",
            "192.0.2.2 b3"
        ]
    );
}

#[test]
fn baglanti_ve_uzak_adres_siniri_kibar_retle_kapatir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.sinir = 2;
    io.baglanti.ayar.uzak_adres_siniri = 1;
    io.baglanti.ayar.dolu_yaniti = Some("421 4.3.2 mesgul".into());
    for uzak in ["192.0.2.1", "192.0.2.1", "192.0.2.2", "192.0.2.3"] {
        io.baglanti
            .bekleyenler
            .push_back(SahteBaglanti::satirlarla("smtp_v4", uzak, &["QUIT"]));
    }
    calistir(YANKI, &mut io);
    assert_eq!(io.baglanti.baglantilar.len(), 2);
    assert_eq!(
        io.baglanti.reddedilenler,
        vec![
            (
                "192.0.2.1".to_string(),
                Some("421 4.3.2 mesgul".to_string())
            ),
            (
                "192.0.2.3".to_string(),
                Some("421 4.3.2 mesgul".to_string())
            ),
        ]
    );
}

#[test]
fn susan_istemci_zaman_asimi_sonucu_alir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let mut b = SahteBaglanti::satirlarla("smtp_v4", "192.0.2.5", &["NOOP"]);
    b.sessiz_kalir = true;
    io.baglanti.bekleyenler.push_back(b);
    let kaynak = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    2 kez tekrarla
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            bağlantıya "250 tamam" satırını yaz
        değilse
            hata komutun hatası olsun
            veri hatanın verisi olsun
            bağlantıya "421 " ile verinin "tür" değeri satırını yaz
"#;
    calistir(kaynak, &mut io);
    assert_eq!(giden(&io, 0), vec!["250 tamam", "421 zaman_aşımı"]);
}

#[test]
fn son_tarih_okuma_noktasinda_yetismezse_kolunu_calistirir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let mut b = SahteBaglanti::satirlarla("smtp_v4", "192.0.2.6", &[]);
    b.sessiz_kalir = true;
    io.baglanti.bekleyenler.push_back(b);
    io.baglanti.ayar.bosta_suresi_ms = 300_000;
    let kaynak = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    2 saniye içinde
        komut bağlantıdan satır okumayı dene olsun
        bağlantıya "250 erken" satırını yaz
    yetişmezse
        bağlantıya "421 4.4.2 komut süresi doldu" satırını yaz
"#;
    calistir(kaynak, &mut io);
    assert_eq!(giden(&io, 0), vec!["421 4.4.2 komut süresi doldu"]);
}

#[test]
fn gorev_hatasi_yalniz_kendi_baglantisini_kapatir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let kaynak = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    komut başarılıysa
        metin komutun değeri olsun
        metin "BOZ" e eşitse
            bağlantıya "satır\nenjeksiyonu" satırını yaz
        bağlantıya "250 " ile metin satırını yaz
"#;
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla("smtp_v4", "192.0.2.7", &["BOZ"]));
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla("smtp_v4", "192.0.2.8", &["IYI"]));
    calistir(kaynak, &mut io);
    assert!(giden(&io, 0).is_empty(), "CR/LF içeren satır yazılmaz");
    assert_eq!(giden(&io, 1), vec!["250 IYI"]);
    assert_eq!(io.baglanti.gunluk.len(), 1);
    assert!(
        io.baglanti.gunluk[0].1.contains("C030"),
        "{:?}",
        io.baglanti.gunluk
    );
    assert!(io.baglanti.baglantilar.iter().all(|b| b.kapandi));
}

#[test]
fn yazmayi_dene_enjeksiyonu_sonuc_olarak_reddeder() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let kaynak = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    sonuç bağlantıya "a\nb" satırını yazmayı dene olsun
    sonuç başarısızsa
        hata sonucun hatası olsun
        veri hatanın verisi olsun
        bağlantıya "ret " ile verinin "tür" değeri satırını yaz
    iyi bağlantıya "250 tamam" satırını yazmayı dene olsun
    iyi başarılıysa
        iyinin değeri yaz
"#;
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla("smtp_v4", "192.0.2.9", &[]));
    calistir(kaynak, &mut io);
    assert_eq!(giden(&io, 0), vec!["ret yazılamadı", "250 tamam"]);
    assert_eq!(io.cikti, vec!["11"]);
}

#[test]
fn kayitsiz_dinleyici_ikinci_blok_ve_blok_disi_kullanim_reddedilir() {
    let program = dil::kaynagi_derle(YANKI).unwrap();
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.dinleyici_adlari = Some(vec!["imap_v4".into()]);
    let hata = calistir_io(&program, &mut io).unwrap_err();
    assert_eq!(hata.kod, "C030");
    assert!(hata.mesaj.contains("smtp_v4"), "{}", hata.mesaj);

    let iki =
        format!("{YANKI}\n\"smtp_v4\" dinleyicisinde bağlantı geldiğinde\n    bağlantıyı kapat\n");
    let program = dil::kaynagi_derle(&iki).unwrap();
    let hata = calistir_io(&program, &mut ToplayanIo::yeni(Vec::new())).unwrap_err();
    assert_eq!(hata.kod, "C030");
    assert!(hata.mesaj.contains("ikinci"), "{}", hata.mesaj);

    let disarida = "sonuç bağlantıdan satır okumayı dene olsun\n";
    let program = dil::kaynagi_derle(disarida).unwrap();
    let hata = calistir_io(&program, &mut ToplayanIo::yeni(Vec::new())).unwrap_err();
    assert_eq!(hata.kod, "C030");

    let web = format!("8080 kapısında sunucu başlat\n{YANKI}");
    let program = dil::kaynagi_derle(&web).unwrap();
    let hata = calistir_io(&program, &mut ToplayanIo::yeni(Vec::new())).unwrap_err();
    assert_eq!(hata.kod, "C030");
    assert!(hata.mesaj.contains("web sunucusu"), "{}", hata.mesaj);
}

#[test]
fn yazilan_satir_metin_olmali_t034() {
    let hata = dil::kaynagi_derle(
        "\"smtp_v4\" dinleyicisinde bağlantı geldiğinde\n    bağlantıya 5 satırını yaz\n",
    )
    .unwrap_err();
    assert_eq!(hata.kod, "T034");
    let hata = dil::kaynagi_derle("5 dinleyicisinde bağlantı geldiğinde\n    bağlantıyı kapat\n")
        .unwrap_err();
    assert_eq!(hata.kod, "T034");
}

#[test]
fn guvenli_cocuk_modu_baglanti_dinleyemez() {
    let program = dil::kaynagi_derle(YANKI).unwrap();
    let mut io = dil::yorumlayici::GuvenliIo::yeni(ToplayanIo::yeni(Vec::new()));
    let hata = calistir_io(&program, &mut io).unwrap_err();
    assert_eq!(hata.kod, "C030");
    assert!(hata.mesaj.contains("güvenli modda"), "{}", hata.mesaj);
}

#[test]
fn bekle_baglanti_gorevinde_kardesi_durdurmaz_ve_iz_replay_ayni_ciktiyi_verir() {
    let kaynak = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    komut başarılıysa
        metin komutun değeri olsun
        metin "YAVAS" e eşitse
            1 saniye bekle
        bağlantıya "250 " ile metin satırını yaz
        metin yaz
"#;
    let mut io = IzKaydedenIo::yeni(ToplayanIo::yeni(Vec::new()));
    io.ic
        .baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla(
            "smtp_v4",
            "192.0.2.1",
            &["YAVAS"],
        ));
    io.ic
        .baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla(
            "smtp_v4",
            "192.0.2.2",
            &["HIZLI"],
        ));
    let program = dil::kaynagi_derle(kaynak).unwrap();
    calistir_io(&program, &mut io).unwrap();
    let iz = io.iz_metni().unwrap();
    let ic = io.icine_al();
    assert_eq!(ic.cikti, vec!["HIZLI", "YAVAS"]);
    assert!(iz.contains("baglanti_bekle") && iz.contains("baglanti_islemi"));

    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    calistir_io(&program, &mut oynatici).unwrap();
    assert_eq!(oynatici.uyusmazlik(), None);
    assert_eq!(oynatici.cikti, vec!["HIZLI", "YAVAS"]);
}

const STARTTLS: &str = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    3 kez tekrarla
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            metin komutun değeri olsun
            metin "STARTTLS" e eşitse
                bağlantıya "220 hazır" satırını yaz
                sonuç bağlantıda TLS başlatmayı dene olsun
                sonuç başarılıysa
                    bağlantıya "250 " ile sonucun değeri satırını yaz
                değilse
                    hata sonucun hatası olsun
                    veri hatanın verisi olsun
                    tür verinin "tür" değeri olsun
                    hatanın mesajı yaz
                    son bağlantıya "454 " ile tür satırını yazmayı dene olsun
            değilse
                bağlantıya "250 " ile metin satırını yaz
"#;

fn tls_ayarli() -> ToplayanIo {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.tls = Some(dil::proje::TlsBildirimi {
        dinleyiciler: Vec::new(),
        sertifika_degiskeni: "ZINCIR".into(),
        anahtar_degiskeni: "ANAHTAR".into(),
    });
    io
}

#[test]
fn starttls_ayni_parcadaki_komutu_reddeder_ve_baglantiyi_kapatir() {
    let mut io = tls_ayarli();
    io.baglanti.bekleyenler.push_back(SahteBaglanti::yeni(
        "smtp_v4",
        "192.0.2.20",
        &[b"STARTTLS\r\nRCPT TO:<kurban@ornek.test>\r\n"],
    ));
    calistir(STARTTLS, &mut io);
    let b = &io.baglanti.baglantilar[0];
    assert!(b.kapandi && !b.tls);
    // Yükseltme reddi kapanmış bağlantıya yazılamaz; RCPT hiç işlenmedi.
    assert_eq!(b.giden, vec!["220 hazır"]);
    assert!(io.cikti[0].contains("komut enjeksiyonu"), "{:?}", io.cikti);
}

#[test]
fn starttls_bos_tamponda_yukseltir_ikinci_yukseltme_ve_kimliksiz_surec_reddedilir() {
    let mut io = tls_ayarli();
    io.baglanti.bekleyenler.push_back(SahteBaglanti::satirlarla(
        "smtp_v4",
        "192.0.2.21",
        &["STARTTLS", "EHLO sifreli", "STARTTLS"],
    ));
    calistir(STARTTLS, &mut io);
    let b = &io.baglanti.baglantilar[0];
    assert!(b.tls);
    assert_eq!(
        b.giden,
        vec![
            "220 hazır",
            "250 TLSv1_3 TLS13_AES_128_GCM_SHA256",
            "250 EHLO sifreli",
            "220 hazır",
            "454 tls"
        ]
    );
    // TLS kimliği bildirilmemiş süreçte bağlantı açık kalır, sonuç `tls`.
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.bekleyenler.push_back(SahteBaglanti::satirlarla(
        "smtp_v4",
        "192.0.2.22",
        &["STARTTLS", "NOOP"],
    ));
    calistir(STARTTLS, &mut io);
    assert_eq!(
        io.baglanti.baglantilar[0].giden,
        vec!["220 hazır", "454 tls", "250 NOOP"]
    );
}
