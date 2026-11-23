//! K-381 (RFC-0039 A2a): bağlantı olay bloğu ve kapatma cümlesi.
use super::*;

impl Ayristirici {
    /// `"smtp_v4" dinleyicisinde bağlantı geldiğinde` + gövde ve
    /// `bağlantıyı kapat`; başka satırda `None`.
    pub(super) fn baglanti_cumlesi(
        &mut self,
        t: &[Token],
        satir_no: usize,
    ) -> Option<Result<Cumle, Tani>> {
        if t.len() == 2 && kelime_mi(&t[0], "bağlantıyı") && kelime_mi(&t[1], "kapat") {
            return Some(Ok(Cumle::BaglantiyiKapat { satir: satir_no }));
        }
        if t.len() == 4
            && kelime_mi(&t[1], "dinleyicisinde")
            && kelime_mi(&t[2], "bağlantı")
            && kelime_mi(&t[3], "geldiğinde")
        {
            return Some((|| {
                let dinleyici = tekil_ifade(t[0].clone())?;
                let govde = self.alt_blok(satir_no)?;
                Ok(Cumle::BaglantiGeldiginde {
                    dinleyici,
                    govde,
                    satir: satir_no,
                })
            })());
        }
        None
    }
}
