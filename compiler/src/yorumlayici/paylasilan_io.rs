//! Scheduler görevleri arasında paylaşılan IO adaptörünün yönlendirmesi.

use super::*;

impl GirdiCikti for PaylasilanIo<'_> {
    fn yazdir(&mut self, satir: String) {
        self.ic.borrow_mut().yazdir(satir);
    }
    fn sor(&mut self, istem: &str) -> Option<String> {
        self.ic.borrow_mut().sor(istem)
    }
    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        self.ic.borrow_mut().rastgele(alt, ust)
    }
    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        self.ic.borrow_mut().dosya_oku(yol)
    }
    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        self.ic.borrow_mut().dosya_yaz(yol, satir, ekleme)
    }
    fn dosya_atomik_tasi(&mut self, kaynak: &str, hedef: &str) -> Result<i64, String> {
        self.ic.borrow_mut().dosya_atomik_tasi(kaynak, hedef)
    }
    fn dosya_sil(&mut self, yol: &str) -> Result<i64, String> {
        self.ic.borrow_mut().dosya_sil(yol)
    }
    fn dosyalari_listele(&mut self, dizin: &str) -> Result<Vec<String>, String> {
        self.ic.borrow_mut().dosyalari_listele(dizin)
    }
    fn dosya_sha256(&mut self, yol: &str) -> Result<String, String> {
        self.ic.borrow_mut().dosya_sha256(yol)
    }
    fn dosya_ilk_baytlari(&mut self, yol: &str, adet: i64) -> Result<String, String> {
        self.ic.borrow_mut().dosya_ilk_baytlari(yol, adet)
    }
    fn dosya_araligi_oku(&mut self, yol: &str, konum: i64, adet: i64) -> Result<Vec<u8>, String> {
        self.ic.borrow_mut().dosya_araligi_oku(yol, konum, adet)
    }
    fn dosyaya_bayt_yaz(
        &mut self,
        yol: &str,
        baytlar: &[u8],
        ekleme: bool,
    ) -> Result<(String, u64), String> {
        self.ic.borrow_mut().dosyaya_bayt_yaz(yol, baytlar, ekleme)
    }
    fn dosyaya_base64_satirlari_ekle(&mut self, kaynak: &str, hedef: &str) -> Result<i64, String> {
        self.ic
            .borrow_mut()
            .dosyaya_base64_satirlari_ekle(kaynak, hedef)
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.ic.borrow_mut().simdi()
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.ic.borrow_mut().argumanlar()
    }
    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        self.ic.borrow_mut().http_getir(url, zaman_asimi_ms)
    }
    fn http_istek(
        &mut self,
        url: &str,
        yontem: &str,
        basliklar: &[(String, String)],
        govde: &str,
        zaman_asimi_ms: i64,
    ) -> Result<(i64, String), String> {
        self.ic
            .borrow_mut()
            .http_istek(url, yontem, basliklar, govde, zaman_asimi_ms)
    }
    fn http_gizli_ortam(&mut self, ad: &str) -> Result<String, String> {
        self.ic.borrow_mut().http_gizli_ortam(ad)
    }

    fn postgresql_oku(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
        zaman_asimi_ms: Option<i64>,
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        self.ic
            .borrow_mut()
            .postgresql_oku(sorgu, parametreler, zaman_asimi_ms)
    }
    fn postgresql_degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
        zaman_asimi_ms: Option<i64>,
    ) -> Result<i64, VeritabaniHatasi> {
        self.ic
            .borrow_mut()
            .postgresql_degistir(sorgu, parametreler, zaman_asimi_ms)
    }
    fn dns_sorgula(&mut self, ad: &str, tur: KayitTuru, ms: i64) -> DnsSonucu {
        self.ic.borrow_mut().dns_sorgula(ad, tur, ms)
    }
    fn kurumsal_alan(&mut self, ad: &str) -> Result<String, String> {
        self.ic.borrow_mut().kurumsal_alan(ad)
    }
    fn dosya_dkim_govde_ozeti(&mut self, yol: &str, bas: i64, kip: &str) -> Result<String, String> {
        self.ic.borrow_mut().dosya_dkim_govde_ozeti(yol, bas, kip)
    }
    fn dkim_imzala(
        &mut self,
        anahtar: &str,
        algoritma: &str,
        veri: &str,
    ) -> Result<String, String> {
        self.ic.borrow_mut().dkim_imzala(anahtar, algoritma, veri)
    }
    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String> {
        self.ic.borrow_mut().sunucu_kur(kapi)
    }
    fn istek_al(&mut self) -> Option<String> {
        self.ic.borrow_mut().istek_al()
    }
    fn istek_islemini_tamamla(&mut self) -> Result<(), String> {
        self.ic.borrow_mut().istek_islemini_tamamla()
    }
    fn istek_islemini_geri_al(&mut self) {
        self.ic.borrow_mut().istek_islemini_geri_al();
    }
    fn yanit_gonder(&mut self, yanit: &str) {
        self.ic.borrow_mut().yanit_gonder(yanit);
    }
    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        self.ic.borrow_mut().durum_yaniti_gonder(durum, yanit);
    }
    fn turlu_yanit_gonder(&mut self, tur: &str, yanit: &str) {
        self.ic.borrow_mut().turlu_yanit_gonder(tur, yanit);
    }
    fn turlu_durumlu_yanit_gonder(&mut self, tur: &str, durum: u16, yanit: &str) {
        self.ic
            .borrow_mut()
            .turlu_durumlu_yanit_gonder(tur, durum, yanit);
    }
    fn yontem_reddi_gonder(&mut self, izinli: &str, yanit: &str) {
        self.ic.borrow_mut().yontem_reddi_gonder(izinli, yanit);
    }
    fn dosya_yaniti_gonder(&mut self, yol: &str, tur: &str, ad: &str) -> Result<i64, String> {
        self.ic.borrow_mut().dosya_yaniti_gonder(yol, tur, ad)
    }
    fn baglanti_dinleyicisi_kaydet(&mut self, ad: &str) -> Result<i64, String> {
        self.ic.borrow_mut().baglanti_dinleyicisi_kaydet(ad)
    }
    fn baglanti_olaylarini_bekle(
        &mut self,
        bekleyenler: &[u64],
        uyanma_an_ms: Option<i64>,
    ) -> Result<Vec<BaglantiOlayi>, String> {
        self.ic
            .borrow_mut()
            .baglanti_olaylarini_bekle(bekleyenler, uyanma_an_ms)
    }
    fn baglanti_islemi(&mut self, kimlik: u64, islem: &BaglantiIslemi) -> BaglantiSonucu {
        self.ic.borrow_mut().baglanti_islemi(kimlik, islem)
    }
    fn baglanti_tanisini_gunlukle(&mut self, kimlik: u64, neden: &str) {
        self.ic
            .borrow_mut()
            .baglanti_tanisini_gunlukle(kimlik, neden)
    }
    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        self.ic.borrow_mut().yonlendir_gonder(adres)
    }
    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        self.ic.borrow_mut().cerez_yaz(ad, deger)
    }
    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        self.ic.borrow_mut().cerez_sil(ad)
    }
    fn yanit_onbellegine_izin_ver(&mut self, saniye: u32) {
        self.ic.borrow_mut().yanit_onbellegine_izin_ver(saniye)
    }
    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        self.ic
            .borrow_mut()
            .rota_guvenligini_denetle(erisim, csrf, csrf_gerekli)
    }
    fn istemci_adresi(&mut self) -> String {
        self.ic.borrow_mut().istemci_adresi()
    }
    fn birincil_surec(&mut self) -> bool {
        self.ic.borrow_mut().birincil_surec()
    }
    fn csrf_belirteci(&mut self) -> Result<String, String> {
        self.ic.borrow_mut().csrf_belirteci()
    }
    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        self.ic.borrow_mut().oturum_ac(kullanici, rol)
    }
    fn oturum_kapat(&mut self) -> Result<(), String> {
        self.ic.borrow_mut().oturum_kapat()
    }
    fn kullanici_oturumlarini_listele(
        &mut self,
        kullanici: &str,
    ) -> Result<Vec<Vec<(String, String)>>, String> {
        self.ic
            .borrow_mut()
            .kullanici_oturumlarini_listele(kullanici)
    }
    fn oturum_kimligini_iptal_et(&mut self, kimlik: &str) -> Result<i64, String> {
        self.ic.borrow_mut().oturum_kimligini_iptal_et(kimlik)
    }
    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        self.ic.borrow_mut().parola_dogrula(parola, ozet)
    }
    fn parola_ozeti_uret(&mut self, parola: &str) -> Result<String, String> {
        self.ic.borrow_mut().parola_ozeti_uret(parola)
    }
    fn totp_anahtari_uret(&mut self) -> Result<String, String> {
        self.ic.borrow_mut().totp_anahtari_uret()
    }
    fn uygulama_parolasi_uret(
        &mut self,
    ) -> Result<crate::guvenlik::uygulama_parolasi::UretilenParola, String> {
        self.ic.borrow_mut().uygulama_parolasi_uret()
    }
    fn totp_dogrula(&mut self, gizli: &str, kod: &str) -> bool {
        self.ic.borrow_mut().totp_dogrula(gizli, kod)
    }
    fn eposta_gonder(
        &mut self,
        baglanti: Option<&str>,
        alici: &str,
        konu: &str,
        govde: &str,
        zaman_asimi_ms: i64,
    ) -> Result<i64, String> {
        self.ic
            .borrow_mut()
            .eposta_gonder(baglanti, alici, konu, govde, zaman_asimi_ms)
    }
    fn gorsel_kucult(&mut self, kaynak: &str, hedef: &str, azami: i64) -> Result<i64, String> {
        self.ic.borrow_mut().gorsel_kucult(kaynak, hedef, azami)
    }
    fn eposta_mime_gonder(
        &mut self,
        b: Option<&str>,
        a: &str,
        k: &str,
        g: &str,
        s: &crate::eposta::MimeGonderim<'_>,
    ) -> Result<i64, String> {
        self.ic.borrow_mut().eposta_mime_gonder(b, a, k, g, s)
    }
    fn gizli_sifrele(&mut self, duz: &str) -> Result<String, String> {
        self.ic.borrow_mut().gizli_sifrele(duz)
    }
    fn gizli_coz(&mut self, sifreli: &str) -> Result<String, String> {
        self.ic.borrow_mut().gizli_coz(sifreli)
    }
    fn eylem_baslat(&mut self) -> Result<(), EylemHatasi> {
        self.ic.borrow_mut().eylem_baslat()
    }
    fn eylem_tamamla(&mut self) -> Result<(), EylemHatasi> {
        self.ic.borrow_mut().eylem_tamamla()
    }
    fn eylem_geri_al(&mut self) -> Result<(), EylemHatasi> {
        self.ic.borrow_mut().eylem_geri_al()
    }
    fn istek_tanisini_gunlukle(&mut self, durum: u16, tani: &Tani) {
        self.ic.borrow_mut().istek_tanisini_gunlukle(durum, tani);
    }
    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        self.ic.borrow_mut().sensor_acik_mi(ad)
    }
    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        self.ic.borrow_mut().isik_ayarla(ad, yansin);
    }
    fn bekle_ms(&mut self, milisaniye: i64) {
        self.ic.borrow_mut().bekle_ms(milisaniye);
    }
    fn an_ms(&mut self) -> i64 {
        self.ic.borrow_mut().an_ms()
    }
}
