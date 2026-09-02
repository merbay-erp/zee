#![no_main]

use dil::http_istegi::HttpIstegi;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|baytlar: &[u8]| {
    // Her byte dizisi ya exact çerçeveli bir HTTP/1.x isteğidir ya da Türkçe
    // bir hata olur; parser panic, kayıplı UTF-8 veya örtük framing üretemez.
    let _ = HttpIstegi::ayristir(baytlar);
});
