//! K-163 PostgreSQL dogfood adaptörü.
//!
//! Loopback geliştirme açık `sslmode=disable`; production profili pinned CA,
//! hostname doğrulamalı `sslmode=require` ve sınırlı havuz kullanır. Değerler
//! SQL metnine eklenmez, extended-query protokolüne açık TEXT parametreleri
//! olarak bind edilir.

mod havuz;

use crate::veritabani_modeli::{VeritabaniBildirimi, VeritabaniHatasi, VeritabaniHedefi};
use postgres::types::{ToSql, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GocRaporu {
    pub uygulanan: usize,
    pub atlanan: usize,
}

pub struct PostgresqlOturumu {
    havuz: havuz::PostgresqlHavuzu,
    kiralik: Option<havuz::PostgresqlKirasi>,
    eylem_derinligi: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SorguSinifi {
    Okuma,
    Yazma,
    Commit,
}

impl PostgresqlOturumu {
    pub fn baglan(
        bildirim: &VeritabaniBildirimi,
        eylem_derinligi: usize,
    ) -> Result<Self, VeritabaniHatasi> {
        let havuz = havuz::kur(bildirim)?;
        // Yapılandırma hatası ilk sorguya kadar saklanmasın; başlangıçta bir
        // gerçek bağlantı checkout/health kontrolünden geçsin.
        drop(havuz::al(&havuz)?);
        let mut sonuc = Self {
            havuz,
            kiralik: None,
            eylem_derinligi: 0,
        };
        for _ in 0..eylem_derinligi {
            sonuc.eylem_baslat()?;
        }
        Ok(sonuc)
    }

    pub fn oku(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        girdiyi_denetle(sorgu, parametreler)?;
        let baglar = metin_baglari(parametreler);
        let satirlar = if self.eylem_derinligi > 0 {
            self.kiralik
                .as_mut()
                .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?
                .query_typed(sorgu, &baglar)
                .map_err(|h| postgres_hatasi("PostgreSQL sorgusu başarısız", h))?
        } else {
            let mut kiralik = havuz::al(&self.havuz)?;
            match kiralik.query_typed(sorgu, &baglar) {
                Ok(satirlar) => satirlar,
                Err(_ilk_hata)
                    if yeniden_baglanabilir(
                        SorguSinifi::Okuma,
                        self.eylem_derinligi,
                        kiralik.is_closed(),
                        0,
                    ) =>
                {
                    drop(kiralik);
                    let mut ikinci = havuz::al(&self.havuz)?;
                    ikinci.query_typed(sorgu, &baglar).map_err(|h| {
                        postgres_hatasi("PostgreSQL sorgusu yeniden denendi ama başarısız", h)
                    })?
                }
                Err(hata_degeri) => {
                    return Err(postgres_hatasi("PostgreSQL sorgusu başarısız", hata_degeri));
                }
            }
        };
        let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.veritabani();
        if satirlar.len() > sinirlar.satir() {
            return Err(hata("PostgreSQL sonucu 10.000 satır sınırını aşıyor"));
        }
        let mut toplam = 0usize;
        let mut sonuc = Vec::with_capacity(satirlar.len());
        for satir in satirlar {
            if satir.len() > sinirlar.sutun() {
                return Err(hata("PostgreSQL sonucu 100 sütun sınırını aşıyor"));
            }
            let mut alanlar = Vec::with_capacity(satir.len());
            for (sira, sutun) in satir.columns().iter().enumerate() {
                if !matches!(
                    *sutun.type_(),
                    Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME
                ) {
                    return Err(verili_hata(
                        format!(
                            "\"{}\" sütunu Metin değil; sorguda açık ::text dönüşümü kullan",
                            sutun.name()
                        ),
                        vec![("sutun".into(), sutun.name().into())],
                    ));
                }
                let deger: Option<String> = satir
                    .try_get(sira)
                    .map_err(|h| postgres_hatasi("PostgreSQL sonuç sütunu okunamadı", h))?;
                let Some(deger) = deger else {
                    return Err(verili_hata(
                        format!(
                            "\"{}\" sütunu NULL; sorguda COALESCE ile açık metin değeri seç",
                            sutun.name()
                        ),
                        vec![("sutun".into(), sutun.name().into())],
                    ));
                };
                toplam = toplam.saturating_add(sutun.name().len() + deger.len());
                if toplam > sinirlar.sonuc_bayti() {
                    return Err(hata("PostgreSQL sonucu 16 MiB sınırını aşıyor"));
                }
                alanlar.push((sutun.name().to_string(), deger));
            }
            sonuc.push(alanlar);
        }
        Ok(sonuc)
    }

    pub fn degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<i64, VeritabaniHatasi> {
        debug_assert!(!yeniden_baglanabilir(
            SorguSinifi::Yazma,
            self.eylem_derinligi,
            self.kiralik.as_ref().is_some_and(|k| k.is_closed()),
            0,
        ));
        if self.eylem_derinligi == 0 {
            return Err(hata(
                "PostgreSQL değişikliği yalnız eylem transaction'ı içinde yapılabilir",
            ));
        }
        girdiyi_denetle(sorgu, parametreler)?;
        let istemci = self
            .kiralik
            .as_mut()
            .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?;
        istemci
            .batch_execute("SAVEPOINT zee_intrinsic")
            .map_err(|h| postgres_hatasi("PostgreSQL sorgu savepoint'i açılamadı", h))?;
        let baglar = metin_baglari(parametreler);
        let adet = match istemci.execute_typed(sorgu, &baglar) {
            Ok(adet) => {
                istemci
                    .batch_execute("RELEASE SAVEPOINT zee_intrinsic")
                    .map_err(|h| postgres_hatasi("PostgreSQL sorgu savepoint'i kapanamadı", h))?;
                adet
            }
            Err(h) => {
                let asil = postgres_hatasi("PostgreSQL değişikliği başarısız", h);
                istemci
                    .batch_execute(
                        "ROLLBACK TO SAVEPOINT zee_intrinsic; RELEASE SAVEPOINT zee_intrinsic",
                    )
                    .map_err(|geri| {
                        postgres_hatasi("PostgreSQL sorgu savepoint'i geri alınamadı", geri)
                    })?;
                return Err(asil);
            }
        };
        i64::try_from(adet).map_err(|_| hata("Etkilenen satır sayısı TamSayı sınırını aşıyor"))
    }

    pub fn eylem_baslat(&mut self) -> Result<(), VeritabaniHatasi> {
        if self.eylem_derinligi == 0 {
            let mut kiralik = havuz::al(&self.havuz)?;
            kiralik
                .batch_execute("BEGIN")
                .map_err(|h| postgres_hatasi("PostgreSQL transaction'ı başlatılamadı", h))?;
            self.kiralik = Some(kiralik);
            self.eylem_derinligi = 1;
            return Ok(());
        }
        let sorgu = format!("SAVEPOINT zee_{}", self.eylem_derinligi);
        self.kiralik
            .as_mut()
            .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?
            .batch_execute(&sorgu)
            .map_err(|h| postgres_hatasi("PostgreSQL transaction'ı başlatılamadı", h))?;
        self.eylem_derinligi += 1;
        Ok(())
    }

    pub fn eylem_tamamla(&mut self) -> Result<(), VeritabaniHatasi> {
        debug_assert!(!yeniden_baglanabilir(
            SorguSinifi::Commit,
            self.eylem_derinligi,
            self.kiralik.as_ref().is_some_and(|k| k.is_closed()),
            0,
        ));
        let Some(yeni_derinlik) = self.eylem_derinligi.checked_sub(1) else {
            return Err(hata("Açık PostgreSQL transaction'ı yok"));
        };
        let sorgu = if yeni_derinlik == 0 {
            "COMMIT".to_string()
        } else {
            format!("RELEASE SAVEPOINT zee_{}", yeni_derinlik)
        };
        let sonuc = self
            .kiralik
            .as_mut()
            .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?
            .batch_execute(&sorgu);
        if let Err(h) = sonuc {
            if yeni_derinlik == 0 {
                self.eylem_derinligi = 0;
                self.kiralik.take();
                return Err(commit_sonucu_belirsiz_hatasi(
                    "PostgreSQL COMMIT sonucu belirsiz; otomatik yeniden deneme yapılmadı",
                    h,
                ));
            }
            return Err(postgres_hatasi("PostgreSQL savepoint'i tamamlanamadı", h));
        }
        self.eylem_derinligi = yeni_derinlik;
        if yeni_derinlik == 0 {
            self.kiralik.take();
        }
        Ok(())
    }

    pub fn eylem_geri_al(&mut self) -> Result<(), VeritabaniHatasi> {
        let Some(yeni_derinlik) = self.eylem_derinligi.checked_sub(1) else {
            return Err(hata("Açık PostgreSQL transaction'ı yok"));
        };
        let sorgu = if yeni_derinlik == 0 {
            "ROLLBACK".to_string()
        } else {
            format!(
                "ROLLBACK TO SAVEPOINT zee_{0}; RELEASE SAVEPOINT zee_{0}",
                yeni_derinlik
            )
        };
        self.kiralik
            .as_mut()
            .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?
            .batch_execute(&sorgu)
            .map_err(|h| postgres_hatasi("PostgreSQL transaction'ı geri alınamadı", h))?;
        self.eylem_derinligi = yeni_derinlik;
        if yeni_derinlik == 0 {
            self.kiralik.take();
        }
        Ok(())
    }
}

fn yeniden_baglanabilir(
    sinif: SorguSinifi,
    eylem_derinligi: usize,
    istemci_kapali: bool,
    yeniden_deneme_sayisi: usize,
) -> bool {
    sinif == SorguSinifi::Okuma
        && eylem_derinligi == 0
        && istemci_kapali
        && yeniden_deneme_sayisi == 0
}

/// Migration klasörünü tarar; her dosyayı `NNNN_aciklama.sql` adı, tek dosya
/// ve toplam byte bütçesi, UTF-8 ve SQL profili açısından doğrular. Sürüme
/// göre sıralı `(sürüm, ad, sha256, sql)` listesi döner.
fn goc_dosyalarini_topla(
    goc_koku: &std::path::Path,
) -> Result<Vec<(i64, String, String, String)>, VeritabaniHatasi> {
    let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.veritabani();
    let mut gocler = Vec::new();
    let mut toplam = 0u64;
    let girdiler = std::fs::read_dir(goc_koku)
        .map_err(|h| hata(format!("Migration klasörü okunamadı: {}", h)))?;
    for girdi in girdiler {
        let girdi = girdi.map_err(|h| hata(format!("Migration girdisi okunamadı: {}", h)))?;
        let tur = girdi
            .file_type()
            .map_err(|h| hata(format!("Migration türü okunamadı: {}", h)))?;
        if !tur.is_file() {
            return Err(hata(
                "Migration klasörü yalnız düzenli .sql dosyaları taşımalı",
            ));
        }
        let ad = girdi
            .file_name()
            .into_string()
            .map_err(|_| hata("Migration dosya adı geçerli UTF-8 olmalı"))?;
        let surum = goc_surumunu_coz(&ad)?;
        let metadata = girdi
            .metadata()
            .map_err(|h| hata(format!("Migration boyutu okunamadı: {}", h)))?;
        if metadata.len() > sinirlar.goc_dosyasi_bayti() {
            return Err(hata("Tek migration 1 MiB sınırını aşıyor"));
        }
        toplam = toplam.saturating_add(metadata.len());
        if toplam > sinirlar.goc_toplami_bayti() {
            return Err(hata("Migration toplamı 8 MiB sınırını aşıyor"));
        }
        let icerik =
            std::fs::read(girdi.path()).map_err(|h| hata(format!("Migration okunamadı: {}", h)))?;
        let sql = String::from_utf8(icerik.clone())
            .map_err(|_| hata("Migration içeriği geçerli UTF-8 olmalı"))?;
        migration_sqlini_denetle(&sql)?;
        gocler.push((surum, ad, crate::guvenlik::sha256_hex(&icerik), sql));
    }
    gocler.sort_by_key(|(surum, _, _, _)| *surum);
    if gocler.windows(2).any(|cift| cift[0].0 == cift[1].0) {
        return Err(hata(
            "Aynı migration sürümü birden çok dosyada kullanılamaz",
        ));
    }
    Ok(gocler)
}

/// `NNNN_aciklama.sql` adından pozitif sürümü çözer; açıklama küçük ASCII
/// harf, rakam ve alt çizgiyle sınırlıdır.
fn goc_surumunu_coz(ad: &str) -> Result<i64, VeritabaniHatasi> {
    let govde_adi = ad
        .strip_suffix(".sql")
        .ok_or_else(|| hata("Migration dosyaları .sql uzantılı olmalı"))?;
    let (surum, aciklama) = govde_adi
        .split_once('_')
        .ok_or_else(|| hata("Migration adı NNNN_aciklama.sql biçiminde olmalı"))?;
    let surum = surum
        .parse::<i64>()
        .ok()
        .filter(|s| *s > 0)
        .ok_or_else(|| hata("Migration sürümü pozitif sayı olmalı"))?;
    if aciklama.is_empty()
        || !aciklama
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(hata(
            "Migration açıklaması küçük ASCII harf, sayı ve _ taşımalı",
        ));
    }
    Ok(surum)
}

pub fn gocleri_uygula(
    bildirim: &VeritabaniBildirimi,
    proje_koku: &std::path::Path,
) -> Result<GocRaporu, VeritabaniHatasi> {
    let proje_koku = std::fs::canonicalize(proje_koku)
        .map_err(|h| hata(format!("Proje kökü çözülemedi: {}", h)))?;
    let goc_koku = std::fs::canonicalize(proje_koku.join(&bildirim.gocler))
        .map_err(|h| hata(format!("Migration klasörü çözülemedi: {}", h)))?;
    if !goc_koku.starts_with(&proje_koku) || !goc_koku.is_dir() {
        return Err(hata(
            "Migration klasörü proje kökü içinde gerçek bir klasör olmalı",
        ));
    }
    let gocler = goc_dosyalarini_topla(&goc_koku)?;

    let havuz = havuz::kur(bildirim)?;
    let mut istemci = havuz::al(&havuz)?;
    let mut tx = istemci
        .transaction()
        .map_err(|h| postgres_hatasi("Migration transaction'ı başlatılamadı", h))?;
    tx.batch_execute(
        "CREATE TABLE IF NOT EXISTS _zee_gocleri (\
         surum BIGINT PRIMARY KEY, ad TEXT NOT NULL, sha256 TEXT NOT NULL, \
         uygulandi TIMESTAMPTZ NOT NULL DEFAULT now()); \
         SELECT pg_advisory_xact_lock(151663);",
    )
    .map_err(|h| postgres_hatasi("Migration kayıt tablosu hazırlanamadı", h))?;
    let kayitlar = tx
        .query(
            "SELECT surum, ad, sha256 FROM _zee_gocleri ORDER BY surum",
            &[],
        )
        .map_err(|h| postgres_hatasi("Migration kayıtları okunamadı", h))?;
    let mut uygulananlar = std::collections::BTreeMap::new();
    for kayit in kayitlar {
        uygulananlar.insert(
            kayit.get::<_, i64>(0),
            (kayit.get::<_, String>(1), kayit.get::<_, String>(2)),
        );
    }
    if uygulananlar
        .keys()
        .any(|surum| !gocler.iter().any(|goc| &goc.0 == surum))
    {
        return Err(hata(
            "Veritabanında kayıtlı bir migration dosyası proje ağacında yok",
        ));
    }
    let mut rapor = GocRaporu {
        uygulanan: 0,
        atlanan: 0,
    };
    for (surum, ad, ozet, sql) in gocler {
        if let Some((eski_ad, eski_ozet)) = uygulananlar.get(&surum) {
            if eski_ad != &ad || eski_ozet != &ozet {
                return Err(verili_hata(
                    format!("{} uygulanmış migration içeriğiyle uyuşmuyor", ad),
                    vec![("surum".into(), surum.to_string())],
                ));
            }
            rapor.atlanan += 1;
            continue;
        }
        tx.batch_execute(&sql)
            .map_err(|h| postgres_hatasi(&format!("{} migration'ı uygulanamadı", ad), h))?;
        tx.execute(
            "INSERT INTO _zee_gocleri (surum, ad, sha256) VALUES ($1, $2, $3)",
            &[&surum, &ad, &ozet],
        )
        .map_err(|h| postgres_hatasi("Migration kaydı yazılamadı", h))?;
        rapor.uygulanan += 1;
    }
    tx.commit()
        .map_err(|h| postgres_hatasi("Migration transaction'ı tamamlanamadı", h))?;
    Ok(rapor)
}

fn migration_sqlini_denetle(sql: &str) -> Result<(), VeritabaniHatasi> {
    let kelimeler = sql
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    for yasak in ["begin", "commit", "rollback", "savepoint"] {
        if kelimeler.iter().any(|kelime| kelime == yasak) {
            return Err(hata(format!(
                "Migration kendi {} transaction komutunu taşıyamaz",
                yasak
            )));
        }
    }
    Ok(())
}

fn girdiyi_denetle(sorgu: &str, parametreler: &[String]) -> Result<(), VeritabaniHatasi> {
    let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.veritabani();
    if sorgu.is_empty() || sorgu.len() > sinirlar.sorgu_bayti() {
        return Err(hata("PostgreSQL sorgusu boş ya da 64 KiB sınırını aşıyor"));
    }
    if parametreler.len() > sinirlar.parametre()
        || parametreler
            .iter()
            .any(|p| p.len() > sinirlar.parametre_bayti())
    {
        return Err(hata(
            "PostgreSQL parametreleri 100 öğe / öğe başına 64 KiB sınırını aşıyor",
        ));
    }
    Ok(())
}

fn metin_baglari(parametreler: &[String]) -> Vec<(&(dyn ToSql + Sync), Type)> {
    parametreler
        .iter()
        .map(|p| (p as &(dyn ToSql + Sync), Type::TEXT))
        .collect()
}

fn postgres_hatasi(on: &str, hata_degeri: postgres::Error) -> VeritabaniHatasi {
    let mut veri = Vec::new();
    let mesaj = if let Some(db) = hata_degeri.as_db_error() {
        veri.push(("sqlstate".into(), db.code().code().into()));
        if let Some(kisit) = db.constraint() {
            veri.push(("kısıt".into(), kisit.into()));
        }
        if let Some(tablo) = db.table() {
            veri.push(("tablo".into(), tablo.into()));
        }
        format!("{}: {}", on, db.message())
    } else {
        on.to_string()
    };
    verili_hata(mesaj, veri)
}

fn commit_sonucu_belirsiz_hatasi(on: &str, hata_degeri: postgres::Error) -> VeritabaniHatasi {
    let mut sonuc = postgres_hatasi(on, hata_degeri);
    sonuc
        .veri
        .push(("hata_sinifi".into(), "db.commit_unknown".into()));
    sonuc
}

fn hata(mesaj: impl Into<String>) -> VeritabaniHatasi {
    verili_hata(mesaj, Vec::new())
}

fn verili_hata(mesaj: impl Into<String>, veri: Vec<(String, String)>) -> VeritabaniHatasi {
    VeritabaniHatasi {
        mesaj: mesaj.into(),
        veri,
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use postgres::Config;
    use std::str::FromStr;

    #[test]
    fn migration_kendi_transaction_sinirini_tasiyamaz() {
        assert!(migration_sqlini_denetle("CREATE TABLE sayfa(id BIGINT);").is_ok());
        for sql in [
            "BEGIN; CREATE TABLE x(id int); COMMIT;",
            "savepoint kendim;",
            "ROLLBACK;",
        ] {
            assert!(
                migration_sqlini_denetle(sql).is_err(),
                "reddedilmeli: {sql}"
            );
        }
    }

    #[test]
    fn baglanti_tam_bildirim_hedefi_ve_guvensiz_profilde_kilitlidir() {
        let hedef = VeritabaniHedefi::ayristir("postgresql://127.0.0.1:5432/uygulama")
            .expect("hedef geçerli");
        let dogru =
            Config::from_str("postgresql://kullanici@127.0.0.1:5432/uygulama?sslmode=disable")
                .expect("ayar geçerli");
        assert!(havuz::hedefi_denetle(&dogru, &hedef).is_ok());

        let yanlis =
            Config::from_str("postgresql://kullanici@127.0.0.1:5432/baska?sslmode=disable")
                .expect("ayar geçerli");
        assert!(havuz::hedefi_denetle(&yanlis, &hedef).is_err());
    }

    #[test]
    fn yalniz_transaction_disindaki_okuma_bir_kez_yeniden_baglanabilir() {
        let mut vaka_sayisi = 0usize;
        for (sira, satir) in include_str!("../tests/fixtures/postgresql-recovery-v1.tsv")
            .lines()
            .enumerate()
        {
            if satir.starts_with('#') || satir.is_empty() {
                continue;
            }
            vaka_sayisi += 1;
            let alanlar = satir.split('\t').collect::<Vec<_>>();
            assert_eq!(alanlar.len(), 5, "{} numaralı fixture satırı", sira + 1);
            let sinif = match alanlar[0] {
                "read" => SorguSinifi::Okuma,
                "write" => SorguSinifi::Yazma,
                "commit" => SorguSinifi::Commit,
                bilinmeyen => panic!("bilinmeyen sorgu sınıfı: {bilinmeyen}"),
            };
            let gercek = yeniden_baglanabilir(
                sinif,
                alanlar[1].parse().expect("eylem derinliği sayı olmalı"),
                alanlar[2].parse().expect("istemci durumu bool olmalı"),
                alanlar[3]
                    .parse()
                    .expect("yeniden deneme sayısı sayı olmalı"),
            );
            assert_eq!(gercek, alanlar[4] == "retry", "fixture satırı: {satir}");
        }
        assert_eq!(vaka_sayisi, 6, "recovery karar matrisi eksilmemeli");
    }
}
