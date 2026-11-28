//! K-378 (RFC-0041 katman A, ADR-163): rotadan ikili dosya yanıtı gerçek
//! `dil` web sürecinde. Kayıtlı içerik türü, `Content-Disposition:
//! attachment` + RFC 6266 `filename*`, `nosniff`, sandbox CSP, CORP, bayt
//! tavanı ve proje dosya sınırı uçtan uca sınanır.
//!
//! K-401 (ZM-F044): tavan 32 MiB; tam sınırdaki dosya akışla bayt bayt iner
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
    ilk "belge.bin" dosyasını "text/plain" türünde "ilk.txt" adıyla ek olarak göndermeyi dene olsun
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

/// RFC-0047: durum değiştiren yöntemde proxy kapısı (spec/12 §5) `Origin`
/// ister; 405/501 yolu ondan sonra gelir.
fn originli_istek(kapi: u16, yontem: &str, hedef: &str) -> (String, Vec<u8>) {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(5)).expect("bağlantı");
    akis.set_read_timeout(Some(Duration::from_secs(30)))
        .expect("okuma süresi");
    write!(
        akis,
        "{yontem} {hedef} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.9;proto=https;host=panel.example\r\nOrigin: https://panel.example\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
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

    // İlk yanıt kazanır: önce bir dosya yanıtı belirlendiyse ikinci dosya
    // yanıtı başarısız Sonuç'tur, ilk yanıt değişmez (K-403: metin yanıtı
    // rotayı bitirdiğinden durum ardışık iki dosya yanıtıyla kurulur).
    let (basliklar, govde) = istek(kapi, "GET", "/iki");
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["text/plain; charset=utf-8"]
    );
    assert_eq!(
        baslik(&basliklar, "Content-Disposition"),
        ["attachment; filename=\"ilk.txt\"; filename*=UTF-8''ilk.txt"]
    );
    assert_eq!(govde, icerik);
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
    // K-403: metin yanıtı rotayı bitirdiğinden "zaten belirlendi" Sonuç'u
    // ardışık iki dosya yanıtıyla sınanır; ilki kalır.
    assert_eq!(io.sunucu_yanitlari[4].1, "%PDF-1.4");
    assert_eq!(
        io.dosya_yanitlari,
        [
            (
                "application/pdf".to_string(),
                "attachment; filename=\"Fatura Ekim.pdf\"; filename*=UTF-8''Fatura%20Ekim.pdf"
                    .to_string()
            ),
            (
                "text/plain; charset=utf-8".to_string(),
                "attachment; filename=\"ilk.txt\"; filename*=UTF-8''ilk.txt".to_string()
            )
        ]
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

/// K-401: 32 MiB'lık dosya yanıtı tam sınırda kabul edilir ve 16 KiB
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

/// K-406 (ADR-163 eki, ZM-F048): bellekteki metin kayıtlı metin türüyle
/// 200 yanıtı olur. Takvim yayını (RFC 5545 CRLF) gövdesi bayt bayt aynı
/// gider: CR/LF dönüştürülmez, `<` ile başlayan gövde HTML koklanmaz.
const TURLU: &str = r#"8080 kapısında sunucu başlat

GET "/takvim/yayin.ics" adresine istek geldiğinde
    herkese açık önbelleklenebilir 60 saniye
    takvim isteğin "g" değeri olsun
    takvim metnini "text/calendar" türünde gönder
    "bu yanıt görünmemeli" yanıtını gönder

HEAD "/takvim/yayin.ics" adresine istek geldiğinde
    herkese açık
    "BEGIN:VCALENDAR" metnini "text/calendar" türünde gönder

GET "/xml" adresine istek geldiğinde
    herkese açık
    belge isteğin "g" değeri olsun
    belge metnini "application/xml" türünde gönder

GET "/csv" adresine istek geldiğinde
    herkese açık
    "ad;soyad" metnini "text/csv" türünde gönder

GET "/takvim/crlf.ics" adresine istek geldiğinde
    herkese açık
    cr_sonucu 13 kod noktası harfini almayı dene olsun
    cr_sonucu başarılıysa
        crlf cr_sonucunun değeri ile "\n" olsun
        takvim "BEGIN:VCALENDAR" ile crlf ile "END:VCALENDAR" ile crlf olsun
        takvim metnini "text/calendar" türünde gönder
"#;

#[test]
fn turlu_metin_yaniti_kayitli_tur_guvenlik_basliklari_ve_ham_crlf_govdeyle_gider() {
    let proje = Proje::yeni(TURLU);
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);
    let ortak = |basliklar: &str| {
        assert!(basliklar.starts_with("HTTP/1.1 200 OK\r\n"), "{basliklar}");
        assert_eq!(baslik(basliklar, "X-Content-Type-Options"), ["nosniff"]);
        assert_eq!(
            baslik(basliklar, "Content-Security-Policy"),
            ["default-src 'none'; sandbox"]
        );
        // Rota önbellek izni (K-335) türlü metin yanıtına uygulanmaz.
        assert_eq!(baslik(basliklar, "Cache-Control"), ["no-store"]);
        assert_eq!(
            baslik(basliklar, "Cross-Origin-Resource-Policy"),
            ["same-origin"]
        );
        assert!(
            baslik(basliklar, "Content-Disposition").is_empty(),
            "{basliklar}"
        );
    };

    let takvim =
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nSUMMARY:Toplantı ç\r\n karma\nsatır\rEND:VCALENDAR\r\n";
    let (basliklar, govde) = istek(
        kapi,
        "GET",
        &format!("/takvim/yayin.ics?g={}", yuzde(takvim)),
    );
    ortak(&basliklar);
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["text/calendar; charset=utf-8"]
    );
    assert_eq!(
        baslik(&basliklar, "Content-Length"),
        [takvim.len().to_string().as_str()]
    );
    // CRLF, çıplak LF ve çıplak CR olduğu gibi; yanıt rotayı bitirdi.
    assert_eq!(govde, takvim.as_bytes());

    let (basliklar, govde) = istek(kapi, "HEAD", "/takvim/yayin.ics");
    ortak(&basliklar);
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["text/calendar; charset=utf-8"]
    );
    assert_eq!(baslik(&basliklar, "Content-Length"), ["15"]);
    assert!(govde.is_empty(), "HEAD gövdesiz");

    // `<` ile başlayan XML text/html'e dönmez; charset eklenmez (RFC 7303).
    let xml = "<?xml version=\"1.0\"?><script>alert(1)</script>";
    let (basliklar, govde) = istek(kapi, "GET", &format!("/xml?g={}", yuzde(xml)));
    ortak(&basliklar);
    assert_eq!(baslik(&basliklar, "Content-Type"), ["application/xml"]);
    assert_eq!(govde, xml.as_bytes());

    // K-402 kod noktası yüzeyiyle kurulan RFC 5545 CRLF satır sonları.
    let (basliklar, govde) = istek(kapi, "GET", "/takvim/crlf.ics");
    ortak(&basliklar);
    assert_eq!(govde, b"BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n");

    let (basliklar, govde) = istek(kapi, "GET", "/csv");
    ortak(&basliklar);
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["text/csv; charset=utf-8"]
    );
    assert_eq!(govde, b"ad;soyad");
}

#[test]
fn turlu_metin_yaniti_kayit_disi_html_betik_ikili_ve_degisken_turu_s037() {
    let rota = |tur: &str| {
        format!(
            "8080 kapısında sunucu başlat\n\nGET \"/t\" adresine istek geldiğinde\n    herkese açık\n    \"x\" metnini {tur} türünde gönder\n"
        )
    };
    for tur in [
        "\"text/html\"",
        "\"image/svg+xml\"",
        "\"application/xhtml+xml\"",
        "\"text/javascript\"",
        "\"application/javascript\"",
        "\"application/pdf\"",
        "\"image/png\"",
        "\"application/octet-stream\"",
        "\"application/x-apple-aspen-config\"",
        "\"Text/Calendar\"",
        "\"text/calendar; charset=utf-8\"",
        "\"text/xml\"",
        "\"\"",
        "tur",
        "\"text/calendar\" \"text/csv\"",
    ] {
        let hata = dil::kaynagi_derle(&rota(tur)).expect_err(tur);
        assert_eq!(hata.kod, "S037", "{tur}: {}", hata.mesaj);
        assert!(
            hata.oneri
                .as_deref()
                .is_some_and(|o| o.contains("text/calendar")),
            "{tur}: öneri kapalı kümeyi anmalı"
        );
    }
    let hata = dil::kaynagi_derle(
        "8080 kapısında sunucu başlat\n\nGET \"/t\" adresine istek geldiğinde\n    herkese açık\n    metnini \"text/calendar\" türünde gönder\n",
    )
    .expect_err("değersiz");
    assert_eq!(hata.kod, "S037");
    // Eylem HTTP adaptörüne bağımlı olamaz (T044).
    let kaynak =
        "eylem yayinla\n    değer döndürmez\n    \"x\" metnini \"text/calendar\" türünde gönder\n";
    assert_eq!(dil::kaynagi_derle(kaynak).expect_err("T044").kod, "T044");
}

#[test]
fn turlu_metin_yaniti_hermetik_adaptorde_ve_io_izinde_ayni_kural() {
    use dil::yorumlayici::{calistir_io, GirdiCikti, IzKaydedenIo, IzYenidenOynatici, ToplayanIo};

    let program = dil::kaynagi_derle(TURLU).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec![
        "GET /takvim/yayin.ics?g=BEGIN%3AVCALENDAR%0D%0AEND%3AVCALENDAR%0D%0A".to_string(),
        "GET /xml?g=%3Ca%2F%3E".to_string(),
    ]
    .into();
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.sunucu_durumlari, [200, 200]);
    assert_eq!(
        io.sunucu_yanitlari[0].1,
        "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n"
    );
    assert_eq!(io.sunucu_yanitlari[1].1, "<a/>");
    assert_eq!(
        io.turlu_yanitlar,
        ["text/calendar; charset=utf-8", "application/xml"]
    );
    assert!(io.dosya_yanitlari.is_empty());

    // Gömme/oynatma yolu da kapalı kümeyi uygular: kayıt dışı tür 500.
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler.push_back("GET /".into());
    assert!(io.istek_al().is_some());
    io.turlu_yanit_gonder("text/html", "<script>");
    io.istek_islemini_tamamla().expect("tamamlanmalı");
    assert_eq!(io.sunucu_durumlari, [500]);
    assert!(io.turlu_yanitlar.is_empty());

    // IO izi: tür ve gövde kaydedilir; oynatıcı aynı olayı bekler, şema
    // kayıt dışı türü reddeder.
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.istekler.push_back("GET /takvim/yayin.ics".into());
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    assert!(kaydeden.istek_al().is_some());
    kaydeden.turlu_yanit_gonder("text/calendar", "A\r\nB");
    let iz = kaydeden.iz_metni().expect("iz");
    assert!(iz.contains("turlu_yanit_gonder"), "{iz}");
    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    assert!(oynatici.istek_al().is_some());
    oynatici.turlu_yanit_gonder("text/calendar", "A\r\nB");
    oynatici.bitir().expect("iz tükenmeli");
    // Alanlar onaltılık kodludur; tür alanı kayıt dışı türle değiştirilir.
    let hex = |m: &str| m.bytes().map(|b| format!("{b:02x}")).collect::<String>();
    assert!(iz.contains(&hex("text/calendar")), "{iz}");
    let bozuk = iz.replace(&hex("text/calendar"), &hex("text/html"));
    assert!(
        IzYenidenOynatici::yeni(&bozuk).is_err(),
        "şema kayıt dışı türü reddeder"
    );
}

/// K-406 + K-403: türlü metin yanıtı kesin yanıttır; lint sonraki cümleyi
/// S047 ile gösterir.
#[test]
fn turlu_metin_yaniti_rotayi_bitirir_lint_s047() {
    let proje = Proje::yeni(TURLU);
    let cikti = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["denetle", "--lint"])
        .arg(&proje.0)
        .output()
        .expect("dil denetle --lint");
    let metin = String::from_utf8_lossy(&cikti.stderr).into_owned()
        + &String::from_utf8_lossy(&cikti.stdout);
    assert!(metin.contains("S047"), "{metin}");
    assert!(metin.contains("bu yanıt görünmemeli"), "{metin}");
}

/// RFC-0047 A + C (ADR-182, K-364): türlü metin yanıtının durumlu kardeşi
/// (207 multistatus, 201, 412), `text/vcard`, kanonik durum satırı, 204
/// gövdesizliği, her rotada 405 + `Allow` ve kayıt dışı yönteme 501.
const DAV_A: &str = r#"8080 kapısında sunucu başlat

GET "/dav/" önekli adrese istek geldiğinde
    herkese açık
    xml "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\"/>" olsun
    xml metnini "application/xml" türünde 207 durumuyla gönder

HEAD "/dav/" önekli adrese istek geldiğinde
    herkese açık
    "" yanıtını 204 durumuyla gönder

GET "/kart.vcf" adresine istek geldiğinde
    herkese açık
    "BEGIN:VCARD\nVERSION:4.0\nEND:VCARD\n" metnini "text/vcard" türünde 201 durumuyla gönder

GET "/cakisma" adresine istek geldiğinde
    herkese açık
    "<e/>" metnini "application/xml" türünde 412 durumuyla gönder
    "bu yanıt görünmemeli" yanıtını gönder

GET "/notlar" adresine istek geldiğinde
    herkese açık
    "liste" yanıtını gönder

DELETE "/notlar" adresine istek geldiğinde
    herkese açık
    "silindi" yanıtını gönder

HEAD "/notlar" adresine istek geldiğinde
    herkese açık
    "" yanıtını 204 durumuyla gönder
"#;

#[test]
fn dav_a_turlu_durumlu_yanit_405_allow_501_ve_204_gercek_surecte() {
    let proje = Proje::yeni(DAV_A);
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);

    let (basliklar, govde) = istek(kapi, "GET", "/dav/takvimler/");
    assert!(
        basliklar.starts_with("HTTP/1.1 207 Multi-Status\r\n"),
        "{basliklar}"
    );
    assert_eq!(baslik(&basliklar, "Content-Type"), ["application/xml"]);
    assert_eq!(baslik(&basliklar, "X-Content-Type-Options"), ["nosniff"]);
    assert_eq!(
        baslik(&basliklar, "Content-Security-Policy"),
        ["default-src 'none'; sandbox"]
    );
    assert_eq!(baslik(&basliklar, "Cache-Control"), ["no-store"]);
    assert_eq!(
        govde,
        b"<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\"/>"
    );

    let (basliklar, govde) = istek(kapi, "GET", "/kart.vcf");
    assert!(
        basliklar.starts_with("HTTP/1.1 201 Created\r\n"),
        "{basliklar}"
    );
    assert_eq!(
        baslik(&basliklar, "Content-Type"),
        ["text/vcard; charset=utf-8"]
    );
    assert_eq!(govde, b"BEGIN:VCARD\nVERSION:4.0\nEND:VCARD\n");

    let (basliklar, govde) = istek(kapi, "GET", "/cakisma");
    assert!(
        basliklar.starts_with("HTTP/1.1 412 Precondition Failed\r\n"),
        "{basliklar}"
    );
    assert_eq!(govde, b"<e/>", "türlü durumlu yanıt rotayı bitirir");

    // 204: gövde, Content-Type ve Content-Length yok (RFC 9110 §8.6).
    let (basliklar, govde) = istek(kapi, "HEAD", "/dav/x");
    assert!(
        basliklar.starts_with("HTTP/1.1 204 No Content\r\n"),
        "{basliklar}"
    );
    assert!(baslik(&basliklar, "Content-Type").is_empty(), "{basliklar}");
    assert!(
        baslik(&basliklar, "Content-Length").is_empty(),
        "{basliklar}"
    );
    assert!(govde.is_empty());

    // 405 + Allow: yolun bütün yöntemleri kanonik sırada, her rota sınıfında.
    for yontem in ["PUT", "POST", "PATCH"] {
        let (basliklar, govde) = originli_istek(kapi, yontem, "/notlar");
        assert!(
            basliklar.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"),
            "{yontem}: {basliklar}"
        );
        assert_eq!(
            baslik(&basliklar, "Allow"),
            ["GET, HEAD, DELETE"],
            "{yontem}"
        );
        assert!(String::from_utf8_lossy(&govde).contains("HTTP yöntemini"));
    }
    let (basliklar, _) = originli_istek(kapi, "PUT", "/dav/a/b.ics");
    assert_eq!(baslik(&basliklar, "Allow"), ["GET, HEAD"], "{basliklar}");
    // Eşleşen yol yoksa 404 kalır ve Allow yazılmaz.
    let (basliklar, _) = originli_istek(kapi, "PUT", "/yok");
    assert!(
        basliklar.starts_with("HTTP/1.1 404 Not Found\r\n"),
        "{basliklar}"
    );
    assert!(baslik(&basliklar, "Allow").is_empty());

    // 501: kayıt dışı yöntem yoldan bağımsız; Allow yok, rota çalışmaz.
    for (yontem, hedef) in [
        ("PROPFIND", "/dav/takvimler/"),
        ("REPORT", "/dav/takvimler/"),
        ("MKCALENDAR", "/dav/yeni/"),
        ("LOCK", "/notlar"),
        ("PROPPATCH", "/yok"),
        ("OPTIONS", "/notlar"),
    ] {
        let (basliklar, _) = originli_istek(kapi, yontem, hedef);
        assert!(
            basliklar.starts_with("HTTP/1.1 501 Not Implemented\r\n"),
            "{yontem} {hedef}: {basliklar}"
        );
        assert!(baslik(&basliklar, "Allow").is_empty(), "{yontem}");
    }
}

#[test]
fn dav_a_turlu_durum_kapali_kumede_s037() {
    let rota = |son: &str| {
        format!(
            "8080 kapısında sunucu başlat\n\nGET \"/t\" adresine istek geldiğinde\n    herkese açık\n    \"x\" metnini \"application/xml\" türünde {son} gönder\n"
        )
    };
    for kabul in [
        "200", "201", "207", "400", "403", "404", "409", "412", "415",
    ] {
        dil::kaynagi_derle(&rota(&format!("{kabul} durumuyla"))).expect(kabul);
    }
    for kotu in [
        "204 durumuyla",
        "302 durumuyla",
        "500 durumuyla",
        "503 durumuyla",
        "99 durumuyla",
        "kod durumuyla",
        "\"207\" durumuyla",
        "durumuyla",
    ] {
        let hata = dil::kaynagi_derle(&rota(kotu)).expect_err(kotu);
        assert_eq!(hata.kod, "S037", "{kotu}: {}", hata.mesaj);
    }
    let hata = dil::kaynagi_derle(&rota("204 durumuyla")).expect_err("204");
    assert!(
        hata.oneri.as_deref().is_some_and(|o| o.contains("207")),
        "öneri kapalı kümeyi anmalı: {:?}",
        hata.oneri
    );
    // Kayıt dışı tür durumlu biçimde de S037.
    let hata = dil::kaynagi_derle(
        "8080 kapısında sunucu başlat\n\nGET \"/t\" adresine istek geldiğinde\n    herkese açık\n    \"x\" metnini \"text/html\" türünde 207 durumuyla gönder\n",
    )
    .expect_err("text/html");
    assert_eq!(hata.kod, "S037");
}

#[test]
fn dav_a_hermetik_adaptor_ve_io_izi_ayni_kural() {
    use dil::yorumlayici::{calistir_io, GirdiCikti, IzKaydedenIo, IzYenidenOynatici, ToplayanIo};

    let program = dil::kaynagi_derle(DAV_A).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec![
        "GET /dav/a/".to_string(),
        "GET /kart.vcf".to_string(),
        "PATCH /notlar".to_string(),
        "PROPFIND /dav/a/".to_string(),
        "GET /yok".to_string(),
    ]
    .into();
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.sunucu_durumlari, [207, 201, 405, 501, 404]);
    assert_eq!(
        io.turlu_yanitlar,
        ["application/xml", "text/vcard; charset=utf-8"]
    );
    assert_eq!(io.izinli_yontemler, ["GET, HEAD, DELETE"]);

    // Gömme yolu kapalı kümeleri uygular: kayıt dışı durum ya da tür 500.
    for (tur, durum) in [("application/xml", 500), ("text/html", 207)] {
        let mut io = ToplayanIo::yeni(Vec::new());
        io.istekler.push_back("GET /".into());
        assert!(io.istek_al().is_some());
        io.turlu_durumlu_yanit_gonder(tur, durum, "<x/>");
        io.istek_islemini_tamamla().expect("tamamlanmalı");
        assert_eq!(io.sunucu_durumlari, [500], "{tur} {durum}");
        assert!(io.turlu_yanitlar.is_empty());
    }

    // IO izi: türlü durumlu olay üç alan; 405 olayı eski `durum_yaniti_gonder`
    // biçimindedir, oynatıcı aynı sırayı bekler; şema kayıt dışı durumu reddeder.
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.istekler.push_back("GET /dav/".into());
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    assert!(kaydeden.istek_al().is_some());
    kaydeden.turlu_durumlu_yanit_gonder("application/xml", 207, "<m/>");
    kaydeden.yontem_reddi_gonder("GET, HEAD", "red");
    let iz = kaydeden.iz_metni().expect("iz");
    assert!(iz.contains("turlu_durumlu_yanit_gonder"), "{iz}");
    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    assert!(oynatici.istek_al().is_some());
    oynatici.turlu_durumlu_yanit_gonder("application/xml", 207, "<m/>");
    oynatici.durum_yaniti_gonder(405, "red");
    oynatici.bitir().expect("iz tükenmeli");
    let hex = |m: &str| m.bytes().map(|b| format!("{b:02x}")).collect::<String>();
    let bozuk = iz.replacen(&hex("207"), &hex("500"), 1);
    assert_ne!(bozuk, iz);
    assert!(
        IzYenidenOynatici::yeni(&bozuk).is_err(),
        "şema kayıt dışı durumu reddeder"
    );
}

/// RFC-0047 A (spec/12, ADR-182): yöntem reddi proxy kapısından sonra gelir;
/// 405/501 yanıtı çerez yazmaz ve `Allow` istekten bayt taşımaz.
#[test]
fn dav_a_yontem_reddi_proxy_kapisindan_sonra_cerezsiz() {
    let proje = Proje::yeni(DAV_A);
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);
    // Origin'siz kayıt dışı yöntem durum değiştiren yöntem sayılır: 403.
    let (basliklar, _) = istek(kapi, "PROPFIND", "/dav/takvimler/");
    assert!(basliklar.starts_with("HTTP/1.1 403 "), "{basliklar}");
    // Origin'li: 501, çerezsiz.
    let (basliklar, _) = originli_istek(kapi, "PROPFIND", "/dav/takvimler/");
    assert!(basliklar.starts_with("HTTP/1.1 501 "), "{basliklar}");
    assert!(baslik(&basliklar, "Set-Cookie").is_empty(), "{basliklar}");
    // 405: Allow yalnız kayıt adları; yol ve yöntem baytı başlığa girmez.
    let (basliklar, _) = originli_istek(kapi, "PATCH", "/notlar");
    assert!(basliklar.starts_with("HTTP/1.1 405 "), "{basliklar}");
    assert!(baslik(&basliklar, "Set-Cookie").is_empty(), "{basliklar}");
    assert_eq!(baslik(&basliklar, "Allow"), ["GET, HEAD, DELETE"]);
    assert!(!basliklar.contains("PATCH"), "{basliklar}");
}
