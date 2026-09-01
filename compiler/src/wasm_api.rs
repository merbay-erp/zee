//! Playground köprüsü (Faz 6): derleyiciyi tarayıcıya taşıyan C-ABI yüzeyi.
//!
//! wasm-bindgen KULLANILMAZ (ADR-001 sıfır bağımlılık): bellek elle yönetilir.
//! Sözleşme:
//! - `dil_bellek_ayir(n)` → n baytlık tampon işaretçisi (JS kaynağı buraya yazar)
//! - `dil_calistir(kaynak, k_len, girdi, g_len, tohum)` → sonuç tamponu:
//!   ilk 4 bayt little-endian uzunluk, ardından UTF-8 çıktı metni.
//!   Girdi metni satır satır "diye sor" cevaplarıdır; tohum rastgeleliği besler.
//! - Dönen tampon `dil_bellek_birak(ptr, 4+len)` ile bırakılır.
//!
//! Bu modül doğal (native) derlemede de derlenir ve testlenir — determinizm
//! garantisi playground'da da aynıdır.

use crate::yorumlayici::{GirdiCikti, ToplayanIo};

/// Tarayıcı IO'su: ToplayanIo'nun determinizmi + tohumlu rastgelelik.
/// Dosya sistemi RAM'dedir (sayfa yenilenince uçar — playground sözleşmesi).
struct PlaygroundIo {
    ic: ToplayanIo,
    tohum: u64,
}

impl GirdiCikti for PlaygroundIo {
    fn yazdir(&mut self, satir: String) {
        self.ic.yazdir(satir);
    }
    fn sor(&mut self, istem: &str) -> Option<String> {
        self.ic.sor(istem)
    }
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        // xorshift64* — CLI'dekiyle aynı üreteç, tohum JS'ten gelir.
        let mut x = self.tohum | 1;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.tohum = x;
        let genislik = (ust - alt) as u64 + 1;
        alt + (x.wrapping_mul(0x2545F4914F6CDD1D) % genislik) as i64
    }
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        self.ic.dosya_oku(yol)
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        self.ic.dosya_yaz(yol, satir, ekleme)
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.ic.simdi()
    }
    fn argumanlar(&mut self) -> Vec<String> {
        Vec::new()
    }
    fn http_getir(
        &mut self,
        _url: &str,
        _zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        Err("playground'da ağ erişimi yok".into())
    }
    fn sunucu_kur(&mut self, _kapi: i64) -> Result<(), String> {
        Err("playground'da sunucu açılamaz".into())
    }
    fn istek_al(&mut self) -> Option<String> {
        None
    }
    fn yanit_gonder(&mut self, _yanit: &str) {}
    fn yonlendir_gonder(&mut self, _adres: &str) {}
    fn cerez_yaz(&mut self, _ad: &str, _deger: &str) {}
    fn cerez_sil(&mut self, _ad: &str) {}
    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        self.ic.sensor_acik_mi(ad)
    }
    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        self.ic.isik_ayarla(ad, yansin);
    }
    fn bekle_ms(&mut self, milisaniye: i64) {
        // Tarayıcıyı gerçekten uyutma; ToplayanIo'nun sanal saatini ilerlet.
        // Böylece ardışık kısa beklemeler de masaüstündekiyle aynı son tarih
        // anlamını taşır.
        self.ic.bekle_ms(milisaniye);
    }
    fn an_ms(&mut self) -> i64 {
        self.ic.an_ms()
    }
}

/// Kaynağı verilen girdi satırları ve tohumla koşturur; çıktı metnini üretir.
/// (Saf çekirdek — hem wasm dışa aktarımı hem doğal testler bunu kullanır.)
pub fn playgroundda_calistir(kaynak: &str, girdiler: &str, tohum: u64) -> String {
    let girdi_listesi: Vec<String> = if girdiler.is_empty() {
        Vec::new()
    } else {
        girdiler.lines().map(str::to_string).collect()
    };
    let mut io = PlaygroundIo {
        ic: ToplayanIo::yeni(girdi_listesi),
        tohum: tohum ^ 0x5EED_2EE5,
    };

    // Playground'da birimler gömülü kitaplıktan gelir (RFC-0014): yerel
    // dosya sistemi yok, ama standart birimler tarayıcıda da çalışır.
    let mut yukleyici = |ad: &str| -> Result<String, String> {
        crate::gomulu_birim(ad).map(str::to_string).ok_or_else(|| {
            format!(
                "playground'da yalnız gömülü birimler kullanılabilir (var olanlar: {})",
                crate::gomulu_birim_adlari().join(", ")
            )
        })
    };
    let program = match crate::kaynagi_derle_birimlerle(kaynak, &mut yukleyici) {
        Ok(program) => program,
        Err(tani) => return tani.raporla(kaynak),
    };
    match crate::yorumlayici::calistir_io(&program, &mut io) {
        Ok(()) => {
            let mut cikti = io.ic.cikti.join("\n");
            // Birimden miras testler (önekli) raporlanmaz: playground yalnız
            // kullanıcının kendi test bloklarını sayar (RFC-0014).
            let birim_testi = |ad: &str| {
                crate::gomulu_birim_adlari()
                    .iter()
                    .any(|birim| ad.starts_with(&format!("{}: ", birim)))
            };
            if program.testler.iter().any(|t| !birim_testi(&t.ad)) {
                let sonuclar: Vec<_> = crate::programi_dene(&program)
                    .into_iter()
                    .filter(|s| !birim_testi(&s.ad))
                    .collect();
                let gecen = sonuclar.iter().filter(|s| s.hata.is_none()).count();
                cikti.push_str(&format!(
                    "\n\n— testler: {} / {} geçti —",
                    gecen,
                    sonuclar.len()
                ));
                for sonuc in sonuclar.iter().filter(|s| s.hata.is_some()) {
                    cikti.push_str(&format!("\n✗ {}", sonuc.ad));
                }
            }
            cikti
        }
        Err(tani) => {
            let mut cikti = io.ic.cikti.join("\n");
            if !cikti.is_empty() {
                cikti.push('\n');
            }
            cikti.push_str(&tani.raporla(kaynak));
            cikti
        }
    }
}

// ---------- C-ABI dışa aktarımları (wasm) ----------

/// # Safety
/// JS tarafı sözleşmeye uyar: işaretçiler bu modülün ayırdığı tamponlardır.
#[no_mangle]
pub extern "C" fn dil_bellek_ayir(uzunluk: usize) -> *mut u8 {
    let mut tampon = Vec::<u8>::with_capacity(uzunluk.max(1));
    let ptr = tampon.as_mut_ptr();
    std::mem::forget(tampon);
    ptr
}

/// # Safety
/// `ptr`, `dil_bellek_ayir(uzunluk)` ile alınmış olmalıdır.
#[no_mangle]
pub unsafe extern "C" fn dil_bellek_birak(ptr: *mut u8, uzunluk: usize) {
    if !ptr.is_null() {
        drop(Vec::from_raw_parts(ptr, 0, uzunluk.max(1)));
    }
}

/// # Safety
/// İşaretçiler geçerli, uzunlukları doğru UTF-8 tamponlara işaret etmelidir.
#[no_mangle]
pub unsafe extern "C" fn dil_calistir(
    kaynak_ptr: *const u8,
    kaynak_uzunluk: usize,
    girdi_ptr: *const u8,
    girdi_uzunluk: usize,
    tohum: u64,
) -> *mut u8 {
    let kaynak = std::str::from_utf8(std::slice::from_raw_parts(kaynak_ptr, kaynak_uzunluk))
        .unwrap_or("");
    let girdiler = if girdi_uzunluk == 0 {
        ""
    } else {
        std::str::from_utf8(std::slice::from_raw_parts(girdi_ptr, girdi_uzunluk)).unwrap_or("")
    };

    let cikti = playgroundda_calistir(kaynak, girdiler, tohum);
    let govde = cikti.as_bytes();

    let mut tampon = Vec::<u8>::with_capacity(4 + govde.len());
    tampon.extend_from_slice(&(govde.len() as u32).to_le_bytes());
    tampon.extend_from_slice(govde);
    let ptr = tampon.as_mut_ptr();
    std::mem::forget(tampon);
    ptr
}
