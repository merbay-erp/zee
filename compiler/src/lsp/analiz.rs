//! Açık belge tanı yayını, overlay seçimi ve birim kökeni konumlaması.
use super::*;

impl Sunucu {
    pub(super) fn acik_tanilari_yenile(
        &mut self,
        degisen: &str,
        cikti: &mut Ciktilar,
        kimlik: Option<&Json>,
    ) -> bool {
        // Açık envanter mevcut belge/adet/bayt sınırlarıyla kısıtlıdır. Disk
        // grafiğini önbelleğe almadan bütün tüketicileri tekrar doğrularız.
        let mut uriler: Vec<_> = self.belgeler.keys().cloned().collect();
        uriler.sort_by(|a, b| (a != degisen, a).cmp(&(b != degisen, b)));
        let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.lsp_yanit_bayti();
        let rpc_bayti = if kimlik.is_some() {
            yanit_uret(kimlik, |yazici| yazici.ham("null"))
                .map(|g| g.len())
                .unwrap_or(azami)
        } else {
            0
        };
        let mut toplam: usize = cikti.govdeler.iter().map(String::len).sum::<usize>() + rpc_bayti;
        if toplam > azami {
            cikti_ekle(cikti, kimlik, Err(cikti::CiktiTasmasi));
            return false;
        }
        for uri in uriler {
            let sonuc = self.tanilari_yayinla(&uri).and_then(|govde| {
                toplam = toplam.checked_add(govde.len()).ok_or(cikti::CiktiTasmasi)?;
                if toplam > azami {
                    return Err(cikti::CiktiTasmasi);
                }
                Ok(govde)
            });
            let tasma = sonuc.is_err();
            cikti_ekle(cikti, kimlik, sonuc);
            if tasma {
                return false;
            }
        }
        true
    }

    pub(super) fn tanilari_yayinla(&mut self, uri: &str) -> CiktiSonucu<String> {
        let metin = self.belgeler.get(uri).cloned().unwrap_or_default();
        let belge_yolu = uri_yolu(uri).and_then(|yol| std::fs::canonicalize(yol).ok());

        // Bir proje içindeyse CLI ile aynı kökenli paket grafiğini kullan.
        // Böylece editörde temiz görünen kaynak komut satırında kırılmaz.
        let oturum = &mut self.sozcukleme;
        let belgeler = &self.belgeler;
        let proje_tanilari = belge_yolu.as_ref().and_then(|belge_yolu| {
            let kok = proje_kokunu_bul(belge_yolu)?;
            let grafik = match crate::paket::ProjeGrafigi::cozumle(&kok) {
                Ok(grafik) => grafik,
                Err(hata) => return Some(vec![*hata.tani]),
            };
            if let Err(hata) = grafik.kilidi_denetle() {
                return Some(vec![*hata.tani]);
            }
            let koken = belge_yolu.to_string_lossy().into_owned();
            let mut yukleyici = |istek: crate::BirimIstegi<'_>| {
                let mut birim = grafik.yukle(istek)?;
                // Paket doğrulaması önce disk üzerinde yapılır. Yalnız aktif proje
                // kökünün editörde açık yerel kaynakları analiz overlay'i olabilir.
                if grafik.ana_kaynagi_mi(std::path::Path::new(&birim.koken)) {
                    if let Some(kaynak) =
                        acik_belge_kaynagi(belgeler, std::path::Path::new(&birim.koken))
                    {
                        birim.kaynak = kaynak.to_string();
                    }
                }
                Ok(birim)
            };
            Some(crate::kaynagi_tanilari_oturumlu(
                &metin,
                Some(&koken),
                &mut yukleyici,
                Some(&mut *oturum),
            ))
        });

        let klasor = uri_klasoru(uri);
        let mut yukleyici =
            move |istek: crate::BirimIstegi<'_>| -> Result<crate::YuklenenBirim, String> {
                let ad = istek.ad;
                if istek.tur == crate::agac::KullanimTuru::Paket {
                    return Err("paket çözümü için proje bağlamı gerekir".into());
                }
                let klasor = klasor.clone().ok_or("birim yolu çözülemedi")?;
                if ad.contains(['/', '\\', '.']) {
                    return Err("birim adı yol içeremez".into());
                }
                let yol = klasor.join(format!("{}.dil", ad));
                match crate::kaynak_sinirlari::kaynak_dosyasi_oku(&yol) {
                    Ok(kaynak) => Ok(crate::YuklenenBirim {
                        kaynak: acik_belge_kaynagi(belgeler, &yol)
                            .unwrap_or(&kaynak)
                            .to_string(),
                        // Önceki köken sözleşmesi korunur; cache anahtarı kaynakla da bağlıdır.
                        koken: ad.to_string(),
                    }),
                    Err(hata) => crate::gomulu_birim(ad)
                        .map(|kaynak| crate::YuklenenBirim {
                            kaynak: kaynak.to_string(),
                            koken: ad.to_string(),
                        })
                        .ok_or_else(|| hata.to_string()),
                }
            };
        let tanilar = proje_tanilari.unwrap_or_else(|| {
            crate::kaynagi_tanilari_oturumlu(&metin, None, &mut yukleyici, Some(oturum))
        });
        let mut tanilar = birim_tanilarini_konumla(&metin, tanilar);
        if self.lint && tanilar.is_empty() {
            let _kokenler = crate::agac::kaynak_kokenlerini_izle(true);
            if let Some(program) = semantik_program_derle_belgeler(uri, &metin, Some(belgeler)) {
                let koken = belge_yolu
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned());
                tanilar = program.hir().uyarilari(koken.as_deref());
            }
        }
        tanilari_yaz(uri, &tanilar)
    }
}

pub(super) fn acik_belge_kaynagi<'a>(
    belgeler: &'a HashMap<String, String>,
    yol: &std::path::Path,
) -> Option<&'a str> {
    let yol = std::fs::canonicalize(yol).ok()?;
    let mut adaylar = belgeler.iter().filter_map(|(uri, metin)| {
        (uri_yolu(uri)
            .and_then(|p| std::fs::canonicalize(p).ok())
            .as_ref()
            == Some(&yol))
        .then_some(metin.as_str())
    });
    let ilk = adaylar.next()?;
    // Aynı gerçek dosyanın çelişen URI alias'ları HashMap sırasına göre seçilemez.
    adaylar.all(|diger| diger == ilk).then_some(ilk)
}

/// Kökendeki birim adı: dosya yolunun gövdesi ya da `gömülü:` sonrası.
fn koken_birim_adi(koken: &str) -> String {
    if let Some(ad) = koken.strip_prefix("gömülü:") {
        return ad.to_string();
    }
    std::path::Path::new(koken)
        .file_stem()
        .and_then(|ad| ad.to_str())
        .unwrap_or(koken)
        .to_string()
}

/// Birim kökenli tanının satırı birimin metnine aittir; açık belgede
/// işaretlenemez. Tanı, o birimin `kullan` satırına taşınır ve mesaj birim
/// adı ile özgün satırı söyler (K-164/ADR-072). Kökensiz tanılar değişmez.
fn birim_tanilarini_konumla(metin: &str, tanilar: Vec<Tani>) -> Vec<Tani> {
    if tanilar.iter().all(|tani| tani.koken.is_none()) {
        return tanilar;
    }
    let kullanimlar = crate::faz::KaynakMetni::yeni(metin)
        .sozcukle()
        .map(|tokenlar: crate::faz::TokenAkisi| {
            crate::ayristirici::kullanilan_birimler(tokenlar.tokenlar())
        })
        .unwrap_or_default();
    tanilar
        .into_iter()
        .map(|tani| {
            let Some(koken) = tani.koken.clone() else {
                return tani;
            };
            let birim_adi = koken_birim_adi(&koken);
            let satir = kullanimlar
                .iter()
                .find(|(ad, _, _)| *ad == birim_adi)
                .map(|(_, _, satir)| *satir)
                .unwrap_or(1);
            let uzunluk = metin
                .lines()
                .nth(satir.saturating_sub(1))
                .map(|satir_metni| satir_metni.chars().count())
                .unwrap_or(1)
                .max(1);
            Tani {
                kod: tani.kod,
                mesaj: format!("{} birimi, satır {}: {}", birim_adi, tani.satir, tani.mesaj),
                satir,
                sutun: 1,
                uzunluk,
                oneri: tani.oneri,
                koken: tani.koken,
            }
        })
        .collect()
}
