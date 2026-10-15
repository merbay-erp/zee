//! Dosya yazma cümlesinin değerlendirme ve son tarih sınırı.
use super::*;

pub(super) async fn yaz_async(
    cumle: &Cumle,
    ortam: &HashMap<String, Deger>,
    program: CalistirmaProgrami<'_>,
    cikti: &mut dyn GirdiCikti,
    derinlik: usize,
) -> Result<(), Tani> {
    let Cumle::DosyayaYaz {
        yol,
        icerik,
        ekleme,
        satir,
    } = cumle
    else {
        return Err(ic_hata(1));
    };
    let yol = match degerlendir_async(yol, ortam, program, cikti, derinlik, *satir).await? {
        Deger::Metin(m) => m,
        _ => return Err(ic_hata(*satir)),
    };
    let icerik_degeri = degerlendir_async(icerik, ortam, program, cikti, derinlik, *satir).await?;
    let icerik = metne_sinirli(&icerik_degeri, *satir)?;
    son_tarihi_denetle(cikti, *satir)?;
    cikti.dosya_yaz(&yol, &icerik, *ekleme).map_err(|hata| {
        Tani::yeni(
            "C013",
            format!("Dosyaya yazılamadı: {}.", hata),
            *satir,
            1,
            1,
        )
    })?;
    Ok(())
}
