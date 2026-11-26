//! K-411 (ADR-174): `dkim_imza_girdisi` fuzz hedefinin ve korpus replay
//! testinin ortak özellik denetimi. Panik ya da özellik ihlali bulgu sayılır.
//!
//! Özellikler:
//! 1. Kanonik gövde özeti (simple/relaxed) girdinin hangi bayttan iki parçaya
//!    bölündüğünden bağımsızdır.
//! 2. CR içermeyen girdide LF ile CRLF satır sonu aynı özeti verir.
//! 3. İmza kâhini denetimi kabul ettiği girdide tutarlıdır: CR yok, sınır
//!    içinde, son etiket boş `b=` (adla `=` arasında FWS olabilir), `d=`/`s=` küçük harfli ve boş değil; kanonik
//!    bayt dönüşümü de başarılıdır.
use dil::dkim_kripto::{self, GovdeKipi, KanonikGovdeOzetleyici};

fn ozet(kip: GovdeKipi, parcalar: &[&[u8]]) -> Result<String, dkim_kripto::DkimHatasi> {
    let mut ozetleyici = KanonikGovdeOzetleyici::yeni(kip);
    for parca in parcalar {
        ozetleyici.ekle(parca)?;
    }
    ozetleyici.bitir()
}

pub fn denetle(baytlar: &[u8]) {
    let Some((&secim, govde)) = baytlar.split_first() else {
        return;
    };
    let kes = usize::from(secim) % (govde.len() + 1);
    for kip in [GovdeKipi::Simple, GovdeKipi::Relaxed] {
        let butun = ozet(kip, &[govde]);
        assert_eq!(
            butun,
            ozet(kip, &[&govde[..kes], &govde[kes..]]),
            "parça sınırı {kes}"
        );
        if !govde.contains(&b'\r') {
            let mut crlf = Vec::with_capacity(govde.len() * 2);
            for &b in govde {
                if b == b'\n' {
                    crlf.push(b'\r');
                }
                crlf.push(b);
            }
            assert_eq!(butun, ozet(kip, &[&crlf]), "LF/CRLF eşdeğerliği");
        }
    }
    if let Ok(metin) = std::str::from_utf8(govde) {
        if let Ok(etiketler) = dil::dkim_imza::imza_girdisini_denetle(metin) {
            assert!(!metin.contains('\r') && metin.len() <= dil::dkim_imza::AZAMI_IMZA_GIRDISI);
            // RFC 6376 §3.2: `tag-spec = [FWS] tag-name [FWS] "=" [FWS]
            // tag-value [FWS]`; son etiket boş `b`, adla `=` arasında FWS
            // (`b\t=`, katlanmış `b\n\t=`) geçerlidir. K-417 RC fuzz bulgusu
            // 26.11.2026: önceki `ends_with("b=")` bu biçimi yanlış reddediyordu.
            let son = metin
                .rsplit(';')
                .next()
                .unwrap_or_default()
                .replace('\n', "");
            let (ad, deger) = son.split_once('=').expect("son etiket `=` taşımalı");
            assert_eq!(ad.trim_matches([' ', '\t']), "b", "son etiket `b` olmalı");
            assert!(
                deger.trim_matches([' ', '\t']).is_empty(),
                "`b=` boş olmalı"
            );
            for ad in [&etiketler.alan, &etiketler.secici] {
                assert!(!ad.is_empty() && *ad == ad.to_ascii_lowercase());
            }
            assert!(dkim_kripto::kanonik_baytlar(metin).is_ok());
        }
    }
}
