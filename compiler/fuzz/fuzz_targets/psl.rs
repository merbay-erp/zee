#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../tests/ortak/psl_kahini.rs"]
mod psl_kahini;

fuzz_target!(|baytlar: &[u8]| {
    // K-418 (RFC-0044 B1b, ADR-177): e-posta başlığından gelen güvenilmez
    // alan adı ve isteğe bağlı küçük kamu son ek listesi. Ayrıştırıcı ve
    // kurumsal alan hesabı panik yapmaz; sonuç naif referans uygulamayla ve
    // spec/31 §2 değişmezleriyle (sonek, boş/geçersiz, eşgüçlülük) uyuşur.
    psl_kahini::denetle(baytlar);
});
