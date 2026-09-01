//! Hata kataloğu tutarlılık testi: kaynaktaki her tanı kodu
//! docs/hata-katalogu.md'de belgelenmiş olmalı — ve tersi.
//!
//! Yeni bir Tani::yeni("X123", ...) eklenip katalog unutulursa bu test kırılır.

use std::collections::BTreeSet;

/// Metindeki `Ö###` biçimli kodları toplar (Ö ∈ {S,A,T,C,D,P,Ç}).
fn kodlari_topla(metin: &str) -> BTreeSet<String> {
    let mut kodlar = BTreeSet::new();
    let karakterler: Vec<char> = metin.chars().collect();
    for i in 0..karakterler.len().saturating_sub(3) {
        let on_ek = karakterler[i];
        if !matches!(on_ek, 'S' | 'A' | 'T' | 'C' | 'D' | 'P' | 'Ç') {
            continue;
        }
        // Önceki karakter harf/rakamsa bu bir kelimenin ortasıdır.
        if i > 0 && (karakterler[i - 1].is_alphanumeric()) {
            continue;
        }
        let rakamlar: String = karakterler[i + 1..i + 4].iter().collect();
        if rakamlar.chars().all(|r| r.is_ascii_digit()) {
            // Sonrası rakam olmamalı (S0011 gibi yanlış eşleşmeye karşı).
            if karakterler.get(i + 4).map(|k| k.is_ascii_digit()) == Some(true) {
                continue;
            }
            kodlar.insert(format!("{}{}", on_ek, rakamlar));
        }
    }
    kodlar
}

#[test]
fn katalog_kaynakla_birebir() {
    let kok = env!("CARGO_MANIFEST_DIR");

    let mut kaynak_kodlari = BTreeSet::new();
    for dosya in [
        "src/sozcukleyici.rs",
        "src/ayristirici.rs",
        "src/cozumleyici.rs",
        "src/eylem.rs",
        "src/yorumlayici.rs",
        "src/bicimleyici.rs",
        "src/proje.rs",
        "src/paket.rs",
        "src/registry.rs",
        "src/tedarik.rs",
        "src/lib.rs",
        "src/main.rs",
    ] {
        let icerik = std::fs::read_to_string(format!("{}/{}", kok, dosya)).expect(dosya);
        // Yalnız gerçekten üretilen kodlar: Tani::yeni("...") ilk argümanları.
        for parca in icerik.split("Tani::yeni(") {
            if let Some(tirnakli) = parca.trim_start().strip_prefix('"') {
                if let Some(kapali) = tirnakli.split('"').next() {
                    kaynak_kodlari.extend(kodlari_topla(kapali));
                }
            }
        }
        // proje.rs aynı tanı iskeletini `proje_hatasi` yardımcısıyla kurar.
        for parca in icerik.split("proje_hatasi(") {
            if let Some(tirnakli) = parca.trim_start().strip_prefix('"') {
                if let Some(kapali) = tirnakli.split('"').next() {
                    kaynak_kodlari.extend(kodlari_topla(kapali));
                }
            }
        }
        // Kaynak konumu olmayan CLI/tedarik kapıları kodu doğrudan başta
        // basabilir (`eprintln!("P012: ...")`). Bunlar da katalog
        // sözleşmesidir; belge güncellemesi unutulamaz.
        for parca in icerik.split("eprintln!(\"") {
            if let Some(ilk) = parca.split('"').next() {
                kaynak_kodlari.extend(kodlari_topla(ilk));
            }
        }
        // Konumsuz registry güven/politika hataları kapalı hata türünde
        // doğrudan `kod: "P..."` alanıyla doğar.
        for parca in icerik.split("kod: \"") {
            if let Some(kod) = parca.split('"').next() {
                kaynak_kodlari.extend(kodlari_topla(kod));
            }
        }
        // Nöbetçi karşılaştırmaları da (tani.kod == "Ç000") kataloğa girmeli.
        for parca in icerik.split("tani.kod == \"") {
            if let Some(kod) = parca.split('"').next() {
                kaynak_kodlari.extend(kodlari_topla(kod));
            }
        }
    }

    let katalog = std::fs::read_to_string(format!("{}/../docs/hata-katalogu.md", kok))
        .expect("docs/hata-katalogu.md okunmalı");
    // "ayrılmış" notundaki kodlar (A004 gibi) kullanımda değildir; muaf tutulur.
    let katalog_govdesi: String = katalog
        .lines()
        .filter(|satir| !satir.contains("ayrılmış"))
        .collect::<Vec<_>>()
        .join("\n");
    let katalog_kodlari = kodlari_topla(&katalog_govdesi);

    let eksikler: Vec<_> = kaynak_kodlari.difference(&katalog_kodlari).collect();
    assert!(
        eksikler.is_empty(),
        "Kaynakta olup katalogda olmayan tanı kodları: {:?} — docs/hata-katalogu.md'yi güncelle.",
        eksikler
    );

    let fazlalar: Vec<_> = katalog_kodlari.difference(&kaynak_kodlari).collect();
    assert!(
        fazlalar.is_empty(),
        "Katalogda olup kaynakta üretilmeyen kodlar: {:?} — katalogdan çıkar ya da 'ayrılmış' notuna taşı.",
        fazlalar
    );

    // Akıl sağlığı: en az bilinen çekirdek kodlar mevcut.
    for cekirdek in ["S001", "A001", "T001", "C003", "D001", "P001"] {
        assert!(
            kaynak_kodlari.contains(cekirdek),
            "çekirdek kod kayıp: {}",
            cekirdek
        );
    }
}
