//! K-125/B-012/ADR-029: Ondalık ile binary float arasında örtük köprü yoktur.

fn enum_govdesi<'a>(kaynak: &'a str, baslik: &str) -> &'a str {
    kaynak
        .split_once(baslik)
        .and_then(|(_, kalan)| kalan.split_once("\n}").map(|(govde, _)| govde))
        .unwrap_or_else(|| panic!("enum gövdesi bulunamadı: {baslik}"))
}

#[test]
fn ondalik_binary_floata_ortuk_eslenemez() {
    let turler = include_str!("../src/semantic_model.rs");
    let runtime = include_str!("../src/yorumlayici.rs");
    for (ad, govde) in [
        ("Tur", enum_govdesi(turler, "pub enum Tur {")),
        ("Deger", enum_govdesi(runtime, "pub enum Deger {")),
    ] {
        for yasak in [
            "GercekSayi",
            "Binary32",
            "Binary64",
            "F32",
            "F64",
            "f32",
            "f64",
        ] {
            assert!(!govde.contains(yasak), "{ad} örtük {yasak} taşıyamaz");
        }
    }

    let eski_ffi = "\
tehlikeli
    dış işlem c_oran
        girdileri GerçekSayı olsun
        çıktısı double olsun
";
    assert!(
        dil::kaynagi_derle(eski_ffi).is_err(),
        "FFI taslağı ve eski GerçekSayı/double köprüsü henüz dil yüzeyi değildir"
    );
    assert_eq!(
        dil::kaynagi_calistir("sonuç 0,1 ile 0,2 nin toplamı olsun\nsonucu yaz\n")
            .expect("Ondalık exact kalmalı"),
        ["0,3"]
    );

    let rfc = include_str!("../../rfcs/0012-ffi-ve-tehlikeli-sinir.md");
    let adr = include_str!("../../adr/029-ondalik-binary-float-siniri.md");
    assert!(!rfc.contains("GerçekSayı↔double"));
    for soz in ["Ondalık", "binary64", "kayıplı", "Sonuç"] {
        assert!(rfc.contains(soz), "RFC-0012 `{soz}` sınırını taşımalı");
        assert!(adr.contains(soz), "ADR-029 `{soz}` kararını taşımalı");
    }
}
