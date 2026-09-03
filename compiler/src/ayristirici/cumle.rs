use super::*;

mod web;

impl Ayristirici {
    pub(super) fn cumle_ayristir(&mut self) -> Result<Cumle, Tani> {
        // "işlem ...", "eylem ..." ve "yapı ..." satırları ilk kelimesinden tanınır
        // (tanım başlıkları, yüklem değil).
        if let TokenTur::Kelime(k) = &self.bak().tur {
            if k == "işlem" {
                return self.islem_ayristir(IslemTuru::Islem);
            }
            if k == "eylem" {
                return self.islem_ayristir(IslemTuru::Eylem);
            }
            if k == "yapı" {
                return self.yapi_ayristir();
            }
            if k == "test" {
                return self.test_ayristir();
            }
        }

        let satir_tokenlari = self.satir_oku();
        let satir_no = satir_tokenlari.first().map(|t| t.satir).unwrap_or(1);
        let son_kelime = son_kelime(&satir_tokenlari);

        match son_kelime.as_deref() {
            Some("açık") => {
                if satir_tokenlari.len() == 2 && kelime_mi(&satir_tokenlari[0], "herkese") {
                    Ok(Cumle::RotaPolitikasi {
                        erisim: RotaErisimi::HerkeseAcik,
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S043",
                        "Açık rota politikası `herkese açık` biçimindedir.".into(),
                        satir_no,
                        1,
                        1,
                    ))
                }
            }
            Some("gerekli") => {
                let t = &satir_tokenlari;
                if t.len() == 2 && kelime_mi(&t[0], "oturum") {
                    Ok(Cumle::RotaPolitikasi {
                        erisim: RotaErisimi::Oturumlu,
                        satir: satir_no,
                    })
                } else if t.len() == 3 && kelime_mi(&t[1], "yetkisi") {
                    let TokenTur::Metin(rol) = &t[0].tur else {
                        return Err(Tani::yeni(
                            "S043",
                            "Yetki rolü sabit Metin olmalı: `\"yönetici\" yetkisi gerekli`.".into(),
                            satir_no,
                            1,
                            1,
                        ));
                    };
                    Ok(Cumle::RotaPolitikasi {
                        erisim: RotaErisimi::Rol(rol.clone()),
                        satir: satir_no,
                    })
                } else if t.len() == 3 && kelime_mi(&t[1], "alanı") {
                    let TokenTur::Metin(ad) = &t[0].tur else {
                        return Err(Tani::yeni(
                            "S043",
                            "İstek alanı sabit Metin olmalı: `\"parola\" alanı gerekli`.".into(),
                            satir_no,
                            1,
                            1,
                        ));
                    };
                    Ok(Cumle::RotaAlaniGerekli {
                        ad: ad.clone(),
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S043",
                        "Rota önsözü `oturum gerekli`, `\"rol\" yetkisi gerekli` ya da `\"alan\" alanı gerekli` biçimindedir."
                            .into(),
                        satir_no,
                        1,
                        1,
                    ))
                }
            }
            Some("al") => {
                let t = &satir_tokenlari;
                if t.len() == 6
                    && kelime_mi(&t[1], "kullanıcısını")
                    && kelime_mi(&t[3], "rolüyle")
                    && kelime_mi(&t[4], "oturuma")
                {
                    Ok(Cumle::OturumAc {
                        kullanici: tekil_ifade(t[0].clone())?,
                        rol: tekil_ifade(t[2].clone())?,
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S043",
                        "Oturum `\"kullanıcı\" kullanıcısını \"rol\" rolüyle oturuma al` biçiminde açılır."
                            .into(),
                        satir_no,
                        1,
                        1,
                    ))
                }
            }
            Some("kapat") => {
                if satir_tokenlari.len() == 2 && kelime_mi(&satir_tokenlari[0], "oturumu") {
                    Ok(Cumle::OturumKapat { satir: satir_no })
                } else {
                    Err(Tani::yeni(
                        "S043",
                        "Oturum `oturumu kapat` biçiminde sonlandırılır.".into(),
                        satir_no,
                        1,
                        1,
                    ))
                }
            }
            Some("yaz") => self.yaz_ayristir(satir_tokenlari, satir_no),
            Some("olsun") => self.olsun_ayristir(satir_tokenlari, satir_no),
            Some("tekrarla") => self.tekrarla_ayristir(satir_tokenlari, satir_no),
            Some("için") => self.aralik_ayristir(satir_tokenlari, satir_no),
            Some("sürece") => self.surece_ayristir(satir_tokenlari, satir_no),
            Some("artır") => self.artir_azalt_ayristir(satir_tokenlari, satir_no, true),
            Some("azalt") => self.artir_azalt_ayristir(satir_tokenlari, satir_no, false),
            Some("sor") => self.sor_ayristir(satir_tokenlari, satir_no),
            Some("ekle") => self.ekle_ayristir(satir_tokenlari, satir_no),
            // Silme (K-059): "<kap>tan <değer> [ayrık ek] sil".
            Some("sil") => {
                let mut t = satir_tokenlari;
                t.pop(); // sil
                         // Çerez silme (K-073): `"oturum" çerezini sil`.
                if t.len() == 2 && kelime_mi(&t[1], "çerezini") {
                    let ad = tekil_ifade(t[0].clone())?;
                    return Ok(Cumle::CerezSil {
                        ad,
                        satir: satir_no,
                    });
                }
                // Değerle "sil" arasında ayrık belirtme eki olabilir: "5 i sil".
                if t.len() == 3 {
                    if let TokenTur::Kelime(k) = &t[2].tur {
                        if ["i", "ı", "u", "ü", "yi", "yı", "yu", "yü"].contains(&k.as_str()) {
                            t.pop();
                        }
                    }
                }
                if t.len() == 2 {
                    let kap = tekil_ifade(t[0].clone())?;
                    let deger = tekil_ifade(t[1].clone())?;
                    Ok(Cumle::Sil {
                        kap,
                        deger,
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S042",
                        "Silme \"<kap>tan <değer> sil\" biçiminde yazılır.".into(),
                        satir_no,
                        1,
                        1,
                    )
                    .onerili("Örnek: sayılardan 5 i sil · defterden \"elma\" yı sil".into()))
                }
            }
            Some("döndür") => self.dondur_ayristir(satir_tokenlari, satir_no),
            Some("böl") => self.bol_ayristir(satir_tokenlari, satir_no),
            Some("göre") => self.gore_ayristir(satir_tokenlari, satir_no),
            // "/liste" adresine yönlendir (K-051).
            Some("yönlendir") => {
                let t = &satir_tokenlari;
                if t.len() == 3 && kelime_mi(&t[1], "adresine") {
                    let adres = tekil_ifade(t[0].clone())?;
                    Ok(Cumle::Yonlendir {
                        adres,
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S041",
                        "Yönlendirme \"<adres> adresine yönlendir\" biçiminde yazılır.".into(),
                        satir_no,
                        1,
                        1,
                    )
                    .onerili("Örnek: \"/liste\" adresine yönlendir".into()))
                }
            }
            Some("olmalı") => self.olmali_ayristir(satir_tokenlari, satir_no),
            // K-071 (K-025 adayları): olumsuz doğrulama + içerme doğrulaması.
            Some("olmamalı") => {
                let mut t = satir_tokenlari;
                t.pop(); // olmamalı
                let ic = kosul_ifadesi(&t, satir_no)?;
                let kosul = konumlu_ifade(Ifade::Degil(Box::new(ic)), &t)?;
                Ok(Cumle::Olmali {
                    kosul,
                    satir: satir_no,
                })
            }
            Some("içermeli") => {
                let t = &satir_tokenlari;
                if t.len() == 3 {
                    let kosul = konumlu_ifade(
                        Ifade::Icerir {
                            metin: Box::new(tekil_ifade(t[0].clone())?),
                            aranan: Box::new(tekil_ifade(t[1].clone())?),
                        },
                        t,
                    )?;
                    Ok(Cumle::Olmali {
                        kosul,
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S026",
                        "İçerme doğrulaması \"<metin> <parça> içermeli\" biçiminde yazılır.".into(),
                        satir_no,
                        1,
                        1,
                    ))
                }
            }
            Some("başlat") => {
                let t = &satir_tokenlari;
                if t.len() == 4 && kelime_mi(&t[1], "kapısında") && kelime_mi(&t[2], "sunucu") {
                    let kapi = tekil_ifade(t[0].clone())?;
                    Ok(Cumle::SunucuBaslat {
                        kapi,
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S035",
                        "Sunucu \"<kapı> kapısında sunucu başlat\" biçiminde açılır.".into(),
                        satir_no,
                        1,
                        1,
                    )
                    .onerili("Örnek: 8080 kapısında sunucu başlat".into()))
                }
            }
            Some("geldiğinde") => {
                let t = &satir_tokenlari;
                let yontem = t.first().and_then(|token| match &token.tur {
                    TokenTur::Kelime(kelime) => HttpYontemi::ayristir(kelime),
                    _ => None,
                });
                let yontemli = yontem.is_some();
                let bas = usize::from(yontemli);
                if t.len() == 4 + bas
                    && kelime_mi(&t[bas + 1], "adresine")
                    && kelime_mi(&t[bas + 2], "istek")
                {
                    let yol = tekil_ifade(t[bas].clone())?;
                    let govde = self.alt_blok(satir_no)?;
                    Ok(Cumle::IstekGeldiginde {
                        yontem,
                        yol,
                        onekli: false,
                        govde,
                        satir: satir_no,
                    })
                } else if t.len() == 5 + bas
                    && kelime_mi(&t[bas + 1], "önekli")
                    && kelime_mi(&t[bas + 2], "adrese")
                    && kelime_mi(&t[bas + 3], "istek")
                {
                    // "/yazi/" önekli adrese istek geldiğinde (K-055).
                    let yol = tekil_ifade(t[bas].clone())?;
                    let govde = self.alt_blok(satir_no)?;
                    Ok(Cumle::IstekGeldiginde {
                        yontem,
                        yol,
                        onekli: true,
                        govde,
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S036",
                        "Web adaptörü `YÖNTEM \"<yol>\" adresine istek geldiğinde` biçimindedir."
                            .into(),
                        satir_no,
                        1,
                        1,
                    )
                    .onerili(
                        "Yöntemlerden birini açıkça yaz: GET, HEAD, POST, PUT, PATCH, DELETE."
                            .into(),
                    ))
                }
            }
            Some("gönder") => self.yanit_ayristir(satir_tokenlari, satir_no),
            Some("olarak") => {
                if satir_tokenlari.len() == 2 && kelime_mi(&satir_tokenlari[0], "eşzamanlı") {
                    self.eszamanli_ayristir(satir_no)
                } else {
                    Err(Tani::yeni(
                        "S038",
                        "Eşzamanlı blok \"eşzamanlı olarak\" ile başlar.".into(),
                        satir_no,
                        1,
                        1,
                    ))
                }
            }
            Some("bekle") => {
                let mut t = satir_tokenlari;
                t.pop(); // bekle
                if t.len() == 1 && kelime_mi(&t[0], "hepsini") {
                    Ok(Cumle::HepsiniBekle { satir: satir_no })
                } else {
                    let sure = ile_ifadesi(&t, satir_no, &self.islem_adlari)?;
                    Ok(Cumle::Bekle {
                        sure,
                        satir: satir_no,
                    })
                }
            }
            Some("içinde") => {
                let mut t = satir_tokenlari;
                t.pop(); // içinde
                let sure = ile_ifadesi(&t, satir_no, &self.islem_adlari)?;
                let govde = self.alt_blok(satir_no)?;
                let mut yetismezse = None;
                if matches!(&self.bak().tur, TokenTur::Kelime(k) if k == "yetişmezse") {
                    let kol_satiri = self.bak().satir;
                    let kol = self.satir_oku();
                    if kol.len() != 1 {
                        self.tani_kaydet(Tani::yeni(
                            "S036",
                            "\"yetişmezse\" tek başına bir satır olmalı.".into(),
                            kol_satiri,
                            1,
                            1,
                        ));
                        self.bekleyen_govdeyi_atla();
                    } else {
                        match self.alt_blok(kol_satiri) {
                            Ok(govde) => yetismezse = Some(govde),
                            Err(tani) => self.tani_kaydet(tani),
                        }
                    }
                }
                Ok(Cumle::IcindeBlogu {
                    sure,
                    govde,
                    yetismezse,
                    satir: satir_no,
                })
            }
            Some("yak") | Some("söndür") => {
                let yansin = son_kelime.as_deref() == Some("yak");
                let t = &satir_tokenlari;
                if t.len() == 3 && kelime_mi(&t[1], "ışığı") {
                    if let TokenTur::Kelime(isik) = &t[0].tur {
                        return Ok(Cumle::IsikAyarla {
                            isik: isik.clone(),
                            yansin,
                            satir: satir_no,
                        });
                    }
                }
                Err(Tani::yeni(
                    "S039",
                    "Işık \"<renk> ışığı yak\" ya da \"<renk> ışığı söndür\" ile sürülür.".into(),
                    satir_no,
                    1,
                    1,
                ))
            }
            Some("kullan") => {
                if self.derinlik > 0 {
                    return Err(Tani::yeni(
                        "S021",
                        "Birim kullanımı en dış düzeyde olmalı.".into(),
                        satir_no,
                        1,
                        1,
                    ));
                }
                match (
                    satir_tokenlari.first().map(|t| &t.tur),
                    satir_tokenlari.get(1).map(|t| &t.tur),
                ) {
                    (Some(TokenTur::Kelime(ad)), Some(TokenTur::Kelime(tur)))
                        if satir_tokenlari.len() == 3
                            && matches!(tur.as_str(), "birimini" | "paketini") =>
                    {
                        Ok(Cumle::Kullan {
                            ad: ad.clone(),
                            tur: if tur == "birimini" {
                                crate::agac::KullanimTuru::Birim
                            } else {
                                crate::agac::KullanimTuru::Paket
                            },
                            satir: satir_no,
                        })
                    }
                    _ => Err(Tani::yeni(
                        "S034",
                        "Kullanım \"<ad> birimini kullan\" ya da \"<ad> paketini kullan\" biçiminde yazılır.".into(),
                        satir_no,
                        1,
                        1,
                    )
                    .onerili("Örnek: hesaplar birimini kullan — aynı klasördeki dosyayı; grafik paketini kullan — proje bağımlılığını alır.".into())),
                }
            }
            Some("bitir") => {
                if satir_tokenlari.len() == 2 && kelime_mi(&satir_tokenlari[0], "programı") {
                    Ok(Cumle::ProgramiBitir {
                        kod: None,
                        satir: satir_no,
                    })
                } else if satir_tokenlari.len() == 4
                    && kelime_mi(&satir_tokenlari[0], "programı")
                    && kelime_mi(&satir_tokenlari[2], "ile")
                {
                    // K-069 (K-024 adayı): `programı 1 ile bitir` — çıkış kodu.
                    let kod = tekil_ifade(satir_tokenlari[1].clone())?;
                    Ok(Cumle::ProgramiBitir {
                        kod: Some(kod),
                        satir: satir_no,
                    })
                } else {
                    Err(Tani::yeni(
                        "S027",
                        "Sonlandırma \"programı bitir\" biçiminde yazılır.".into(),
                        satir_no,
                        1,
                        1,
                    ))
                }
            }
            // Satır sonundaki "değilse": olumsuzlanmış koşul başlığı
            // ("x 5 e eşit değilse"). Tek başına "değilse" ise başıboş else'tir.
            Some("değilse") => {
                if satir_tokenlari.len() == 1 {
                    Err(Tani::yeni(
                        "S031",
                        "\"değilse\" tek başına duramaz.".into(),
                        satir_no,
                        1,
                        1,
                    )
                    .onerili(
                        "\"değilse\" bir \"... ise\" bloğunun hemen ardından, aynı hizada gelir."
                            .into(),
                    ))
                } else {
                    self.ise_ayristir(satir_tokenlari, satir_no)
                }
            }
            // "asal ise" — Mantıksal adın kendisi koşuldur (K-044).
            Some("ise") => self.ise_ayristir(satir_tokenlari, satir_no),
            Some(k) if kosul_kelimesi(k) => self.ise_ayristir(satir_tokenlari, satir_no),
            _ => {
                // Tanımlı bir işlem adına biten satır → çağrı cümlesi.
                if let Some(cagri) = cagri_kalibi(&satir_tokenlari, satir_no, &self.islem_adlari)? {
                    return Ok(Cumle::CagriCumlesi {
                        cagri,
                        satir: satir_no,
                    });
                }
                let ilk = satir_tokenlari.first().cloned();
                let (sutun, uzunluk) = ilk.map(|t| (t.sutun, t.uzunluk)).unwrap_or((1, 1));
                Err(Tani::yeni(
                    "S004",
                    "Bu cümle tanınmadı: cümleler eylemle biter.".into(),
                    satir_no,
                    sutun,
                    uzunluk,
                )
                .onerili(
                    "Desteklenen kalıplar: \"... yaz\", \"<ad> ... olsun\", \"<n> kez tekrarla\", \
                     \"<a> den <b> e kadar her <ad> için\", \"... olduğu sürece\", \"... ise / değilse\", \
                     \"<ad> <n> artır/azalt\", \"işlem <ad>\", \"... döndür\", işlem çağrısı. \
                     Çağrılan işlem bu dosyada (ya da kullanılan bir birimde) tanımlı olmalı."
                        .into(),
                ))
            }
        }
    }
}
