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
