//! Programın dış dünya gereksinimlerini merkezî çalışma politikasına bağlar.
use crate::agac::{Cumle, Ifade, Program, RotaErisimi};
use crate::intrinsic::{self, HTTP_GETIR};
use crate::tani::Tani;
use crate::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};
use std::collections::BTreeMap;
type Konum = (usize, usize, usize);
#[derive(Default)]
struct Bilgi {
    yetkinlikler: BTreeMap<Yetkinlik, Konum>,
    ag_istekleri: Vec<(String, Konum)>,
}
impl Bilgi {
    fn ekle(&mut self, yetkinlik: Yetkinlik, konum: Konum) {
        if yetkinlik == Yetkinlik::WebOturumu {
            self.ekle(Yetkinlik::AgSunucusu, konum);
        }
        self.yetkinlikler
            .entry(yetkinlik)
            .and_modify(|onceki| *onceki = (*onceki).min(konum))
            .or_insert(konum);
    }
}
pub(super) fn denetle(program: &Program, politika: &YetkinlikPolitikasi) -> Result<(), Tani> {
    let mut bilgi = Bilgi::default();
    blok_bilgisi(&program.cumleler, &mut bilgi);
    for islem in program.islemler.values() {
        blok_bilgisi(&islem.govde, &mut bilgi);
    }
    for test in &program.testler {
        blok_bilgisi(&test.govde, &mut bilgi);
    }
    let reddedilen = bilgi
        .yetkinlikler
        .iter()
        .filter(|(yetkinlik, _)| !politika.izin_verir(**yetkinlik))
        .min_by_key(|(_, konum)| **konum);
    if let Some((yetkinlik, (satir, sutun, uzunluk))) = reddedilen {
        return Err(Tani::yeni(
            "T054",
            format!(
                "Program `{}` yetkinliğini kullanıyor ama çalışma politikası bunu açmıyor.",
                yetkinlik.yazimi()
            ),
            *satir,
            *sutun,
            *uzunluk,
        )
        .onerili(format!(
            "Güven sınırını inceledikten sonra proje.dil içindeki `yetkinlikler` listesine \"{}\" ekle.",
            yetkinlik.yazimi()
        )));
    }
    bilgi
        .ag_istekleri
        .sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    for (url, (satir, sutun, uzunluk)) in &bilgi.ag_istekleri {
        if let Err(neden) = politika.ag_istegini_denetle(url) {
            return Err(Tani::yeni(
                "T054",
                format!("Sabit outbound ağ hedefi çalışma politikasına uymuyor: {}.", neden),
                *satir,
                *sutun,
                *uzunluk,
            )
            .onerili(
                "Hedefi proje.dil içindeki `ağ_hedefleri` listesine tam şema+host+port olarak ekle."
                    .into(),
            ));
        }
    }
    Ok(())
}
fn blok_bilgisi(cumleler: &[Cumle], bilgi: &mut Bilgi) {
    for cumle in cumleler {
        cumle_bilgisi(cumle, bilgi);
    }
}
fn cumle_bilgisi(cumle: &Cumle, bilgi: &mut Bilgi) {
    match cumle {
        Cumle::Yaz { deger, .. }
        | Cumle::Olsun { deger, .. }
        | Cumle::Olmali { kosul: deger, .. }
        | Cumle::Dondur { deger, .. } => ifade_bilgisi(deger, bilgi),
        Cumle::Sor { istem, .. } => ifade_bilgisi(istem, bilgi),
        Cumle::HataDondur {
            mesaj, neden, veri, ..
        } => {
            ifade_bilgisi(mesaj, bilgi);
            neden.iter().for_each(|ifade| ifade_bilgisi(ifade, bilgi));
            veri.iter().for_each(|ifade| ifade_bilgisi(ifade, bilgi));
        }
        Cumle::KezTekrarla { adet, govde, .. } => {
            ifade_bilgisi(adet, bilgi);
            blok_bilgisi(govde, bilgi);
        }
        Cumle::AralikDongusu {
            bastan,
            sona,
            govde,
            ..
        } => {
            ifade_bilgisi(bastan, bilgi);
            ifade_bilgisi(sona, bilgi);
            blok_bilgisi(govde, bilgi);
        }
        Cumle::OlduguSurece { kosul, govde, .. } | Cumle::OlanaKadar { kosul, govde, .. } => {
            ifade_bilgisi(kosul, bilgi);
            blok_bilgisi(govde, bilgi);
        }
        Cumle::Ise {
            kollar, degilse, ..
        } => {
            for kol in kollar {
                ifade_bilgisi(&kol.kosul, bilgi);
                blok_bilgisi(&kol.govde, bilgi);
            }
            if let Some(govde) = degilse {
                blok_bilgisi(govde, bilgi);
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
            kaynak.iter().for_each(|ifade| ifade_bilgisi(ifade, bilgi));
            blok_bilgisi(govde, bilgi);
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
                blok_bilgisi(govde, bilgi);
            }
            if let Some(govde) = degilse {
                blok_bilgisi(govde, bilgi);
            }
        }
        Cumle::IslemTanimi(islem) => blok_bilgisi(&islem.govde, bilgi),
        Cumle::TestBlogu(test) => blok_bilgisi(&test.govde, bilgi),
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
            kod.iter().for_each(|ifade| ifade_bilgisi(ifade, bilgi));
        }
        Cumle::SunucuBaslat { kapi, satir } => {
            bilgi.ekle(Yetkinlik::AgSunucusu, (*satir, 1, 1));
            ifade_bilgisi(kapi, bilgi);
        }
        Cumle::IstekGeldiginde {
            yol, govde, satir, ..
        } => {
            bilgi.ekle(Yetkinlik::AgSunucusu, (*satir, 1, 1));
            ifade_bilgisi(yol, bilgi);
            blok_bilgisi(govde, bilgi);
        }
        Cumle::YanitGonder { deger, satir } => {
            bilgi.ekle(Yetkinlik::AgSunucusu, (*satir, 1, 1));
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::Yonlendir { adres, satir } => {
            bilgi.ekle(Yetkinlik::AgSunucusu, (*satir, 1, 1));
            ifade_bilgisi(adres, bilgi);
        }
        Cumle::CerezYaz { ad, deger, satir } => {
            web_oturumu_ekle(*satir, bilgi);
            ifade_bilgisi(ad, bilgi);
            ifade_bilgisi(deger, bilgi);
        }
        Cumle::CerezSil { ad, satir } => {
            web_oturumu_ekle(*satir, bilgi);
            ifade_bilgisi(ad, bilgi);
        }
        Cumle::RotaPolitikasi {
            erisim: RotaErisimi::HerkeseAcik,
            satir,
        } => bilgi.ekle(Yetkinlik::AgSunucusu, (*satir, 1, 1)),
        Cumle::RotaPolitikasi { satir, .. } => web_oturumu_ekle(*satir, bilgi),
        Cumle::RotaAlaniGerekli { satir, .. } => {
            bilgi.ekle(Yetkinlik::AgSunucusu, (*satir, 1, 1));
        }
        Cumle::OturumAc {
            kullanici,
            rol,
            satir,
        } => {
            web_oturumu_ekle(*satir, bilgi);
            ifade_bilgisi(kullanici, bilgi);
            ifade_bilgisi(rol, bilgi);
        }
        Cumle::OturumKapat { satir } => web_oturumu_ekle(*satir, bilgi),
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
            blok_bilgisi(govde, bilgi);
            if let Some(govde) = yetismezse {
                blok_bilgisi(govde, bilgi);
            }
        }
        Cumle::IsikAyarla { satir, .. } => {
            bilgi.ekle(Yetkinlik::Donanim, (*satir, 1, 1));
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
        Cumle::DosyayaYaz {
            yol, icerik, satir, ..
        } => {
            bilgi.ekle(Yetkinlik::DosyaYazma, (*satir, 1, 1));
            ifade_bilgisi(yol, bilgi);
            ifade_bilgisi(icerik, bilgi);
        }
        Cumle::YapiTanimi(_) | Cumle::Kullan { .. } | Cumle::HepsiniBekle { .. } => {}
    }
}
fn web_oturumu_ekle(satir: usize, bilgi: &mut Bilgi) {
    bilgi.ekle(Yetkinlik::AgSunucusu, (satir, 1, 1));
    bilgi.ekle(Yetkinlik::WebOturumu, (satir, 1, 1));
}
fn ifade_bilgisi(ifade: &Ifade, bilgi: &mut Bilgi) {
    match ifade.turu() {
        Ifade::IslemCagrisi { argumanlar, .. } | Ifade::Intrinsic { argumanlar, .. } => {
            if let Ifade::Intrinsic { kimlik, .. } = ifade.turu() {
                if let Some(tanim) = intrinsic::tanim(kimlik) {
                    bilgi.ekle(tanim.yetkinlik, ifade_konumu(ifade));
                }
                if kimlik == HTTP_GETIR {
                    if let Some(Ifade::MetinSabiti(url)) = argumanlar.first().map(Ifade::turu) {
                        bilgi.ag_istekleri.push((url.clone(), ifade_konumu(ifade)));
                    }
                }
            }
            argumanlar
                .iter()
                .for_each(|ifade| ifade_bilgisi(ifade, bilgi));
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
        Ifade::Birlestir(parcalar)
        | Ifade::MantiksalZincir { parcalar, .. }
        | Ifade::ListeSabiti(parcalar) => {
            parcalar
                .iter()
                .for_each(|ifade| ifade_bilgisi(ifade, bilgi));
        }
        Ifade::DosyaOkumayiDene(ic)
        | Ifade::DosyaSatirlari(ic)
        | Ifade::TabloOku(ic)
        | Ifade::VeriOku(ic) => {
            bilgi.ekle(Yetkinlik::DosyaOkuma, ifade_konumu(ifade));
            ifade_bilgisi(ic, bilgi);
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
        | Ifade::YeniYapi { .. }
        | Ifade::Kaynakli { .. } => {}
    }
}

fn ifade_konumu(ifade: &Ifade) -> Konum {
    ifade
        .kaynak_araligi()
        .map(|aralik| aralik.uclu())
        .unwrap_or((1, 1, 1))
}
