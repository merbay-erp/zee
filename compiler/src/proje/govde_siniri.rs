//! K-391 (ADR-168): web isteği ve SQL parametresi boyut ayarları. Hepsi
//! isteğe bağlıdır; bildirimsiz projede spec/11 §5 ve spec/25 varsayılanları
//! bayt bayt aynıdır.
//!
//! - `gövde_aşımı_rotası "/yol" olsun`: native adaptör gövde boyutu yüzünden
//!   reddettiği güvenli olmayan isteği ham 413 yerine bu GET rotasıyla
//!   yanıtlar (durum yine 413).
//! - `geniş_gövde_rotaları "/a", "/b/" listesi olsun` + `geniş_gövde_sınırı N
//!   olsun`: eşleşen yolun POST/PUT/PATCH/DELETE form bütçesi N bayttır
//!   (`/` ile biten öge önek, diğeri tam yol).
//! - `veritabanı_parametre_sınırı N olsun`: tek SQL parametresi tavanı.
use super::*;

/// K-391 alan adları.
pub(super) const ALANLAR: [&str; 4] = [
    "gövde_aşımı_rotası",
    "geniş_gövde_rotaları",
    "geniş_gövde_sınırı",
    "veritabanı_parametre_sınırı",
];

/// Rota yolu bayt tavanı.
const AZAMI_YOL: usize = 256;
/// Geniş gövdeli rota ögesi tavanı.
const AZAMI_GENIS_ROTA: usize = 32;
/// Geniş gövde sınırının alt (varsayılan + 1) ve üst tavanı (bayt).
pub const GENIS_GOVDE_ALT: usize = VARSAYILAN_FORM_BUTCESI + 1;
pub const GENIS_GOVDE_UST: usize = 1024 * 1024;
/// SQL tek parametre tavanı aralığı (bayt).
pub const SQL_PARAMETRE_ALT: usize = 64 * 1024;
pub const SQL_PARAMETRE_UST: usize = 4 * 1024 * 1024;

const VARSAYILAN_FORM_BUTCESI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .http()
    .istek_govde_bayti();

const ORNEK: &str = "Örnek: gövde_aşımı_rotası \"/ileti/cok-buyuk\" olsun · geniş_gövde_rotaları \"/posta/gonder\" listesi olsun · geniş_gövde_sınırı 262144 olsun · veritabanı_parametre_sınırı 1048576 olsun";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GovdeSiniriAyari {
    /// Gövde aşımında çalıştırılan GET rotasının yolu.
    pub asim_rotasi: Option<String>,
    /// Geniş form bütçeli yollar (`/` ile biten öge önek).
    pub genis_rotalar: Vec<String>,
    /// Geniş form bütçesi (bayt); rota listesi boşsa kullanılmaz.
    pub genis_sinir: usize,
    /// Tek SQL parametresi tavanı (bayt); `None` spec/25 varsayılanı.
    pub sql_parametre_bayti: Option<usize>,
}

impl GovdeSiniriAyari {
    /// İsteğin urlencoded form ve multipart metin bütçesi. Yalnız güvenli
    /// olmayan yöntemde ve sorgusuz yol geniş listeyle eşleşirse geniştir.
    pub fn form_butcesi(&self, yontem: &str, yol: &str) -> usize {
        let unsafe_yontem = ["POST", "PUT", "PATCH", "DELETE"]
            .iter()
            .any(|y| yontem.eq_ignore_ascii_case(y));
        let eslesti = self.genis_rotalar.iter().any(|oge| {
            if oge.ends_with('/') {
                yol.starts_with(oge.as_str())
            } else {
                yol == oge
            }
        });
        if unsafe_yontem && eslesti {
            self.genis_sinir
        } else {
            VARSAYILAN_FORM_BUTCESI
        }
    }
}

fn hata(mesaj: &str, satir: usize) -> Tani {
    proje_hatasi("P022", mesaj, satir, ORNEK)
}

/// Rota yolu `/` ile başlar; sorgu, parça, yüzde kaçışı, boşluk ve denetim
/// karakteri taşımaz (yorumlayıcıya istek satırı olarak doğrudan girer).
pub(super) fn yol_gecerli(yol: &str) -> bool {
    yol.starts_with('/')
        && yol.len() <= AZAMI_YOL
        && !yol
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '?' | '#' | '%'))
}

fn sayi_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    ad: &str,
    alt: usize,
    ust: usize,
) -> Result<Option<(usize, usize)>, Tani> {
    let Some((ifade, satir)) = alanlar.remove(ad) else {
        return Ok(None);
    };
    match ifade.turu() {
        Ifade::SayiSabiti(deger) if (alt as i64..=ust as i64).contains(deger) => {
            Ok(Some((*deger as usize, satir)))
        }
        _ => Err(hata(
            &format!("\"{ad}\" {alt} ile {ust} arasında sabit Sayı olmalı."),
            satir,
        )),
    }
}

fn genis_rotalari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
) -> Result<(Vec<String>, usize), Tani> {
    let rotalar = if alanlar.contains_key("geniş_gövde_rotaları") {
        let (rotalar, satir) = metin_listesini_al(alanlar, "geniş_gövde_rotaları", "P022")?;
        if rotalar.is_empty() || rotalar.len() > AZAMI_GENIS_ROTA {
            return Err(hata(
                "\"geniş_gövde_rotaları\" 1 ile 32 arasında yol taşımalı.",
                satir,
            ));
        }
        for (sira, yol) in rotalar.iter().enumerate() {
            if !yol_gecerli(yol) || rotalar[..sira].contains(yol) {
                return Err(hata(
                    &format!("\"geniş_gövde_rotaları\" ögesi \"{}\" geçersiz ya da yinelenmiş; `/` ile başlayan sorgusuz yol yaz.", metni_kacir(yol)),
                    satir,
                ));
            }
        }
        Some((rotalar, satir))
    } else {
        None
    };
    let sinir = sayi_al(
        alanlar,
        "geniş_gövde_sınırı",
        GENIS_GOVDE_ALT,
        GENIS_GOVDE_UST,
    )?;
    match (rotalar, sinir) {
        (None, None) => Ok((Vec::new(), 0)),
        (Some((rotalar, _)), Some((sinir, _))) => Ok((rotalar, sinir)),
        (Some((_, satir)), None) | (None, Some((_, satir))) => Err(hata(
            "\"geniş_gövde_rotaları\" ve \"geniş_gövde_sınırı\" birlikte yazılmalı.",
            satir,
        )),
    }
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    veritabani_var: bool,
) -> Result<GovdeSiniriAyari, Tani> {
    let asim_rotasi = match alanlar.remove("gövde_aşımı_rotası") {
        None => None,
        Some((ifade, satir)) => match ifade.turu() {
            Ifade::MetinSabiti(yol) if yol_gecerli(yol) => Some(yol.clone()),
            _ => {
                return Err(hata(
                    "\"gövde_aşımı_rotası\" `/` ile başlayan, en çok 256 baytlık, sorgu/parça/boşluk içermeyen bir yol Metni olmalı.",
                    satir,
                ))
            }
        },
    };
    let (genis_rotalar, genis_sinir) = genis_rotalari_al(alanlar)?;
    let sql = sayi_al(
        alanlar,
        "veritabanı_parametre_sınırı",
        SQL_PARAMETRE_ALT,
        SQL_PARAMETRE_UST,
    )?;
    if let (Some((_, satir)), false) = (sql, veritabani_var) {
        return Err(hata(
            "\"veritabanı_parametre_sınırı\" var ama `veritabanı_hedefi` bildirimi yok.",
            satir,
        ));
    }
    Ok(GovdeSiniriAyari {
        asim_rotasi,
        genis_rotalar,
        genis_sinir,
        sql_parametre_bayti: sql.map(|(deger, _)| deger),
    })
}
