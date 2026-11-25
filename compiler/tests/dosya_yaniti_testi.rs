//! K-378 (RFC-0041 katman A, ADR-163): rotadan ikili dosya yanıtı gerçek
//! `dil` web sürecinde. Kayıtlı içerik türü, `Content-Disposition:
//! attachment` + RFC 6266 `filename*`, `nosniff`, sandbox CSP, CORP, bayt
//! tavanı ve proje dosya sınırı uçtan uca sınanır.
//!
//! K-395b (ZM-F044): tavan 32 MiB; tam sınırdaki dosya akışla bayt bayt iner
//! ve worker'ın yerleşik belleği aktarım boyunca dosya boyu kadar büyümez.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[path = "semantic_regresyon/native_web_kopma.rs"]
mod kopma;

static SIRA: AtomicU64 = AtomicU64::new(0);

struct Proje(PathBuf);

impl Proje {
    fn yeni(ana: &str) -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-k378-dosya-yaniti-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&yol).expect("proje klasörü");
        std::fs::write(
            yol.join("proje.dil"),
            "proje \"dosya-yaniti\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
             yetkinlikler \"ağ-sunucusu\", \"dosya-okuma\" listesi olsun\nağ_hedefleri boş liste olsun\n",
        )
        .expect("proje.dil");
        std::fs::write(yol.join("ana.dil"), ana).expect("ana.dil");
        Self(yol)
    }
}

impl Drop for Proje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Sunucu(Child);

impl Drop for Sunucu {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn baslat(proje: &Path, kapi: u16) -> Sunucu {
    Sunucu(kopma::worker_baslat(proje, kapi, Stdio::null))
}

const ANA: &str = r#"8080 kapısında sunucu başlat

GET "/ek" adresine istek geldiğinde
    herkese açık
    yol isteğin "yol" değeri olsun
    tur isteğin "tur" değeri olsun
    ad isteğin "ad" değeri olsun
    sonuç yol dosyasını tur türünde ad adıyla ek olarak göndermeyi dene olsun
    sonuç başarısızsa
        hata sonucun hatası olsun
        ileti hatanın kodu ile " " ile hatanın mesajı olsun
        ileti yanıtını 404 durumuyla gönder
    "bu yanıt görünmemeli" yanıtını gönder

HEAD "/ek" adresine istek geldiğinde
    herkese açık
    sonuç "belge.bin" dosyasını "application/pdf" türünde "a.pdf" adıyla ek olarak göndermeyi dene olsun
    sonuç başarısızsa
        "yok" yanıtını 404 durumuyla gönder

GET "/iki" adresine istek geldiğinde
    herkese açık
    "önce metin" yanıtını gönder
    sonuç "belge.bin" dosyasını "application/pdf" türünde "a.pdf" adıyla ek olarak göndermeyi dene olsun
    sonuç başarısızsa
        hata sonucun hatası olsun
        hatanın mesajı yaz
"#;

fn yuzde(metin: &str) -> String {
    metin
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-') {
                char::from(b).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

/// Ham yanıt: (başlık metni, gövde baytları).
fn istek(kapi: u16, yontem: &str, hedef: &str) -> (String, Vec<u8>) {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(5)).expect("bağlantı");
    akis.set_read_timeout(Some(Duration::from_secs(30)))
        .expect("okuma süresi");
    write!(
        akis,
        "{yontem} {hedef} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.9;proto=https;host=panel.example\r\nConnection: close\r\n\r\n"
    )
    .expect("istek");
    let mut ham = Vec::new();
    akis.read_to_end(&mut ham).expect("yanıt");
    let son = ham
        .windows(4)
        .position(|p| p == b"\r\n\r\n")
        .expect("başlık sonu")
        + 4;
    (
        String::from_utf8(ham[..son].to_vec()).expect("başlık UTF-8"),
        ham[son..].to_vec(),
    )
}

fn ek(kapi: u16, yontem: &str, yol: &str, tur: &str, ad: &str) -> (String, Vec<u8>) {
    istek(
        kapi,
        yontem,
        &format!("/ek?yol={}&tur={}&ad={}", yuzde(yol), yuzde(tur), yuzde(ad)),
    )
}

fn baslik<'a>(basliklar: &'a str, ad: &str) -> Vec<&'a str> {
    basliklar
        .split("\r\n")
        .filter_map(|satir| satir.split_once(": "))
        .filter(|(anahtar, _)| anahtar.eq_ignore_ascii_case(ad))
        .map(|(_, deger)| deger)
        .collect()
}

#[test]
fn dosya_yaniti_ikili_govdeyi_kayitli_tur_ve_ek_basligiyla_akitir() {
    let proje = Proje::yeni(ANA);
    // UTF-8 olmayan, her bayt değerini taşıyan ve 16 KiB parçadan büyük gövde.
    let icerik: Vec<u8> = (0..300_000u32)
        .map(|i| (i.wrapping_mul(31) % 256) as u8)
        .collect();
    std::fs::write(proje.0.join("belge.bin"), &icerik).expect("ikili dosya");
    std::fs::write(
        proje.0.join("kisiler.txt"),
        "ad;e-posta\nZeynep;z@ornek.test\n",
    )
    .expect("csv");
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);

    let (basliklar, govde) = ek(
        kapi,
        "GET",
        "belge.bin",
        "application/pdf",
        "Fatura Ekim.pdf",
    );
    assert!(basliklar.starts_with("HTTP/1.1 200 OK\r\n"), "{basliklar}");
    assert_eq!(baslik(&basliklar, "Content-Type"), ["application/pdf"]);
    assert_eq!(
        baslik(&basliklar, "Content-Length"),
        [icerik.len().to_string().as_str()]
    );
    assert_eq!(
        baslik(&basliklar, "Content-Disposition"),
        ["attachment; filename=\"Fatura Ekim.pdf\"; filename*=UTF-8''Fatura%20Ekim.pdf"]
    );
    assert_eq!(baslik(&basliklar, "X-Content-Type-Options"), ["nosniff"]);
    assert_eq!(
        baslik(&basliklar, "Content-Security-Policy"),
        ["default-src 'none'; sandbox"]
    );
    assert_eq!(
        baslik(&basliklar, "Cross-Origin-Resource-Policy"),
        ["same-origin"]
    );
    assert_eq!(baslik(&basliklar, "Cache-Control"), ["no-store"]);
    assert!(govde == icerik, "gövde bayt bayt aynı olmalı");

    let (basliklar, govde) = ek(kapi, "GET", "belge.bin", "application/zip", "a.zip");
    assert_eq!(baslik(&basliklar, "Content-Type"), ["application/zip"]);
    assert!(govde == icerik);

    let (basliklar, govde) = istek(kapi, "HEAD", "/ek");
    assert!(basliklar.starts_with("HTTP/1.1 200 OK\r\n"), "{basliklar}");
    assert_eq!(
        baslik(&basliklar, "Content-Length"),
        [icerik.len().to_string().as_str()]
    );
    assert_eq!(
        baslik(&basliklar, "Content-Disposition"),
        ["attachment; filename=\"a.pdf\"; filename*=UTF-8''a.pdf"]
    );
    assert!(govde.is_empty(), "HEAD gövdesiz");

    let (basliklar, govde) = ek(kapi, "GET", "kisiler.txt", "text/csv", "kişiler ğüşİ.csv");
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["text/csv; charset=utf-8"]
    );
    assert_eq!(
        baslik(&basliklar, "Content-Disposition"),
        ["attachment; filename=\"ki_iler ____.csv\"; filename*=UTF-8''ki%C5%9Filer%20%C4%9F%C3%BC%C5%9F%C4%B0.csv"]
    );
    assert_eq!(govde, b"ad;e-posta\nZeynep;z@ornek.test\n");

    // K-379 ile birleşim: kullanıcı kaynaklı `<script>` taşıyan dosya HTML
    // CSP özet akışına girmez; her zaman sandbox CSP, betik özeti yok.
    std::fs::write(proje.0.join("betik.txt"), "<script>alert(1)</script>").expect("betik");
    let (basliklar, govde) = ek(kapi, "GET", "betik.txt", "text/plain", "betik.txt");
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["text/plain; charset=utf-8"]
    );
    assert_eq!(
        baslik(&basliklar, "Content-Security-Policy"),
        ["default-src 'none'; sandbox"]
    );
    assert!(!basliklar.contains("sha256-"), "{basliklar}");
    assert_eq!(govde, b"<script>alert(1)</script>");

    // İlk yanıt kazanır: önce metin yanıtı belirlendiyse dosya yanıtı
    // başarısız Sonuç'tur, metin yanıtı değişmez.
    let (basliklar, govde) = istek(kapi, "GET", "/iki");
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["text/plain; charset=utf-8"]
    );
    assert_eq!(govde, "önce metin".as_bytes());
}

#[test]
fn dosya_yaniti_kayitsiz_tur_ad_enjeksiyonu_yol_gecisi_ve_tavani_reddeder() {
    let proje = Proje::yeni(ANA);
    std::fs::write(proje.0.join("belge.bin"), b"%PDF-1.4 ornek").expect("dosya");
    std::fs::create_dir(proje.0.join("klasor")).expect("klasör");
    // Tavanın bir bayt üstü (seyrek dosya; disk harcamaz).
    std::fs::File::create(proje.0.join("buyuk.bin"))
        .expect("büyük")
        .set_len(32 * 1024 * 1024 + 1)
        .expect("boyut");
    let dis = std::env::temp_dir().join(format!("zee-k378-dis-{}.txt", std::process::id()));
    std::fs::write(&dis, "proje dışı gizli").expect("dış dosya");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&dis, proje.0.join("dis-bag.txt")).expect("dış bağ");
        std::os::unix::fs::symlink(proje.0.join("belge.bin"), proje.0.join("ic-bag.bin"))
            .expect("iç bağ");
    }
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);

    let mut vakalar = vec![
        // MIME karışıklığı: dosya sınıfında HTML/SVG/XHTML/betik yok; parametre ve büyük harf yok.
        (
            "belge.bin",
            "text/html",
            "a.html",
            "kayıtlı içerik türü değil",
        ),
        (
            "belge.bin",
            "image/svg+xml",
            "a.svg",
            "kayıtlı içerik türü değil",
        ),
        (
            "belge.bin",
            "application/xhtml+xml",
            "a.xhtml",
            "kayıtlı içerik türü değil",
        ),
        // K-385: XML yalnız `application/xml` ile; `text/xml` takma adı,
        // XML tabanlı belge/betik türleri ve parametreli profil türü açılmaz.
        (
            "belge.bin",
            "text/xml",
            "a.xml",
            "kayıtlı içerik türü değil",
        ),
        (
            "belge.bin",
            "application/xslt+xml",
            "a.xsl",
            "kayıtlı içerik türü değil",
        ),
        (
            "belge.bin",
            "application/x-apple-aspen-config; charset=utf-8",
            "a.mobileconfig",
            "kayıtlı içerik türü değil",
        ),
        (
            "belge.bin",
            "text/javascript",
            "a.js",
            "kayıtlı içerik türü değil",
        ),
        (
            "belge.bin",
            "Application/PDF",
            "a.pdf",
            "kayıtlı içerik türü değil",
        ),
        (
            "belge.bin",
            "application/pdf; charset=utf-8",
            "a.pdf",
            "kayıtlı içerik türü değil",
        ),
        (
            "belge.bin",
            "application/pdf\r\nX-Evil: 1",
            "a.pdf",
            "kayıtlı içerik türü değil",
        ),
        // Başlık enjeksiyonu ve ad sahteciliği.
        (
            "belge.bin",
            "application/pdf",
            "a\r\nSet-Cookie: x=1.pdf",
            "denetim karakteri",
        ),
        ("belge.bin", "application/pdf", "a\".pdf", "çift tırnak"),
        (
            "belge.bin",
            "application/pdf",
            "fatura\u{202E}fdp.exe",
            "yön denetim",
        ),
        ("belge.bin", "application/pdf", "../a.pdf", "yol ayracı"),
        ("belge.bin", "application/pdf", "", "boş"),
        // Yol geçişi ve normal dosya şartı.
        ("../belge.bin", "application/pdf", "a.pdf", "göreli yol"),
        ("yok.bin", "application/pdf", "a.pdf", "bulunamadı"),
        ("klasor", "application/pdf", "a.pdf", "normal dosya"),
        // Tavan: başlık yazılmadan beklenen hata.
        ("buyuk.bin", "application/zip", "a.zip", "32 MiB"),
    ];
    if cfg!(unix) {
        vakalar.push(("/etc/hosts", "application/pdf", "a.pdf", "göreli yol"));
        vakalar.push(("dis-bag.txt", "text/plain", "a.txt", "sembolik bağ"));
        vakalar.push(("ic-bag.bin", "application/pdf", "a.pdf", "normal dosya"));
    }
    for (yol, tur, ad, beklenen) in vakalar {
        let (basliklar, govde) = ek(kapi, "GET", yol, tur, ad);
        let govde = String::from_utf8(govde).expect("hata gövdesi UTF-8");
        assert!(
            basliklar.starts_with("HTTP/1.1 404 "),
            "{yol} {tur} {ad:?}: {basliklar}"
        );
        assert!(
            govde.starts_with("C013 ") && govde.contains(beklenen),
            "{yol} {tur} {ad:?}: {govde}"
        );
        assert!(baslik(&basliklar, "Content-Disposition").is_empty());
        assert!(baslik(&basliklar, "Set-Cookie").is_empty());
        assert!(baslik(&basliklar, "X-Evil").is_empty());
        assert!(!govde.contains("proje dışı gizli"));
    }
    let _ = std::fs::remove_file(&dis);
}

#[test]
fn dosya_yaniti_yetkinliksiz_projede_derlenmez() {
    let proje = Proje::yeni(ANA);
    std::fs::write(
        proje.0.join("proje.dil"),
        "proje \"dosya-yaniti\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
         yetkinlikler \"ağ-sunucusu\" listesi olsun\nağ_hedefleri boş liste olsun\n",
    )
    .expect("proje.dil");
    let cikti = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["denetle"])
        .arg(&proje.0)
        .output()
        .expect("dil denetle");
    let hata = String::from_utf8_lossy(&cikti.stderr).into_owned()
        + &String::from_utf8_lossy(&cikti.stdout);
    assert!(!cikti.status.success(), "{hata}");
    assert!(hata.contains("dosya-okuma"), "{hata}");
}

/// `dil dene` / kütüphane hermetik adaptörü gerçek çalışma zamanıyla aynı
/// kayıt, ad ve “ilk yanıt kazanır” kuralını uygular.
#[test]
fn hermetik_adaptor_ayni_tur_ad_kuralini_uygular_ve_iz_oynatilir() {
    use dil::yorumlayici::{calistir_io, GirdiCikti, IzKaydedenIo, IzYenidenOynatici, ToplayanIo};

    let program = dil::kaynagi_derle(ANA).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dosyalar.insert("belge.bin".into(), "%PDF-1.4".into());
    io.istekler = vec![
        "GET /ek?yol=belge.bin&tur=application%2Fpdf&ad=Fatura%20Ekim.pdf".to_string(),
        "GET /ek?yol=belge.bin&tur=text%2Fhtml&ad=a.html".to_string(),
        "GET /ek?yol=belge.bin&tur=application%2Fpdf&ad=a%0D%0AX%3A1.pdf".to_string(),
        "GET /ek?yol=yok.bin&tur=application%2Fpdf&ad=a.pdf".to_string(),
        "GET /iki".to_string(),
    ]
    .into();
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.sunucu_durumlari, [200, 404, 404, 404, 200]);
    assert_eq!(io.sunucu_yanitlari[0].1, "%PDF-1.4");
    assert!(io.sunucu_yanitlari[1].1.starts_with("C013 \"text/html\""));
    assert!(io.sunucu_yanitlari[2].1.contains("denetim karakteri"));
    assert!(io.sunucu_yanitlari[3].1.contains("bulunamadı"));
    assert_eq!(io.sunucu_yanitlari[4].1, "önce metin");
    assert_eq!(
        io.dosya_yanitlari,
        [(
            "application/pdf".to_string(),
            "attachment; filename=\"Fatura Ekim.pdf\"; filename*=UTF-8''Fatura%20Ekim.pdf"
                .to_string()
        )]
    );
    assert_eq!(io.cikti, ["bu isteğin yanıtı zaten belirlendi"]);

    // IO izi: yol/tür/ad ve bayt sonucu kaydedilir, içerik ize girmez;
    // oynatıcı aynı sonucu fiziksel dosya olmadan verir.
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.dosyalar.insert("belge.bin".into(), "%PDF-1.4".into());
    taban.istekler.push_back("GET /ek".into());
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    assert!(kaydeden.istek_al().is_some());
    assert_eq!(
        kaydeden.dosya_yaniti_gonder("belge.bin", "application/pdf", "a.pdf"),
        Ok(8)
    );
    assert!(kaydeden
        .dosya_yaniti_gonder("belge.bin", "text/html", "a.html")
        .is_err());
    let iz = kaydeden.iz_metni().expect("iz");
    assert!(!iz.contains("%PDF"), "dosya içeriği ize girmez: {iz}");
    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    assert!(oynatici.istek_al().is_some());
    assert_eq!(
        oynatici.dosya_yaniti_gonder("belge.bin", "application/pdf", "a.pdf"),
        Ok(8)
    );
    assert!(oynatici
        .dosya_yaniti_gonder("belge.bin", "text/html", "a.html")
        .is_err());
    oynatici.bitir().expect("iz tükenmeli");
}

/// Eylem HTTP adaptörüne bağımlı olamaz (T044); dosya yanıtı yalnız rotada.
#[test]
fn eylem_icinde_dosya_yaniti_t044() {
    let kaynak = "eylem indir\n    değer döndürmez\n    sonuç \"a.bin\" dosyasını \"application/pdf\" türünde \"a.pdf\" adıyla ek olarak göndermeyi dene olsun\n";
    assert_eq!(dil::kaynagi_derle(kaynak).expect_err("T044").kod, "T044");
}

/// K-385 (ADR-163 eki, ZM-F033): e-posta istemcisi otomatik yapılandırma
/// XML'i ve Apple yapılandırma profili sabit yollu rotadan kayıtlı türle,
/// diğer dosya yanıtlarıyla aynı `attachment` + sandbox CSP + `nosniff`
/// başlıklarıyla gider; gövde ayrıştırılmadan bayt bayt aynı akar.
#[test]
fn dosya_yaniti_autoconfig_xml_ve_apple_profili_kayitli_turle_gider() {
    const AUTOCONFIG: &str = r#"8080 kapısında sunucu başlat

GET "/.well-known/autoconfig/mail/config-v1.1.xml" adresine istek geldiğinde
    herkese açık
    sonuç "yapilandirma/autoconfig.xml" dosyasını "application/xml" türünde "config-v1.1.xml" adıyla ek olarak göndermeyi dene olsun
    sonuç başarısızsa
        "yok" yanıtını 404 durumuyla gönder

GET "/profil" adresine istek geldiğinde
    herkese açık
    sonuç "yapilandirma/posta.mobileconfig" dosyasını "application/x-apple-aspen-config" türünde "posta.mobileconfig" adıyla ek olarak göndermeyi dene olsun
    sonuç başarısızsa
        "yok" yanıtını 404 durumuyla gönder
"#;
    let proje = Proje::yeni(AUTOCONFIG);
    std::fs::create_dir(proje.0.join("yapilandirma")).expect("klasör");
    // XXE/DTD taşıyan XML: sunucu ayrıştırmaz, gövde olduğu gibi akar.
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE x [<!ENTITY e SYSTEM \"file:///etc/passwd\">]>\n<clientConfig version=\"1.1\"><emailProvider id=\"ornek.test\">&e;</emailProvider></clientConfig>\n";
    std::fs::write(proje.0.join("yapilandirma/autoconfig.xml"), xml).expect("xml");
    // İmzalı profil ikili CMS'tir: UTF-8 olmayan baytlar da aynen gitmeli.
    let profil: Vec<u8> = [0x30, 0x82, 0xff, 0x00, 0xfe]
        .into_iter()
        .chain(b"<plist version=\"1.0\"></plist>".iter().copied())
        .collect();
    std::fs::write(proje.0.join("yapilandirma/posta.mobileconfig"), &profil).expect("profil");
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);

    let ortak = |basliklar: &str| {
        assert!(basliklar.starts_with("HTTP/1.1 200 OK\r\n"), "{basliklar}");
        assert_eq!(baslik(basliklar, "X-Content-Type-Options"), ["nosniff"]);
        assert_eq!(
            baslik(basliklar, "Content-Security-Policy"),
            ["default-src 'none'; sandbox"]
        );
        assert_eq!(baslik(basliklar, "Cache-Control"), ["no-store"]);
        assert_eq!(
            baslik(basliklar, "Cross-Origin-Resource-Policy"),
            ["same-origin"]
        );
    };

    let (basliklar, govde) = istek(kapi, "GET", "/.well-known/autoconfig/mail/config-v1.1.xml");
    ortak(&basliklar);
    assert_eq!(baslik(&basliklar, "Content-Type"), ["application/xml"]);
    assert_eq!(
        baslik(&basliklar, "Content-Disposition"),
        ["attachment; filename=\"config-v1.1.xml\"; filename*=UTF-8''config-v1.1.xml"]
    );
    assert_eq!(govde, xml.as_bytes());

    let (basliklar, govde) = istek(kapi, "GET", "/profil");
    ortak(&basliklar);
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["application/x-apple-aspen-config"]
    );
    assert_eq!(
        baslik(&basliklar, "Content-Disposition"),
        ["attachment; filename=\"posta.mobileconfig\"; filename*=UTF-8''posta.mobileconfig"]
    );
    assert!(govde == profil, "profil bayt bayt aynı olmalı");
}

/// Worker sürecinin yerleşik belleği (KiB).
#[cfg(unix)]
fn yerlesik_kib(pid: u32) -> u64 {
    let cikti = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .expect("ps");
    String::from_utf8_lossy(&cikti.stdout)
        .trim()
        .parse()
        .expect("RSS KiB")
}

/// K-395b: 32 MiB'lık dosya yanıtı tam sınırda kabul edilir ve 16 KiB
/// parçalarla akar. Aktarım boyunca örneklenen yerleşik bellek tepesi
/// tabandan dosya boyunun dörtte birinden (8 MiB) az büyür; dosyayı belleğe
/// alan bir uygulama en az 32 MiB büyürdü. Bir bayt fazlası yukarıdaki ret
/// vakasında (`32 MiB`) sınanır.
#[cfg(unix)]
#[test]
fn dosya_yaniti_32_mib_tam_sinirda_akar_bellek_tepesi_buyumez() {
    let tavan: usize = 32 * 1024 * 1024;
    let proje = Proje::yeni(ANA);
    let icerik: Vec<u8> = (0..tavan).map(|i| (i % 251) as u8).collect();
    std::fs::write(proje.0.join("tam.bin"), &icerik).expect("32 MiB dosya");
    std::fs::write(proje.0.join("belge.bin"), b"%PDF-1.4 ornek").expect("ısınma dosyası");
    let kapi = kopma::bos_kapi();
    let sunucu = baslat(&proje.0, kapi);
    let pid = sunucu.0.id();
    // Isınma: ilk isteğin bir kerelik tahsisleri tabana girsin.
    let (basliklar, _) = ek(kapi, "GET", "belge.bin", "application/pdf", "a.pdf");
    assert!(basliklar.starts_with("HTTP/1.1 200 "), "{basliklar}");
    let taban = yerlesik_kib(pid);

    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(5)).expect("bağlantı");
    akis.set_read_timeout(Some(Duration::from_secs(60)))
        .expect("okuma süresi");
    write!(
        akis,
        "GET /ek?yol=tam.bin&tur=application%2Fzip&ad=tam.zip HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.9;proto=https;host=panel.example\r\nConnection: close\r\n\r\n"
    )
    .expect("istek");
    let mut ham = Vec::with_capacity(tavan + 4096);
    let mut tampon = vec![0u8; 64 * 1024];
    let mut tepe = taban;
    let mut sonraki_ornek = 0usize;
    loop {
        let okunan = akis.read(&mut tampon).expect("yanıt");
        if okunan == 0 {
            break;
        }
        ham.extend_from_slice(&tampon[..okunan]);
        if ham.len() >= sonraki_ornek {
            tepe = tepe.max(yerlesik_kib(pid));
            sonraki_ornek += 2 * 1024 * 1024;
        }
    }
    let son = ham
        .windows(4)
        .position(|p| p == b"\r\n\r\n")
        .expect("başlık sonu")
        + 4;
    let basliklar = String::from_utf8(ham[..son].to_vec()).expect("başlık UTF-8");
    assert!(basliklar.starts_with("HTTP/1.1 200 OK\r\n"), "{basliklar}");
    assert_eq!(
        baslik(&basliklar, "Content-Length"),
        [tavan.to_string().as_str()]
    );
    assert!(ham[son..] == icerik[..], "32 MiB bayt bayt iner");
    assert!(
        tepe.saturating_sub(taban) < 8 * 1024,
        "yerleşik bellek tepesi akış boyunca büyümemeli: taban {taban} KiB, tepe {tepe} KiB"
    );
}
