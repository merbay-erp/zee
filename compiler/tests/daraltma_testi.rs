//! Akış-duyarlı daraltma (RFC-0008 §4.2, v0.2): varsa/başarılıysa dallarında
//! değeri/hatası statik güvenli; korumasız erişim T036 derleme hatası.

use dil::kaynagi_calistir;

const BULUCU: &str = "\
işlem çift bul
    sayıyı al
    sayı çiftse
        sayıyı döndür
    yok döndür
";

#[test]
fn varsa_dalinda_deger_guvenli() {
    let kaynak = format!(
        "{}\nbulunan 4 için çift bul olsun\nbulunan varsa\n    bulunanın değeri yaz\n",
        BULUCU
    );
    assert_eq!(kaynagi_calistir(&kaynak).expect("çalışmalı"), vec!["4"]);
}

#[test]
fn yoksa_nin_degilse_dali_da_guvenli() {
    let kaynak = format!(
        "{}\nbulunan 4 için çift bul olsun\nbulunan yoksa\n    \"boş\" yaz\ndeğilse\n    bulunanın değeri yaz\n",
        BULUCU
    );
    assert_eq!(kaynagi_calistir(&kaynak).expect("çalışmalı"), vec!["4"]);
}

#[test]
fn korumasiz_erisim_t036() {
    let kaynak = format!("{}\nbulunan 4 için çift bul olsun\nbulunanın değeri yaz\n", BULUCU);
    let hata = kaynagi_calistir(&kaynak).expect_err("T036");
    assert_eq!(hata.kod, "T036");
}

#[test]
fn dal_disina_cikinca_koruma_biter() {
    let kaynak = format!(
        "{}\nbulunan 4 için çift bul olsun\nbulunan varsa\n    \"var\" yaz\nbulunanın değeri yaz\n",
        BULUCU
    );
    let hata = kaynagi_calistir(&kaynak).expect_err("dal dışı T036");
    assert_eq!(hata.kod, "T036");
}

#[test]
fn sonuc_hatasi_ancak_basarisizsa_dalinda() {
    let kaynak = "sonuç \"yok.txt\" dosyasını okumayı dene olsun\nsonucun hatası yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("korumasız hatası T036");
    assert_eq!(hata.kod, "T036");
    assert!(hata.oneri.as_deref().unwrap_or("").contains("başarısızsa"));
}

#[test]
fn basarisizsa_dalinda_hata_guvenli() {
    let kaynak = "\
sonuç \"yok.txt\" dosyasını okumayı dene olsun
sonuç başarısızsa
    sonucun hatası yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["\"yok.txt\" dosyası bulunamadı"]);
}
