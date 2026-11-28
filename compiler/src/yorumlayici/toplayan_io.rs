//! Hermetik test ve kütüphane adaptörünün deterministik IO durumu.

use super::*;

/// zee-io-1 hermetik profilinin sabit gizli anahtarı (yalnız test/oynatma; üretim anahtarı ortam değişkenindedir).
const HERMETIK_GIZLI_ANAHTAR: [u8; 32] = [0x5a; 32];

/// Eylem başı metin ve ikili dosya tabloları.
type DosyaTablosuYedegi = (HashMap<String, String>, HashMap<String, Vec<u8>>);

pub struct ToplayanIo {
    pub girdiler: VecDeque<String>,
    pub rastgele_degerler: VecDeque<i64>,
    /// Sahte dosya sistemi: yol → içerik (testlerde determinizm).
    pub dosyalar: HashMap<String, String>,
    /// K-389: çözerek yazılan bayt-kesin dosyalar (UTF-8 olmayabilir).
    pub ikili_dosyalar: HashMap<String, Vec<u8>>,
    /// Sabit "şimdi" — testlerde determinizm.
    pub zaman: (i64, u32, u32, u32, u32),
    pub argumanlar: Vec<String>,
    /// Sahte HTTP: url → (durum, gövde).
    pub http_yanitlari: HashMap<String, (i64, String)>,
    /// Gerçekten başlatılan sahte HTTP çağrıları ve çağrı anındaki kalan süre.
    pub http_istekleri: Vec<(String, Option<i64>)>,
    /// Hermetik PostgreSQL cevap kuyrukları ve gerçekten bind edilen istekler.
    pub postgresql: ToplayanVeritabani,
    /// Sahte sunucu: istek kuyruğu ve (istek → yanıt) kayıtları.
    pub istekler: VecDeque<String>,
    /// Sunucunun Set-Cookie ile yazdığı çerezler (K-052 testleri için).
    pub yazilan_cerezler: Vec<(String, String)>,
    /// K-088 güvenli çerez niteliklerinin hermetik kanıtı.
    pub guvenli_cerezler: Vec<GuvenliCerezKaydi>,
    pub sunucu_yanitlari: Vec<(String, String)>,
    /// Her sahte HTTP yanıtının durum kodu; istek alındığında 200 ile başlar.
    pub sunucu_durumlari: Vec<u16>,
    /// K-335: eşleşen rotanın bildirdiği önbellek izinleri (saniye).
    pub onbellek_izinleri: Vec<u32>,
    /// K-378: hermetik dosya yanıtları — (Content-Type, Content-Disposition).
    /// Gövde `sunucu_yanitlari`na dosya içeriği olarak yazılır.
    pub dosya_yanitlari: Vec<(String, String)>,
    /// K-406: hermetik türlü metin yanıtlarının `Content-Type` değerleri.
    /// Gövde `sunucu_yanitlari`na bayt bayt yazılır.
    pub turlu_yanitlar: Vec<String>,
    /// RFC-0047 A (ADR-182): 405 yanıtlarının `Allow` değerleri, sırayla.
    pub izinli_yontemler: Vec<String>,
    /// RFC-0047 (ADR-182 dilim C): DAV isteklerinin `kullanıcı:parola` Basic
    /// kimlikleri (`None` = başlık yok), DAV rotası çağrıldıkça sırayla.
    pub dav_basic: VecDeque<Option<String>>,
    /// `dav_kimlik_çözücüsü` bildirimi ve hermetik HMAC anahtarı (`dene`).
    pub dav_kimlik_cozucusu: Option<String>,
    /// 401/429 kararları ve OPTIONS `Allow` değerleri, sırayla.
    pub dav_kararlari: Vec<String>,
    /// RFC-0047 dilim D: DAV isteklerinin `Depth`/`If-Match`/`If-None-Match`
    /// başlıkları (kimlik çözüldükçe sırayla) ve yazılan ETag'ler.
    pub dav_basliklari: VecDeque<Vec<(String, String)>>,
    pub etagler: Vec<String>,
    dav_gecerli_basliklar: Vec<(String, String)>,
    dav_parolasi: Option<(String, String)>,
    /// Operatör günlüğüne düşen istek tanıları: (durum, kod, satır) (K-324).
    pub istek_tani_gunlugu: Vec<(u16, String, usize)>,
    /// Sahte sensörler (varsayılan kapalı) ve an ölçümü kuyruğu.
    pub sensorler: HashMap<String, bool>,
    pub an_degerleri: VecDeque<i64>,
    pub eylem_baslat_sonuclari: VecDeque<Result<(), EylemHatasi>>,
    pub eylem_tamamla_sonuclari: VecDeque<Result<(), EylemHatasi>>,
    pub dosya_yaz_sonuclari: VecDeque<Result<(), String>>,
    /// K-381: betiklenmiş bağlantılar ve sunucunun yazdıkları.
    pub baglanti: super::toplayan_baglanti::ToplayanBaglanti,
    /// K-210: sahte SMTP giden kutusu — (alıcı, konu, gövde).
    pub eposta_kutusu: Vec<(String, String, String)>,
    /// K-390: sahte çözücü bölgesi — (kanonik ad, tür) → yanıt. Kayıtsız
    /// sorgu `alan_yok` döner (sahte çözücü yetkili ve boş bölgedir).
    pub dns_bolgesi: HashMap<(String, crate::dns_teli::KayitTuru), DnsSahteYaniti>,
    /// K-390: gerçekten yapılan sahte DNS sorguları ve verilen süre (ms).
    pub dns_sorgulari: Vec<(String, crate::dns_teli::KayitTuru, i64)>,
    /// K-390 B1b: `kurumsal alanı` için kamu son ek listesi (yoksa hata).
    pub kamu_son_ekleri: Option<std::sync::Arc<crate::kamu_son_ekleri::KamuSonEkleri>>,
    /// K-397 B3: `imzalamayı dene` için DKIM anahtarları (yoksa `anahtar_bilinmiyor`).
    #[cfg(not(target_arch = "wasm32"))]
    pub dkim_anahtarlari: Option<std::sync::Arc<crate::dkim_imza::DkimAnahtarDeposu>>,
    an_son_degeri: i64,
    pub cikti: Vec<String>,
    /// Eylem başı dosya tabloları (K-389: ikili tablo da geri alınır).
    eylem_yedekleri: Vec<DosyaTablosuYedegi>,
    web_guvenligi: WebGuvenligi,
    guvenli_belirtec_sirasi: u64,
    web_istek_yedegi: Option<ToplayanWebIstekYedegi>,
    bekleyen_sunucu_yaniti: Option<SunucuYanitTaslagi>,
}

/// K-390: hermetik DNS yanıtı.
pub type DnsSahteYaniti = Result<crate::dns_teli::Cozum, crate::dns_teli::DnsHatasi>;

struct ToplayanWebIstekYedegi {
    yazilan_cerez_sayisi: usize,
    guvenli_cerez_sayisi: usize,
    /// K-330: kayar kipte bu isteğin ham oturum belirteci; commit'te çerez yenilenir.
    kayar_oturum_belirteci: Option<String>,
}

enum SunucuYanitTaslagi {
    Govde(String),
    Durum(u16, String),
    Yonlendirme(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuvenliCerezKaydi {
    pub ad: String,
    pub deger: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: &'static str,
    pub yol: &'static str,
    pub azami_omur_saniye: i64,
}

impl ToplayanIo {
    /// K-375: hermetik PostgreSQL'in son tarih bütçesine uyan sanal süresi.
    fn postgresql_suresini_isle(
        &mut self,
        zaman_asimi_ms: Option<i64>,
    ) -> Option<VeritabaniHatasi> {
        self.postgresql.sureler.push(zaman_asimi_ms);
        let sure = self.postgresql.sorgu_suresi_ms?;
        let gecen = zaman_asimi_ms.map_or(sure, |butce| butce.min(sure)).max(0);
        self.an_son_degeri = self.an_son_degeri.saturating_add(gecen);
        zaman_asimi_ms
            .filter(|butce| *butce < sure)
            .map(|_| super::veritabani::son_tarih_iptali("57014"))
    }

    pub fn yeni(girdiler: Vec<String>) -> ToplayanIo {
        ToplayanIo {
            girdiler: girdiler.into(),
            rastgele_degerler: VecDeque::new(),
            dosyalar: HashMap::new(),
            ikili_dosyalar: HashMap::new(),
            zaman: (2026, 8, 31, 14, 30),
            argumanlar: Vec::new(),
            http_yanitlari: HashMap::new(),
            http_istekleri: Vec::new(),
            postgresql: ToplayanVeritabani::default(),
            istekler: VecDeque::new(),
            yazilan_cerezler: Vec::new(),
            guvenli_cerezler: Vec::new(),
            sunucu_yanitlari: Vec::new(),
            sunucu_durumlari: Vec::new(),
            onbellek_izinleri: Vec::new(),
            dosya_yanitlari: Vec::new(),
            turlu_yanitlar: Vec::new(),
            izinli_yontemler: Vec::new(),
            dav_basic: VecDeque::new(),
            dav_kimlik_cozucusu: None,
            dav_kararlari: Vec::new(),
            dav_basliklari: VecDeque::new(),
            etagler: Vec::new(),
            dav_gecerli_basliklar: Vec::new(),
            dav_parolasi: None,
            istek_tani_gunlugu: Vec::new(),
            sensorler: HashMap::new(),
            an_degerleri: VecDeque::new(),
            eylem_baslat_sonuclari: VecDeque::new(),
            eylem_tamamla_sonuclari: VecDeque::new(),
            dosya_yaz_sonuclari: VecDeque::new(),
            eposta_kutusu: Vec::new(),
            dns_bolgesi: HashMap::new(),
            dns_sorgulari: Vec::new(),
            kamu_son_ekleri: None,
            #[cfg(not(target_arch = "wasm32"))]
            dkim_anahtarlari: None,
            baglanti: Default::default(),
            an_son_degeri: 0,
            cikti: Vec::new(),
            eylem_yedekleri: Vec::new(),
            web_guvenligi: WebGuvenligi::yeni(),
            guvenli_belirtec_sirasi: 0,
            web_istek_yedegi: None,
            bekleyen_sunucu_yaniti: None,
        }
    }

    /// K-330/ADR-158 eki: proje bildirimindeki kimlikli oturum ömrü/kayar
    /// ayarını hermetik adaptöre uygular; `dil dene` gerçek runtime ile aynı
    /// `OturumAyari`'yı kullanır.
    pub fn oturum_ayarini_belirle(&mut self, ayar: crate::web_guvenligi::OturumAyari) {
        self.web_guvenligi.oturum_ayarini_belirle(ayar);
    }

    fn oturum_cerezini_yaz(&mut self, yeni: YeniOturum) {
        let ad = "__Host-zee-oturum".to_string();
        self.yazilan_cerezler
            .push((ad.clone(), yeni.belirtec.clone()));
        self.guvenli_cerezler.push(GuvenliCerezKaydi {
            ad,
            deger: yeni.belirtec,
            secure: true,
            http_only: true,
            same_site: "Lax",
            yol: "/",
            azami_omur_saniye: yeni.azami_omur_saniye,
        });
    }

    fn sunucu_yanitini_uygula(&mut self, taslak: SunucuYanitTaslagi) {
        match taslak {
            SunucuYanitTaslagi::Govde(govde) => {
                if let Some((_, yanit)) = self.sunucu_yanitlari.last_mut() {
                    *yanit = govde;
                }
            }
            SunucuYanitTaslagi::Durum(durum, govde) => {
                if let Some(son) = self.sunucu_durumlari.last_mut() {
                    *son = durum;
                }
                if let Some((_, yanit)) = self.sunucu_yanitlari.last_mut() {
                    *yanit = govde;
                }
            }
            SunucuYanitTaslagi::Yonlendirme(adres) => {
                if let Some((_, yanit)) = self.sunucu_yanitlari.last_mut() {
                    *yanit = format!("→ {}", adres);
                }
            }
        }
    }
}

impl GirdiCikti for ToplayanIo {
    fn yazdir(&mut self, satir: String) {
        self.cikti.push(satir);
    }
    fn sor(&mut self, istem: &str) -> Option<String> {
        self.cikti.push(istem.to_string());
        self.girdiler.pop_front()
    }
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        if alt > ust {
            return alt;
        }
        self.rastgele_degerler
            .pop_front()
            .unwrap_or(alt)
            .clamp(alt, ust)
    }
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        self.dosyalar
            .get(yol)
            .cloned()
            .ok_or_else(|| format!("\"{}\" dosyası bulunamadı", yol))
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        if let Some(sonuc) = self.dosya_yaz_sonuclari.pop_front() {
            sonuc?;
        }
        let girdi = self.dosyalar.entry(yol.to_string()).or_default();
        if !ekleme {
            girdi.clear();
        }
        girdi.push_str(satir);
        girdi.push('\n');
        Ok(())
    }
    fn dosya_atomik_tasi(&mut self, kaynak: &str, hedef: &str) -> Result<i64, String> {
        if self.dosyalar.contains_key(hedef) {
            return Err(format!("\"{}\" hedef dosyası zaten var", hedef));
        }
        let icerik = self
            .dosyalar
            .remove(kaynak)
            .ok_or_else(|| format!("\"{}\" kaynak dosyası bulunamadı", kaynak))?;
        self.dosyalar.insert(hedef.into(), icerik);
        Ok(1)
    }
    fn dosya_sil(&mut self, yol: &str) -> Result<i64, String> {
        Ok(i64::from(self.dosyalar.remove(yol).is_some()))
    }
    fn dosyalari_listele(&mut self, dizin: &str) -> Result<Vec<String>, String> {
        let onek = if dizin.is_empty() {
            String::new()
        } else {
            format!("{}/", dizin.trim_end_matches('/'))
        };
        let mut yollar = self
            .dosyalar
            .keys()
            .filter(|yol| yol.starts_with(&onek) && !yol[onek.len()..].contains('/'))
            .cloned()
            .collect::<Vec<_>>();
        yollar.sort();
        Ok(yollar)
    }
    fn dosya_sha256(&mut self, yol: &str) -> Result<String, String> {
        let icerik = self
            .dosyalar
            .get(yol)
            .ok_or_else(|| format!("\"{}\" dosyası bulunamadı", yol))?;
        Ok(crate::guvenlik::sha256_hex(icerik.as_bytes()))
    }
    fn dosya_araligi_oku(&mut self, yol: &str, konum: i64, adet: i64) -> Result<Vec<u8>, String> {
        let baytlar = match self.ikili_dosyalar.get(yol) {
            Some(baytlar) => baytlar.as_slice(),
            None => self
                .dosyalar
                .get(yol)
                .map(String::as_bytes)
                .ok_or_else(|| format!("\"{}\" dosyası bulunamadı", yol))?,
        };
        let bas = usize::try_from(konum)
            .unwrap_or(usize::MAX)
            .min(baytlar.len());
        let adet = usize::try_from(adet).unwrap_or(0);
        Ok(baytlar[bas..].iter().take(adet).copied().collect())
    }
    fn dosyaya_bayt_yaz(
        &mut self,
        yol: &str,
        baytlar: &[u8],
        ekleme: bool,
    ) -> Result<(String, u64), String> {
        let var = self.dosyalar.contains_key(yol) || self.ikili_dosyalar.contains_key(yol);
        if !ekleme && var {
            return Err(format!("\"{}\" hedef dosyası zaten var", yol));
        }
        if ekleme && !self.eylem_yedekleri.is_empty() {
            return Err("çözerek eklemeyi dene eylem içinde kullanılamaz".into());
        }
        if ekleme && self.dosyalar.contains_key(yol) {
            return Err(format!("\"{}\" metin dosyasına bayt eklenemez", yol));
        }
        let hedef = self.ikili_dosyalar.entry(yol.to_string()).or_default();
        hedef.extend_from_slice(baytlar);
        Ok((crate::guvenlik::sha256_hex(hedef), hedef.len() as u64))
    }
    /// K-408: hermetik taraf aynı saf kodlamayla (`kodlama`) ikili hedefe ekler.
    fn dosyaya_base64_satirlari_ekle(&mut self, kaynak: &str, hedef: &str) -> Result<i64, String> {
        let ham = match self.ikili_dosyalar.get(kaynak) {
            Some(baytlar) => baytlar.clone(),
            None => self
                .dosyalar
                .get(kaynak)
                .map(|metin| metin.as_bytes().to_vec())
                .ok_or_else(|| format!("\"{kaynak}\" dosyası bulunamadı"))?,
        };
        if ham.len() as u64 > crate::kodlama::BASE64_SATIRLARI_TAVANI {
            return Err(format!(
                "\"{kaynak}\" kaynak dosyası 32 MiB base64 satırları tavanını aşıyor; hiçbir bayt yazılmadı"
            ));
        }
        if !self.eylem_yedekleri.is_empty() {
            return Err("base64 satırlarıyla eklemeyi dene eylem içinde kullanılamaz".into());
        }
        let eklenecek = crate::kodlama::base64_satirli_uzunluk(ham.len() as u64);
        let onceki = match self.dosyalar.get(hedef) {
            Some(metin) => metin.len(),
            None => self.ikili_dosyalar.get(hedef).map_or(0, Vec::len),
        } as u64;
        if onceki + eklenecek > crate::kodlama::BASE64_SATIRLARI_TAVANI {
            return Err(format!(
                "\"{hedef}\" dosyasına yazılamadı: hedefin toplam boyutu 32 MiB sınırını aşıyor"
            ));
        }
        let mut cikti = Vec::new();
        crate::kodlama::base64_satirlari_kodla(&ham, &mut cikti);
        // Base64 satırları ASCII'dir: metin hedef (ör. MIME başlıkları) metin kalır.
        match self.dosyalar.get_mut(hedef) {
            Some(metin) => metin.push_str(std::str::from_utf8(&cikti).unwrap_or_default()),
            None => self
                .ikili_dosyalar
                .entry(hedef.to_string())
                .or_default()
                .extend_from_slice(&cikti),
        }
        Ok(eklenecek as i64)
    }
    fn dosya_ilk_baytlari(&mut self, yol: &str, adet: i64) -> Result<String, String> {
        let icerik = self
            .dosyalar
            .get(yol)
            .ok_or_else(|| format!("\"{}\" dosyası bulunamadı", yol))?;
        let adet = usize::try_from(adet).unwrap_or(0);
        Ok(icerik
            .as_bytes()
            .iter()
            .take(adet)
            .map(|bayt| format!("{bayt:02x}"))
            .collect())
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.zaman
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.argumanlar.clone()
    }
    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        self.http_istekleri.push((url.to_string(), zaman_asimi_ms));
        self.http_yanitlari
            .get(url)
            .cloned()
            .ok_or_else(|| format!("\"{}\" adresine bağlanılamadı", url))
    }
    fn postgresql_oku(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
        zaman_asimi_ms: Option<i64>,
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        if let Some(hata) = self.postgresql_suresini_isle(zaman_asimi_ms) {
            self.postgresql
                .istekler
                .push((sorgu.into(), parametreler.to_vec(), false));
            return Err(hata);
        }
        self.postgresql.oku(sorgu, parametreler)
    }
    fn postgresql_degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
        zaman_asimi_ms: Option<i64>,
    ) -> Result<i64, VeritabaniHatasi> {
        if let Some(hata) = self.postgresql_suresini_isle(zaman_asimi_ms) {
            self.postgresql
                .istekler
                .push((sorgu.into(), parametreler.to_vec(), true));
            return Err(hata);
        }
        self.postgresql.degistir(sorgu, parametreler)
    }
    fn sunucu_kur(&mut self, _kapi: i64) -> Result<(), String> {
        Ok(())
    }
    fn istek_al(&mut self) -> Option<String> {
        let ham = self.istekler.pop_front()?;
        let (ilk_satir, cerez_satiri, _) = istek_zarfi_parcala(&ham);
        let hedef = ilk_satir
            .split_once(' ')
            .map(|(_, hedef)| hedef.trim())
            .unwrap_or_else(|| ilk_satir.trim());
        let yol = hedef.split_once('?').map(|(yol, _)| yol).unwrap_or(hedef);
        let cerezler = cerezleri_parcala(cerez_satiri);
        let belirtec = cerezler
            .iter()
            .find(|(ad, _)| ad == "__Host-zee-oturum" || ad == "zee-oturum")
            .map(|(_, deger)| deger.as_str());
        self.web_istek_yedegi = Some(ToplayanWebIstekYedegi {
            yazilan_cerez_sayisi: self.yazilan_cerezler.len(),
            guvenli_cerez_sayisi: self.guvenli_cerezler.len(),
            kayar_oturum_belirteci: None,
        });
        self.bekleyen_sunucu_yaniti = None;
        if self
            .web_guvenligi
            .istegi_baslat_kimlikle(belirtec, "hermetik", self.an_son_degeri)
            .is_err()
        {
            self.web_istek_yedegi = None;
            return None;
        }
        // K-330: gerçek runtime gibi kayar yenilemede aynı belirteç yeni Max-Age ile yazılır.
        if let Some(yedek) = &mut self.web_istek_yedegi {
            yedek.kayar_oturum_belirteci = self
                .web_guvenligi
                .kayar_cerez_omru()
                .and(belirtec)
                .filter(|deger| crate::web_guvenligi::cerez_degeri_gecerli(deger))
                .map(str::to_string);
        }
        let bilgi = self.web_guvenligi.oturum_bilgisi(self.an_son_degeri);
        let ham = dogrulanmis_oturumu_istege_ekle(&ham, bilgi.as_ref());
        self.sunucu_yanitlari.push((yol.to_string(), String::new()));
        self.sunucu_durumlari.push(200);
        Some(ham)
    }
    fn istek_islemini_tamamla(&mut self) -> Result<(), String> {
        let Some(yedek) = self.web_istek_yedegi.take() else {
            return Ok(());
        };
        if let Some(taslak) = self.bekleyen_sunucu_yaniti.take() {
            // Giriş/çıkış/iptal bu istekte kayar süresini düşürdüyse omur None'dır;
            // bu istekte yazılmış oturum çerezi varsa o kazanır (gerçek runtime ile aynı).
            let kayar = yedek
                .kayar_oturum_belirteci
                .zip(self.web_guvenligi.kayar_cerez_omru())
                .filter(|_| {
                    !self.guvenli_cerezler[yedek.guvenli_cerez_sayisi..]
                        .iter()
                        .any(|cerez| cerez.ad == "__Host-zee-oturum")
                });
            self.web_guvenligi.istegi_tamamla(self.an_son_degeri)?;
            if let Some((belirtec, omur)) = kayar {
                self.oturum_cerezini_yaz(YeniOturum {
                    belirtec,
                    azami_omur_saniye: omur,
                });
            }
            self.sunucu_yanitini_uygula(taslak);
        } else {
            self.web_guvenligi.istegi_geri_al();
            self.yazilan_cerezler.truncate(yedek.yazilan_cerez_sayisi);
            self.guvenli_cerezler.truncate(yedek.guvenli_cerez_sayisi);
        }
        Ok(())
    }
    fn istek_islemini_geri_al(&mut self) {
        if let Some(yedek) = self.web_istek_yedegi.take() {
            self.web_guvenligi.istegi_geri_al();
            self.yazilan_cerezler.truncate(yedek.yazilan_cerez_sayisi);
            self.guvenli_cerezler.truncate(yedek.guvenli_cerez_sayisi);
        }
        self.bekleyen_sunucu_yaniti = None;
    }
    fn yanit_gonder(&mut self, yanit: &str) {
        let taslak = SunucuYanitTaslagi::Govde(yanit.to_string());
        if self.web_istek_yedegi.is_some() {
            if self.bekleyen_sunucu_yaniti.is_none() {
                self.bekleyen_sunucu_yaniti = Some(taslak);
            }
        } else {
            self.sunucu_yanitini_uygula(taslak);
        }
    }
    fn dosya_yaniti_gonder(&mut self, yol: &str, tur: &str, ad: &str) -> Result<i64, String> {
        // Gerçek adaptörle aynı kayıt, ad ve tavan kuralı (K-378); sahte dosya
        // sistemi metin taşıdığından içerik bayt olarak aynen gönderilir.
        if self.web_istek_yedegi.is_none() {
            return Err("dosya yanıtı yalnız web isteği içinde gönderilebilir".into());
        }
        let kayit = crate::icerik_turu::web_dosya_turu(tur)
            .ok_or_else(|| format!("\"{tur}\" dosya yanıtı için kayıtlı içerik türü değil"))?;
        let disposition = crate::icerik_turu::ek_disposition(ad).map_err(str::to_string)?;
        let icerik = self
            .dosyalar
            .get(yol)
            .ok_or_else(|| format!("\"{}\" dosyası bulunamadı", yol))?
            .clone();
        if icerik.len() as u64 > crate::icerik_turu::WEB_DOSYA_YANITI_BAYTI {
            return Err(format!(
                "dosya yanıtı {} MiB sınırını aşıyor",
                crate::icerik_turu::WEB_DOSYA_YANITI_BAYTI / (1024 * 1024)
            ));
        }
        if self.bekleyen_sunucu_yaniti.is_some() {
            return Err("bu isteğin yanıtı zaten belirlendi".into());
        }
        self.dosya_yanitlari
            .push((crate::icerik_turu::baslik_degeri(kayit), disposition));
        let bayt = icerik.len() as i64;
        self.bekleyen_sunucu_yaniti = Some(SunucuYanitTaslagi::Govde(icerik));
        Ok(bayt)
    }
    fn turlu_yanit_gonder(&mut self, tur: &str, yanit: &str) {
        // Gerçek adaptörle aynı kapalı küme (K-406); kayıt dışı tür HTML
        // koklamasına düşmez, 500 olur.
        let Some(kayit) = crate::icerik_turu::web_metin_turu(tur) else {
            self.durum_yaniti_gonder(500, "istek işlenemedi; sunucu hatası");
            return;
        };
        if self.web_istek_yedegi.is_some() && self.bekleyen_sunucu_yaniti.is_some() {
            return;
        }
        self.turlu_yanitlar
            .push(crate::icerik_turu::baslik_degeri(kayit));
        let taslak = SunucuYanitTaslagi::Govde(yanit.to_string());
        if self.web_istek_yedegi.is_some() {
            self.bekleyen_sunucu_yaniti = Some(taslak);
        } else {
            self.sunucu_yanitini_uygula(taslak);
        }
    }
    fn turlu_durumlu_yanit_gonder(&mut self, tur: &str, durum: u16, yanit: &str) {
        // Gerçek adaptörle aynı kapalı kümeler (RFC-0047 C); kayıt dışı tür
        // ya da durum 500 olur.
        let Some(kayit) = crate::icerik_turu::web_metin_turu(tur)
            .filter(|_| crate::agac::TURLU_YANIT_DURUMLARI.contains(&durum))
        else {
            self.durum_yaniti_gonder(500, "istek işlenemedi; sunucu hatası");
            return;
        };
        if self.web_istek_yedegi.is_some() && self.bekleyen_sunucu_yaniti.is_some() {
            return;
        }
        self.turlu_yanitlar
            .push(crate::icerik_turu::baslik_degeri(kayit));
        let taslak = SunucuYanitTaslagi::Durum(durum, yanit.to_string());
        if self.web_istek_yedegi.is_some() {
            self.bekleyen_sunucu_yaniti = Some(taslak);
        } else {
            self.sunucu_yanitini_uygula(taslak);
        }
    }
    fn yontem_reddi_gonder(&mut self, izinli: &str, yanit: &str) {
        self.izinli_yontemler.push(izinli.to_string());
        self.durum_yaniti_gonder(405, yanit);
    }
    fn dav_kimligi(&mut self) -> Option<super::DavKimligi> {
        use crate::guvenlik::uygulama_parolasi as up;
        self.dav_gecerli_basliklar = self.dav_basliklari.pop_front().unwrap_or_default();
        let basic = self.dav_basic.pop_front().flatten()?;
        let (kullanici, parola) = basic.split_once(':')?;
        self.dav_parolasi = up::parolayi_coz(parola);
        Some(super::DavKimligi {
            kullanici: kullanici.to_string(),
            onek: self.dav_parolasi.as_ref().map(|(onek, _)| onek.clone()),
            cozucu: self.dav_kimlik_cozucusu.clone(),
        })
    }
    fn dav_ozet_dogrula(&mut self, ozet: Option<&str>) -> bool {
        use crate::guvenlik::uygulama_parolasi as up;
        let anahtarlar = [("dene".to_string(), HERMETIK_GIZLI_ANAHTAR)];
        let sahte = ("AAAAAAAA".to_string(), "A".repeat(up::GIZLI_KARAKTERI));
        let parola = self.dav_parolasi.take();
        let (onek, gizli) = parola.as_ref().unwrap_or(&sahte);
        up::ozet_dogrula(&anahtarlar, onek, gizli, ozet) && parola.is_some()
    }
    fn dav_kovasi(&mut self, kova: super::DavKova, anahtar: &str) -> Result<(), i64> {
        use crate::web_guvenligi::RateLimitTuru::{DavBasarili, DavBasarisiz, DavIstemci};
        let an = self.an_son_degeri;
        let guv = &self.web_guvenligi;
        let kararlar = match kova {
            super::DavKova::Bak => vec![
                guv.rate_limit_bak(DavBasarisiz, anahtar, an),
                guv.rate_limit_bak(DavIstemci, "", an),
            ],
            super::DavKova::Basarisiz => vec![
                guv.rate_limit_artir(DavBasarisiz, anahtar, an),
                guv.rate_limit_artir(DavIstemci, "", an),
            ],
            super::DavKova::Basarili => vec![guv.rate_limit_artir(DavBasarili, anahtar, an)],
        };
        match kararlar
            .into_iter()
            .find(|karar| !karar.as_ref().is_ok_and(|k| k.izinli))
        {
            Some(Ok(karar)) => Err(karar.yeniden_dene_saniye.max(1)),
            Some(Err(_)) => Err(1),
            None => Ok(()),
        }
    }
    fn dav_kimlik_reddi_gonder(&mut self) {
        self.dav_kararlari
            .push("401 WWW-Authenticate: Basic".into());
        self.durum_yaniti_gonder(401, super::DAV_KIMLIK_REDDI);
    }
    fn dav_kova_reddi_gonder(&mut self, saniye: i64) {
        self.dav_kararlari
            .push(format!("429 Retry-After: {saniye}"));
        self.durum_yaniti_gonder(429, super::DAV_KOVA_REDDI);
    }
    fn etag_yaz(&mut self, etag: &str) -> Result<(), String> {
        self.etagler.push(etag.to_string());
        Ok(())
    }
    fn dav_istek_basliklari(&mut self) -> Vec<(String, String)> {
        self.dav_gecerli_basliklar.clone()
    }
    fn secenekler_gonder(&mut self, izinli: &str) {
        self.dav_kararlari.push(format!("200 Allow: {izinli}"));
        self.durum_yaniti_gonder(200, "");
    }
    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        let taslak = SunucuYanitTaslagi::Durum(durum, yanit.to_string());
        if let Some(yedek) = &self.web_istek_yedegi {
            self.yazilan_cerezler.truncate(yedek.yazilan_cerez_sayisi);
            self.guvenli_cerezler.truncate(yedek.guvenli_cerez_sayisi);
            if self.bekleyen_sunucu_yaniti.is_none() {
                self.bekleyen_sunucu_yaniti = Some(taslak);
            }
        } else {
            self.sunucu_yanitini_uygula(taslak);
        }
    }
    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        if !crate::web_guvenligi::yerel_yonlendirme_gecerli(adres) {
            return Err("yönlendirme yalnız yerel `/...` adresine yapılabilir".into());
        }
        let taslak = SunucuYanitTaslagi::Yonlendirme(adres.to_string());
        if self.web_istek_yedegi.is_some() {
            if self.bekleyen_sunucu_yaniti.is_none() {
                self.bekleyen_sunucu_yaniti = Some(taslak);
            }
        } else {
            self.sunucu_yanitini_uygula(taslak);
        }
        Ok(())
    }
    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        if !crate::web_guvenligi::cerez_adi_gecerli(ad)
            || !crate::web_guvenligi::cerez_degeri_gecerli(deger)
        {
            return Err("çerez adı/değeri HTTP başlığı için güvenli değil".into());
        }
        self.yazilan_cerezler
            .push((ad.to_string(), deger.to_string()));
        Ok(())
    }
    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        if !crate::web_guvenligi::cerez_adi_gecerli(ad) {
            return Err("çerez adı HTTP başlığı için güvenli değil".into());
        }
        self.yazilan_cerezler
            .push((ad.to_string(), "×silindi".to_string()));
        Ok(())
    }
    fn yanit_onbellegine_izin_ver(&mut self, saniye: u32) {
        self.onbellek_izinleri.push(saniye);
    }
    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        if csrf_gerekli {
            let karar = self
                .web_guvenligi
                .rate_limit_artir(
                    crate::web_guvenligi::RateLimitTuru::Csrf,
                    "csrf",
                    self.an_son_degeri,
                )
                .map_err(|_| WebReddi {
                    durum: 503,
                    mesaj: "CSRF oran sınırı deposuna erişilemedi",
                })?;
            if !karar.izinli {
                return Err(WebReddi {
                    durum: 429,
                    mesaj: "çok fazla CSRF doğrulama isteği",
                });
            }
        }
        self.web_guvenligi
            .denetle(erisim, csrf, csrf_gerekli, self.an_son_degeri)
    }
    fn csrf_belirteci(&mut self) -> Result<String, String> {
        let mut sira = self.guvenli_belirtec_sirasi;
        let (csrf, yeni) = self.web_guvenligi.csrf_belirteci(self.an_son_degeri, || {
            sira = sira.saturating_add(1);
            Ok(format!("{:064x}", sira))
        })?;
        self.guvenli_belirtec_sirasi = sira;
        if let Some(yeni) = yeni {
            self.oturum_cerezini_yaz(yeni);
        }
        Ok(csrf)
    }
    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        let mut sira = self.guvenli_belirtec_sirasi;
        let yeni = self.web_guvenligi.oturum_ac(
            kullanici.to_string(),
            rol.to_string(),
            self.an_son_degeri,
            || {
                sira = sira.saturating_add(1);
                Ok(format!("{:064x}", sira))
            },
        )?;
        self.guvenli_belirtec_sirasi = sira;
        self.oturum_cerezini_yaz(yeni);
        Ok(())
    }
    fn oturum_kapat(&mut self) -> Result<(), String> {
        self.web_guvenligi.oturum_kapat();
        let ad = "__Host-zee-oturum".to_string();
        self.yazilan_cerezler
            .push((ad.clone(), "×silindi".to_string()));
        self.guvenli_cerezler.push(GuvenliCerezKaydi {
            ad,
            deger: String::new(),
            secure: true,
            http_only: true,
            same_site: "Lax",
            yol: "/",
            azami_omur_saniye: 0,
        });
        Ok(())
    }
    fn kullanici_oturumlarini_listele(
        &mut self,
        kullanici: &str,
    ) -> Result<Vec<Vec<(String, String)>>, String> {
        self.web_guvenligi
            .kullanici_oturumlarini_listele(kullanici, self.an_son_degeri)
            .map(|oturumlar| {
                oturumlar
                    .into_iter()
                    .map(crate::web_guvenligi::oturum_satiri)
                    .collect()
            })
    }
    fn oturum_kimligini_iptal_et(&mut self, kimlik: &str) -> Result<i64, String> {
        self.web_guvenligi.oturum_kimligini_iptal_et(kimlik)
    }
    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        if self.web_istek_yedegi.is_none() {
            return crate::guvenlik::parola_dogrula(parola, ozet);
        }
        let karar = self.web_guvenligi.rate_limit_artir(
            crate::web_guvenligi::RateLimitTuru::Giris,
            "parola",
            self.an_son_degeri,
        );
        match karar {
            Ok(karar) if karar.izinli => crate::guvenlik::parola_dogrula(parola, ozet),
            Ok(_) => {
                self.durum_yaniti_gonder(429, crate::web_guvenligi::GIRIS_ORAN_ASIMI);
                false
            }
            Err(_) => {
                self.durum_yaniti_gonder(503, crate::web_guvenligi::GIRIS_ORAN_DEPOSU_YOK);
                false
            }
        }
    }
    fn parola_ozeti_uret(&mut self, parola: &str) -> Result<String, String> {
        if self.web_istek_yedegi.is_some() {
            let karar = self.web_guvenligi.rate_limit_artir(
                crate::web_guvenligi::RateLimitTuru::Giris,
                "parola",
                self.an_son_degeri,
            );
            match karar {
                Ok(karar) if karar.izinli => {}
                Ok(_) => return Err(crate::web_guvenligi::GIRIS_ORAN_ASIMI.into()),
                Err(_) => return Err(crate::web_guvenligi::GIRIS_ORAN_DEPOSU_YOK.into()),
            }
        }
        // zee-io-1: tuz tohumlu sayaçtan gelir; aynı program aynı özeti üretir.
        self.guvenli_belirtec_sirasi = self.guvenli_belirtec_sirasi.saturating_add(1);
        let mut tuz = [0u8; 16];
        tuz[..8].copy_from_slice(&self.guvenli_belirtec_sirasi.to_be_bytes());
        tuz[8..].copy_from_slice(b"zee-io-1");
        crate::guvenlik::parola_ozeti_uret_tuzlu(parola, &tuz)
    }
    fn gizli_sifrele(&mut self, duz: &str) -> Result<String, String> {
        // zee-io-1: sabit test anahtarı, nonce tohumlu sayaçtan; aynı program aynı şifreli metin.
        self.guvenli_belirtec_sirasi += 1;
        let ozet = crate::guvenlik::sha256_hex(
            format!("zee-io-1-nonce-{}", self.guvenli_belirtec_sirasi).as_bytes(),
        );
        let mut nonce = [0u8; 24];
        nonce.copy_from_slice(&ozet.as_bytes()[..24]);
        crate::guvenlik::gizli::sifrele(&HERMETIK_GIZLI_ANAHTAR, &nonce, duz)
    }
    fn gizli_coz(&mut self, sifreli: &str) -> Result<String, String> {
        crate::guvenlik::gizli::coz(&HERMETIK_GIZLI_ANAHTAR, sifreli)
    }
    fn uygulama_parolasi_uret(
        &mut self,
    ) -> Result<crate::guvenlik::uygulama_parolasi::UretilenParola, String> {
        // zee-io-1: tohumlu sayaçtan 25 bayt, hermetik anahtar `dene`.
        self.guvenli_belirtec_sirasi = self.guvenli_belirtec_sirasi.saturating_add(1);
        let mut baytlar = [0u8; crate::guvenlik::uygulama_parolasi::RASTGELE_BAYT];
        baytlar[..8].copy_from_slice(&self.guvenli_belirtec_sirasi.to_be_bytes());
        baytlar[8..].copy_from_slice(b"zee-io-1-uygulama");
        crate::guvenlik::uygulama_parolasi::uret(&HERMETIK_GIZLI_ANAHTAR, "dene", &baytlar)
    }
    fn totp_anahtari_uret(&mut self) -> Result<String, String> {
        // zee-io-1: tohumlu sayaçtan türetilen 20 bayt; aynı program aynı anahtar.
        self.guvenli_belirtec_sirasi = self.guvenli_belirtec_sirasi.saturating_add(1);
        let mut baytlar = [0u8; 20];
        baytlar[..8].copy_from_slice(&self.guvenli_belirtec_sirasi.to_be_bytes());
        baytlar[8..].copy_from_slice(b"zee-io-1-tot");
        Ok(crate::guvenlik::base32_kodla(&baytlar))
    }
    fn totp_dogrula(&mut self, gizli: &str, kod: &str) -> bool {
        if self.web_istek_yedegi.is_some() {
            let karar = self.web_guvenligi.rate_limit_artir(
                crate::web_guvenligi::RateLimitTuru::Giris,
                "totp",
                self.an_son_degeri,
            );
            match karar {
                Ok(karar) if karar.izinli => {}
                Ok(_) => {
                    self.durum_yaniti_gonder(429, crate::web_guvenligi::GIRIS_ORAN_ASIMI);
                    return false;
                }
                Err(_) => {
                    self.durum_yaniti_gonder(503, crate::web_guvenligi::GIRIS_ORAN_DEPOSU_YOK);
                    return false;
                }
            }
        }
        let (yil, ay, gun, saat, dakika) = self.zaman;
        let saniye = crate::zaman::tarihten_gunler(yil, ay, gun) * 86_400
            + i64::from(saat) * 3_600
            + i64::from(dakika) * 60;
        crate::guvenlik::totp_dogrula(gizli, kod, saniye)
    }
    fn eposta_gonder(
        &mut self,
        baglanti: Option<&str>,
        alici: &str,
        konu: &str,
        govde: &str,
        _zaman_asimi_ms: i64,
    ) -> Result<i64, String> {
        if let Some(url) = baglanti {
            crate::eposta::EpostaAyari::ayristir(url)?;
        }
        if !crate::eposta::adres_gecerli(alici) {
            return Err("alıcı e-posta adresi geçersiz".into());
        }
        self.eposta_kutusu
            .push((alici.to_string(), konu.to_string(), govde.to_string()));
        Ok(1)
    }
    fn gorsel_kucult(&mut self, kaynak: &str, hedef: &str, azami: i64) -> Result<i64, String> {
        if !(1..=8192).contains(&azami) {
            return Err("azami kenar 1..8192 piksel arasında olmalı".into());
        }
        let icerik = self
            .dosyalar
            .get(kaynak)
            .cloned()
            .ok_or_else(|| format!("\"{}\" kaynak görseli bulunamadı", kaynak))?;
        let yeni = format!("küçültülmüş({azami}): {icerik}");
        let bayt = yeni.len() as i64;
        self.dosyalar.insert(hedef.into(), yeni);
        Ok(bayt)
    }
    fn eylem_baslat(&mut self) -> Result<(), EylemHatasi> {
        if let Some(sonuc) = self.eylem_baslat_sonuclari.pop_front() {
            sonuc?;
        }
        self.eylem_yedekleri
            .push((self.dosyalar.clone(), self.ikili_dosyalar.clone()));
        Ok(())
    }
    fn eylem_tamamla(&mut self) -> Result<(), EylemHatasi> {
        if let Some(Err(hata)) = self.eylem_tamamla_sonuclari.pop_front() {
            (self.dosyalar, self.ikili_dosyalar) = self
                .eylem_yedekleri
                .pop()
                .ok_or_else(|| EylemHatasi::genel("açık eylem transaction'ı yok"))?;
            return Err(hata);
        }
        self.eylem_yedekleri
            .pop()
            .map(|_| ())
            .ok_or_else(|| EylemHatasi::genel("açık eylem transaction'ı yok"))
    }
    fn eylem_geri_al(&mut self) -> Result<(), EylemHatasi> {
        (self.dosyalar, self.ikili_dosyalar) = self
            .eylem_yedekleri
            .pop()
            .ok_or_else(|| EylemHatasi::genel("açık eylem transaction'ı yok"))?;
        Ok(())
    }
    fn istek_tanisini_gunlukle(&mut self, durum: u16, tani: &Tani) {
        self.istek_tani_gunlugu
            .push((durum, tani.kod.clone(), tani.satir));
    }
    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        self.sensorler.get(ad).copied().unwrap_or(false)
    }
    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        self.cikti.push(format!(
            "[ışık] {} {}",
            ad,
            if yansin { "yandı" } else { "söndü" }
        ));
    }
    fn baglanti_dinleyicisi_kaydet(&mut self, ad: &str) -> Result<i64, String> {
        self.baglanti.kaydet(ad)
    }
    fn baglanti_olaylarini_bekle(
        &mut self,
        bekleyenler: &[u64],
        uyanma_an_ms: Option<i64>,
    ) -> Result<Vec<BaglantiOlayi>, String> {
        let (olaylar, an, ekle) = self.baglanti.bekle(bekleyenler, uyanma_an_ms);
        if let Some(an) = an {
            self.an_son_degeri = self.an_son_degeri.max(an);
        }
        self.an_son_degeri = self.an_son_degeri.saturating_add(ekle);
        Ok(olaylar)
    }
    fn baglanti_islemi(&mut self, kimlik: u64, islem: &BaglantiIslemi) -> BaglantiSonucu {
        // K-384: hermetik dosyalar bağlantı tarafıyla paylaşılır.
        if let BaglantiIslemi::DosyaAktar(yol) | BaglantiIslemi::Base64Aktar { yol, .. } = islem {
            if let Some(icerik) = self.dosyalar.get(yol) {
                self.baglanti
                    .dosyalar
                    .insert(yol.clone(), icerik.as_bytes().to_vec());
            }
        }
        let sonuc = self.baglanti.islem(kimlik, islem);
        if let BaglantiSonucu::Blok { yol, .. } = &sonuc {
            if let Some(icerik) = self.baglanti.dosyalar.get(yol) {
                self.dosyalar
                    .insert(yol.clone(), String::from_utf8_lossy(icerik).into_owned());
            }
        }
        sonuc
    }
    fn baglanti_tanisini_gunlukle(&mut self, kimlik: u64, neden: &str) {
        self.baglanti.gunluk.push((kimlik, neden.to_string()));
    }
    fn bekle_ms(&mut self, milisaniye: i64) {
        self.an_son_degeri = self.an_son_degeri.saturating_add(milisaniye.max(0));
    }
    fn an_ms(&mut self) -> i64 {
        if let Some(an) = self.an_degerleri.pop_front() {
            self.an_son_degeri = self.an_son_degeri.max(an);
        }
        self.an_son_degeri
    }

    fn dns_sorgula(
        &mut self,
        ad: &str,
        tur: crate::dns_teli::KayitTuru,
        zaman_asimi_ms: i64,
    ) -> Result<crate::dns_teli::Cozum, crate::dns_teli::DnsHatasi> {
        let ad = crate::dns_teli::ad_dogrula(ad)?;
        self.dns_sorgulari.push((ad.clone(), tur, zaman_asimi_ms));
        self.dns_bolgesi
            .get(&(ad.clone(), tur))
            .cloned()
            .unwrap_or_else(|| {
                Ok(crate::dns_teli::Cozum {
                    durum: crate::dns_teli::Durum::AlanYok,
                    kayitlar: Vec::new(),
                    dogrulanmis: false,
                    kanonik_ad: ad,
                })
            })
    }

    fn kurumsal_alan(&mut self, ad: &str) -> Result<String, String> {
        self.kamu_son_ekleri
            .as_ref()
            .map(|liste| liste.kurumsal_alan(ad))
            .ok_or_else(|| "proje `kamu_son_ekleri` bildirmiyor".into())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn dosya_dkim_govde_ozeti(&mut self, yol: &str, bas: i64, kip: &str) -> Result<String, String> {
        let kip = crate::dkim_kripto::GovdeKipi::ayristir(kip)
            .ok_or_else(|| "aralık: gövde kipi simple ya da relaxed olmalı".to_string())?;
        let baytlar = match self.ikili_dosyalar.get(yol) {
            Some(baytlar) => baytlar.as_slice(),
            None => self
                .dosyalar
                .get(yol)
                .map(String::as_bytes)
                .ok_or_else(|| format!("\"{yol}\" dosyası bulunamadı"))?,
        };
        let mut govde = usize::try_from(bas)
            .ok()
            .and_then(|bas| baytlar.get(bas..))
            .ok_or_else(|| {
                format!(
                    "aralık: gövde başı {bas} dosya boyunun ({}) dışında",
                    baytlar.len()
                )
            })?;
        crate::dkim_kripto::govde_ozetini_akit(&mut govde, kip, yol)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn dkim_imzala(
        &mut self,
        anahtar: &str,
        algoritma: &str,
        veri: &str,
    ) -> Result<String, String> {
        let depo = self
            .dkim_anahtarlari
            .as_ref()
            .ok_or_else(|| "anahtar_bilinmiyor".to_string())?;
        depo.imzala(anahtar, algoritma, veri)
            .map_err(|hata| hata.turu().to_string())
    }
}
