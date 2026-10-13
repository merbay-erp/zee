use super::*;

/// Görünüm ağacını HTML'e yazar: metin ve öznitelik değerleri kaçışlı, `ham` kaçışsız,
/// Mantıksal öznitelik `doğru` ise yalnız ad, `yanlış` ise yok.
pub(super) fn gorunum_yaz_async<'a>(
    dugumler: &'a [crate::agac::GorunumDugumu],
    ortam: &'a HashMap<String, Deger>,
    program: CalistirmaProgrami<'a>,
    cikti: &'a mut dyn GirdiCikti,
    derinlik: usize,
    hedef: &'a mut String,
) -> Pin<Box<dyn Future<Output = Result<(), Tani>> + 'a>> {
    use crate::agac::GorunumDugumu;
    Box::pin(async move {
        for dugum in dugumler {
            match dugum {
                GorunumDugumu::Oge {
                    etiket,
                    oznitelikler,
                    cocuklar,
                    satir,
                } => {
                    metin_parcasi_ekle(hedef, "<", *satir)?;
                    metin_parcasi_ekle(hedef, etiket, *satir)?;
                    for (ad, ifade) in oznitelikler {
                        let deger =
                            degerlendir_async(ifade, ortam, program, cikti, derinlik, *satir)
                                .await?;
                        match deger {
                            Deger::Mantiksal(false) => {}
                            Deger::Mantiksal(true) => {
                                metin_parcasi_ekle(hedef, " ", *satir)?;
                                metin_parcasi_ekle(hedef, ad, *satir)?;
                            }
                            Deger::Metin(m) => {
                                metin_parcasi_ekle(hedef, " ", *satir)?;
                                metin_parcasi_ekle(hedef, ad, *satir)?;
                                metin_parcasi_ekle(hedef, "=\"", *satir)?;
                                let kacisli = html_kacisla(&m, *satir)?;
                                metin_parcasi_ekle(hedef, &kacisli, *satir)?;
                                metin_parcasi_ekle(hedef, "\"", *satir)?;
                            }
                            Deger::TamSayi(n) => {
                                metin_parcasi_ekle(hedef, " ", *satir)?;
                                metin_parcasi_ekle(hedef, ad, *satir)?;
                                metin_parcasi_ekle(hedef, "=\"", *satir)?;
                                metin_parcasi_ekle(hedef, &n.to_string(), *satir)?;
                                metin_parcasi_ekle(hedef, "\"", *satir)?;
                            }
                            _ => return Err(ic_hata(*satir)),
                        }
                    }
                    metin_parcasi_ekle(hedef, ">", *satir)?;
                    if BOS_OGELER.contains(&etiket.as_str()) {
                        continue;
                    }
                    gorunum_yaz_async(cocuklar, ortam, program, cikti, derinlik, hedef).await?;
                    metin_parcasi_ekle(hedef, "</", *satir)?;
                    metin_parcasi_ekle(hedef, etiket, *satir)?;
                    metin_parcasi_ekle(hedef, ">", *satir)?;
                }
                GorunumDugumu::Metin { ifade, satir } => {
                    let satir = *satir;
                    let deger =
                        degerlendir_async(ifade, ortam, program, cikti, derinlik, satir).await?;
                    let Deger::Metin(m) = deger else {
                        return Err(ic_hata(satir));
                    };
                    let kacisli = html_kacisla(&m, satir)?;
                    metin_parcasi_ekle(hedef, &kacisli, satir)?;
                }
                GorunumDugumu::Ham { ifade, satir } => {
                    let satir = *satir;
                    let deger =
                        degerlendir_async(ifade, ortam, program, cikti, derinlik, satir).await?;
                    let Deger::Metin(m) = deger else {
                        return Err(ic_hata(satir));
                    };
                    metin_parcasi_ekle(hedef, &m, satir)?;
                }
            }
        }
        Ok(())
    })
}
