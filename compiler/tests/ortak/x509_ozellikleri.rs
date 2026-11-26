//! K-411 (ADR-173): `x509_spki` fuzz hedefinin ve korpus replay testinin
//! ortak özellik denetimi. Kâhin, aynı X.690 DER kurallarını ayrı yazılmış
//! bir referans okuyucuyla uygulayan diferansiyel karşılaştırmadır; ayrıca
//! dönen dilim girdinin içinde, kendi başına tam bir SEQUENCE TLV'si olmalı.

const AZAMI: usize = 64 * 1024;

/// Referans TLV başlığı: (etiket, başlık boyu, içerik boyu). X.690 §8.1.3:
/// uzun biçim en çok 4 bayt, baştaki sıfır ve 128'den küçük değer kanonik
/// değil; belirsiz uzunluk ve yüksek etiket numarası yok.
fn baslik(veri: &[u8]) -> Option<(u8, usize, usize)> {
    let etiket = *veri.first()?;
    if etiket & 0x1f == 0x1f {
        return None;
    }
    let ilk = *veri.get(1)?;
    if ilk & 0x80 == 0 {
        return Some((etiket, 2, usize::from(ilk)));
    }
    let adet = usize::from(ilk & 0x7f);
    if !(1..=4).contains(&adet) || veri.len() < 2 + adet || veri[2] == 0 {
        return None;
    }
    let boy = veri[2..2 + adet]
        .iter()
        .fold(0u64, |t, b| t * 256 + u64::from(*b));
    if boy < 0x80 {
        return None;
    }
    Some((etiket, 2 + adet, usize::try_from(boy).ok()?))
}

/// Tam TLV: (etiket, içerik, TLV'nin tamamı, kalan).
type Tlv<'a> = (u8, &'a [u8], &'a [u8], &'a [u8]);

fn tlv(veri: &[u8]) -> Option<Tlv<'_>> {
    let (etiket, b, n) = baslik(veri)?;
    let son = b.checked_add(n)?;
    (veri.len() >= son).then(|| (etiket, &veri[b..son], &veri[..son], &veri[son..]))
}

/// Referans SPKI çıkarımı (RFC 5280 §4.1 iskeleti).
pub fn referans_spki(sertifika: &[u8]) -> Option<&[u8]> {
    if sertifika.len() > AZAMI {
        return None;
    }
    let (0x30, govde, _, []) = tlv(sertifika)? else {
        return None;
    };
    let (0x30, mut tbs, _, _) = tlv(govde)? else {
        return None;
    };
    if tbs.first() == Some(&0xa0) {
        tbs = tlv(tbs)?.3;
    }
    for _ in 0..5 {
        tbs = tlv(tbs)?.3;
    }
    let (0x30, _, spki, _) = tlv(tbs)? else {
        return None;
    };
    Some(spki)
}

pub fn denetle(baytlar: &[u8]) {
    let zee = dil::ag_satiri::x509::spki_dilimi(baytlar);
    assert_eq!(zee, referans_spki(baytlar), "referans okuyucuyla ayrılık");
    if let Some(spki) = zee {
        let bas = spki.as_ptr() as usize - baytlar.as_ptr() as usize;
        assert!(bas + spki.len() <= baytlar.len(), "dilim girdinin dışında");
        let (etiket, _, tam, kalan) = tlv(spki).expect("dilim tam bir TLV olmalı");
        assert!(etiket == 0x30 && tam.len() == spki.len() && kalan.is_empty());
    }
}
