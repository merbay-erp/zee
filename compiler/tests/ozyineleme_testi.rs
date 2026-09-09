//! Özyineleme (v0.2, RFC-0006 §5.1): temel-durum-önce kuralı + derinlik korkuluğu.

use dil::kaynagi_calistir;

const FAKTORIYEL: &str = "\
işlem faktöriyelini hesapla
    sayıyı al

    sayı 1 den küçükse
        1 döndür
    bir_eksiği sayı ile 1 in farkı olsun
    alt bir_eksiği için faktöriyelini hesapla olsun
    sonucu sayı ile altın çarpımı olsun
    sonucu döndür
";

#[test]
fn faktoriyel_calisir() {
    let kaynak = format!(
        "{}\nx 5 için faktöriyelini hesapla olsun\nx yaz\n",
        FAKTORIYEL
    );
    let cikti = kaynagi_calistir(&kaynak).expect("faktöriyel çalışmalı");
    assert_eq!(cikti, vec!["120"]);
}

#[test]
fn fibonacci_cift_ozyineleme() {
    let kaynak = "\
işlem fibonaççiyi hesapla
    sayıyı al

    sayı 2 den küçükse
        sayıyı döndür
    bir_önce sayı ile 1 in farkı olsun
    iki_önce sayı ile 2 nin farkı olsun
    a bir_önce için fibonaççiyi hesapla olsun
    b iki_önce için fibonaççiyi hesapla olsun
    toplam a ile b nin toplamı olsun
    toplamı döndür

x 10 için fibonaççiyi hesapla olsun
x yaz
";
    // Linux aarch64 debug derlemesinde yorumlayıcı çerçevesi düzey başına
    // ~200–400 KiB tutar: on düzeylik fib(10) 2 MiB'lık test iş parçacığında
    // taşar. Sınıra dokunan test CLI ile aynı yığını getirir (K-040/K-178).
    let cikti = std::thread::Builder::new()
        .stack_size(dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.calistirma_yigin_bayti())
        .spawn(move || kaynagi_calistir(kaynak).expect("fibonacci çalışmalı"))
        .expect("iş parçacığı açılamadı")
        .join()
        .expect("iş parçacığı düştü");
    assert_eq!(cikti, vec!["55"]);
}

#[test]
fn karsilikli_ozyineleme() {
    // çift mi / tek mi birbirini çağırır.
    let kaynak = "\
işlem çiftliğine bak
    sayıyı al
    sayı 0 a eşitse
        doğru döndür
    bir_eksiği sayı ile 1 in farkı olsun
    sonuç bir_eksiği için tekliğine bak olsun
    sonucu döndür

işlem tekliğine bak
    sayıyı al
    sayı 0 a eşitse
        yanlış döndür
    bir_eksiği sayı ile 1 in farkı olsun
    sonuç bir_eksiği için çiftliğine bak olsun
    sonucu döndür

x 7 için çiftliğine bak olsun
x doğru olana kadar tekrarla
    \"7 tek\" yaz
    programı bitir
";
    let cikti = kaynagi_calistir(kaynak).expect("karşılıklı özyineleme çalışmalı");
    assert_eq!(cikti, vec!["7 tek"]);
}

#[test]
fn derinlik_korkulugu() {
    // Temel durum yazılı ama argüman değişmediği için asla yakalanmıyor:
    // Rust yığını taşmadan Türkçe C019 tanısı gelmeli.
    let kaynak = "\
işlem düş
    sayıyı al
    sayı 0 a eşitse
        1 döndür
    sonuç sayı için düş olsun
    sonucu döndür

x 5 için düş olsun
";
    // K-178: sınıra dokunan test CLI ile aynı resmî yığında koşar; sözün
    // kanıtı ayrı bir yığın değil, ürünün kendi yapılandırmasıdır.
    let hata = std::thread::Builder::new()
        .stack_size(dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.calistirma_yigin_bayti())
        .spawn(move || kaynagi_calistir(kaynak).expect_err("sonsuz iniş C019 vermeli"))
        .expect("iş parçacığı açılamadı")
        .join()
        .expect("iş parçacığı düştü");
    assert_eq!(hata.kod, "C019");
    assert!(hata.oneri.as_deref().unwrap_or("").contains("temel durum"));
}

#[test]
fn ozyinelemeli_tur_uyusmazligi() {
    let kaynak = "\
işlem karıştır
    sayıyı al
    sayı 0 a eşitse
        1 döndür
    x sayı için karıştır olsun
    \"metin\" döndür

y 3 için karıştır olsun
";
    let hata = kaynagi_calistir(kaynak).expect_err("dönüş birleşimi T018");
    assert_eq!(hata.kod, "T018");
}

#[test]
fn cok_tokenli_cagri_argumanlari() {
    // v0.2: argümanlar bölge olabilir (S020 esnetildi).
    let kaynak = "\
işlem topla
    birinciyi al
    ikinciyi al
    toplam birinci ile ikincinin toplamı olsun
    toplamı döndür

sayılar 1, 2, 3 listesi olsun
taban 3,5 olsun
x sayıların adedi ve tabanın tam kısmı ile topla olsun
x yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("bölge argümanlar çalışmalı");
    assert_eq!(cikti, vec!["6"]);
}

#[test]
fn c019_sinirinin_altindaki_derinlik_resmi_yiginda_her_profilde_sigar() {
    // spec/05: 500 derinlik her platformda C019 ile karşılanır, doğal yığın
    // taşmasıyla değil. K-178 ölçümü: debug ~194 KiB/düzey, release ~11 KiB.
    // 499 düzey resmî yığında (debug testte de) sığmalı; sığmazsa çerçeve
    // maliyeti büyümüştür ve K-040 dersi yeniden yaşanır.
    let kaynak = "\
işlem düş
    sayıyı al
    sayı 0 a eşitse
        0 döndür
    kalan sayı ile 1 in farkı olsun
    sonuç kalan için düş olsun
    toplam sonuç ile 1 in toplamı olsun
    toplamı döndür

x 499 için düş olsun
x yaz
";
    let cikti = std::thread::Builder::new()
        .stack_size(dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.calistirma_yigin_bayti())
        .spawn(move || kaynagi_calistir(kaynak).expect("499 düzey resmî yığında çalışmalı"))
        .expect("iş parçacığı açılamadı")
        .join()
        .expect("iş parçacığı düştü");
    assert_eq!(cikti, vec!["499"]);
}
