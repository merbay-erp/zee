//! K-362 / GB-109: kalıcı web oturum deposunun çökme kesitleri.
//!
//! Commit yolu `KaliciDepo::guncelle` → `kalici_dosya::kilit_altinda_guncelle`
//! adımlarından geçer: kilit → (Windows kurtarma) → metadata → geçici yazım +
//! `sync_all` → rename/`ReplaceFileW` → (Windows yedek silme) → klasör
//! eşzamanlama. Her adım noktasında depo klasörünün o anki görüntüsü
//! alınır; bu, sürecin tam o sistem çağrısından önce öldüğü diskin
//! kullanıcı alanından görünen hâlidir (çekirdek önbelleği kaybı kapsam
//! dışıdır; içerik rename'den önce `sync_all` ile kalıcıdır). Her görüntüde
//! yeni süreç depoyu açar: hedef tam eski ya da tam yeni bayttır, yarım
//! değildir; oturum görünürlüğü buna uyar ve depo yazılabilir kalır.

use super::*;
use crate::kalici_dosya::test_kancasi::{kur, temizle, Enjeksiyon};
use crate::kalici_dosya::YazimAdimi as Adim;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const ISTEMCI: &str = "198.51.100.7";

/// Platformun web deposu commit'inde geçtiği adım noktaları, sırasıyla.
fn commit_adimlari() -> Vec<Adim> {
    if cfg!(windows) {
        vec![
            Adim::Kilit,
            Adim::Kurtarma,
            Adim::Metadata,
            Adim::GeciciYazim,
            Adim::ReplaceFileW,
            Adim::YedekSilme,
            Adim::KlasorEszamanlama,
        ]
    } else {
        vec![
            Adim::Kilit,
            Adim::Metadata,
            Adim::GeciciYazim,
            Adim::Rename,
            Adim::KlasorEszamanlama,
        ]
    }
}

/// Kesit hedef değiştirme sistem çağrısından sonra mı (yeni bayt görünür)?
fn yayim_sonrasi(adim: Adim) -> bool {
    matches!(adim, Adim::YedekSilme | Adim::KlasorEszamanlama)
}

fn depo_klasoru(yol: &Path) -> &Path {
    yol.parent().expect("depo dosyasının klasörü olmalı")
}

/// Kilit dosyası dışındaki normal dosyaların görüntüsü (çökmede işletim
/// sistemi kilidi bırakır; kilit dosyasının içeriği anlamsızdır).
fn goruntu_al(kaynak: &Path, hedef: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(hedef)?;
    for girdi in std::fs::read_dir(kaynak)? {
        let girdi = girdi?;
        let ad = girdi.file_name();
        if !girdi.file_type()?.is_file()
            || ad == crate::kaynak_sinirlari::degistirme_penceresi::YAZMA_KILIDI_ADI
        {
            continue;
        }
        std::fs::write(hedef.join(&ad), std::fs::read(girdi.path())?)?;
    }
    Ok(())
}

fn gecici_dosyalar(klasor: &Path) -> Vec<PathBuf> {
    let mut yollar: Vec<_> = std::fs::read_dir(klasor)
        .unwrap()
        .filter_map(Result::ok)
        .map(|girdi| girdi.path())
        .filter(|yol| {
            let ad = yol.file_name().unwrap().to_string_lossy();
            ad.contains("zee-gecici") && !ad.ends_with(".zee-yedek")
        })
        .collect();
    yollar.sort();
    yollar
}

/// Senaryo: önce kalıcı bir oturum, sonra ölçülen istek (`ac` true ise yeni
/// giriş, değilse mevcut oturumdan çıkış). Ölçülen commit'in eski/yeni
/// baytlarını, belirteci ve oturumun commit sonrası görünür olup olmadığını
/// döndürür.
struct Senaryo {
    yol: PathBuf,
    belirtec: String,
    yeni_gorunur: bool,
}

fn hazirla(ad: &str, ac: bool) -> (Senaryo, WebGuvenligi) {
    let yol = gecici_depo_yolu(ad);
    let mut ilk = WebGuvenligi::kalici(&yol);
    ilk.istegi_baslat_kimlikle(None, ISTEMCI, 0).unwrap();
    let mevcut = ilk
        .oturum_ac("Zeynep".into(), "okur".into(), 0, || Ok("kalici-1".into()))
        .unwrap();
    ilk.istegi_tamamla(0).unwrap();

    let mut olculen = WebGuvenligi::kalici(&yol);
    let belirtec = if ac {
        olculen.istegi_baslat_kimlikle(None, ISTEMCI, 1).unwrap();
        let mut sira = 0;
        olculen
            .oturum_ac("Eliz".into(), "yönetici".into(), 1, || {
                sira += 1;
                Ok(format!("yeni-{sira}"))
            })
            .unwrap()
            .belirtec
    } else {
        olculen
            .istegi_baslat_kimlikle(Some(&mevcut.belirtec), ISTEMCI, 1)
            .unwrap();
        olculen.oturum_kapat();
        mevcut.belirtec
    };
    (
        Senaryo {
            yol,
            belirtec,
            yeni_gorunur: ac,
        },
        olculen,
    )
}

/// Yeniden başlatılan süreç `yol`daki depoyu açar; belirtecin oturumlu olup
/// olmadığını döndürür ve deponun yazılabilir kaldığını sınar.
fn yeniden_baslat(yol: &Path, belirtec: &str) -> Result<bool, String> {
    let mut yeni = WebGuvenligi::kalici(yol);
    yeni.istegi_baslat_kimlikle(Some(belirtec), ISTEMCI, 2)?;
    let gorunur = yeni.denetle(&RotaErisimi::Oturumlu, None, false, 2).is_ok();
    let mut yazar = WebGuvenligi::kalici(yol);
    yazar.istegi_baslat_kimlikle(None, ISTEMCI, 3)?;
    yazar.oturum_ac(
        "Sonraki".into(),
        "okur".into(),
        3,
        || Ok("sonraki-1".into()),
    )?;
    yazar.istegi_tamamla(3)?;
    let mut denetci = WebGuvenligi::kalici(yol);
    denetci.istegi_baslat_kimlikle(Some("sonraki-1"), ISTEMCI, 4)?;
    if denetci
        .denetle(&RotaErisimi::Oturumlu, None, false, 4)
        .is_err()
    {
        return Err("yeniden başlatma sonrası yeni commit görünmedi".into());
    }
    Ok(gorunur)
}

/// Görüntüyü doğrular: hedef tam eski ya da tam yeni bayttır ve beklenen
/// taraftadır; yeniden başlatma oturum görünürlüğünü buna göre verir.
/// `hedef_yok` Windows yarım `ReplaceFileW` görüntüsüdür: hedef yalnız
/// yedektedir ve yeniden başlatmadaki ilk kilit alımı onu geri koyar.
fn goruntuyu_dogrula(
    etiket: &str,
    klasor: &Path,
    senaryo: &Senaryo,
    (eski, yeni): (&[u8], &[u8]),
    yeni_beklenir: bool,
    hedef_yok: bool,
) {
    let hedef = klasor.join(senaryo.yol.file_name().unwrap());
    if hedef_yok {
        assert!(!hedef.exists(), "{etiket}: hedef yedekte olmalı");
    } else {
        let baytlar = std::fs::read(&hedef).unwrap();
        assert!(
            baytlar == eski || baytlar == yeni,
            "{etiket}: hedef yarım ({} bayt)",
            baytlar.len()
        );
        assert_eq!(baytlar == yeni, yeni_beklenir, "{etiket}: yanlış taraf");
    }
    let gorunur = yeniden_baslat(&hedef, &senaryo.belirtec)
        .unwrap_or_else(|hata| panic!("{etiket}: yeniden başlatma: {hata}"));
    assert_eq!(
        gorunur,
        senaryo.yeni_gorunur == yeni_beklenir,
        "{etiket}: oturum görünürlüğü"
    );
}

fn kesitleri_sina(ad: &str, ac: bool) {
    let (senaryo, mut olculen) = hazirla(ad, ac);
    let klasor = depo_klasoru(&senaryo.yol).to_path_buf();
    let eski = std::fs::read(&senaryo.yol).unwrap();
    let kok = klasor.parent().unwrap().join("kesitler");
    let gunluk = Rc::new(RefCell::new(Vec::<String>::new()));
    for (sira, adim) in commit_adimlari().into_iter().enumerate() {
        let (kaynak, hedef, gunluk) = (
            klasor.clone(),
            kok.join(format!("{sira}-{adim:?}")),
            Rc::clone(&gunluk),
        );
        kur(
            adim,
            Enjeksiyon::Eylem(Box::new(move || {
                gunluk.borrow_mut().push(format!("{adim:?}"));
                goruntu_al(&kaynak, &hedef)
            })),
        );
    }
    let commit = olculen.istegi_tamamla(1);
    gunluk.borrow_mut().push("commit-dondu".into());
    temizle();
    commit.expect("kesitsiz commit başarılı");
    let yeni = std::fs::read(&senaryo.yol).unwrap();
    assert_ne!(eski, yeni);

    // Yanıt kalıcılıktan önce gönderilemez: commit, klasör eşzamanlaması
    // dahil her adım bittikten sonra döner; main.rs yanıtı ancak Ok'ta yazar.
    let mut beklenen: Vec<String> = commit_adimlari()
        .into_iter()
        .map(|adim| format!("{adim:?}"))
        .collect();
    beklenen.push("commit-dondu".into());
    assert_eq!(*gunluk.borrow(), beklenen, "{ad}: adım sırası");

    for (sira, adim) in commit_adimlari().into_iter().enumerate() {
        let goruntu = kok.join(format!("{sira}-{adim:?}"));
        let etiket = format!("{ad}/{adim:?}");
        let taraflar = (eski.as_slice(), yeni.as_slice());
        // Türevler, yeniden başlatma görüntüye yeni commit yazmadan önce alınır.
        let tohum = kok.join(format!("{sira}-{adim:?}-tohum"));
        goruntu_al(&goruntu, &tohum).unwrap();
        goruntuyu_dogrula(
            &etiket,
            &goruntu,
            &senaryo,
            taraflar,
            yayim_sonrasi(adim),
            false,
        );

        // Geçici yazım ortasında ölüm: yayımlanmamış geçici dosya her boyda
        // yırtık kalabilir; hedef ve kurtarma ondan etkilenmez.
        if matches!(adim, Adim::Rename | Adim::ReplaceFileW) {
            let gecici = gecici_dosyalar(&tohum);
            assert_eq!(gecici.len(), 1, "{etiket}: tek geçici beklenir");
            let tam = std::fs::read(&gecici[0]).unwrap();
            assert_eq!(tam, yeni, "{etiket}: geçici fsync'li yeni bütün");
            for kesim in [0, 1, tam.len() / 2, tam.len() - 1] {
                let yirtik = kok.join(format!("{sira}-{adim:?}-yirtik-{kesim}"));
                goruntu_al(&tohum, &yirtik).unwrap();
                std::fs::write(gecici_dosyalar(&yirtik)[0].as_path(), &tam[..kesim]).unwrap();
                let etiket = format!("{etiket}/yirtik-{kesim}");
                goruntuyu_dogrula(&etiket, &yirtik, &senaryo, taraflar, false, false);
            }
        }

        // Windows: ReplaceFileW eski hedefi yedeğe taşıdıktan sonra ölüm.
        // Hedef yoktur; ilk kilit alımı eski bütünü yedekten geri koyar.
        if cfg!(windows) && adim == Adim::ReplaceFileW {
            let yarim = kok.join(format!("{sira}-{adim:?}-yarim"));
            goruntu_al(&tohum, &yarim).unwrap();
            let hedef = yarim.join(senaryo.yol.file_name().unwrap());
            let gecici = gecici_dosyalar(&yarim).remove(0);
            let mut yedek = gecici.into_os_string();
            yedek.push(".zee-yedek");
            std::fs::rename(&hedef, &yedek).unwrap();
            let etiket = format!("{etiket}/yarim");
            goruntuyu_dogrula(&etiket, &yarim, &senaryo, taraflar, false, true);
        }
    }
    gecici_depoyu_sil(&senaryo.yol);
}

#[test]
fn giris_commiti_her_cokme_kesitinde_eski_ya_da_yenidir() {
    kesitleri_sina("kesit-giris", true);
}

#[test]
fn cikis_commiti_her_cokme_kesitinde_eski_ya_da_yenidir() {
    kesitleri_sina("kesit-cikis", false);
}

/// Her adımda sistem çağrısı hatası: commit `Err` döner (main.rs 503 yazar,
/// belirteç/çerez gitmez); hedef değiştirmeden önceki adımda eski bütün
/// korunur. Klasör eşzamanlaması rename'den sonradır: hata yine `Err`
/// olur, hedef yenidir ama istemciye başarı bildirilmez (giriş için
/// belirteci kimsenin bilmediği öksüz oturum, çıkış için fail-closed iptal).
#[test]
fn adim_hatasinda_commit_basarili_donmez_ve_depo_kurtarilir() {
    for adim in commit_adimlari() {
        if adim == Adim::YedekSilme {
            continue; // K-351: en iyi çaba; hatası yazımı düşürmez.
        }
        for ac in [true, false] {
            let (senaryo, mut olculen) = hazirla(&format!("hata-{adim:?}-{ac}"), ac);
            let eski = std::fs::read(&senaryo.yol).unwrap();
            kur(
                adim,
                Enjeksiyon::OsHatasi(if cfg!(windows) { 5 } else { 13 }),
            );
            let sonuc = olculen.istegi_tamamla(1);
            temizle();
            let Err(hata) = sonuc else {
                panic!("{adim:?}: enjekte hata commit'i düşürmeli");
            };
            assert!(hata.contains("kalıcı web deposu"), "{adim:?}: {hata}");
            olculen.istegi_geri_al();
            let baytlar = std::fs::read(&senaryo.yol).unwrap();
            let yayimlandi = adim == Adim::KlasorEszamanlama;
            assert_eq!(baytlar == eski, !yayimlandi, "{adim:?}/{ac}");
            assert!(gecici_dosyalar(depo_klasoru(&senaryo.yol)).is_empty());
            let gorunur = yeniden_baslat(&senaryo.yol, &senaryo.belirtec).unwrap();
            assert_eq!(gorunur, ac == yayimlandi, "{adim:?}/{ac}");
            gecici_depoyu_sil(&senaryo.yol);
        }
    }
}
