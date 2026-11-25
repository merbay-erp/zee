//! K-390 (RFC-0044 B1, ADR-169): DNS sorgu yüzeyinin hermetik sınaması —
//! sahte çözücü bölgesi (`ToplayanIo::dns_bolgesi`), yetkinlik, proje
//! bildirimi (P015), IO izi kayıt/replay.

use dil::dns_teli::{Cozum, DnsHataTuru, DnsHatasi, Durum, Kayit, KayitTuru, KayitVerisi};
use dil::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};
use dil::yorumlayici::{calistir_io, ToplayanIo};

const RAPOR: &str = "\
sonuç \"Ornek.Test\" için MX kayıtlarını sorgulamayı dene olsun
sonuç başarılıysa
    kayıtlar sonucun değeri olsun
    kayıtların adedini yaz
    kayıtlardaki her satır için
        satırın \"öncelik\" değeri yaz
        satırın \"hedef\" değeri yaz
        satırın \"doğrulanmış\" değeri yaz
değilse
    hata sonucun hatası olsun
    hatanın kodunu yaz
    veri hatanın verisi olsun
    verinin \"tür\" değeri yaz
";

fn mx(oncelik: u16, hedef: &str) -> Kayit {
    Kayit {
        ad: "ornek.test".into(),
        ttl: 300,
        veri: KayitVerisi::Mx {
            oncelik,
            hedef: hedef.into(),
        },
    }
}

fn var(kayitlar: Vec<Kayit>, dogrulanmis: bool) -> Result<Cozum, DnsHatasi> {
    Ok(Cozum {
        durum: Durum::Var,
        kayitlar,
        dogrulanmis,
        kanonik_ad: "ornek.test".into(),
    })
}

fn calistir(kaynak: &str, io: &mut ToplayanIo) -> Vec<String> {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    calistir_io(&program, io).expect("çalışmalı");
    io.cikti.clone()
}

#[test]
fn mx_kayitlari_satir_satir_ve_kanonik_adla_sorgulanir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dns_bolgesi.insert(
        ("ornek.test".into(), KayitTuru::Mx),
        var(
            vec![mx(10, "mx1.ornek.test"), mx(20, "mx2.ornek.test")],
            true,
        ),
    );
    let cikti = calistir(RAPOR, &mut io);
    assert_eq!(
        cikti,
        [
            "2",
            "10",
            "mx1.ornek.test",
            "doğru",
            "20",
            "mx2.ornek.test",
            "doğru"
        ]
    );
    assert_eq!(
        io.dns_sorgulari,
        [("ornek.test".to_string(), KayitTuru::Mx, 5_000)]
    );
}

#[test]
fn kayit_yok_ve_alan_yok_bos_listedir_gecici_hata_c031() {
    let mut io = ToplayanIo::yeni(Vec::new());
    assert_eq!(calistir(RAPOR, &mut io), ["0"], "kayıtsız ad alan_yok");
    for (tur, yazim) in [
        (DnsHataTuru::SunucuHatasi, "sunucu_hatası"),
        (DnsHataTuru::ZamanAsimi, "zaman_aşımı"),
        (DnsHataTuru::Ret, "ret"),
        (DnsHataTuru::BozukYanit, "bozuk_yanıt"),
    ] {
        let mut io = ToplayanIo::yeni(Vec::new());
        io.dns_bolgesi.insert(
            ("ornek.test".into(), KayitTuru::Mx),
            Err(DnsHatasi::yeni(tur, "sahte")),
        );
        let cikti = calistir(RAPOR, &mut io);
        assert_eq!(cikti, ["C033", yazim]);
    }
}

#[test]
fn gecersiz_ad_tel_bicimine_girmeden_reddedilir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let kaynak = RAPOR.replace("\"Ornek.Test\"", "\"örnek.test\"");
    let cikti = calistir(&kaynak, &mut io);
    assert_eq!(cikti, ["C033", "geçersiz_ad"]);
    assert!(io.dns_sorgulari.is_empty());
}

#[test]
fn adres_txt_ve_tlsa_satirlari() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dns_bolgesi.insert(
        ("ornek.test".into(), KayitTuru::A),
        var(
            vec![Kayit {
                ad: "ornek.test".into(),
                ttl: 60,
                veri: KayitVerisi::A([192, 0, 2, 7]),
            }],
            false,
        ),
    );
    let mut v6 = [0u8; 16];
    v6[..4].copy_from_slice(&[0x20, 0x01, 0x0d, 0xb8]);
    v6[15] = 1;
    io.dns_bolgesi.insert(
        ("ornek.test".into(), KayitTuru::Aaaa),
        var(
            vec![Kayit {
                ad: "ornek.test".into(),
                ttl: 60,
                veri: KayitVerisi::Aaaa(v6),
            }],
            false,
        ),
    );
    io.dns_bolgesi.insert(
        ("_dmarc.ornek.test".into(), KayitTuru::Txt),
        var(
            vec![Kayit {
                ad: "_dmarc.ornek.test".into(),
                ttl: 60,
                veri: KayitVerisi::Txt(b"v=DMARC1; p=reject".to_vec()),
            }],
            false,
        ),
    );
    io.dns_bolgesi.insert(
        ("_25._tcp.mx.ornek.test".into(), KayitTuru::Tlsa),
        var(
            vec![Kayit {
                ad: "_25._tcp.mx.ornek.test".into(),
                ttl: 60,
                veri: KayitVerisi::Tlsa {
                    kullanim: 3,
                    secici: 1,
                    esleme: 1,
                    veri: vec![0xAB, 0xCD],
                },
            }],
            true,
        ),
    );
    let kaynak = "\
adresler \"ornek.test\" için adres kayıtlarını sorgulamayı dene olsun
adresler başarılıysa
    liste adreslerin değeri olsun
    listedeki her satır için
        satırın \"aile\" değeri yaz
        satırın \"adres\" değeri yaz
metinler \"_dmarc.ornek.test\" için TXT kayıtlarını sorgulamayı dene olsun
metinler başarılıysa
    txtler metinlerin değeri olsun
    txtlerdeki her satır için
        satırın \"metin\" değeri yaz
tlsalar \"_25._tcp.mx.ornek.test\" için TLSA kayıtlarını sorgulamayı dene olsun
tlsalar başarılıysa
    kümeler tlsaların değeri olsun
    kümelerdeki her satır için
        satırın \"kullanım\" değeri yaz
        satırın \"seçici\" değeri yaz
        satırın \"eşleme\" değeri yaz
        satırın \"veri\" değeri yaz
        satırın \"doğrulanmış\" değeri yaz
";
    let cikti = calistir(kaynak, &mut io);
    assert_eq!(
        cikti,
        [
            "ipv4",
            "192.0.2.7",
            "ipv6",
            "2001:db8::1",
            "v=DMARC1; p=reject",
            "3",
            "1",
            "1",
            "abcd",
            "doğru"
        ]
    );
}

#[test]
fn son_tarih_kalani_sorgu_suresini_kisaltir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.an_degerleri = vec![0, 0, 0, 0].into();
    let kaynak = "\
2 saniye içinde
    sonuç \"ornek.test\" için TXT kayıtlarını sorgulamayı dene olsun
    sonuç başarılıysa
        \"tamam\" yaz
yetişmezse
    \"süre doldu\" yaz
";
    let cikti = calistir(kaynak, &mut io);
    assert_eq!(cikti, ["tamam"]);
    let (_, _, sure) = &io.dns_sorgulari[0];
    assert!(*sure <= 2_000, "süre son tarihle kısalmalı: {sure}");
}

#[test]
fn dns_yetkinligi_olmadan_kalip_derlemede_reddedilir() {
    let program = dil::kaynagi_derle(RAPOR).expect("derlenmeli");
    let yok = YetkinlikPolitikasi::proje(Default::default(), Default::default());
    let tani = dil::cozumleyici::yetkinlikleri_denetle(&program, &yok).expect_err("dns kapalı");
    assert!(tani.mesaj.contains("dns"), "{}", tani.mesaj);
    let acik = YetkinlikPolitikasi::proje([Yetkinlik::Dns].into(), Default::default());
    dil::cozumleyici::yetkinlikleri_denetle(&program, &acik).expect("dns açık");
    // Çocuk (--güvenli) modunda kapalıdır.
    assert!(
        dil::cozumleyici::yetkinlikleri_denetle(&program, &YetkinlikPolitikasi::cocuk()).is_err()
    );
    // Bildirimsiz geliştirici akışında çözücü olmadığı için kapalıdır.
    assert!(
        dil::cozumleyici::yetkinlikleri_denetle(&program, &YetkinlikPolitikasi::gelistirici())
            .is_err()
    );
}

fn bildirim(ek: &str) -> Result<dil::proje::ProjeBildirimi, dil::tani::Tani> {
    dil::proje::bildirimi_oku(&format!(
        "proje \"d\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n{ek}"
    ))
}

#[test]
fn proje_dns_bildirimi_loopback_ve_kip_ister() {
    let tamam = bildirim(
        "yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"127.0.0.1:53\" olsun\ndns_dnssec \"zorunlu\" olsun\n",
    )
    .expect("geçerli");
    let ayar = tamam.dns.expect("dns ayarı");
    assert_eq!(ayar.cozucu.to_string(), "127.0.0.1:53");
    assert_eq!(ayar.dnssec, dil::dns_teli::DnssecKipi::Zorunlu);
    bildirim("yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"[::1]:5353\" olsun\ndns_dnssec \"bilgi\" olsun\n")
        .expect("IPv6 loopback");
    for (ek, parca) in [
        ("yetkinlikler \"dns\" listesi olsun\n", "ikisini de"),
        ("yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"127.0.0.1:53\" olsun\n", "ikisini de"),
        ("dns_çözücü \"127.0.0.1:53\" olsun\ndns_dnssec \"bilgi\" olsun\n", "yalnız `dns`"),
        (
            "yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"8.8.8.8:53\" olsun\ndns_dnssec \"bilgi\" olsun\n",
            "loopback",
        ),
        (
            "yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"127.0.0.1:0\" olsun\ndns_dnssec \"bilgi\" olsun\n",
            "0",
        ),
        (
            "yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"localhost:53\" olsun\ndns_dnssec \"bilgi\" olsun\n",
            "IP:kapı",
        ),
        (
            "yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"127.0.0.1:53\" olsun\ndns_dnssec \"evet\" olsun\n",
            "kipi değil",
        ),
    ] {
        let tani = bildirim(ek).expect_err(ek);
        assert_eq!(tani.kod, "P015", "{ek}");
        assert!(tani.mesaj.contains(parca), "{ek}: {}", tani.mesaj);
    }
}

#[test]
fn io_izi_dns_sorgusunu_kaydeder_ve_agsiz_oynatir() {
    let program = dil::kaynagi_derle(RAPOR).expect("derlenmeli");
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.dns_bolgesi.insert(
        ("ornek.test".into(), KayitTuru::Mx),
        var(vec![mx(5, "mx.ornek.test")], false),
    );
    let mut kaydeden = dil::yorumlayici::IzKaydedenIo::yeni(taban);
    calistir_io(&program, &mut kaydeden).expect("kayıt");
    let iz = kaydeden.iz_metni().expect("iz");
    assert!(iz.contains("dns_sorgula"));
    let mut oynatici = dil::yorumlayici::IzYenidenOynatici::yeni(&iz).expect("iz okunur");
    calistir_io(&program, &mut oynatici).expect("replay");
    assert_eq!(oynatici.cikti, ["1", "5", "mx.ornek.test", "yanlış"]);
    // Argümanı farklı sorgu (başka ad) izle eşleşmez.
    let baska = dil::kaynagi_derle(&RAPOR.replace("Ornek.Test", "baska.test")).unwrap();
    let mut oynatici = dil::yorumlayici::IzYenidenOynatici::yeni(&iz).expect("iz okunur");
    let sonuc = calistir_io(&baska, &mut oynatici);
    let cikti = oynatici.cikti.clone();
    assert!(
        sonuc.is_err() || oynatici.bitir().is_err(),
        "uyuşmaz iz kabul edildi: {cikti:?}"
    );
}
