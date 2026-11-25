//! K-398 eski parametre yazımı uyumluluğu (ADR-175).
//!
//! Parametre artık sözlük köküne (`kalıp`) bağlanır. K-398 öncesi programın
//! yazdığı yumuşamış gövde (`kalıb`, `metn`) ya da onu veren ekli biçim,
//! doğrudan ve morfolojik eşleşme yoksa aynı parametreye çözülür. Daha önce
//! çözülen hiçbir başvurunun anlamı değişmez; yalnız A001 olacak yazım kabul edilir.

use super::SembolTablosu;
use crate::morfoloji::{baglama_koku, kok_adaylari};
use crate::tani::Tani;

#[derive(Default)]
pub(super) struct EskiParametreYazimi {
    /// Eski yumuşamış yazımın çözülebileceği parametre adları.
    parametreler: Vec<String>,
    /// Gövdede sözlük köküyle (`kalıp … olsun`) yeniden atanmış parametreler;
    /// K-398 öncesi bu bir yerel tanımdı.
    kokle_atananlar: Vec<String>,
}

impl SembolTablosu {
    /// İşlem parametresini K-398 eski yazım hedefi olarak işaretler.
    pub(in crate::cozumleyici) fn parametre_isaretle(&mut self, ad: &str) {
        if !self.parametre_mi(ad) {
            self.k398.parametreler.push(ad.to_string());
        }
    }

    pub(in crate::cozumleyici) fn parametre_mi(&self, ad: &str) -> bool {
        self.k398.parametreler.iter().any(|kayitli| kayitli == ad)
    }

    /// `ad … olsun` hedefi tam yazımıyla bir parametreyse işaretlenir.
    pub(in crate::cozumleyici) fn parametre_kokle_atandi(&mut self, ad: &str) {
        if self.parametre_mi(ad) && !self.k398.kokle_atananlar.iter().any(|k| k == ad) {
            self.k398.kokle_atananlar.push(ad.to_string());
        }
    }

    /// `ad … olsun` hedefi: tam yazım tanımlı değilse ve eski parametre
    /// yazımıysa (`belirtec`) hedef parametreye (`belirteç`) çevrilir; atama
    /// yeni ad doğurmaz. Hedef tam yazımıyla tanımlıysa `true` döner.
    pub(in crate::cozumleyici) fn eski_yazimi_hedefe_cevir(&self, ad: &mut String) -> bool {
        if self.kimlik(ad).is_some() {
            return true;
        }
        if let Some(kok) = self.eski_parametre_yazimi(ad) {
            *ad = kok;
        }
        false
    }

    /// Parametreye farklı türde atama T002 önerisi (K-398 öncesi ayrı yereldi).
    pub(in crate::cozumleyici) fn parametre_tur_onerisi(
        &self,
        ad: &str,
        tur: &str,
    ) -> Option<String> {
        self.parametre_mi(ad).then(|| {
            format!(
                "\"{}\" bu işlemin parametresidir (K-398: ekli parametre adı sözlük köküne \
                 bağlanır, ör. `başlığı al` → `başlık`). Yerel değere başka bir ad ver ya da \
                 parametreye yine {} türünde bir değer ver.",
                ad, tur
            )
        })
    }

    /// Parametre sözlük köküne bağlandıysa, K-398 öncesi bağlanan yumuşamış
    /// gövde (`kalıb`, `metn`) o parametreyi verir.
    pub(in crate::cozumleyici) fn eski_parametre_yazimi(&self, govde: &str) -> Option<String> {
        let kok = baglama_koku(govde)?;
        (self.parametre_mi(&kok) && self.kimlik(&kok).is_some()).then_some(kok)
    }

    /// A001 olacak başvuruyu eski parametre yazımıyla yeniden dener. Eşleşme
    /// yoksa özgün tanı döner; birden çok eşleşme A002'dir. Parametre gövdede
    /// sözlük köküyle yeniden atandıysa eski yazım belirsizdir: K-398 öncesi
    /// `kalıp` ayrı bir yerel, `kalıb` parametreydi. Sessiz anlam değişimi
    /// yerine A002 verilir.
    pub(super) fn eski_yazimla_coz(&self, ham: &str, hata: Tani) -> Result<String, Tani> {
        if hata.kod != "A001" {
            return Err(hata);
        }
        let mut govdeler = vec![ham.to_string()];
        govdeler.extend(kok_adaylari(ham));
        let mut eslesenler: Vec<String> = govdeler
            .iter()
            .filter_map(|govde| self.eski_parametre_yazimi(govde))
            .collect();
        eslesenler.sort();
        eslesenler.dedup();
        let (satir, sutun, uzunluk) = (hata.satir, hata.sutun, hata.uzunluk);
        match eslesenler.as_slice() {
            [] => Err(hata),
            [ad] if self.k398.kokle_atananlar.contains(ad) => Err(Tani::yeni(
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
            ))),
            [ad] => Ok(ad.clone()),
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
}
