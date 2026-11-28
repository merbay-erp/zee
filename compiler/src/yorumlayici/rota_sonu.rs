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

/// RFC-0047 C (ADR-182 dilim D): ETag yanıt başlığı. Değer güçlü
/// entity-tag olmalıdır (`"` + etagc* + `"`, etagc = %x21 / %x23-7E, ≤ 128
/// bayt; `W/` yok); geçersiz değer başlığa girmez, istek C022 ile 500 olur.
pub(super) fn etag_yaz<C: super::GirdiCikti + ?Sized>(
    cikti: &mut C,
    etag: &str,
    satir: usize,
) -> Result<(), Tani> {
    let gecerli = etag.len() >= 2
        && etag.len() <= 128
        && etag.starts_with('"')
        && etag.ends_with('"')
        && etag.as_bytes()[1..etag.len() - 1]
            .iter()
            .all(|b| *b == 0x21 || (0x23..=0x7e).contains(b));
    let sonuc = if gecerli {
        cikti.etag_yaz(etag)
    } else {
        Err("ETag güçlü entity-tag (`\"...\"`, görünür ASCII, ≤ 128 bayt) olmalı".into())
    };
    sonuc.map_err(|hata| {
        Tani::yeni(
            "C022",
            format!("ETag başlığı reddedildi: {hata}."),
            satir,
            1,
            1,
        )
    })
}

/// Metin değerli web cümleleri: K-051/K-403 yönlendirme (rotayı bitirir) ve
/// RFC-0047 ETag başlığı.
pub(super) fn metin_cumlesi(
    cumle: &Cumle,
    cikti: &mut dyn super::GirdiCikti,
    metin: &str,
) -> Result<(), Tani> {
    match cumle {
        Cumle::YanitEtagi { satir, .. } => etag_yaz(cikti, metin, *satir),
        Cumle::Yonlendir { satir, .. } => {
            son_tarihi_denetle(cikti, *satir)?;
            cikti.yonlendir_gonder(metin).map_err(|hata| {
                Tani::yeni(
                    "C022",
                    format!("Yönlendirme reddedildi: {}.", hata),
                    *satir,
                    1,
                    1,
                )
            })?;
            // K-403: yönlendirme rotayı bitirir (yetki koruması fail-closed).
            rota_yaniti_belirlendi(*satir)
        }
        _ => Ok(()),
    }
}
