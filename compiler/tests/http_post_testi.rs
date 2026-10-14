use dil::yorumlayici::{calistir_io, GirdiCikti, IzKaydedenIo, ToplayanIo};

struct Sahte {
    ic: ToplayanIo,
    yanit: Result<(i64, String), String>,
    cagrilar: usize,
    urun: bool,
}
impl Sahte {
    fn yeni(yanit: Result<(i64, String), String>) -> Self {
        Self {
            ic: ToplayanIo::yeni(Vec::new()),
            yanit,
            cagrilar: 0,
            urun: false,
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
    fn http_istek(
        &mut self,
        u: &str,
        m: &str,
        h: &[(String, String)],
        b: &str,
        t: i64,
    ) -> Result<(i64, String), String> {
        if self.urun {
            assert_eq!(
                u,
                "https://challenges.cloudflare.com/turnstile/v0/siteverify"
            );
            assert_eq!(m, "POST");
            assert_eq!(t, 5000);
            assert_eq!(h, &[("Content-Type".into(), "application/json".into())]);
            let body: serde_json::Value = serde_json::from_str(b).unwrap();
            assert_eq!(body["secret"], "SENTINEL_SECRET");
            assert_eq!(body["response"], "jeton");
            assert_eq!(body["remoteip"], "203.0.113.7");
        } else {
            assert_eq!(u, "https://service.example/siteverify");
            assert_eq!(m, "POST");
            assert_eq!(
                h,
                &[("Authorization".into(), "Bearer SENTINEL_SECRET".into())]
            );
            assert_eq!(b, "response=jeton");
            assert_eq!(t, 5000);
        }
        self.cagrilar += 1;
        self.yanit.clone()
    }
    fn http_gizli_ortam(&mut self, _: &str) -> Result<String, String> {
        self.cagrilar += 1;
        Ok("SENTINEL_SECRET".into())
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
}
const KAYNAK: &str = r#"adres "https://service.example/siteverify" olsun
yontem "POST" olsun
basliklar boş sözlük olsun
basliklarin "Authorization" değeri "Bearer SENTINEL_SECRET" olsun
govde "response=jeton" olsun
sonuc adres yontem basliklar govde 5000 ile HTTP isteğini dene olsun
sonuc başarılıysa
    yanit sonucun değeri olsun
    yanitin "durum" değeri yaz
    yanitin "gövde" değeri yaz
değilse
    sonucun hatası yaz
"#;

#[test]
fn http_post_baslik_govde_durum_ve_metin_sozluk_sonucu_tasinir() {
    let p = dil::kaynagi_derle(KAYNAK).unwrap();
    let mut io = Sahte::yeni(Ok((
        200,
        r#"{"success":true,"hostname":"site.example"}"#.into(),
    )));
    calistir_io(&p, &mut io).unwrap();
    assert_eq!(io.cagrilar, 1);
    assert_eq!(
        io.ic.cikti,
        ["200", r#"{"success":true,"hostname":"site.example"}"#]
    );
}
#[test]
fn http_transport_hatasi_sonuctur_ve_adaptor_sirri_sizmaz() {
    let p = dil::kaynagi_derle(KAYNAK).unwrap();
    let mut io = Sahte::yeni(Err("SENTINEL_SECRET private body".into()));
    calistir_io(&p, &mut io).unwrap();
    assert_eq!(io.cagrilar, 1);
    assert_eq!(io.ic.cikti.len(), 1);
    assert!(io.ic.cikti[0].contains("HTTP işlemi başarısız; sır içeren ayrıntılar gizlendi"));
    assert!(!io.ic.cikti[0].contains("SENTINEL_SECRET"));
}
#[test]
fn http_iz_kaydinda_yurutulmez_ve_iz_disari_verilmez() {
    let p = dil::kaynagi_derle(KAYNAK).unwrap();
    let mut io = IzKaydedenIo::yeni(Sahte::yeni(Ok((200, "SENTINEL_SECRET".into()))));
    calistir_io(&p, &mut io).unwrap();
    assert_eq!(io.ic.cagrilar, 0);
    let hata = io.iz_metni().unwrap_err();
    assert!(!hata.contains("SENTINEL_SECRET"));
    assert!(hata.contains("HTTP"));
}
#[test]
fn gizli_ortam_da_iz_kaydinda_okunmaz() {
    let p = dil::kaynagi_derle(
        r#"sonuc "TURNSTILE_SECRET" gizli ortam değerini okumayı dene olsun
sonuc başarılıysa
    sonucun değeri yaz
"#,
    )
    .unwrap();
    let mut io = IzKaydedenIo::yeni(Sahte::yeni(Ok((200, "".into()))));
    calistir_io(&p, &mut io).unwrap();
    assert_eq!(io.ic.cagrilar, 0);
    assert!(io.iz_metni().is_err());
}
#[test]
fn http_yanlis_baslik_turu_derlemede_reddedilir_ve_bos_sozluk_kabul_edilir() {
    assert_eq!(
        dil::kaynagi_derle(&KAYNAK.replace(
            "basliklarin \"Authorization\" değeri \"Bearer SENTINEL_SECRET\"",
            "basliklarin \"Authorization\" değeri 1"
        ))
        .unwrap_err()
        .kod,
        "T034"
    );
    assert!(dil::kaynagi_derle(&KAYNAK.replace(
        "basliklarin \"Authorization\" değeri \"Bearer SENTINEL_SECRET\" olsun\n",
        ""
    ))
    .is_ok());
}
#[test]
fn http_post_eylem_transactioninda_geri_alinabilir_sayilamaz() {
    let kaynak = "eylem gonder\n    adresi Metin olarak al\n    değer döndürmez\n    basliklar boş sözlük olsun\n    sonuc adres \"POST\" basliklar \"g\" 5000 ile HTTP isteğini dene olsun\n";
    assert_eq!(dil::kaynagi_derle(kaynak).unwrap_err().kod, "T048");
}
#[test]
fn http_gizli_ortam_listesi_sinirli_tekil_ve_ag_izinlidir() {
    let temel="proje \"http\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ\" listesi olsun\nağ_hedefleri \"https://service.example\" listesi olsun\n";
    let b = dil::proje::bildirimi_oku(
        &(temel.to_string() + "http_gizli_değişkenleri \"TURNSTILE_SECRET\" listesi olsun\n"),
    )
    .unwrap();
    assert_eq!(b.http_gizli_degiskenleri, ["TURNSTILE_SECRET"]);
    for liste in [
        "\"A\", \"A\" listesi".to_string(),
        "\"a\" listesi".into(),
        format!(
            "{} listesi",
            (0..17)
                .map(|i| format!("\"A_{i}\""))
                .collect::<Vec<_>>()
                .join(",")
        ),
    ] {
        assert_eq!(
            dil::proje::bildirimi_oku(
                &(temel.to_string() + &format!("http_gizli_değişkenleri {liste} olsun\n"))
            )
            .unwrap_err()
            .kod,
            "P015"
        );
    }
    assert!(dil::proje::bildirimi_oku(
        &((temel.replace("yetkinlikler \"ağ\" listesi olsun\n", ""))
            + "http_gizli_değişkenleri \"A\" listesi olsun\n")
    )
    .is_err());
}

#[test]
fn http_json_tipleri_kanonik_metinle_korur_ve_taklit_true_kabul_edilmez() {
    for (body, expected) in [
        (
            r#"{"success":true,"hostname":"site.example","error-codes":[]}"#,
            vec!["true", r#""site.example""#],
        ),
        (
            r#"{"success":"true","hostname":"other.example"}"#,
            vec![r#""true""#, r#""other.example""#],
        ),
    ] {
        let kaynak=format!("govde {} olsun\nsonuc govde HTTP JSON sözlüğünü okumayı dene olsun\nsonuc başarılıysa\n    veri sonucun değeri olsun\n    verinin \"success\" değeri yaz\n    verinin \"hostname\" değeri yaz\n",serde_json::to_string(body).unwrap());
        let p = dil::kaynagi_derle(&kaynak).unwrap();
        let mut io = ToplayanIo::yeni(Vec::new());
        calistir_io(&p, &mut io).unwrap();
        assert_eq!(io.cikti, expected);
    }
}
#[test]
fn http_json_tekrar_alan_dizi_trailing_derinlik_ve_boyut_hatasi_sonuctur() {
    let mut inputs = vec![
        "[]".to_string(),
        "{\"success\":false,\"success\":true}".into(),
        "{} x".into(),
        format!("{{\"a\":{}0{}}}", "[".repeat(140), "]".repeat(140)),
        " ".repeat(1_048_577),
    ];
    inputs.push(format!(
        "{{{}}}",
        (0..129)
            .map(|i| format!("\"k{i}\":0"))
            .collect::<Vec<_>>()
            .join(",")
    ));
    for body in inputs {
        let kaynak=format!("govde {} olsun\nsonuc govde HTTP JSON sözlüğünü okumayı dene olsun\nsonuc başarısızsa\n    \"reddedildi\" yaz\n",serde_json::to_string(&body).unwrap());
        let p = dil::kaynagi_derle(&kaynak).unwrap();
        let mut io = ToplayanIo::yeni(Vec::new());
        calistir_io(&p, &mut io).unwrap();
        assert_eq!(io.cikti, ["reddedildi"]);
    }
}

#[test]
fn gercek_turnstile_helper_basari_gecersiz_ve_yanlis_hostta_kapali_kalir() {
    let temel = include_str!("../../dogfood/zee-cekirdek/http-post/turnstile.dil");
    let kaynak=format!("{temel}\nkarar \"jeton\" ve \"203.0.113.7\" ve \"site.example\" ile insan doğrula olsun\nkarar yaz\n");
    let p = dil::kaynagi_derle(&kaynak).unwrap();
    for (yanit, beklenen) in [
        (
            Ok((
                200,
                r#"{"success":true,"hostname":"site.example","error-codes":[]}"#.into(),
            )),
            "gecti",
        ),
        (
            Ok((
                200,
                r#"{"success":false,"error-codes":["invalid-input-response"]}"#.into(),
            )),
            "gecersiz",
        ),
        (
            Ok((200, r#"{"success":true,"hostname":"other.example"}"#.into())),
            "gecersiz",
        ),
        (
            Ok((
                200,
                r#"{"success":"true","hostname":"site.example"}"#.into(),
            )),
            "gecersiz",
        ),
        (Ok((200, r#"{"success":true}"#.into())), "gecersiz"),
        (
            Ok((200, r#"{"hostname":"site.example"}"#.into())),
            "erisilemedi",
        ),
        (Ok((200, "bozuk JSON".into())), "erisilemedi"),
        (Ok((503, "service unavailable".into())), "erisilemedi"),
        (Err("network timeout SENTINEL_SECRET".into()), "erisilemedi"),
    ] {
        let mut io = Sahte::yeni(yanit);
        io.urun = true;
        calistir_io(&p, &mut io).unwrap();
        assert_eq!(io.ic.cikti, [beklenen]);
        assert_eq!(io.cagrilar, 2);
    }
}

#[test]
fn cli_ortam_sirri_yalniz_ana_projenin_izin_listesiyle_okur() {
    let tmp = std::env::temp_dir().join(format!("zee-http-env-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let kaynak = "sonuc \"ZEE_HTTP_POST_TEST_SECRET\" gizli ortam değerini okumayı dene olsun\nsonuc başarılıysa\n    sonucun değeri yaz\ndeğilse\n    \"reddedildi\" yaz\n";
    std::fs::write(tmp.join("ana.dil"), kaynak).unwrap();
    let temel = "proje \"http-fixture\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ\" listesi olsun\nağ_hedefleri \"https://service.example\" listesi olsun\n";
    for (izin, beklenen) in [
        ("", "reddedildi"),
        (
            "http_gizli_değişkenleri \"ZEE_HTTP_POST_TEST_SECRET\" listesi olsun\n",
            "TEST_SENTINEL",
        ),
    ] {
        std::fs::write(tmp.join("proje.dil"), format!("{temel}{izin}")).unwrap();
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
            .args(["çalıştır", "."])
            .current_dir(&tmp)
            .env("ZEE_HTTP_POST_TEST_SECRET", "TEST_SENTINEL")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), beklenen);
    }
    std::fs::remove_dir_all(tmp).unwrap();
}
