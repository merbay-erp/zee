//! K-163 PostgreSQL dogfood adaptörü.
//!
//! Loopback geliştirme açık `sslmode=disable`; production profili pinned CA,
//! hostname doğrulamalı `sslmode=require` ve sınırlı havuz kullanır. Değerler
//! SQL metnine eklenmez, extended-query protokolüne açık TEXT parametreleri
//! olarak bind edilir.

mod havuz;
mod sonuc;

use crate::veritabani_modeli::{VeritabaniBildirimi, VeritabaniHatasi, VeritabaniHedefi};
use postgres::types::{ToSql, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GocRaporu {
    pub uygulanan: usize,
    pub atlanan: usize,
}

/// SQL yürütmeyen migration durum/kuru çalışma görüntüsü.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GocPlani {
    pub adimlar: Vec<GocAdimi>,
    pub uygulanmis: usize,
    pub bekleyen: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GocAdimi {
    pub surum: i64,
    pub ad: String,
    pub sha256: String,
    pub uygulanmis: bool,
}

pub struct PostgresqlOturumu {
    havuz: havuz::PostgresqlHavuzu,
    kiralik: Option<havuz::PostgresqlKirasi>,
    eylem_derinligi: usize,
    istek_baglami_acik: bool,
    gecersiz_eylem: Option<VeritabaniHatasi>,
    /// K-375: açık transaction'da son tarihten türetilmiş `SET LOCAL`
    /// zaman aşımı etkin olabilir mi? Sonraki süresiz sorgu ve COMMIT öncesi
    /// varsayılana döndürülür.
    sure_ayari_etkin: bool,
    /// Transaction içinde en az bir kez süre ayarlandı; `ROLLBACK TO
    /// SAVEPOINT` varsayılana dönüşü de geri alabileceği için yeniden kirli sayılır.
    sure_ayari_transactionda: bool,
    /// K-391 (ADR-168): tek parametre tavanı; proje `veritabanı_parametre_sınırı`.
    parametre_bayti: usize,
}

/// K-391: bütün parametrelerin toplam bayt tavanı — varsayılan tek tavan ×
/// parametre sayısı (eski örtük en kötü durum; yükseltme bunu aşamaz).
const TOPLAM_PARAMETRE_BAYTI: usize = {
    let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.veritabani();
    sinirlar.parametre() * sinirlar.parametre_bayti()
};

/// K-375/RFC-0042: son tarihin sunucu tarafı bütçesi.
const SURE_AYARI_VARSAYILAN: &str =
    "SET LOCAL statement_timeout TO DEFAULT; SET LOCAL lock_timeout TO DEFAULT";

fn sure_ayari(bitis: std::time::Instant) -> Result<String, VeritabaniHatasi> {
    let kalan = bitis.saturating_duration_since(std::time::Instant::now());
    if kalan.is_zero() {
        return Err(son_tarih_hatasi(None));
    }
    // Yukarı yuvarlama: sunucu iptali son tarihten önce gelemez; 0 PostgreSQL'de
    // "sınırsız" demektir, en az 1 ms gönderilir.
    let ms = kalan.as_micros().div_ceil(1000).clamp(1, i32::MAX as u128);
    Ok(format!(
        "SET LOCAL statement_timeout = {ms}; SET LOCAL lock_timeout = {ms}"
    ))
}

/// Sunucu `statement_timeout` (57014) ya da `lock_timeout` (55P03) iptali
/// yalnız kendi bitiş anımız geçtiyse son tarih sayılır; süresi dolmadan gelen
/// `NOWAIT` 55P03 ya da yönetici iptali olağan C025 olarak kalır.
fn son_tarih_iptali(h: &postgres::Error, bitis: std::time::Instant) -> Option<VeritabaniHatasi> {
    let kod = h.as_db_error()?.code().code();
    (matches!(kod, "57014" | "55P03") && std::time::Instant::now() >= bitis)
        .then(|| son_tarih_hatasi(Some(kod)))
}

fn son_tarih_hatasi(sqlstate: Option<&str>) -> VeritabaniHatasi {
    let mut veri = Vec::new();
    if let Some(kod) = sqlstate {
        veri.push(("sqlstate".into(), kod.into()));
    }
    veri.push((
        "hata_sinifi".into(),
        crate::veritabani_modeli::SON_TARIH_IPTALI_SINIFI.into(),
    ));
    verili_hata(
        "PostgreSQL sorgusu son tarih dolduğu için iptal edildi; otomatik tekrar yapılmadı",
        veri,
    )
}

enum SureliOkumaHatasi {
    Surucu(postgres::Error),
    Zee(VeritabaniHatasi),
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
        Self::baglan_ic(bildirim, eylem_derinligi, None)
    }

    /// Private CLI telemetry bridge; not part of the supported Rust v1 facade.
    #[doc(hidden)]
    pub fn baglan_gozlemli(
        bildirim: &VeritabaniBildirimi,
        eylem_derinligi: usize,
        gozlem: std::sync::Arc<dyn Fn([u64; 8]) + Send + Sync>,
    ) -> Result<Self, VeritabaniHatasi> {
        Self::baglan_ic(bildirim, eylem_derinligi, Some(gozlem))
    }
    /// K-391 (ADR-168): proje `veritabanı_parametre_sınırı` ile tek SQL
    /// parametresi tavanını yükseltir; `None` spec/25 varsayılanıdır.
    pub fn parametre_sinirini_ayarla(&mut self, bayt: Option<usize>) {
        self.parametre_bayti = bayt.unwrap_or(
            crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
                .veritabani()
                .parametre_bayti(),
        );
    }
    #[doc(hidden)]
    pub fn metrik_ozetini_yayinla(&self) {
        havuz::olcum_yayinla(&self.havuz);
    }
    fn baglan_ic(
        bildirim: &VeritabaniBildirimi,
        eylem_derinligi: usize,
        gozlem: Option<std::sync::Arc<dyn Fn([u64; 8]) + Send + Sync>>,
    ) -> Result<Self, VeritabaniHatasi> {
        let havuz = match gozlem {
            Some(gozlem) => havuz::kur_gozlemli(bildirim, Some(gozlem))?,
            None => havuz::kur(bildirim)?,
        };
        // Yapılandırma hatası ilk sorguya kadar saklanmasın; başlangıçta bir
        // gerçek bağlantı checkout/health kontrolünden geçsin.
        drop(havuz::al(&havuz)?);
        let mut sonuc = Self {
            havuz,
            kiralik: None,
            eylem_derinligi: 0,
            istek_baglami_acik: false,
            gecersiz_eylem: None,
            sure_ayari_etkin: false,
            sure_ayari_transactionda: false,
            parametre_bayti: crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
                .veritabani()
                .parametre_bayti(),
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
        self.oku_son_tarihli(sorgu, parametreler, None)
    }

    /// K-375/RFC-0042: `bitis` varsa sorgu sunucuda kalan süreyle sınırlanır.
    /// Transaction içinde `SET LOCAL` + sorgu savepoint'i, dışında tek okumalık
    /// `BEGIN … COMMIT` kullanılır; oturum düzeyinde ayar havuza sızmaz.
    /// `bitis` yoksa round trip sayısı ve davranış değişmez.
    pub fn oku_son_tarihli(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
        bitis: Option<std::time::Instant>,
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        self.eylemi_denetle()?;
        girdiyi_denetle(sorgu, parametreler, self.parametre_bayti)?;
        let baglar = metin_baglari(parametreler);
        if let Some(bitis) = bitis {
            return self.sureli_oku(sorgu, &baglar, bitis);
        }
        if self.eylem_derinligi > 0 {
            self.sure_ayarini_sifirla()?;
            let istemci = self
                .kiralik
                .as_mut()
                .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?;
            return sonuc::oku(istemci, sorgu, &baglar)
                .map_err(|h| self.sorgu_hatasi("PostgreSQL sorgusu başarısız", h))?;
        }
        let mut kiralik = havuz::al(&self.havuz)?;
        match sonuc::oku(&mut kiralik, sorgu, &baglar) {
            Ok(sonuc) => sonuc,
            Err(h) if wire_hatasi(&h).is_some() => {
                Err(postgres_hatasi("PostgreSQL sorgusu başarısız", h))
            }
            Err(_ilk_hata)
                if yeniden_baglanabilir(
                    SorguSinifi::Okuma,
                    self.eylem_derinligi,
                    kiralik.is_closed(),
                    0,
                    false,
                ) =>
            {
                drop(kiralik);
                let mut ikinci = havuz::al(&self.havuz)?;
                sonuc::oku(&mut ikinci, sorgu, &baglar).map_err(|h| {
                    postgres_hatasi("PostgreSQL sorgusu yeniden denendi ama başarısız", h)
                })?
            }
            Err(h) => Err(postgres_hatasi("PostgreSQL sorgusu başarısız", h)),
        }
    }

    pub fn degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<i64, VeritabaniHatasi> {
        self.degistir_son_tarihli(sorgu, parametreler, None)
    }

    /// K-375: değişiklik zaten sorgu savepoint'i taşır; son tarih varsa aynı
    /// batch `SET LOCAL` ile başlar ve iptal yalnız bu ifadeyi geri alır.
    pub fn degistir_son_tarihli(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
        bitis: Option<std::time::Instant>,
    ) -> Result<i64, VeritabaniHatasi> {
        self.eylemi_denetle()?;
        debug_assert!(!yeniden_baglanabilir(
            SorguSinifi::Yazma,
            self.eylem_derinligi,
            self.kiralik.as_ref().is_some_and(|k| k.is_closed()),
            0,
            false
        ));
        if self.eylem_derinligi == 0 {
            return Err(hata(
                "PostgreSQL değişikliği yalnız eylem transaction'ı içinde yapılabilir",
            ));
        }
        girdiyi_denetle(sorgu, parametreler, self.parametre_bayti)?;
        let acilis = match bitis {
            Some(bitis) => {
                let ayar = sure_ayari(bitis)?;
                self.sure_ayari_etkin = true;
                self.sure_ayari_transactionda = true;
                format!("{ayar}; SAVEPOINT zee_intrinsic")
            }
            None if self.sure_ayari_etkin => {
                self.sure_ayari_etkin = false;
                format!("{SURE_AYARI_VARSAYILAN}; SAVEPOINT zee_intrinsic")
            }
            None => "SAVEPOINT zee_intrinsic".to_string(),
        };
        let istemci = self
            .kiralik
            .as_mut()
            .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?;
        let baglar = metin_baglari(parametreler);
        let sonuc = (|| -> Result<u64, (&str, postgres::Error)> {
            istemci
                .batch_execute(&acilis)
                .map_err(|h| ("PostgreSQL sorgu savepoint'i açılamadı", h))?;
            match istemci.execute_typed(sorgu, &baglar) {
                Ok(adet) => {
                    istemci
                        .batch_execute("RELEASE SAVEPOINT zee_intrinsic")
                        .map_err(|h| ("PostgreSQL sorgu savepoint'i kapanamadı", h))?;
                    Ok(adet)
                }
                Err(h) => {
                    // A frame rejection closed the connection. Do not replace its
                    // typed cause with the inevitable failed rollback acknowledgement.
                    if wire_hatasi(&h).is_none() {
                        istemci.batch_execute(
                            "ROLLBACK TO SAVEPOINT zee_intrinsic; RELEASE SAVEPOINT zee_intrinsic",
                        ).map_err(|h| ("PostgreSQL sorgu savepoint'i geri alınamadı", h))?;
                    }
                    Err(("PostgreSQL değişikliği başarısız", h))
                }
            }
        })();
        let adet = sonuc.map_err(|(on, h)| {
            match bitis.and_then(|bitis| son_tarih_iptali(&h, bitis)) {
                // Sorgu savepoint'i geri alındı; transaction kullanılabilir kalır.
                Some(son_tarih) => son_tarih,
                None => self.sorgu_hatasi(on, h),
            }
        })?;
        i64::try_from(adet).map_err(|_| hata("Etkilenen satır sayısı TamSayı sınırını aşıyor"))
    }

    fn sureli_oku(
        &mut self,
        sorgu: &str,
        baglar: &[(&(dyn ToSql + Sync), Type)],
        bitis: std::time::Instant,
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        if self.eylem_derinligi > 0 {
            let ayar = sure_ayari(bitis)?;
            self.sure_ayari_etkin = true;
            self.sure_ayari_transactionda = true;
            let istemci = self
                .kiralik
                .as_mut()
                .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?;
            // Okuma da değişiklik gibi sorgu savepoint'i taşır: iptal edilen
            // ifade açık transaction'ı abort etmez, `yetişmezse` sonrası eylem
            // aynı transaction'la sürebilir.
            let sonuc = (|| -> Result<_, (&str, postgres::Error)> {
                istemci
                    .batch_execute(&format!("{ayar}; SAVEPOINT zee_sure"))
                    .map_err(|h| ("PostgreSQL sorgu savepoint'i açılamadı", h))?;
                match sonuc::oku(istemci, sorgu, baglar) {
                    Ok(sonuc) => {
                        if let Err(h) = istemci.batch_execute("RELEASE SAVEPOINT zee_sure") {
                            // Bütçe tam RELEASE'te dolarsa transaction abort kalmasın.
                            if wire_hatasi(&h).is_none() {
                                let _ = istemci.batch_execute(
                                    "ROLLBACK TO SAVEPOINT zee_sure; RELEASE SAVEPOINT zee_sure",
                                );
                            }
                            return Err(("PostgreSQL sorgu savepoint'i kapanamadı", h));
                        }
                        Ok(sonuc)
                    }
                    Err(h) => {
                        if wire_hatasi(&h).is_none() {
                            istemci
                                .batch_execute(
                                    "ROLLBACK TO SAVEPOINT zee_sure; RELEASE SAVEPOINT zee_sure",
                                )
                                .map_err(|h| ("PostgreSQL sorgu savepoint'i geri alınamadı", h))?;
                        }
                        Err(("PostgreSQL sorgusu başarısız", h))
                    }
                }
            })();
            return match sonuc {
                Ok(sonuc) => sonuc,
                Err((on, h)) => Err(match son_tarih_iptali(&h, bitis) {
                    Some(son_tarih) => son_tarih,
                    None => self.sorgu_hatasi(on, h),
                }),
            };
        }
        let mut deneme = 0usize;
        loop {
            // Yeniden bağlanma kalan süreyi tüketir; ayar her girişimde yeniden hesaplanır.
            let ayar = sure_ayari(bitis)?;
            let mut kiralik = havuz::al(&self.havuz)?;
            match tek_sureli_okuma(&mut kiralik, &ayar, sorgu, baglar) {
                Ok(sonuc) => return sonuc,
                Err(SureliOkumaHatasi::Zee(h)) => return Err(h),
                Err(SureliOkumaHatasi::Surucu(h)) => {
                    let iptal = son_tarih_iptali(&h, bitis);
                    if wire_hatasi(&h).is_none()
                        && yeniden_baglanabilir(
                            SorguSinifi::Okuma,
                            self.eylem_derinligi,
                            kiralik.is_closed(),
                            deneme,
                            iptal.is_some(),
                        )
                    {
                        deneme += 1;
                        continue;
                    }
                    if let Some(son_tarih) = iptal {
                        return Err(son_tarih);
                    }
                    return Err(postgres_hatasi(
                        if deneme == 0 {
                            "PostgreSQL sorgusu başarısız"
                        } else {
                            "PostgreSQL sorgusu yeniden denendi ama başarısız"
                        },
                        h,
                    ));
                }
            }
        }
    }

    /// Son tarih bloğu bittiği hâlde transaction sürüyorsa eski `SET LOCAL`
    /// bütçesi süresiz sorguya taşınmaz. Yalnız ayar yapılmış transaction'da
    /// round trip ekler.
    fn sure_ayarini_sifirla(&mut self) -> Result<(), VeritabaniHatasi> {
        if !self.sure_ayari_etkin || self.eylem_derinligi == 0 {
            return Ok(());
        }
        let sonuc = self
            .kiralik
            .as_mut()
            .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?
            .batch_execute(SURE_AYARI_VARSAYILAN);
        sonuc.map_err(|h| self.sorgu_hatasi("PostgreSQL süre ayarı sıfırlanamadı", h))?;
        self.sure_ayari_etkin = false;
        Ok(())
    }

    /// Doğrulanmış web oturumunu aynı bağlantıda request-local RLS bağlamına
    /// dönüştürür. Çözücü adı proje bildirimi tarafından kapalı profilde
    /// doğrulanır; kullanıcı ise yalnız sunucu oturumundan gelir.
    pub fn istek_baglamini_baslat(
        &mut self,
        cozucu: &str,
        kullanici: &str,
    ) -> Result<(), VeritabaniHatasi> {
        self.eylemi_denetle()?;
        if self.istek_baglami_acik || self.eylem_derinligi != 0 {
            return Err(hata("PostgreSQL istek bağlamı zaten açık"));
        }
        self.eylem_baslat()?;
        let sonuc = (|| {
            let sorgu = format!(
                "SELECT tenant_id::text AS tenant_id FROM {}($1::text)",
                cozucu
            );
            let satirlar = self.oku(&sorgu, &[kullanici.to_string()])?;
            let tenant_id = match satirlar.as_slice() {
                [satir] => match satir.as_slice() {
                    [(ad, deger)] if ad == "tenant_id" => deger.clone(),
                    _ => {
                        return Err(hata("İstek bağlamı çözücüsü yalnız tenant_id döndürmeli"));
                    }
                },
                _ => return Err(hata("İstek bağlamı çözücüsü tam bir satır döndürmeli")),
            };
            let ayar = self.oku(
                "SELECT set_config('app.tenant_id', $1::uuid::text, true) AS tenant_id",
                &[tenant_id],
            )?;
            if ayar.len() != 1 {
                return Err(hata("PostgreSQL tenant bağlamı kurulamadı"));
            }
            Ok(())
        })();
        match sonuc {
            Ok(()) => {
                self.istek_baglami_acik = true;
                Ok(())
            }
            Err(asil) => match self.eylem_geri_al() {
                Ok(()) => Err(asil),
                Err(geri) => Err(hata(format!(
                    "{}; istek transaction'ı geri alınamadı: {}",
                    asil.mesaj, geri.mesaj
                ))),
            },
        }
    }

    pub fn istek_baglamini_tamamla(&mut self) -> Result<(), VeritabaniHatasi> {
        self.eylemi_denetle()?;
        if !self.istek_baglami_acik {
            return Ok(());
        }
        if self.eylem_derinligi != 1 {
            return Err(hata("PostgreSQL istek transaction sınırı dengeli değil"));
        }
        self.istek_baglami_acik = false;
        self.eylem_tamamla()
    }

    pub fn istek_baglamini_geri_al(&mut self) -> Result<(), VeritabaniHatasi> {
        if !self.istek_baglami_acik {
            return Ok(());
        }
        while self.eylem_derinligi > 0 {
            self.eylem_geri_al()?;
        }
        self.istek_baglami_acik = false;
        Ok(())
    }

    pub fn eylem_baslat(&mut self) -> Result<(), VeritabaniHatasi> {
        self.eylemi_denetle()?;
        if self.eylem_derinligi == 0 {
            let mut kiralik = havuz::al(&self.havuz)?;
            kiralik
                .batch_execute("BEGIN")
                .map_err(|h| self.sorgu_hatasi("PostgreSQL transaction'ı başlatılamadı", h))?;
            self.kiralik = Some(kiralik);
            self.eylem_derinligi = 1;
            self.sure_ayari_etkin = false;
            self.sure_ayari_transactionda = false;
            return Ok(());
        }
        let sorgu = format!("SAVEPOINT zee_{}", self.eylem_derinligi);
        self.kiralik
            .as_mut()
            .ok_or_else(|| hata("PostgreSQL transaction bağlantısı kayıp"))?
            .batch_execute(&sorgu)
            .map_err(|h| self.sorgu_hatasi("PostgreSQL transaction'ı başlatılamadı", h))?;
        self.eylem_derinligi += 1;
        Ok(())
    }

    pub fn eylem_tamamla(&mut self) -> Result<(), VeritabaniHatasi> {
        self.eylemi_denetle()?;
        debug_assert!(!yeniden_baglanabilir(
            SorguSinifi::Commit,
            self.eylem_derinligi,
            self.kiralik.as_ref().is_some_and(|k| k.is_closed()),
            0,
            false,
        ));
        let Some(yeni_derinlik) = self.eylem_derinligi.checked_sub(1) else {
            return Err(hata("Açık PostgreSQL transaction'ı yok"));
        };
        let sorgu = if yeni_derinlik == 0 {
            "COMMIT".to_string()
        } else {
            format!("RELEASE SAVEPOINT zee_{}", yeni_derinlik)
        };
        if yeni_derinlik == 0 && self.sure_ayari_etkin {
            // K-375: COMMIT son tarih bütçesiyle iptal EDİLMEZ (senkron
            // replikasyon beklemesinde iptal yerel commit'i belirsiz bırakır).
            // Hata COMMIT'e bırakılır; sonucu C027 sınıflandırması karşılar.
            if let Some(istemci) = self.kiralik.as_mut() {
                let _ = istemci.batch_execute(SURE_AYARI_VARSAYILAN);
            }
            self.sure_ayari_etkin = false;
        }
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
            return Err(self.sorgu_hatasi("PostgreSQL savepoint'i tamamlanamadı", h));
        }
        self.eylem_derinligi = yeni_derinlik;
        if yeni_derinlik == 0 {
            self.kiralik.take();
        }
        Ok(())
    }

    pub fn eylem_geri_al(&mut self) -> Result<(), VeritabaniHatasi> {
        if let Some(mut h) = self.gecersiz_eylem.take() {
            self.kiralik.take();
            self.eylem_derinligi = 0;
            self.istek_baglami_acik = false;
            h.mesaj = "PostgreSQL bağlantısı kapalı; yerel transaction temizlendi, ROLLBACK onayı alınamadı".into();
            h.veri.push(("geri_alma_onayi".into(), "false".into()));
            return Err(h);
        }
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
            .map_err(|h| self.sorgu_hatasi("PostgreSQL transaction'ı geri alınamadı", h))?;
        // ROLLBACK TO SAVEPOINT, savepoint içindeki varsayılana dönüşü de geri alır.
        self.sure_ayari_etkin |= self.sure_ayari_transactionda;
        self.eylem_derinligi = yeni_derinlik;
        if yeni_derinlik == 0 {
            self.kiralik.take();
        }
        Ok(())
    }
    fn eylemi_denetle(&self) -> Result<(), VeritabaniHatasi> {
        self.gecersiz_eylem.clone().map_or(Ok(()), Err)
    }

    fn sorgu_hatasi(&mut self, on: &str, h: postgres::Error) -> VeritabaniHatasi {
        if let Some(hata) = wire_hatasi(&h) {
            if self.eylem_derinligi > 0 {
                self.kiralik.take();
                self.gecersiz_eylem = Some(hata.clone());
            }
            return hata;
        }
        postgres_hatasi(on, h)
    }
}

/// K-375: transaction dışı tek okuma için `BEGIN; SET LOCAL …` → sorgu → `COMMIT`.
/// Hata yolunda bağlantı açıksa `ROLLBACK` ile havuza temiz döner; kapalıysa
/// r2d2 onu bozuk sayıp atar. Oturum düzeyinde ayar hiç yapılmaz.
fn tek_sureli_okuma(
    istemci: &mut postgres::Client,
    ayar: &str,
    sorgu: &str,
    baglar: &[(&(dyn ToSql + Sync), Type)],
) -> Result<crate::veritabani_modeli::VeritabaniOkumaSonucu, SureliOkumaHatasi> {
    let sonuc = (|| {
        istemci.batch_execute(&format!("BEGIN; {ayar}"))?;
        let sonuc = sonuc::oku(istemci, sorgu, baglar)?;
        istemci.batch_execute("COMMIT")?;
        Ok(sonuc)
    })();
    if sonuc.is_err() && !istemci.is_closed() {
        if let Err(h) = istemci.batch_execute("ROLLBACK") {
            return Err(SureliOkumaHatasi::Zee(postgres_hatasi(
                "PostgreSQL okuma transaction'ı geri alınamadı",
                h,
            )));
        }
    }
    sonuc.map_err(SureliOkumaHatasi::Surucu)
}

/// K-375: son tarih iptali bağlantı kaybı değildir; sorgu sunucuda
/// çalışmış olabilir ve süre bitmiştir, otomatik tekrar YASAKTIR.
fn yeniden_baglanabilir(
    sinif: SorguSinifi,
    eylem_derinligi: usize,
    istemci_kapali: bool,
    yeniden_deneme_sayisi: usize,
    son_tarih_iptali: bool,
) -> bool {
    sinif == SorguSinifi::Okuma
        && eylem_derinligi == 0
        && istemci_kapali
        && yeniden_deneme_sayisi == 0
        && !son_tarih_iptali
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

fn projenin_goclerini_topla(
    bildirim: &VeritabaniBildirimi,
    proje_koku: &std::path::Path,
) -> Result<Vec<(i64, String, String, String)>, VeritabaniHatasi> {
    let proje_koku = std::fs::canonicalize(proje_koku)
        .map_err(|h| hata(format!("Proje kökü çözülemedi: {}", h)))?;
    let goc_koku = std::fs::canonicalize(proje_koku.join(&bildirim.gocler))
        .map_err(|h| hata(format!("Migration klasörü çözülemedi: {}", h)))?;
    if !goc_koku.starts_with(&proje_koku) || !goc_koku.is_dir() {
        return Err(hata(
            "Migration klasörü proje kökü içinde gerçek bir klasör olmalı",
        ));
    }
    goc_dosyalarini_topla(&goc_koku)
}

fn goc_kayitlarini_oku(
    istemci: &mut impl postgres::GenericClient,
) -> Result<Vec<postgres::Row>, VeritabaniHatasi> {
    istemci
        .query(
            "SELECT surum, ad, sha256 FROM _zee_gocleri ORDER BY surum",
            &[],
        )
        .map_err(|h| postgres_hatasi("Migration kayıtları okunamadı", h))
}

fn goc_kayitlarini_guvenli_coz(
    kayitlar: Vec<postgres::Row>,
) -> Result<std::collections::BTreeMap<i64, (String, String)>, VeritabaniHatasi> {
    let mut uygulananlar = std::collections::BTreeMap::new();
    for kayit in kayitlar {
        let surum = kayit
            .try_get::<_, i64>(0)
            .map_err(|h| postgres_hatasi("Migration sürümü okunamadı", h))?;
        let ad = kayit
            .try_get::<_, String>(1)
            .map_err(|h| postgres_hatasi("Migration adı okunamadı", h))?;
        let sha256 = kayit
            .try_get::<_, String>(2)
            .map_err(|h| postgres_hatasi("Migration özeti okunamadı", h))?;
        uygulananlar.insert(surum, (ad, sha256));
    }
    Ok(uygulananlar)
}

fn goc_planini_kur(
    gocler: &[(i64, String, String, String)],
    uygulananlar: &std::collections::BTreeMap<i64, (String, String)>,
) -> Result<GocPlani, VeritabaniHatasi> {
    if uygulananlar
        .keys()
        .any(|surum| !gocler.iter().any(|goc| &goc.0 == surum))
    {
        return Err(hata(
            "Veritabanında kayıtlı bir migration dosyası proje ağacında yok",
        ));
    }
    let mut plan = GocPlani {
        adimlar: Vec::with_capacity(gocler.len()),
        uygulanmis: 0,
        bekleyen: 0,
    };
    for (surum, ad, ozet, _) in gocler {
        let uygulanmis = if let Some((eski_ad, eski_ozet)) = uygulananlar.get(surum) {
            if eski_ad != ad || eski_ozet != ozet {
                return Err(verili_hata(
                    format!("{} uygulanmış migration içeriğiyle uyuşmuyor", ad),
                    vec![("surum".into(), surum.to_string())],
                ));
            }
            plan.uygulanmis += 1;
            true
        } else {
            plan.bekleyen += 1;
            false
        };
        plan.adimlar.push(GocAdimi {
            surum: *surum,
            ad: ad.clone(),
            sha256: ozet.clone(),
            uygulanmis,
        });
    }
    Ok(plan)
}

/// Kayıt tablosu yoksa da sadece read-only snapshot alır; SQL/DDL/DML uygulamaz.
/// `--durum` ve `--kuru` aynı dosya, sürüm, ad ve hash sözleşmesini paylaşır.
pub fn gocleri_planla(
    bildirim: &VeritabaniBildirimi,
    proje_koku: &std::path::Path,
) -> Result<GocPlani, VeritabaniHatasi> {
    let gocler = projenin_goclerini_topla(bildirim, proje_koku)?;
    let havuz = havuz::kur(bildirim)?;
    let mut istemci = havuz::al(&havuz)?;
    let mut tx = istemci
        .build_transaction()
        .isolation_level(postgres::IsolationLevel::RepeatableRead)
        .read_only(true)
        .start()
        .map_err(|h| postgres_hatasi("Migration durum transaction'ı başlatılamadı", h))?;
    let tablo_var: bool = tx
        .query_one("SELECT to_regclass('_zee_gocleri') IS NOT NULL", &[])
        .map_err(|h| postgres_hatasi("Migration kayıt tablosu sorgulanamadı", h))?
        .get(0);
    let kayitlar = if tablo_var {
        goc_kayitlarini_guvenli_coz(goc_kayitlarini_oku(&mut tx)?)?
    } else {
        std::collections::BTreeMap::new()
    };
    let plan = goc_planini_kur(&gocler, &kayitlar)?;
    tx.commit()
        .map_err(|h| postgres_hatasi("Migration durum transaction'ı tamamlanamadı", h))?;
    Ok(plan)
}

pub fn gocleri_uygula(
    bildirim: &VeritabaniBildirimi,
    proje_koku: &std::path::Path,
) -> Result<GocRaporu, VeritabaniHatasi> {
    let gocler = projenin_goclerini_topla(bildirim, proje_koku)?;
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
    // Existing apply metadata decoding is retained exactly. Malformed schema
    // panic correction requires its own bugfix/reproducer/provenance slice.
    let kayitlar = goc_kayitlarini_oku(&mut tx)?
        .into_iter()
        .map(|kayit| {
            (
                kayit.get::<_, i64>(0),
                (kayit.get::<_, String>(1), kayit.get::<_, String>(2)),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let plan = goc_planini_kur(&gocler, &kayitlar)?;
    let mut rapor = GocRaporu {
        uygulanan: 0,
        atlanan: plan.uygulanmis,
    };
    for ((surum, ad, ozet, sql), adim) in gocler.into_iter().zip(plan.adimlar) {
        if adim.uygulanmis {
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
    let kelimeler = migration_ust_seviye_kelimeleri(sql);
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

/// Transaction sınırı yalnız migration'ın üst seviye SQL komutlarında aranır.
/// String, yorum, quoted identifier ve dollar-quoted işlev gövdelerindeki
/// `BEGIN`/`COMMIT` sözcükleri SQL transaction komutu değildir.
fn migration_ust_seviye_kelimeleri(sql: &str) -> Vec<String> {
    let baytlar = sql.as_bytes();
    let mut kelimeler = Vec::new();
    let mut i = 0usize;
    while i < baytlar.len() {
        if baytlar[i] == b'\'' || baytlar[i] == b'"' {
            let tirnak = baytlar[i];
            i += 1;
            while i < baytlar.len() {
                if baytlar[i] == tirnak {
                    if i + 1 < baytlar.len() && baytlar[i + 1] == tirnak {
                        i += 2;
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            continue;
        }
        if baytlar[i..].starts_with(b"--") {
            i += 2;
            while i < baytlar.len() && baytlar[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if baytlar[i..].starts_with(b"/*") {
            i += 2;
            let mut derinlik = 1usize;
            while i < baytlar.len() && derinlik > 0 {
                if baytlar[i..].starts_with(b"/*") {
                    derinlik += 1;
                    i += 2;
                } else if baytlar[i..].starts_with(b"*/") {
                    derinlik -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        if baytlar[i] == b'$' {
            let mut etiket_sonu = i + 1;
            let ilk_uygun = etiket_sonu >= baytlar.len()
                || baytlar[etiket_sonu] == b'$'
                || baytlar[etiket_sonu] == b'_'
                || baytlar[etiket_sonu].is_ascii_alphabetic();
            if ilk_uygun {
                while etiket_sonu < baytlar.len()
                    && (baytlar[etiket_sonu].is_ascii_alphanumeric()
                        || baytlar[etiket_sonu] == b'_')
                {
                    etiket_sonu += 1;
                }
                if etiket_sonu < baytlar.len() && baytlar[etiket_sonu] == b'$' {
                    let ayirici = &baytlar[i..=etiket_sonu];
                    i = etiket_sonu + 1;
                    if let Some(uzaklik) = baytlar[i..]
                        .windows(ayirici.len())
                        .position(|pencere| pencere == ayirici)
                    {
                        i += uzaklik + ayirici.len();
                    } else {
                        i = baytlar.len();
                    }
                    continue;
                }
            }
        }
        if baytlar[i].is_ascii_alphanumeric() || baytlar[i] == b'_' {
            let baslangic = i;
            while i < baytlar.len() && (baytlar[i].is_ascii_alphanumeric() || baytlar[i] == b'_') {
                i += 1;
            }
            kelimeler.push(String::from_utf8_lossy(&baytlar[baslangic..i]).to_ascii_lowercase());
            continue;
        }
        i += 1;
    }
    kelimeler
}

fn girdiyi_denetle(
    sorgu: &str,
    parametreler: &[String],
    parametre_bayti: usize,
) -> Result<(), VeritabaniHatasi> {
    let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.veritabani();
    if sorgu.is_empty() || sorgu.len() > sinirlar.sorgu_bayti() {
        return Err(hata("PostgreSQL sorgusu boş ya da 64 KiB sınırını aşıyor"));
    }
    if parametreler.len() > sinirlar.parametre()
        || parametreler.iter().any(|p| p.len() > parametre_bayti)
    {
        return Err(hata(format!(
            "PostgreSQL parametreleri 100 öğe / öğe başına {} KiB sınırını aşıyor",
            parametre_bayti / 1024
        )));
    }
    // K-391: tek tavan yükseltilse de toplam eski en kötü durumu aşamaz.
    if parametreler.iter().map(String::len).sum::<usize>() > TOPLAM_PARAMETRE_BAYTI {
        return Err(hata(format!(
            "PostgreSQL parametreleri toplam {} KiB sınırını aşıyor",
            TOPLAM_PARAMETRE_BAYTI / 1024
        )));
    }
    Ok(())
}

fn metin_baglari(parametreler: &[String]) -> Vec<(&(dyn ToSql + Sync), Type)> {
    parametreler
        .iter()
        .map(|p| (p as &(dyn ToSql + Sync), Type::TEXT))
        .collect()
}

fn wire_hatasi(h: &postgres::Error) -> Option<VeritabaniHatasi> {
    use postgres_protocol::message::backend::FrameLimitExceeded;
    use std::error::Error;
    let mut cause = h.source();
    while let Some(inner) = cause {
        let limit = inner.downcast_ref::<FrameLimitExceeded>().or_else(|| {
            inner
                .downcast_ref::<std::io::Error>()
                .and_then(std::io::Error::get_ref)
                .and_then(|v| v.downcast_ref::<FrameLimitExceeded>())
        });
        if let Some(limit) = limit {
            return Some(verili_hata("PostgreSQL yanıt mesajı 32 MiB sınırını aşıyor; bağlantı kapatıldı, otomatik tekrar yapılmadı",
                vec![("hata_sinifi".into(), "db.response_frame_limit".into()),
                     ("sinir_bayt".into(), limit.limit.to_string()),
                     ("bildirilen_bayt".into(), limit.announced.to_string())]));
        }
        cause = inner.source();
    }
    None
}

fn postgres_hatasi(on: &str, hata_degeri: postgres::Error) -> VeritabaniHatasi {
    if let Some(h) = wire_hatasi(&hata_degeri) {
        return h;
    }
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
    sonuc.veri.retain(|(ad, _)| ad != "hata_sinifi");
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

    /// K-391 (ADR-168, ZM-F036): tek parametre varsayılan 64 KiB; proje
    /// tavanı 4 MiB'a kadar yükseltir, toplam eski en kötü 6400 KiB'ı aşamaz.
    #[test]
    fn parametre_tavani_varsayilan_ve_proje_yukseltmesi_toplamla_sinirli() {
        let kib = 1024;
        let sorgu = "SELECT $1";
        let varsayilan = 64 * kib;
        assert!(girdiyi_denetle(sorgu, &["a".repeat(varsayilan)], varsayilan).is_ok());
        let red = girdiyi_denetle(sorgu, &["a".repeat(varsayilan + 1)], varsayilan).unwrap_err();
        assert!(red.mesaj.contains("öğe başına 64 KiB"), "{}", red.mesaj);
        // Türkçe-yoğun 1 MiB ileti gövdesi (2 baytlık harfler) tek parametre.
        let ileti = "ışığı".repeat(1024 * kib / "ışığı".len());
        assert!(ileti.len() > varsayilan);
        assert!(girdiyi_denetle(sorgu, std::slice::from_ref(&ileti), 1024 * kib).is_ok());
        let red = girdiyi_denetle(sorgu, &[ileti], varsayilan).unwrap_err();
        assert!(red.mesaj.contains("64 KiB"), "{}", red.mesaj);
        let alti = vec!["a".repeat(1024 * kib); 6];
        assert!(girdiyi_denetle(sorgu, &alti, 4096 * kib).is_ok());
        let yedi = vec!["a".repeat(1024 * kib); 7];
        let red = girdiyi_denetle(sorgu, &yedi, 4096 * kib).unwrap_err();
        assert!(red.mesaj.contains("toplam 6400 KiB"), "{}", red.mesaj);
        assert_eq!(TOPLAM_PARAMETRE_BAYTI, 100 * 64 * kib);
    }

    #[test]
    fn goc_plani_uygulanmis_ve_bekleyeni_ayni_sirali_kaynaktan_ayirir() {
        let gocler = vec![
            (1, "0001_ilk.sql".into(), "aaa".into(), "SELECT 1".into()),
            (3, "0003_son.sql".into(), "ccc".into(), "SELECT 3".into()),
        ];
        let kayitlar =
            std::collections::BTreeMap::from([(1, ("0001_ilk.sql".into(), "aaa".into()))]);
        let plan = goc_planini_kur(&gocler, &kayitlar).unwrap();
        assert_eq!((plan.uygulanmis, plan.bekleyen), (1, 1));
        assert_eq!(
            plan.adimlar.iter().map(|a| a.surum).collect::<Vec<_>>(),
            [1, 3]
        );
        assert!(plan.adimlar[0].uygulanmis);
        assert!(!plan.adimlar[1].uygulanmis);
        assert_eq!(plan.adimlar[1].sha256, "ccc");
    }

    #[test]
    fn goc_plani_degismis_ad_ozet_ve_kayip_dosyayi_reddeder() {
        let gocler = vec![(1, "0001_ilk.sql".into(), "aaa".into(), "SELECT 1".into())];
        for (ad, hash) in [("0001_degismis.sql", "aaa"), ("0001_ilk.sql", "bbb")] {
            let kayitlar = std::collections::BTreeMap::from([(1, (ad.into(), hash.into()))]);
            let h = goc_planini_kur(&gocler, &kayitlar).unwrap_err();
            assert_eq!(h.veri, [("surum".into(), "1".into())]);
            assert_eq!(
                h.mesaj,
                "0001_ilk.sql uygulanmış migration içeriğiyle uyuşmuyor"
            );
        }
        let kayip =
            std::collections::BTreeMap::from([(2, ("0002_kayip.sql".into(), "bbb".into()))]);
        assert_eq!(
            goc_planini_kur(&gocler, &kayip).unwrap_err().mesaj,
            "Veritabanında kayıtlı bir migration dosyası proje ağacında yok"
        );
    }

    #[test]
    fn izole_wire_siniri_retry_transaction_rls_ve_commit_sinirini_korur() {
        let Ok(target) = std::env::var("ZEE_POSTGRES_K206_HEDEFI") else {
            return;
        };
        let url = std::env::var("ZEE_POSTGRES_K206_URL").unwrap();
        let hedef = VeritabaniHedefi::ayristir(&target).unwrap();
        assert!(hedef.loopback_mi());
        let bildirim = VeritabaniBildirimi {
            hedef,
            baglanti_degiskeni: "ZEE_POSTGRES_K206_URL".into(),
            gocler: "unused".into(),
            istek_baglami_cozucusu: None,
        };
        let mut admin = postgres::Client::connect(&url, postgres::NoTls).unwrap();
        admin.batch_execute("CREATE SEQUENCE zee_wire_attempts; CREATE TABLE zee_wire_probe(value text); CREATE FUNCTION zee_wire_tenant(text) RETURNS TABLE(tenant_id uuid) LANGUAGE SQL AS $$ SELECT '00000000-0000-0000-0000-000000000001'::uuid $$;
            CREATE FUNCTION zee_wire_notice() RETURNS text LANGUAGE plpgsql AS $$ BEGIN RAISE NOTICE '%', repeat('x',67108864); RETURN 'notice'; END $$;
            CREATE FUNCTION zee_wire_commit_notice() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE NOTICE '%', repeat('x',67108864); RETURN NEW; END $$;
            CREATE TABLE zee_wire_commit(value text);
            CREATE CONSTRAINT TRIGGER zee_wire_deferred AFTER INSERT ON zee_wire_commit DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION zee_wire_commit_notice();").unwrap();
        let mut session = PostgresqlOturumu::baglan(&bildirim, 0).unwrap();
        let huge = "SELECT repeat('x',67108864 + nextval('zee_wire_attempts')::integer * 0)::text AS payload";
        let class = |h: &VeritabaniHatasi, expected: &str| {
            assert_eq!(h.veri.iter().filter(|(k, _)| k == "hata_sinifi").count(), 1);
            assert!(
                h.veri
                    .iter()
                    .any(|(k, v)| k == "hata_sinifi" && v == expected),
                "{h:?}"
            );
        };
        class(
            &session.oku(huge, &[]).unwrap_err(),
            "db.response_frame_limit",
        );
        assert!(session.oku("SELECT 'ok'::text", &[]).is_ok());
        assert_eq!(
            admin
                .query_one("SELECT last_value FROM zee_wire_attempts", &[])
                .unwrap()
                .get::<_, i64>(0),
            1,
            "outside transaction frame rejection must not replay the query"
        );
        for request in [false, true] {
            if request {
                session
                    .istek_baglamini_baslat("zee_wire_tenant", "synthetic")
                    .unwrap();
            } else {
                session.eylem_baslat().unwrap();
            }
            session.eylem_baslat().unwrap();
            session
                .degistir("INSERT INTO zee_wire_probe VALUES ($1)", &["before".into()])
                .unwrap();
            class(
                &session.oku(huge, &[]).unwrap_err(),
                "db.response_frame_limit",
            );
            assert!(session.kiralik.is_none());
            assert!(session.oku("SELECT 'blocked'::text", &[]).is_err());
            assert!(session
                .degistir(
                    "INSERT INTO zee_wire_probe VALUES ($1)",
                    &["blocked".into()]
                )
                .is_err());
            assert!(session.eylem_baslat().is_err());
            assert!(session.eylem_tamamla().is_err());
            let error = if request {
                session.istek_baglamini_geri_al().unwrap_err()
            } else {
                session.eylem_geri_al().unwrap_err()
            };
            assert!(error
                .veri
                .iter()
                .any(|(k, v)| k == "geri_alma_onayi" && v == "false"));
            assert_eq!(session.eylem_derinligi, 0);
            assert!(!session.istek_baglami_acik);
            assert!(session.gecersiz_eylem.is_none());
            assert_eq!(
                admin
                    .query_one("SELECT count(*) FROM zee_wire_probe", &[])
                    .unwrap()
                    .get::<_, i64>(0),
                0
            );
            assert!(session.oku("SELECT 'fresh'::text", &[]).is_ok());
        }
        session.eylem_baslat().unwrap();
        class(
            &session
                .degistir("INSERT INTO zee_wire_probe VALUES (zee_wire_notice())", &[])
                .unwrap_err(),
            "db.response_frame_limit",
        );
        assert!(session.eylem_geri_al().is_err());
        assert_eq!(session.eylem_derinligi, 0);
        session.eylem_baslat().unwrap();
        session
            .degistir(
                "INSERT INTO zee_wire_commit VALUES ($1)",
                &["commit-unknown".into()],
            )
            .unwrap();
        class(&session.eylem_tamamla().unwrap_err(), "db.commit_unknown");
        assert_eq!(session.eylem_derinligi, 0);
        assert!(session.gecersiz_eylem.is_none());
        session
            .istek_baglamini_baslat("zee_wire_tenant", "fresh-request")
            .unwrap();
        assert_eq!(
            session
                .oku(
                    "SELECT current_setting('app.tenant_id')::text AS tenant",
                    &[]
                )
                .unwrap()[0][0]
                .1,
            "00000000-0000-0000-0000-000000000001"
        );
        session.istek_baglamini_tamamla().unwrap();
        let error = session
            .oku(
                "SELECT repeat('x',1048576)::text FROM generate_series(1,64)",
                &[],
            )
            .unwrap_err();
        assert!(error.mesaj.contains("16 MiB"));
        assert!(session.oku("SELECT 'healthy'::text", &[]).is_ok());
    }

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
    fn migration_islev_govdesini_transaction_komutu_sanmaz() {
        let sql = r#"
            -- BEGIN yorumda transaction değildir
            CREATE FUNCTION cycle_guard() RETURNS trigger
            LANGUAGE plpgsql AS $body$
            BEGIN
                RAISE NOTICE 'COMMIT metni';
                RETURN NEW;
            END
            $body$;
            COMMENT ON FUNCTION cycle_guard() IS 'ROLLBACK değil';
            CREATE TABLE "savepoint" (id bigint);
            /* BEGIN /* ROLLBACK */ COMMIT */
        "#;
        assert!(migration_sqlini_denetle(sql).is_ok());
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
            assert_eq!(alanlar.len(), 6, "{} numaralı fixture satırı", sira + 1);
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
                alanlar[4].parse().expect("son tarih iptali bool olmalı"),
            );
            assert_eq!(gercek, alanlar[5] == "retry", "fixture satırı: {satir}");
        }
        assert_eq!(vaka_sayisi, 7, "recovery karar matrisi eksilmemeli");
    }
}
