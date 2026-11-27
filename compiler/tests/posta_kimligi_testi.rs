//! K-390 (RFC-0044 B1b/B2b, ADR-177): kurumsal alan (PSL) ve DKIM imza
//! doğrulama ilkelinin Zee yüzeyi. RSA ve Ed25519 vektörleri bağımsız
//! uygulamayla (openssl 4.1) üretildi: `fixtures/dkim/dkim-vektorleri.tsv`.

use std::sync::Arc;

use dil::kamu_son_ekleri::KamuSonEkleri;
use dil::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};
use dil::yorumlayici::{calistir_io, ToplayanIo};

/// Zee'nin kanonikleştirdiği imza girdisi (relaxed başlıklar + `b=` boş);
/// Zee satır sonunu `\n` yazar, ilkel imza girdisinde CRLF kurar (openssl
/// vektörleri CRLF'li baytlar üzerinde imzalandı).
const IMZA_VERISI: &str = "from:Zee Test <gonderen@ornek.test>\nsubject:Merhaba\ndkim-signature:v=1; a=rsa-sha256; c=relaxed/relaxed; d=ornek.test; s=z2026a; h=from:subject; bh=47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=; b=";

struct Vektor {
    ad: String,
    bit: String,
    p: String,
    b: String,
    pkcs1: String,
}

fn vektorler() -> Vec<Vektor> {
    include_str!("fixtures/dkim/dkim-vektorleri.tsv")
        .lines()
        .filter(|s| !s.starts_with('#') && !s.is_empty())
        .map(|s| {
            let a: Vec<&str> = s.split('\t').collect();
            Vektor {
                ad: a[0].into(),
                bit: a[1].into(),
                p: a[2].into(),
                b: a[3].into(),
                pkcs1: a[4].into(),
            }
        })
        .collect()
}

fn vektor(ad: &str) -> Vektor {
    vektorler().into_iter().find(|v| v.ad == ad).expect(ad)
}

fn zee_metni(m: &str) -> String {
    format!(
        "\"{}\"",
        m.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}

/// DKIM sonucunu tek satıra indirir: `doğru 2048 rsa-sha256` ya da `C034 tür`.
fn dkim(b: &str, veri: &str, p: &str, a: &str) -> String {
    let kaynak = format!(
        "imza {} olsun\nveri {} olsun\nanahtar {} olsun\nalgoritma {} olsun\n\
sonuç imza dkim imzasını veri verisi için anahtar anahtarıyla algoritma olarak doğrulamayı dene olsun\n\
sonuç başarılıysa\n    s sonucun değeri olsun\n    s \"doğrulandı\" değeri yaz\n    \
s \"anahtar_biti\" değeri yaz\n    s \"algoritma\" değeri yaz\n\
değilse\n    hata sonucun hatası olsun\n    veri2 hatanın verisi olsun\n    \
hatanın kodunu yaz\n    veri2 \"tür\" değeri yaz\n",
        zee_metni(b),
        zee_metni(veri),
        zee_metni(p),
        zee_metni(a)
    );
    calistir(&kaynak, &mut ToplayanIo::yeni(Vec::new())).join(" ")
}

fn calistir(kaynak: &str, io: &mut ToplayanIo) -> Vec<String> {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    calistir_io(&program, io).expect("çalışmalı");
    io.cikti.clone()
}

#[test]
fn rsa_sha256_1024_2048_4096_spki_ve_pkcs1_dogrulanir() {
    for ad in ["rsa1024", "rsa2048", "rsa4096"] {
        let v = vektor(ad);
        let beklenen = format!("doğru {} rsa-sha256", v.bit);
        assert_eq!(
            dkim(&v.b, IMZA_VERISI, &v.p, "rsa-sha256"),
            beklenen,
            "{ad}"
        );
        assert_eq!(
            dkim(&v.b, IMZA_VERISI, &v.pkcs1, "RSA-SHA256"),
            beklenen,
            "{ad} pkcs1"
        );
        // `p=` ve `b=` DNS/başlıkta katlanmış gelebilir (FWS atlanır).
        let katli = format!("{} \r\n {}", &v.p[..20], &v.p[20..]);
        assert_eq!(dkim(&v.b, IMZA_VERISI, &katli, "rsa-sha256"), beklenen);
        let degisik = IMZA_VERISI.replace("Merhaba", "Merhabа");
        assert_eq!(
            dkim(&v.b, &degisik, &v.p, "rsa-sha256"),
            format!("yanlış {} rsa-sha256", v.bit),
            "{ad} değiştirilmiş başlık"
        );
    }
}

#[test]
fn ed25519_sha256_openssl_vektoru_dogrulanir() {
    let v = vektor("ed25519");
    assert_eq!(
        dkim(&v.b, IMZA_VERISI, &v.p, "ed25519-sha256"),
        "doğru 256 ed25519-sha256"
    );
    assert_eq!(
        dkim(
            &v.b,
            &IMZA_VERISI.replace("z2026a", "z2026b"),
            &v.p,
            "ed25519-sha256"
        ),
        "yanlış 256 ed25519-sha256"
    );
    // Ed25519 anahtarı RSA olarak (ve tersi) yorumlanamaz.
    let rsa = vektor("rsa2048");
    assert_eq!(
        dkim(&v.b, IMZA_VERISI, &v.p, "rsa-sha256"),
        "C034 anahtar_bozuk"
    );
    assert_eq!(
        dkim(&v.b, IMZA_VERISI, &rsa.p, "ed25519-sha256"),
        "C034 anahtar_bozuk"
    );
}

#[test]
fn rsa_sha1_kisa_uzun_iptal_ve_bozuk_imza_c034_reddidir() {
    let v1024 = vektor("rsa1024");
    let v2048 = vektor("rsa2048");
    let kisa = vektor("rsa1023");
    let uzun = vektor("rsa4104");
    let vakalar = [
        (
            v2048.b.as_str(),
            v2048.p.as_str(),
            "rsa-sha1",
            "algoritma_reddedildi",
        ),
        (
            v2048.b.as_str(),
            v2048.p.as_str(),
            "rsa-sha512",
            "algoritma_bilinmiyor",
        ),
        (
            kisa.b.as_str(),
            kisa.p.as_str(),
            "rsa-sha256",
            "anahtar_kısa",
        ),
        (
            uzun.b.as_str(),
            uzun.p.as_str(),
            "rsa-sha256",
            "anahtar_uzun",
        ),
        (v2048.b.as_str(), "", "rsa-sha256", "anahtar_iptal"),
        (v2048.b.as_str(), "MIIB%%%", "rsa-sha256", "anahtar_bozuk"),
        ("bozuk!", v2048.p.as_str(), "rsa-sha256", "imza_bozuk"),
        // 1024 bitlik imza 2048 bitlik anahtarla: boy uyuşmazlığı.
        (
            v1024.b.as_str(),
            v2048.p.as_str(),
            "rsa-sha256",
            "imza_bozuk",
        ),
    ];
    for (b, p, a, tur) in vakalar {
        assert_eq!(
            dkim(b, IMZA_VERISI, p, a),
            format!("C034 {tur}"),
            "{a} {tur}"
        );
    }
}

/// K-419 (dkim_anahtari fuzz kâhini bulgusu): ring'in çözemediği RSA
/// bileşeni (kanonik olmayan/çift/1 üs, çift modül) doğru imzayı sessizce
/// `yanlış` yapmaz; spec/31 §3 gereği C034 `anahtar_bozuk` rettir.
#[test]
fn ring_disi_rsa_bileseni_c034_anahtar_bozuk_reddidir() {
    let v = vektor("rsa2048");
    let pkcs1 = dil::kodlama::base64_mime_coz(&v.pkcs1, 2048).unwrap();
    assert_eq!(&pkcs1[pkcs1.len() - 5..], [0x02, 0x03, 0x01, 0x00, 0x01]);
    let govde = &pkcs1[4..pkcs1.len() - 5];
    let anahtar = |modul: &[u8], us: &[u8]| {
        let mut ic = modul.to_vec();
        ic.push(0x02);
        ic.push(us.len() as u8);
        ic.extend_from_slice(us);
        let mut der = vec![0x30, 0x82, (ic.len() >> 8) as u8, ic.len() as u8];
        der.extend_from_slice(&ic);
        dil::kodlama::base64_kodla(&der)
    };
    assert_eq!(
        dkim(
            &v.b,
            IMZA_VERISI,
            &anahtar(govde, &[0x01, 0x00, 0x01]),
            "rsa-sha256"
        ),
        "doğru 2048 rsa-sha256"
    );
    let mut cift_modul = govde.to_vec();
    *cift_modul.last_mut().unwrap() &= 0xfe;
    for p in [
        anahtar(govde, &[0x00, 0x01, 0x00, 0x01]),
        anahtar(govde, &[0x01, 0x00, 0x00]),
        anahtar(govde, &[0x01]),
        anahtar(govde, &[0x02, 0x00, 0x00, 0x00, 0x01]),
        anahtar(&cift_modul, &[0x01, 0x00, 0x01]),
    ] {
        assert_eq!(
            dkim(&v.b, IMZA_VERISI, &p, "rsa-sha256"),
            "C034 anahtar_bozuk",
            "{p}"
        );
    }
}

/// RFC 6376 §3.4.3 (boş simple gövde CRLF), §3.4.4 (boş relaxed gövde) ve
/// Ek A.2 örnek gövdesi; Zee satır sonunu `\n` yazar, özet CRLF üzerindedir.
#[test]
fn dkim_govde_ozeti_rfc6376_vektorleriyle_bh_degerini_verir() {
    let kaynak = "\
simple \"\\n\" olsun
relaxed \"\" olsun
ornek \"Hi.\\n\\nWe lost the game. Are you hungry yet?\\n\\nJoe.\\n\" olsun
simple gövdesinin dkim gövde özeti yaz
relaxed gövdesinin dkim gövde özeti yaz
bh ornek gövdesinin dkim gövde özeti olsun
bh yaz
bh \"2jUSOH9NhtVGCQWNr9BrIAPreKQjO6Sn7XIkfJVOzv8=\" e eşitse
    \"bh tuttu\" yaz
";
    let cikti = calistir(kaynak, &mut ToplayanIo::yeni(Vec::new()));
    assert_eq!(
        cikti,
        [
            "frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=",
            "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
            "2jUSOH9NhtVGCQWNr9BrIAPreKQjO6Sn7XIkfJVOzv8=",
            "bh tuttu"
        ]
    );
}

const PSL: &str = "com\nco.uk\n*.ck\n!www.ck\n";

const KURUMSAL: &str = "\
adlar \"posta.ornek.co.uk\", \"Ornek.COM.\", \"co.uk\", \"a.b.c.ck\", \"www.ck\", \"örnek.com\", \"\" listesi olsun
adlardaki her ad için
    k ad alanının kurumsal alanı olsun
    \"[\" ile k ile \"]\" yaz
";

#[test]
fn kurumsal_alan_kamu_son_ek_listesinden_hesaplanir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.kamu_son_ekleri = Some(Arc::new(KamuSonEkleri::ayristir(PSL.as_bytes()).unwrap()));
    assert_eq!(
        calistir(KURUMSAL, &mut io),
        [
            "[ornek.co.uk]",
            "[ornek.com]",
            "[]",
            "[b.c.ck]",
            "[www.ck]",
            "[]",
            "[]"
        ]
    );
}

#[test]
fn kamu_son_ekleri_yoksa_kurumsal_alan_c034_ile_durur() {
    let program = dil::kaynagi_derle("\"a.ornek.com\" alanının kurumsal alanı yaz\n").unwrap();
    let hata = calistir_io(&program, &mut ToplayanIo::yeni(Vec::new())).expect_err("durmalı");
    assert_eq!(hata.kod, "C034");
    assert!(hata.mesaj.contains("kamu_son_ekleri"), "{}", hata.mesaj);
}

#[test]
fn yetkinlikler_dns_ve_kriptografi_ister() {
    let kurumsal = dil::kaynagi_derle("\"a.ornek.com\" alanının kurumsal alanı yaz\n").unwrap();
    let dkim = dil::kaynagi_derle(
        "x \"\" olsun\ns x dkim imzasını x verisi için x anahtarıyla x olarak doğrulamayı dene olsun\n",
    )
    .unwrap();
    let govde_ozeti = dil::kaynagi_derle("\"\" gövdesinin dkim gövde özeti yaz\n").unwrap();
    for (program, yetkinlik) in [
        (&kurumsal, Yetkinlik::Dns),
        (&dkim, Yetkinlik::Kriptografi),
        (&govde_ozeti, Yetkinlik::Kriptografi),
    ] {
        let yalniz = YetkinlikPolitikasi::proje([yetkinlik].into(), Default::default());
        dil::cozumleyici::yetkinlikleri_denetle(program, &yalniz).expect("izinli");
        let bos = YetkinlikPolitikasi::proje(Default::default(), Default::default());
        assert!(dil::cozumleyici::yetkinlikleri_denetle(program, &bos).is_err());
    }
}

#[test]
fn io_izi_kurumsal_alani_kaydeder_ve_listesiz_oynatir() {
    let program = dil::kaynagi_derle(KURUMSAL).expect("derlenmeli");
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.kamu_son_ekleri = Some(Arc::new(KamuSonEkleri::ayristir(PSL.as_bytes()).unwrap()));
    let mut kaydeden = dil::yorumlayici::IzKaydedenIo::yeni(taban);
    calistir_io(&program, &mut kaydeden).expect("kayıt");
    let iz = kaydeden.iz_metni().expect("iz");
    assert!(iz.contains("kurumsal_alan"));
    let mut oynatici = dil::yorumlayici::IzYenidenOynatici::yeni(&iz).expect("iz okunur");
    calistir_io(&program, &mut oynatici).expect("replay");
    assert_eq!(oynatici.cikti[0], "[ornek.co.uk]");
    assert_eq!(oynatici.cikti.len(), 7);
    oynatici.bitir().expect("iz tükendi");
}

fn bildirim(ek: &str) -> Result<dil::proje::ProjeBildirimi, dil::tani::Tani> {
    dil::proje::bildirimi_oku(&format!(
        "proje \"d\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n{ek}"
    ))
}

#[test]
fn proje_kamu_son_ekleri_dns_ile_birlikte_ve_exact_ozetle_bildirilir() {
    let dns = "yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"127.0.0.1:53\" olsun\ndns_dnssec \"bilgi\" olsun\n";
    let ozet = "a".repeat(64);
    let tamam = bildirim(&format!(
        "{dns}kamu_son_ekleri \"veri/psl.dat\" olsun\nkamu_son_ekleri_özeti \"{ozet}\" olsun\n"
    ))
    .expect("geçerli");
    let psl = tamam.kamu_son_ekleri.expect("bildirim");
    assert_eq!(
        (psl.yol.as_str(), psl.ozet.as_str()),
        ("veri/psl.dat", ozet.as_str())
    );
    assert!(bildirim(dns).unwrap().kamu_son_ekleri.is_none());
    for (ek, parca) in [
        (format!("kamu_son_ekleri \"psl.dat\" olsun\nkamu_son_ekleri_özeti \"{ozet}\" olsun\n"), "yalnız `dns`"),
        (format!("{dns}kamu_son_ekleri \"psl.dat\" olsun\n"), "birlikte"),
        (format!("{dns}kamu_son_ekleri_özeti \"{ozet}\" olsun\n"), "birlikte"),
        (format!("{dns}kamu_son_ekleri \"../psl.dat\" olsun\nkamu_son_ekleri_özeti \"{ozet}\" olsun\n"), "göreli"),
        (format!("{dns}kamu_son_ekleri \"/etc/psl.dat\" olsun\nkamu_son_ekleri_özeti \"{ozet}\" olsun\n"), "göreli"),
        (format!("{dns}kamu_son_ekleri \"psl.dat\" olsun\nkamu_son_ekleri_özeti \"{}\" olsun\n", "A".repeat(64)), "64 küçük"),
        (format!("{dns}kamu_son_ekleri \"psl.dat\" olsun\nkamu_son_ekleri_özeti \"abc\" olsun\n"), "64 küçük"),
    ] {
        let tani = bildirim(&ek).expect_err(&ek);
        assert_eq!(tani.kod, "P015", "{ek}");
        assert!(tani.mesaj.contains(parca), "{ek}: {}", tani.mesaj);
    }
}

fn surec(psl: &[u8], ozet: &str, ad: &str) -> (i32, String, String) {
    let dizin = std::env::temp_dir().join(format!("zee-psl-{}-{ad}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dizin);
    std::fs::create_dir_all(dizin.join("veri")).unwrap();
    std::fs::write(dizin.join("veri/psl.dat"), psl).unwrap();
    std::fs::write(
        dizin.join("proje.dil"),
        format!(
            "proje \"psl\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
             yetkinlikler \"dns\" listesi olsun\ndns_çözücü \"127.0.0.1:53\" olsun\n\
             dns_dnssec \"bilgi\" olsun\nkamu_son_ekleri \"veri/psl.dat\" olsun\n\
             kamu_son_ekleri_özeti \"{ozet}\" olsun\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dizin.join("ana.dil"),
        "\"posta.ornek.co.uk\" alanının kurumsal alanı yaz\n",
    )
    .unwrap();
    let cikti = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(&dizin)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dizin);
    (
        cikti.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&cikti.stdout).into_owned(),
        String::from_utf8_lossy(&cikti.stderr).into_owned(),
    )
}

#[test]
fn gercek_surec_ozeti_tutan_listeyi_yukler_tutmayani_reddeder() {
    use sha2::{Digest, Sha256};
    let ozet: String = Sha256::digest(PSL.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let (kod, cikti, hata) = surec(PSL.as_bytes(), &ozet, "tamam");
    assert_eq!((kod, cikti.as_str()), (0, "ornek.co.uk\n"), "{hata}");
    let (kod, cikti, hata) = surec(b"com\n", &ozet, "ozet");
    assert_eq!(kod, 2, "{cikti}{hata}");
    assert!(hata.contains("SHA-256"), "{hata}");
    assert!(cikti.is_empty());
    let bos_ozet = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let (kod, _, hata) = surec(b"", bos_ozet, "bos");
    assert_eq!(kod, 2);
    assert!(hata.contains("kural yok"), "{hata}");
}

/// Gövde özeti duruşu (C034) yalnız o bağlantı görevini sonlandırır: CR'li
/// gövdeyi (dosyadan okunmuş) özetleyen bağlantı kapanır, sonraki bağlantı
/// aynı dinleyicide normal yanıt alır.
#[test]
fn govde_ozeti_cr_durusu_yalniz_kendi_baglanti_gorevini_sonlandirir() {
    use dil::yorumlayici::{calistir_io_kodla, SahteBaglanti};
    let kaynak = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    komut başarılıysa
        yol komutun değeri olsun
        okuma yol dosyasının 0 baytından 100 baytını metin olarak okumayı dene olsun
        okuma başarılıysa
            gövde okumanın değeri olsun
            bh gövde gövdesinin dkim gövde özeti olsun
            bağlantıya "250 " ile bh satırını yaz
"#;
    let mut io = ToplayanIo::yeni(Vec::new());
    io.ikili_dosyalar
        .insert("cr.eml".into(), b"Merhaba\r\n".to_vec());
    io.ikili_dosyalar.insert("lf.eml".into(), b"\n".to_vec());
    for (adres, yol) in [("192.0.2.7", "cr.eml"), ("192.0.2.8", "lf.eml")] {
        io.baglanti
            .bekleyenler
            .push_back(SahteBaglanti::satirlarla("smtp_v4", adres, &[yol]));
    }
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    calistir_io_kodla(&program, &mut io).expect("sunucu durmamalı");
    assert!(io.baglanti.baglantilar[0].giden.is_empty());
    assert_eq!(
        io.baglanti.baglantilar[1].giden,
        ["250 frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY="]
    );
    assert_eq!(io.baglanti.gunluk.len(), 1);
    assert!(
        io.baglanti.gunluk[0].1.contains("C034"),
        "{:?}",
        io.baglanti.gunluk
    );
    assert!(io.baglanti.baglantilar.iter().all(|b| b.kapandi));
}
