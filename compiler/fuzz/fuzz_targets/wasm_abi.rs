#![no_main]

use dil::wasm_api::{
    dil_bellek_ayir, dil_bellek_birak, dil_calistir, dil_sonuc_tamponu_uzunlugu, DIL_ABI_BASARILI,
};
use libfuzzer_sys::fuzz_target;

const AZAMI_FUZZ_GIRDISI: usize = 4 * 1024;

fuzz_target!(|veri: &[u8]| {
    let denetim = veri.first().copied().unwrap_or(0);
    let kip = denetim % 6;
    let girdi_mi = denetim & 0x40 != 0;
    let baytlar = &veri[veri.len().min(1)..veri.len().min(1 + AZAMI_FUZZ_GIRDISI)];
    let (asil_ptr, asil_uzunluk) = if baytlar.is_empty() {
        (std::ptr::null_mut(), 0)
    } else {
        let ptr = dil_bellek_ayir(baytlar.len());
        if ptr.is_null() {
            return;
        }
        // SAFETY: ABI exact uzunlukta canlı girdi tamponu döndürdü.
        unsafe { std::ptr::copy_nonoverlapping(baytlar.as_ptr(), ptr, baytlar.len()) };
        (ptr, baytlar.len())
    };

    let (cagri_ptr, cagri_uzunluk) = match kip {
        0 => (asil_ptr.cast_const(), asil_uzunluk),
        1 => (std::ptr::null(), asil_uzunluk.saturating_add(1)),
        2 if asil_uzunluk > 1 => (asil_ptr.wrapping_add(1).cast_const(), asil_uzunluk - 1),
        3 => (std::ptr::dangling::<u8>(), usize::MAX),
        4 => (asil_ptr.cast_const(), asil_uzunluk.saturating_add(1)),
        _ => (asil_ptr.cast_const(), 0),
    };

    let sonuc = if girdi_mi {
        dil_calistir(std::ptr::null(), 0, cagri_ptr, cagri_uzunluk, 7)
    } else {
        dil_calistir(cagri_ptr, cagri_uzunluk, std::ptr::null(), 0, 7)
    };
    if !sonuc.is_null() {
        let toplam = dil_sonuc_tamponu_uzunlugu(sonuc);
        assert!(toplam >= 4);
        // SAFETY: sonuç sorgusu canlı sonuç tamponunun exact boyunu verdi.
        let baytlar = unsafe { std::slice::from_raw_parts(sonuc, toplam) };
        let govde = u32::from_le_bytes([baytlar[0], baytlar[1], baytlar[2], baytlar[3]]) as usize;
        assert_eq!(govde, toplam - 4);
        assert!(std::str::from_utf8(&baytlar[4..]).is_ok());
        assert_eq!(dil_bellek_birak(sonuc, toplam), DIL_ABI_BASARILI);
        assert_eq!(dil_bellek_birak(sonuc, toplam), 0);
    }

    if !asil_ptr.is_null() {
        assert_eq!(dil_bellek_birak(asil_ptr, asil_uzunluk + 1), 0);
        assert_eq!(dil_bellek_birak(asil_ptr, asil_uzunluk), DIL_ABI_BASARILI);
        assert_eq!(dil_bellek_birak(asil_ptr, asil_uzunluk), 0);
    }
});
