//! K-412 (RFC-0043): `kodlama` fuzz hedefinin kalıcı korpusunu ve
//! deterministik üretilmiş girdileri kararlı Rust'ta sözleşme kâhiniyle
//! oynatır. Fuzz kampanyası olmadan da aynı değişmezler her `cargo test`te
//! denetlenir.

#[path = "ortak/kodlama_kahini.rs"]
mod kodlama_kahini;

#[test]
fn kalici_fuzz_korpusu_kahinle_uyusur() {
    let dizin = concat!(env!("CARGO_MANIFEST_DIR"), "/fuzz/corpus/kodlama");
    let mut sayi = 0;
    for giris in std::fs::read_dir(dizin).expect("korpus") {
        kodlama_kahini::denetle(&std::fs::read(giris.unwrap().path()).unwrap());
        sayi += 1;
    }
    assert!(sayi >= 8, "korpus tohumu eksik: {sayi}");
}

#[test]
fn uretilmis_girdiler_kahinle_uyusur() {
    // Her bayt değeri, base64/QP alfabesi ağırlıklı karışım ve 0–600 boy.
    const ALFABE: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/= \r\n\t=";
    let mut durum = 0x9e37_79b9u32;
    let mut sonraki = || {
        durum ^= durum << 13;
        durum ^= durum >> 17;
        durum ^= durum << 5;
        durum
    };
    for boy in 0..600usize {
        let ham: Vec<u8> = (0..boy).map(|_| (sonraki() >> 24) as u8).collect();
        kodlama_kahini::denetle(&ham);
        let alfabeli: Vec<u8> = (0..boy)
            .map(|_| ALFABE[sonraki() as usize % ALFABE.len()])
            .collect();
        kodlama_kahini::denetle(&alfabeli);
    }
}
