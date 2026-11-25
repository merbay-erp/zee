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
    /// K-398: eski yumuşamış yazımın (`kalıb`) çözülebileceği parametre adları.
    parametre_adlari: Vec<String>,
    /// K-398: gövdede sözlük köküyle (`kalıp … olsun`) yeniden atanmış
    /// parametreler; K-398 öncesi bu bir yerel tanımdı.
    kokle_atanan_parametreler: Vec<String>,
}

impl SembolTablosu {
    pub(super) fn yeni(kapsam: usize) -> Self {
        Self {
            kapsam,
            siradaki: 0,
            kayitlar: HashMap::new(),
            yeni_adlar: Vec::new(),
            parametre_adlari: Vec::new(),
            kokle_atanan_parametreler: Vec::new(),
        }
    }

    pub(super) fn get(&self, ad: &str) -> Option<&Tur> {
        self.kayitlar.get(ad).map(|kayit| &kayit.tur)
    }

    pub(super) fn kimlik(&self, ad: &str) -> Option<SymbolId> {
        self.kayitlar.get(ad).map(|kayit| kayit.kimlik)
    }

    /// İşlem parametresini K-398 eski yazım hedefi olarak işaretler.
    pub(super) fn parametre_isaretle(&mut self, ad: &str) {
        if !self.parametre_adlari.iter().any(|kayitli| kayitli == ad) {
            self.parametre_adlari.push(ad.to_string());
        }
    }

    pub(super) fn parametre_mi(&self, ad: &str) -> bool {
        self.parametre_adlari.iter().any(|kayitli| kayitli == ad)
    }

    /// `ad … olsun` hedefi tam yazımıyla bir parametreyse işaretlenir.
    pub(super) fn parametre_kokle_atandi(&mut self, ad: &str) {
        if self.parametre_mi(ad) && !self.kokle_atanan_parametreler.iter().any(|k| k == ad) {
            self.kokle_atanan_parametreler.push(ad.to_string());
        }
    }

    /// K-398 eski yazım: parametre sözlük köküne bağlandıysa, K-398 öncesi
    /// bağlanan yumuşamış gövde (`kalıb`, `metn`) o parametreyi verir.
    pub(super) fn eski_parametre_yazimi(&self, govde: &str) -> Option<String> {
        let kok = crate::morfoloji::baglama_koku(govde)?;
        (self.parametre_adlari.contains(&kok) && self.kimlik(&kok).is_some()).then_some(kok)
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
    eski_yazim: impl Fn(&str) -> Option<String>,
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

    // K-398 eski yazım: parametre artık sözlük köküne (`kalıp`) bağlanır. K-398
    // öncesi programın yazdığı yumuşamış gövde (`kalıb`, `metn`) ya da onu veren
    // ekli biçim, doğrudan eşleşme yoksa aynı parametreye çözülür. Daha önce
    // çözülen hiçbir başvurunun anlamı değişmez; yalnız A001 olacak yazım kabul edilir.
    if eslesenler.is_empty() {
        let mut govdeler = vec![ham.to_string()];
        govdeler.extend(crate::morfoloji::kok_adaylari(ham));
        eslesenler = govdeler
            .iter()
            .filter_map(|govde| eski_yazim(govde))
            .collect();
        eslesenler.sort();
        eslesenler.dedup();
    }

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
        |govde| ortam.eski_parametre_yazimi(govde),
        || ortam.keys().cloned().collect(),
        satir,
        sutun,
        uzunluk,
    )?;
    // K-398: eski yazım (`kalıb`) yalnız eski yazım yoluyla çözüldüyse ve
    // parametre gövdede sözlük köküyle (`kalıp … olsun`) yeniden atandıysa
    // belirsizdir: K-398 öncesi `kalıp` ayrı bir yerel, `kalıb` parametreydi.
    // Sessiz anlam değişimi yerine A002 verilir.
    if ad != ham
        && ortam.kokle_atanan_parametreler.contains(&ad)
        && !crate::morfoloji::kok_adaylari(ham).contains(&ad)
    {
        return Err(Tani::yeni(
            "A002",
            format!(
                "\"{}\" eski parametre yazımıdır; \"{}\" parametresi gövdede yeniden atandığı için hangi değeri kastettiğin belirsiz.",
                ham, ad
            ),
            satir,
            sutun,
            uzunluk,
        )
        .onerili(format!(
            "K-398: parametre artık \"{}\" adıyla bağlanır. Parametrenin kendisi için \"{}\" yaz; ayrı bir yerel değer gerekiyorsa ona başka ad ver.",
            ad, ad
        )));
    }
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
        |_| None,
        || ortam.keys().cloned().collect(),
        satir,
        sutun,
        uzunluk,
    )
}

#[cfg(test)]
mod testler;
