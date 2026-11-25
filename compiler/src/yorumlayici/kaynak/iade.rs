//! K-393 (ADR-173): C024 zarfına düşen değerin payını iade eden yollar.
//! Güvenlik değişmezi: saklanan her değerde ölçü ≤ tüketim; iade ölçü kadar
//! yapılır ve kalan zarf başlangıç zarfını aşamaz.

use super::*;

/// K-393: değeri ortama yazar; zarftan yeni değer düşülür, üzerine yazılan
/// eski değerin ölçüsü iade edilir. Önce tüketim, sonra iade: geçiş anındaki
/// iki kopya da zarfa sığmalıdır.
pub(in crate::yorumlayici) fn ortama_yaz(
    ortam: &mut HashMap<String, Deger>,
    ad: &str,
    deger: Deger,
    satir: usize,
) -> Result<(), Tani> {
    ortama_yazma_butcesini_tuket(ortam, ad, &deger, satir)?;
    if let Some(eski) = ortam.insert(ad.to_string(), deger) {
        heap_butcesini_iade_et(deger_iade_bayti(&eski));
    }
    Ok(())
}

/// Tüketilen bayt sayısını döndürür; çağıran kopyayı düşürünce aynı sayıyı
/// iade eder (K-393).
pub(in crate::yorumlayici) fn deger_tahsis_butcesini_tuket(
    deger: &Deger,
    satir: usize,
) -> Result<usize, Tani> {
    let bayt = deger_heap_bayti(deger, satir)?;
    heap_butcesini_tuket(bayt, satir)?;
    Ok(bayt)
}

/// K-393: saklı koleksiyona yerinde ekleme. Zarftan öğenin ölçüsü ile
/// tamponun gerçek kapasite artışı düşülür; böylece ölçü hiçbir zaman
/// tüketilenden büyük olmaz ve sonraki iade güvenlidir. Bütçe tükenirse
/// kapasite geri bırakılır ve koleksiyon değişmez.
pub(in crate::yorumlayici) fn kapasiteli_ekleme_butcesini_tuket<T>(
    tampon: &mut Vec<T>,
    oge_bayti: usize,
    satir: usize,
) -> Result<(), Tani> {
    let eski = tampon.capacity();
    tampon.reserve(1);
    let artis = (tampon.capacity() - eski).saturating_mul(std::mem::size_of::<T>());
    let toplam = oge_bayti
        .checked_add(artis)
        .ok_or_else(|| heap_hatasi(usize::MAX, heap_siniri(), satir))?;
    if let Err(tani) = heap_butcesini_tuket(toplam, satir) {
        tampon.shrink_to(eski);
        return Err(tani);
    }
    Ok(())
}

/// Saklanan değerin zarf ölçüsü (tüketimle aynı ölçü).
pub(in crate::yorumlayici) fn deger_olcusu(deger: &Deger, satir: usize) -> Result<usize, Tani> {
    deger_heap_bayti(deger, satir)
}

/// Düşen saklı değerin iade edilecek ölçüsü. Saklanan her değer tüketimden
/// geçtiği için ölçü sınır içindedir; ölçülemezse iade edilmez (güvenli yön).
pub(in crate::yorumlayici) fn deger_iade_bayti(deger: &Deger) -> usize {
    deger_heap_bayti(deger, 1).unwrap_or(0)
}

/// Ortamdan düşen ad+değer girdisinin iadesi: `ortama_yazma_butcesini_tuket`
/// yeni ad için yuva payı ve ad uzunluğu tüketir; aynısı geri verilir.
pub(in crate::yorumlayici) fn ortam_girdisi_iade_bayti(ad: &str, deger: &Deger) -> usize {
    ad.len()
        .saturating_add(std::mem::size_of::<(String, Deger)>() * 2)
        .saturating_add(deger_iade_bayti(deger))
}

/// K-393: zarfa iade. Kalan hiçbir zaman başlangıç zarfını aşamaz; böylece
/// başka bütçeden tüketilmiş bir değerin iadesi bile sınırı genişletmez.
pub(in crate::yorumlayici) fn heap_butcesini_iade_et(bayt: usize) {
    if bayt == 0 {
        return;
    }
    CALISTIRMA_BUTCESI.with(|yuva| {
        if let Some(butce) = yuva.borrow_mut().as_mut() {
            butce.kalan_heap_bayti = butce
                .kalan_heap_bayti
                .saturating_add(bayt)
                .min(heap_siniri());
        }
    });
}

/// K-393: saklı koleksiyondaki bir öğeyi yerinde değiştirir; yeni öğenin
/// ölçüsü tüketilir, çıkan öğeninki iade edilir. Bütçe tükenirse öğe değişmez.
pub(in crate::yorumlayici) fn deger_degistir_butceli(
    yer: &mut Deger,
    yeni: Deger,
    satir: usize,
) -> Result<(), Tani> {
    heap_butcesini_tuket(deger_heap_bayti(&yeni, satir)?, satir)?;
    let eski = std::mem::replace(yer, yeni);
    heap_butcesini_iade_et(deger_iade_bayti(&eski));
    Ok(())
}

/// K-393: işlem çerçevesi ya da blok kapsamı düşünce bütün girdilerin iadesi.
pub(in crate::yorumlayici) fn ortam_iade_bayti(ortam: &HashMap<String, Deger>) -> usize {
    ortam.iter().fold(0usize, |toplam, (ad, deger)| {
        toplam.saturating_add(ortam_girdisi_iade_bayti(ad, deger))
    })
}

/// K-393: listeden çıkan öğenin ölçüsü iade edilir; tampon kapasitesi aynı
/// kaldığından ölçü ≤ tüketim değişmezi korunur.
pub(in crate::yorumlayici) fn listeden_cikar(ogeler: &mut Vec<Deger>, yer: usize) {
    let cikan = ogeler.remove(yer);
    heap_butcesini_iade_et(deger_iade_bayti(&cikan));
}

/// K-393: sözlükten silinen girdilerin anahtar kapasitesi ve değer ölçüsü
/// iade edilir.
pub(in crate::yorumlayici) fn sozlukten_sil(girdiler: &mut Vec<(String, Deger)>, anahtar: &str) {
    let mut iade = 0usize;
    girdiler.retain(|(a, d)| {
        let kalir = a != anahtar;
        if !kalir {
            iade = iade
                .saturating_add(a.capacity())
                .saturating_add(deger_iade_bayti(d));
        }
        kalir
    });
    heap_butcesini_iade_et(iade);
}

/// K-393: `ekle` — öğenin ölçüsü ve gerçek kapasite artışı tüketilir.
pub(in crate::yorumlayici) fn listeye_ekle(
    ogeler: &mut Vec<Deger>,
    deger: Deger,
    satir: usize,
) -> Result<(), Tani> {
    kapasiteli_ekleme_butcesini_tuket(ogeler, deger_olcusu(&deger, satir)?, satir)?;
    ogeler.push(deger);
    Ok(())
}
