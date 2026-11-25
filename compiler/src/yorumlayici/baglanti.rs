//! K-381 (RFC-0039 A2a): adlı dinleyici bağlantı görevleri.
//!
//! Her kabul edilen bağlantı, `"<ad>" dinleyicisinde bağlantı geldiğinde`
//! bloğunu taze ortamda koşan bir görevdir. Görevler tek iş parçacıklı
//! bağlantı scheduler'ında ilerler: host okuma/yazma ilerleyemeyince
//! `Bekle` döner, görev askıya alınır, scheduler hostun `poll(2)` tabanlı
//! olay beklemesine hazır görevleri sorar. Görev hatası yalnız kendi
//! bağlantısını kapatır (yalıtılmış bağlantı kapsamı); kardeşler iptal
//! edilmez. Tek iş parçacığı olduğu için data race yoktur; ağ olaylarının
//! sırası dış girdidir ve IO izine yazılır.
//!
//! K-384 (A2c): blok okuma (SMTP nokta bloğu ve IMAP literal'i geçici
//! dosyaya), bayt-kesin okuma, satır sonusuz parça, dosyadan bağlantıya
//! aktarım ve kısa süreli satır okuması aynı işlem kanalından geçer.
//!
//! K-397 (RFC-0044 B4a, ADR-173): giden bağlantı. `"<ana>:<kapı>" adresine
//! bağlanmayı dene` host'ta `ağ_hedefleri` ve IP politikasından geçer; başarıda
//! bağlantı o görevin bağlantısı olur ve yukarıdaki bütün kalıplar onun
//! üzerinde çalışır. Görev başına aynı anda bir giden bağlantı vardır;
//! dinleyici görevi giden bağlantı açamaz. `bağlantıda TLS istemcisi
//! başlatmayı dene "<ad>" ile` istemci STARTTLS'idir ve PKIX sonucunu açık
//! alanla bildirir.

use super::*;

/// Hostun bir bağlantı işlemine verdiği sonuç.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaglantiSonucu {
    /// CRLF'siz tam satır.
    Satir(String),
    /// Yazmada kuyruğa alınan bayt; kapatma ve beklemede 0.
    Tamam(i64),
    /// İşlem henüz ilerleyemez (veri yok ya da giden tampon dolu).
    Bekle,
    /// Sınıflandırılmış ret: (`tür`, ileti).
    Hata(String, String),
    /// K-384: dosyaya alınan blok — göreli geçici yol, SHA-256, bayt.
    Blok {
        yol: String,
        sha256: String,
        bayt: i64,
    },
    /// K-397: açılan giden bağlantı (kimlik sıfırdan büyük).
    Giden(YeniBaglanti),
    /// K-397: istemci TLS el sıkışmasının sonucu (kutulu: kayıt büyük).
    TlsIstemci(Box<TlsIstemciKaydi>),
}

/// K-397: istemci STARTTLS kaydı. PKIX sonucu ayrı alandır; el sıkışma
/// imzası her durumda raporlanan SPKI'nin anahtarıyla doğrulanmıştır.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TlsIstemciKaydi {
    pub dogrulandi: bool,
    /// Boş ya da `ad_uyuşmuyor`, `süresi_dolmuş`, `henüz_geçerli_değil`,
    /// `bilinmeyen_yayıncı`, `iptal`, `geçersiz_sertifika`.
    pub dogrulama_hatasi: String,
    pub surum: String,
    pub takim: String,
    pub sunucu_adi: String,
    /// Uç sertifika DER'inin SHA-256'sı (küçük harf onaltılık).
    pub sertifika_sha256: String,
    /// Uç sertifikanın `SubjectPublicKeyInfo` DER'inin SHA-256'sı.
    pub spki_sha256: String,
    /// Sunucunun gönderdiği sırayla bütün sertifikaların özetleri.
    pub zincir_sha256: Vec<String>,
    pub zincir_spki_sha256: Vec<String>,
}

impl BaglantiSonucu {
    pub fn hata(tur: &str, mesaj: impl Into<String>) -> Self {
        Self::Hata(tur.into(), mesaj.into())
    }
}

/// Zee'nin o anki bağlantı üzerinde istediği işlem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaglantiIslemi {
    SatirOku,
    SatirYaz(String),
    Kapat,
    /// STARTTLS: ara bellek boşsa TLS el sıkışması; başarıda sürüm/takım.
    TlsBaslat,
    /// K-384: SMTP `DATA` nokta bloğu → geçici dosya.
    NoktaBlogu,
    /// K-384: tam `n` baytlık literal → geçici dosya.
    BaytBlogu(i64),
    /// K-384: tam `n` bayt → Metin (en çok satır sınırı).
    BaytOku(i64),
    /// K-384: satır sonu eklemeden yazma.
    ParcaYaz(String),
    /// K-384: dosyayı bağlantıya akıtma; yol Zee yoludur, adaptör çözer.
    DosyaAktar(String),
    /// K-384: süresi (ms) `bağlantı_boşta_süresi`nden kısa olabilen satır
    /// okuması; zaman aşımı bağlantıyı kapatmaz.
    SatirOkuSureli(i64),
    /// Scheduler dışından (iç içe eşzamanlı görev) engelleyerek hazırlık
    /// bekler; argüman Zee son tarihinin mutlak anıdır.
    Engelle(Option<i64>),
    /// K-397: kimlik 0 ile çağrılır; `"<ana>:<kapı>"` hedefine giden bağlantı.
    /// İkinci alan Zee son tarihinin mutlak anı (ms).
    GidenBaglan(String, Option<i64>),
    /// K-397: istemci STARTTLS; argüman SNI ve PKIX ad denetimi için sunucu adı.
    TlsIstemci(String),
}

/// Hostun kabul ettiği yeni bağlantı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YeniBaglanti {
    pub kimlik: u64,
    pub dinleyici: String,
    pub uzak_adres: String,
    pub uzak_kapi: u16,
    pub yerel_kapi: u16,
    /// Implicit TLS dinleyicisinden (el sıkışma tamamlanmış) geldi.
    pub tls: bool,
}

/// Bağlantı olay beklemesinin sonucu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaglantiOlayi {
    Yeni(YeniBaglanti),
    /// Bekleyen görevin işlemi artık ilerleyebilir (ya da zaman aşımıyla
    /// sınıflandırılmış sonuca hazırdır).
    Hazir(u64),
    /// SIGTERM: yeni kabul durdu; okumalar `kapanıyor` döner.
    Kapanis,
}

/// SIGTERM'den sonra kalan görevlere verilen en uzun boşaltma süresi.
pub const BOSALTMA_SURESI_MS: i64 = 10_000;

thread_local! {
    /// Scheduler bir bağlantı görevini poll ederken o görevin bağlantısı;
    /// K-397 ile ana akışın ya da eşzamanlı görevin giden bağlantısı.
    static GECERLI_BAGLANTI: Cell<Option<u64>> = const { Cell::new(None) };
    /// K-397: açık giden bağlantıların kimlikleri (dinleyici bağlantısından ayırt).
    static GIDEN_BAGLANTILAR: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    /// Yalnız bağlantı görevinin kendi poll'u sırasında dolu: IO beklemesi
    /// bu kanala işaret bırakıp Pending döner.
    static BAGLANTI_IO_SINYALI: RefCell<Option<Rc<Cell<bool>>>> =
        const { RefCell::new(None) };
}

/// İç içe görev scheduler'ı girerken IO sinyalini saklar (iç görevler
/// scheduler noktası olmadan engelleyerek bekler).
pub(super) fn io_sinyalini_al() -> Option<Rc<Cell<bool>>> {
    BAGLANTI_IO_SINYALI.with(|yuva| yuva.borrow_mut().take())
}

pub(super) fn io_sinyalini_geri_koy(sinyal: Option<Rc<Cell<bool>>>) {
    BAGLANTI_IO_SINYALI.with(|yuva| *yuva.borrow_mut() = sinyal);
}

fn baglanti_tanisi(mesaj: String, satir: usize) -> Tani {
    Tani::yeni("C030", mesaj, satir, 1, 1)
}

fn gecerli_baglanti(satir: usize) -> Result<u64, Tani> {
    GECERLI_BAGLANTI.with(Cell::get).ok_or_else(|| {
        baglanti_tanisi(
            "Bağlantı kalıbı yalnız `\"<ad>\" dinleyicisinde bağlantı geldiğinde` bloğunda (ya da onun çağırdığı işlemde) ya da başarılı `\"<ana>:<kapı>\" adresine bağlanmayı dene` sonrasında kullanılabilir.".into(),
            satir,
        )
    })
}

/// K-397: eşzamanlı görev scheduler'ı her görev için kendi bağlantı yuvasını
/// takar; dönen değer önceki yuvadır.
pub(super) fn gecerli_baglantiyi_degistir(yeni: Option<u64>) -> Option<u64> {
    GECERLI_BAGLANTI.with(|yuva| yuva.replace(yeni))
}

pub(super) fn gorevin_baglantisi() -> Option<u64> {
    GECERLI_BAGLANTI.with(|yuva| yuva.get())
}

pub(super) fn giden_mi(kimlik: u64) -> bool {
    GIDEN_BAGLANTILAR.with(|liste| liste.borrow().contains(&kimlik))
}

fn giden_birak(kimlik: u64) {
    GIDEN_BAGLANTILAR.with(|liste| liste.borrow_mut().retain(|k| *k != kimlik));
}

/// K-397: program koşusunun giden durumu (yuva, açık giden kimlikler).
pub(super) type GidenKapsami = (Option<u64>, Vec<u64>);

pub(super) fn giden_kapsamini_ac() -> GidenKapsami {
    (
        GECERLI_BAGLANTI.with(|yuva| yuva.replace(None)),
        GIDEN_BAGLANTILAR.with(|liste| std::mem::take(&mut *liste.borrow_mut())),
    )
}

pub(super) fn giden_kapsamini_kapat(io: &mut dyn GirdiCikti, onceki: GidenKapsami) {
    let acik = GIDEN_BAGLANTILAR.with(|liste| std::mem::take(&mut *liste.borrow_mut()));
    for kimlik in acik {
        let _ = io.baglanti_islemi(kimlik, &BaglantiIslemi::Kapat);
    }
    GECERLI_BAGLANTI.with(|yuva| yuva.set(onceki.0));
    GIDEN_BAGLANTILAR.with(|liste| *liste.borrow_mut() = onceki.1);
}

/// K-397: görev biterken kendi açtığı giden bağlantıyı kapatır.
pub(super) fn gideni_kapat(io: &mut dyn GirdiCikti, kimlik: u64) {
    if giden_mi(kimlik) {
        giden_birak(kimlik);
        let _ = io.baglanti_islemi(kimlik, &BaglantiIslemi::Kapat);
    }
}

struct IoBeklemeNoktasi {
    sinyal: Rc<Cell<bool>>,
    verildi: bool,
}

impl Future for IoBeklemeNoktasi {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _baglam: &mut Context<'_>) -> Poll<()> {
        if self.verildi {
            return Poll::Ready(());
        }
        self.sinyal.set(true);
        self.verildi = true;
        Poll::Pending
    }
}

/// Host `Bekle` dedi: scheduler noktası ya da (iç içe görevde) engelleyici bekleme.
async fn hazirligi_bekle(io: &mut dyn GirdiCikti, kimlik: u64) {
    let sinyal = BAGLANTI_IO_SINYALI.with(|yuva| yuva.borrow().clone());
    match sinyal {
        Some(sinyal) => {
            IoBeklemeNoktasi {
                sinyal,
                verildi: false,
            }
            .await
        }
        None => {
            let son = etkin_son_tarih().map(|son| son.an_ms);
            let _ = io.baglanti_islemi(kimlik, &BaglantiIslemi::Engelle(son));
        }
    }
}

/// Bir işlemi `Bekle` bitene kadar tekrarlar; her turda son tarih denetlenir.
async fn islemi_yurut(
    io: &mut dyn GirdiCikti,
    islem: BaglantiIslemi,
    satir: usize,
) -> Result<BaglantiSonucu, Tani> {
    let kimlik = gecerli_baglanti(satir)?;
    loop {
        son_tarihi_denetle(io, satir)?;
        match io.baglanti_islemi(kimlik, &islem) {
            BaglantiSonucu::Bekle => hazirligi_bekle(io, kimlik).await,
            sonuc => return Ok(sonuc),
        }
    }
}

fn hata_sonucu(tur: String, mesaj: String) -> Deger {
    Deger::Sonuc {
        basarili: false,
        icerik: Box::new(Deger::Hata(Box::new(HataDegeri {
            kod: "C030".into(),
            mesaj,
            neden: None,
            veri: vec![("tür".into(), Deger::Metin(tur))],
        }))),
    }
}

fn basari(icerik: Deger) -> Deger {
    Deger::Sonuc {
        basarili: true,
        icerik: Box::new(icerik),
    }
}

fn gecersiz_sonuc(mesaj: &str) -> Deger {
    hata_sonucu("geçersiz".into(), mesaj.into())
}

/// K-388 (ZM-F035): başarısız okuma dönüşü protokol birimi sınırı mı?
/// Girdi tüketen retler ve süresini dolduran okuma sınırdır; anında dönen
/// `kapandı`, `kapanıyor`, `geçersiz`, `tls` ve okumadan dönen bayt
/// `boyut_aşımı` değildir (bunları yok sayan döngü adım bütçesiyle biter).
fn birim_siniri_mi(nokta_blogu: bool, tur: &str) -> bool {
    match tur {
        "zaman_aşımı" | "satır_sonu" | "satır_uzun" | "kodlama" => true,
        "boyut_aşımı" => nokta_blogu,
        _ => false,
    }
}

/// Başarısız okuma `Sonuç`u; birim sınırındaysa bütçe yenilenir.
fn okuma_hatasi(nokta_blogu: bool, tur: String, mesaj: String) -> Deger {
    if birim_siniri_mi(nokta_blogu, &tur) {
        calistirma_butcesini_yenile();
    }
    hata_sonucu(tur, mesaj)
}

/// Metin veren okuma (satır, süreli satır, bayt): başarıda ve birim
/// sınırındaki başarısız dönüşte komut başına bütçe yenilenir (spec/12 §9).
async fn metin_oku(
    io: &mut dyn GirdiCikti,
    islem: BaglantiIslemi,
    satir: usize,
) -> Result<Deger, Tani> {
    Ok(match islemi_yurut(io, islem, satir).await? {
        BaglantiSonucu::Satir(metin) => {
            // Komut başına bütçe: her tam satır yeni bir adım/heap bütçesi açar.
            calistirma_butcesini_yenile();
            basari(Deger::Metin(metin))
        }
        BaglantiSonucu::Hata(tur, mesaj) => okuma_hatasi(false, tur, mesaj),
        _ => return Err(ic_hata(satir)),
    })
}

/// `bağlantıdan satır okumayı dene` → `Sonuç<Metin>`.
pub(super) async fn satir_oku(io: &mut dyn GirdiCikti, satir: usize) -> Result<Deger, Tani> {
    metin_oku(io, BaglantiIslemi::SatirOku, satir).await
}

/// K-384 (ZM-F032): `bağlantıdan satır okumayı <n> saniye içinde dene`.
/// Süre en az 1 sn'dir; `bağlantı_boşta_süresi`ni aşan süre ona indirilir.
/// Zaman aşımı `tür=zaman_aşımı` Sonucu'dur ve bağlantı açık kalır.
async fn satir_oku_sureli(
    io: &mut dyn GirdiCikti,
    saniye: i64,
    satir: usize,
) -> Result<Deger, Tani> {
    if saniye < 1 {
        return Ok(gecersiz_sonuc("okuma süresi en az 1 saniye olmalı"));
    }
    let ms = saniye.saturating_mul(1000);
    metin_oku(io, BaglantiIslemi::SatirOkuSureli(ms), satir).await
}

/// K-384 (ZM-F031): `bağlantıdan <n> bayt okumayı dene` → `Sonuç<Metin>`.
async fn bayt_oku(io: &mut dyn GirdiCikti, adet: i64, satir: usize) -> Result<Deger, Tani> {
    if adet < 0 {
        return Ok(gecersiz_sonuc("bayt sayısı negatif olamaz"));
    }
    metin_oku(io, BaglantiIslemi::BaytOku(adet), satir).await
}

/// K-384: nokta bloğu ya da bayt bloğu → `Sonuç<Sözlük>` (`yol`, `sha256`,
/// `bayt`; hepsi Metin, web yüklemesinin `yukleme_*` alanlarıyla aynı biçim).
async fn blok_al(
    io: &mut dyn GirdiCikti,
    islem: BaglantiIslemi,
    satir: usize,
) -> Result<Deger, Tani> {
    if let BaglantiIslemi::BaytBlogu(adet) = islem {
        if adet < 0 {
            return Ok(gecersiz_sonuc("bayt sayısı negatif olamaz"));
        }
    }
    let nokta_blogu = matches!(islem, BaglantiIslemi::NoktaBlogu);
    Ok(match islemi_yurut(io, islem, satir).await? {
        BaglantiSonucu::Blok { yol, sha256, bayt } => {
            calistirma_butcesini_yenile();
            basari(Deger::Sozluk(vec![
                ("yol".into(), Deger::Metin(yol)),
                ("sha256".into(), Deger::Metin(sha256)),
                ("bayt".into(), Deger::Metin(bayt.to_string())),
            ]))
        }
        BaglantiSonucu::Hata(tur, mesaj) => okuma_hatasi(nokta_blogu, tur, mesaj),
        _ => return Err(ic_hata(satir)),
    })
}

/// K-384: `bağlantıya X parçasını yazmayı dene` ve `bağlantıya Y dosyasını
/// aktarmayı dene` → `Sonuç<TamSayı>` (kuyruğa alınan bayt).
async fn bayt_yaz(
    io: &mut dyn GirdiCikti,
    islem: BaglantiIslemi,
    satir: usize,
) -> Result<Deger, Tani> {
    Ok(match islemi_yurut(io, islem, satir).await? {
        BaglantiSonucu::Tamam(bayt) => basari(Deger::TamSayi(bayt)),
        BaglantiSonucu::Hata(tur, mesaj) => hata_sonucu(tur, mesaj),
        _ => return Err(ic_hata(satir)),
    })
}

/// `a2c_yurut`un dağıttığı kimlikler (K-384 A2c ve K-397 giden bağlantı).
pub(super) fn a2c_mi(kimlik: &str) -> bool {
    use crate::intrinsic as i;
    matches!(
        kimlik,
        i::BAGLANTI_NOKTA_BLOGU
            | i::BAGLANTI_BAYT_BLOGU
            | i::BAGLANTI_BAYT_OKU
            | i::BAGLANTI_SATIR_OKU_SURELI
            | i::BAGLANTI_PARCA_YAZ
            | i::BAGLANTI_DOSYA_AKTAR
            | i::GIDEN_BAGLAN
            | i::BAGLANTI_TLS_ISTEMCI
    )
}

/// K-384 (A2c) ve K-397 intrinsic'lerinin dağıtımı (tür denetimi derleme zamanında).
pub(super) async fn a2c_yurut(
    kimlik: &str,
    degerler: &[Deger],
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    use crate::intrinsic as i;
    match (kimlik, degerler) {
        (i::BAGLANTI_NOKTA_BLOGU, []) => blok_al(io, BaglantiIslemi::NoktaBlogu, satir).await,
        (i::BAGLANTI_BAYT_BLOGU, [Deger::TamSayi(adet)]) => {
            blok_al(io, BaglantiIslemi::BaytBlogu(*adet), satir).await
        }
        (i::BAGLANTI_BAYT_OKU, [Deger::TamSayi(adet)]) => bayt_oku(io, *adet, satir).await,
        (i::BAGLANTI_SATIR_OKU_SURELI, [Deger::TamSayi(saniye)]) => {
            satir_oku_sureli(io, *saniye, satir).await
        }
        (i::BAGLANTI_PARCA_YAZ, [metin]) => {
            let metin = metne_sinirli(metin, satir)?;
            bayt_yaz(io, BaglantiIslemi::ParcaYaz(metin), satir).await
        }
        (i::BAGLANTI_DOSYA_AKTAR, [yol]) => {
            let yol = metne_sinirli(yol, satir)?;
            bayt_yaz(io, BaglantiIslemi::DosyaAktar(yol), satir).await
        }
        // K-397: giden bağlanma ve istemci STARTTLS.
        (i::GIDEN_BAGLAN, [adres]) => giden_baglan(io, metne_sinirli(adres, satir)?, satir).await,
        (i::BAGLANTI_TLS_ISTEMCI, [ad]) => tls_istemci(io, metne_sinirli(ad, satir)?, satir).await,
        _ => Err(ic_hata(satir)),
    }
}

/// `bağlantıya X satırını yazmayı dene` → `Sonuç<TamSayı>`.
pub(super) async fn satir_yaz(
    io: &mut dyn GirdiCikti,
    metin: String,
    satir: usize,
) -> Result<Deger, Tani> {
    Ok(
        match islemi_yurut(io, BaglantiIslemi::SatirYaz(metin), satir).await? {
            BaglantiSonucu::Tamam(bayt) => Deger::Sonuc {
                basarili: true,
                icerik: Box::new(Deger::TamSayi(bayt)),
            },
            BaglantiSonucu::Hata(tur, mesaj) => hata_sonucu(tur, mesaj),
            _ => return Err(ic_hata(satir)),
        },
    )
}

/// `bağlantıda TLS başlatmayı dene` → `Sonuç<Metin>` (ör. `TLSv1_3 TLS13_AES_128_GCM_SHA256`).
/// STARTTLS öncesi ara bellekte bekleyen düz metin varsa yükseltme yapılmaz,
/// bağlantı kapanır ve `tür=tls` döner (CVE-2011-0411 sınıfı).
pub(super) async fn tls_baslat(io: &mut dyn GirdiCikti, satir: usize) -> Result<Deger, Tani> {
    Ok(
        match islemi_yurut(io, BaglantiIslemi::TlsBaslat, satir).await? {
            BaglantiSonucu::Satir(bilgi) => Deger::Sonuc {
                basarili: true,
                icerik: Box::new(Deger::Metin(bilgi)),
            },
            BaglantiSonucu::Hata(tur, mesaj) => hata_sonucu(tur, mesaj),
            _ => return Err(ic_hata(satir)),
        },
    )
}

/// `bağlantıya X satırını yaz` / `… parçasını yaz` cümlesi: hata bağlantı
/// görevini C030 ile bitirir.
pub(super) async fn yaz_cumlesi(
    io: &mut dyn GirdiCikti,
    islem: BaglantiIslemi,
    satir: usize,
) -> Result<(), Tani> {
    match islemi_yurut(io, islem, satir).await? {
        BaglantiSonucu::Tamam(_) => Ok(()),
        BaglantiSonucu::Hata(tur, mesaj) => Err(baglanti_tanisi(
            format!("Bağlantıya yazılamadı ({tur}): {mesaj}."),
            satir,
        )),
        _ => Err(ic_hata(satir)),
    }
}

/// `bağlantıyı kapat`. Giden bağlantıda yuva boşalır (yeni bağlantı açılabilir).
pub(super) async fn kapat(io: &mut dyn GirdiCikti, satir: usize) -> Result<(), Tani> {
    let kimlik = gecerli_baglanti(satir)?;
    let sonuc = islemi_yurut(io, BaglantiIslemi::Kapat, satir).await?;
    if giden_mi(kimlik) {
        giden_birak(kimlik);
        GECERLI_BAGLANTI.with(|yuva| yuva.set(None));
    }
    match sonuc {
        BaglantiSonucu::Tamam(_) | BaglantiSonucu::Hata(..) => Ok(()),
        _ => Err(ic_hata(satir)),
    }
}

/// K-397: `"<ana>:<kapı>" adresine bağlanmayı dene` → `Sonuç<Sözlük>`
/// (`uzak_adres`, `uzak_kapı`, `yerel_kapı`). Bağlanma tek host çağrısıdır;
/// süresi host'ta `min(Zee son tarihi, 30 sn)` ile sınırlıdır.
async fn giden_baglan(io: &mut dyn GirdiCikti, adres: String, satir: usize) -> Result<Deger, Tani> {
    son_tarihi_denetle(io, satir)?;
    match GECERLI_BAGLANTI.with(Cell::get) {
        Some(kimlik) if giden_mi(kimlik) => {
            return Ok(hata_sonucu(
                "zaten_bağlı".into(),
                "bu görevin açık giden bağlantısı var; önce `bağlantıyı kapat`".into(),
            ))
        }
        Some(_) => return Ok(gecersiz_sonuc(
            "dinleyici görevinde giden bağlantı açılamaz; gelen oturumun bağlantısı değiştirilemez",
        )),
        None => {}
    }
    let son = etkin_son_tarih().map(|son| son.an_ms);
    Ok(
        match io.baglanti_islemi(0, &BaglantiIslemi::GidenBaglan(adres, son)) {
            BaglantiSonucu::Giden(yeni) if yeni.kimlik > 0 => {
                GIDEN_BAGLANTILAR.with(|liste| liste.borrow_mut().push(yeni.kimlik));
                GECERLI_BAGLANTI.with(|yuva| yuva.set(Some(yeni.kimlik)));
                basari(Deger::Sozluk(vec![
                    ("uzak_adres".into(), Deger::Metin(yeni.uzak_adres)),
                    ("uzak_kapı".into(), Deger::Metin(yeni.uzak_kapi.to_string())),
                    (
                        "yerel_kapı".into(),
                        Deger::Metin(yeni.yerel_kapi.to_string()),
                    ),
                ]))
            }
            BaglantiSonucu::Hata(tur, mesaj) => hata_sonucu(tur, mesaj),
            _ => return Err(ic_hata(satir)),
        },
    )
}

/// K-397: `bağlantıda TLS istemcisi başlatmayı dene "<ad>" ile` →
/// `Sonuç<Sözlük>`. PKIX başarısızlığı `doğrulandı=hayır` ile açıkça döner
/// (DANE-EE ve fırsatçı TLS el sıkışmayı ister); TLS kurulamazsa `tür=tls`.
async fn tls_istemci(
    io: &mut dyn GirdiCikti,
    sunucu_adi: String,
    satir: usize,
) -> Result<Deger, Tani> {
    let kimlik = gecerli_baglanti(satir)?;
    if !giden_mi(kimlik) {
        return Ok(gecersiz_sonuc(
            "TLS istemcisi yalnız giden bağlantıda başlatılır; gelen bağlantıda `bağlantıda TLS başlatmayı dene`",
        ));
    }
    Ok(
        match islemi_yurut(io, BaglantiIslemi::TlsIstemci(sunucu_adi), satir).await? {
            BaglantiSonucu::TlsIstemci(kayit) => basari(tls_kaydi_sozlugu(*kayit)),
            BaglantiSonucu::Hata(tur, mesaj) => hata_sonucu(tur, mesaj),
            _ => return Err(ic_hata(satir)),
        },
    )
}

fn tls_kaydi_sozlugu(k: TlsIstemciKaydi) -> Deger {
    let metin = |m: String| Deger::Metin(m);
    Deger::Sozluk(vec![
        (
            "doğrulandı".into(),
            metin(if k.dogrulandi { "evet" } else { "hayır" }.into()),
        ),
        ("doğrulama_hatası".into(), metin(k.dogrulama_hatasi)),
        ("sürüm".into(), metin(k.surum)),
        ("takım".into(), metin(k.takim)),
        ("sunucu_adı".into(), metin(k.sunucu_adi)),
        ("sertifika_sha256".into(), metin(k.sertifika_sha256)),
        ("spki_sha256".into(), metin(k.spki_sha256)),
        ("zincir_sha256".into(), metin(k.zincir_sha256.join(" "))),
        (
            "zincir_spki_sha256".into(),
            metin(k.zincir_spki_sha256.join(" ")),
        ),
    ])
}

/// K-381 cümlelerinin yürütülmesi (bağlantı bloğu kaydı, yazma, kapatma).
pub(super) async fn cumle_yurut(
    cumle: &Cumle,
    ortam: &mut HashMap<String, Deger>,
    program: CalistirmaProgrami<'_>,
    cikti: &mut dyn GirdiCikti,
    derinlik: usize,
) -> Result<(), Tani> {
    match cumle {
        Cumle::BaglantiGeldiginde {
            dinleyici, satir, ..
        } => {
            let deger =
                degerlendir_async(dinleyici, ortam, program, cikti, derinlik, *satir).await?;
            let ad = metne_sinirli(&deger, *satir)?;
            son_tarihi_denetle(cikti, *satir)?;
            kaydet(&ad, ortam, cikti, *satir)
        }
        Cumle::BaglantiyaYaz {
            deger,
            satir_sonu,
            satir,
        } => {
            let deger = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
            let metin = metne_sinirli(&deger, *satir)?;
            let islem = if *satir_sonu {
                BaglantiIslemi::SatirYaz(metin)
            } else {
                BaglantiIslemi::ParcaYaz(metin)
            };
            yaz_cumlesi(cikti, islem, *satir).await
        }
        Cumle::BaglantiyiKapat { satir } => kapat(cikti, *satir).await,
        // Yalnız kayıt: rota gövdesi sunucu döngüsünde istek gelince koşulur.
        Cumle::IstekGeldiginde { .. } => Ok(()),
        _ => Err(ic_hata(1)),
    }
}

/// Üst düzey kayıt: `"<ad>" dinleyicisinde bağlantı geldiğinde` cümlesinin
/// yürütülmesi. Gövde scheduler'da bağlantı başına koşar.
fn kaydet(
    ad: &str,
    ortam: &mut HashMap<String, Deger>,
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<(), Tani> {
    let anahtar = format!("(bağlantı:{ad})");
    if ortam.contains_key(&anahtar) {
        return Err(baglanti_tanisi(
            format!("\"{ad}\" dinleyicisine ikinci bağlantı bloğu yazıldı; her dinleyicinin tek bloğu olur."),
            satir,
        ));
    }
    let omur = io.baglanti_dinleyicisi_kaydet(ad).map_err(|hata| {
        baglanti_tanisi(
            format!("\"{ad}\" dinleyicisi kullanılamıyor: {hata}."),
            satir,
        )
    })?;
    ortama_yaz(ortam, &anahtar, Deger::TamSayi(omur), satir)?;
    ortama_yaz(ortam, "(bağlantı)", Deger::Mantiksal(true), satir)?;
    Ok(())
}

struct BaglantiGorevi<'a> {
    kimlik: u64,
    gelecek: Option<GorevGelecegi<'a>>,
    zaman_sinyali: Rc<Cell<Option<i64>>>,
    io_sinyali: Rc<Cell<bool>>,
    /// `bekle` ile uyanma anı.
    uyanma_ani: Option<i64>,
    io_bekliyor: bool,
    son_tarihler: Vec<SonTarih>,
    butce: Option<CalistirmaButcesi>,
}

impl BaglantiGorevi<'_> {
    fn hazir(&self, simdi: i64) -> bool {
        if self.gelecek.is_none() {
            return false;
        }
        if self.io_bekliyor {
            // Zee son tarihi dolduysa okuma noktasında Ç001 için uyanır.
            return self.son_tarih().is_some_and(|an| an <= simdi);
        }
        self.uyanma_ani.is_none_or(|an| an <= simdi)
    }

    fn son_tarih(&self) -> Option<i64> {
        self.son_tarihler.iter().map(|son| son.an_ms).min()
    }

    /// Scheduler'ın bu görev için uyanması gereken en erken an.
    fn uyanma(&self) -> Option<i64> {
        let zaman = if self.io_bekliyor {
            None
        } else {
            self.uyanma_ani
        };
        [zaman, self.son_tarih()].into_iter().flatten().min()
    }
}

fn baglanti_sozlugu(yeni: &YeniBaglanti) -> Deger {
    Deger::Sozluk(vec![
        ("dinleyici".into(), Deger::Metin(yeni.dinleyici.clone())),
        ("uzak_adres".into(), Deger::Metin(yeni.uzak_adres.clone())),
        ("uzak_kapı".into(), Deger::Metin(yeni.uzak_kapi.to_string())),
        (
            "yerel_kapı".into(),
            Deger::Metin(yeni.yerel_kapi.to_string()),
        ),
        (
            "tls".into(),
            Deger::Metin(if yeni.tls { "evet" } else { "hayır" }.into()),
        ),
    ])
}

/// Kayıtlı dinleyici adı → (gövde, ömür ms).
type Isleyiciler<'a> = Vec<(String, &'a [Cumle], i64)>;

fn isleyicileri_topla<'a>(
    program: CalistirmaProgrami<'a>,
    ortam: &HashMap<String, Deger>,
    io: &mut dyn GirdiCikti,
) -> Result<Isleyiciler<'a>, Tani> {
    let mut isleyiciler = Vec::new();
    for cumle in &program.program().cumleler {
        let Cumle::BaglantiGeldiginde {
            dinleyici,
            govde,
            satir,
        } = cumle
        else {
            continue;
        };
        let ad = metne_sinirli(
            &degerlendir(dinleyici, &HashMap::new(), program, io, 0, *satir)?,
            *satir,
        )?;
        let Some(Deger::TamSayi(omur)) = ortam.get(&format!("(bağlantı:{ad})")) else {
            return Err(baglanti_tanisi(
                format!("\"{ad}\" bağlantı bloğu üst düzeyde kaydedilmedi."),
                *satir,
            ));
        };
        isleyiciler.push((ad, govde.as_slice(), *omur));
    }
    Ok(isleyiciler)
}

/// Görevi kendi bağlam yuvalarıyla bir kez poll eder.
fn gorevi_poll_et(
    gorev: &mut BaglantiGorevi<'_>,
    baglam: &mut Context<'_>,
) -> Poll<Result<Deger, Tani>> {
    SON_TARIHLER.with(|yuva| *yuva.borrow_mut() = std::mem::take(&mut gorev.son_tarihler));
    CALISTIRMA_BUTCESI.with(|yuva| *yuva.borrow_mut() = gorev.butce.take());
    GOREV_BEKLEME_SINYALI.with(|yuva| *yuva.borrow_mut() = Some(Rc::clone(&gorev.zaman_sinyali)));
    BAGLANTI_IO_SINYALI.with(|yuva| *yuva.borrow_mut() = Some(Rc::clone(&gorev.io_sinyali)));
    let onceki = GECERLI_BAGLANTI.with(|yuva| yuva.replace(Some(gorev.kimlik)));
    let poll = match gorev.gelecek.as_mut() {
        Some(gelecek) => gelecek.as_mut().poll(baglam),
        None => Poll::Ready(Err(ic_hata(1))),
    };
    GECERLI_BAGLANTI.with(|yuva| yuva.set(onceki));
    BAGLANTI_IO_SINYALI.with(|yuva| *yuva.borrow_mut() = None);
    GOREV_BEKLEME_SINYALI.with(|yuva| *yuva.borrow_mut() = None);
    gorev.butce = CALISTIRMA_BUTCESI.with(|yuva| yuva.borrow_mut().take());
    gorev.son_tarihler = SON_TARIHLER.with(|yuva| std::mem::take(&mut *yuva.borrow_mut()));
    poll
}

/// Görev sonucu: bağlantı kapatılır; hata yalnız günlüğe düşer.
fn gorev_bitti(io: &mut dyn GirdiCikti, kimlik: u64, sonuc: Result<Deger, Tani>) -> Option<i64> {
    let mut cikis = None;
    match sonuc {
        Ok(_) => {}
        Err(tani) if tani.kod == "Ç000" => cikis = Some(tani.mesaj.parse::<i64>().unwrap_or(0)),
        Err(tani) => {
            let neden = if tani.kod == "Ç001" {
                "bağlantı ömrü ya da son tarih doldu".to_string()
            } else {
                format!("{} satır {}: {}", tani.kod, tani.satir, tani.mesaj)
            };
            io.baglanti_tanisini_gunlukle(kimlik, &neden);
        }
    }
    let _ = io.baglanti_islemi(kimlik, &BaglantiIslemi::Kapat);
    cikis
}

type OrtakIo<'a> = Rc<RefCell<&'a mut dyn GirdiCikti>>;

/// Tek iş parçacıklı bağlantı scheduler'ının durumu.
struct BaglantiSunucusu<'a> {
    program: CalistirmaProgrami<'a>,
    isleyiciler: Isleyiciler<'a>,
    ortak: OrtakIo<'a>,
    gorevler: Vec<BaglantiGorevi<'a>>,
    bosaltma_sonu: Option<i64>,
}

impl<'a> BaglantiSunucusu<'a> {
    fn simdi(&self) -> i64 {
        self.ortak.borrow_mut().an_ms()
    }

    fn kapat(&self, kimlik: u64) {
        let _ = self
            .ortak
            .borrow_mut()
            .baglanti_islemi(kimlik, &BaglantiIslemi::Kapat);
    }

    fn hepsini_kapat(&mut self, neden: Option<&str>) {
        for kalan in std::mem::take(&mut self.gorevler) {
            if let Some(neden) = neden {
                self.ortak
                    .borrow_mut()
                    .baglanti_tanisini_gunlukle(kalan.kimlik, neden);
            }
            self.kapat(kalan.kimlik);
        }
    }

    /// Hazır görevleri kabul sırasıyla birer kez ilerletir; `Some` çıkış kodudur.
    fn hazirlari_ilerlet(&mut self, baglam: &mut Context<'_>) -> Result<Option<i64>, Tani> {
        let mut sira = 0;
        while sira < self.gorevler.len() {
            let simdi = self.simdi();
            if !self.gorevler[sira].hazir(simdi) {
                sira += 1;
                continue;
            }
            let gorev = &mut self.gorevler[sira];
            gorev.uyanma_ani = None;
            gorev.io_bekliyor = false;
            gorev.zaman_sinyali.set(None);
            gorev.io_sinyali.set(false);
            match gorevi_poll_et(gorev, baglam) {
                Poll::Ready(sonuc) => {
                    let kimlik = gorev.kimlik;
                    self.gorevler.remove(sira);
                    let cikis = gorev_bitti(&mut **self.ortak.borrow_mut(), kimlik, sonuc);
                    if let Some(kod) = cikis {
                        self.hepsini_kapat(None);
                        return Ok(Some(kod));
                    }
                }
                Poll::Pending if gorev.io_sinyali.get() => {
                    gorev.io_bekliyor = true;
                    sira += 1;
                }
                Poll::Pending => {
                    let sure = gorev.zaman_sinyali.take().ok_or_else(|| ic_hata(1))?;
                    let simdi = self.ortak.borrow_mut().an_ms();
                    self.gorevler[sira].uyanma_ani = Some(simdi.saturating_add(sure));
                    sira += 1;
                }
            }
        }
        Ok(None)
    }

    fn gorev_kur(&mut self, yeni: YeniBaglanti) {
        let Some((_, govde, omur)) = self
            .isleyiciler
            .iter()
            .find(|(ad, _, _)| *ad == yeni.dinleyici)
        else {
            self.kapat(yeni.kimlik);
            return;
        };
        let govde: &'a [Cumle] = govde;
        let program = self.program;
        let mut gorev_io = PaylasilanIo {
            ic: Rc::clone(&self.ortak),
        };
        let mut gorev_ortami = HashMap::new();
        gorev_ortami.insert("bağlantı".to_string(), baglanti_sozlugu(&yeni));
        let bitis = self.simdi().saturating_add(*omur);
        let gelecek: GorevGelecegi<'a> = Box::pin(async move {
            let _omur = SonTarihNobetcisi::yeni(bitis);
            gorev_ortami_butcesini_tuket(&gorev_ortami, 0, 1)?;
            blok_calistir_async(govde, &mut gorev_ortami, program, &mut gorev_io, 0)
                .await
                .map(|_| Deger::Mantiksal(true))
        });
        self.gorevler.push(BaglantiGorevi {
            kimlik: yeni.kimlik,
            gelecek: Some(gelecek),
            zaman_sinyali: Rc::new(Cell::new(None)),
            io_sinyali: Rc::new(Cell::new(false)),
            uyanma_ani: None,
            io_bekliyor: false,
            son_tarihler: Vec::new(),
            butce: Some(CalistirmaButcesi::yeni()),
        });
    }

    fn olayi_isle(&mut self, olay: BaglantiOlayi) {
        match olay {
            BaglantiOlayi::Hazir(kimlik) => {
                if let Some(gorev) = self.gorevler.iter_mut().find(|g| g.kimlik == kimlik) {
                    gorev.io_bekliyor = false;
                }
            }
            BaglantiOlayi::Kapanis if self.bosaltma_sonu.is_none() => {
                self.bosaltma_sonu = Some(self.simdi().saturating_add(BOSALTMA_SURESI_MS));
                for gorev in &mut self.gorevler {
                    gorev.io_bekliyor = false;
                }
            }
            BaglantiOlayi::Kapanis => {}
            BaglantiOlayi::Yeni(yeni) => self.gorev_kur(yeni),
        }
    }

    /// Hazır görev yoksa host olaylarını bekler ve işler.
    fn bekle(&mut self) -> Result<(), Tani> {
        let bekleyenler: Vec<u64> = self
            .gorevler
            .iter()
            .filter(|g| g.io_bekliyor)
            .map(|g| g.kimlik)
            .collect();
        let uyanma = self
            .gorevler
            .iter()
            .filter_map(BaglantiGorevi::uyanma)
            .chain(self.bosaltma_sonu)
            .min();
        let olaylar = self
            .ortak
            .borrow_mut()
            .baglanti_olaylarini_bekle(&bekleyenler, uyanma)
            .map_err(|hata| baglanti_tanisi(format!("Bağlantı beklemesi başarısız: {hata}."), 1))?;
        olaylar.into_iter().for_each(|olay| self.olayi_isle(olay));
        Ok(())
    }

    fn calistir(&mut self) -> Result<i64, Tani> {
        let uyandirici = bos_uyandirici();
        let mut baglam = Context::from_waker(&uyandirici);
        loop {
            if let Some(kod) = self.hazirlari_ilerlet(&mut baglam)? {
                return Ok(kod);
            }
            let simdi = self.simdi();
            if let Some(son) = self.bosaltma_sonu {
                if self.gorevler.is_empty() {
                    return Ok(0);
                }
                if simdi >= son {
                    self.hepsini_kapat(Some("boşaltma süresi doldu, bağlantı iptal edildi"));
                    return Ok(0);
                }
            }
            if !self.gorevler.iter().any(|g| g.hazir(simdi)) {
                self.bekle()?;
            }
        }
    }
}

/// Program gövdesi bittikten sonra kayıtlı bağlantı bloklarını sunar.
/// SIGTERM ve boşaltmadan sonra (ya da hermetik adaptör bağlantı tükenince)
/// 0, bağlantı içindeki `programı bitir` ile o kod döner.
pub(super) fn baglantilari_sun(
    program: CalistirmaProgrami<'_>,
    ortam: &HashMap<String, Deger>,
    io: &mut dyn GirdiCikti,
) -> Result<i64, Tani> {
    if ortam.contains_key("(sunucu)") {
        return Err(baglanti_tanisi(
            "Aynı programda web sunucusu (`sunucu başlat`) ve bağlantı bloğu birlikte çalışmaz (RFC-0039 A2a sınırı); ikisini ayrı hizmetlere böl.".into(),
            1,
        ));
    }
    let isleyiciler = isleyicileri_topla(program, ortam, io)?;
    let ust_sinyal = GOREV_BEKLEME_SINYALI.with(|yuva| yuva.borrow_mut().take());
    let ust_son_tarihler = SON_TARIHLER.with(|yuva| std::mem::take(&mut *yuva.borrow_mut()));
    let ust_butce = CALISTIRMA_BUTCESI.with(|yuva| yuva.borrow_mut().take());
    let mut sunucu = BaglantiSunucusu {
        program,
        isleyiciler,
        ortak: Rc::new(RefCell::new(io)),
        gorevler: Vec::new(),
        bosaltma_sonu: None,
    };
    let sonuc = sunucu.calistir();
    // Kalan görev future'ları `ortak`ı ödünç tutar; önce düşürülür.
    drop(sunucu);
    GOREV_BEKLEME_SINYALI.with(|yuva| *yuva.borrow_mut() = ust_sinyal);
    SON_TARIHLER.with(|yuva| *yuva.borrow_mut() = ust_son_tarihler);
    CALISTIRMA_BUTCESI.with(|yuva| *yuva.borrow_mut() = ust_butce);
    sonuc
}
