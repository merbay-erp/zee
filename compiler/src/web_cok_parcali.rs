//! K-378 (RFC-0041 katman B, ADR-163): sınırlı, tek geçişli
//! `multipart/form-data` ayrıştırıcısı.
//!
//! Gövde belleğe alınmaz: ayrıştırıcı parça parça beslenir, sınır (boundary)
//! araması için en çok `ayraç uzunluğu - 1` bayt tutar ve olayları
//! [`Alici`]'ya iletir. Dosya parçasının baytları `dosya_verisi` ile akar;
//! çağıran onları spec/26 `.parca` yükleyicisine yazar. K-391 (ADR-168):
//! metin alanları sıradan form alanıdır; ham baytlarının toplamı isteğin
//! form bütçesine tabidir ve dosya parçası isteğe bağlıdır. Her ihlal sınıflandırılmış [`Red`] ile kısmi kabul
//! olmadan durur (fail-closed); CSRF ve oturum kararı çağırandadır
//! (`dosya_basladi` dönüşü).

/// Toplam parça (alan + dosya) tavanı.
pub const AZAMI_PARCA: usize = 8;
/// Tek parçanın başlık bloğu tavanı (bayt).
pub const AZAMI_BASLIK_BLOGU: usize = 8 * 1024;
/// Tek parçanın başlık satırı tavanı.
pub const AZAMI_BASLIK_SATIRI: usize = 8;
/// Alan adı tavanı (bayt).
pub const AZAMI_ALAN_ADI: usize = 128;
/// İlk sınırdan önceki önsöz ve kapanıştan sonraki sonsöz tavanı (bayt).
pub const AZAMI_ONSOZ: usize = 1024;
/// RFC 2046 §5.1.1 sınır uzunluğu.
pub const AZAMI_SINIR: usize = 70;
/// Bildirilen içerik türü bu uzunluktan uzunsa ya da ASCII değilse boş verilir.
const AZAMI_BILDIRILEN_TUR: usize = 127;

/// Sınıflandırılmış ret: HTTP durumu ve sır taşımayan ileti.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Red {
    pub durum: u16,
    pub mesaj: String,
}

impl Red {
    pub fn kotu(mesaj: impl Into<String>) -> Self {
        Self {
            durum: 400,
            mesaj: mesaj.into(),
        }
    }
}

/// Dosya parçasının bilgi amaçlı (güvenilmez) üst verisi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DosyaBilgisi {
    /// Form alanının adı.
    pub alan: String,
    /// Doğrulanmış basename ya da boş (geçersiz ad reddedilmez, boşalır).
    pub dosya_adi: String,
    /// Parçanın beyan ettiği `Content-Type` (ASCII, ≤ 127 bayt) ya da boş.
    pub bildirilen_tur: String,
}

/// Ayrıştırma olaylarının alıcısı.
pub trait Alici {
    /// Tamamlanmış, UTF-8 strict metin alanı.
    fn alan(&mut self, ad: &str, deger: &str) -> Result<(), Red>;
    /// Dosya parçasının başlıkları bitti; gövde baytları henüz gelmedi.
    /// `Err` dosya baytı yazılmadan isteği durdurur (CSRF ön kapısı).
    fn dosya_basladi(&mut self, bilgi: DosyaBilgisi) -> Result<(), Red>;
    fn dosya_verisi(&mut self, veri: &[u8]) -> Result<(), Red>;
    fn dosya_bitti(&mut self) -> Result<(), Red>;
}

/// `Content-Type` değerinden sınırı çıkarır. Tür `multipart/form-data`
/// değilse `Ok(None)` (çağıran bugünkü yolda kalır); türdür ama parametre
/// kuralı bozuksa `Err` (400).
pub fn siniri_ayristir(icerik_turu: &str) -> Result<Option<String>, Red> {
    let (tur, parametreler) = icerik_turu.split_once(';').unwrap_or((icerik_turu, ""));
    if !tur.trim().eq_ignore_ascii_case("multipart/form-data") {
        return Ok(None);
    }
    let mut sinir = None;
    let mut kalan = parametreler;
    while !kalan.trim().is_empty() {
        let (ad, deger, sonraki) = parametre_oku(kalan)
            .ok_or_else(|| Red::kotu("multipart Content-Type parametreleri bozuk"))?;
        kalan = sonraki;
        if !ad.eq_ignore_ascii_case("boundary") {
            return Err(Red::kotu(
                "multipart Content-Type yalnız boundary parametresi taşır",
            ));
        }
        if sinir.replace(deger).is_some() {
            return Err(Red::kotu("multipart boundary birden çok kez verildi"));
        }
    }
    let sinir = sinir.ok_or_else(|| Red::kotu("multipart boundary eksik"))?;
    let izinli = |c: char| c.is_ascii_alphanumeric() || "'()+_,-./:=? ".contains(c);
    if sinir.is_empty()
        || sinir.len() > AZAMI_SINIR
        || sinir.ends_with(' ')
        || !sinir.chars().all(izinli)
    {
        return Err(Red::kotu("multipart boundary RFC 2046 kuralına uymuyor"));
    }
    Ok(Some(sinir))
}

/// `; ad=değer` (değer token ya da kaçışsız tırnaklı) okur; kalan metni verir.
fn parametre_oku(metin: &str) -> Option<(String, String, &str)> {
    let metin = metin.trim_start_matches([' ', '\t']);
    let metin = metin.strip_prefix(';').unwrap_or(metin);
    let metin = metin.trim_start_matches([' ', '\t']);
    let (ad, kalan) = metin.split_once('=')?;
    let ad = ad.trim_end_matches([' ', '\t']);
    if ad.is_empty()
        || !ad
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_*".contains(&b))
    {
        return None;
    }
    let kalan = kalan.trim_start_matches([' ', '\t']);
    let (deger, sonra) = if let Some(tirnakli) = kalan.strip_prefix('"') {
        let son = tirnakli.find('"')?;
        (&tirnakli[..son], &tirnakli[son + 1..])
    } else {
        let son = kalan.find(';').unwrap_or(kalan.len());
        let deger = kalan[..son].trim_end_matches([' ', '\t']);
        if deger.is_empty() || deger.contains([' ', '\t', '"']) {
            return None;
        }
        (deger, &kalan[son..])
    };
    let sonra = sonra.trim_start_matches([' ', '\t']);
    if !(sonra.is_empty() || sonra.starts_with(';')) {
        return None;
    }
    Some((ad.to_string(), deger.to_string(), sonra))
}

enum Durum {
    Onsoz,
    SinirSonrasi,
    Basliklar,
    Alan { ad: String, deger: Vec<u8> },
    Dosya,
    Sonsoz,
}

pub struct Ayristirici {
    ayrac: Vec<u8>,
    tampon: Vec<u8>,
    durum: Durum,
    parca: usize,
    dosya_goruldu: bool,
    adlar: std::collections::BTreeSet<String>,
    onsoz: usize,
    sonsoz: usize,
    /// K-391: metin alanlarının ham bayt toplamı ve tavanı (form bütçesi).
    metin: usize,
    metin_butcesi: usize,
}

impl Ayristirici {
    /// `metin_butcesi`, bütün metin alanı değerlerinin toplam bayt tavanıdır
    /// (isteğin urlencoded form bütçesi; K-391).
    pub fn yeni(sinir: &str, metin_butcesi: usize) -> Self {
        let mut ayrac = b"\r\n--".to_vec();
        ayrac.extend_from_slice(sinir.as_bytes());
        Self {
            ayrac,
            // İlk sınır CRLF'siz gelir; sanal CRLF tek arama kuralı sağlar.
            tampon: b"\r\n".to_vec(),
            durum: Durum::Onsoz,
            parca: 0,
            dosya_goruldu: false,
            adlar: Default::default(),
            onsoz: 0,
            sonsoz: 0,
            metin: 0,
            metin_butcesi,
        }
    }

    /// Sıradaki gövde parçasını işler.
    pub fn besle(&mut self, veri: &[u8], alici: &mut dyn Alici) -> Result<(), Red> {
        self.tampon.extend_from_slice(veri);
        self.isle(alici)
    }

    /// Gövde bitti: kapanış sınırı ZORUNLUDUR; dosya parçası en çok birdir
    /// ve K-391'den beri isteğe bağlıdır (yalnız metin alanlı form).
    pub fn bitir(mut self, alici: &mut dyn Alici) -> Result<(), Red> {
        self.isle(alici)?;
        if !matches!(self.durum, Durum::Sonsoz) {
            return Err(Red::kotu("multipart gövdesi kapanış sınırı olmadan bitti"));
        }
        Ok(())
    }

    fn ayraci_bul(&self) -> Option<usize> {
        ayraci_bul(&self.tampon, &self.ayrac)
    }

    fn guvenli_onek(&self) -> usize {
        guvenli_onek(&self.tampon, &self.ayrac)
    }

    /// Durum makinesini veri bitene dek ilerletir. Her adım `true` (ilerledi)
    /// ya da `false` (daha fazla bayt gerekli) döndürür.
    fn isle(&mut self, alici: &mut dyn Alici) -> Result<(), Red> {
        loop {
            let ilerledi = match self.durum {
                Durum::Onsoz => self.onsoz_adimi()?,
                Durum::SinirSonrasi => self.sinir_sonrasi_adimi()?,
                Durum::Basliklar => self.basliklar_adimi(alici)?,
                Durum::Alan { .. } => self.alan_adimi(alici)?,
                Durum::Dosya => self.dosya_adimi(alici)?,
                Durum::Sonsoz => {
                    self.sonsoz += self.tampon.len();
                    self.tampon.clear();
                    if self.sonsoz > AZAMI_ONSOZ {
                        return Err(Red::kotu("multipart sonsözü 1 KiB sınırını aşıyor"));
                    }
                    false
                }
            };
            if !ilerledi {
                return Ok(());
            }
        }
    }

    fn onsoz_adimi(&mut self) -> Result<bool, Red> {
        let bulunan = self.ayraci_bul();
        let tuketilen = bulunan.unwrap_or_else(|| self.guvenli_onek());
        self.onsoz += tuketilen;
        if self.onsoz > AZAMI_ONSOZ + 2 {
            return Err(Red::kotu("multipart önsözü 1 KiB sınırını aşıyor"));
        }
        match bulunan {
            Some(konum) => {
                self.tampon.drain(..konum + self.ayrac.len());
                self.durum = Durum::SinirSonrasi;
                Ok(true)
            }
            None => {
                self.tampon.drain(..tuketilen);
                Ok(false)
            }
        }
    }

    fn sinir_sonrasi_adimi(&mut self) -> Result<bool, Red> {
        if self.tampon.len() < 2 {
            return Ok(false);
        }
        self.durum = match &self.tampon[..2] {
            b"\r\n" => {
                self.parca += 1;
                if self.parca > AZAMI_PARCA {
                    return Err(Red::kotu("multipart parça sayısı 8 sınırını aşıyor"));
                }
                Durum::Basliklar
            }
            b"--" => Durum::Sonsoz,
            _ => {
                return Err(Red::kotu(
                    "multipart sınırından sonra CRLF ya da -- gelmedi",
                ))
            }
        };
        self.tampon.drain(..2);
        Ok(true)
    }

    fn basliklar_adimi(&mut self, alici: &mut dyn Alici) -> Result<bool, Red> {
        let Some(son) = self.tampon.windows(4).position(|p| p == b"\r\n\r\n") else {
            if self.tampon.len() > AZAMI_BASLIK_BLOGU + 4 {
                return Err(Red::kotu("multipart parça başlığı 8 KiB sınırını aşıyor"));
            }
            if self.tampon.starts_with(b"\r\n") {
                return Err(Red::kotu("multipart parçasında Content-Disposition yok"));
            }
            return Ok(false);
        };
        if son > AZAMI_BASLIK_BLOGU {
            return Err(Red::kotu("multipart parça başlığı 8 KiB sınırını aşıyor"));
        }
        let blok = std::str::from_utf8(&self.tampon[..son])
            .map_err(|_| Red::kotu("multipart parça başlığı geçerli UTF-8 değil"))?
            .to_string();
        self.tampon.drain(..son + 4);
        let parca = basliklari_ayristir(&blok)?;
        if parca.ad.starts_with("yukleme_") {
            return Err(Red::kotu("multipart alan adı `yukleme_` ile başlayamaz"));
        }
        if !self.adlar.insert(parca.ad.clone()) {
            return Err(Red::kotu("multipart alan adı yinelendi"));
        }
        let Some((dosya_adi, bildirilen_tur)) = parca.dosya else {
            self.durum = Durum::Alan {
                ad: parca.ad,
                deger: Vec::new(),
            };
            return Ok(true);
        };
        if std::mem::replace(&mut self.dosya_goruldu, true) {
            return Err(Red::kotu("multipart gövdesinde birden çok dosya var"));
        }
        alici.dosya_basladi(DosyaBilgisi {
            alan: parca.ad,
            dosya_adi,
            bildirilen_tur,
        })?;
        self.durum = Durum::Dosya;
        Ok(true)
    }

    fn alan_adimi(&mut self, alici: &mut dyn Alici) -> Result<bool, Red> {
        let bulunan = self.ayraci_bul();
        let tuketilen = bulunan.unwrap_or_else(|| self.guvenli_onek());
        let Durum::Alan { ad, deger } = &mut self.durum else {
            return Err(Red::kotu("multipart iç durum hatası"));
        };
        deger.extend_from_slice(&self.tampon[..tuketilen]);
        self.metin += tuketilen;
        if self.metin > self.metin_butcesi {
            return Err(metin_asimi(self.metin_butcesi));
        }
        let Some(konum) = bulunan else {
            self.tampon.drain(..tuketilen);
            return Ok(false);
        };
        let metin = String::from_utf8(std::mem::take(deger))
            .map_err(|_| Red::kotu("multipart metin alanı geçerli UTF-8 değil"))?;
        let ad = std::mem::take(ad);
        self.tampon.drain(..konum + self.ayrac.len());
        self.durum = Durum::SinirSonrasi;
        alici.alan(&ad, &metin)?;
        Ok(true)
    }

    fn dosya_adimi(&mut self, alici: &mut dyn Alici) -> Result<bool, Red> {
        match self.ayraci_bul() {
            Some(konum) => {
                alici.dosya_verisi(&self.tampon[..konum])?;
                self.tampon.drain(..konum + self.ayrac.len());
                self.durum = Durum::SinirSonrasi;
                alici.dosya_bitti()?;
                Ok(true)
            }
            None => {
                let onek = self.guvenli_onek();
                if onek > 0 {
                    alici.dosya_verisi(&self.tampon[..onek])?;
                    self.tampon.drain(..onek);
                }
                Ok(false)
            }
        }
    }
}

/// K-391: metin alanları form bütçesini aştı (gövde boyutu → 413).
pub fn metin_asimi(butce: usize) -> Red {
    Red {
        durum: 413,
        mesaj: format!(
            "multipart metin alanları {} KiB sınırını aşıyor",
            butce / 1024
        ),
    }
}

/// Ayraçta CR yalnız ilk konumdadır (sınır karakterleri CR içermez);
/// başarısız kısmi eşleşme bir sonraki CR'den önce biter, arama doğrusal
/// kalır.
fn ayraci_bul(tampon: &[u8], ayrac: &[u8]) -> Option<usize> {
    let ilk = ayrac[0];
    let mut i = 0;
    while let Some(konum) = tampon[i..].iter().position(|&b| b == ilk) {
        let bas = i + konum;
        let son = bas + ayrac.len();
        if son > tampon.len() {
            return None;
        }
        if tampon[bas..son] == ayrac[..] {
            return Some(bas);
        }
        i = bas + 1;
    }
    None
}

/// Ayraç bulunamadığında güvenle tüketilebilecek önek uzunluğu.
fn guvenli_onek(tampon: &[u8], ayrac: &[u8]) -> usize {
    tampon.len().saturating_sub(ayrac.len().saturating_sub(1))
}

struct Parca {
    ad: String,
    /// Dosya parçasıysa (doğrulanmış ad ya da boş, bildirilen tür ya da boş).
    dosya: Option<(String, String)>,
}

fn basliklari_ayristir(blok: &str) -> Result<Parca, Red> {
    let satirlar: Vec<&str> = blok.split("\r\n").collect();
    if satirlar.len() > AZAMI_BASLIK_SATIRI {
        return Err(Red::kotu(
            "multipart parçası 8'den fazla başlık satırı taşıyor",
        ));
    }
    let mut disposition = None;
    let mut tur = None;
    for satir in satirlar {
        if satir.starts_with([' ', '\t']) {
            return Err(Red::kotu("multipart başlık katlaması reddedildi"));
        }
        if satir.chars().any(|c| c.is_control() && c != '\t') {
            return Err(Red::kotu("multipart başlığı denetim karakteri taşıyor"));
        }
        let (ad, deger) = satir
            .split_once(':')
            .ok_or_else(|| Red::kotu("multipart başlık satırı `ad: değer` değil"))?;
        let deger = deger.trim_matches([' ', '\t']);
        if ad.eq_ignore_ascii_case("content-disposition") {
            if disposition.replace(deger).is_some() {
                return Err(Red::kotu("multipart Content-Disposition yinelendi"));
            }
        } else if ad.eq_ignore_ascii_case("content-type") {
            if tur.replace(deger).is_some() {
                return Err(Red::kotu("multipart parça Content-Type yinelendi"));
            }
        } else {
            // Content-Transfer-Encoding (RFC 7578 §4.7) dahil.
            return Err(Red::kotu(format!(
                "multipart parça başlığı tanınmıyor: {}",
                ad.chars().take(40).collect::<String>()
            )));
        }
    }
    let disposition =
        disposition.ok_or_else(|| Red::kotu("multipart parçasında Content-Disposition yok"))?;
    disposition_ayristir(disposition, tur)
}

/// `form-data; name=…[; filename=…|filename*=…]` değerini parçaya çevirir.
fn disposition_ayristir(disposition: &str, tur: Option<&str>) -> Result<Parca, Red> {
    let (bicim, mut kalan) = disposition.split_once(';').unwrap_or((disposition, ""));
    if !bicim.trim().eq_ignore_ascii_case("form-data") {
        return Err(Red::kotu("multipart parçası form-data değil"));
    }
    let mut ad = None;
    let mut dosya_adi = None;
    let mut dosya_adi_yildiz = None;
    while !kalan.trim().is_empty() {
        let (anahtar, deger, sonraki) = parametre_oku(kalan)
            .ok_or_else(|| Red::kotu("multipart Content-Disposition parametreleri bozuk"))?;
        kalan = sonraki;
        let yuva = match anahtar.to_ascii_lowercase().as_str() {
            "name" => &mut ad,
            "filename" => &mut dosya_adi,
            "filename*" => &mut dosya_adi_yildiz,
            _ => {
                return Err(Red::kotu(
                    "multipart Content-Disposition parametresi tanınmıyor",
                ))
            }
        };
        if yuva.replace(deger).is_some() {
            return Err(Red::kotu(
                "multipart Content-Disposition parametresi yinelendi",
            ));
        }
    }
    let ad = ad.ok_or_else(|| Red::kotu("multipart parçasında name yok"))?;
    if ad.is_empty() || ad.len() > AZAMI_ALAN_ADI {
        return Err(Red::kotu("multipart alan adı boş ya da 128 baytı aşıyor"));
    }
    let dosya = if dosya_adi.is_some() || dosya_adi_yildiz.is_some() {
        let ham = match dosya_adi_yildiz {
            Some(yildiz) => rfc8187_coz(&yildiz)?,
            None => dosya_adi.unwrap_or_default(),
        };
        // Tarayıcı tam yol gönderebilir; yalnız son bileşen bilgi olarak kalır.
        let taban = ham.rsplit(['/', '\\']).next().unwrap_or("").to_string();
        let taban = if dil::icerik_turu::dosya_adini_denetle(&taban).is_ok() {
            taban
        } else {
            String::new()
        };
        let bildirilen = tur
            .filter(|t| t.len() <= AZAMI_BILDIRILEN_TUR && t.is_ascii())
            .unwrap_or("")
            .to_string();
        Some((taban, bildirilen))
    } else {
        None
    };
    Ok(Parca { ad, dosya })
}

/// RFC 8187 `UTF-8''yüzde-kodlu` değeri çözer; başka karakter kümesi 400.
fn rfc8187_coz(deger: &str) -> Result<String, Red> {
    let mut parcalar = deger.splitn(3, '\'');
    let (Some(kume), Some(_dil), Some(kodlu)) = (parcalar.next(), parcalar.next(), parcalar.next())
    else {
        return Err(Red::kotu("multipart filename* RFC 8187 biçiminde değil"));
    };
    if !kume.eq_ignore_ascii_case("utf-8") {
        return Err(Red::kotu("multipart filename* yalnız UTF-8 olabilir"));
    }
    let mut baytlar = Vec::with_capacity(kodlu.len());
    let mut kalan = kodlu.bytes();
    while let Some(b) = kalan.next() {
        if b == b'%' {
            let hane = |h: Option<u8>| h.and_then(|h| char::from(h).to_digit(16));
            let (Some(y), Some(a)) = (hane(kalan.next()), hane(kalan.next())) else {
                return Err(Red::kotu("multipart filename* yüzde kaçışı bozuk"));
            };
            baytlar.push((y * 16 + a) as u8);
        } else {
            baytlar.push(b);
        }
    }
    String::from_utf8(baytlar).map_err(|_| Red::kotu("multipart filename* geçerli UTF-8 değil"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug)]
    struct Kayit {
        alanlar: Vec<(String, String)>,
        dosya: Option<DosyaBilgisi>,
        veri: Vec<u8>,
        bitti: bool,
        csrf_red: bool,
    }

    impl Alici for Kayit {
        fn alan(&mut self, ad: &str, deger: &str) -> Result<(), Red> {
            self.alanlar.push((ad.into(), deger.into()));
            Ok(())
        }
        fn dosya_basladi(&mut self, bilgi: DosyaBilgisi) -> Result<(), Red> {
            if self.csrf_red && !self.alanlar.iter().any(|(ad, _)| ad == "_csrf") {
                return Err(Red {
                    durum: 403,
                    mesaj: "CSRF belirteci eksik".into(),
                });
            }
            self.dosya = Some(bilgi);
            Ok(())
        }
        fn dosya_verisi(&mut self, veri: &[u8]) -> Result<(), Red> {
            self.veri.extend_from_slice(veri);
            Ok(())
        }
        fn dosya_bitti(&mut self) -> Result<(), Red> {
            self.bitti = true;
            Ok(())
        }
    }

    const S: &str = "----zeeSinir7MA4YWxk";

    fn govde(parcalar: &[(&str, &[u8])]) -> Vec<u8> {
        let mut g = Vec::new();
        for (baslik, icerik) in parcalar {
            g.extend_from_slice(format!("--{S}\r\n{baslik}\r\n\r\n").as_bytes());
            g.extend_from_slice(icerik);
            g.extend_from_slice(b"\r\n");
        }
        g.extend_from_slice(format!("--{S}--\r\n").as_bytes());
        g
    }

    const BUTCE: usize = 64 * 1024;

    fn calistir(g: &[u8], dilim: usize, csrf_red: bool) -> (Kayit, Result<(), Red>) {
        calistir_butceyle(g, dilim, csrf_red, BUTCE)
    }

    fn calistir_butceyle(
        g: &[u8],
        dilim: usize,
        csrf_red: bool,
        butce: usize,
    ) -> (Kayit, Result<(), Red>) {
        let mut a = Ayristirici::yeni(S, butce);
        let mut k = Kayit {
            csrf_red,
            ..Default::default()
        };
        for parca in g.chunks(dilim.max(1)) {
            if let Err(e) = a.besle(parca, &mut k) {
                return (k, Err(e));
            }
        }
        let sonuc = a.bitir(&mut k);
        (k, sonuc)
    }

    #[test]
    fn sinir_parametresi_strict() {
        assert_eq!(
            siniri_ayristir("multipart/form-data; boundary=abc").unwrap(),
            Some("abc".into())
        );
        assert_eq!(
            siniri_ayristir("Multipart/Form-Data;boundary=\"a b'c\"").unwrap(),
            Some("a b'c".into())
        );
        assert_eq!(siniri_ayristir("application/octet-stream").unwrap(), None);
        assert_eq!(siniri_ayristir("text/plain; boundary=x").unwrap(), None);
        for kotu in [
            "multipart/form-data",
            "multipart/form-data; boundary=",
            "multipart/form-data; boundary=a; boundary=b",
            "multipart/form-data; boundary=a; charset=utf-8",
            "multipart/form-data; boundary=\"abc \"",
            "multipart/form-data; boundary=ab\u{e7}",
            "multipart/form-data; boundary=a\r\nX: y",
            "multipart/form-data; boundary=\"abc",
        ] {
            assert!(siniri_ayristir(kotu).is_err(), "{kotu:?}");
        }
        let uzun = format!("multipart/form-data; boundary={}", "a".repeat(71));
        assert!(siniri_ayristir(&uzun).is_err());
        let sinirda = format!("multipart/form-data; boundary={}", "a".repeat(70));
        assert!(siniri_ayristir(&sinirda).unwrap().is_some());
    }

    #[test]
    fn alanlar_ve_tek_dosya_her_dilimlemede_ayni_ayrisir() {
        let ikili: Vec<u8> = (0..5000u32).map(|i| (i * 7 % 256) as u8).collect();
        // Dosya içinde ayraca benzeyen ama eksik diziler.
        let mut icerik = ikili.clone();
        icerik.extend_from_slice(format!("\r\n--{}", &S[..S.len() - 1]).as_bytes());
        icerik.extend_from_slice(b"\r\n-");
        let g = govde(&[
            ("Content-Disposition: form-data; name=\"_csrf\"", b"belirtec"),
            (
                "Content-Disposition: form-data; name=\"dosya\"; filename=\"C:\\\\Belgeler\\\\rapor.pdf\"\r\nContent-Type: application/pdf",
                &icerik,
            ),
            ("Content-Disposition: form-data; name=\"not\"", "çağrı".as_bytes()),
        ]);
        for dilim in [1, 2, 3, 7, 64, 1000, g.len()] {
            let (k, sonuc) = calistir(&g, dilim, true);
            sonuc.unwrap_or_else(|e| panic!("dilim {dilim}: {e:?}"));
            assert_eq!(
                k.alanlar,
                [
                    ("_csrf".to_string(), "belirtec".to_string()),
                    ("not".to_string(), "çağrı".to_string())
                ]
            );
            let bilgi = k.dosya.expect("dosya");
            assert_eq!(bilgi.alan, "dosya");
            assert_eq!(bilgi.dosya_adi, "rapor.pdf");
            assert_eq!(bilgi.bildirilen_tur, "application/pdf");
            assert_eq!(k.veri, icerik, "dilim {dilim}");
            assert!(k.bitti);
        }
    }

    #[test]
    fn filename_yildiz_ve_gecersiz_ad_bilgi_olarak_bosalir() {
        let g = govde(&[(
            "Content-Disposition: form-data; name=\"d\"; filename=\"a.txt\"; filename*=UTF-8''%C3%A7izelge%20%C4%9F.xlsx",
            b"x",
        )]);
        let (k, sonuc) = calistir(&g, 5, false);
        sonuc.unwrap();
        assert_eq!(k.dosya.unwrap().dosya_adi, "çizelge ğ.xlsx");
        for (ad, beklenen) in [
            ("filename=\"..\"", ""),
            ("filename=\"\"", ""),
            ("filename=\"../../etc/passwd\"", "passwd"),
            ("filename=\"a\u{202E}fdp.exe\"", ""),
            ("filename*=UTF-8''a%0D%0Ab.pdf", ""),
        ] {
            let baslik = format!("Content-Disposition: form-data; name=\"d\"; {ad}");
            let (k, sonuc) = calistir(&govde(&[(&baslik, b"x")]), 3, false);
            sonuc.unwrap_or_else(|e| panic!("{ad}: {e:?}"));
            assert_eq!(k.dosya.unwrap().dosya_adi, beklenen, "{ad}");
        }
    }

    #[test]
    fn csrf_dosyadan_once_gelmezse_dosya_bayti_alinmaz() {
        let g = govde(&[
            (
                "Content-Disposition: form-data; name=\"dosya\"; filename=\"a.bin\"",
                b"GIZLI",
            ),
            ("Content-Disposition: form-data; name=\"_csrf\"", b"b"),
        ]);
        let (k, sonuc) = calistir(&g, 4, true);
        assert_eq!(sonuc.unwrap_err().durum, 403);
        assert!(k.veri.is_empty(), "dosya baytı alıcıya ulaşmamalı");
    }

    #[test]
    fn bozuk_ve_saldirgan_govdeler_400() {
        let dosya = (
            "Content-Disposition: form-data; name=\"d\"; filename=\"a\"",
            &b"x"[..],
        );
        let mut vakalar: Vec<(String, Vec<u8>)> = vec![
            ("kapanışsız".into(), {
                let mut g = govde(&[dosya]);
                g.truncate(g.len() - 4);
                g
            }),
            ("iki dosya".into(), govde(&[dosya, ("Content-Disposition: form-data; name=\"e\"; filename=\"b\"", b"y")])),
            ("CTE".into(), govde(&[("Content-Disposition: form-data; name=\"d\"; filename=\"a\"\r\nContent-Transfer-Encoding: base64", b"eA==")])),
            ("bilinmeyen başlık".into(), govde(&[("Content-Disposition: form-data; name=\"d\"; filename=\"a\"\r\nX-Ek: 1", b"x")])),
            ("katlama".into(), govde(&[("Content-Disposition: form-data;\r\n name=\"d\"; filename=\"a\"", b"x")])),
            ("disposition yok".into(), govde(&[("Content-Type: text/plain", b"x")])),
            ("form-data değil".into(), govde(&[("Content-Disposition: attachment; name=\"d\"; filename=\"a\"", b"x")])),
            ("name yok".into(), govde(&[("Content-Disposition: form-data; filename=\"a\"", b"x")])),
            ("bilinmeyen parametre".into(), govde(&[("Content-Disposition: form-data; name=\"d\"; filename=\"a\"; size=3", b"x")])),
            ("yinelenen alan".into(), govde(&[("Content-Disposition: form-data; name=\"a\"", b"1"), ("Content-Disposition: form-data; name=\"a\"", b"2"), dosya])),
            ("ayrılmış ad".into(), govde(&[("Content-Disposition: form-data; name=\"yukleme_gecici_yolu\"", b"/etc/passwd"), dosya])),
            ("UTF-8 dışı alan".into(), govde(&[("Content-Disposition: form-data; name=\"a\"", b"\xff\xfe"), dosya])),
            ("filename* başka küme".into(), govde(&[("Content-Disposition: form-data; name=\"d\"; filename*=ISO-8859-9''a", b"x")])),
            ("sınır sonrası çöp".into(), {
                let mut g = format!("--{S}XX\r\n").into_bytes();
                g.extend_from_slice(&govde(&[dosya]));
                g
            }),
            ("dokuz parça".into(), {
                let mut p: Vec<(String, &[u8])> = (0..8)
                    .map(|i| (format!("Content-Disposition: form-data; name=\"a{i}\""), &b"1"[..]))
                    .collect();
                p.push((dosya.0.to_string(), dosya.1));
                let p: Vec<(&str, &[u8])> = p.iter().map(|(a, b)| (a.as_str(), *b)).collect();
                govde(&p)
            }),
            ("dev başlık".into(), govde(&[(&format!("Content-Disposition: form-data; name=\"d\"; filename=\"{}\"", "a".repeat(9000)), b"x")])),
            ("dev önsöz".into(), {
                let mut g = vec![b'p'; 2000];
                g.extend_from_slice(b"\r\n");
                g.extend_from_slice(&govde(&[dosya]));
                g
            }),
            ("dev sonsöz".into(), {
                let mut g = govde(&[dosya]);
                g.extend_from_slice(&vec![b'e'; 2000]);
                g
            }),
        ];
        // Başlık bloğu hiç bitmeyen gövde de tampon tavanında durur.
        let mut sonsuz =
            format!("--{S}\r\nContent-Disposition: form-data; name=\"d\"; filename=\"a\"")
                .into_bytes();
        sonsuz.extend(std::iter::repeat_n(b'x', 20_000));
        vakalar.push(("bitmeyen başlık".into(), sonsuz));
        for (ad, g) in vakalar {
            for dilim in [1, 13, 4096] {
                let (_, sonuc) = calistir(&g, dilim, false);
                let red = sonuc.expect_err(&ad);
                assert_eq!(red.durum, 400, "{ad}: {red:?}");
            }
        }
    }

    /// K-391: dosyasız form ve Türkçe-yoğun 25 000 harflik metin alanı
    /// sıradan form alanı olarak çözülür (urlencoded şişmesi yok).
    #[test]
    fn dosyasiz_form_ve_turkce_yogun_metin_alani_kabul() {
        let ileti: String = "ışığı çğüöşİ ".chars().cycle().take(25_000).collect();
        assert!(
            ileti.len() < BUTCE && ileti.len() > 30_000,
            "{}",
            ileti.len()
        );
        let g = govde(&[
            ("Content-Disposition: form-data; name=\"_csrf\"", b"b"),
            (
                "Content-Disposition: form-data; name=\"govde\"",
                ileti.as_bytes(),
            ),
        ]);
        for dilim in [1, 7, 4096, g.len()] {
            let (k, sonuc) = calistir(&g, dilim, true);
            sonuc.unwrap_or_else(|e| panic!("dilim {dilim}: {e:?}"));
            assert_eq!(k.alanlar[1], ("govde".to_string(), ileti.clone()));
            assert!(k.dosya.is_none() && k.veri.is_empty());
        }
    }

    /// K-391: metin alanlarının TOPLAMI bütçeyi aşarsa 413; tek alan sınırı
    /// yoktur, bütçe alanlar arasında paylaşılır.
    #[test]
    fn metin_butcesi_alanlar_toplaminda_413() {
        let yarim = vec![b'x'; BUTCE / 2];
        let tam = govde(&[
            ("Content-Disposition: form-data; name=\"a\"", &yarim),
            ("Content-Disposition: form-data; name=\"b\"", &yarim),
        ]);
        calistir(&tam, 13, false).1.expect("tam bütçe kabul");
        let mut fazla = yarim.clone();
        fazla.push(b'y');
        let asan = govde(&[
            ("Content-Disposition: form-data; name=\"a\"", &yarim),
            ("Content-Disposition: form-data; name=\"b\"", &fazla),
        ]);
        for dilim in [1, 13, 4096] {
            let red = calistir(&asan, dilim, false).1.expect_err("aşım");
            assert_eq!(red.durum, 413, "{red:?}");
        }
        // Rota bazlı geniş bütçe aynı gövdeyi kabul eder; dosya baytı sayılmaz.
        calistir_butceyle(&asan, 4096, false, 4 * BUTCE)
            .1
            .expect("geniş bütçe");
        let dosyali = govde(&[
            ("Content-Disposition: form-data; name=\"a\"", b"1"),
            (
                "Content-Disposition: form-data; name=\"d\"; filename=\"a\"",
                &vec![b'z'; 2 * BUTCE],
            ),
        ]);
        calistir(&dosyali, 4096, false)
            .1
            .expect("dosya metin bütçesine girmez");
    }

    #[test]
    fn ayrac_aramasi_dogrusal_kalir() {
        // CR yoğun ve ayraca çok benzeyen 4 MiB gövde makul sürede biter.
        let mut icerik = Vec::new();
        while icerik.len() < 4 * 1024 * 1024 {
            icerik.extend_from_slice(format!("\r\n--{}", &S[..S.len() - 1]).as_bytes());
            icerik.extend_from_slice(b"\r\r\r");
        }
        let g = govde(&[(
            "Content-Disposition: form-data; name=\"d\"; filename=\"a\"",
            &icerik,
        )]);
        let basla = std::time::Instant::now();
        let (k, sonuc) = calistir(&g, 16 * 1024, false);
        sonuc.unwrap();
        assert_eq!(k.veri.len(), icerik.len());
        assert!(
            basla.elapsed() < std::time::Duration::from_secs(20),
            "{:?}",
            basla.elapsed()
        );
    }
}
