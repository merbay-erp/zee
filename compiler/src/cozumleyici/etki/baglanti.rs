//! K-381: bağlantı cümlelerinin etkisi. Okuma/yazma/kapatma geri alınamaz
//! dış etkidir (eylem içinde T048); blok gövdesi kendi etkisini taşır.
use super::*;

pub(super) fn etkisi(cumle: &Cumle, bilgi: &mut Bilgi) {
    match cumle {
        Cumle::BaglantiGeldiginde {
            dinleyici, govde, ..
        } => {
            ifade_bilgisi(dinleyici, bilgi);
            bilgi.birlestir(&blok_bilgisi(govde));
        }
        Cumle::BaglantiyaYaz { deger, .. } => {
            bilgi.geri_alinamaz = true;
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::BaglantiyiKapat { .. } => bilgi.geri_alinamaz = true,
        _ => {}
    }
}
