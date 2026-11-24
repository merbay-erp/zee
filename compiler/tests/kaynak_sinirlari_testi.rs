//! B-025 merkezî kaynak politikası regresyonları.

use dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI;

#[test]
fn runtime_cikti_olay_butcesini_kontrollu_taniyla_keser() {
    let adet = VARSAYILAN_KAYNAK_SINIRLARI.cikti_olayi() + 1;
    let kaynak = format!("{} kez tekrarla\n    \"x\" yaz\n", adet);
    let hata = dil::kaynagi_calistir(&kaynak).expect_err("çıktı bütçesi aşılmalı");
    assert_eq!(hata.kod, "C023", "{hata:?}");
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

#[test]
fn metin_buyumesi_tahsis_oncesi_kesilir() {
    let kaynak = "metin \"x\" olsun\n25 kez tekrarla\n    metin metin ile metin olsun\n";
    let hata = dil::kaynagi_calistir(kaynak).expect_err("metin sınırı aşılmalı");
    assert_eq!(hata.kod, "C024");
    assert!(hata.mesaj.contains("Metin"));
}

#[test]
fn saklanan_degerler_calisma_heap_butcesini_asamaz() {
    let kaynak = "yük \"x\" olsun\n20 kez tekrarla\n    yük yük ile yük olsun\ndepo boş liste olsun\n70 kez tekrarla\n    depoya yükü ekle\n";
    let hata = dil::kaynagi_calistir(kaynak).expect_err("heap sınırı aşılmalı");
    assert_eq!(hata.kod, "C024", "{hata:?}");
    assert!(hata.mesaj.contains("heap sınırını"));
}

#[test]
fn surec_geneli_baglanti_izinleri_sinirli_ve_iade_edilir() {
    let azami = VARSAYILAN_KAYNAK_SINIRLARI.ag_baglantisi();
    let mut izinler = (0..azami)
        .map(|_| dil::kaynak_sinirlari::baglanti_izni_al().expect("izin alınmalı"))
        .collect::<Vec<_>>();
    assert!(dil::kaynak_sinirlari::baglanti_izni_al().is_err());
    izinler.pop();
    assert!(dil::kaynak_sinirlari::baglanti_izni_al().is_ok());
}

#[test]
fn gorev_ortami_klonlari_kurulmadan_heap_butcesine_girer() {
    let mut kaynak = String::from(
        "yük \"x\" olsun\n20 kez tekrarla\n    yük yük ile yük olsun\n\
         işlem bir ver\n    1 döndür\n\n\
         eşzamanlı olarak\n",
    );
    for sira in 0..64 {
        kaynak.push_str(&format!("    görev{} bir ver\n", sira));
    }
    kaynak.push_str("hepsini bekle\n");

    let hata = dil::kaynagi_calistir(&kaynak).expect_err("ortam klonları reddedilmeli");
    assert_eq!(hata.kod, "C024", "{hata:?}");
    assert!(hata.mesaj.contains("heap sınırını"));
}

#[test]
fn kendine_metin_eklemek_heap_butcesini_karesel_tuketmez() {
    // `x x ile y olsun` yerinde büyür; zarftan yalnız kapasite artışı düşer.
    // 50.000 × 116 bayt ≈ 5,8 MB sonuç, eski muhasebede ~290 GB sayılırdı.
    let kaynak = "sayfa \"\" olsun\n50000 kez tekrarla\n    sayfa sayfa ile \"<tr><td>kayıt</td><td>örnek açıklama metni</td><td>12,50</td><td>tamam</td><td>2026-10-20</td><td>-</td></tr>\" olsun\nsayfanın uzunluğu yaz\n";
    let cikti = dil::kaynagi_calistir(kaynak).expect("yerinde büyüme bütçeyi aşmamalı");
    assert_eq!(cikti.len(), 1);
    assert!(
        cikti[0].parse::<usize>().unwrap() > 50_000 * 100,
        "{cikti:?}"
    );
}

#[test]
fn kendine_ekleme_metin_ve_heap_sinirlarini_korur() {
    let metin = "metin \"x\" olsun\n25 kez tekrarla\n    metin metin ile metin olsun\n";
    let hata = dil::kaynagi_calistir(metin).expect_err("16 MiB metin sınırı korunmalı");
    assert_eq!(hata.kod, "C024");
    assert!(hata.mesaj.contains("Metin"), "{hata:?}");

    let birikim = "yük \"x\" olsun\n20 kez tekrarla\n    yük yük ile yük olsun\ndepo boş liste olsun\n70 kez tekrarla\n    depoya yükü ekle\n";
    let hata = dil::kaynagi_calistir(birikim).expect_err("heap sınırı korunmalı");
    assert_eq!(hata.kod, "C024");
    assert!(
        hata.oneri.as_deref().unwrap_or("").contains("birleşmişi"),
        "öneri liste+birleştir kalıbını göstermeli: {hata:?}"
    );
}

#[test]
fn baska_adla_baslayan_birlestirme_eski_anlamini_korur() {
    let kaynak =
        "baş \"<\" olsun\nson \"\" olsun\n3 kez tekrarla\n    son baş ile son olsun\nson yaz\n";
    let cikti = dil::kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, ["<<<"]);
}

/// K-393: ~1 KiB'lık parçadan 1.000 öğelik (~1 MiB) liste kuran önek.
const MIB_LISTE: &str = "parça \"0123456789abcdef\" olsun\n6 kez tekrarla\n    parça parça ile parça olsun\ndizi boş liste olsun\n1000 kez tekrarla\n    diziye parçayı ekle\n";

#[test]
fn atilan_degerin_zarf_payi_iade_edilir() {
    // Her yol eskiden iade edilmeyen zarfta birikip 64 MiB'da C024 veriyordu.
    let vakalar = [
        // İşlem çerçevesi: parametre kopyası çağrı bitince düşer.
        "işlem say\n    öğeleri Metin listesi olarak al\n    TamSayı döndürür\n    öğelerin adedi döndür\n\nVAKA_ONEK\nk 0 olsun\n200 kez tekrarla\n    k dizi için say olsun\nk yaz\n",
        // Değişkenin üzerine yazma.
        "VAKA_ONEK\nkopya boş liste olsun\n200 kez tekrarla\n    kopya dizi olsun\nkopyanın adedi yaz\n",
        // Blok kapsamında doğup düşen ad.
        "VAKA_ONEK\n200 kez tekrarla\n    doğru ise\n        geçici dizi olsun\ndizinin adedi yaz\n",
        // Gezme kopyası döngü bitince düşer.
        "VAKA_ONEK\nn 0 olsun\n200 kez tekrarla\n    dizideki her öğe için\n        n 1 artır\nn yaz\n",
        // Sözlük öğesi ve yapı alanının üzerine yazma.
        "yapı Kutu\n    içerik Metin\n\nVAKA_ONEK\nkutu yeni Kutu olsun\nönbellek boş sözlük olsun\n100000 kez tekrarla\n    önbelleğin \"son\" değeri parça olsun\n    kutunun içeriği parça olsun\nson kutunun içeriği olsun\nsonun uzunluğu yaz\n",
        // Silinen öğe.
        "VAKA_ONEK\nkutu boş liste olsun\n100000 kez tekrarla\n    kutuya parçayı ekle\n    kutudan parçayı sil\nkutunun adedi yaz\n",
    ];
    for vaka in vakalar {
        let kaynak = vaka.replace("VAKA_ONEK\n", MIB_LISTE);
        let cikti = dil::kaynagi_calistir(&kaynak)
            .unwrap_or_else(|hata| panic!("iade beklenirdi: {hata:?}\n{kaynak}"));
        assert_eq!(cikti.len(), 1, "{kaynak}");
    }
}

#[test]
fn iade_canli_birikimi_zarftan_kacirmaz() {
    // Canlı biriktirme iade almaz; düşen değerlerin iadesi sınırı genişletmez.
    let vakalar = [
        // Düz canlı birikim.
        "VAKA_ONEK\ndepo boş liste olsun\n200 kez tekrarla\n    depoya diziyi ekle\n",
        // Çerçeve, kapsam, sözlük ve silme iadeleriyle karışık canlı birikim.
        "işlem çöp\n    s Metin olarak al\n    TamSayı döndürür\n    kutu boş liste olsun\n    100 kez tekrarla\n        kutuya s ekle\n    kutudan s yi sil\n    sözlük boş sözlük olsun\n    sözlüğün \"a\" değeri s olsun\n    sözlüğün \"a\" değeri \"\" olsun\n    1 döndür\n\nVAKA_ONEK\ndepo boş liste olsun\nk 0 olsun\n100000 kez tekrarla\n    depoya parçayı ekle\n    k parça için çöp olsun\n    doğru ise\n        yerel parça ile \"x\" olsun\n",
        // Gezme imleci geri yazımı kaynağı büyütür; kopya tüketilmek zorunda.
        "VAKA_ONEK\nsatırlar boş liste olsun\n2000 kez tekrarla\n    satırlara \"a\" ekle\n200 kez tekrarla\n    her satır için\n        satır satır ile dizinin ilki olsun\n",
        // Dönen değer çağıranda yeniden tüketilir.
        "işlem ver\n    l Metin listesi olarak al\n    Metin listesi döndürür\n    l döndür\n\nVAKA_ONEK\ndepo boş liste olsun\n200 kez tekrarla\n    depoya dizi için ver ekle\n",
    ];
    for vaka in vakalar {
        let kaynak = vaka.replace("VAKA_ONEK\n", MIB_LISTE);
        let hata = dil::kaynagi_calistir(&kaynak).expect_err(&kaynak);
        assert_eq!(hata.kod, "C024", "{hata:?}\n{kaynak}");
        assert!(hata.mesaj.contains("heap sınırını"), "{hata:?}");
    }
}
