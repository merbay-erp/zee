//! K-412 / RFC-0043: `kodlama` için sözleşme kâhini (yeni bağımlılık yok).
//!
//! Kâhin dış bir kitaplık değil, RFC 4648/2045 sözleşmesinin kendisinden
//! çıkan değişmezlerdir:
//!
//! - her bayt dizisi kodlanıp tek satır, MIME (76 + CRLF) ve satırlı
//!   biçimde katı çözülünce aynısına döner; çıktı boyu önceden hesaplanan
//!   boya eşittir;
//! - katı tek satır çözücünün kabul ettiği her metin, kodlayıcının o bayt
//!   için ürettiği tek kanonik yazımdır (ikinci yazım yok); MIME çözücüsü
//!   için aynı şey yalnız CR/LF/boşluk/sekme atıldıktan sonra geçerlidir;
//! - çözülen çıktı istenen tavanı aşmaz; quoted-printable çıktısı girdiden
//!   uzun olamaz; hata konumu girdi içindedir;
//! - `utf8_coz` standart kitaplıkla aynı kabul/ret ve hata konumunu verir;
//! - satırlı base64 penceresi, bütün çıktının aynı dilimine eşittir.
//!
//! Aynı dosya `tests/kodlama_fuzz_testi.rs` ve
//! `fuzz/fuzz_targets/kodlama.rs` tarafından `#[path]` ile kullanılır.

use dil::kodlama::{self, KodlamaHatasi, AZAMI_PARCA_BAYTI};

/// Fuzz girdisinin baytları ham veri ve metin olarak birlikte sınanır.
pub fn denetle(baytlar: &[u8]) {
    gidis_donus(baytlar);
    let tavan = baytlar.first().map_or(0, |&b| usize::from(b));
    let metin = String::from_utf8_lossy(baytlar);
    kati_tek_satir(&metin, tavan);
    kati_tek_satir(&metin, AZAMI_PARCA_BAYTI);
    mime(&metin, tavan);
    qp(&metin, tavan);
    utf8(baytlar);
    pencere(baytlar);
}

fn konum_girdide(hata: &KodlamaHatasi, uzunluk: usize) {
    if let Some(konum) = hata.konum {
        assert!(
            konum <= uzunluk as u64,
            "konum {konum} > {uzunluk}: {hata:?}"
        );
    }
}

fn gidis_donus(ham: &[u8]) {
    let kod = kodlama::base64_kodla(ham);
    assert_eq!(kod.len(), kodlama::base64_boyu(ham.len()));
    assert_eq!(kodlama::base64_coz(&kod, ham.len()).unwrap(), ham);
    if !ham.is_empty() {
        let hata = kodlama::base64_coz(&kod, ham.len() - 1).unwrap_err();
        assert_eq!(hata.tur, "boyut_aşımı");
    }
    let mime: String = kod
        .as_bytes()
        .chunks(76)
        .map(|s| format!("{}\r\n", std::str::from_utf8(s).unwrap()))
        .collect();
    assert_eq!(kodlama::base64_mime_coz(&mime, ham.len()).unwrap(), ham);
    let mut satirli = Vec::new();
    kodlama::base64_satirlari_kodla(ham, &mut satirli);
    assert_eq!(
        satirli.len() as u64,
        kodlama::base64_satirli_uzunluk(ham.len() as u64)
    );
    let satirli = String::from_utf8(satirli).unwrap();
    assert_eq!(kodlama::base64_mime_coz(&satirli, ham.len()).unwrap(), ham);
}

fn kati_tek_satir(metin: &str, tavan: usize) {
    match kodlama::base64_coz(metin, tavan) {
        Ok(baytlar) => {
            assert!(baytlar.len() <= tavan);
            assert_eq!(
                kodlama::base64_kodla(&baytlar),
                metin,
                "ikinci yazım kabul edildi"
            );
        }
        Err(hata) => konum_girdide(&hata, metin.len()),
    }
}

fn mime(metin: &str, tavan: usize) {
    match kodlama::base64_mime_coz(metin, tavan) {
        Ok(baytlar) => {
            assert!(baytlar.len() <= tavan);
            let sikisik: String = metin
                .chars()
                .filter(|c| !matches!(c, '\r' | '\n' | ' ' | '\t'))
                .collect();
            assert_eq!(kodlama::base64_kodla(&baytlar), sikisik);
        }
        Err(hata) => konum_girdide(&hata, metin.len()),
    }
}

fn qp(metin: &str, tavan: usize) {
    match kodlama::qp_coz(metin, tavan) {
        Ok(baytlar) => {
            assert!(baytlar.len() <= tavan);
            assert!(baytlar.len() <= metin.len());
        }
        Err(hata) => konum_girdide(&hata, metin.len()),
    }
}

fn utf8(baytlar: &[u8]) {
    let taban = baytlar.len() as u64 * 7;
    match (
        std::str::from_utf8(baytlar),
        kodlama::utf8_coz(baytlar.to_vec(), taban),
    ) {
        (Ok(beklenen), Ok(bulunan)) => assert_eq!(beklenen, bulunan),
        (Err(e), Err(hata)) => {
            assert_eq!(hata.tur, "utf8");
            assert_eq!(hata.konum, Some(taban + e.valid_up_to() as u64));
        }
        (beklenen, bulunan) => panic!("utf8 ayrılığı: {beklenen:?} / {bulunan:?}"),
    }
}

fn pencere(ham: &[u8]) {
    let mut tum = Vec::new();
    kodlama::base64_satirlari_kodla(ham, &mut tum);
    let b = ham.len() as u64;
    // Konum ve adet girdinin kendisinden; uç değerler de denenir.
    let konum = ham.get(1).map_or(0, |&x| u64::from(x) * 3) % (tum.len() as u64 + 3);
    let adet = ham.get(2).map_or(0, |&x| u64::from(x) * 5);
    for (konum, adet) in [(konum, adet), (0, u64::MAX), (u64::MAX, 1)] {
        let p = kodlama::base64_satir_penceresi(b, konum, adet);
        // Ham konum dosya içindedir ve satır başıdır; yalnız boş pencere
        // dosya sonuna (57'nin katı olmayabilir) sabitlenir (K-414).
        assert!(p.atla < 78 && p.ham_baslangic <= b, "{p:?} b={b}");
        assert!(
            p.ham_baslangic.is_multiple_of(57) || p.uzunluk == 0,
            "{p:?} b={b}"
        );
        let mut parca = Vec::new();
        kodlama::base64_satirlari_kodla(&ham[p.ham_baslangic as usize..], &mut parca);
        let bas = (p.atla as usize).min(parca.len());
        let son = (bas + p.uzunluk as usize).min(parca.len());
        let tum_bas = (konum.min(tum.len() as u64)) as usize;
        assert_eq!(&parca[bas..son], &tum[tum_bas..tum_bas + (son - bas)]);
        assert_eq!(son - bas, p.uzunluk as usize);
    }
}
