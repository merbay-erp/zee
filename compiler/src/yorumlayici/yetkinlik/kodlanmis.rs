//! K-389/K-395 (ADR-167/171): kodlanmış dosya ve bayt akışı kapıları. Kaynak
//! `dosya-okuma`, hedef `dosya-yazma` ister; yollar proje sınırından geçer.
macro_rules! yontemler {
    () => {
        fn dosya_araligi_oku(
            &mut self,
            yol: &str,
            konum: i64,
            adet: i64,
        ) -> Result<Vec<u8>, String> {
            self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)?;
            self.ic.dosya_araligi_oku(yol, konum, adet)
        }

        fn dosyaya_bayt_yaz(
            &mut self,
            yol: &str,
            baytlar: &[u8],
            ekleme: bool,
        ) -> Result<(String, u64), String> {
            self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaYazma)?;
            self.ic.dosyaya_bayt_yaz(yol, baytlar, ekleme)
        }

        fn dosyadan_base64_ekle(
            &mut self,
            kaynak: &str,
            konum: i64,
            adet: i64,
            hedef: &str,
        ) -> Result<crate::kalici_dosya::AkisliEkleme, crate::kodlama::AkisHatasi> {
            use crate::kodlama::AkisHatasi::Dosya;
            self.dosya_yolunu_denetle(kaynak, Yetkinlik::DosyaOkuma)
                .map_err(Dosya)?;
            self.dosya_yolunu_denetle(hedef, Yetkinlik::DosyaYazma)
                .map_err(Dosya)?;
            self.ic.dosyadan_base64_ekle(kaynak, konum, adet, hedef)
        }

        fn dosyada_bayt_ara(&mut self, yol: &str, konum: i64, igne: &[u8]) -> Result<i64, String> {
            self.dosya_yolunu_denetle(yol, Yetkinlik::DosyaOkuma)?;
            self.ic.dosyada_bayt_ara(yol, konum, igne)
        }
    };
}
pub(super) use yontemler;
