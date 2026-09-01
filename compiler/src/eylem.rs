//! K-087 uygulama eylemi ve web adaptörü sözleşmesi.
//!
//! Bu katman tür denetiminden önce, kapalı bir etki kümesiyle iki değişmezi
//! kanıtlar: uygulama durumu yalnız `eylem` sınırından değiştirilir ve
//! GET/HEAD adaptörleri hiçbir durum-yazma etkisine ulaşamaz.

use crate::agac::{Cumle, HttpYontemi, Ifade, Islem, IslemTuru, Program};
use crate::intrinsic::{self, IntrinsicEtkisi};
use crate::tani::Tani;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
struct Bilgi {
    /// Dosya/donanım gibi uygulama durumu. Web çerezi adaptör durumudur.
    uygulama_yazma: bool,
    /// Çerez yazma/silme dahil bütün kalıcı durum etkileri.
    durum_yazma: bool,
    /// HTTP'ye özgü yanıt, yönlendirme, çerez ya da rota yüzeyi.
    web: bool,
    /// Transaction adaptörünün geri alamadığı gözlemlenebilir dış etki.
    geri_alinamaz: bool,
    cagrilar: HashSet<String>,
}

impl Bilgi {
    fn birlestir(&mut self, baska: &Bilgi) {
        self.uygulama_yazma |= baska.uygulama_yazma;
        self.durum_yazma |= baska.durum_yazma;
        self.web |= baska.web;
        self.geri_alinamaz |= baska.geri_alinamaz;
        self.cagrilar.extend(baska.cagrilar.iter().cloned());
    }
}

pub(crate) fn denetle(program: &Program) -> Result<(), Tani> {
    eylem_imzalarini_denetle(&program.islemler)?;

    let dogrudan = program
        .islemler
        .iter()
        .map(|(ad, islem)| (ad.clone(), blok_bilgisi(&islem.govde)))
        .collect::<HashMap<_, _>>();
    let etkiler = etkileri_yay(&dogrudan);

    let mut adlar = program.islemler.keys().cloned().collect::<Vec<_>>();
    adlar.sort();
    for ad in adlar {
        let islem = program.islemler.get(&ad).expect("ad haritadan geldi");
        if islem.tur == IslemTuru::Eylem && etkiler.get(&ad).is_some_and(|etki| etki.web) {
            return Err(Tani::yeni(
                "T044",
                format!(
                    "\"{}\" eylemi web adaptörüne bağımlı; eylemler HTTP'den bağımsız olmalı.",
                    ad
                ),
                islem.satir,
                1,
                1,
            )
            .onerili(
                "Yanıt, yönlendirme ve çerez işini rotada bırak; eylem yalnız iş kuralını çalıştırıp değer döndürsün."
                    .into(),
            ));
        }
        if islem.tur == IslemTuru::Eylem && etkiler.get(&ad).is_some_and(|etki| etki.geri_alinamaz)
        {
            return Err(Tani::yeni(
                "T048",
                format!(
                    "\"{}\" eylemi geri alınamayan çıktı, girdi veya donanım etkisi taşıyor.",
                    ad
                ),
                islem.satir,
                1,
                1,
            )
            .onerili(
                "Eylemde yalnız transaction adaptörünün desteklediği kalıcı yazmaları kullan; ekran, soru ve donanım işini çağıran adaptörde bırak."
                    .into(),
            ));
        }
    }

    let mut rotalar = HashSet::new();
    for cumle in &program.cumleler {
        let Cumle::IstekGeldiginde {
            yontem,
            yol,
            onekli,
            govde,
            satir,
        } = cumle
        else {
            continue;
        };
        let yontem = yontem.unwrap_or(HttpYontemi::Get);
        rota_onsozunu_denetle(yontem, govde, *satir)?;
        let dogrudan_rota = blok_bilgisi(govde);
        let toplam = toplam_bilgi(&dogrudan_rota, &etkiler);

        if yontem.guvenli() && toplam.durum_yazma {
            return Err(Tani::yeni(
                "T045",
                format!(
                    "{} rotası durum değiştiremez; GET ve HEAD yalnız salt okumadır.",
                    yontem.yazimi()
                ),
                *satir,
                1,
                1,
            )
            .onerili(
                "Durum değiştiren işi açık bir POST/PUT/PATCH/DELETE rotasından `eylem` olarak çağır."
                    .into(),
            ));
        }

        if dogrudan_rota.uygulama_yazma {
            return Err(Tani::yeni(
                "T046",
                "Rota gövdesi uygulama durumunu doğrudan değiştiremez.".into(),
                *satir,
                1,
                1,
            )
            .onerili(
                "Dosya/donanım değişikliğini yeniden kullanılabilir bir `eylem` içine taşı ve rotadan onu çağır."
                    .into(),
            ));
        }

        for cagri in &dogrudan_rota.cagrilar {
            let Some(cagrilan) = program.islemler.get(cagri) else {
                continue;
            };
            if cagrilan.tur == IslemTuru::Islem
                && etkiler.get(cagri).is_some_and(|etki| etki.durum_yazma)
            {
                return Err(Tani::yeni(
                    "T046",
                    format!(
                        "Rota, durum değiştiren \"{}\" işlemini çağırıyor; bu sınır bir eylem olmalı.",
                        cagri
                    ),
                    *satir,
                    1,
                    1,
                )
                .onerili(format!(
                    "`işlem {0}` başlığını açık imzalı `eylem {0}` olarak değiştir.",
                    cagri
                )));
            }
        }

        if let Ifade::MetinSabiti(yol) = yol {
            let anahtar = (yontem, *onekli, yol.clone());
            if !rotalar.insert(anahtar) {
                return Err(Tani::yeni(
                    "T047",
                    format!(
                        "{} \"{}\" rotası birden çok kez tanımlandı.",
                        yontem.yazimi(),
                        yol
                    ),
                    *satir,
                    1,
                    1,
                ));
            }
        }
    }
    Ok(())
}

fn rota_onsozunu_denetle(
    yontem: HttpYontemi,
    govde: &[Cumle],
    rota_satiri: usize,
) -> Result<(), Tani> {
    let mut politika_goruldu = false;
    let mut onsoz_bitti = false;
    for cumle in govde {
        match cumle {
            Cumle::RotaPolitikasi { satir, .. } => {
                if politika_goruldu || onsoz_bitti {
                    return Err(Tani::yeni(
                        "T050",
                        "Rota erişim politikası gövdenin ilk ve tek politika cümlesi olmalı."
                            .into(),
                        *satir,
                        1,
                        1,
                    ));
                }
                politika_goruldu = true;
            }
            Cumle::RotaAlaniGerekli { satir, .. } => {
                if !politika_goruldu || onsoz_bitti {
                    return Err(Tani::yeni(
                        "T050",
                        "Zorunlu istek alanları erişim politikasının hemen ardından gelmeli."
                            .into(),
                        *satir,
                        1,
                        1,
                    ));
                }
            }
            _ => onsoz_bitti = true,
        }
    }
    if !yontem.guvenli() && !politika_goruldu {
        return Err(Tani::yeni(
            "T049",
            format!(
                "{} rotası açık bir erişim politikası taşımıyor.",
                yontem.yazimi()
            ),
            rota_satiri,
            1,
            1,
        )
        .onerili(
            "Gövdenin ilk satırına `herkese açık`, `oturum gerekli` ya da `\"rol\" yetkisi gerekli` yaz."
                .into(),
        ));
    }
    Ok(())
}

fn eylem_imzalarini_denetle(islemler: &HashMap<String, Islem>) -> Result<(), Tani> {
    let mut adlar = islemler.keys().cloned().collect::<Vec<_>>();
    adlar.sort();
    for ad in adlar {
        let islem = islemler.get(&ad).expect("ad haritadan geldi");
        if islem.tur != IslemTuru::Eylem {
            continue;
        }
        let parametreler_acik = islem
            .parametreler
            .iter()
            .all(|parametre| parametre.tur_yazimi.is_some());
        if !parametreler_acik || islem.donus_turu_yazimi.is_none() {
            return Err(Tani::yeni(
                "T043",
                format!(
                    "\"{}\" eylemi tam girdi ve dönüş sözleşmesi taşımıyor.",
                    islem.ad
                ),
                islem.satir,
                1,
                1,
            )
            .onerili(
                "Her girdiyi `<ad> <Tür> olarak al` biçiminde yaz; ardından `<Tür> döndürür` ya da `değer döndürmez` bildir."
                    .into(),
            ));
        }
    }
    Ok(())
}

fn etkileri_yay(dogrudan: &HashMap<String, Bilgi>) -> HashMap<String, Bilgi> {
    let mut sonuc = dogrudan.clone();
    loop {
        let onceki = sonuc.clone();
        let mut degisti = false;
        for (ad, bilgi) in &mut sonuc {
            let cagrilar = dogrudan
                .get(ad)
                .map(|bilgi| bilgi.cagrilar.clone())
                .unwrap_or_default();
            for cagri in cagrilar {
                if let Some(cagri_bilgisi) = onceki.get(&cagri) {
                    let eski = (
                        bilgi.uygulama_yazma,
                        bilgi.durum_yazma,
                        bilgi.web,
                        bilgi.geri_alinamaz,
                    );
                    bilgi.birlestir(cagri_bilgisi);
                    degisti |= eski
                        != (
                            bilgi.uygulama_yazma,
                            bilgi.durum_yazma,
                            bilgi.web,
                            bilgi.geri_alinamaz,
                        );
                }
            }
        }
        if !degisti {
            return sonuc;
        }
    }
}

fn toplam_bilgi(dogrudan: &Bilgi, etkiler: &HashMap<String, Bilgi>) -> Bilgi {
    let mut toplam = dogrudan.clone();
    for cagri in &dogrudan.cagrilar {
        if let Some(etki) = etkiler.get(cagri) {
            toplam.birlestir(etki);
        }
    }
    toplam
}

fn blok_bilgisi(cumleler: &[Cumle]) -> Bilgi {
    let mut bilgi = Bilgi::default();
    for cumle in cumleler {
        cumle_bilgisi(cumle, &mut bilgi);
    }
    bilgi
}

fn cumle_bilgisi(cumle: &Cumle, bilgi: &mut Bilgi) {
    match cumle {
        Cumle::Yaz { deger, .. } | Cumle::Sor { istem: deger, .. } => {
            bilgi.geri_alinamaz = true;
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::Olmali { kosul: deger, .. } | Cumle::Dondur { deger, .. } => {
            ifade_bilgisi(deger, bilgi)
        }
        Cumle::HataDondur { mesaj, neden, veri, .. } => {
            ifade_bilgisi(mesaj, bilgi);
            if let Some(neden) = neden {
                ifade_bilgisi(neden, bilgi);
            }
            if let Some(veri) = veri {
                ifade_bilgisi(veri, bilgi);
            }
        }
        Cumle::Olsun { deger, .. } => ifade_bilgisi(deger, bilgi),
        Cumle::KezTekrarla { adet, govde, .. } => {
            ifade_bilgisi(adet, bilgi);
            bilgi.birlestir(&blok_bilgisi(govde));
        }
        Cumle::AralikDongusu {
            bastan,
            sona,
            govde,
            ..
        } => {
            ifade_bilgisi(bastan, bilgi);
            ifade_bilgisi(sona, bilgi);
            bilgi.birlestir(&blok_bilgisi(govde));
        }
        Cumle::OlduguSurece { kosul, govde, .. } | Cumle::OlanaKadar { kosul, govde, .. } => {
            ifade_bilgisi(kosul, bilgi);
            bilgi.birlestir(&blok_bilgisi(govde));
        }
        Cumle::Ise {
            kollar, degilse, ..
        } => {
            for kol in kollar {
                ifade_bilgisi(&kol.kosul, bilgi);
                bilgi.birlestir(&blok_bilgisi(&kol.govde));
            }
            if let Some(govde) = degilse {
                bilgi.birlestir(&blok_bilgisi(govde));
            }
        }
        Cumle::Artir { ifade, miktar, .. } | Cumle::Azalt { ifade, miktar, .. } => {
            ifade_bilgisi(ifade, bilgi);
            ifade_bilgisi(miktar, bilgi);
        }
        Cumle::Ekle { hedef, deger, .. } => {
            ifade_bilgisi(hedef, bilgi);
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::HerBiri { kaynak, govde, .. } => {
            if let Some(kaynak) = kaynak {
                ifade_bilgisi(kaynak, bilgi);
            }
            bilgi.birlestir(&blok_bilgisi(govde));
        }
        Cumle::Gore {
            konu,
            kollar,
            degilse,
            ..
        } => {
            ifade_bilgisi(konu, bilgi);
            for (deger, govde) in kollar {
                ifade_bilgisi(deger, bilgi);
                bilgi.birlestir(&blok_bilgisi(govde));
            }
            if let Some(govde) = degilse {
                bilgi.birlestir(&blok_bilgisi(govde));
            }
        }
        Cumle::AlanAta { nesne, deger, .. } => {
            ifade_bilgisi(nesne, bilgi);
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::BolVeAta { pay, payda, .. } => {
            ifade_bilgisi(pay, bilgi);
            ifade_bilgisi(payda, bilgi);
        }
        Cumle::CagriCumlesi { cagri, .. } => ifade_bilgisi(cagri, bilgi),
        Cumle::ProgramiBitir { kod, .. } => {
            if let Some(kod) = kod {
                ifade_bilgisi(kod, bilgi);
            }
        }
        Cumle::SunucuBaslat { kapi, .. } => {
            bilgi.web = true;
            ifade_bilgisi(kapi, bilgi);
        }
        Cumle::IstekGeldiginde { yol, govde, .. } => {
            bilgi.web = true;
            ifade_bilgisi(yol, bilgi);
            bilgi.birlestir(&blok_bilgisi(govde));
        }
        Cumle::YanitGonder { deger, .. } | Cumle::Yonlendir { adres: deger, .. } => {
            bilgi.web = true;
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::CerezYaz { ad, deger, .. } => {
            bilgi.web = true;
            bilgi.durum_yazma = true;
            ifade_bilgisi(ad, bilgi);
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::CerezSil { ad, .. } => {
            bilgi.web = true;
            bilgi.durum_yazma = true;
            ifade_bilgisi(ad, bilgi);
        }
        Cumle::RotaPolitikasi { .. } | Cumle::RotaAlaniGerekli { .. } => {
            bilgi.web = true;
        }
        Cumle::OturumAc { kullanici, rol, .. } => {
            bilgi.web = true;
            bilgi.durum_yazma = true;
            ifade_bilgisi(kullanici, bilgi);
            ifade_bilgisi(rol, bilgi);
        }
        Cumle::OturumKapat { .. } => {
            bilgi.web = true;
            bilgi.durum_yazma = true;
        }
        Cumle::Sil { kap, deger, .. } => {
            ifade_bilgisi(kap, bilgi);
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::Eszamanli { gorevler, .. } => {
            for (_, ifade, _) in gorevler {
                ifade_bilgisi(ifade, bilgi);
            }
        }
        Cumle::IcindeBlogu {
            sure,
            govde,
            yetismezse,
            ..
        } => {
            ifade_bilgisi(sure, bilgi);
            bilgi.birlestir(&blok_bilgisi(govde));
            if let Some(govde) = yetismezse {
                bilgi.birlestir(&blok_bilgisi(govde));
            }
        }
        Cumle::IsikAyarla { .. } => {
            bilgi.uygulama_yazma = true;
            bilgi.durum_yazma = true;
            bilgi.geri_alinamaz = true;
        }
        Cumle::Bekle { sure, .. } => ifade_bilgisi(sure, bilgi),
        Cumle::SozlukAta {
            sozluk,
            anahtar,
            deger,
            ..
        } => {
            ifade_bilgisi(sozluk, bilgi);
            ifade_bilgisi(anahtar, bilgi);
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::DosyayaYaz { yol, icerik, .. } => {
            bilgi.uygulama_yazma = true;
            bilgi.durum_yazma = true;
            ifade_bilgisi(yol, bilgi);
            ifade_bilgisi(icerik, bilgi);
        }
        Cumle::IslemTanimi(_)
        | Cumle::YapiTanimi(_)
        | Cumle::Kullan { .. }
        | Cumle::TestBlogu(_)
        | Cumle::HepsiniBekle { .. } => {}
    }
}

fn ifade_bilgisi(ifade: &Ifade, bilgi: &mut Bilgi) {
    match ifade {
        Ifade::IslemCagrisi {
            islem_adi,
            argumanlar,
            ..
        } => {
            bilgi.cagrilar.insert(islem_adi.clone());
            for arguman in argumanlar {
                ifade_bilgisi(arguman, bilgi);
            }
        }
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
            ifade_bilgisi(metin, bilgi);
            ifade_bilgisi(ayrac, bilgi);
        }
        Ifade::Degistir { metin, eski, yeni } => {
            ifade_bilgisi(metin, bilgi);
            ifade_bilgisi(eski, bilgi);
            ifade_bilgisi(yeni, bilgi);
        }
        Ifade::MetinSinari { metin, parca, .. }
        | Ifade::Icerir {
            metin,
            aranan: parca,
        } => {
            ifade_bilgisi(metin, bilgi);
            ifade_bilgisi(parca, bilgi);
        }
        Ifade::Rastgele { alt, ust } => {
            ifade_bilgisi(alt, bilgi);
            ifade_bilgisi(ust, bilgi);
        }
        Ifade::Intrinsic { kimlik, argumanlar } => {
            for arguman in argumanlar {
                ifade_bilgisi(arguman, bilgi);
            }
            // Uygulama durumu değil, güvenlik adaptörünün kısa ömürlü ve
            // anlam taşımayan synchronizer oturumudur. GET'te üretilebilir.
            if intrinsic::tanim(kimlik)
                .is_some_and(|tanim| tanim.etki == IntrinsicEtkisi::WebAdaptoru)
            {
                bilgi.web = true;
            }
        }
        Ifade::Birlestir(parcalar)
        | Ifade::MantiksalZincir { parcalar, .. }
        | Ifade::ListeSabiti(parcalar) => {
            for parca in parcalar {
                ifade_bilgisi(parca, bilgi);
            }
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
        | Ifade::AlanErisim { nesne: ic, .. } => ifade_bilgisi(ic, bilgi),
        Ifade::SozlukDegeri { sozluk, anahtar }
        | Ifade::SozlukteVar {
            sozluk, anahtar, ..
        } => {
            ifade_bilgisi(sozluk, bilgi);
            ifade_bilgisi(anahtar, bilgi);
        }
        Ifade::GunSonrasi { tarih, miktar } => {
            ifade_bilgisi(tarih, bilgi);
            ifade_bilgisi(miktar, bilgi);
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
    }
}
