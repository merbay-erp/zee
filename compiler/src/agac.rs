//! Soyut sözdizimi ağacı (AST).

use std::collections::HashMap;

/// Derlenmiş program: üst düzey cümleler + ada göre işlem tanımları.
/// İşlem gövdeleri buraya kaldırılır (hoist); hem denetleyici hem yorumlayıcı
/// aynı kayıttan okur.
#[derive(Debug, Clone)]
pub struct Program {
    pub cumleler: Vec<Cumle>,
    pub islemler: HashMap<String, Islem>,
    /// Yapı tanımları; Tur::Yapi bu listeye indeksle işaret eder.
    pub yapilar: Vec<Yapi>,
    /// `test "..."` blokları — `dil çalıştır` bunları atlar, `dil dene` koşar.
    pub testler: Vec<Test>,
}

/// `test "<açıklama>"` bloğu (K-025). Her test taze bir ortamda koşar.
#[derive(Debug, Clone)]
pub struct Test {
    pub ad: String,
    pub govde: Vec<Cumle>,
    pub satir: usize,
}

/// "yapı Öğrenci" tanımı: alan adı + tür yazımı ("TamSayı", "Metin"...).
#[derive(Debug, Clone)]
pub struct Yapi {
    pub ad: String,
    pub alanlar: Vec<(String, String)>,
    pub satir: usize,
}

/// "işlem ortalamayı hesapla" tanımı. Ad çok kelimeli bir eylem cümlesidir.
#[derive(Debug, Clone)]
pub struct Islem {
    pub ad: String,
    /// Yalın parametre adları ("sayıları al" → "sayılar").
    pub parametreler: Vec<String>,
    pub govde: Vec<Cumle>,
    pub satir: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Islec {
    Buyuk,
    Kucuk,
    BuyukEsit,
    KucukEsit,
    Esit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AritmetikIslec {
    Topla,
    Cikar,
    Carp,
    Bol,
}

#[derive(Debug, Clone)]
pub enum Ifade {
    MetinSabiti(String),
    SayiSabiti(i64),
    /// "doğru" / "yanlış" (master plan bölüm 7).
    MantiksalSabiti(bool),
    /// "1 ile 100 arasında rastgele sayı" — iki uç dahil.
    Rastgele { alt: Box<Ifade>, ust: Box<Ifade> },
    /// Kaynaktaki ham kelime; ad çözümleme ek ayıklamasıyla `cozulmus`ü doldurur.
    Degisken {
        ham: String,
        cozulmus: Option<String>,
        satir: usize,
        sutun: usize,
        uzunluk: usize,
    },
    /// "ile" ile zincirlenmiş parçaların birleştirilmesi (K-004).
    Birlestir(Vec<Ifade>),
    Karsilastirma {
        sol: Box<Ifade>,
        sag: Box<Ifade>,
        islec: Islec,
    },
    /// "sayı çiftse" (doğruysa çift).
    Cift(Box<Ifade>),
    /// "sayı tekse".
    Tek(Box<Ifade>),
    /// "3, 7, 1, 9 listesi" (K-012). Öğeler tek türden olmalı.
    ListeSabiti(Vec<Ifade>),
    /// "boş liste".
    BosListe,
    /// Liste/metin özellikleri: "sayıların adedi", "cümlenin uzunluğu"...
    Ozellik { nesne: Box<Ifade>, ozellik: Ozellik },
    /// "boş sözlük" — v0: Sözlük<Metin, TamSayı>.
    BosSozluk,
    /// "yaşların (anahtar) değeri" — sözlükten okuma (K-015).
    SozlukDegeri { sozluk: Box<Ifade>, anahtar: Box<Ifade> },
    /// "yaşlarda (anahtar) varsa/yoksa" (K-015).
    SozlukteVar {
        sozluk: Box<Ifade>,
        anahtar: Box<Ifade>,
        olumsuz: bool,
    },
    /// "cümlenin büyük/küçük harflisi" — Türkçe kurallarla (İ/i, I/ı).
    MetinDonusum { nesne: Box<Ifade>, buyuk: bool },
    /// "cümle (aranan) içeriyorsa".
    Icerir { metin: Box<Ifade>, aranan: Box<Ifade> },
    /// "yok" sabiti — Seçenek türünün boş hali (K-017).
    YokSabiti,
    /// "X varsa/yoksa" — Seçenek dolu mu (K-017).
    SecenekVar { nesne: Box<Ifade>, olumsuz: bool },
    /// "X in değeri" — Seçenek/Sonuç içindeki değer; boşken çalışma hatası.
    IcDeger(Box<Ifade>),
    /// "sonucun hatası" — Sonuç'un hata metni (K-018).
    SonucHatasi(Box<Ifade>),
    /// "X başarılıysa/başarısızsa" (K-018).
    SonucBasarili { nesne: Box<Ifade>, olumsuz: bool },
    /// `"..." dosyasını okumayı dene` → Sonuç (K-018).
    DosyaOkumayiDene(Box<Ifade>),
    /// `"..." dosyasının satırları` → Liste<Metin>. Düz biçim: hata anında
    /// Türkçe çalışma hatası verir (Sonuç isteyen "dene" kullanır — K-018).
    DosyaSatirlari(Box<Ifade>),
    /// `"..." dosyasından okunan tablo` → Liste<Sözlük> (CSV; başlık satırı
    /// anahtar olur, hücreler v0'da TamSayı).
    TabloOku(Box<Ifade>),
    /// `"..." dosyasından okunan veri` → Sözlük<Metin, Metin> (düz JSON nesnesi).
    VeriOku(Box<Ifade>),
    /// `bugünün tarihi` → Tarih (saat kaynağı IO soyutlamasından gelir).
    BugununTarihi,
    /// `şu anın saati` → Saat.
    SuAninSaati,
    /// `bugünün 1 gün sonrası` → Tarih.
    GunSonrasi { tarih: Box<Ifade>, miktar: Box<Ifade> },
    /// `komut satırından gelenler` → Liste<Metin>.
    KomutArgumanlari,
    /// `argümanlar boşsa` — liste/metin boş mu.
    BosMu { nesne: Box<Ifade>, olumsuz: bool },
    /// Genitif aritmetik (K-008): "a ile b nin toplamı", "x in y ye bölümü".
    Aritmetik {
        islec: AritmetikIslec,
        sol: Box<Ifade>,
        sag: Box<Ifade>,
    },
    /// Metinden sayıya dönüşüm (K-009): "yanıtın sayısı".
    Sayisi(Box<Ifade>),
    /// "yeni Öğrenci" — alanları varsayılan değerli yeni yapı örneği (K-020).
    YeniYapi { yapi_adi: String },
    /// "ayşenin adı" — iyelik ekiyle alan okuma (K-020). `alan` ham yazımdır
    /// ("adı"); çözümleyici yapı tanımındaki yalın ada ("ad") çevirir.
    AlanErisim { nesne: Box<Ifade>, alan: String },
    /// İşlem çağrısı (K-016, geçici sözdizimi): "notlar için ortalamayı hesapla",
    /// çok argüman: "a ve b ile selamla".
    IslemCagrisi {
        islem_adi: String,
        argumanlar: Vec<Ifade>,
        satir: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ozellik {
    /// Liste: öğe sayısı.
    Adet,
    Ilk,
    Son,
    /// Metin: karakter sayısı ("cümlenin uzunluğu").
    Uzunluk,
    /// Metin: boşluklardan bölünmüş kelime listesi ("cümlenin kelimeleri").
    Kelimeler,
    /// Tarih: yıl bileşeni ("bugünün yılı").
    Yil,
}

#[derive(Debug, Clone)]
pub struct KosulKolu {
    pub kosul: Ifade,
    pub govde: Vec<Cumle>,
}

#[derive(Debug, Clone)]
pub enum Cumle {
    /// `<ifade> yaz`
    Yaz { deger: Ifade, satir: usize },
    /// `<ad> <ifade> olsun`
    Olsun {
        ad: String,
        deger: Ifade,
        satir: usize,
        sutun: usize,
        uzunluk: usize,
    },
    /// `<n> kez tekrarla` + blok
    KezTekrarla { adet: Ifade, govde: Vec<Cumle>, satir: usize },
    /// `<a> den <b> e kadar her <ad> için` + blok (K-006)
    AralikDongusu {
        ad: String,
        bastan: Ifade,
        sona: Ifade,
        govde: Vec<Cumle>,
        satir: usize,
    },
    /// `<koşul> olduğu sürece` + blok
    OlduguSurece { kosul: Ifade, govde: Vec<Cumle>, satir: usize },
    /// `<koşul> olana kadar tekrarla` + blok (K-006). Koşul her turdan ÖNCE
    /// sınanır; doğruysa döngü biter.
    OlanaKadar { kosul: Ifade, govde: Vec<Cumle>, satir: usize },
    /// `... ise / değilse ... ise / değilse` zinciri (K-005)
    Ise {
        kollar: Vec<KosulKolu>,
        degilse: Option<Vec<Cumle>>,
        satir: usize,
    },
    /// `<ad-ekli> <ifade> artır`
    Artir { ifade: Ifade, miktar: Ifade, satir: usize },
    /// `<ad-ekli> <ifade> azalt`
    Azalt { ifade: Ifade, miktar: Ifade, satir: usize },
    /// `"..." diye sor` — son cevap örtük "yanıt" adına bağlanır (K-007).
    Sor { istem: Ifade, satir: usize },
    /// `sayılara 5 ekle` — listeye öğe ekler.
    Ekle { hedef: Ifade, deger: Ifade, satir: usize },
    /// `her sayı için` — koleksiyon döngüsü. `kaynak` None ise örtük çoğul
    /// kuralı (K-013) uygulanır: çözümleyici "sayılar"ı bulup doldurur.
    HerBiri {
        ad: String,
        kaynak: Option<Ifade>,
        govde: Vec<Cumle>,
        satir: usize,
    },
    /// `<konu> a göre` + `<değer> ise` kolları + isteğe bağlı `değilse` (K-021).
    Gore {
        konu: Ifade,
        kollar: Vec<(Ifade, Vec<Cumle>)>,
        degilse: Option<Vec<Cumle>>,
        satir: usize,
    },
    /// `işlem <ad>` tanımı — hoist ile Program.islemler'e taşınır.
    IslemTanimi(Islem),
    /// `yapı <Ad>` tanımı — hoist ile Program.yapilar'a taşınır.
    YapiTanimi(Yapi),
    /// `test "..."` bloğu — hoist ile Program.testler'e taşınır.
    TestBlogu(Test),
    /// `kare 16 ya eşit olmalı` — doğrulama (K-025). Koşul tutmazsa D001.
    Olmali { kosul: Ifade, satir: usize },
    /// `ayşenin adı "Ayşe" olsun` — alan yazma (K-020).
    AlanAta {
        nesne: Ifade,
        alan: String,
        deger: Ifade,
        satir: usize,
    },
    /// `sonucu döndür` — yalnız işlem içinde geçerli.
    Dondur { deger: Ifade, satir: usize },
    /// `sonucu toplamı sayıların adedine böl` — payı paydaya bölüp hedefe atar.
    BolVeAta {
        hedef: String,
        pay: Ifade,
        payda: Ifade,
        satir: usize,
    },
    /// Değer beklemeyen işlem çağrısı cümlesi: `"Ayşe" ve 10 ile selamla`.
    CagriCumlesi { cagri: Ifade, satir: usize },
    /// `programı bitir` — programı olağan biçimde sonlandırır (K-024).
    ProgramiBitir { satir: usize },
    /// `yaşların "Ayşe" değeri 10 olsun` — sözlüğe yazma (K-015).
    SozlukAta {
        sozluk: Ifade,
        anahtar: Ifade,
        deger: Ifade,
        satir: usize,
    },
    /// `"X" dosyasına ... yaz/ekle` — hedefli yazma (K-019).
    /// `ekleme` true ise sona ekler, değilse dosyayı baştan yazar.
    DosyayaYaz {
        yol: Ifade,
        icerik: Ifade,
        ekleme: bool,
        satir: usize,
    },
}
