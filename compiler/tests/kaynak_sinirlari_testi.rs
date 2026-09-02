//! B-025 merkezî kaynak politikası regresyonları.

use dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI;

#[test]
fn runtime_cikti_olay_butcesini_kontrollu_taniyla_keser() {
    let adet = VARSAYILAN_KAYNAK_SINIRLARI.cikti_olayi() + 1;
    let kaynak = format!("{} kez tekrarla\n    \"x\" yaz\n", adet);
    let hata = dil::kaynagi_calistir(&kaynak).expect_err("çıktı bütçesi aşılmalı");
    assert_eq!(hata.kod, "C023");
    assert!(hata.mesaj.contains("çıktı bütçesini"));
}

#[test]
fn buyuk_veri_dosyasi_tahsis_oncesi_reddedilir() {
    let kok = std::env::temp_dir().join(format!(
        "zee-kaynak-siniri-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::create_dir_all(&kok).unwrap();
    let yol = kok.join("buyuk.txt");
    let dosya = std::fs::File::create(&yol).unwrap();
    dosya
        .set_len(VARSAYILAN_KAYNAK_SINIRLARI.dosya_okuma_bayti() as u64 + 1)
        .unwrap();
    drop(dosya);

    let hata = dil::kaynak_sinirlari::veri_dosyasi_oku(&yol)
        .expect_err("boyut metadata aşamasında reddedilmeli");
    assert_eq!(hata.kind(), std::io::ErrorKind::InvalidData);
    let _ = std::fs::remove_dir_all(kok);
}

#[test]
fn eszamanli_gorev_grubu_tahsis_oncesi_sinirlanir() {
    let mut kaynak = String::from("işlem bir ver\n    1 döndür\n\neşzamanlı olarak\n");
    for sira in 0..=VARSAYILAN_KAYNAK_SINIRLARI.eszamanli_gorev() {
        kaynak.push_str(&format!("    görev{} bir ver\n", sira));
    }
    kaynak.push_str("\nhepsini bekle\n");

    let hata = dil::kaynagi_calistir(&kaynak).expect_err("görev bütçesi aşılmalı");
    assert_eq!(hata.kod, "C023");
    assert!(hata.mesaj.contains("Eşzamanlı grup"));
}
