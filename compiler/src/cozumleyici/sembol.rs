use super::*;

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
pub(super) fn kapsam_baslat(ortam: &HashMap<String, Tur>) -> std::collections::HashSet<String> {
    ortam.keys().cloned().collect()
}

pub(super) fn kapsam_bitir(ortam: &mut HashMap<String, Tur>, kapsam: &std::collections::HashSet<String>) {
    ortam.retain(|ad, _| kapsam.contains(ad));
}

/// Hal eki almış tanımlayıcıyı kapsamdaki tanımlı adlara karşı çözer.
pub fn ad_cozumle(
    ham: &str,
    ortam: &HashMap<String, Tur>,
    satir: usize,
    sutun: usize,
    uzunluk: usize,
) -> Result<String, Tani> {
    // Doğrudan eşleşme her zaman kazanır.
    if ortam.contains_key(ham) {
        return Ok(ham.to_string());
    }

    let adaylar = crate::morfoloji::kok_adaylari(ham);
    let eslesenler: Vec<String> = adaylar
        .into_iter()
        .filter(|aday| ortam.contains_key(aday))
        .collect();

    match eslesenler.len() {
        1 => Ok(eslesenler.into_iter().next().unwrap()),
        0 => {
            let tanimli: Vec<&str> = ortam.keys().map(|s| s.as_str()).collect();
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
