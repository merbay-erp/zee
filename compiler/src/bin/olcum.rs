//! `olcum` — performans ölçüm koşucusu (master plan bölüm 22).
//!
//! İlke: önce doğruluk, sonra hız — ama hız SÜREKLİ ölçülür. Bu koşucu
//! sabit iş yüklerini hermetik IO ile koşar ve süreleri raporlar; sonuçlar
//! sürümler arası docs/olcumler.md'de arşivlenir.
//!
//! Kullanım:  cargo run --release --bin olcum          (tam: 5 tur, medyan)
//!            cargo run --release --bin olcum -- --hizli  (CI dumanı: 1 tur)

use std::time::Instant;

struct IsYuku {
    ad: &'static str,
    aciklama: &'static str,
    kaynak: String,
}

fn is_yukleri() -> Vec<IsYuku> {
    // Derleme yükü: 500 blokluk sentetik program (anlık typecheck hedefi).
    let mut buyuk_kaynak = String::new();
    for i in 0..500 {
        buyuk_kaynak.push_str(&format!(
            "değer{i} {i} olsun\nkatı{i} değer{i} ile 3 ün çarpımı olsun\nkatı{i} 0 dan büyükse\n    toplam{i} katı{i} ile 1 in toplamı olsun\n"
        ));
    }

    vec![
        IsYuku {
            ad: "derleme",
            aciklama: "2000 satırlık programın derlenmesi (yalnız denetim)",
            kaynak: buyuk_kaynak,
        },
        IsYuku {
            ad: "özyineleme",
            aciklama: "fibonacci(22) — ~28 bin çağrı",
            kaynak: "işlem fibonaçiyi hesapla\n    sayıyı al\n    sayı 2 den küçükse\n        sayıyı döndür\n    a sayı ile 1 in farkı için fibonaçiyi hesapla olsun\n    b sayı ile 2 nin farkı için fibonaçiyi hesapla olsun\n    sonucu a ile b nin toplamı olsun\n    sonucu döndür\n\nx 22 için fibonaçiyi hesapla olsun\nx yaz\n".into(),
        },
        IsYuku {
            ad: "döngü",
            aciklama: "100 bin turluk sayaç döngüsü",
            kaynak: "toplam 0 olsun\n100000 kez tekrarla\n    toplamı 1 artır\ntoplam yaz\n".into(),
        },
        IsYuku {
            ad: "liste",
            aciklama: "5 bin öğe ekle + gezerek topla",
            kaynak: "sayılar boş liste olsun\n1 den 5000 e kadar her tur için\n    sayılara turu ekle\ntoplam 0 olsun\nher sayı için\n    toplamı sayıyla artır\ntoplam yaz\n".into(),
        },
        IsYuku {
            ad: "ondalık",
            aciklama: "20 bin onluk toplama (0,1 adımlı)",
            kaynak: "t 0,0 olsun\n20000 kez tekrarla\n    t t ile 0,1 in toplamı olsun\nt yaz\n".into(),
        },
        IsYuku {
            ad: "metin",
            aciklama: "2 bin birleştirme",
            kaynak: "birleşik \"\" olsun\n2000 kez tekrarla\n    birleşik birleşik ile \"a\" olsun\nbirleşiğin uzunluğu yaz\n".into(),
        },
    ]
}

fn olc(yuk: &IsYuku, tur: usize) -> Vec<f64> {
    let mut sureler = Vec::with_capacity(tur);
    for _ in 0..tur {
        let baslangic = Instant::now();
        if yuk.ad == "derleme" {
            dil::kaynagi_derle(&yuk.kaynak).expect("derleme yükü geçerli olmalı");
        } else {
            dil::kaynagi_calistir(&yuk.kaynak).expect("iş yükü çalışmalı");
        }
        sureler.push(baslangic.elapsed().as_secs_f64() * 1000.0);
    }
    sureler
}

fn medyan(sureler: &mut [f64]) -> f64 {
    sureler.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sureler[sureler.len() / 2]
}

fn main() {
    let hizli = std::env::args().any(|a| a == "--hizli");
    let tur = if hizli { 1 } else { 5 };
    let profil = if cfg!(debug_assertions) { "debug" } else { "release" };

    println!("dil {} ölçümleri — {} profili, {} tur (medyan)", env!("CARGO_PKG_VERSION"), profil, tur);
    println!("{:<12} {:>10}  açıklama", "yük", "süre");
    println!("{}", "-".repeat(64));

    for yuk in is_yukleri() {
        // Isınma turu ölçüme girmez.
        olc(&yuk, 1);
        let mut sureler = olc(&yuk, tur);
        println!("{:<12} {:>8.1} ms  {}", yuk.ad, medyan(&mut sureler), yuk.aciklama);
    }

    if cfg!(debug_assertions) {
        println!("\nNot: arşive yalnız --release ölçümleri girer (docs/olcumler.md).");
    }
}
