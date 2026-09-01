//! Ağaç-yürüyen yorumlayıcı (ADR-003: ilk execution modeli).
//!
//! Tür denetiminden geçmiş programı çalıştırır. Çıktı satır listesi olarak
//! döner; CLI bunu ekrana basar, testler doğrudan karşılaştırır.

mod cumle;
mod hir_gecisi;
mod ifade;
mod io_izi;
mod io_profili;

use self::cumle::blok_calistir_async;
use self::hir_gecisi::CalistirmaProgrami;
use self::ifade::degerlendir_async;
pub use self::hir_gecisi::{calistir_baglanmis, calistir_baglanmis_io, test_calistir_baglanmis};
pub use self::io_izi::{
    IzKaydedenIo, IzYenidenOynatici, AZAMI_IO_IZ_BAYTI, AZAMI_IO_IZ_OLAYI,
};
pub use self::io_profili::{SurumluRastgele, DETERMINISTIK_IO_PROFILI};

use crate::agac::{
    AritmetikIslec, Cumle, HttpYontemi, Ifade, Islec, IslemTuru, Ozellik, Program, RotaErisimi,
};
use crate::intrinsic::{self, CSRF_BELIRTECI, HTTP_GETIR, PAROLA_DOGRULA, SENSOR_ACIK_MI};
use crate::ondalik::Ondalik;
use crate::tani::Tani;
use crate::web_guvenligi::{WebGuvenligi, WebReddi, YeniOturum};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

/// Girdi/çıktı ve rastgelelik soyutlaması: testler deterministik kuyruk
/// kullanır, CLI gerçek klavye/ekran ve gerçek rastgelelik.
/// Ham istek metnini çözer (K-051). Biçim: "YÖNTEM yol?sorgu\ngövde" ya da
/// yalnız "/yol" (= GET). Dönen: (yöntem, salt yol, istek sözlüğü girdileri).
pub fn istek_parcala(ham: &str) -> (String, String, Vec<(String, String)>) {
    let (yontem, yol, veriler, _) = istek_parcala_cerezli(ham);
    (yontem, yol, veriler)
}

/// (ad, değer) çiftleri — istek verileri ve çerezler bu biçimde taşınır.
pub type AdDegerler = Vec<(String, String)>;

pub const AZAMI_ISTEK_GOVDESI: usize = 64 * 1024;
pub const AZAMI_ISTEK_ALANI: usize = 100;

fn istek_sinirlarini_denetle(ham: &str) -> Result<(), (u16, &'static str)> {
    let (ilk_satir, kalan) = ham.split_once('\n').unwrap_or((ham, ""));
    let (yontem, hedef) = ilk_satir.split_once(' ').unwrap_or(("GET", ilk_satir));
    let (_, sorgu) = hedef.split_once('?').unwrap_or((hedef, ""));
    let govde = match kalan.strip_prefix("çerez ") {
        Some(devam) => devam.split_once('\n').map(|(_, govde)| govde).unwrap_or(""),
        None => kalan,
    };
    if govde.len() > AZAMI_ISTEK_GOVDESI {
        return Err((413, "istek gövdesi 64 KiB sınırını aşıyor"));
    }
    let govde_kullanilir = matches!(
        yontem.to_ascii_uppercase().as_str(),
        "POST" | "PUT" | "PATCH" | "DELETE"
    );
    let alan_sayisi = sorgu.split('&').filter(|alan| !alan.is_empty()).count()
        + if govde_kullanilir {
            govde.split('&').filter(|alan| !alan.is_empty()).count()
        } else {
            0
        };
    if alan_sayisi > AZAMI_ISTEK_ALANI {
        return Err((413, "istek 100 alan sınırını aşıyor"));
    }
    Ok(())
}

/// istek_parcala + çerezler (K-052). İkinci satır "çerez a=1; b=2" ise
/// Cookie başlığıdır; kalan satırlar gövdedir.
pub fn istek_parcala_cerezli(ham: &str) -> (String, String, AdDegerler, AdDegerler) {
    let (ilk_satir, kalan) = ham.split_once('\n').unwrap_or((ham, ""));
    let (cerez_satiri, govde) = match kalan.strip_prefix("çerez ") {
        Some(devam) => match devam.split_once('\n') {
            Some((c, g)) => (c, g),
            None => (devam, ""),
        },
        None => ("", kalan),
    };
    let mut cerezler: Vec<(String, String)> = Vec::new();
    for cift in cerez_satiri.split(';') {
        let cift = cift.trim();
        if cift.is_empty() {
            continue;
        }
        let (ad, deger) = cift.split_once('=').unwrap_or((cift, ""));
        cerezler.push((ad.trim().to_string(), deger.trim().to_string()));
    }
    let ham = ilk_satir;
    let govde_tam = govde;
    let (yontem, yol, veriler) = istek_govdesiyle(ham, govde_tam);
    (yontem, yol, veriler, cerezler)
}

fn istek_govdesiyle(ilk_satir: &str, govde: &str) -> (String, String, Vec<(String, String)>) {
    let (yontem, hedef) = match ilk_satir.split_once(' ') {
        Some((y, h)) => (y.to_uppercase(), h.trim()),
        None => ("GET".to_string(), ilk_satir.trim()),
    };
    let (yol, sorgu) = hedef.split_once('?').unwrap_or((hedef, ""));
    let mut veriler: Vec<(String, String)> = Vec::new();
    veriler.push(("yol".into(), yol.to_string()));
    veriler.push(("yöntem".into(), yontem.clone()));
    let govde_kullanilir = matches!(yontem.as_str(), "POST" | "PUT" | "PATCH" | "DELETE");
    for kaynak in [sorgu, if govde_kullanilir { govde.trim() } else { "" }] {
        for cift in kaynak.split('&').filter(|p| !p.is_empty()) {
            let (ad, deger) = cift.split_once('=').unwrap_or((cift, ""));
            let ad = url_coz(ad);
            let deger = url_coz(deger);
            match veriler.iter_mut().find(|(v_ad, _)| *v_ad == ad) {
                Some((_, v)) => *v = deger,
                None => veriler.push((ad, deger)),
            }
        }
    }
    (yontem, yol.to_string(), veriler)
}

/// Yüzde-kodlamayı ve formdaki artıyı çözer (UTF-8).
fn url_coz(metin: &str) -> String {
    let mut baytlar: Vec<u8> = Vec::with_capacity(metin.len());
    let mut karakterler = metin.bytes().peekable();
    while let Some(b) = karakterler.next() {
        match b {
            b'+' => baytlar.push(b' '),
            b'%' => {
                let yuksek = karakterler.next().and_then(|k| (k as char).to_digit(16));
                let dusuk = karakterler.next().and_then(|k| (k as char).to_digit(16));
                match (yuksek, dusuk) {
                    (Some(y), Some(d)) => baytlar.push((y * 16 + d) as u8),
                    _ => baytlar.push(b'%'),
                }
            }
            b => baytlar.push(b),
        }
    }
    String::from_utf8_lossy(&baytlar).into_owned()
}

/// Türk alfabesi sırası (K-056): a b c ç d e f g ğ h ı i j k l m n o ö p r s ş t u ü v y z.
fn turkce_harf_sirasi(k: char) -> (u8, u32) {
    const ALFABE: &str = "abcçdefgğhıijklmnoöprsştuüvyz";
    let kucuk = match k {
        'İ' => 'i',
        'I' => 'ı',
        _ => k.to_lowercase().next().unwrap_or(k),
    };
    match ALFABE.chars().position(|a| a == kucuk) {
        Some(sira) => (0, sira as u32),
        None => (1, k as u32),
    }
}

/// İki metni Türk alfabesine göre karşılaştırır (K-056, TANIMLI).
fn turkce_karsilastir(a: &str, b: &str) -> std::cmp::Ordering {
    a.chars()
        .map(turkce_harf_sirasi)
        .cmp(b.chars().map(turkce_harf_sirasi))
}

/// Sıralama anahtarı: sayılar sayısal, metinler Türk alfabesiyle.
fn deger_sirasi(a: &Deger, b: &Deger) -> std::cmp::Ordering {
    match (a, b) {
        (Deger::TamSayi(x), Deger::TamSayi(y)) => x.cmp(y),
        (Deger::Ondalik(_), _) | (_, Deger::Ondalik(_)) => {
            match (sayisal_ac(a), sayisal_ac(b)) {
                (Some(sol), Some(sag)) => sol.karsilastir(&sag),
                _ => std::cmp::Ordering::Equal,
            }
        }
        (Deger::Metin(x), Deger::Metin(y)) => turkce_karsilastir(x, y),
        _ => std::cmp::Ordering::Equal,
    }
}

/// Üyelik eşitliği (K-058): sayı/metin/mantıksal değerler.
fn degerler_esit(a: &Deger, b: &Deger) -> bool {
    match (a, b) {
        (Deger::TamSayi(x), Deger::TamSayi(y)) => x == y,
        (Deger::Metin(x), Deger::Metin(y)) => x == y,
        (Deger::Mantiksal(x), Deger::Mantiksal(y)) => x == y,
        (Deger::Ondalik(_), _) | (_, Deger::Ondalik(_)) => match (sayisal_ac(a), sayisal_ac(b)) {
            (Some(sol), Some(sag)) => sol.karsilastir(&sag).is_eq(),
            _ => false,
        },
        _ => false,
    }
}

/// Satır sözlükleri listesini CSV metnine çevirir (K-058): başlıklar ilk
/// satırın anahtar sırasından; virgül/tırnak/yeni satır RFC 4180 gibi kaçar.
fn csv_yaz(satirlar: &[Deger]) -> String {
    let mut cikti = String::new();
    let Some(Deger::Sozluk(ilk)) = satirlar.first() else {
        return cikti;
    };
    let basliklar: Vec<&String> = ilk.iter().map(|(a, _)| a).collect();
    let hucre = |m: &str| -> String {
        if m.contains(',') || m.contains('"') || m.contains('\n') {
            format!("\"{}\"", m.replace('"', "\"\""))
        } else {
            m.to_string()
        }
    };
    cikti.push_str(&basliklar.iter().map(|b| hucre(b)).collect::<Vec<_>>().join(","));
    cikti.push('\n');
    for satir in satirlar {
        if let Deger::Sozluk(girdiler) = satir {
            let hucreler: Vec<String> = basliklar
                .iter()
                .map(|b| {
                    girdiler
                        .iter()
                        .find(|(a, _)| a == *b)
                        .map(|(_, d)| hucre(&d.metne()))
                        .unwrap_or_default()
                })
                .collect();
            cikti.push_str(&hucreler.join(","));
            cikti.push('\n');
        }
    }
    cikti
}

/// Değeri JSON metnine serileştirir (K-054): Sözlük, Liste, Metin, sayılar,
/// Mantıksal. Sözlük anahtar sırası korunur (determinizm).
fn json_yaz(deger: &Deger) -> String {
    match deger {
        Deger::TamSayi(s) => s.to_string(),
        Deger::Ondalik(ondalik) => ondalik.json_metni(),
        Deger::Mantiksal(b) => if *b { "true".into() } else { "false".into() },
        Deger::Metin(m) => json_metin_kacir(m),
        Deger::Liste(ogeler) => format!(
            "[{}]",
            ogeler.iter().map(json_yaz).collect::<Vec<_>>().join(",")
        ),
        Deger::Yapi(alanlar) => format!(
            "{{{}}}",
            alanlar
                .iter()
                .map(|(a, d)| format!("{}:{}", json_metin_kacir(a), json_yaz(d)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Deger::Sozluk(girdiler) => format!(
            "{{{}}}",
            girdiler
                .iter()
                .map(|(a, d)| format!("{}:{}", json_metin_kacir(a), json_yaz(d)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Deger::Hata(hata) => {
            let neden = hata
                .neden
                .as_deref()
                .map(|neden| json_yaz(&Deger::Hata(Box::new(neden.clone()))))
                .unwrap_or_else(|| "null".into());
            let veri = json_yaz(&Deger::Sozluk(hata.veri.clone()));
            format!(
                "{{\"kod\":{},\"mesaj\":{},\"neden\":{},\"veri\":{}}}",
                json_metin_kacir(&hata.kod),
                json_metin_kacir(&hata.mesaj),
                neden,
                veri
            )
        }
        baska => json_metin_kacir(&baska.metne()),
    }
}

fn json_metin_kacir(m: &str) -> String {
    let mut cikti = String::with_capacity(m.len() + 2);
    cikti.push('"');
    for k in m.chars() {
        match k {
            '"' => cikti.push_str("\\\""),
            '\\' => cikti.push_str("\\\\"),
            '\n' => cikti.push_str("\\n"),
            '\r' => cikti.push_str("\\r"),
            '\t' => cikti.push_str("\\t"),
            k if (k as u32) < 0x20 => cikti.push_str(&format!("\\u{:04x}", k as u32)),
            k => cikti.push(k),
        }
    }
    cikti.push('"');
    cikti
}

pub trait GirdiCikti {
    fn yazdir(&mut self, satir: String);
    /// İstem gösterilir, bir satır cevap beklenir. `None` = girdi tükendi.
    fn sor(&mut self, istem: &str) -> Option<String>;
    /// [alt, ust] aralığında (uçlar dahil) rastgele sayı.
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64;
    /// Dosya içeriğini okur; hata durumunda Türkçe hata metni döner.
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String>;
    /// Bir satırı dosyaya yazar (ekleme=false: baştan yaz; true: sona ekle).
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String>;
    /// Şimdiki zaman: (yıl, ay, gün, saat, dakika). v0'da UTC.
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32);
    /// Komut satırı argümanları (programa aktarılanlar).
    fn argumanlar(&mut self) -> Vec<String>;
    /// HTTP GET: (durum kodu, gövde). Son tarih içindeyse kalan süre verilir.
    /// GercekIo v0 yalnız http:// destekler.
    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String>;
    /// Sunucu dinlemesini kurar (golden 25).
    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String>;
    /// Sıradaki isteğin yolunu verir; None = sunucu kapanıyor.
    fn istek_al(&mut self) -> Option<String>;
    /// Son isteğe yanıt gönderir.
    fn yanit_gonder(&mut self, yanit: &str);
    /// Protokol hataları için açık HTTP durumlu yanıt. Eski IO adaptörleri
    /// gövdeyi yine de gösterebilsin diye güvenli bir varsayılanı vardır.
    fn durum_yaniti_gonder(&mut self, _durum: u16, yanit: &str) {
        self.yanit_gonder(yanit);
    }
    /// 303 yönlendirmesi gönderir (K-051).
    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String>;
    /// Sonraki yanıta Set-Cookie iliştirir (K-052).
    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String>;
    /// Sonraki yanıtla çerezi tarayıcıdan siler (Max-Age=0, K-073).
    fn cerez_sil(&mut self, ad: &str) -> Result<(), String>;
    /// Rota önsözünün kimlik/yetki ve unsafe-method CSRF kapısı (K-088).
    fn rota_guvenligini_denetle(
        &mut self,
        _erisim: &RotaErisimi,
        _csrf: Option<&str>,
        _csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        Err(WebReddi {
            durum: 503,
            mesaj: "IO adaptörü web güvenlik profilini desteklemiyor",
        })
    }
    fn csrf_belirteci(&mut self) -> Result<String, String> {
        Err("IO adaptörü CSRF oturumu desteklemiyor".into())
    }
    fn oturum_ac(&mut self, _kullanici: &str, _rol: &str) -> Result<(), String> {
        Err("IO adaptörü güvenli oturum desteklemiyor".into())
    }
    fn oturum_kapat(&mut self) -> Result<(), String> {
        Err("IO adaptörü güvenli oturum desteklemiyor".into())
    }
    fn parola_dogrula(&mut self, _parola: &str, _ozet: &str) -> bool {
        false
    }
    /// Her `eylem` çağrısı bir transaction/savepoint sınırıdır. Adaptör,
    /// desteklediği kalıcı kaynakları başarıda tamamlar, hata dönüşünde geri alır.
    fn eylem_baslat(&mut self) -> Result<(), String> {
        Err("IO adaptörü eylem transaction'ını desteklemiyor".into())
    }
    fn eylem_tamamla(&mut self) -> Result<(), String> {
        Err("IO adaptörü eylem transaction'ını desteklemiyor".into())
    }
    fn eylem_geri_al(&mut self) -> Result<(), String> {
        Err("IO adaptörü eylem transaction'ını desteklemiyor".into())
    }
    /// Sensör durumu (IoT simülatörü): "kapı" açık mı?
    fn sensor_acik_mi(&mut self, ad: &str) -> bool;
    /// Işık eyleyicisi (IoT simülatörü).
    fn isik_ayarla(&mut self, ad: &str, yansin: bool);
    /// Bekleme (GercekIo gerçekten uyur; testlerde sessiz).
    fn bekle_ms(&mut self, milisaniye: i64);
    /// Tekdüze artan an ölçümü (zaman aşımı hesabı, milisaniye).
    fn an_ms(&mut self) -> i64;
}

/// Eşzamanlı görev geleceklerinin aynı gerçek IO'yu sırayla kullanmasını sağlar.
/// `RefCell` yalnız tek iş parçacıklı scheduler içinde ödünç denetimidir; aynı
/// anda yalnız poll edilen görev IO'ya erişebilir.
#[derive(Clone)]
struct PaylasilanIo<'a> {
    ic: Rc<RefCell<&'a mut dyn GirdiCikti>>,
}

impl GirdiCikti for PaylasilanIo<'_> {
    fn yazdir(&mut self, satir: String) {
        self.ic.borrow_mut().yazdir(satir);
    }
    fn sor(&mut self, istem: &str) -> Option<String> {
        self.ic.borrow_mut().sor(istem)
    }
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        self.ic.borrow_mut().rastgele(alt, ust)
    }
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        self.ic.borrow_mut().dosya_oku(yol)
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        self.ic.borrow_mut().dosya_yaz(yol, satir, ekleme)
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.ic.borrow_mut().simdi()
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.ic.borrow_mut().argumanlar()
    }
    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        self.ic.borrow_mut().http_getir(url, zaman_asimi_ms)
    }
    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String> {
        self.ic.borrow_mut().sunucu_kur(kapi)
    }
    fn istek_al(&mut self) -> Option<String> {
        self.ic.borrow_mut().istek_al()
    }
    fn yanit_gonder(&mut self, yanit: &str) {
        self.ic.borrow_mut().yanit_gonder(yanit);
    }
    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        self.ic.borrow_mut().durum_yaniti_gonder(durum, yanit);
    }
    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        self.ic.borrow_mut().yonlendir_gonder(adres)
    }
    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        self.ic.borrow_mut().cerez_yaz(ad, deger)
    }
    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        self.ic.borrow_mut().cerez_sil(ad)
    }
    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        self.ic
            .borrow_mut()
            .rota_guvenligini_denetle(erisim, csrf, csrf_gerekli)
    }
    fn csrf_belirteci(&mut self) -> Result<String, String> {
        self.ic.borrow_mut().csrf_belirteci()
    }
    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        self.ic.borrow_mut().oturum_ac(kullanici, rol)
    }
    fn oturum_kapat(&mut self) -> Result<(), String> {
        self.ic.borrow_mut().oturum_kapat()
    }
    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        self.ic.borrow_mut().parola_dogrula(parola, ozet)
    }
    fn eylem_baslat(&mut self) -> Result<(), String> {
        self.ic.borrow_mut().eylem_baslat()
    }
    fn eylem_tamamla(&mut self) -> Result<(), String> {
        self.ic.borrow_mut().eylem_tamamla()
    }
    fn eylem_geri_al(&mut self) -> Result<(), String> {
        self.ic.borrow_mut().eylem_geri_al()
    }
    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        self.ic.borrow_mut().sensor_acik_mi(ad)
    }
    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        self.ic.borrow_mut().isik_ayarla(ad, yansin);
    }
    fn bekle_ms(&mut self, milisaniye: i64) {
        self.ic.borrow_mut().bekle_ms(milisaniye);
    }
    fn an_ms(&mut self) -> i64 {
        self.ic.borrow_mut().an_ms()
    }
}

/// Çocuk modu sargısı (K-047, master plan bölüm 16): sarılan IO ne olursa
/// olsun ağ ve sunucu kapalıdır; dosya erişimi çalışma klasörüyle sınırlıdır
/// (mutlak yol ve ".." yasak). Diğer her şey içteki IO'ya aynen gider.
pub struct GuvenliIo<T: GirdiCikti> {
    pub ic: T,
}

impl<T: GirdiCikti> GuvenliIo<T> {
    pub fn yeni(ic: T) -> GuvenliIo<T> {
        GuvenliIo { ic }
    }

    fn yol_izinli(yol: &str) -> Result<(), String> {
        let mutlak = yol.starts_with('/')
            || yol.starts_with('\\')
            || yol.chars().nth(1) == Some(':');
        let ust_dizin = yol.split(['/', '\\']).any(|parca| parca == "..");
        if mutlak || ust_dizin {
            return Err(format!(
                "güvenli modda yalnız çalışma klasöründeki dosyalara erişilir; \"{}\" dışarıyı gösteriyor",
                yol
            ));
        }
        Ok(())
    }
}

impl<T: GirdiCikti> GirdiCikti for GuvenliIo<T> {
    fn yazdir(&mut self, satir: String) {
        self.ic.yazdir(satir);
    }
    fn sor(&mut self, istem: &str) -> Option<String> {
        self.ic.sor(istem)
    }
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        self.ic.rastgele(alt, ust)
    }
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        Self::yol_izinli(yol)?;
        self.ic.dosya_oku(yol)
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        Self::yol_izinli(yol)?;
        self.ic.dosya_yaz(yol, satir, ekleme)
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.ic.simdi()
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.ic.argumanlar()
    }
    fn http_getir(
        &mut self,
        _url: &str,
        _zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        Err("güvenli modda ağ erişimi kapalı".into())
    }
    fn sunucu_kur(&mut self, _kapi: i64) -> Result<(), String> {
        Err("güvenli modda sunucu açılamaz".into())
    }
    fn istek_al(&mut self) -> Option<String> {
        None
    }
    fn yanit_gonder(&mut self, _yanit: &str) {}
    fn durum_yaniti_gonder(&mut self, _durum: u16, _yanit: &str) {}
    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        if crate::web_guvenligi::yerel_yonlendirme_gecerli(adres) {
            Ok(())
        } else {
            Err("yönlendirme yalnız CR/LF içermeyen yerel `/...` adresine yapılabilir".into())
        }
    }
    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        if crate::web_guvenligi::cerez_adi_gecerli(ad)
            && crate::web_guvenligi::cerez_degeri_gecerli(deger)
        {
            Ok(())
        } else {
            Err("çerez adı/değeri HTTP başlığı için güvenli değil".into())
        }
    }
    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        if crate::web_guvenligi::cerez_adi_gecerli(ad) {
            Ok(())
        } else {
            Err("çerez adı HTTP başlığı için güvenli değil".into())
        }
    }
    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        self.ic.rota_guvenligini_denetle(erisim, csrf, csrf_gerekli)
    }
    fn csrf_belirteci(&mut self) -> Result<String, String> {
        self.ic.csrf_belirteci()
    }
    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        self.ic.oturum_ac(kullanici, rol)
    }
    fn oturum_kapat(&mut self) -> Result<(), String> {
        self.ic.oturum_kapat()
    }
    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        self.ic.parola_dogrula(parola, ozet)
    }
    fn eylem_baslat(&mut self) -> Result<(), String> {
        self.ic.eylem_baslat()
    }
    fn eylem_tamamla(&mut self) -> Result<(), String> {
        self.ic.eylem_tamamla()
    }
    fn eylem_geri_al(&mut self) -> Result<(), String> {
        self.ic.eylem_geri_al()
    }
    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        self.ic.sensor_acik_mi(ad)
    }
    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        self.ic.isik_ayarla(ad, yansin);
    }
    fn bekle_ms(&mut self, milisaniye: i64) {
        self.ic.bekle_ms(milisaniye);
    }
    fn an_ms(&mut self) -> i64 {
        self.ic.an_ms()
    }
}

/// Çıktıyı toplayan, girdiyi ve "rastgele" sayıları hazır kuyruktan veren IO
/// (testler ve lib arayüzü — determinizm burada da korunur).
pub struct ToplayanIo {
    pub girdiler: VecDeque<String>,
    pub rastgele_degerler: VecDeque<i64>,
    /// Sahte dosya sistemi: yol → içerik (testlerde determinizm).
    pub dosyalar: HashMap<String, String>,
    /// Sabit "şimdi" — testlerde determinizm.
    pub zaman: (i64, u32, u32, u32, u32),
    pub argumanlar: Vec<String>,
    /// Sahte HTTP: url → (durum, gövde).
    pub http_yanitlari: HashMap<String, (i64, String)>,
    /// Sahte sunucu: istek kuyruğu ve (istek → yanıt) kayıtları.
    pub istekler: VecDeque<String>,
    /// Sunucunun Set-Cookie ile yazdığı çerezler (K-052 testleri için).
    pub yazilan_cerezler: Vec<(String, String)>,
    /// K-088 güvenli çerez niteliklerinin hermetik kanıtı.
    pub guvenli_cerezler: Vec<GuvenliCerezKaydi>,
    pub sunucu_yanitlari: Vec<(String, String)>,
    /// Her sahte HTTP yanıtının durum kodu; istek alındığında 200 ile başlar.
    pub sunucu_durumlari: Vec<u16>,
    /// Sahte sensörler (varsayılan kapalı) ve an ölçümü kuyruğu.
    pub sensorler: HashMap<String, bool>,
    pub an_degerleri: VecDeque<i64>,
    an_son_degeri: i64,
    pub cikti: Vec<String>,
    eylem_yedekleri: Vec<HashMap<String, String>>,
    web_guvenligi: WebGuvenligi,
    guvenli_belirtec_sirasi: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuvenliCerezKaydi {
    pub ad: String,
    pub deger: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: &'static str,
    pub yol: &'static str,
    pub azami_omur_saniye: i64,
}

impl ToplayanIo {
    pub fn yeni(girdiler: Vec<String>) -> ToplayanIo {
        ToplayanIo {
            girdiler: girdiler.into(),
            rastgele_degerler: VecDeque::new(),
            dosyalar: HashMap::new(),
            zaman: (2026, 8, 31, 14, 30),
            argumanlar: Vec::new(),
            http_yanitlari: HashMap::new(),
            istekler: VecDeque::new(),
            yazilan_cerezler: Vec::new(),
            guvenli_cerezler: Vec::new(),
            sunucu_yanitlari: Vec::new(),
            sunucu_durumlari: Vec::new(),
            sensorler: HashMap::new(),
            an_degerleri: VecDeque::new(),
            an_son_degeri: 0,
            cikti: Vec::new(),
            eylem_yedekleri: Vec::new(),
            web_guvenligi: WebGuvenligi::yeni(),
            guvenli_belirtec_sirasi: 0,
        }
    }

    fn oturum_cerezini_yaz(&mut self, yeni: YeniOturum) {
        let ad = "__Host-zee-oturum".to_string();
        self.yazilan_cerezler
            .push((ad.clone(), yeni.belirtec.clone()));
        self.guvenli_cerezler.push(GuvenliCerezKaydi {
            ad,
            deger: yeni.belirtec,
            secure: true,
            http_only: true,
            same_site: "Lax",
            yol: "/",
            azami_omur_saniye: yeni.azami_omur_saniye,
        });
    }
}

impl GirdiCikti for ToplayanIo {
    fn yazdir(&mut self, satir: String) {
        self.cikti.push(satir);
    }
    fn sor(&mut self, istem: &str) -> Option<String> {
        self.cikti.push(istem.to_string());
        self.girdiler.pop_front()
    }
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        if alt > ust {
            return alt;
        }
        self.rastgele_degerler
            .pop_front()
            .unwrap_or(alt)
            .clamp(alt, ust)
    }
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        self.dosyalar
            .get(yol)
            .cloned()
            .ok_or_else(|| format!("\"{}\" dosyası bulunamadı", yol))
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        let girdi = self.dosyalar.entry(yol.to_string()).or_default();
        if !ekleme {
            girdi.clear();
        }
        girdi.push_str(satir);
        girdi.push('\n');
        Ok(())
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.zaman
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.argumanlar.clone()
    }
    fn http_getir(
        &mut self,
        url: &str,
        _zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        self.http_yanitlari
            .get(url)
            .cloned()
            .ok_or_else(|| format!("\"{}\" adresine bağlanılamadı", url))
    }
    fn sunucu_kur(&mut self, _kapi: i64) -> Result<(), String> {
        Ok(())
    }
    fn istek_al(&mut self) -> Option<String> {
        let ham = self.istekler.pop_front()?;
        let (_, yol, _) = istek_parcala(&ham);
        let (_, _, _, cerezler) = istek_parcala_cerezli(&ham);
        let belirtec = cerezler
            .iter()
            .find(|(ad, _)| ad == "__Host-zee-oturum" || ad == "zee-oturum")
            .map(|(_, deger)| deger.as_str());
        self.web_guvenligi
            .istegi_baslat(belirtec, self.an_son_degeri);
        self.sunucu_yanitlari.push((yol, String::new()));
        self.sunucu_durumlari.push(200);
        Some(ham)
    }
    fn yanit_gonder(&mut self, yanit: &str) {
        if let Some((_, bos)) = self.sunucu_yanitlari.last_mut() {
            *bos = yanit.to_string();
        }
    }
    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        if let Some(son) = self.sunucu_durumlari.last_mut() {
            *son = durum;
        }
        self.yanit_gonder(yanit);
    }
    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        if !crate::web_guvenligi::yerel_yonlendirme_gecerli(adres) {
            return Err("yönlendirme yalnız yerel `/...` adresine yapılabilir".into());
        }
        if let Some((_, bos)) = self.sunucu_yanitlari.last_mut() {
            *bos = format!("→ {}", adres);
        }
        Ok(())
    }
    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        if !crate::web_guvenligi::cerez_adi_gecerli(ad)
            || !crate::web_guvenligi::cerez_degeri_gecerli(deger)
        {
            return Err("çerez adı/değeri HTTP başlığı için güvenli değil".into());
        }
        self.yazilan_cerezler
            .push((ad.to_string(), deger.to_string()));
        Ok(())
    }
    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        if !crate::web_guvenligi::cerez_adi_gecerli(ad) {
            return Err("çerez adı HTTP başlığı için güvenli değil".into());
        }
        self.yazilan_cerezler
            .push((ad.to_string(), "×silindi".to_string()));
        Ok(())
    }
    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        self.web_guvenligi
            .denetle(erisim, csrf, csrf_gerekli, self.an_son_degeri)
    }
    fn csrf_belirteci(&mut self) -> Result<String, String> {
        let mut sira = self.guvenli_belirtec_sirasi;
        let (csrf, yeni) = self.web_guvenligi.csrf_belirteci(self.an_son_degeri, || {
            sira = sira.saturating_add(1);
            Ok(format!("{:064x}", sira))
        })?;
        self.guvenli_belirtec_sirasi = sira;
        if let Some(yeni) = yeni {
            self.oturum_cerezini_yaz(yeni);
        }
        Ok(csrf)
    }
    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        let mut sira = self.guvenli_belirtec_sirasi;
        let yeni = self.web_guvenligi.oturum_ac(
            kullanici.to_string(),
            rol.to_string(),
            self.an_son_degeri,
            || {
                sira = sira.saturating_add(1);
                Ok(format!("{:064x}", sira))
            },
        )?;
        self.guvenli_belirtec_sirasi = sira;
        self.oturum_cerezini_yaz(yeni);
        Ok(())
    }
    fn oturum_kapat(&mut self) -> Result<(), String> {
        self.web_guvenligi.oturum_kapat();
        let ad = "__Host-zee-oturum".to_string();
        self.yazilan_cerezler
            .push((ad.clone(), "×silindi".to_string()));
        self.guvenli_cerezler.push(GuvenliCerezKaydi {
            ad,
            deger: String::new(),
            secure: true,
            http_only: true,
            same_site: "Lax",
            yol: "/",
            azami_omur_saniye: 0,
        });
        Ok(())
    }
    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        crate::guvenlik::parola_dogrula(parola, ozet)
    }
    fn eylem_baslat(&mut self) -> Result<(), String> {
        self.eylem_yedekleri.push(self.dosyalar.clone());
        Ok(())
    }
    fn eylem_tamamla(&mut self) -> Result<(), String> {
        self.eylem_yedekleri
            .pop()
            .map(|_| ())
            .ok_or_else(|| "açık eylem transaction'ı yok".into())
    }
    fn eylem_geri_al(&mut self) -> Result<(), String> {
        self.dosyalar = self
            .eylem_yedekleri
            .pop()
            .ok_or_else(|| "açık eylem transaction'ı yok".to_string())?;
        Ok(())
    }
    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        self.sensorler.get(ad).copied().unwrap_or(false)
    }
    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        self.cikti.push(format!(
            "[ışık] {} {}",
            ad,
            if yansin { "yandı" } else { "söndü" }
        ));
    }
    fn bekle_ms(&mut self, milisaniye: i64) {
        self.an_son_degeri = self.an_son_degeri.saturating_add(milisaniye.max(0));
    }
    fn an_ms(&mut self) -> i64 {
        if let Some(an) = self.an_degerleri.pop_front() {
            self.an_son_degeri = self.an_son_degeri.max(an);
        }
        self.an_son_degeri
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HataDegeri {
    pub kod: String,
    pub mesaj: String,
    pub neden: Option<Box<HataDegeri>>,
    pub veri: Vec<(String, Deger)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Deger {
    TamSayi(i64),
    /// Onluk tam değer: keyfî uzunlukta katsayı/ölçek; hep normalize saklanır.
    /// Box, büyük çekirdeğin çalışma zamanı değer çerçevesini şişirmesini önler.
    Ondalik(Box<Ondalik>),
    Metin(String),
    Mantiksal(bool),
    Liste(Vec<Deger>),
    /// Ekleme sırası korunur (deterministik gezinme).
    Sozluk(Vec<(String, Deger)>),
    /// Seçenek'in boş hali; dolu hali değerin kendisidir.
    Yok,
    /// Sonuç: başarılıysa değer, değilse yapılandırılmış Hata taşır.
    Sonuc { basarili: bool, icerik: Box<Deger> },
    /// Kod, insana dönük mesaj, isteğe bağlı neden zinciri ve bağlam verisi.
    Hata(Box<HataDegeri>),
    /// Yapı örneği: yalın alan adı → değer (tanım sırasıyla).
    Yapi(Vec<(String, Deger)>),
    Tarih { yil: i64, ay: u32, gun: u32 },
    Saat { saat: u32, dakika: u32 },
    /// Milisaniye cinsinden süre.
    Sure { milisaniye: i64 },
    /// HTTP yanıtı: durum kodu + gövde.
    AgYaniti { durum: i64, govde: String },
}

const AY_ADLARI: [&str; 12] = [
    "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
    "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık",
];

impl Deger {
    fn hata(kod: impl Into<String>, mesaj: impl Into<String>) -> Self {
        Self::Hata(Box::new(HataDegeri {
            kod: kod.into(),
            mesaj: mesaj.into(),
            neden: None,
            veri: Vec::new(),
        }))
    }

    fn metne(&self) -> String {
        match self {
            Deger::TamSayi(s) => s.to_string(),
            Deger::Ondalik(ondalik) => ondalik.metne(),
            Deger::Metin(m) => m.clone(),
            Deger::Mantiksal(b) => if *b { "doğru" } else { "yanlış" }.to_string(),
            Deger::Liste(ogeler) => ogeler
                .iter()
                .map(|o| o.metne())
                .collect::<Vec<_>>()
                .join(", "),
            Deger::Sozluk(girdiler) => girdiler
                .iter()
                .map(|(anahtar, deger)| format!("{}: {}", anahtar, deger.metne()))
                .collect::<Vec<_>>()
                .join(", "),
            Deger::Yok => "yok".to_string(),
            Deger::Yapi(alanlar) => alanlar
                .iter()
                .map(|(alan, deger)| format!("{}: {}", alan, deger.metne()))
                .collect::<Vec<_>>()
                .join(", "),
            Deger::Sonuc { basarili, icerik } => {
                if *basarili {
                    icerik.metne()
                } else {
                    format!("hata: {}", icerik.metne())
                }
            }
            // Geriye uyum: `sonucun hatası yaz` eskisi gibi yalnız anlaşılır
            // mesajı gösterir; kod/veri açık özelliklerle alınır (K-091).
            Deger::Hata(hata) => hata.mesaj.clone(),
            Deger::Tarih { yil, ay, gun } => {
                format!("{} {} {}", gun, AY_ADLARI[(*ay as usize).saturating_sub(1) % 12], yil)
            }
            Deger::Saat { saat, dakika } => format!("{:02}:{:02}", saat, dakika),
            Deger::AgYaniti { durum, govde } => format!("[{}] {}", durum, govde),
            Deger::Sure { milisaniye } => {
                let ms = *milisaniye;
                if ms % 3_600_000 == 0 {
                    format!("{} saat", ms / 3_600_000)
                } else if ms % 60_000 == 0 {
                    format!("{} dakika", ms / 60_000)
                } else if ms % 1000 == 0 {
                    format!("{} saniye", ms / 1000)
                } else {
                    // Küsuratlı: saniye cinsinden ondalık basım (1500 → "1,5 saniye").
                    let ondalik = Ondalik::katsayidan(ms, 3);
                    format!("{} saniye", ondalik.metne())
                }
            }
        }
    }
}

/// CLI'nin sistem saatini çevirmesi için dışa açık sarmalayıcı.
pub fn gunlerden_tarih_utc(gunler: i64) -> (i64, u32, u32) {
    gunlerden_tarih(gunler)
}

/// Gregoryen tarih ↔ gün sayısı (Howard Hinnant'ın algoritmaları; 1970-01-01 = 0).
fn gunlerden_tarih(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let devir = if z >= 0 { z } else { z - 146096 } / 146097;
    let devir_gunu = (z - devir * 146097) as u64;
    let yil_gunu = (devir_gunu - devir_gunu / 1460 + devir_gunu / 36524 - devir_gunu / 146096) / 365;
    let yil = yil_gunu as i64 + devir * 400;
    let yilin_gunu = devir_gunu - (365 * yil_gunu + yil_gunu / 4 - yil_gunu / 100);
    let ay_kaba = (5 * yilin_gunu + 2) / 153;
    let gun = (yilin_gunu - (153 * ay_kaba + 2) / 5 + 1) as u32;
    let ay = if ay_kaba < 10 { ay_kaba + 3 } else { ay_kaba - 9 } as u32;
    (if ay <= 2 { yil + 1 } else { yil }, ay, gun)
}

fn tarihten_gunler(yil: i64, ay: u32, gun: u32) -> i64 {
    let yil = if ay <= 2 { yil - 1 } else { yil };
    let devir = if yil >= 0 { yil } else { yil - 399 } / 400;
    let devir_yili = (yil - devir * 400) as u64;
    let yilin_gunu =
        (153 * (if ay > 2 { ay - 3 } else { ay + 9 }) as u64 + 2) / 5 + gun as u64 - 1;
    let devir_gunu = devir_yili * 365 + devir_yili / 4 - devir_yili / 100 + yilin_gunu;
    devir * 146097 + devir_gunu as i64 - 719468
}

fn ondalik_degeri(ondalik: Ondalik) -> Deger {
    Deger::Ondalik(Box::new(ondalik))
}

fn tasma(satir: usize) -> Tani {
    Tani::yeni("C002", "İşlem sonucu sayı sınırını aştı.".into(), satir, 1, 1)
}

/// Sayısal değeri kayıpsız ortak Ondalık çekirdeğine açar.
fn sayisal_ac(deger: &Deger) -> Option<Ondalik> {
    match deger {
        Deger::TamSayi(v) => Some(Ondalik::tam(*v)),
        Deger::Ondalik(ondalik) => Some((**ondalik).clone()),
        _ => None,
    }
}

/// Türkçe kurallarla büyük harfe çevirme: i→İ, ı→I (A07 anti-örneğindeki tuzak).
fn turkce_buyuk(metin: &str) -> String {
    metin
        .chars()
        .flat_map(|k| match k {
            'i' => vec!['İ'],
            'ı' => vec!['I'],
            _ => k.to_uppercase().collect(),
        })
        .collect()
}

/// Türkçe kurallarla küçük harfe çevirme: İ→i, I→ı.
fn turkce_kucuk(metin: &str) -> String {
    metin
        .chars()
        .flat_map(|k| match k {
            'İ' => vec!['i'],
            'I' => vec!['ı'],
            _ => k.to_lowercase().collect(),
        })
        .collect()
}

pub fn calistir(program: &Program) -> Result<Vec<String>, Tani> {
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(program, &mut io)?;
    Ok(io.cikti)
}

pub fn calistir_io(program: &Program, io: &mut dyn GirdiCikti) -> Result<(), Tani> {
    calistir_io_kodla(program, io).map(|_| ())
}

/// calistir_io + çıkış kodu (K-069): "programı 1 ile bitir" → Ok(1);
/// olağan bitiş → Ok(0). CLI süreç çıkış kodunu buradan alır.
pub fn calistir_io_kodla(program: &Program, io: &mut dyn GirdiCikti) -> Result<i64, Tani> {
    calistir_program_kodla(CalistirmaProgrami::Ham(program), io)
}

fn calistir_program_kodla(
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
) -> Result<i64, Tani> {
    let kodu = |tani: &Tani| tani.mesaj.parse::<i64>().unwrap_or(0);
    let mut ortam: HashMap<String, Deger> = HashMap::new();
    match blok_calistir(&program.program().cumleler, &mut ortam, program, io, 0) {
        // "programı bitir" olağan bir sonlanmadır (Ç000 iç nöbetçisi).
        Err(tani) if tani.kod == "Ç000" => return Ok(kodu(&tani)),
        Err(tani) => return Err(tani),
        Ok(_) => {}
    }

    // Sunucu kurulduysa dinlemeye geç (golden 25): kayıtlı "geldiğinde"
    // gövdeleri istek başına taze ortamda koşulur.
    if ortam.contains_key("(sunucu)") {
        while let Some(ham) = io.istek_al() {
            if let Err((durum, mesaj)) = istek_sinirlarini_denetle(&ham) {
                io.durum_yaniti_gonder(durum, mesaj);
                continue;
            }
            let (gelen_yontem, yol, veriler, cerezler) = istek_parcala_cerezli(&ham);
            let istek_sozlugu = Deger::Sozluk(
                veriler
                    .iter()
                    .cloned()
                    .map(|(a, d)| (a, Deger::Metin(d)))
                    .collect(),
            );
            let cerez_sozlugu = Deger::Sozluk(
                cerezler.into_iter().map(|(a, d)| (a, Deger::Metin(d))).collect(),
            );
            let mut eslesti = false;
            let mut yol_eslesti = false;
            for cumle in &program.program().cumleler {
                if let Cumle::IstekGeldiginde {
                    yontem,
                    yol: kayitli,
                    onekli,
                    govde,
                    satir,
                } = cumle
                {
                    let mut bos_ortam: HashMap<String, Deger> = HashMap::new();
                    let kayitli =
                        degerlendir(kayitli, &bos_ortam, program, io, 0, *satir)?.metne();
                    let uydu = if *onekli { yol.starts_with(&kayitli) } else { kayitli == yol };
                    if uydu {
                        yol_eslesti = true;
                    }
                    let beklenen = yontem.unwrap_or(HttpYontemi::Get).yazimi();
                    if uydu && beklenen == gelen_yontem {
                        let erisim = match govde.first() {
                            Some(Cumle::RotaPolitikasi { erisim, .. }) => erisim.clone(),
                            _ => RotaErisimi::HerkeseAcik,
                        };
                        let csrf = veriler
                            .iter()
                            .find(|(ad, _)| ad == "_csrf")
                            .map(|(_, deger)| deger.as_str());
                        if let Err(red) = io.rota_guvenligini_denetle(
                            &erisim,
                            csrf,
                            !yontem.unwrap_or(HttpYontemi::Get).guvenli(),
                        ) {
                            io.durum_yaniti_gonder(red.durum, red.mesaj);
                            eslesti = true;
                            break;
                        }
                        let eksik_alan = govde.iter().find_map(|cumle| match cumle {
                            Cumle::RotaAlaniGerekli { ad, .. }
                                if !veriler.iter().any(|(gelen, deger)| {
                                    gelen == ad && !deger.trim().is_empty()
                                }) =>
                            {
                                Some(ad.as_str())
                            }
                            Cumle::RotaPolitikasi { .. } | Cumle::RotaAlaniGerekli { .. } => None,
                            _ => None,
                        });
                        if let Some(ad) = eksik_alan {
                            io.durum_yaniti_gonder(
                                400,
                                &format!("zorunlu istek alanı eksik ya da boş: {}", ad),
                            );
                            eslesti = true;
                            break;
                        }
                        bos_ortam.insert("istek".into(), istek_sozlugu.clone());
                        bos_ortam.insert("çerezler".into(), cerez_sozlugu.clone());
                        // Her istek K-085'in işbirlikli iptal çekirdeğinde 30 saniyelik
                        // varsayılan bütçe taşır. Daha kısa iç son tarih yine kazanır.
                        let nobetci = SonTarihNobetcisi::yeni(io.an_ms().saturating_add(30_000));
                        let sonuc = blok_calistir(govde, &mut bos_ortam, program, io, 0);
                        drop(nobetci);
                        match sonuc {
                            Err(tani) if tani.kod == "Ç000" => return Ok(kodu(&tani)),
                            Err(tani) if tani.kod == "Ç001" => {
                                io.durum_yaniti_gonder(504, "istek 30 saniyelik son tarihini aştı");
                            }
                            Err(tani) => return Err(tani),
                            Ok(_) => {}
                        }
                        eslesti = true;
                        break;
                    }
                }
            }
            if !eslesti {
                if yol_eslesti {
                    io.durum_yaniti_gonder(405, "bu adres istenen HTTP yöntemini kabul etmiyor");
                } else {
                    io.durum_yaniti_gonder(404, &format!("aranan sayfa yok: {}", yol));
                }
            }
        }
    }
    Ok(0)
}

/// Tek bir testi taze ortamda koşar; ilk doğrulama/çalışma hatasında durur.
pub fn test_calistir(
    program: &Program,
    test: &crate::agac::Test,
    io: &mut dyn GirdiCikti,
) -> Result<(), Tani> {
    let mut ortam: HashMap<String, Deger> = HashMap::new();
    match blok_calistir(
        &test.govde,
        &mut ortam,
        CalistirmaProgrami::Ham(program),
        io,
        0,
    ) {
        Err(tani) if tani.kod == "Ç000" => Ok(()),
        sonuc => sonuc.map(|_| ()),
    }
}

/// Blok çalıştırmanın sonucu: normal akış mı, "döndür" ile erken çıkış mı.
pub enum Akis {
    Devam,
    Don(Deger),
}

#[derive(Clone, Copy)]
struct SonTarih {
    kimlik: u64,
    an_ms: i64,
}

thread_local! {
    static SON_TARIHLER: std::cell::RefCell<Vec<SonTarih>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// Yalnız scheduler bir görev future'ını poll ederken doludur. `bekle`
    /// bu kanala süreyi bırakıp Pending döner; scheduler zamanı ilerletir.
    static GOREV_BEKLEME_SINYALI: RefCell<Option<Rc<Cell<Option<i64>>>>> =
        const { RefCell::new(None) };
}

static SON_TARIH_KIMLIGI: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1);

struct SonTarihNobetcisi {
    kimlik: u64,
}

impl SonTarihNobetcisi {
    fn yeni(an_ms: i64) -> Self {
        let kimlik = SON_TARIH_KIMLIGI.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        SON_TARIHLER.with(|son_tarihler| {
            son_tarihler.borrow_mut().push(SonTarih { kimlik, an_ms });
        });
        Self { kimlik }
    }
}

impl Drop for SonTarihNobetcisi {
    fn drop(&mut self) {
        SON_TARIHLER.with(|son_tarihler| {
            let mut son_tarihler = son_tarihler.borrow_mut();
            if let Some(yer) = son_tarihler.iter().rposition(|son| son.kimlik == self.kimlik) {
                son_tarihler.remove(yer);
            }
        });
    }
}

fn etkin_son_tarih() -> Option<SonTarih> {
    SON_TARIHLER.with(|son_tarihler| {
        son_tarihler
            .borrow()
            .iter()
            .min_by_key(|son| son.an_ms)
            .copied()
    })
}

fn son_tarih_tanisi(son: SonTarih, satir: usize) -> Tani {
    // Ç001 yalnız runtime içi iptal nöbetçisidir; onu kendi `içinde` bloğu
    // yakalar. Kullanıcıya kataloglu bir hata olarak sızmamalıdır.
    Tani::yeni("Ç001", son.kimlik.to_string(), satir, 1, 1)
}

fn son_tarih_kalani(
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Option<(SonTarih, i64)>, Tani> {
    let Some(son) = etkin_son_tarih() else {
        return Ok(None);
    };
    let kalan = son.an_ms.saturating_sub(io.an_ms());
    if kalan <= 0 {
        Err(son_tarih_tanisi(son, satir))
    } else {
        Ok(Some((son, kalan)))
    }
}

fn son_tarihi_denetle(io: &mut dyn GirdiCikti, satir: usize) -> Result<(), Tani> {
    son_tarih_kalani(io, satir).map(|_| ())
}

fn bu_son_tarihin_iptali(tani: &Tani, kimlik: u64) -> bool {
    tani.kod == "Ç001" && tani.mesaj.parse::<u64>() == Ok(kimlik)
}

struct BosUyandirici;

impl Wake for BosUyandirici {
    fn wake(self: Arc<Self>) {}
}

fn bos_uyandirici() -> Waker {
    Waker::from(Arc::new(BosUyandirici))
}

/// Üst düzey yorumlama Pending üretmez; Pending yalnız görev scheduler'ının
/// sahip olduğu future'larda anlamlıdır.
fn hazir_calistir<T>(
    mut gelecek: Pin<Box<dyn Future<Output = Result<T, Tani>> + '_>>,
) -> Result<T, Tani> {
    let uyandirici = bos_uyandirici();
    let mut baglam = Context::from_waker(&uyandirici);
    match gelecek.as_mut().poll(&mut baglam) {
        Poll::Ready(sonuc) => sonuc,
        Poll::Pending => Err(ic_hata(1)),
    }
}

struct GorevBeklemeNoktasi {
    milisaniye: i64,
    sinyal_verildi: bool,
}

impl Future for GorevBeklemeNoktasi {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _baglam: &mut Context<'_>) -> Poll<Self::Output> {
        if self.sinyal_verildi {
            return Poll::Ready(());
        }
        let bulundu = GOREV_BEKLEME_SINYALI.with(|yuva| {
            let yuva = yuva.borrow();
            if let Some(sinyal) = yuva.as_ref() {
                sinyal.set(Some(self.milisaniye.max(0)));
                true
            } else {
                false
            }
        });
        if bulundu {
            self.sinyal_verildi = true;
            Poll::Pending
        } else {
            // Bu future yalnız görev içinde kurulmalıdır.
            Poll::Ready(())
        }
    }
}

fn gorevde_miyiz() -> bool {
    GOREV_BEKLEME_SINYALI.with(|yuva| yuva.borrow().is_some())
}

async fn gorev_bekleme_noktasi(milisaniye: i64) {
    GorevBeklemeNoktasi { milisaniye, sinyal_verildi: false }.await;
}

struct BekleyenGorev<'a> {
    ad: String,
    ifade: &'a Ifade,
    satir: usize,
    ortam: HashMap<String, Deger>,
}

type GorevGelecegi<'a> = Pin<Box<dyn Future<Output = Result<Deger, Tani>> + 'a>>;

struct GorevCalismasi<'a> {
    ad: String,
    satir: usize,
    gelecek: Option<GorevGelecegi<'a>>,
    sinyal: Rc<Cell<Option<i64>>>,
    uyanma_ani: Option<i64>,
    son_tarihler: Vec<SonTarih>,
    sonuc: Option<Deger>,
}

/// Kaynak sırasını bağlayıcı zamanlama sırası yapar. Her turda hazır görevler
/// birer kez poll edilir; hiçbiri hazır değilse saat en yakın beklemeye kadar
/// tek adımda ilerler. Aynı anda yalnız bir görev çalıştığı için data race yoktur.
async fn gorevleri_calistir<'a>(
    gorevler: Vec<BekleyenGorev<'a>>,
    program: CalistirmaProgrami<'a>,
    io: &'a mut dyn GirdiCikti,
    derinlik: usize,
) -> Result<Vec<(String, Deger)>, Tani> {
    let ortak = Rc::new(RefCell::new(io));
    let ana_son_tarihler = SON_TARIHLER.with(|yuva| std::mem::take(&mut *yuva.borrow_mut()));
    let ana_sinyal = GOREV_BEKLEME_SINYALI.with(|yuva| yuva.borrow_mut().take());

    let mut calismalar = gorevler
        .into_iter()
        .map(|gorev| {
            let mut gorev_io = PaylasilanIo { ic: Rc::clone(&ortak) };
            let ifade = gorev.ifade;
            let ortam = gorev.ortam;
            let satir = gorev.satir;
            let gelecek = Box::pin(async move {
                degerlendir_async(
                    ifade,
                    &ortam,
                    program,
                    &mut gorev_io,
                    derinlik,
                    satir,
                )
                .await
            });
            GorevCalismasi {
                ad: gorev.ad,
                satir,
                gelecek: Some(gelecek),
                sinyal: Rc::new(Cell::new(None)),
                uyanma_ani: None,
                son_tarihler: ana_son_tarihler.clone(),
                sonuc: None,
            }
        })
        .collect::<Vec<_>>();

    let uyandirici = bos_uyandirici();
    let mut poll_baglami = Context::from_waker(&uyandirici);
    let mut simdi = ortak.borrow_mut().an_ms();
    let sonuc = (async {
        loop {
            let mut ilerledi = false;
            for sira in 0..calismalar.len() {
                let hazir = calismalar[sira].gelecek.is_some()
                    && calismalar[sira].uyanma_ani.is_none_or(|an| an <= simdi);
                if !hazir {
                    continue;
                }
                ilerledi = true;
                calismalar[sira].uyanma_ani = None;
                calismalar[sira].sinyal.set(None);

                SON_TARIHLER.with(|yuva| {
                    *yuva.borrow_mut() = std::mem::take(&mut calismalar[sira].son_tarihler);
                });
                GOREV_BEKLEME_SINYALI.with(|yuva| {
                    *yuva.borrow_mut() = Some(Rc::clone(&calismalar[sira].sinyal));
                });
                let poll = match calismalar[sira].gelecek.as_mut() {
                    Some(gelecek) => gelecek.as_mut().poll(&mut poll_baglami),
                    None => return Err(ic_hata(calismalar[sira].satir)),
                };
                GOREV_BEKLEME_SINYALI.with(|yuva| {
                    *yuva.borrow_mut() = ana_sinyal.clone();
                });
                SON_TARIHLER.with(|yuva| {
                    calismalar[sira].son_tarihler = std::mem::take(&mut *yuva.borrow_mut());
                });
                // Atomik bir host çağrısı ya da iç görev grubu saati ilerletmiş
                // olabilir; kardeşlerin uyanma anları gerçek tekdüze saatle kalır.
                simdi = simdi.max(ortak.borrow_mut().an_ms());

                match poll {
                    Poll::Ready(Ok(deger)) => {
                        calismalar[sira].gelecek = None;
                        calismalar[sira].sonuc = Some(deger);
                    }
                    Poll::Ready(Err(mut tani)) => {
                        let iptal_edilenler = calismalar
                            .iter()
                            .enumerate()
                            .filter(|(i, g)| *i != sira && g.gelecek.is_some())
                            .map(|(_, g)| g.ad.as_str())
                            .collect::<Vec<_>>();
                        let iptal = if iptal_edilenler.is_empty() {
                            String::new()
                        } else {
                            format!(
                                " Kardeş görevler iptal edildi: {}.",
                                iptal_edilenler.join(", ")
                            )
                        };
                        // Ç000 çıkış kodunu, Ç001 deadline sahibini mesajında
                        // taşır; iç akış nöbetçilerini metinsel olarak sarmalama.
                        if tani.kod != "Ç000" && tani.kod != "Ç001" {
                            tani.mesaj = format!(
                                "\"{}\" görevi başarısız oldu.{} {}",
                                calismalar[sira].ad, iptal, tani.mesaj
                            );
                        }
                        return Err(tani);
                    }
                    Poll::Pending => {
                        let sure = calismalar[sira].sinyal.take().ok_or_else(|| {
                            Tani::yeni(
                                "C000",
                                format!(
                                    "\"{}\" görevi bekleme nedeni bildirmeden durdu.",
                                    calismalar[sira].ad
                                ),
                                calismalar[sira].satir,
                                1,
                                1,
                            )
                        })?;
                        calismalar[sira].uyanma_ani = Some(simdi.saturating_add(sure));
                    }
                }
            }

            if calismalar.iter().all(|g| g.gelecek.is_none()) {
                let mut sonuclar = Vec::with_capacity(calismalar.len());
                for gorev in &mut calismalar {
                    let sonuc = gorev.sonuc.take().ok_or_else(|| ic_hata(gorev.satir))?;
                    sonuclar.push((gorev.ad.clone(), sonuc));
                }
                return Ok(sonuclar);
            }

            let en_yakin = calismalar
                .iter()
                .filter_map(|g| g.uyanma_ani)
                .min()
                .ok_or_else(|| ic_hata(1))?;
            if en_yakin > simdi {
                let gecis = en_yakin.saturating_sub(simdi);
                if ana_sinyal.is_some() {
                    // İç görev ağacının beklemesi üst scheduler'a çıkar; böylece
                    // amca/teyze görevler de çocuklar uyurken ilerleyebilir.
                    gorev_bekleme_noktasi(gecis).await;
                    simdi = en_yakin.max(ortak.borrow_mut().an_ms());
                } else {
                    ortak.borrow_mut().bekle_ms(gecis);
                    simdi = en_yakin;
                }
            } else if !ilerledi {
                return Err(ic_hata(1));
            }
        }
    })
    .await;

    GOREV_BEKLEME_SINYALI.with(|yuva| *yuva.borrow_mut() = ana_sinyal);
    SON_TARIHLER.with(|yuva| *yuva.borrow_mut() = ana_son_tarihler);
    sonuc
}

/// Blok kapsamı (RFC-0004): gövdede doğan adlar gövde bitince düşer;
/// dıştaki ada atama kalıcıdır. Çözümleyicideki kuralın birebir aynısı.
fn kapsam_baslat(ortam: &HashMap<String, Deger>) -> std::collections::HashSet<String> {
    ortam.keys().cloned().collect()
}

fn kapsam_bitir(ortam: &mut HashMap<String, Deger>, kapsam: &std::collections::HashSet<String>) {
    ortam.retain(|ad, _| kapsam.contains(ad));
}

/// K-093 değer-sonuç gezme imleci: turun son döngü değerini kaynak listenin
/// aynı sırasına kopyalar. Kaynağın biçimi denetleyicide T053 ile sabitlenir;
/// bu yüzden sıra kayması ya da başka bir listeye sessiz yazma mümkün değildir.
fn gezme_ogesini_geri_yaz(
    ortam: &mut HashMap<String, Deger>,
    kaynak_adi: Option<&str>,
    dongu_adi: &str,
    sira: usize,
) {
    let Some(kaynak_adi) = kaynak_adi else {
        return;
    };
    let Some(guncel) = ortam.get(dongu_adi).cloned() else {
        return;
    };
    if let Some(Deger::Liste(ogeler)) = ortam.get_mut(kaynak_adi) {
        if let Some(yer) = ogeler.get_mut(sira) {
            *yer = guncel;
        }
    }
}

fn blok_calistir(
    cumleler: &[Cumle],
    ortam: &mut HashMap<String, Deger>,
    program: CalistirmaProgrami<'_>,
    cikti: &mut dyn GirdiCikti,
    derinlik: usize,
) -> Result<Akis, Tani> {
    hazir_calistir(blok_calistir_async(cumleler, ortam, program, cikti, derinlik))
}

/// İşlemi taze bir ortamda çalıştırır; "döndür" değeri varsa onu verir.
async fn islem_cagir(
    cagri: &Ifade,
    kaynak_adi: &str,
    argumanlar: Vec<Deger>,
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
    derinlik: usize,
    satir: usize,
) -> Result<Option<Deger>, Tani> {
    // Özyineleme korkuluğu (v0.2): Rust yığını taşmadan Türkçe tanı ver.
    // Sınır, tarayıcı motorlarının ~1 MB'lik çağrı yığınına bile payla sığmalı (K-040).
    if derinlik > 500 {
        return Err(Tani::yeni(
            "C019",
            format!("\"{}\" çağrı derinliği 500'ü aştı: temel durum hiç yakalanmıyor olabilir.", kaynak_adi),
            satir,
            1,
            1,
        )
        .onerili("Özyinelemeli adımın her seferinde temel duruma yaklaştığından emin ol.".into()));
    }
    let islem = program.islem(cagri, kaynak_adi).ok_or_else(|| ic_hata(satir))?;
    let ad = islem.ad.as_str();
    let eylem = islem.tur == IslemTuru::Eylem;
    let mut yerel: HashMap<String, Deger> = HashMap::new();
    for (param, deger) in islem.parametreler.iter().zip(argumanlar) {
        let deger = parametre_degerini_genislet(deger, param.tur_yazimi.as_deref());
        yerel.insert(param.ad.clone(), deger);
    }
    if eylem {
        io.eylem_baslat().map_err(|hata| {
            Tani::yeni(
                "C021",
                format!("\"{}\" eylem transaction'ı başlatılamadı: {}.", ad, hata),
                satir,
                1,
                1,
            )
        })?;
    }
    // Eylem transaction'ı tek bir scheduler dilimidir. Böylece iki görevin
    // savepoint'leri iç içe geçmez; başarı/geri alma her zaman doğru sahibindir.
    let askidaki_gorev_sinyali = if eylem {
        GOREV_BEKLEME_SINYALI.with(|yuva| yuva.borrow_mut().take())
    } else {
        None
    };
    let sonuc = blok_calistir_async(&islem.govde, &mut yerel, program, io, derinlik).await;
    if eylem {
        GOREV_BEKLEME_SINYALI.with(|yuva| *yuva.borrow_mut() = askidaki_gorev_sinyali);
    }
    if !eylem {
        return match sonuc? {
            Akis::Don(deger) => Ok(Some(deger)),
            Akis::Devam => Ok(None),
        };
    }

    match sonuc {
        Ok(Akis::Don(deger @ Deger::Sonuc { basarili: false, .. })) => {
            io.eylem_geri_al()
                .map_err(|hata| transaction_hatasi(ad, "geri alınamadı", &hata, satir))?;
            Ok(Some(deger))
        }
        Ok(akis) => {
            io.eylem_tamamla()
                .map_err(|hata| transaction_hatasi(ad, "tamamlanamadı", &hata, satir))?;
            match akis {
                Akis::Don(deger) => Ok(Some(deger)),
                Akis::Devam => Ok(None),
            }
        }
        Err(tani) => {
            io.eylem_geri_al()
                .map_err(|hata| transaction_hatasi(ad, "geri alınamadı", &hata, satir))?;
            Err(tani)
        }
    }
}

fn transaction_hatasi(ad: &str, eylem: &str, hata: &str, satir: usize) -> Tani {
    Tani::yeni(
        "C021",
        format!("\"{}\" eylem transaction'ı {}: {}.", ad, eylem, hata),
        satir,
        1,
        1,
    )
}

/// Açık Ondalık sözleşmesine gelen TamSayıyı runtime'da da genişletir; statik
/// tür ile gerçek değer ayrışmaz. Kapsayıcılarda aynı kural özyinelemelidir.
fn parametre_degerini_genislet(
    deger: Deger,
    tur_yazimi: Option<&str>,
) -> Deger {
    let Some(yazim) = tur_yazimi else {
        return deger;
    };
    match (yazim, deger) {
        ("Ondalık", Deger::TamSayi(sayi)) => ondalik_degeri(Ondalik::tam(sayi)),
        ("Ondalık listesi", Deger::Liste(ogeler)) => Deger::Liste(
            ogeler
                .into_iter()
                .map(|oge| parametre_degerini_genislet(oge, Some("Ondalık")))
                .collect(),
        ),
        ("Ondalık sözlüğü", Deger::Sozluk(girdiler)) => Deger::Sozluk(
            girdiler
                .into_iter()
                .map(|(ad, deger)| {
                    (ad, parametre_degerini_genislet(deger, Some("Ondalık")))
                })
                .collect(),
        ),
        ("Ondalık seçeneği", Deger::Yok) => Deger::Yok,
        ("Ondalık seçeneği", deger) => {
            parametre_degerini_genislet(deger, Some("Ondalık"))
        }
        ("Ondalık sonucu", Deger::Sonuc { basarili, icerik }) if basarili => {
            Deger::Sonuc {
                basarili,
                icerik: Box::new(parametre_degerini_genislet(
                    *icerik,
                    Some("Ondalık"),
                )),
            }
        }
        (_, deger) => deger,
    }
}

/// Genitif aritmetiğin sayısal çekirdeği: iki TamSayı → TamSayı (tam bölme);
/// Ondalık karışımı → keyfî hassasiyetli Ondalık. Sonlu bölüm tamdır; sonsuz
/// açılım yalnız Ondalık çekirdeğinin açık 34 anlamlı hane kuralıyla yuvarlanır.
fn sayisal_islem(
    islec: &AritmetikIslec,
    sol: &Deger,
    sag: &Deger,
    satir: usize,
) -> Result<Deger, Tani> {
    // Süre + Süre (checker yalnız topla/çıkar bırakır).
    if let (Deger::Sure { milisaniye: a }, Deger::Sure { milisaniye: b }) = (sol, sag) {
        let sonuc = match islec {
            AritmetikIslec::Topla => a.checked_add(*b),
            AritmetikIslec::Cikar => a.checked_sub(*b),
            _ => return Err(ic_hata(satir)),
        }
        .ok_or_else(|| tasma(satir))?;
        return Ok(Deger::Sure { milisaniye: sonuc });
    }

    // TamSayı sıcak yolu BigInt kurmaz; keyfî çekirdek yalnız Ondalık gerçekten
    // işleme girdiğinde devreye girer.
    if let (Deger::TamSayi(a), Deger::TamSayi(b)) = (sol, sag) {
        if matches!(islec, AritmetikIslec::Bol | AritmetikIslec::Kalan) && *b == 0 {
            return Err(Tani::yeni("C003", "Sıfıra bölme yapılamaz.".into(), satir, 1, 1)
                .onerili("Bölmeden önce bölenin sıfır olup olmadığını kontrol et.".into()));
        }
        let sonuc = match islec {
            AritmetikIslec::Topla => a.checked_add(*b),
            AritmetikIslec::Cikar => a.checked_sub(*b),
            AritmetikIslec::Carp => a.checked_mul(*b),
            AritmetikIslec::Bol => a.checked_div(*b),
            // K-046: okul kuralı — kalan daima negatif değildir.
            AritmetikIslec::Kalan => a.checked_rem_euclid(*b),
        }
        .ok_or_else(|| tasma(satir))?;
        return Ok(Deger::TamSayi(sonuc));
    }

    let a = sayisal_ac(sol).ok_or_else(|| ic_hata(satir))?;
    let b = sayisal_ac(sag).ok_or_else(|| ic_hata(satir))?;
    if matches!(islec, AritmetikIslec::Bol | AritmetikIslec::Kalan) && b.sifir_mi() {
        return Err(Tani::yeni("C003", "Sıfıra bölme yapılamaz.".into(), satir, 1, 1)
            .onerili("Bölmeden önce bölenin sıfır olup olmadığını kontrol et.".into()));
    }

    let sonuc = match islec {
        AritmetikIslec::Topla => a.topla(&b),
        AritmetikIslec::Cikar => a.cikar(&b),
        AritmetikIslec::Carp => a.carp(&b).ok_or_else(|| tasma(satir))?,
        AritmetikIslec::Bol => a.bol(&b).ok_or_else(|| tasma(satir))?,
        // Denetleyici kalanı Ondalık'a hiç bırakmaz (K-046, T008).
        AritmetikIslec::Kalan => return Err(ic_hata(satir)),
    };
    Ok(ondalik_degeri(sonuc))
}

#[allow(clippy::too_many_arguments)] // iç yürütme yardımcı; bağlam nesnesi v0.3'te
async fn guncelle(
    hedef: &Ifade,
    miktar: &Ifade,
    ortam: &mut HashMap<String, Deger>,
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
    derinlik: usize,
    satir: usize,
    yon: i64,
) -> Result<(), Tani> {
    let ham_ad = match hedef {
        Ifade::Degisken { cozulmus, .. } => cozulmus.as_deref(),
        _ => None,
    };
    let ad = program
        .sembol_adi(hedef, ham_ad)
        .ok_or_else(|| ic_hata(satir))?;
    let miktar = degerlendir_async(miktar, ortam, program, io, derinlik, satir).await?;
    let eski = ortam.get(&ad).cloned().ok_or_else(|| ic_hata(satir))?;
    let islec = if yon > 0 { AritmetikIslec::Topla } else { AritmetikIslec::Cikar };
    let yeni = sayisal_islem(&islec, &eski, &miktar, satir)?;
    ortam.insert(ad, yeni);
    Ok(())
}

fn degerlendir(
    ifade: &Ifade,
    ortam: &HashMap<String, Deger>,
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
    derinlik: usize,
    satir: usize,
) -> Result<Deger, Tani> {
    hazir_calistir(degerlendir_async(ifade, ortam, program, io, derinlik, satir))
}


fn tam_sayi(deger: Deger, satir: usize) -> Result<i64, Tani> {
    match deger {
        Deger::TamSayi(s) => Ok(s),
        _ => Err(ic_hata(satir)),
    }
}

fn mantiksal(deger: Deger, satir: usize) -> Result<bool, Tani> {
    match deger {
        Deger::Mantiksal(b) => Ok(b),
        _ => Err(ic_hata(satir)),
    }
}

/// CSV ayrıştırma (v0): ilk satır başlıklar, hücreler TamSayı.
/// Ayırıcı virgüldür; hücre içi tırnaklama v0'da desteklenmez.
fn csv_ayristir(icerik: &str, satir: usize) -> Result<Deger, Tani> {
    let mut satirlar = icerik.lines().filter(|s| !s.trim().is_empty());
    let basliklar: Vec<String> = match satirlar.next() {
        Some(baslik) => baslik.split(',').map(|b| b.trim().to_string()).collect(),
        None => {
            return Err(Tani::yeni("C015", "CSV dosyası boş.".into(), satir, 1, 1));
        }
    };
    let mut tablo = Vec::new();
    for (indeks, veri_satiri) in satirlar.enumerate() {
        let hucreler: Vec<&str> = veri_satiri.split(',').map(str::trim).collect();
        if hucreler.len() != basliklar.len() {
            return Err(Tani::yeni(
                "C015",
                format!(
                    "CSV {}. veri satırında {} hücre var; başlıkta {} sütun tanımlı.",
                    indeks + 1,
                    hucreler.len(),
                    basliklar.len()
                ),
                satir,
                1,
                1,
            ));
        }
        let mut kayit = Vec::new();
        for (baslik, hucre) in basliklar.iter().zip(hucreler) {
            // K-062: hücreler Metin okunur — gerçek tablolar isim taşır;
            // sayı gerekirse `değerin sayısı` ile bilinçli çevrilir.
            kayit.push((baslik.clone(), Deger::Metin(hucre.to_string())));
        }
        tablo.push(Deger::Sozluk(kayit));
    }
    Ok(Deger::Liste(tablo))
}

/// Düz JSON nesnesi ayrıştırma (v0): {"anahtar": "metin", ...}.
/// İç içe nesne/dizi ve metin dışı değerler v0'da desteklenmez.
fn json_nesnesi_ayristir(icerik: &str, satir: usize) -> Result<Deger, Tani> {
    let hata = |mesaj: String| Tani::yeni("C016", mesaj, satir, 1, 1);
    let mut karakterler = icerik.chars().peekable();

    fn bosluk_atla(k: &mut std::iter::Peekable<std::str::Chars>) {
        while matches!(k.peek(), Some(' ' | '\n' | '\r' | '\t')) {
            k.next();
        }
    }
    fn metin_oku(
        k: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<String, String> {
        if k.next() != Some('"') {
            return Err("tırnak bekleniyordu".into());
        }
        let mut metin = String::new();
        loop {
            match k.next() {
                Some('"') => return Ok(metin),
                Some('\\') => match k.next() {
                    Some('"') => metin.push('"'),
                    Some('\\') => metin.push('\\'),
                    Some('n') => metin.push('\n'),
                    Some('t') => metin.push('\t'),
                    _ => return Err("bilinmeyen kaçış".into()),
                },
                Some(c) => metin.push(c),
                None => return Err("metin kapanmadı".into()),
            }
        }
    }

    bosluk_atla(&mut karakterler);
    if karakterler.next() != Some('{') {
        return Err(hata("JSON verisi \"{\" ile başlamalı (v0: düz nesne).".into()));
    }
    let mut girdiler = Vec::new();
    loop {
        bosluk_atla(&mut karakterler);
        if karakterler.peek() == Some(&'}') {
            karakterler.next();
            break;
        }
        let anahtar = metin_oku(&mut karakterler)
            .map_err(|m| hata(format!("JSON anahtarı okunamadı: {}.", m)))?;
        bosluk_atla(&mut karakterler);
        if karakterler.next() != Some(':') {
            return Err(hata(format!("\"{}\" anahtarından sonra \":\" bekleniyor.", anahtar)));
        }
        bosluk_atla(&mut karakterler);
        // K-063: sayı/true/false/null değerleri de METİN olarak gelir (CSV
        // felsefesi): sayı gerekirse `değerin sayısı` ile bilinçli çevrilir.
        // Ondalık nokta, dilin virgülüne çevrilir; true/false → doğru/yanlış.
        let deger = if karakterler.peek() == Some(&'"') {
            metin_oku(&mut karakterler)
                .map_err(|m| hata(format!("JSON değeri okunamadı: {}.", m)))?
        } else {
            let mut ham = String::new();
            while matches!(
                karakterler.peek(),
                Some(k) if !matches!(k, ',' | '}' | ' ' | '\n' | '\r' | '\t')
            ) {
                if let Some(karakter) = karakterler.next() {
                    ham.push(karakter);
                }
            }
            match ham.as_str() {
                "" => {
                    return Err(hata(format!(
                        "\"{}\" anahtarının değeri okunamadı.",
                        anahtar
                    )))
                }
                "true" => "doğru".to_string(),
                "false" => "yanlış".to_string(),
                "null" => String::new(),
                sayi if sayi.chars().all(|k| k.is_ascii_digit() || k == '-' || k == '.') => {
                    sayi.replace('.', ",")
                }
                _ => {
                    return Err(hata(format!(
                        "\"{}\" anahtarının değeri anlaşılamadı (iç içe nesne/dizi v0'da yok).",
                        anahtar
                    )))
                }
            }
        };
        girdiler.push((anahtar, Deger::Metin(deger)));
        bosluk_atla(&mut karakterler);
        match karakterler.next() {
            Some(',') => continue,
            Some('}') => break,
            _ => return Err(hata("JSON nesnesinde \",\" ya da \"}\" bekleniyor.".into())),
        }
    }
    Ok(Deger::Sozluk(girdiler))
}

/// Tür denetiminden geçmiş programda görünmemesi gereken durum.
fn ic_hata(satir: usize) -> Tani {
    Tani::yeni(
        "C000",
        "İç tutarlılık hatası: tür denetiminden geçen program çalışırken bozuldu. \
         Bu bir derleyici hatasıdır, lütfen bildir."
            .into(),
        satir,
        1,
        1,
    )
}
