use dil::http_istegi::{baslik_sonunu_bul, HttpIstegi, HttpIstekBasligi};

fn reddedilir(istek: &[u8]) {
    assert!(HttpIstegi::ayristir(istek).is_err(), "{istek:?}");
}

#[test]
fn origin_formu_crlf_ve_exact_govde_uzunluguyla_cozulur() {
    let ham =
        b"POST /kaydet?q=1 HTTP/1.1\r\nHost: panel.example\r\nContent-Length: 7\r\n\r\nnot=iyi";
    let istek = HttpIstegi::ayristir(ham).expect("geçerli HTTP isteği");
    assert_eq!(istek.baslik.yontem(), "POST");
    assert_eq!(istek.baslik.hedef(), "/kaydet?q=1");
    assert_eq!(istek.baslik.surum(), "HTTP/1.1");
    assert_eq!(istek.baslik.govde_uzunlugu(), 7);
    assert_eq!(istek.baslik.baslik_degerleri("host"), ["panel.example"]);
    assert_eq!(istek.govde, "not=iyi");
}

#[test]
fn bare_lf_bare_cr_obs_fold_ve_nul_fail_closed_reddedilir() {
    for ham in [
        b"GET / HTTP/1.1\nHost: panel.example\n\n".as_slice(),
        b"GET / HTTP/1.1\rHost: panel.example\r\r".as_slice(),
        b"GET / HTTP/1.1\r\nHost: panel.example\r\n devam\r\n\r\n".as_slice(),
        b"GET / HTTP/1.1\r\nHost: panel\0.example\r\n\r\n".as_slice(),
    ] {
        reddedilir(ham);
    }
    assert!(baslik_sonunu_bul(b"GET / HTTP/1.1\r").unwrap().is_none());
}

#[test]
fn absolute_authority_asterisk_ve_bozuk_request_line_reddedilir() {
    for ham in [
        b"GET https://panel.example/yol HTTP/1.1\r\n\r\n".as_slice(),
        b"CONNECT panel.example:443 HTTP/1.1\r\n\r\n".as_slice(),
        b"OPTIONS * HTTP/1.1\r\n\r\n".as_slice(),
        b"GET  / HTTP/1.1\r\n\r\n".as_slice(),
        b"GET\t/ HTTP/1.1\r\n\r\n".as_slice(),
        b"GET / HTTP/2\r\n\r\n".as_slice(),
        b"GET /#parca HTTP/1.1\r\n\r\n".as_slice(),
    ] {
        reddedilir(ham);
    }
}

#[test]
fn transfer_encoding_duplicate_ve_kuralsiz_content_length_reddedilir() {
    for ham in [
        b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n".as_slice(),
        b"POST / HTTP/1.1\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n".as_slice(),
        b"POST / HTTP/1.1\r\nContent-Length: +7\r\n\r\nnot=iyi".as_slice(),
        b"POST / HTTP/1.1\r\nContent-Length: 7, 7\r\n\r\nnot=iyi".as_slice(),
        b"POST / HTTP/1.1\r\nContent-Length : 7\r\n\r\nnot=iyi".as_slice(),
    ] {
        reddedilir(ham);
    }
}

#[test]
fn gecersiz_utf8_nul_eksik_ve_fazla_govde_reddedilir() {
    let mut bozuk_baslik = b"GET / HTTP/1.1\r\nX-Bozuk: ".to_vec();
    bozuk_baslik.push(0xff);
    bozuk_baslik.extend_from_slice(b"\r\n\r\n");
    reddedilir(&bozuk_baslik);

    for ham in [
        b"POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\n\xff".as_slice(),
        b"POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\n\0".as_slice(),
        b"POST / HTTP/1.1\r\nContent-Length: 2\r\n\r\na".as_slice(),
        b"POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\nab".as_slice(),
        b"POST / HTTP/1.1\r\n\r\nx".as_slice(),
    ] {
        reddedilir(ham);
    }
}

#[test]
fn baslik_parseri_ilk_govde_baytlarini_basliktan_ayirir() {
    let ham = b"POST / HTTP/1.0\r\nContent-Length: 4\r\n\r\ntest";
    let son = baslik_sonunu_bul(ham).unwrap().expect("başlık sonu");
    let baslik = HttpIstekBasligi::ayristir(&ham[..son]).unwrap();
    assert_eq!(baslik.govde_uzunlugu(), 4);
    assert_eq!(&ham[son..], b"test");
}

#[test]
fn kalici_http_fuzz_korpusu_her_ana_testte_yeniden_oynatilir() {
    let korpus: &[(&str, &[u8], bool)] = &[
        (
            "valid-get",
            include_bytes!("../fuzz/corpus/http_istegi/valid-get.http"),
            true,
        ),
        (
            "valid-post",
            include_bytes!("../fuzz/corpus/http_istegi/valid-post.http"),
            true,
        ),
        (
            "bare-lf",
            include_bytes!("../fuzz/corpus/http_istegi/bare-lf.http"),
            false,
        ),
        (
            "absolute-form",
            include_bytes!("../fuzz/corpus/http_istegi/absolute-form.http"),
            false,
        ),
        (
            "duplicate-content-length",
            include_bytes!("../fuzz/corpus/http_istegi/duplicate-content-length.http"),
            false,
        ),
    ];
    for (ad, ham, gecerli) in korpus {
        let sonuc = std::panic::catch_unwind(|| HttpIstegi::ayristir(ham));
        assert!(sonuc.is_ok(), "{ad} parser'ı panikletti");
        assert_eq!(sonuc.unwrap().is_ok(), *gecerli, "{ad}");
    }
}
