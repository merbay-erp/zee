use super::*;

/// K-212 görünüm bloğu: her satır bir düğümdür, girinti çocukları verir.
impl Ayristirici {
    pub(super) fn gorunum_ayristir(
        &mut self,
        ad: String,
        satir_no: usize,
        sutun: usize,
        uzunluk: usize,
    ) -> Result<Cumle, Tani> {
        match self.bak().tur {
            TokenTur::Girinti => {
                self.ilerle();
            }
            _ => {
                return Err(Tani::yeni(
                    "S007",
                    "Bu satırdan sonra girintili bir blok bekleniyor.".into(),
                    satir_no,
                    1,
                    1,
                )
                .onerili(
                    "Görünüm satırlarını 4 boşluk içeriden yaz: `form method \"post\"`.".into(),
                ))
            }
        }
        self.derinlik += 1;
        let dugumler = self.gorunum_dugumleri()?;
        self.derinlik -= 1;
        if let TokenTur::Cikinti = self.bak().tur {
            self.ilerle();
        }
        if dugumler.is_empty() {
            return Err(gorunum_tanisi(
                "Görünüm bloğu boş.",
                satir_no,
                "En az bir öğe ya da metin satırı yaz.",
            ));
        }
        Ok(Cumle::Gorunum {
            ad,
            dugumler,
            satir: satir_no,
            sutun,
            uzunluk,
        })
    }

    fn gorunum_dugumleri(&mut self) -> Result<Vec<GorunumDugumu>, Tani> {
        let mut dugumler = Vec::new();
        loop {
            match self.bak().tur {
                TokenTur::Cikinti | TokenTur::DosyaSonu => break,
                TokenTur::SatirSonu => {
                    self.ilerle();
                }
                TokenTur::Girinti => {
                    let satir = self.bak().satir;
                    return Err(gorunum_tanisi(
                        "Görünümde beklenmeyen girinti.",
                        satir,
                        "Çocuk satırlar yalnız bir öğe satırının altında, 4 boşluk içeride olur.",
                    ));
                }
                _ => {
                    let tokenlar = self.satir_oku();
                    let mut dugum = gorunum_dugumu_kur(tokenlar)?;
                    if let TokenTur::Girinti = self.bak().tur {
                        self.ilerle();
                        self.derinlik += 1;
                        let cocuklar = self.gorunum_dugumleri()?;
                        self.derinlik -= 1;
                        if let TokenTur::Cikinti = self.bak().tur {
                            self.ilerle();
                        }
                        match &mut dugum {
                            GorunumDugumu::Oge { cocuklar: yer, .. } => *yer = cocuklar,
                            GorunumDugumu::Metin { satir, .. }
                            | GorunumDugumu::Ham { satir, .. } => {
                                return Err(gorunum_tanisi(
                                    "Metin ve ham satırların çocuğu olamaz.",
                                    *satir,
                                    "Çocukları bir öğe satırının (ör. `div`) altına al.",
                                ));
                            }
                        }
                    }
                    dugumler.push(dugum);
                }
            }
        }
        Ok(dugumler)
    }
}

fn gorunum_tanisi(mesaj: &str, satir: usize, oneri: &str) -> Tani {
    Tani::yeni("S046", mesaj.into(), satir, 1, 1).onerili(oneri.into())
}

fn gorunum_dugumu_kur(tokenlar: Vec<Token>) -> Result<GorunumDugumu, Tani> {
    let satir = tokenlar.first().map(|t| t.satir).unwrap_or(1);
    let Some(ilk) = tokenlar.first() else {
        return Err(gorunum_tanisi(
            "Görünümde boş satır.",
            satir,
            "Bir öğe ya da metin yaz.",
        ));
    };
    match &ilk.tur {
        TokenTur::Metin(_) if tokenlar.len() == 1 => {
            return Ok(GorunumDugumu::Metin {
                ifade: tekil_ifade(tokenlar[0].clone())?,
                satir,
            });
        }
        TokenTur::Kelime(k) if (k == "ham" || k == "metin") && tokenlar.len() == 2 => {
            let ifade = tekil_ifade(tokenlar[1].clone())?;
            return Ok(if k == "ham" {
                GorunumDugumu::Ham { ifade, satir }
            } else {
                GorunumDugumu::Metin { ifade, satir }
            });
        }
        TokenTur::Kelime(k) if k == "ham" || k == "metin" => {
            return Err(gorunum_tanisi(
                "`ham` ve `metin` tek bir değer alır.",
                satir,
                "`metin baslik` ya da `ham parca` biçiminde yaz.",
            ));
        }
        _ => {}
    }
    let TokenTur::Kelime(etiket) = &ilk.tur else {
        return Err(gorunum_tanisi(
            "Görünüm satırı bir öğe adıyla, sabit metinle, `metin` ya da `ham` ile başlar.",
            satir,
            "Örnek: `input type \"email\" name \"eposta\"`.",
        ));
    };
    if etiket.is_empty()
        || !etiket.chars().all(|c| c.is_ascii_alphanumeric())
        || !etiket
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
    {
        return Err(gorunum_tanisi(
            "Öğe adı yalnız ASCII harf ve rakam içerir.",
            satir,
            "HTML etiket adı kullan: div, form, input, span…",
        ));
    }
    let kalan = &tokenlar[1..];
    if !kalan.len().is_multiple_of(2) {
        return Err(gorunum_tanisi(
            "Öznitelikler ad-değer çiftleri hâlinde yazılır.",
            satir,
            "Metin içeriği ayrı satıra al: `\"E-posta\"` ya da `metin baslik`.",
        ));
    }
    let mut oznitelikler = Vec::with_capacity(kalan.len() / 2);
    for cift in kalan.chunks(2) {
        let ad = match &cift[0].tur {
            TokenTur::Kelime(a) | TokenTur::Metin(a) => a.clone(),
            _ => {
                return Err(gorunum_tanisi(
                    "Öznitelik adı kelime ya da sabit metin olmalı.",
                    satir,
                    "Tire içeren adları tırnakla yaz: `\"data-modal\" doğru`.",
                ))
            }
        };
        if ad.is_empty()
            || !ad
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':' || c == '_')
        {
            return Err(gorunum_tanisi(
                "Öznitelik adı yalnız harf, rakam, tire, alt çizgi ve iki nokta içerir.",
                satir,
                "Örnek: `class`, `data-modal`, `aria-label`.",
            ));
        }
        oznitelikler.push((ad, tekil_ifade(cift[1].clone())?));
    }
    Ok(GorunumDugumu::Oge {
        etiket: etiket.to_ascii_lowercase(),
        oznitelikler,
        cocuklar: Vec::new(),
        satir,
    })
}
