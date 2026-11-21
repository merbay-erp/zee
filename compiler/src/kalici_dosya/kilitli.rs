//! K-353: oku-değiştir-yaz tek süreçler-arası kilit penceresinde.

use super::{atomik_degistir, atomik_icerik_yaz, mevcut_icerik, DosyaKilidi};
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
}
