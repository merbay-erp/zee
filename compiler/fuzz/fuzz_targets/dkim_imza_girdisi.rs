#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../tests/ortak/dkim_ozellikleri.rs"]
mod dkim_ozellikleri;

fuzz_target!(|baytlar: &[u8]| {
    // K-411 (ADR-174): saldırganın gönderdiği ileti gövdesi ve Zee'nin kurduğu
    // DKIM imza girdisi. İlk bayt parça sınırını seçer; kanonik gövde
    // özetleyicisi ve imza kâhini denetimi panik yapmaz, sınırsız ayırmaz ve
    // ortak özellikleri korur.
    dkim_ozellikleri::denetle(baytlar);
});
