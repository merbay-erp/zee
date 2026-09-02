use super::*;

pub(super) fn degerlendir_async<'a>(
    ifade: &'a Ifade,
    ortam: &'a HashMap<String, Deger>,
    program: CalistirmaProgrami<'a>,
    io: &'a mut dyn GirdiCikti,
    derinlik: usize,
    satir: usize,
) -> Pin<Box<dyn Future<Output = Result<Deger, Tani>> + 'a>> {
    Box::pin(async move {
    match ifade.turu() {
        Ifade::MetinSabiti(m) => Ok(Deger::Metin(m.clone())),
        Ifade::SayiSabiti(s) => Ok(Deger::TamSayi(*s)),
        Ifade::OndalikSabiti { govde, olcek } => Ondalik::govdeden(govde, *olcek)
            .map(ondalik_degeri)
            .ok_or_else(|| ic_hata(satir)),
        Ifade::MantiksalSabiti(b) => Ok(Deger::Mantiksal(*b)),
        Ifade::Intrinsic { kimlik, argumanlar } => {
            let Some(tanim) = intrinsic::tanim(kimlik) else {
                return Err(ic_hata(satir));
            };
            if argumanlar.len() != tanim.arguman_turleri.len() {
                return Err(ic_hata(satir));
            }
            let mut degerler = Vec::with_capacity(argumanlar.len());
            for arguman in argumanlar {
                degerler.push(
                    degerlendir_async(arguman, ortam, program, io, derinlik, satir).await?,
                );
            }
            match kimlik.as_str() {
                CSRF_BELIRTECI => io.csrf_belirteci().map(Deger::Metin).map_err(|hata| {
                    Tani::yeni(
                        "C022",
                        format!("CSRF belirteci üretilemedi: {}.", hata),
                        satir,
                        1,
                        1,
                    )
                }),
                PAROLA_DOGRULA => {
                    let [parola, ozet] = degerler.as_slice() else {
                        return Err(ic_hata(satir));
                    };
                    Ok(Deger::Mantiksal(
                        io.parola_dogrula(&parola.metne(), &ozet.metne()),
                    ))
                }
                HTTP_GETIR => {
                    let [Deger::Metin(url)] = degerler.as_slice() else {
                        return Err(ic_hata(satir));
                    };
                    let zaman_asimi_ms = son_tarih_kalani(io, satir)?.map(|(_, kalan)| kalan);
                    if gorevde_miyiz() {
                        // İstek adaptörüne girmeden kardeşlere bir tur ver. Mevcut IO
                        // trait'i senkrondur; adaptör çağrısının içi atomik kalır.
                        gorev_bekleme_noktasi(0).await;
                    }
                    let (durum, govde) = match io.http_getir(url, zaman_asimi_ms) {
                        Ok(yanit) => {
                            son_tarihi_denetle(io, satir)?;
                            yanit
                        }
                        Err(hata) => {
                            son_tarihi_denetle(io, satir)?;
                            return Err(
                                Tani::yeni(
                                    "C018",
                                    format!("Ağ isteği başarısız: {}.", hata),
                                    satir,
                                    1,
                                    1,
                                )
                                .onerili(
                                    "Ağ hatası yönetilecekse ileride \"getirmeyi dene\" gelecek (RFC-0008 §4.3)."
                                        .into(),
                                ),
                            );
                        }
                    };
                    Ok(Deger::AgYaniti { durum, govde })
                }
                SENSOR_ACIK_MI => {
                    let [Deger::Metin(ad)] = degerler.as_slice() else {
                        return Err(ic_hata(satir));
                    };
                    Ok(Deger::Mantiksal(io.sensor_acik_mi(ad)))
                }
                _ => Err(ic_hata(satir)),
            }
        }
        Ifade::BosListe => Ok(Deger::Liste(Vec::new())),
        Ifade::ListeSabiti(ogeler) => {
            let mut degerler = Vec::new();
            for oge in ogeler {
                degerler.push(degerlendir_async(oge, ortam, program, io, derinlik, satir).await?);
            }
            // Sayısal karışım Ondalık'a genişler (RFC-0013 §2): öğeler gerçekten
            // dönüştürülür ki listenin türü ile içeriği tutarlı kalsın.
            if degerler.iter().any(|d| matches!(d, Deger::Ondalik(_))) {
                for deger in degerler.iter_mut() {
                    if let Deger::TamSayi(v) = deger {
                        *deger = ondalik_degeri(Ondalik::tam(*v));
                    }
                }
            }
            Ok(Deger::Liste(degerler))
        }
        Ifade::Ozellik { nesne, ozellik } => {
            let nesne = degerlendir_async(nesne, ortam, program, io, derinlik, satir).await?;
            match (ozellik, nesne) {
                (Ozellik::Adet, Deger::Liste(ogeler)) => Ok(Deger::TamSayi(ogeler.len() as i64)),
                (Ozellik::Ilk, Deger::Liste(ogeler)) | (Ozellik::Son, Deger::Liste(ogeler)) => {
                    let oge = if *ozellik == Ozellik::Ilk {
                        ogeler.first()
                    } else {
                        ogeler.last()
                    };
                    oge.cloned().ok_or_else(|| {
                        Tani::yeni(
                            "C007",
                            "Liste boş: ilki/sonu alınamaz.".into(),
                            satir,
                            1,
                            1,
                        )
                        .onerili("Önce \"listenin adedi\" ile boş olup olmadığını kontrol et.".into())
                    })
                }
                (Ozellik::Kirpilmis, Deger::Metin(m)) => Ok(Deger::Metin(m.trim().to_string())),
                (Ozellik::Metni, deger) => Ok(Deger::Metin(deger.metne())),
                (
                    Ozellik::BinlikliKuruslu,
                    deger @ (Deger::Ondalik(_) | Deger::TamSayi(_)),
                ) => {
                    let ondalik = sayisal_ac(&deger).ok_or_else(|| ic_hata(satir))?;
                    Ok(Deger::Metin(ondalik.kuruslu(true)))
                }
                (Ozellik::Kuruslu, deger @ (Deger::Ondalik(_) | Deger::TamSayi(_))) => {
                    // K-065: daima iki hane; yarımlar sıfırdan uzağa (dil kuralı).
                    let ondalik = sayisal_ac(&deger).ok_or_else(|| ic_hata(satir))?;
                    Ok(Deger::Metin(ondalik.kuruslu(false)))
                }
                (Ozellik::Siralanmis, Deger::Liste(mut ogeler)) => {
                    ogeler.sort_by(deger_sirasi);
                    Ok(Deger::Liste(ogeler))
                }
                (Ozellik::Ters, Deger::Liste(mut ogeler)) => {
                    ogeler.reverse();
                    Ok(Deger::Liste(ogeler))
                }
                (Ozellik::CsvMetin, Deger::Liste(satirlar)) => {
                    Ok(Deger::Metin(csv_yaz(&satirlar)))
                }
                (Ozellik::Harfler, Deger::Metin(m)) => Ok(Deger::Liste(
                    m.chars().map(|k| Deger::Metin(k.to_string())).collect(),
                )),
                (Ozellik::JsonMetin, deger) => Ok(Deger::Metin(json_yaz(&deger))),
                (Ozellik::HtmlGuvenli, Deger::Metin(m)) => {
                    let mut kacisli = String::with_capacity(m.len());
                    for k in m.chars() {
                        match k {
                            '&' => kacisli.push_str("&amp;"),
                            '<' => kacisli.push_str("&lt;"),
                            '>' => kacisli.push_str("&gt;"),
                            '"' => kacisli.push_str("&quot;"),
                            '\'' => kacisli.push_str("&#39;"),
                            b => kacisli.push(b),
                        }
                    }
                    Ok(Deger::Metin(kacisli))
                }
                (Ozellik::Uzunluk, Deger::Metin(m)) => {
                    Ok(Deger::TamSayi(m.chars().count() as i64))
                }
                (Ozellik::Kelimeler, Deger::Metin(m)) => Ok(Deger::Liste(
                    m.split_whitespace()
                        .map(|k| Deger::Metin(k.to_string()))
                        .collect(),
                )),
                (Ozellik::Yil, Deger::Tarih { yil, .. }) => Ok(Deger::TamSayi(yil)),
                (Ozellik::HataKodu, Deger::Hata(hata)) => Ok(Deger::Metin(hata.kod)),
                (Ozellik::HataMesaji, Deger::Hata(hata)) => Ok(Deger::Metin(hata.mesaj)),
                (Ozellik::HataNedeni, Deger::Hata(hata)) => Ok(match hata.neden {
                    Some(neden) => Deger::Hata(neden),
                    None => Deger::Yok,
                }),
                (Ozellik::HataVerisi, Deger::Hata(hata)) => Ok(Deger::Sozluk(hata.veri)),
                // K-067 terfisi: TamSayı üzerinde tam kısmı/yuvarlanmışı kimliktir.
                (Ozellik::TamKisim, Deger::TamSayi(s))
                | (Ozellik::Yuvarlanmis, Deger::TamSayi(s)) => Ok(Deger::TamSayi(s)),
                (Ozellik::TamKisim, Deger::Ondalik(ondalik)) => ondalik
                    .tam_kismi()
                    .map(Deger::TamSayi)
                    .ok_or_else(|| tasma(satir)),
                (Ozellik::Yuvarlanmis, Deger::Ondalik(ondalik)) => ondalik
                    .yuvarlanmisi()
                    .map(Deger::TamSayi)
                    .ok_or_else(|| tasma(satir)),
                _ => Err(ic_hata(satir)),
            }
        }
        Ifade::BosSozluk => Ok(Deger::Sozluk(Vec::new())),
        Ifade::SozlukDegeri { sozluk, anahtar } => {
            let girdiler = match degerlendir_async(sozluk, ortam, program, io, derinlik, satir).await? {
                Deger::Sozluk(girdiler) => girdiler,
                _ => return Err(ic_hata(satir)),
            };
            let anahtar = match degerlendir_async(anahtar, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            girdiler
                .into_iter()
                .find(|(a, _)| *a == anahtar)
                .map(|(_, d)| d)
                .ok_or_else(|| {
                    Tani::yeni(
                        "C010",
                        format!("Sözlükte \"{}\" anahtarı yok.", anahtar),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Önce \"sözlükte <anahtar> varsa\" ile kontrol et.".into())
                })
        }
        Ifade::SozlukteVar { sozluk, anahtar, olumsuz } => {
            {
                // K-058: liste üyeliği — aynı yüzey.
                let kap = degerlendir_async(sozluk, ortam, program, io, derinlik, satir).await?;
                if let Deger::Liste(ogeler) = kap {
                    let aranan = degerlendir_async(anahtar, ortam, program, io, derinlik, satir).await?;
                    let var = ogeler.iter().any(|o| degerler_esit(o, &aranan));
                    return Ok(Deger::Mantiksal(var != *olumsuz));
                }
            }
            let girdiler = match degerlendir_async(sozluk, ortam, program, io, derinlik, satir).await? {
                Deger::Sozluk(girdiler) => girdiler,
                _ => return Err(ic_hata(satir)),
            };
            let anahtar = match degerlendir_async(anahtar, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let var = girdiler.iter().any(|(a, _)| *a == anahtar);
            Ok(Deger::Mantiksal(var != *olumsuz))
        }
        Ifade::MetinDonusum { nesne, buyuk } => {
            let metin = match degerlendir_async(nesne, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            Ok(Deger::Metin(if *buyuk {
                turkce_buyuk(&metin)
            } else {
                turkce_kucuk(&metin)
            }))
        }
        Ifade::Icerir { metin, aranan } => {
            let metin = match degerlendir_async(metin, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let aranan = match degerlendir_async(aranan, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            Ok(Deger::Mantiksal(metin.contains(&aranan)))
        }
        Ifade::YokSabiti => Ok(Deger::Yok),
        Ifade::BugununTarihi => {
            let (yil, ay, gun, _, _) = io.simdi();
            Ok(Deger::Tarih { yil, ay, gun })
        }
        Ifade::SuAninSaati => {
            let (_, _, _, saat, dakika) = io.simdi();
            Ok(Deger::Saat { saat, dakika })
        }
        Ifade::SureSabiti { milisaniye } => Ok(Deger::Sure { milisaniye: *milisaniye }),
        Ifade::DurumKodu(nesne) => match degerlendir_async(nesne, ortam, program, io, derinlik, satir).await? {
            Deger::AgYaniti { durum, .. } => Ok(Deger::TamSayi(durum)),
            _ => Err(ic_hata(satir)),
        },
        Ifade::Govde(nesne) => match degerlendir_async(nesne, ortam, program, io, derinlik, satir).await? {
            Deger::AgYaniti { govde, .. } => Ok(Deger::Metin(govde)),
            _ => Err(ic_hata(satir)),
        },
        Ifade::KomutArgumanlari => Ok(Deger::Liste(
            io.argumanlar().into_iter().map(Deger::Metin).collect(),
        )),
        Ifade::GunFarki { birinci, ikinci } => {
            let bir = degerlendir_async(birinci, ortam, program, io, derinlik, satir).await?;
            let iki = degerlendir_async(ikinci, ortam, program, io, derinlik, satir).await?;
            match (bir, iki) {
                (
                    Deger::Tarih { yil: y1, ay: a1, gun: g1 },
                    Deger::Tarih { yil: y2, ay: a2, gun: g2 },
                ) => Ok(Deger::TamSayi(
                    tarihten_gunler(y2, a2, g2) - tarihten_gunler(y1, a1, g1),
                )),
                _ => Err(ic_hata(satir)),
            }
        }
        Ifade::GunSonrasi { tarih, miktar } => {
            let (yil, ay, gun) = match degerlendir_async(tarih, ortam, program, io, derinlik, satir).await? {
                Deger::Tarih { yil, ay, gun } => (yil, ay, gun),
                _ => return Err(ic_hata(satir)),
            };
            let miktar = tam_sayi(degerlendir_async(miktar, ortam, program, io, derinlik, satir).await?, satir)?;
            let (yil, ay, gun) = gunlerden_tarih(tarihten_gunler(yil, ay, gun) + miktar);
            Ok(Deger::Tarih { yil, ay, gun })
        }
        Ifade::BosMu { nesne, olumsuz } => {
            let bos = match degerlendir_async(nesne, ortam, program, io, derinlik, satir).await? {
                Deger::Liste(ogeler) => ogeler.is_empty(),
                Deger::Sozluk(girdiler) => girdiler.is_empty(),
                Deger::Metin(m) => m.is_empty(),
                _ => return Err(ic_hata(satir)),
            };
            Ok(Deger::Mantiksal(bos != *olumsuz))
        }
        Ifade::YeniYapi { yapi_adi, .. } => {
            let yapi = program
                .yapi(ifade, yapi_adi)
                .ok_or_else(|| ic_hata(satir))?;
            let alanlar = yapi
                .alanlar
                .iter()
                .map(|(alan, tur)| {
                    let varsayilan = match tur.as_str() {
                        "TamSayı" => Deger::TamSayi(0),
                        "Ondalık" => ondalik_degeri(Ondalik::tam(0)),
                        "Mantıksal" => Deger::Mantiksal(false),
                        _ => Deger::Metin(String::new()),
                    };
                    (alan.clone(), varsayilan)
                })
                .collect();
            Ok(Deger::Yapi(alanlar))
        }
        Ifade::AlanErisim { nesne, alan } => {
            let alanlar = match degerlendir_async(nesne, ortam, program, io, derinlik, satir).await? {
                Deger::Yapi(alanlar) => alanlar,
                _ => return Err(ic_hata(satir)),
            };
            alanlar
                .into_iter()
                .find(|(a, _)| a == alan)
                .map(|(_, d)| d)
                .ok_or_else(|| ic_hata(satir))
        }
        Ifade::SecenekVar { nesne, olumsuz } => {
            let deger = degerlendir_async(nesne, ortam, program, io, derinlik, satir).await?;
            let var = deger != Deger::Yok;
            Ok(Deger::Mantiksal(var != *olumsuz))
        }
        Ifade::IcDeger(nesne) => match degerlendir_async(nesne, ortam, program, io, derinlik, satir).await? {
            Deger::Yok => Err(Tani::yeni(
                "C008",
                "Değer yok: boş Seçenek'in değeri alınamaz.".into(),
                satir,
                1,
                1,
            )
            .onerili("Önce \"... varsa\" ile kontrol et.".into())),
            Deger::Sonuc { basarili: true, icerik } => Ok(*icerik),
            Deger::Sonuc { basarili: false, .. } => Err(Tani::yeni(
                "C009",
                "Sonuç başarısız: değeri yerine hatası var.".into(),
                satir,
                1,
                1,
            )
            .onerili("Önce \"... başarılıysa\" ile kontrol et.".into())),
            dolu => Ok(dolu),
        },
        Ifade::SonucHatasi(nesne) => match degerlendir_async(nesne, ortam, program, io, derinlik, satir).await? {
            Deger::Sonuc { basarili: false, icerik } => Ok(*icerik),
            Deger::Sonuc { basarili: true, .. } => Err(Tani::yeni(
                "C009",
                "Sonuç başarılı: hatası yok, değeri var.".into(),
                satir,
                1,
                1,
            )),
            _ => Err(ic_hata(satir)),
        },
        Ifade::SonucBasarili { nesne, olumsuz } => {
            match degerlendir_async(nesne, ortam, program, io, derinlik, satir).await? {
                Deger::Sonuc { basarili, .. } => Ok(Deger::Mantiksal(basarili != *olumsuz)),
                _ => Err(ic_hata(satir)),
            }
        }
        Ifade::DosyaOkumayiDene(yol) => {
            let yol = match degerlendir_async(yol, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            Ok(match io.dosya_oku(&yol) {
                Ok(icerik) => Deger::Sonuc {
                    basarili: true,
                    icerik: Box::new(Deger::Metin(icerik)),
                },
                Err(hata) => Deger::Sonuc {
                    basarili: false,
                    icerik: Box::new(Deger::hata("DOSYA_OKUMA", hata)),
                },
            })
        }
        Ifade::TabloOku(yol) => {
            let yol = match degerlendir_async(yol, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let icerik = io.dosya_oku(&yol).map_err(|hata| {
                Tani::yeni("C012", format!("Dosya okunamadı: {}.", hata), satir, 1, 1)
            })?;
            csv_ayristir(&icerik, satir)
        }
        Ifade::VeriOku(yol) => {
            let yol = match degerlendir_async(yol, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let icerik = io.dosya_oku(&yol).map_err(|hata| {
                Tani::yeni("C012", format!("Dosya okunamadı: {}.", hata), satir, 1, 1)
            })?;
            json_nesnesi_ayristir(&icerik, satir)
        }
        Ifade::DosyaSatirlari(yol) => {
            let yol = match degerlendir_async(yol, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let icerik = io.dosya_oku(&yol).map_err(|hata| {
                Tani::yeni("C012", format!("Dosya okunamadı: {}.", hata), satir, 1, 1).onerili(
                    "Hatası yönetilecekse \"... dosyasını okumayı dene\" ile Sonuç al.".into(),
                )
            })?;
            Ok(Deger::Liste(
                icerik
                    .lines()
                    .map(|satir| Deger::Metin(satir.to_string()))
                    .collect(),
            ))
        }
        Ifade::Parcala { metin, ayrac } => {
            let m = degerlendir_async(metin, ortam, program, io, derinlik, satir).await?.metne();
            let a = degerlendir_async(ayrac, ortam, program, io, derinlik, satir).await?.metne();
            let parcalar: Vec<Deger> = if a.is_empty() {
                m.chars().map(|k| Deger::Metin(k.to_string())).collect()
            } else {
                m.split(&a).map(|p| Deger::Metin(p.to_string())).collect()
            };
            Ok(Deger::Liste(parcalar))
        }
        Ifade::ListeBirlestir { liste, ayrac } => {
            let l = degerlendir_async(liste, ortam, program, io, derinlik, satir).await?;
            let a = degerlendir_async(ayrac, ortam, program, io, derinlik, satir).await?.metne();
            match l {
                Deger::Liste(ogeler) => Ok(Deger::Metin(
                    ogeler.iter().map(|o| o.metne()).collect::<Vec<_>>().join(&a),
                )),
                _ => Err(ic_hata(satir)),
            }
        }
        Ifade::Degistir { metin, eski, yeni } => {
            let m = degerlendir_async(metin, ortam, program, io, derinlik, satir).await?.metne();
            let e = degerlendir_async(eski, ortam, program, io, derinlik, satir).await?.metne();
            let y = degerlendir_async(yeni, ortam, program, io, derinlik, satir).await?.metne();
            if e.is_empty() {
                return Err(Tani::yeni(
                    "C004",
                    "Boş metnin yerine koyma yapılamaz.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("\"değişmişi\" için aranan parça boş olamaz.".into()));
            }
            Ok(Deger::Metin(m.replace(&e, &y)))
        }
        Ifade::MetinSinari { metin, parca, bitis } => {
            let m = degerlendir_async(metin, ortam, program, io, derinlik, satir).await?.metne();
            let p = degerlendir_async(parca, ortam, program, io, derinlik, satir).await?.metne();
            Ok(Deger::Mantiksal(if *bitis { m.ends_with(&p) } else { m.starts_with(&p) }))
        }
        Ifade::Rastgele { alt, ust } => {
            let alt = tam_sayi(degerlendir_async(alt, ortam, program, io, derinlik, satir).await?, satir)?;
            let ust = tam_sayi(degerlendir_async(ust, ortam, program, io, derinlik, satir).await?, satir)?;
            if alt > ust {
                return Err(Tani::yeni(
                    "C006",
                    format!("Rastgele aralığı ters: {} ile {} arasında sayı üretilemez.", alt, ust),
                    satir,
                    1,
                    1,
                ));
            }
            let deger = io.rastgele(alt, ust).clamp(alt, ust);
            Ok(Deger::TamSayi(deger))
        }
        Ifade::Degisken { cozulmus, ham, .. } => {
            let ad = program
                .sembol_adi(ifade, cozulmus.as_deref())
                .ok_or_else(|| ic_hata(satir))?;
            ortam
                .get(&ad)
                .cloned()
                .ok_or_else(|| {
                    Tani::yeni("C001", format!("\"{}\" için değer bulunamadı.", ham), satir, 1, 1)
                })
        }
        Ifade::Birlestir(parcalar) => {
            let mut metin = String::new();
            for parca in parcalar {
                metin.push_str(&degerlendir_async(parca, ortam, program, io, derinlik, satir).await?.metne());
            }
            Ok(Deger::Metin(metin))
        }
        Ifade::Karsilastirma { sol, sag, islec } => {
            let sol = degerlendir_async(sol, ortam, program, io, derinlik, satir).await?;
            let sag = degerlendir_async(sag, ortam, program, io, derinlik, satir).await?;
            // Sayısal çift değer üzerinden hizalanarak karşılaştırılır
            // (2 = 2,0 doğrudur; 1,5 < 2 çalışır — RFC-0013 §2).
            if let (Deger::Sure { milisaniye: a }, Deger::Sure { milisaniye: b }) = (&sol, &sag)
            {
                let sonuc = match islec {
                    Islec::Esit => a == b,
                    Islec::Buyuk => a > b,
                    Islec::Kucuk => a < b,
                    Islec::BuyukEsit => a >= b,
                    Islec::KucukEsit => a <= b,
                };
                return Ok(Deger::Mantiksal(sonuc));
            }
            let sonuc = match (&sol, &sag) {
                (Deger::TamSayi(a), Deger::TamSayi(b)) => match islec {
                    Islec::Esit => a == b,
                    Islec::Buyuk => a > b,
                    Islec::Kucuk => a < b,
                    Islec::BuyukEsit => a >= b,
                    Islec::KucukEsit => a <= b,
                },
                _ => match (sayisal_ac(&sol), sayisal_ac(&sag)) {
                    (Some(a), Some(b)) => match islec {
                        Islec::Esit => a.karsilastir(&b).is_eq(),
                        Islec::Buyuk => a.karsilastir(&b).is_gt(),
                        Islec::Kucuk => a.karsilastir(&b).is_lt(),
                        Islec::BuyukEsit => !a.karsilastir(&b).is_lt(),
                        Islec::KucukEsit => !a.karsilastir(&b).is_gt(),
                    },
                    _ => match islec {
                        Islec::Esit => sol == sag,
                        _ => return Err(ic_hata(satir)),
                    },
                },
            };
            Ok(Deger::Mantiksal(sonuc))
        }
        Ifade::MantiksalZincir { hepsi, parcalar } => {
            // Kısa devre: VE ilk yanlışta, VEYA ilk doğruda durur.
            for parca in parcalar {
                let deger = mantiksal(degerlendir_async(parca, ortam, program, io, derinlik, satir).await?, satir)?;
                if deger != *hepsi {
                    return Ok(Deger::Mantiksal(deger));
                }
            }
            Ok(Deger::Mantiksal(*hepsi))
        }
        Ifade::Degil(ic) => {
            let deger = mantiksal(degerlendir_async(ic, ortam, program, io, derinlik, satir).await?, satir)?;
            Ok(Deger::Mantiksal(!deger))
        }
        Ifade::Cift(ic) => {
            let s = tam_sayi(degerlendir_async(ic, ortam, program, io, derinlik, satir).await?, satir)?;
            Ok(Deger::Mantiksal(s % 2 == 0))
        }
        Ifade::Tek(ic) => {
            let s = tam_sayi(degerlendir_async(ic, ortam, program, io, derinlik, satir).await?, satir)?;
            Ok(Deger::Mantiksal(s % 2 != 0))
        }
        Ifade::Aritmetik { islec, sol, sag } => {
            let sol = degerlendir_async(sol, ortam, program, io, derinlik, satir).await?;
            let sag = degerlendir_async(sag, ortam, program, io, derinlik, satir).await?;
            sayisal_islem(islec, &sol, &sag, satir)
        }
        Ifade::IslemCagrisi { islem_adi, argumanlar, .. } => {
            let mut degerler = Vec::new();
            for arg in argumanlar {
                degerler.push(degerlendir_async(arg, ortam, program, io, derinlik, satir).await?);
            }
            islem_cagir(ifade, islem_adi, degerler, program, io, derinlik + 1, satir).await?
                .ok_or_else(|| ic_hata(satir))
        }
        Ifade::SayiyiDene(ic) => {
            let metin = match degerlendir_async(ic, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let kirpilmis = metin.trim();
            Ok(match kirpilmis.parse::<i64>() {
                Ok(sayi) => Deger::Sonuc {
                    basarili: true,
                    icerik: Box::new(Deger::TamSayi(sayi)),
                },
                Err(_) => Deger::Sonuc {
                    basarili: false,
                    icerik: Box::new(Deger::hata("SAYI_BICIMI", format!(
                        "\"{}\" sayıya çevrilemedi",
                        kirpilmis
                    ))),
                },
            })
        }
        Ifade::OndaligiDene(ic) => {
            let metin = match degerlendir_async(ic, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let kirpilmis = metin.trim().to_string();
            let deneme = Ondalik::metinden(&kirpilmis).map(ondalik_degeri);
            Ok(match deneme {
                Some(deger) => Deger::Sonuc { basarili: true, icerik: Box::new(deger) },
                None => Deger::Sonuc {
                    basarili: false,
                    icerik: Box::new(Deger::hata("ONDALIK_BICIMI", format!(
                        "\"{}\" ondalığa çevrilemedi",
                        kirpilmis
                    ))),
                },
            })
        }
        Ifade::Ondaligi(ic) => {
            let metin = match degerlendir_async(ic, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let kirpilmis = metin.trim();
            let hata = || {
                Tani::yeni(
                    "C004",
                    format!("\"{}\" ondalığa çevrilemedi.", kirpilmis),
                    satir,
                    1,
                    1,
                )
                .onerili("Ondalık, virgülle yazılır. Örnek: 3,14".into())
            };
            Ondalik::metinden(kirpilmis)
                .map(ondalik_degeri)
                .ok_or_else(hata)
        }
        Ifade::Sayisi(ic) => {
            let metin = match degerlendir_async(ic, ortam, program, io, derinlik, satir).await? {
                Deger::Metin(m) => m,
                _ => return Err(ic_hata(satir)),
            };
            let kirpilmis = metin.trim();
            kirpilmis.parse::<i64>().map(Deger::TamSayi).map_err(|_| {
                Tani::yeni(
                    "C004",
                    format!("\"{}\" sayıya çevrilemedi.", kirpilmis),
                    satir,
                    1,
                    1,
                )
                .onerili("Sayı yalnız rakamlardan oluşmalı. Örnek: 42".into())
            })
        }
        Ifade::Kaynakli { .. } => Err(ic_hata(satir)),
    }
    })
}
