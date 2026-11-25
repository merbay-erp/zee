//! K-381 (RFC-0039 A2a): bağlantı satır akışının saf, platformdan bağımsız
//! çekirdeği. Gerçek soket reaktörü (CLI) ve hermetik adaptör aynı okuyucuyu
//! kullanır; böylece çıplak CR/LF, uzun satır ve kodlama kuralı tek yerdedir.
//!
//! Tek satır sonu CRLF'dir. Çıplak LF, çıplak CR ve karışık diziler satır sonu
//! sayılmaz ve normalleştirilmez (SMTP smuggling sınıfı); sınırı aşan satır
//! kırpılıp kabul edilmez.
//!
//! K-384 (RFC-0039 A2c): blok okuma da burada tek kuraldır. SMTP `DATA`
//! nokta bloğu yalnız birebir `CRLF . CRLF` ile biter; çıplak CR/LF
//! normalleştirilmez, bloğun kalanı sonlandırıcıya kadar tüketilip atılır
//! (smuggling sınıfı). IMAP literal'i tam `n` bayttır.

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
    /// K-381 A2b: sunucu TLS'i (implicit dinleyiciler + STARTTLS kimliği).
    pub tls: Option<TlsBildirimi>,
    /// K-384 A2c: dosyaya alınan blok (nokta bloğu, literal) ve dosyadan
    /// bağlantıya aktarımın en çok baytı.
    pub blok_siniri: u64,
}

/// Sunucu TLS kimliği ve implicit TLS dinleyicileri (proje.dil P021).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TlsBildirimi {
    /// El sıkışmanın Zee bloğundan önce yapıldığı dinleyici adları (465/993).
    pub dinleyiciler: Vec<String>,
    /// PEM sertifika zincirini taşıyan credential/ortam adı (K-339 modeli).
    pub sertifika_degiskeni: String,
    /// PEM özel anahtarı taşıyan credential/ortam adı; değer hiç loglanmaz.
    pub anahtar_degiskeni: String,
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
            tls: None,
            blok_siniri: BLOK_SINIRI_VARSAYILAN,
        }
    }
}

/// K-384: `bağlantı_blok_sınırı` varsayılanı ve aralığı (bayt).
pub const BLOK_SINIRI_VARSAYILAN: u64 = 32 * 1024 * 1024;
pub const BLOK_SINIRI_ALT: u64 = 1024;
pub const BLOK_SINIRI_UST: u64 = 1024 * 1024 * 1024;
/// Blok akışının (okuma ve aktarma) asgari hızı, bayt/sn. Süre bütçesi
/// `bağlantı_boşta_süresi + bayt / hız` olarak çağrının başından mutlaktır;
/// damla damla bayt bütçeyi bu hızın altında uzatamaz (K-372 ilkesi).
pub const AKIS_ASGARI_HIZI: u64 = 64 * 1024;

/// Blok akışının çağrı başından mutlak süre bütçesi (ms).
pub fn akis_butcesi_ms(bosta_ms: i64, bayt: u64) -> i64 {
    let ek = bayt.saturating_mul(1000) / AKIS_ASGARI_HIZI;
    bosta_ms.saturating_add(i64::try_from(ek).unwrap_or(i64::MAX))
}

/// Yarıda kalan blok/literal okuma yönünü kapattı.
pub const OKUMA_KAPALI: &str =
    "önceki blok yarıda kaldığı için okuma yönü kapandı; bağlantıyı kapat";
/// Bildirilen literal/aktarım `bağlantı_blok_sınırı`nı aşıyor (bayt tüketilmez).
pub const BLOK_SINIRI_ASILDI: &str = "blok bağlantı_blok_sınırını aşıyor; hiçbir bayt okunmadı";
/// K-396: satırlı base64 aktarımının kaynak dosya tavanı (bayt). K-395'in
/// 32 MiB çözerek yazma hedef tavanıyla hizalıdır; `bağlantı_blok_sınırı`
/// daha büyük olsa da kaynak bunu aşamaz (kodlanmış çıktı ≈ 1,37 kat).
pub const BASE64_KAYNAK_TAVANI: u64 = 32 * 1024 * 1024;
/// Kaynak dosya [`BASE64_KAYNAK_TAVANI`]'nı aşıyor (bayt yazılmaz).
pub const BASE64_KAYNAK_TAVANI_ASILDI: &str =
    "kaynak dosya 32 MiB base64 aktarım tavanını aşıyor; hiçbir bayt yazılmadı";
/// Bellekteki bayt okuması satır sınırını aşıyor (bayt tüketilmez).
pub const BAYT_OKUMA_SINIRI: &str =
    "bellekteki bayt okuması bağlantı_satır_sınırını aşamaz; büyük literal'i dosyaya al";

/// Blok okumasının sınıflandırılmış reddi; Zee'ye C030 `tür` olarak çıkar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlokHatasi {
    /// Blok `bağlantı_blok_sınırı`nı aştı; kalanı sonlandırıcıya kadar atıldı.
    BoyutAsimi,
    /// Nokta bloğunda çıplak CR ya da LF; kalanı sonlandırıcıya kadar atıldı.
    SatirSonu,
}

impl BlokHatasi {
    pub const fn turu(self) -> &'static str {
        match self {
            Self::BoyutAsimi => "boyut_aşımı",
            Self::SatirSonu => "satır_sonu",
        }
    }

    pub const fn mesaji(self) -> &'static str {
        match self {
            Self::BoyutAsimi => "blok bağlantı_blok_sınırını aşıyor; kalanı atıldı",
            Self::SatirSonu => {
                "blokta çıplak CR ya da LF var (yalnız CRLF satır sonudur); blok atıldı"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoktaDurumu {
    /// Satır başı (blok başı da satır başıdır: komut satırının CRLF'si).
    SatirBasi,
    /// Satır başında `.` görüldü.
    SatirBasiNokta,
    /// Satır başında `.` CR görüldü; LF sonlandırıcıdır.
    NoktaCr,
    Satir,
    /// Satır içinde CR görüldü; yalnız LF geçerlidir.
    Cr,
}

/// Akışlı blok çözücü: SMTP `DATA` nokta bloğu (nokta doldurma çözülür,
/// satırlar CRLF ile korunur, sonlandırıcı çıktıya girmez) ya da tam `n`
/// baytlık IMAP literal'i. Bellek kullanımı sabittir; çıktı çağırana parça
/// parça verilir. Hata (sınır, çıplak CR/LF) ilk olanıdır ve sonrasında
/// çıktı üretilmez ama akış sonlandırıcıya kadar tüketilir.
#[derive(Debug, Clone)]
pub struct BlokOkuyucu {
    nokta: bool,
    durum: NoktaDurumu,
    kalan: u64,
    sinir: u64,
    yazilan: u64,
    hata: Option<BlokHatasi>,
    bitti: bool,
}

impl BlokOkuyucu {
    /// SMTP `DATA` nokta bloğu; `sinir` çözülmüş içerik baytıdır.
    pub fn nokta(sinir: u64) -> Self {
        Self {
            nokta: true,
            durum: NoktaDurumu::SatirBasi,
            kalan: 0,
            sinir,
            yazilan: 0,
            hata: None,
            bitti: false,
        }
    }

    /// Tam `adet` bayt (sınır denetimini çağıran yapar).
    pub fn bayt(adet: u64) -> Self {
        Self {
            nokta: false,
            durum: NoktaDurumu::Satir,
            kalan: adet,
            sinir: adet,
            yazilan: 0,
            hata: None,
            bitti: adet == 0,
        }
    }

    pub fn bitti(&self) -> bool {
        self.bitti
    }

    pub fn hata(&self) -> Option<BlokHatasi> {
        self.hata
    }

    /// Çıktıya verilen (dosyaya yazılacak) bayt.
    pub fn yazilan(&self) -> u64 {
        self.yazilan
    }

    fn yaz(&mut self, parca: &[u8], cikti: &mut Vec<u8>) {
        if self.hata.is_some() || parca.is_empty() {
            return;
        }
        let yeni = self.yazilan.saturating_add(parca.len() as u64);
        if yeni > self.sinir {
            self.hata = Some(BlokHatasi::BoyutAsimi);
            return;
        }
        cikti.extend_from_slice(parca);
        self.yazilan = yeni;
    }

    fn bozuk(&mut self) {
        self.hata.get_or_insert(BlokHatasi::SatirSonu);
    }

    /// `giris`ten tüketir, çözülmüş içeriği `cikti`ya ekler; tüketilen bayt
    /// sayısını döndürür. Blok bittiyse kalan girdi tüketilmez (sonraki
    /// satırlar komuttur).
    pub fn besle(&mut self, giris: &[u8], cikti: &mut Vec<u8>) -> usize {
        if self.bitti {
            return 0;
        }
        if !self.nokta {
            let adet = self.kalan.min(giris.len() as u64) as usize;
            cikti.extend_from_slice(&giris[..adet]);
            self.kalan -= adet as u64;
            self.yazilan += adet as u64;
            self.bitti = self.kalan == 0;
            return adet;
        }
        let mut i = 0;
        while i < giris.len() {
            let c = giris[i];
            self.durum = match self.durum {
                NoktaDurumu::Satir => {
                    // Hızlı yol: satır sonuna kadar olan baytları tek parça yaz.
                    let son = giris[i..]
                        .iter()
                        .position(|b| *b == b'\r' || *b == b'\n')
                        .map_or(giris.len(), |p| i + p);
                    if son > i {
                        self.yaz(&giris[i..son], cikti);
                        i = son;
                        continue;
                    }
                    if c == b'\r' {
                        NoktaDurumu::Cr
                    } else {
                        self.bozuk();
                        NoktaDurumu::Satir
                    }
                }
                NoktaDurumu::SatirBasi => match c {
                    b'.' => NoktaDurumu::SatirBasiNokta,
                    b'\r' => NoktaDurumu::Cr,
                    b'\n' => {
                        self.bozuk();
                        NoktaDurumu::Satir
                    }
                    _ => {
                        self.yaz(&[c], cikti);
                        NoktaDurumu::Satir
                    }
                },
                NoktaDurumu::SatirBasiNokta => match c {
                    b'\r' => NoktaDurumu::NoktaCr,
                    b'\n' => {
                        self.bozuk();
                        NoktaDurumu::Satir
                    }
                    // Nokta doldurma: satır başındaki ilk nokta atılır.
                    _ => {
                        self.yaz(&[c], cikti);
                        NoktaDurumu::Satir
                    }
                },
                NoktaDurumu::NoktaCr if c == b'\n' => {
                    self.bitti = true;
                    return i + 1;
                }
                NoktaDurumu::Cr if c == b'\n' => {
                    self.yaz(b"\r\n", cikti);
                    NoktaDurumu::SatirBasi
                }
                // Çıplak CR: bayt yeniden değerlendirilir ama satır başı sayılmaz.
                NoktaDurumu::NoktaCr | NoktaDurumu::Cr => {
                    self.bozuk();
                    if c == b'\r' {
                        NoktaDurumu::Cr
                    } else {
                        NoktaDurumu::Satir
                    }
                }
            };
            i += 1;
        }
        giris.len()
    }
}

/// STARTTLS anında Zee'nin tüketmediği düz metin (CVE-2011-0411 sınıfı).
pub const STARTTLS_ENJEKSIYONU: &str =
    "STARTTLS'ten sonra el sıkışmadan önce şifresiz veri geldi; bağlantı kapatıldı (komut enjeksiyonu)";
/// TLS kimliği bildirilmemiş süreçte STARTTLS.
pub const TLS_KIMLIGI_YOK: &str =
    "TLS kimliği bildirilmedi (tls_sertifika_değişkeni / tls_anahtar_değişkeni)";
/// TLS bağlantısında ikinci yükseltme.
pub const ZATEN_TLS: &str = "bağlantı zaten TLS";

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

    /// K-384: blok okuması için henüz tüketilmemiş ham baytlar.
    pub fn tampon(&self) -> &[u8] {
        &self.tampon
    }

    /// K-384: blok okumasının tükettiği baytları atar.
    pub fn tuket_bayt(&mut self, adet: usize) {
        self.tuket(adet);
    }

    /// K-384: tam `adet` bayt hazırsa alır (bayt-kesin literal).
    pub fn bayt_al(&mut self, adet: usize) -> Option<Vec<u8>> {
        if self.tampon.len() < adet {
            return None;
        }
        let bayt = self.tampon[..adet].to_vec();
        self.tuket(adet);
        Some(bayt)
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

/// K-384: bellekteki bayt-kesin okumanın Metin'i: geçerli UTF-8, NUL yok
/// (CR/LF ve TAB literal içeriği olabilir; bu Metin satır olarak geri
/// yazılmak istenirse `giden_satir` reddeder).
pub fn bayt_metni(bayt: Vec<u8>) -> Result<String, SatirHatasi> {
    if bayt.contains(&0) {
        return Err(SatirHatasi::Kodlama);
    }
    String::from_utf8(bayt).map_err(|_| SatirHatasi::Kodlama)
}

/// K-384 (ZM-F031): satır sonu eklemeden yazılacak parça. Satır kuralı
/// `giden_satir` ile aynıdır (CR, LF, NUL ve denetim karakteri yok); satır
/// sonunu her zaman `… satırını yaz` üretir, böylece Zee metninde `\r`
/// kaçışı gerekmez ve CRLF dışı satır sonu hiç yazılamaz.
pub fn giden_parca(metin: &str, sinir: usize) -> Result<Vec<u8>, &'static str> {
    if metin.bytes().any(denetim_karakteri) {
        return Err("parça CR, LF, NUL ya da denetim karakteri taşıyamaz");
    }
    if metin.len() > sinir {
        return Err("parça bağlantı satır sınırını aşıyor");
    }
    Ok(metin.as_bytes().to_vec())
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

    fn blok(sinir: u64, parcalar: &[&[u8]]) -> (Vec<u8>, Option<BlokHatasi>, bool, usize) {
        let mut okuyucu = BlokOkuyucu::nokta(sinir);
        let mut cikti = Vec::new();
        let mut tuketilen = 0;
        for parca in parcalar {
            tuketilen += okuyucu.besle(parca, &mut cikti);
        }
        (cikti, okuyucu.hata(), okuyucu.bitti(), tuketilen)
    }

    #[test]
    fn nokta_blogu_doldurmayi_cozer_sonlandiriciyi_atar() {
        let girdi = b"Subject: a\r\n\r\n..nokta\r\n.\r\nQUIT\r\n";
        let (cikti, hata, bitti, tuketilen) = blok(1 << 20, &[girdi]);
        assert_eq!(cikti, b"Subject: a\r\n\r\n.nokta\r\n");
        assert_eq!((hata, bitti), (None, true));
        // Sonlandırıcıdan sonraki komut tüketilmez.
        assert_eq!(&girdi[tuketilen..], b"QUIT\r\n");
        // Boş ileti ve tek tek bayt teslimi aynı sonucu verir.
        assert_eq!(blok(10, &[b".\r\n"]).0, b"");
        let parcalar: Vec<&[u8]> = girdi.chunks(1).collect();
        let (tek, _, bitti, _) = blok(1 << 20, &parcalar);
        assert_eq!((tek.as_slice(), bitti), (&cikti[..], true));
    }

    #[test]
    fn nokta_blogunda_smuggling_dizileri_sonlandirici_sayilmaz() {
        // <LF>.<LF>, <LF>.<CR><LF>, <CR>.<CR>: blok bitmez, satır_sonu ile atılır.
        for kacak in [
            &b"a\n.\nMAIL FROM:<x@y>\r\n.\r\n"[..],
            b"a\n.\r\nMAIL FROM:<x@y>\r\n.\r\n",
            b"a\r.\rMAIL FROM:<x@y>\r\n.\r\n",
            b"a\r\n.\nMAIL FROM:<x@y>\r\n.\r\n",
        ] {
            let (cikti, hata, bitti, tuketilen) = blok(1 << 20, &[kacak]);
            assert_eq!(hata, Some(BlokHatasi::SatirSonu), "{kacak:?}");
            assert!(bitti, "{kacak:?}");
            assert_eq!(tuketilen, kacak.len(), "{kacak:?}");
            assert!(!cikti.windows(4).any(|w| w == b"MAIL"), "{kacak:?}");
        }
        // Sonlandırıcı gelmezse blok sürer (kaçak ". " komut olarak görünmez).
        let (_, hata, bitti, _) = blok(1 << 20, &[b"a\n.\nQUIT\r\n"]);
        assert_eq!((hata, bitti), (Some(BlokHatasi::SatirSonu), false));
    }

    #[test]
    fn nokta_blogu_siniri_asinca_sonlandiriciya_kadar_atar() {
        let mut girdi = vec![b'x'; 2000];
        girdi.extend_from_slice(b"\r\n.\r\nNOOP\r\n");
        let (cikti, hata, bitti, tuketilen) = blok(1024, &[&girdi]);
        assert_eq!((hata, bitti), (Some(BlokHatasi::BoyutAsimi), true));
        assert!(cikti.len() <= 1024);
        assert_eq!(&girdi[tuketilen..], b"NOOP\r\n");
        // Tam sınır geçer (CRLF dahil çözülmüş içerik).
        let mut tam = vec![b'y'; 1022];
        tam.extend_from_slice(b"\r\n.\r\n");
        assert_eq!(blok(1024, &[&tam]).1, None);
    }

    #[test]
    fn bayt_blogu_tam_adet_tuketir() {
        let mut okuyucu = BlokOkuyucu::bayt(5);
        let mut cikti = Vec::new();
        assert_eq!(okuyucu.besle(b"ab", &mut cikti), 2);
        assert!(!okuyucu.bitti());
        assert_eq!(okuyucu.besle(b"c\r\nX)", &mut cikti), 3);
        assert!(okuyucu.bitti());
        assert_eq!(cikti, b"abc\r\n");
        assert_eq!(okuyucu.besle(b"X", &mut cikti), 0);
        assert!(BlokOkuyucu::bayt(0).bitti());
    }

    #[test]
    fn bayt_al_ve_bayt_metni_kesin_adet_ister() {
        let mut okuyucu = SatirOkuyucu::yeni(64);
        okuyucu.besle(b"gizli\r\n");
        assert_eq!(okuyucu.bayt_al(9), None);
        assert_eq!(okuyucu.bayt_al(5), Some(b"gizli".to_vec()));
        assert_eq!(okuyucu.siradaki(), Some(Ok(String::new())));
        assert_eq!(bayt_metni(b"a\r\nb".to_vec()), Ok("a\r\nb".into()));
        assert_eq!(bayt_metni(b"a\0".to_vec()), Err(SatirHatasi::Kodlama));
        assert_eq!(bayt_metni(b"\xff".to_vec()), Err(SatirHatasi::Kodlama));
    }

    #[test]
    fn giden_parca_satir_sonu_eklemez_enjeksiyonu_reddeder() {
        assert_eq!(giden_parca("+ ", 1000).unwrap(), b"+ ");
        assert_eq!(giden_parca("", 1000).unwrap(), b"");
        for kotu in ["a\r\n", "a\nb", "a\rb", "a\0b"] {
            assert!(giden_parca(kotu, 1000).is_err(), "{kotu:?}");
        }
        assert!(giden_parca(&"x".repeat(64), 64).is_ok());
        assert!(giden_parca(&"x".repeat(65), 64).is_err());
        assert_eq!(akis_butcesi_ms(1000, 64 * 1024), 2000);
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
