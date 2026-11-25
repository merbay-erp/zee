use super::*;

#[derive(Debug, Clone, Copy)]
struct SembolKaydi {
    kimlik: SymbolId,
    tur: Tur,
}

/// Checker'ın ad, kimlik ve türü tek kayıtta tuttuğu kapsam tablosu.
/// `ad_cozumle` public yüzeyi HashMap, iç geçişler semantic identity taşır.
pub(super) struct SembolTablosu {
    kapsam: usize,
    siradaki: usize,
    kayitlar: HashMap<String, SembolKaydi>,
    yeni_adlar: Vec<(usize, String)>,
    k398: eski_yazim::EskiParametreYazimi,
}

impl SembolTablosu {
    pub(super) fn yeni(kapsam: usize) -> Self {
        Self {
            kapsam,
            siradaki: 0,
            kayitlar: HashMap::new(),
            yeni_adlar: Vec::new(),
            k398: Default::default(),
        }
    }

    pub(super) fn get(&self, ad: &str) -> Option<&Tur> {
        self.kayitlar.get(ad).map(|kayit| &kayit.tur)
    }

    pub(super) fn kimlik(&self, ad: &str) -> Option<SymbolId> {
        self.kayitlar.get(ad).map(|kayit| kayit.kimlik)
    }

    pub(super) fn keys(&self) -> impl Iterator<Item = &String> {
        self.kayitlar.keys()
    }

    pub(super) fn insert(&mut self, ad: String, tur: Tur) -> SymbolId {
        if let Some(kayit) = self.kayitlar.get_mut(&ad) {
            kayit.tur = tur;
            return kayit.kimlik;
        }
        let kimlik = SymbolId::yeni(self.kapsam, self.siradaki);
        self.yeni_adlar.push((self.siradaki, ad.clone()));
        self.siradaki += 1;
        self.kayitlar.insert(ad, SembolKaydi { kimlik, tur });
        kimlik
    }
}

/// Blok kapsamı (RFC-0004 kararı, v0.2): gövdeye girerken sıra sınırı alınır,
/// çıkarken gövdede DOĞAN adlar düşer. Dıştaki ada atama kalıcıdır; içerde
/// aynı adla yeniden tanım diye bir şey yoktur (gölgeleme yapısal olarak yok).
pub(super) fn kapsam_baslat(ortam: &SembolTablosu) -> usize {
    ortam.siradaki
}

pub(super) fn kapsam_bitir(ortam: &mut SembolTablosu, kapsam: &usize) {
    let bas = ortam.yeni_adlar.partition_point(|(sira, _)| sira < kapsam);
    for (_, ad) in ortam.yeni_adlar.drain(bas..) {
        ortam.kayitlar.remove(&ad);
    }
}

fn adi_coz(
    ham: &str,
    tanimli_mi: impl Fn(&str) -> bool,
    tanimli_adlar: impl FnOnce() -> Vec<String>,
    satir: usize,
    sutun: usize,
    uzunluk: usize,
) -> Result<String, Tani> {
    if tanimli_mi(ham) {
        return Ok(ham.to_string());
    }

    let mut eslesenler = crate::morfoloji::kok_adaylari(ham)
        .into_iter()
        .filter(|aday| tanimli_mi(aday))
        .collect::<Vec<_>>();
    // Tanı sırası HashMap yerleşimine veya aday üretim sırasına bağlı değildir.
    eslesenler.sort();
    eslesenler.dedup();

    match eslesenler.len() {
        1 => eslesenler
            .into_iter()
            .next()
            .ok_or_else(|| ic_tutarlilik_hatasi("Tek morfoloji eşleşmesi kayboldu", satir)),
        0 => {
            let mut tanimli = tanimli_adlar();
            tanimli.sort();
            let oneri = if tanimli.is_empty() {
                "Bir değeri kullanmadan önce \"<ad> <değer> olsun\" ile tanımla.".to_string()
            } else {
                format!(
                    "Bu ad tanımlı değil. Tanımlı adlar: {}. Önce \"<ad> <değer> olsun\" ile tanımla.",
                    tanimli.join(", ")
                )
            };
            Err(Tani::yeni(
                "A001",
                format!("\"{}\" adı bu kapsamda tanımlı değil.", ham),
                satir,
                sutun,
                uzunluk,
            )
            .onerili(oneri))
        }
        _ => Err(Tani::yeni(
            "A002",
            format!(
                "\"{}\" birden çok ada çözülebiliyor: {}. Hangisini kastettiğin belirsiz.",
                ham,
                eslesenler.join(", ")
            ),
            satir,
            sutun,
            uzunluk,
        )
        .onerili("Adlardan birini değiştir; belirsizlik dilde hata sayılır.".into())),
    }
}

pub(super) fn sembol_cozumle(
    ham: &str,
    ortam: &SembolTablosu,
    satir: usize,
    sutun: usize,
    uzunluk: usize,
) -> Result<(String, SymbolId), Tani> {
    let ad = adi_coz(
        ham,
        |ad| ortam.kimlik(ad).is_some(),
        || ortam.keys().cloned().collect(),
        satir,
        sutun,
        uzunluk,
    )
    .or_else(|hata| ortam.eski_yazimla_coz(ham, hata))?;
    let kimlik = ortam
        .kimlik(&ad)
        .ok_or_else(|| ic_tutarlilik_hatasi("Çözülmüş adın sembol kaydı bulunamadı", satir))?;
    Ok((ad, kimlik))
}

/// Hal eki almış tanımlayıcıyı kapsamdaki tanımlı adlara karşı çözer.
pub fn ad_cozumle(
    ham: &str,
    ortam: &HashMap<String, Tur>,
    satir: usize,
    sutun: usize,
    uzunluk: usize,
) -> Result<String, Tani> {
    adi_coz(
        ham,
        |ad| ortam.contains_key(ad),
        || ortam.keys().cloned().collect(),
        satir,
        sutun,
        uzunluk,
    )
}

mod alan;
mod eski_yazim;
pub(super) use self::alan::alan_cozumle;

#[cfg(test)]
mod testler;
