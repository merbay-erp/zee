//! Sürümlü, kanonik ve sıralı IO trace/replay katmanı (K-115, ADR-026).
//! İz özel veridir; parola doğrulama argümanları yalnız SHA-256 parmak iziyle kaydedilir.

mod eylem;

use self::eylem::{semayi_denetle as eylem_semasi, sonuc_yaz as eylem_sonuc_yaz};
use super::io_izi_veritabani::{
    degistirme_coz as veritabani_degistirme_coz, degistirme_yaz as veritabani_degistirme_yaz,
    okuma_coz as veritabani_okuma_coz, okuma_yaz as veritabani_okuma_yaz,
};
use super::io_izi_web::{
    rota_arguman_semasi, rota_argumanlari, sonuc_coz as web_sonuc_coz, sonuc_yaz as web_sonuc_yaz,
};
use super::{EylemHatasi, GirdiCikti, VeritabaniHatasi};
use crate::agac::RotaErisimi;
use crate::guvenlik::sha256_hex;
use crate::web_guvenligi::WebReddi;
use std::collections::VecDeque;

const BASLIK: &str = "zee-io-izi\t1\n";
pub const AZAMI_IO_IZ_BAYTI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .io_izi()
    .bayt();
pub const AZAMI_IO_IZ_OLAYI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .io_izi()
    .olay();
const AZAMI_ALAN_SAYISI: usize = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
    .io_izi()
    .alan();
const IZ_WEB_REDDI: &str = "IO izi web reddi yeniden oluşturulamadı";

#[derive(Debug, Clone, PartialEq, Eq)]
struct IzOlay {
    sira: u64,
    islem: String,
    argumanlar: Vec<String>,
    sonuc: Vec<String>,
}

fn gecerli_islem(islem: &str) -> bool {
    matches!(
        islem,
        "yazdir"
            | "sor"
            | "rastgele"
            | "dosya_oku"
            | "dosya_yaz"
            | "postgresql_oku"
            | "postgresql_degistir"
            | "simdi"
            | "argumanlar"
            | "http_getir"
            | "sunucu_kur"
            | "istek_al"
            | "yanit_gonder"
            | "durum_yaniti_gonder"
            | "yonlendir_gonder"
            | "cerez_yaz"
            | "cerez_sil"
            | "rota_guvenligini_denetle"
            | "csrf_belirteci"
            | "oturum_ac"
            | "oturum_kapat"
            | "parola_dogrula"
            | "eylem_baslat"
            | "eylem_tamamla"
            | "eylem_geri_al"
            | "sensor_acik_mi"
            | "isik_ayarla"
            | "bekle_ms"
            | "an_ms"
    )
}

fn hex_yaz(metin: &str) -> String {
    const RAKAMLAR: &[u8; 16] = b"0123456789abcdef";
    let mut cikti = String::with_capacity(metin.len().saturating_mul(2));
    for bayt in metin.as_bytes() {
        cikti.push(RAKAMLAR[(bayt >> 4) as usize] as char);
        cikti.push(RAKAMLAR[(bayt & 0x0f) as usize] as char);
    }
    cikti
}

fn hex_rakam(bayt: u8) -> Option<u8> {
    match bayt {
        b'0'..=b'9' => Some(bayt - b'0'),
        b'a'..=b'f' => Some(bayt - b'a' + 10),
        _ => None,
    }
}

fn hex_coz(metin: &str) -> Result<String, String> {
    if !metin.len().is_multiple_of(2) {
        return Err("hex alan çift sayıda karakter taşımalı".into());
    }
    let mut baytlar = Vec::with_capacity(metin.len() / 2);
    for cift in metin.as_bytes().chunks_exact(2) {
        let yuksek = hex_rakam(cift[0]).ok_or_else(|| "hex alan küçük harfli değil".to_string())?;
        let dusuk = hex_rakam(cift[1]).ok_or_else(|| "hex alan küçük harfli değil".to_string())?;
        baytlar.push((yuksek << 4) | dusuk);
    }
    String::from_utf8(baytlar).map_err(|_| "hex alan geçerli UTF-8 değil".into())
}

fn olaylari_yaz(olaylar: &[IzOlay]) -> Result<String, String> {
    if olaylar.len() > AZAMI_IO_IZ_OLAYI {
        return Err(format!("IO izi {} olay sınırını aşıyor", AZAMI_IO_IZ_OLAYI));
    }
    let mut cikti = String::from(BASLIK);
    for (indis, olay) in olaylar.iter().enumerate() {
        let beklenen = (indis as u64).saturating_add(1);
        if olay.sira != beklenen || !gecerli_islem(&olay.islem) {
            return Err(format!("IO izi {}. olay kimliği geçersiz", beklenen));
        }
        if olay.argumanlar.len().saturating_add(olay.sonuc.len()) > AZAMI_ALAN_SAYISI {
            return Err(format!("IO izi {}. olay çok fazla alan taşıyor", beklenen));
        }
        let mut alanlar = vec![
            olay.sira.to_string(),
            olay.islem.clone(),
            olay.argumanlar.len().to_string(),
            olay.sonuc.len().to_string(),
        ];
        alanlar.extend(olay.argumanlar.iter().map(|alan| hex_yaz(alan)));
        alanlar.extend(olay.sonuc.iter().map(|alan| hex_yaz(alan)));
        cikti.push_str(&alanlar.join("\t"));
        cikti.push('\n');
        if cikti.len() > AZAMI_IO_IZ_BAYTI {
            return Err(format!(
                "IO izi {} MiB sınırını aşıyor",
                AZAMI_IO_IZ_BAYTI / 1024 / 1024
            ));
        }
    }
    Ok(cikti)
}

fn kanonik_sayi(metin: &str, ad: &str) -> Result<usize, String> {
    let sayi = metin
        .parse::<usize>()
        .map_err(|_| format!("{ad} sayı olmalı"))?;
    if sayi.to_string() != metin {
        return Err(format!("{ad} kanonik onluk biçimde olmalı"));
    }
    Ok(sayi)
}

fn olaylari_oku(metin: &str) -> Result<VecDeque<IzOlay>, String> {
    if metin.len() > AZAMI_IO_IZ_BAYTI {
        return Err(format!(
            "IO izi {} MiB sınırını aşıyor",
            AZAMI_IO_IZ_BAYTI / 1024 / 1024
        ));
    }
    let govde = metin
        .strip_prefix(BASLIK)
        .ok_or_else(|| "IO izi başlığı `zee-io-izi\\t1` olmalı".to_string())?;
    if !metin.ends_with('\n') {
        return Err("IO izi son satırı yeni satırla bitmeli".into());
    }
    let mut olaylar = Vec::new();
    for (indis, satir) in govde.lines().enumerate() {
        if satir.is_empty() {
            return Err(format!("IO izi {}. olay satırı boş", indis + 1));
        }
        if olaylar.len() >= AZAMI_IO_IZ_OLAYI {
            return Err(format!("IO izi {} olay sınırını aşıyor", AZAMI_IO_IZ_OLAYI));
        }
        let hucreler = satir.split('\t').collect::<Vec<_>>();
        if hucreler.len() < 4 {
            return Err(format!("IO izi {}. olay başlığı eksik", indis + 1));
        }
        let sira_usize = kanonik_sayi(hucreler[0], "olay sırası")?;
        let beklenen = indis.saturating_add(1);
        if sira_usize != beklenen {
            return Err(format!(
                "IO izi olay sırası {} olmalı, {} bulundu",
                beklenen, sira_usize
            ));
        }
        let islem = hucreler[1];
        if !gecerli_islem(islem) {
            return Err(format!(
                "IO izi {}. olay işlemi bilinmiyor: {}",
                beklenen, islem
            ));
        }
        let arguman_sayisi = kanonik_sayi(hucreler[2], "argüman sayısı")?;
        let sonuc_sayisi = kanonik_sayi(hucreler[3], "sonuç sayısı")?;
        let alan_sayisi = arguman_sayisi.saturating_add(sonuc_sayisi);
        if alan_sayisi > AZAMI_ALAN_SAYISI || hucreler.len() != 4 + alan_sayisi {
            return Err(format!("IO izi {}. olay alan sayısı uyuşmuyor", beklenen));
        }
        let alanlar = hucreler[4..]
            .iter()
            .map(|alan| hex_coz(alan))
            .collect::<Result<Vec<_>, _>>()?;
        let (argumanlar, sonuc) = alanlar.split_at(arguman_sayisi);
        let olay = IzOlay {
            sira: sira_usize as u64,
            islem: islem.to_string(),
            argumanlar: argumanlar.to_vec(),
            sonuc: sonuc.to_vec(),
        };
        olay_semasini_denetle(&olay)
            .map_err(|hata| format!("IO izi {}. olay şeması bozuk: {}", beklenen, hata))?;
        olaylar.push(olay);
    }
    let yeniden = olaylari_yaz(&olaylar)?;
    if yeniden != metin {
        return Err("IO izi kanonik byte biçiminde değil".into());
    }
    Ok(olaylar.into())
}

fn bool_yaz(deger: bool) -> String {
    if deger { "1" } else { "0" }.into()
}

fn bool_coz(metin: &str) -> Result<bool, String> {
    match metin {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(format!("boolean alan 0/1 olmalı, `{metin}` bulundu")),
    }
}

fn secenek_yaz(deger: Option<&str>) -> Vec<String> {
    match deger {
        Some(deger) => vec!["var".into(), deger.into()],
        None => vec!["yok".into()],
    }
}

fn secenek_coz(alanlar: &[String]) -> Result<Option<String>, String> {
    match alanlar {
        [etiket] if etiket == "yok" => Ok(None),
        [etiket, deger] if etiket == "var" => Ok(Some(deger.clone())),
        _ => Err("seçenek sonucu `yok` veya `var,<değer>` olmalı".into()),
    }
}

fn birim_sonuc_yaz(sonuc: &Result<(), String>) -> Vec<String> {
    match sonuc {
        Ok(()) => vec!["ok".into()],
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

fn birim_sonuc_coz(alanlar: &[String]) -> Result<Result<(), String>, String> {
    match alanlar {
        [etiket] if etiket == "ok" => Ok(Ok(())),
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        _ => Err("birim sonuç `ok` veya `hata,<mesaj>` olmalı".into()),
    }
}

fn metin_sonuc_yaz(sonuc: &Result<String, String>) -> Vec<String> {
    match sonuc {
        Ok(deger) => vec!["ok".into(), deger.clone()],
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

fn metin_sonuc_coz(alanlar: &[String]) -> Result<Result<String, String>, String> {
    match alanlar {
        [etiket, deger] if etiket == "ok" => Ok(Ok(deger.clone())),
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        _ => Err("metin sonuç `ok,<değer>` veya `hata,<mesaj>` olmalı".into()),
    }
}

fn kanonik_tamsayi<T>(metin: &str) -> bool
where
    T: std::str::FromStr + ToString,
{
    metin
        .parse::<T>()
        .ok()
        .is_some_and(|deger| deger.to_string() == metin)
}

fn alan_sayisi(olay: &IzOlay, arguman: usize, sonuc: usize) -> Result<(), String> {
    if olay.argumanlar.len() != arguman || olay.sonuc.len() != sonuc {
        return Err(format!(
            "`{}` {} argüman/{} sonuç alanı bekler",
            olay.islem, arguman, sonuc
        ));
    }
    Ok(())
}

fn birim_semasi(olay: &IzOlay, arguman: usize) -> Result<(), String> {
    if olay.argumanlar.len() != arguman || birim_sonuc_coz(&olay.sonuc).is_err() {
        return Err(format!("`{}` birim sonuç şeması geçersiz", olay.islem));
    }
    Ok(())
}

fn secenek_semasi(olay: &IzOlay, arguman: usize) -> Result<(), String> {
    if olay.argumanlar.len() != arguman || secenek_coz(&olay.sonuc).is_err() {
        return Err(format!("`{}` seçenek sonuç şeması geçersiz", olay.islem));
    }
    Ok(())
}

fn metin_sonuc_semasi(olay: &IzOlay, arguman: usize) -> Result<(), String> {
    if olay.argumanlar.len() != arguman || metin_sonuc_coz(&olay.sonuc).is_err() {
        return Err(format!("`{}` metin sonuç şeması geçersiz", olay.islem));
    }
    Ok(())
}

fn olay_semasini_denetle(olay: &IzOlay) -> Result<(), String> {
    match olay.islem.as_str() {
        "yazdir" | "yanit_gonder" => alan_sayisi(olay, 1, 0),
        "durum_yaniti_gonder" => {
            alan_sayisi(olay, 2, 0)?;
            if kanonik_tamsayi::<u16>(&olay.argumanlar[0]) {
                Ok(())
            } else {
                Err("HTTP durum kodu kanonik u16 olmalı".into())
            }
        }
        "sor" => secenek_semasi(olay, 1),
        "rastgele" => {
            alan_sayisi(olay, 2, 1)?;
            if olay
                .argumanlar
                .iter()
                .chain(&olay.sonuc)
                .all(|alan| kanonik_tamsayi::<i64>(alan))
            {
                Ok(())
            } else {
                Err("rastgele alanları kanonik i64 olmalı".into())
            }
        }
        "dosya_oku" | "csrf_belirteci" => {
            metin_sonuc_semasi(olay, usize::from(olay.islem == "dosya_oku"))
        }
        "dosya_yaz" => {
            birim_semasi(olay, 3)?;
            bool_coz(&olay.argumanlar[2]).map(|_| ())
        }
        "postgresql_oku" | "postgresql_degistir" => {
            if olay.argumanlar.is_empty() {
                return Err("PostgreSQL izi sorgu argümanı taşımalı".into());
            }
            if olay.islem == "postgresql_oku" {
                veritabani_okuma_coz(&olay.sonuc).map(|_| ())
            } else {
                veritabani_degistirme_coz(&olay.sonuc).map(|_| ())
            }
        }
        "simdi" => {
            alan_sayisi(olay, 0, 5)?;
            let gecerli = kanonik_tamsayi::<i64>(&olay.sonuc[0])
                && olay.sonuc[1..]
                    .iter()
                    .all(|alan| kanonik_tamsayi::<u32>(alan));
            if gecerli {
                Ok(())
            } else {
                Err("zaman alanları kanonik sayı olmalı".into())
            }
        }
        "argumanlar" => {
            if olay.argumanlar.is_empty() {
                Ok(())
            } else {
                Err("`argumanlar` çağrısı argüman taşıyamaz".into())
            }
        }
        "http_getir" => {
            let arguman_gecerli = match olay.argumanlar.as_slice() {
                [_, etiket] if etiket == "yok" => true,
                [_, etiket, sure] if etiket == "var" && kanonik_tamsayi::<i64>(sure) => true,
                _ => false,
            };
            let sonuc_gecerli = match olay.sonuc.as_slice() {
                [etiket, durum, _] if etiket == "ok" && kanonik_tamsayi::<i64>(durum) => true,
                [etiket, _] if etiket == "hata" => true,
                _ => false,
            };
            if arguman_gecerli && sonuc_gecerli {
                Ok(())
            } else {
                Err("HTTP argüman/sonuç şeması geçersiz".into())
            }
        }
        "sunucu_kur" => {
            birim_semasi(olay, 1)?;
            if kanonik_tamsayi::<i64>(&olay.argumanlar[0]) {
                Ok(())
            } else {
                Err("sunucu kapısı kanonik i64 olmalı".into())
            }
        }
        "istek_al" => secenek_semasi(olay, 0),
        "yonlendir_gonder" | "cerez_sil" => birim_semasi(olay, 1),
        "cerez_yaz" | "oturum_ac" => birim_semasi(olay, 2),
        "rota_guvenligini_denetle" => {
            if rota_arguman_semasi(&olay.argumanlar) && web_sonuc_coz(&olay.sonuc).is_ok() {
                Ok(())
            } else {
                Err("rota güvenliği argüman/sonuç şeması geçersiz".into())
            }
        }
        "oturum_kapat" => birim_semasi(olay, 0),
        "eylem_baslat" | "eylem_tamamla" | "eylem_geri_al" => eylem_semasi(olay),
        "parola_dogrula" => {
            alan_sayisi(olay, 2, 1)?;
            let ozetler = olay.argumanlar.iter().all(|alan| {
                alan.len() == 64
                    && alan
                        .bytes()
                        .all(|bayt| matches!(bayt, b'0'..=b'9' | b'a'..=b'f'))
            });
            if ozetler && bool_coz(&olay.sonuc[0]).is_ok() {
                Ok(())
            } else {
                Err("parola doğrulama parmak izi/sonuç şeması geçersiz".into())
            }
        }
        "sensor_acik_mi" => {
            alan_sayisi(olay, 1, 1)?;
            bool_coz(&olay.sonuc[0]).map(|_| ())
        }
        "isik_ayarla" => {
            alan_sayisi(olay, 2, 0)?;
            bool_coz(&olay.argumanlar[1]).map(|_| ())
        }
        "bekle_ms" => {
            alan_sayisi(olay, 1, 0)?;
            if kanonik_tamsayi::<i64>(&olay.argumanlar[0]) {
                Ok(())
            } else {
                Err("bekleme alanı kanonik i64 olmalı".into())
            }
        }
        "an_ms" => {
            alan_sayisi(olay, 0, 1)?;
            if kanonik_tamsayi::<i64>(&olay.sonuc[0]) {
                Ok(())
            } else {
                Err("an ölçümü kanonik i64 olmalı".into())
            }
        }
        _ => Err(format!("bilinmeyen işlem: {}", olay.islem)),
    }
}

/// Bir IO adaptörünü çağırırken bütün olayları sıralı şema-1 izine kaydeder.
pub struct IzKaydedenIo<T: GirdiCikti> {
    pub ic: T,
    olaylar: Vec<IzOlay>,
    yaklasik_bayt: usize,
    hata: Option<String>,
}

impl<T: GirdiCikti> IzKaydedenIo<T> {
    pub fn yeni(ic: T) -> Self {
        Self {
            ic,
            olaylar: Vec::new(),
            yaklasik_bayt: BASLIK.len(),
            hata: None,
        }
    }

    pub fn olay_sayisi(&self) -> usize {
        self.olaylar.len()
    }

    pub fn iz_metni(&self) -> Result<String, String> {
        if let Some(hata) = &self.hata {
            return Err(hata.clone());
        }
        olaylari_yaz(&self.olaylar)
    }

    pub fn icine_al(self) -> T {
        self.ic
    }

    fn kaydet(&mut self, islem: &str, argumanlar: Vec<String>, sonuc: Vec<String>) {
        if self.hata.is_some() {
            return;
        }
        if self.olaylar.len() >= AZAMI_IO_IZ_OLAYI {
            self.hata = Some(format!("IO izi {} olay sınırını aşıyor", AZAMI_IO_IZ_OLAYI));
            return;
        }
        if argumanlar.len().saturating_add(sonuc.len()) > AZAMI_ALAN_SAYISI {
            self.hata = Some(format!("IO izi `{islem}` olayı çok fazla alan taşıyor"));
            return;
        }
        let olay_bayti = islem
            .len()
            .saturating_add(
                argumanlar
                    .iter()
                    .map(String::len)
                    .sum::<usize>()
                    .saturating_mul(2),
            )
            .saturating_add(
                sonuc
                    .iter()
                    .map(String::len)
                    .sum::<usize>()
                    .saturating_mul(2),
            )
            .saturating_add(64);
        self.yaklasik_bayt = self.yaklasik_bayt.saturating_add(olay_bayti);
        if self.yaklasik_bayt > AZAMI_IO_IZ_BAYTI {
            self.hata = Some(format!(
                "IO izi {} MiB sınırını aşıyor",
                AZAMI_IO_IZ_BAYTI / 1024 / 1024
            ));
            return;
        }
        self.olaylar.push(IzOlay {
            sira: (self.olaylar.len() as u64).saturating_add(1),
            islem: islem.into(),
            argumanlar,
            sonuc,
        });
    }
}

impl<T: GirdiCikti> GirdiCikti for IzKaydedenIo<T> {
    fn yazdir(&mut self, satir: String) {
        self.ic.yazdir(satir.clone());
        self.kaydet("yazdir", vec![satir], Vec::new());
    }

    fn sor(&mut self, istem: &str) -> Option<String> {
        let sonuc = self.ic.sor(istem);
        self.kaydet("sor", vec![istem.into()], secenek_yaz(sonuc.as_deref()));
        sonuc
    }

    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        let sonuc = self.ic.rastgele(alt, ust);
        self.kaydet(
            "rastgele",
            vec![alt.to_string(), ust.to_string()],
            vec![sonuc.to_string()],
        );
        sonuc
    }

    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        let sonuc = self.ic.dosya_oku(yol);
        self.kaydet("dosya_oku", vec![yol.into()], metin_sonuc_yaz(&sonuc));
        sonuc
    }

    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        let sonuc = self.ic.dosya_yaz(yol, satir, ekleme);
        self.kaydet(
            "dosya_yaz",
            vec![yol.into(), satir.into(), bool_yaz(ekleme)],
            birim_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn postgresql_oku(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        let sonuc = self.ic.postgresql_oku(sorgu, parametreler);
        let mut argumanlar = vec![sorgu.into()];
        argumanlar.extend(parametreler.iter().cloned());
        self.kaydet("postgresql_oku", argumanlar, veritabani_okuma_yaz(&sonuc));
        sonuc
    }

    fn postgresql_degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<i64, VeritabaniHatasi> {
        let sonuc = self.ic.postgresql_degistir(sorgu, parametreler);
        let mut argumanlar = vec![sorgu.into()];
        argumanlar.extend(parametreler.iter().cloned());
        self.kaydet(
            "postgresql_degistir",
            argumanlar,
            veritabani_degistirme_yaz(&sonuc),
        );
        sonuc
    }

    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        let sonuc = self.ic.simdi();
        self.kaydet(
            "simdi",
            Vec::new(),
            vec![
                sonuc.0.to_string(),
                sonuc.1.to_string(),
                sonuc.2.to_string(),
                sonuc.3.to_string(),
                sonuc.4.to_string(),
            ],
        );
        sonuc
    }

    fn argumanlar(&mut self) -> Vec<String> {
        let sonuc = self.ic.argumanlar();
        self.kaydet("argumanlar", Vec::new(), sonuc.clone());
        sonuc
    }

    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        let sonuc = self.ic.http_getir(url, zaman_asimi_ms);
        let mut argumanlar = vec![url.into()];
        argumanlar.extend(secenek_yaz(
            zaman_asimi_ms.map(|deger| deger.to_string()).as_deref(),
        ));
        let iz_sonucu = match &sonuc {
            Ok((durum, govde)) => vec!["ok".into(), durum.to_string(), govde.clone()],
            Err(hata) => vec!["hata".into(), hata.clone()],
        };
        self.kaydet("http_getir", argumanlar, iz_sonucu);
        sonuc
    }

    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String> {
        let sonuc = self.ic.sunucu_kur(kapi);
        self.kaydet(
            "sunucu_kur",
            vec![kapi.to_string()],
            birim_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn istek_al(&mut self) -> Option<String> {
        let sonuc = self.ic.istek_al();
        self.kaydet("istek_al", Vec::new(), secenek_yaz(sonuc.as_deref()));
        sonuc
    }

    // İstek transaction'ı host adaptörünün sahiplik ayrıntısıdır; zee-io-1
    // gözlem olayına dönüşmez, yalnız iç adaptöre iletilir.
    fn istek_islemini_tamamla(&mut self) -> Result<(), String> {
        self.ic.istek_islemini_tamamla()
    }

    fn istek_islemini_geri_al(&mut self) {
        self.ic.istek_islemini_geri_al();
    }

    fn yanit_gonder(&mut self, yanit: &str) {
        self.ic.yanit_gonder(yanit);
        self.kaydet("yanit_gonder", vec![yanit.into()], Vec::new());
    }

    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        self.ic.durum_yaniti_gonder(durum, yanit);
        self.kaydet(
            "durum_yaniti_gonder",
            vec![durum.to_string(), yanit.into()],
            Vec::new(),
        );
    }

    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        let sonuc = self.ic.yonlendir_gonder(adres);
        self.kaydet(
            "yonlendir_gonder",
            vec![adres.into()],
            birim_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        let sonuc = self.ic.cerez_yaz(ad, deger);
        self.kaydet(
            "cerez_yaz",
            vec![ad.into(), deger.into()],
            birim_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        let sonuc = self.ic.cerez_sil(ad);
        self.kaydet("cerez_sil", vec![ad.into()], birim_sonuc_yaz(&sonuc));
        sonuc
    }

    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        let sonuc = self.ic.rota_guvenligini_denetle(erisim, csrf, csrf_gerekli);
        self.kaydet(
            "rota_guvenligini_denetle",
            rota_argumanlari(erisim, csrf, csrf_gerekli),
            web_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn csrf_belirteci(&mut self) -> Result<String, String> {
        let sonuc = self.ic.csrf_belirteci();
        self.kaydet("csrf_belirteci", Vec::new(), metin_sonuc_yaz(&sonuc));
        sonuc
    }

    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        let sonuc = self.ic.oturum_ac(kullanici, rol);
        self.kaydet(
            "oturum_ac",
            vec![kullanici.into(), rol.into()],
            birim_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn oturum_kapat(&mut self) -> Result<(), String> {
        let sonuc = self.ic.oturum_kapat();
        self.kaydet("oturum_kapat", Vec::new(), birim_sonuc_yaz(&sonuc));
        sonuc
    }

    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        let sonuc = self.ic.parola_dogrula(parola, ozet);
        self.kaydet(
            "parola_dogrula",
            vec![sha256_hex(parola.as_bytes()), sha256_hex(ozet.as_bytes())],
            vec![bool_yaz(sonuc)],
        );
        sonuc
    }

    fn eylem_baslat(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.ic.eylem_baslat();
        self.kaydet("eylem_baslat", Vec::new(), eylem_sonuc_yaz(&sonuc));
        sonuc
    }

    fn eylem_tamamla(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.ic.eylem_tamamla();
        self.kaydet("eylem_tamamla", Vec::new(), eylem_sonuc_yaz(&sonuc));
        sonuc
    }

    fn eylem_geri_al(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.ic.eylem_geri_al();
        self.kaydet("eylem_geri_al", Vec::new(), eylem_sonuc_yaz(&sonuc));
        sonuc
    }

    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        let sonuc = self.ic.sensor_acik_mi(ad);
        self.kaydet("sensor_acik_mi", vec![ad.into()], vec![bool_yaz(sonuc)]);
        sonuc
    }

    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        self.ic.isik_ayarla(ad, yansin);
        self.kaydet("isik_ayarla", vec![ad.into(), bool_yaz(yansin)], Vec::new());
    }

    fn bekle_ms(&mut self, milisaniye: i64) {
        self.ic.bekle_ms(milisaniye);
        self.kaydet("bekle_ms", vec![milisaniye.to_string()], Vec::new());
    }

    fn an_ms(&mut self) -> i64 {
        let sonuc = self.ic.an_ms();
        self.kaydet("an_ms", Vec::new(), vec![sonuc.to_string()]);
        sonuc
    }
}

/// Şema-1 izini dış IO olmadan, işlem ve argümanları eşleştirerek yeniden oynatır.
pub struct IzYenidenOynatici {
    olaylar: VecDeque<IzOlay>,
    hata: Option<String>,
    pub cikti: Vec<String>,
}

impl IzYenidenOynatici {
    pub fn yeni(metin: &str) -> Result<Self, String> {
        Ok(Self {
            olaylar: olaylari_oku(metin)?,
            hata: None,
            cikti: Vec::new(),
        })
    }

    pub fn uyusmazlik(&self) -> Option<&str> {
        self.hata.as_deref()
    }

    pub fn bitir(self) -> Result<(), String> {
        if let Some(hata) = self.hata {
            return Err(hata);
        }
        if let Some(olay) = self.olaylar.front() {
            return Err(format!(
                "IO izi tamamlanmadı: sıradaki olay {} `{}`",
                olay.sira, olay.islem
            ));
        }
        Ok(())
    }

    fn hata_yaz(&mut self, hata: String) {
        if self.hata.is_none() {
            self.hata = Some(hata);
        }
    }

    fn siradaki(&mut self, islem: &str, argumanlar: Vec<String>) -> Option<Vec<String>> {
        if self.hata.is_some() {
            return None;
        }
        let Some(olay) = self.olaylar.pop_front() else {
            self.hata_yaz(format!(
                "IO izi erken bitti; `{islem}` çağrısı beklenmiyordu"
            ));
            return None;
        };
        if olay.islem != islem {
            self.hata_yaz(format!(
                "IO izi {}. olayda `{}` bekliyordu, `{}` çağrıldı",
                olay.sira, olay.islem, islem
            ));
            return None;
        }
        if olay.argumanlar != argumanlar {
            self.hata_yaz(format!(
                "IO izi {}. `{islem}` argümanları uyuşmuyor",
                olay.sira
            ));
            return None;
        }
        Some(olay.sonuc)
    }

    fn bozuk_sonuc(&mut self, islem: &str, hata: String) {
        self.hata_yaz(format!("IO izi `{islem}` sonucu bozuk: {hata}"));
    }

    fn bos_sonuc(&mut self, islem: &str, sonuc: Option<Vec<String>>) -> bool {
        match sonuc {
            Some(alanlar) if alanlar.is_empty() => true,
            Some(_) => {
                self.bozuk_sonuc(islem, "sonuç alanı olmamalı".into());
                false
            }
            None => false,
        }
    }

    fn birim_sonuc(&mut self, islem: &str, sonuc: Option<Vec<String>>) -> Result<(), String> {
        let Some(alanlar) = sonuc else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        match birim_sonuc_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc(islem, hata);
                Err(self
                    .hata
                    .clone()
                    .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
            }
        }
    }

    fn metin_sonuc(&mut self, islem: &str, sonuc: Option<Vec<String>>) -> Result<String, String> {
        let Some(alanlar) = sonuc else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        match metin_sonuc_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc(islem, hata);
                Err(self
                    .hata
                    .clone()
                    .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
            }
        }
    }

    fn sayi_sonuc<T>(&mut self, islem: &str, sonuc: Option<Vec<String>>, varsayilan: T) -> T
    where
        T: std::str::FromStr,
    {
        let Some(alanlar) = sonuc else {
            return varsayilan;
        };
        let [deger] = alanlar.as_slice() else {
            self.bozuk_sonuc(islem, "tek sayı alanı bekleniyor".into());
            return varsayilan;
        };
        match deger.parse() {
            Ok(deger) => deger,
            Err(_) => {
                self.bozuk_sonuc(islem, format!("`{deger}` sayı değil"));
                varsayilan
            }
        }
    }

    fn bool_sonuc(&mut self, islem: &str, sonuc: Option<Vec<String>>) -> bool {
        let Some(alanlar) = sonuc else {
            return false;
        };
        let [deger] = alanlar.as_slice() else {
            self.bozuk_sonuc(islem, "tek boolean alanı bekleniyor".into());
            return false;
        };
        match bool_coz(deger) {
            Ok(deger) => deger,
            Err(hata) => {
                self.bozuk_sonuc(islem, hata);
                false
            }
        }
    }
}

impl GirdiCikti for IzYenidenOynatici {
    fn yazdir(&mut self, satir: String) {
        let sonuc = self.siradaki("yazdir", vec![satir.clone()]);
        if self.bos_sonuc("yazdir", sonuc) {
            self.cikti.push(satir);
        }
    }

    fn sor(&mut self, istem: &str) -> Option<String> {
        let sonuc = self.siradaki("sor", vec![istem.into()]);
        let alanlar = sonuc?;
        match secenek_coz(&alanlar) {
            Ok(sonuc) => {
                self.cikti.push(istem.into());
                sonuc
            }
            Err(hata) => {
                self.bozuk_sonuc("sor", hata);
                None
            }
        }
    }

    fn rastgele(&mut self, alt: i64, ust: i64) -> i64 {
        let sonuc = self.siradaki("rastgele", vec![alt.to_string(), ust.to_string()]);
        self.sayi_sonuc("rastgele", sonuc, alt)
    }

    fn dosya_oku(&mut self, yol: &str) -> Result<String, String> {
        let sonuc = self.siradaki("dosya_oku", vec![yol.into()]);
        self.metin_sonuc("dosya_oku", sonuc)
    }

    fn dosya_yaz(&mut self, yol: &str, satir: &str, ekleme: bool) -> Result<(), String> {
        let sonuc = self.siradaki(
            "dosya_yaz",
            vec![yol.into(), satir.into(), bool_yaz(ekleme)],
        );
        self.birim_sonuc("dosya_yaz", sonuc)
    }

    fn postgresql_oku(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<Vec<Vec<(String, String)>>, VeritabaniHatasi> {
        let mut argumanlar = vec![sorgu.into()];
        argumanlar.extend(parametreler.iter().cloned());
        let Some(alanlar) = self.siradaki("postgresql_oku", argumanlar) else {
            return Err(VeritabaniHatasi {
                mesaj: self
                    .hata
                    .clone()
                    .unwrap_or_else(|| "IO izi uyuşmazlığı".into()),
                veri: Vec::new(),
            });
        };
        match veritabani_okuma_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc("postgresql_oku", hata);
                Err(VeritabaniHatasi {
                    mesaj: self
                        .hata
                        .clone()
                        .unwrap_or_else(|| "IO izi sonucu bozuk".into()),
                    veri: Vec::new(),
                })
            }
        }
    }

    fn postgresql_degistir(
        &mut self,
        sorgu: &str,
        parametreler: &[String],
    ) -> Result<i64, VeritabaniHatasi> {
        let mut argumanlar = vec![sorgu.into()];
        argumanlar.extend(parametreler.iter().cloned());
        let Some(alanlar) = self.siradaki("postgresql_degistir", argumanlar) else {
            return Err(VeritabaniHatasi {
                mesaj: self
                    .hata
                    .clone()
                    .unwrap_or_else(|| "IO izi uyuşmazlığı".into()),
                veri: Vec::new(),
            });
        };
        match veritabani_degistirme_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc("postgresql_degistir", hata);
                Err(VeritabaniHatasi {
                    mesaj: self
                        .hata
                        .clone()
                        .unwrap_or_else(|| "IO izi sonucu bozuk".into()),
                    veri: Vec::new(),
                })
            }
        }
    }

    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        let sonuc = self.siradaki("simdi", Vec::new());
        let Some(alanlar) = sonuc else {
            return (0, 1, 1, 0, 0);
        };
        let [yil, ay, gun, saat, dakika] = alanlar.as_slice() else {
            self.bozuk_sonuc("simdi", "beş zaman alanı bekleniyor".into());
            return (0, 1, 1, 0, 0);
        };
        match (
            yil.parse(),
            ay.parse(),
            gun.parse(),
            saat.parse(),
            dakika.parse(),
        ) {
            (Ok(yil), Ok(ay), Ok(gun), Ok(saat), Ok(dakika)) => (yil, ay, gun, saat, dakika),
            _ => {
                self.bozuk_sonuc("simdi", "zaman alanlarından biri sayı değil".into());
                (0, 1, 1, 0, 0)
            }
        }
    }

    fn argumanlar(&mut self) -> Vec<String> {
        self.siradaki("argumanlar", Vec::new()).unwrap_or_default()
    }

    fn http_getir(
        &mut self,
        url: &str,
        zaman_asimi_ms: Option<i64>,
    ) -> Result<(i64, String), String> {
        let mut argumanlar = vec![url.into()];
        argumanlar.extend(secenek_yaz(
            zaman_asimi_ms.map(|deger| deger.to_string()).as_deref(),
        ));
        let Some(alanlar) = self.siradaki("http_getir", argumanlar) else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        match alanlar.as_slice() {
            [etiket, durum, govde] if etiket == "ok" => match durum.parse() {
                Ok(durum) => Ok((durum, govde.clone())),
                Err(_) => {
                    self.bozuk_sonuc("http_getir", "durum kodu sayı değil".into());
                    Err(self
                        .hata
                        .clone()
                        .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
                }
            },
            [etiket, hata] if etiket == "hata" => Err(hata.clone()),
            _ => {
                self.bozuk_sonuc("http_getir", "geçersiz HTTP sonuç alanları".into());
                Err(self
                    .hata
                    .clone()
                    .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
            }
        }
    }

    fn sunucu_kur(&mut self, kapi: i64) -> Result<(), String> {
        let sonuc = self.siradaki("sunucu_kur", vec![kapi.to_string()]);
        self.birim_sonuc("sunucu_kur", sonuc)
    }

    fn istek_al(&mut self) -> Option<String> {
        let alanlar = self.siradaki("istek_al", Vec::new())?;
        match secenek_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc("istek_al", hata);
                None
            }
        }
    }

    fn yanit_gonder(&mut self, yanit: &str) {
        let sonuc = self.siradaki("yanit_gonder", vec![yanit.into()]);
        self.bos_sonuc("yanit_gonder", sonuc);
    }

    fn durum_yaniti_gonder(&mut self, durum: u16, yanit: &str) {
        let sonuc = self.siradaki("durum_yaniti_gonder", vec![durum.to_string(), yanit.into()]);
        self.bos_sonuc("durum_yaniti_gonder", sonuc);
    }

    fn yonlendir_gonder(&mut self, adres: &str) -> Result<(), String> {
        let sonuc = self.siradaki("yonlendir_gonder", vec![adres.into()]);
        self.birim_sonuc("yonlendir_gonder", sonuc)
    }

    fn cerez_yaz(&mut self, ad: &str, deger: &str) -> Result<(), String> {
        let sonuc = self.siradaki("cerez_yaz", vec![ad.into(), deger.into()]);
        self.birim_sonuc("cerez_yaz", sonuc)
    }

    fn cerez_sil(&mut self, ad: &str) -> Result<(), String> {
        let sonuc = self.siradaki("cerez_sil", vec![ad.into()]);
        self.birim_sonuc("cerez_sil", sonuc)
    }

    fn rota_guvenligini_denetle(
        &mut self,
        erisim: &RotaErisimi,
        csrf: Option<&str>,
        csrf_gerekli: bool,
    ) -> Result<(), WebReddi> {
        let Some(alanlar) = self.siradaki(
            "rota_guvenligini_denetle",
            rota_argumanlari(erisim, csrf, csrf_gerekli),
        ) else {
            return Err(WebReddi {
                durum: 500,
                mesaj: IZ_WEB_REDDI,
            });
        };
        match web_sonuc_coz(&alanlar) {
            Ok(sonuc) => sonuc,
            Err(hata) => {
                self.bozuk_sonuc("rota_guvenligini_denetle", hata);
                Err(WebReddi {
                    durum: 500,
                    mesaj: IZ_WEB_REDDI,
                })
            }
        }
    }

    fn csrf_belirteci(&mut self) -> Result<String, String> {
        let sonuc = self.siradaki("csrf_belirteci", Vec::new());
        self.metin_sonuc("csrf_belirteci", sonuc)
    }

    fn oturum_ac(&mut self, kullanici: &str, rol: &str) -> Result<(), String> {
        let sonuc = self.siradaki("oturum_ac", vec![kullanici.into(), rol.into()]);
        self.birim_sonuc("oturum_ac", sonuc)
    }

    fn oturum_kapat(&mut self) -> Result<(), String> {
        let sonuc = self.siradaki("oturum_kapat", Vec::new());
        self.birim_sonuc("oturum_kapat", sonuc)
    }

    fn parola_dogrula(&mut self, parola: &str, ozet: &str) -> bool {
        let sonuc = self.siradaki(
            "parola_dogrula",
            vec![sha256_hex(parola.as_bytes()), sha256_hex(ozet.as_bytes())],
        );
        self.bool_sonuc("parola_dogrula", sonuc)
    }

    fn eylem_baslat(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.siradaki("eylem_baslat", Vec::new());
        self.eylem_sonuc("eylem_baslat", sonuc)
    }

    fn eylem_tamamla(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.siradaki("eylem_tamamla", Vec::new());
        self.eylem_sonuc("eylem_tamamla", sonuc)
    }

    fn eylem_geri_al(&mut self) -> Result<(), EylemHatasi> {
        let sonuc = self.siradaki("eylem_geri_al", Vec::new());
        self.eylem_sonuc("eylem_geri_al", sonuc)
    }

    fn sensor_acik_mi(&mut self, ad: &str) -> bool {
        let sonuc = self.siradaki("sensor_acik_mi", vec![ad.into()]);
        self.bool_sonuc("sensor_acik_mi", sonuc)
    }

    fn isik_ayarla(&mut self, ad: &str, yansin: bool) {
        let sonuc = self.siradaki("isik_ayarla", vec![ad.into(), bool_yaz(yansin)]);
        if self.bos_sonuc("isik_ayarla", sonuc) {
            self.cikti.push(format!(
                "[ışık] {} {}",
                ad,
                if yansin { "yandı" } else { "söndü" }
            ));
        }
    }

    fn bekle_ms(&mut self, milisaniye: i64) {
        let sonuc = self.siradaki("bekle_ms", vec![milisaniye.to_string()]);
        self.bos_sonuc("bekle_ms", sonuc);
    }

    fn an_ms(&mut self) -> i64 {
        let sonuc = self.siradaki("an_ms", Vec::new());
        self.sayi_sonuc("an_ms", sonuc, 0)
    }
}
