//! K-403: rota gövdesini yanıt/yönlendirmede bitiren iç akış nöbetçisi; K-406 yanıt cümlesi dağıtımı.

use super::*;

/// K-403: rota yanıtının belirlendiğini taşıyan iç akış nöbetçisi. Rota
/// gövdesindeki `yanıtını gönder`/`yönlendir` başarıyla yanıt taslağını
/// belirleyince gövde (çağrılan işlemler dahil) burada sonlanır; nöbetçi
/// `web_istegini_calistir` dışına sızmaz, kullanıcıya görünmez.
const ROTA_SONU: &str = "Ç004";

thread_local! {
    static ROTA_YURUTULUYOR: Cell<bool> = const { Cell::new(false) };
}

/// Rota gövdesi koşarken yanıt cümlesinin rotayı bitirmesini sağlar. Rota
/// dışında (üst düzey, test, CLI) yanıt cümlesi akışı sürdürür.
pub(super) fn rota_yaniti_belirlendi(satir: usize) -> Result<(), Tani> {
    if ROTA_YURUTULUYOR.with(Cell::get) {
        return Err(Tani::yeni(
            "Ç004",
            "rota yanıtı belirlendi".into(),
            satir,
            1,
            1,
        ));
    }
    Ok(())
}

/// Rota gövdesini nöbetçi açıkken koşar; yanıtla biten gövde (Ç004) olağan
/// bitiştir ve taslak başarı yolundaki gibi commit edilir.
pub(super) fn kos(govde: impl FnOnce() -> Result<Akis, Tani>) -> Result<Akis, Tani> {
    let _nobetci = RotaNobetcisi::yeni();
    match govde() {
        Err(tani) if tani.kod == ROTA_SONU => Ok(Akis::Devam),
        sonuc => sonuc,
    }
}

struct RotaNobetcisi(bool);

impl RotaNobetcisi {
    fn yeni() -> Self {
        Self(ROTA_YURUTULUYOR.with(|r| r.replace(true)))
    }
}

impl Drop for RotaNobetcisi {
    fn drop(&mut self) {
        ROTA_YURUTULUYOR.with(|r| r.set(self.0));
    }
}

/// Yanıt cümlesinin üç biçimini adaptöre dağıtır: K-406 türlü metin (tür
/// ayrıştırıcıda kapalı kayda karşı denetlendi), sabit durumlu ya da düz 200.
pub(super) fn yanit_cumlesini_gonder<C: super::GirdiCikti + ?Sized>(
    cikti: &mut C,
    tur: Option<&str>,
    durum: Option<u16>,
    metin: &str,
) {
    match (tur, durum) {
        (Some(tur), None) => cikti.turlu_yanit_gonder(tur, metin),
        (Some(tur), Some(durum)) => cikti.turlu_durumlu_yanit_gonder(tur, durum, metin),
        (None, Some(durum)) => cikti.durum_yaniti_gonder(durum, metin),
        (None, None) => cikti.yanit_gonder(metin),
    }
}
