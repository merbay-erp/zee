//! K-384 (RFC-0039 A2c): bağlantıda blok okuma — hermetik adaptörle SMTP
//! nokta bloğu, IMAP literal'i, bayt-kesin okuma, satır sonusuz parça,
//! dosyadan aktarım, kısa süreli okuma ve IO izi.

use dil::yorumlayici::{
    calistir_io, calistir_io_kodla, GirdiCikti, IzKaydedenIo, IzYenidenOynatici, SahteBaglanti,
    ToplayanIo,
};

/// SMTP alıcısının DATA kesiti: blok sonucu (bayt, sha256) ya da tür yazılır.
const DATA: &str = r#"
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    açık doğru olsun
    açık olduğu sürece
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            metin komutun değeri olsun
            metin "DATA" e eşitse
                bağlantıya "354 bitir" satırını yaz
                ileti bağlantıdan nokta bloğunu dosyaya almayı dene olsun
                ileti başarılıysa
                    bilgi iletinin değeri olsun
                    bağlantıya "250 " ile bilginin "bayt" değeri ile " " ile bilginin "sha256" değeri satırını yaz
                    bilginin "yol" değeri yaz
                değilse
                    hata iletinin hatası olsun
                    veri hatanın verisi olsun
                    bağlantıya "554 " ile verinin "tür" değeri satırını yaz
            değilse metin "QUIT" e eşitse
                bağlantıya "221 hoşça kal" satırını yaz
                açık yanlış olsun
            değilse
                bağlantıya "250 " ile metin satırını yaz
        değilse
            hata komutun hatası olsun
            veri hatanın verisi olsun
            tür verinin "tür" değeri olsun
            son bağlantıya "421 " ile tür satırını yazmayı dene olsun
            açık yanlış olsun
"#;

fn calistir(kaynak: &str, io: &mut ToplayanIo) -> i64 {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    calistir_io_kodla(&program, io).expect("çalışmalı")
}

fn giden(io: &ToplayanIo) -> Vec<String> {
    io.baglanti.baglantilar[0].giden.clone()
}

fn sha(icerik: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(icerik)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn buyuk_nokta_blogu_parca_parca_akar_dolgu_cozulur_sonraki_komut_okunur() {
    let mut io = ToplayanIo::yeni(Vec::new());
    // 3 MiB'lık gövde 64 KiB'lık teslimlerle; satır başında nokta doldurulmuş.
    let satir = format!("{}\r\n", "x".repeat(998));
    let mut govde = Vec::new();
    let mut beklenen = Vec::new();
    while govde.len() < 3 * 1024 * 1024 {
        govde.extend_from_slice(b"..nokta\r\n");
        beklenen.extend_from_slice(b".nokta\r\n");
        govde.extend_from_slice(satir.as_bytes());
        beklenen.extend_from_slice(satir.as_bytes());
    }
    let mut akis = b"DATA\r\n".to_vec();
    akis.extend_from_slice(&govde);
    akis.extend_from_slice(b".\r\nQUIT\r\n");
    let parcalar: Vec<&[u8]> = akis.chunks(64 * 1024).collect();
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::yeni("smtp_v4", "192.0.2.10", &parcalar));
    calistir(DATA, &mut io);
    assert_eq!(
        giden(&io),
        vec![
            "354 bitir".to_string(),
            format!("250 {} {}", beklenen.len(), sha(&beklenen)),
            "221 hoşça kal".to_string(),
        ]
    );
    assert_eq!(io.baglanti.baglantilar[0].bloklar, vec![beklenen]);
    let yol = &io.cikti[0];
    assert!(
        yol.starts_with(".zee/baglanti-bloklari/") && yol.ends_with(".parca"),
        "{yol}"
    );
    assert!(io.dosyalar.contains_key(yol));
}

#[test]
fn blok_siniri_asilinca_sonlandiriciya_kadar_tuketilir_oturum_surer() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.blok_siniri = 1024;
    let mut akis = b"DATA\r\n".to_vec();
    akis.extend_from_slice(&[b'a'; 5000]);
    akis.extend_from_slice(b"\r\n.\r\nNOOP\r\nQUIT\r\n");
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::yeni("smtp_v4", "192.0.2.11", &[&akis]));
    calistir(DATA, &mut io);
    assert_eq!(
        giden(&io),
        vec!["354 bitir", "554 boyut_aşımı", "250 NOOP", "221 hoşça kal"]
    );
    assert!(io.baglanti.baglantilar[0].bloklar.is_empty());
}

#[test]
fn smtp_smuggling_ciplak_lf_nokta_komut_olarak_gorunmez() {
    // <LF>.<LF> ve <LF>.<CR><LF> sonlandırıcı değildir; kaçak MAIL FROM blokla
    // birlikte atılır, gerçek CRLF.CRLF'den sonra oturum eşzamanlı sürer.
    for kacak in [
        &b"DATA\r\nSubject: a\r\n\n.\nMAIL FROM:<kotu@ornek.test>\r\nRCPT TO:<x@y>\r\n.\r\nQUIT\r\n"[..],
        b"DATA\r\nSubject: a\r\n\n.\r\nMAIL FROM:<kotu@ornek.test>\r\n.\r\nQUIT\r\n",
        b"DATA\r\nSubject: a\r.\rMAIL FROM:<kotu@ornek.test>\r\n.\r\nQUIT\r\n",
    ] {
        let mut io = ToplayanIo::yeni(Vec::new());
        io.baglanti
            .bekleyenler
            .push_back(SahteBaglanti::yeni("smtp_v4", "192.0.2.12", &[kacak]));
        calistir(DATA, &mut io);
        assert_eq!(
            giden(&io),
            vec!["354 bitir", "554 satır_sonu", "221 hoşça kal"],
            "{kacak:?}"
        );
        assert!(io.baglanti.baglantilar[0].bloklar.is_empty());
    }
}

#[test]
fn blok_bitmeden_kesilen_baglanti_kapandi_doner() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.bekleyenler.push_back(SahteBaglanti::yeni(
        "smtp_v4",
        "192.0.2.13",
        &[b"DATA\r\nSubject: yarim\r\n", b"govde"],
    ));
    calistir(DATA, &mut io);
    // 421 yazımı da denenir; karşı taraf gitmiştir (hermetikte kuyruğa alınır).
    assert_eq!(giden(&io), vec!["354 bitir", "554 kapandı", "421 kapandı"]);
    assert!(io.baglanti.baglantilar[0].bloklar.is_empty());
}

#[test]
fn yavas_istemci_blok_suresinde_kesilir_okuma_yonu_kapanir_yazma_surer() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let mut b = SahteBaglanti::yeni("smtp_v4", "192.0.2.14", &[b"DATA\r\nSubject: a\r\n"]);
    b.sessiz_kalir = true;
    io.baglanti.bekleyenler.push_back(b);
    calistir(DATA, &mut io);
    // Blok yarıda kaldı: zaman_aşımı; sonraki okuma eşzamansız akışı komut
    // olarak yorumlamaz (`kapandı`) ama 421 yazılabilir.
    assert_eq!(
        giden(&io),
        vec!["354 bitir", "554 zaman_aşımı", "421 kapandı"]
    );
}

const IMAP: &str = r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    satır bağlantıdan satır okumayı dene olsun
    satır başarılıysa
        bağlantıya "+ " parçasını yaz
        bağlantıya "hazır" satırını yaz
        ek bağlantıdan 12 baytı dosyaya almayı dene olsun
        ek başarılıysa
            bilgi ekin değeri olsun
            bağlantıya "A1 OK " ile bilginin "bayt" değeri satırını yaz
        değilse
            hata ekin hatası olsun
            veri hatanın verisi olsun
            bağlantıya "A1 NO " ile verinin "tür" değeri satırını yaz
        parola bağlantıdan 5 bayt okumayı dene olsun
        parola başarılıysa
            bağlantıya "A2 OK " ile parolanın değeri satırını yaz
        değilse
            hata parolanın hatası olsun
            veri hatanın verisi olsun
            bağlantıya "A2 NO " ile verinin "tür" değeri satırını yaz
        sonraki bağlantıdan satır okumayı dene olsun
        sonraki başarılıysa
            bağlantıya "A3 " ile sonrakinin değeri satırını yaz
"#;

#[test]
fn literal_tam_bayt_alir_fazlasi_sonraki_komuttur_parca_satir_sonu_eklemez() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.bekleyenler.push_back(SahteBaglanti::yeni(
        "imap_v4",
        "192.0.2.20",
        &[
            b"A1 APPEND kutu {12}\r\n",
            b"Konu: a\r\n\r\nX",
            b"gizliA3 NOOP\r\n",
        ],
    ));
    calistir(IMAP, &mut io);
    let b = &io.baglanti.baglantilar[0];
    assert_eq!(b.bloklar, vec![b"Konu: a\r\n\r\nX".to_vec()]);
    assert_eq!(
        b.giden,
        vec!["hazır", "A1 OK 12", "A2 OK gizli", "A3 A3 NOOP"]
    );
    // `+ ` satır sonusuz, ardından satır CRLF ile: ham akış birebir.
    assert!(b
        .ham_giden
        .starts_with("+ hazır\r\nA1 OK 12\r\n".as_bytes()));
}

#[test]
fn literal_eksik_bayt_kapandi_sinir_asimi_hic_tuketmez() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.bekleyenler.push_back(SahteBaglanti::yeni(
        "imap_v4",
        "192.0.2.21",
        &[b"A1 APPEND kutu {12}\r\n", b"kisa"],
    ));
    calistir(IMAP, &mut io);
    assert_eq!(giden(&io), vec!["hazır", "A1 NO kapandı", "A2 NO kapandı"]);

    // Bildirilen boyut sınırı aşarsa hiçbir bayt okunmaz; Zee NO yazar.
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.blok_siniri = 1024;
    io.baglanti.ayar.satir_siniri = 64;
    let kaynak = r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    ek bağlantıdan 2048 baytı dosyaya almayı dene olsun
    ek başarısızsa
        hata ekin hatası olsun
        veri hatanın verisi olsun
        bağlantıya "NO " ile verinin "tür" değeri satırını yaz
    bayt bağlantıdan 65 bayt okumayı dene olsun
    bayt başarısızsa
        hata baytın hatası olsun
        veri hatanın verisi olsun
        bağlantıya "NO " ile verinin "tür" değeri satırını yaz
    negatif bağlantıdan -1 bayt okumayı dene olsun
    negatif başarısızsa
        hata negatifin hatası olsun
        veri hatanın verisi olsun
        bağlantıya "NO " ile verinin "tür" değeri satırını yaz
    satır bağlantıdan satır okumayı dene olsun
    satır başarılıysa
        bağlantıya "OK " ile satırın değeri satırını yaz
"#;
    io.baglanti.bekleyenler.push_back(SahteBaglanti::satirlarla(
        "imap_v4",
        "192.0.2.22",
        &["A2 NOOP"],
    ));
    calistir(kaynak, &mut io);
    assert_eq!(
        giden(&io),
        vec![
            "NO boyut_aşımı",
            "NO boyut_aşımı",
            "NO geçersiz",
            "OK A2 NOOP"
        ]
    );
}

#[test]
fn parca_yazma_enjeksiyonu_reddeder_bayt_okuma_nul_kodlamadir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::yeni("imap_v4", "192.0.2.23", &[b"a\0b"]));
    let kaynak = r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    parça bağlantıya "+ " parçasını yazmayı dene olsun
    parça başarılıysa
        bağlantıya "yazıldı " ile parçanın değeri satırını yaz
    okunan bağlantıdan 3 bayt okumayı dene olsun
    okunan başarısızsa
        hata okunanın hatası olsun
        veri hatanın verisi olsun
        bağlantıya "NO " ile verinin "tür" değeri satırını yaz
"#;
    calistir(kaynak, &mut io);
    assert_eq!(giden(&io), vec!["yazıldı 2", "NO kodlama"]);
    assert_eq!(
        io.baglanti.baglantilar[0].ham_giden,
        "+ yazıldı 2\r\nNO kodlama\r\n".as_bytes()
    );
}

#[test]
fn sureli_okuma_zaman_asiminda_oturum_surer_bosta_suresini_asmaz() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.bosta_suresi_ms = 1_800_000;
    let mut b = SahteBaglanti::satirlarla("imap_v4", "192.0.2.24", &["A1 IDLE"]);
    b.sessiz_kalir = true;
    io.baglanti.bekleyenler.push_back(b);
    let kaynak = r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    ilk bağlantıdan satır okumayı dene olsun
    bağlantıya "+ idling" satırını yaz
    3 kez tekrarla
        satır bağlantıdan satır okumayı 30 saniye içinde dene olsun
        satır başarısızsa
            hata satırın hatası olsun
            veri hatanın verisi olsun
            bağlantıya "* yoklama " ile verinin "tür" değeri satırını yaz
    sıfır bağlantıdan satır okumayı 0 saniye içinde dene olsun
    sıfır başarısızsa
        hata sıfırın hatası olsun
        veri hatanın verisi olsun
        bağlantıya "* " ile verinin "tür" değeri satırını yaz
"#;
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    calistir_io_kodla(&program, &mut io).expect("çalışmalı");
    assert_eq!(
        giden(&io),
        vec![
            "+ idling",
            "* yoklama zaman_aşımı",
            "* yoklama zaman_aşımı",
            "* yoklama zaman_aşımı",
            "* geçersiz"
        ]
    );
    // Bağlantı çalışma zamanınca kapatılmadı; blok bitince kendiliğinden kapanır.
    assert!(io.baglanti.baglantilar[0].kapandi);
    // Üç kısa bekleme 30'ar sn sanal saat ilerletir (1800 sn değil).
    assert!(io.an_ms() < 1_800_000, "sanal saat {}", io.an_ms());
}

#[test]
fn dosya_baglantiya_aktarilir_sinir_ve_yokluk_sonuctur() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.blok_siniri = 1024;
    io.dosyalar
        .insert("kutu/1.eml".into(), "Konu: a\r\n\r\ngövde\r\n".into());
    io.dosyalar
        .insert("kutu/buyuk.eml".into(), "x".repeat(2000));
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla("imap_v4", "192.0.2.25", &[]));
    let kaynak = r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    bağlantıya "* 1 FETCH (BODY[] {19}" satırını yaz
    küçük bağlantıya "kutu/1.eml" dosyasını aktarmayı dene olsun
    bağlantıya ")" satırını yaz
    küçük başarılıysa
        küçüğün değeri yaz
    büyük bağlantıya "kutu/buyuk.eml" dosyasını aktarmayı dene olsun
    büyük başarısızsa
        hata büyüğün hatası olsun
        veri hatanın verisi olsun
        verinin "tür" değeri yaz
    olmayan bağlantıya "kutu/yok.eml" dosyasını aktarmayı dene olsun
    olmayan başarısızsa
        hata olmayanın hatası olsun
        veri hatanın verisi olsun
        verinin "tür" değeri yaz
"#;
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    calistir_io_kodla(&program, &mut io).expect("çalışmalı");
    assert_eq!(
        io.baglanti.baglantilar[0].ham_giden,
        "* 1 FETCH (BODY[] {19}\r\nKonu: a\r\n\r\ngövde\r\n)\r\n".as_bytes()
    );
    assert_eq!(io.cikti, vec!["19", "boyut_aşımı", "dosya"]);
}

#[test]
fn blok_iz_kaydi_replay_ayni_ciktiyi_verir() {
    let mut io = IzKaydedenIo::yeni(ToplayanIo::yeni(Vec::new()));
    io.ic.baglanti.bekleyenler.push_back(SahteBaglanti::yeni(
        "smtp_v4",
        "192.0.2.30",
        &[b"DATA\r\nKonu: iz\r\n", b"\r\nmetin\r\n.\r\nQUIT\r\n"],
    ));
    let program = dil::kaynagi_derle(DATA).unwrap();
    calistir_io(&program, &mut io).unwrap();
    let iz = io.iz_metni().unwrap();
    let ic = io.icine_al();
    // "nokta" işlemi izde onaltılık alan olarak durur.
    assert!(iz.contains("6e6f6b7461"), "{iz}");
    let beklenen = b"Konu: iz\r\n\r\nmetin\r\n";
    assert_eq!(ic.baglanti.baglantilar[0].bloklar, vec![beklenen.to_vec()]);

    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    calistir_io(&program, &mut oynatici).unwrap();
    assert_eq!(oynatici.uyusmazlik(), None);
    assert_eq!(oynatici.cikti, ic.cikti);
}

#[test]
fn blok_kaliplari_yalniz_baglantida_ve_turlu_arguman_ister() {
    let program =
        dil::kaynagi_derle("x bağlantıdan nokta bloğunu dosyaya almayı dene olsun\n").unwrap();
    let hata = calistir_io(&program, &mut ToplayanIo::yeni(Vec::new())).unwrap_err();
    assert_eq!(hata.kod, "C030");
    let hata = dil::kaynagi_derle("x bağlantıdan \"beş\" bayt okumayı dene olsun\n").unwrap_err();
    assert_eq!(hata.kod, "T034", "{}", hata.mesaj);
    let hata = dil::kaynagi_derle("x bağlantıya 5 dosyasını aktarmayı dene olsun\n").unwrap_err();
    assert_eq!(hata.kod, "T034", "{}", hata.mesaj);
}

/// K-384 (ZeeMail gözlemleri): yerleşik `yaz`/`gönder` ile biten işlem adı
/// cümle biçiminde çağrılınca tanı ad seçimi ipucu verir; örtük `bağlantı`
/// Sözlüğü işlemde görünmez ve A001 bunu söyler. Kod ve ret davranışı değişmez.
#[test]
fn ad_cakismasi_ve_ortuk_baglanti_tanisi_ipucu_verir() {
    let yaz = dil::kaynagi_derle(
        "işlem yanıtı yaz\n    metni al\n    metin yaz\n\n\"250 tamam\" ile yanıtı yaz\n",
    )
    .unwrap_err();
    assert_eq!(yaz.kod, "A001");
    let oneri = yaz.oneri.unwrap_or_default();
    assert!(
        oneri.contains("\"yanıtı yaz\" işlemi yerleşik \"yaz\""),
        "{oneri}"
    );
    assert!(oneri.contains("işlem yanıtı ilet"), "{oneri}");

    let gonder = dil::kaynagi_derle(
        "işlem yanıt satırlarını gönder\n    satırları al\n    satırlar yaz\n\n\"250 tamam\" ile yanıt satırlarını gönder\n",
    )
    .unwrap_err();
    assert_eq!(gonder.kod, "S037");
    assert!(gonder
        .oneri
        .unwrap_or_default()
        .contains("yerleşik \"gönder\" cümlesiyle çakışıyor"));

    // İfade konumunda aynı işlem çağrılabilir (davranış değişmedi).
    let program = dil::kaynagi_derle(
        "işlem yanıtı yaz\n    metni al\n    metni döndür\n\nx \"a\" ile yanıtı yaz olsun\nx yaz\n",
    )
    .expect("ifade çağrısı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).unwrap();
    assert_eq!(io.cikti, vec!["a"]);

    let baglanti = dil::kaynagi_derle(
        "işlem adresi bildir\n    bağlantının \"uzak_adres\" değeri yaz\n\n\"smtp_v4\" dinleyicisinde bağlantı geldiğinde\n    adresi bildir\n",
    )
    .unwrap_err();
    assert_eq!(baglanti.kod, "A001");
    assert!(baglanti
        .oneri
        .unwrap_or_default()
        .contains("parametre olarak geçir"));
}

/// K-396: kütüphaneden bağımsız başvuru — bit tamponlu RFC 4648 alfabesi,
/// 76 karakterde bir ve en sonda CRLF (RFC 2045 §6.8).
pub fn basvuru_base64_satirli(veri: &[u8]) -> Vec<u8> {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let (mut kod, mut tampon, mut bit) = (Vec::new(), 0u32, 0u32);
    for &b in veri {
        tampon = (tampon << 8) | u32::from(b);
        bit += 8;
        while bit >= 6 {
            bit -= 6;
            kod.push(A[((tampon >> bit) & 63) as usize]);
        }
    }
    if bit > 0 {
        kod.push(A[((tampon << (6 - bit)) & 63) as usize]);
    }
    while kod.len() % 4 != 0 {
        kod.push(b'=');
    }
    let mut cikti = Vec::new();
    for satir in kod.chunks(76) {
        cikti.extend_from_slice(satir);
        cikti.extend_from_slice(b"\r\n");
    }
    cikti
}

/// K-396: tek aktarım satırı olan hermetik bağlantı; (ham giden, çıktı).
fn base64_aktar(icerik: &str, satir: &str, blok_siniri: u64) -> (Vec<u8>, Vec<String>) {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.baglanti.ayar.blok_siniri = blok_siniri;
    io.dosyalar.insert("ekler/a.bin".into(), icerik.into());
    io.baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla("imap_v4", "192.0.2.26", &[]));
    let kaynak = format!(
        r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    aktarım {satir} olsun
    aktarım başarılıysa
        aktarımın değeri yaz
    değilse
        hata aktarımın hatası olsun
        veri hatanın verisi olsun
        verinin "tür" değeri yaz
"#
    );
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    calistir_io_kodla(&program, &mut io).expect("çalışmalı");
    (io.baglanti.baglantilar[0].ham_giden.clone(), io.cikti)
}

fn ascii_icerik(boy: usize) -> String {
    (0..boy)
        .map(|i| (b' ' + (i * 7 % 95) as u8) as char)
        .collect()
}

#[test]
fn base64_satirli_aktarim_basvuruyla_bayt_bayt_ve_formul_uzunlugunda() {
    for boy in [0usize, 1, 2, 3, 57, 58, 114, 1000, 70_000] {
        let icerik = ascii_icerik(boy);
        let (giden, cikti) = base64_aktar(
            &icerik,
            "\"ekler/a.bin\" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene",
            1 << 20,
        );
        let beklenen = basvuru_base64_satirli(icerik.as_bytes());
        assert_eq!(giden, beklenen, "boy={boy}");
        let e = 4 * boy.div_ceil(3);
        let formul = e + 2 * e.div_ceil(76);
        assert_eq!(cikti, vec![formul.to_string()], "boy={boy}");
        assert!(giden.split(|b| *b == b'\n').all(|s| s.len() <= 77));
    }
}

#[test]
fn base64_aralik_kodlanmis_ciktinin_kenarlarinda_dilimle_esit() {
    let icerik = ascii_icerik(200);
    let tum = basvuru_base64_satirli(icerik.as_bytes());
    let toplam = tum.len() as i64;
    for (konum, adet) in [
        (0, 0),
        (0, 1),
        (0, 1024),
        (75, 3),
        (76, 2),
        (77, 1),
        (78, 78),
        (79, 200),
        (toplam - 1, 5),
        (toplam, 10),
        (toplam + 5, 1),
        (3, i64::MAX),
    ] {
        let (giden, cikti) = base64_aktar(
            &icerik,
            &format!("\"ekler/a.bin\" dosyasının base64 satırlarının {konum} baytından {adet} baytını bağlantıya aktarmayı dene"),
            1 << 20,
        );
        let bas = konum.min(toplam) as usize;
        let son = konum.saturating_add(adet).min(toplam) as usize;
        assert_eq!(giden, &tum[bas..son], "konum={konum} adet={adet}");
        assert_eq!(cikti, vec![(son - bas).to_string()]);
    }
}

#[test]
fn base64_aktarim_negatif_aralik_yok_ve_sinir_sonuctur() {
    let icerik = ascii_icerik(100);
    let aralik = |konum: &str, adet: &str| {
        format!("\"ekler/a.bin\" dosyasının base64 satırlarının {konum} baytından {adet} baytını bağlantıya aktarmayı dene")
    };
    for satir in [aralik("-1", "5"), aralik("0", "-1")] {
        let (giden, cikti) = base64_aktar(&icerik, &satir, 1 << 20);
        assert!(giden.is_empty());
        assert_eq!(cikti, vec!["geçersiz"]);
    }
    let (giden, cikti) = base64_aktar(
        &icerik,
        "\"ekler/yok.bin\" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene",
        1 << 20,
    );
    assert!(giden.is_empty());
    assert_eq!(cikti, vec!["dosya"]);
    // Sınır ham dosya boyuna uygulanır (kodlanmış çıktı ~%37 büyür).
    let (giden, cikti) = base64_aktar(
        &icerik,
        "\"ekler/a.bin\" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene",
        99,
    );
    assert!(giden.is_empty());
    assert_eq!(cikti, vec!["boyut_aşımı"]);
    let (_, cikti) = base64_aktar(
        &icerik,
        "\"ekler/a.bin\" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene",
        100,
    );
    assert_eq!(cikti, vec!["140"]);
}

#[test]
fn base64_aktarim_kaliplari_turlu_arguman_ister_ve_iz_replay_eder() {
    for kaynak in [
        "x 5 dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun\n",
        "x \"a\" dosyasının base64 satırlarının \"0\" baytından 1 baytını bağlantıya aktarmayı dene olsun\n",
        "x \"a\" dosyasının base64 satırlarının 0 baytından doğru baytını bağlantıya aktarmayı dene olsun\n",
    ] {
        let hata = dil::kaynagi_derle(kaynak).unwrap_err();
        assert_eq!(hata.kod, "T034", "{kaynak}: {}", hata.mesaj);
    }
    let program = dil::kaynagi_derle(
        "x \"a\" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun\n",
    )
    .unwrap();
    let hata = calistir_io(&program, &mut ToplayanIo::yeni(Vec::new())).unwrap_err();
    assert_eq!(hata.kod, "C030");

    let kaynak = r#"
"imap_v4" dinleyicisinde bağlantı geldiğinde
    tüm "ekler/a.bin" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun
    parça "ekler/a.bin" dosyasının base64 satırlarının 70 baytından 20 baytını bağlantıya aktarmayı dene olsun
    tüm başarılıysa
        parça başarılıysa
            tümün değeri ile " " ile parçanın değeri yaz
"#;
    let mut io = IzKaydedenIo::yeni(ToplayanIo::yeni(Vec::new()));
    io.ic
        .dosyalar
        .insert("ekler/a.bin".into(), ascii_icerik(90));
    io.ic
        .baglanti
        .bekleyenler
        .push_back(SahteBaglanti::satirlarla("imap_v4", "192.0.2.27", &[]));
    let program = dil::kaynagi_derle(kaynak).unwrap();
    calistir_io(&program, &mut io).unwrap();
    let iz = io.iz_metni().unwrap();
    let ic = io.icine_al();
    assert_eq!(ic.cikti, vec!["124 20"]);
    // İz içeriksizdir: yol ve aralık var, dosya içeriği ya da base64 yok.
    let hex = |m: &str| m.bytes().map(|b| format!("{b:02x}")).collect::<String>();
    assert!(
        iz.contains(&hex("aktar_b64")) && iz.contains(&hex("70+20")),
        "{iz}"
    );
    let kod = basvuru_base64_satirli(ascii_icerik(90).as_bytes());
    assert!(
        !iz.contains(&hex(std::str::from_utf8(&kod[..20]).unwrap())),
        "{iz}"
    );
    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    calistir_io(&program, &mut oynatici).unwrap();
    assert_eq!(oynatici.uyusmazlik(), None);
    assert_eq!(oynatici.cikti, ic.cikti);
}
