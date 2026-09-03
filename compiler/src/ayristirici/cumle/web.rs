use super::*;

fn yanit_bicimi_tanisi(satir: usize) -> Tani {
    Tani::yeni(
        "S037",
        "Yanıt `<değer> yanıtını gönder` ya da `<değer> yanıtını <100..599> durumuyla gönder` biçimindedir."
            .into(),
        satir,
        1,
        1,
    )
    .onerili("Örnek: `\"hazır değil\" yanıtını 503 durumuyla gönder`.".into())
}

impl Ayristirici {
    pub(super) fn yanit_ayristir(
        &self,
        mut tokenlar: Vec<Token>,
        satir: usize,
    ) -> Result<Cumle, Tani> {
        tokenlar.pop(); // gönder
        let durum = if matches!(tokenlar.last(), Some(son) if kelime_mi(son, "durumuyla")) {
            tokenlar.pop();
            let Some(kod_tokeni) = tokenlar.pop() else {
                return Err(yanit_bicimi_tanisi(satir));
            };
            let TokenTur::TamSayi(kod) = kod_tokeni.tur else {
                return Err(yanit_bicimi_tanisi(satir));
            };
            if !(100..=599).contains(&kod) {
                return Err(Tani::yeni(
                    "S037",
                    "HTTP yanıt durum kodu 100..599 arasında sabit TamSayı olmalı.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnek: `\"hazır değil\" yanıtını 503 durumuyla gönder`.".into()));
            }
            Some(kod as u16)
        } else {
            None
        };
        if matches!(tokenlar.last(), Some(son) if kelime_mi(son, "yanıtını")) {
            tokenlar.pop();
            let deger = ile_ifadesi(&tokenlar, satir, &self.islem_adlari)?;
            Ok(Cumle::YanitGonder {
                deger,
                durum,
                satir,
            })
        } else {
            Err(yanit_bicimi_tanisi(satir))
        }
    }
}
