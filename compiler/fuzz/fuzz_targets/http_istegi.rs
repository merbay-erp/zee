#![no_main]

use dil::http_istegi::HttpIstegi;
use libfuzzer_sys::fuzz_target;

#[path = "../../tests/ortak/http_kahini.rs"]
mod http_kahini;

fuzz_target!(|baytlar: &[u8]| {
    // Her byte dizisi ya exact çerçeveli bir HTTP/1.x isteğidir ya da Türkçe
    // bir hata olur; parser panic, kayıplı UTF-8 veya örtük framing üretemez.
    let _ = HttpIstegi::ayristir(baytlar);
    // K-362: bağımsız httparse kâhini; zee kâhinin reddettiğini kabul edemez,
    // kâhinin kabul ettiğini yalnız belgelenmiş katı kuralla reddedebilir.
    if let Err(ayrilik) = http_kahini::karsilastir(baytlar) {
        panic!("{ayrilik}");
    }
});
