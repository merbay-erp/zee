//! Soyut sözdizimi ağacı (AST).

use std::collections::HashMap;

/// Derlenmiş program: üst düzey cümleler + ada göre işlem tanımları.
/// İşlem gövdeleri buraya kaldırılır (hoist); hem denetleyici hem yorumlayıcı
/// aynı kayıttan okur.
#[derive(Debug, Clone)]
pub struct Program {
    pub cumleler: Vec<Cumle>,
    pub islemler: HashMap<String, Islem>,
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
    /// "3, 7, 1, 9 listesi" (K-012). v0: yalnız TamSayı öğeler.
    ListeSabiti(Vec<Ifade>),
    /// "boş liste".
    BosListe,
    /// Liste özellikleri: "sayıların adedi / ilki / sonu".
    Ozellik { nesne: Box<Ifade>, ozellik: Ozellik },
    /// Genitif aritmetik (K-008): "a ile b nin toplamı", "x in y ye bölümü".
    Aritmetik {
        islec: AritmetikIslec,
        sol: Box<Ifade>,
        sag: Box<Ifade>,
    },
    /// Metinden sayıya dönüşüm (K-009): "yanıtın sayısı".
    Sayisi(Box<Ifade>),
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
    Adet,
    Ilk,
    Son,
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
    /// `işlem <ad>` tanımı — hoist ile Program.islemler'e taşınır.
    IslemTanimi(Islem),
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
}
