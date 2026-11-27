use dil::faz::KaynakMetni;

const KORPUS: &[(&str, &str)] = &[
    ("boş", include_str!("../fuzz/corpus/lexer_parser/bos.dil")),
    (
        "temel",
        include_str!("../fuzz/corpus/lexer_parser/temel.dil"),
    ),
    (
        "unicode",
        include_str!("../fuzz/corpus/lexer_parser/unicode.dil"),
    ),
    (
        "girinti",
        include_str!("../fuzz/corpus/lexer_parser/girinti.dil"),
    ),
    (
        "sayılar",
        include_str!("../fuzz/corpus/lexer_parser/sayilar.dil"),
    ),
    (
        "virgüller",
        include_str!("../fuzz/corpus/lexer_parser/virguller.dil"),
    ),
    (
        "metinler",
        include_str!("../fuzz/corpus/lexer_parser/metinler.dil"),
    ),
    (
        "bloklar",
        include_str!("../fuzz/corpus/lexer_parser/bloklar.dil"),
    ),
];

fn hatti_panik_bekcisiyle_calistir(ad: &str, kaynak: &str) {
    let sonuc = std::panic::catch_unwind(|| {
        let Ok(tokenlar) = KaynakMetni::yeni(kaynak).sozcukle() else {
            return;
        };
        let _ = tokenlar.clone().ayristir(Vec::new());
        let _ = tokenlar.ayristir_kurtarmali(Vec::new());
    });
    assert!(sonuc.is_ok(), "{ad} lexer/parser hattını panikletti");
}

#[test]
fn saldiri_korpusu_lexer_ve_iki_parser_yolunda_panik_uretmez() {
    for (ad, kaynak) in KORPUS {
        hatti_panik_bekcisiyle_calistir(ad, kaynak);
    }
}

#[test]
fn deterministik_utf8_uretimi_panik_uretmez() {
    const ATOMLAR: &[&str] = &[
        "a", "ç", "İ", "â", "9", "-", ",", " ", "    ", "\t", "\n", "\r\n", "\"", "\\", "#", "🧿",
        "中", "а", "\u{0306}", "\0", "ise", "olsun", "yaz",
    ];

    for tohum in 0..4_096u64 {
        let mut durum = tohum.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut kaynak = String::new();
        for _ in 0..(durum as usize % 96) {
            durum = durum
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            kaynak.push_str(ATOMLAR[durum as usize % ATOMLAR.len()]);
        }
        hatti_panik_bekcisiyle_calistir(&format!("üretilmiş-{tohum}"), &kaynak);
    }
}

#[test]
fn dev_sayi_virgul_ve_girinti_girdileri_panik_uretmez() {
    let rakamlar = "9".repeat(65_536);
    let derin = (1..=256)
        .map(|seviye| format!("{}doğru ise\n", " ".repeat(seviye)))
        .collect::<String>();
    for (ad, kaynak) in [
        ("dev-tam-sayı", rakamlar.clone()),
        ("dev-ondalık", format!("0,{rakamlar}")),
        ("dev-virgül", ",".repeat(65_536)),
        ("artan-girinti", derin),
    ] {
        hatti_panik_bekcisiyle_calistir(ad, &kaynak);
    }
}

// K-182: bash betiği (fuzz artefakt doğrulayıcısı) POSIX kabuk ister; CI onu yalnız
// ubuntu'da koşar, Windows Git Bash desteklenen koruk hostu değildir.
#[cfg(unix)]
#[test]
fn nightly_ogrenimi_cache_disinda_provenanceli_artefaktta_kalir() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler depo içinde olmalı");
    let workflow = std::fs::read_to_string(depo.join(".github/workflows/fuzz.yml"))
        .expect("fuzz workflow okunmalı");
    for parca in [
        "Öğrenilen korpusu her koşuda sakla",
        "if: always()",
        "zee-fuzz-corpus-artifact-1",
        "source_commit",
        "run_attempt",
        "retention-days: 90",
        "if-no-files-found: error",
        "compiler/fuzz/corpus/$FUZZ_KORPUS/.",
    ] {
        assert!(
            workflow.contains(parca),
            "fuzz kalıcılık kapısı eksik: {parca}"
        );
    }
    let upload_sayisi = workflow.matches("actions/upload-artifact@").count();
    assert_eq!(upload_sayisi, 2, "korpus ve crash ayrı artefakt olmalı");

    let dogrulayici =
        std::fs::read_to_string(depo.join("scripts/fuzz-korpus-artefakti-dogrula.sh"))
            .expect("fuzz artefakt doğrulayıcısı okunmalı");
    assert!(dogrulayici.contains("# zee-fuzz-corpus-artifact-1"));
    assert!(dogrulayici.contains("sha256sum"));
    assert!(dogrulayici.contains("shasum -a 256"));
    assert!(dogrulayici.contains("manifest_sayisi"));

    let damga = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("saat")
        .as_nanos();
    let gecici =
        std::env::temp_dir().join(format!("zee-fuzz-artefakt-{}-{damga}", std::process::id()));
    std::fs::create_dir_all(gecici.join("corpus")).expect("geçici korpus");
    std::fs::write(gecici.join("corpus/tohum"), b"abc").expect("seed");
    std::fs::write(
        gecici.join("manifest.tsv"),
        "# zee-fuzz-corpus-artifact-1\n\
         # target\tlexer_parser\n\
         # corpus\tlexer_parser\n\
         # source_commit\t1111111111111111111111111111111111111111\n\
         # run_id\t42\n\
         # run_attempt\t1\n\
         # toolchain\tnightly-2026-08-31\n\
         # cargo_fuzz\t0.13.2\n\
         ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\ttohum\n",
    )
    .expect("manifest");
    let betik = depo.join("scripts/fuzz-korpus-artefakti-dogrula.sh");
    let gecerli = std::process::Command::new("bash")
        .arg(&betik)
        .arg("lexer_parser")
        .arg(&gecici)
        .output()
        .expect("doğrulayıcı çalışmalı");
    assert!(gecerli.status.success(), "doğru artifact kabul edilmeli");

    std::fs::write(gecici.join("corpus/tohum"), b"bozuk").expect("seed bozma");
    let bozuk = std::process::Command::new("bash")
        .arg(&betik)
        .arg("lexer_parser")
        .arg(&gecici)
        .output()
        .expect("doğrulayıcı çalışmalı");
    assert!(
        !bozuk.status.success(),
        "değiştirilmiş artifact reddedilmeli"
    );
    std::fs::remove_dir_all(gecici).expect("geçici artifact temizlenmeli");
}

#[test]
fn rc_kampanyasi_dort_hedefi_address_sanitizer_ve_miriyle_korur() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler depo içinde olmalı");
    let workflow = std::fs::read_to_string(depo.join(".github/workflows/fuzz-rc.yml"))
        .expect("RC fuzz workflow okunmalı");
    for hedef in ["lexer_parser", "morfoloji", "http_istegi", "wasm_abi"] {
        assert!(
            workflow.contains(&format!("hedef: {hedef}")),
            "eksik RC hedefi: {hedef}"
        );
    }
    for parca in [
        "timeout-minutes: 75",
        "-max_total_time=3600",
        "--sanitizer address",
        "-print_final_stats=1",
        "# zee-fuzz-rc-sonucu-1",
        "if: always()",
        "retention-days: 90",
        "component add --toolchain nightly-2026-08-31 miri rust-src",
        "miri test --lib ondalik::testler::",
        "miri test --lib zaman::testler::",
    ] {
        assert!(workflow.contains(parca), "RC fuzz kapısı eksik: {parca}");
    }
}

/// Workflow matrisinden `(hedef, azami_girdi)` çiftlerini sırasıyla çıkarır.
fn matris_hedefleri(workflow: &str) -> Vec<(String, String)> {
    let mut hedefler = Vec::new();
    let mut hedef = None;
    for satir in workflow.lines().map(str::trim) {
        if let Some(ad) = satir.strip_prefix("- hedef: ") {
            hedef = Some(ad.to_string());
        } else if let Some(azami) = satir.strip_prefix("azami_girdi: ") {
            let ad = hedef.take().expect("azami_girdi bir hedefi izlemeli");
            hedefler.push((ad, azami.to_string()));
        }
    }
    hedefler
}

#[test]
fn rc_kapisi_on_hedefi_gecelik_matris_ve_rc_betikleriyle_hizalar() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler depo içinde olmalı");
    let oku = |yol: &str| std::fs::read_to_string(depo.join(yol)).expect("dosya okunmalı");
    let gecelik = matris_hedefleri(&oku(".github/workflows/fuzz.yml"));
    let rc = matris_hedefleri(&oku(".github/workflows/fuzz-rc.yml"));
    assert_eq!(rc.len(), 10, "RC fuzz kapısı on hedef ister: {rc:?}");
    assert_eq!(
        rc, gecelik,
        "RC matrisi gecelik hedef/azami_girdi matrisiyle aynı olmalı"
    );
    for (hedef, azami) in [("psl", "16384"), ("dkim_anahtari", "4097")] {
        assert!(
            rc.contains(&(hedef.to_string(), azami.to_string())),
            "K-418/K-419 hedefi RC kapısında değil: {hedef}"
        );
    }
    let kampanya = oku("scripts/rc-kampanyasi.py");
    let kanit = oku("scripts/rc-kaniti.py");
    let kapi = oku("scripts/guvenlik-kapisi.sh");
    let liste = rc
        .iter()
        .map(|(h, _)| h.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        kapi.contains(&format!("for hedef in {liste}; do")),
        "güvenlik kapısı RC listesi: {liste}"
    );
    let mut sira = 0;
    for (hedef, azami) in &rc {
        let kayit = format!("('{hedef}', {azami})");
        let konum = kampanya
            .find(&kayit)
            .unwrap_or_else(|| panic!("rc-kampanyasi.py: {kayit}"));
        assert!(
            konum > sira,
            "rc-kampanyasi.py hedef sırası matrisle aynı olmalı: {hedef}"
        );
        sira = konum;
        assert!(
            kanit.contains(&format!("'{hedef}':{azami}")),
            "rc-kaniti.py: {hedef}:{azami}"
        );
    }
    for parca in ["zee-local-asan-miri-campaign-2", "zee-rc-fuzz-hedefleri-10"] {
        assert!(
            kampanya.contains(parca) && kanit.contains(parca),
            "RC hedef ilkesi sürümü: {parca}"
        );
    }
    assert!(
        kanit.contains("LEGACY_CAMPAIGN_SCHEMA = 'zee-local-asan-miri-campaign-1'"),
        "eski sekiz hedefli paketler yeni kuralla yeniden yargılanmaz"
    );
}
