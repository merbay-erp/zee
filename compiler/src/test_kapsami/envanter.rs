use super::*;
use crate::agac::GorunumDugumu;

struct Gezici {
    t: Toplayici,
    kaynak: String,
    sahip: String,
    tur: String,
    sira: usize,
}
impl Gezici {
    fn sahip(&mut self, kaynak: &str, sahip: &str, tur: &str) -> Result<(), String> {
        if !self.t.kaynaklar.contains_key(kaynak) || sahip.len() > 512 {
            return Err("kapsam sahip kökeni geçersiz".into());
        }
        self.kaynak = kaynak.into();
        self.sahip = sahip.into();
        self.tur = tur.into();
        Ok(())
    }
    fn ekle(&mut self, key: Anahtar, satir: usize, karar: &str) -> Result<(), String> {
        if self.t.dallar.len() + 2 > AZAMI_DAL || self.t.dizin.contains_key(&key) {
            return Err("kapsam dal bütçesi veya tekilliği geçersiz".into());
        }
        let ilk = self.t.dallar.len();
        self.t.dizin.insert(key, ilk);
        for sonuc in [false, true] {
            self.t.dallar.push(Dal {
                id: self.t.dallar.len(),
                kaynak: self.kaynak.clone(),
                sahip_turu: self.tur.clone(),
                sahip: self.sahip.clone(),
                satir,
                karar: karar.into(),
                sira: self.sira,
                sonuc,
                vurus: 0,
            });
        }
        self.sira += 1;
        Ok(())
    }
    fn derinlik(d: usize) -> Result<(), String> {
        if d > 256 {
            Err("kapsam gezinme derinliği aşıldı".into())
        } else {
            Ok(())
        }
    }
    fn blok(&mut self, cs: &[Cumle], d: usize) -> Result<(), String> {
        Self::derinlik(d)?;
        for c in cs {
            self.cumle(c, d + 1)?;
        }
        Ok(())
    }
    fn ifade(&mut self, i: &Ifade, satir: usize, d: usize) -> Result<(), String> {
        Self::derinlik(d)?;
        let satir = i.kaynak_araligi().map(|a| a.satir()).unwrap_or(satir);
        match i.turu() {
            Ifade::Kaynakli { .. } => return Err("kapsam kaynak zarfı geçersiz".into()),
            Ifade::MantiksalZincir { parcalar, .. } => {
                for (n, p) in parcalar.iter().enumerate() {
                    self.ekle(
                        (true, i as *const Ifade as usize, n),
                        satir,
                        "logical-operand",
                    )?;
                    self.ifade(p, satir, d + 1)?;
                }
            }
            Ifade::Birlestir(xs)
            | Ifade::ListeSabiti(xs)
            | Ifade::Intrinsic { argumanlar: xs, .. }
            | Ifade::IslemCagrisi { argumanlar: xs, .. } => {
                for x in xs {
                    self.ifade(x, satir, d + 1)?;
                }
            }
            Ifade::Parcala { metin: a, ayrac: b }
            | Ifade::ListeBirlestir { liste: a, ayrac: b }
            | Ifade::MetinSinari {
                metin: a, parca: b, ..
            }
            | Ifade::Rastgele { alt: a, ust: b }
            | Ifade::Karsilastirma { sol: a, sag: b, .. }
            | Ifade::Aritmetik { sol: a, sag: b, .. }
            | Ifade::SozlukDegeri {
                sozluk: a,
                anahtar: b,
            }
            | Ifade::SozlukteVar {
                sozluk: a,
                anahtar: b,
                ..
            }
            | Ifade::Icerir {
                metin: a,
                aranan: b,
            }
            | Ifade::GunSonrasi {
                tarih: a,
                miktar: b,
            }
            | Ifade::GunFarki {
                birinci: a,
                ikinci: b,
            } => {
                self.ifade(a, satir, d + 1)?;
                self.ifade(b, satir, d + 1)?;
            }
            Ifade::Degistir { metin, eski, yeni } => {
                for x in [metin, eski, yeni] {
                    self.ifade(x, satir, d + 1)?;
                }
            }
            Ifade::Degil(x)
            | Ifade::Cift(x)
            | Ifade::Tek(x)
            | Ifade::IcDeger(x)
            | Ifade::SonucHatasi(x)
            | Ifade::DosyaOkumayiDene(x)
            | Ifade::DosyaSatirlari(x)
            | Ifade::TabloOku(x)
            | Ifade::VeriOku(x)
            | Ifade::DurumKodu(x)
            | Ifade::Govde(x)
            | Ifade::Sayisi(x)
            | Ifade::Ondaligi(x)
            | Ifade::SayiyiDene(x)
            | Ifade::OndaligiDene(x)
            | Ifade::Ozellik { nesne: x, .. }
            | Ifade::MetinDonusum { nesne: x, .. }
            | Ifade::SecenekVar { nesne: x, .. }
            | Ifade::SonucBasarili { nesne: x, .. }
            | Ifade::BosMu { nesne: x, .. }
            | Ifade::AlanErisim { nesne: x, .. } => self.ifade(x, satir, d + 1)?,
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
        }
        Ok(())
    }
    fn gorunum(&mut self, xs: &[GorunumDugumu], d: usize) -> Result<(), String> {
        Self::derinlik(d)?;
        for x in xs {
            match x {
                GorunumDugumu::Oge {
                    oznitelikler,
                    cocuklar,
                    satir,
                    ..
                } => {
                    for (_, i) in oznitelikler {
                        self.ifade(i, *satir, d + 1)?;
                    }
                    self.gorunum(cocuklar, d + 1)?;
                }
                GorunumDugumu::Metin { ifade, satir } | GorunumDugumu::Ham { ifade, satir } => {
                    self.ifade(ifade, *satir, d + 1)?
                }
            }
        }
        Ok(())
    }
    fn cumle(&mut self, c: &Cumle, d: usize) -> Result<(), String> {
        Self::derinlik(d)?;
        let key = |n| (false, c as *const Cumle as usize, n);
        match c {
            Cumle::Ise {
                kollar,
                degilse,
                satir,
            } => {
                for (n, k) in kollar.iter().enumerate() {
                    self.ekle(key(n), *satir, "if-condition")?;
                    self.ifade(&k.kosul, *satir, d + 1)?;
                    self.blok(&k.govde, d + 1)?;
                }
                if let Some(b) = degilse {
                    self.blok(b, d + 1)?;
                }
            }
            Cumle::Gore {
                konu,
                kollar,
                degilse,
                satir,
            } => {
                self.ifade(konu, *satir, d + 1)?;
                for (n, (i, b)) in kollar.iter().enumerate() {
                    self.ekle(key(n), *satir, "match-comparison")?;
                    self.ifade(i, *satir, d + 1)?;
                    self.blok(b, d + 1)?;
                }
                if let Some(b) = degilse {
                    self.blok(b, d + 1)?;
                }
            }
            Cumle::KezTekrarla { adet, govde, satir } => {
                self.ekle(key(0), *satir, "repeat-body-exhaustion")?;
                self.ifade(adet, *satir, d + 1)?;
                self.blok(govde, d + 1)?;
            }
            Cumle::AralikDongusu {
                bastan,
                sona,
                govde,
                satir,
                ..
            } => {
                self.ekle(key(0), *satir, "range-body-exhaustion")?;
                self.ifade(bastan, *satir, d + 1)?;
                self.ifade(sona, *satir, d + 1)?;
                self.blok(govde, d + 1)?;
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
                self.ekle(key(0), *satir, "loop-body-exhaustion")?;
                self.ifade(kosul, *satir, d + 1)?;
                self.blok(govde, d + 1)?;
            }
            Cumle::HerBiri {
                kaynak,
                govde,
                satir,
                ..
            } => {
                self.ekle(key(0), *satir, "foreach-body-exhaustion")?;
                if let Some(i) = kaynak {
                    self.ifade(i, *satir, d + 1)?;
                }
                self.blok(govde, d + 1)?;
            }
            Cumle::IcindeBlogu {
                sure,
                govde,
                yetismezse,
                satir,
            } => {
                self.ekle(key(0), *satir, "deadline-normal-timeout")?;
                self.ifade(sure, *satir, d + 1)?;
                self.blok(govde, d + 1)?;
                if let Some(b) = yetismezse {
                    self.blok(b, d + 1)?;
                }
            }
            Cumle::IstekGeldiginde {
                koken,
                yol,
                govde,
                satir,
                ..
            } => {
                let koken = koken
                    .as_deref()
                    .ok_or("kapsam rota kökeni derleme scope gerektirir")?;
                let onceki = (self.kaynak.clone(), self.sahip.clone(), self.tur.clone());
                self.sahip(koken, &format!("route:{satir}"), "route")?;
                self.ifade(yol, *satir, d + 1)?;
                self.blok(govde, d + 1)?;
                self.sahip(&onceki.0, &onceki.1, &onceki.2)?;
            }
            Cumle::Gorunum { dugumler, .. } => self.gorunum(dugumler, d + 1)?,
            Cumle::Yaz { deger: i, satir }
            | Cumle::Olsun {
                deger: i, satir, ..
            }
            | Cumle::Sor { istem: i, satir }
            | Cumle::Dondur {
                deger: i, satir, ..
            }
            | Cumle::CagriCumlesi { cagri: i, satir }
            | Cumle::SunucuBaslat { kapi: i, satir }
            | Cumle::YanitGonder {
                deger: i, satir, ..
            }
            | Cumle::Yonlendir { adres: i, satir }
            | Cumle::CerezSil { ad: i, satir }
            | Cumle::Bekle { sure: i, satir }
            | Cumle::Olmali { kosul: i, satir } => self.ifade(i, *satir, d + 1)?,
            Cumle::Artir {
                ifade: a,
                miktar: b,
                satir,
            }
            | Cumle::Azalt {
                ifade: a,
                miktar: b,
                satir,
            }
            | Cumle::Ekle {
                hedef: a,
                deger: b,
                satir,
            }
            | Cumle::Sil {
                kap: a,
                deger: b,
                satir,
            }
            | Cumle::CerezYaz {
                ad: a,
                deger: b,
                satir,
            }
            | Cumle::BolVeAta {
                pay: a,
                payda: b,
                satir,
                ..
            }
            | Cumle::AlanAta {
                nesne: a,
                deger: b,
                satir,
                ..
            }
            | Cumle::DosyayaYaz {
                yol: a,
                icerik: b,
                satir,
                ..
            }
            | Cumle::OturumAc {
                kullanici: a,
                rol: b,
                satir,
            } => {
                self.ifade(a, *satir, d + 1)?;
                self.ifade(b, *satir, d + 1)?;
            }
            Cumle::SozlukAta {
                sozluk,
                anahtar,
                deger,
                satir,
            } => {
                for i in [sozluk, anahtar, deger] {
                    self.ifade(i, *satir, d + 1)?;
                }
            }
            Cumle::HataDondur {
                mesaj,
                neden,
                veri,
                satir,
                ..
            } => {
                self.ifade(mesaj, *satir, d + 1)?;
                for i in [neden, veri].into_iter().flatten() {
                    self.ifade(i, *satir, d + 1)?;
                }
            }
            Cumle::ProgramiBitir { kod, satir } => {
                if let Some(i) = kod {
                    self.ifade(i, *satir, d + 1)?;
                }
            }
            Cumle::Eszamanli { gorevler, .. } => {
                for (_, i, s) in gorevler {
                    self.ifade(i, *s, d + 1)?;
                }
            }
            Cumle::IslemTanimi(_)
            | Cumle::TestBlogu(_)
            | Cumle::YapiTanimi(_)
            | Cumle::Kullan { .. } => {
                return Err("kapsam yalnız hoist edilmiş program kabul eder".into())
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

pub(super) fn kur(
    program: &Program,
    ana: &str,
    kaynaklar: BTreeMap<String, String>,
) -> Result<Toplayici, String> {
    let mut g = Gezici {
        t: Toplayici {
            dallar: Vec::new(),
            dizin: HashMap::new(),
            kaynaklar,
            hata: false,
        },
        kaynak: String::new(),
        sahip: String::new(),
        tur: String::new(),
        sira: 0,
    };
    g.sahip(ana, "main", "program")?;
    g.blok(&program.cumleler, 0)?;
    let mut islemler = program.islemler.values().collect::<Vec<_>>();
    islemler.sort_by(|a, b| (&a.koken, &a.ad).cmp(&(&b.koken, &b.ad)));
    for i in islemler {
        g.sahip(i.koken.as_deref().unwrap_or(ana), &i.ad, "procedure")?;
        g.blok(&i.govde, 0)?;
    }
    for (n, t) in program.testler.iter().enumerate() {
        g.sahip(
            t.koken.as_deref().unwrap_or(ana),
            &format!("{n}:{}", t.ad),
            "test",
        )?;
        g.blok(&t.govde, 0)?;
    }
    Ok(g.t)
}
