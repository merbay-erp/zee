//! K-210: görsel küçültme adaptörü (`image` crate; JPEG/PNG/WebP).
//!
//! Zee'de FFI yoktur; dönüşüm derleyicinin denetlenmiş bağımlılığıyla süreç
//! içinde yapılır. Kaynak baytları 16 MiB ile, çözülen piksel sayısı 64 MP ile
//! sınırlıdır. Hedef biçimi hedef dosya uzantısından seçilir: `.jpg/.jpeg`
//! (kalite 82), `.png`, `.webp` (kayıpsız). WASM playground bu adaptörü sunmaz.

/// En çok kaç bayt kaynak görsel okunur.
pub const AZAMI_KAYNAK_BAYTI: u64 = 16 * 1024 * 1024;
/// En çok kaç piksel çözülür (genişlik × yükseklik).
pub const AZAMI_PIKSEL: u64 = 64 * 1024 * 1024;

#[cfg(not(target_arch = "wasm32"))]
pub fn kucult(
    kaynak: &std::path::Path,
    hedef: &std::path::Path,
    azami_kenar: u32,
) -> Result<(u32, u32), String> {
    use image::codecs::jpeg::JpegEncoder;
    use image::codecs::png::PngEncoder;
    use image::codecs::webp::WebPEncoder;
    use image::{GenericImageView, ImageEncoder, ImageReader};
    use std::io::BufWriter;

    if azami_kenar == 0 || azami_kenar > 8192 {
        return Err("azami kenar 1..8192 piksel arasında olmalı".into());
    }
    let meta =
        std::fs::metadata(kaynak).map_err(|hata| format!("kaynak görsel okunamadı: {hata}"))?;
    if !meta.is_file() {
        return Err("kaynak görsel normal bir dosya değil".into());
    }
    if meta.len() > AZAMI_KAYNAK_BAYTI {
        return Err("kaynak görsel 16 MiB sınırını aşıyor".into());
    }
    let okuyucu = ImageReader::open(kaynak)
        .map_err(|hata| format!("kaynak görsel açılamadı: {hata}"))?
        .with_guessed_format()
        .map_err(|hata| format!("kaynak görsel biçimi tanınmadı: {hata}"))?;
    let (genislik, yukseklik) = okuyucu
        .into_dimensions()
        .map_err(|hata| format!("kaynak görsel boyutu okunamadı: {hata}"))?;
    if u64::from(genislik) * u64::from(yukseklik) > AZAMI_PIKSEL {
        return Err("kaynak görsel 64 megapiksel sınırını aşıyor".into());
    }
    let gorsel = ImageReader::open(kaynak)
        .map_err(|hata| format!("kaynak görsel açılamadı: {hata}"))?
        .with_guessed_format()
        .map_err(|hata| format!("kaynak görsel biçimi tanınmadı: {hata}"))?
        .decode()
        .map_err(|hata| format!("kaynak görsel çözülemedi: {hata}"))?;
    let kucuk = if genislik <= azami_kenar && yukseklik <= azami_kenar {
        gorsel
    } else {
        gorsel.thumbnail(azami_kenar, azami_kenar)
    };
    let (yeni_g, yeni_y) = kucuk.dimensions();
    let uzanti = hedef
        .extension()
        .and_then(|u| u.to_str())
        .map(|u| u.to_ascii_lowercase())
        .unwrap_or_default();
    let dosya =
        std::fs::File::create(hedef).map_err(|hata| format!("hedef görsel yazılamadı: {hata}"))?;
    let mut yazici = BufWriter::new(dosya);
    match uzanti.as_str() {
        "jpg" | "jpeg" => {
            let rgb = kucuk.to_rgb8();
            JpegEncoder::new_with_quality(&mut yazici, 82)
                .write_image(rgb.as_raw(), yeni_g, yeni_y, image::ExtendedColorType::Rgb8)
                .map_err(|hata| format!("JPEG kodlanamadı: {hata}"))?;
        }
        "png" => {
            let rgba = kucuk.to_rgba8();
            PngEncoder::new(&mut yazici)
                .write_image(
                    rgba.as_raw(),
                    yeni_g,
                    yeni_y,
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|hata| format!("PNG kodlanamadı: {hata}"))?;
        }
        "webp" => {
            let rgba = kucuk.to_rgba8();
            WebPEncoder::new_lossless(&mut yazici)
                .write_image(
                    rgba.as_raw(),
                    yeni_g,
                    yeni_y,
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|hata| format!("WebP kodlanamadı: {hata}"))?;
        }
        _ => return Err("hedef görsel uzantısı .jpg, .png ya da .webp olmalı".into()),
    }
    use std::io::Write;
    yazici
        .flush()
        .map_err(|hata| format!("hedef görsel yazılamadı: {hata}"))?;
    Ok((yeni_g, yeni_y))
}

#[cfg(target_arch = "wasm32")]
pub fn kucult(
    _kaynak: &std::path::Path,
    _hedef: &std::path::Path,
    _azami_kenar: u32,
) -> Result<(u32, u32), String> {
    Err("playground görsel küçültemez".into())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn png_kucultulur_ve_jpeg_yazilir() {
        let dizin = std::env::temp_dir().join(format!("zee-gorsel-{}", std::process::id()));
        std::fs::create_dir_all(&dizin).unwrap();
        let kaynak = dizin.join("buyuk.png");
        let mut tuval = image::RgbImage::new(400, 200);
        for (x, _y, p) in tuval.enumerate_pixels_mut() {
            *p = image::Rgb([(x % 255) as u8, 90, 30]);
        }
        tuval.save(&kaynak).unwrap();
        let hedef = dizin.join("kucuk.jpg");
        let (g, y) = kucult(&kaynak, &hedef, 100).unwrap();
        assert_eq!((g, y), (100, 50));
        let geri = image::open(&hedef).unwrap();
        assert_eq!(geri.width(), 100);
        let webp = dizin.join("kucuk.webp");
        assert_eq!(
            kucult(&kaynak, &webp, 1000).unwrap(),
            (400, 200),
            "büyütülmez"
        );
        assert!(kucult(&kaynak, &dizin.join("x.gif"), 50).is_err());
        assert!(kucult(&kaynak, &hedef, 0).is_err());
        std::fs::remove_dir_all(&dizin).ok();
    }
}
