//! Runtime değer ve metin allocation sınırları.

use super::*;

enum Ziyaret<'a> {
    Deger(&'a Deger),
    Hata(&'a HataDegeri),
}

pub(super) fn metin_sinirini_denetle(bayt: usize, satir: usize) -> Result<(), Tani> {
    let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.metin_bayti();
    if bayt <= azami {
        return Ok(());
    }
    Err(deger_kaynak_tanisi(
        satir,
        &format!(
            "Metin {} bayt; güvenli profil {} MiB sınırını aşıyor.",
            bayt,
            azami / 1024 / 1024
        ),
        "Metni üretmeden önce daha küçük parçalara böl.",
    ))
}

pub(super) fn metin_parcasi_ekle(
    hedef: &mut String,
    parca: &str,
    satir: usize,
) -> Result<(), Tani> {
    let yeni = hedef.len().checked_add(parca.len()).ok_or_else(|| {
        deger_kaynak_tanisi(
            satir,
            "Metin boyutu sayı sınırını aştı.",
            "Daha küçük metin üret.",
        )
    })?;
    metin_sinirini_denetle(yeni, satir)?;
    hedef.push_str(parca);
    Ok(())
}

pub(super) fn deger_sinirini_denetle(deger: &Deger, satir: usize) -> Result<(), Tani> {
    let bayt = deger_heap_bayti(deger, satir)?;
    let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.calisma_heap_bayti();
    if bayt <= azami {
        return Ok(());
    }
    Err(heap_hatasi(bayt, azami, satir))
}

pub(super) fn ortama_yazma_butcesini_tuket(
    ortam: &HashMap<String, Deger>,
    ad: &str,
    deger: &Deger,
    satir: usize,
) -> Result<(), Tani> {
    let mut bayt = deger_heap_bayti(deger, satir)?;
    if !ortam.contains_key(ad) {
        bayt = bayt
            .checked_add(ad.len())
            .and_then(|n| n.checked_add(std::mem::size_of::<(String, Deger)>() * 2))
            .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
    }
    heap_butcesini_tuket(bayt, satir)
}

pub(super) fn deger_tahsis_butcesini_tuket(deger: &Deger, satir: usize) -> Result<(), Tani> {
    heap_butcesini_tuket(deger_heap_bayti(deger, satir)?, satir)
}

pub(super) fn koleksiyon_yazma_butcesini_tuket(
    deger: &Deger,
    anahtar: Option<&str>,
    satir: usize,
) -> Result<(), Tani> {
    let mut bayt = deger_heap_bayti(deger, satir)?;
    bayt = bayt
        .checked_add(match anahtar {
            Some(anahtar) => anahtar.len() + std::mem::size_of::<(String, Deger)>() * 2,
            None => std::mem::size_of::<Deger>() * 2,
        })
        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
    heap_butcesini_tuket(bayt, satir)
}

pub(super) fn gorev_ortami_butcesini_tuket(
    ortam: &HashMap<String, Deger>,
    gorev_sayisi: usize,
    satir: usize,
) -> Result<(), Tani> {
    let tek = ortam_heap_bayti(ortam, satir)?;
    let toplam = tek
        .checked_mul(gorev_sayisi.saturating_add(1))
        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
    heap_butcesini_tuket(toplam, satir)
}

fn ortam_heap_bayti(ortam: &HashMap<String, Deger>, satir: usize) -> Result<usize, Tani> {
    let mut toplam = ortam
        .len()
        .checked_mul(std::mem::size_of::<(String, Deger)>() * 2)
        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
    for (ad, deger) in ortam {
        toplam = toplam
            .checked_add(ad.len())
            .and_then(|n| n.checked_add(deger_heap_bayti(deger, satir).ok()?))
            .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
    }
    Ok(toplam)
}

fn deger_heap_bayti(deger: &Deger, satir: usize) -> Result<usize, Tani> {
    let mut toplam = 0usize;
    let mut ziyaretler = vec![Ziyaret::Deger(deger)];
    while let Some(ziyaret) = ziyaretler.pop() {
        match ziyaret {
            Ziyaret::Deger(deger) => match deger {
                Deger::TamSayi(_)
                | Deger::Mantiksal(_)
                | Deger::Yok
                | Deger::Tarih { .. }
                | Deger::Saat { .. }
                | Deger::Sure { .. } => {}
                Deger::Ondalik(ondalik) => {
                    toplam = toplam
                        .checked_add(std::mem::size_of::<Ondalik>())
                        .and_then(|n| n.checked_add(ondalik.yaklasik_heap_bayti()))
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                }
                Deger::Metin(metin) => {
                    metin_sinirini_denetle(metin.len(), satir)?;
                    toplam = toplam
                        .checked_add(metin.capacity())
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                }
                Deger::AgYaniti { govde, .. } => {
                    metin_sinirini_denetle(govde.len(), satir)?;
                    toplam = toplam
                        .checked_add(govde.capacity())
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                }
                Deger::Liste(ogeler) => {
                    toplam = toplam
                        .checked_add(
                            ogeler
                                .capacity()
                                .saturating_mul(std::mem::size_of::<Deger>()),
                        )
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                    ziyaretler.extend(ogeler.iter().map(Ziyaret::Deger));
                }
                Deger::Sozluk(girdiler) | Deger::Yapi(girdiler) => {
                    toplam = toplam
                        .checked_add(girdiler.capacity().saturating_mul(std::mem::size_of::<(
                            String,
                            Deger,
                        )>(
                        )))
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                    for (anahtar, deger) in girdiler {
                        metin_sinirini_denetle(anahtar.len(), satir)?;
                        toplam = toplam
                            .checked_add(anahtar.capacity())
                            .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                        ziyaretler.push(Ziyaret::Deger(deger));
                    }
                }
                Deger::Sonuc { icerik, .. } => {
                    toplam = toplam
                        .checked_add(std::mem::size_of::<Deger>())
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                    ziyaretler.push(Ziyaret::Deger(icerik));
                }
                Deger::Hata(hata) => {
                    toplam = toplam
                        .checked_add(std::mem::size_of::<HataDegeri>())
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                    ziyaretler.push(Ziyaret::Hata(hata));
                }
            },
            Ziyaret::Hata(hata) => {
                for metin in [&hata.kod, &hata.mesaj] {
                    metin_sinirini_denetle(metin.len(), satir)?;
                    toplam = toplam
                        .checked_add(metin.capacity())
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                }
                toplam = toplam
                    .checked_add(
                        hata.veri
                            .capacity()
                            .saturating_mul(std::mem::size_of::<(String, Deger)>()),
                    )
                    .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                for (anahtar, deger) in &hata.veri {
                    toplam = toplam
                        .checked_add(anahtar.capacity())
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                    ziyaretler.push(Ziyaret::Deger(deger));
                }
                if let Some(neden) = hata.neden.as_deref() {
                    toplam = toplam
                        .checked_add(std::mem::size_of::<HataDegeri>())
                        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
                    ziyaretler.push(Ziyaret::Hata(neden));
                }
            }
        }
        if toplam > heap_siniri() {
            return Err(heap_hatasi(toplam, heap_siniri(), satir));
        }
    }
    Ok(toplam)
}

fn heap_butcesini_tuket(bayt: usize, satir: usize) -> Result<(), Tani> {
    CALISTIRMA_BUTCESI.with(|yuva| {
        let mut yuva = yuva.borrow_mut();
        let Some(butce) = yuva.as_mut() else {
            return Ok(());
        };
        let azami = heap_siniri();
        let kullanilan = azami.saturating_sub(butce.kalan_heap_bayti);
        butce.kalan_heap_bayti = butce
            .kalan_heap_bayti
            .checked_sub(bayt)
            .ok_or_else(|| heap_hatasi(kullanilan.saturating_add(bayt), azami, satir))?;
        Ok(())
    })
}

fn heap_siniri() -> usize {
    crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.calisma_heap_bayti()
}

fn heap_hatasi(bayt: usize, azami: usize, satir: usize) -> Tani {
    deger_kaynak_tanisi(
        satir,
        &format!(
            "Çalışma değeri yaklaşık {} bayt; güvenli profil {} MiB heap sınırını aşıyor.",
            bayt,
            azami / 1024 / 1024
        ),
        "Büyük metin ve koleksiyonları daha küçük çalışma adımlarına böl.",
    )
}

pub(super) fn deger_kaynak_tanisi(satir: usize, mesaj: &str, oneri: &str) -> Tani {
    Tani::yeni("C024", mesaj.into(), satir, 1, 1).onerili(oneri.into())
}
