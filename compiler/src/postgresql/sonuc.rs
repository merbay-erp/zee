//! K-206: sonuç sahipliğini sınırla; transport hatasını ayrı tut.
use super::{hata, postgres_hatasi, verili_hata};
use crate::veritabani_modeli::{VeritabaniHatasi, VeritabaniOkumaSonucu, VeritabaniSatiri};
use postgres::{
    fallible_iterator::FallibleIterator,
    types::{ToSql, Type},
    Client, Row,
};

#[derive(Default)]
struct Toplayici {
    satir: usize,
    toplam: usize,
    sonuc: Vec<VeritabaniSatiri>,
    ilk_hata: Option<VeritabaniHatasi>,
}

impl Toplayici {
    fn kaydet(
        &mut self,
        donustur: impl FnOnce(&mut usize) -> Result<VeritabaniSatiri, VeritabaniHatasi>,
    ) {
        self.satir = self.satir.saturating_add(1);
        let sinir = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .veritabani()
            .satir();
        // Eski Vec algoritmasında toplam satır sınırı bütün hücre hatalarından önceydi.
        if self.satir == sinir.saturating_add(1) {
            self.ilk_hata = Some(hata("PostgreSQL sonucu 10.000 satır sınırını aşıyor"));
            self.sonuc.clear();
        } else if self.ilk_hata.is_none() {
            match donustur(&mut self.toplam) {
                Ok(alanlar) => self.sonuc.push(alanlar),
                Err(h) => {
                    self.ilk_hata = Some(h);
                    self.sonuc.clear();
                }
            }
        }
    }
    fn bitir(self) -> VeritabaniOkumaSonucu {
        match self.ilk_hata {
            Some(h) => Err(h),
            None => Ok(self.sonuc),
        }
    }
}

pub(super) fn oku(
    istemci: &mut Client,
    sorgu: &str,
    baglar: &[(&(dyn ToSql + Sync), Type)],
) -> Result<VeritabaniOkumaSonucu, postgres::Error> {
    let mut akim = istemci.query_typed_raw(sorgu, baglar.iter().map(|(v, t)| (*v, t.clone())))?;
    let mut toplayici = Toplayici::default();
    // Limit/hücre hatasından sonra da tüket: sunucu hatası önceliği ve bağlantı sağlığı korunur.
    // Tek wire satırının sürücü tahsisi bu uygulama sonucu sınırına dahil değildir.
    while let Some(satir) = akim.next()? {
        toplayici.kaydet(|toplam| alanlar(&satir, toplam));
    }
    Ok(toplayici.bitir())
}

fn alanlar(satir: &Row, toplam: &mut usize) -> Result<VeritabaniSatiri, VeritabaniHatasi> {
    let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.veritabani();
    if satir.len() > sinirlar.sutun() {
        return Err(hata("PostgreSQL sonucu 100 sütun sınırını aşıyor"));
    }
    let mut sonuc = Vec::with_capacity(satir.len());
    for (sira, sutun) in satir.columns().iter().enumerate() {
        if !matches!(
            *sutun.type_(),
            Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME
        ) {
            return Err(verili_hata(
                format!(
                    "\"{}\" sütunu Metin değil; sorguda açık ::text dönüşümü kullan",
                    sutun.name()
                ),
                vec![("sutun".into(), sutun.name().into())],
            ));
        }
        let deger: Option<&str> = satir
            .try_get(sira)
            .map_err(|h| postgres_hatasi("PostgreSQL sonuç sütunu okunamadı", h))?;
        let Some(deger) = deger else {
            return Err(verili_hata(
                format!(
                    "\"{}\" sütunu NULL; sorguda COALESCE ile açık metin değeri seç",
                    sutun.name()
                ),
                vec![("sutun".into(), sutun.name().into())],
            ));
        };
        *toplam = toplam.saturating_add(sutun.name().len() + deger.len());
        if *toplam > sinirlar.sonuc_bayti() {
            return Err(hata("PostgreSQL sonucu 16 MiB sınırını aşıyor"));
        }
        sonuc.push((sutun.name().to_string(), deger.to_string()));
    }
    Ok(sonuc)
}

#[cfg(test)]
mod testler;
