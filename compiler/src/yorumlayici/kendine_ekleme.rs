//! `x x ile y olsun` kendine metin eklemesinin yerinde büyümesi (K-323).

use super::*;

/// Hedef zaten metinse ve ifade tam olarak hedefle başlayan `Birlestir` ise
/// sol kopya kurulmaz: sağ parçalar `Birlestir` sırası ve 16 MiB tanısıyla
/// değerlendirilir, hedef yerinde büyür. İfade ortamı değiştiremediğinden
/// hedefin önce okunması anlamı korur. Kalıp tutmazsa `false` döner ve
/// çağıran olağan atamayı yapar.
pub(super) async fn kendine_metin_ekle(
    ad: &str,
    deger: &Ifade,
    ortam: &mut HashMap<String, Deger>,
    program: CalistirmaProgrami<'_>,
    cikti: &mut dyn GirdiCikti,
    derinlik: usize,
    satir: usize,
) -> Result<bool, Tani> {
    let Ifade::Birlestir(parcalar) = deger.turu() else {
        return Ok(false);
    };
    let Some((ilk, kalan)) = parcalar.split_first() else {
        return Ok(false);
    };
    let Ifade::Degisken { cozulmus, .. } = ilk.turu() else {
        return Ok(false);
    };
    if program.sembol_adi(ilk, cozulmus.as_deref()).as_deref() != Some(ad) {
        return Ok(false);
    }
    let Some(Deger::Metin(hedef)) = ortam.get(ad) else {
        return Ok(false);
    };
    let mevcut = hedef.len();
    let mut ek = String::new();
    for parca in kalan {
        let parca = metne_sinirli(
            &degerlendir_async(parca, ortam, program, cikti, derinlik, satir).await?,
            satir,
        )?;
        metin_sinirini_denetle(boyut_topla(mevcut + ek.len(), parca.len(), satir)?, satir)?;
        ek.push_str(&parca);
    }
    match ortam.get_mut(ad) {
        Some(Deger::Metin(hedef)) => metni_yerinde_buyut(hedef, &ek, satir)?,
        _ => return Err(ic_hata(satir)),
    }
    Ok(true)
}

/// Zarftan yalnız yeni kapasite düşülür: eski tampon yeniden tahsiste
/// bırakıldığından toplam sayım canlı metnin kapasitesine eşittir ve döngü
/// birikimi O(n) kalır. Bütçe tükenirse hedef değişmeden C024 döner.
fn metni_yerinde_buyut(hedef: &mut String, ek: &str, satir: usize) -> Result<(), Tani> {
    let gereken = boyut_topla(hedef.len(), ek.len(), satir)?;
    metin_sinirini_denetle(gereken, satir)?;
    let eski = hedef.capacity();
    if gereken > eski {
        let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.metin_bayti();
        let yeni = eski.saturating_mul(2).min(azami).max(gereken);
        heap_butcesini_tuket(yeni - eski, satir)?;
        hedef.reserve_exact(yeni - hedef.len());
    }
    hedef.push_str(ek);
    Ok(())
}

fn boyut_topla(sol: usize, sag: usize, satir: usize) -> Result<usize, Tani> {
    sol.checked_add(sag).ok_or_else(|| {
        deger_kaynak_tanisi(
            satir,
            "Metin boyutu sayı sınırını aştı.",
            "Daha küçük metin üret.",
        )
    })
}
