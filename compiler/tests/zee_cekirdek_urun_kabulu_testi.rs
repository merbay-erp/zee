//! K-223/K-220: gerçek ürün B7/B8 taleplerinin anonim, mevcut dilde kabulü.
//! DB modeli hermetiktir; SQL concurrency ve canlı ürün kabulü iddia etmez.

use dil::agac::{Program, RotaErisimi};
use dil::web_guvenligi::WebReddi;
use dil::yorumlayici::{calistir_io, EylemHatasi, GirdiCikti, ToplayanIo, VeritabaniHatasi};
use std::collections::BTreeMap;

const LEGACY: &str = "$2b$11$ps3248yUh0B5w2jKirP8QufvI/UGGxj8vc8YKmJKwj0plgNdg5jLy";
const PAROLA: &str = "dogru-parola";
const YUKSELT: &str = include_str!("../../dogfood/zee-cekirdek/parola-goc/giriste-yukselt.dil");
const FORMLAR: &str = include_str!("../../dogfood/zee-cekirdek/http-post/form-kabulu.dil");
const TURNSTILE: &str = include_str!("../../dogfood/zee-cekirdek/http-post/turnstile.dil");
const GUNCELLE: &str =
    "UPDATE zee_rehash_kullanicilar SET parola_ozeti = $3 WHERE kimlik = $1 AND parola_ozeti = $2";
const BAGLA: &str = "INSERT INTO zee_rehash_oturumlar (kimlik) SELECT kimlik FROM zee_rehash_kullanicilar WHERE kimlik = $1 AND parola_ozeti = $2";
const FORM_EKLE: &str = "INSERT INTO zee_form_kabul (rota, govde) VALUES ($1, $2)";
const ATOMIK_GIRIS: &str = r#"
eylem anonimAtomikGiris
    anahtarı Metin olarak al
    parolayi Metin olarak al
    eskiyi Metin olarak al
    Metin sonucu döndürür
    yukseltme anahtar ve parola ve eski ile girişte parolayı yükselt olsun
    yukseltme başarılıysa
        beklenen yukseltmenin değeri olsun
        parametreler anahtar, beklenen listesi olsun
        baglama "INSERT INTO zee_rehash_oturumlar (kimlik) SELECT kimlik FROM zee_rehash_kullanicilar WHERE kimlik = $1 AND parola_ozeti = $2" sorgusunu parametreler ile değiştirmeyi dene olsun
        baglama başarılıysa
            adet baglamanin değeri olsun
            adet 1 e eşit değilse
                "giris-eski-snapshot" hatasını döndür
            "girdi" döndür
        değilse
            "giris-baglama-reddi" hatasını döndür
    değilse
        "giris-yukseltme-reddi" hatasını döndür
"#;

#[derive(Clone, Default)]
struct Durum {
    kullanicilar: BTreeMap<String, String>,
    formlar: Vec<(String, String)>,
    oturumlar: usize,
}

#[derive(Clone, Copy, Default)]
enum Commit {
    #[default]
    Tamam,
    Red,
    BelirsizKayitVar,
    BelirsizKayitYok,
}

struct UrunIo {
    ic: ToplayanIo,
    durum: Durum,
    yedekler: Vec<Durum>,
    sorgular: Vec<(String, Vec<String>)>,
    degisim_hatasi: bool,
    baglama_hatasi: bool,
    ozet_hatasi: bool,
    smtp_hatasi: bool,
    commit: Commit,
    geri_almalar: usize,
    ozet_uretimleri: usize,
    legacy_dogrulamalar: usize,
    modern_dogrulamalar: usize,
    http_sonucu: Result<(i64, String), String>,
    gizli_sonucu: Result<String, String>,
    http_cagrilari: usize,
}

impl UrunIo {
    fn yeni() -> Self {
        Self {
            ic: ToplayanIo::yeni(Vec::new()),
            durum: Durum::default(),
            yedekler: Vec::new(),
            sorgular: Vec::new(),
            degisim_hatasi: false,
            baglama_hatasi: false,
            ozet_hatasi: false,
            smtp_hatasi: false,
            commit: Commit::Tamam,
            geri_almalar: 0,
            ozet_uretimleri: 0,
            legacy_dogrulamalar: 0,
            modern_dogrulamalar: 0,
            http_sonucu: Ok((200, r#"{"success":true,"hostname":"site.example"}"#.into())),
            gizli_sonucu: Ok("TEST_SENTINEL".into()),
            http_cagrilari: 0,
        }
    }

    fn legacy(ozet: &str) -> Self {
        let mut io = Self::yeni();
        io.durum.kullanicilar.insert("hesap-1".into(), ozet.into());
        io
    }

    fn csrf_hazirla(&mut self) -> (String, String) {
        self.ic.istekler.push_back("GET /csrf-hazirlik".into());
        self.ic.istek_al().expect("başlangıç isteği");
        let csrf = self.ic.csrf_belirteci().expect("CSRF oturumu");
        let oturum = self.ic.yazilan_cerezler[0].1.clone();
        self.ic.yanit_gonder("");
        self.ic.istek_islemini_tamamla().unwrap();
        self.ic.sunucu_yanitlari.clear();
        self.ic.sunucu_durumlari.clear();
        (oturum, csrf)
    }
}

fn db_hatasi() -> VeritabaniHatasi {
    VeritabaniHatasi {
        mesaj: "sentetik SQL reddi".into(),
        veri: vec![("sqlstate".into(), "23514".into())],
    }
}

impl GirdiCikti for UrunIo {
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
    fn istek_islemini_tamamla(&mut self) -> Result<(), String> {
        self.ic.istek_islemini_tamamla()
    }
    fn istek_islemini_geri_al(&mut self) {
        self.ic.istek_islemini_geri_al()
    }
    fn yanit_gonder(&mut self, s: &str) {
        self.ic.yanit_gonder(s)
    }
    fn durum_yaniti_gonder(&mut self, d: u16, s: &str) {
        self.ic.durum_yaniti_gonder(d, s)
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
    fn rota_guvenligini_denetle(
        &mut self,
        e: &RotaErisimi,
        c: Option<&str>,
        g: bool,
    ) -> Result<(), WebReddi> {
        self.ic.rota_guvenligini_denetle(e, c, g)
    }
    fn istemci_adresi(&mut self) -> String {
        "203.0.113.7".into()
    }
    fn csrf_belirteci(&mut self) -> Result<String, String> {
        self.ic.csrf_belirteci()
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

    fn parola_dogrula(&mut self, p: &str, o: &str) -> bool {
        if o.starts_with("$2") {
            self.legacy_dogrulamalar += 1;
        } else {
            self.modern_dogrulamalar += 1;
        }
        self.ic.parola_dogrula(p, o)
    }
    fn parola_ozeti_uret(&mut self, p: &str) -> Result<String, String> {
        self.ozet_uretimleri += 1;
        if self.ozet_hatasi {
            return Err("sentetik CSPRNG reddi".into());
        }
        self.ic.parola_ozeti_uret(p)
    }
    fn postgresql_degistir(&mut self, s: &str, p: &[String]) -> Result<i64, VeritabaniHatasi> {
        assert!(!self.yedekler.is_empty(), "SQL yazısı eylemde olmalı");
        self.sorgular.push((s.into(), p.to_vec()));
        match s {
            GUNCELLE => {
                assert_eq!(p.len(), 3);
                if self.degisim_hatasi {
                    return Err(db_hatasi());
                }
                if self.durum.kullanicilar.get(&p[0]) == Some(&p[1]) {
                    self.durum.kullanicilar.insert(p[0].clone(), p[2].clone());
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            BAGLA => {
                assert_eq!(p.len(), 2);
                if self.baglama_hatasi {
                    return Err(db_hatasi());
                }
                if self.durum.kullanicilar.get(&p[0]) == Some(&p[1]) {
                    self.durum.oturumlar += 1;
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            FORM_EKLE => {
                assert_eq!(p.len(), 2);
                if self.degisim_hatasi {
                    return Err(db_hatasi());
                }
                self.durum.formlar.push((p[0].clone(), p[1].clone()));
                Ok(1)
            }
            _ => panic!("anonim modelde bilinmeyen SQL: {s}"),
        }
    }
    fn eylem_baslat(&mut self) -> Result<(), EylemHatasi> {
        self.ic.eylem_baslat()?;
        self.yedekler.push(self.durum.clone());
        Ok(())
    }
    fn eylem_tamamla(&mut self) -> Result<(), EylemHatasi> {
        let yedek = self.yedekler.pop().expect("açık eylem");
        if self.yedekler.is_empty() {
            match std::mem::take(&mut self.commit) {
                Commit::Tamam => {}
                Commit::Red => {
                    self.durum = yedek;
                    self.ic.eylem_geri_al()?;
                    return Err(EylemHatasi::genel("sentetik COMMIT reddi"));
                }
                Commit::BelirsizKayitYok => {
                    self.durum = yedek;
                    self.ic.eylem_geri_al()?;
                    return Err(EylemHatasi::commit_sonucu_belirsiz(
                        "sentetik COMMIT cevap kaybı",
                    ));
                }
                Commit::BelirsizKayitVar => {
                    self.ic.eylem_tamamla()?;
                    return Err(EylemHatasi::commit_sonucu_belirsiz(
                        "sentetik COMMIT cevap kaybı",
                    ));
                }
            }
        }
        self.ic.eylem_tamamla()
    }
    fn eylem_geri_al(&mut self) -> Result<(), EylemHatasi> {
        self.geri_almalar += 1;
        self.durum = self.yedekler.pop().expect("açık eylem");
        self.ic.eylem_geri_al()
    }
    fn http_gizli_ortam(&mut self, a: &str) -> Result<String, String> {
        assert_eq!(a, "TURNSTILE_SECRET");
        self.gizli_sonucu.clone()
    }
    fn http_istek(
        &mut self,
        u: &str,
        m: &str,
        h: &[(String, String)],
        b: &str,
        t: i64,
    ) -> Result<(i64, String), String> {
        assert_eq!(
            u,
            "https://challenges.cloudflare.com/turnstile/v0/siteverify"
        );
        assert_eq!(m, "POST");
        assert_eq!(h, &[("Content-Type".into(), "application/json".into())]);
        assert_eq!(t, 5000);
        let veri: serde_json::Value = serde_json::from_str(b).unwrap();
        assert_eq!(veri["secret"], "TEST_SENTINEL");
        assert_eq!(veri["response"], "jeton");
        assert_eq!(veri["remoteip"], "203.0.113.7");
        self.http_cagrilari += 1;
        self.http_sonucu.clone()
    }
    fn eposta_gonder(&mut self, b: Option<&str>, a: &str, k: &str, g: &str) -> Result<i64, String> {
        if self.smtp_hatasi {
            return Err("sentetik SMTP reddi".into());
        }
        self.ic.eposta_gonder(b, a, k, g)
    }
}

fn giris_programi(parola: &str, ozet: &str, atomik: bool) -> Program {
    let ek = if atomik { ATOMIK_GIRIS } else { "" };
    let islem = if atomik {
        "anonimAtomikGiris"
    } else {
        "girişte parolayı yükselt"
    };
    let kaynak = format!(
        "{YUKSELT}\n{ek}\nsonuc \"hesap-1\" ve {} ve {} ile {islem} olsun\nsonuc başarılıysa\n    \"gecti\" yaz\ndeğilse\n    \"reddedildi\" yaz\n",
        serde_json::to_string(parola).unwrap(), serde_json::to_string(ozet).unwrap()
    );
    dil::kaynagi_derle(&kaynak).expect("mevcut dilde rehash derlenmeli")
}

fn form_programi() -> Program {
    dil::kaynagi_derle_birimlerle(FORMLAR, &mut |ad| {
        assert_eq!(ad, "turnstile");
        Ok(TURNSTILE.into())
    })
    .expect("mevcut Turnstile birimiyle altı form derlenmeli")
}

fn form_istegi(io: &mut UrunIo, yol: &str, govde: &str, csrf_gecerli: bool) {
    let (oturum, csrf) = io.csrf_hazirla();
    let csrf = if csrf_gecerli { csrf } else { "yanlis".into() };
    io.ic.istekler.push_back(format!(
        "POST {yol}\nçerez __Host-zee-oturum={oturum}\n{govde}&_csrf={csrf}"
    ));
}

#[test]
fn legacy_ilk_giriste_argon2ye_gecer_ikinci_giriste_bcrypt_ve_update_yoktur() {
    for surum in ["2a", "2b", "2y"] {
        let legacy = format!("${surum}{}", &LEGACY[3..]);
        let mut io = UrunIo::legacy(&legacy);
        calistir_io(&giris_programi(PAROLA, &legacy, true), &mut io).unwrap();
        let modern = io.durum.kullanicilar["hesap-1"].clone();
        assert!(modern.starts_with("$argon2id$v=19$"));
        assert!(dil::guvenlik::parola_dogrula(PAROLA, &modern));
        assert_eq!(
            (
                io.ozet_uretimleri,
                io.legacy_dogrulamalar,
                io.durum.oturumlar
            ),
            (1, 1, 1)
        );
        assert_eq!(
            io.sorgular[0].1,
            ["hesap-1", legacy.as_str(), modern.as_str()]
        );
        assert_eq!(io.sorgular[1].1, ["hesap-1", modern.as_str()]);
        calistir_io(&giris_programi(PAROLA, &modern, true), &mut io).unwrap();
        assert_eq!(io.ic.cikti, ["gecti", "gecti"]);
        assert_eq!(
            (
                io.ozet_uretimleri,
                io.legacy_dogrulamalar,
                io.modern_dogrulamalar
            ),
            (1, 1, 1)
        );
        assert_eq!(io.sorgular.iter().filter(|(s, _)| s == GUNCELLE).count(), 1);
        assert_eq!(io.durum.oturumlar, 2);
    }
}

#[test]
fn yanlis_bozuk_ve_uzun_parola_hicbir_ozet_ve_sql_uretmez() {
    for (parola, ozet) in [
        ("yanlis", LEGACY),
        (PAROLA, "$2x$11$bozuk"),
        (PAROLA, "bozuk"),
    ] {
        let mut io = UrunIo::legacy(ozet);
        calistir_io(&giris_programi(parola, ozet, true), &mut io).unwrap();
        assert_eq!(io.ic.cikti, ["reddedildi"]);
        assert_eq!(io.ozet_uretimleri, 0);
        assert!(io.sorgular.is_empty());
        assert_eq!(io.durum.kullanicilar["hesap-1"], ozet);
    }
    let uzun = "x".repeat(73);
    let mut io = UrunIo::legacy(LEGACY);
    calistir_io(&giris_programi(&uzun, LEGACY, false), &mut io).unwrap();
    assert_eq!(io.ic.cikti, ["reddedildi"]);
    assert_eq!(io.ozet_uretimleri, 0);
    assert!(io.sorgular.is_empty());
}

#[test]
fn iki_eski_snapshot_ve_parola_degisim_yarisi_yeni_ozeti_ezmez() {
    let program = giris_programi(PAROLA, LEGACY, true);
    let mut io = UrunIo::legacy(LEGACY);
    calistir_io(&program, &mut io).unwrap();
    let ilk = io.durum.kullanicilar["hesap-1"].clone();
    calistir_io(&program, &mut io).unwrap();
    assert_eq!(io.ic.cikti, ["gecti", "reddedildi"]);
    assert_eq!(io.durum.kullanicilar["hesap-1"], ilk);
    assert_eq!(io.durum.oturumlar, 1);
    let degisen = dil::guvenlik::parola_ozeti_uret("baska-parola").unwrap();
    let mut io = UrunIo::legacy(&degisen);
    calistir_io(&program, &mut io).unwrap();
    assert_eq!(io.ic.cikti, ["reddedildi"]);
    assert_eq!(io.durum.kullanicilar["hesap-1"], degisen);
    assert_eq!(io.durum.oturumlar, 0);
    // Modern doğrulama da eski snapshot ile oturum bağlayamaz. Helper
    // yeniden özet üretmez; dış eylemin exact özet koşulu girişi reddeder.
    let mut io = UrunIo::legacy(&degisen);
    calistir_io(&giris_programi(PAROLA, &ilk, true), &mut io).unwrap();
    assert_eq!(io.ic.cikti, ["reddedildi"]);
    assert_eq!(io.durum.kullanicilar["hesap-1"], degisen);
    assert_eq!(io.durum.oturumlar, 0);
    assert_eq!(io.ozet_uretimleri, 0);
    assert_eq!(io.sorgular.len(), 1);
    assert_eq!(io.sorgular[0].0, BAGLA);
}

#[test]
fn sql_ve_oturum_baglama_hatasi_tum_dis_eylemi_geri_alir() {
    for bind in [false, true] {
        let mut io = UrunIo::legacy(LEGACY);
        io.degisim_hatasi = !bind;
        io.baglama_hatasi = bind;
        calistir_io(&giris_programi(PAROLA, LEGACY, true), &mut io).unwrap();
        assert_eq!(io.ic.cikti, ["reddedildi"]);
        assert_eq!(io.durum.kullanicilar["hesap-1"], LEGACY);
        assert_eq!(io.durum.oturumlar, 0);
        assert!(io.geri_almalar >= 1);
        assert!(io.yedekler.is_empty());
        assert_eq!(io.sorgular.len(), if bind { 2 } else { 1 });
    }
}

#[test]
fn ozet_uretimi_ve_commit_hatasi_girisi_basari_gibi_gostermez() {
    let mut io = UrunIo::legacy(LEGACY);
    io.ozet_hatasi = true;
    let hata = calistir_io(&giris_programi(PAROLA, LEGACY, true), &mut io).unwrap_err();
    assert_eq!(hata.kod, "C028");
    assert!(io.sorgular.is_empty());
    assert!(io.ic.cikti.is_empty());
    assert_eq!(io.durum.kullanicilar["hesap-1"], LEGACY);
    let mut io = UrunIo::legacy(LEGACY);
    io.commit = Commit::Red;
    let hata = calistir_io(&giris_programi(PAROLA, LEGACY, true), &mut io).unwrap_err();
    assert_eq!(hata.kod, "C021");
    assert!(io.ic.cikti.is_empty());
    assert_eq!(io.durum.kullanicilar["hesap-1"], LEGACY);
    assert_eq!(io.durum.oturumlar, 0);
}

#[test]
fn commit_belirsizliginde_iki_db_sonucu_ayni_c027dir_otomatik_tekrar_yoktur() {
    for (commit, kayit_var) in [
        (Commit::BelirsizKayitYok, false),
        (Commit::BelirsizKayitVar, true),
    ] {
        let mut io = UrunIo::legacy(LEGACY);
        io.commit = commit;
        let hata = calistir_io(&giris_programi(PAROLA, LEGACY, true), &mut io).unwrap_err();
        assert_eq!(hata.kod, "C027");
        assert!(io.ic.cikti.is_empty());
        assert_eq!(io.ozet_uretimleri, 1);
        assert_eq!(io.sorgular.len(), 2);
        assert_eq!(io.durum.oturumlar, usize::from(kayit_var));
        assert_eq!(
            io.durum.kullanicilar["hesap-1"].starts_with("$argon2id$"),
            kayit_var
        );
        assert!(io.yedekler.is_empty());
    }
}

#[test]
fn alti_form_gecerli_turnstile_ile_bir_kayit_bir_bildirim_ve_yonlendirme_verir() {
    let program = form_programi();
    for yol in [
        "/iletisim-gonder",
        "/katalog-talep-gonder",
        "/teklif-talep",
        "/randevu-talep",
        "/kariyer-basvuru",
        "/ebulten-kayit",
    ] {
        let mut io = UrunIo::yeni();
        let (govde, beklenen) = if yol == "/teklif-talep" {
            ("%27%20OR%20true%20--", "' OR true --")
        } else {
            ("anonim", "anonim")
        };
        form_istegi(
            &mut io,
            yol,
            &format!("cf-turnstile-response=jeton&govde={govde}"),
            true,
        );
        calistir_io(&program, &mut io).unwrap();
        assert_eq!(
            io.ic.sunucu_yanitlari[0].1, "→ /form-sonucu?durum=kaydedildi",
            "{yol}"
        );
        assert_eq!(io.durum.formlar, [(yol.into(), beklenen.into())]);
        assert_eq!(
            io.sorgular[0],
            (FORM_EKLE.into(), vec![yol.into(), beklenen.into()])
        );
        assert_eq!(io.http_cagrilari, 1);
        assert_eq!(io.ic.eposta_kutusu.len(), 1);
        assert!(io.ic.cikti.is_empty());
    }
}

#[test]
fn turnstile_gecersiz_tip_host_timeout_ve_sir_yoklugunda_hicbir_yan_etki_yoktur() {
    let program = form_programi();
    for (cevap, durum) in [
        (
            Ok((200, r#"{"success":false,"hostname":"site.example"}"#.into())),
            403,
        ),
        (
            Ok((200, r#"{"success":true,"hostname":"baska.example"}"#.into())),
            403,
        ),
        (
            Ok((
                200,
                r#"{"success":"true","hostname":"site.example"}"#.into(),
            )),
            403,
        ),
        (Ok((200, r#"{"success":true}"#.into())), 403),
        (Ok((200, "bozuk JSON".into())), 503),
        (Ok((503, "bakim".into())), 503),
        (Err("timeout TEST_SENTINEL".into()), 503),
    ] {
        let mut io = UrunIo::yeni();
        io.http_sonucu = cevap;
        form_istegi(
            &mut io,
            "/iletisim-gonder",
            "cf-turnstile-response=jeton",
            true,
        );
        calistir_io(&program, &mut io).unwrap();
        assert_eq!(io.ic.sunucu_durumlari, [durum]);
        assert!(io.durum.formlar.is_empty());
        assert!(io.sorgular.is_empty());
        assert!(io.ic.eposta_kutusu.is_empty());
        assert!(!format!("{:?}{:?}", io.ic.cikti, io.ic.sunucu_yanitlari).contains("TEST_SENTINEL"));
    }
    let mut io = UrunIo::yeni();
    io.gizli_sonucu = Err("sentetik sır yok".into());
    form_istegi(
        &mut io,
        "/iletisim-gonder",
        "cf-turnstile-response=jeton",
        true,
    );
    calistir_io(&program, &mut io).unwrap();
    assert_eq!(io.ic.sunucu_durumlari, [503]);
    assert_eq!(io.http_cagrilari, 0);
    assert!(io.sorgular.is_empty());
    assert!(io.ic.eposta_kutusu.is_empty());
}

#[test]
fn csrf_eksik_jeton_ve_bal_kupu_http_ve_kayittan_once_reddedilir() {
    let program = form_programi();
    for (govde, csrf, durum) in [
        ("cf-turnstile-response=jeton", false, 403),
        ("govde=anonim", true, 403),
        ("website=bot&cf-turnstile-response=jeton", true, 200),
    ] {
        let mut io = UrunIo::yeni();
        form_istegi(&mut io, "/iletisim-gonder", govde, csrf);
        calistir_io(&program, &mut io).unwrap();
        assert_eq!(io.ic.sunucu_durumlari, [durum]);
        if durum == 200 {
            assert_eq!(
                io.ic.sunucu_yanitlari[0].1,
                "→ /form-sonucu?durum=kaydedildi"
            );
        }
        assert_eq!(io.http_cagrilari, 0);
        assert!(io.sorgular.is_empty());
        assert!(io.durum.formlar.is_empty());
        assert!(io.ic.eposta_kutusu.is_empty());
    }
}

#[test]
fn form_sql_commit_hatasi_bildirim_yapmaz_smtp_hatasi_kaydi_kaybetmez() {
    let program = form_programi();
    for mod_ in [0, 1, 2] {
        let mut io = UrunIo::yeni();
        io.degisim_hatasi = mod_ == 0;
        io.commit = match mod_ {
            1 => Commit::Red,
            2 => Commit::BelirsizKayitVar,
            _ => Commit::Tamam,
        };
        form_istegi(
            &mut io,
            "/iletisim-gonder",
            "cf-turnstile-response=jeton",
            true,
        );
        calistir_io(&program, &mut io).unwrap();
        assert_eq!(io.ic.sunucu_durumlari, [503]);
        assert_eq!(io.sorgular.len(), 1, "otomatik tekrar yok");
        assert!(io.ic.eposta_kutusu.is_empty());
        assert_eq!(io.durum.formlar.len(), usize::from(mod_ == 2));
    }
    let mut io = UrunIo::yeni();
    io.smtp_hatasi = true;
    form_istegi(
        &mut io,
        "/iletisim-gonder",
        "cf-turnstile-response=jeton",
        true,
    );
    calistir_io(&program, &mut io).unwrap();
    assert_eq!(
        io.ic.sunucu_yanitlari[0].1,
        "→ /form-sonucu?durum=kaydedildi"
    );
    assert_eq!(io.durum.formlar.len(), 1);
    assert!(io.ic.eposta_kutusu.is_empty());
}
