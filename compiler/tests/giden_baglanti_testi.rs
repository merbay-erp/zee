//! K-397 (RFC-0044 B4a, ADR-173): giden TCP ve istemci STARTTLS — hermetik
//! adaptörle kaynaktan sahte uzak sunucuya: `ağ_hedefleri` `tcp://`
//! politikası, SSRF aralıkları, tek çözümleme (yeniden bağlama), görev
//! başına bağlantı, açık PKIX sonucu, yanıt enjeksiyonu, derleme yetkinliği
//! ve IO izi kaydı/oynatması.

use dil::yetkinlik::{AgHedefi, Yetkinlik, YetkinlikPolitikasi};
use dil::yorumlayici::{
    calistir_io, calistir_io_kodla, IzKaydedenIo, IzYenidenOynatici, PolitikaliIo, SahteBaglanti,
    TlsIstemciKaydi, ToplayanIo,
};
use std::collections::BTreeSet;
use std::net::IpAddr;

#[path = "ortak/x509_ozellikleri.rs"]
mod x509_ozellikleri;

const MX: &str = "93.184.216.34";

fn politika(hedefler: &[&str], yerel: bool) -> YetkinlikPolitikasi {
    let mut izinler = BTreeSet::from([Yetkinlik::Ag]);
    if yerel {
        izinler.insert(Yetkinlik::YerelAg);
    }
    YetkinlikPolitikasi::proje(
        izinler,
        hedefler
            .iter()
            .map(|h| AgHedefi::bildirimden(h).unwrap())
            .collect(),
    )
}

fn hermetik(hedefler: &[&str]) -> ToplayanIo {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.giden_politikasi = Some(politika(hedefler, false));
    io
}

fn dogrulanmis_kayit(ad: &str) -> TlsIstemciKaydi {
    TlsIstemciKaydi {
        dogrulandi: true,
        surum: "TLSv1_3".into(),
        takim: "TLS13_AES_128_GCM_SHA256".into(),
        sunucu_adi: ad.into(),
        sertifika_sha256: "a".repeat(64),
        spki_sha256: "b".repeat(64),
        zincir_sha256: vec!["a".repeat(64), "c".repeat(64)],
        zincir_spki_sha256: vec!["b".repeat(64), "d".repeat(64)],
        ..Default::default()
    }
}

fn calistir(kaynak: &str, io: &mut ToplayanIo) -> Vec<String> {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    calistir_io_kodla(&program, io).expect("çalışmalı");
    io.cikti.clone()
}

/// Bağlanır; başarıda uzak adresi, retta `tür`ü yazar.
fn baglan_kaynagi(adres: &str) -> String {
    format!(
        r#"sonuç "{adres}" adresine bağlanmayı dene olsun
sonuç başarılıysa
    bilgi sonucun değeri olsun
    bilginin "uzak_adres" değeri yaz
değilse
    hata sonucun hatası olsun
    veri hatanın verisi olsun
    verinin "tür" değeri yaz
"#
    )
}

const SMTP: &str = r#"
sonuç "93.184.216.34:25" adresine bağlanmayı dene olsun
sonuç başarılıysa
    bilgi sonucun değeri olsun
    bilginin "uzak_kapı" değeri yaz
    selam bağlantıdan satır okumayı dene olsun
    selam başarılıysa
        selamın değeri yaz
    bağlantıya "EHLO gonderen.ornek" satırını yaz
    yanıt bağlantıdan satır okumayı 5 saniye içinde dene olsun
    yanıt başarılıysa
        yanıtın değeri yaz
    bağlantıya "STARTTLS" satırını yaz
    hazır bağlantıdan satır okumayı dene olsun
    hazır başarılıysa
        hazırın değeri yaz
    tls bağlantıda TLS istemcisi başlatmayı dene "mx1.alici.ornek" ile olsun
    tls başarılıysa
        rapor tlsin değeri olsun
        raporun "doğrulandı" değeri yaz
        raporun "doğrulama_hatası" değeri yaz
        raporun "sunucu_adı" değeri yaz
        raporun "spki_sha256" değeri yaz
        raporun "zincir_sha256" değeri yaz
    son bağlantıya "QUIT" parçasını yazmayı dene olsun
    son başarılıysa
        bağlantıyı kapat
"#;

fn smtp_sunucusu(tls: Option<TlsIstemciKaydi>) -> SahteBaglanti {
    let mut sunucu = SahteBaglanti::giden_sunucu(&[
        "220 mx1.alici.ornek ESMTP",
        "250-mx1.alici.ornek",
        "220 2.0.0 hazır",
    ]);
    sunucu.tls_kaydi = tls;
    sunucu
}

#[test]
fn smtp_oturumu_giden_baglantida_a2_kaliplariyla_ve_tls_kaydiyla_yurur() {
    let mut io = hermetik(&["tcp://*:25"]);
    io.baglanti
        .giden_sunuculari
        .push_back(smtp_sunucusu(Some(dogrulanmis_kayit("mx1.alici.ornek"))));
    let cikti = calistir(SMTP, &mut io);
    assert_eq!(
        cikti,
        vec![
            "25",
            "220 mx1.alici.ornek ESMTP",
            "250-mx1.alici.ornek",
            "220 2.0.0 hazır",
            "evet",
            "",
            "mx1.alici.ornek",
            &"b".repeat(64),
            &format!("{} {}", "a".repeat(64), "c".repeat(64)),
        ]
    );
    let sunucu = &io.baglanti.baglantilar[0];
    assert_eq!(sunucu.giden, vec!["EHLO gonderen.ornek", "STARTTLS"]);
    assert!(sunucu.ham_giden.ends_with(b"QUIT"));
    assert!(sunucu.tls && sunucu.kapandi);
    assert_eq!(
        io.baglanti.giden_adresleri,
        vec![format!("{MX}:25").parse().unwrap()]
    );
}

#[test]
fn dogrulama_hatasi_sessiz_degil_ve_yanlis_sni_ad_uyusmuyor_verir() {
    let mut kendinden_imzali = dogrulanmis_kayit("mx1.alici.ornek");
    kendinden_imzali.dogrulandi = false;
    kendinden_imzali.dogrulama_hatasi = "bilinmeyen_yayıncı".into();
    let mut io1 = hermetik(&["tcp://*:25"]);
    io1.baglanti
        .giden_sunuculari
        .push_back(smtp_sunucusu(Some(kendinden_imzali)));
    let cikti = calistir(SMTP, &mut io1);
    assert_eq!(&cikti[4..6], ["hayır", "bilinmeyen_yayıncı"]);
    // Doğrulanmamış el sıkışmada da SPKI özeti DANE için raporlanır.
    assert_eq!(cikti[7], "b".repeat(64));

    let mut io2 = hermetik(&["tcp://*:25"]);
    io2.baglanti
        .giden_sunuculari
        .push_back(smtp_sunucusu(Some(dogrulanmis_kayit("baska.ornek"))));
    let cikti = calistir(SMTP, &mut io2);
    assert_eq!(&cikti[4..7], ["hayır", "ad_uyuşmuyor", "mx1.alici.ornek"]);
}

#[test]
fn ssrf_araliklari_ve_izinsiz_kapi_baglanti_acmadan_reddedilir() {
    for adres in [
        "127.0.0.1:25",
        "10.0.0.5:25",
        "100.64.1.1:25",
        "169.254.169.254:25",
        "192.168.1.1:25",
        "203.0.113.7:25",
        "224.0.0.1:25",
        "[::1]:25",
        "[::ffff:127.0.0.1]:25",
        "[::ffff:93.184.216.34]:25",
        "[fd00::1]:25",
        "[fe80::1]:25",
        "93.184.216.34:587",
        "93.184.216.34:22",
    ] {
        let mut io = hermetik(&["tcp://*:25"]);
        io.baglanti.giden_sunuculari.push_back(smtp_sunucusu(None));
        assert_eq!(
            calistir(&baglan_kaynagi(adres), &mut io),
            vec!["hedef_yasak"],
            "{adres}"
        );
        assert!(io.baglanti.baglantilar.is_empty(), "{adres}");
    }
    let mut biçimsiz = hermetik(&["tcp://*:25"]);
    for adres in ["93.184.216.34", "tcp://93.184.216.34:25", "*:25", ""] {
        assert_eq!(
            calistir(&baglan_kaynagi(adres), &mut biçimsiz)
                .last()
                .unwrap(),
            "geçersiz",
            "{adres}"
        );
    }
    let mut listesiz = hermetik(&["tcp://mx.izinli.ornek:25"]);
    assert_eq!(
        calistir(&baglan_kaynagi("93.184.216.34:25"), &mut listesiz),
        vec!["hedef_yasak"]
    );
}

#[test]
fn ad_tek_kez_cozulur_baglanti_denetlenen_ipye_sabitlenir() {
    let mut io = hermetik(&["tcp://*:25"]);
    let ip: IpAddr = MX.parse().unwrap();
    io.baglanti
        .giden_cozucu
        .insert("mx1.alici.ornek".into(), vec![ip]);
    io.baglanti
        .giden_cozucu
        .insert("kotu.ornek".into(), vec![ip, "10.0.0.5".parse().unwrap()]);
    io.baglanti
        .giden_cozucu
        .insert("ic.ornek".into(), vec!["127.0.0.1".parse().unwrap()]);
    io.baglanti.giden_sunuculari.push_back(smtp_sunucusu(None));
    let kaynak = format!(
        "{}bağlantıyı kapat\n{}{}{}",
        baglan_kaynagi("mx1.alici.ornek:25"),
        baglan_kaynagi("kotu.ornek:25"),
        baglan_kaynagi("ic.ornek:25"),
        baglan_kaynagi("yok.ornek:25"),
    );
    assert_eq!(
        calistir(&kaynak, &mut io),
        vec![MX, "hedef_yasak", "hedef_yasak", "çözülemedi"]
    );
    assert_eq!(
        io.baglanti.cozucu_cagrilari,
        vec!["mx1.alici.ornek", "kotu.ornek", "ic.ornek", "yok.ornek"]
    );
    assert_eq!(
        io.baglanti.giden_adresleri,
        vec![format!("{MX}:25").parse().unwrap()]
    );
}

#[test]
fn gorev_basina_tek_giden_kapatinca_yenisi_ve_dinleyicide_ret() {
    let mut io = hermetik(&["tcp://*:25"]);
    for _ in 0..2 {
        io.baglanti.giden_sunuculari.push_back(smtp_sunucusu(None));
    }
    let kaynak = format!(
        "{}{}bağlantıyı kapat\n{}",
        baglan_kaynagi("93.184.216.34:25"),
        baglan_kaynagi("93.184.216.34:25"),
        baglan_kaynagi("93.184.216.35:25"),
    );
    assert_eq!(
        calistir(&kaynak, &mut io),
        vec![MX, "zaten_bağlı", "93.184.216.35"]
    );

    let dinleyici = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    sonuç "93.184.216.34:25" adresine bağlanmayı dene olsun
    sonuç başarısızsa
        hata sonucun hatası olsun
        veri hatanın verisi olsun
        verinin "tür" değeri yaz
"#;
    let mut io = hermetik(&["tcp://*:25"]);
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla("smtp_v4", "192.0.2.9", &[]));
    io.baglanti.giden_sunuculari.push_back(smtp_sunucusu(None));
    assert_eq!(calistir(dinleyici, &mut io), vec!["geçersiz"]);
    assert_eq!(io.baglanti.baglantilar.len(), 1, "giden açılmadı");
}

#[test]
fn starttls_yaniti_sonrasi_duz_metin_enjeksiyonu_baglantiyi_kapatir() {
    let mut io = hermetik(&["tcp://*:25"]);
    let mut sunucu = SahteBaglanti::giden_sunucu(&[]);
    sunucu
        .parcalar
        .push_back(b"220 hazir\r\n250 enjekte\r\n".to_vec());
    sunucu.tls_kaydi = Some(dogrulanmis_kayit("mx1.alici.ornek"));
    io.baglanti.giden_sunuculari.push_back(sunucu);
    let kaynak = r#"
sonuç "93.184.216.34:25" adresine bağlanmayı dene olsun
hazır bağlantıdan satır okumayı dene olsun
hazır başarılıysa
    hazırın değeri yaz
tls bağlantıda TLS istemcisi başlatmayı dene "mx1.alici.ornek" ile olsun
tls başarılıysa
    "yükseldi" yaz
değilse
    hata tlsin hatası olsun
    veri hatanın verisi olsun
    verinin "tür" değeri yaz
"#;
    assert_eq!(calistir(kaynak, &mut io), vec!["220 hazir", "tls"]);
    assert!(io.baglanti.baglantilar[0].kapandi);
}

#[test]
fn gecersiz_sunucu_adi_ve_tls_sunmayan_sunucu_acik_hata_verir() {
    for (ad, sunucu_tls, beklenen) in [
        ("93.184.216.34", true, "geçersiz"),
        ("", true, "geçersiz"),
        ("mx1.alici.ornek.", true, "geçersiz"),
        ("tek", true, "geçersiz"),
        ("mx1.alici.ornek", false, "tls"),
    ] {
        let mut io = hermetik(&["tcp://*:25"]);
        io.baglanti.giden_sunuculari.push_back(smtp_sunucusu(
            sunucu_tls.then(|| dogrulanmis_kayit("mx1.alici.ornek")),
        ));
        let kaynak = format!(
            r#"sonuç "93.184.216.34:25" adresine bağlanmayı dene olsun
tls bağlantıda TLS istemcisi başlatmayı dene "{ad}" ile olsun
tls başarısızsa
    hata tlsin hatası olsun
    veri hatanın verisi olsun
    verinin "tür" değeri yaz
"#
        );
        assert_eq!(calistir(&kaynak, &mut io), vec![beklenen], "{ad}");
    }
}

#[test]
fn baglanti_kaliplari_ag_ile_derlenir_dinleyicisiz_ve_agsiz_t054() {
    let program = dil::kaynagi_derle(SMTP).unwrap();
    let yalniz_ag = politika(&["tcp://*:25"], false);
    dil::cozumleyici::yetkinlikleri_denetle(&program, &yalniz_ag).expect("ağ yeter");
    let dinleyicili =
        YetkinlikPolitikasi::proje(BTreeSet::from([Yetkinlik::AgDinleyici]), BTreeSet::new());
    let hata = dil::cozumleyici::yetkinlikleri_denetle(&program, &dinleyicili)
        .expect_err("bağlanma ağ ister");
    assert_eq!(hata.kod, "T054");
    assert!(hata.mesaj.contains("`ağ`"), "{}", hata.mesaj);
    let okuma = dil::kaynagi_derle("x bağlantıdan satır okumayı dene olsun\n").unwrap();
    let hata = dil::cozumleyici::yetkinlikleri_denetle(&okuma, &YetkinlikPolitikasi::kapali())
        .expect_err("ağsız ve dinleyicisiz");
    assert!(hata.mesaj.contains("ağ-dinleyici"), "{}", hata.mesaj);
    dil::cozumleyici::yetkinlikleri_denetle(&okuma, &dinleyicili).expect("dinleyici yeter");
    // K-396 satırlı base64 aktarımı da bağlantı işlemidir: giden bağlantıda
    // `ağ` (+ dosya okuma) yeter, `ağ-dinleyici` istenmez.
    let base64 = dil::kaynagi_derle(
        "x \"ek.bin\" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun\n\
         y \"ek.bin\" dosyasının base64 satırlarının 0 baytından 76 baytını bağlantıya aktarmayı dene olsun\n",
    )
    .unwrap();
    let ag_ve_dosya = YetkinlikPolitikasi::proje(
        BTreeSet::from([Yetkinlik::Ag, Yetkinlik::DosyaOkuma]),
        BTreeSet::from([AgHedefi::bildirimden("tcp://*:25").unwrap()]),
    );
    dil::cozumleyici::yetkinlikleri_denetle(&base64, &ag_ve_dosya).expect("ağ + dosya okuma yeter");
}

#[test]
fn politikali_io_agsiz_giden_baglanmayi_ic_adaptore_ulastirmaz() {
    let program = dil::kaynagi_derle(&baglan_kaynagi("93.184.216.34:25")).unwrap();
    let mut ic = hermetik(&["tcp://*:25"]);
    ic.baglanti.giden_sunuculari.push_back(smtp_sunucusu(None));
    let mut io = PolitikaliIo::politikali(
        ic,
        YetkinlikPolitikasi::proje(BTreeSet::from([Yetkinlik::AgDinleyici]), BTreeSet::new()),
    );
    calistir_io(&program, &mut io).unwrap();
    assert_eq!(io.ic.cikti, vec!["hedef_yasak"]);
    assert!(io.ic.baglanti.baglantilar.is_empty());
}

#[test]
fn giden_oturum_iz_kaydi_ve_oynatmasi_birebir() {
    let mut kayit = IzKaydedenIo::yeni(hermetik(&["tcp://*:25"]));
    kayit
        .ic
        .baglanti
        .giden_sunuculari
        .push_back(smtp_sunucusu(Some(dogrulanmis_kayit("mx1.alici.ornek"))));
    let program = dil::kaynagi_derle(SMTP).unwrap();
    calistir_io(&program, &mut kayit).unwrap();
    let iz = kayit.iz_metni().unwrap();
    let cikti = kayit.icine_al().cikti;
    let hex = |m: &str| m.bytes().map(|b| format!("{b:02x}")).collect::<String>();
    assert!(
        iz.contains(&hex("baglan")) && iz.contains(&hex("tls_istemci")),
        "{iz}"
    );
    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    calistir_io(&program, &mut oynatici).unwrap();
    assert_eq!(oynatici.uyusmazlik(), None);
    assert_eq!(oynatici.cikti, cikti);
}

#[test]
fn eszamanli_gorevler_kendi_giden_baglantisini_tasir_ve_bitiste_kapatir() {
    let mut io = hermetik(&["tcp://*:25"]);
    for _ in 0..2 {
        io.baglanti.giden_sunuculari.push_back(smtp_sunucusu(None));
    }
    let kaynak = r#"
işlem teslimi yap
    adresi al
    sonuç adres adresine bağlanmayı dene olsun
    selam bağlantıdan satır okumayı dene olsun
    metin "" olsun
    selam başarılıysa
        metin selamın değeri olsun
    metni döndür

eşzamanlı olarak
    birinci "93.184.216.34:25" için teslimi yap
    ikinci "93.184.216.35:25" için teslimi yap

hepsini bekle
birinci yaz
ikinci yaz
"#;
    let cikti = calistir(kaynak, &mut io);
    assert_eq!(cikti, vec!["220 mx1.alici.ornek ESMTP"; 2]);
    assert_eq!(io.baglanti.baglantilar.len(), 2);
    assert!(io.baglanti.baglantilar.iter().all(|b| b.kapandi));
}

#[test]
fn proje_bildirimi_gecersiz_joker_ve_kapisiz_tcp_hedefini_reddeder() {
    let bildirim = |hedef: &str| {
        dil::proje::bildirimi_oku(&format!(
            "proje \"p\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ\" listesi olsun\nağ_hedefleri \"{hedef}\" listesi olsun\n"
        ))
    };
    for hedef in ["tcp://*:25", "https://mta-sts.*", "tcp://mx.ornek:2525"] {
        assert!(bildirim(hedef).is_ok(), "{hedef}");
    }
    for hedef in [
        "tcp://*:26",
        "tcp://*",
        "tcp://mx.ornek",
        "https://*.ornek",
        "https://mta-sts.*:8443",
        "tcp://*:25/yol",
    ] {
        let hata = bildirim(hedef).expect_err(hedef);
        assert_eq!(hata.kod, "P015", "{hedef}: {}", hata.mesaj);
    }
}

/// K-411: `x509_spki` fuzz korpusu stable'da referans DER okuyucusuyla
/// diferansiyel denetimden geçer; her tohum 3000 kez deterministik bozulur.
/// Gerçek (openssl üretimi) sertifikalarda SPKI, openssl'in kendi çıkardığı
/// açık anahtar DER'ine bayt bayt eşittir.
#[test]
fn x509_fuzz_korpusu_referans_ve_openssl_ile_tutar() {
    let kok = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/x509_spki");
    let mut yollar: Vec<_> = std::fs::read_dir(&kok)
        .unwrap()
        .map(|g| g.unwrap().path())
        .collect();
    yollar.sort();
    assert!(yollar.len() >= 6, "korpus tohumu eksik");
    let mut gecerli = 0;
    let mut durum = 0x2545_f491_4f6c_dd1d_u64;
    for yol in &yollar {
        let der = std::fs::read(yol).unwrap();
        x509_ozellikleri::denetle(&der);
        gecerli += usize::from(dil::ag_satiri::x509::spki_dilimi(&der).is_some());
        for _ in 0..3000 {
            durum ^= durum << 13;
            durum ^= durum >> 7;
            durum ^= durum << 17;
            let mut bozuk = der.clone();
            let yer = (durum as usize) % bozuk.len();
            match durum % 3 {
                0 => bozuk[yer] ^= (durum >> 8) as u8 | 1,
                1 => bozuk.truncate(yer),
                _ => bozuk.insert(yer, (durum >> 16) as u8),
            }
            x509_ozellikleri::denetle(&bozuk);
        }
        let ad = yol.file_name().unwrap().to_string_lossy();
        if ad.starts_with("rsa") || ad.starts_with("ed25519") {
            let cikti = std::process::Command::new("openssl")
                .args(["x509", "-inform", "DER", "-noout", "-pubkey", "-in"])
                .arg(yol)
                .output()
                .expect("openssl bulunmalı");
            assert!(cikti.status.success(), "{ad}");
            let pem = String::from_utf8(cikti.stdout).unwrap();
            let govde: String = pem.lines().filter(|s| !s.starts_with("-----")).collect();
            let beklenen = dil::kodlama::base64_mime_coz(&govde, 4096).unwrap();
            assert_eq!(
                dil::ag_satiri::x509::spki_dilimi(&der),
                Some(beklenen.as_slice()),
                "{ad}"
            );
        }
    }
    assert!(gecerli >= 3, "SPKI dalını kapsayan geçerli tohum yok");
}
