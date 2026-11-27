#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../tests/ortak/dkim_anahtari_kahini.rs"]
mod dkim_anahtari_kahini;

fuzz_target!(|baytlar: &[u8]| {
    // K-419 (RFC-0044 B2b, ADR-177): DNS'ten gelen güvenilmez DKIM `p=`
    // açık anahtarı (SPKI ya da çıplak PKCS#1 DER, Ed25519 ham 32 bayt,
    // base64 metin). Okuyucu panik yapmaz; kabul/ret, ret türü,
    // `anahtar_biti` ve saklanan baytlar ayrı yazılmış referans DER
    // okuyucusuyla ve ring'in RSA bileşen kabulüyle birebir uyuşur.
    dkim_anahtari_kahini::denetle(baytlar);
});
