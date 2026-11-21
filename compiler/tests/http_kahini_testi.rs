//! K-362 / GB-109: zee HTTP istek ayrıştırıcısının bağımsız `httparse`
//! kâhiniyle farklılık testi. Sözleşme `ortak/http_kahini.rs` başındadır.

#[path = "ortak/http_kahini.rs"]
mod http_kahini;

use http_kahini::{karsilastir, BilincliRet, Karar};
use std::collections::BTreeSet;

fn karar(ad: &str, baytlar: &[u8]) -> Karar {
    match std::panic::catch_unwind(|| karsilastir(baytlar)) {
        Ok(Ok(karar)) => karar,
        Ok(Err(ayrilik)) => panic!("{ad}: {ayrilik}\n{:?}", String::from_utf8_lossy(baytlar)),
        Err(_) => panic!("{ad}: ayrıştırıcı panikledi"),
    }
}

fn kati(ad: &str, baytlar: &[u8], beklenen: BilincliRet) {
    match karar(ad, baytlar) {
        Karar::ZeeDahaKati(siniflar) => {
            assert!(siniflar.contains(&beklenen), "{ad}: {siniflar:?}");
        }
        diger => panic!("{ad}: zee daha katı beklenirdi, {diger:?}"),
    }
}

const KORPUS: &[(&str, &[u8])] = &[
    (
        "valid-get",
        include_bytes!("../fuzz/corpus/http_istegi/valid-get.http"),
    ),
    (
        "valid-post",
        include_bytes!("../fuzz/corpus/http_istegi/valid-post.http"),
    ),
    (
        "bare-lf",
        include_bytes!("../fuzz/corpus/http_istegi/bare-lf.http"),
    ),
    (
        "absolute-form",
        include_bytes!("../fuzz/corpus/http_istegi/absolute-form.http"),
    ),
    (
        "duplicate-content-length",
        include_bytes!("../fuzz/corpus/http_istegi/duplicate-content-length.http"),
    ),
];

#[test]
fn kalici_korpus_kahinle_ayni_ya_da_belgelenmis_katidir() {
    for (ad, baytlar) in KORPUS {
        karar(ad, baytlar);
    }
    assert_eq!(karar("valid-get", KORPUS[0].1), Karar::IkisiKabul);
    assert_eq!(karar("valid-post", KORPUS[1].1), Karar::IkisiKabul);
}

/// Görevde adı geçen her saldırı sınıfı için açık vaka: kâhinle aynı kabul
/// ya da zee'nin belgelenmiş katı reddi.
#[test]
fn smuggling_ve_cerceve_siniflari_kahine_gore_siniflanir() {
    use BilincliRet::*;
    let kabul: &[&[u8]] = &[
        b"GET / HTTP/1.1\r\n\r\n",
        b"GET /a?b=c HTTP/1.0\r\nHost: x\r\n\r\n",
        b"POST /f HTTP/1.1\r\nContent-Length: 3\r\n\r\nabc",
        b"POST /f HTTP/1.1\r\ncontent-length:  0003 \t\r\n\r\nabc",
        b"GET / HTTP/1.1\r\nX:\r\nY: \t\r\n\r\n",
        b"GET / HTTP/1.1\r\nA: 1\r\na: 2\r\n\r\n",
        b"PATCH /x HTTP/1.1\r\nX-A!#$%&'*+-.^_`|~: v\r\n\r\n",
    ];
    for (sira, istek) in kabul.iter().enumerate() {
        assert_eq!(karar(&format!("kabul-{sira}"), istek), Karar::IkisiKabul);
    }
    let iki_ret: &[&[u8]] = &[
        // Geçersiz tokenlar: yöntem, başlık adı, başlık adından önce/sonra boşluk.
        b"G(T / HTTP/1.1\r\n\r\n",
        b"GET / HTTP/1.1\r\nBa slik: x\r\n\r\n",
        b"GET / HTTP/1.1\r\nHost : x\r\n\r\n",
        b"GET / HTTP/1.1\r\n: x\r\n\r\n",
        b"GET / HTTP/1.1\r\nX: a\x7fb\r\n\r\n",
        b"GET / HTTP/1.1\r\nX: a\x01b\r\n\r\n",
        // Request-line biçimi ve sürüm.
        b"GET  / HTTP/1.1\r\n\r\n",
        b"GET / HTTP/2.0\r\n\r\n",
        b"GET / http/1.1\r\n\r\n",
        b"GET /\r\n\r\n",
        b"GET / HTTP/1.1 x\r\n\r\n",
        // obs-fold (RFC 9112 §5.2 isteklerde reddedilir).
        b"GET / HTTP/1.1\r\nX: a\r\n b\r\n\r\n",
        b"GET / HTTP/1.1\r\nX: a\r\n\tb\r\n\r\n",
        // Yalın CR ve NUL.
        b"GET / HTTP/1.1\rX: a\r\n\r\n",
        b"GET / HTTP/1.1\r\nX: a\rb\r\n\r\n",
        b"GET / HTTP/1.1\r\nX: a\0b\r\n\r\n",
    ];
    for (sira, istek) in iki_ret.iter().enumerate() {
        assert_eq!(karar(&format!("ret-{sira}"), istek), Karar::IkisiRet);
    }
    kati("on-crlf", b"\r\nGET / HTTP/1.1\r\n\r\n", OnBosSatir);
    kati("yalin-lf", b"GET / HTTP/1.1\nHost: x\n\n", YalinLf);
    kati("karisik-lf", b"GET / HTTP/1.1\r\nHost: x\n\r\n", YalinLf);
    kati("asterisk", b"OPTIONS * HTTP/1.1\r\n\r\n", OriginFormuDisi);
    kati(
        "absolute",
        b"GET http://a.example/ HTTP/1.1\r\n\r\n",
        OriginFormuDisi,
    );
    kati("parca", b"GET /a#b HTTP/1.1\r\n\r\n", OriginFormuDisi);
    kati(
        "obs-text-hedef",
        "GET /ğ HTTP/1.1\r\n\r\n".as_bytes(),
        OriginFormuDisi,
    );
    kati(
        "obs-text-deger",
        b"GET / HTTP/1.1\r\nX: \xc3\xa7\r\n\r\n",
        AsciiDisiBaslikDegeri,
    );
    kati(
        "te-cl",
        b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\nContent-Length: 4\r\n\r\n0\r\n\r\n",
        TransferEncoding,
    );
    kati(
        "cl-te",
        b"POST / HTTP/1.1\r\nContent-Length: 0\r\ntransfer-encoding: identity\r\n\r\n",
        TransferEncoding,
    );
    kati(
        "esit-cift-cl",
        b"POST / HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\nx",
        CokluContentLength,
    );
    kati(
        "farkli-cift-cl",
        b"POST / HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\nx",
        CokluContentLength,
    );
    for (ad, deger) in [
        ("isaretli", "+1"),
        ("eksi", "-1"),
        ("liste", "1, 1"),
        ("bos", ""),
        ("onaltilik", "0x1"),
        ("tasma", "999999999999999999999999"),
    ] {
        kati(
            &format!("cl-{ad}"),
            format!("POST / HTTP/1.1\r\nContent-Length: {deger}\r\n\r\nx").as_bytes(),
            GecersizContentLength,
        );
    }
    kati(
        "pipelining",
        b"GET / HTTP/1.1\r\n\r\nGET /b HTTP/1.1\r\n\r\n",
        GovdeCercevesi,
    );
    kati(
        "kisa-govde",
        b"POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\nab",
        GovdeCercevesi,
    );
    kati(
        "govde-nul",
        b"POST / HTTP/1.1\r\nContent-Length: 2\r\n\r\na\0",
        GovdeMetni,
    );
    kati(
        "govde-utf8",
        b"POST / HTTP/1.1\r\nContent-Length: 2\r\n\r\n\xc3(",
        GovdeMetni,
    );
}

/// Native okuyucu sınırına (16 KiB) kadar büyük başlıklar ayrıştırıcıda iki
/// tarafta da aynı çözülür; sınırın kendisi `bin|dil|web_profili_testleri`
/// içindeki 431 vakasıyla kanıtlıdır.
#[test]
fn sinira_yakin_buyuk_baslik_bolumu_ayni_cozulur() {
    let sinir = dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
        .http()
        .istek_baslik_bayti();
    let mut tek = b"GET / HTTP/1.1\r\nX: ".to_vec();
    tek.resize(sinir - 4, b'a');
    tek.extend_from_slice(b"\r\n\r\n");
    assert_eq!(tek.len(), sinir);
    assert_eq!(karar("tek-buyuk", &tek), Karar::IkisiKabul);

    let mut cok = b"GET / HTTP/1.1\r\n".to_vec();
    while cok.len() + 6 < sinir {
        cok.extend_from_slice(b"a:\r\n");
    }
    cok.extend_from_slice(b"\r\n");
    assert_eq!(karar("cok-baslik", &cok), Karar::IkisiKabul);
}

struct Rastgele(u64);

impl Rastgele {
    fn sonraki(&mut self) -> u64 {
        // splitmix64: platformdan bağımsız deterministik akış.
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// İlk seçenek geçerli atomdur ve 3/4 olasılıkla seçilir; böylece iki
    /// tarafın da kabul ettiği istekler ret kadar sık üretilir.
    fn sec<T: Copy>(&mut self, secenekler: &[T]) -> T {
        if !self.sonraki().is_multiple_of(4) {
            return secenekler[0];
        }
        secenekler[(self.sonraki() % secenekler.len() as u64) as usize]
    }
}

const YONTEMLER: &[&[u8]] = &[b"GET", b"POST", b"PUT", b"X-Y", b"G\"T", b"", b"get"];
const HEDEFLER: &[&[u8]] = &[
    b"/",
    b"/a?b=c&d",
    b"*",
    b"http://h/",
    b"/a#b",
    b"/\xc3\xa7",
    b"/%00",
    b"",
    b"/a b",
];
const SURUMLER: &[&[u8]] = &[
    b"HTTP/1.1",
    b"HTTP/1.0",
    b"HTTP/1.2",
    b"HTTP/2",
    b"HTTP/1.1 ",
];
const AYIRICILAR: &[&[u8]] = &[b" ", b" ", b" ", b"  ", b"\t"];
const SATIR_SONLARI: &[&[u8]] = &[b"\r\n", b"\r\n", b"\r\n", b"\r\n", b"\n", b"\r", b"\r\r\n"];
const ADLAR: &[&[u8]] = &[
    b"Host",
    b"Content-Length",
    b"content-length",
    b"Transfer-Encoding",
    b"X-A",
    b"Cookie",
    b"Ba slik",
    b"",
    b"X\xc3",
    b" Host",
    b"Host ",
];
const DEGERLER: &[&[u8]] = &[
    b"",
    b"a",
    b" a ",
    b"0",
    b"3",
    b"03",
    b"+3",
    b"3, 3",
    b"chunked",
    b"\xc3\xa7",
    b"a\x7f",
    b"a\0",
    b"\t\t",
    b"99999999999999999999999",
];
const GOVDELER: &[&[u8]] = &[
    b"",
    b"abc",
    b"a",
    b"\xff\xfe\xfd",
    b"a\0c",
    b"GET / HTTP/1.1\r\n\r\n",
];

fn uret(rastgele: &mut Rastgele) -> Vec<u8> {
    let mut istek = Vec::new();
    if rastgele.sonraki().is_multiple_of(23) {
        istek.extend_from_slice(b"\r\n");
    }
    istek.extend_from_slice(rastgele.sec(YONTEMLER));
    istek.extend_from_slice(rastgele.sec(AYIRICILAR));
    istek.extend_from_slice(rastgele.sec(HEDEFLER));
    istek.extend_from_slice(rastgele.sec(AYIRICILAR));
    istek.extend_from_slice(rastgele.sec(SURUMLER));
    istek.extend_from_slice(rastgele.sec(SATIR_SONLARI));
    for _ in 0..rastgele.sonraki() % 5 {
        if rastgele.sonraki().is_multiple_of(17) {
            // obs-fold devam satırı
            istek.extend_from_slice(b" katlanmis");
            istek.extend_from_slice(rastgele.sec(SATIR_SONLARI));
        }
        istek.extend_from_slice(rastgele.sec(ADLAR));
        istek.push(b':');
        istek.extend_from_slice(rastgele.sec(DEGERLER));
        istek.extend_from_slice(rastgele.sec(SATIR_SONLARI));
    }
    istek.extend_from_slice(rastgele.sec(SATIR_SONLARI));
    istek.extend_from_slice(rastgele.sec(GOVDELER));
    istek
}

/// 40 000 deterministik üretilmiş istek: her sonuç kâhinle aynı ya da
/// belgelenmiş katılık; her katılık sınıfı ve iki yönlü kabul görülür.
#[test]
fn uretilmis_isteklerde_belgelenmemis_ayrilik_yoktur() {
    let mut rastgele = Rastgele(0x5a45_455f_4b33_3632);
    let mut gorulen = BTreeSet::new();
    let (mut kabul, mut ret) = (0, 0);
    for sira in 0..40_000 {
        let istek = uret(&mut rastgele);
        match karar(&format!("uretilen-{sira}"), &istek) {
            Karar::IkisiKabul => kabul += 1,
            Karar::IkisiRet => ret += 1,
            Karar::ZeeDahaKati(siniflar) => gorulen.extend(siniflar),
        }
    }
    use BilincliRet::*;
    let beklenen: BTreeSet<_> = [
        OnBosSatir,
        YalinLf,
        OriginFormuDisi,
        AsciiDisiBaslikDegeri,
        TransferEncoding,
        CokluContentLength,
        GecersizContentLength,
        GovdeCercevesi,
        GovdeMetni,
    ]
    .into_iter()
    .collect();
    assert_eq!(gorulen, beklenen, "üretici her katılık sınıfına ulaşmalı");
    assert!(kabul > 1_000 && ret > 1_000, "kabul {kabul}, ret {ret}");
}

/// Geçerli isteklerin her bayt konumunda tek bayt değiştirme, ekleme ve
/// silme: dar sınır değişimleri (CR/LF/SP/HTAB/NUL/`:`/obs-text) kâhinle
/// belgelenmemiş biçimde ayrışmaz.
#[test]
fn tek_bayt_mutasyonlarinda_belgelenmemis_ayrilik_yoktur() {
    const ILGINC: &[u8] = &[
        b'\r', b'\n', b' ', b'\t', 0, b':', b'#', b'/', b'*', b'0', b'9', b'a', b'(', 0x7f, 0x80,
        0xc3, 0xff,
    ];
    let tohumlar: &[&[u8]] = &[
        KORPUS[0].1,
        KORPUS[1].1,
        b"POST /f HTTP/1.1\r\nContent-Length: 2\r\nX: y\r\n\r\nab",
    ];
    let mut denenen = 0;
    for tohum in tohumlar {
        for konum in 0..=tohum.len() {
            for &bayt in ILGINC {
                let mut ekli = tohum.to_vec();
                ekli.insert(konum, bayt);
                karar("ekleme", &ekli);
                if konum < tohum.len() {
                    let mut degisik = tohum.to_vec();
                    degisik[konum] = bayt;
                    karar("degistirme", &degisik);
                    denenen += 1;
                }
                denenen += 1;
            }
            if konum < tohum.len() {
                let mut silik = tohum.to_vec();
                silik.remove(konum);
                karar("silme", &silik);
                denenen += 1;
            }
        }
    }
    assert!(denenen > 3_000, "{denenen}");
}
