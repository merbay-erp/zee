//! K-419 (RFC-0044 B2b, ADR-177): `dkim_anahtari` fuzz hedefinin kalıcı
//! korpusunu ve deterministik bozulmuş anahtarları kararlı Rust'ta
//! diferansiyel kâhinle oynatır. Fuzz kampanyası olmadan da aynı
//! değişmezler her `cargo test`te denetlenir.

#[path = "ortak/dkim_anahtari_kahini.rs"]
mod dkim_anahtari_kahini;

fn korpus() -> Vec<Vec<u8>> {
    let dizin = concat!(env!("CARGO_MANIFEST_DIR"), "/fuzz/corpus/dkim_anahtari");
    let mut dosyalar: Vec<_> = std::fs::read_dir(dizin)
        .expect("korpus")
        .map(|giris| giris.unwrap().path())
        .collect();
    dosyalar.sort();
    dosyalar
        .iter()
        .map(|yol| std::fs::read(yol).unwrap())
        .collect()
}

#[test]
fn kalici_fuzz_korpusu_kahinle_uyusur() {
    let tohumlar = korpus();
    assert!(
        tohumlar.len() >= 20,
        "korpus tohumu eksik: {}",
        tohumlar.len()
    );
    for tohum in &tohumlar {
        dkim_anahtari_kahini::denetle(tohum);
    }
}

#[test]
fn bozulmus_anahtarlar_kahinle_uyusur() {
    // Her tohumun bayt değiştirme, kesme ve ekleme ile bozulmuş kopyaları;
    // INTEGER/uzunluk baytlarına isabet için küçük değerler ağırlıklı. Ham
    // DER kopyası ayrıca `p=` base64 kipinde oynatılır.
    const BAYTLAR: [u8; 11] = [
        0x00, 0x01, 0x02, 0x03, 0x05, 0x30, 0x7f, 0x80, 0x81, 0x82, 0xff,
    ];
    let tohumlar = korpus();
    let mut durum = 0x9e37_79b9_7f4a_7c15u64;
    let mut sonraki = move || {
        durum ^= durum << 13;
        durum ^= durum >> 7;
        durum ^= durum << 17;
        durum
    };
    for tur in 0..6_000usize {
        let tohum = &tohumlar[tur % tohumlar.len()];
        let mut girdi = tohum[1..].to_vec();
        for _ in 0..(sonraki() % 3 + 1) {
            if girdi.is_empty() {
                break;
            }
            let konum = (sonraki() as usize) % girdi.len();
            let bayt = BAYTLAR[(sonraki() % BAYTLAR.len() as u64) as usize];
            match sonraki() % 3 {
                0 => girdi[konum] = bayt,
                1 => girdi.truncate(konum),
                _ => girdi.insert(konum, bayt),
            }
        }
        // Ara sıra algoritma çevrilir (RSA anahtarı Ed25519 olarak ve tersi).
        let kip = tohum[0] ^ u8::from(sonraki().is_multiple_of(8));
        let mut ham = vec![kip];
        ham.extend_from_slice(&girdi);
        dkim_anahtari_kahini::denetle(&ham);
        if kip & 2 == 0 {
            let mut metin = vec![kip | 2];
            metin.extend_from_slice(dil::kodlama::base64_kodla(&girdi).as_bytes());
            dkim_anahtari_kahini::denetle(&metin);
        }
    }
}
