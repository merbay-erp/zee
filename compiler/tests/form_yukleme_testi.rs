//! K-378 (RFC-0041 katman B, ADR-163): JS'siz tarayıcı formu ile
//! `multipart/form-data` yükleme gerçek `dil` web sürecinde. `_csrf` form
//! alanının dosya parçasından önce gelip oturumla eşleşmesi, dosya baytı
//! diske akmadan önce denetlenir; bozuk/saldırgan gövdeler fail-closed.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[path = "semantic_regresyon/native_web_kopma.rs"]
mod kopma;

static SIRA: AtomicU64 = AtomicU64::new(0);
const SINIR: &str = "----zeeFormSinir5x";

struct Proje(PathBuf);

impl Proje {
    fn yeni(acik: bool) -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-k378-form-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(yol.join("depo")).expect("proje klasörü");
        std::fs::write(
            yol.join("proje.dil"),
            format!(
                "proje \"form\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
                 yetkinlikler \"ağ-sunucusu\", \"web-oturumu\", \"dosya-okuma\", \"dosya-yazma\" listesi olsun\n\
                 ağ_hedefleri boş liste olsun\n{}",
                if acik { "çok_parçalı_yükleme doğru olsun\n" } else { "" }
            ),
        )
        .expect("proje.dil");
        std::fs::write(yol.join("ana.dil"), ANA).expect("ana.dil");
        Self(yol)
    }

    fn parcalar(&self) -> usize {
        std::fs::read_dir(self.0.join(".zee/yuklemeler"))
            .map(|girdiler| {
                girdiler
                    .filter_map(Result::ok)
                    .filter(|g| g.file_name().to_string_lossy().ends_with(".parca"))
                    .count()
            })
            .unwrap_or(0)
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

const ANA: &str = r#"eylem yuklemeyi sakla
    yolu Metin olarak al
    hedefi Metin olarak al
    TamSayı sonucu döndürür
    yolu dosyasını hedefi yoluna atomik taşımayı dene döndür

8080 kapısında sunucu başlat

GET "/form" adresine istek geldiğinde
    csrf csrf belirteci olsun
    csrf yanıtını gönder

POST "/yukle" adresine istek geldiğinde
    herkese açık
    "yukleme_gecici_yolu" alanı gerekli
    yol isteğin "yukleme_gecici_yolu" değeri olsun
    ad "" olsun
    istekte "yukleme_dosya_adi" varsa
        ad isteğin "yukleme_dosya_adi" değeri olsun
    tur "" olsun
    istekte "yukleme_bildirilen_tur" varsa
        tur isteğin "yukleme_bildirilen_tur" değeri olsun
    bayt isteğin "yukleme_bayti" değeri olsun
    ozet isteğin "yukleme_sha256" değeri olsun
    not "" olsun
    istekte "not" varsa
        not isteğin "not" değeri olsun
    hedef "depo/" ile ozet olsun
    tasima yol ve hedef ile yuklemeyi sakla olsun
    cevap ad ile "|" ile tur ile "|" ile bayt ile "|" ile ozet ile "|" ile not olsun
    tasima başarılıysa
        cevap yanıtını gönder
    değilse
        "taşınamadı" yanıtını 500 durumuyla gönder
"#;

fn baslat(proje: &Path, kapi: u16) -> Sunucu {
    Sunucu(kopma::worker_baslat(proje, kapi, Stdio::null))
}

fn baglan(kapi: u16) -> TcpStream {
    let akis = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], kapi)),
        Duration::from_secs(5),
    )
    .expect("bağlantı");
    akis.set_read_timeout(Some(Duration::from_secs(30)))
        .expect("okuma süresi");
    akis
}

fn yanit_oku(mut akis: TcpStream) -> (u16, String) {
    let mut ham = Vec::new();
    let _ = akis.read_to_end(&mut ham);
    let metin = String::from_utf8_lossy(&ham).into_owned();
    let durum = metin
        .split_whitespace()
        .nth(1)
        .and_then(|d| d.parse().ok())
        .unwrap_or(0);
    let govde = metin
        .split_once("\r\n\r\n")
        .map(|(_, g)| g.to_string())
        .unwrap_or_default();
    (durum, govde)
}

/// (oturum çerezi, CSRF belirteci)
fn form_al(kapi: u16) -> (String, String) {
    let mut akis = baglan(kapi);
    akis.write_all(
        b"GET /form HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.5;proto=https;host=panel.example\r\nConnection: close\r\n\r\n",
    )
    .expect("istek");
    let mut ham = String::new();
    akis.read_to_string(&mut ham).expect("yanıt");
    let cerez = ham
        .lines()
        .find_map(|s| s.strip_prefix("Set-Cookie: __Host-zee-oturum="))
        .and_then(|k| k.split(';').next())
        .expect("oturum çerezi")
        .to_string();
    let csrf = ham.split_once("\r\n\r\n").expect("gövde").1.to_string();
    (cerez, csrf)
}

fn istek_basi(cerez: Option<&str>, tur: &str, uzunluk: usize, ek: &str) -> Vec<u8> {
    let mut b = format!(
        "POST /yukle HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.5;proto=https;host=panel.example\r\nOrigin: https://panel.example\r\nConnection: close\r\nContent-Type: {tur}\r\nContent-Length: {uzunluk}\r\n{ek}"
    );
    if let Some(cerez) = cerez {
        b.push_str(&format!("Cookie: __Host-zee-oturum={cerez}\r\n"));
    }
    b.push_str("\r\n");
    b.into_bytes()
}

fn mp_tur() -> String {
    format!("multipart/form-data; boundary={SINIR}")
}

fn alan(ad: &str, deger: &[u8]) -> Vec<u8> {
    let mut p =
        format!("--{SINIR}\r\nContent-Disposition: form-data; name=\"{ad}\"\r\n\r\n").into_bytes();
    p.extend_from_slice(deger);
    p.extend_from_slice(b"\r\n");
    p
}

fn dosya_basi(ad: &str, tur: &str) -> Vec<u8> {
    format!(
        "--{SINIR}\r\nContent-Disposition: form-data; name=\"dosya\"; filename=\"{ad}\"\r\nContent-Type: {tur}\r\n\r\n"
    )
    .into_bytes()
}

fn kapanis() -> Vec<u8> {
    format!("\r\n--{SINIR}--\r\n").into_bytes()
}

/// Gövdeyi `once` + bekleme + `sonra` olarak yollar; beklemede `.parca`
/// sayısını ölçer.
fn yavas_gonder(
    kapi: u16,
    proje: &Proje,
    cerez: Option<&str>,
    once: &[u8],
    sonra: &[u8],
) -> (u16, String, usize) {
    let mut akis = baglan(kapi);
    let mut ham = istek_basi(cerez, &mp_tur(), once.len() + sonra.len(), "");
    ham.extend_from_slice(once);
    akis.write_all(&ham).expect("ilk yarı");
    std::thread::sleep(Duration::from_millis(400));
    let ara = proje.parcalar();
    let _ = akis.write_all(sonra);
    let (durum, govde) = yanit_oku(akis);
    (durum, govde, ara)
}

fn gonder(kapi: u16, cerez: Option<&str>, tur: &str, govde: &[u8], ek: &str) -> (u16, String) {
    let mut akis = baglan(kapi);
    let mut ham = istek_basi(cerez, tur, govde.len(), ek);
    ham.extend_from_slice(govde);
    let _ = akis.write_all(&ham);
    yanit_oku(akis)
}

fn sha256_hex(veri: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(veri)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn form_yuklemesi_csrf_once_dogrular_dosyayi_parca_modeline_akitir() {
    let proje = Proje::yeni(true);
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);
    let (cerez, csrf) = form_al(kapi);
    let icerik: Vec<u8> = (0..100_000u32).map(|i| (i * 13 % 256) as u8).collect();

    // Olumlu: dosya parçası beklerken `.parca` gerçekten oluşur (ölçüm
    // yönteminin kanıtı); rota yol/özet/bayt/ad/türü alır ve dosyayı taşır.
    let mut once = alan("_csrf", csrf.as_bytes());
    once.extend(alan("not", "çağrı notu".as_bytes()));
    once.extend(dosya_basi("C:\\fakepath\\rapor ğ.pdf", "application/pdf"));
    once.extend_from_slice(&icerik[..50_000]);
    let mut sonra = icerik[50_000..].to_vec();
    sonra.extend(kapanis());
    let (durum, govde, ara) = yavas_gonder(kapi, &proje, Some(&cerez), &once, &sonra);
    assert_eq!(durum, 200, "{govde}");
    assert_eq!(ara, 1, "akış sırasında .parca vardır");
    let ozet = sha256_hex(&icerik);
    assert_eq!(
        govde,
        format!("rapor ğ.pdf|application/pdf|100000|{ozet}|çağrı notu")
    );
    assert_eq!(
        std::fs::read(proje.0.join("depo").join(&ozet)).expect("taşınan dosya"),
        icerik
    );
    assert_eq!(proje.parcalar(), 0);

    // `_csrf` yok / yanlış / dosyadan sonra: dosya baytı hiç diske yazılmaz.
    let vakalar: Vec<(&str, Vec<u8>, Vec<u8>)> = vec![
        ("eksik", Vec::new(), Vec::new()),
        ("yanlış", alan("_csrf", b"sahte-belirtec"), Vec::new()),
        ("dosyadan sonra", Vec::new(), alan("_csrf", csrf.as_bytes())),
    ];
    for (ad, onceki, sonraki) in vakalar {
        let mut once = onceki;
        once.extend(dosya_basi("a.bin", "application/octet-stream"));
        once.extend_from_slice(&icerik[..40_000]);
        let mut sonra = icerik[40_000..].to_vec();
        sonra.extend_from_slice(b"\r\n");
        sonra.extend(sonraki);
        sonra.extend(format!("--{SINIR}--\r\n").into_bytes());
        let (durum, govde, ara) = yavas_gonder(kapi, &proje, Some(&cerez), &once, &sonra);
        assert_eq!(durum, 403, "{ad}: {govde}");
        assert!(govde.contains("CSRF"), "{ad}: {govde}");
        assert_eq!(ara, 0, "{ad}: dosya baytı diske akmamalı");
        assert_eq!(proje.parcalar(), 0, "{ad}");
    }

    // Oturum çerezi yoksa gövde okunmadan 403.
    let mut g = alan("_csrf", csrf.as_bytes());
    g.extend(dosya_basi("a.bin", "application/octet-stream"));
    g.extend_from_slice(b"x");
    g.extend(kapanis());
    let (durum, govde) = gonder(kapi, None, &mp_tur(), &g, "");
    assert_eq!(durum, 403, "{govde}");
    // Multipart yolunda X-Zee-CSRF başlığı yok sayılır (iki kaynak birleşmez).
    let mut g = dosya_basi("a.bin", "application/octet-stream");
    g.extend_from_slice(b"x");
    g.extend(kapanis());
    let (durum, _) = gonder(
        kapi,
        Some(&cerez),
        &mp_tur(),
        &g,
        &format!("X-Zee-CSRF: {csrf}\r\n"),
    );
    assert_eq!(durum, 403);
    assert_eq!(proje.parcalar(), 0);

    // Mevcut octet-stream + X-Zee-CSRF yolu değişmeden çalışır.
    let (durum, govde) = gonder(
        kapi,
        Some(&cerez),
        "application/octet-stream",
        b"ham ikili",
        &format!("X-Zee-CSRF: {csrf}\r\n"),
    );
    assert_eq!(durum, 200, "{govde}");
    assert!(
        govde.ends_with(&format!("||9|{}|", sha256_hex(b"ham ikili"))),
        "{govde}"
    );
}

#[test]
fn bozuk_ve_asiri_buyuk_multipart_reddedilir_parca_birakmaz() {
    let proje = Proje::yeni(true);
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);
    let (cerez, csrf) = form_al(kapi);
    let csrf_alani = alan("_csrf", csrf.as_bytes());
    let dosya = |icerik: &[u8]| {
        let mut p = dosya_basi("a.bin", "application/octet-stream");
        p.extend_from_slice(icerik);
        p.extend_from_slice(b"\r\n");
        p
    };
    let govde = |parcalar: &[Vec<u8>], kapanisli: bool| {
        let mut g = parcalar.concat();
        if kapanisli {
            g.extend(format!("--{SINIR}--\r\n").into_bytes());
        }
        g
    };
    let mut vakalar: Vec<(&str, String, Vec<u8>, u16)> = vec![
        ("kapanışsız", mp_tur(), govde(&[csrf_alani.clone(), dosya(b"x")], false), 400),
        ("iki dosya", mp_tur(), govde(&[csrf_alani.clone(), dosya(b"x"), {
            format!("--{SINIR}\r\nContent-Disposition: form-data; name=\"ikinci\"; filename=\"b\"\r\n\r\ny\r\n").into_bytes()
        }], true), 400),
        ("CTE", mp_tur(), govde(&[csrf_alani.clone(), format!("--{SINIR}\r\nContent-Disposition: form-data; name=\"d\"; filename=\"a\"\r\nContent-Transfer-Encoding: base64\r\n\r\neA==\r\n").into_bytes()], true), 400),
        ("dokuz parça", mp_tur(), {
            let mut p: Vec<Vec<u8>> = vec![csrf_alani.clone()];
            for i in 0..7 {
                p.push(alan(&format!("a{i}"), b"1"));
            }
            p.push(dosya(b"x"));
            govde(&p, true)
        }, 400),
        ("ayrılmış ad", mp_tur(), govde(&[csrf_alani.clone(), alan("yukleme_gecici_yolu", b"/etc/passwd"), dosya(b"x")], true), 400),
        ("çift boundary", format!("multipart/form-data; boundary={SINIR}; boundary=x"), govde(&[csrf_alani.clone(), dosya(b"x")], true), 400),
        ("geçersiz boundary", "multipart/form-data; boundary=".into(), govde(&[csrf_alani.clone(), dosya(b"x")], true), 400),
    ];
    // Dosya 16 MiB + 1 bayt; gövde zarf payı içinde: akış sırasında 413.
    let buyuk = vec![b'z'; 16 * 1024 * 1024 + 1];
    vakalar.push((
        "tavan üstü dosya",
        mp_tur(),
        govde(&[csrf_alani.clone(), dosya(&buyuk)], true),
        413,
    ));
    for (ad, tur, g, beklenen) in vakalar {
        let (durum, yanit) = gonder(kapi, Some(&cerez), &tur, &g, "");
        assert_eq!(durum, beklenen, "{ad}: {yanit}");
        assert_eq!(proje.parcalar(), 0, "{ad}: .parca kalmamalı");
    }
    // Zarf tavanı üstündeki Content-Length gövde okunmadan 413.
    let mut akis = baglan(kapi);
    akis.write_all(&istek_basi(
        Some(&cerez),
        &mp_tur(),
        16 * 1024 * 1024 + 64 * 1024 + 1,
        "",
    ))
    .expect("başlık");
    let (durum, _) = yanit_oku(akis);
    assert_eq!(durum, 413);
}

#[test]
fn bildirimsiz_projede_multipart_davranisi_degismez() {
    let proje = Proje::yeni(false);
    let kapi = kopma::bos_kapi();
    let _sunucu = baslat(&proje.0, kapi);
    let (cerez, csrf) = form_al(kapi);
    let mut g = alan("_csrf", csrf.as_bytes());
    g.extend(dosya_basi("a.bin", "application/octet-stream"));
    g.extend_from_slice(b"x");
    g.extend(kapanis());
    // Bugünkü yol: 64 KiB form çözümü, `_csrf` parça başlığında kalır → 403.
    let (durum, govde) = gonder(kapi, Some(&cerez), &mp_tur(), &g, "");
    assert_eq!(durum, 403, "{govde}");
    assert_eq!(proje.parcalar(), 0);
    // 64 KiB üstü multipart gövde bildirimsizken 413 (16 MiB'a açılmaz).
    let mut buyuk = alan("_csrf", csrf.as_bytes());
    buyuk.extend(dosya_basi("a.bin", "application/octet-stream"));
    buyuk.extend(vec![b'q'; 70 * 1024]);
    buyuk.extend(kapanis());
    let (durum, _) = gonder(kapi, Some(&cerez), &mp_tur(), &buyuk, "");
    assert_eq!(durum, 413);
}
