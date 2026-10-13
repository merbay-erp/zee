//! K-207: yeni lookup yolu eski clone/sort kararlarıyla aynı gözlemi üretir.
use dil::cozumleyici::{ad_cozumle, Tur};
use dil::morfoloji::kok_adaylari;
use dil::tani::Tani;
use std::collections::HashMap;

// 29be85b tabanındaki karar algoritması uyumluluk oracle'ıdır.
fn onceki(ham: &str, ortam: &HashMap<String, Tur>) -> Result<String, Tani> {
    let mut tanimli = ortam.keys().cloned().collect::<Vec<_>>();
    tanimli.sort();
    if tanimli.iter().any(|ad| ad == ham) {
        return Ok(ham.to_string());
    }
    let adaylar = kok_adaylari(ham);
    let eslesenler = tanimli
        .iter()
        .filter(|ad| adaylar.iter().any(|aday| aday == *ad))
        .cloned()
        .collect::<Vec<_>>();
    match eslesenler.len() {
        1 => Ok(eslesenler[0].clone()),
        0 => {
            let oneri = if tanimli.is_empty() {
                "Bir değeri kullanmadan önce \"<ad> <değer> olsun\" ile tanımla.".to_string()
            } else {
                format!("Bu ad tanımlı değil. Tanımlı adlar: {}. Önce \"<ad> <değer> olsun\" ile tanımla.", tanimli.join(", "))
            };
            Err(Tani::yeni(
                "A001",
                format!("\"{}\" adı bu kapsamda tanımlı değil.", ham),
                7,
                3,
                5,
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
            7,
            3,
            5,
        )
        .onerili("Adlardan birini değiştir; belirsizlik dilde hata sayılır.".into())),
    }
}

#[test]
fn karar_tani_span_ve_oneri_onceki_algoritmayla_ayni_kalir() {
    let adlar = [
        "sayac", "sayaç", "sayacı", "kitap", "kitab", "elma", "elması", "kâr", "renk", "su",
    ];
    let sorgular = [
        "sayac",
        "sayaç",
        "sayacı",
        "sayacının",
        "kitabı",
        "kitap",
        "elması",
        "elmasıyla",
        "kârı",
        "rengi",
        "suyu",
        "yok",
        "İzmir",
    ];
    for maske in 0..(1usize << adlar.len()) {
        // Ters ekleme sırası ve HashMap'in farklı seed'leri tanı sırasını değiştiremez.
        for ters in [false, true] {
            let mut ortam = HashMap::new();
            for i in 0..adlar.len() {
                let i = if ters { adlar.len() - 1 - i } else { i };
                if maske & (1 << i) != 0 {
                    ortam.insert(adlar[i].to_string(), Tur::TamSayi);
                }
            }
            for ham in sorgular {
                let yeni = ad_cozumle(ham, &ortam, 7, 3, 5);
                let eski = onceki(ham, &ortam);
                match (yeni, eski) {
                    (Ok(a), Ok(b)) => assert_eq!(a, b),
                    (Err(a), Err(b)) => {
                        assert_eq!(
                            (a.kod, a.mesaj, a.satir, a.sutun, a.uzunluk, a.oneri, a.koken),
                            (b.kod, b.mesaj, b.satir, b.sutun, b.uzunluk, b.oneri, b.koken)
                        );
                    }
                    (a, b) => panic!("karar farklı: {ham} maske={maske}: {a:?} / {b:?}"),
                }
            }
        }
    }
}
