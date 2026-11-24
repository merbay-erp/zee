//! K-389 (RFC-0043 Dilim 1, ADR-167): dosya ↔ kodlanmış içerik ve bellekte
//! SHA-256. Hermetik adaptörde kalıp/tür/hata biçimi; gerçek `dil` sürecinde
//! MIME ileti dosyasından aralıklı okuma + çözerek yazma ile çıkarılan ekin
//! bayt-kesinliği, RFC-0041 dosya yanıtına bağlanması, no-clobber, tavan,
//! proje dosya sınırı ve içeriksiz IO izi.
//!
//! K-395 (ADR-167 eki, ZM-F039): çözerek yazma/ekleme hedef tavanı 32 MiB;
//! tam sınır kabul, bir bayt fazlası hedefe dokunmadan ret.

use dil::kaynagi_derle;
use dil::yorumlayici::{self, IzKaydedenIo, IzYenidenOynatici, ToplayanIo};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[path = "semantic_regresyon/native_web_kopma.rs"]
mod kopma;

static SIRA: AtomicU64 = AtomicU64::new(0);

fn hermetik(kaynak: &str, io: &mut ToplayanIo) -> Vec<String> {
    let program = kaynagi_derle(kaynak).expect("K-389 kaynağı derlenmeli");
    yorumlayici::calistir_io(&program, io).expect("K-389 programı çalışmalı");
    io.cikti.clone()
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

/// RFC 2045 biçiminde 76 karakterlik CRLF satırları.
fn mime_base64(veri: &[u8]) -> String {
    base64(veri)
        .as_bytes()
        .chunks(76)
        .map(|s| format!("{}\r\n", std::str::from_utf8(s).unwrap()))
        .collect()
}

fn sha256(veri: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(veri)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// UTF-8 olmayan, her bayt değerini taşıyan deterministik ikili içerik.
fn ikili(boy: usize) -> Vec<u8> {
    let mut durum = 0x9e37_79b9u32;
    (0..boy)
        .map(|_| {
            durum ^= durum << 13;
            durum ^= durum >> 17;
            durum ^= durum << 5;
            (durum >> 24) as u8
        })
        .collect()
}

#[test]
fn bellekte_sha256_nist_vektorleri_ve_dosya_ozetiyle_esit() {
    let kaynak = r#"
sifir "" olsun
uclu "abc" olsun
uzun "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq" olsun
turkce "çğıöşü" olsun
sifirin sha256 özeti yaz
uclunun sha256 özeti yaz
uzunun sha256 özeti yaz
turkcenin sha256 özeti yaz
okuma "a.txt" dosyasının sha256 özetini almayı dene olsun
okuma başarılıysa
    dosya_ozeti okumanın değeri olsun
    bellek_ozeti uclunun sha256 özeti olsun
    dosya_ozeti bellek_ozetine eşitse
        "eşit" yaz
"#;
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert("a.txt".into(), "abc".into());
    assert_eq!(
        hermetik(kaynak, &mut io),
        [
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            sha256("çğıöşü".as_bytes()).as_str(),
            "eşit",
        ]
    );
}

#[test]
fn sha256_ozeti_metin_disinda_tur_hatasi() {
    let hata = kaynagi_derle("sayi 5 olsun\nsayinin sha256 özeti yaz\n")
        .expect_err("TamSayı özeti derlenmemeli");
    assert!(hata.kod.starts_with('T'), "{hata:?}");
}

#[test]
fn hermetik_cozerek_yazma_ekleme_ve_c032_verisi() {
    let kaynak = r#"
işlem sonucuYaz
    sonucu Metin sözlüğü sonucu olarak al
    değer döndürmez
    sonucu başarılıysa
        tutamak sonucun değeri olsun
        tutamağın "bayt" değeri yaz
        tutamağın "sha256" değeri yaz
    değilse
        hata sonucun hatası olsun
        hatanın kodu yaz
        hatanın verisi yaz

işlem yeniYaz
    parcasi Metin olarak al
    yolu Metin olarak al
    değer döndürmez
    sonuç parcasi base64 çözerek yolu dosyasına yazmayı dene olsun
    sonuç ile sonucuYaz

işlem sonaEkle
    parcasi Metin olarak al
    yolu Metin olarak al
    değer döndürmez
    sonuç parcasi base64 çözerek yolu dosyasına eklemeyi dene olsun
    sonuç ile sonucuYaz

"AAEC/w==" ve "ek.bin" ile yeniYaz
"AAEC/w==" ve "ek.bin" ile yeniYaz
satirli "satirli.b64" dosyasının 0 baytından 100 baytını metin olarak okumayı dene olsun
satirli başarılıysa
    icerik satirlinin değeri olsun
    icerik ve "akis.bin" ile sonaEkle
"R0g=" ve "akis.bin" ile sonaEkle
"QUJD!EVG" ve "bozuk.bin" ile yeniYaz
"QUJ" ve "bozuk.bin" ile yeniYaz
"QQ==QQ==" ve "bozuk.bin" ile yeniYaz
"QR==" ve "bozuk.bin" ile yeniYaz
qp_okuma "govde.qp" dosyasının 0 baytından 100 baytını metin olarak okumayı dene olsun
qp_okuma başarılıysa
    govde qp_okumanın değeri olsun
    cozum govde quoted_printable çözerek "qp.bin" dosyasına yazmayı dene olsun
    cozum başarılıysa
        cozumun değeri yaz
bozuk_qp "ab=G1" olsun
deneme bozuk_qp quoted_printable çözerek "qp2.bin" dosyasına yazmayı dene olsun
deneme başarısızsa
    hata denemenin hatası olsun
    hatanın verisi yaz
"#;
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar
        .insert("satirli.b64".into(), "QUJD\r\nREVG\r\n".into());
    io.dosyalar
        .insert("govde.qp".into(), "A=C3=A7=\r\nB =   \r\n=3D\r\nson".into());
    let cikti = hermetik(kaynak, &mut io);
    let beklenen_qp = "AçB =\r\nson".as_bytes();
    let beklenen: Vec<String> = vec![
        "4".into(),
        sha256(&[0, 1, 2, 255]),
        "C013".into(),
        "".into(),
        "6".into(),
        sha256(b"ABCDEF"),
        "8".into(),
        sha256(b"ABCDEFGH"),
        "C032".into(),
        "tür: alfabe, konum: 4".into(),
        "C032".into(),
        "tür: uzunluk, konum: 3".into(),
        "C032".into(),
        "tür: dolgu, konum: 4".into(),
        "C032".into(),
        "tür: dolgu, konum: 3".into(),
        format!(
            "yol: qp.bin, sha256: {}, bayt: {}",
            sha256(beklenen_qp),
            beklenen_qp.len()
        ),
        "tür: kaçış, konum: 2".into(),
    ];
    assert_eq!(cikti, beklenen);
    assert_eq!(io.ikili_dosyalar["qp.bin"], beklenen_qp);
    assert_eq!(io.ikili_dosyalar["ek.bin"], [0, 1, 2, 255]);
    assert_eq!(io.ikili_dosyalar["akis.bin"], b"ABCDEFGH");
    assert!(
        !io.ikili_dosyalar.contains_key("bozuk.bin") && !io.ikili_dosyalar.contains_key("qp2.bin"),
        "bozuk kodlamada hedef oluşmaz"
    );
}

#[test]
fn hermetik_aralikli_okuma_utf8_bolunmesi_ve_gecersiz_aralik() {
    let kaynak = r#"
işlem oku
    yolu Metin olarak al
    konumu TamSayı olarak al
    adedi TamSayı olarak al
    değer döndürmez
    okuma yolu dosyasının konumu baytından adedi baytını metin olarak okumayı dene olsun
    okuma başarılıysa
        okumanın değeri yaz
    değilse
        hata okumanın hatası olsun
        hatanın kodu yaz
        hatanın verisi yaz
    kodlu yolu dosyasının konumu baytından adedi baytını base64 okumayı dene olsun
    kodlu başarılıysa
        kodlunun değeri yaz
    değilse
        hata kodlunun hatası olsun
        hatanın kodu yaz

"ileti.eml" ve 0 ve 9 ile oku
"ileti.eml" ve 9 ve 2 ile oku
"ileti.eml" ve 9 ve 1 ile oku
"ileti.eml" ve 100 ve 5 ile oku
"ileti.eml" ve -1 ve 5 ile oku
"ileti.eml" ve 0 ve 0 ile oku
"ileti.eml" ve 0 ve 1048577 ile oku
"yok.eml" ve 0 ve 1 ile oku
"#;
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar
        .insert("ileti.eml".into(), "Konu: x\r\nçay".into());
    assert_eq!(
        hermetik(kaynak, &mut io),
        [
            "Konu: x\r\n",
            "S29udTogeA0K",
            "ç",
            "w6c=",
            "C032",
            "tür: utf8, konum: 9",
            "ww==",
            "",
            "",
            "C013",
            "",
            "C013",
            "C013",
            "",
            "C013",
            "C013",
            "",
            "C013",
            "C013",
            "",
            "C013",
        ]
    );
}

#[test]
fn hermetik_cozme_tavani_bir_mib_ayirmadan_once() {
    let kaynak = r#"
okuma "buyuk.b64" dosyasını okumayı dene olsun
okuma başarılıysa
    icerik okumanın değeri olsun
    sonuç icerik base64 çözerek "ek.bin" dosyasına yazmayı dene olsun
    sonuç başarısızsa
        hata sonucun hatası olsun
        hatanın verisi yaz
"#;
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar
        .insert("buyuk.b64".into(), mime_base64(&vec![7u8; 1024 * 1024 + 1]));
    assert_eq!(hermetik(kaynak, &mut io), ["tür: boyut_aşımı"]);
    assert!(io.ikili_dosyalar.is_empty());
    io.cikti.clear();
    io.dosyalar
        .insert("buyuk.b64".into(), mime_base64(&vec![7u8; 1024 * 1024]));
    assert_eq!(hermetik(kaynak, &mut io), Vec::<String>::new());
    assert_eq!(io.ikili_dosyalar["ek.bin"].len(), 1024 * 1024);
}

#[test]
fn iz_icerik_tasimaz_yazim_oynatilir_aralik_oynatilmaz() {
    let gizli = b"GIZLI-EK-ICERIGI-K389";
    let kaynak = format!(
        r#"
kodlu "{}" olsun
sonuç kodlu base64 çözerek "ek.bin" dosyasına yazmayı dene olsun
sonuç başarılıysa
    sonucun değeri yaz
"#,
        base64(gizli)
    );
    let program = kaynagi_derle(&kaynak).unwrap();
    let mut kaydeden = IzKaydedenIo::yeni(ToplayanIo::yeni(Vec::new()));
    yorumlayici::calistir_io(&program, &mut kaydeden).unwrap();
    let iz = kaydeden.iz_metni().unwrap();
    assert!(iz.contains("dosyaya_bayt_yaz"), "{iz}");
    // İz alanları onaltılık kodludur.
    let hex = |m: &str| m.bytes().map(|b| format!("{b:02x}")).collect::<String>();
    assert!(iz.contains(&hex(&sha256(gizli))), "iz özet taşır: {iz}");
    assert!(
        !iz.contains(&hex("GIZLI")) && !iz.contains(&hex(&base64(gizli))),
        "iz içerik taşımaz: {iz}"
    );
    let mut oynatici = IzYenidenOynatici::yeni(&iz).unwrap();
    yorumlayici::calistir_io(&program, &mut oynatici).unwrap();
    assert_eq!(
        oynatici.cikti,
        [format!("yol: ek.bin, sha256: {}, bayt: 21", sha256(gizli))]
    );
    oynatici.bitir().unwrap();

    let okuma = kaynagi_derle(
        "okuma \"a.txt\" dosyasının 0 baytından 3 baytını metin olarak okumayı dene olsun\nokuma başarılıysa\n    icerik okumanın değeri olsun\n    icerigin sha256 özeti yaz\n",
    )
    .unwrap();
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.dosyalar.insert("a.txt".into(), "GIZLI".into());
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    yorumlayici::calistir_io(&okuma, &mut kaydeden).unwrap();
    let iz = kaydeden.iz_metni().unwrap();
    assert!(
        iz.contains("dosya_araligi_oku") && !iz.contains(&hex("GIZ")),
        "{iz}"
    );
    let mut oynatici = IzYenidenOynatici::yeni(&iz).unwrap();
    let _ = yorumlayici::calistir_io(&okuma, &mut oynatici);
    let hata = oynatici.bitir().expect_err("içeriksiz okuma oynatılamaz");
    assert!(hata.contains("yeniden oynatılamaz"), "{hata}");
}

// ---------- gerçek `dil` süreci ----------

struct Proje(PathBuf);

impl Proje {
    fn yeni(yetkinlikler: &str, ana: &str) -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-k389-kodlanmis-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&yol).unwrap();
        std::fs::write(
            yol.join("proje.dil"),
            format!(
                "proje \"kodlanmis\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler {yetkinlikler} listesi olsun\nağ_hedefleri boş liste olsun\n"
            ),
        )
        .unwrap();
        std::fs::write(yol.join("ana.dil"), ana).unwrap();
        Self(yol)
    }

    fn calistir(&self) -> (bool, String, String) {
        let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
            .args(["çalıştır", self.0.to_str().unwrap()])
            .output()
            .unwrap();
        (
            cikti.status.success(),
            String::from_utf8_lossy(&cikti.stdout).into_owned(),
            String::from_utf8_lossy(&cikti.stderr).into_owned(),
        )
    }
}

impl Drop for Proje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const DOSYA: &str = "\"dosya-okuma\", \"dosya-yazma\"";

/// A2c blok dosyası biçiminde ileti: QP gövde + büyük base64 ek. Dönen
/// (ileti, qp_bas, qp_son, b64_bas, b64_son).
fn ileti(ek: &[u8]) -> (Vec<u8>, usize, usize, usize, usize) {
    let mut m = b"From: a@ornek.test\r\nSubject: ek\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"s\"\r\n\r\n--s\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\n".to_vec();
    let qp_bas = m.len();
    m.extend_from_slice(b"Merhaba =C3=A7ok uzun bir sat=\r\nir ve =3D i=C5=9Faret\r\n");
    let qp_son = m.len();
    m.extend_from_slice(
        b"--s\r\nContent-Type: application/pdf\r\nContent-Transfer-Encoding: base64\r\n\r\n",
    );
    let b64_bas = m.len();
    m.extend_from_slice(mime_base64(ek).as_bytes());
    let b64_son = m.len();
    m.extend_from_slice(b"--s--\r\n");
    (m, qp_bas, qp_son, b64_bas, b64_son)
}

#[test]
fn mime_iletisinden_buyuk_ikili_ek_aralikli_okunup_bayt_kesin_cikarilir() {
    let ek = ikili(2_500_000);
    let (m, qp_bas, qp_son, b64_bas, b64_son) = ileti(&ek);
    // Satır başına 78 bayt; 1 MiB altındaki en büyük tam satır katı. Her
    // ekleme çağrısı tam 4'lük gruplardan oluşur (RFC-0043 §5b).
    let adim = 78 * (1024 * 1024 / 78);
    let tam = (b64_son - b64_bas) / adim;
    let son = (b64_son - b64_bas) % adim;
    let ana = format!(
        r#"konum {b64_bas} olsun
adim {adim} olsun
{tam} kez tekrarla
    okuma "ileti.eml" dosyasının konum baytından adim baytını metin olarak okumayı dene olsun
    okuma başarılıysa
        parca okumanın değeri olsun
        ekleme parca base64 çözerek "ekler/ek.pdf" dosyasına eklemeyi dene olsun
        ekleme başarısızsa
            eklemenin hatası yaz
    konum konum ile adimin toplamı olsun
okuma "ileti.eml" dosyasının konum baytından {son} baytını metin olarak okumayı dene olsun
okuma başarılıysa
    parca okumanın değeri olsun
    ekleme parca base64 çözerek "ekler/ek.pdf" dosyasına eklemeyi dene olsun
    ekleme başarılıysa
        eklemenin değeri yaz
govde_okuma "ileti.eml" dosyasının {qp_bas} baytından {qp_uzun} baytını metin olarak okumayı dene olsun
govde_okuma başarılıysa
    govde govde_okumanın değeri olsun
    yazma govde quoted_printable çözerek "ekler/govde.txt" dosyasına yazmayı dene olsun
    yazma başarılıysa
        yazmanın değeri yaz
ozet "ekler/ek.pdf" dosyasının sha256 özetini almayı dene olsun
ozet başarılıysa
    ozetin değeri yaz
kodlu "ileti.eml" dosyasının 0 baytından 16 baytını base64 okumayı dene olsun
kodlu başarılıysa
    kodlunun değeri yaz
"#,
        qp_uzun = qp_son - qp_bas
    );
    let proje = Proje::yeni(DOSYA, &ana);
    std::fs::create_dir_all(proje.0.join("ekler")).unwrap();
    std::fs::write(proje.0.join("ileti.eml"), &m).unwrap();
    let (tamam, stdout, stderr) = proje.calistir();
    assert!(tamam, "{stdout}\n{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        satirlar[0],
        format!(
            "yol: ekler/ek.pdf, sha256: {}, bayt: {}",
            sha256(&ek),
            ek.len()
        )
    );
    let govde = "Merhaba çok uzun bir satir ve = işaret\r\n".as_bytes();
    assert_eq!(
        satirlar[1],
        format!(
            "yol: ekler/govde.txt, sha256: {}, bayt: {}",
            sha256(govde),
            govde.len()
        )
    );
    assert_eq!(satirlar[2], sha256(&ek), "dosya özeti tutamakla eşit");
    assert_eq!(satirlar[3], base64(&m[..16]));
    assert!(std::fs::read(proje.0.join("ekler/ek.pdf")).unwrap() == ek);
    assert_eq!(
        std::fs::read(proje.0.join("ekler/govde.txt")).unwrap(),
        govde
    );
}

#[test]
fn sekiz_bit_govde_metin_okumada_utf8_base64_okumada_basari() {
    let proje = Proje::yeni(
        "\"dosya-okuma\"",
        r#"okuma "blok.parca" dosyasının 0 baytından 64 baytını metin olarak okumayı dene olsun
okuma başarısızsa
    hata okumanın hatası olsun
    hatanın kodu yaz
    hatanın verisi yaz
kodlu "blok.parca" dosyasının 0 baytından 64 baytını base64 okumayı dene olsun
kodlu başarılıysa
    kodlunun değeri yaz
"#,
    );
    let blok = b"Subject: 8bit\r\n\r\n\xe7ay \xff\x00\r\n";
    std::fs::write(proje.0.join("blok.parca"), blok).unwrap();
    let (tamam, stdout, stderr) = proje.calistir();
    assert!(tamam, "{stdout}\n{stderr}");
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        ["C032", "tür: utf8, konum: 17", base64(blok).as_str()]
    );
}

#[test]
fn no_clobber_tavan_yol_gecisi_ve_eylem_reddi_hedefi_bozmaz() {
    let ana = r#"işlem sonucuYaz
    sonucu Metin sözlüğü sonucu olarak al
    değer döndürmez
    sonucu başarılıysa
        sonucun değeri yaz
    değilse
        hata sonucun hatası olsun
        hatanın kodu ile " " ile hatanın mesajı yaz

işlem yeniYaz
    yolu Metin olarak al
    değer döndürmez
    sonuç "QUJD" base64 çözerek yolu dosyasına yazmayı dene olsun
    sonuç ile sonucuYaz

işlem sonaEkle
    yolu Metin olarak al
    değer döndürmez
    sonuç "QUJD" base64 çözerek yolu dosyasına eklemeyi dene olsun
    sonuç ile sonucuYaz

eylem eylemdeYaz
    Metin sözlüğü sonucu döndürür
    "QUJD" base64 çözerek "eylem.bin" dosyasına yazmayı dene döndür

eylem eylemdeGeriAl
    Metin sözlüğü sonucu döndürür
    sonuç "QUJD" base64 çözerek "geri.bin" dosyasına yazmayı dene olsun
    "QU" base64 çözerek "geri2.bin" dosyasına yazmayı dene döndür

eylem eylemdeEkle
    Metin sözlüğü sonucu döndürür
    "QUJD" base64 çözerek "akis.bin" dosyasına eklemeyi dene döndür

"var.bin" ile yeniYaz
"dolu.bin" ile sonaEkle
"../kacis.bin" ile yeniYaz
"bag/kacis.bin" ile yeniYaz
"yok/klasor.bin" ile yeniYaz
"dizin" ile sonaEkle
"dizin" ile yeniYaz
birinci eylemdeYaz olsun
birinci ile sonucuYaz
ikinci eylemdeGeriAl olsun
ikinci ile sonucuYaz
ucuncu eylemdeEkle olsun
ucuncu ile sonucuYaz
"#;
    let proje = Proje::yeni(DOSYA, ana);
    let dis = proje.0.with_extension("dis");
    std::fs::create_dir_all(&dis).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&dis, proje.0.join("bag")).unwrap();
    std::fs::create_dir_all(proje.0.join("dizin")).unwrap();
    std::fs::write(proje.0.join("var.bin"), b"eski").unwrap();
    // K-395: hedef tavanı 16 → 32 MiB.
    let tavan = 32 * 1024 * 1024;
    std::fs::write(proje.0.join("dolu.bin"), vec![1u8; tavan - 2]).unwrap();
    let (tamam, stdout, stderr) = proje.calistir();
    let _ = std::fs::remove_dir_all(&dis);
    assert!(tamam, "{stdout}\n{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert_eq!(satirlar[0], "C013 \"var.bin\" hedef dosyası zaten var");
    assert!(
        satirlar[1].starts_with("C013") && satirlar[1].contains("32 MiB"),
        "{satirlar:?}"
    );
    assert!(satirlar[2].contains("proje dosya sınırı"), "{satirlar:?}");
    if cfg!(unix) {
        assert!(satirlar[3].contains("sembolik bağ"), "{satirlar:?}");
    } else {
        assert!(satirlar[3].contains("hedef klasör"), "{satirlar:?}");
    }
    assert!(satirlar[4].contains("hedef klasör"), "{satirlar:?}");
    assert!(satirlar[5].contains("normal dosya değil"), "{satirlar:?}");
    assert!(satirlar[6].contains("zaten var"), "{satirlar:?}");
    assert!(satirlar[7].starts_with("yol: eylem.bin"), "{satirlar:?}");
    assert!(satirlar[8].starts_with("C032"), "{satirlar:?}");
    assert!(satirlar[9].contains("eylem içinde"), "{satirlar:?}");
    assert_eq!(std::fs::read(proje.0.join("var.bin")).unwrap(), b"eski");
    assert_eq!(
        std::fs::metadata(proje.0.join("dolu.bin")).unwrap().len(),
        (tavan - 2) as u64,
        "tavanı aşan ekleme hiç yazmaz"
    );
    assert_eq!(std::fs::read(proje.0.join("eylem.bin")).unwrap(), b"ABC");
    assert!(
        !proje.0.join("geri.bin").exists(),
        "başarısız Sonuç dönen eylem yeni dosyayı geri alır"
    );
    assert!(!proje.0.join("akis.bin").exists());
}

#[test]
fn yetkinliksiz_projede_cozerek_yazma_derlenmez() {
    let proje = Proje::yeni(
        "\"dosya-okuma\"",
        "s \"QUJD\" base64 çözerek \"a.bin\" dosyasına yazmayı dene olsun\n",
    );
    let (tamam, stdout, stderr) = proje.calistir();
    assert!(!tamam, "{stdout}");
    assert!(
        stderr.contains("T054") || stdout.contains("T054"),
        "{stderr}"
    );
    assert!(!proje.0.join("a.bin").exists());
}

fn istek(kapi: u16, hedef: &str) -> (String, Vec<u8>) {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(5)).unwrap();
    akis.set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    write!(
        akis,
        "GET {hedef} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.9;proto=https;host=panel.example\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut ham = Vec::new();
    akis.read_to_end(&mut ham).unwrap();
    let son = ham.windows(4).position(|p| p == b"\r\n\r\n").unwrap() + 4;
    (
        String::from_utf8(ham[..son].to_vec()).unwrap(),
        ham[son..].to_vec(),
    )
}

#[test]
fn cozulen_ek_tutamagi_rfc0041_dosya_yanitina_baglanir() {
    // Gelen ek bağlantı/CLI tarafında çözülür (GET salt okumadır, T045);
    // webmail tutamağın `yol` alanını dosya yanıtına verir.
    let ek = ikili(200_000);
    let ana = format!(
        r#"okuma "parca.b64" dosyasının 0 baytından {boy} baytını metin olarak okumayı dene olsun
okuma başarılıysa
    parca okumanın değeri olsun
    yazma parca base64 çözerek "ekler/ek.bin" dosyasına yazmayı dene olsun
    yazma başarılıysa
        tutamak yazmanın değeri olsun
        yol tutamağın "yol" değeri olsun
        "ekler/tutamak.txt" dosyasına yol yaz

8080 kapısında sunucu başlat

GET "/ek" adresine istek geldiğinde
    herkese açık
    yol isteğin "yol" değeri olsun
    yanit yol dosyasını "application/pdf" türünde "Ek.pdf" adıyla ek olarak göndermeyi dene olsun
    yanit başarısızsa
        "gönderilemedi" yanıtını 404 durumuyla gönder
"#,
        boy = mime_base64(&ek).len()
    );
    let proje = Proje::yeni("\"ağ-sunucusu\", \"dosya-okuma\", \"dosya-yazma\"", &ana);
    std::fs::create_dir_all(proje.0.join("ekler")).unwrap();
    std::fs::write(proje.0.join("parca.b64"), mime_base64(&ek)).unwrap();
    let kapi = kopma::bos_kapi();
    let mut sunucu = kopma::worker_baslat(&proje.0, kapi, Stdio::null);
    let yol = std::fs::read_to_string(proje.0.join("ekler/tutamak.txt")).unwrap();
    let (basliklar, govde) = istek(kapi, &format!("/ek?yol={}", yol.trim().replace('/', "%2F")));
    let _ = sunucu.kill();
    let _ = sunucu.wait();
    assert_eq!(yol.trim(), "ekler/ek.bin");
    assert!(basliklar.starts_with("HTTP/1.1 200 OK\r\n"), "{basliklar}");
    assert!(
        basliklar.contains("Content-Type: application/pdf"),
        "{basliklar}"
    );
    assert!(govde == ek, "indirilen ek bayt bayt aynı");
}

/// K-395 (ADR-167 eki, ZM-F039): 25 MB iletinin ~18 MB eki için hedef tavanı
/// 32 MiB. Tam 32 MiB'a ulaşan ekleme kabul, bir bayt fazlası reddedilir ve
/// hedef değişmez; eski 16 MiB sınırının ötesi artık geçerlidir.
#[test]
fn eklemeyi_dene_hedef_tavani_32_mib_tam_sinirda() {
    let tavan: usize = 32 * 1024 * 1024;
    let ana = r#"işlem sonaEkle
    yolu Metin olarak al
    değer döndürmez
    sonuç "QUJD" base64 çözerek yolu dosyasına eklemeyi dene olsun
    sonuç başarılıysa
        sonucun değeri yaz
    değilse
        hata sonucun hatası olsun
        hatanın kodu ile " " ile hatanın mesajı yaz

"tam.bin" ile sonaEkle
"tasan.bin" ile sonaEkle
"eski-sinir.bin" ile sonaEkle
"#;
    let proje = Proje::yeni(DOSYA, ana);
    std::fs::write(proje.0.join("tam.bin"), vec![7u8; tavan - 3]).unwrap();
    std::fs::write(proje.0.join("tasan.bin"), vec![7u8; tavan - 2]).unwrap();
    std::fs::write(proje.0.join("eski-sinir.bin"), vec![7u8; 16 * 1024 * 1024]).unwrap();
    let (tamam, stdout, stderr) = proje.calistir();
    assert!(tamam, "{stdout}\n{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    let mut tam = vec![7u8; tavan - 3];
    tam.extend_from_slice(b"ABC");
    assert_eq!(
        satirlar[0],
        format!("yol: tam.bin, sha256: {}, bayt: {tavan}", sha256(&tam)),
        "tam 32 MiB kabul"
    );
    assert!(
        satirlar[1].starts_with("C013") && satirlar[1].contains("32 MiB"),
        "{satirlar:?}"
    );
    assert!(
        satirlar[2].ends_with(&format!("bayt: {}", 16 * 1024 * 1024 + 3)),
        "16 MiB ötesi artık geçerli: {satirlar:?}"
    );
    assert_eq!(
        std::fs::metadata(proje.0.join("tam.bin")).unwrap().len(),
        tavan as u64
    );
    assert_eq!(
        std::fs::metadata(proje.0.join("tasan.bin")).unwrap().len(),
        (tavan - 2) as u64,
        "bir bayt fazlası hedefe hiç yazmaz"
    );
}
