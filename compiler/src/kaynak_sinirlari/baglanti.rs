//! Süreç genelindeki inbound ve outbound ağ bağlantısı izinleri.

use super::VARSAYILAN_KAYNAK_SINIRLARI;
use std::sync::atomic::{AtomicUsize, Ordering};

static ACIK_BAGLANTI: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct BaglantiIzni {
    _ozel: (),
}

pub fn baglanti_izni_al() -> Result<BaglantiIzni, String> {
    let azami = VARSAYILAN_KAYNAK_SINIRLARI.ag_baglantisi();
    ACIK_BAGLANTI
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |acik| {
            (acik < azami).then_some(acik + 1)
        })
        .map(|_| BaglantiIzni { _ozel: () })
        .map_err(|_| format!("süreç {} eşzamanlı ağ bağlantısı sınırına ulaştı", azami))
}

impl Drop for BaglantiIzni {
    fn drop(&mut self) {
        let onceki = ACIK_BAGLANTI.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(onceki > 0, "bağlantı izni sayacı sıfırın altına inemez");
    }
}
