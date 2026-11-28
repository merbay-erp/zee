//! Opt-in araç uyarıları: çalıştırma ve tür denetimi sonucunu değiştirmez.
use super::{HirBagi, HirProgram};
use crate::{
    agac::{Cumle, Ifade},
    tani::Tani,
};
use std::collections::HashSet;

impl HirProgram {
    /// İç araç yüzeyi; yalnız istenen ana kaynağa ait, başarıyla bağlanmış kod.
    /// Bağımlılık tanımları aynı satırda olsa bile yerel uyarı sayılamaz.
    pub fn uyarilari(&self, koken: Option<&str>) -> Vec<Tani> {
        let mut kapsamlar = HashSet::from([0]);
        let mut sonuc = Vec::new();
        // İfade içindeki kullanıcı çağrısı da yanıt etkisi taşıyabilir. Bu dar
        // lint interprocedural etki çözmez; herhangi bir böyle bağ varken
        // S048'i program boyunca bastırır, diğer uyarılar çalışmaya devam eder.
        let cagri_etkisi_belirsiz = self
            .ifadeler
            .values()
            .any(|b| matches!(b.bag(), HirBagi::Islem(_)));
        for kimlik in self.islem_kimlikleri() {
            if let Some(islem) = self.islem(kimlik) {
                if yerel(islem.koken.as_deref(), koken) {
                    kapsamlar.insert(kimlik.sirasi() + 1);
                    blok(&islem.govde, &mut sonuc, false);
                }
            }
        }
        for (i, test) in self.program.testler.iter().enumerate() {
            if yerel(test.koken.as_deref(), koken) {
                kapsamlar.insert((1usize << 30) + i);
                blok(&test.govde, &mut sonuc, false);
            }
        }
        // Kapsam kimlikleri checker Baglam ile aynı tanım sözleşmesidir.
        for c in &self.program.cumleler {
            if let Cumle::IstekGeldiginde {
                koken: sahibi,
                yol,
                govde,
                satir,
                ..
            } = c
            {
                if !yerel(sahibi.as_deref(), koken) {
                    continue;
                }
                if let Some(bilgi) = self.ifade_bilgisi(yol) {
                    kapsamlar.insert((1usize << 31) + bilgi.kimlik().sirasi());
                }
                // K-403: rota gövdesinde yanıt/yönlendirme rotayı bitirir.
                blok(govde, &mut sonuc, true);
                if !cagri_etkisi_belirsiz && !yanit_var(govde) && !belirsiz_cagri(govde) {
                    ekle(&mut sonuc, Tani::yeni("S048", "Bu rotanın bazı yolları yanıt veya yönlendirme üretmeden bitebilir.".into(), *satir, 1, 1)
                        .onerili("Her dala uygun yanıt ekle; örnek: değilse → girintili `\"Geçersiz istek\" yanıtını 400 durumuyla gönder`. Yanıt veya yönlendirme rotayı orada bitirir.".into()));
                }
            }
        }
        self.baglanti_bloklari(&mut kapsamlar, &mut sonuc);
        blok(&self.program.cumleler, &mut sonuc, false);
        let okunanlar: HashSet<_> = self
            .ifadeler
            .values()
            .filter_map(|b| match b.bag() {
                HirBagi::Sembol(k) => Some(k),
                _ => None,
            })
            .collect();
        for (kimlik, aralik) in &self.sembol_tanimlari {
            if !kapsamlar.contains(&kimlik.kapsam()) || okunanlar.contains(kimlik) {
                continue;
            }
            let Some(ad) = self.sembol_adi(*kimlik) else {
                continue;
            };
            let (satir, sutun, uzunluk) = aralik.kesin_konumu().unwrap_or((aralik.satiri(), 1, 1));
            ekle(&mut sonuc, Tani::yeni("A012", format!("\"{ad}\" adı tanımlanıyor ama okunmuyor."), satir, sutun, uzunluk)
                .onerili("Adı kullan veya gereksiz bağlamayı kaldır. Değer ifadesinin etkisini incelemeden satırı silme.".into()));
        }
        crate::tani::tanilari_sirala_ve_sinirla(&mut sonuc);
        sonuc
    }
}

impl HirProgram {
    /// K-381: bağlantı blokları rota gibi kendi kapsamında denetlenir.
    fn baglanti_bloklari(&self, kapsamlar: &mut HashSet<usize>, sonuc: &mut Vec<Tani>) {
        for c in &self.program.cumleler {
            if let Cumle::BaglantiGeldiginde {
                dinleyici, govde, ..
            } = c
            {
                if let Some(bilgi) = self.ifade_bilgisi(dinleyici) {
                    kapsamlar.insert((1usize << 31) + bilgi.kimlik().sirasi());
                }
                blok(govde, sonuc, false);
            }
        }
    }
}

fn yerel(sahibi: Option<&str>, ana: Option<&str>) -> bool {
    sahibi.is_none() || sahibi == ana
}

/// `rota` doğruysa blok bir rota gövdesine aittir: K-403 uyarınca orada
/// `yanıtını gönder`/`yönlendir` de kesin sondur.
fn blok(cs: &[Cumle], out: &mut Vec<Tani>, rota: bool) {
    let mut bitti = false;
    let mut yanitla_bitti = false;
    for c in cs {
        // İşlem/test/rota tanımları üst düzey çalışma akışının cümlesi değildir.
        if matches!(
            c,
            Cumle::IslemTanimi(_)
                | Cumle::YapiTanimi(_)
                | Cumle::TestBlogu(_)
                | Cumle::IstekGeldiginde { .. }
                | Cumle::BaglantiGeldiginde { .. }
        ) {
            continue;
        }
        if bitti && yanitla_bitti {
            ekle(out, Tani::yeni("S047", "Bu cümleye ulaşılamaz: önceki yanıt veya yönlendirme rotayı bitirir.".into(), satir(c), 1, 1)
                .onerili("Yanıttan sonra çalışması gereken işi yanıttan önceye taşı; yalnız yanıt yolunda kalmayacak kodu `değilse` dalına al.".into()));
            break;
        }
        if bitti {
            ekle(out, Tani::yeni("S047", "Bu cümleye önceki kesin dönüş veya program sonlandırması nedeniyle ulaşılamaz.".into(), satir(c), 1, 1)
                .onerili("Cümleyi uygun dala taşı veya ulaşılamayan kodu kaldır; dönüşün sırasını bilinçli seç.".into()));
            // Bir ölü bölge için tek kök uyarı.
            break;
        }
        match c {
            Cumle::Ise {
                kollar, degilse, ..
            } => {
                let mut secildi = false;
                for kol in kollar {
                    if secildi || matches!(kol.kosul.turu(), Ifade::MantiksalSabiti(false)) {
                        if let Some(ilk) = kol.govde.first() {
                            ekle(
                                out,
                                Tani::yeni(
                                    "S047",
                                    "Sabit koşul bu dalı erişilemez yapıyor.".into(),
                                    satir(ilk),
                                    1,
                                    1,
                                )
                                .onerili("Koşulu düzelt veya erişilemeyen dalı kaldır.".into()),
                            );
                        }
                    } else {
                        blok(&kol.govde, out, rota);
                    }
                    secildi |= matches!(kol.kosul.turu(), Ifade::MantiksalSabiti(true));
                }
                if let Some(d) = degilse {
                    if secildi {
                        if let Some(ilk) = d.first() {
                            ekle(
                                out,
                                Tani::yeni(
                                    "S047",
                                    "Önceki sabit doğru koşul nedeniyle değilse dalına ulaşılamaz."
                                        .into(),
                                    satir(ilk),
                                    1,
                                    1,
                                )
                                .onerili(
                                    "Sabit koşulu veya gereksiz değilse dalını gözden geçir."
                                        .into(),
                                ),
                            );
                        }
                    } else {
                        blok(d, out, rota);
                    }
                }
            }
            Cumle::Gore {
                kollar, degilse, ..
            } => {
                for (_, b) in kollar {
                    blok(b, out, rota);
                }
                if let Some(b) = degilse {
                    blok(b, out, rota);
                }
            }
            Cumle::KezTekrarla { govde, .. }
            | Cumle::HerBiri { govde, .. }
            | Cumle::AralikDongusu { govde, .. }
            | Cumle::OlduguSurece { govde, .. }
            | Cumle::OlanaKadar { govde, .. } => blok(govde, out, rota),
            Cumle::IcindeBlogu {
                govde, yetismezse, ..
            } => {
                blok(govde, out, rota);
                if let Some(b) = yetismezse {
                    blok(b, out, rota);
                }
            }
            _ => {}
        }
        bitti = kesin_son(c, rota);
        yanitla_bitti = bitti && !kesin_son(c, false);
    }
}

fn kesin_son(c: &Cumle, rota: bool) -> bool {
    let kesin_son = |c: &Cumle| kesin_son(c, rota);
    match c {
        Cumle::Dondur { .. } | Cumle::HataDondur { .. } | Cumle::ProgramiBitir { .. } => true,
        Cumle::YanitGonder { .. } | Cumle::Yonlendir { .. } => rota,
        Cumle::Ise {
            kollar, degilse, ..
        } => {
            !kollar.is_empty()
                && kollar.iter().all(|k| k.govde.iter().any(kesin_son))
                && degilse.as_ref().is_some_and(|b| b.iter().any(kesin_son))
        }
        Cumle::Gore {
            kollar, degilse, ..
        } => {
            !kollar.is_empty()
                && kollar.iter().all(|(_, b)| b.iter().any(kesin_son))
                && degilse.as_ref().is_some_and(|b| b.iter().any(kesin_son))
        }
        Cumle::IcindeBlogu {
            govde, yetismezse, ..
        } => {
            govde.iter().any(kesin_son)
                && yetismezse.as_ref().is_some_and(|b| b.iter().any(kesin_son))
        }
        _ => false,
    }
}

// K-403 sonrası yanıt rotayı bitirir; burada yalnız bütün olağan yollarda
// en az bir yanıtın bulunup bulunmadığı incelenir. Döngü çalışabilir/çalışmayabilir.
fn yanit_var(cs: &[Cumle]) -> bool {
    cs.iter().any(|c| match c {
        Cumle::YanitGonder { .. } | Cumle::Yonlendir { .. } | Cumle::ProgramiBitir { .. } => true,
        Cumle::Ise {
            kollar, degilse, ..
        } => {
            !kollar.is_empty()
                && kollar.iter().all(|k| yanit_var(&k.govde))
                && degilse.as_deref().is_some_and(yanit_var)
        }
        Cumle::Gore {
            kollar, degilse, ..
        } => {
            !kollar.is_empty()
                && kollar.iter().all(|(_, b)| yanit_var(b))
                && degilse.as_deref().is_some_and(yanit_var)
        }
        _ => false,
    })
}

// Çağrı etkisi için bu lint ayrı interprocedural kanıt üretmez: belirsizse susar.
fn belirsiz_cagri(cs: &[Cumle]) -> bool {
    cs.iter().any(|c| match c {
        Cumle::CagriCumlesi { .. } => true,
        Cumle::Ise {
            kollar, degilse, ..
        } => {
            kollar.iter().any(|k| belirsiz_cagri(&k.govde))
                || degilse.as_deref().is_some_and(belirsiz_cagri)
        }
        Cumle::Gore {
            kollar, degilse, ..
        } => {
            kollar.iter().any(|(_, b)| belirsiz_cagri(b))
                || degilse.as_deref().is_some_and(belirsiz_cagri)
        }
        Cumle::KezTekrarla { govde, .. }
        | Cumle::HerBiri { govde, .. }
        | Cumle::AralikDongusu { govde, .. }
        | Cumle::OlduguSurece { govde, .. }
        | Cumle::OlanaKadar { govde, .. } => belirsiz_cagri(govde),
        Cumle::IcindeBlogu {
            govde, yetismezse, ..
        } => belirsiz_cagri(govde) || yetismezse.as_deref().is_some_and(belirsiz_cagri),
        _ => false,
    })
}

fn satir(c: &Cumle) -> usize {
    match c {
        Cumle::Yaz { satir, .. }
        | Cumle::Olsun { satir, .. }
        | Cumle::Gorunum { satir, .. }
        | Cumle::KezTekrarla { satir, .. }
        | Cumle::AralikDongusu { satir, .. }
        | Cumle::OlduguSurece { satir, .. }
        | Cumle::OlanaKadar { satir, .. }
        | Cumle::Ise { satir, .. }
        | Cumle::Artir { satir, .. }
        | Cumle::Azalt { satir, .. }
        | Cumle::Sor { satir, .. }
        | Cumle::Ekle { satir, .. }
        | Cumle::HerBiri { satir, .. }
        | Cumle::Gore { satir, .. }
        | Cumle::Kullan { satir, .. }
        | Cumle::Olmali { satir, .. }
        | Cumle::AlanAta { satir, .. }
        | Cumle::Dondur { satir, .. }
        | Cumle::HataDondur { satir, .. }
        | Cumle::BolVeAta { satir, .. }
        | Cumle::CagriCumlesi { satir, .. }
        | Cumle::ProgramiBitir { satir, .. }
        | Cumle::SunucuBaslat { satir, .. }
        | Cumle::IstekGeldiginde { satir, .. }
        | Cumle::BaglantiGeldiginde { satir, .. }
        | Cumle::BaglantiyaYaz { satir, .. }
        | Cumle::BaglantiyiKapat { satir }
        | Cumle::YanitGonder { satir, .. }
        | Cumle::YanitEtagi { satir, .. }
        | Cumle::Yonlendir { satir, .. }
        | Cumle::CerezYaz { satir, .. }
        | Cumle::Sil { satir, .. }
        | Cumle::CerezSil { satir, .. }
        | Cumle::RotaPolitikasi { satir, .. }
        | Cumle::RotaAlaniGerekli { satir, .. }
        | Cumle::OturumAc { satir, .. }
        | Cumle::OturumKapat { satir, .. }
        | Cumle::Eszamanli { satir, .. }
        | Cumle::HepsiniBekle { satir, .. }
        | Cumle::IcindeBlogu { satir, .. }
        | Cumle::IsikAyarla { satir, .. }
        | Cumle::Bekle { satir, .. }
        | Cumle::SozlukAta { satir, .. }
        | Cumle::DosyayaYaz { satir, .. } => *satir,
        Cumle::IslemTanimi(x) => x.satir,
        Cumle::YapiTanimi(x) => x.satir,
        Cumle::TestBlogu(x) => x.satir,
    }
}

fn ekle(out: &mut Vec<Tani>, tani: Tani) {
    out.push(tani);
    crate::tani::tanilari_sirala_ve_sinirla(out);
}
