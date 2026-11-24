//! K-381 (RFC-0039 A2a): bağlantı bloğu ve bağlantı cümlelerinin tür denetimi.
use super::*;

fn metin_olmali(tur: Tur, ne: &str, satir: usize) -> Result<(), Tani> {
    if tur == Tur::Metin {
        return Ok(());
    }
    Err(Tani::yeni(
        "T034",
        format!("{ne} Metin olmalı; burada {} var.", tur.adi()),
        satir,
        1,
        1,
    ))
}

pub(super) fn denetle(
    cumle: &mut Cumle,
    ortam: &SembolTablosu,
    baglam: &mut Baglam,
) -> Result<(), Tani> {
    match cumle {
        Cumle::BaglantiGeldiginde {
            dinleyici,
            govde,
            satir,
        } => {
            let satir = *satir;
            let tur = ifade_denetle(dinleyici, ortam, baglam, satir)?;
            metin_olmali(tur, "Dinleyici adı", satir)?;
            // Her bağlantı taze ortamda; tek örtük ad `bağlantı`.
            let mut baglanti_ortami = SembolTablosu::yeni(baglam.istek_kapsami(dinleyici, satir)?);
            baglanti_ortami.insert("bağlantı".into(), Tur::Sozluk(SozlukDegerTuru::Metin));
            blok_denetle(govde, &mut baglanti_ortami, baglam)
        }
        Cumle::BaglantiyaYaz { deger, satir, .. } => {
            let tur = ifade_denetle(deger, ortam, baglam, *satir)?;
            metin_olmali(tur, "Bağlantıya yazılan satır", *satir)
        }
        _ => Ok(()),
    }
}
