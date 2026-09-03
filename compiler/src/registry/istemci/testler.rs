use super::*;
use crate::artefakt_dogrulama::{anahtar_uret, yayini_dogrula};
use crate::yayin::paketle_zamanla;
use ed25519_dalek::{Signer, SigningKey};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

static GECICI_SAYACI: AtomicU64 = AtomicU64::new(0);

struct GeciciKlasor(PathBuf);

impl GeciciKlasor {
    fn yeni() -> Self {
        let sira = GECICI_SAYACI.fetch_add(1, Ordering::Relaxed);
        let yol = std::env::temp_dir().join(format!(
            "zee-registry-istemci-test-{}-{}",
            std::process::id(),
            sira
        ));
        std::fs::create_dir(&yol).expect("geçici klasör");
        Self(yol)
    }

    fn yol(&self) -> &Path {
        &self.0
    }
}

impl Drop for GeciciKlasor {
    fn drop(&mut self) {
        fn agaci_yazilabilir_yap(yol: &Path) {
            let Ok(metadata) = std::fs::symlink_metadata(yol) else {
                return;
            };
            if metadata.is_dir() {
                if let Ok(girdiler) = std::fs::read_dir(yol) {
                    for girdi in girdiler.flatten() {
                        agaci_yazilabilir_yap(&girdi.path());
                    }
                }
            }
            dosyayi_yazilabilir_yap(yol);
        }
        agaci_yazilabilir_yap(&self.0);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn dosyayi_yazilabilir_yap(yol: &Path) {
    let Ok(mut izin) = std::fs::metadata(yol).map(|metadata| metadata.permissions()) else {
        return;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        izin.set_mode(izin.mode() | 0o200);
    }
    #[cfg(not(unix))]
    izin.set_readonly(false);
    let _ = std::fs::set_permissions(yol, izin);
}

#[derive(Default)]
struct SahteTasiyici {
    dosyalar: BTreeMap<String, Vec<u8>>,
    istekler: Vec<String>,
}

impl RegistryTasiyici for SahteTasiyici {
    fn getir(
        &mut self,
        goreli_yol: &str,
        azami_bayt: usize,
    ) -> Result<Option<Vec<u8>>, RegistryHatasi> {
        self.istekler.push(goreli_yol.into());
        let sonuc = self.dosyalar.get(goreli_yol).cloned();
        if sonuc
            .as_ref()
            .is_some_and(|baytlar| baytlar.len() > azami_bayt)
        {
            return Err(RegistryHatasi::tasima("Sahte taşıma sınırı aştı."));
        }
        Ok(sonuc)
    }
}

struct Anahtarlar {
    root: SigningKey,
    targets: SigningKey,
    snapshot: SigningKey,
    timestamp: SigningKey,
}

fn anahtarlar() -> Anahtarlar {
    Anahtarlar {
        root: SigningKey::from_bytes(&[31; 32]),
        targets: SigningKey::from_bytes(&[32; 32]),
        snapshot: SigningKey::from_bytes(&[33; 32]),
        timestamp: SigningKey::from_bytes(&[34; 32]),
    }
}

fn yeni_anahtarlar() -> Anahtarlar {
    Anahtarlar {
        root: SigningKey::from_bytes(&[41; 32]),
        targets: SigningKey::from_bytes(&[42; 32]),
        snapshot: SigningKey::from_bytes(&[43; 32]),
        timestamp: SigningKey::from_bytes(&[44; 32]),
    }
}

fn kimlik(anahtar: &SigningKey) -> String {
    format!("sha256:{}", sha256_hex(&anahtar.verifying_key().to_bytes()))
}

fn zarf<T: Serialize + Clone>(rol: &str, imzali: T, anahtar: &SigningKey) -> Vec<u8> {
    coklu_zarf(rol, imzali, &[anahtar])
}

fn coklu_zarf<T: Serialize + Clone>(rol: &str, imzali: T, anahtarlar: &[&SigningKey]) -> Vec<u8> {
    let girdi = imza_girdisi(rol, &imzali).expect("imza girdisi");
    let mut imzalar = anahtarlar
        .iter()
        .map(|anahtar| RegistryImzasi {
            anahtar_kimligi: kimlik(anahtar),
            ed25519: hex_yaz(&anahtar.sign(&girdi).to_bytes()),
        })
        .collect::<Vec<_>>();
    imzalar.sort_by(|a, b| a.anahtar_kimligi.cmp(&b.anahtar_kimligi));
    let zarf = MetadataZarfi { imzali, imzalar };
    let mut baytlar = serde_json::to_vec_pretty(&zarf).expect("metadata JSON");
    baytlar.push(b'\n');
    baytlar
}

fn root(anahtarlar: &Anahtarlar) -> Vec<u8> {
    zarf("root", kok_metadata(1, anahtarlar), &anahtarlar.root)
}

fn kok_metadata(surum: u64, anahtarlar: &Anahtarlar) -> KokMetadata {
    let mut anahtar_haritasi = BTreeMap::new();
    let mut roller = BTreeMap::new();
    for (rol, anahtar) in [
        ("root", &anahtarlar.root),
        ("targets", &anahtarlar.targets),
        ("snapshot", &anahtarlar.snapshot),
        ("timestamp", &anahtarlar.timestamp),
    ] {
        let kimlik = kimlik(anahtar);
        anahtar_haritasi.insert(
            kimlik.clone(),
            RegistryAnahtari {
                tur: "ed25519".into(),
                sema: "ed25519".into(),
                acik: hex_yaz(&anahtar.verifying_key().to_bytes()),
            },
        );
        roller.insert(
            rol.into(),
            RolYetkisi {
                anahtar_kimlikleri: vec![kimlik],
                esik: 1,
            },
        );
    }
    KokMetadata {
        sema: "zee-registry-root-v1".into(),
        surum,
        sona_erme: "2100-01-01T00:00:00Z".into(),
        tutarli_anlik: true,
        anahtarlar: anahtar_haritasi,
        roller,
    }
}

struct YayinFixture {
    hedef: HedefMetadata,
    arsiv: Vec<u8>,
    sbom: Vec<u8>,
    provenance: Vec<u8>,
    yayin: Vec<u8>,
}

fn yayin_fixture(gecici: &GeciciKlasor) -> YayinFixture {
    let proje = gecici.yol().join("miras");
    let cikti = gecici.yol().join("yayin");
    std::fs::create_dir(&proje).expect("paket projesi");
    std::fs::write(
        proje.join("proje.dil"),
        "proje \"miras\" olsun\nsürüm \"1.2.3\" olsun\ngiriş \"paket.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::write(
        proje.join("paket.dil"),
        "işlem yedi ver\n    TamSayı döndürür\n    7 döndür\n",
    )
    .expect("paket kaynağı");
    let anahtar_yolu = gecici.yol().join("yayinci.anahtar");
    anahtar_uret(&anahtar_yolu).expect("yayıncı anahtarı");
    let yollar =
        paketle_zamanla(&proje, &anahtar_yolu, &cikti, 1_700_000_000).expect("yayın üretimi");
    let arsiv = std::fs::read(&yollar.paket).expect("arsiv");
    let sbom = std::fs::read(&yollar.sbom).expect("sbom");
    let provenance = std::fs::read(&yollar.provenance).expect("provenance");
    let yayin = std::fs::read(&yollar.yayin).expect("yayın");
    let imzali = yayini_dogrula(&yayin, &arsiv, &sbom, &provenance).expect("yayın doğrulama");
    let yayin_dosyasi = YayinDosyasi {
        ad: yollar
            .yayin
            .file_name()
            .expect("yayın adı")
            .to_string_lossy()
            .into_owned(),
        boyut: yayin.len() as u64,
        sha256: sha256_hex(&yayin),
    };
    YayinFixture {
        hedef: HedefMetadata {
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
        },
        arsiv,
        sbom,
        provenance,
        yayin,
    }
}

fn tasiyici(anahtarlar: &Anahtarlar, yayin: &YayinFixture, surum: u64) -> SahteTasiyici {
    tasiyici_duyurularla(anahtarlar, yayin, surum, &[])
}

fn tasiyici_duyurularla(
    anahtarlar: &Anahtarlar,
    yayin: &YayinFixture,
    surum: u64,
    kritik_kimlikleri: &[&str],
) -> SahteTasiyici {
    let mut hedefler = BTreeMap::new();
    let mut hedef = yayin.hedef.clone();
    hedef.duyurular = kritik_kimlikleri
        .iter()
        .map(|kimlik| (*kimlik).to_string())
        .collect();
    hedefler.insert("miras@1.2.3".into(), hedef);
    let duyurular = kritik_kimlikleri
        .iter()
        .map(|kimlik| {
            (
                (*kimlik).to_string(),
                DuyuruMetadata {
                    kimlik: (*kimlik).to_string(),
                    paket: "miras".into(),
                    etkilenen_surumler: vec!["1.2.3".into()],
                    onem: "kritik".into(),
                    duzeltilen_surum: None,
                    etkin: true,
                },
            )
        })
        .collect();
    let targets = zarf(
        "targets",
        TargetsMetadata {
            sema: "zee-registry-targets-v1".into(),
            surum,
            sona_erme: "2100-01-01T00:00:00Z".into(),
            hedefler,
            duyurular,
        },
        &anahtarlar.targets,
    );
    let snapshot = zarf(
        "snapshot",
        SnapshotMetadata {
            sema: "zee-registry-snapshot-v1".into(),
            surum,
            sona_erme: "2100-01-01T00:00:00Z".into(),
            targets: MetadataDosyasi {
                surum,
                boyut: targets.len() as u64,
                sha256: sha256_hex(&targets),
            },
        },
        &anahtarlar.snapshot,
    );
    let timestamp = zarf(
        "timestamp",
        TimestampMetadata {
            sema: "zee-registry-timestamp-v1".into(),
            surum,
            sona_erme: "2100-01-01T00:00:00Z".into(),
            snapshot: MetadataDosyasi {
                surum,
                boyut: snapshot.len() as u64,
                sha256: sha256_hex(&snapshot),
            },
        },
        &anahtarlar.timestamp,
    );
    let mut dosyalar = BTreeMap::from([
        ("metadata/timestamp.json".into(), timestamp),
        (format!("metadata/{}.snapshot.json", surum), snapshot),
        (format!("metadata/{}.targets.json", surum), targets),
    ]);
    for (dosya, baytlar) in [
        (&yayin.hedef.arsiv, &yayin.arsiv),
        (&yayin.hedef.sbom, &yayin.sbom),
        (&yayin.hedef.provenance, &yayin.provenance),
        (&yayin.hedef.yayin, &yayin.yayin),
    ] {
        dosyalar.insert(format!("hedefler/sha256/{}", dosya.sha256), baytlar.clone());
    }
    SahteTasiyici {
        dosyalar,
        istekler: Vec::new(),
    }
}

#[test]
fn cevrimici_zincir_dogrulanmadan_cache_ve_durum_yayimlanmaz() {
    let gecici = GeciciKlasor::yeni();
    let anahtarlar = anahtarlar();
    let root = root(&anahtarlar);
    let root_ozeti = format!("sha256:{}", sha256_hex(&root));
    let yayin = yayin_fixture(&gecici);
    let cache = gecici.yol().join("cache");
    let istemci = RegistryIstemcisi::yeni(&cache, root, root_ozeti);
    let mut tasiyici = tasiyici(&anahtarlar, &yayin, 1);
    let hedef_yolu = format!("hedefler/sha256/{}", yayin.hedef.arsiv.sha256);
    tasiyici
        .dosyalar
        .get_mut(&hedef_yolu)
        .expect("arşiv")
        .push(0);

    let hata = istemci
        .paketi_guncelle_zamanla(
            &mut tasiyici,
            "miras",
            "1.2.3",
            HedefPolitikasi::default(),
            1,
        )
        .expect_err("bozuk hedef reddedilmeli");
    assert!(matches!(hata.kod, "P014" | "P016"));
    assert!(!cache.join(DURUM_DOSYASI).exists());
    assert!(!cache.join("nesneler").exists());
}

#[test]
fn dogrulanmis_cache_offline_acilir_ve_bozuk_nesne_fail_closed_kalir() {
    let gecici = GeciciKlasor::yeni();
    let anahtarlar = anahtarlar();
    let root = root(&anahtarlar);
    let root_ozeti = format!("sha256:{}", sha256_hex(&root));
    let yayin = yayin_fixture(&gecici);
    let cache = gecici.yol().join("cache");
    let istemci = RegistryIstemcisi::yeni(&cache, root, root_ozeti);
    let mut tasiyici = tasiyici(&anahtarlar, &yayin, 3);

    let cevrimici = istemci
        .paketi_guncelle_zamanla(
            &mut tasiyici,
            "miras",
            "1.2.3",
            HedefPolitikasi::default(),
            1,
        )
        .expect("çevrimiçi doğrulama");
    assert_eq!(cevrimici.metadata_surumleri.targets, 3);
    assert!(!cevrimici.cevrimdisi);
    assert!(cevrimici.arsiv.is_file());
    assert!(std::fs::metadata(&cevrimici.arsiv)
        .expect("cache metadata")
        .permissions()
        .readonly());

    let offline = istemci
        .paketi_cevrimdisi_al("miras", "1.2.3", HedefPolitikasi::default())
        .expect("offline hit");
    assert!(offline.cevrimdisi);
    assert_eq!(offline.arsiv, cevrimici.arsiv);

    dosyayi_yazilabilir_yap(&offline.arsiv);
    std::fs::write(&offline.arsiv, b"bozuk").expect("cache bozma");
    let hata = istemci
        .paketi_cevrimdisi_al("miras", "1.2.3", HedefPolitikasi::default())
        .expect_err("bozuk cache reddedilmeli");
    assert_eq!(hata.kod, "P016");
    assert!(!hata.mesaj.is_empty());
}

#[test]
fn rollback_kalici_durumu_ve_offline_miss_sessizce_gecemez() {
    let gecici = GeciciKlasor::yeni();
    let anahtarlar = anahtarlar();
    let root = root(&anahtarlar);
    let root_ozeti = format!("sha256:{}", sha256_hex(&root));
    let yayin = yayin_fixture(&gecici);
    let cache = gecici.yol().join("cache");
    let istemci = RegistryIstemcisi::yeni(&cache, root.clone(), root_ozeti.clone());

    let miss = RegistryIstemcisi::yeni(gecici.yol().join("bos"), root, root_ozeti)
        .paketi_cevrimdisi_al("miras", "1.2.3", HedefPolitikasi::default())
        .expect_err("offline miss");
    assert_eq!(miss.kod, "P016");

    let mut ileri = tasiyici(&anahtarlar, &yayin, 4);
    istemci
        .paketi_guncelle_zamanla(&mut ileri, "miras", "1.2.3", HedefPolitikasi::default(), 1)
        .expect("ileri sürüm");
    let durum_yolu = cache.join(DURUM_DOSYASI);
    let onceki = std::fs::read(&durum_yolu).expect("kalıcı durum");
    let mut eski = tasiyici(&anahtarlar, &yayin, 3);
    let hata = istemci
        .paketi_guncelle_zamanla(&mut eski, "miras", "1.2.3", HedefPolitikasi::default(), 1)
        .expect_err("rollback");
    assert_eq!(hata.kod, "P013");
    assert!(hata.mesaj.contains("rollback"));
    assert_eq!(std::fs::read(durum_yolu).expect("korunan durum"), onceki);
}

#[test]
fn tasinan_cift_esikli_root_kalici_duruma_ve_offline_dogrulamaya_gecer() {
    let gecici = GeciciKlasor::yeni();
    let eski = anahtarlar();
    let yeni = yeni_anahtarlar();
    let root1 = root(&eski);
    let root_ozeti = format!("sha256:{}", sha256_hex(&root1));
    let root2 = coklu_zarf("root", kok_metadata(2, &yeni), &[&eski.root, &yeni.root]);
    let yayin = yayin_fixture(&gecici);
    let cache = gecici.yol().join("cache");
    let istemci = RegistryIstemcisi::yeni(&cache, root1, root_ozeti);
    let mut tasiyici = tasiyici(&yeni, &yayin, 5);
    tasiyici
        .dosyalar
        .insert("metadata/2.root.json".into(), root2);

    let sonuc = istemci
        .paketi_guncelle_zamanla(
            &mut tasiyici,
            "miras",
            "1.2.3",
            HedefPolitikasi::default(),
            1,
        )
        .expect("root rotasyonlu güncelleme");
    assert_eq!(sonuc.metadata_surumleri.root, 2);
    assert!(tasiyici
        .istekler
        .iter()
        .any(|yol| yol == "metadata/2.root.json"));
    assert!(istemci
        .paketi_cevrimdisi_al("miras", "1.2.3", HedefPolitikasi::default())
        .is_ok());
}

#[test]
fn gercek_tasiyici_yalniz_https_origin_ve_guvenli_statik_yol_kabul_eder() {
    assert!(HttpsRegistryTasiyici::yeni("http://registry.example").is_err());
    assert!(HttpsRegistryTasiyici::yeni("https://registry.example/yol").is_err());
    assert!(HttpsRegistryTasiyici::yeni("https://registry.example").is_ok());
    assert!(goreli_yolu_dogrula("metadata/1.targets.json").is_ok());
    assert!(goreli_yolu_dogrula("../root.json").is_err());
}

#[test]
fn exact_registry_paketi_proje_grafigi_kilit_ve_offline_derlemeye_baglanir() {
    let gecici = GeciciKlasor::yeni();
    let anahtarlar = anahtarlar();
    let root = root(&anahtarlar);
    let root_ozeti = format!("sha256:{}", sha256_hex(&root));
    let yayin = yayin_fixture(&gecici);
    let uygulama = gecici.yol().join("uygulama");
    std::fs::create_dir(&uygulama).expect("uygulama");
    std::fs::write(
        uygulama.join("proje.dil"),
        format!(
            concat!(
                "proje \"uygulama\" olsun\n",
                "sürüm \"0.1.0\" olsun\n",
                "giriş \"ana.dil\" olsun\n",
                "registry \"https://registry.example\" olsun\n",
                "registry_kök_sürümü \"1\" olsun\n",
                "registry_kök_özeti \"{}\" olsun\n",
                "uzak_bağımlılıklar \"miras@1.2.3\" listesi olsun\n"
            ),
            root_ozeti
        ),
    )
    .expect("uygulama bildirimi");
    std::fs::write(
        uygulama.join("ana.dil"),
        "miras paketini kullan\nyedi ver yaz\n",
    )
    .expect("uygulama kaynağı");

    let cache = uygulama
        .join(".zee")
        .join("registry")
        .join(root_ozeti.strip_prefix("sha256:").expect("önek"));
    let istemci = RegistryIstemcisi::yeni(&cache, root, &root_ozeti);
    let mut tasiyici = tasiyici(&anahtarlar, &yayin, 9);
    istemci
        .paketi_guncelle_zamanla(
            &mut tasiyici,
            "miras",
            "1.2.3",
            HedefPolitikasi::default(),
            1,
        )
        .expect("cache önceden doğrulanmalı");

    let grafik = crate::paket::ProjeGrafigi::cozumle(&uygulama)
        .expect("normal proje çözümü ağsız cache kullanmalı");
    let giris = grafik.ana_giris().expect("ana giriş");
    let mut yukleyici = |istek: crate::BirimIstegi<'_>| grafik.yukle(istek);
    crate::kaynagi_derle_kokenlerle(&giris.kaynak, Some(&giris.koken), &mut yukleyici)
        .expect("registry paketi kaynak yükleyicisinden derlenmeli");
    let kilit = grafik.kilit_metni();
    assert!(kilit.contains("kilit_sürümü 3"));
    assert!(kilit.contains("uzak \"miras\" \"1.2.3\""));
    assert!(kilit.contains(&root_ozeti));
    assert!(kilit.contains(&yayin.hedef.yayinci_anahtar_kimligi));
    assert!(kilit.contains(&yayin.hedef.arsiv.sha256));
    grafik.kilidi_yaz().expect("kilit");
    grafik.kilidi_denetle().expect("kilit doğrulama");
    let paket = grafik.paketler().pop().expect("registry paketi");
    assert_eq!(paket.ad, "miras");
    assert_eq!(
        paket.registry_kok_sha256.as_deref(),
        Some(root_ozeti.as_str())
    );

    let kurulum = uygulama
        .join(".zee/paketler/sha256")
        .join(&yayin.hedef.arsiv.sha256)
        .join("paket.dil");
    dosyayi_yazilabilir_yap(&kurulum);
    std::fs::write(&kurulum, "bozuk\n").expect("kurulum bozma");
    let hata = match crate::paket::ProjeGrafigi::cozumle(&uygulama) {
        Ok(_) => panic!("bozuk kurulum fail-closed kalmalı"),
        Err(hata) => hata,
    };
    assert_eq!(hata.tani.kod, "P016");

    std::fs::write(
        &kurulum,
        "işlem yedi ver\n    TamSayı döndürür\n    7 döndür\n",
    )
    .expect("kurulumu geri yükle");
    let mut bir_duyuru = tasiyici_duyurularla(&anahtarlar, &yayin, 10, &["ZEE-2026-1"]);
    istemci
        .paketi_guncelle_zamanla(
            &mut bir_duyuru,
            "miras",
            "1.2.3",
            HedefPolitikasi {
                yanked_kabul: false,
                kritik_duyuru_kabul: true,
            },
            2,
        )
        .expect("tek kritik duyurulu cache");
    let kabul = crate::paket::RegistryCozumPolitikasi::yeni(
        true,
        None,
        Some("geçiş tamamlanana dek".into()),
    )
    .expect("gerekçeli politika");
    crate::paket::ProjeGrafigi::cozumle_registry_ile(&uygulama, &kabul)
        .expect("açık kabul")
        .kilidi_yaz()
        .expect("gerekçeli kilit");

    let mut yeni_duyuru =
        tasiyici_duyurularla(&anahtarlar, &yayin, 11, &["ZEE-2026-1", "ZEE-2026-2"]);
    istemci
        .paketi_guncelle_zamanla(
            &mut yeni_duyuru,
            "miras",
            "1.2.3",
            HedefPolitikasi {
                yanked_kabul: false,
                kritik_duyuru_kabul: true,
            },
            3,
        )
        .expect("iki kritik duyurulu cache");
    let hata = match crate::paket::ProjeGrafigi::cozumle(&uygulama) {
        Ok(_) => panic!("yeni kritik duyuru eski gerekçeyi kullanmamalı"),
        Err(hata) => hata,
    };
    assert_eq!(hata.tani.kod, "P014");
    assert!(hata.tani.mesaj.contains("yeni açık gerekçe"));
}

fn hex_yaz(baytlar: &[u8]) -> String {
    const BASAMAK: &[u8; 16] = b"0123456789abcdef";
    let mut sonuc = String::with_capacity(baytlar.len() * 2);
    for bayt in baytlar {
        sonuc.push(BASAMAK[(bayt >> 4) as usize] as char);
        sonuc.push(BASAMAK[(bayt & 0x0f) as usize] as char);
    }
    sonuc
}
