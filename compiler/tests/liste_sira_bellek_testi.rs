//! K-386 / RFC-0029 / spec/17: sıra ile okuma listenin tamamını KOPYALAMAZ.
//!
//! Zamanlama yerine deterministik kanıt: bu test ikilisinin global ayırıcısı
//! ayrılan baytları sayar. Aynı programın 1 ve 1001 sıra okumalı iki hâli
//! arasındaki fark, erişim başına liste boyundan bağımsız küçük bir sabit
//! olmalıdır. Liste kopyası erişim başına ≥ n × öğe boyu ayırırdı. Dosyada tek
//! test vardır; başka iş parçacığı sayacı kirletmez.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct SayanAyirici;

static AYRILAN: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for SayanAyirici {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        AYRILAN.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, yeni: usize) -> *mut u8 {
        AYRILAN.fetch_add(yeni, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, yeni) }
    }
}

#[global_allocator]
static AYIRICI: SayanAyirici = SayanAyirici;

const N: usize = 20_000;

fn program(okuma: usize) -> String {
    format!(
        "sayılar boş liste olsun\n1 den {N} e kadar her i için\n    sayılara i ekle\n\
         toplam 0 olsun\nsıra {N} olsun\n1 den {okuma:05} e kadar her j için\n    \
         değer sayıların sıra. öğesi olsun\n    toplamı değerle artır\ntoplam yaz\n"
    )
}

fn ayrilan_bayt(kaynak: &str) -> usize {
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    let once = AYRILAN.load(Ordering::Relaxed);
    dil::yorumlayici::calistir_io_kodla(&program, &mut io).expect("çalışmalı");
    let sonra = AYRILAN.load(Ordering::Relaxed);
    assert_eq!(io.cikti.len(), 1);
    sonra - once
}

#[test]
fn sira_ile_okuma_liste_boyundan_bagimsiz_bellek_ayirir() {
    let tek = ayrilan_bayt(&program(1));
    let bin = ayrilan_bayt(&program(1001));
    let erisim_basina = bin.saturating_sub(tek) / 1000;
    let liste_kopyasi = N * std::mem::size_of::<dil::yorumlayici::Deger>();
    assert!(
        erisim_basina * 100 < liste_kopyasi,
        "erişim başına {erisim_basina} bayt; tam liste kopyası {liste_kopyasi} bayt olurdu"
    );
}
