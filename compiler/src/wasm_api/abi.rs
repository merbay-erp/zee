//! Hasım host girdisini Rust'ın güvenli dünyasına almadan önce doğrulayan
//! sürümlü WASM C ABI katmanı (K-142/ADR-039).

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::playgroundda_calistir;
use crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI;

pub const DIL_ABI_SURUMU: u32 = 2;
pub const DIL_ABI_BASARILI: u32 = 1;
const DIL_ABI_REDDI: u32 = 0;
const UZUNLUK_ONEKI: usize = 4;
const AZAMI_CANLI_TAMPON: usize = 8;
const AZAMI_TEK_TAMPON_BAYTI: usize = VARSAYILAN_KAYNAK_SINIRLARI.metin_bayti() + UZUNLUK_ONEKI;
const AZAMI_CANLI_TAMPON_BAYTI: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TamponTuru {
    Girdi,
    Sonuc,
}

struct Tampon {
    baytlar: Box<[u8]>,
    tur: TamponTuru,
}

#[derive(Default)]
struct TamponKayitlari {
    tamponlar: HashMap<usize, Tampon>,
    toplam_bayt: usize,
}

static TAMPONLAR: OnceLock<Mutex<TamponKayitlari>> = OnceLock::new();

fn kayitlar() -> MutexGuard<'static, TamponKayitlari> {
    let kilit = TAMPONLAR.get_or_init(|| Mutex::new(TamponKayitlari::default()));
    match kilit.lock() {
        Ok(kayitlar) => kayitlar,
        Err(zehirli) => zehirli.into_inner(),
    }
}

fn bos_tampon(uzunluk: usize) -> Option<Box<[u8]>> {
    if uzunluk == 0 || uzunluk > AZAMI_TEK_TAMPON_BAYTI {
        return None;
    }
    let mut baytlar = Vec::new();
    if baytlar.try_reserve_exact(uzunluk).is_err() {
        return None;
    }
    baytlar.resize(uzunluk, 0);
    Some(baytlar.into_boxed_slice())
}

fn tamponu_kaydet(mut baytlar: Box<[u8]>, tur: TamponTuru) -> *mut u8 {
    let uzunluk = baytlar.len();
    let ptr = baytlar.as_mut_ptr();
    let anahtar = ptr as usize;
    let mut kayitlar = kayitlar();
    let Some(yeni_toplam) = kayitlar.toplam_bayt.checked_add(uzunluk) else {
        return std::ptr::null_mut();
    };
    if kayitlar.tamponlar.len() >= AZAMI_CANLI_TAMPON
        || yeni_toplam > AZAMI_CANLI_TAMPON_BAYTI
        || kayitlar.tamponlar.contains_key(&anahtar)
    {
        return std::ptr::null_mut();
    }
    if kayitlar.tamponlar.try_reserve(1).is_err() {
        return std::ptr::null_mut();
    }
    kayitlar.tamponlar.insert(anahtar, Tampon { baytlar, tur });
    kayitlar.toplam_bayt = yeni_toplam;
    ptr
}

fn giris_baytlarini_kopyala(ptr: *const u8, uzunluk: usize) -> Result<Vec<u8>, &'static str> {
    if uzunluk == 0 {
        return if ptr.is_null() {
            Ok(Vec::new())
        } else {
            Err("sıfır uzunluk yalnız null pointer ile verilebilir")
        };
    }
    if ptr.is_null() {
        return Err("null pointer sıfır olmayan uzunluk taşıyamaz");
    }
    let kayitlar = kayitlar();
    let Some(tampon) = kayitlar.tamponlar.get(&(ptr as usize)) else {
        return Err("pointer bu ABI tarafından ayrılmış canlı bir tampon değil");
    };
    if tampon.tur != TamponTuru::Girdi {
        return Err("sonuç tamponu girdi olarak kullanılamaz");
    }
    if tampon.baytlar.len() != uzunluk {
        return Err("pointer ve uzunluk ayrılmış tamponla birebir eşleşmiyor");
    }
    Ok(tampon.baytlar.to_vec())
}

fn sonuc_tamponu(metin: &str) -> *mut u8 {
    let govde = metin.as_bytes();
    let Ok(govde_uzunlugu) = u32::try_from(govde.len()) else {
        return std::ptr::null_mut();
    };
    let Some(toplam) = UZUNLUK_ONEKI.checked_add(govde.len()) else {
        return std::ptr::null_mut();
    };
    let Some(mut tampon) = bos_tampon(toplam) else {
        return std::ptr::null_mut();
    };
    tampon[..UZUNLUK_ONEKI].copy_from_slice(&govde_uzunlugu.to_le_bytes());
    tampon[UZUNLUK_ONEKI..].copy_from_slice(govde);
    tamponu_kaydet(tampon, TamponTuru::Sonuc)
}

fn abi_hatasi(neden: &str) -> *mut u8 {
    sonuc_tamponu(&format!("WASM ABI HATASI: {neden}"))
}

/// Hostun bağlanması gereken exact ABI sürümünü döndürür.
#[no_mangle]
pub extern "C" fn dil_abi_surumu() -> u32 {
    DIL_ABI_SURUMU
}

/// `uzunluk` baytlık, sıfırla başlatılmış ve kayıtlı girdi tamponu ayırır.
/// Sıfır, aşırı boyut, canlı tampon kotası veya tahsis hatasında null döner.
#[no_mangle]
pub extern "C" fn dil_bellek_ayir(uzunluk: usize) -> *mut u8 {
    let Some(tampon) = bos_tampon(uzunluk) else {
        return std::ptr::null_mut();
    };
    tamponu_kaydet(tampon, TamponTuru::Girdi)
}

/// Yalnız tam başlangıç pointer'ı ve exact uzunluk eşleşirse tamponu bırakır.
/// Yanlış, kayıt dışı veya daha önce bırakılmış çiftler durumu değiştirmez.
#[no_mangle]
pub extern "C" fn dil_bellek_birak(ptr: *mut u8, uzunluk: usize) -> u32 {
    if ptr.is_null() {
        return if uzunluk == 0 {
            DIL_ABI_BASARILI
        } else {
            DIL_ABI_REDDI
        };
    }
    let anahtar = ptr as usize;
    let mut kayitlar = kayitlar();
    let Some(tampon) = kayitlar.tamponlar.get(&anahtar) else {
        return DIL_ABI_REDDI;
    };
    if tampon.baytlar.len() != uzunluk {
        return DIL_ABI_REDDI;
    }
    let Some(tampon) = kayitlar.tamponlar.remove(&anahtar) else {
        return DIL_ABI_REDDI;
    };
    kayitlar.toplam_bayt = kayitlar.toplam_bayt.saturating_sub(tampon.baytlar.len());
    DIL_ABI_BASARILI
}

/// Kayıtlı bir sonuç tamponunun başlık dahil exact boyutunu döndürür.
/// Girdi, kayıt dışı veya bırakılmış pointer için sıfırdır.
#[no_mangle]
pub extern "C" fn dil_sonuc_tamponu_uzunlugu(ptr: *const u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let kayitlar = kayitlar();
    kayitlar
        .tamponlar
        .get(&(ptr as usize))
        .filter(|tampon| tampon.tur == TamponTuru::Sonuc)
        .map_or(0, |tampon| tampon.baytlar.len())
}

/// Kayıtlı ve exact pointer/uzunluk çiftlerini kopyalayıp UTF-8 doğruladıktan
/// sonra çekirdeği çalıştırır. Her ABI reddi sahipli, uzunluk-önekli Türkçe
/// hata tamponudur; sonuç tahsis edilemezse null döner.
#[no_mangle]
pub extern "C" fn dil_calistir(
    kaynak_ptr: *const u8,
    kaynak_uzunluk: usize,
    girdi_ptr: *const u8,
    girdi_uzunluk: usize,
    tohum: u64,
) -> *mut u8 {
    let kaynak_baytlari = match giris_baytlarini_kopyala(kaynak_ptr, kaynak_uzunluk) {
        Ok(baytlar) => baytlar,
        Err(neden) => return abi_hatasi(&format!("kaynak: {neden}")),
    };
    let girdi_baytlari = match giris_baytlarini_kopyala(girdi_ptr, girdi_uzunluk) {
        Ok(baytlar) => baytlar,
        Err(neden) => return abi_hatasi(&format!("girdi: {neden}")),
    };
    let kaynak = match String::from_utf8(kaynak_baytlari) {
        Ok(kaynak) => kaynak,
        Err(_) => return abi_hatasi("kaynak geçerli UTF-8 değil"),
    };
    let girdiler = match String::from_utf8(girdi_baytlari) {
        Ok(girdiler) => girdiler,
        Err(_) => return abi_hatasi("girdi geçerli UTF-8 değil"),
    };
    sonuc_tamponu(&playgroundda_calistir(&kaynak, &girdiler, tohum))
}
