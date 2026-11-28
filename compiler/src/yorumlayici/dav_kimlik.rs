//! RFC-0047 E/F (ADR-182 dilim C, K-364): `dav istemcisi gerekli` rotasının
//! rota öncesi kimlik kararı.
//!
//! Adaptör `Authorization: Basic` başlığını çözer ve yalnız kullanıcı adını,
//! uygulama parolası önekini ve bildirilen çözücünün adını verir; gizli kısım
//! adaptörde kalır ve hiçbir zaman Zee değeri ya da IO izi alanı olmaz.
//! Karar sırası (RFC-0047 §6):
//!
//! 1. kimlik yok ya da bozuk → 401 (`WWW-Authenticate`);
//! 2. başarısız kova (istemci + kullanıcı, istemci geneli) doluysa 429;
//! 3. biçim kapısından geçmeyen parola (hesap parolası dahil) çözücü çağrılmadan
//!    sahte karşılaştırmayla başarısız sayılır;
//! 4. çözücü `(katlanmış kullanıcı, önek)` ile çağrılır; önbellek yok;
//! 5. özet sabit zamanlı karşılaştırılır, kullanıcı adı ASCII katlanmış
//!    biçimle eşleşmelidir; başarısızlık kovayı artırır, 401;
//! 6. başarılı kova (uygulama parolası kimliği) doluysa 429; salt okuma
//!    kapsamında durum değiştiren yöntem 403;
//! 7. `oturum` istek yerel olarak dolar.

use super::*;
use crate::guvenlik::uygulama_parolasi::kullanici_katla;

/// Adaptörün çözdüğü istek kimliği. `onek` biçim kapısından geçen
/// uygulama parolasının arama önekidir; `None` parolanın biçimsiz olduğunu
/// söyler. `cozucu` projenin `dav_kimlik_çözücüsü` bildirimidir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DavKimligi {
    pub kullanici: String,
    pub onek: Option<String>,
    pub cozucu: Option<String>,
}

/// DAV oran kovası olayı (RFC-0047 §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DavKova {
    /// Başarısız kovaları artırmadan denetler (istemci + kullanıcı, istemci).
    Bak,
    /// Başarısız denemeyi iki kovaya da yazar.
    Basarisiz,
    /// Başarılı isteği uygulama parolası kimliği kovasına yazar.
    Basarili,
}

impl DavKova {
    pub fn yazimi(self) -> &'static str {
        match self {
            Self::Bak => "bak",
            Self::Basarisiz => "basarisiz",
            Self::Basarili => "basarili",
        }
    }

    pub fn ayristir(yazim: &str) -> Option<Self> {
        match yazim {
            "bak" => Some(Self::Bak),
            "basarisiz" => Some(Self::Basarisiz),
            "basarili" => Some(Self::Basarili),
            _ => None,
        }
    }
}

/// 401 gövdesi: kullanıcı varlığından bağımsız tek sabit metin.
pub const DAV_KIMLIK_REDDI: &str = "kimlik doğrulaması gerekli";
/// 429 gövdesi.
pub const DAV_KOVA_REDDI: &str = "çok fazla kimlik denemesi; daha sonra yeniden deneyin";

struct Kayit {
    ozet: String,
    kullanici: String,
    rol: String,
    kapsam: String,
    kimlik: String,
}

fn metin_alani(girdiler: &[(String, Deger)], ad: &str) -> Option<String> {
    girdiler.iter().find_map(|(anahtar, deger)| match deger {
        Deger::Metin(metin) if anahtar == ad => Some(metin.clone()),
        _ => None,
    })
}

/// Çözücü işlemini taze ortamda çalıştırır. Boş sözlük, `yok` ya da eksik
/// alanlı sonuç "kayıt yok"tur (fail-closed).
fn cozucuyu_cagir(
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
    ad: &str,
    kullanici: &str,
    onek: &str,
    satir: usize,
) -> Result<Option<Kayit>, Tani> {
    let Some(islem) = program.program().islemler.get(ad) else {
        return Ok(None);
    };
    let mut yerel = HashMap::new();
    let argumanlar = [kullanici, onek];
    for (param, deger) in islem.parametreler.iter().zip(argumanlar) {
        ortama_yaz(&mut yerel, &param.ad, Deger::Metin(deger.into()), satir)?;
    }
    let sonuc = blok_calistir(&islem.govde, &mut yerel, program, io, 1);
    heap_butcesini_iade_et(ortam_iade_bayti(&yerel));
    let Akis::Don(Deger::Sozluk(girdiler)) = sonuc? else {
        return Ok(None);
    };
    Ok((|| {
        Some(Kayit {
            ozet: metin_alani(&girdiler, "özet")?,
            kullanici: metin_alani(&girdiler, "kullanıcı")?,
            rol: metin_alani(&girdiler, "rol")?,
            kapsam: metin_alani(&girdiler, "kapsam")?,
            kimlik: metin_alani(&girdiler, "kimlik")?,
        })
    })())
}

/// Kimlik kararı. `Some(oturum)` rotanın çalışacağını, `None` yanıtın
/// (401/403/429) adaptöre zaten gönderildiğini söyler.
fn kimligi_coz(
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
    yontem: HttpYontemi,
    satir: usize,
) -> Result<Option<Deger>, Tani> {
    let Some(kimlik) = io.dav_kimligi() else {
        io.dav_kimlik_reddi_gonder();
        return Ok(None);
    };
    let katlanmis = kullanici_katla(&kimlik.kullanici);
    if let Err(saniye) = io.dav_kovasi(DavKova::Bak, &katlanmis) {
        io.dav_kova_reddi_gonder(saniye);
        return Ok(None);
    }
    let kayit = match (&kimlik.onek, &kimlik.cozucu) {
        (Some(onek), Some(cozucu)) => cozucuyu_cagir(program, io, cozucu, &katlanmis, onek, satir)?,
        _ => None,
    };
    let ozet_dogru = io.dav_ozet_dogrula(kayit.as_ref().map(|k| k.ozet.as_str()));
    let kayit = kayit.filter(|k| ozet_dogru && kullanici_katla(&k.kullanici) == katlanmis);
    let Some(kayit) = kayit else {
        match io.dav_kovasi(DavKova::Basarisiz, &katlanmis) {
            Err(saniye) => io.dav_kova_reddi_gonder(saniye),
            Ok(()) => io.dav_kimlik_reddi_gonder(),
        }
        return Ok(None);
    };
    if let Err(saniye) = io.dav_kovasi(DavKova::Basarili, &kayit.kimlik) {
        io.dav_kova_reddi_gonder(saniye);
        return Ok(None);
    }
    if kayit.kapsam == "salt_okuma" && !yontem.guvenli() {
        io.durum_yaniti_gonder(403, "uygulama parolası salt okuma kapsamında");
        return Ok(None);
    }
    Ok(Some(Deger::Sozluk(vec![
        ("kullanıcı".into(), Deger::Metin(kayit.kullanici)),
        ("rol".into(), Deger::Metin(kayit.rol)),
        (
            "kimlik_türü".into(),
            Deger::Metin("uygulama_parolası".into()),
        ),
        (
            "uygulama_parolası_kimliği".into(),
            Deger::Metin(kayit.kimlik),
        ),
        ("kapsam".into(), Deger::Metin(kayit.kapsam)),
    ])))
}

/// Rotanın `çerezler` ve `oturum` sözlükleri. Olağan rota gelen oturumu
/// görür; önbelleklenebilir ve makine rotası boş sözlük alır (T057/T061
/// kapısını aşan yol olsa bile yanıt kimliğe bağlanamaz); DAV rotası boş
/// çerez ve kimlik kararından gelen `oturum` alır. `None`: yanıt gönderildi.
pub(super) fn rota_sozlukleri(
    kip: RotaKipi,
    (program, io, yontem, satir): (
        CalistirmaProgrami<'_>,
        &mut dyn GirdiCikti,
        HttpYontemi,
        usize,
    ),
    cerezler: &Deger,
    oturum: &Deger,
) -> Result<Option<(Deger, Deger)>, Tani> {
    let bos = || Deger::Sozluk(Vec::new());
    Ok(match kip {
        RotaKipi::Olagan => Some((cerezler.clone(), oturum.clone())),
        RotaKipi::Dav => kimligi_coz(program, io, yontem, satir)?.map(|oturum| (bos(), oturum)),
        RotaKipi::Onbellekli(_) | RotaKipi::Makine => Some((bos(), bos())),
    })
}

/// OPTIONS: yolda en az bir DAV rotası varsa kimliksiz 200 + `Allow` +
/// `DAV: calendar-access, addressbook` (RFC-0047 soru 4); değilse 501.
pub(super) fn secenekler(
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
    yol: &str,
) -> Result<(), Tani> {
    let mut izinli = Vec::new();
    let mut dav = false;
    for cumle in &program.program().cumleler {
        let Cumle::IstekGeldiginde {
            yontem,
            yol: kayitli,
            onekli,
            govde,
            satir,
            ..
        } = cumle
        else {
            continue;
        };
        let kayitli = metne_sinirli(
            &degerlendir(kayitli, &HashMap::new(), program, io, 0, *satir)?,
            *satir,
        )?;
        let uydu = if *onekli {
            yol.starts_with(&kayitli)
        } else {
            kayitli == yol
        };
        if uydu {
            izinli.push(yontem.unwrap_or(HttpYontemi::Get));
            dav |= matches!(
                govde.first(),
                Some(Cumle::RotaPolitikasi {
                    kip: RotaKipi::Dav,
                    ..
                })
            );
        }
    }
    if dav {
        io.secenekler_gonder(&format!("{}, OPTIONS", HttpYontemi::allow_degeri(&izinli)));
    } else {
        io.durum_yaniti_gonder(501, "bu HTTP yöntemi desteklenmiyor");
    }
    Ok(())
}
