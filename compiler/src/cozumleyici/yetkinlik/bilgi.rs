//! Yetkinlik gereksinimi biriktiricisi.
use super::{blok_bilgisi, ifade_bilgisi, Konum};
use crate::agac::Cumle;
use crate::yetkinlik::Yetkinlik;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Bilgi {
    pub(super) yetkinlikler: BTreeMap<Yetkinlik, Konum>,
    pub(super) ag_istekleri: Vec<(String, Konum)>,
}
impl Bilgi {
    pub(super) fn ekle(&mut self, yetkinlik: Yetkinlik, konum: Konum) {
        if yetkinlik == Yetkinlik::WebOturumu {
            self.ekle(Yetkinlik::AgSunucusu, konum);
        }
        self.yetkinlikler
            .entry(yetkinlik)
            .and_modify(|onceki| *onceki = (*onceki).min(konum))
            .or_insert(konum);
    }
}

impl Bilgi {
    /// K-381: bağlantı bloğu ve cümleleri `ağ-dinleyici` ister.
    pub(super) fn baglanti(&mut self, cumle: &Cumle) {
        match cumle {
            Cumle::BaglantiGeldiginde {
                dinleyici,
                govde,
                satir,
            } => {
                self.ekle(Yetkinlik::AgDinleyici, (*satir, 1, 1));
                ifade_bilgisi(dinleyici, self);
                blok_bilgisi(govde, self);
            }
            Cumle::BaglantiyaYaz { deger, satir, .. } => {
                self.ekle(Yetkinlik::AgDinleyici, (*satir, 1, 1));
                ifade_bilgisi(deger, self);
            }
            Cumle::BaglantiyiKapat { satir } => self.ekle(Yetkinlik::AgDinleyici, (*satir, 1, 1)),
            _ => {}
        }
    }
}
