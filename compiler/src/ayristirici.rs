//! Ayrıştırıcı (parser) — elle yazılmış yüklem-sonlu recursive descent
//! (ADR-002, RFC-0021/spec-20).
//!
//! Türkçe yüklem-sonlu olduğu için cümlenin türü satırın SON kelimesinden
//! anlaşılır: "... yaz", "... olsun", "... tekrarla", "... ise". Bu, İngilizce
//! dillerdeki "ilk keyword'e bak" yaklaşımının aynadaki karşılığıdır ve
//! deterministik ayrıştırmayı mümkün kılar.

mod cumle;
mod ifade;
mod kurtarma;

use self::ifade::*;

use crate::agac::{
    AritmetikIslec, Cumle, HttpYontemi, Ifade, Islec, Islem, IslemTuru, KosulKolu, Ozellik,
    Parametre, RotaErisimi, Test, Yapi,
};
use crate::intrinsic::{CSRF_BELIRTECI, HTTP_GETIR, PAROLA_DOGRULA, SENSOR_ACIK_MI};
use crate::ondalik::Ondalik;
use crate::sozcukleyici::{Token, TokenTur};
use crate::tani::Tani;

pub fn ayristir(tokenlar: Vec<Token>) -> Result<Vec<Cumle>, Tani> {
    ayristir_tohumla(tokenlar, Vec::new())
}

/// Birimlerden gelen işlem adlarıyla tohumlanmış ayrıştırma (RFC-0009):
/// çağrı tanıma "tanımlı işlem adıyla bitiyor mu?" kuralına dayandığı için
/// içe alınan adların ayrıştırma başlamadan bilinmesi gerekir.
pub fn ayristir_tohumla(tokenlar: Vec<Token>, islem_adlari: Vec<String>) -> Result<Vec<Cumle>, Tani> {
    let (cumleler, mut tanilar) = ayristir_kurtarmali(tokenlar, islem_adlari);
    if tanilar.is_empty() {
        Ok(cumleler)
    } else {
        Err(tanilar.remove(0))
    }
}

/// Hata KURTARMALI ayrıştırma (RFC-0010 §2.1): bir cümle ayrıştırılamazsa
/// tanı kaydedilir, yalnız o satır ve varsa kendisine ait dengeli gövde
/// atlanır. Aynı girintideki sağlam kardeşlerden sürülür.
pub fn ayristir_kurtarmali(
    tokenlar: Vec<Token>,
    mut islem_adlari: Vec<String>,
) -> (Vec<Cumle>, Vec<Tani>) {
    // Dosyanın kendi işlem başlıkları önden tohumlanır: çağrı tanıma tanım
    // sırasından bağımsızdır (karşılıklı özyineleme, ana-kod-üstte düzeni).
    for ad in islem_adlarini_tara(&tokenlar) {
        if !islem_adlari.contains(&ad) {
            islem_adlari.push(ad);
        }
    }
    let mut ayristirici = Ayristirici {
        tokenlar,
        konum: 0,
        derinlik: 0,
        islem_adlari,
        tanilar: Vec::new(),
    };
    let cumleler = ayristirici.koku_ayristir();
    ayristirici.tanilari_sirala();
    (cumleler, ayristirici.tanilar)
}

/// Üst düzeydeki `işlem <ad>` başlıklarını önden toplar: çağrı tanıma böylece
/// dosya GENELİNDE çalışır (tanım çağrıdan sonra da gelebilir; karşılıklı
/// özyineleme parse düzeyinde mümkün olur — RFC-0006 güncellemesi).
pub fn islem_adlarini_tara(tokenlar: &[Token]) -> Vec<String> {
    let mut adlar = Vec::new();
    let mut derinlik = 0usize;
    let mut satir_basi = true;
    let mut i = 0;
    while i < tokenlar.len() {
        match &tokenlar[i].tur {
            TokenTur::Girinti => {
                derinlik += 1;
                satir_basi = true;
            }
            TokenTur::Cikinti => {
                derinlik = derinlik.saturating_sub(1);
                satir_basi = true;
            }
            TokenTur::SatirSonu => satir_basi = true,
            TokenTur::Kelime(k)
                if satir_basi && derinlik == 0 && (k == "işlem" || k == "eylem") =>
            {
                let mut kelimeler = Vec::new();
                let mut j = i + 1;
                while let Some(token) = tokenlar.get(j) {
                    match &token.tur {
                        TokenTur::Kelime(ad) => kelimeler.push(ad.clone()),
                        _ => break,
                    }
                    j += 1;
                }
                if !kelimeler.is_empty() {
                    adlar.push(kelimeler.join(" "));
                }
                satir_basi = false;
            }
            _ => satir_basi = false,
        }
        i += 1;
    }
    adlar
}

/// Üst düzeydeki `X birimini kullan` satırlarını (ad, satır) olarak toplar.
/// Tam ayrıştırmadan ÖNCE çağrılır ki birimler yüklenip işlem adları
/// tohumlanabilsin; kalıp, ayrıştırıcıdaki Kullan koluyla birebir aynıdır.
pub fn kullanilan_birimler(tokenlar: &[Token]) -> Vec<(String, crate::agac::KullanimTuru, usize)> {
    let mut sonuc = Vec::new();
    let mut derinlik = 0usize;
    let mut satir_basi = true;
    let mut i = 0;
    while i < tokenlar.len() {
        match &tokenlar[i].tur {
            TokenTur::Girinti => {
                derinlik += 1;
                satir_basi = true;
            }
            TokenTur::Cikinti => {
                derinlik = derinlik.saturating_sub(1);
                satir_basi = true;
            }
            TokenTur::SatirSonu => satir_basi = true,
            TokenTur::Kelime(ad) if satir_basi && derinlik == 0 => {
                let tur = match tokenlar.get(i + 1).map(|t| &t.tur) {
                    Some(TokenTur::Kelime(k)) if k == "birimini" => {
                        Some(crate::agac::KullanimTuru::Birim)
                    }
                    Some(TokenTur::Kelime(k)) if k == "paketini" => {
                        Some(crate::agac::KullanimTuru::Paket)
                    }
                    _ => None,
                };
                let kullan = matches!(
                    tokenlar.get(i + 2).map(|t| &t.tur),
                    Some(TokenTur::Kelime(k)) if k == "kullan"
                );
                let satir_bitti = matches!(
                    tokenlar.get(i + 3).map(|t| &t.tur),
                    Some(TokenTur::SatirSonu) | None
                );
                if let Some(tur) = tur.filter(|_| kullan && satir_bitti) {
                    sonuc.push((ad.clone(), tur, tokenlar[i].satir));
                }
                satir_basi = false;
            }
            _ => satir_basi = false,
        }
        i += 1;
    }
    sonuc
}

struct Ayristirici {
    tokenlar: Vec<Token>,
    konum: usize,
    /// Blok derinliği: işlem tanımları yalnız en dış düzeyde.
    derinlik: usize,
    /// Dosya başlıkları ve yüklenen birimlerden önceden toplanmış işlem adları.
    /// Çağrı eşlemesi kaynak tanım sırasından bağımsızdır; tanım çağrıdan sonra
    /// gelebilir ve karşılıklı özyineleme bu ön-taramaya dayanır.
    islem_adlari: Vec<String>,
    /// Kurtarmalı parser'ın kaynak dosya başına sınırlı tanı biriktiricisi.
    tanilar: Vec<Tani>,
}

/// Sayı/sabit sonrası ayrık yazılan hal ekleri (K-011): "1 den", "100 e", "0 dan".
const AYRIK_EKLER: [&str; 8] = ["den", "dan", "ten", "tan", "e", "a", "ye", "ya"];

impl Ayristirici {
    fn bak(&self) -> &Token {
        &self.tokenlar[self.konum]
    }

    fn ilerle(&mut self) -> Token {
        let token = self.tokenlar[self.konum].clone();
        if self.konum + 1 < self.tokenlar.len() {
            self.konum += 1;
        }
        token
    }

    /// Bir satırın tokenlarını (SatirSonu hariç) toplar.
    fn satir_oku(&mut self) -> Vec<Token> {
        let mut satir = Vec::new();
        loop {
            match self.bak().tur {
                TokenTur::SatirSonu => {
                    self.ilerle();
                    break;
                }
                TokenTur::DosyaSonu => break,
                _ => satir.push(self.ilerle()),
            }
        }
        satir
    }

    /// Girintili alt bloğu ayrıştırır (başındaki Girinti ve sonundaki Cikinti dahil).
    fn alt_blok(&mut self, ana_satir: usize) -> Result<Vec<Cumle>, Tani> {
        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "Bu satırdan sonra girintili bir blok bekleniyor.".into(),
                    ana_satir,
                    1,
                    1,
                )
                .onerili("Alt satırları 4 boşluk içeriden yaz.".into()))
            }
        }
        self.derinlik += 1;
        let govde = self.blok_ayristir();
        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }
        Ok(govde)
    }

    /// `işlem <çok kelimeli ad>` + gövde. Gövdenin başındaki "X al" satırları
    /// parametre bildirimidir.
    fn islem_ayristir(&mut self, tur: IslemTuru) -> Result<Cumle, Tani> {
        let mut baslik = self.satir_oku();
        let satir = baslik.first().map(|t| t.satir).unwrap_or(1);
        if self.derinlik > 0 {
            return Err(Tani::yeni(
                "S021",
                "İşlem tanımı en dış düzeyde olmalı.".into(),
                satir,
                1,
                1,
            ));
        }
        baslik.remove(0); // "işlem" / "eylem"
        let mut ad_kelimeleri = Vec::new();
        for token in &baslik {
            match &token.tur {
                TokenTur::Kelime(k) => ad_kelimeleri.push(k.clone()),
                _ => {
                    return Err(Tani::yeni(
                        "S022",
                        "İşlem adı yalnız kelimelerden oluşur.".into(),
                        token.satir,
                        token.sutun,
                        token.uzunluk,
                    ))
                }
            }
        }
        if ad_kelimeleri.is_empty() {
            return Err(Tani::yeni(
                "S022",
                "İşlemin bir adı olmalı.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: işlem ortalamayı hesapla".into()));
        }
        let ad = ad_kelimeleri.join(" ");
        if !self.islem_adlari.contains(&ad) {
            self.islem_adlari.push(ad.clone());
        }

        // Gövde.
        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "İşlem başlığından sonra girintili bir gövde bekleniyor.".into(),
                    satir,
                    1,
                    1,
                ))
            }
        }
        self.derinlik += 1;

        // Parametre satırları: `<ad> al` ya da K-083 açık sözleşmesi
        // `<ad> <tür yazımı> olarak al`.
        let mut parametreler = Vec::new();
        loop {
            let satir_tokenlari = self.tokenlar[self.konum..]
                .iter()
                .take_while(|token| !matches!(token.tur, TokenTur::SatirSonu))
                .collect::<Vec<_>>();
            let kelimeler = satir_tokenlari
                .iter()
                .map(|token| match &token.tur {
                    TokenTur::Kelime(kelime) => Some(kelime.as_str()),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>();
            let param = kelimeler.and_then(|kelimeler| {
                let ilk = satir_tokenlari.first()?;
                match kelimeler.as_slice() {
                    [ad, al] if *al == "al" => Some(Parametre {
                        ad: yalin_ad(ad),
                        tur_yazimi: None,
                        satir: ilk.satir,
                    }),
                    [ad, ortalar @ .., olarak, al]
                        if !ortalar.is_empty() && *olarak == "olarak" && *al == "al" =>
                    {
                        Some(Parametre {
                            ad: yalin_ad(ad),
                            tur_yazimi: Some(ortalar.join(" ")),
                            satir: ilk.satir,
                        })
                    }
                    _ => None,
                }
            });
            match param {
                Some(p) => {
                    parametreler.push(p);
                    self.satir_oku();
                }
                None => break,
            }
        }

        // K-086 açık dönüş sözleşmesi, parametrelerden hemen sonra gelir:
        // `<Tür> döndürür` ya da değer üretmeyen işlem için `değer döndürmez`.
        while matches!(self.bak().tur, TokenTur::SatirSonu) {
            self.ilerle();
        }
        let satir_tokenlari = self.tokenlar[self.konum..]
            .iter()
            .take_while(|token| !matches!(token.tur, TokenTur::SatirSonu))
            .collect::<Vec<_>>();
        let kelimeler = satir_tokenlari
            .iter()
            .map(|token| match &token.tur {
                TokenTur::Kelime(kelime) => Some(kelime.as_str()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>();
        let donus = kelimeler.and_then(|kelimeler| {
            let ilk = satir_tokenlari.first()?;
            match kelimeler.as_slice() {
                [deger, dondurmez] if *deger == "değer" && *dondurmez == "döndürmez" => {
                    Some(("DeğerDöndürmez".to_string(), ilk.satir))
                }
                [tur @ .., dondurur] if !tur.is_empty() && *dondurur == "döndürür" => {
                    Some((tur.join(" "), ilk.satir))
                }
                _ => None,
            }
        });
        let (donus_turu_yazimi, donus_satiri) = match donus {
            Some((yazim, satir)) => {
                self.satir_oku();
                (Some(yazim), Some(satir))
            }
            None => (None, None),
        };

        let govde = self.blok_ayristir();
        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }

        Ok(Cumle::IslemTanimi(Islem {
            ad,
            parametreler,
            tur,
            disari_acik: false,
            donus_turu_yazimi,
            donus_satiri,
            govde,
            satir,
        }))
    }

    /// `yapı <Ad>` + alan satırları (`ad Metin`, `yaş TamSayı`).
    fn yapi_ayristir(&mut self) -> Result<Cumle, Tani> {
        let baslik = self.satir_oku();
        let satir = baslik.first().map(|t| t.satir).unwrap_or(1);
        if self.derinlik > 0 {
            return Err(Tani::yeni(
                "S021",
                "Yapı tanımı en dış düzeyde olmalı.".into(),
                satir,
                1,
                1,
            ));
        }
        let ad = match baslik.get(1).map(|t| &t.tur) {
            Some(TokenTur::Kelime(ad)) if baslik.len() == 2 => ad.clone(),
            _ => {
                return Err(Tani::yeni(
                    "S025",
                    "Yapı tanımı \"yapı <Ad>\" biçiminde başlar.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: yapı Öğrenci — tür adları büyük harfle başlar.".into()))
            }
        };

        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "Yapı başlığından sonra girintili alan listesi bekleniyor.".into(),
                    satir,
                    1,
                    1,
                ))
            }
        }
        self.derinlik += 1;

        let mut alanlar = Vec::new();
        loop {
            match &self.bak().tur {
                TokenTur::Cikinti | TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu => {
                    self.ilerle();
                }
                _ => {
                    let alan_satiri = self.satir_oku();
                    let alan_no = alan_satiri.first().map(|t| t.satir).unwrap_or(satir);
                    match (alan_satiri.first().map(|t| &t.tur), alan_satiri.get(1).map(|t| &t.tur)) {
                        (Some(TokenTur::Kelime(alan)), Some(TokenTur::Kelime(tur)))
                            if alan_satiri.len() == 2 =>
                        {
                            alanlar.push((alan.clone(), tur.clone()));
                        }
                        _ => {
                            self.tani_kaydet(Tani::yeni(
                                "S025",
                                "Yapı alanı \"<ad> <Tür>\" biçiminde yazılır.".into(),
                                alan_no,
                                1,
                                1,
                            )
                            .onerili("Örnek: yaş TamSayı".into()));
                            self.bekleyen_govdeyi_atla();
                            if self.tani_limiti_doldu() {
                                break;
                            }
                        }
                    }
                }
            }
        }

        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }
        Ok(Cumle::YapiTanimi(Yapi { ad, alanlar, satir }))
    }

    /// `eşzamanlı olarak` bloğu: her satır `<ad> <ifade>` görev bağlamasıdır
    /// (RFC-0011 §1). K-090 çalıştırıcısı görevleri `hepsini bekle`de
    /// deterministik, tek iş parçacıklı ve işbirlikli olarak ilerletir.
    fn eszamanli_ayristir(&mut self, satir: usize) -> Result<Cumle, Tani> {
        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S038",
                    "\"eşzamanlı olarak\" satırından sonra girintili görevler gelir.".into(),
                    satir,
                    1,
                    1,
                ))
            }
        }
        self.derinlik += 1;
        let mut gorevler = Vec::new();
        let mut gorev_tanilari = Vec::new();
        loop {
            match &self.bak().tur {
                TokenTur::Cikinti | TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu => {
                    self.ilerle();
                }
                _ => {
                    let gorev_satiri = self.satir_oku();
                    let gorev_no = gorev_satiri.first().map(|t| t.satir).unwrap_or(satir);
                    let sonuc = match gorev_satiri.first().map(|t| &t.tur) {
                        Some(TokenTur::Kelime(ad)) if gorev_satiri.len() >= 2 => {
                            ile_ifadesi(&gorev_satiri[1..], gorev_no, &self.islem_adlari)
                                .map(|deger| (ad.clone(), deger, gorev_no))
                        }
                        _ => Err(Tani::yeni(
                                "S038",
                                "Görev satırı \"<ad> <ifade>\" biçimindedir.".into(),
                                gorev_no,
                                1,
                                1,
                            )),
                    };
                    match sonuc {
                        Ok(gorev) => gorevler.push(gorev),
                        Err(tani) => {
                            gorev_tanilari.push(tani);
                            self.bekleyen_govdeyi_atla();
                            if self.tanilar.len() + gorev_tanilari.len()
                                >= crate::tani::AZAMI_TANI_SAYISI
                            {
                                break;
                            }
                        }
                    }
                }
            }
        }
        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }
        if gorevler.is_empty() {
            let mut tanilar = gorev_tanilari.into_iter();
            let ilk = tanilar.next().unwrap_or_else(|| {
                Tani::yeni(
                    "S038",
                    "Eşzamanlı blokta en az bir görev olmalı.".into(),
                    satir,
                    1,
                    1,
                )
            });
            self.tanilari_kaydet(tanilar.collect());
            return Err(ilk);
        }
        self.tanilari_kaydet(gorev_tanilari);
        Ok(Cumle::Eszamanli { gorevler, satir })
    }

    /// `test "<açıklama>"` + gövde (K-025).
    fn test_ayristir(&mut self) -> Result<Cumle, Tani> {
        let baslik = self.satir_oku();
        let satir = baslik.first().map(|t| t.satir).unwrap_or(1);
        if self.derinlik > 0 {
            return Err(Tani::yeni(
                "S021",
                "Test bloğu en dış düzeyde olmalı.".into(),
                satir,
                1,
                1,
            ));
        }
        let ad = match baslik.get(1).map(|t| &t.tur) {
            Some(TokenTur::Metin(ad)) if baslik.len() == 2 => ad.clone(),
            _ => {
                return Err(Tani::yeni(
                    "S026",
                    "Test bloğu \"test \\\"<açıklama>\\\"\" biçiminde başlar.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: test \"toplama doğru çalışır\"".into()))
            }
        };
        let govde = self.alt_blok(satir)?;
        Ok(Cumle::TestBlogu(Test { ad, govde, satir }))
    }

    /// `<koşul> olmalı` — doğrulama cümlesi (K-025).
    fn olmali_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "olmalı"
        let kosul = kosul_ifadesi(&tokenlar, satir)?;
        Ok(Cumle::Olmali { kosul, satir })
    }

    fn dondur_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "döndür"

        // `<mesaj> hatasını döndür` — geriye uyumlu Sonuç-hata dönüşü.
        // K-091 yapılandırılmış biçimi:
        // `<KOD> kodlu <mesaj> hatasını [<hata> nedeniyle] [<sözlük> verisiyle] döndür`.
        if let Some(hata_konumu) = tokenlar.iter().position(|t| kelime_mi(t, "hatasını")) {
            let bas = &tokenlar[..hata_konumu];
            let ekler = &tokenlar[hata_konumu + 1..];
            let kodlu = bas.iter().position(|t| kelime_mi(t, "kodlu"));
            let (kod, mesaj_tokenlari) = match kodlu {
                Some(konum) => {
                    if konum != 1 {
                        return Err(Tani::yeni(
                            "S044",
                            "Hata kodu tek bir metin sabiti olmalı.".into(),
                            satir,
                            1,
                            1,
                        )
                        .onerili("Örnek: \"DOSYA_YOK\" kodlu \"Dosya bulunamadı\" hatasını döndür".into()));
                    }
                    let kod = match &bas[0].tur {
                        TokenTur::Metin(kod) if gecerli_hata_kodu(kod) => kod.clone(),
                        TokenTur::Metin(_) => {
                            return Err(Tani::yeni(
                                "S044",
                                "Hata kodu A-Z ile başlamalı; yalnız A-Z, 0-9 ve _ içermeli.".into(),
                                satir,
                                bas[0].sutun,
                                bas[0].uzunluk,
                            )
                            .onerili("Örnek kod: \"DOSYA_YOK\"".into()))
                        }
                        _ => {
                            return Err(Tani::yeni(
                                "S044",
                                "Hata kodu kararlı bir metin sabiti olmalı.".into(),
                                satir,
                                bas[0].sutun,
                                bas[0].uzunluk,
                            )
                            .onerili("Örnek kod: \"DOSYA_YOK\"".into()))
                        }
                    };
                    (Some(kod), &bas[konum + 1..])
                }
                None => (None, bas),
            };
            let mesaj = ile_ifadesi(mesaj_tokenlari, satir, &self.islem_adlari)?;

            let neden_konumu = ekler.iter().position(|t| kelime_mi(t, "nedeniyle"));
            let veri_konumu = ekler.iter().position(|t| kelime_mi(t, "verisiyle"));
            if neden_konumu.is_some_and(|n| veri_konumu.is_some_and(|v| n > v))
                || ekler.iter().filter(|t| kelime_mi(t, "nedeniyle")).count() > 1
                || ekler.iter().filter(|t| kelime_mi(t, "verisiyle")).count() > 1
            {
                return Err(Tani::yeni(
                    "S044",
                    "Hata ekleri önce `nedeniyle`, sonra `verisiyle` yazılır.".into(),
                    satir,
                    1,
                    1,
                ));
            }
            if ekler.last().is_some_and(|t| {
                !kelime_mi(t, "nedeniyle") && !kelime_mi(t, "verisiyle")
            }) || (neden_konumu.is_none() && veri_konumu.is_none() && !ekler.is_empty())
            {
                return Err(Tani::yeni(
                    "S044",
                    "Hata ekleri `<hata> nedeniyle` ve `<sözlük> verisiyle` biçimindedir.".into(),
                    satir,
                    1,
                    1,
                ));
            }
            let neden = match neden_konumu {
                Some(konum) => Some(ile_ifadesi(
                    &ekler[..konum],
                    satir,
                    &self.islem_adlari,
                )?),
                None => None,
            };
            let veri = match veri_konumu {
                Some(konum) => {
                    let baslangic = neden_konumu.map_or(0, |n| n + 1);
                    Some(ile_ifadesi(
                        &ekler[baslangic..konum],
                        satir,
                        &self.islem_adlari,
                    )?)
                }
                None => None,
            };
            return Ok(Cumle::HataDondur { kod, mesaj, neden, veri, satir });
        }

        let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Dondur { deger, sonuca_sarmala: false, satir })
    }

    /// `sonucu toplamı sayıların adedine böl`
    fn bol_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "böl"
        if tokenlar.len() < 3 {
            return Err(Tani::yeni(
                "S023",
                "Bölme cümlesi \"<hedef> <pay> <payda> böl\" biçiminde yazılır.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: sonucu toplamı sayıların adedine böl".into()));
        }
        let hedef_token = tokenlar.remove(0);
        let hedef = match hedef_token.tur {
            TokenTur::Kelime(k) => yalin_ad(&k),
            _ => {
                return Err(Tani::yeni(
                    "S023",
                    "Bölmenin hedefi bir ad olmalı.".into(),
                    satir,
                    hedef_token.sutun,
                    hedef_token.uzunluk,
                ))
            }
        };
        let pay = tekil_ifade(tokenlar.remove(0))?;
        let payda = bolge_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::BolVeAta { hedef, pay, payda, satir })
    }

    // ---- cümle türleri ----

    fn yaz_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "yaz"

        // Çereze yazma (K-052): `"oturum" çerezine kimlik yaz`.
        if tokenlar.len() >= 3 && kelime_mi(&tokenlar[1], "çerezine") {
            let ad = tekil_ifade(tokenlar[0].clone())?;
            let deger = ile_ifadesi(&tokenlar[2..], satir, &self.islem_adlari)?;
            return Ok(Cumle::CerezYaz { ad, deger, satir });
        }

        // Hedefli yazma (K-019): `"X" dosyasına ... yaz`.
        if tokenlar.len() >= 3 && kelime_mi(&tokenlar[1], "dosyasına") {
            let yol = tekil_ifade(tokenlar[0].clone())?;
            let icerik = ile_ifadesi(&tokenlar[2..], satir, &self.islem_adlari)?;
            return Ok(Cumle::DosyayaYaz { yol, icerik, ekleme: false, satir });
        }

        let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Yaz { deger, satir })
    }

    fn olsun_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "olsun"

        // Sözlüğe yazma: `yaşların "Ayşe" değeri 10 olsun` (K-015).
        // Ayrım: "değeri"nden sonra "ile" geliyorsa bu bir sözlük ataması değil,
        // değeri `X in değeri ile ...` zinciri olan normal bir tanımdır.
        if tokenlar.len() >= 4
            && kelime_mi(&tokenlar[2], "değeri")
            && !kelime_mi(&tokenlar[3], "ile")
        {
            let sozluk = tekil_ifade(tokenlar[0].clone())?;
            let anahtar = tekil_ifade(tokenlar[1].clone())?;
            let deger = ile_ifadesi(&tokenlar[3..], satir, &self.islem_adlari)?;
            return Ok(Cumle::SozlukAta { sozluk, anahtar, deger, satir });
        }

        // Alan yazma: `ayşenin adı "Ayşe" olsun` (K-020). Kural: tam üç token,
        // ilki tamlayan ekli bir ad, ikincisi alan adı, üçüncüsü tek değer —
        // AMA kuyruk `yanıtın sayısı` gibi yapılı bir kalıpsa bu normal bir
        // değer tanımıdır ("tahmin yanıtın sayısı olsun"); kalıplar önce gelir.
        if tokenlar.len() == 3 {
            if let (TokenTur::Kelime(nesne), TokenTur::Kelime(alan)) =
                (&tokenlar[0].tur, &tokenlar[1].tur)
            {
                let kuyruk_kalip = yapili_kalip(&tokenlar[1..], &self.islem_adlari)?.is_some();
                if !kuyruk_kalip && tamlayan_ekli(nesne) && alan != "ile" {
                    let nesne = tekil_ifade(tokenlar[0].clone())?;
                    let alan = alan.clone();
                    let deger = tekil_ifade(tokenlar[2].clone())?;
                    return Ok(Cumle::AlanAta { nesne, alan, deger, satir });
                }
            }
        }

        if tokenlar.len() < 2 {
            return Err(Tani::yeni(
                "S008",
                "\"olsun\" ile değer tanımı için ad ve değer gerekir.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: isim \"Ayşe\" olsun".into()));
        }
        let ad_token = tokenlar.remove(0);
        let (ad, sutun, uzunluk) = match ad_token.tur {
            TokenTur::Kelime(k) => (k, ad_token.sutun, ad_token.uzunluk),
            _ => {
                return Err(Tani::yeni(
                    "S008",
                    "Değer tanımı bir adla başlamalı.".into(),
                    satir,
                    ad_token.sutun,
                    ad_token.uzunluk,
                )
                .onerili("Örnek: yaş 10 olsun".into()))
            }
        };
        let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Olsun { ad, deger, satir, sutun, uzunluk })
    }

    fn tekrarla_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "tekrarla"

        // <koşul> olana kadar tekrarla
        let n = tokenlar.len();
        if n >= 3
            && kelime_mi(&tokenlar[n - 1], "kadar")
            && kelime_mi(&tokenlar[n - 2], "olana")
        {
            tokenlar.truncate(n - 2);
            let kosul = kosul_ifadesi(&tokenlar, satir)?;
            let govde = self.alt_blok(satir)?;
            return Ok(Cumle::OlanaKadar { kosul, govde, satir });
        }

        // <ifade> kez tekrarla
        match tokenlar.last() {
            Some(t) if kelime_mi(t, "kez") => {
                tokenlar.pop();
            }
            _ => {
                return Err(Tani::yeni(
                    "S009",
                    "Döngü \"<n> kez tekrarla\" ya da \"<koşul> olana kadar tekrarla\" biçiminde yazılır.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnekler: 10 kez tekrarla · bildi doğru olana kadar tekrarla".into()))
            }
        }
        let adet = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        let govde = self.alt_blok(satir)?;
        Ok(Cumle::KezTekrarla { adet, govde, satir })
    }

    fn aralik_ayristir(&mut self, tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        // <koleksiyon>daki her <ad> için — açık kaynaklı döngü
        if tokenlar.len() == 4 && kelime_mi(&tokenlar[1], "her") {
            if let TokenTur::Kelime(kaynakli) = &tokenlar[0].tur {
                let kok = ["daki", "deki", "taki", "teki"]
                    .iter()
                    .find_map(|ek| kaynakli.strip_suffix(ek));
                if let (Some(kok), TokenTur::Kelime(ad)) = (kok, &tokenlar[2].tur) {
                    let t = &tokenlar[0];
                    let kaynak = Ifade::Degisken {
                        ham: kok.to_string(),
                        cozulmus: None,
                        sembol_kimligi: None,
                        satir: t.satir,
                        sutun: t.sutun,
                        uzunluk: t.uzunluk,
                    };
                    let ad = ad.clone();
                    let govde = self.alt_blok(satir)?;
                    return Ok(Cumle::HerBiri { ad, kaynak: Some(kaynak), govde, satir });
                }
            }
        }

        // her <ad> için — koleksiyon döngüsü (örtük çoğul, K-013)
        if tokenlar.len() == 3 && kelime_mi(&tokenlar[0], "her") {
            let ad = match &tokenlar[1].tur {
                TokenTur::Kelime(k) => k.clone(),
                _ => {
                    return Err(Tani::yeni(
                        "S010",
                        "\"her ... için\" döngüsünde bir ad bekleniyor.".into(),
                        satir,
                        1,
                        1,
                    ))
                }
            };
            let govde = self.alt_blok(satir)?;
            return Ok(Cumle::HerBiri { ad, kaynak: None, govde, satir });
        }

        // Beklenen biçim: <a> den <b> e kadar her <ad> için
        let hata = || {
            Tani::yeni(
                "S010",
                "Döngü tanınmadı.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnekler: her sayı için · 1 den 100 e kadar her sayı için".into())
        };
        let mut t = tokenlar.into_iter().peekable();

        let bastan = tekil_ifade(t.next().ok_or_else(hata)?)?;
        eslesen_ek(&mut t).ok_or_else(hata)?; // den/dan
        let sona = tekil_ifade(t.next().ok_or_else(hata)?)?;
        eslesen_ek(&mut t).ok_or_else(hata)?; // e/a
        bekle_kelime(&mut t, "kadar").ok_or_else(hata)?;
        bekle_kelime(&mut t, "her").ok_or_else(hata)?;
        let ad = match t.next().map(|x| x.tur) {
            Some(TokenTur::Kelime(k)) => k,
            _ => return Err(hata()),
        };
        bekle_kelime(&mut t, "için").ok_or_else(hata)?;
        if t.next().is_some() {
            return Err(hata());
        }

        let govde = self.alt_blok(satir)?;
        Ok(Cumle::AralikDongusu { ad, bastan, sona, govde, satir })
    }

    fn surece_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "sürece"
        match tokenlar.last() {
            Some(t) if kelime_mi(t, "olduğu") => {
                tokenlar.pop();
            }
            _ => {
                return Err(Tani::yeni(
                    "S011",
                    "Koşullu döngü \"<koşul> olduğu sürece\" biçiminde yazılır.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: sayaç 0 dan büyük olduğu sürece".into()))
            }
        }
        let kosul = kosul_ifadesi(&tokenlar, satir)?;
        let govde = self.alt_blok(satir)?;
        Ok(Cumle::OlduguSurece { kosul, govde, satir })
    }

    fn artir_azalt_ayristir(
        &mut self,
        mut tokenlar: Vec<Token>,
        satir: usize,
        artir: bool,
    ) -> Result<Cumle, Tani> {
        tokenlar.pop(); // eylem
        if tokenlar.len() < 2 {
            return Err(Tani::yeni(
                "S012",
                "Artırma/azaltma için hedef ve miktar gerekir.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: toplamı sayıyla artır — ya da: sayacı 1 azalt".into()));
        }
        let hedef_token = tokenlar.remove(0);
        let ifade = tekil_ifade(hedef_token)?;
        let miktar = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        if artir {
            Ok(Cumle::Artir { ifade, miktar, satir })
        } else {
            Ok(Cumle::Azalt { ifade, miktar, satir })
        }
    }

    fn ekle_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "ekle"

        // Dosyanın sonuna ekleme (K-019): `"X" dosyasına ... ekle`.
        if tokenlar.len() >= 3 && kelime_mi(&tokenlar[1], "dosyasına") {
            let yol = tekil_ifade(tokenlar[0].clone())?;
            let icerik = ile_ifadesi(&tokenlar[2..], satir, &self.islem_adlari)?;
            return Ok(Cumle::DosyayaYaz { yol, icerik, ekleme: true, satir });
        }

        if tokenlar.len() < 2 {
            return Err(Tani::yeni(
                "S018",
                "Ekleme için hedef liste ve değer gerekir.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: sayılara 5 ekle".into()));
        }
        let hedef = tekil_ifade(tokenlar.remove(0))?;
        let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Ekle { hedef, deger, satir })
    }

    /// `<konu> a göre` — desen eşleştirme başlığı (K-021). Gövde kolları
    /// `<değer> ise` başlıklı bloklardır; varsayılan kol `değilse`.
    fn gore_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "göre"
        // Yönelme eki konuya bitişik ("şekle") ya da sabit sonrası ayrık olabilir.
        if let Some(t) = tokenlar.last() {
            if matches!(&t.tur, TokenTur::Kelime(k) if AYRIK_EKLER.contains(&k.as_str())) {
                tokenlar.pop();
            }
        }
        if tokenlar.len() != 1 {
            return Err(Tani::yeni(
                "S024",
                "Eşleştirme \"<konu> a göre\" biçiminde başlar.".into(),
                satir,
                1,
                1,
            )
            .onerili("Örnek: şekle göre".into()));
        }
        let konu_tokeni = tokenlar.pop().ok_or_else(|| {
            Tani::yeni("S024", "Eşleştirme konusu eksik.".into(), satir, 1, 1)
        })?;
        let konu = tekil_ifade(konu_tokeni)?;

        // Gövde: kollar.
        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "\"göre\" başlığından sonra girintili kollar bekleniyor.".into(),
                    satir,
                    1,
                    1,
                ))
            }
        }
        self.derinlik += 1;

        let mut kollar = Vec::new();
        let mut degilse = None;
        let mut kol_tanilari = Vec::new();
        loop {
            match &self.bak().tur {
                TokenTur::Cikinti | TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu => {
                    self.ilerle();
                }
                _ => {
                    let kol_satiri = self.satir_oku();
                    let kol_no = kol_satiri.first().map(|t| t.satir).unwrap_or(satir);
                    if kol_satiri.len() == 1 && kelime_mi(&kol_satiri[0], "değilse") {
                        match self.alt_blok(kol_no) {
                            Ok(govde) => degilse = Some(govde),
                            Err(tani) => kol_tanilari.push(tani),
                        }
                        continue;
                    }
                    if kol_satiri.len() == 2 && kelime_mi(&kol_satiri[1], "ise") {
                        match tekil_ifade(kol_satiri[0].clone())
                            .and_then(|deger| self.alt_blok(kol_no).map(|govde| (deger, govde)))
                        {
                            Ok(kol) => kollar.push(kol),
                            Err(tani) => {
                                kol_tanilari.push(tani);
                                self.bekleyen_govdeyi_atla();
                            }
                        }
                        continue;
                    }
                    kol_tanilari.push(Tani::yeni(
                        "S024",
                        "Eşleştirme kolu \"<değer> ise\" ya da \"değilse\" olmalı.".into(),
                        kol_no,
                        1,
                        1,
                    )
                    .onerili("Örnek:\n    \"kare\" ise\n        \"4 köşesi var\" yaz".into()));
                    self.bekleyen_govdeyi_atla();
                    if self.tanilar.len() + kol_tanilari.len()
                        >= crate::tani::AZAMI_TANI_SAYISI
                    {
                        break;
                    }
                }
            }
        }

        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }
        if kollar.is_empty() {
            let mut tanilar = kol_tanilari.into_iter();
            let ilk = tanilar.next().unwrap_or_else(|| {
                Tani::yeni(
                    "S024",
                    "Eşleştirmede en az bir \"<değer> ise\" kolu olmalı.".into(),
                    satir,
                    1,
                    1,
                )
            });
            self.tanilari_kaydet(tanilar.collect());
            return Err(ilk);
        }
        self.tanilari_kaydet(kol_tanilari);
        Ok(Cumle::Gore { konu, kollar, degilse, satir })
    }

    fn sor_ayristir(&mut self, mut tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        tokenlar.pop(); // "sor"
        match tokenlar.last() {
            Some(t) if kelime_mi(t, "diye") => {
                tokenlar.pop();
            }
            _ => {
                return Err(Tani::yeni(
                    "S017",
                    "Soru \"<metin> diye sor\" biçiminde yazılır.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: \"Adın ne?\" diye sor — cevap \"yanıt\" adıyla kullanılır.".into()))
            }
        }
        let istem = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
        Ok(Cumle::Sor { istem, satir })
    }

    fn ise_ayristir(&mut self, tokenlar: Vec<Token>, satir: usize) -> Result<Cumle, Tani> {
        let kosul = kosul_ifadesi(&tokenlar, satir)?;
        let govde = self.alt_blok(satir)?;
        let mut kollar = vec![KosulKolu { kosul, govde }];
        let mut degilse = None;

        // "değilse" zinciri: aynı düzeyde peş peşe gelebilir.
        loop {
            match &self.bak().tur {
                TokenTur::Kelime(k) if k == "değilse" => {
                    let devam_satiri = self.bak().satir;
                    let mut satir_tokenlari = self.satir_oku();
                    satir_tokenlari.remove(0); // "değilse"
                    if satir_tokenlari.is_empty() {
                        match self.alt_blok(devam_satiri) {
                            Ok(govde) => degilse = Some(govde),
                            Err(tani) => self.tani_kaydet(tani),
                        }
                        break;
                    } else {
                        match kosul_ifadesi(&satir_tokenlari, devam_satiri)
                            .and_then(|kosul| {
                                self.alt_blok(devam_satiri)
                                    .map(|govde| KosulKolu { kosul, govde })
                            })
                        {
                            Ok(kol) => kollar.push(kol),
                            Err(tani) => {
                                self.tani_kaydet(tani);
                                self.bekleyen_govdeyi_atla();
                            }
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(Cumle::Ise { kollar, degilse, satir })
    }
}
