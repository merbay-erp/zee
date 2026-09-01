use super::*;

/// İşlem çağrısını denetler. İlk çağrıda gövde argüman türleriyle denetlenir;
/// sayısal imza K-067 ile genişleyebilir, diğer çağrılar imzaya uymalıdır.
///
/// ÖZYİNELEME (v0.2): denetimi süren bir işlem kendini (ya da karşılıklı
/// olarak birbirini) çağırabilir. Özyinelemeli çağrının türü, o ana dek
/// görülen dönüş dallarından çıkarılır — bu yüzden TEMEL DURUM ÖNCE yazılır
/// (T035): hem tür çıkarımı hem sonsuz döngüye karşı pedagojik korkuluk.
pub(super) fn cagri_denetle(
    ad: &str,
    arg_turleri: &[Tur],
    baglam: &mut Baglam,
    satir: usize,
) -> Result<Option<Tur>, Tani> {
    if let Some(imza) = baglam.imzalar.get(ad) {
        if imza.parametre_turleri.len() != arg_turleri.len() {
            return Err(Tani::yeni(
                "T015",
                format!(
                    "\"{}\" {} parametre bekler, {} argüman verildi.",
                    ad,
                    imza.parametre_turleri.len(),
                    arg_turleri.len()
                ),
                satir,
                1,
                1,
            ));
        }
        // K-067: çağrıda genişleme — TamSayı argüman Ondalık parametreye,
        // Liste<TamSayı> argüman Liste<Ondalık> parametreye uyar (skaler
        // genişleme kuralının doğal uzantısı; ters yön yine bilinçli değil).
        let uyumlu = imza
            .parametre_turleri
            .iter()
            .zip(arg_turleri.iter())
            .all(|(param, arg)| cagri_turu_uyumlu(param, arg));
        if !uyumlu {
            // K-067 imza terfisi: uyumsuzluk YALNIZ ters-genişlemeyse
            // (param TamSayı[-listesi], arg Ondalık[-listesi]) imza kaldırılır
            // ve gövde geniş türlerle ilk-çağrı gibi yeniden denetlenir —
            // saklanan imza bu iki tür içinde en geniş biçime ulaşır. Bu,
            // public sözleşmeyi bütün çağrı yerlerinden bağımsız yapmaz
            // (V1-P0-01). Gövde geniş türle geçerli değilse doğal tanısı çıkar.
            // Özyineleme denetimi
            // sürerken terfi yapılmaz (T017 kalır).
            let yalniz_ters_genisleme = imza
                .parametre_turleri
                .iter()
                .zip(arg_turleri.iter())
                .all(|(param, arg)| {
                    param == arg
                        || matches!((param, arg), (Tur::Ondalik, Tur::TamSayi))
                        || matches!(
                            (param, arg),
                            (Tur::Liste(VeriTuru::Ondalik), Tur::Liste(VeriTuru::TamSayi))
                        )
                        || matches!((param, arg), (Tur::TamSayi, Tur::Ondalik))
                        || matches!(
                            (param, arg),
                            (Tur::Liste(VeriTuru::TamSayi), Tur::Liste(VeriTuru::Ondalik))
                        )
                });
            let ozyinelemede = baglam.denetim_yigini.iter().any(|k| k.ad == ad);
            if yalniz_ters_genisleme && !ozyinelemede && !imza.acik {
                baglam.imzalar.remove(ad);
                return cagri_denetle(ad, arg_turleri, baglam, satir);
            }
            return Err(Tani::yeni(
                "T017",
                format!("\"{}\" çağrısındaki argüman türleri işlemin imzasına uymuyor.", ad),
                satir,
                1,
                1,
            ));
        }
        return Ok(imza.donus);
    }

    // Özyinelemeli (ya da karşılıklı özyinelemeli) çağrı: denetim yığınında.
    if let Some(indeks) = baglam.denetim_yigini.iter().position(|k| k.ad == ad) {
        if baglam.denetim_yigini[indeks].parametre_turleri != arg_turleri {
            return Err(Tani::yeni(
                "T017",
                format!(
                    "\"{}\" özyinelemeli çağrısındaki argüman türleri ilk çağrıyla uyuşmuyor.",
                    ad
                ),
                satir,
                1,
                1,
            ));
        }
        let tahmin =
            donusleri_birlestir(ad, &baglam.denetim_yigini[indeks].donusler, satir)?;
        let tahmin = match tahmin {
            Some(tur) => tur,
            None => {
                return Err(Tani::yeni(
                    "T035",
                    format!(
                        "\"{}\" özyinelemeli çağrıdan ÖNCE en az bir dalda değer döndürmeli.",
                        ad
                    ),
                    satir,
                    1,
                    1,
                )
                .onerili(
                    "Temel durumu üste yaz: önce \"n 1 e eşitse\" gibi bir dalda döndür, \
                     sonra özyinelemeli adım. Bu, sonsuz döngüye karşı da ilk korkuluktur."
                        .into(),
                ));
            }
        };
        let kayit = &mut baglam.denetim_yigini[indeks];
        match kayit.verilen_ozyineleme {
            None => kayit.verilen_ozyineleme = Some(tahmin),
            Some(onceki) if onceki != tahmin => {
                return Err(Tani::yeni(
                    "T018",
                    format!("\"{}\" özyinelemeli kullanımları farklı türlere çıkıyor.", ad),
                    satir,
                    1,
                    1,
                ));
            }
            _ => {}
        }
        return Ok(Some(tahmin));
    }

    let mut islem = baglam.islemler.remove(ad).ok_or_else(|| {
        Tani::yeni(
            "T016",
            format!(
                "\"{}\" işleminin kaydı bulunamadı — derleyici iç hatası olabilir, bildir.",
                ad
            ),
            satir,
            1,
            1,
        )
    })?;

    if islem.parametreler.len() != arg_turleri.len() {
        let beklenen = islem.parametreler.len();
        baglam.islemler.insert(ad.to_string(), islem);
        return Err(Tani::yeni(
            "T015",
            format!(
                "\"{}\" {} parametre bekler, {} argüman verildi.",
                ad,
                beklenen,
                arg_turleri.len()
            ),
            satir,
            1,
            1,
        ));
    }

    let acik_turler = match acik_parametre_turleri(&islem, &baglam.yapilar) {
        Ok(turler) => turler,
        Err(tani) => {
            baglam.islemler.insert(ad.to_string(), islem);
            return Err(tani);
        }
    };
    let bildirilmis_donus = match bildirilmis_donus_turu(&islem, &baglam.yapilar) {
        Ok(donus) => donus,
        Err(tani) => {
            baglam.islemler.insert(ad.to_string(), islem);
            return Err(tani);
        }
    };
    let donus_bildirim_satiri = islem.donus_satiri.unwrap_or(satir);
    let acik = acik_turler.is_some();
    let denetim_turleri = acik_turler.unwrap_or_else(|| arg_turleri.to_vec());
    if !denetim_turleri
        .iter()
        .zip(arg_turleri.iter())
        .all(|(parametre, arguman)| cagri_turu_uyumlu(parametre, arguman))
    {
        let beklenen = denetim_turleri
            .iter()
            .map(Tur::adi)
            .collect::<Vec<_>>()
            .join(", ");
        let bulunan = arg_turleri
            .iter()
            .map(Tur::adi)
            .collect::<Vec<_>>()
            .join(", ");
        baglam.islemler.insert(ad.to_string(), islem);
        return Err(Tani::yeni(
            "T017",
            format!(
                "\"{}\" çağrısı açık işlem imzasına uymuyor: beklenen {}, bulunan {}.",
                ad, beklenen, bulunan
            ),
            satir,
            1,
            1,
        ));
    }

    let mut islem_ortami: HashMap<String, Tur> = HashMap::new();
    for (param, tur) in islem.parametreler.iter().zip(denetim_turleri.iter()) {
        islem_ortami.insert(param.ad.clone(), *tur);
    }

    baglam.denetim_yigini.push(ImzaKaydi {
        ad: ad.to_string(),
        parametre_turleri: denetim_turleri.clone(),
        donusler: Vec::new(),
        verilen_ozyineleme: None,
    });
    let denetim = blok_denetle(&mut islem.govde, &mut islem_ortami, baglam);
    let kayit = baglam.denetim_yigini.pop().expect("kayıt az önce eklendi");
    // Gövde her durumda kayda geri konur; hata olsa bile kayıt tutarlı kalır.
    if denetim.is_ok() && kayit.donusler.contains(&Tur::HataDonusu) {
        donusleri_sarmala(&mut islem.govde);
    }
    let kesin_sonlanir = blok_kesin_sonlanir(&islem.govde);
    baglam.islemler.insert(ad.to_string(), islem);
    denetim?;

    let donus = donusleri_birlestir(ad, &kayit.donusler, satir)?;

    if let Some(beklenen) = bildirilmis_donus {
        if donus != beklenen {
            let beklenen_adi = beklenen
                .map(|tur| tur.adi())
                .unwrap_or_else(|| "değer döndürmez".into());
            let bulunan_adi = donus
                .map(|tur| tur.adi())
                .unwrap_or_else(|| "değer döndürmez".into());
            return Err(Tani::yeni(
                "T041",
                format!(
                    "\"{}\" işlemi {} döndüreceğini bildiriyor; gövde {} üretiyor.",
                    ad, beklenen_adi, bulunan_adi
                ),
                donus_bildirim_satiri,
                1,
                1,
            )
            .onerili("Dönüş bildirimini ve bütün `döndür` dallarını aynı türde buluştur.".into()));
        }
        if beklenen.is_some() && !kesin_sonlanir {
            return Err(Tani::yeni(
                "T042",
                format!(
                    "\"{}\" işleminin bazı yolları değer döndürmeden bitebilir.",
                    ad
                ),
                donus_bildirim_satiri,
                1,
                1,
            )
            .onerili(
                "Her koşul/eşleştirme yolunda değer döndür veya en sona ortak bir `döndür` ekle."
                    .into(),
            ));
        }
    }

    // Özyinelemeli kullanıma verilen tür, son birleşimle çelişmemeli
    // (örn. temel durum TamSayı verip sonradan "yok döndür" eklemek).
    if let Some(verilen) = kayit.verilen_ozyineleme {
        if donus != Some(verilen) {
            return Err(Tani::yeni(
                "T018",
                format!(
                    "\"{}\" işleminde özyinelemeli kullanım {} sayılmıştı ama dönüş birleşimi {} çıktı.",
                    ad,
                    verilen.adi(),
                    donus.map(|d| d.adi()).unwrap_or_else(|| "dönüşsüz".into())
                ),
                satir,
                1,
                1,
            )
            .onerili(
                "yok/hatasını döndür dallarını özyinelemeli adımdan ÖNCE, temel durumla \
                 birlikte en üste yaz."
                    .into(),
            ));
        }
    }

    baglam.imzalar.insert(
        ad.to_string(),
        Imza {
            parametre_turleri: denetim_turleri,
            donus,
            acik,
        },
    );
    Ok(donus)
}
