use super::*;

impl Denetleyici<'_> {
    pub(super) fn ifadeyi_dogrula(
        &mut self,
        ifade: &Ifade,
        yol: &str,
        satir: usize,
        deger_dondurmez_olabilir: bool,
    ) -> Result<(), InvariantHatasi> {
        let kaynak_araligi = ifade.kaynak_araligi().ok_or_else(|| {
            self.hata(
                yol,
                "AST ifadesinin kesin kaynak aralığı yok",
                satir,
            )
        })?;
        // Zincir cümlelerin (değilse/ise ve göre kolları) sahibi ilk başlık
        // satırını taşır; her ifade için gerçek kaynak satırı kendi zarfıdır.
        let satir = kaynak_araligi.satir();
        if let Ifade::Kaynakli { ifade: ic, .. } = ifade {
            if ic.dogrudan_kaynakli_mi() {
                return Err(self.hata(yol, "AST ifadesi iç içe kaynak zarfı taşıyor", satir));
            }
        }
        let beklenen_bag = self.ifadenin_faz_bagini_dogrula(ifade, yol, satir)?;
        self.hir_ifadesini_dogrula(ifade, yol, satir, beklenen_bag, deger_dondurmez_olabilir)?;

        match ifade.turu() {
            Ifade::Parcala { metin, ayrac }
            | Ifade::ListeBirlestir {
                liste: metin,
                ayrac,
            }
            | Ifade::GunFarki {
                birinci: metin,
                ikinci: ayrac,
            }
            | Ifade::Aritmetik {
                sol: metin,
                sag: ayrac,
                ..
            }
            | Ifade::Karsilastirma {
                sol: metin,
                sag: ayrac,
                ..
            } => {
                self.ifadeyi_dogrula(metin, &format!("{yol}.sol"), satir, false)?;
                self.ifadeyi_dogrula(ayrac, &format!("{yol}.sağ"), satir, false)?;
            }
            Ifade::Degistir { metin, eski, yeni } => {
                self.ifadeyi_dogrula(metin, &format!("{yol}.metin"), satir, false)?;
                self.ifadeyi_dogrula(eski, &format!("{yol}.eski"), satir, false)?;
                self.ifadeyi_dogrula(yeni, &format!("{yol}.yeni"), satir, false)?;
            }
            Ifade::MetinSinari { metin, parca, .. }
            | Ifade::Icerir {
                metin,
                aranan: parca,
            } => {
                self.ifadeyi_dogrula(metin, &format!("{yol}.metin"), satir, false)?;
                self.ifadeyi_dogrula(parca, &format!("{yol}.parça"), satir, false)?;
            }
            Ifade::Rastgele { alt, ust } => {
                self.ifadeyi_dogrula(alt, &format!("{yol}.alt"), satir, false)?;
                self.ifadeyi_dogrula(ust, &format!("{yol}.üst"), satir, false)?;
            }
            Ifade::Intrinsic { kimlik, argumanlar } => {
                if kimlik.is_empty() {
                    return Err(self.hata(yol, "intrinsic kimliği boş", satir));
                }
                for (sira, arguman) in argumanlar.iter().enumerate() {
                    self.ifadeyi_dogrula(
                        arguman,
                        &format!("{yol}.argümanlar[{sira}]"),
                        satir,
                        false,
                    )?;
                }
            }
            Ifade::Birlestir(parcalar) | Ifade::MantiksalZincir { parcalar, .. } => {
                if parcalar.len() < 2 {
                    return Err(self.hata(yol, "zincir en az iki parça taşımalı", satir));
                }
                self.ifade_listesini_dogrula(parcalar, yol, satir)?;
            }
            Ifade::ListeSabiti(ogeler) => {
                if ogeler.is_empty() {
                    return Err(self.hata(
                        yol,
                        "boş liste ListeSabiti değil BosListe olmalı",
                        satir,
                    ));
                }
                self.ifade_listesini_dogrula(ogeler, yol, satir)?;
            }
            Ifade::Degil(ic)
            | Ifade::Cift(ic)
            | Ifade::Tek(ic)
            | Ifade::Ozellik { nesne: ic, .. }
            | Ifade::MetinDonusum { nesne: ic, .. }
            | Ifade::SecenekVar { nesne: ic, .. }
            | Ifade::IcDeger(ic)
            | Ifade::SonucHatasi(ic)
            | Ifade::SonucBasarili { nesne: ic, .. }
            | Ifade::DosyaOkumayiDene(ic)
            | Ifade::DosyaSatirlari(ic)
            | Ifade::TabloOku(ic)
            | Ifade::VeriOku(ic)
            | Ifade::DurumKodu(ic)
            | Ifade::Govde(ic)
            | Ifade::BosMu { nesne: ic, .. }
            | Ifade::Sayisi(ic)
            | Ifade::Ondaligi(ic)
            | Ifade::SayiyiDene(ic)
            | Ifade::OndaligiDene(ic)
            | Ifade::AlanErisim { nesne: ic, .. } => {
                self.ifadeyi_dogrula(ic, &format!("{yol}.iç"), satir, false)?;
            }
            Ifade::SozlukDegeri { sozluk, anahtar }
            | Ifade::SozlukteVar {
                sozluk, anahtar, ..
            } => {
                self.ifadeyi_dogrula(sozluk, &format!("{yol}.sözlük"), satir, false)?;
                self.ifadeyi_dogrula(anahtar, &format!("{yol}.anahtar"), satir, false)?;
            }
            Ifade::GunSonrasi { tarih, miktar } => {
                self.ifadeyi_dogrula(tarih, &format!("{yol}.tarih"), satir, false)?;
                self.ifadeyi_dogrula(miktar, &format!("{yol}.miktar"), satir, false)?;
            }
            Ifade::IslemCagrisi { argumanlar, .. } => {
                for (sira, arguman) in argumanlar.iter().enumerate() {
                    self.ifadeyi_dogrula(
                        arguman,
                        &format!("{yol}.argümanlar[{sira}]"),
                        satir,
                        false,
                    )?;
                }
            }
            Ifade::MetinSabiti(_)
            | Ifade::SayiSabiti(_)
            | Ifade::OndalikSabiti { .. }
            | Ifade::MantiksalSabiti(_)
            | Ifade::Degisken { .. }
            | Ifade::BosListe
            | Ifade::BosSozluk
            | Ifade::YokSabiti
            | Ifade::BugununTarihi
            | Ifade::SuAninSaati
            | Ifade::KomutArgumanlari
            | Ifade::SureSabiti { .. }
            | Ifade::YeniYapi { .. } => {}
            Ifade::Kaynakli { .. } => {
                return Err(self.hata(yol, "AST ifadesi çözülemeyen kaynak zarfı taşıyor", satir));
            }
        }
        Ok(())
    }

    fn ifade_listesini_dogrula(
        &mut self,
        ifadeler: &[Ifade],
        yol: &str,
        satir: usize,
    ) -> Result<(), InvariantHatasi> {
        for (sira, ifade) in ifadeler.iter().enumerate() {
            self.ifadeyi_dogrula(ifade, &format!("{yol}[{sira}]"), satir, false)?;
        }
        Ok(())
    }

    fn ifadenin_faz_bagini_dogrula(
        &self,
        ifade: &Ifade,
        yol: &str,
        satir: usize,
    ) -> Result<HirBagi, InvariantHatasi> {
        match ifade.turu() {
            Ifade::Degisken {
                ham,
                cozulmus,
                sembol_kimligi,
                satir: ifade_satiri,
                sutun,
                uzunluk,
            } => {
                if ham.is_empty() {
                    return Err(self.hata(yol, "değişkenin ham adı boş", satir));
                }
                if *ifade_satiri == 0 || *sutun == 0 || *uzunluk == 0 {
                    return Err(self.hata(yol, "değişken kaynak aralığı sıfır", satir));
                }
                match self.faz {
                    InvariantFazi::AyristirilmisAst => {
                        if cozulmus.is_some() || sembol_kimligi.is_some() {
                            return Err(self.hata(
                                yol,
                                "parser AST'si çözülmüş ad/SymbolId taşıyor",
                                *ifade_satiri,
                            ));
                        }
                        Ok(HirBagi::Yok)
                    }
                    InvariantFazi::BaglanmisHir => {
                        let ad = cozulmus.as_deref().ok_or_else(|| {
                            self.hata(yol, "bağlanmış değişkenin çözülmüş adı yok", *ifade_satiri)
                        })?;
                        let kimlik = sembol_kimligi.ok_or_else(|| {
                            self.hata(yol, "bağlanmış değişkenin SymbolId'si yok", *ifade_satiri)
                        })?;
                        let hir = self.hir(yol, *ifade_satiri)?;
                        if hir.sembol_adi(kimlik) != Some(ad) {
                            return Err(self.hata(
                                yol,
                                format!("SymbolId {:?}, {:?} adına bağlı değil", kimlik, ad),
                                *ifade_satiri,
                            ));
                        }
                        Ok(HirBagi::Sembol(kimlik))
                    }
                }
            }
            Ifade::YeniYapi {
                yapi_adi,
                yapi_kimligi,
            } => {
                if yapi_adi.is_empty() {
                    return Err(self.hata(yol, "yapı adı boş", satir));
                }
                match self.faz {
                    InvariantFazi::AyristirilmisAst => {
                        if yapi_kimligi.is_some() {
                            return Err(self.hata(yol, "parser AST'si YapiId taşıyor", satir));
                        }
                        Ok(HirBagi::Yok)
                    }
                    InvariantFazi::BaglanmisHir => {
                        let kimlik = yapi_kimligi.ok_or_else(|| {
                            self.hata(yol, "bağlanmış yapı ifadesinin YapiId'si yok", satir)
                        })?;
                        let hir = self.hir(yol, satir)?;
                        let yapi = hir.yapi(kimlik).ok_or_else(|| {
                            self.hata(yol, format!("YapiId {:?} tabloda yok", kimlik), satir)
                        })?;
                        if yapi.ad != *yapi_adi {
                            return Err(self.hata(
                                yol,
                                format!("YapiId {:?}, {:?} yapısına ait", kimlik, yapi.ad),
                                satir,
                            ));
                        }
                        Ok(HirBagi::Yapi(kimlik))
                    }
                }
            }
            Ifade::IslemCagrisi {
                islem_adi,
                islem_kimligi,
                satir: cagri_satiri,
                ..
            } => {
                if islem_adi.is_empty() || *cagri_satiri == 0 {
                    return Err(self.hata(yol, "işlem çağrısı adı/satırı geçersiz", satir));
                }
                match self.faz {
                    InvariantFazi::AyristirilmisAst => {
                        if islem_kimligi.is_some() {
                            return Err(self.hata(
                                yol,
                                "parser AST'si IslemId taşıyor",
                                *cagri_satiri,
                            ));
                        }
                        Ok(HirBagi::Yok)
                    }
                    InvariantFazi::BaglanmisHir => {
                        let kimlik = islem_kimligi.ok_or_else(|| {
                            self.hata(yol, "bağlanmış çağrının IslemId'si yok", *cagri_satiri)
                        })?;
                        let hir = self.hir(yol, *cagri_satiri)?;
                        let islem = hir.islem(kimlik).ok_or_else(|| {
                            self.hata(
                                yol,
                                format!("IslemId {:?} tabloda yok", kimlik),
                                *cagri_satiri,
                            )
                        })?;
                        if islem.ad != *islem_adi {
                            return Err(self.hata(
                                yol,
                                format!("IslemId {:?}, {:?} işlemine ait", kimlik, islem.ad),
                                *cagri_satiri,
                            ));
                        }
                        Ok(HirBagi::Islem(kimlik))
                    }
                }
            }
            Ifade::Kaynakli { .. } => {
                Err(self.hata(yol, "AST ifadesi çözülemeyen kaynak zarfı taşıyor", satir))
            }
            _ => Ok(HirBagi::Yok),
        }
    }
}
