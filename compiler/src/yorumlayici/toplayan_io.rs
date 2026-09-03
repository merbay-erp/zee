//! Hermetik test ve kütüphane adaptörünün deterministik IO durumu.

use super::*;

pub struct ToplayanIo {
    pub girdiler: VecDeque<String>,
    pub rastgele_degerler: VecDeque<i64>,
    /// Sahte dosya sistemi: yol → içerik (testlerde determinizm).
    pub dosyalar: HashMap<String, String>,
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
    /// Sahte sensörler (varsayılan kapalı) ve an ölçümü kuyruğu.
    pub sensorler: HashMap<String, bool>,
    pub an_degerleri: VecDeque<i64>,
    pub eylem_baslat_sonuclari: VecDeque<Result<(), EylemHatasi>>,
    pub eylem_tamamla_sonuclari: VecDeque<Result<(), EylemHatasi>>,
    pub dosya_yaz_sonuclari: VecDeque<Result<(), String>>,
    an_son_degeri: i64,
    pub cikti: Vec<String>,
    eylem_yedekleri: Vec<HashMap<String, String>>,
    web_guvenligi: WebGuvenligi,
    guvenli_belirtec_sirasi: u64,
    web_istek_yedegi: Option<ToplayanWebIstekYedegi>,
    bekleyen_sunucu_yaniti: Option<SunucuYanitTaslagi>,
}

struct ToplayanWebIstekYedegi {
    yazilan_cerez_sayisi: usize,
    guvenli_cerez_sayisi: usize,
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
    pub fn yeni(girdiler: Vec<String>) -> ToplayanIo {
        ToplayanIo {
            girdiler: girdiler.into(),
            rastgele_degerler: VecDeque::new(),
            dosyalar: HashMap::new(),
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
            sensorler: HashMap::new(),
            an_degerleri: VecDeque::new(),
            eylem_baslat_sonuclari: VecDeque::new(),
            eylem_tamamla_sonuclari: VecDeque::new(),
            dosya_yaz_sonuclari: VecDeque::new(),
            an_son_degeri: 0,
            cikti: Vec::new(),
            eylem_yedekleri: Vec::new(),
            web_guvenligi: WebGuvenligi::yeni(),
            guvenli_belirtec_sirasi: 0,
            web_istek_yedegi: None,
            bekleyen_sunucu_yaniti: None,
        }
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
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        self.postgresql.oku(sorgu, parametreler)
    }
    fn postgresql_degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<i64, VeritabaniHatasi> {
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
        self.sunucu_yanitlari.push((yol.to_string(), String::new()));
        self.sunucu_durumlari.push(200);
        Some(ham)
    }
    fn istek_islemini_tamamla(&mut self) -> Result<(), String> {
        let Some(yedek) = self.web_istek_yedegi.take() else {
            return Ok(());
        };
        if let Some(taslak) = self.bekleyen_sunucu_yaniti.take() {
            self.web_guvenligi.istegi_tamamla(self.an_son_degeri)?;
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
                self.durum_yaniti_gonder(429, "çok fazla giriş denemesi");
                false
            }
            Err(_) => {
                self.durum_yaniti_gonder(503, "giriş oran sınırı deposuna erişilemedi");
                false
            }
        }
    }
    fn eylem_baslat(&mut self) -> Result<(), EylemHatasi> {
        if let Some(sonuc) = self.eylem_baslat_sonuclari.pop_front() {
            sonuc?;
        }
        self.eylem_yedekleri.push(self.dosyalar.clone());
        Ok(())
    }
    fn eylem_tamamla(&mut self) -> Result<(), EylemHatasi> {
        if let Some(Err(hata)) = self.eylem_tamamla_sonuclari.pop_front() {
            self.dosyalar = self
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
        self.dosyalar = self
            .eylem_yedekleri
            .pop()
            .ok_or_else(|| EylemHatasi::genel("açık eylem transaction'ı yok"))?;
        Ok(())
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
    fn bekle_ms(&mut self, milisaniye: i64) {
        self.an_son_degeri = self.an_son_degeri.saturating_add(milisaniye.max(0));
    }
    fn an_ms(&mut self) -> i64 {
        if let Some(an) = self.an_degerleri.pop_front() {
            self.an_son_degeri = self.an_son_degeri.max(an);
        }
        self.an_son_degeri
    }
}
