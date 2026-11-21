//! K-353: oku-değiştir-yaz tek süreçler-arası kilit penceresinde.
//! K-347: bildirim ve türetilmiş dosyası birlikte, karşılaştırmalı yayın.

use super::{atomik_degistir, atomik_icerik_yaz, ebeveyn, mevcut_icerik, DosyaKilidi};
use std::io;
use std::path::Path;

/// [`kilit_altinda_guncelle`] hatası: hangi adımın düştüğü korunur.
#[derive(Debug)]
pub enum KilitliGuncellemeHatasi<E> {
    /// Kilit dosyası ya da mevcut içerik okunamadı.
    Okuma(io::Error),
    /// Çağıranın dönüşümü reddetti; dosyaya dokunulmadı.
    Islem(E),
    /// Kilit alınamadı ya da yeni içerik yayımlanamadı.
    Yazma(io::Error),
}

/// K-353: oku-değiştir-yaz tek süreçler-arası kilit penceresinde. `islem`
/// mevcut bütün içeriği (yoksa `None`) görür ve yayımlanacak yeni içeriği
/// döndürür. İyimser oku+CAS döngüsünün aksine yavaş bir yazar hızlı
/// yazarlara sınırsızca yenilmez: ADR-075 kilit kuyruğu sırası ilerlemeyi
/// garanti eder, tek sınır spec/08'in 5 sn kilit bekleme süresidir.
pub fn kilit_altinda_guncelle<R, E>(
    yol: &Path,
    islem: impl FnOnce(Option<Vec<u8>>) -> Result<(Vec<u8>, R), E>,
) -> Result<R, KilitliGuncellemeHatasi<E>> {
    let _kilit = DosyaKilidi::al(yol).map_err(KilitliGuncellemeHatasi::Yazma)?;
    let onceki = mevcut_icerik(yol).map_err(KilitliGuncellemeHatasi::Okuma)?;
    let (yeni, sonuc) = islem(onceki).map_err(KilitliGuncellemeHatasi::Islem)?;
    atomik_icerik_yaz(yol, &yeni, atomik_degistir).map_err(KilitliGuncellemeHatasi::Yazma)?;
    Ok(sonuc)
}

/// [`karsilastirip_cift_yaz`] hatası.
#[derive(Debug)]
pub enum CiftYazimHatasi {
    /// Birincil dosya çağıranın okuduğu byte'larda değil; hiçbir dosya yazılmadı.
    Degisti,
    /// Kilit, okuma ya da birincil yayın düştü; hiçbir dosya değişmedi.
    Birincil(io::Error),
    /// İkincil yayın düştü; `geri_yuklendi` iki dosyanın da kilit altında
    /// yakalanan önceki byte'larına dönüp dönmediğini söyler.
    Ikincil {
        hata: io::Error,
        geri_yuklendi: bool,
    },
}

/// K-347: `birincil` hâlâ çağıranın okuduğu `beklenen` byte'lardaysa onu ve
/// aynı klasördeki `ikincil`i tek süreçler-arası kilit penceresinde yayımlar.
/// `proje.dil` + `proje.kilit` gibi bir bildirim ve ondan türetilen dosya
/// için: karşılaştırmasız yazımda eşzamanlı iki komut birbirinin
/// bağımlılığını eziyordu. İkincil yazılamazsa geri alma da aynı kilit
/// altında yapılır; araya başka yazar giremediğinden önceki byte'lar
/// karşılaştırmasız geri konabilir.
pub fn karsilastirip_cift_yaz(
    birincil: &Path,
    beklenen: &[u8],
    yeni: &[u8],
    ikincil: &Path,
    ikincil_yeni: &[u8],
) -> Result<(), CiftYazimHatasi> {
    if ebeveyn(birincil) != ebeveyn(ikincil) || birincil == ikincil {
        return Err(CiftYazimHatasi::Birincil(io::Error::new(
            io::ErrorKind::InvalidInput,
            "çift yazım aynı klasörde iki ayrı dosya ister",
        )));
    }
    let _kilit = DosyaKilidi::al(birincil).map_err(CiftYazimHatasi::Birincil)?;
    #[cfg(windows)]
    super::yasam_dongusu::yarim_degistirmeyi_kurtar(ikincil).map_err(CiftYazimHatasi::Birincil)?;
    let guncel = mevcut_icerik(birincil).map_err(CiftYazimHatasi::Birincil)?;
    if guncel.as_deref() != Some(beklenen) {
        return Err(CiftYazimHatasi::Degisti);
    }
    let ikincil_onceki = mevcut_icerik(ikincil).map_err(CiftYazimHatasi::Birincil)?;
    atomik_icerik_yaz(birincil, yeni, atomik_degistir).map_err(CiftYazimHatasi::Birincil)?;
    let Err(hata) = atomik_icerik_yaz(ikincil, ikincil_yeni, atomik_degistir) else {
        return Ok(());
    };
    let birincil_geri = atomik_icerik_yaz(birincil, beklenen, atomik_degistir);
    let ikincil_geri = match ikincil_onceki {
        Some(onceki) => atomik_icerik_yaz(ikincil, &onceki, atomik_degistir),
        None => match std::fs::remove_file(ikincil) {
            Err(hata) if hata.kind() != io::ErrorKind::NotFound => Err(hata),
            _ => Ok(()),
        },
    };
    Err(CiftYazimHatasi::Ikincil {
        hata,
        geri_yuklendi: birincil_geri.is_ok() && ikincil_geri.is_ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// K-353: dönüşüm kilit altında koşar; başka yazar dönüşüm boyunca
    /// bekler ve güncelleme kaybolmaz. Dönüşüm reddi dosyaya dokunmaz.
    #[test]
    fn kilit_altinda_guncelle_yazari_bekletir_ve_reddi_yazmaz() {
        let gecici = super::super::tests::GeciciKlasor::yeni();
        let yol = gecici.0.join("sayac.txt");
        let hata = kilit_altinda_guncelle(&yol, |onceki| {
            assert!(onceki.is_none());
            Err::<(Vec<u8>, ()), _>("ret")
        });
        assert!(matches!(hata, Err(KilitliGuncellemeHatasi::Islem("ret"))));
        assert!(!yol.exists(), "reddedilen dönüşüm dosya oluşturmamalı");

        let iplikler: Vec<_> = (0..4)
            .map(|_| {
                let yol = yol.clone();
                std::thread::spawn(move || {
                    for _ in 0..25 {
                        kilit_altinda_guncelle(&yol, |onceki| {
                            let sayi = onceki
                                .map(|b| String::from_utf8(b).unwrap().parse::<u32>().unwrap())
                                .unwrap_or(0);
                            // Kilit dışı okuma olsaydı bu pencerede araya giren
                            // yazar artışı ezerdi.
                            std::thread::sleep(Duration::from_millis(1));
                            Ok::<_, ()>(((sayi + 1).to_string().into_bytes(), ()))
                        })
                        .expect("kilitli güncelleme");
                    }
                })
            })
            .collect();
        for iplik in iplikler {
            iplik.join().unwrap();
        }
        assert_eq!(std::fs::read_to_string(&yol).unwrap(), "100");
    }

    /// K-347: birincil değiştiyse hiçbir dosya yazılmaz; ikincil yazılamazsa
    /// iki dosya da önceki byte'larına döner.
    #[test]
    fn cift_yazim_karsilastirir_ve_ikincil_hatasinda_geri_alir() {
        let gecici = super::super::tests::GeciciKlasor::yeni();
        let (bildirim, kilit) = (gecici.0.join("proje.dil"), gecici.0.join("proje.kilit"));
        std::fs::write(&bildirim, "v1").unwrap();
        let hata = karsilastirip_cift_yaz(&bildirim, b"v0", b"v2", &kilit, b"k2").unwrap_err();
        assert!(matches!(hata, CiftYazimHatasi::Degisti));
        assert_eq!(std::fs::read(&bildirim).unwrap(), b"v1");
        assert!(!kilit.exists());

        karsilastirip_cift_yaz(&bildirim, b"v1", b"v2", &kilit, b"k2").unwrap();
        assert_eq!(std::fs::read(&bildirim).unwrap(), b"v2");
        assert_eq!(std::fs::read(&kilit).unwrap(), b"k2");

        super::super::adim::test_kancasi::kur(
            super::super::Adim::GeciciYazim,
            super::super::adim::test_kancasi::Enjeksiyon::Gecikme(Duration::ZERO),
        );
        super::super::adim::test_kancasi::kur(
            super::super::Adim::GeciciYazim,
            super::super::adim::test_kancasi::Enjeksiyon::OsHatasi(5),
        );
        let hata = karsilastirip_cift_yaz(&bildirim, b"v2", b"v3", &kilit, b"k3").unwrap_err();
        assert!(
            matches!(
                hata,
                CiftYazimHatasi::Ikincil {
                    geri_yuklendi: true,
                    ..
                }
            ),
            "{hata:?}"
        );
        assert_eq!(std::fs::read(&bildirim).unwrap(), b"v2");
        assert_eq!(std::fs::read(&kilit).unwrap(), b"k2");
    }
}
