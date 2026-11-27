//! K-418 (RFC-0044 B1b, ADR-177): `psl` fuzz hedefinin kalıcı korpusunu ve
//! deterministik üretilmiş girdileri kararlı Rust'ta diferansiyel kâhinle
//! oynatır. Fuzz kampanyası olmadan da aynı değişmezler her `cargo test`te
//! denetlenir.

#[path = "ortak/psl_kahini.rs"]
mod psl_kahini;

#[test]
fn kalici_fuzz_korpusu_kahinle_uyusur() {
    let dizin = concat!(env!("CARGO_MANIFEST_DIR"), "/fuzz/corpus/psl");
    let mut sayi = 0;
    for giris in std::fs::read_dir(dizin).expect("korpus") {
        psl_kahini::denetle(&std::fs::read(giris.unwrap().path()).unwrap());
        sayi += 1;
    }
    assert!(sayi >= 10, "korpus tohumu eksik: {sayi}");
}

#[test]
fn uretilmis_girdiler_kahinle_uyusur() {
    // Etiket parçaları örnek listenin kurallarına, joker/istisna ve geçersiz
    // karakterlere ağırlıklı; her iki kip (örnek liste / girdideki liste).
    // `|` ayırıcıdır; `||` boş etiket üretir.
    const PARCALAR: &str =
        "a|com|ck|www|mm|jp|kobe|city|uk|tr|x_y|-||A|ö|*|!|xn--55qx5d|cn|blogspot";
    let parcalar: Vec<&str> = PARCALAR.split('|').collect();
    let mut durum = 0x2545_f491_4f6c_dd1du64;
    let mut sonraki = move || {
        durum ^= durum << 13;
        durum ^= durum >> 7;
        durum ^= durum << 17;
        durum
    };
    let ad = |sonraki: &mut dyn FnMut() -> u64| {
        let n = (sonraki() % 6) as usize;
        let mut ad: Vec<&str> = (0..n)
            .map(|_| parcalar[(sonraki() % parcalar.len() as u64) as usize])
            .collect();
        if sonraki().is_multiple_of(4) {
            ad.push("");
        }
        ad.join(".")
    };
    for tur in 0..3_000u32 {
        let mut girdi = Vec::new();
        if tur.is_multiple_of(2) {
            girdi.push(b'0');
        } else {
            girdi.push(b'1');
            for _ in 0..(sonraki() % 8) {
                let on = ["", "*.", "!", "// "][(sonraki() % 4) as usize];
                girdi.extend_from_slice(on.as_bytes());
                girdi.extend_from_slice(ad(&mut sonraki).as_bytes());
                girdi.push(b'\n');
            }
            girdi.push(0);
        }
        for _ in 0..8 {
            girdi.extend_from_slice(ad(&mut sonraki).as_bytes());
            girdi.push(b'\n');
        }
        psl_kahini::denetle(&girdi);
    }
}
