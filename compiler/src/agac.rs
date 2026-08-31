//! Soyut sözdizimi ağacı (AST).

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
    /// Genitif aritmetik (K-008): "a ile b nin toplamı", "x in y ye bölümü".
    Aritmetik {
        islec: AritmetikIslec,
        sol: Box<Ifade>,
        sag: Box<Ifade>,
    },
    /// Metinden sayıya dönüşüm (K-009): "yanıtın sayısı".
    Sayisi(Box<Ifade>),
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
}
