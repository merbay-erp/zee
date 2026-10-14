//! K-248: private URI byte/UTF-8 çözümünün olumlu ve olumsuz sınırları.
use super::uri_yolu;

#[test]
fn uri_yuzde_kacislari_utf8_baytlari_olarak_cozulur() {
    for uri in [
        "file:///Türkçe ö 😀+%25/ana.dil",
        "file:///T%C3%BCrk%C3%A7e%20%C3%B6%20%F0%9F%98%80+%25/ana.dil",
        "file:///T%c3%bcrk%c3%a7e%20%c3%b6%20%f0%9f%98%80+%25/ana.dil",
    ] {
        assert_eq!(
            uri_yolu(uri),
            Some(std::path::PathBuf::from("/Türkçe ö 😀+%/ana.dil")),
            "{uri}"
        );
    }
}

#[test]
fn uri_eksik_hex_ve_gecersiz_utf8_kacisini_reddeder() {
    for uri in [
        "file:///a%",
        "file:///a%2",
        "file:///a%GG",
        "file:///a%C3",
        "file:///a%80",
        "file:///a%C0%AF",
        "file:///a%ED%A0%80",
        "https:///a.dil",
    ] {
        assert!(uri_yolu(uri).is_none(), "{uri}");
    }
}
