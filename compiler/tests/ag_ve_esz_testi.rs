//! Son beş golden: HTTP (24), sunucu (25), eşzamanlılık (26), zaman aşımı (27),
//! ESP32 simülatörü (29) — hepsi hermetik IO ile deterministik.

use dil::yorumlayici::{calistir_io, GirdiCikti, ToplayanIo};

fn golden(ad: &str) -> String {
    let yol = format!("{}/../golden/{}", env!("CARGO_MANIFEST_DIR"), ad);
    std::fs::read_to_string(&yol).unwrap_or_else(|_| panic!("golden bulunamadı: {}", yol))
}

#[test]
fn golden_24_http_istemcisi() {
    let program = dil::kaynagi_derle(&golden("24-http-istemcisi.dil")).expect("24 derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.http_yanitlari.insert(
        "https://ornek.dev/durum".into(),
        (200, "çalışıyor".into()),
    );
    calistir_io(&program, &mut io).expect("24 çalışmalı");
    assert_eq!(io.cikti, vec!["Durum: 200", "çalışıyor"]);
}

#[test]
fn http_baglanti_hatasi_turkce() {
    let kaynak = "cevap \"https://yok.example\" adresinden gelen yanıt olsun\ncevabın gövdesini yaz\n";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    let hata = calistir_io(&program, &mut io).expect_err("bağlantı hatası");
    assert_eq!(hata.kod, "C018");
}

#[test]
fn golden_25_web_sunucusu() {
    let program = dil::kaynagi_derle(&golden("25-web-sunucusu.dil")).expect("25 derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.istekler = vec!["/durum".into(), "/selam".into(), "/kayip".into()].into();
    calistir_io(&program, &mut io).expect("25 çalışmalı");
    assert_eq!(
        io.sunucu_yanitlari,
        vec![
            ("/durum".to_string(), "çalışıyor".to_string()),
            ("/selam".to_string(), "Merhaba ziyaretçi".to_string()),
            ("/kayip".to_string(), "aranan sayfa yok: /kayip".to_string()),
        ]
    );
}

#[test]
fn golden_26_paralel_gorevler() {
    let cikti = dil::kaynagi_calistir(&golden("26-paralel-gorevler.dil")).expect("26 çalışmalı");
    assert_eq!(cikti, vec!["Ayşe profili", "Ayşe faturaları"]);
}

#[test]
fn bekle_oncesi_erisim_derleme_hatasi() {
    // RFC-0011 §1: görev sonucuna bekle'den önce erişim T033.
    let kaynak = "\
işlem bir ver
    1 döndür

eşzamanlı olarak
    görev bir ver

görev yaz
hepsini bekle
";
    let hata = dil::kaynagi_calistir(kaynak).expect_err("T033 bekleniyor");
    assert_eq!(hata.kod, "T033");

    let yeniden_atama = "\
işlem bir ver
    1 döndür
eşzamanlı olarak
    görev bir ver
görev 2 olsun
hepsini bekle
";
    assert_eq!(
        dil::kaynagi_derle(yeniden_atama)
            .expect_err("bekleyen sonuç yeniden atanamaz")
            .kod,
        "T033"
    );
}

#[test]
fn scheduler_bekleme_noktalarinda_kaynak_sirasiyla_ilerler() {
    let kaynak = "\
işlem yavaş işi yap
    \"yavaş başladı\" yaz
    2 saniye bekle
    \"yavaş bitti\" yaz
    20 döndür

işlem hızlı işi yap
    \"hızlı başladı\" yaz
    1 saniye bekle
    \"hızlı bitti\" yaz
    10 döndür

eşzamanlı olarak
    yavaş yavaş işi yap
    hızlı hızlı işi yap

hepsini bekle
yavaş yaz
hızlı yaz
";
    let program = dil::kaynagi_derle(kaynak).expect("scheduler kaynağı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("görevler tamamlanmalı");
    assert_eq!(
        io.cikti,
        vec![
            "yavaş başladı",
            "hızlı başladı",
            "hızlı bitti",
            "yavaş bitti",
            "20",
            "10",
        ]
    );
    assert_eq!(io.an_ms(), 2_000, "beklemeler toplanmamalı; en uzunu kazanmalı");
}

#[test]
fn join_gorevleri_baslatir_ve_esit_uyanis_kaynak_sirasidir() {
    let kaynak = "\
işlem a işini yap
    \"a başladı\" yaz
    1 saniye bekle
    \"a bitti\" yaz
    1 döndür
işlem b işini yap
    \"b başladı\" yaz
    1 saniye bekle
    \"b bitti\" yaz
    2 döndür
eşzamanlı olarak
    a a işini yap
    b b işini yap
\"ana kapsam\" yaz
hepsini bekle
";
    assert_eq!(
        dil::kaynagi_calistir(kaynak).expect("eşit uyanışlar çalışmalı"),
        vec!["ana kapsam", "a başladı", "b başladı", "a bitti", "b bitti"]
    );
}

#[test]
fn gorev_hatasi_bekleyen_kardesi_iptal_eder() {
    let kaynak = "\
işlem yavaş işi yap
    \"yavaş başladı\" yaz
    5 saniye bekle
    \"yavaş yan etkisi yasak\" yaz
    1 döndür

işlem bozuk işi yap
    \"bozuk başladı\" yaz
    sonuç 10 un 0 a bölümü olsun
    sonucu döndür

eşzamanlı olarak
    yavaş yavaş işi yap
    bozuk bozuk işi yap

hepsini bekle
\"join sonrası yasak\" yaz
";
    let program = dil::kaynagi_derle(kaynak).expect("iptal kaynağı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    let hata = calistir_io(&program, &mut io).expect_err("bozuk görev yayılmalı");
    assert_eq!(hata.kod, "C003");
    assert!(hata.mesaj.contains("\"bozuk\" görevi başarısız oldu"));
    assert!(hata.mesaj.contains("yavaş"));
    assert_eq!(io.cikti, vec!["yavaş başladı", "bozuk başladı"]);
    assert_eq!(io.an_ms(), 0, "iptal edilen bekleme zamanı ilerletmemeli");
}

#[test]
fn eylem_gorevde_atomik_dilimdir_ve_hatasinda_geri_alinir() {
    let kaynak = "\
eylem bozuk kaydet
    TamSayı döndürür
    \"durum.txt\" dosyasına \"yarım\" yaz
    1 saniye bekle
    sonuç 10 un 0 a bölümü olsun
    sonucu döndür

işlem kardeşi çalıştır
    \"kardeş başlamamalı\" yaz
    1 döndür

eşzamanlı olarak
    bozuk bozuk kaydet
    kardeş kardeşi çalıştır
hepsini bekle
";
    let program = dil::kaynagi_derle(kaynak).expect("eylem görev kaynağı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    let hata = calistir_io(&program, &mut io).expect_err("eylem hatası yayılmalı");
    assert_eq!(hata.kod, "C003");
    assert!(io.dosyalar.is_empty(), "eylem savepoint'i geri alınmalı");
    assert!(io.cikti.is_empty(), "atomik eylemin arasına kardeş girmemeli");
    assert_eq!(io.an_ms(), 1_000);
}

#[test]
fn gorevde_programi_bitir_koku_koduyla_iptal_eder() {
    let kaynak = "\
işlem bitir
    TamSayı döndürür
    programı 7 ile bitir
    1 döndür

işlem kardeşi çalıştır
    \"kardeş başlamamalı\" yaz
    1 döndür

eşzamanlı olarak
    bitiş bitir
    kardeş kardeşi çalıştır
hepsini bekle
";
    let program = dil::kaynagi_derle(kaynak).expect("görev çıkışı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    let kod = dil::yorumlayici::calistir_io_kodla(&program, &mut io)
        .expect("görev kökü olağan bitirmeli");
    assert_eq!(kod, 7);
    assert!(io.cikti.is_empty(), "kardeş görev iptal edilmiş olmalı");
}

#[test]
fn dis_son_tarih_gorev_agacini_birlikte_iptal_eder() {
    let kaynak = "\
işlem uzun işi yap
    \"başladı\" yaz
    5 saniye bekle
    \"deadline sonrası yasak\" yaz
    1 döndür

1 saniye içinde
    eşzamanlı olarak
        uzun uzun işi yap
    hepsini bekle
    \"join sonrası yasak\" yaz
yetişmezse
    \"grup iptal edildi\" yaz
";
    assert_eq!(
        dil::kaynagi_calistir(kaynak).expect("deadline sahibi iptali yakalamalı"),
        vec!["başladı", "grup iptal edildi"]
    );
}

#[test]
fn ic_gorev_agacinin_beklemesi_dis_kardese_yol_verir() {
    let kaynak = "\
işlem iç işi yap
    \"iç başladı\" yaz
    2 saniye bekle
    \"iç bitti\" yaz
    2 döndür

işlem dalı çalıştır
    eşzamanlı olarak
        iç iç işi yap
    hepsini bekle
    \"dal bitti\" yaz
    içi döndür

işlem kardeşi çalıştır
    \"kardeş başladı\" yaz
    1 saniye bekle
    \"kardeş bitti\" yaz
    1 döndür

eşzamanlı olarak
    dal dalı çalıştır
    kardeş kardeşi çalıştır
hepsini bekle
";
    let program = dil::kaynagi_derle(kaynak).expect("iç görev ağacı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("iç görev ağacı tamamlanmalı");
    assert_eq!(
        io.cikti,
        vec![
            "iç başladı",
            "kardeş başladı",
            "kardeş bitti",
            "iç bitti",
            "dal bitti",
        ]
    );
    assert_eq!(io.an_ms(), 2_000);
}

#[test]
fn gorev_grubu_ayni_sozcuksel_kapsamda_join_edilmelidir() {
    let bekle_yok = "\
işlem bir ver
    1 döndür
eşzamanlı olarak
    görev bir ver
";
    assert_eq!(
        dil::kaynagi_derle(bekle_yok).expect_err("açık görev grubu").kod,
        "T051"
    );

    let bos_bekle = "hepsini bekle\n";
    assert_eq!(
        dil::kaynagi_derle(bos_bekle).expect_err("boş join").kod,
        "T051"
    );

    let erken_donus = "\
işlem bir ver
    1 döndür
işlem erken dön
    TamSayı döndürür
    eşzamanlı olarak
        görev bir ver
    2 döndür
    hepsini bekle
";
    assert_eq!(
        dil::kaynagi_derle(erken_donus).expect_err("açık görevle dönüş").kod,
        "T051"
    );

    let ust_uste = "\
işlem bir ver
    1 döndür
eşzamanlı olarak
    ilk bir ver
eşzamanlı olarak
    ikinci bir ver
hepsini bekle
";
    assert_eq!(
        dil::kaynagi_derle(ust_uste).expect_err("üst üste grup").kod,
        "T051"
    );
}

#[test]
fn coklu_tani_gecisi_gecerli_gorev_grubunu_tek_kapsam_sayar() {
    let kaynak = "\
işlem bir ver
    1 döndür
eşzamanlı olarak
    görev bir ver
hepsini bekle
görev yaz
";
    let mut yukleyici = |_: &str| Err("yok".to_string());
    let tanilar = dil::kaynagi_tanilari(kaynak, &mut yukleyici);
    assert!(tanilar.is_empty(), "beklenmeyen tanılar: {:?}", tanilar);
}

#[test]
fn golden_27_zaman_asimi() {
    let program = dil::kaynagi_derle(&golden("27-zaman-asimi.dil")).expect("27 derlenmeli");

    // Yetişen durum: an ölçümleri 0 → 10ms; gövde çıktısı görünür, yetişmezse koşulmaz.
    let mut io = ToplayanIo::yeni(Vec::new());
    io.an_degerleri = vec![0, 10].into();
    io.http_yanitlari.insert(
        "https://ornek.dev/rapor".into(),
        (200, "rapor içeriği".into()),
    );
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti, vec!["rapor içeriği"]);

    // Ağ dönüşünde son tarih aşılmıştır: yanıt değişkene/çıktıya dönüşmeden
    // iptal edilir; yalnız yetişmezse kolu koşulur.
    let mut io = ToplayanIo::yeni(Vec::new());
    io.an_degerleri = vec![0, 0, 0, 9000].into();
    io.http_yanitlari.insert(
        "https://ornek.dev/rapor".into(),
        (200, "rapor içeriği".into()),
    );
    calistir_io(&program, &mut io).expect("çalışmalı");
    assert_eq!(io.cikti, vec!["Zaman aşımı, sonra tekrar dene"]);
}

#[test]
fn zaman_asimi_beklemeyi_keser_ve_sonraki_yan_etkiyi_engeller() {
    let kaynak = "\
5 saniye içinde
    10 saniye bekle
    \"bu çıktı yasak\" yaz
yetişmezse
    \"iptal edildi\" yaz

\"program devam etti\" yaz
";
    let program = dil::kaynagi_derle(kaynak).expect("deadline kaynağı");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("iptal yönetilmeli");
    assert_eq!(io.cikti, vec!["iptal edildi", "program devam etti"]);
}

#[test]
fn son_tarih_dosya_yazma_etkisinin_hemen_onunde_yeniden_denetlenir() {
    let kaynak = "\
1 saniye içinde
    \"durum.txt\" dosyasına \"bu yazı yasak\" yaz
yetişmezse
    \"dosya iptal edildi\" yaz
";
    let program = dil::kaynagi_derle(kaynak).expect("dosya iptal kaynağı");
    let mut io = ToplayanIo::yeni(Vec::new());
    // Deadline kurulumu 0, blok girişi 0; etki kapısında tam 1 saniye.
    io.an_degerleri = vec![0, 0, 1_000].into();
    calistir_io(&program, &mut io).expect("iptal kendi kolunda yönetilmeli");

    assert!(io.dosyalar.is_empty(), "iptal sonrası dosya etkisi başlamamalı");
    assert_eq!(io.cikti, vec!["dosya iptal edildi"]);
}

#[test]
fn gorev_httpden_once_sira_verince_son_tarihi_eski_sureyle_kullanmaz() {
    let kaynak = "\
1 saniye içinde
    eşzamanlı olarak
        cevap \"https://ornek.dev/gec\" adresinden gelen yanıt
        sayı 1
    hepsini bekle
yetişmezse
    \"HTTP iptal edildi\" yaz
";
    let program = dil::kaynagi_derle(kaynak).expect("HTTP iptal kaynağı");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.http_yanitlari.insert(
        "https://ornek.dev/gec".into(),
        (200, "geç yanıt".into()),
    );
    // İlk HTTP görevi 0 ms'de sıra verir. İkinci görev poll'undan sonra saat
    // 1.000 ms olur; HTTP adaptörüne girmeden yeni kontrol iptali görmelidir.
    io.an_degerleri = vec![0, 0, 0, 0, 0, 1_000].into();
    calistir_io(&program, &mut io).expect("dış deadline iptali yakalamalı");

    assert!(
        io.http_istekleri.is_empty(),
        "süresi dolmuş görev HTTP isteği başlatmamalı"
    );
    assert_eq!(io.cikti, vec!["HTTP iptal edildi"]);
}

#[test]
fn ic_ve_dis_son_tarihlerin_sahibi_karistirilmaz() {
    let ic_once = "\
10 saniye içinde
    2 saniye içinde
        5 saniye bekle
        \"iç gövde yasak\" yaz
    yetişmezse
        \"iç iptal\" yaz
    \"dış devam\" yaz
yetişmezse
    \"dış iptal yasak\" yaz
";
    assert_eq!(
        dil::kaynagi_calistir(ic_once).expect("iç deadline"),
        vec!["iç iptal", "dış devam"]
    );

    let dis_once = "\
2 saniye içinde
    10 saniye içinde
        5 saniye bekle
        \"iç gövde yasak\" yaz
    yetişmezse
        \"iç iptal yasak\" yaz
yetişmezse
    \"dış iptal\" yaz
";
    assert_eq!(
        dil::kaynagi_calistir(dis_once).expect("dış deadline"),
        vec!["dış iptal"]
    );
}

#[test]
fn golden_29_esp32_simulatoru() {
    let program = dil::kaynagi_derle(&golden("29-esp32-led.dil")).expect("29 derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.sensorler.insert("kapı".into(), true);
    calistir_io(&program, &mut io).expect("29 çalışmalı");
    let mut beklenen = vec!["[ışık] kırmızı yandı".to_string()];
    for _ in 0..10 {
        beklenen.push("[ışık] mavi yandı".into());
        beklenen.push("[ışık] mavi söndü".into());
    }
    assert_eq!(io.cikti, beklenen);

    // Kapı kapalıyken yeşil yanar (varsayılan sensör durumu).
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("29 çalışmalı");
    assert_eq!(io.cikti[0], "[ışık] yeşil yandı");
}

#[test]
fn golden_31_birimler() {
    let klasor = format!("{}/../golden", env!("CARGO_MANIFEST_DIR"));
    let mut yukleyici = |ad: &str| -> Result<String, String> {
        std::fs::read_to_string(format!("{}/{}.dil", klasor, ad)).map_err(|e| e.to_string())
    };
    let program = dil::kaynagi_derle_birimlerle(&golden("31-birimler.dil"), &mut yukleyici)
        .expect("31 derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("31 çalışmalı");
    assert_eq!(io.cikti, vec!["Ödenecek: 59,88 lira"]);

    // Birimin kendi testi de dene kapsamında.
    let sonuclar = dil::programi_dene(&program);
    assert!(sonuclar.iter().any(|s| s.ad == "hesap_araclari: kdv doğru eklenir"));
    assert!(sonuclar.iter().all(|s| s.hata.is_none()));
}

#[test]
fn golden_32_ondalik_market() {
    let cikti = dil::kaynagi_calistir(&golden("32-ondalik-market.dil")).expect("32 çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "Tutar: 49,975 lira",
            "Yuvarlak: 50 lira",
            "Ondalıklar tam: sürpriz yok",
        ]
    );
}
