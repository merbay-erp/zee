//! K-116 `zee-io-1` rastgelelik, sanal saat ve hermetik IO sözleşmesi.

use dil::wasm_api::playgroundda_calistir;
use dil::yorumlayici::{GirdiCikti, SurumluRastgele, ToplayanIo, DETERMINISTIK_IO_PROFILI};

#[test]
fn profil_kimligi_ve_tohumlu_dizi_snapshot_ile_sabitlenir() {
    assert_eq!(DETERMINISTIK_IO_PROFILI, "zee-io-1");
    let mut rastgele = SurumluRastgele::yeni(7);
    let dizi = (0..6)
        .map(|_| rastgele.aralikta(1, 100))
        .collect::<Vec<_>>();
    assert_eq!(dizi, vec![47, 29, 69, 71, 26, 60]);
}

#[test]
fn rastgele_araligi_uclari_ve_tam_i64_uzayini_tasmadan_korur() {
    let mut rastgele = SurumluRastgele::yeni(0);
    assert_eq!(rastgele.aralikta(5, 5), 5);
    for _ in 0..256 {
        let deger = rastgele.aralikta(i64::MIN, i64::MAX);
        assert!((i64::MIN..=i64::MAX).contains(&deger));
    }
    assert_eq!(rastgele.aralikta(10, 1), 10);
}

#[test]
fn toplayan_io_rastgele_kuyrugunu_araliga_kirpar_ve_alt_uca_duser() {
    let mut io = ToplayanIo::yeni(Vec::new());
    io.rastgele_degerler.extend([-10, 4, 99]);
    assert_eq!(io.rastgele(1, 6), 1);
    assert_eq!(io.rastgele(1, 6), 4);
    assert_eq!(io.rastgele(1, 6), 6);
    assert_eq!(io.rastgele(1, 6), 1);
}

#[test]
fn sanal_saat_geriye_gitmez_ve_takvim_saatinden_ayridir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let takvim = io.simdi();
    io.an_degerleri.extend([5, 3]);
    assert_eq!(io.an_ms(), 5);
    assert_eq!(io.an_ms(), 5, "geri giden enjeksiyon gözlenmemeli");
    io.bekle_ms(-10);
    assert_eq!(io.an_ms(), 5, "negatif bekleme anı değiştirmemeli");
    io.bekle_ms(25);
    assert_eq!(io.an_ms(), 30, "bekleme sanal anı toplamalı");
    io.an_degerleri.extend([20, 40]);
    assert_eq!(io.an_ms(), 30, "eski enjeksiyon anı geri götürmemeli");
    assert_eq!(io.an_ms(), 40, "yeni enjeksiyon anı ileri taşımalı");
    assert_eq!(io.simdi(), takvim, "bekleme takvim saatini ilerletmemeli");
}

#[test]
fn hermetik_io_ve_playground_ayni_gozlenebilir_profili_korur() {
    let mut io = ToplayanIo::yeni(vec!["Zeynep".into()]);
    io.argumanlar = vec!["bir".into(), "iki".into()];
    io.http_yanitlari
        .insert("https://ornek.dev".into(), (200, "tamam".into()));
    io.sensorler.insert("kapı".into(), true);

    assert_eq!(io.sor("Ad?"), Some("Zeynep".into()));
    io.dosya_yaz("not.txt", "ilk", false).expect("yazılmalı");
    io.dosya_yaz("not.txt", "ikinci", true).expect("eklenmeli");
    assert_eq!(io.dosya_oku("not.txt"), Ok("ilk\nikinci\n".into()));
    assert!(io.dosya_oku("yok.txt").unwrap_err().contains("bulunamadı"));
    assert_eq!(
        io.http_getir("https://ornek.dev", None),
        Ok((200, "tamam".into()))
    );
    assert!(io.http_getir("https://yok.dev", None).is_err());
    assert_eq!(io.argumanlar(), vec!["bir", "iki"]);
    assert!(io.sensor_acik_mi("kapı"));
    assert!(!io.sensor_acik_mi("pencere"));
    io.isik_ayarla("yeşil", true);
    assert_eq!(io.cikti, vec!["Ad?", "[ışık] yeşil yandı"]);

    let kaynak = "a 1 ile 100 arasında rastgele sayı olsun\na yaz\nb 1 ile 100 arasında rastgele sayı olsun\nb yaz\nc 1 ile 100 arasında rastgele sayı olsun\nc yaz\n";
    assert_eq!(
        playgroundda_calistir(kaynak, "", 7),
        playgroundda_calistir(kaynak, "", 7)
    );
}
