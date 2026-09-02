//! K-092 keyfî hassasiyetli onluk sayı çekirdeği.
//!
//! `Ondalik`, ikili kayan nokta kullanmaz: imzalı keyfî uzunlukta katsayı ve
//! onluk ölçek taşır. Sonlu işlemler tamdır; yalnız sonsuz açılımlı bölme
//! RFC-0013'teki açık 34 anlamlı hane bağlamında yuvarlanır.

use num_bigint::{BigInt, Sign};
use num_traits::{Signed, ToPrimitive};
use std::cmp::Ordering;
use std::str::FromStr;

pub const BOLUM_ANLAMLI_HANE: u32 = 34;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ondalik {
    govde: BigInt,
    olcek: u32,
}

impl Ondalik {
    /// Katsayı/ölçek çiftini kanonikleştirir. Ölçek en az birdir; böylece
    /// TamSayıdan genişleyen `2`, Ondalık olarak `2,0` görünür kalır.
    pub fn yeni(mut govde: BigInt, mut olcek: u32) -> Self {
        if olcek == 0 {
            govde *= 10u8;
            olcek = 1;
        }
        if govde == BigInt::ZERO {
            return Self { govde, olcek: 1 };
        }
        if olcek > 1 {
            let sondaki_sifir = govde
                .abs()
                .to_str_radix(10)
                .bytes()
                .rev()
                .take_while(|r| *r == b'0')
                .count();
            let atilacak = u32::try_from(sondaki_sifir)
                .unwrap_or(u32::MAX)
                .min(olcek - 1);
            if atilacak > 0 {
                govde /= on_us(atilacak);
                olcek -= atilacak;
            }
        }
        Self { govde, olcek }
    }

    pub fn govdeden(govde: &str, olcek: u32) -> Option<Self> {
        BigInt::from_str(govde)
            .ok()
            .map(|govde| Self::yeni(govde, olcek))
    }

    pub fn tam(sayi: i64) -> Self {
        Self::yeni(BigInt::from(sayi), 0)
    }

    pub fn katsayidan(sayi: i64, olcek: u32) -> Self {
        Self::yeni(BigInt::from(sayi), olcek)
    }

    /// Kullanıcı metni: isteğe bağlı `-`, rakamlar ve isteğe bağlı Türkçe
    /// ondalık virgülü. Nokta kabul edilmez.
    pub fn metinden(metin: &str) -> Option<Self> {
        let (tam, kesir) = metin.split_once(',').unwrap_or((metin, "0"));
        if tam.is_empty() || kesir.is_empty() || metin.matches(',').count() > 1 {
            return None;
        }
        let (eksi, tam_rakamlar) = tam.strip_prefix('-').map_or((false, tam), |t| (true, t));
        if tam_rakamlar.is_empty()
            || !tam_rakamlar.chars().all(|k| k.is_ascii_digit())
            || !kesir.chars().all(|k| k.is_ascii_digit())
        {
            return None;
        }
        let olcek = u32::try_from(kesir.len()).ok()?;
        let mut govde = BigInt::from_str(&format!("{}{}", tam_rakamlar, kesir)).ok()?;
        if eksi {
            govde = -govde;
        }
        Some(Self::yeni(govde, olcek))
    }

    pub fn negatif_mi(&self) -> bool {
        self.govde.sign() == Sign::Minus
    }

    pub fn sifir_mi(&self) -> bool {
        self.govde == BigInt::ZERO
    }

    /// Değer grafiği kaynak hesabı için katsayının yaklaşık dinamik boyutu.
    pub fn yaklasik_heap_bayti(&self) -> usize {
        usize::try_from(self.govde.bits().saturating_add(7) / 8).unwrap_or(usize::MAX)
    }

    /// `metne`/`json_metni` çağrısından önce allocation bütçesi denetimi için
    /// sonucu kapsayan ucuz bir üst sınır verir.
    pub fn metin_bayti_ust_siniri(&self) -> usize {
        let rakam = self
            .katsayi_onluk_hanesi_ust_siniri()
            .max((self.olcek as usize).saturating_add(1));
        rakam
            .saturating_add(usize::from(self.negatif_mi()))
            .saturating_add(1)
    }

    /// Kuruşlu basımın işaret, ayraç ve olası binlik noktaları dahil üst sınırı.
    pub fn kuruslu_metin_bayti_ust_siniri(&self, binlikli: bool) -> usize {
        let kurus_hanesi = if self.olcek > 2 {
            self.katsayi_onluk_hanesi_ust_siniri().saturating_add(1)
        } else {
            self.katsayi_onluk_hanesi_ust_siniri()
                .saturating_add((2 - self.olcek) as usize)
        }
        .max(3);
        let tam_hanesi = kurus_hanesi.saturating_sub(2).max(1);
        let binlik = if binlikli {
            tam_hanesi.saturating_sub(1) / 3
        } else {
            0
        };
        usize::from(self.negatif_mi())
            .saturating_add(tam_hanesi)
            .saturating_add(binlik)
            .saturating_add(3)
    }

    fn katsayi_onluk_hanesi_ust_siniri(&self) -> usize {
        let bit = usize::try_from(self.govde.bits()).unwrap_or(usize::MAX);
        bit.saturating_mul(30_103)
            .saturating_add(99_999)
            .checked_div(100_000)
            .unwrap_or(usize::MAX)
            .max(1)
    }

    pub fn metne(&self) -> String {
        let isaret = if self.negatif_mi() { "-" } else { "" };
        let mut rakamlar = self.govde.abs().to_str_radix(10);
        let olcek = self.olcek as usize;
        if rakamlar.len() <= olcek {
            let sifirlar = "0".repeat(olcek + 1 - rakamlar.len());
            rakamlar = format!("{}{}", sifirlar, rakamlar);
        }
        let ayrim = rakamlar.len() - olcek;
        format!("{}{},{}", isaret, &rakamlar[..ayrim], &rakamlar[ayrim..])
    }

    pub fn json_metni(&self) -> String {
        self.metne().replace(',', ".")
    }

    pub fn karsilastir(&self, diger: &Self) -> Ordering {
        let (sol, sag, _) = hizala(self, diger);
        sol.cmp(&sag)
    }

    pub fn topla(&self, diger: &Self) -> Self {
        let (sol, sag, olcek) = hizala(self, diger);
        Self::yeni(sol + sag, olcek)
    }

    pub fn cikar(&self, diger: &Self) -> Self {
        let (sol, sag, olcek) = hizala(self, diger);
        Self::yeni(sol - sag, olcek)
    }

    pub fn carp(&self, diger: &Self) -> Option<Self> {
        let olcek = self.olcek.checked_add(diger.olcek)?;
        Some(Self::yeni(&self.govde * &diger.govde, olcek))
    }

    /// Sonlu onluk bölüm tam döner. Sonsuz açılım yalnız burada 34 anlamlı
    /// haneye, yarımlar sıfırdan uzağa yuvarlanır.
    pub fn bol(&self, diger: &Self) -> Option<Self> {
        if diger.sifir_mi() {
            return None;
        }
        let eksi = self.negatif_mi() != diger.negatif_mi();
        let mut pay = self.govde.abs() * on_us(diger.olcek);
        let mut payda = diger.govde.abs() * on_us(self.olcek);
        let ortak = obeb(pay.clone(), payda.clone());
        pay /= &ortak;
        payda /= ortak;

        if let Some(olcek) = sonlu_olcek(&payda) {
            let mut iki = 0u32;
            let mut bes = 0u32;
            let mut kalan = payda.clone();
            while (&kalan % 2u8) == BigInt::ZERO {
                kalan /= 2u8;
                iki += 1;
            }
            while (&kalan % 5u8) == BigInt::ZERO {
                kalan /= 5u8;
                bes += 1;
            }
            debug_assert_eq!(kalan, BigInt::from(1u8));
            pay *= BigInt::from(2u8).pow(olcek - iki);
            pay *= BigInt::from(5u8).pow(olcek - bes);
            if eksi {
                pay = -pay;
            }
            return Some(Self::yeni(pay, olcek));
        }

        let us = ondalik_ussu(&pay, &payda);
        let hedef_olcek = i64::from(BOLUM_ANLAMLI_HANE - 1) - us;
        let mut govde = if hedef_olcek >= 0 {
            let olcek = u32::try_from(hedef_olcek).ok()?;
            yuvarla_bol(&(pay * on_us(olcek)), &payda)
        } else {
            let sola = u32::try_from(-hedef_olcek).ok()?;
            yuvarla_bol(&pay, &(payda * on_us(sola))) * on_us(sola)
        };
        if eksi {
            govde = -govde;
        }
        let olcek = u32::try_from(hedef_olcek.max(0)).ok()?;
        Some(Self::yeni(govde, olcek))
    }

    pub fn tam_kismi(&self) -> Option<i64> {
        (&self.govde / on_us(self.olcek)).to_i64()
    }

    pub fn yuvarlanmisi(&self) -> Option<i64> {
        yuvarla_bol(&self.govde, &on_us(self.olcek)).to_i64()
    }

    pub fn katsayiyla_yuvarla_i128(&self, katsayi: i128) -> Option<i128> {
        yuvarla_bol(&(&self.govde * katsayi), &on_us(self.olcek)).to_i128()
    }

    pub fn kuruslu(&self, binlikli: bool) -> String {
        let kurus = if self.olcek > 2 {
            yuvarla_bol(&self.govde, &on_us(self.olcek - 2))
        } else {
            &self.govde * on_us(2 - self.olcek)
        };
        let isaret = if kurus.sign() == Sign::Minus { "-" } else { "" };
        let mut rakamlar = kurus.abs().to_str_radix(10);
        if rakamlar.len() < 3 {
            rakamlar = format!("{}{}", "0".repeat(3 - rakamlar.len()), rakamlar);
        }
        let ayrim = rakamlar.len() - 2;
        let tam = if binlikli {
            binlik_grupla(&rakamlar[..ayrim])
        } else {
            rakamlar[..ayrim].to_string()
        };
        format!("{}{},{}", isaret, tam, &rakamlar[ayrim..])
    }
}

fn on_us(us: u32) -> BigInt {
    BigInt::from(10u8).pow(us)
}

fn hizala(a: &Ondalik, b: &Ondalik) -> (BigInt, BigInt, u32) {
    let ortak = a.olcek.max(b.olcek);
    (
        &a.govde * on_us(ortak - a.olcek),
        &b.govde * on_us(ortak - b.olcek),
        ortak,
    )
}

fn obeb(mut a: BigInt, mut b: BigInt) -> BigInt {
    while b != BigInt::ZERO {
        let kalan = &a % &b;
        a = b;
        b = kalan;
    }
    a.abs()
}

fn sonlu_olcek(payda: &BigInt) -> Option<u32> {
    let mut kalan = payda.clone();
    let mut iki = 0u32;
    let mut bes = 0u32;
    while (&kalan % 2u8) == BigInt::ZERO {
        kalan /= 2u8;
        iki = iki.checked_add(1)?;
    }
    while (&kalan % 5u8) == BigInt::ZERO {
        kalan /= 5u8;
        bes = bes.checked_add(1)?;
    }
    (kalan == BigInt::from(1u8)).then_some(iki.max(bes))
}

/// floor(log10(pay/payda)); iki pozitif tam sayı için yalnız rakam uzunluğu
/// ve tek karşılaştırma kullanır.
fn ondalik_ussu(pay: &BigInt, payda: &BigInt) -> i64 {
    let pay_hane = pay.to_str_radix(10).len() as i64;
    let payda_hane = payda.to_str_radix(10).len() as i64;
    let mut us = pay_hane - payda_hane;
    let kucuk = if us >= 0 {
        pay < &(payda * on_us(us as u32))
    } else {
        &(pay * on_us((-us) as u32)) < payda
    };
    if kucuk {
        us -= 1;
    }
    us
}

/// Yarımlar sıfırdan uzağa. Payda sıfır değildir.
fn yuvarla_bol(pay: &BigInt, payda: &BigInt) -> BigInt {
    let eksi = (pay.sign() == Sign::Minus) != (payda.sign() == Sign::Minus);
    let pay = pay.abs();
    let payda = payda.abs();
    let mut bolum = &pay / &payda;
    let kalan = pay % &payda;
    if kalan * 2u8 >= payda {
        bolum += 1u8;
    }
    if eksi {
        -bolum
    } else {
        bolum
    }
}

fn binlik_grupla(tam: &str) -> String {
    let mut cikti = String::with_capacity(tam.len() + tam.len() / 3);
    for (i, k) in tam.chars().enumerate() {
        if i > 0 && (tam.len() - i).is_multiple_of(3) {
            cikti.push('.');
        }
        cikti.push(k);
    }
    cikti
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn sonlu_ve_sonsuz_bolum_ayrilir() {
        let bir = Ondalik::tam(1);
        let sekiz = Ondalik::tam(8);
        assert_eq!(bir.bol(&sekiz).unwrap().metne(), "0,125");
        let uc = Ondalik::tam(3);
        assert_eq!(
            bir.bol(&uc).unwrap().metne(),
            "0,3333333333333333333333333333333333"
        );
        let eksi_bir = Ondalik::tam(-1);
        let alti = Ondalik::tam(6);
        assert_eq!(
            eksi_bir.bol(&alti).unwrap().metne(),
            "-0,1666666666666666666666666666666667"
        );
    }

    #[test]
    fn keyfi_katsayi_tam_islemleri_korur() {
        let a = Ondalik::metinden("123456789012345678901234567890,12").unwrap();
        let b = Ondalik::metinden("0,88").unwrap();
        assert!(a.metne().len() <= a.metin_bayti_ust_siniri());
        assert!(a.kuruslu(true).len() <= a.kuruslu_metin_bayti_ust_siniri(true));
        assert_eq!(a.topla(&b).metne(), "123456789012345678901234567891,0");
        assert_eq!(
            a.carp(&Ondalik::tam(100)).unwrap().metne(),
            "12345678901234567890123456789012,0"
        );
        assert!(Ondalik::katsayidan(1, 20_000_000).metin_bayti_ust_siniri() > 20_000_000);
    }
}
