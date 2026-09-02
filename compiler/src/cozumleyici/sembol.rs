use super::*;

#[derive(Debug, Clone, Copy)]
struct SembolKaydi {
    kimlik: SymbolId,
    tur: Tur,
}

/// Checker'ın ad, kimlik ve türü tek kayıtta tuttuğu kapsam tablosu.
/// `HashMap<String, Tur>` yalnız geriye uyumlu public `ad_cozumle` yüzeyinde
/// kalır; iç geçişler semantic identity taşır.
pub(super) struct SembolTablosu {
    kapsam: usize,
    siradaki: usize,
    kayitlar: HashMap<String, SembolKaydi>,
}

impl SembolTablosu {
    pub(super) fn yeni(kapsam: usize) -> Self {
        Self {
            kapsam,
            siradaki: 0,
            kayitlar: HashMap::new(),
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
        self.siradaki += 1;
        self.kayitlar.insert(ad, SembolKaydi { kimlik, tur });
        kimlik
    }

    pub(super) fn retain(&mut self, mut tut: impl FnMut(&str, &Tur) -> bool) {
        self.kayitlar.retain(|ad, kayit| tut(ad, &kayit.tur));
    }
}

/// Ham alan yazımını ("adı", "yaşı") yapı tanımındaki yalın ada çözer.
pub(super) fn alan_cozumle(yapi: &Yapi, ham: &str, satir: usize) -> Result<String, Tani> {
    if yapi.alanlar.iter().any(|(a, _)| a == ham) {
        return Ok(ham.to_string());
    }
    let adaylar = crate::morfoloji::kok_adaylari(ham);
    let eslesenler: Vec<&String> = yapi
        .alanlar
        .iter()
        .map(|(a, _)| a)
        .filter(|a| adaylar.iter().any(|aday| aday == *a))
        .collect();
    match eslesenler.len() {
        1 => Ok(eslesenler[0].clone()),
        _ => Err(Tani::yeni(
            "T028",
            format!(
                "\"{}\" yapısında \"{}\" diye bir alan yok. Alanlar: {}.",
                yapi.ad,
                ham,
                yapi.alanlar
                    .iter()
                    .map(|(a, _)| a.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            satir,
            1,
            1,
        )),
    }
}

/// Blok kapsamı (RFC-0004 kararı, v0.2): gövdeye girerken ad kümesi alınır,
/// çıkarken gövdede DOĞAN adlar düşer. Dıştaki ada atama kalıcıdır; içerde
/// aynı adla yeniden tanım diye bir şey yoktur (gölgeleme yapısal olarak yok).
pub(super) fn kapsam_baslat(ortam: &SembolTablosu) -> std::collections::HashSet<String> {
    ortam.keys().cloned().collect()
}

pub(super) fn kapsam_bitir(ortam: &mut SembolTablosu, kapsam: &std::collections::HashSet<String>) {
    ortam.retain(|ad, _| kapsam.contains(ad));
}

fn adi_coz(
    ham: &str,
    mut tanimli: Vec<String>,
    satir: usize,
    sutun: usize,
    uzunluk: usize,
) -> Result<String, Tani> {
    tanimli.sort();
    if tanimli.iter().any(|ad| ad == ham) {
        return Ok(ham.to_string());
    }

    let adaylar = crate::morfoloji::kok_adaylari(ham);
    let eslesenler = tanimli
        .iter()
        .filter(|ad| adaylar.iter().any(|aday| aday == *ad))
        .cloned()
        .collect::<Vec<_>>();

    match eslesenler.len() {
        1 => eslesenler
            .into_iter()
            .next()
            .ok_or_else(|| ic_tutarlilik_hatasi("Tek morfoloji eşleşmesi kayboldu", satir)),
        0 => {
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
    let ad = adi_coz(ham, ortam.keys().cloned().collect(), satir, sutun, uzunluk)?;
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
    adi_coz(ham, ortam.keys().cloned().collect(), satir, sutun, uzunluk)
}
