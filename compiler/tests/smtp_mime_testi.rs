//! K-232/B2: anonim katalog talebinin additive native MIME kabulü.
//! SMTP fixture hermetiktir; canlı sağlayıcı/queue/ürün kabulü değildir.
use dil::agac::{Cumle, Ifade};
use dil::eposta::MimeGonderim;
use dil::intrinsic::{self, EPOSTA_MIME_GONDER};
use dil::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};
use dil::yorumlayici::{
    calistir_io, GirdiCikti, IzKaydedenIo, IzYenidenOynatici, PolitikaliIo, ToplayanIo,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const KARAR: &str = "sonuc başarılıysa\n    sonucun değeri yaz\ndeğilse\n    sonucun hatası yaz\n";
const PDF: &[u8] = b"%PDF-1.7\n\x00\xff\r\n.nokta";
const URUN_TALEBI: &str = include_str!("../../dogfood/zee-cekirdek/mime/katalog-talep.dil");

fn kaynak(ekler: &str, baglanti: &str) -> String {
    URUN_TALEBI
        .replace(
            "ekler \"katalog.pdf\" listesi olsun",
            &format!("ekler {ekler} olsun"),
        )
        .replace(
            "baglanti \"\" olsun",
            &format!("baglanti \"{baglanti}\" olsun"),
        )
}

struct Sahte {
    ic: ToplayanIo,
    cagrilar: usize,
    butce: i64,
    hata: bool,
}
impl Sahte {
    fn yeni() -> Self {
        Self {
            ic: ToplayanIo::yeni(Vec::new()),
            cagrilar: 0,
            butce: 0,
            hata: false,
        }
    }
}
impl GirdiCikti for Sahte {
    fn yazdir(&mut self, s: String) {
        self.ic.yazdir(s)
    }
    fn sor(&mut self, s: &str) -> Option<String> {
        self.ic.sor(s)
    }
    fn rastgele(&mut self, a: i64, b: i64) -> i64 {
        self.ic.rastgele(a, b)
    }
    fn dosya_oku(&mut self, p: &str) -> Result<String, String> {
        self.ic.dosya_oku(p)
    }
    fn dosya_yaz(&mut self, p: &str, s: &str, e: bool) -> Result<(), String> {
        self.ic.dosya_yaz(p, s, e)
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.ic.simdi()
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.ic.argumanlar()
    }
    fn http_getir(&mut self, u: &str, t: Option<i64>) -> Result<(i64, String), String> {
        self.ic.http_getir(u, t)
    }
    fn sunucu_kur(&mut self, p: i64) -> Result<(), String> {
        self.ic.sunucu_kur(p)
    }
    fn istek_al(&mut self) -> Option<String> {
        self.ic.istek_al()
    }
    fn yanit_gonder(&mut self, s: &str) {
        self.ic.yanit_gonder(s)
    }
    fn yonlendir_gonder(&mut self, s: &str) -> Result<(), String> {
        self.ic.yonlendir_gonder(s)
    }
    fn cerez_yaz(&mut self, a: &str, b: &str) -> Result<(), String> {
        self.ic.cerez_yaz(a, b)
    }
    fn cerez_sil(&mut self, a: &str) -> Result<(), String> {
        self.ic.cerez_sil(a)
    }
    fn sensor_acik_mi(&mut self, a: &str) -> bool {
        self.ic.sensor_acik_mi(a)
    }
    fn isik_ayarla(&mut self, a: &str, b: bool) {
        self.ic.isik_ayarla(a, b)
    }
    fn bekle_ms(&mut self, t: i64) {
        self.ic.bekle_ms(t)
    }
    fn an_ms(&mut self) -> i64 {
        self.ic.an_ms()
    }
    fn eposta_mime_gonder(
        &mut self,
        b: Option<&str>,
        a: &str,
        k: &str,
        g: &str,
        s: &MimeGonderim<'_>,
    ) -> Result<i64, String> {
        self.cagrilar += 1;
        self.butce = s.zaman_asimi_ms;
        assert_eq!(b, None);
        assert_eq!(
            (a, k, g),
            (
                "recipient@example.invalid",
                "Katalog Şalça",
                "Katalog PDF'i ektedir. / The catalogue PDF is attached."
            )
        );
        assert_eq!(
            s.html,
            "<p>Katalog PDF'i ektedir.</p><p>The catalogue PDF is attached.</p>"
        );
        assert_eq!(s.yanit_adresi, "reply@example.invalid");
        if self.hata {
            Err(
                "smtp://SENTINEL_USER:SENTINEL_PASSWORD@private.invalid; PDF SENTINEL_CONTENT"
                    .into(),
            )
        } else {
            Ok(1)
        }
    }
}

#[test]
fn typed_kalip_merkezi_intrinsic_kaydi_ve_sonuc_turuyle_baglanir() {
    let program = dil::kaynagi_derle(&kaynak("boş liste", "")).unwrap();
    let Cumle::Olsun { deger, .. } = &program.cumleler[7] else {
        panic!("atama")
    };
    let Ifade::Intrinsic { kimlik, argumanlar } = deger.turu() else {
        panic!("intrinsic")
    };
    assert_eq!(kimlik, EPOSTA_MIME_GONDER);
    assert_eq!(argumanlar.len(), 7);
    let tanim = intrinsic::tanim(kimlik).unwrap();
    assert_eq!(tanim.donus_turu, intrinsic::IntrinsicTuru::TamSayiSonucu);
    assert_eq!(tanim.yetkinlik, intrinsic::Yetkinlik::Ag);
    assert_eq!(tanim.etki, intrinsic::IntrinsicEtkisi::GeriAlinamaz);
    let mut io = Sahte::yeni();
    calistir_io(&program, &mut io).unwrap();
    assert_eq!(io.cagrilar, 1);
    assert_eq!(io.ic.cikti, ["1"]);
    assert_eq!(io.butce, 20_000);
}

#[test]
fn typed_alan_ve_ek_listesi_yanlis_turleri_reddeder() {
    for degisken in ["alici", "konu", "duz", "html", "yanit", "baglanti"] {
        let src = kaynak("boş liste", "");
        let satir = src
            .lines()
            .find(|s| s.starts_with(&format!("{degisken} ")))
            .unwrap();
        let src = src.replace(satir, &format!("{degisken} 7 olsun"));
        assert_eq!(
            dil::kaynagi_derle(&src).unwrap_err().kod,
            "T034",
            "{degisken}"
        );
    }
    for liste in ["7 listesi", "\"ek.pdf\"", "7"] {
        assert_eq!(
            dil::kaynagi_derle(&kaynak(liste, "")).unwrap_err().kod,
            "T034"
        );
    }
}

#[test]
fn geri_alinamaz_mime_eylemde_t048_ile_reddedilir() {
    let src = format!(
        "eylem katalog gönder\n    değer döndürmez\n{}",
        kaynak("boş liste", "")
            .lines()
            .map(|s| format!("    {s}\n"))
            .collect::<String>()
    );
    assert_eq!(dil::kaynagi_derle(&src).unwrap_err().kod, "T048");
}

#[test]
fn runtime_hata_siri_icerigi_hedefi_ve_yolu_gizler_tekrar_yapmaz() {
    let mut io = Sahte::yeni();
    io.hata = true;
    let src = kaynak("boş liste", "").replace(KARAR, "sonuc başarısızsa\n    hata sonucun hatası olsun\n    hatanın mesajı yaz\n    hatanın kodu yaz\n");
    calistir_io(&dil::kaynagi_derle(&src).unwrap(), &mut io).unwrap();
    assert_eq!(io.cagrilar, 1);
    assert_eq!(io.ic.cikti.len(), 2);
    assert_eq!(io.ic.cikti[1], "C029");
    let hata = &io.ic.cikti[0];
    assert!(
        hata.contains("MIME") && hata.contains("otomatik tekrar yok"),
        "{hata}"
    );
    for sir in ["SENTINEL", "private.invalid", "smtp://"] {
        assert!(!hata.contains(sir));
    }
}

#[test]
fn politika_ag_dosya_okuma_ve_goreli_yolu_ic_adaptorden_once_denetler() {
    let p = dil::kaynagi_derle(&kaynak("\"ek.pdf\" listesi", "")).unwrap();
    for izinler in [vec![], vec![Yetkinlik::Ag], vec![Yetkinlik::DosyaOkuma]] {
        let mut io = PolitikaliIo::politikali(
            Sahte::yeni(),
            YetkinlikPolitikasi::proje(izinler.into_iter().collect(), Default::default()),
        );
        calistir_io(&p, &mut io).unwrap();
        assert_eq!(io.ic.cagrilar, 0);
    }
    let politika = YetkinlikPolitikasi::proje(
        [Yetkinlik::Ag, Yetkinlik::DosyaOkuma].into_iter().collect(),
        Default::default(),
    );
    for yol in ["../ek.pdf", "/ek.pdf", "C:/ek.pdf"] {
        let mut io = PolitikaliIo::politikali(Sahte::yeni(), politika.clone());
        calistir_io(
            &dil::kaynagi_derle(&kaynak(&format!("\"{yol}\" listesi"), "")).unwrap(),
            &mut io,
        )
        .unwrap();
        assert_eq!(io.ic.cagrilar, 0);
    }
    let mut io = PolitikaliIo::politikali(Sahte::yeni(), politika);
    calistir_io(&p, &mut io).unwrap();
    assert_eq!(io.ic.cagrilar, 1, "dosya-yazma gerekmez");
}

#[test]
fn kayit_yurutme_export_ve_replay_mimeyi_sirsiz_reddeder() {
    let p = dil::kaynagi_derle(&kaynak("\"SENTINEL_FILE.pdf\" listesi", "")).unwrap();
    let mut kayit = IzKaydedenIo::yeni(Sahte::yeni());
    calistir_io(&p, &mut kayit).unwrap();
    assert_eq!(kayit.ic.cagrilar, 0);
    assert!(!kayit.iz_metni().unwrap_err().contains("SENTINEL"));
    let mut replay = IzYenidenOynatici::yeni("zee-io-izi\t1\n").unwrap();
    calistir_io(&p, &mut replay).unwrap();
    let hata = replay.bitir().unwrap_err();
    assert!(hata.contains("MIME"));
    assert!(!hata.contains("SENTINEL"));
}

#[test]
fn playground_ve_varsayilan_io_native_mime_basarisi_uydurmaz() {
    let src = kaynak("boş liste", "");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&dil::kaynagi_derle(&src).unwrap(), &mut io).unwrap();
    assert!(io.cikti[0].contains("MIME"));
    let cikti = dil::wasm_api::playgroundda_calistir(&src, "", 1);
    assert!(cikti.contains("MIME"), "{cikti}");
    assert!(!cikti.contains("SENTINEL"));
}

#[test]
fn dis_son_tarih_mime_butcesini_kisaltir_ve_sinir_merkezi_sahiptir() {
    let src = format!(
        "1 saniye içinde\n{}yetişmezse\n    \"iptal\" yaz\n",
        kaynak("boş liste", "")
            .lines()
            .map(|s| format!("    {s}\n"))
            .collect::<String>()
    );
    let mut io = Sahte::yeni();
    calistir_io(&dil::kaynagi_derle(&src).unwrap(), &mut io).unwrap();
    assert!(io.butce > 0 && io.butce <= 1_000);
    let s = dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.eposta();
    assert_eq!(s.ek_adedi(), dil::eposta::mime::AZAMI_EK_ADEDI);
    assert_eq!(s.ek_bayti(), dil::eposta::mime::AZAMI_EK_BAYTI);
    assert_eq!(
        s.toplam_ek_bayti(),
        dil::eposta::mime::AZAMI_TOPLAM_EK_BAYTI
    );
    assert_eq!(s.metin_bayti(), dil::eposta::mime::AZAMI_METIN_BAYTI);
    assert_eq!(s.ileti_bayti(), dil::eposta::mime::AZAMI_ILETI_BAYTI);
}

static SIRA: AtomicU64 = AtomicU64::new(0);
struct Kopya(PathBuf);
impl Kopya {
    fn yeni() -> Self {
        let path = std::env::temp_dir().join(format!(
            "zee-k232-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn hazirla(&self, kapi: u16, izin: &str, yol: &str, baglanti: &str) {
        std::fs::write(self.0.join("ana.dil"), kaynak(yol, baglanti)).unwrap();
        std::fs::write(self.0.join("proje.dil"),format!("proje \"mime-fixture\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler {izin} listesi olsun\nağ_hedefleri \"smtp://127.0.0.1:{kapi}\" listesi olsun\ne_posta_bağlantı_değişkeni \"ZEE_MIME_TEST_SMTP\" olsun\n")).unwrap();
    }
    fn komut(&self, url: Option<&str>) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_dil"));
        c.current_dir(&self.0)
            .args(["çalıştır", "."])
            .env_remove("ZEE_MIME_TEST_SMTP");
        if let Some(url) = url {
            c.env("ZEE_MIME_TEST_SMTP", url);
        }
        c.output().unwrap()
    }
}
impl Drop for Kopya {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn url(kapi: u16) -> String {
    format!("smtp://sentinel:SENTINEL_PASSWORD@127.0.0.1:{kapi}?from=sender@example.invalid")
}
fn red_kalmali(out: Output) {
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.contains("MIME") && text.contains("otomatik tekrar yok"),
        "{text}"
    );
    assert!(!text.contains("SENTINEL"));
    assert!(!String::from_utf8_lossy(&out.stderr).contains("SENTINEL"));
}
fn snapshot(path: &Path) -> (Vec<u8>, std::time::SystemTime, u64) {
    let m = std::fs::metadata(path).unwrap();
    (std::fs::read(path).unwrap(), m.modified().unwrap(), m.len())
}

#[test]
fn cli_gercek_smtp_pdf_alternative_reply_to_ve_dosya_yazmadan_kabul_eder() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let kapi = l.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (s, _) = l.accept().unwrap();
        s.set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        let mut s = BufReader::new(s);
        let mut line = String::new();
        s.get_mut().write_all(b"220 fixture\r\n").unwrap();
        for (beklenen, cevap) in [
            ("EHLO ", "250 fixture\r\n"),
            ("AUTH PLAIN ", "235 ok\r\n"),
            ("MAIL FROM:", "250 ok\r\n"),
            ("RCPT TO:", "250 ok\r\n"),
            ("DATA", "354 go\r\n"),
        ] {
            line.clear();
            s.read_line(&mut line).unwrap();
            assert!(line.starts_with(beklenen), "{line}");
            s.get_mut().write_all(cevap.as_bytes()).unwrap();
        }
        let mut wire = String::new();
        loop {
            line.clear();
            assert!(s.read_line(&mut line).unwrap() > 0);
            if line == ".\r\n" {
                break;
            }
            wire.push_str(&line);
        }
        s.get_mut().write_all(b"250 accepted\r\n").unwrap();
        line.clear();
        s.read_line(&mut line).unwrap();
        assert_eq!(line, "QUIT\r\n");
        s.get_mut().write_all(b"221 bye\r\n").unwrap();
        wire
    });
    let k = Kopya::yeni();
    k.hazirla(
        kapi,
        "\"ağ\", \"yerel-ağ\", \"dosya-okuma\"",
        "\"Katalog Şalça.pdf\" listesi",
        "",
    );
    let ek = k.0.join("Katalog Şalça.pdf");
    std::fs::write(&ek, PDF).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ek, std::fs::Permissions::from_mode(0o440)).unwrap();
    }
    let before = snapshot(&ek);
    let out = k.komut(Some(&url(kapi)));
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), "1");
    assert_eq!(snapshot(&ek), before);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&ek).unwrap().permissions().mode() & 0o777,
            0o440
        );
    }
    let wire = server.join().unwrap();
    for needle in [
        "multipart/mixed",
        "multipart/alternative",
        "Reply-To: reply@example.invalid",
        "Content-Type: application/pdf",
        "filename*=UTF-8''Katalog%20%C5%9Eal%C3%A7a.pdf",
    ] {
        assert!(wire.contains(needle), "{needle}");
    }
    assert!(wire.contains("JVBERi0xLjcKAP8NCi5ub2t0YQ=="));
    assert!(!wire.contains("SENTINEL"));
}

#[test]
fn cli_ortam_hedef_dosya_izni_ve_kokten_kacisi_dial_oncesi_reddeder() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let kapi = l.local_addr().unwrap().port();
    let k = Kopya::yeni();
    std::fs::write(k.0.join("ek.pdf"), PDF).unwrap();
    let u = url(kapi);
    let izin = "\"ağ\", \"yerel-ağ\", \"dosya-okuma\"";
    for (izin, yol) in [
        ("\"ağ\", \"yerel-ağ\"", "\"ek.pdf\" listesi"),
        (izin, "\"../ek.pdf\" listesi"),
        (izin, "\"/ek.pdf\" listesi"),
    ] {
        k.hazirla(kapi, izin, yol, "");
        red_kalmali(k.komut(Some(&u)));
    }
    k.hazirla(kapi, izin, "\"ek.pdf\" listesi", "");
    red_kalmali(k.komut(None));
    let wrong = format!(
        "smtp://sentinel:SENTINEL_PASSWORD@127.0.0.1:{}?from=sender@example.invalid",
        kapi.saturating_add(1)
    );
    red_kalmali(k.komut(Some(&wrong)));
    assert!(matches!(l.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock));
}

#[cfg(unix)]
#[test]
fn cli_sembolik_bagla_proje_disindaki_pdfi_okumaz() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let kapi = l.local_addr().unwrap().port();
    let k = Kopya::yeni();
    let dis = Kopya::yeni();
    std::fs::write(dis.0.join("sir.pdf"), PDF).unwrap();
    std::os::unix::fs::symlink(dis.0.join("sir.pdf"), k.0.join("ek.pdf")).unwrap();
    k.hazirla(
        kapi,
        "\"ağ\", \"yerel-ağ\", \"dosya-okuma\"",
        "\"ek.pdf\" listesi",
        "",
    );
    red_kalmali(k.komut(Some(&url(kapi))));
    assert!(matches!(l.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock));
}

#[cfg(unix)]
#[test]
fn cli_writer_yok_fifo_pdfde_bloklanmaz_smtp_etkisi_baslatmaz() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let kapi = l.local_addr().unwrap().port();
    let k = Kopya::yeni();
    let fifo = std::ffi::CString::new(k.0.join("ek.pdf").to_str().unwrap()).unwrap();
    // SAFETY: CString NUL sonlu yoludur; yalnız testin sahip olduğu temp kökte
    // writersız FIFO oluşturulur ve Drop ile bu temp kök silinir.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    k.hazirla(
        kapi,
        "\"ağ\", \"yerel-ağ\", \"dosya-okuma\"",
        "\"ek.pdf\" listesi",
        "",
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_dil"))
        .current_dir(&k.0)
        .args(["çalıştır", "."])
        .env("ZEE_MIME_TEST_SMTP", url(kapi))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let baslangic = std::time::Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if baslangic.elapsed() > std::time::Duration::from_secs(3) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("writer-yok FIFO native ek açmayı blokladı");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    red_kalmali(child.wait_with_output().unwrap());
    assert!(matches!(l.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock));
}
