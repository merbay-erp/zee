#![no_main]

use dil::dns_teli::{self, KayitTuru};
use libfuzzer_sys::fuzz_target;

#[path = "../../tests/ortak/dns_kahini.rs"]
mod dns_kahini;

fuzz_target!(|baytlar: &[u8]| {
    // K-390/ADR-169: her bayt dizisi ya sınırlı bir DNS yanıtıdır ya da
    // hata; ayrıştırıcı panik, sonsuz işaretçi döngüsü veya sınırsız
    // ayırma üretemez. Çözüm adımı da her tür için toplam ve panik yok.
    if let Ok(yanit) = dns_teli::yanit_ayristir(baytlar) {
        for tur in KayitTuru::TUMU {
            let _ = dns_teli::coz(&yanit, &yanit.soru_adi, tur);
        }
    }
    let _ = dns_teli::basligi_eslesiyor(baytlar, 0, "ornek.test", KayitTuru::Mx);
    // Bağımsız hickory-proto kâhini: Zee'nin kabul ettiği her paket kâhinde
    // de geçerli ve cevap bölümü aynı okunmalı (ayrılıkta panik).
    dns_kahini::karsilastir(baytlar);
});
