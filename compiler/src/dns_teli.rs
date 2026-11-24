//! K-390 (RFC-0044 B1, ADR-169): sınırlı DNS tel biçimi kodlayıcı/ayrıştırıcısı.
//!
//! Üretimde DNS crate'i yoktur. Bu modül yalnız stub istemcinin ihtiyacını
//! karşılar: tek soruluk özyinelemeli sorgu kurmak (RD=1, AD=1, EDNS0 1232
//! bayt, DO=0) ve yanıtı fail-closed ayrıştırmak. Kabul kümesi bilinçli
//! olarak dardır: tanınmayan kayıt türü, sondaki fazla bayt, ileri ya da
//! zincirleme taşan sıkıştırma işaretçisi, rdata uzunluğuyla tutarsız kayıt
//! reddedilir. `hickory-proto` yalnız dev-dependency kâhinidir; Zee'nin kabul
//! ettiği her paketi o da kabul etmeli ve cevap bölümünü aynı okumalıdır.
//!
//! Saf modüldür (ağ, saat, rastgelelik yok); wasm32'de de derlenir.
#![forbid(unsafe_code)]

/// UDP EDNS tamponu (DNS Flag Day 2020 parçalanma önerisi).
pub const UDP_TAMPONU: u16 = 1232;
/// TCP yanıtının tel biçimi üst sınırı.
pub const AZAMI_TCP_YANITI: usize = 65_535;
/// Cevap bölümündeki kayıt sınırı (amplifikasyon ve bellek).
pub const AZAMI_CEVAP: usize = 64;
/// Dört bölümün toplam kayıt sınırı.
pub const AZAMI_TOPLAM_KAYIT: usize = 128;
/// İzlenecek en uzun CNAME zinciri.
pub const AZAMI_CNAME: usize = 8;
/// Bir TXT kaydının birleştirilmiş en büyük boyu (4096 bit DKIM anahtarı sığar).
pub const AZAMI_TXT_BAYTI: usize = 4096;
/// TLSA ilişkili verisinin en büyük boyu (tam sertifika dahil).
pub const AZAMI_TLSA_VERISI: usize = 4096;
/// Metin biçimli adın en büyük oktet sayısı (sondaki nokta hariç).
pub const AZAMI_AD: usize = 253;
/// Tel biçimli adın en büyük boyu (uzunluk baytları dahil).
const AZAMI_TEL_ADI: usize = 255;
/// Bir adın çözümünde izlenecek en çok sıkıştırma işaretçisi.
const AZAMI_ISARETCI: usize = 16;

const TUR_A: u16 = 1;
const TUR_NS: u16 = 2;
const TUR_CNAME: u16 = 5;
const TUR_SOA: u16 = 6;
const TUR_MX: u16 = 15;
const TUR_TXT: u16 = 16;
const TUR_AAAA: u16 = 28;
const TUR_DNAME: u16 = 39;
const TUR_OPT: u16 = 41;
const TUR_TLSA: u16 = 52;
const SINIF_IN: u16 = 1;

/// Zee yüzeyinin sorgulayabildiği kayıt türleri (SOA yalnız DNSSEC sınaması).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KayitTuru {
    A,
    Aaaa,
    Mx,
    Txt,
    Tlsa,
    Soa,
}

impl KayitTuru {
    pub const TUMU: [Self; 6] = [
        Self::A,
        Self::Aaaa,
        Self::Mx,
        Self::Txt,
        Self::Tlsa,
        Self::Soa,
    ];

    pub fn kodu(self) -> u16 {
        match self {
            Self::A => TUR_A,
            Self::Aaaa => TUR_AAAA,
            Self::Mx => TUR_MX,
            Self::Txt => TUR_TXT,
            Self::Tlsa => TUR_TLSA,
            Self::Soa => TUR_SOA,
        }
    }

    /// IO izi ve tanılardaki kararlı yazım.
    pub fn yazimi(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::Aaaa => "aaaa",
            Self::Mx => "mx",
            Self::Txt => "txt",
            Self::Tlsa => "tlsa",
            Self::Soa => "soa",
        }
    }

    pub fn ayristir(yazim: &str) -> Option<Self> {
        Self::TUMU.into_iter().find(|tur| tur.yazimi() == yazim)
    }
}

/// Tel biçimi hatası: bozuk ya da sınırı aşan paket, geçersiz sorgu adı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelHatasi {
    GecersizAd(&'static str),
    Bozuk(&'static str),
    BoyutAsimi(&'static str),
}

/// Zee'ye `tür` verisiyle görünen geçici hata sınıfları (RFC-0044 B1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsHataTuru {
    ZamanAsimi,
    SunucuHatasi,
    Ret,
    Ag,
    BoyutAsimi,
    Butce,
    GecersizAd,
    BozukYanit,
}

impl DnsHataTuru {
    pub const TUMU: [Self; 8] = [
        Self::ZamanAsimi,
        Self::SunucuHatasi,
        Self::Ret,
        Self::Ag,
        Self::BoyutAsimi,
        Self::Butce,
        Self::GecersizAd,
        Self::BozukYanit,
    ];

    pub fn yazimi(self) -> &'static str {
        match self {
            Self::ZamanAsimi => "zaman_aşımı",
            Self::SunucuHatasi => "sunucu_hatası",
            Self::Ret => "ret",
            Self::Ag => "ağ",
            Self::BoyutAsimi => "boyut_aşımı",
            Self::Butce => "bütçe",
            Self::GecersizAd => "geçersiz_ad",
            Self::BozukYanit => "bozuk_yanıt",
        }
    }

    pub fn ayristir(yazim: &str) -> Option<Self> {
        Self::TUMU.into_iter().find(|tur| tur.yazimi() == yazim)
    }
}

/// DNS sorgusunun başarısız sonucu. Hiçbiri "kayıt yok" anlamına gelmez.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsHatasi {
    pub tur: DnsHataTuru,
    pub mesaj: String,
}

impl DnsHatasi {
    pub fn yeni(tur: DnsHataTuru, mesaj: impl Into<String>) -> Self {
        Self {
            tur,
            mesaj: mesaj.into(),
        }
    }
}

/// IO adaptörünün DNS sorgu sonucu.
pub type DnsSonucu = Result<Cozum, DnsHatasi>;

/// DNS desteklemeyen IO adaptörünün varsayılan sonucu.
pub fn desteklenmeyen() -> DnsSonucu {
    Err(DnsHatasi::yeni(
        DnsHataTuru::Ag,
        "IO adaptörü DNS sorgusunu desteklemiyor",
    ))
}

impl From<TelHatasi> for DnsHatasi {
    fn from(hata: TelHatasi) -> Self {
        match hata {
            TelHatasi::GecersizAd(neden) => Self::yeni(DnsHataTuru::GecersizAd, neden),
            TelHatasi::Bozuk(neden) => Self::yeni(DnsHataTuru::BozukYanit, neden),
            TelHatasi::BoyutAsimi(neden) => Self::yeni(DnsHataTuru::BoyutAsimi, neden),
        }
    }
}

/// Ayrıştırılmış kayıt verisi. Adlar küçük harfli ve sondaki noktasızdır
/// (kök `""`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KayitVerisi {
    A([u8; 4]),
    Aaaa([u8; 16]),
    Mx {
        oncelik: u16,
        hedef: String,
    },
    /// Parçaları birleştirilmiş TXT baytları (RFC 7208 §3.3, RFC 6376 §3.6.2.2).
    Txt(Vec<u8>),
    Tlsa {
        kullanim: u8,
        secici: u8,
        esleme: u8,
        veri: Vec<u8>,
    },
    Soa {
        birincil: String,
    },
    Cname(String),
    /// Yapısı doğrulanmış, içeriği kullanılmayan kayıt (NS, DNAME, OPT).
    Diger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HamKayit {
    pub ad: String,
    pub tur: u16,
    pub sinif: u16,
    pub ttl: u32,
    pub veri: KayitVerisi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yanit {
    pub kimlik: u16,
    pub kesik: bool,
    /// Başlıktaki AD biti; anlamı yalnız loopback doğrulayıcı çözücüden gelir.
    pub ad_biti: bool,
    pub yanit_kodu: u8,
    pub soru_adi: String,
    pub soru_turu: u16,
    pub soru_sinifi: u16,
    pub cevaplar: Vec<HamKayit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durum {
    Var,
    /// NOERROR + istenen türde kayıt yok (NODATA).
    KayitYok,
    /// NXDOMAIN.
    AlanYok,
}

impl Durum {
    pub fn yazimi(self) -> &'static str {
        match self {
            Self::Var => "var",
            Self::KayitYok => "kayıt_yok",
            Self::AlanYok => "alan_yok",
        }
    }
    pub fn ayristir(yazim: &str) -> Option<Self> {
        [Self::Var, Self::KayitYok, Self::AlanYok]
            .into_iter()
            .find(|d| d.yazimi() == yazim)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kayit {
    pub ad: String,
    pub ttl: u32,
    pub veri: KayitVerisi,
}

/// Bir sorgunun kalıcı (yetkili) cevabı. `AlanYok` ve `KayitYok` başarıdır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cozum {
    pub durum: Durum,
    pub kayitlar: Vec<Kayit>,
    /// Çözücünün AD biti; adaptör `dns_dnssec: "zorunlu"` değilse yanlışa indirir.
    pub dogrulanmis: bool,
    /// CNAME zinciri sonrası kanonik ad.
    pub kanonik_ad: String,
}

/// Sorgu adını doğrular ve kanonikleştirir: ASCII harf/rakam/`-`/`_`
/// etiketleri, etiket 1–63, toplam ≤ 253 oktet, küçük harf, sondaki nokta
/// atılır. Kök `"."` boş adla döner. U-label (ASCII dışı) reddedilir.
pub fn ad_dogrula(ad: &str) -> Result<String, TelHatasi> {
    if ad == "." {
        return Ok(String::new());
    }
    let govde = ad.strip_suffix('.').unwrap_or(ad);
    if govde.is_empty() {
        return Err(TelHatasi::GecersizAd("ad boş"));
    }
    if govde.len() > AZAMI_AD {
        return Err(TelHatasi::GecersizAd("ad 253 okteti aşıyor"));
    }
    if !govde.is_ascii() {
        return Err(TelHatasi::GecersizAd(
            "yalnız ASCII (A-label) adlar desteklenir",
        ));
    }
    for etiket in govde.split('.') {
        if etiket.is_empty() || etiket.len() > 63 {
            return Err(TelHatasi::GecersizAd("etiket 1-63 oktet olmalı"));
        }
        if !etiket
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(TelHatasi::GecersizAd(
                "etiket yalnız harf, rakam, - ve _ içerebilir",
            ));
        }
        if etiket.starts_with('-') || etiket.ends_with('-') {
            return Err(TelHatasi::GecersizAd(
                "etiket - ile başlayamaz ya da bitemez",
            ));
        }
    }
    Ok(govde.to_ascii_lowercase())
}

/// Tek soruluk sorgu paketi: RD=1, AD=1 (RFC 6840 §5.7), EDNS0 OPT
/// (1232 bayt, DO=0). `ad` [`ad_dogrula`] ile kanonikleştirilir; boş ad
/// (kanonik kök) yalnız iç DNSSEC sınamasından gelir.
pub fn sorgu_kur(kimlik: u16, ad: &str, tur: KayitTuru) -> Result<Vec<u8>, TelHatasi> {
    let ad = if ad.is_empty() {
        String::new()
    } else {
        ad_dogrula(ad)?
    };
    let mut paket = Vec::with_capacity(32 + ad.len());
    paket.extend_from_slice(&kimlik.to_be_bytes());
    paket.extend_from_slice(&0x0120u16.to_be_bytes()); // RD + AD
    paket.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
    paket.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT
    paket.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    paket.extend_from_slice(&1u16.to_be_bytes()); // ARCOUNT (OPT)
    if !ad.is_empty() {
        for etiket in ad.split('.') {
            paket.push(etiket.len() as u8);
            paket.extend_from_slice(etiket.as_bytes());
        }
    }
    paket.push(0);
    paket.extend_from_slice(&tur.kodu().to_be_bytes());
    paket.extend_from_slice(&SINIF_IN.to_be_bytes());
    // OPT: kök ad, tür 41, sınıf = UDP tamponu, TTL 0 (DO=0), rdata yok.
    paket.push(0);
    paket.extend_from_slice(&TUR_OPT.to_be_bytes());
    paket.extend_from_slice(&UDP_TAMPONU.to_be_bytes());
    paket.extend_from_slice(&0u32.to_be_bytes());
    paket.extend_from_slice(&0u16.to_be_bytes());
    Ok(paket)
}

struct Okuyucu<'a> {
    paket: &'a [u8],
    konum: usize,
}

impl<'a> Okuyucu<'a> {
    fn bayt(&mut self) -> Result<u8, TelHatasi> {
        let deger = *self
            .paket
            .get(self.konum)
            .ok_or(TelHatasi::Bozuk("paket beklenmedik biçimde bitti"))?;
        self.konum += 1;
        Ok(deger)
    }
    fn u16(&mut self) -> Result<u16, TelHatasi> {
        Ok(u16::from_be_bytes([self.bayt()?, self.bayt()?]))
    }
    fn u32(&mut self) -> Result<u32, TelHatasi> {
        Ok(u32::from_be_bytes([
            self.bayt()?,
            self.bayt()?,
            self.bayt()?,
            self.bayt()?,
        ]))
    }
    fn dilim(&mut self, uzunluk: usize) -> Result<&'a [u8], TelHatasi> {
        let son = self
            .konum
            .checked_add(uzunluk)
            .filter(|son| *son <= self.paket.len())
            .ok_or(TelHatasi::Bozuk("alan uzunluğu paketi aşıyor"))?;
        let dilim = &self.paket[self.konum..son];
        self.konum = son;
        Ok(dilim)
    }

    /// Sıkıştırmalı adı okur. İşaretçi yalnız geriye (bu adın başladığı
    /// segmentten önceye) ve en çok 16 kez izlenir; döngü yapısal olarak
    /// imkânsızdır. Tel adı ≤ 255 bayt.
    fn ad(&mut self) -> Result<String, TelHatasi> {
        let mut metin = String::new();
        let mut konum = self.konum;
        let mut segment_basi = self.konum;
        let mut donus: Option<usize> = None;
        let mut isaretci = 0usize;
        let mut tel_boyu = 1usize; // kök bayt
        loop {
            let uzunluk = *self
                .paket
                .get(konum)
                .ok_or(TelHatasi::Bozuk("ad paket sonunda kesildi"))?;
            match uzunluk & 0xC0 {
                0x00 => {
                    if uzunluk == 0 {
                        konum += 1;
                        break;
                    }
                    let uzunluk = usize::from(uzunluk);
                    let etiket = self
                        .paket
                        .get(konum + 1..konum + 1 + uzunluk)
                        .ok_or(TelHatasi::Bozuk("etiket paketi aşıyor"))?;
                    tel_boyu += 1 + uzunluk;
                    if tel_boyu > AZAMI_TEL_ADI {
                        return Err(TelHatasi::Bozuk("ad 255 baytı aşıyor"));
                    }
                    if !etiket
                        .iter()
                        .all(|b| matches!(b, 0x21..=0x7E) && *b != b'.' && *b != b'\\')
                    {
                        return Err(TelHatasi::Bozuk("etikette desteklenmeyen bayt"));
                    }
                    if !metin.is_empty() {
                        metin.push('.');
                    }
                    for b in etiket {
                        metin.push(char::from(b.to_ascii_lowercase()));
                    }
                    konum += 1 + uzunluk;
                }
                0xC0 => {
                    let ikinci = *self
                        .paket
                        .get(konum + 1)
                        .ok_or(TelHatasi::Bozuk("işaretçi kesildi"))?;
                    let hedef = (usize::from(uzunluk & 0x3F) << 8) | usize::from(ikinci);
                    isaretci += 1;
                    if isaretci > AZAMI_ISARETCI {
                        return Err(TelHatasi::Bozuk("çok fazla sıkıştırma işaretçisi"));
                    }
                    if hedef < 12 || hedef >= segment_basi {
                        return Err(TelHatasi::Bozuk("sıkıştırma işaretçisi geriye göstermiyor"));
                    }
                    if donus.is_none() {
                        donus = Some(konum + 2);
                    }
                    konum = hedef;
                    segment_basi = hedef;
                }
                _ => return Err(TelHatasi::Bozuk("desteklenmeyen etiket türü")),
            }
        }
        self.konum = donus.unwrap_or(konum);
        Ok(metin)
    }
}

/// TXT rdata: bir ya da daha çok `<uzunluk><bayt>` parçası, birleşik ≤ 4 KiB.
fn txt_oku(
    okuyucu: &mut Okuyucu<'_>,
    uzunluk: usize,
    son: usize,
) -> Result<KayitVerisi, TelHatasi> {
    if uzunluk == 0 {
        return Err(TelHatasi::Bozuk("TXT kaydı en az bir parça taşımalı"));
    }
    if uzunluk > AZAMI_TXT_BAYTI + AZAMI_TXT_BAYTI / 255 + 1 {
        return Err(TelHatasi::BoyutAsimi("TXT kaydı 4 KiB'ı aşıyor"));
    }
    let mut metin = Vec::new();
    while okuyucu.konum < son {
        let parca = usize::from(okuyucu.bayt()?);
        if okuyucu.konum + parca > son {
            return Err(TelHatasi::Bozuk("TXT parçası rdata'yı aşıyor"));
        }
        metin.extend_from_slice(okuyucu.dilim(parca)?);
    }
    if metin.len() > AZAMI_TXT_BAYTI {
        return Err(TelHatasi::BoyutAsimi("TXT kaydı 4 KiB'ı aşıyor"));
    }
    Ok(KayitVerisi::Txt(metin))
}

/// OPT (yalnız ek bölümde, kök adlı): seçenek TLV'leri rdata'yı tam doldurur.
/// NSID (3) ve ECS (8) istenmez; içerikleri ayrıştırılmadığı için kabul
/// kümesini dar tutmak üzere reddedilir.
fn opt_oku(okuyucu: &mut Okuyucu<'_>, ad: &str, son: usize) -> Result<KayitVerisi, TelHatasi> {
    if !ad.is_empty() {
        return Err(TelHatasi::Bozuk("OPT kaydının adı kök olmalı"));
    }
    while okuyucu.konum < son {
        if matches!(okuyucu.u16()?, 3 | 8) {
            return Err(TelHatasi::Bozuk("istenmeyen EDNS seçeneği"));
        }
        let secenek = usize::from(okuyucu.u16()?);
        if okuyucu.konum + secenek > son {
            return Err(TelHatasi::Bozuk("EDNS seçeneği rdata'yı aşıyor"));
        }
        okuyucu.dilim(secenek)?;
    }
    Ok(KayitVerisi::Diger)
}

/// Bir bölüm kaydı: yapısı her bölümde doğrulanır, içerik yalnız cevapta kullanılır.
fn kayit_oku(okuyucu: &mut Okuyucu<'_>, ek_bolum: bool) -> Result<HamKayit, TelHatasi> {
    let ad = okuyucu.ad()?;
    let tur = okuyucu.u16()?;
    let sinif = okuyucu.u16()?;
    let ttl = okuyucu.u32()?;
    let uzunluk = usize::from(okuyucu.u16()?);
    let baslangic = okuyucu.konum;
    let son = baslangic
        .checked_add(uzunluk)
        .filter(|son| *son <= okuyucu.paket.len())
        .ok_or(TelHatasi::Bozuk("rdata paketi aşıyor"))?;
    let veri = match tur {
        TUR_A => {
            let dilim = okuyucu.dilim(uzunluk)?;
            KayitVerisi::A(
                dilim
                    .try_into()
                    .map_err(|_| TelHatasi::Bozuk("A kaydı 4 bayt olmalı"))?,
            )
        }
        TUR_AAAA => {
            let dilim = okuyucu.dilim(uzunluk)?;
            KayitVerisi::Aaaa(
                dilim
                    .try_into()
                    .map_err(|_| TelHatasi::Bozuk("AAAA kaydı 16 bayt olmalı"))?,
            )
        }
        TUR_MX => {
            let oncelik = okuyucu.u16()?;
            let hedef = okuyucu.ad()?;
            KayitVerisi::Mx { oncelik, hedef }
        }
        TUR_CNAME => KayitVerisi::Cname(okuyucu.ad()?),
        TUR_NS | TUR_DNAME => {
            okuyucu.ad()?;
            KayitVerisi::Diger
        }
        TUR_SOA => {
            let birincil = okuyucu.ad()?;
            okuyucu.ad()?;
            okuyucu.dilim(20)?;
            KayitVerisi::Soa { birincil }
        }
        TUR_TXT => txt_oku(okuyucu, uzunluk, son)?,
        TUR_TLSA => {
            if uzunluk < 4 {
                return Err(TelHatasi::Bozuk("TLSA kaydı en az 4 bayt olmalı"));
            }
            if uzunluk - 3 > AZAMI_TLSA_VERISI {
                return Err(TelHatasi::BoyutAsimi("TLSA verisi 4 KiB'ı aşıyor"));
            }
            let kullanim = okuyucu.bayt()?;
            let secici = okuyucu.bayt()?;
            let esleme = okuyucu.bayt()?;
            let veri = okuyucu.dilim(uzunluk - 3)?.to_vec();
            KayitVerisi::Tlsa {
                kullanim,
                secici,
                esleme,
                veri,
            }
        }
        TUR_OPT if ek_bolum => opt_oku(okuyucu, &ad, son)?,
        _ => return Err(TelHatasi::Bozuk("desteklenmeyen kayıt türü")),
    };
    if okuyucu.konum != son {
        return Err(TelHatasi::Bozuk("rdata uzunluğu içerikle uyuşmuyor"));
    }
    Ok(HamKayit {
        ad,
        tur,
        sinif,
        ttl,
        veri,
    })
}

/// Yanıt paketini ayrıştırır. Yalnız `QR=1`, `OPCODE=0`, tek soru; bütün
/// bölümler yapısal olarak doğrulanır, sondaki fazla bayt reddedilir.
pub fn yanit_ayristir(paket: &[u8]) -> Result<Yanit, TelHatasi> {
    if paket.len() > AZAMI_TCP_YANITI {
        return Err(TelHatasi::BoyutAsimi("yanıt 64 KiB'ı aşıyor"));
    }
    let mut okuyucu = Okuyucu { paket, konum: 0 };
    let kimlik = okuyucu.u16()?;
    let bayraklar = okuyucu.u16()?;
    if bayraklar & 0x8000 == 0 {
        return Err(TelHatasi::Bozuk("paket yanıt değil"));
    }
    if (bayraklar >> 11) & 0x0F != 0 {
        return Err(TelHatasi::Bozuk("desteklenmeyen işlem kodu"));
    }
    let soru = okuyucu.u16()?;
    let cevap = usize::from(okuyucu.u16()?);
    let yetki = usize::from(okuyucu.u16()?);
    let ek = usize::from(okuyucu.u16()?);
    if soru != 1 {
        return Err(TelHatasi::Bozuk("yanıt tek soru taşımalı"));
    }
    if cevap > AZAMI_CEVAP {
        return Err(TelHatasi::BoyutAsimi("cevap bölümü 64 kaydı aşıyor"));
    }
    if cevap + yetki + ek > AZAMI_TOPLAM_KAYIT {
        return Err(TelHatasi::BoyutAsimi("yanıt 128 kaydı aşıyor"));
    }
    let soru_adi = okuyucu.ad()?;
    let soru_turu = okuyucu.u16()?;
    let soru_sinifi = okuyucu.u16()?;
    let mut cevaplar = Vec::with_capacity(cevap);
    for _ in 0..cevap {
        let kayit = kayit_oku(&mut okuyucu, false)?;
        cevaplar.push(kayit);
    }
    for _ in 0..yetki {
        kayit_oku(&mut okuyucu, false)?;
    }
    let mut opt = 0;
    for _ in 0..ek {
        if kayit_oku(&mut okuyucu, true)?.tur == TUR_OPT {
            opt += 1;
        }
    }
    if opt > 1 {
        return Err(TelHatasi::Bozuk("birden çok OPT kaydı"));
    }
    if okuyucu.konum != paket.len() {
        return Err(TelHatasi::Bozuk("yanıtın sonunda fazla bayt var"));
    }
    Ok(Yanit {
        kimlik,
        kesik: bayraklar & 0x0200 != 0,
        ad_biti: bayraklar & 0x0020 != 0,
        yanit_kodu: (bayraklar & 0x000F) as u8,
        soru_adi,
        soru_turu,
        soru_sinifi,
        cevaplar,
    })
}

/// Yanıt bu sorguya mı ait? Kimlik, soru adı (büyük/küçük harf duyarsız),
/// tür ve sınıf exact eşleşmeli; eşleşmeyen paket atılır.
pub fn eslesiyor(yanit: &Yanit, kimlik: u16, ad: &str, tur: KayitTuru) -> bool {
    yanit.kimlik == kimlik
        && yanit.soru_adi == ad
        && yanit.soru_turu == tur.kodu()
        && yanit.soru_sinifi == SINIF_IN
}

/// Yalnız başlık ve soru bölümünden: paket bu sorgunun yanıtı mı? (QR=1,
/// kimlik, tek soru, ad/tür/sınıf exact). Gövde bozuk olsa da eşleşen paket
/// bu sorgunun yanıtıdır; eşleşmeyen paket atılır (zehirleme önlemi).
pub fn basligi_eslesiyor(paket: &[u8], kimlik: u16, ad: &str, tur: KayitTuru) -> bool {
    let mut okuyucu = Okuyucu { paket, konum: 0 };
    let baslik = (|| -> Result<bool, TelHatasi> {
        let gelen = okuyucu.u16()?;
        let bayraklar = okuyucu.u16()?;
        let soru = okuyucu.u16()?;
        okuyucu.dilim(6)?;
        Ok(gelen == kimlik
            && bayraklar & 0x8000 != 0
            && soru == 1
            && okuyucu.ad()? == ad
            && okuyucu.u16()? == tur.kodu()
            && okuyucu.u16()? == SINIF_IN)
    })();
    baslik.unwrap_or(false)
}

/// CNAME zincirini (≤ 8 adım) izler: kanonik ad ve o adın istenen türdeki
/// sıralı, tekil kayıtları (yoksa boş).
fn zinciri_izle(
    yanit: &Yanit,
    ad: &str,
    tur: KayitTuru,
) -> Result<(String, Vec<Kayit>), DnsHatasi> {
    let mut guncel = ad.to_string();
    let mut adim = 0usize;
    loop {
        let mut kayitlar: Vec<Kayit> = yanit
            .cevaplar
            .iter()
            .filter(|k| k.sinif == SINIF_IN && k.tur == tur.kodu() && k.ad == guncel)
            .map(|k| Kayit {
                ad: k.ad.clone(),
                ttl: k.ttl,
                veri: k.veri.clone(),
            })
            .collect();
        if !kayitlar.is_empty() {
            kayitlar.sort_by(|a, b| a.veri.cmp(&b.veri));
            kayitlar.dedup_by(|a, b| a.veri == b.veri);
            return Ok((guncel, kayitlar));
        }
        let mut takma = yanit.cevaplar.iter().filter_map(|k| match &k.veri {
            KayitVerisi::Cname(hedef) if k.sinif == SINIF_IN && k.ad == guncel => Some(hedef),
            _ => None,
        });
        let Some(hedef) = takma.next() else {
            return Ok((guncel, Vec::new()));
        };
        if takma.next().is_some() {
            return Err(DnsHatasi::yeni(
                DnsHataTuru::BozukYanit,
                "aynı ad için birden çok CNAME",
            ));
        }
        adim += 1;
        if adim > AZAMI_CNAME {
            return Err(DnsHatasi::yeni(
                DnsHataTuru::BoyutAsimi,
                "CNAME zinciri 8 adımı aştı ya da döngü içeriyor",
            ));
        }
        guncel = hedef.clone();
    }
}

/// Eşleşmiş yanıttan kalıcı cevabı çıkarır. `SERVFAIL` (DNSSEC `bogus` dahil),
/// `REFUSED` ve diğer kodlar geçici hatadır; hiçbir yolda `AlanYok` olmaz.
pub fn coz(yanit: &Yanit, ad: &str, tur: KayitTuru) -> Result<Cozum, DnsHatasi> {
    match yanit.yanit_kodu {
        0 | 3 => {}
        2 => {
            return Err(DnsHatasi::yeni(
                DnsHataTuru::SunucuHatasi,
                "çözücü SERVFAIL döndü (DNSSEC doğrulaması başarısız olabilir)",
            ))
        }
        5 => {
            return Err(DnsHatasi::yeni(
                DnsHataTuru::Ret,
                "çözücü sorguyu reddetti (REFUSED)",
            ))
        }
        _ => {
            return Err(DnsHatasi::yeni(
                DnsHataTuru::SunucuHatasi,
                "çözücü beklenmeyen yanıt kodu döndü",
            ))
        }
    }
    if yanit.kesik {
        return Err(DnsHatasi::yeni(
            DnsHataTuru::BozukYanit,
            "kesik yanıt tam yanıt olarak kullanılamaz",
        ));
    }
    let (guncel, kayitlar) = zinciri_izle(yanit, ad, tur)?;
    if !kayitlar.is_empty() {
        return Ok(Cozum {
            durum: Durum::Var,
            kayitlar,
            dogrulanmis: yanit.ad_biti,
            kanonik_ad: guncel,
        });
    }
    Ok(Cozum {
        durum: if yanit.yanit_kodu == 3 {
            Durum::AlanYok
        } else {
            Durum::KayitYok
        },
        kayitlar: Vec::new(),
        dogrulanmis: yanit.ad_biti,
        kanonik_ad: guncel,
    })
}

/// `dns_dnssec` bildirimi (varsayılan yoktur).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnssecKipi {
    /// Açılışta çözücünün doğruladığı sınanır; `doğrulanmış` AD bitini taşır.
    Zorunlu,
    /// AD biti hiçbir güvenlik kararına bağlanmaz; `doğrulanmış` hep yanlış.
    Bilgi,
}

impl DnssecKipi {
    pub fn ayristir(yazim: &str) -> Option<Self> {
        match yazim {
            "zorunlu" => Some(Self::Zorunlu),
            "bilgi" => Some(Self::Bilgi),
            _ => None,
        }
    }
    pub fn yazimi(self) -> &'static str {
        match self {
            Self::Zorunlu => "zorunlu",
            Self::Bilgi => "bilgi",
        }
    }
}

/// `proje.dil` DNS bildirimi: yalnız loopback doğrulayıcı çözücü.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DnsAyari {
    pub cozucu: std::net::SocketAddr,
    pub dnssec: DnssecKipi,
}

impl DnsAyari {
    /// `127.0.0.1:53` / `[::1]:53` biçimi; loopback dışı adres ve 0 kapısı reddedilir.
    /// Uzak çözücünün AD biti güvenilmez (RFC 4035 §4.9.3, RFC 6698 §4.1).
    pub fn yeni(cozucu: &str, dnssec: DnssecKipi) -> Result<Self, &'static str> {
        let adres: std::net::SocketAddr = cozucu
            .parse()
            .map_err(|_| "çözücü IP:kapı biçiminde olmalı (örnek 127.0.0.1:53)")?;
        if !adres.ip().is_loopback() {
            return Err("çözücü yalnız loopback adres olabilir");
        }
        if adres.port() == 0 {
            return Err("çözücü kapısı 0 olamaz");
        }
        Ok(Self {
            cozucu: adres,
            dnssec,
        })
    }
}

/// Küçük harf onaltılık yazım (TLSA verisi ve IO izi TXT baytları).
pub fn onaltilik(baytlar: &[u8]) -> String {
    let mut metin = String::with_capacity(baytlar.len() * 2);
    for b in baytlar {
        metin.push(char::from(b"0123456789abcdef"[usize::from(b >> 4)]));
        metin.push(char::from(b"0123456789abcdef"[usize::from(b & 0x0F)]));
    }
    metin
}

/// [`onaltilik`]'in tersi; yalnız küçük harf, çift uzunluk.
pub fn onaltilik_coz(metin: &str) -> Option<Vec<u8>> {
    if !metin.len().is_multiple_of(2) {
        return None;
    }
    let deger = |b: u8| match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    };
    metin
        .as_bytes()
        .chunks(2)
        .map(|cift| Some(deger(cift[0])? << 4 | deger(cift[1])?))
        .collect()
}

#[cfg(test)]
mod testler {
    use super::*;

    fn ad_yaz(paket: &mut Vec<u8>, ad: &str) {
        for etiket in ad.split('.').filter(|e| !e.is_empty()) {
            paket.push(etiket.len() as u8);
            paket.extend_from_slice(etiket.as_bytes());
        }
        paket.push(0);
    }

    fn baslik(kimlik: u16, bayrak: u16, an: u16, ns: u16, ar: u16) -> Vec<u8> {
        let mut p = Vec::new();
        for alan in [kimlik, bayrak, 1, an, ns, ar] {
            p.extend_from_slice(&alan.to_be_bytes());
        }
        p
    }

    fn soru(p: &mut Vec<u8>, ad: &str, tur: u16) {
        ad_yaz(p, ad);
        p.extend_from_slice(&tur.to_be_bytes());
        p.extend_from_slice(&1u16.to_be_bytes());
    }

    fn kayit(p: &mut Vec<u8>, ad: &[u8], tur: u16, ttl: u32, veri: &[u8]) {
        p.extend_from_slice(ad);
        p.extend_from_slice(&tur.to_be_bytes());
        p.extend_from_slice(&1u16.to_be_bytes());
        p.extend_from_slice(&ttl.to_be_bytes());
        p.extend_from_slice(&(veri.len() as u16).to_be_bytes());
        p.extend_from_slice(veri);
    }

    #[test]
    fn ad_dogrulamasi_sinirlari_uygular() {
        assert_eq!(ad_dogrula("Ornek.TEST.").unwrap(), "ornek.test");
        assert_eq!(
            ad_dogrula("_dmarc.ornek.test").unwrap(),
            "_dmarc.ornek.test"
        );
        assert_eq!(ad_dogrula(".").unwrap(), "");
        assert!(ad_dogrula("").is_err());
        assert!(ad_dogrula("a..b").is_err());
        assert!(ad_dogrula("örnek.test").is_err());
        assert!(ad_dogrula("-a.test").is_err());
        assert!(ad_dogrula("a b.test").is_err());
        assert!(ad_dogrula(&"a".repeat(64)).is_err());
        let uzun = vec!["a".repeat(63); 4].join(".");
        assert_eq!(uzun.len(), 255);
        assert!(ad_dogrula(&uzun).is_err());
        let sinirda = format!("{}.{}", vec!["a".repeat(63); 3].join("."), "b".repeat(61));
        assert_eq!(sinirda.len(), 253);
        assert!(ad_dogrula(&sinirda).is_ok());
        let asan = format!("{}a", sinirda);
        assert!(ad_dogrula(&asan).is_err());
    }

    #[test]
    fn sorgu_paketi_rd_ad_ve_edns_tasir() {
        let paket = sorgu_kur(0xBEEF, "ornek.test", KayitTuru::Mx).unwrap();
        assert_eq!(&paket[..4], &[0xBE, 0xEF, 0x01, 0x20]);
        assert_eq!(&paket[4..12], &[0, 1, 0, 0, 0, 0, 0, 1]);
        let soru = &paket[12..12 + 12];
        assert_eq!(soru, b"\x05ornek\x04test\x00");
        assert_eq!(&paket[24..28], &[0, 15, 0, 1]);
        assert_eq!(&paket[28..], &[0, 0, 41, 0x04, 0xD0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn mx_siralamasi_ve_sikistirma() {
        let mut p = baslik(7, 0x81A0, 2, 0, 0);
        soru(&mut p, "ornek.test", TUR_MX);
        let mut veri = 20u16.to_be_bytes().to_vec();
        veri.extend_from_slice(b"\x02mx\xc0\x0c");
        kayit(&mut p, &[0xC0, 0x0C], TUR_MX, 300, &veri);
        let mut veri = 10u16.to_be_bytes().to_vec();
        veri.extend_from_slice(b"\x02mz\xc0\x0c");
        kayit(&mut p, &[0xC0, 0x0C], TUR_MX, 200, &veri);
        let yanit = yanit_ayristir(&p).unwrap();
        assert!(yanit.ad_biti);
        assert!(eslesiyor(&yanit, 7, "ornek.test", KayitTuru::Mx));
        assert!(!eslesiyor(&yanit, 8, "ornek.test", KayitTuru::Mx));
        assert!(!eslesiyor(&yanit, 7, "ornek.test", KayitTuru::Txt));
        let cozum = coz(&yanit, "ornek.test", KayitTuru::Mx).unwrap();
        assert_eq!(cozum.durum, Durum::Var);
        assert!(cozum.dogrulanmis);
        let hedefler: Vec<_> = cozum
            .kayitlar
            .iter()
            .map(|k| match &k.veri {
                KayitVerisi::Mx { oncelik, hedef } => (*oncelik, hedef.as_str()),
                _ => panic!(),
            })
            .collect();
        assert_eq!(hedefler, [(10, "mz.ornek.test"), (20, "mx.ornek.test")]);
    }

    #[test]
    fn txt_parcalari_birlesir() {
        let mut p = baslik(1, 0x8180, 1, 0, 0);
        soru(&mut p, "ornek.test", TUR_TXT);
        kayit(&mut p, &[0xC0, 0x0C], TUR_TXT, 60, b"\x05v=spf\x07 1 -all");
        let cozum = coz(&yanit_ayristir(&p).unwrap(), "ornek.test", KayitTuru::Txt).unwrap();
        assert_eq!(
            cozum.kayitlar[0].veri,
            KayitVerisi::Txt(b"v=spf 1 -all".to_vec())
        );
        assert!(!cozum.dogrulanmis);
    }

    #[test]
    fn nxdomain_nodata_ve_gecici_hatalar_ayrilir() {
        for (kod, beklenen) in [(0u16, Some(Durum::KayitYok)), (3, Some(Durum::AlanYok))] {
            let mut p = baslik(1, 0x8180 | kod, 0, 0, 0);
            soru(&mut p, "yok.test", TUR_A);
            let sonuc = coz(&yanit_ayristir(&p).unwrap(), "yok.test", KayitTuru::A);
            assert_eq!(sonuc.map(|c| c.durum).ok(), beklenen);
        }
        for (kod, tur) in [
            (2u16, DnsHataTuru::SunucuHatasi),
            (5, DnsHataTuru::Ret),
            (1, DnsHataTuru::SunucuHatasi),
            (4, DnsHataTuru::SunucuHatasi),
        ] {
            let mut p = baslik(1, 0x8180 | kod, 0, 0, 0);
            soru(&mut p, "yok.test", TUR_A);
            assert_eq!(
                coz(&yanit_ayristir(&p).unwrap(), "yok.test", KayitTuru::A)
                    .unwrap_err()
                    .tur,
                tur
            );
        }
    }

    /// RFC-0044 B1 kabul 3: SERVFAIL hiçbir AD/TC/kayıt birleşiminde `AlanYok`a dönüşmez.
    #[test]
    fn servfail_hicbir_yolda_alan_yok_olmaz() {
        for bayrak in [0x8182u16, 0x81A2, 0x8382, 0x8582, 0x85A2] {
            for cevap in [0u16, 1] {
                let mut p = baslik(9, bayrak, cevap, 0, 0);
                soru(&mut p, "ornek.test", TUR_A);
                if cevap == 1 {
                    kayit(&mut p, &[0xC0, 0x0C], TUR_A, 5, &[192, 0, 2, 1]);
                }
                let yanit = yanit_ayristir(&p).unwrap();
                assert!(coz(&yanit, "ornek.test", KayitTuru::A).is_err());
            }
        }
    }

    #[test]
    fn cname_zinciri_izlenir_ve_sinirlanir() {
        let mut p = baslik(1, 0x8180, 2, 0, 0);
        soru(&mut p, "www.ornek.test", TUR_A);
        kayit(
            &mut p,
            &[0xC0, 0x0C],
            TUR_CNAME,
            30,
            b"\x03web\x05ornek\x04test\x00",
        );
        let mut ad = Vec::new();
        ad_yaz(&mut ad, "web.ornek.test");
        kayit(&mut p, &ad, TUR_A, 30, &[192, 0, 2, 7]);
        let cozum = coz(&yanit_ayristir(&p).unwrap(), "www.ornek.test", KayitTuru::A).unwrap();
        assert_eq!(cozum.kanonik_ad, "web.ornek.test");
        assert_eq!(cozum.kayitlar[0].veri, KayitVerisi::A([192, 0, 2, 7]));

        // 9 CNAME: c0 → c1 → … → c9.
        let mut p = baslik(1, 0x8180, 9, 0, 0);
        soru(&mut p, "c0.test", TUR_A);
        for i in 0..9 {
            let mut sahip = Vec::new();
            ad_yaz(&mut sahip, &format!("c{i}.test"));
            let mut hedef = Vec::new();
            ad_yaz(&mut hedef, &format!("c{}.test", i + 1));
            kayit(&mut p, &sahip, TUR_CNAME, 1, &hedef);
        }
        let hata = coz(&yanit_ayristir(&p).unwrap(), "c0.test", KayitTuru::A).unwrap_err();
        assert_eq!(hata.tur, DnsHataTuru::BoyutAsimi);

        // Döngü: a → b → a.
        let mut p = baslik(1, 0x8180, 2, 0, 0);
        soru(&mut p, "a.test", TUR_A);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        ad_yaz(&mut a, "a.test");
        ad_yaz(&mut b, "b.test");
        kayit(&mut p, &a, TUR_CNAME, 1, &b);
        kayit(&mut p, &b, TUR_CNAME, 1, &a);
        let hata = coz(&yanit_ayristir(&p).unwrap(), "a.test", KayitTuru::A).unwrap_err();
        assert_eq!(hata.tur, DnsHataTuru::BoyutAsimi);
    }

    #[test]
    fn bozuk_ve_sinir_disi_paketler_reddedilir() {
        // 65 kayıt.
        let mut p = baslik(1, 0x8180, 65, 0, 0);
        soru(&mut p, "ornek.test", TUR_A);
        assert_eq!(
            yanit_ayristir(&p).unwrap_err(),
            TelHatasi::BoyutAsimi("cevap bölümü 64 kaydı aşıyor")
        );
        // İleri işaretçi ve kendine işaretçi (döngü).
        for isaretci in [[0xC0u8, 0x40], [0xC0, 0x0C]] {
            let mut p = baslik(1, 0x8180, 0, 0, 0);
            p.extend_from_slice(&isaretci);
            p.extend_from_slice(&[0, 1, 0, 1]);
            assert!(yanit_ayristir(&p).is_err());
        }
        // Sorgu (QR=0), fazla bayt, rdata uyuşmazlığı, bilinmeyen tür.
        let mut p = baslik(1, 0x0100, 0, 0, 0);
        soru(&mut p, "a.test", TUR_A);
        assert!(yanit_ayristir(&p).is_err());
        let mut p = baslik(1, 0x8180, 0, 0, 0);
        soru(&mut p, "a.test", TUR_A);
        p.push(0);
        assert!(yanit_ayristir(&p).is_err());
        let mut p = baslik(1, 0x8180, 1, 0, 0);
        soru(&mut p, "a.test", TUR_A);
        kayit(&mut p, &[0xC0, 0x0C], TUR_A, 1, &[1, 2, 3]);
        assert!(yanit_ayristir(&p).is_err());
        let mut p = baslik(1, 0x8180, 1, 0, 0);
        soru(&mut p, "a.test", TUR_A);
        kayit(&mut p, &[0xC0, 0x0C], 99, 1, &[1]);
        assert!(yanit_ayristir(&p).is_err());
        // TXT parçası rdata'yı aşıyor.
        let mut p = baslik(1, 0x8180, 1, 0, 0);
        soru(&mut p, "a.test", TUR_TXT);
        kayit(&mut p, &[0xC0, 0x0C], TUR_TXT, 1, b"\x05ab");
        assert!(yanit_ayristir(&p).is_err());
    }

    #[test]
    fn kesik_yanit_tam_yanit_sayilmaz() {
        let mut p = baslik(1, 0x8380, 0, 0, 0);
        soru(&mut p, "a.test", TUR_TXT);
        let yanit = yanit_ayristir(&p).unwrap();
        assert!(yanit.kesik);
        assert_eq!(
            coz(&yanit, "a.test", KayitTuru::Txt).unwrap_err().tur,
            DnsHataTuru::BozukYanit
        );
    }

    #[test]
    fn onaltilik_gidis_donus() {
        assert_eq!(onaltilik(&[0, 0xAB, 0x10]), "00ab10");
        assert_eq!(onaltilik_coz("00ab10").unwrap(), [0, 0xAB, 0x10]);
        assert!(onaltilik_coz("0").is_none());
        assert!(onaltilik_coz("AB").is_none());
    }
}
