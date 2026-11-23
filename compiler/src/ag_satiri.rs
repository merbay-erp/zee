//! K-381 (RFC-0039 A2a): bağlantı satır akışının saf, platformdan bağımsız
//! çekirdeği. Gerçek soket reaktörü (CLI) ve hermetik adaptör aynı okuyucuyu
//! kullanır; böylece çıplak CR/LF, uzun satır ve kodlama kuralı tek yerdedir.
//!
//! Tek satır sonu CRLF'dir. Çıplak LF, çıplak CR ve karışık diziler satır sonu
//! sayılmaz ve normalleştirilmez (SMTP smuggling sınıfı); sınırı aşan satır
//! kırpılıp kabul edilmez.

/// Bağlantı başına giden tamponun üst sınırı (geri basınç eşiği).
pub const GIDEN_TAMPON_BAYTI: usize = 64 * 1024;
/// Tek okuma çağrısının çekebileceği en çok bayt; gelen tampon bu kadar
/// satır sınırını aşabilir.
pub const OKUMA_PARCASI: usize = 4096;
/// Bağlantı ayarlarının alt/üst sınırları (P020 ile aynı).
pub const SATIR_SINIRI_ALT: usize = 64;
pub const SATIR_SINIRI_UST: usize = 65_536;

/// Bir projenin bağlantı yüzeyi sınırları (proje.dil P020; K-381).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaglantiAyari {
    /// Aynı anda açık en çok bağlantı (süreç geneli).
    pub sinir: usize,
    /// Aynı uzak IP adresinden en çok eşzamanlı bağlantı.
    pub uzak_adres_siniri: usize,
    /// CRLF dahil en çok satır baytı (okuma ve yazma).
    pub satir_siniri: usize,
    /// Bir okuma ya da tıkanan yazma çağrısının mutlak süresi.
    pub bosta_suresi_ms: i64,
    /// Bağlantı görevinin mutlak ömrü.
    pub omur_ms: i64,
    /// Sınır dolunca yazılan tek satır (CRLF'yi çalışma zamanı ekler).
    pub dolu_yaniti: Option<String>,
}

impl Default for BaglantiAyari {
    fn default() -> Self {
        Self {
            sinir: 64,
            uzak_adres_siniri: 8,
            satir_siniri: 1000,
            bosta_suresi_ms: 300_000,
            omur_ms: 3_600_000,
            dolu_yaniti: None,
        }
    }
}

/// Okunan satırın sınıflandırılmış reddi; Zee'ye C030 `tür` olarak çıkar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatirHatasi {
    /// CRLF dahil satır sınırı aşıldı; satırın kalanı atlanır.
    Uzun,
    /// Çıplak CR ya da çıplak LF; bozuk satır tümüyle atılır.
    SatirSonu,
    /// Geçersiz UTF-8, NUL ya da TAB dışı denetim karakteri.
    Kodlama,
}

impl SatirHatasi {
    pub const fn turu(self) -> &'static str {
        match self {
            Self::Uzun => "satır_uzun",
            Self::SatirSonu => "satır_sonu",
            Self::Kodlama => "kodlama",
        }
    }

    pub const fn mesaji(self) -> &'static str {
        match self {
            Self::Uzun => "satır bağlantı satır sınırını aşıyor",
            Self::SatirSonu => "satır CRLF ile bitmiyor (çıplak CR ya da LF)",
            Self::Kodlama => "satır geçersiz UTF-8 ya da denetim karakteri taşıyor",
        }
    }
}

/// CRLF satır okuyucusu. `besle` ile gelen baytlar eklenir, `siradaki` tam
/// bir satır ya da sınıflandırılmış ret verir; `None` daha çok veri ister.
#[derive(Debug, Clone)]
pub struct SatirOkuyucu {
    tampon: Vec<u8>,
    /// CRLF dahil en çok satır baytı.
    sinir: usize,
    /// Ret edilen satırın kalanı bir sonraki LF'ye kadar atılır.
    atla: bool,
}

impl SatirOkuyucu {
    pub fn yeni(sinir: usize) -> Self {
        Self {
            tampon: Vec::new(),
            sinir: sinir.clamp(SATIR_SINIRI_ALT, SATIR_SINIRI_UST),
            atla: false,
        }
    }

    /// Şu an gelen tampona alınabilecek bayt (bellek üst sınırı).
    pub fn bos_alan(&self) -> usize {
        (self.sinir + OKUMA_PARCASI).saturating_sub(self.tampon.len())
    }

    pub fn besle(&mut self, veri: &[u8]) {
        self.tampon.extend_from_slice(veri);
    }

    /// Zee'nin henüz tüketmediği gelen bayt sayısı (STARTTLS denetimi).
    pub fn tamponlu_bayt(&self) -> usize {
        self.tampon.len()
    }

    /// Bütün gelen durumu atar (TLS yükseltmesinden sonra sıfır durum).
    pub fn sifirla(&mut self) {
        self.tampon.clear();
        self.atla = false;
    }

    fn tuket(&mut self, adet: usize) {
        self.tampon.drain(..adet.min(self.tampon.len()));
    }

    pub fn siradaki(&mut self) -> Option<Result<String, SatirHatasi>> {
        if self.atla {
            match self.tampon.iter().position(|b| *b == b'\n') {
                Some(lf) => {
                    self.tuket(lf + 1);
                    self.atla = false;
                }
                None => {
                    self.tampon.clear();
                    return None;
                }
            }
        }
        let icerik_siniri = self.sinir - 2;
        let Some(son) = self.tampon.iter().position(|b| *b == b'\r' || *b == b'\n') else {
            if self.tampon.len() > icerik_siniri {
                self.tampon.clear();
                self.atla = true;
                return Some(Err(SatirHatasi::Uzun));
            }
            return None;
        };
        if self.tampon[son] == b'\n' {
            // Çıplak LF: CR'siz satır sonu kabul edilmez.
            self.tuket(son + 1);
            return Some(Err(SatirHatasi::SatirSonu));
        }
        // CR bulundu.
        if son + 1 == self.tampon.len() {
            if son > icerik_siniri {
                self.tampon.clear();
                self.atla = true;
                return Some(Err(SatirHatasi::Uzun));
            }
            return None; // LF bekleniyor
        }
        if self.tampon[son + 1] != b'\n' {
            // Çıplak CR: satırın tamamı bir sonraki LF'ye kadar atılır.
            self.tuket(son + 1);
            self.atla = true;
            return Some(Err(SatirHatasi::SatirSonu));
        }
        let satir: Vec<u8> = self.tampon[..son].to_vec();
        self.tuket(son + 2);
        if son > icerik_siniri {
            return Some(Err(SatirHatasi::Uzun));
        }
        Some(satir_metni(satir))
    }
}

fn denetim_karakteri(b: u8) -> bool {
    (b < 0x20 && b != b'\t') || b == 0x7f
}

fn satir_metni(satir: Vec<u8>) -> Result<String, SatirHatasi> {
    if satir.iter().any(|b| denetim_karakteri(*b)) {
        return Err(SatirHatasi::Kodlama);
    }
    String::from_utf8(satir).map_err(|_| SatirHatasi::Kodlama)
}

/// Zee metnini tek giden satıra çevirir (CRLF'yi çalışma zamanı ekler).
/// CR, LF, NUL ve TAB dışı denetim karakteri yanıt bölme/enjeksiyon olduğu
/// için reddedilir; CRLF dahil uzunluk satır sınırını aşamaz.
pub fn giden_satir(metin: &str, sinir: usize) -> Result<Vec<u8>, &'static str> {
    if metin.bytes().any(denetim_karakteri) {
        return Err("satır CR, LF, NUL ya da denetim karakteri taşıyamaz");
    }
    if metin.len() + 2 > sinir {
        return Err("satır bağlantı satır sınırını aşıyor");
    }
    let mut bayt = Vec::with_capacity(metin.len() + 2);
    bayt.extend_from_slice(metin.as_bytes());
    bayt.extend_from_slice(b"\r\n");
    Ok(bayt)
}

/// Kibar ret satırı: ≤ 512 bayt yazdırılabilir ASCII.
pub fn dolu_yaniti_gecerli(metin: &str) -> bool {
    !metin.is_empty() && metin.len() <= 512 && metin.bytes().all(|b| (0x20..0x7f).contains(&b))
}

#[cfg(test)]
mod testler {
    use super::*;

    fn oku(sinir: usize, girdi: &[u8]) -> Vec<Result<String, SatirHatasi>> {
        let mut okuyucu = SatirOkuyucu::yeni(sinir);
        okuyucu.besle(girdi);
        std::iter::from_fn(|| okuyucu.siradaki()).collect()
    }

    #[test]
    fn crlf_satirlari_sirayla_verilir() {
        assert_eq!(
            oku(1000, b"EHLO a\r\nMAIL FROM:<x@y>\r\n"),
            vec![Ok("EHLO a".into()), Ok("MAIL FROM:<x@y>".into())]
        );
        assert_eq!(oku(1000, b"\r\n"), vec![Ok(String::new())]);
    }

    #[test]
    fn eksik_satir_veri_bekler_ve_parcali_tamamlanir() {
        let mut okuyucu = SatirOkuyucu::yeni(1000);
        okuyucu.besle(b"EH");
        assert_eq!(okuyucu.siradaki(), None);
        okuyucu.besle(b"LO\r");
        assert_eq!(okuyucu.siradaki(), None);
        okuyucu.besle(b"\nQ");
        assert_eq!(okuyucu.siradaki(), Some(Ok("EHLO".into())));
        assert_eq!(okuyucu.siradaki(), None);
        assert_eq!(okuyucu.tamponlu_bayt(), 1);
    }

    #[test]
    fn ciplak_lf_ve_cr_smuggling_dizileri_reddedilir() {
        use SatirHatasi::SatirSonu;
        assert_eq!(oku(1000, b"a\nb\r\n"), vec![Err(SatirSonu), Ok("b".into())]);
        // <LF>.<LF>, <CR>.<CR> ve <CR><CR><LF> hiçbir zaman satır sonu olmaz.
        assert_eq!(
            oku(1000, b"x\n.\nQUIT\r\n"),
            vec![Err(SatirSonu), Err(SatirSonu), Ok("QUIT".into())]
        );
        assert_eq!(
            oku(1000, b"x\r.\rDATA\r\nNOOP\r\n"),
            vec![Err(SatirSonu), Ok("NOOP".into())]
        );
        assert_eq!(
            oku(1000, b"a\r\r\nNOOP\r\n"),
            vec![Err(SatirSonu), Ok("NOOP".into())]
        );
    }

    #[test]
    fn uzun_satir_kirpilmadan_reddedilir_kalan_atlanir() {
        let mut girdi = vec![b'A'; 70];
        girdi.extend_from_slice(b"\r\nNOOP\r\n");
        assert_eq!(
            oku(64, &girdi),
            vec![Err(SatirHatasi::Uzun), Ok("NOOP".into())]
        );
        // Tam sınır: 62 bayt + CRLF = 64 geçer, 63 bayt geçmez.
        let mut tam = vec![b'B'; 62];
        tam.extend_from_slice(b"\r\n");
        assert_eq!(oku(64, &tam), vec![Ok("B".repeat(62))]);
        let mut fazla = vec![b'B'; 63];
        fazla.extend_from_slice(b"\r\n");
        assert_eq!(oku(64, &fazla), vec![Err(SatirHatasi::Uzun)]);
        // Satır sonu hiç gelmeden sınır aşılırsa bellek büyümez.
        let mut okuyucu = SatirOkuyucu::yeni(64);
        okuyucu.besle(&[b'C'; 200]);
        assert_eq!(okuyucu.siradaki(), Some(Err(SatirHatasi::Uzun)));
        okuyucu.besle(&[b'C'; 200]);
        assert_eq!(okuyucu.siradaki(), None);
        assert_eq!(okuyucu.tamponlu_bayt(), 0);
        okuyucu.besle(b"C\r\nNOOP\r\n");
        assert_eq!(okuyucu.siradaki(), Some(Ok("NOOP".into())));
    }

    #[test]
    fn kodlama_ve_denetim_karakteri_reddedilir_tab_gecer() {
        assert_eq!(
            oku(1000, b"\xff\xfe\r\na\x00b\r\nc\td\r\n"),
            vec![
                Err(SatirHatasi::Kodlama),
                Err(SatirHatasi::Kodlama),
                Ok("c\td".into())
            ]
        );
        assert_eq!(oku(1000, "şü\r\n".as_bytes()), vec![Ok("şü".into())]);
    }

    #[test]
    fn giden_satir_enjeksiyonu_ve_siniri_reddeder() {
        assert_eq!(giden_satir("250 tamam", 1000).unwrap(), b"250 tamam\r\n");
        for kotu in ["a\r\nRCPT", "a\nb", "a\rb", "a\0b", "a\x1bb"] {
            assert!(giden_satir(kotu, 1000).is_err(), "{kotu:?}");
        }
        assert!(giden_satir(&"x".repeat(62), 64).is_ok());
        assert!(giden_satir(&"x".repeat(63), 64).is_err());
    }

    #[test]
    fn dolu_yaniti_yazdirilabilir_ascii_olur() {
        assert!(dolu_yaniti_gecerli(
            "421 4.3.2 meşgul".replace('ş', "s").as_str()
        ));
        assert!(!dolu_yaniti_gecerli("421 meşgul"));
        assert!(!dolu_yaniti_gecerli("421\r\nX"));
        assert!(!dolu_yaniti_gecerli(""));
        assert!(!dolu_yaniti_gecerli(&"4".repeat(513)));
    }

    #[test]
    fn sifirla_tamponu_ve_atlamayi_temizler() {
        let mut okuyucu = SatirOkuyucu::yeni(64);
        okuyucu.besle(&[b'A'; 100]);
        assert_eq!(okuyucu.siradaki(), Some(Err(SatirHatasi::Uzun)));
        okuyucu.besle(b"EHLO");
        okuyucu.sifirla();
        assert_eq!(okuyucu.tamponlu_bayt(), 0);
        okuyucu.besle(b"EHLO\r\n");
        assert_eq!(okuyucu.siradaki(), Some(Ok("EHLO".into())));
    }
}
