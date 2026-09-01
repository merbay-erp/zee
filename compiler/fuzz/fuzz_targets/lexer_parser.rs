#![no_main]

use dil::faz::KaynakMetni;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|kaynak: &str| {
    let Ok(tokenlar) = KaynakMetni::yeni(kaynak).sozcukle() else {
        return;
    };

    // Her iki public parser yolu da aynı lexer üretimini güvenle tüketmeli.
    let _ = tokenlar.clone().ayristir(Vec::new());
    let _ = tokenlar.ayristir_kurtarmali(Vec::new());
});
