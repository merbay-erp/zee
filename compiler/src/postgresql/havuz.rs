//! Production PostgreSQL TLS ve bounded connection-pool sahipliği.

use super::{hata, VeritabaniBildirimi, VeritabaniHatasi, VeritabaniHedefi};
use native_tls::{Certificate, Protocol, TlsConnector};
use postgres::config::{Host, SslMode};
use postgres::Config;
use postgres_native_tls::MakeTlsConnector;
use r2d2::ManageConnection;
use r2d2_postgres::PostgresConnectionManager;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const BAGLANTI_SINIRI: u32 = 4;
pub const CHECKOUT_ZAMAN_ASIMI: Duration = Duration::from_secs(2);
pub const BOS_BAGLANTI_OMRU: Duration = Duration::from_secs(30);
pub const BAGLANTI_AZAMI_OMRU: Duration = Duration::from_secs(300);
const AZAMI_CA_PEM_BAYTI: usize = 256 * 1024;
const AZAMI_HATA_AYRINTISI_KARAKTERI: usize = 512;
const AZAMI_HATA_NEDENI: usize = 4;

type Yonetici = PostgresConnectionManager<MakeTlsConnector>;
pub(super) struct PostgresqlHavuzu {
    ic: r2d2::Pool<Yonetici>,
    son_hata: Arc<Mutex<Option<String>>>,
}
pub(super) type PostgresqlKirasi = r2d2::PooledConnection<Yonetici>;

#[derive(Debug)]
struct HavuzHataKaydi(Arc<Mutex<Option<String>>>);

impl r2d2::HandleError<postgres::Error> for HavuzHataKaydi {
    fn handle_error(&self, hata: postgres::Error) {
        use std::error::Error;
        let mut zincir = vec![hata.to_string()];
        let mut neden = hata.source();
        while let Some(guncel) = neden.filter(|_| zincir.len() < AZAMI_HATA_NEDENI) {
            zincir.push(guncel.to_string());
            neden = guncel.source();
        }
        let ayrinti = hata_ayrintisini_sinirla(zincir);
        if let Ok(mut hedef) = self.0.lock() {
            *hedef = Some(ayrinti);
        }
    }
}

fn hata_ayrintisini_sinirla(zincir: Vec<String>) -> String {
    zincir
        .into_iter()
        .take(AZAMI_HATA_NEDENI)
        .collect::<Vec<_>>()
        .join(": ")
        .chars()
        .take(AZAMI_HATA_AYRINTISI_KARAKTERI)
        .collect()
}

pub(super) fn kur(bildirim: &VeritabaniBildirimi) -> Result<PostgresqlHavuzu, VeritabaniHatasi> {
    let baglanti = std::env::var(&bildirim.baglanti_degiskeni).map_err(|_| {
        hata(format!(
            "{} ortam değişkeninde PostgreSQL bağlantısı bulunamadı",
            bildirim.baglanti_degiskeni
        ))
    })?;
    let mut ayar = Config::from_str(&baglanti)
        .map_err(|h| hata(format!("PostgreSQL bağlantı ayarı geçersiz: {}", h)))?;
    hedefi_denetle(&ayar, &bildirim.hedef)?;
    ayar.connect_timeout(Duration::from_secs(5));
    let tls = tls_baglayicisi(&ayar, bildirim)?;
    let yonetici = PostgresConnectionManager::new(ayar, tls);
    let son_hata = Arc::new(Mutex::new(None));
    let ic = havuz_kurucusu()
        .error_handler(Box::new(HavuzHataKaydi(Arc::clone(&son_hata))))
        .build(yonetici)
        .map_err(|neden| hata(format!("PostgreSQL bağlantı havuzu kurulamadı: {}", neden)))?;
    Ok(PostgresqlHavuzu { ic, son_hata })
}

fn havuz_kurucusu<M: ManageConnection>() -> r2d2::Builder<M> {
    r2d2::Pool::builder()
        .max_size(BAGLANTI_SINIRI)
        .min_idle(Some(0))
        .test_on_check_out(true)
        .connection_timeout(CHECKOUT_ZAMAN_ASIMI)
        .idle_timeout(Some(BOS_BAGLANTI_OMRU))
        .max_lifetime(Some(BAGLANTI_AZAMI_OMRU))
}

pub(super) fn al(havuz: &PostgresqlHavuzu) -> Result<PostgresqlKirasi, VeritabaniHatasi> {
    if let Ok(mut son_hata) = havuz.son_hata.lock() {
        *son_hata = None;
    }
    havuz.ic.get_timeout(CHECKOUT_ZAMAN_ASIMI).map_err(|neden| {
        let ayrinti = havuz
            .son_hata
            .lock()
            .ok()
            .and_then(|hata| hata.clone())
            .unwrap_or_else(|| neden.to_string());
        hata(format!(
            "PostgreSQL bağlantı havuzu 2 saniye içinde bağlantı veremedi: {}",
            ayrinti
        ))
    })
}

fn tls_baglayicisi(
    ayar: &Config,
    bildirim: &VeritabaniBildirimi,
) -> Result<MakeTlsConnector, VeritabaniHatasi> {
    let mut kurucu = TlsConnector::builder();
    match ayar.get_ssl_mode() {
        SslMode::Disable => {
            if !bildirim.hedef.loopback_mi() {
                return Err(hata(
                    "sslmode=disable yalnız loopback PostgreSQL hedefinde kullanılabilir",
                ));
            }
        }
        SslMode::Require => {
            let degisken = format!("{}_TLS_CA_PEM", bildirim.baglanti_degiskeni);
            let pem = std::env::var(&degisken).map_err(|_| {
                hata(format!(
                    "TLS PostgreSQL bağlantısı sabit kök sertifikayı {} ortam değişkeninde ister",
                    degisken
                ))
            })?;
            if pem.is_empty() || pem.len() > AZAMI_CA_PEM_BAYTI {
                return Err(hata(
                    "PostgreSQL TLS kök sertifikası boş olamaz ve 256 KiB sınırını aşamaz",
                ));
            }
            let sertifika = Certificate::from_pem(pem.as_bytes())
                .map_err(|_| hata("PostgreSQL TLS kök sertifikası geçerli PEM değil"))?;
            kurucu.disable_built_in_roots(true);
            kurucu.add_root_certificate(sertifika);
            kurucu.min_protocol_version(Some(Protocol::Tlsv12));
        }
        _ => {
            return Err(hata(
                "PostgreSQL sslmode açıkça disable veya require olmalı; prefer kabul edilmez",
            ));
        }
    }
    let baglayici = kurucu
        .build()
        .map_err(|_| hata("PostgreSQL TLS doğrulayıcısı kurulamadı"))?;
    Ok(MakeTlsConnector::new(baglayici))
}

pub(super) fn hedefi_denetle(
    ayar: &Config,
    hedef: &VeritabaniHedefi,
) -> Result<(), VeritabaniHatasi> {
    let [Host::Tcp(konak)] = ayar.get_hosts() else {
        return Err(hata("PostgreSQL bağlantısı tek TCP hostu taşımalı"));
    };
    let kapi = ayar.get_ports().first().copied().unwrap_or(5432);
    let veritabani = ayar.get_dbname().unwrap_or("");
    if konak != &hedef.konak || kapi != hedef.kapi || veritabani != hedef.veritabani {
        return Err(super::verili_hata(
            "PostgreSQL bağlantısı proje bildirimindeki secretsiz hedefle eşleşmiyor",
            vec![("beklenen_hedef".into(), hedef.yazimi())],
        ));
    }
    if !ayar.get_hostaddrs().is_empty() {
        return Err(hata(
            "PostgreSQL hostaddr kullanılamaz; TLS hostname doğrulaması atlanamaz",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::io;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[derive(Clone)]
    struct SahteYonetici {
        sonraki: Arc<AtomicU32>,
        acik: Arc<AtomicU32>,
    }

    struct SahteBaglanti {
        kimlik: u32,
        bozuk: bool,
        acik: Arc<AtomicU32>,
    }

    impl Drop for SahteBaglanti {
        fn drop(&mut self) {
            self.acik.fetch_sub(1, Ordering::SeqCst);
        }
    }

    impl ManageConnection for SahteYonetici {
        type Connection = SahteBaglanti;
        type Error = io::Error;

        fn connect(&self) -> Result<Self::Connection, Self::Error> {
            self.acik.fetch_add(1, Ordering::SeqCst);
            Ok(SahteBaglanti {
                kimlik: self.sonraki.fetch_add(1, Ordering::SeqCst),
                bozuk: false,
                acik: Arc::clone(&self.acik),
            })
        }

        fn is_valid(&self, baglanti: &mut Self::Connection) -> Result<(), Self::Error> {
            if baglanti.bozuk {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "stale"))
            } else {
                Ok(())
            }
        }

        fn has_broken(&self, baglanti: &mut Self::Connection) -> bool {
            baglanti.bozuk
        }
    }

    fn sahte_havuz() -> (r2d2::Pool<SahteYonetici>, Arc<AtomicU32>) {
        let acik = Arc::new(AtomicU32::new(0));
        let yonetici = SahteYonetici {
            sonraki: Arc::new(AtomicU32::new(1)),
            acik: Arc::clone(&acik),
        };
        (havuz_kurucusu().build(yonetici).unwrap(), acik)
    }

    #[test]
    fn havuz_siniri_exhaustion_recovery_ve_stale_atimini_korur() {
        let (havuz, _) = sahte_havuz();
        let mut kiralar = (0..BAGLANTI_SINIRI)
            .map(|_| havuz.get().unwrap())
            .collect::<Vec<_>>();
        assert!(havuz.get_timeout(Duration::from_millis(20)).is_err());

        let ilk_kimlik = kiralar[0].kimlik;
        kiralar[0].bozuk = true;
        drop(kiralar.swap_remove(0));
        let yeni = havuz.get().unwrap();
        assert_ne!(yeni.kimlik, ilk_kimlik, "bozuk bağlantı havuza dönmemeli");

        drop(yeni);
        drop(kiralar);
        assert_eq!(havuz.state().idle_connections, BAGLANTI_SINIRI);
    }

    #[test]
    fn son_kira_birakilinca_kontrollu_kapanis_baglantiyi_kapatir() {
        let (havuz, acik) = sahte_havuz();
        let kira = havuz.get().unwrap();
        assert_eq!(acik.load(Ordering::SeqCst), 1);
        drop(havuz);
        assert_eq!(
            acik.load(Ordering::SeqCst),
            1,
            "aktif kira erken kapatılmamalı"
        );
        drop(kira);
        // K-182: son kiranın kapanışı r2d2'nin iş parçacığında tamamlanır;
        // Windows'ta hemen görünmedi. Sonuç kesin ama zamanı değil: en çok 2 sn
        // beklenir, süre dolarsa hâlâ açık bağlantı hatadır.
        let son = std::time::Instant::now() + Duration::from_secs(2);
        while acik.load(Ordering::SeqCst) != 0 && std::time::Instant::now() < son {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(acik.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn tls_profili_plaintext_uzak_hedefi_preferi_ve_koksuz_requirei_reddeder() {
        let uzak = VeritabaniBildirimi {
            hedef: VeritabaniHedefi::ayristir("postgresql://db.example.internal:5432/uygulama")
                .unwrap(),
            baglanti_degiskeni: "ZEE_TESTTE_OLMAYAN_DATABASE_URL".into(),
            gocler: "göçler".into(),
        };
        let plaintext = Config::from_str(
            "postgresql://kullanici@db.example.internal:5432/uygulama?sslmode=disable",
        )
        .unwrap();
        assert!(tls_baglayicisi(&plaintext, &uzak).is_err());

        let loopback = VeritabaniBildirimi {
            hedef: VeritabaniHedefi::ayristir("postgresql://127.0.0.1:5432/uygulama").unwrap(),
            ..uzak.clone()
        };
        let prefer = Config::from_str("postgresql://kullanici@127.0.0.1:5432/uygulama").unwrap();
        assert!(tls_baglayicisi(&prefer, &loopback).is_err());

        let require = Config::from_str(
            "postgresql://kullanici@db.example.internal:5432/uygulama?sslmode=require",
        )
        .unwrap();
        assert!(tls_baglayicisi(&require, &uzak).is_err());
    }

    #[test]
    fn havuz_hata_ayrintisi_neden_ve_karakter_butcesini_asmaz() {
        let ayrinti = hata_ayrintisini_sinirla(vec!["x".repeat(600); 8]);
        assert_eq!(ayrinti.chars().count(), AZAMI_HATA_AYRINTISI_KARAKTERI);
        assert!(!ayrinti.contains(": "));

        let dort = hata_ayrintisini_sinirla((1..=8).map(|s| s.to_string()).collect());
        assert_eq!(dort, "1: 2: 3: 4");
    }

    #[test]
    fn saha_profili_gercek_tls_exhaustion_ve_stale_recoveryyi_olcer() {
        let Ok(hedef) = std::env::var("ZEE_POSTGRES_SAHA_HEDEFI") else {
            // Hermetik CI yukarıdaki sahte manager ile aynı pool invariants'ını
            // zorlar. Bu dal yalnız açık gerçek PostgreSQL saha koşusunda açılır.
            return;
        };
        let bildirim = VeritabaniBildirimi {
            hedef: VeritabaniHedefi::ayristir(&hedef).expect("saha hedefi geçerli olmalı"),
            baglanti_degiskeni: "ZEE_POSTGRES_SAHA_URL".into(),
            gocler: "göçler".into(),
        };
        let havuz = kur(&bildirim).expect("gerçek TLS havuzu kurulmalı");
        let mut kiralar = (0..BAGLANTI_SINIRI)
            .map(|_| havuz.ic.get().expect("saha bağlantısı alınmalı"))
            .collect::<Vec<_>>();
        let pidler = kiralar
            .iter_mut()
            .map(|istemci| {
                istemci
                    .query_one("SELECT pg_backend_pid()", &[])
                    .expect("backend pid")
                    .get::<_, i32>(0)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            pidler
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
        assert!(havuz.ic.get_timeout(Duration::from_millis(100)).is_err());

        let stale_pid = pidler[1];
        let sonlandirildi: bool = kiralar[0]
            .query_one("SELECT pg_terminate_backend($1)", &[&stale_pid])
            .expect("backend sonlandırma sorgusu")
            .get(0);
        assert!(sonlandirildi);
        assert!(kiralar[1].simple_query("SELECT 1").is_err());
        drop(kiralar.swap_remove(1));
        let mut yeni = havuz.ic.get().expect("stale yerine yeni bağlantı");
        let yeni_pid: i32 = yeni
            .query_one("SELECT pg_backend_pid()", &[])
            .expect("yeni backend pid")
            .get(0);
        assert_ne!(yeni_pid, stale_pid);
        drop(yeni);
        drop(kiralar);

        if std::env::var_os("ZEE_POSTGRES_SAHA_UZUN_SOAK").is_some() {
            let mut eski = havuz.ic.get().expect("uzun soak bağlantısı");
            let eski_pid: i32 = eski
                .query_one("SELECT pg_backend_pid()", &[])
                .expect("uzun soak başlangıç pid")
                .get(0);
            std::thread::sleep(BAGLANTI_AZAMI_OMRU + Duration::from_secs(2));
            eski.simple_query("SELECT 1")
                .expect("aktif lease azami ömürde zorla kesilmemeli");
            drop(eski);
            // r2d2 azami ömrü aktif kirayı keserek değil, sonraki bakım
            // çevriminde havuzdan emekli ederek uygular.
            std::thread::sleep(BOS_BAGLANTI_OMRU + Duration::from_secs(2));
            let mut yenilenen = havuz.ic.get().expect("ömrü dolan bağlantı yenilenmeli");
            let yenilenen_pid: i32 = yenilenen
                .query_one("SELECT pg_backend_pid()", &[])
                .expect("uzun soak yenilenen pid")
                .get(0);
            assert_ne!(yenilenen_pid, eski_pid);
        }
    }
}
