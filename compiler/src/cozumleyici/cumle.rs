use super::*;

fn sembol_yazimi_kaydet(baglam: &mut Baglam, kimlik: SymbolId, ad: &str, kaynak_araligi: crate::hir::HirKaynakAraligi) {
    baglam.hir_sembol_adi_ekle(kimlik, ad.to_string());
    baglam.hir_sembol_yazimi_ekle(kimlik, kaynak_araligi);
}

pub(super) fn blok_denetle(cumleler: &mut [Cumle], ortam: &mut SembolTablosu, baglam: &mut Baglam) -> Result<(), Tani> {
    let giriste_bekleyenler = baglam.bekleyen_gorevler.clone();
    let mut acik_gorev_satiri = None;
    for cumle in cumleler.iter_mut() {
        let sonuc = (|| -> Result<(), Tani> {
        match cumle {
            Cumle::Yaz { deger, satir } => {
                let satir = *satir;
                ifade_denetle(deger, ortam, baglam, satir)?;
            }
            Cumle::Olsun {
                ad,
                deger,
                satir,
                sutun,
                uzunluk,
            } => {
                let satir = *satir;
                if baglam.gezilen_koleksiyonlar.contains(ad) {
                    return Err(gezilen_koleksiyonu_degistirme_tanisi(ad, satir));
                }
                if baglam.bekleyen_gorevler.contains(ad) {
                    return Err(Tani::yeni(
                        "T033",
                        format!(
                            "\"{}\" bir eşzamanlı görev sonucudur; `hepsini bekle`den önce yeniden atanamaz.",
                            ad
                        ),
                        satir,
                        *sutun,
                        *uzunluk,
                    )
                    .onerili("Önce `hepsini bekle`; sonra sonuç adına sıradan bir değer gibi eriş.".into()));
                }
                let mut tur = ifade_denetle(deger, ortam, baglam, satir)?;
                if let Some(eski) = ortam.get(ad.as_str()) {
                    // K-045: boş koleksiyon somut eşiyle iki yönde uzlaşır —
                    // "x boş liste olsun" sonrası somut liste (ve tersi) T002 değildir.
                    if let Some(uzlasi) = bos_koleksiyon_uzlasi(eski, &tur) {
                        tur = uzlasi;
                    } else if *eski != tur {
                        return Err(Tani::yeni(
                            "T002",
                            format!(
                                "\"{}\" daha önce {} olarak tanımlandı; şimdi {} verilemez.",
                                ad,
                                eski.adi(),
                                tur.adi()
                            ),
                            satir,
                            *sutun,
                            *uzunluk,
                        )
                        .onerili(format!(
                            "Bir değerin türü sonradan değişemez. Farklı türde bir değer \
                             gerekiyorsa yeni bir ad kullan; ya da \"{}\" değerine yine {} türünde \
                             bir değer ver.",
                            ad,
                            eski.adi()
                        )));
                    }
                }
                let kimlik = ortam.insert(ad.clone(), tur);
                let kaynak_araligi =
                    crate::hir::HirKaynakAraligi::kesin(satir, *sutun, *uzunluk).ok_or_else(|| hir_kaynak_hatasi(satir))?;
                sembol_yazimi_kaydet(baglam, kimlik, ad, kaynak_araligi);
            }
            Cumle::KezTekrarla { adet, govde, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(adet, ortam, baglam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T003",
                        format!("Tekrar adedi TamSayı olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                let kapsam = kapsam_baslat(ortam);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::AralikDongusu {
                ad,
                bastan,
                sona,
                govde,
                satir,
            } => {
                let satir = *satir;
                for uc in [&mut *bastan, &mut *sona] {
                    let tur = ifade_denetle(uc, ortam, baglam, satir)?;
                    if tur != Tur::TamSayi {
                        return Err(Tani::yeni(
                            "T004",
                            format!("Aralık uçları TamSayı olmalı; burada {} var.", tur.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                // Döngü değişkeni gövde kapsamındadır ve gövdeyle ölür (RFC-0004).
                let kapsam = kapsam_baslat(ortam);
                let kimlik = ortam.insert(ad.clone(), Tur::TamSayi);
                let kaynak_araligi = crate::hir::HirKaynakAraligi::satir(satir).ok_or_else(|| hir_kaynak_hatasi(satir))?;
                sembol_yazimi_kaydet(baglam, kimlik, ad, kaynak_araligi);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::OlduguSurece { kosul, govde, satir } | Cumle::OlanaKadar { kosul, govde, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(kosul, ortam, baglam, satir)?;
                if tur != Tur::Mantiksal {
                    return Err(Tani::yeni("T005", "Koşullu döngü bir koşul ister.".into(), satir, 1, 1));
                }
                let kapsam = kapsam_baslat(ortam);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::Ise { kollar, degilse, satir } => {
                let satir = *satir;
                let mut son_kol_daraltmasi: Option<(u8, String)> = None;
                for kol in kollar.iter_mut() {
                    let tur = ifade_denetle(&mut kol.kosul, ortam, baglam, satir)?;
                    if tur != Tur::Mantiksal {
                        return Err(Tani::yeni("T005", "\"ise\" bir koşul ister.".into(), satir, 1, 1));
                    }
                    // Daraltma çıkarımı (RFC-0008 §4.2).
                    let daraltma = daraltma_cikar(&kol.kosul);
                    son_kol_daraltmasi = daraltma.clone();
                    if let Some((tur_kodu, ad)) = &daraltma {
                        daraltma_ekle(baglam, *tur_kodu, ad);
                    }
                    let kapsam = kapsam_baslat(ortam);
                    let sonuc = blok_denetle(&mut kol.govde, ortam, baglam);
                    if let Some((tur_kodu, ad)) = &daraltma {
                        daraltma_cikar_geri(baglam, *tur_kodu, ad);
                    }
                    sonuc?;
                    kapsam_bitir(ortam, &kapsam);
                }
                if let Some(blok) = degilse {
                    // "X varsa ... değilse" dalında X boştur; "X yoksa ... değilse"
                    // ve "başarısızsa ... değilse" dallarında TERSİ daraltılır.
                    let ters = son_kol_daraltmasi
                        .filter(|_| kollar.len() == 1)
                        .and_then(|(kod, ad)| match kod {
                            1 => None,          // varsa'nın değilse'si: boş
                            2 => Some((1, ad)), // yoksa'nın değilse'si: dolu
                            3 => Some((4, ad)), // başarılıysa'nın değilse'si: başarısız
                            4 => Some((3, ad)), // başarısızsa'nın değilse'si: başarılı
                            _ => None,
                        });
                    if let Some((tur_kodu, ad)) = &ters {
                        daraltma_ekle(baglam, *tur_kodu, ad);
                    }
                    let kapsam = kapsam_baslat(ortam);
                    let sonuc = blok_denetle(blok, ortam, baglam);
                    if let Some((tur_kodu, ad)) = &ters {
                        daraltma_cikar_geri(baglam, *tur_kodu, ad);
                    }
                    sonuc?;
                    kapsam_bitir(ortam, &kapsam);
                }
            }
            Cumle::Ekle { hedef, deger, satir } => {
                let satir = *satir;
                gezilen_hedefi_denetle(hedef, baglam, satir)?;
                let hedef_tur = ifade_denetle(hedef, ortam, baglam, satir)?;
                let oge = match hedef_tur {
                    // K-045: boş listenin öğe türü ilk eklemeyle somutlaşır.
                    Tur::Liste(VeriTuru::Bilinmeyen) => {
                        let deger_tur = ifade_denetle(deger, ortam, baglam, satir)?;
                        let Some(yeni_oge) = veri_turu_yap(&deger_tur) else {
                            return Err(
                                Tani::yeni("T011", format!("Liste öğesi {} olamaz.", deger_tur.adi()), satir, 1, 1).onerili(
                                    "v0'da liste öğesi TamSayı, Ondalık, Metin ya da satır (Sözlük) olabilir.".into(),
                                ),
                            );
                        };
                        if let Some(ad) = nesne_adi(hedef) {
                            ortam.insert(ad, Tur::Liste(yeni_oge));
                        }
                        yeni_oge
                    }
                    Tur::Liste(oge) => oge,
                    baska => {
                        return Err(Tani::yeni(
                            "T012",
                            format!("Ekleme bir listeye yapılır; hedef {} türünde.", baska.adi()),
                            satir,
                            1,
                            1,
                        )
                        .onerili("Önce \"<ad> boş liste olsun\" ya da \"... listesi olsun\" ile liste tanımla.".into()));
                    }
                };
                let deger_tur = ifade_denetle(deger, ortam, baglam, satir)?;
                if deger_tur != oge.ture() {
                    return Err(Tani::yeni(
                        "T011",
                        format!("{} listesine {} eklenemez.", oge.adi(), deger_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::HerBiri {
                ad,
                kaynak,
                govde,
                satir,
            } => {
                let satir = *satir;
                if kaynak.is_none() {
                    // Örtük çoğul (K-013): "her sayı için" → kapsamda "sayılar" aranır.
                    let adaylar = [format!("{}lar", ad), format!("{}ler", ad)];
                    let bulunanlar: Vec<String> = adaylar
                        .iter()
                        .filter(|aday| matches!(ortam.get(aday.as_str()), Some(Tur::Liste(_)) | Some(Tur::Sozluk(_))))
                        .cloned()
                        .collect();
                    match bulunanlar.len() {
                        1 => {
                            let kaynak_adi = bulunanlar
                                .into_iter()
                                .next()
                                .ok_or_else(|| ic_tutarlilik_hatasi("Örtük çoğul adayı kayboldu", satir))?;
                            *kaynak = Some(Ifade::Degisken {
                                ham: kaynak_adi.clone(),
                                sembol_kimligi: ortam.kimlik(&kaynak_adi),
                                cozulmus: Some(kaynak_adi),
                                satir,
                                sutun: 1,
                                uzunluk: 1,
                            });
                        }
                        0 => {
                            return Err(Tani::yeni(
                                "A003",
                                format!(
                                    "\"her {} için\" gezilecek listeyi bulamadı: kapsamda \"{}lar\" ya da \"{}ler\" adında bir liste yok.",
                                    ad, ad, ad
                                ),
                                satir,
                                1,
                                1,
                            )
                            .onerili(format!("Önce listeyi tanımla: {}lar 1, 2, 3 listesi olsun", ad)))
                        }
                        _ => {
                            return Err(Tani::yeni(
                                "A002",
                                format!("\"her {} için\" iki listeye birden çözülebiliyor.", ad),
                                satir,
                                1,
                                1,
                            ))
                        }
                    }
                }
                let oge_turu = match kaynak {
                    Some(k) => match ifade_denetle(k, ortam, baglam, satir)? {
                        Tur::Liste(VeriTuru::Bilinmeyen) => {
                            return Err(
                                Tani::yeni("T013", "Bu liste henüz boş: öğe türü belli değil.".into(), satir, 1, 1)
                                    .onerili("Gezmeden önce listeye en az bir öğe ekle.".into()),
                            );
                        }
                        Tur::Liste(oge) => oge.ture(),
                        // Sözlük üzerinde gezinme anahtarları (Metin) verir.
                        Tur::Sozluk(_) => Tur::Metin,
                        baska => {
                            return Err(Tani::yeni(
                                "T013",
                                format!("\"her ... için\" bir liste ya da sözlük ister; burada {} var.", baska.adi()),
                                satir,
                                1,
                                1,
                            ));
                        }
                    },
                    None => {
                        return Err(ic_tutarlilik_hatasi("Gezme kaynağı çözümlenmeden kaldı", satir));
                    }
                };
                let kaynak_adi = kaynak
                    .as_ref()
                    .and_then(nesne_adi)
                    .ok_or_else(|| ic_tutarlilik_hatasi("Gezme kaynağı çözülmüş bir ad değil", satir))?;
                if kaynak_adi == *ad || baglam.gezilen_koleksiyonlar.contains(&kaynak_adi) {
                    return Err(gezilen_koleksiyonu_degistirme_tanisi(&kaynak_adi, satir));
                }
                let kapsam = kapsam_baslat(ortam);
                let kimlik = ortam.insert(ad.clone(), oge_turu);
                let kaynak_araligi = crate::hir::HirKaynakAraligi::satir(satir).ok_or_else(|| hir_kaynak_hatasi(satir))?;
                sembol_yazimi_kaydet(baglam, kimlik, ad, kaynak_araligi);
                baglam.gezilen_koleksiyonlar.insert(kaynak_adi.clone());
                let sonuc = blok_denetle(govde, ortam, baglam);
                baglam.gezilen_koleksiyonlar.remove(&kaynak_adi);
                sonuc?;
                kapsam_bitir(ortam, &kapsam);
            }
            Cumle::ProgramiBitir { kod, satir } => {
                bekleyen_gorev_olmadigini_denetle(
                    baglam,
                    &giriste_bekleyenler,
                    *satir,
                    "Program bitmeden önce görevleri bekle.",
                )?;
                if let Some(kod) = kod {
                    let tur = ifade_denetle(kod, ortam, baglam, *satir)?;
                    if tur != Tur::TamSayi {
                        return Err(Tani::yeni(
                            "T034",
                            format!("Çıkış kodu TamSayı olmalı; burada {} var.", tur.adi()),
                            *satir,
                            1,
                            1,
                        ));
                    }
                }
            }
            Cumle::SunucuBaslat { kapi, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(kapi, ortam, baglam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T034",
                        format!("Kapı numarası TamSayı olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::IstekGeldiginde { yol, govde, satir, .. } => {
                let satir = *satir;
                let tur = ifade_denetle(yol, ortam, baglam, satir)?;
                if tur != Tur::Metin {
                    return Err(Tani::yeni(
                        "T034",
                        format!("İstek yolu Metin olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                // Her istek taze ortamda işlenir (kapsülleme); form ve sorgu
                // verisi örtük "istek" sözlüğünde gelir (K-051).
                let mut istek_ortami = SembolTablosu::yeni(baglam.istek_kapsami(satir));
                istek_ortami.insert("istek".into(), Tur::Sozluk(SozlukDegerTuru::Metin));
                istek_ortami.insert("çerezler".into(), Tur::Sozluk(SozlukDegerTuru::Metin));
                blok_denetle(govde, &mut istek_ortami, baglam)?;
            }
            Cumle::YanitGonder { deger, satir } => {
                let satir = *satir;
                ifade_denetle(deger, ortam, baglam, satir)?;
            }
            Cumle::Sil { kap, deger, satir } => {
                let satir = *satir;
                gezilen_hedefi_denetle(kap, baglam, satir)?;
                let kap_turu = ifade_denetle(kap, ortam, baglam, satir)?;
                let deger_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                match kap_turu {
                    Tur::Liste(oge) if oge != VeriTuru::Bilinmeyen => {
                        if deger_turu != oge.ture() {
                            return Err(Tani::yeni(
                                "T011",
                                format!("{} listesinden {} silinemez.", oge.adi(), deger_turu.adi()),
                                satir,
                                1,
                                1,
                            ));
                        }
                    }
                    Tur::Sozluk(_) => {
                        if deger_turu != Tur::Metin {
                            return Err(Tani::yeni(
                                "T021",
                                "Sözlükten silme anahtarla (Metin) yapılır.".into(),
                                satir,
                                1,
                                1,
                            ));
                        }
                    }
                    baska => {
                        return Err(Tani::yeni(
                            "T012",
                            format!("Silme bir liste ya da sözlük ister; hedef {} türünde.", baska.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
            }
            Cumle::CerezSil { ad, satir } => {
                let satir = *satir;
                if ifade_denetle(ad, ortam, baglam, satir)? != Tur::Metin {
                    return Err(Tani::yeni("T034", "Çerez adı Metin olmalı.".into(), satir, 1, 1));
                }
            }
            Cumle::CerezYaz { ad, deger, satir } => {
                let satir = *satir;
                let ad_turu = ifade_denetle(ad, ortam, baglam, satir)?;
                let deger_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                if ad_turu != Tur::Metin || deger_turu != Tur::Metin {
                    return Err(Tani::yeni("T034", "Çerez adı ve değeri Metin olmalı.".into(), satir, 1, 1));
                }
            }
            Cumle::RotaPolitikasi { .. } | Cumle::RotaAlaniGerekli { .. } => {}
            Cumle::OturumAc { kullanici, rol, satir } => {
                let satir = *satir;
                let kullanici_turu = ifade_denetle(kullanici, ortam, baglam, satir)?;
                let rol_turu = ifade_denetle(rol, ortam, baglam, satir)?;
                if kullanici_turu != Tur::Metin || rol_turu != Tur::Metin {
                    return Err(Tani::yeni(
                        "T034",
                        "Oturum kullanıcısı ve rolü Metin olmalı.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::OturumKapat { .. } => {}
            Cumle::Yonlendir { adres, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(adres, ortam, baglam, satir)?;
                if tur != Tur::Metin {
                    return Err(Tani::yeni(
                        "T034",
                        format!("Yönlendirme adresi Metin olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::Eszamanli { gorevler, satir } => {
                let satir = *satir;
                if baglam.bekleyen_gorevler != giriste_bekleyenler {
                    return Err(gorev_kapsami_tanisi(
                        satir,
                        "Yeni bir eşzamanlı grup açmadan önce mevcut görevleri bekle.",
                    ));
                }
                acik_gorev_satiri = Some(satir);
                for (ad, deger, gorev_satiri) in gorevler.iter_mut() {
                    let tur = ifade_denetle(deger, ortam, baglam, *gorev_satiri)?;
                    if baglam.bekleyen_gorevler.contains(ad) {
                        return Err(gorev_kapsami_tanisi(
                            *gorev_satiri,
                            &format!("\"{}\" adı aynı görev grubunda iki kez kullanılamaz.", ad),
                        ));
                    }
                    if let Some(eski) = ortam.get(ad.as_str()) {
                        if *eski != tur {
                            return Err(Tani::yeni(
                                "T002",
                                format!("\"{}\" daha önce {} türündeydi.", ad, eski.adi()),
                                satir,
                                1,
                                1,
                            ));
                        }
                    }
                    let kimlik = ortam.insert(ad.clone(), tur);
                    let kaynak_araligi = crate::hir::HirKaynakAraligi::satir(*gorev_satiri)
                        .ok_or_else(|| hir_kaynak_hatasi(*gorev_satiri))?;
                    sembol_yazimi_kaydet(baglam, kimlik, ad, kaynak_araligi);
                    baglam.bekleyen_gorevler.insert(ad.clone());
                }
            }
            Cumle::HepsiniBekle { satir } => {
                let yerel = baglam
                    .bekleyen_gorevler
                    .difference(&giriste_bekleyenler)
                    .cloned()
                    .collect::<Vec<_>>();
                if yerel.is_empty() {
                    return Err(gorev_kapsami_tanisi(
                        *satir,
                        "Bu kapsamda beklenebilecek açık bir görev grubu yok.",
                    ));
                }
                baglam.bekleyen_gorevler.retain(|ad| giriste_bekleyenler.contains(ad));
                acik_gorev_satiri = None;
            }
            Cumle::IcindeBlogu {
                sure,
                govde,
                yetismezse,
                satir,
            } => {
                let satir = *satir;
                let tur = ifade_denetle(sure, ortam, baglam, satir)?;
                if tur != Tur::Sure {
                    return Err(Tani::yeni(
                        "T034",
                        format!("\"içinde\" bir Süre ister; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Örnek: 5 saniye içinde".into()));
                }
                let kapsam = kapsam_baslat(ortam);
                blok_denetle(govde, ortam, baglam)?;
                kapsam_bitir(ortam, &kapsam);
                if let Some(blok) = yetismezse {
                    let kapsam = kapsam_baslat(ortam);
                    blok_denetle(blok, ortam, baglam)?;
                    kapsam_bitir(ortam, &kapsam);
                }
            }
            Cumle::IsikAyarla { .. } => {}
            Cumle::Bekle { sure, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(sure, ortam, baglam, satir)?;
                if tur != Tur::Sure {
                    return Err(Tani::yeni(
                        "T034",
                        format!("\"bekle\" bir Süre ister; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Örnek: yarım saniye bekle".into()));
                }
            }
            Cumle::Sor { istem, satir } => {
                let satir = *satir;
                ifade_denetle(istem, ortam, baglam, satir)?;
                // Son cevap örtük "yanıt" adına Metin olarak bağlanır (K-007).
                ortam.insert("yanıt".to_string(), Tur::Metin);
            }
            Cumle::Gore {
                konu,
                kollar,
                degilse,
                satir,
            } => {
                let satir = *satir;
                let konu_turu = ifade_denetle(konu, ortam, baglam, satir)?;
                if konu_turu.veri_turu().is_none() {
                    return Err(Tani::yeni(
                        "T026",
                        format!(
                            "\"göre\" eşleştirmesi TamSayı ya da Metin ister; burada {} var.",
                            konu_turu.adi()
                        ),
                        satir,
                        1,
                        1,
                    ));
                }
                for (deger, govde) in kollar.iter_mut() {
                    let kol_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                    if kol_turu != konu_turu {
                        return Err(Tani::yeni(
                            "T026",
                            format!(
                                "Eşleştirme kolu {} olmalı ({} ile karşılaştırılıyor); burada {} var.",
                                konu_turu.adi(),
                                konu_turu.adi(),
                                kol_turu.adi()
                            ),
                            satir,
                            1,
                            1,
                        ));
                    }
                    let kapsam = kapsam_baslat(ortam);
                    blok_denetle(govde, ortam, baglam)?;
                    kapsam_bitir(ortam, &kapsam);
                }
                if let Some(blok) = degilse {
                    let kapsam = kapsam_baslat(ortam);
                    blok_denetle(blok, ortam, baglam)?;
                    kapsam_bitir(ortam, &kapsam);
                }
            }
            Cumle::IslemTanimi(islem) => {
                // Hoist sonrası burada görünmemeli.
                return Err(Tani::yeni(
                    "S021",
                    format!("\"{}\" işlem tanımı beklenmeyen yerde.", islem.ad),
                    islem.satir,
                    1,
                    1,
                ));
            }
            Cumle::YapiTanimi(yapi) => {
                return Err(Tani::yeni(
                    "S021",
                    format!("\"{}\" yapı tanımı beklenmeyen yerde.", yapi.ad),
                    yapi.satir,
                    1,
                    1,
                ));
            }
            Cumle::Kullan { ad, tur, satir } => {
                // Birimler derleme öncesi çözülüp hoist'te düşürülür.
                let tur_adi = match tur {
                    crate::agac::KullanimTuru::Birim => "birim",
                    crate::agac::KullanimTuru::Paket => "paket",
                };
                return Err(Tani::yeni(
                    "S021",
                    format!("\"{}\" {} kullanımı beklenmeyen yerde.", ad, tur_adi),
                    *satir,
                    1,
                    1,
                ));
            }
            Cumle::TestBlogu(test) => {
                return Err(Tani::yeni(
                    "S021",
                    format!("\"{}\" test bloğu beklenmeyen yerde.", test.ad),
                    test.satir,
                    1,
                    1,
                ));
            }
            Cumle::Olmali { kosul, satir } => {
                let satir = *satir;
                let tur = ifade_denetle(kosul, ortam, baglam, satir)?;
                if tur != Tur::Mantiksal {
                    return Err(Tani::yeni("T005", "\"olmalı\" bir koşul ister.".into(), satir, 1, 1)
                        .onerili("Örnek: kare 16 ya eşit olmalı".into()));
                }
            }
            Cumle::AlanAta {
                nesne,
                alan,
                deger,
                satir,
            } => {
                let satir = *satir;
                let nesne_turu = ifade_denetle(nesne, ortam, baglam, satir)?;
                let yapi_kimligi = match nesne_turu {
                    Tur::Yapi(kimlik) => kimlik,
                    baska => {
                        return Err(Tani::yeni(
                            "T028",
                            format!("Alan yazma bir yapı ister; burada {} var.", baska.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                };
                let yapi = baglam
                    .yapi(yapi_kimligi)
                    .ok_or_else(|| ic_tutarlilik_hatasi("Yapı kimliği dizinde kayıtlı değil", satir))?
                    .clone();
                let yalin = alan_cozumle(&yapi, alan, satir)?;
                let beklenen = yapi
                    .alanlar
                    .iter()
                    .find(|(a, _)| *a == yalin)
                    .and_then(|(_, t)| alan_turu(t))
                    .ok_or_else(|| ic_tutarlilik_hatasi("Çözülmüş alanın türü bulunamadı", satir))?;
                let deger_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                if deger_turu != beklenen {
                    return Err(Tani::yeni(
                        "T028",
                        format!(
                            "\"{}\" alanı {} türünde; {} verilemez.",
                            yalin,
                            beklenen.adi(),
                            deger_turu.adi()
                        ),
                        satir,
                        1,
                        1,
                    ));
                }
                *alan = yalin;
            }
            Cumle::Dondur { deger, satir, .. } => {
                let satir = *satir;
                bekleyen_gorev_olmadigini_denetle(
                    baglam,
                    &giriste_bekleyenler,
                    satir,
                    "Değer döndürmeden önce görevleri bekle.",
                )?;
                let tur = ifade_denetle(deger, ortam, baglam, satir)?;
                match baglam.denetim_yigini.last_mut() {
                    Some(kayit) => kayit.donusler.push(tur),
                    None => {
                        return Err(Tani::yeni(
                            "T020",
                            "\"döndür\" yalnız bir işlemin içinde kullanılır.".into(),
                            satir,
                            1,
                            1,
                        ))
                    }
                }
            }
            Cumle::HataDondur {
                kod,
                mesaj,
                neden,
                veri,
                satir,
            } => {
                let satir = *satir;
                bekleyen_gorev_olmadigini_denetle(
                    baglam,
                    &giriste_bekleyenler,
                    satir,
                    "Hata döndürmeden önce görevleri bekle.",
                )?;
                let tur = ifade_denetle(mesaj, ortam, baglam, satir)?;
                let yeniden_yayma = kod.is_none() && tur == Tur::Hata;
                if tur != Tur::Metin && !yeniden_yayma {
                    return Err(Tani::yeni(
                        "T032",
                        format!(
                            "Hata mesajı Metin, yeniden yayılan değer Hata olmalı; burada {} var.",
                            tur.adi()
                        ),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Örnek: \"sıfıra bölünmez\" hatasını döndür".into()));
                }
                if yeniden_yayma && (neden.is_some() || veri.is_some()) {
                    return Err(Tani::yeni(
                        "T052",
                        "Yeniden yayılan Hata'ya neden/veri eklenemez; yeni kodlu bir hata ile sar.".into(),
                        satir,
                        1,
                        1,
                    )
                    .onerili(
                        "Örnek: \"UST_HATA\" kodlu \"İşlem tamamlanamadı\" hatasını eski_hata nedeniyle döndür".into(),
                    ));
                }
                if let Some(neden) = neden {
                    let neden_turu = ifade_denetle(neden, ortam, baglam, satir)?;
                    if neden_turu != Tur::Hata {
                        return Err(Tani::yeni(
                            "T052",
                            format!("Hatanın nedeni Hata olmalı; burada {} var.", neden_turu.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                if let Some(veri) = veri {
                    let veri_turu = ifade_denetle(veri, ortam, baglam, satir)?;
                    if !matches!(veri_turu, Tur::Sozluk(SozlukDegerTuru::Metin | SozlukDegerTuru::Bilinmeyen)) {
                        return Err(Tani::yeni(
                            "T052",
                            format!("Hatanın verisi Metin sözlüğü olmalı; burada {} var.", veri_turu.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                match baglam.denetim_yigini.last_mut() {
                    Some(kayit) => kayit.donusler.push(Tur::HataDonusu),
                    None => {
                        return Err(Tani::yeni(
                            "T020",
                            "\"hatasını döndür\" yalnız bir işlemin içinde kullanılır.".into(),
                            satir,
                            1,
                            1,
                        ))
                    }
                }
            }
            Cumle::BolVeAta {
                hedef,
                pay,
                payda,
                satir,
            } => {
                let satir = *satir;
                let mut ondalik_var = false;
                for taraf in [&mut *pay, &mut *payda] {
                    let tur = ifade_denetle(taraf, ortam, baglam, satir)?;
                    if !tur.sayisal() {
                        return Err(Tani::yeni(
                            "T008",
                            format!("Bölme sayılar arasında yapılır; burada {} var.", tur.adi()),
                            satir,
                            1,
                            1,
                        ));
                    }
                    ondalik_var |= tur == Tur::Ondalik;
                }
                // İki TamSayı → tam bölme (mevcut davranış); Ondalık karışımı → Ondalık.
                let sonuc_turu = if ondalik_var { Tur::Ondalik } else { Tur::TamSayi };
                if let Some(eski) = ortam.get(hedef.as_str()) {
                    if *eski != sonuc_turu {
                        return Err(Tani::yeni(
                            "T002",
                            format!(
                                "\"{}\" {} türünde; {} bölme sonucu verilemez.",
                                hedef,
                                eski.adi(),
                                sonuc_turu.adi()
                            ),
                            satir,
                            1,
                            1,
                        ));
                    }
                }
                let kimlik = ortam.insert(hedef.clone(), sonuc_turu);
                let kaynak_araligi = crate::hir::HirKaynakAraligi::satir(satir).ok_or_else(|| hir_kaynak_hatasi(satir))?;
                sembol_yazimi_kaydet(baglam, kimlik, hedef, kaynak_araligi);
            }
            Cumle::SozlukAta {
                sozluk,
                anahtar,
                deger,
                satir,
            } => {
                let satir = *satir;
                gezilen_hedefi_denetle(sozluk, baglam, satir)?;
                let sozluk_turu = ifade_denetle(sozluk, ortam, baglam, satir)?;
                let beklenen_deger = match sozluk_turu {
                    // K-045: boş sözlüğün değer türü ilk atamayla somutlaşır.
                    Tur::Sozluk(SozlukDegerTuru::Bilinmeyen) => {
                        let deger_tur = ifade_denetle(deger, ortam, baglam, satir)?;
                        let yeni_deger = match deger_tur {
                            Tur::TamSayi => SozlukDegerTuru::TamSayi,
                            Tur::Ondalik => SozlukDegerTuru::Ondalik,
                            Tur::Metin => SozlukDegerTuru::Metin,
                            baska => {
                                return Err(Tani::yeni(
                                    "T021",
                                    format!("Sözlük değeri {} olamaz.", baska.adi()),
                                    satir,
                                    1,
                                    1,
                                )
                                .onerili("Sözlük değeri TamSayı, Ondalık ya da Metin olabilir.".into()));
                            }
                        };
                        if let Some(ad) = nesne_adi(sozluk) {
                            ortam.insert(ad, Tur::Sozluk(yeni_deger));
                        }
                        yeni_deger.ture()
                    }
                    Tur::Sozluk(e) => e.ture(),
                    baska => {
                        return Err(Tani::yeni(
                            "T021",
                            format!("\"değeri ... olsun\" bir sözlük ister; hedef {} türünde.", baska.adi()),
                            satir,
                            1,
                            1,
                        )
                        .onerili("Önce \"<ad> boş sözlük olsun\" ile sözlük tanımla.".into()));
                    }
                };
                let anahtar_turu = ifade_denetle(anahtar, ortam, baglam, satir)?;
                if anahtar_turu != Tur::Metin {
                    return Err(Tani::yeni("T021", "v0'da sözlük anahtarı Metin olmalı.".into(), satir, 1, 1));
                }
                let deger_turu = ifade_denetle(deger, ortam, baglam, satir)?;
                if deger_turu != beklenen_deger {
                    return Err(Tani::yeni(
                        "T021",
                        format!(
                            "Bu sözlük {} değerler taşıyor; {} verilemez.",
                            beklenen_deger.adi(),
                            deger_turu.adi()
                        ),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Cumle::DosyayaYaz { yol, icerik, satir, .. } => {
                let satir = *satir;
                let yol_turu = ifade_denetle(yol, ortam, baglam, satir)?;
                if yol_turu != Tur::Metin {
                    return Err(Tani::yeni(
                        "T025",
                        format!("Dosya yolu Metin olmalı; burada {} var.", yol_turu.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                ifade_denetle(icerik, ortam, baglam, satir)?;
            }
            Cumle::CagriCumlesi { cagri, satir } => {
                let satir = *satir;
                if let Ifade::IslemCagrisi {
                    islem_adi,
                    islem_kimligi,
                    argumanlar,
                    ..
                } = cagri
                {
                    *islem_kimligi = baglam.islem_kimligi(islem_adi);
                    let mut arg_turleri = Vec::new();
                    for arg in argumanlar.iter_mut() {
                        arg_turleri.push(ifade_denetle(arg, ortam, baglam, satir)?);
                    }
                    // Cümle konumunda dönüş değeri kullanılmaz; HIR yine de
                    // değer/dönüşsüz sözleşmesini açıkça kaydeder.
                    let donus = cagri_denetle(islem_adi, &arg_turleri, baglam, satir)?;
                    let kimlik = (*islem_kimligi).ok_or_else(|| {
                        Tani::yeni("T016", "İşlem çağrısının semantic kimliği kurulamadı.".into(), satir, 1, 1)
                    })?;
                    let hir_turu = donus
                        .map(crate::hir::HirIfadeTuru::Deger)
                        .unwrap_or(crate::hir::HirIfadeTuru::DegerDondurmez);
                    let kaynak_araligi =
                        crate::hir::HirKaynakAraligi::satir(satir).ok_or_else(|| hir_kaynak_hatasi(satir))?;
                    hir_ifadesi_kaydet(
                        baglam,
                        crate::hir::ifade_adresi(cagri),
                        hir_turu,
                        crate::hir::HirBagi::Islem(kimlik),
                        kaynak_araligi,
                    );
                } else {
                    return Err(Tani::yeni("S004", "Geçersiz çağrı cümlesi.".into(), satir, 1, 1));
                }
            }
            Cumle::Artir { ifade, miktar, satir } | Cumle::Azalt { ifade, miktar, satir } => {
                let satir = *satir;
                let hedef_tur = ifade_denetle(ifade, ortam, baglam, satir)?;
                if !hedef_tur.sayisal() {
                    return Err(Tani::yeni(
                        "T006",
                        format!("Artırma/azaltma sayı ister; hedef {} türünde.", hedef_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                let miktar_tur = ifade_denetle(miktar, ortam, baglam, satir)?;
                if !miktar_tur.sayisal() {
                    return Err(Tani::yeni(
                        "T006",
                        format!("Artırma/azaltma miktarı sayı olmalı; burada {} var.", miktar_tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                if hedef_tur == Tur::TamSayi && miktar_tur == Tur::Ondalik {
                    return Err(Tani::yeni(
                        "T006",
                        "TamSayı hedefe Ondalık miktar eklenemez: sonuç tam sayı kalamaz.".into(),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Hedefi ondalık başlat (örn. 0,0 olsun) ya da miktarı tam sayı yap.".into()));
                }
            }
        }
        Ok(())
        })();
        if let Err(tani) = sonuc {
            if !baglam.cikarim_kesfi {
                return Err(tani);
            }
        }
    }
    bekleyen_gorev_olmadigini_denetle(
        baglam,
        &giriste_bekleyenler,
        acik_gorev_satiri.unwrap_or(1),
        "Eşzamanlı görev grubu kapsamdan çıkmadan önce `hepsini bekle` yaz.",
    )?;
    Ok(())
}
