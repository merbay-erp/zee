#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../tests/ortak/kodlama_kahini.rs"]
mod kodlama_kahini;

fuzz_target!(|baytlar: &[u8]| {
    // K-412 (RFC-0043): katı base64 (tek satır/MIME/satırlı), quoted-printable
    // ve UTF-8 çözümü panik, sınırsız ayırma ya da ikinci yazım kabulü
    // üretemez; gidiş-dönüş ve pencere eşitliği her girdide geçerlidir.
    kodlama_kahini::denetle(baytlar);
});
