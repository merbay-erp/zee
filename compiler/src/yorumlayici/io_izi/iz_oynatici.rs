use super::*;

impl GirdiCikti for IzYenidenOynatici {
    fn yazdir(&mut self, satir: String) {
        let sonuc = self.siradaki("yazdir", vec![satir.clone()]);
        if self.bos_sonuc("yazdir", sonuc) {
            self.cikti.push(satir);
        }
    }

    fn sor(&mut self, istem: &str) -> Option<String> {
        let sonuc = self.siradaki("sor", vec![istem.into()]);
        let alanlar = sonuc?;
        match secenek_coz(&alanlar) {
            Ok(sonuc) => {
                self.cikti.push(istem.into());
                sonuc
            }
            Err(hata) => {
                self.bozuk_sonuc("sor", hata);
                None
            }
        }
    }

    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        let sonuc = self.siradaki("rastgele", vec![alt.to_string(), ust.to_string()]);
        self.sayi_sonuc("rastgele", sonuc, alt)
    }

    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        let sonuc = self.siradaki("dosya_oku", vec![yol.into()]);
        self.metin_sonuc("dosya_oku", sonuc)
    }

    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        let sonuc = self.siradaki(
            "dosya_yaz",
            vec![yol.into(), satir.into(), bool_yaz(ekleme)],
        );
        self.birim_sonuc("dosya_yaz", sonuc)
    }

    fn dosya_atomik_tasi(&mut self, kaynak: &str, hedef: &str) -> Result<i64, String> {
        let sonuc = self.siradaki("dosya_atomik_tasi", vec![kaynak.into(), hedef.into()]);
        let Some(alanlar) = sonuc else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        tamsayi_sonuc_coz(&alanlar).unwrap_or_else(|hata| {
            self.bozuk_sonuc("dosya_atomik_tasi", hata);
            Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
        })
    }

    fn dosya_sil(&mut self, yol: &str) -> Result<i64, String> {
        let sonuc = self.siradaki("dosya_sil", vec![yol.into()]);
        let Some(alanlar) = sonuc else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        tamsayi_sonuc_coz(&alanlar).unwrap_or_else(|hata| {
            self.bozuk_sonuc("dosya_sil", hata);
            Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
        })
    }

    fn dosyalari_listele(&mut self, dizin: &str) -> Result<Vec<String>, String> {
        let sonuc = self.siradaki("dosyalari_listele", vec![dizin.into()]);
        let Some(alanlar) = sonuc else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        metin_listesi_sonuc_coz(&alanlar).unwrap_or_else(|hata| {
            self.bozuk_sonuc("dosyalari_listele", hata);
            Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
        })
    }

    fn dosya_sha256(&mut self, yol: &str) -> Result<String, String> {
        let sonuc = self.siradaki("dosya_sha256", vec![yol.into()]);
        self.metin_sonuc("dosya_sha256", sonuc)
    }

    fn postgresql_oku(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        tasima::postgresql_oku_oynat(self, sorgu, parametreler)
    }

    fn postgresql_degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<i64, VeritabaniHatasi> {
        tasima::postgresql_degistir_oynat(self, sorgu, parametreler)
    }

    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        tasima::simdi_oynat(self)
    }

    fn argumanlar(&mut self) -> Vec<String> {
        self.siradaki("argumanlar", Vec::new()).unwrap_or_default()
    }

    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        tasima::http_getir_oynat(self, url, zaman_asimi_ms)
    }

    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String> {
        let sonuc = self.siradaki("sunucu_kur", vec![kapi.to_string()]);
        self.birim_sonuc("sunucu_kur", sonuc)
    }

    fn istek_al(&mut self) -> Option<String> {
        let alanlar = self.siradaki("istek_al", Vec::new())?;
        match secenek_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc("istek_al", hata);
                None
            }
        }
    }

    fn yanit_gonder(&mut self, yanit: &str) {
        let sonuc = self.siradaki("yanit_gonder", vec![yanit.into()]);
        self.bos_sonuc("yanit_gonder", sonuc);
    }

    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        let sonuc = self.siradaki("durum_yaniti_gonder", vec![durum.to_string(), yanit.into()]);
        self.bos_sonuc("durum_yaniti_gonder", sonuc);
    }

    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        let sonuc = self.siradaki("yonlendir_gonder", vec![adres.into()]);
        self.birim_sonuc("yonlendir_gonder", sonuc)
    }

    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        let sonuc = self.siradaki("cerez_yaz", vec![ad.into(), deger.into()]);
        self.birim_sonuc("cerez_yaz", sonuc)
    }

    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        let sonuc = self.siradaki("cerez_sil", vec![ad.into()]);
        self.birim_sonuc("cerez_sil", sonuc)
    }

    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        let Some(alanlar) = self.siradaki(
            "rota_guvenligini_denetle",
            rota_argumanlari(erisim, csrf, csrf_gerekli),
        ) else {
            return Err(WebReddi {
                durum: 500,
                mesaj: IZ_WEB_REDDI,
            });
        };
        match web_sonuc_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc("rota_guvenligini_denetle", hata);
                Err(WebReddi {
                    durum: 500,
                    mesaj: IZ_WEB_REDDI,
                })
            }
        }
    }

    fn istemci_adresi(&mut self) -> String {
        istemci::yeniden_oynat(self)
    }

    fn csrf_belirteci(&mut self) -> Result<String, String> {
        let sonuc = self.siradaki("csrf_belirteci", Vec::new());
        self.metin_sonuc("csrf_belirteci", sonuc)
    }

    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        let sonuc = self.siradaki("oturum_ac", vec![kullanici.into(), rol.into()]);
        self.birim_sonuc("oturum_ac", sonuc)
    }

    fn oturum_kapat(&mut self) -> Result<(), String> {
        let sonuc = self.siradaki("oturum_kapat", Vec::new());
        self.birim_sonuc("oturum_kapat", sonuc)
    }

    fn kullanici_oturumlarini_listele(
        &mut self,
        kullanici: &str,
    ) -> Result<Vec<Vec<(String, String)>>, String> {
        let sonuc = self.siradaki("kullanici_oturumlarini_listele", vec![kullanici.into()]);
        let Some(alanlar) = sonuc else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        oturum_listesi_sonuc_coz(&alanlar).unwrap_or_else(|hata| {
            self.bozuk_sonuc("kullanici_oturumlarini_listele", hata);
            Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
        })
    }

    fn oturum_kimligini_iptal_et(&mut self, kimlik: &str) -> Result<i64, String> {
        let sonuc = self.siradaki("oturum_kimligini_iptal_et", vec![kimlik.into()]);
        let Some(alanlar) = sonuc else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        tamsayi_sonuc_coz(&alanlar).unwrap_or_else(|hata| {
            self.bozuk_sonuc("oturum_kimligini_iptal_et", hata);
            Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
        })
    }

    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        let sonuc = self.siradaki(
            "parola_dogrula",
            vec![sha256_hex(parola.as_bytes()), sha256_hex(ozet.as_bytes())],
        );
        self.bool_sonuc("parola_dogrula", sonuc)
    }

    fn parola_ozeti_uret(&mut self, parola: &str) -> Result<String, String> {
        k181::oynat_parola_ozeti_uret(self, parola)
    }
    fn gizli_sifrele(&mut self, duz: &str) -> Result<String, String> {
        k181::oynat_gizli_sifrele(self, duz)
    }
    fn gizli_coz(&mut self, sifreli: &str) -> Result<String, String> {
        k181::oynat_gizli_coz(self, sifreli)
    }

    fn totp_anahtari_uret(&mut self) -> Result<String, String> {
        k181::oynat_totp_anahtari_uret(self)
    }

    fn totp_dogrula(&mut self, gizli: &str, kod: &str) -> bool {
        k181::oynat_totp_dogrula(self, gizli, kod)
    }

    fn eposta_gonder(
        &mut self,
        b: Option<&str>,
        alici: &str,
        konu: &str,
        govde: &str,
    ) -> Result<i64, String> {
        k181::oynat_eposta_gonder(self, b, alici, konu, govde)
    }

    fn gorsel_kucult(&mut self, kaynak: &str, hedef: &str, azami: i64) -> Result<i64, String> {
        k181::oynat_gorsel_kucult(self, kaynak, hedef, azami)
    }

    fn eposta_mime_gonder(
        &mut self,
        _b: Option<&str>,
        _a: &str,
        _k: &str,
        _g: &str,
        _s: &crate::eposta::MimeGonderim<'_>,
    ) -> Result<i64, String> {
        self.hata_yaz("MIME e-posta IO yeniden oynatımında kapalı".into());
        Err("MIME e-posta IO yeniden oynatımında kapalı".into())
    }

    fn eylem_baslat(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.siradaki("eylem_baslat", Vec::new());
        self.eylem_sonuc("eylem_baslat", sonuc)
    }

    fn eylem_tamamla(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.siradaki("eylem_tamamla", Vec::new());
        self.eylem_sonuc("eylem_tamamla", sonuc)
    }

    fn eylem_geri_al(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.siradaki("eylem_geri_al", Vec::new());
        self.eylem_sonuc("eylem_geri_al", sonuc)
    }

    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        let sonuc = self.siradaki("sensor_acik_mi", vec![ad.into()]);
        self.bool_sonuc("sensor_acik_mi", sonuc)
    }

    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        let sonuc = self.siradaki("isik_ayarla", vec![ad.into(), bool_yaz(yansin)]);
        if self.bos_sonuc("isik_ayarla", sonuc) {
            self.cikti.push(format!(
                "[ışık] {} {}",
                ad,
                if yansin { "yandı" } else { "söndü" }
            ));
        }
    }

    fn bekle_ms(&mut self, milisaniye: i64) {
        let sonuc = self.siradaki("bekle_ms", vec![milisaniye.to_string()]);
        self.bos_sonuc("bekle_ms", sonuc);
    }

    fn an_ms(&mut self) -> i64 {
        let sonuc = self.siradaki("an_ms", Vec::new());
        self.sayi_sonuc("an_ms", sonuc, 0)
    }
}
