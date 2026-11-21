//! K-363 (GB-109 paket/cache yayın süresi): registry cache'inin yayın
//! sırası, eşzamanlı nesne yayını ve süre/eskime sınırları. spec/19: önce
//! değişmez nesneler atomik yazılır, en son `durum-v1.json` yayımlanır;
//! süre sonu güncellemenin başındaki andan kesin büyüktür; çevrimdışı kip
//! yalnız son çevrimiçi kabul anını kullanır ve güncellik iddia etmez.

use super::*;
use std::sync::{Arc, Barrier};

/// `2030-01-01T00:00:00Z` Unix saniyesi.
const T: i64 = 1_893_456_000;

fn kur() -> (GeciciKlasor, Anahtarlar, YayinFixture, Vec<u8>, String) {
    let gecici = GeciciKlasor::yeni();
    let anahtarlar = anahtarlar();
    let yayin = yayin_fixture(&gecici);
    let root = root(&anahtarlar);
    let root_ozeti = format!("sha256:{}", sha256_hex(&root));
    (gecici, anahtarlar, yayin, root, root_ozeti)
}

fn guncelle(
    istemci: &RegistryIstemcisi,
    tasiyici: &mut SahteTasiyici,
    zaman: i64,
) -> Result<RegistryPaketCiktisi, RegistryHatasi> {
    istemci.paketi_guncelle_zamanla(
        tasiyici,
        "miras",
        "1.2.3",
        HedefPolitikasi::default(),
        zaman,
    )
}

fn cevrimdisi(istemci: &RegistryIstemcisi) -> Result<RegistryPaketCiktisi, RegistryHatasi> {
    istemci.paketi_cevrimdisi_al("miras", "1.2.3", HedefPolitikasi::default())
}

fn agaci_kopyala(kaynak: &Path, hedef: &Path) {
    std::fs::create_dir_all(hedef).expect("hedef klasör");
    for girdi in std::fs::read_dir(kaynak).expect("kaynak klasör") {
        let girdi = girdi.expect("girdi");
        let yol = girdi.path();
        if girdi.file_type().expect("tür").is_dir() {
            agaci_kopyala(&yol, &hedef.join(girdi.file_name()));
        } else {
            std::fs::copy(&yol, hedef.join(girdi.file_name())).expect("kopya");
        }
    }
}

/// Cache'teki her dosya kendi içerik adresinde, salt okunur ve tam baytlıdır;
/// yarım geçici dosya kalmaz.
fn nesneler_icerik_adresli(cache: &Path) -> usize {
    let klasor = cache.join("nesneler").join("sha256");
    let mut sayi = 0;
    for girdi in std::fs::read_dir(&klasor).expect("nesne klasörü") {
        let yol = girdi.expect("nesne").path();
        // atomik_yaz'ın klasör kilidi boş kalıcı dosyadır; başka gizli
        // (yarım geçici) dosya kalmamalı.
        if yol.file_name().is_some_and(|ad| ad == ".zee-yazma-kilidi") {
            assert_eq!(std::fs::metadata(&yol).expect("kilit").len(), 0);
            continue;
        }
        let baytlar = std::fs::read(&yol).expect("nesne oku");
        assert_eq!(
            nesne_yolu(cache, &sha256_hex(&baytlar)).expect("adres"),
            yol,
            "nesne kendi SHA-256 adresinde olmalı"
        );
        assert!(std::fs::metadata(&yol)
            .expect("izin")
            .permissions()
            .readonly());
        sayi += 1;
    }
    sayi
}

/// Çökme `durum-v1.json` yayımlanmadan önce gelirse (yalnız değişmez nesneler
/// yazılmış) cache çevrimdışı kullanılamaz: P016 cache miss. Sonraki
/// çevrimiçi güncelleme aynı nesneleri baytça doğrulayıp yeniden kullanır ve
/// durumu en son yayımlar.
#[test]
fn durum_yayimlanmadan_kalan_nesneler_gorunmez_ve_sonra_yeniden_kullanilir() {
    let (gecici, anahtarlar, yayin, root, root_ozeti) = kur();
    let tam = gecici.yol().join("tam");
    let tam_istemci = RegistryIstemcisi::yeni(&tam, root.clone(), root_ozeti.clone());
    guncelle(&tam_istemci, &mut tasiyici(&anahtarlar, &yayin, 3), 1).expect("tam güncelleme");

    let yarim = gecici.yol().join("yarim");
    agaci_kopyala(&tam.join("nesneler"), &yarim.join("nesneler"));
    assert!(!yarim.join(DURUM_DOSYASI).exists());
    let istemci = RegistryIstemcisi::yeni(&yarim, root, root_ozeti);
    let miss = cevrimdisi(&istemci).expect_err("durumsuz nesneler görünmez");
    assert_eq!(miss.kod, "P016");
    assert!(miss.mesaj.contains("kalıcı doğrulama durumu yok"), "{miss}");

    let sonuc = guncelle(&istemci, &mut tasiyici(&anahtarlar, &yayin, 3), 2)
        .expect("var olan doğru nesneler yeniden kullanılır");
    assert_eq!(
        sonuc.arsiv,
        nesne_yolu(&yarim, &yayin.hedef.arsiv.sha256).expect("adres")
    );
    assert!(yarim.join(DURUM_DOSYASI).is_file());
    assert_eq!(
        nesneler_icerik_adresli(&yarim),
        nesneler_icerik_adresli(&tam)
    );
    assert!(cevrimdisi(&istemci).expect("durum yayımlandı").cevrimdisi);
}

/// Aynı içerik adresinde yarım (kesik) nesne ağ varken bile sessizce
/// iyileştirilmez: çevrimiçi güncelleme P016 ile durur, kalıcı durum ve bozuk
/// nesne olduğu gibi kalır; çevrimdışı açılış da P016'dır.
#[test]
fn ayni_adreste_yarim_nesne_ag_varken_de_iyilestirilmez() {
    let (gecici, anahtarlar, yayin, root, root_ozeti) = kur();
    let cache = gecici.yol().join("cache");
    let istemci = RegistryIstemcisi::yeni(&cache, root, root_ozeti);
    let ilk = guncelle(&istemci, &mut tasiyici(&anahtarlar, &yayin, 3), 1).expect("ilk");
    let durum = std::fs::read(cache.join(DURUM_DOSYASI)).expect("durum");
    dosyayi_yazilabilir_yap(&ilk.arsiv);
    let yarim = &yayin.arsiv[..yayin.arsiv.len() / 2];
    std::fs::write(&ilk.arsiv, yarim).expect("nesneyi kes");

    let hata = guncelle(&istemci, &mut tasiyici(&anahtarlar, &yayin, 4), 2)
        .expect_err("kesik nesne yeniden yazılmaz");
    assert_eq!(hata.kod, "P016", "{hata}");
    assert_eq!(
        std::fs::read(cache.join(DURUM_DOSYASI)).expect("durum"),
        durum
    );
    assert_eq!(std::fs::read(&ilk.arsiv).expect("nesne"), yarim);
    assert_eq!(cevrimdisi(&istemci).expect_err("bozuk").kod, "P016");
}

/// Süre sınırı kesin büyüktür: `sona_erme` anından bir saniye önce kabul,
/// tam anında ret. Eskiyen zincir kalıcı durumu değiştirmez; çevrimdışı kip
/// son çevrimiçi kabul anıyla açılmaya devam eder (güncellik iddiası yok).
/// Yeni süreli zincir gelince kabul anı ilerler; eski zincir geri sarılamaz.
#[test]
fn sure_siniri_kesin_ve_eskiyen_zincir_durumu_degistirmez() {
    let (gecici, anahtarlar, yayin, root, root_ozeti) = kur();
    let cache = gecici.yol().join("cache");
    let istemci = RegistryIstemcisi::yeni(&cache, root, root_ozeti);
    let eski = || tasiyici_sureli(&anahtarlar, &yayin, 3, &[], "2030-01-01T00:00:00Z");
    let kabul_ani = |istemci: &RegistryIstemcisi| {
        istemci
            .durumu_oku()
            .expect("durum")
            .0
            .expect("durum var")
            .dogrulama_zamani
    };

    guncelle(&istemci, &mut eski(), T - 1).expect("bitimden bir saniye önce geçerli");
    assert_eq!(kabul_ani(&istemci), T - 1);
    let durum = std::fs::read(cache.join(DURUM_DOSYASI)).expect("durum");

    for zaman in [T, T + 86_400] {
        let hata = guncelle(&istemci, &mut eski(), zaman).expect_err("süresi dolmuş zincir");
        assert!(hata.mesaj.contains("süresi"), "{zaman}: {hata}");
        assert_eq!(
            std::fs::read(cache.join(DURUM_DOSYASI)).expect("durum"),
            durum,
            "{zaman}: eskiyen zincir durumu değiştirmez"
        );
        let offline = cevrimdisi(&istemci).expect("son kabul anıyla açılır");
        assert!(offline.cevrimdisi);
        assert_eq!(offline.metadata_surumleri.targets, 3);
    }

    let mut yeni = tasiyici_sureli(&anahtarlar, &yayin, 4, &[], "2031-01-01T00:00:00Z");
    guncelle(&istemci, &mut yeni, T + 10).expect("yeni süreli zincir");
    assert_eq!(kabul_ani(&istemci), T + 10);
    let geri = guncelle(
        &istemci,
        &mut tasiyici_sureli(&anahtarlar, &yayin, 3, &[], "2031-01-01T00:00:00Z"),
        T + 20,
    )
    .expect_err("eski sürüm geri sarılamaz");
    assert_eq!(geri.kod, "P013");
    assert_eq!(kabul_ani(&istemci), T + 10);
}

/// Sekiz istemci boş cache'e aynı anda aynı zinciri yayımlar. Hepsi başarılı;
/// her nesne tam bir kez kendi adresinde, salt okunur ve tam baytlıdır; durum
/// tek ve tutarlıdır (Windows'ta salt okunur hedefe ikinci atomik yazımın
/// düşmesi var olan doğru nesneyi kabul eder).
#[test]
fn sekiz_eszamanli_guncelleme_nesneleri_bir_kez_dogru_yayimlar() {
    let (gecici, anahtarlar, yayin, root, root_ozeti) = kur();
    let cache = gecici.yol().join("cache");
    let istemci = RegistryIstemcisi::yeni(&cache, root, root_ozeti);
    let engel = Arc::new(Barrier::new(8));
    let sonuclar = (0..8)
        .map(|_| {
            let (istemci, engel) = (istemci.clone(), engel.clone());
            let mut tasiyici = tasiyici(&anahtarlar, &yayin, 3);
            std::thread::spawn(move || {
                engel.wait();
                guncelle(&istemci, &mut tasiyici, 1)
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|is| is.join().expect("güncelleme iş parçacığı"))
        .collect::<Vec<_>>();
    for sonuc in &sonuclar {
        let sonuc = sonuc.as_ref().expect("eşzamanlı aynı zincir düşmez");
        assert_eq!(sonuc.metadata_surumleri.targets, 3);
    }
    // root + timestamp + snapshot + targets + dört yayın dosyası.
    assert_eq!(nesneler_icerik_adresli(&cache), 8);
    let durum = istemci.durumu_oku().expect("durum").0.expect("durum var");
    assert_eq!(durum.metadata.targets, 3);
    assert!(cevrimdisi(&istemci).is_ok());
}

/// K-363: registry'nin imzaladığı duyuru kimliği ekrana ve `proje.kilit`
/// politika anahtarına yazılır; bidi/sıfır genişlikli biçim karakteri
/// taşıyan kimlik (Trojan Source) metadata hatasıdır, durum yayımlanmaz.
#[test]
fn gorunmez_bicim_karakterli_duyuru_kimligi_reddedilir() {
    for kimlik in [
        "ZSA-2026\u{202e}-0001",
        "ZSA\u{200b}-2026-0001",
        "ZSA\u{061c}1",
    ] {
        let (gecici, anahtarlar, yayin, root, root_ozeti) = kur();
        let cache = gecici.yol().join("cache");
        let istemci = RegistryIstemcisi::yeni(&cache, root, root_ozeti);
        let hata = istemci
            .paketi_guncelle_zamanla(
                &mut tasiyici_duyurularla(&anahtarlar, &yayin, 3, &[kimlik]),
                "miras",
                "1.2.3",
                HedefPolitikasi {
                    kritik_duyuru_kabul: true,
                    ..HedefPolitikasi::default()
                },
                1,
            )
            .expect_err("görünmez biçimli kimlik");
        assert!(
            hata.mesaj.contains("duyuru kimliği güvenli değil"),
            "{kimlik:?}: {hata}"
        );
        assert!(!cache.join(DURUM_DOSYASI).exists());
    }
}
