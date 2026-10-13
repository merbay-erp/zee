//! K-210 — Site E dogfood'unun gerektirdiği kriptografi/iletişim yüzeyleri.
//!
//! Her yüzey: lowering (merkezi intrinsic kaydı), hermetik davranış
//! (`ToplayanIo`, zee-io-1 determinizmi) ve yetkinlik kapısı olumsuzu.

use dil::agac::{Cumle, Ifade};
use dil::intrinsic::{self, IntrinsicEtkisi, Yetkinlik, PAROLA_OZETI};
use dil::yorumlayici::{calistir_io, ToplayanIo};

fn kostur(kaynak: &str) -> Result<Vec<String>, dil::tani::Tani> {
    let program = dil::kaynagi_derle(kaynak)?;
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io)?;
    Ok(io.cikti)
}

#[test]
fn argon2_ozeti_kalibi_kriptografi_intrinsicine_iner() {
    let program = dil::kaynagi_derle("özet \"gizli parola\" için argon2 özeti olsun\n")
        .or_else(|_| {
            dil::kaynagi_derle("parola \"gizli parola\" olsun\nözet parolanın argon2 özeti olsun\n")
        })
        .expect("argon2 özeti derlenmeli");
    let son = program.cumleler.last().expect("cümle");
    let Cumle::Olsun { deger, .. } = son else {
        panic!("atama bekleniyordu")
    };
    let Ifade::Intrinsic { kimlik, argumanlar } = deger.turu() else {
        panic!("intrinsic bekleniyordu: {deger:?}")
    };
    assert_eq!(kimlik, PAROLA_OZETI);
    assert_eq!(argumanlar.len(), 1);
    let tanim = intrinsic::tanim(PAROLA_OZETI).expect("kayıt");
    assert_eq!(tanim.yetkinlik, Yetkinlik::Kriptografi);
    assert_eq!(tanim.etki, IntrinsicEtkisi::Saf);
}

#[test]
fn argon2_ozeti_hermetik_profilde_deterministik_ve_dogrulanabilir() {
    let kaynak = "\
parola \"uzun bir deneme parolası\" olsun
özet parolanın argon2 özeti olsun
özet yaz
parola özet ile doğrulanıyorsa
    \"doğru\" yaz
\"yanlış parola\" özet ile doğrulanıyorsa
    \"olmamalı\" yaz
";
    let bir = kostur(kaynak).expect("çalışmalı");
    let iki = kostur(kaynak).expect("çalışmalı");
    assert_eq!(bir, iki, "zee-io-1: aynı tohum aynı özet");
    assert_eq!(bir.len(), 2, "{bir:?}");
    assert!(bir[0].starts_with("$argon2id$v=19$"), "{}", bir[0]);
    assert_eq!(bir[1], "doğru");
}

#[test]
fn argon2_ozeti_sayiya_uygulanamaz() {
    let hata = dil::kaynagi_derle("sayı 5 olsun\nözet sayının argon2 özeti olsun\n")
        .expect_err("T034 bekleniyordu");
    assert_eq!(hata.kod, "T034");
}

#[test]
fn totp_anahtari_ve_dogrulama_hermetik_saatle_calisir() {
    // ToplayanIo sabit saati (2026-09-09 12:00 UTC) ile kod deterministiktir.
    let program = dil::kaynagi_derle(
        "gizli yeni totp anahtarı olsun\ngizli yaz\n\"000000\" gizli anahtarıyla doğrulanıyorsa\n    \"yanlış kabul\" yaz\n",
    )
    .expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.zaman = (2026, 9, 9, 12, 0);
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti.len(), 1, "{:?}", io.cikti);
    let gizli = io.cikti[0].clone();
    assert_eq!(gizli.len(), 32, "160 bit Base32 = 32 karakter");
    assert!(gizli
        .bytes()
        .all(|b| b.is_ascii_uppercase() || (b'2'..=b'7').contains(&b)));

    // Aynı gizli için doğru kodu Rust tarafında üretip zee'de doğrula.
    let epoch = 20_705i64 * 86_400 + 12 * 3_600; // 2026-09-09 12:00:00 UTC
    let kod = dil::guvenlik::totp_kodu(&base32_coz_test(&gizli), epoch);
    let kaynak = format!(
        "kod \"{kod}\" olsun\nkod \"{gizli}\" anahtarıyla doğrulanıyorsa\n    \"geçerli\" yaz\n\"123456\" \"{gizli}\" anahtarıyla doğrulanıyorsa\n    \"olmamalı\" yaz\n"
    );
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.zaman = (2026, 9, 9, 12, 0);
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti, vec!["geçerli"]);
}

fn base32_coz_test(metin: &str) -> Vec<u8> {
    const A: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let (mut tampon, mut bit, mut cikti) = (0u64, 0u32, Vec::new());
    for k in metin.bytes() {
        tampon = (tampon << 5) | A.iter().position(|&a| a == k).expect("base32") as u64;
        bit += 5;
        if bit >= 8 {
            bit -= 8;
            cikti.push(((tampon >> bit) & 0xff) as u8);
        }
    }
    cikti
}

#[test]
fn kriptografi_yetkinligi_kapaliyken_totp_ve_ozet_reddedilir() {
    use dil::yetkinlik::YetkinlikPolitikasi;
    use dil::yorumlayici::PolitikaliIo;
    let program = dil::kaynagi_derle("parola \"gizli\" olsun\nözet parolanın argon2 özeti olsun\n")
        .expect("derlenmeli");
    let mut io =
        PolitikaliIo::politikali(ToplayanIo::yeni(Vec::new()), YetkinlikPolitikasi::kapali());
    let hata = calistir_io(&program, &mut io).expect_err("kapalı politika C028 vermeli");
    assert_eq!(hata.kod, "C028", "{hata:?}");
}

#[test]
fn eposta_gonderimi_hermetik_kutuya_duser_ve_eylemde_yasaktir() {
    let program = dil::kaynagi_derle(
        "sonuç \"ali@ornek.com\" \"Yeni mesaj\" \"Merhaba\" ile ileti göndermeyi dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\n",
    )
    .expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti, vec!["1"]);
    assert_eq!(io.eposta_kutusu.len(), 1);
    assert_eq!(io.eposta_kutusu[0].0, "ali@ornek.com");
    assert_eq!(io.eposta_kutusu[0].1, "Yeni mesaj");

    let bozuk = dil::kaynagi_derle(
        "sonuç \"bozuk adres\" \"k\" \"g\" ile ileti göndermeyi dene olsun\nsonuç başarısızsa\n    sonucun hatası yaz\n",
    )
    .expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&bozuk, &mut io).expect("dene hatayı Sonuç yapar");
    assert!(io.cikti[0].contains("geçersiz"), "{:?}", io.cikti);
    assert!(io.eposta_kutusu.is_empty());

    // Geri alınamaz dış etki eylem içinde T048'dir.
    let hata = dil::kaynagi_derle(
        "eylem bildirim gönder\n    alıcıyı Metin olarak al\n    değer döndürmez\n    sonuç alıcı \"k\" \"g\" ile ileti göndermeyi dene olsun\n",
    )
    .expect_err("T048 bekleniyordu");
    assert_eq!(hata.kod, "T048", "{hata:?}");
}

#[test]
fn gorsel_kucultme_hermetik_dosya_sisteminde_calisir() {
    let program = dil::kaynagi_derle(
        "sonuç \"buyuk.jpg\" görselini \"kucuk.jpg\" dosyasına 800 piksele küçültmeyi dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\nikinci \"yok.jpg\" görselini \"k.jpg\" dosyasına 800 piksele küçültmeyi dene olsun\nikinci başarısızsa\n    \"kaynak yok\" yaz\n",
    )
    .expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert("buyuk.jpg".into(), "JPEGDATA".into());
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti, vec!["32", "kaynak yok"], "yazılan bayt sayısı");
    assert!(io.dosyalar.contains_key("kucuk.jpg"));

    let hata = dil::kaynagi_derle(
        "sonuç \"a.jpg\" görselini \"b.jpg\" dosyasına \"800\" piksele küçültmeyi dene olsun\n",
    )
    .expect_err("T034 bekleniyordu");
    assert_eq!(hata.kod, "T034");
}

#[test]
fn ag_hedefi_smtp_semalarini_tanir() {
    use dil::yetkinlik::{AgHedefi, AgSemasi};
    let h = AgHedefi::bildirimden("smtps://mail.example.invalid").unwrap();
    assert_eq!(h.semasi(), AgSemasi::Smtps);
    assert_eq!(h.kapisi(), 465);
    let d = AgHedefi::bildirimden("smtp://127.0.0.1:2525").unwrap();
    assert_eq!(d.semasi(), AgSemasi::Smtp);
    assert_eq!(d.kapisi(), 2525);
}

#[test]
fn baglantili_eposta_gonderimi_baglantiyi_dogrular_ve_kutuya_duser() {
    // K-210 eki: bağlantı metni değerden gelir (panelden ayarlanan SMTP); hermetikte
    // yalnız ayrıştırılır ve ileti kutuya düşer. Bağlantı izde parmak iziyle kalır.
    let program = dil::kaynagi_derle(
        "sonuç \"ali@ornek.com\" \"Konu\" \"Gövde\" ile \"smtps://k:p@mail.ornek.com:465?from=info@ornek.com\" üzerinden ileti göndermeyi dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\n",
    )
    .expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti, vec!["1"]);
    assert_eq!(io.eposta_kutusu.len(), 1);
    assert_eq!(io.eposta_kutusu[0].0, "ali@ornek.com");

    let bozuk = dil::kaynagi_derle(
        "sonuç \"ali@ornek.com\" \"k\" \"g\" ile \"http://mail.ornek.com\" üzerinden ileti göndermeyi dene olsun\nsonuç başarısızsa\n    sonucun hatası yaz\n",
    )
    .expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&bozuk, &mut io).expect("dene hatayı Sonuç yapar");
    assert!(io.cikti[0].contains("smtp"), "{:?}", io.cikti);
    assert!(io.eposta_kutusu.is_empty());

    let tanim = intrinsic::tanim(intrinsic::EPOSTA_GONDER_BAGLANTILI).expect("kayıt");
    assert_eq!(tanim.yetkinlik, Yetkinlik::Ag);
    assert_eq!(tanim.etki, IntrinsicEtkisi::GeriAlinamaz);
}

#[test]
fn gizli_deger_sifreleme_hermetikte_deterministik_ve_dogrulamali() {
    // K-210 eki 3: SMTP parolası gibi sırlar veritabanına `zee1:` metniyle yazılır.
    let program = dil::kaynagi_derle(
        "parola \"Gizli:P@ss/1\" olsun\nsifreli parolanın gizli şifrelenmişi olsun\nsifreli yaz\nsonuç sifreli gizli çözmeyi dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\nbozuk \"zee1:AAAA\" gizli çözmeyi dene olsun\nbozuk başarısızsa\n    bozuğun hatası yaz\n",
    )
    .expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti.len(), 3, "{:?}", io.cikti);
    assert!(io.cikti[0].starts_with("zee1:"), "{:?}", io.cikti);
    assert_eq!(io.cikti[1], "Gizli:P@ss/1");
    assert!(io.cikti[2].contains("gizli değer"), "{:?}", io.cikti);
    let mut io2 = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io2).expect("çalışmalı");
    assert_eq!(io.cikti[0], io2.cikti[0], "zee-io-1 determinizmi");

    let sifrele = intrinsic::tanim(intrinsic::GIZLI_SIFRELE).expect("kayıt");
    assert_eq!(sifrele.yetkinlik, Yetkinlik::Kriptografi);
    assert_eq!(sifrele.etki, IntrinsicEtkisi::Saf);
    let coz = intrinsic::tanim(intrinsic::GIZLI_COZ).expect("kayıt");
    assert_eq!(coz.yetkinlik, Yetkinlik::Kriptografi);
}
