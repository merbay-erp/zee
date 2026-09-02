//! Yerel çıkarımlı işlem imzalarının kaynak sırasından bağımsız keşfi.

use super::*;

pub(super) fn cagri_turlerini_birlestir(sol: &Tur, sag: &Tur) -> Option<Tur> {
    if sol == sag {
        return Some(*sol);
    }
    match (sol, sag) {
        (Tur::TamSayi, Tur::Ondalik) | (Tur::Ondalik, Tur::TamSayi) => Some(Tur::Ondalik),
        (Tur::Liste(VeriTuru::TamSayi), Tur::Liste(VeriTuru::Ondalik))
        | (Tur::Liste(VeriTuru::Ondalik), Tur::Liste(VeriTuru::TamSayi)) => {
            Some(Tur::Liste(VeriTuru::Ondalik))
        }
        (Tur::Sozluk(SozlukDegerTuru::TamSayi), Tur::Sozluk(SozlukDegerTuru::Ondalik))
        | (Tur::Sozluk(SozlukDegerTuru::Ondalik), Tur::Sozluk(SozlukDegerTuru::TamSayi)) => {
            Some(Tur::Sozluk(SozlukDegerTuru::Ondalik))
        }
        (Tur::Secenek(VeriTuru::TamSayi), Tur::Secenek(VeriTuru::Ondalik))
        | (Tur::Secenek(VeriTuru::Ondalik), Tur::Secenek(VeriTuru::TamSayi)) => {
            Some(Tur::Secenek(VeriTuru::Ondalik))
        }
        (Tur::Sonuc(VeriTuru::TamSayi), Tur::Sonuc(VeriTuru::Ondalik))
        | (Tur::Sonuc(VeriTuru::Ondalik), Tur::Sonuc(VeriTuru::TamSayi)) => {
            Some(Tur::Sonuc(VeriTuru::Ondalik))
        }
        _ => None,
    }
}

/// Normal checker'ın ilk-sonuç yan etkisini HIR'a taşımamak için bütün
/// erişilebilir çağrıları ayrı AST kopyasında gezer. Yerel bir cümledeki hata
/// sonraki bağımsız çağrı kısıtını gizlemez; asıl tanı yalnız ikinci,
/// nihai imzalı geçişten gelir.
pub(super) fn cikarim_onbilgisi(program: &Program) -> Result<HashMap<IslemId, Imza>, Tani> {
    let mut kesif = program.clone();
    etki::denetle(&kesif)?;
    if let Some(tani) = yapi_turu_tanilari(&kesif.yapilar).into_iter().next() {
        return Err(tani);
    }

    let mut ortam = SembolTablosu::yeni(0);
    let mut baglam = Baglam::yeni(std::mem::take(&mut kesif.islemler), kesif.yapilar.clone());
    baglam.cikarim_kesfi = true;
    // Açık gövdedeki kullanıcı hatası nihai geçişin tanısıdır. Burada başarılı
    // açık imzaları tohumlamak yeterlidir; ilk hatanın keşfi durdurmasına izin
    // verilmez.
    let _ = acik_islemleri_denetle(&mut baglam);

    let mut bas = 0;
    while bas < kesif.cumleler.len() {
        let mut son = bas + 1;
        if matches!(kesif.cumleler[bas], Cumle::Eszamanli { .. }) {
            while son < kesif.cumleler.len() {
                let join = matches!(kesif.cumleler[son], Cumle::HepsiniBekle { .. });
                son += 1;
                if join {
                    break;
                }
            }
        }
        let onceki_bekleyenler = baglam.bekleyen_gorevler.clone();
        if blok_denetle(&mut kesif.cumleler[bas..son], &mut ortam, &mut baglam).is_err() {
            baglam.bekleyen_gorevler = onceki_bekleyenler;
        }
        bas = son;
    }

    for (test_indeksi, test) in kesif.testler.iter_mut().enumerate() {
        let mut test_ortami = SembolTablosu::yeni(baglam.test_kapsami(test_indeksi));
        let onceki_bekleyenler = baglam.bekleyen_gorevler.clone();
        if blok_denetle(&mut test.govde, &mut test_ortami, &mut baglam).is_err() {
            baglam.bekleyen_gorevler = onceki_bekleyenler;
        }
    }

    let mut imzalar = std::mem::take(&mut baglam.imzalar);
    for imza in imzalar.values_mut() {
        imza.govde_dogrulandi = false;
    }
    Ok(imzalar)
}
