//! K-408 (ADR-172 eki, RFC-0039 A2d eki, ZM-F049): `<kaynak> dosyasını base64
//! satırlarıyla <hedef> dosyasına eklemeyi dene` — K-396 bağlantı aktarımının
//! dosya hedefli aynası. Hermetik adaptörde kalıp, uzunluk formülü, metin
//! öneki ve içeriksiz IO izi; gerçek `dil` sürecinde boş dosya, 57/76 kat
//! boyutlar, önek korunumu (ekleme hedefin üzerine yazmaz), 32 MiB kaynak ve
//! hedef tavanı (tam sınır kabul, bir bayt fazlası hedefe dokunmadan ret),
//! eylem reddi, proje sınırı ve `veri:` kökünde `veri-okuma`/`veri-yazma`.
//! Bağlantı çıktısıyla bayt bayt eşitlik `ag_baglantisi_surec_testi`ndedir.

use dil::kaynagi_derle;
use dil::yorumlayici::{self, IzKaydedenIo, IzYenidenOynatici, ToplayanIo};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static SIRA: AtomicU64 = AtomicU64::new(0);
const TAVAN: usize = 32 * 1024 * 1024;

/// Kütüphaneden bağımsız başvuru: bit tamponlu kodlama, 76'da bir CRLF.
fn basvuru(veri: &[u8]) -> Vec<u8> {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let (mut kod, mut tampon, mut bit) = (Vec::new(), 0u32, 0u32);
    for &b in veri {
        tampon = ((tampon << 8) | u32::from(b)) & 0xffff;
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

fn uzunluk(b: usize) -> usize {
    let e = 4 * b.div_ceil(3);
    e + 2 * e.div_ceil(76)
}

fn ikili(boy: usize) -> Vec<u8> {
    (0..boy as u64)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
        .collect()
}

const SONUC_YAZ: &str = r#"işlem kodlaEkle
    kaynağı Metin olarak al
    hedefi Metin olarak al
    değer döndürmez
    sonuç kaynak dosyasını base64 satırlarıyla hedef dosyasına eklemeyi dene olsun
    sonuç başarılıysa
        sonucun değeri yaz
    değilse
        hata sonucun hatası olsun
        hatanın kodu ile " " ile hatanın mesajı yaz

"#;

#[test]
fn hermetik_bos_57_ve_76_kati_metin_onekine_eklenir() {
    let kaynak = format!(
        "{SONUC_YAZ}\"bos.bin\" ve \"govde.eml\" ile kodlaEkle\n\"k114.bin\" ve \"govde.eml\" ile kodlaEkle\n\"k76.bin\" ve \"govde.eml\" ile kodlaEkle\n\"k76.bin\" ve \"yeni.b64\" ile kodlaEkle\n\"yok.bin\" ve \"govde.eml\" ile kodlaEkle\n"
    );
    let program = kaynagi_derle(&kaynak).expect("K-408 kaynağı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.ikili_dosyalar.insert("bos.bin".into(), Vec::new());
    io.ikili_dosyalar.insert("k114.bin".into(), ikili(114));
    io.ikili_dosyalar.insert("k76.bin".into(), ikili(76));
    let onek = "Content-Transfer-Encoding: base64\r\n\r\n";
    io.dosyalar.insert("govde.eml".into(), onek.into());
    yorumlayici::calistir_io(&program, &mut io).expect("K-408 programı çalışmalı");
    assert_eq!(io.cikti[..4], ["0", "156", "108", "108"], "{:?}", io.cikti);
    assert!(io.cikti[4].starts_with("C013"), "{:?}", io.cikti);
    let mut beklenen = onek.as_bytes().to_vec();
    beklenen.extend(basvuru(&ikili(114)));
    beklenen.extend(basvuru(&ikili(76)));
    assert_eq!(io.dosyalar["govde.eml"].as_bytes(), beklenen.as_slice());
    assert_eq!(io.ikili_dosyalar["yeni.b64"], basvuru(&ikili(76)));
}

#[test]
fn hermetik_tavanlar_ve_eylem_reddi_hedefe_dokunmaz() {
    let kaynak = format!(
        "{SONUC_YAZ}eylem eylemde\n    TamSayı sonucu döndürür\n    \"k.bin\" dosyasını base64 satırlarıyla \"e.b64\" dosyasına eklemeyi dene döndür\n\n\"buyuk.bin\" ve \"h.b64\" ile kodlaEkle\n\"k.bin\" ve \"dolu.b64\" ile kodlaEkle\nsonuç eylemde olsun\nsonuç başarısızsa\n    sonucun hatası yaz\n"
    );
    let program = kaynagi_derle(&kaynak).expect("K-408 kaynağı derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.ikili_dosyalar
        .insert("buyuk.bin".into(), vec![0; TAVAN + 1]);
    io.ikili_dosyalar.insert("k.bin".into(), ikili(3));
    io.ikili_dosyalar
        .insert("dolu.b64".into(), vec![b'x'; TAVAN - 5]);
    yorumlayici::calistir_io(&program, &mut io).expect("K-408 programı çalışmalı");
    assert!(
        io.cikti[0].contains("kaynak dosyası 32 MiB"),
        "{:?}",
        io.cikti
    );
    assert!(io.cikti[1].contains("32 MiB sınırını"), "{:?}", io.cikti);
    assert!(io.cikti[2].contains("eylem içinde"), "{:?}", io.cikti);
    assert!(!io.ikili_dosyalar.contains_key("h.b64"));
    assert!(!io.ikili_dosyalar.contains_key("e.b64"));
    assert_eq!(io.ikili_dosyalar["dolu.b64"].len(), TAVAN - 5);
}

#[test]
fn iz_icerik_tasimaz_yalniz_yol_ve_boyut_oynatilir() {
    let kaynak = format!("{SONUC_YAZ}\"gizli.bin\" ve \"g.b64\" ile kodlaEkle\n");
    let program = kaynagi_derle(&kaynak).unwrap();
    let mut io = ToplayanIo::yeni(Vec::new());
    io.ikili_dosyalar
        .insert("gizli.bin".into(), b"GIZLI-ICERIK".to_vec());
    let mut kaydeden = IzKaydedenIo::yeni(io);
    yorumlayici::calistir_io(&program, &mut kaydeden).unwrap();
    let iz = kaydeden.iz_metni().unwrap();
    // İz alanları onaltılık kodludur.
    let hex = |m: &[u8]| m.iter().map(|b| format!("{b:02x}")).collect::<String>();
    assert!(iz.contains("dosyaya_base64_satirlari_ekle"), "{iz}");
    assert!(
        iz.contains(&hex(b"gizli.bin")) && iz.contains(&hex(b"g.b64")),
        "{iz}"
    );
    let kodlu = basvuru(b"GIZLI-ICERIK");
    assert!(
        !iz.contains(&hex(b"GIZLI")) && !iz.contains(&hex(&kodlu[..8])),
        "iz içerik taşımaz: {iz}"
    );
    let mut oynatici = IzYenidenOynatici::yeni(&iz).unwrap();
    yorumlayici::calistir_io(&program, &mut oynatici).unwrap();
    assert_eq!(oynatici.cikti, ["18"], "12 bayt → 16 karakter + CRLF");
    oynatici
        .bitir()
        .expect("yazılan bayt sayısı izden oynatılır");
}

// ---------- gerçek `dil` süreci ----------

struct Proje(PathBuf);

impl Proje {
    fn yeni(yetkinlikler: &str, ek_bildirim: &str, ana: &str) -> Self {
        let yol = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "zee-k408-b64-ekle-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&yol).unwrap();
        std::fs::write(
            yol.join("proje.dil"),
            format!(
                "proje \"b64ekle\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler {yetkinlikler} listesi olsun\nağ_hedefleri boş liste olsun\n{ek_bildirim}"
            ),
        )
        .unwrap();
        std::fs::write(yol.join("ana.dil"), ana).unwrap();
        Self(yol)
    }

    fn calistir(&self, ortam: &[(&str, &std::path::Path)]) -> (bool, String, String) {
        let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
        komut.args(["çalıştır", self.0.to_str().unwrap()]);
        for (ad, deger) in ortam {
            komut.env(ad, deger);
        }
        let cikti = komut.output().unwrap();
        (
            cikti.status.success(),
            String::from_utf8_lossy(&cikti.stdout).into_owned(),
            String::from_utf8_lossy(&cikti.stderr).into_owned(),
        )
    }

    fn oku(&self, ad: &str) -> Vec<u8> {
        std::fs::read(self.0.join(ad)).unwrap()
    }
}

impl Drop for Proje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const DOSYA: &str = "\"dosya-okuma\", \"dosya-yazma\"";

#[test]
fn surecte_bos_kat_boyutlar_ve_onek_bayt_kesin_korunur() {
    let ana = format!(
        "{SONUC_YAZ}\"bos.bin\" ve \"govde.eml\" ile kodlaEkle\n\"k171.bin\" ve \"govde.eml\" ile kodlaEkle\n\"k76.bin\" ve \"govde.eml\" ile kodlaEkle\n\"tek.bin\" ve \"govde.eml\" ile kodlaEkle\n\"bos.bin\" ve \"yeni.b64\" ile kodlaEkle\n"
    );
    let proje = Proje::yeni(DOSYA, "", &ana);
    std::fs::write(proje.0.join("bos.bin"), b"").unwrap();
    std::fs::write(proje.0.join("k171.bin"), ikili(171)).unwrap();
    std::fs::write(proje.0.join("k76.bin"), ikili(76)).unwrap();
    std::fs::write(proje.0.join("tek.bin"), [0xffu8]).unwrap();
    let onek = b"MIME-Version: 1.0\r\nContent-Transfer-Encoding: base64\r\n\r\n";
    std::fs::write(proje.0.join("govde.eml"), onek).unwrap();
    let (tamam, stdout, stderr) = proje.calistir(&[]);
    assert!(tamam, "{stdout}\n{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    // 171 = 3·57 → tam üç 76'lık satır (234); 76 → 104 karakter, iki satır.
    assert_eq!(satirlar, ["0", "234", "108", "6", "0"]);
    assert_eq!(uzunluk(171), 234);
    let mut beklenen = onek.to_vec();
    for parca in [ikili(171), ikili(76), vec![0xff]] {
        beklenen.extend(basvuru(&parca));
    }
    assert_eq!(
        proje.oku("govde.eml"),
        beklenen,
        "önek korunur, sona eklenir"
    );
    assert!(
        proje.oku("yeni.b64").is_empty(),
        "boş kaynak boş hedef oluşturur"
    );
}

#[test]
fn surecte_32_mib_tavanlari_tam_sinirda_ve_asimda_hedef_bozulmaz() {
    let ana = format!(
        "{SONUC_YAZ}\"tam.bin\" ve \"tam.eml\" ile kodlaEkle\n\"tam.bin\" ve \"tasan.eml\" ile kodlaEkle\n\"buyuk.bin\" ve \"b.eml\" ile kodlaEkle\n\"tavan.bin\" ve \"t.eml\" ile kodlaEkle\n"
    );
    let proje = Proje::yeni(DOSYA, "", &ana);
    // 57·430185 bayt → 430185 tam satır = 33 554 430; 2 baytlık önekle tam 32 MiB.
    let ham = 57 * 430_185;
    assert_eq!(uzunluk(ham), TAVAN - 2);
    std::fs::write(proje.0.join("tam.bin"), ikili(ham)).unwrap();
    std::fs::write(proje.0.join("tam.eml"), b"\r\n").unwrap();
    std::fs::write(proje.0.join("tasan.eml"), b"X\r\n").unwrap();
    let buyuk = std::fs::File::create(proje.0.join("buyuk.bin")).unwrap();
    buyuk.set_len(TAVAN as u64 + 1).unwrap();
    // Tam 32 MiB kaynak kaynak tavanını geçer, kodlanmışı hedef tavanını aşar.
    let tavan = std::fs::File::create(proje.0.join("tavan.bin")).unwrap();
    tavan.set_len(TAVAN as u64).unwrap();
    let (tamam, stdout, stderr) = proje.calistir(&[]);
    assert!(tamam, "{stdout}\n{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert_eq!(satirlar[0], (TAVAN - 2).to_string(), "{satirlar:?}");
    assert!(
        satirlar[1].starts_with("C013") && satirlar[1].contains("32 MiB sınırını"),
        "{satirlar:?}"
    );
    assert!(
        satirlar[2].starts_with("C013") && satirlar[2].contains("kaynak dosyası 32 MiB"),
        "{satirlar:?}"
    );
    assert!(satirlar[3].contains("32 MiB sınırını"), "{satirlar:?}");
    let tam = proje.oku("tam.eml");
    assert_eq!(tam.len(), TAVAN);
    assert_eq!(
        &tam[2..80],
        &basvuru(&ikili(57))[..],
        "ilk satır bayt kesin"
    );
    assert_eq!(
        proje.oku("tasan.eml"),
        b"X\r\n",
        "bir bayt fazlası hiç yazmaz"
    );
    assert!(!proje.0.join("b.eml").exists() && !proje.0.join("t.eml").exists());
}

#[test]
fn surecte_eylem_yol_kacisi_ve_dizin_reddedilir() {
    let ana = format!(
        "{SONUC_YAZ}eylem eylemde\n    TamSayı sonucu döndürür\n    \"k.bin\" dosyasını base64 satırlarıyla \"e.b64\" dosyasına eklemeyi dene döndür\n\n\"../dis.bin\" ve \"a.b64\" ile kodlaEkle\n\"k.bin\" ve \"../dis.b64\" ile kodlaEkle\n\"dizin\" ve \"a.b64\" ile kodlaEkle\n\"k.bin\" ve \"dizin\" ile kodlaEkle\n\"yok.bin\" ve \"a.b64\" ile kodlaEkle\nsonuç eylemde olsun\nsonuç başarısızsa\n    sonucun hatası yaz\n"
    );
    let proje = Proje::yeni(DOSYA, "", &ana);
    let dis = proje.0.with_extension("dis");
    std::fs::create_dir_all(&dis).unwrap();
    std::fs::write(dis.join("dis.bin"), b"KOK-DISI").unwrap();
    std::fs::write(proje.0.join("k.bin"), b"abc").unwrap();
    std::fs::create_dir_all(proje.0.join("dizin")).unwrap();
    let (tamam, stdout, stderr) = proje.calistir(&[]);
    let _ = std::fs::remove_dir_all(&dis);
    assert!(tamam, "{stdout}\n{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert!(satirlar[0].contains("proje dosya sınırı"), "{satirlar:?}");
    assert!(satirlar[1].contains("proje dosya sınırı"), "{satirlar:?}");
    assert!(satirlar[2].contains("normal dosya değil"), "{satirlar:?}");
    assert!(satirlar[3].contains("normal dosya değil"), "{satirlar:?}");
    assert!(satirlar[4].starts_with("C013"), "{satirlar:?}");
    assert!(satirlar[5].contains("eylem içinde"), "{satirlar:?}");
    assert!(!proje.0.join("a.b64").exists() && !proje.0.join("e.b64").exists());
    assert!(!proje.0.with_extension("dis").join("dis.b64").exists());
}

#[test]
fn yazma_yetkisiz_projede_derlenmez() {
    let proje = Proje::yeni(
        "\"dosya-okuma\"",
        "",
        "s \"a.bin\" dosyasını base64 satırlarıyla \"b.b64\" dosyasına eklemeyi dene olsun\n",
    );
    std::fs::write(proje.0.join("a.bin"), b"abc").unwrap();
    let (tamam, stdout, stderr) = proje.calistir(&[]);
    assert!(!tamam, "{stdout}");
    assert!(
        stderr.contains("T054") || stdout.contains("T054"),
        "{stderr}"
    );
    assert!(!proje.0.join("b.b64").exists());
}

#[cfg(unix)]
#[test]
fn veri_kokunde_okuma_ve_yazma_yetkisi_ayri_istenir() {
    const DEGISKEN: &str = "ZEE_K408_DEPO";
    let ana = format!(
        "{SONUC_YAZ}\"veri:ekler/ek.bin\" ve \"govde.eml\" ile kodlaEkle\n\"ek.bin\" ve \"veri:giden/govde.eml\" ile kodlaEkle\n"
    );
    let bildirim = format!("veri_kökü \"{DEGISKEN}\" olsun\n");
    let depo = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "zee-k408-depo-{}-{}",
        std::process::id(),
        SIRA.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(depo.join("ekler")).unwrap();
    std::fs::create_dir_all(depo.join("giden")).unwrap();
    std::fs::write(depo.join("ekler/ek.bin"), ikili(100)).unwrap();
    // Yalnız `veri-okuma`: veri kaynağı okunur, veri hedefine yazılamaz.
    let salt = Proje::yeni(
        "\"veri-okuma\", \"dosya-okuma\", \"dosya-yazma\"",
        &bildirim,
        &ana,
    );
    std::fs::write(salt.0.join("ek.bin"), ikili(5)).unwrap();
    let (tamam, stdout, stderr) = salt.calistir(&[(DEGISKEN, &depo)]);
    assert!(tamam, "{stdout}\n{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert_eq!(satirlar[0], uzunluk(100).to_string(), "{satirlar:?}");
    assert!(satirlar[1].contains("veri-yazma"), "{satirlar:?}");
    assert_eq!(salt.oku("govde.eml"), basvuru(&ikili(100)));
    assert!(!depo.join("giden/govde.eml").exists());
    // `veri-okuma` yoksa veri kaynağı reddedilir, hedef oluşmaz.
    let yazan = Proje::yeni(
        "\"veri-yazma\", \"dosya-okuma\", \"dosya-yazma\"",
        &bildirim,
        &ana,
    );
    std::fs::write(yazan.0.join("ek.bin"), ikili(5)).unwrap();
    let (tamam, stdout, stderr) = yazan.calistir(&[(DEGISKEN, &depo)]);
    assert!(tamam, "{stdout}\n{stderr}");
    let satirlar: Vec<&str> = stdout.lines().collect();
    assert!(satirlar[0].contains("veri-okuma"), "{satirlar:?}");
    assert_eq!(satirlar[1], uzunluk(5).to_string(), "{satirlar:?}");
    assert!(!yazan.0.join("govde.eml").exists());
    assert_eq!(
        std::fs::read(depo.join("giden/govde.eml")).unwrap(),
        basvuru(&ikili(5))
    );
    let _ = std::fs::remove_dir_all(&depo);
}
