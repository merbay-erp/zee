//! K-402 (RFC-0043 Dilim 2, ADR-167 eki): bellekteki metin ↔ bayt kalıpları.
//! UTF-8 oktet sayısı, tek satır base64 kodla/çöz (katı RFC 4648 + katı
//! UTF-8), kod noktası ↔ harf. Saflık (yetkinlik/IO izi yok), C032 verisi,
//! girdiyi taşımayan hata mesajı, ayırmadan önce 1 MiB çözme tavanı.

use dil::kaynagi_derle;
use dil::yorumlayici::{self, IzKaydedenIo, ToplayanIo};

fn calistir(kaynak: &str) -> Vec<String> {
    calistir_girdiyle(kaynak, Vec::new())
}

fn calistir_girdiyle(kaynak: &str, girdiler: Vec<String>) -> Vec<String> {
    let program = kaynagi_derle(kaynak).expect("K-402 kaynağı derlenmeli");
    let mut io = ToplayanIo::yeni(girdiler);
    yorumlayici::calistir_io(&program, &mut io).expect("K-402 programı çalışmalı");
    io.cikti
}

/// `kod` değişkenini çözer; başarıda değeri, başarısızlıkta C032 verisini yazar.
fn coz(kod: &str) -> String {
    let kaynak = format!(
        "kod \"{kod}\" olsun\nsonuç kodun base64 çözümünü almayı dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\nsonuç başarısızsa\n    hata sonucun hatası olsun\n    hatanın kodu yaz\n    hatanın verisi yaz\n"
    );
    calistir(&kaynak).join("|")
}

fn base64(veri: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut cikti = String::new();
    for grup in veri.chunks(3) {
        let b = [
            grup[0],
            *grup.get(1).unwrap_or(&0),
            *grup.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            cikti.push(if i <= grup.len() {
                A[((n >> (18 - 6 * i)) & 63) as usize] as char
            } else {
                '='
            });
        }
    }
    cikti
}

#[test]
fn utf8_bayt_sayisi_oktet_uzunlugu_harf_sayar() {
    let cikti = calistir(
        "harf \"ç\" olsun\nharfin utf8 bayt sayısı yaz\nharfin uzunluğu yaz\nyüz \"😀\" olsun\nyüzün utf8 bayt sayısı yaz\nyüzün uzunluğu yaz\nboşluk \"\" olsun\nboşluğun utf8 bayt sayısı yaz\nkomut \"MAIL FROM:<ğ@örnek.test>\" olsun\nkomutun utf8 bayt sayısı yaz\n",
    );
    assert_eq!(cikti, ["2", "1", "4", "1", "0", "26"]);
}

#[test]
fn rfc4648_vektorleri_gidis_donus() {
    for (duz, kod) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        let cikti = calistir(&format!(
            "düz \"{duz}\" olsun\nkod düzün base64 karşılığı olsun\nkod yaz\nsonuç kodun base64 çözümünü almayı dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\n"
        ));
        assert_eq!(cikti, [kod, duz], "{duz:?}");
    }
    // ASCII dışı: UTF-8 baytları kodlanır (elle tablo döngüsünün yanlışı).
    assert_eq!(
        calistir("metin \"çğ\" olsun\nmetnin base64 karşılığı yaz\n"),
        [base64("çğ".as_bytes())]
    );
}

#[test]
fn kati_cozme_gevsek_girdiyi_konumla_reddeder() {
    for (kod, beklenen) in [
        ("Zg", "C032|tür: uzunluk, konum: 2"),
        ("Zg=", "C032|tür: uzunluk, konum: 3"),
        ("Zg===", "C032|tür: dolgu, konum: 4"),
        ("Zm9v\\n", "C032|tür: alfabe, konum: 4"),
        ("Zm 9v", "C032|tür: alfabe, konum: 2"),
        ("Zm9v-_==", "C032|tür: alfabe, konum: 4"),
        ("Zh==", "C032|tür: dolgu, konum: 3"),
        ("Zg==Zg==", "C032|tür: dolgu, konum: 4"),
    ] {
        assert_eq!(coz(kod), beklenen, "{kod:?}");
    }
}

#[test]
fn kati_utf8_olumsuz_vektorleri_konumla_reddedilir() {
    for (baytlar, konum) in [
        (&b"\xC0\xAF"[..], 0),
        (b"\xED\xA0\x80", 0),
        (b"\xF4\x90\x80\x80", 0),
        (b"ab\xE2\x82", 2),
        (b"\xFF", 0),
    ] {
        assert_eq!(
            coz(&base64(baytlar)),
            format!("C032|tür: utf8, konum: {konum}"),
            "{baytlar:?}"
        );
    }
    // NUL ve BOM geçerli UTF-8'dir; onarım ya da atlama yok.
    let cikti = calistir(&format!(
        "kod \"{}\" olsun\nçözüm kodun base64 çözümünü almayı dene olsun\nçözüm başarılıysa\n    metin çözümün değeri olsun\n    metnin utf8 bayt sayısı yaz\n    metnin uzunluğu yaz\n",
        base64("\u{feff}a\u{0}b".as_bytes())
    ));
    assert_eq!(cikti, ["6", "4"]);
}

/// ZM-F023 reproducer'ının küçültmesi: AUTH PLAIN (RFC 4616)
/// `\0kullanıcı\0parola` istemci ve sunucu tarafı, NUL kod noktasından.
#[test]
fn auth_plain_gidis_donusu_ve_hata_yolunda_parola_gorunmez() {
    let kaynak = r#"
nul 0 kod noktası harfini almayı dene olsun
nul başarılıysa
    ayraç nulün değeri olsun
    öğeler boş liste olsun
    öğelere "" ekle
    öğelere "ayşe" ekle
    öğelere "Gizli-Parola-ğ1" ekle
    kimlik öğelerin ayraç ile birleşmişi olsun
    kod kimliğin base64 karşılığı olsun
    kod yaz
    çözüm kodun base64 çözümünü almayı dene olsun
    çözüm başarılıysa
        düz çözümün değeri olsun
        parçalar düzün ayraç ile parçaları olsun
        parçaların adedi yaz
        parçaların sonu yaz
"#;
    let parola = "Gizli-Parola-ğ1";
    let beklenen = base64(format!("\0ayşe\0{parola}").as_bytes());
    assert_eq!(calistir(kaynak), [beklenen.as_str(), "3", parola]);

    // Bozulmuş AUTH satırı: mesaj ve veri girdiyi taşımaz.
    let bozuk = format!("{}!", &beklenen[..beklenen.len() - 1]);
    let program = kaynagi_derle(&format!(
        "kod \"{bozuk}\" olsun\nsonuç kodun base64 çözümünü almayı dene olsun\nsonuç başarısızsa\n    hata sonucun hatası olsun\n    hatanın mesajı yaz\n    hatanın verisi yaz\n"
    ))
    .unwrap();
    let mut io = ToplayanIo::yeni(Vec::new());
    yorumlayici::calistir_io(&program, &mut io).unwrap();
    let hepsi = io.cikti.join("\n");
    assert!(hepsi.contains("tür: alfabe"), "{hepsi}");
    assert!(
        !hepsi.contains(&bozuk) && !hepsi.contains("Gizli"),
        "{hepsi}"
    );
}

#[test]
fn cozme_tavani_bir_mib_ayirmadan_once() {
    // Kod değeri girdiden gelir (kaynak metni sınırına takılmasın).
    let kaynak = "\"kod?\" diye sor\nkod yanıt olsun\nsonuç kodun base64 çözümünü almayı dene olsun\nsonuç başarılıysa\n    metin sonucun değeri olsun\n    metnin utf8 bayt sayısı yaz\nsonuç başarısızsa\n    hata sonucun hatası olsun\n    hatanın verisi yaz\n";
    let mib = 1024 * 1024;
    let tam = calistir_girdiyle(kaynak, vec![base64(&vec![b'a'; mib])]);
    assert_eq!(tam.last().unwrap(), &mib.to_string());
    let fazla = calistir_girdiyle(kaynak, vec![base64(&vec![b'a'; mib + 1])]);
    assert_eq!(fazla.last().unwrap(), "tür: boyut_aşımı");
}

#[test]
fn kod_noktasi_harf_ve_sinirlari() {
    let cikti = calistir(
        "harf \"A\" olsun\nharfin kod noktası yaz\nyüz \"😀\" olsun\nyüzün kod noktası yaz\nson 1114111 kod noktası harfini almayı dene olsun\nson başarılıysa\n    metin sonun değeri olsun\n    metnin utf8 bayt sayısı yaz\nsatırbaşı 13 kod noktası harfini almayı dene olsun\nsatırbaşı başarılıysa\n    \"satır\" yaz\n",
    );
    assert_eq!(cikti, ["65", "128512", "4", "satır"]);
    for sayi in ["1114112", "55296", "57343", "0 ile 1 in farkı"] {
        let kaynak = format!(
            "sayı {sayi} olsun\nsonuç sayının kod noktası harfini almayı dene olsun\nsonuç başarısızsa\n    hata sonucun hatası olsun\n    hatanın kodu yaz\n    hatanın verisi yaz\n"
        );
        assert_eq!(calistir(&kaynak), ["C032", "tür: kod_noktası"], "{sayi}");
    }
}

#[test]
fn kod_noktasi_tek_harf_disinda_calisma_hatasi() {
    for metin in ["", "ab"] {
        let program = kaynagi_derle(&format!(
            "metin \"{metin}\" olsun\nmetnin kod noktası yaz\n"
        ))
        .unwrap();
        let mut io = ToplayanIo::yeni(Vec::new());
        let hata = yorumlayici::calistir_io(&program, &mut io).unwrap_err();
        assert_eq!(hata.kod, "C032", "{metin:?}");
        assert!(hata.oneri.as_deref().unwrap_or("").contains("harfleri"));
    }
}

#[test]
fn tur_hatalari_derleme_zamaninda() {
    for kaynak in [
        "sayı 5 olsun\nsayının utf8 bayt sayısı yaz\n",
        "sayı 5 olsun\nsayının base64 karşılığı yaz\n",
        "sayı 5 olsun\nsonuç sayının base64 çözümünü almayı dene olsun\n",
        "sayı 5 olsun\nsayının kod noktası yaz\n",
        "metin \"a\" olsun\nsonuç metnin kod noktası harfini almayı dene olsun\n",
    ] {
        let hata = kaynagi_derle(kaynak).expect_err(kaynak);
        assert!(hata.kod.starts_with('T'), "{kaynak}: {hata:?}");
    }
}

#[test]
fn saf_yetkinliksiz_ve_iz_olay_yazmaz() {
    let saf = "metin \"gizli-ğ\" olsun\nkod metnin base64 karşılığı olsun\nmetnin utf8 bayt sayısı yaz\nçözüm kodun base64 çözümünü almayı dene olsun\nçözüm başarılıysa\n    çözümün değeri yaz\n";
    let sabit = "\"8\" yaz\n\"gizli-ğ\" yaz\n";
    let iz = |kaynak: &str| {
        let program = kaynagi_derle(kaynak).unwrap();
        let mut kaydeden = IzKaydedenIo::yeni(ToplayanIo::yeni(Vec::new()));
        yorumlayici::calistir_io(&program, &mut kaydeden).unwrap();
        kaydeden.iz_metni().unwrap()
    };
    assert_eq!(iz(saf), iz(sabit), "saf dönüşüm IO izine olay yazmamalı");
}
