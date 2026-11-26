#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../tests/ortak/x509_ozellikleri.rs"]
mod x509_ozellikleri;

fuzz_target!(|baytlar: &[u8]| {
    // K-411 (ADR-173): TLS istemcisinin karşı taraftan aldığı uç sertifika
    // DER'i. SPKI yürüyücüsü panik yapmaz, sınır dışına çıkmaz ve ayrı
    // yazılmış referans DER okuyucusuyla aynı sonucu verir.
    x509_ozellikleri::denetle(baytlar);
});
