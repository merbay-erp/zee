//! Türkçe programlama dili — bootstrap derleyici (Stage 0).
//!
//! Boru hattı (master plan bölüm 11'in v0 dilimi):
//! kaynak → sözcükleyici → ayrıştırıcı → ad çözümleme + tür denetimi → yorumlayıcı.

pub mod agac;
pub mod ayristirici;
pub mod bicimleyici;
pub mod cozumleyici;
pub mod sozcukleyici;
pub mod tani;
pub mod yorumlayici;

use agac::{Cumle, Program};
use tani::Tani;

/// Kaynağı çalıştırılabilir programa derler (sözcükle + ayrıştır + denetle).
/// İşlem tanımları Program.islemler'e kaldırılır (hoist).
pub fn kaynagi_derle(kaynak: &str) -> Result<Program, Tani> {
    let tokenlar = sozcukleyici::sozcukle(kaynak)?;
    let cumleler = ayristirici::ayristir(tokenlar)?;

    let mut islemler = std::collections::HashMap::new();
    let mut yapilar: Vec<agac::Yapi> = Vec::new();
    let mut testler: Vec<agac::Test> = Vec::new();
    let mut kalan = Vec::new();
    for cumle in cumleler {
        match cumle {
            Cumle::IslemTanimi(islem) => {
                if islemler.contains_key(&islem.ad) {
                    return Err(Tani::yeni(
                        "A005",
                        format!("\"{}\" işlemi birden çok kez tanımlandı.", islem.ad),
                        islem.satir,
                        1,
                        1,
                    ));
                }
                islemler.insert(islem.ad.clone(), islem);
            }
            Cumle::TestBlogu(test) => testler.push(test),
            Cumle::YapiTanimi(yapi) => {
                if yapilar.iter().any(|y| y.ad == yapi.ad) {
                    return Err(Tani::yeni(
                        "A006",
                        format!("\"{}\" yapısı birden çok kez tanımlandı.", yapi.ad),
                        yapi.satir,
                        1,
                        1,
                    ));
                }
                yapilar.push(yapi);
            }
            baska => kalan.push(baska),
        }
    }

    let mut program = Program { cumleler: kalan, islemler, yapilar, testler };
    cozumleyici::denetle(&mut program)?;
    Ok(program)
}

/// Kaynağı denetler, çalıştırmaz.
pub fn kaynagi_denetle(kaynak: &str) -> Result<(), Tani> {
    kaynagi_derle(kaynak).map(|_| ())
}

/// Kaynağı uçtan uca çalıştırır; çıktı satırlarını döndürür.
pub fn kaynagi_calistir(kaynak: &str) -> Result<Vec<String>, Tani> {
    kaynagi_calistir_girdiyle(kaynak, Vec::new())
}

/// Kaynağı hazır girdi satırlarıyla çalıştırır ("diye sor" cevapları sırayla
/// bu listeden gelir); istemler de çıktıya dahildir.
pub fn kaynagi_calistir_girdiyle(kaynak: &str, girdiler: Vec<String>) -> Result<Vec<String>, Tani> {
    let program = kaynagi_derle(kaynak)?;
    let mut io = yorumlayici::ToplayanIo::yeni(girdiler);
    yorumlayici::calistir_io(&program, &mut io)?;
    Ok(io.cikti)
}

/// Bir testin sonucu: `hata` None ise geçti.
pub struct TestSonucu {
    pub ad: String,
    pub hata: Option<Tani>,
}

/// Programın testlerini koşar. v0: her test taze ortamda ve dış dünyaya
/// dokunmayan hermetik IO ile çalışır (gerçek dosya/ağ erişimi yok).
pub fn programi_dene(program: &Program) -> Vec<TestSonucu> {
    program
        .testler
        .iter()
        .map(|test| {
            let mut io = yorumlayici::ToplayanIo::yeni(Vec::new());
            let hata = yorumlayici::test_calistir(program, test, &mut io).err();
            TestSonucu { ad: test.ad.clone(), hata }
        })
        .collect()
}

/// Kaynağı derleyip testlerini koşar.
pub fn kaynagi_dene(kaynak: &str) -> Result<Vec<TestSonucu>, Tani> {
    let program = kaynagi_derle(kaynak)?;
    Ok(programi_dene(&program))
}
