//! Çocuk modu (K-047): GuvenliIo sargısı — ağ/sunucu kapalı, dosyalar
//! çalışma klasörüyle sınırlı; kalan davranış içteki IO ile birebir.

use dil::yorumlayici::{calistir_io, GirdiCikti, GuvenliIo, ToplayanIo};

fn guvenli_kostur(kaynak: &str) -> Result<Vec<String>, dil::tani::Tani> {
    let program = dil::kaynagi_derle(kaynak)?;
    let mut io = GuvenliIo::yeni(ToplayanIo::yeni(Vec::new()));
    calistir_io(&program, &mut io)?;
    Ok(io.ic.cikti)
}

#[test]
fn normal_isler_aynen_calisir() {
    let cikti = guvenli_kostur("\"selam\" yaz\nx 1 ile 6 arasında rastgele sayı olsun\n")
        .expect("çalışmalı");
    assert_eq!(cikti, vec!["selam"]);
}

#[test]
fn ust_dizin_engellenir() {
    let kaynak = "\
sonuç \"../gizli.txt\" dosyasını okumayı dene olsun
sonuç başarısızsa
    sonucun hatası yaz
";
    let cikti = guvenli_kostur(kaynak).expect("dene hatayı Sonuç yapar");
    assert_eq!(cikti.len(), 1);
    assert!(cikti[0].contains("güvenli modda"), "{}", cikti[0]);
    assert!(cikti[0].contains("dışarıyı gösteriyor"), "{}", cikti[0]);
}

#[test]
fn mutlak_yol_engellenir() {
    let kaynak = "\
sonuç \"/etc/parola\" dosyasını okumayı dene olsun
sonuç başarısızsa
    sonucun hatası yaz
";
    let cikti = guvenli_kostur(kaynak).expect("çalışmalı");
    assert!(cikti[0].contains("güvenli modda"), "{}", cikti[0]);
}

#[test]
fn yerel_yol_icteki_ioya_gecer() {
    // Sahte dünyada dosya var: sargı yerel yolu geçirmeli.
    let program = dil::kaynagi_derle("satırlar \"siir.txt\" dosyasının satırları olsun\nsatırların ilki yaz\n")
        .expect("derlenmeli");
    let mut ic = ToplayanIo::yeni(Vec::new());
    ic.dosya_yaz("siir.txt", "ilk dize", false).unwrap();
    let mut io = GuvenliIo::yeni(ic);
    calistir_io(&program, &mut io).expect("yerel dosya serbest");
    assert_eq!(io.ic.cikti, vec!["ilk dize"]);
}

#[test]
fn ag_kapali() {
    let kaynak = "cevap \"https://ornek.dev/x\" adresinden gelen yanıt olsun\n";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = GuvenliIo::yeni(ToplayanIo::yeni(Vec::new()));
    let hata = calistir_io(&program, &mut io).expect_err("ağ kapalı olmalı");
    assert!(hata.mesaj.contains("güvenli modda"), "{}", hata.mesaj);
}

#[test]
fn sunucu_kapali() {
    let kaynak = "8080 kapısında sunucu başlat\n";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let mut io = GuvenliIo::yeni(ToplayanIo::yeni(Vec::new()));
    let hata = calistir_io(&program, &mut io).expect_err("sunucu kapalı olmalı");
    assert!(hata.mesaj.contains("güvenli modda"), "{}", hata.mesaj);
}
