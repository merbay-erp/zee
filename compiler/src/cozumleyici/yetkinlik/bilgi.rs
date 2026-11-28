//! Yetkinlik gereksinimi biriktiricisi.
use super::{blok_bilgisi, ifade_bilgisi, Konum};
use crate::agac::Cumle;
use crate::intrinsic;
use crate::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Bilgi {
    pub(super) yetkinlikler: BTreeMap<Yetkinlik, Konum>,
    pub(super) ag_istekleri: Vec<(String, Konum)>,
    /// K-397: bağlantı kalıbının ilk konumu (`ağ-dinleyici` ya da `ağ` yeter).
    pub(super) baglanti_islemi: Option<Konum>,
}
impl Bilgi {
    /// RFC-0047 (ADR-182 dilim C): DAV kimliği HMAC'li uygulama parolasıdır.
    pub(super) fn dav_politikasi_ekle(&mut self, cumle: &Cumle) {
        if let Cumle::RotaPolitikasi {
            kip: crate::agac::RotaKipi::Dav,
            satir,
            ..
        } = cumle
        {
            self.ekle(Yetkinlik::Kriptografi, (*satir, 1, 1));
        }
    }

    pub(super) fn baglanti_islemi_ekle(&mut self, konum: Konum) {
        self.baglanti_islemi = Some(self.baglanti_islemi.map_or(konum, |k| k.min(konum)));
    }

    /// Intrinsic'in tablo yetkinliği; K-397 bağlantı işlemleri ayrı biriktirilir.
    pub(super) fn intrinsic_ekle(&mut self, kimlik: &str, konum: Konum) {
        if intrinsic::baglanti_islemi_mi(kimlik) {
            self.baglanti_islemi_ekle(konum);
        } else if let Some(tanim) = intrinsic::tanim(kimlik) {
            self.ekle(tanim.yetkinlik, konum);
        }
    }

    /// K-397: bağlantı kalıbı `ağ-dinleyici` ya da `ağ` ile karşılanır;
    /// ikisi de yoksa `ağ-dinleyici` gereksinimi olarak raporlanır.
    pub(super) fn baglanti_gereksinimini_coz(&mut self, politika: &YetkinlikPolitikasi) {
        if let Some(konum) = self.baglanti_islemi {
            if !politika.izin_verir(Yetkinlik::AgDinleyici) && !politika.izin_verir(Yetkinlik::Ag) {
                self.ekle(Yetkinlik::AgDinleyici, konum);
            }
        }
    }

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
                self.baglanti_islemi_ekle((*satir, 1, 1));
                ifade_bilgisi(deger, self);
            }
            Cumle::BaglantiyiKapat { satir } => self.baglanti_islemi_ekle((*satir, 1, 1)),
            _ => {}
        }
    }
}

/// K-394 (RFC-0045): yol sınıfı (`veri:` ya da proje dosyası) çoğu zaman
/// çalışma anında belli olur; statik dosya gereksinimi aynı yönlü `veri-*`
/// yetkinliğiyle de karşılanır. Yolun kesin sınıfını ve yetkinliğini
/// çalışma zamanındaki merkezî politika her çağrıda ayrıca denetler.
pub(super) fn statik_izin(politika: &YetkinlikPolitikasi, yetkinlik: Yetkinlik) -> bool {
    politika.izin_verir(yetkinlik)
        || match yetkinlik {
            Yetkinlik::DosyaOkuma => politika.izin_verir(Yetkinlik::VeriOkuma),
            Yetkinlik::DosyaYazma => politika.izin_verir(Yetkinlik::VeriYazma),
            _ => false,
        }
}
