//! K-361: registry istemcisi kalıcı doğrulama durumunu (`durum-v1.json`)
//! kilit dışında okuyup zinciri ağdan doğruladıktan sonra kilit altında tek
//! bir karşılaştırmayla (CAS) yayımlıyordu. Aynı projede eşzamanlı iki
//! çevrimiçi çözüm (ör. iki `dil` komutu) aynı başlangıç durumunu okuyunca
//! ikinci yazım, aynı zinciri getirse bile, `P016 … başka bir süreççe
//! değiştirildi` ile düşüyordu.
//!
//! Harness yarışı uyku kullanmadan kurar: iki gerçek iş parçacığı aynı cache
//! köküyle `paketi_guncelle_zamanla` çağırır; taşıyıcı ilk istekte (ikisi de
//! durumu okuduktan sonra) ortak engelde buluşturur, ikinci istemci targets
//! isteğinde birincinin bütün güncellemeyi bitirmesini bekler. Senaryolar:
//! aynı zincir, ikinci istemcide ileri zincir, ikinci istemcide geride kalan
//! zincir ve aynı sürümde farklı içerik (çelişki). Çıktı: başarılı güncelleme
//! sayısı, son kalıcı targets sürümü, geride kalanın döndürdüğü sürüm ve
//! çelişkinin tanı kodu.
#![allow(dead_code)]

use dil::registry::{
    HedefMetadata, HedefPolitikasi, KokMetadata, MetadataDosyasi, RegistryAnahtari, RegistryHatasi,
    RegistryIstemcisi, RegistryTasiyici, RolYetkisi, SnapshotMetadata, TargetsMetadata,
    TimestampMetadata,
};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Barrier};

const SONA_ERME: &str = "2100-01-01T00:00:00Z";

fn hex(baytlar: &[u8]) -> String {
    baytlar.iter().map(|bayt| format!("{bayt:02x}")).collect()
}

fn ozet(baytlar: &[u8]) -> String {
    hex(&Sha256::digest(baytlar))
}

fn kimlik(anahtar: &SigningKey) -> String {
    format!("sha256:{}", ozet(&anahtar.verifying_key().to_bytes()))
}

fn zarf<T: serde::Serialize>(rol: &str, imzali: &T, anahtar: &SigningKey) -> Vec<u8> {
    let mut girdi = b"zee-registry-v1\0".to_vec();
    girdi.extend_from_slice(rol.as_bytes());
    girdi.push(0);
    girdi.extend_from_slice(&serde_json::to_vec(imzali).expect("imza girdisi"));
    // Alan sırası registry'nin kanonik zarf biçimidir (`imzali`, `imzalar`).
    #[derive(serde::Serialize)]
    struct Imza {
        anahtar_kimligi: String,
        ed25519: String,
    }
    #[derive(serde::Serialize)]
    struct Zarf<'a, T> {
        imzali: &'a T,
        imzalar: [Imza; 1],
    }
    let zarf = Zarf {
        imzali,
        imzalar: [Imza {
            anahtar_kimligi: kimlik(anahtar),
            ed25519: hex(&anahtar.sign(&girdi).to_bytes()),
        }],
    };
    let mut baytlar = serde_json::to_vec_pretty(&zarf).expect("zarf");
    baytlar.push(b'\n');
    baytlar
}

/// Tek anahtarın bütün rolleri imzaladığı en küçük root.
fn root(anahtar: &SigningKey) -> Vec<u8> {
    let kimlik = kimlik(anahtar);
    let roller = ["root", "snapshot", "targets", "timestamp"]
        .into_iter()
        .map(|rol| {
            (
                rol.to_string(),
                RolYetkisi {
                    anahtar_kimlikleri: vec![kimlik.clone()],
                    esik: 1,
                },
            )
        })
        .collect();
    let kok = KokMetadata {
        sema: "zee-registry-root-v1".into(),
        surum: 1,
        sona_erme: SONA_ERME.into(),
        tutarli_anlik: true,
        anahtarlar: BTreeMap::from([(
            kimlik.clone(),
            RegistryAnahtari {
                tur: "ed25519".into(),
                sema: "ed25519".into(),
                acik: hex(&anahtar.verifying_key().to_bytes()),
            },
        )]),
        roller,
    };
    zarf("root", &kok, anahtar)
}

struct Yayin {
    hedef: HedefMetadata,
    dosyalar: Vec<(String, Vec<u8>)>,
}

fn yayin(kok: &Path, kaynak: &Path) -> Yayin {
    let proje = kok.join("miras");
    std::fs::create_dir_all(&proje).expect("paket projesi");
    std::fs::write(
        proje.join("proje.dil"),
        "proje \"miras\" olsun\nsürüm \"1.2.3\" olsun\ngiriş \"paket.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::copy(kaynak, proje.join("paket.dil")).expect("paket kaynağı");
    let anahtar_yolu = kok.join("yayinci.anahtar");
    dil::artefakt_dogrulama::anahtar_uret(&anahtar_yolu).expect("yayıncı anahtarı");
    let yollar =
        dil::yayin::paketle_zamanla(&proje, &anahtar_yolu, &kok.join("yayin"), 1_700_000_000)
            .expect("yayın");
    let oku = |yol: &PathBuf| std::fs::read(yol).expect("yayın dosyası");
    let (arsiv, sbom, provenance, bildirim) = (
        oku(&yollar.paket),
        oku(&yollar.sbom),
        oku(&yollar.provenance),
        oku(&yollar.yayin),
    );
    let imzali = dil::artefakt_dogrulama::yayini_dogrula(&bildirim, &arsiv, &sbom, &provenance)
        .expect("yayın doğrulama");
    let yayin_dosyasi = dil::paket_modeli::YayinDosyasi {
        ad: yollar
            .yayin
            .file_name()
            .expect("ad")
            .to_string_lossy()
            .into_owned(),
        boyut: bildirim.len() as u64,
        sha256: ozet(&bildirim),
    };
    let hedef = HedefMetadata {
        paket: imzali.paket,
        surum: imzali.surum,
        morfoloji: imzali.morfoloji,
        yayinci_anahtar_kimligi: imzali.yayinci_anahtar_kimligi,
        yanked: false,
        duyurular: Vec::new(),
        arsiv: imzali.arsiv,
        sbom: imzali.sbom,
        provenance: imzali.provenance,
        yayin: yayin_dosyasi,
    };
    let dosyalar = [
        (&hedef.arsiv.sha256, arsiv),
        (&hedef.sbom.sha256, sbom),
        (&hedef.provenance.sha256, provenance),
        (&hedef.yayin.sha256, bildirim),
    ]
    .into_iter()
    .map(|(ozet, baytlar)| (format!("hedefler/sha256/{ozet}"), baytlar))
    .collect();
    Yayin { hedef, dosyalar }
}

/// `surum` sürümlü timestamp/snapshot/targets zinciri; `varyant` aynı
/// sürümde farklı targets içeriği (çelişki) üretir.
fn zincir(anahtar: &SigningKey, yayin: &Yayin, surum: u64, varyant: bool) -> Tasiyici {
    let targets = zarf(
        "targets",
        &TargetsMetadata {
            sema: "zee-registry-targets-v1".into(),
            surum,
            sona_erme: if varyant {
                "2100-01-01T00:00:01Z".into()
            } else {
                SONA_ERME.into()
            },
            hedefler: BTreeMap::from([("miras@1.2.3".into(), yayin.hedef.clone())]),
            duyurular: BTreeMap::new(),
        },
        anahtar,
    );
    let dosya = |baytlar: &[u8]| MetadataDosyasi {
        surum,
        boyut: baytlar.len() as u64,
        sha256: ozet(baytlar),
    };
    let snapshot = zarf(
        "snapshot",
        &SnapshotMetadata {
            sema: "zee-registry-snapshot-v1".into(),
            surum,
            sona_erme: SONA_ERME.into(),
            targets: dosya(&targets),
        },
        anahtar,
    );
    let timestamp = zarf(
        "timestamp",
        &TimestampMetadata {
            sema: "zee-registry-timestamp-v1".into(),
            surum,
            sona_erme: SONA_ERME.into(),
            snapshot: dosya(&snapshot),
        },
        anahtar,
    );
    let mut dosyalar: BTreeMap<String, Vec<u8>> = yayin.dosyalar.iter().cloned().collect();
    dosyalar.insert("metadata/timestamp.json".into(), timestamp);
    dosyalar.insert(format!("metadata/{surum}.snapshot.json"), snapshot);
    dosyalar.insert(format!("metadata/{surum}.targets.json"), targets);
    Tasiyici {
        dosyalar,
        engel: None,
        bekle: None,
    }
}

struct Tasiyici {
    dosyalar: BTreeMap<String, Vec<u8>>,
    engel: Option<Arc<Barrier>>,
    bekle: Option<mpsc::Receiver<()>>,
}

impl RegistryTasiyici for Tasiyici {
    fn getir(&mut self, yol: &str, azami: usize) -> Result<Option<Vec<u8>>, RegistryHatasi> {
        if let Some(engel) = self.engel.take() {
            engel.wait();
        }
        if yol.ends_with(".targets.json") {
            if let Some(bekle) = self.bekle.take() {
                bekle.recv().expect("birinci güncelleme bitti");
            }
        }
        Ok(self
            .dosyalar
            .get(yol)
            .filter(|baytlar| baytlar.len() <= azami)
            .cloned())
    }
}

type Sonuc = Result<u64, &'static str>;

/// İki istemci aynı boş cache'i okur; birinci yazdıktan sonra ikinci yazar.
/// Dönüş: (birinci, ikinci, son kalıcı targets sürümü).
fn yaris(cache: PathBuf, kok: &[u8], birinci: Tasiyici, ikinci: Tasiyici) -> (Sonuc, Sonuc, u64) {
    let istemci = RegistryIstemcisi::yeni(&cache, kok.to_vec(), format!("sha256:{}", ozet(kok)));
    let engel = Arc::new(Barrier::new(2));
    let (bitti, bekle) = mpsc::channel();
    let kos = |mut tasiyici: Tasiyici, bekle: Option<mpsc::Receiver<()>>| {
        let (istemci, engel) = (istemci.clone(), engel.clone());
        tasiyici.engel = Some(engel);
        tasiyici.bekle = bekle;
        std::thread::spawn(move || {
            istemci
                .paketi_guncelle_zamanla(
                    &mut tasiyici,
                    "miras",
                    "1.2.3",
                    HedefPolitikasi::default(),
                    1,
                )
                .map(|cikti| cikti.metadata_surumleri.targets)
                .map_err(|hata| {
                    eprintln!("K-361 harness: güncelleme düştü: {hata}");
                    hata.kod
                })
        })
    };
    let ikinci = kos(ikinci, Some(bekle));
    let birinci = kos(birinci, None).join().expect("birinci");
    // İkinci istemci targets isteğinden önce düştüyse alıcı yoktur.
    let _ = bitti.send(());
    let ikinci = ikinci.join().expect("ikinci");
    let durum: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cache.join("durum-v1.json")).expect("kalıcı durum"))
            .expect("durum JSON");
    let kalici = durum["metadata"]["targets"]
        .as_u64()
        .expect("targets sürümü");
    (birinci, ikinci, kalici)
}

fn yaz(sonuc: Sonuc) -> String {
    match sonuc {
        Ok(surum) => surum.to_string(),
        Err(kod) => kod.to_string(),
    }
}

pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    struct Temizle(PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            fn yazilabilir(yol: &Path) {
                if let Ok(ogeler) = std::fs::read_dir(yol) {
                    for oge in ogeler.flatten() {
                        yazilabilir(&oge.path());
                    }
                }
                if let Ok(meta) = std::fs::metadata(yol) {
                    let mut izin = meta.permissions();
                    #[allow(clippy::permissions_set_readonly_false)]
                    izin.set_readonly(false);
                    let _ = std::fs::set_permissions(yol, izin);
                }
            }
            yazilabilir(&self.0);
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let kok = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-k361-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    let _temizle = Temizle(kok.clone());
    let anahtar = SigningKey::from_bytes(&[61; 32]);
    let root = root(&anahtar);
    let yayin = yayin(&kok, kaynak);
    let cache = |ad: &str| kok.join(ad).join("cache");

    let (a, b, _) = yaris(
        cache("ayni"),
        &root,
        zincir(&anahtar, &yayin, 3, false),
        zincir(&anahtar, &yayin, 3, false),
    );
    let ayni_basarili = [a, b].iter().filter(|sonuc| sonuc.is_ok()).count();
    let (_, ileri, ileri_durum) = yaris(
        cache("ileri"),
        &root,
        zincir(&anahtar, &yayin, 3, false),
        zincir(&anahtar, &yayin, 4, false),
    );
    let (_, geri, geri_durum) = yaris(
        cache("geri"),
        &root,
        zincir(&anahtar, &yayin, 4, false),
        zincir(&anahtar, &yayin, 3, false),
    );
    let (_, celiski, celiski_durum) = yaris(
        cache("celiski"),
        &root,
        zincir(&anahtar, &yayin, 3, true),
        zincir(&anahtar, &yayin, 3, false),
    );
    (
        0,
        vec![
            format!("aynı zincir başarılı {ayni_basarili}"),
            format!("ileri {} durum {ileri_durum}", yaz(ileri)),
            format!("geri {} durum {geri_durum}", yaz(geri)),
            format!("çelişki {} durum {celiski_durum}", yaz(celiski)),
        ],
    )
}
