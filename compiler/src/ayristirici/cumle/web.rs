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
        // K-384: `… gönder` ile biten işlem adı cümle olarak çağrılamaz; S037'ye
        // ad seçimi ipucu eklenir (program zaten S037 ile reddedilirdi).
        let cakisan = (1..=tokenlar.len()).find_map(|k| {
            let kelimeler: Option<Vec<&str>> = tokenlar[tokenlar.len() - k..]
                .iter()
                .map(|t| match &t.tur {
                    TokenTur::Kelime(k) => Some(k.as_str()),
                    _ => None,
                })
                .collect();
            let ad = format!("{} gönder", kelimeler?.join(" "));
            self.islem_adlari.contains(&ad).then_some(ad)
        });
        if let (Some(ad), false) = (
            cakisan,
            matches!(tokenlar.last(), Some(son) if kelime_mi(son, "yanıtını")),
        ) {
            if let Some(ipucu) = crate::tani::fiil_cakismasi_ipucu(&ad) {
                return Err(yanit_bicimi_tanisi(satir).onerili(ipucu));
            }
        }
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
