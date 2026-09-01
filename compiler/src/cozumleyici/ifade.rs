use super::*;

pub(super) fn ifade_denetle(
    ifade: &mut Ifade,
    ortam: &SembolTablosu,
    baglam: &mut Baglam,
    satir: usize,
) -> Result<Tur, Tani> {
    let adres = crate::hir::ifade_adresi(ifade);
    let kaynak_araligi = crate::hir::HirKaynakAraligi::ifadeden(ifade, satir)
        .ok_or_else(|| hir_kaynak_hatasi(satir))?;
    let tur = ifade_denetle_ic(ifade, ortam, baglam, satir)?;
    let bag = match ifade {
        Ifade::Degisken {
            sembol_kimligi: Some(kimlik),
            cozulmus: Some(ad),
            ..
        } => {
            baglam.hir_sembol_adi_ekle(*kimlik, ad.clone());
            crate::hir::HirBagi::Sembol(*kimlik)
        }
        Ifade::IslemCagrisi { islem_kimligi: Some(kimlik), .. } => {
            crate::hir::HirBagi::Islem(*kimlik)
        }
        Ifade::YeniYapi { yapi_kimligi: Some(kimlik), .. } => {
            crate::hir::HirBagi::Yapi(*kimlik)
        }
        _ => crate::hir::HirBagi::Yok,
    };
    hir_ifadesi_kaydet(
        baglam,
        adres,
        crate::hir::HirIfadeTuru::Deger(tur),
        bag,
        kaynak_araligi,
    );
    Ok(tur)
}

pub(super) fn hir_ifadesi_kaydet(
    baglam: &mut Baglam,
    adres: usize,
    tur: crate::hir::HirIfadeTuru,
    bag: crate::hir::HirBagi,
    kaynak_araligi: crate::hir::HirKaynakAraligi,
) {
    let hir_kimligi = baglam
        .hir_ifadeleri
        .get(&adres)
        .map(|bilgi| bilgi.kimlik())
        .unwrap_or_else(|| crate::hir::HirDugumId::yeni(baglam.hir_ifadeleri.len()));
    baglam
        .hir_ifadeleri
        .insert(
            adres,
            crate::hir::HirIfadeBilgisi::yeni_turle(hir_kimligi, tur, bag, kaynak_araligi),
        );
}

fn ifade_denetle_ic(
    ifade: &mut Ifade,
    ortam: &SembolTablosu,
    baglam: &mut Baglam,
    satir: usize,
) -> Result<Tur, Tani> {
    match ifade {
        Ifade::MetinSabiti(_) => Ok(Tur::Metin),
        Ifade::SayiSabiti(_) => Ok(Tur::TamSayi),
        Ifade::OndalikSabiti { .. } => Ok(Tur::Ondalik),
        Ifade::MantiksalSabiti(_) => Ok(Tur::Mantiksal),
        Ifade::Intrinsic { kimlik, argumanlar } => {
            let Some(tanim) = intrinsic::tanim(kimlik) else {
                return Err(Tani::yeni(
                    "T034",
                    format!("Derleyici \"{}\" iç işlemini tanımıyor.", kimlik),
                    satir,
                    1,
                    1,
                ));
            };
            if argumanlar.len() != tanim.arguman_turleri.len() {
                return Err(Tani::yeni("T034", tanim.tur_hatasi.into(), satir, 1, 1));
            }
            for (arguman, beklenen) in argumanlar.iter_mut().zip(tanim.arguman_turleri) {
                let bulunan = ifade_denetle(arguman, ortam, baglam, satir)?;
                if bulunan != intrinsic_turunu_cevir(*beklenen) {
                    let mesaj = if kimlik == intrinsic::HTTP_GETIR {
                        format!("Adres Metin olmalı; burada {} var.", bulunan.adi())
                    } else {
                        tanim.tur_hatasi.into()
                    };
                    return Err(Tani::yeni("T034", mesaj, satir, 1, 1));
                }
            }
            Ok(intrinsic_turunu_cevir(tanim.donus_turu))
        }
        // Boş listenin öğe türü v0'da TamSayı varsayılır (tür çıkarımı RFC-0007).
        // K-045: öğe türü ilk eklemede somutlaşır.
        Ifade::BosListe => Ok(Tur::Liste(VeriTuru::Bilinmeyen)),
        Ifade::ListeSabiti(ogeler) => {
            let mut oge_turu: Option<VeriTuru> = None;
            for oge in ogeler {
                let tur = ifade_denetle(oge, ortam, baglam, satir)?;
                let veri = tur.veri_turu().ok_or_else(|| {
                    Tani::yeni(
                        "T011",
                        format!("Liste öğesi TamSayı ya da Metin olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                })?;
                match oge_turu {
                    None => oge_turu = Some(veri),
                    Some(onceki) if onceki != veri => {
                        // TamSayı + Ondalık karışımı Ondalık'a genişler (RFC-0013 §2);
                        // yorumlayıcı öğeleri gerçekten genişletir.
                        let sayisal_karisim = matches!(
                            (onceki, veri),
                            (VeriTuru::TamSayi, VeriTuru::Ondalik)
                                | (VeriTuru::Ondalik, VeriTuru::TamSayi)
                        );
                        if sayisal_karisim {
                            oge_turu = Some(VeriTuru::Ondalik);
                        } else {
                            return Err(Tani::yeni(
                                "T011",
                                "Bir listenin bütün öğeleri aynı türden olmalı.".into(),
                                satir,
                                1,
                                1,
                            ));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Tur::Liste(oge_turu.unwrap_or(VeriTuru::TamSayi)))
        }
        Ifade::Ozellik { nesne, ozellik } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            // K-064: nesne bir yapıysa ve özellik kelimesi bir ALANA çözülüyorsa
            // bu aslında alan erişimidir ("ürünün adedi" → alan "adet").
            // Örtük-çoğul emsalindeki gibi ifade yeniden yazılır.
            if let Tur::Yapi(yapi_kimligi) = tur {
                let soz = match ozellik {
                    Ozellik::Adet => "adedi",
                    Ozellik::Ilk => "ilki",
                    Ozellik::Son => "sonu",
                    Ozellik::Uzunluk => "uzunluğu",
                    Ozellik::Kelimeler => "kelimeleri",
                    Ozellik::Yil => "yılı",
                    Ozellik::TamKisim => "kısmı",
                    Ozellik::Yuvarlanmis => "yuvarlanmışı",
                    Ozellik::HtmlGuvenli => "güvenlisi",
                    Ozellik::Kirpilmis => "kırpılmışı",
                    Ozellik::Harfler => "harfleri",
                    Ozellik::JsonMetin => "metni",
                    Ozellik::Siralanmis => "sıralanmışı",
                    Ozellik::Ters => "tersi",
                    Ozellik::CsvMetin => "metni",
                    Ozellik::Kuruslu => "kuruşlusu",
                    Ozellik::Metni => "metni",
                    Ozellik::BinlikliKuruslu => "kuruşlusu",
                    Ozellik::HataKodu => "kodu",
                    Ozellik::HataMesaji => "mesajı",
                    Ozellik::HataNedeni => "nedeni",
                    Ozellik::HataVerisi => "verisi",
                };
                let yapi = baglam.yapi(yapi_kimligi).expect("yapı kimliği dizinde kayıtlı");
                if let Ok(alan) = alan_cozumle(yapi, soz, satir) {
                    let yeni = Ifade::AlanErisim {
                        nesne: nesne.clone(),
                        alan: alan.clone(),
                    };
                    *ifade = yeni;
                    return ifade_denetle(ifade, ortam, baglam, satir);
                }
            }
            match (ozellik, tur) {
                (Ozellik::Adet, Tur::Liste(_)) => Ok(Tur::TamSayi),
                (Ozellik::Ilk, Tur::Liste(VeriTuru::Bilinmeyen))
                | (Ozellik::Son, Tur::Liste(VeriTuru::Bilinmeyen)) => Err(Tani::yeni(
                    "T014",
                    "Bu liste henüz boş: ilki/sonu yok.".into(),
                    satir,
                    1,
                    1,
                )
                .onerili("Önce listeye öğe ekle.".into())),
                (Ozellik::Ilk, Tur::Liste(e)) | (Ozellik::Son, Tur::Liste(e)) => Ok(e.ture()),
                (Ozellik::Uzunluk, Tur::Metin) => Ok(Tur::TamSayi),
                (Ozellik::HtmlGuvenli, Tur::Metin) => Ok(Tur::Metin),
                (Ozellik::Kirpilmis, Tur::Metin) => Ok(Tur::Metin),
                (Ozellik::Siralanmis, Tur::Liste(oge))
                    if matches!(oge, VeriTuru::TamSayi | VeriTuru::Ondalik | VeriTuru::Metin) =>
                {
                    Ok(Tur::Liste(oge))
                }
                (Ozellik::Ters, Tur::Liste(oge)) if oge != VeriTuru::Bilinmeyen => {
                    Ok(Tur::Liste(oge))
                }
                (Ozellik::Kuruslu, Tur::Ondalik)
                | (Ozellik::Kuruslu, Tur::TamSayi)
                | (Ozellik::BinlikliKuruslu, Tur::Ondalik)
                | (Ozellik::BinlikliKuruslu, Tur::TamSayi) => Ok(Tur::Metin),
                // K-066: her değerin resmî metin hali (yaz ile aynı temsil).
                (Ozellik::Metni, _) => Ok(Tur::Metin),
                (Ozellik::CsvMetin, Tur::Liste(VeriTuru::Sozluk))
                | (Ozellik::CsvMetin, Tur::Liste(VeriTuru::MetinSozluk)) => Ok(Tur::Metin),
                (Ozellik::Harfler, Tur::Metin) => Ok(Tur::Liste(VeriTuru::Metin)),
                (Ozellik::JsonMetin, Tur::Sozluk(_))
                | (Ozellik::JsonMetin, Tur::Liste(_))
                | (Ozellik::JsonMetin, Tur::Metin)
                | (Ozellik::JsonMetin, Tur::TamSayi)
                | (Ozellik::JsonMetin, Tur::Ondalik)
                | (Ozellik::JsonMetin, Tur::Mantiksal)
                | (Ozellik::JsonMetin, Tur::Hata) => Ok(Tur::Metin),
                (Ozellik::Kelimeler, Tur::Metin) => Ok(Tur::Liste(VeriTuru::Metin)),
                (Ozellik::Yil, Tur::Tarih) => Ok(Tur::TamSayi),
                (Ozellik::TamKisim, Tur::Ondalik) | (Ozellik::Yuvarlanmis, Tur::Ondalik) => {
                    Ok(Tur::TamSayi)
                }
                (Ozellik::HataKodu, Tur::Hata) | (Ozellik::HataMesaji, Tur::Hata) => {
                    Ok(Tur::Metin)
                }
                (Ozellik::HataNedeni, Tur::Hata) => Ok(Tur::Secenek(VeriTuru::Hata)),
                (Ozellik::HataVerisi, Tur::Hata) => {
                    Ok(Tur::Sozluk(SozlukDegerTuru::Metin))
                }
                (_, baska) => Err(Tani::yeni(
                    "T014",
                    format!("Bu özellik {} türüne uygulanamaz.", baska.adi()),
                    satir,
                    1,
                    1,
                )
                .onerili(
                    "adedi/ilki/sonu listeler, uzunluğu/kelimeleri metinler, tam kısmı/yuvarlanmışı ondalıklar içindir.".into(),
                )),
            }
        }
        // K-045: değer türü ilk atamada somutlaşır (eski varsayım TamSayı idi).
        Ifade::BosSozluk => Ok(Tur::Sozluk(SozlukDegerTuru::Bilinmeyen)),
        Ifade::SozlukDegeri { sozluk, anahtar } => {
            let sozluk_turu = ifade_denetle(sozluk, ortam, baglam, satir)?;
            let deger_turu = match sozluk_turu {
                Tur::Sozluk(SozlukDegerTuru::Bilinmeyen) => {
                    return Err(Tani::yeni(
                        "T021",
                        "Bu sözlük henüz boş: değer türü belli değil.".into(),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Önce bir değer ata: sözlüğün \"anahtar\" değeri ... olsun".into()));
                }
                Tur::Sozluk(e) => e,
                baska => {
                    return Err(Tani::yeni(
                        "T021",
                        format!("\"değeri\" ile okuma bir sözlük ister; burada {} var.", baska.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            };
            let anahtar_turu = ifade_denetle(anahtar, ortam, baglam, satir)?;
            if anahtar_turu != Tur::Metin {
                return Err(Tani::yeni(
                    "T021",
                    format!("v0'da sözlük anahtarı Metin olmalı; burada {} var.", anahtar_turu.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(deger_turu.ture())
        }
        Ifade::SozlukteVar { sozluk, anahtar, .. } => {
            let sozluk_turu = ifade_denetle(sozluk, ortam, baglam, satir)?;
            // K-058: aynı yüzey listede üyelik de sorar: "sayılarda 5 varsa".
            if let Tur::Liste(oge) = sozluk_turu {
                if !matches!(oge, VeriTuru::TamSayi | VeriTuru::Ondalik | VeriTuru::Metin) {
                    return Err(Tani::yeni(
                        "T021",
                        "Listede üyelik yalnız sayı/metin listelerinde sorulur.".into(),
                        satir,
                        1,
                        1,
                    ));
                }
                let aranan = ifade_denetle(anahtar, ortam, baglam, satir)?;
                if aranan != oge.ture() {
                    return Err(Tani::yeni(
                        "T021",
                        format!("{} listesinde {} aranamaz.", oge.adi(), aranan.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
                return Ok(Tur::Mantiksal);
            }
            if !matches!(sozluk_turu, Tur::Sozluk(_)) {
                return Err(Tani::yeni(
                    "T021",
                    format!("\"varsa\" sorgusu burada bir sözlük ister; {} var.", sozluk_turu.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            let anahtar_turu = ifade_denetle(anahtar, ortam, baglam, satir)?;
            if anahtar_turu != Tur::Metin {
                return Err(Tani::yeni(
                    "T021",
                    "v0'da sözlük anahtarı Metin olmalı.".into(),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::MetinDonusum { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T022",
                    format!("büyük/küçük harfli dönüşümü Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Metin)
        }
        Ifade::Icerir { metin, aranan } => {
            for taraf in [&mut **metin, &mut **aranan] {
                let tur = ifade_denetle(taraf, ortam, baglam, satir)?;
                if tur != Tur::Metin {
                    return Err(Tani::yeni(
                        "T022",
                        format!("\"içeriyorsa\" metinler arasında sorgulanır; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::YokSabiti => Ok(Tur::Yok),
        Ifade::BugununTarihi => Ok(Tur::Tarih),
        Ifade::SuAninSaati => Ok(Tur::Saat),
        Ifade::KomutArgumanlari => Ok(Tur::Liste(VeriTuru::Metin)),
        Ifade::SureSabiti { .. } => Ok(Tur::Sure),
        Ifade::DurumKodu(nesne) => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if tur != Tur::AgYaniti {
                return Err(Tani::yeni(
                    "T034",
                    format!("\"durum kodu\" bir ağ yanıtı ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::TamSayi)
        }
        Ifade::Govde(nesne) => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if tur != Tur::AgYaniti {
                return Err(Tani::yeni(
                    "T034",
                    format!("\"gövdesi\" bir ağ yanıtı ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Metin)
        }
        Ifade::GunSonrasi { tarih, miktar } => {
            let tarih_turu = ifade_denetle(tarih, ortam, baglam, satir)?;
            if tarih_turu != Tur::Tarih {
                return Err(Tani::yeni(
                    "T029",
                    format!("\"gün sonrası\" bir Tarih ister; burada {} var.", tarih_turu.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            let miktar_turu = ifade_denetle(miktar, ortam, baglam, satir)?;
            if miktar_turu != Tur::TamSayi {
                return Err(Tani::yeni(
                    "T029",
                    format!("Gün sayısı TamSayı olmalı; burada {} var.", miktar_turu.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Tarih)
        }
        Ifade::BosMu { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            match tur {
                Tur::Liste(_) | Tur::Sozluk(_) | Tur::Metin => Ok(Tur::Mantiksal),
                baska => Err(Tani::yeni(
                    "T030",
                    format!("\"boşsa\" liste, sözlük ya da metin ister; burada {} var.", baska.adi()),
                    satir,
                    1,
                    1,
                )),
            }
        }
        Ifade::YeniYapi {
            yapi_adi,
            yapi_kimligi,
        } => {
            match baglam.yapi_kimligi(yapi_adi) {
                Some(kimlik) => {
                    *yapi_kimligi = Some(kimlik);
                    Ok(Tur::Yapi(kimlik))
                }
                None => Err(Tani::yeni(
                    "A007",
                    format!("\"{}\" adında bir yapı tanımlı değil.", yapi_adi),
                    satir,
                    1,
                    1,
                )
                .onerili("Önce \"yapı <Ad>\" ile tanımla; yapı, kullanımından önce gelmeli.".into())),
            }
        }
        Ifade::AlanErisim { nesne, alan } => {
            let nesne_turu = ifade_denetle(nesne, ortam, baglam, satir)?;
            let yapi_kimligi = match nesne_turu {
                Tur::Yapi(kimlik) => kimlik,
                baska => {
                    return Err(Tani::yeni(
                        "T028",
                        format!("Alan okuma bir yapı ister; burada {} var.", baska.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            };
            let yapi = baglam
                .yapi(yapi_kimligi)
                .expect("yapı kimliği dizinde kayıtlı")
                .clone();
            let yalin = alan_cozumle(&yapi, alan, satir)?;
            let tur = yapi
                .alanlar
                .iter()
                .find(|(a, _)| *a == yalin)
                .and_then(|(_, t)| alan_turu(t))
                .expect("alan türü doğrulandı");
            *alan = yalin;
            Ok(tur)
        }
        Ifade::SecenekVar { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            match tur {
                Tur::Secenek(_) => Ok(Tur::Mantiksal),
                baska => Err(Tani::yeni(
                    "T023",
                    format!("\"varsa\" sorgusu bir Seçenek ister; burada {} var.", baska.adi()),
                    satir,
                    1,
                    1,
                )
                .onerili("Seçenek, değer döndüren bir işlemin \"yok döndür\" ile karışık dönüşünden doğar.".into())),
            }
        }
        Ifade::IcDeger(nesne) => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            let ad = nesne_adi(nesne);
            match tur {
                Tur::Secenek(e) => {
                    if let Some(ad) = ad {
                        if !baglam.dolu_secenekler.contains(&ad) {
                            return Err(Tani::yeni(
                                "T036",
                                format!(
                                    "\"{}\" boş olabilir: değeri ancak \"varsa\" dalında alınır.",
                                    ad
                                ),
                                satir,
                                1,
                                1,
                            )
                            .onerili(format!("Önce kontrol et: {} varsa", ad)));
                        }
                    }
                    Ok(e.ture())
                }
                Tur::Sonuc(e) => {
                    if let Some(ad) = ad {
                        if !baglam.basarili_sonuclar.contains(&ad) {
                            return Err(Tani::yeni(
                                "T036",
                                format!(
                                    "\"{}\" başarısız olabilir: değeri ancak \"başarılıysa\" dalında alınır.",
                                    ad
                                ),
                                satir,
                                1,
                                1,
                            )
                            .onerili(format!("Önce kontrol et: {} başarılıysa", ad)));
                        }
                    }
                    Ok(e.ture())
                }
                baska => Err(Tani::yeni(
                    "T024",
                    format!("\"değeri\" bir Seçenek ya da Sonuç ister; burada {} var.", baska.adi()),
                    satir,
                    1,
                    1,
                )),
            }
        }
        Ifade::SonucHatasi(nesne) => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if matches!(tur, Tur::Sonuc(_)) {
                if let Some(ad) = nesne_adi(nesne) {
                    if !baglam.basarisiz_sonuclar.contains(&ad) {
                        return Err(Tani::yeni(
                            "T036",
                            format!(
                                "\"{}\" başarılı olabilir: hatası ancak \"başarısızsa\" dalında okunur.",
                                ad
                            ),
                            satir,
                            1,
                            1,
                        )
                        .onerili(format!("Önce kontrol et: {} başarısızsa", ad)));
                    }
                }
            }
            if !matches!(tur, Tur::Sonuc(_)) {
                return Err(Tani::yeni(
                    "T024",
                    format!("\"hatası\" bir Sonuç ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Hata)
        }
        Ifade::SonucBasarili { nesne, .. } => {
            let tur = ifade_denetle(nesne, ortam, baglam, satir)?;
            if !matches!(tur, Tur::Sonuc(_)) {
                return Err(Tani::yeni(
                    "T024",
                    format!("\"başarılıysa\" bir Sonuç ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::DosyaOkumayiDene(yol) => {
            let tur = ifade_denetle(yol, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T025",
                    format!("Dosya yolu Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Sonuc(VeriTuru::Metin))
        }
        Ifade::DosyaSatirlari(yol) => {
            let tur = ifade_denetle(yol, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T025",
                    format!("Dosya yolu Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Liste(VeriTuru::Metin))
        }
        Ifade::TabloOku(yol) => {
            let tur = ifade_denetle(yol, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T025",
                    format!("Dosya yolu Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            // K-062: hücreler Metin okunur; sayı gerekirse `değerin sayısı`.
            Ok(Tur::Liste(VeriTuru::MetinSozluk))
        }
        Ifade::VeriOku(yol) => {
            let tur = ifade_denetle(yol, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T025",
                    format!("Dosya yolu Metin olmalı; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Sozluk(SozlukDegerTuru::Metin))
        }
        Ifade::GunFarki { birinci, ikinci } => {
            let b = ifade_denetle(birinci, ortam, baglam, satir)?;
            let i = ifade_denetle(ikinci, ortam, baglam, satir)?;
            if b != Tur::Tarih || i != Tur::Tarih {
                return Err(Tani::yeni(
                    "T029",
                    format!("\"arasındaki günler\" iki Tarih ister; burada {} ile {} var.", b.adi(), i.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::TamSayi)
        }
        Ifade::Parcala { metin, ayrac } => {
            let m = ifade_denetle(metin, ortam, baglam, satir)?;
            let a = ifade_denetle(ayrac, ortam, baglam, satir)?;
            if m != Tur::Metin || a != Tur::Metin {
                return Err(Tani::yeni("T022", "\"parçaları\" iki Metin ister: metnin ayraçla parçaları.".into(), satir, 1, 1));
            }
            Ok(Tur::Liste(VeriTuru::Metin))
        }
        Ifade::ListeBirlestir { liste, ayrac } => {
            let l = ifade_denetle(liste, ortam, baglam, satir)?;
            let a = ifade_denetle(ayrac, ortam, baglam, satir)?;
            if l != Tur::Liste(VeriTuru::Metin) || a != Tur::Metin {
                return Err(Tani::yeni("T022", format!("\"birleşmişi\" Metin listesi ile Metin ayraç ister; burada {} ile {} var.", l.adi(), a.adi()), satir, 1, 1));
            }
            Ok(Tur::Metin)
        }
        Ifade::Degistir { metin, eski, yeni } => {
            for parca in [metin, eski, yeni] {
                if ifade_denetle(parca, ortam, baglam, satir)? != Tur::Metin {
                    return Err(Tani::yeni("T022", "\"değişmişi\" üç Metin ister: metnin eski yerine yeni değişmişi.".into(), satir, 1, 1));
                }
            }
            Ok(Tur::Metin)
        }
        Ifade::MetinSinari { metin, parca, .. } => {
            let m = ifade_denetle(metin, ortam, baglam, satir)?;
            let p = ifade_denetle(parca, ortam, baglam, satir)?;
            if m != Tur::Metin || p != Tur::Metin {
                return Err(Tani::yeni("T022", "başlıyorsa/bitiyorsa iki Metin ister.".into(), satir, 1, 1));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::Rastgele { alt, ust } => {
            for uc in [&mut **alt, &mut **ust] {
                let tur = ifade_denetle(uc, ortam, baglam, satir)?;
                if tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T010",
                        format!("Rastgele sayının uçları TamSayı olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Ok(Tur::TamSayi)
        }
        Ifade::Degisken {
            ham,
            cozulmus,
            sembol_kimligi,
            satir,
            sutun,
            uzunluk,
        } => {
            let (ad, kimlik) = sembol_cozumle(ham, ortam, *satir, *sutun, *uzunluk)?;
            // RFC-0011 §1: görev sonucuna "hepsini bekle"den önce erişilemez.
            if baglam.bekleyen_gorevler.contains(&ad) {
                return Err(Tani::yeni(
                    "T033",
                    format!(
                        "\"{}\" bir eşzamanlı görev: sonucuna \"hepsini bekle\"den önce erişilemez.",
                        ad
                    ),
                    *satir,
                    *sutun,
                    *uzunluk,
                ));
            }
            let tur = *ortam.get(&ad).expect("çözülen sembolün tür kaydı var");
            *cozulmus = Some(ad);
            *sembol_kimligi = Some(kimlik);
            Ok(tur)
        }
        Ifade::Birlestir(parcalar) => {
            for parca in parcalar {
                ifade_denetle(parca, ortam, baglam, satir)?;
            }
            // "ile" zinciri yazım bağlamında metne birleşir (K-004).
            Ok(Tur::Metin)
        }
        Ifade::Karsilastirma { sol, sag, islec } => {
            let sol_tur = ifade_denetle(sol, ortam, baglam, satir)?;
            let sag_tur = ifade_denetle(sag, ortam, baglam, satir)?;
            let esitlik = *islec == Islec::Esit;
            let iki_sure = sol_tur == Tur::Sure && sag_tur == Tur::Sure;
            if !esitlik && !iki_sure && (!sol_tur.sayisal() || !sag_tur.sayisal()) {
                let sorunlu = if !sol_tur.sayisal() { sol_tur } else { sag_tur };
                return Err(Tani::yeni(
                    "T001",
                    format!(
                        "Büyüklük karşılaştırması sayılar arasında yapılır; burada {} var.",
                        sorunlu.adi()
                    ),
                    satir,
                    1,
                    1,
                )
                .onerili("Karşılaştırılan iki değerin de sayı olduğundan emin ol.".into()));
            }
            if esitlik && sol_tur != sag_tur && !(sol_tur.sayisal() && sag_tur.sayisal()) {
                return Err(Tani::yeni(
                    "T001",
                    format!(
                        "Eşitlik ancak aynı türden değerler arasında sorgulanır: {} ile {} karşılaştırılamaz.",
                        sol_tur.adi(),
                        sag_tur.adi()
                    ),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::MantiksalZincir { parcalar, .. } => {
            for parca in parcalar {
                let tur = ifade_denetle(parca, ortam, baglam, satir)?;
                if tur != Tur::Mantiksal {
                    return Err(Tani::yeni(
                        "T031",
                        format!("ve/veya zincirinin her parçası koşul olmalı; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    ));
                }
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::Degil(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Mantiksal {
                return Err(Tani::yeni(
                    "T031",
                    format!("\"değilse\" bir koşulu olumsuzlar; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                )
                .onerili("Örnekler: x 5 e eşit değilse · bildi doğru değilse".into()));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::Cift(ic) | Ifade::Tek(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::TamSayi {
                return Err(Tani::yeni(
                    "T007",
                    format!("Çift/tek sorgusu TamSayı ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Mantiksal)
        }
        Ifade::Aritmetik { islec, sol, sag } => {
            // K-046: kalan yalnız TamSayılar arasında (okul kavramı tam bölmeye ait).
            if matches!(islec, crate::agac::AritmetikIslec::Kalan) {
                let sol_tur = ifade_denetle(sol, ortam, baglam, satir)?;
                let sag_tur = ifade_denetle(sag, ortam, baglam, satir)?;
                if sol_tur != Tur::TamSayi || sag_tur != Tur::TamSayi {
                    return Err(Tani::yeni(
                        "T008",
                        format!(
                            "Kalan iki TamSayı ister; burada {} ile {} var.",
                            sol_tur.adi(),
                            sag_tur.adi()
                        ),
                        satir,
                        1,
                        1,
                    )
                    .onerili("Ondalık için önce tam kısmını al.".into()));
                }
                return Ok(Tur::TamSayi);
            }
            // Süre + Süre: yalnız toplama/çıkarma (RFC-0011).
            {
                let sol_on = ifade_denetle(sol, ortam, baglam, satir)?;
                let sag_on = ifade_denetle(sag, ortam, baglam, satir)?;
                if sol_on == Tur::Sure || sag_on == Tur::Sure {
                    if sol_on != Tur::Sure || sag_on != Tur::Sure {
                        return Err(Tani::yeni(
                            "T008",
                            format!(
                                "Süre yalnız süreyle toplanıp çıkarılır; burada {} ile {} var.",
                                sol_on.adi(),
                                sag_on.adi()
                            ),
                            satir,
                            1,
                            1,
                        ));
                    }
                    return match islec {
                        crate::agac::AritmetikIslec::Topla
                        | crate::agac::AritmetikIslec::Cikar => Ok(Tur::Sure),
                        _ => Err(Tani::yeni(
                            "T008",
                            "Süre çarpılamaz ve bölünemez (v0).".into(),
                            satir,
                            1,
                            1,
                        )),
                    };
                }
            }
            let mut ondalik_var = false;
            for taraf in [&mut **sol, &mut **sag] {
                let tur = ifade_denetle(taraf, ortam, baglam, satir)?;
                if !tur.sayisal() {
                    return Err(Tani::yeni(
                        "T008",
                        format!("Aritmetik işlem sayılar arasında yapılır; burada {} var.", tur.adi()),
                        satir,
                        1,
                        1,
                    )
                    .onerili(
                        "Metni sayıya çevirmek için \"<metnin> sayısı\" kalıbını kullan.".into(),
                    ));
                }
                ondalik_var |= tur == Tur::Ondalik;
            }
            // TamSayı → Ondalık genişlemesi kayıpsızdır (RFC-0013 §2).
            Ok(if ondalik_var { Tur::Ondalik } else { Tur::TamSayi })
        }
        Ifade::Sayisi(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T009",
                    format!("\"sayısı\" kalıbı Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::TamSayi)
        }
        Ifade::SayiyiDene(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T009",
                    format!("\"almayı dene\" kalıbı Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Sonuc(VeriTuru::TamSayi))
        }
        Ifade::OndaligiDene(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T009",
                    format!("\"almayı dene\" kalıbı Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Sonuc(VeriTuru::Ondalik))
        }
        Ifade::Ondaligi(ic) => {
            let tur = ifade_denetle(ic, ortam, baglam, satir)?;
            if tur != Tur::Metin {
                return Err(Tani::yeni(
                    "T009",
                    format!("\"ondalığı\" kalıbı Metin ister; burada {} var.", tur.adi()),
                    satir,
                    1,
                    1,
                ));
            }
            Ok(Tur::Ondalik)
        }
        Ifade::IslemCagrisi {
            islem_adi,
            islem_kimligi,
            argumanlar,
            satir: cagri_satiri,
        } => {
            let cagri_satiri = *cagri_satiri;
            *islem_kimligi = baglam.islem_kimligi(islem_adi);
            let mut arg_turleri = Vec::new();
            for arg in argumanlar.iter_mut() {
                arg_turleri.push(ifade_denetle(arg, ortam, baglam, cagri_satiri)?);
            }
            match cagri_denetle(islem_adi, &arg_turleri, baglam, cagri_satiri)? {
                Some(tur) => Ok(tur),
                None => Err(Tani::yeni(
                    "T019",
                    format!(
                        "\"{}\" bir değer döndürmüyor; burada değer bekleniyor.",
                        islem_adi
                    ),
                    cagri_satiri,
                    1,
                    1,
                )
                .onerili("İşlemin içinde \"... döndür\" ile bir sonuç döndür.".into())),
            }
        }
    }
}
