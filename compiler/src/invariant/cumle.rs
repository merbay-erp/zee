use std::collections::HashSet;

use crate::agac::{Cumle, Program};

use super::*;

impl Denetleyici<'_> {
    pub(super) fn programi_dogrula(&mut self, program: &Program) -> Result<(), InvariantHatasi> {
        self.cumleleri_dogrula(&program.cumleler, "program.cümleler", true)?;

        let mut islemler = program.islemler.iter().collect::<Vec<_>>();
        islemler.sort_by(|(sol, _), (sag, _)| sol.cmp(sag));
        for (ad, islem) in islemler {
            let yol = format!("program.işlemler[{ad:?}]");
            if ad != &islem.ad || ad.is_empty() {
                return Err(self.hata(
                    &yol,
                    format!("işlem tablo anahtarı {:?}, kayıt adı {:?}", ad, islem.ad),
                    islem.satir,
                ));
            }
            self.cumleleri_dogrula(&islem.govde, &format!("{yol}.gövde"), true)?;
        }

        let mut yapi_adlari = HashSet::new();
        for (sira, yapi) in program.yapilar.iter().enumerate() {
            let yol = format!("program.yapılar[{sira}]");
            if yapi.ad.is_empty() || !yapi_adlari.insert(&yapi.ad) {
                return Err(self.hata(&yol, "yapı adı boş veya yinelenmiş", yapi.satir));
            }
            let mut alan_adlari = HashSet::new();
            for (alan, tur) in &yapi.alanlar {
                if alan.is_empty() || tur.is_empty() || !alan_adlari.insert(alan) {
                    return Err(self.hata(
                        &yol,
                        "yapı alanı/türü boş veya alan adı yinelenmiş",
                        yapi.satir,
                    ));
                }
            }
        }

        for (sira, test) in program.testler.iter().enumerate() {
            self.cumleleri_dogrula(&test.govde, &format!("program.testler[{sira}].gövde"), true)?;
        }
        Ok(())
    }

    pub(super) fn cumleleri_dogrula(
        &mut self,
        cumleler: &[Cumle],
        yol: &str,
        hoist_sonrasi: bool,
    ) -> Result<(), InvariantHatasi> {
        for (sira, cumle) in cumleler.iter().enumerate() {
            self.cumleyi_dogrula(cumle, &format!("{yol}[{sira}]"), hoist_sonrasi)?;
        }
        Ok(())
    }

    fn cumleyi_dogrula(
        &mut self,
        cumle: &Cumle,
        yol: &str,
        hoist_sonrasi: bool,
    ) -> Result<(), InvariantHatasi> {
        match cumle {
            Cumle::Yaz { deger, satir }
            | Cumle::Olmali {
                kosul: deger,
                satir,
            }
            | Cumle::Sor {
                istem: deger,
                satir,
            }
            | Cumle::YanitGonder { deger, satir }
            | Cumle::Yonlendir {
                adres: deger,
                satir,
            }
            | Cumle::SunucuBaslat { kapi: deger, satir }
            | Cumle::CerezSil { ad: deger, satir }
            | Cumle::Bekle { sure: deger, satir } => {
                self.ifadeyi_dogrula(deger, &format!("{yol}.ifade"), *satir, false)?;
            }
            Cumle::Olsun { deger, satir, .. } => {
                self.ifadeyi_dogrula(deger, &format!("{yol}.değer"), *satir, false)?;
            }
            Cumle::KezTekrarla { adet, govde, satir } => {
                self.ifadeyi_dogrula(adet, &format!("{yol}.adet"), *satir, false)?;
                self.cumleleri_dogrula(govde, &format!("{yol}.gövde"), hoist_sonrasi)?;
            }
            Cumle::AralikDongusu {
                bastan,
                sona,
                govde,
                satir,
                ..
            } => {
                self.ifadeyi_dogrula(bastan, &format!("{yol}.baştan"), *satir, false)?;
                self.ifadeyi_dogrula(sona, &format!("{yol}.sona"), *satir, false)?;
                self.cumleleri_dogrula(govde, &format!("{yol}.gövde"), hoist_sonrasi)?;
            }
            Cumle::OlduguSurece {
                kosul,
                govde,
                satir,
            }
            | Cumle::OlanaKadar {
                kosul,
                govde,
                satir,
            } => {
                self.ifadeyi_dogrula(kosul, &format!("{yol}.koşul"), *satir, false)?;
                self.cumleleri_dogrula(govde, &format!("{yol}.gövde"), hoist_sonrasi)?;
            }
            Cumle::Ise {
                kollar,
                degilse,
                satir,
            } => {
                if kollar.is_empty() {
                    return Err(self.hata(yol, "ise zincirinin koşul kolu yok", *satir));
                }
                for (sira, kol) in kollar.iter().enumerate() {
                    let kol_yolu = format!("{yol}.kollar[{sira}]");
                    self.ifadeyi_dogrula(&kol.kosul, &format!("{kol_yolu}.koşul"), *satir, false)?;
                    self.cumleleri_dogrula(
                        &kol.govde,
                        &format!("{kol_yolu}.gövde"),
                        hoist_sonrasi,
                    )?;
                }
                if let Some(degilse) = degilse {
                    self.cumleleri_dogrula(degilse, &format!("{yol}.değilse"), hoist_sonrasi)?;
                }
            }
            Cumle::Artir {
                ifade,
                miktar,
                satir,
            }
            | Cumle::Azalt {
                ifade,
                miktar,
                satir,
            } => {
                self.ifadeyi_dogrula(ifade, &format!("{yol}.hedef"), *satir, false)?;
                self.ifadeyi_dogrula(miktar, &format!("{yol}.miktar"), *satir, false)?;
            }
            Cumle::Ekle {
                hedef,
                deger,
                satir,
            }
            | Cumle::Sil {
                kap: hedef,
                deger,
                satir,
            } => {
                self.ifadeyi_dogrula(hedef, &format!("{yol}.hedef"), *satir, false)?;
                self.ifadeyi_dogrula(deger, &format!("{yol}.değer"), *satir, false)?;
            }
            Cumle::HerBiri {
                kaynak,
                govde,
                satir,
                ..
            } => {
                if self.faz == InvariantFazi::BaglanmisHir && kaynak.is_none() {
                    return Err(self.hata(
                        yol,
                        "bağlanmış gezme cümlesinin kaynak ifadesi yok",
                        *satir,
                    ));
                }
                if let Some(kaynak) = kaynak {
                    self.ifadeyi_dogrula(kaynak, &format!("{yol}.kaynak"), *satir, false)?;
                }
                self.cumleleri_dogrula(govde, &format!("{yol}.gövde"), hoist_sonrasi)?;
            }
            Cumle::Gore {
                konu,
                kollar,
                degilse,
                satir,
            } => {
                if kollar.is_empty() {
                    return Err(self.hata(yol, "göre zincirinin değer kolu yok", *satir));
                }
                self.ifadeyi_dogrula(konu, &format!("{yol}.konu"), *satir, false)?;
                for (sira, (deger, govde)) in kollar.iter().enumerate() {
                    let kol_yolu = format!("{yol}.kollar[{sira}]");
                    self.ifadeyi_dogrula(deger, &format!("{kol_yolu}.değer"), *satir, false)?;
                    self.cumleleri_dogrula(govde, &format!("{kol_yolu}.gövde"), hoist_sonrasi)?;
                }
                if let Some(degilse) = degilse {
                    self.cumleleri_dogrula(degilse, &format!("{yol}.değilse"), hoist_sonrasi)?;
                }
            }
            Cumle::AlanAta {
                nesne,
                deger,
                satir,
                ..
            } => {
                self.ifadeyi_dogrula(nesne, &format!("{yol}.nesne"), *satir, false)?;
                self.ifadeyi_dogrula(deger, &format!("{yol}.değer"), *satir, false)?;
            }
            Cumle::Dondur {
                deger,
                sonuca_sarmala,
                satir,
            } => {
                if self.faz == InvariantFazi::AyristirilmisAst && *sonuca_sarmala {
                    return Err(self.hata(
                        yol,
                        "parser AST'si checker'a ait sonuca_sarmala işareti taşıyor",
                        *satir,
                    ));
                }
                self.ifadeyi_dogrula(deger, &format!("{yol}.değer"), *satir, false)?;
            }
            Cumle::HataDondur {
                mesaj,
                neden,
                veri,
                satir,
                ..
            } => {
                self.ifadeyi_dogrula(mesaj, &format!("{yol}.mesaj"), *satir, false)?;
                for (ad, ifade) in [("neden", neden), ("veri", veri)] {
                    if let Some(ifade) = ifade {
                        self.ifadeyi_dogrula(ifade, &format!("{yol}.{ad}"), *satir, false)?;
                    }
                }
            }
            Cumle::BolVeAta {
                pay, payda, satir, ..
            } => {
                self.ifadeyi_dogrula(pay, &format!("{yol}.pay"), *satir, false)?;
                self.ifadeyi_dogrula(payda, &format!("{yol}.payda"), *satir, false)?;
            }
            Cumle::CagriCumlesi { cagri, satir } => {
                if !matches!(cagri, Ifade::IslemCagrisi { .. }) {
                    return Err(self.hata(yol, "çağrı cümlesi işlem çağrısı taşımıyor", *satir));
                }
                self.ifadeyi_dogrula(cagri, &format!("{yol}.çağrı"), *satir, true)?;
            }
            Cumle::ProgramiBitir { kod, satir } => {
                if let Some(kod) = kod {
                    self.ifadeyi_dogrula(kod, &format!("{yol}.kod"), *satir, false)?;
                }
            }
            Cumle::IstekGeldiginde {
                yol: adres,
                govde,
                satir,
                ..
            } => {
                self.ifadeyi_dogrula(adres, &format!("{yol}.yol"), *satir, false)?;
                self.cumleleri_dogrula(govde, &format!("{yol}.gövde"), hoist_sonrasi)?;
            }
            Cumle::CerezYaz { ad, deger, satir }
            | Cumle::OturumAc {
                kullanici: ad,
                rol: deger,
                satir,
            } => {
                self.ifadeyi_dogrula(ad, &format!("{yol}.birinci"), *satir, false)?;
                self.ifadeyi_dogrula(deger, &format!("{yol}.ikinci"), *satir, false)?;
            }
            Cumle::Eszamanli { gorevler, satir } => {
                if gorevler.is_empty() {
                    return Err(self.hata(yol, "eşzamanlı blok görev taşımıyor", *satir));
                }
                for (sira, (_, ifade, gorev_satiri)) in gorevler.iter().enumerate() {
                    self.ifadeyi_dogrula(
                        ifade,
                        &format!("{yol}.görevler[{sira}]"),
                        *gorev_satiri,
                        false,
                    )?;
                }
            }
            Cumle::IcindeBlogu {
                sure,
                govde,
                yetismezse,
                satir,
            } => {
                self.ifadeyi_dogrula(sure, &format!("{yol}.süre"), *satir, false)?;
                self.cumleleri_dogrula(govde, &format!("{yol}.gövde"), hoist_sonrasi)?;
                if let Some(yetismezse) = yetismezse {
                    self.cumleleri_dogrula(
                        yetismezse,
                        &format!("{yol}.yetişmezse"),
                        hoist_sonrasi,
                    )?;
                }
            }
            Cumle::SozlukAta {
                sozluk,
                anahtar,
                deger,
                satir,
            } => {
                self.ifadeyi_dogrula(sozluk, &format!("{yol}.sözlük"), *satir, false)?;
                self.ifadeyi_dogrula(anahtar, &format!("{yol}.anahtar"), *satir, false)?;
                self.ifadeyi_dogrula(deger, &format!("{yol}.değer"), *satir, false)?;
            }
            Cumle::DosyayaYaz {
                yol: adres,
                icerik,
                satir,
                ..
            } => {
                self.ifadeyi_dogrula(adres, &format!("{yol}.yol"), *satir, false)?;
                self.ifadeyi_dogrula(icerik, &format!("{yol}.içerik"), *satir, false)?;
            }
            Cumle::IslemTanimi(islem) => {
                if hoist_sonrasi {
                    return Err(self.hata(
                        yol,
                        "hoist sonrası işlem tanımı cümlede kaldı",
                        islem.satir,
                    ));
                }
                self.cumleleri_dogrula(&islem.govde, &format!("{yol}.gövde"), hoist_sonrasi)?;
            }
            Cumle::TestBlogu(test) => {
                if hoist_sonrasi {
                    return Err(self.hata(
                        yol,
                        "hoist sonrası test bloğu cümlede kaldı",
                        test.satir,
                    ));
                }
                self.cumleleri_dogrula(&test.govde, &format!("{yol}.gövde"), hoist_sonrasi)?;
            }
            Cumle::YapiTanimi(yapi) => {
                if hoist_sonrasi {
                    return Err(self.hata(
                        yol,
                        "hoist sonrası yapı tanımı cümlede kaldı",
                        yapi.satir,
                    ));
                }
            }
            Cumle::Kullan { satir, .. } => {
                if hoist_sonrasi {
                    return Err(self.hata(yol, "bağlama sonrası kullan cümlesi kaldı", *satir));
                }
            }
            Cumle::RotaPolitikasi { .. }
            | Cumle::RotaAlaniGerekli { .. }
            | Cumle::OturumKapat { .. }
            | Cumle::HepsiniBekle { .. }
            | Cumle::IsikAyarla { .. } => {}
        }
        Ok(())
    }
}
