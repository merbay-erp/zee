use super::*;

pub(super) fn blok_calistir_async<'a>(
    cumleler: &'a [Cumle],
    ortam: &'a mut HashMap<String, Deger>,
    program: CalistirmaProgrami<'a>,
    cikti: &'a mut dyn GirdiCikti,
    derinlik: usize,
) -> Pin<Box<dyn Future<Output = Result<Akis, Tani>> + 'a>> {
    Box::pin(async move {
    son_tarihi_denetle(cikti, 1)?;
    let mut bekleyen_gorevler: Option<Vec<BekleyenGorev>> = None;
    for cumle in cumleler {
        match cumle {
            Cumle::Yaz { deger, satir } => {
                let sonuc = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                cikti.yazdir(sonuc.metne());
            }
            Cumle::Sor { istem, satir } => {
                let istem = degerlendir_async(istem, ortam, program, cikti, derinlik, *satir).await?.metne();
                let cevap = cikti.sor(&istem).ok_or_else(|| {
                    Tani::yeni(
                        "C005",
                        "Soruya verilecek girdi kalmadı.".into(),
                        *satir,
                        1,
                        1,
                    )
                })?;
                ortam.insert("yanıt".to_string(), Deger::Metin(cevap));
            }
            Cumle::Olsun { ad, deger, satir, .. } => {
                let sonuc = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                ortam.insert(ad.clone(), sonuc);
            }
            Cumle::KezTekrarla { adet, govde, satir } => {
                let adet = tam_sayi(degerlendir_async(adet, ortam, program, cikti, derinlik, *satir).await?, *satir)?;
                let kapsam = kapsam_baslat(ortam);
                for _ in 0..adet.max(0) {
                    if let Akis::Don(d) = blok_calistir_async(govde, ortam, program, cikti, derinlik).await? {
                        return Ok(Akis::Don(d));
                    }
                }
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::AralikDongusu { ad, bastan, sona, govde, satir } => {
                let bastan = tam_sayi(degerlendir_async(bastan, ortam, program, cikti, derinlik, *satir).await?, *satir)?;
                let sona = tam_sayi(degerlendir_async(sona, ortam, program, cikti, derinlik, *satir).await?, *satir)?;
                let kapsam = kapsam_baslat(ortam);
                // K-068: aralık iki yönde çalışır — "5 ten 1 e kadar" geri sayar.
                let degerler: Vec<i64> = if bastan <= sona {
                    (bastan..=sona).collect()
                } else {
                    (sona..=bastan).rev().collect()
                };
                for deger in degerler {
                    ortam.insert(ad.clone(), Deger::TamSayi(deger));
                    if let Akis::Don(d) = blok_calistir_async(govde, ortam, program, cikti, derinlik).await? {
                        return Ok(Akis::Don(d));
                    }
                }
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::OlduguSurece { kosul, govde, satir } => {
                let kapsam = kapsam_baslat(ortam);
                loop {
                    let devam = mantiksal(degerlendir_async(kosul, ortam, program, cikti, derinlik, *satir).await?, *satir)?;
                    if !devam {
                        break;
                    }
                    if let Akis::Don(d) = blok_calistir_async(govde, ortam, program, cikti, derinlik).await? {
                        return Ok(Akis::Don(d));
                    }
                }
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::OlanaKadar { kosul, govde, satir } => {
                let kapsam = kapsam_baslat(ortam);
                loop {
                    let bitti = mantiksal(degerlendir_async(kosul, ortam, program, cikti, derinlik, *satir).await?, *satir)?;
                    if bitti {
                        break;
                    }
                    if let Akis::Don(d) = blok_calistir_async(govde, ortam, program, cikti, derinlik).await? {
                        return Ok(Akis::Don(d));
                    }
                }
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::Ise { kollar, degilse, satir } => {
                let mut islendi = false;
                for kol in kollar {
                    if mantiksal(degerlendir_async(&kol.kosul, ortam, program, cikti, derinlik, *satir).await?, *satir)? {
                        let kapsam = kapsam_baslat(ortam);
                        if let Akis::Don(d) = blok_calistir_async(&kol.govde, ortam, program, cikti, derinlik).await? {
                            return Ok(Akis::Don(d));
                        }
                        kapsam_bitir(ortam, &kapsam);
                        islendi = true;
                        break;
                    }
                }
                if !islendi {
                    if let Some(blok) = degilse {
                        let kapsam = kapsam_baslat(ortam);
                        if let Akis::Don(d) = blok_calistir_async(blok, ortam, program, cikti, derinlik).await? {
                            return Ok(Akis::Don(d));
                        }
                        kapsam_bitir(ortam, &kapsam);
                    }
                }
            }
            Cumle::Ekle { hedef, deger, satir } => {
                let ham_ad = match hedef {
                    Ifade::Degisken { cozulmus, .. } => cozulmus.as_deref(),
                    _ => None,
                };
                let ad = program
                    .sembol_adi(hedef, ham_ad)
                    .ok_or_else(|| ic_hata(*satir))?;
                let deger = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                match ortam.get_mut(&ad) {
                    Some(Deger::Liste(ogeler)) => ogeler.push(deger),
                    _ => return Err(ic_hata(*satir)),
                }
            }
            Cumle::Sil { kap, deger, satir } => {
                let ham_ad = match kap {
                    Ifade::Degisken { cozulmus, .. } => cozulmus.as_deref(),
                    _ => None,
                };
                let ad = program
                    .sembol_adi(kap, ham_ad)
                    .ok_or_else(|| ic_hata(*satir))?;
                let aranan = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                match ortam.get_mut(&ad) {
                    Some(Deger::Liste(ogeler)) => {
                        // İlk eşleşen öğe çıkar; yoksa sessizce hiçbir şey olmaz (K-059).
                        if let Some(yer) = ogeler.iter().position(|o| degerler_esit(o, &aranan)) {
                            ogeler.remove(yer);
                        }
                    }
                    Some(Deger::Sozluk(girdiler)) => {
                        let anahtar = aranan.metne();
                        girdiler.retain(|(a, _)| *a != anahtar);
                    }
                    _ => return Err(ic_hata(*satir)),
                }
            }
            Cumle::HerBiri { ad, kaynak, govde, satir } => {
                let kaynak = kaynak.as_ref().ok_or_else(|| ic_hata(*satir))?;
                let ogeler = match degerlendir_async(kaynak, ortam, program, cikti, derinlik, *satir).await? {
                    Deger::Liste(ogeler) => ogeler,
                    // Sözlük üzerinde gezinme anahtarları verir (ekleme sırasıyla).
                    Deger::Sozluk(girdiler) => girdiler
                        .into_iter()
                        .map(|(anahtar, _)| Deger::Metin(anahtar))
                        .collect(),
                    _ => return Err(ic_hata(*satir)),
                };
                // K-093: kaynak listeyse döngü adı değer-sonuç imlecidir;
                // alan yazma ve yeniden bağlama aynı sıraya GERİ YAZILIR.
                let kaynak_adi = match kaynak {
                    Ifade::Degisken { cozulmus, .. } => {
                        program.sembol_adi(kaynak, cozulmus.as_deref())
                    }
                    _ => None,
                };
                let liste_mi = matches!(
                    kaynak_adi.as_deref().and_then(|a| ortam.get(a)),
                    Some(Deger::Liste(_))
                );
                let kapsam = kapsam_baslat(ortam);
                for (sira, oge) in ogeler.into_iter().enumerate() {
                    ortam.insert(ad.clone(), oge);
                    let akis = blok_calistir_async(govde, ortam, program, cikti, derinlik).await?;
                    if liste_mi {
                        gezme_ogesini_geri_yaz(
                            ortam,
                            kaynak_adi.as_deref(),
                            ad,
                            sira,
                        );
                    }
                    if let Akis::Don(d) = akis {
                        return Ok(Akis::Don(d));
                    }
                }
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::Artir { ifade, miktar, satir } => {
                guncelle(ifade, miktar, ortam, program, cikti, derinlik, *satir, 1).await?;
            }
            Cumle::Azalt { ifade, miktar, satir } => {
                guncelle(ifade, miktar, ortam, program, cikti, derinlik, *satir, -1).await?;
            }
            Cumle::Gore { konu, kollar, degilse, satir } => {
                let konu = degerlendir_async(konu, ortam, program, cikti, derinlik, *satir).await?;
                let mut eslesti = false;
                for (deger, govde) in kollar {
                    let deger = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                    if deger == konu {
                        let kapsam = kapsam_baslat(ortam);
                        if let Akis::Don(d) = blok_calistir_async(govde, ortam, program, cikti, derinlik).await? {
                            return Ok(Akis::Don(d));
                        }
                        kapsam_bitir(ortam, &kapsam);
                        eslesti = true;
                        break;
                    }
                }
                if !eslesti {
                    if let Some(blok) = degilse {
                        let kapsam = kapsam_baslat(ortam);
                        if let Akis::Don(d) = blok_calistir_async(blok, ortam, program, cikti, derinlik).await? {
                            return Ok(Akis::Don(d));
                        }
                        kapsam_bitir(ortam, &kapsam);
                    }
                }
            }
            Cumle::SunucuBaslat { kapi, satir } => {
                let kapi = tam_sayi(degerlendir_async(kapi, ortam, program, cikti, derinlik, *satir).await?, *satir)?;
                cikti.sunucu_kur(kapi).map_err(|hata| {
                    Tani::yeni("C017", format!("Sunucu kurulamadı: {}.", hata), *satir, 1, 1)
                })?;
                // Dinleme, program gövdesi bitince başlar (calistir_io).
                ortam.insert("(sunucu)".to_string(), Deger::TamSayi(kapi));
            }
            Cumle::IstekGeldiginde { .. } => {
                // Yalnız kayıt: gövde, sunucu döngüsünde istek gelince koşulur.
            }
            Cumle::YanitGonder { deger, satir } => {
                let deger = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                cikti.yanit_gonder(&deger.metne());
            }
            Cumle::Yonlendir { adres, satir } => {
                let hedef = degerlendir_async(adres, ortam, program, cikti, derinlik, *satir).await?.metne();
                cikti.yonlendir_gonder(&hedef).map_err(|hata| {
                    Tani::yeni(
                        "C022",
                        format!("Yönlendirme reddedildi: {}.", hata),
                        *satir,
                        1,
                        1,
                    )
                })?;
            }
            Cumle::CerezSil { ad, satir } => {
                let ad = degerlendir_async(ad, ortam, program, cikti, derinlik, *satir).await?.metne();
                cikti.cerez_sil(&ad).map_err(|hata| {
                    Tani::yeni("C022", format!("Çerez silinemedi: {}.", hata), *satir, 1, 1)
                })?;
            }
            Cumle::CerezYaz { ad, deger, satir } => {
                let ad = degerlendir_async(ad, ortam, program, cikti, derinlik, *satir).await?.metne();
                let deger = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?.metne();
                cikti.cerez_yaz(&ad, &deger).map_err(|hata| {
                    Tani::yeni("C022", format!("Çerez yazılamadı: {}.", hata), *satir, 1, 1)
                })?;
            }
            Cumle::RotaPolitikasi { .. } | Cumle::RotaAlaniGerekli { .. } => {
                // Rota döngüsü gövde çalışmadan önce uygular.
            }
            Cumle::OturumAc {
                kullanici,
                rol,
                satir,
            } => {
                let kullanici =
                    degerlendir_async(kullanici, ortam, program, cikti, derinlik, *satir).await?.metne();
                let rol = degerlendir_async(rol, ortam, program, cikti, derinlik, *satir).await?.metne();
                cikti.oturum_ac(&kullanici, &rol).map_err(|hata| {
                    Tani::yeni(
                        "C022",
                        format!("Güvenli oturum açılamadı: {}.", hata),
                        *satir,
                        1,
                        1,
                    )
                })?;
            }
            Cumle::OturumKapat { satir } => {
                cikti.oturum_kapat().map_err(|hata| {
                    Tani::yeni(
                        "C022",
                        format!("Güvenli oturum kapatılamadı: {}.", hata),
                        *satir,
                        1,
                        1,
                    )
                })?;
            }
            Cumle::Eszamanli { gorevler, satir } => {
                if bekleyen_gorevler.is_some() {
                    return Err(ic_hata(*satir));
                }
                let baslangic_ortami = ortam.clone();
                bekleyen_gorevler = Some(
                    gorevler
                        .iter()
                        .map(|(ad, ifade, gorev_satiri)| BekleyenGorev {
                            ad: ad.clone(),
                            ifade,
                            satir: *gorev_satiri,
                            ortam: baslangic_ortami.clone(),
                        })
                        .collect(),
                );
            }
            Cumle::HepsiniBekle { satir } => {
                let gorevler = bekleyen_gorevler.take().ok_or_else(|| ic_hata(*satir))?;
                for (ad, sonuc) in
                    gorevleri_calistir(gorevler, program, cikti, derinlik).await?
                {
                    ortam.insert(ad, sonuc);
                }
                son_tarihi_denetle(cikti, *satir)?;
            }
            Cumle::IcindeBlogu { sure, govde, yetismezse, satir } => {
                let sure_ms = match degerlendir_async(sure, ortam, program, cikti, derinlik, *satir).await? {
                    Deger::Sure { milisaniye } => milisaniye,
                    _ => return Err(ic_hata(*satir)),
                };
                let baslangic = cikti.an_ms();
                let son_tarih = baslangic.saturating_add(sure_ms);
                let nobetci = SonTarihNobetcisi::yeni(son_tarih);
                let kimlik = nobetci.kimlik;
                let kapsam = kapsam_baslat(ortam);
                let sonuc = blok_calistir_async(govde, ortam, program, cikti, derinlik).await;
                kapsam_bitir(ortam, &kapsam);
                drop(nobetci);
                match sonuc {
                    Ok(Akis::Don(d)) => return Ok(Akis::Don(d)),
                    Ok(Akis::Devam) => {}
                    Err(tani) if bu_son_tarihin_iptali(&tani, kimlik) => {
                        if let Some(blok) = yetismezse {
                            let kapsam = kapsam_baslat(ortam);
                            if let Akis::Don(d) =
                                blok_calistir_async(blok, ortam, program, cikti, derinlik).await?
                            {
                                return Ok(Akis::Don(d));
                            }
                            kapsam_bitir(ortam, &kapsam);
                        }
                    }
                    Err(tani) => return Err(tani),
                }
            }
            Cumle::IsikAyarla { isik, yansin, .. } => {
                cikti.isik_ayarla(isik, *yansin);
            }
            Cumle::Bekle { sure, satir } => {
                let sure_ms = match degerlendir_async(sure, ortam, program, cikti, derinlik, *satir).await? {
                    Deger::Sure { milisaniye } => milisaniye,
                    _ => return Err(ic_hata(*satir)),
                };
                if let Some((son, kalan)) = son_tarih_kalani(cikti, *satir)? {
                    if sure_ms >= kalan {
                        if gorevde_miyiz() {
                            gorev_bekleme_noktasi(kalan).await;
                        } else {
                            cikti.bekle_ms(kalan);
                        }
                        return Err(son_tarih_tanisi(son, *satir));
                    }
                }
                if gorevde_miyiz() {
                    gorev_bekleme_noktasi(sure_ms).await;
                } else {
                    cikti.bekle_ms(sure_ms);
                }
            }
            Cumle::ProgramiBitir { kod, satir } => {
                // Ç000 mesajı çıkış kodunu taşır (K-069): "0" ya da verilen kod.
                let kod = match kod {
                    Some(ifade) => {
                        let deger = tam_sayi(
                            degerlendir_async(ifade, ortam, program, cikti, derinlik, *satir).await?,
                            *satir,
                        )?;
                        if !(0..=255).contains(&deger) {
                            return Err(Tani::yeni(
                                "C020",
                                format!("Çıkış kodu 0–255 arasında olmalı; {} verildi.", deger),
                                *satir,
                                1,
                                1,
                            ));
                        }
                        deger
                    }
                    None => 0,
                };
                return Err(Tani::yeni("Ç000", format!("{}", kod), *satir, 1, 1));
            }
            Cumle::IslemTanimi(islem) => return Err(ic_hata(islem.satir)),
            Cumle::YapiTanimi(yapi) => return Err(ic_hata(yapi.satir)),
            Cumle::TestBlogu(test) => return Err(ic_hata(test.satir)),
            Cumle::Kullan { satir, .. } => return Err(ic_hata(*satir)),
            Cumle::Olmali { kosul, satir } => {
                // Karşılaştırmalarda iki tarafın değeri tanıya yazılır —
                // "beklenen/bulunan" göstermek öğretici hata ilkesinin gereği.
                let (tuttu, detay) = match kosul {
                    Ifade::Karsilastirma { sol, sag, .. } => {
                        let sol_deger = degerlendir_async(sol, ortam, program, cikti, derinlik, *satir).await?;
                        let sag_deger = degerlendir_async(sag, ortam, program, cikti, derinlik, *satir).await?;
                        let sonuc =
                            mantiksal(degerlendir_async(kosul, ortam, program, cikti, derinlik, *satir).await?, *satir)?;
                        (
                            sonuc,
                            format!(
                                " Beklenen: {} — bulunan: {}.",
                                sag_deger.metne(),
                                sol_deger.metne()
                            ),
                        )
                    }
                    _ => (
                        mantiksal(degerlendir_async(kosul, ortam, program, cikti, derinlik, *satir).await?, *satir)?,
                        String::new(),
                    ),
                };
                if !tuttu {
                    return Err(Tani::yeni(
                        "D001",
                        format!("Doğrulama tutmadı.{}", detay),
                        *satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::AlanAta { nesne, alan, deger, satir } => {
                let ham_ad = match nesne {
                    Ifade::Degisken { cozulmus, .. } => cozulmus.as_deref(),
                    _ => None,
                };
                let ad = program
                    .sembol_adi(nesne, ham_ad)
                    .ok_or_else(|| ic_hata(*satir))?;
                let deger = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                match ortam.get_mut(&ad) {
                    Some(Deger::Yapi(alanlar)) => {
                        match alanlar.iter_mut().find(|(a, _)| a == alan) {
                            Some((_, eski)) => *eski = deger,
                            None => return Err(ic_hata(*satir)),
                        }
                    }
                    _ => return Err(ic_hata(*satir)),
                }
            }
            Cumle::Dondur { deger, sonuca_sarmala, satir } => {
                let sonuc = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                // Erken dönüş, cümle sonundaki ortak denetimi atlamamalıdır.
                son_tarihi_denetle(cikti, *satir)?;
                let sonuc = if *sonuca_sarmala {
                    Deger::Sonuc { basarili: true, icerik: Box::new(sonuc) }
                } else {
                    sonuc
                };
                return Ok(Akis::Don(sonuc));
            }
            Cumle::HataDondur { kod, mesaj, neden, veri, satir } => {
                let mesaj = degerlendir_async(mesaj, ortam, program, cikti, derinlik, *satir).await?;
                let mut hata = match (kod, mesaj) {
                    (None, hata @ Deger::Hata(_)) => hata,
                    (kod, Deger::Metin(mesaj)) => {
                        Deger::hata(kod.clone().unwrap_or_else(|| "GENEL".into()), mesaj)
                    }
                    _ => return Err(ic_hata(*satir)),
                };
                let neden = match neden {
                    Some(ifade) => match degerlendir_async(
                        ifade, ortam, program, cikti, derinlik, *satir,
                    ).await? {
                        Deger::Hata(hata) => Some(hata),
                        _ => return Err(ic_hata(*satir)),
                    },
                    None => None,
                };
                let veri = match veri {
                    Some(ifade) => match degerlendir_async(
                        ifade, ortam, program, cikti, derinlik, *satir,
                    ).await? {
                        Deger::Sozluk(veri) => veri,
                        _ => return Err(ic_hata(*satir)),
                    },
                    None => Vec::new(),
                };
                if let Deger::Hata(yapilandirilmis) = &mut hata {
                    if neden.is_some() {
                        yapilandirilmis.neden = neden;
                    }
                    if !veri.is_empty() {
                        yapilandirilmis.veri = veri;
                    }
                }
                son_tarihi_denetle(cikti, *satir)?;
                return Ok(Akis::Don(Deger::Sonuc {
                    basarili: false,
                    icerik: Box::new(hata),
                }));
            }
            Cumle::BolVeAta { hedef, pay, payda, satir } => {
                let pay = degerlendir_async(pay, ortam, program, cikti, derinlik, *satir).await?;
                let payda = degerlendir_async(payda, ortam, program, cikti, derinlik, *satir).await?;
                let sonuc = sayisal_islem(&AritmetikIslec::Bol, &pay, &payda, *satir)?;
                ortam.insert(hedef.clone(), sonuc);
            }
            Cumle::CagriCumlesi { cagri, satir } => {
                if let Ifade::IslemCagrisi { islem_adi, argumanlar, .. } = cagri {
                    let mut degerler = Vec::new();
                    for arg in argumanlar {
                        degerler.push(degerlendir_async(arg, ortam, program, cikti, derinlik, *satir).await?);
                    }
                    islem_cagir(cagri, islem_adi, degerler, program, cikti, derinlik + 1, *satir).await?;
                } else {
                    return Err(ic_hata(*satir));
                }
            }
            Cumle::DosyayaYaz { yol, icerik, ekleme, satir } => {
                let yol = match degerlendir_async(yol, ortam, program, cikti, derinlik, *satir).await? {
                    Deger::Metin(m) => m,
                    _ => return Err(ic_hata(*satir)),
                };
                let icerik = degerlendir_async(icerik, ortam, program, cikti, derinlik, *satir).await?.metne();
                cikti.dosya_yaz(&yol, &icerik, *ekleme).map_err(|hata| {
                    Tani::yeni("C013", format!("Dosyaya yazılamadı: {}.", hata), *satir, 1, 1)
                })?;
            }
            Cumle::SozlukAta { sozluk, anahtar, deger, satir } => {
                let ham_ad = match sozluk {
                    Ifade::Degisken { cozulmus, .. } => cozulmus.as_deref(),
                    _ => None,
                };
                let ad = program
                    .sembol_adi(sozluk, ham_ad)
                    .ok_or_else(|| ic_hata(*satir))?;
                let anahtar = match degerlendir_async(anahtar, ortam, program, cikti, derinlik, *satir).await? {
                    Deger::Metin(m) => m,
                    _ => return Err(ic_hata(*satir)),
                };
                let deger = degerlendir_async(deger, ortam, program, cikti, derinlik, *satir).await?;
                match ortam.get_mut(&ad) {
                    Some(Deger::Sozluk(girdiler)) => {
                        match girdiler.iter_mut().find(|(a, _)| *a == anahtar) {
                            Some((_, eski)) => *eski = deger,
                            None => girdiler.push((anahtar, deger)),
                        }
                    }
                    _ => return Err(ic_hata(*satir)),
                }
            }
        }
        son_tarihi_denetle(cikti, 1)?;
    }
    if let Some(gorevler) = bekleyen_gorevler {
        let satir = gorevler.first().map(|g| g.satir).unwrap_or(1);
        return Err(ic_hata(satir));
    }
    Ok(Akis::Devam)
    })
}
