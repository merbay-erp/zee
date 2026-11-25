//! K-390 / ADR-169: `dns_teli` için bağımsız DNS tel biçimi kâhini.
//!
//! Kâhin `hickory-proto`dur (yalnız dev/fuzz bağımlılığı; derleyici
//! ikilisine girmez). Sözleşme tek yönlüdür: Zee bir yanıtı kabul ederse
//! hickory de kabul etmeli ve cevap bölümünü (ad, tür, sınıf, TTL, rdata)
//! aynı okumalıdır. Zee'nin reddedip hickory'nin kabul ettiği sınıflar
//! bilinçli katı kurallardır (bilinmeyen tür, sondaki fazla bayt, ileri ya
//! da zincirleme taşan işaretçi, NSID/ECS seçeneği, boş TXT).
//!
//! Aynı dosya `tests/dns_kahin_testi.rs` ve `fuzz/fuzz_targets/dns_yaniti.rs`
//! tarafından `#[path]` ile kullanılır.

use dil::dns_teli::{self, KayitVerisi};
use hickory_proto::op::Message;
use hickory_proto::rr::{Name, RData};

fn ad_metni(ad: &Name) -> String {
    ad.iter()
        .map(|etiket| String::from_utf8_lossy(etiket).to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(".")
}

/// Zee kabul ettiyse (`Ok(true)`) hickory de kabul etmeli ve cevap bölümü aynı
/// olmalı; ayrılıkta panik eder. Zee reddettiyse `false`.
pub fn karsilastir(paket: &[u8]) -> bool {
    let Ok(zee) = dns_teli::yanit_ayristir(paket) else {
        return false;
    };
    let kahin = Message::from_vec(paket).unwrap_or_else(|hata| {
        panic!("Zee'nin kabul ettiği paketi hickory reddetti ({hata}): {paket:02x?}")
    });
    assert_eq!(zee.kimlik, kahin.metadata.id, "kimlik: {paket:02x?}");
    assert_eq!(
        zee.cevaplar.len(),
        kahin.answers.len(),
        "cevap sayısı: {paket:02x?}"
    );
    for (z, h) in zee.cevaplar.iter().zip(&kahin.answers) {
        assert_eq!(z.ad, ad_metni(&h.name), "ad: {paket:02x?}");
        assert_eq!(z.tur, u16::from(h.record_type()), "tür: {paket:02x?}");
        assert_eq!(z.sinif, u16::from(h.dns_class), "sınıf: {paket:02x?}");
        assert_eq!(z.ttl, h.ttl, "ttl: {paket:02x?}");
        match (&z.veri, &h.data) {
            (KayitVerisi::A(a), RData::A(b)) => assert_eq!(*a, b.0.octets()),
            (KayitVerisi::Aaaa(a), RData::AAAA(b)) => assert_eq!(*a, b.0.octets()),
            (KayitVerisi::Mx { oncelik, hedef }, RData::MX(mx)) => {
                assert_eq!(*oncelik, mx.preference);
                assert_eq!(*hedef, ad_metni(&mx.exchange));
            }
            (KayitVerisi::Txt(a), RData::TXT(txt)) => {
                assert_eq!(*a, txt.txt_data.concat());
            }
            (
                KayitVerisi::Tlsa {
                    kullanim,
                    secici,
                    esleme,
                    veri,
                },
                RData::TLSA(t),
            ) => {
                assert_eq!(*kullanim, u8::from(t.cert_usage));
                assert_eq!(*secici, u8::from(t.selector));
                assert_eq!(*esleme, u8::from(t.matching));
                assert_eq!(*veri, t.cert_data);
            }
            (KayitVerisi::Cname(a), RData::CNAME(b)) => assert_eq!(*a, ad_metni(&b.0)),
            (KayitVerisi::Soa { birincil }, RData::SOA(soa)) => {
                assert_eq!(*birincil, ad_metni(&soa.mname));
            }
            (KayitVerisi::Diger, _) => {}
            (z, h) => panic!("kayıt verisi farklı okundu: {z:?} / {h:?} — {paket:02x?}"),
        }
    }
    true
}
