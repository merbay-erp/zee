//! Sürümlü, kanonik ve sıralı IO trace/replay katmanı (K-115, ADR-026).
//! İz özel veridir; parola doğrulama argümanları yalnız SHA-256 parmak iziyle kaydedilir.

mod codec;
mod eylem;
mod http;
mod istemci;
mod iz_oynatici;
mod k181;
mod oturum;
mod tasima;

use self::codec::*;
use self::eylem::{semayi_denetle as eylem_semasi, sonuc_yaz as eylem_sonuc_yaz};
use self::oturum::{sonuc_coz as oturum_listesi_sonuc_coz, sonuc_yaz as oturum_listesi_sonuc_yaz};
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
            | "dosya_atomik_tasi"
            | "dosya_sil"
            | "dosyalari_listele"
            | "dosya_sha256"
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
            | "istemci_adresi"
            | "csrf_belirteci"
            | "oturum_ac"
            | "oturum_kapat"
            | "kullanici_oturumlarini_listele"
            | "oturum_kimligini_iptal_et"
            | "parola_dogrula"
            | "parola_ozeti_uret"
            | "gizli_sifrele"
            | "gizli_coz"
            | "totp_anahtari_uret"
            | "totp_dogrula"
            | "eposta_gonder"
            | "gorsel_kucult"
            | "eylem_baslat"
            | "eylem_tamamla"
            | "eylem_geri_al"
            | "sensor_acik_mi"
            | "isik_ayarla"
            | "bekle_ms"
            | "an_ms"
    )
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

fn tamsayi_sonuc_semasi(olay: &IzOlay, arguman: usize) -> Result<(), String> {
    if olay.argumanlar.len() != arguman || tamsayi_sonuc_coz(&olay.sonuc).is_err() {
        return Err(format!("`{}` tam sayı sonuç şeması geçersiz", olay.islem));
    }
    Ok(())
}

fn metin_listesi_sonuc_semasi(olay: &IzOlay, arguman: usize) -> Result<(), String> {
    if olay.argumanlar.len() != arguman || metin_listesi_sonuc_coz(&olay.sonuc).is_err() {
        return Err(format!(
            "`{}` metin listesi sonuç şeması geçersiz",
            olay.islem
        ));
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
        "istemci_adresi" => istemci::semayi_denetle(olay),
        "dosya_oku" | "csrf_belirteci" => {
            metin_sonuc_semasi(olay, usize::from(olay.islem == "dosya_oku"))
        }
        "dosya_yaz" => {
            birim_semasi(olay, 3)?;
            bool_coz(&olay.argumanlar[2]).map(|_| ())
        }
        "dosya_atomik_tasi" => tamsayi_sonuc_semasi(olay, 2),
        "dosya_sil" => tamsayi_sonuc_semasi(olay, 1),
        "dosyalari_listele" => metin_listesi_sonuc_semasi(olay, 1),
        "dosya_sha256" => metin_sonuc_semasi(olay, 1),
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
        "kullanici_oturumlarini_listele" => {
            if olay.argumanlar.len() == 1 && oturum_listesi_sonuc_coz(&olay.sonuc).is_ok() {
                Ok(())
            } else {
                Err("web oturum listesi argüman/sonuç şeması geçersiz".into())
            }
        }
        "oturum_kimligini_iptal_et" => tamsayi_sonuc_semasi(olay, 1),
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
        "parola_ozeti_uret" | "gizli_sifrele" | "gizli_coz" | "totp_anahtari_uret"
        | "totp_dogrula" | "eposta_gonder" | "gorsel_kucult" => k181::sema(olay),
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

    fn dosya_atomik_tasi(&mut self, kaynak: &str, hedef: &str) -> Result<i64, String> {
        let sonuc = self.ic.dosya_atomik_tasi(kaynak, hedef);
        self.kaydet(
            "dosya_atomik_tasi",
            vec![kaynak.into(), hedef.into()],
            tamsayi_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn dosya_sil(&mut self, yol: &str) -> Result<i64, String> {
        let sonuc = self.ic.dosya_sil(yol);
        self.kaydet("dosya_sil", vec![yol.into()], tamsayi_sonuc_yaz(&sonuc));
        sonuc
    }

    fn dosyalari_listele(&mut self, dizin: &str) -> Result<Vec<String>, String> {
        let sonuc = self.ic.dosyalari_listele(dizin);
        self.kaydet(
            "dosyalari_listele",
            vec![dizin.into()],
            metin_listesi_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn dosya_sha256(&mut self, yol: &str) -> Result<String, String> {
        let sonuc = self.ic.dosya_sha256(yol);
        self.kaydet("dosya_sha256", vec![yol.into()], metin_sonuc_yaz(&sonuc));
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

    http::yontemler!();

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

    fn istemci_adresi(&mut self) -> String {
        istemci::kaydet(self)
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

    fn kullanici_oturumlarini_listele(
        &mut self,
        kullanici: &str,
    ) -> Result<Vec<Vec<(String, String)>>, String> {
        let sonuc = self.ic.kullanici_oturumlarini_listele(kullanici);
        self.kaydet(
            "kullanici_oturumlarini_listele",
            vec![kullanici.into()],
            oturum_listesi_sonuc_yaz(&sonuc),
        );
        sonuc
    }

    fn oturum_kimligini_iptal_et(&mut self, kimlik: &str) -> Result<i64, String> {
        let sonuc = self.ic.oturum_kimligini_iptal_et(kimlik);
        self.kaydet(
            "oturum_kimligini_iptal_et",
            vec![kimlik.into()],
            tamsayi_sonuc_yaz(&sonuc),
        );
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

    fn parola_ozeti_uret(&mut self, parola: &str) -> Result<String, String> {
        k181::kaydet_parola_ozeti_uret(self, parola)
    }
    fn gizli_sifrele(&mut self, duz: &str) -> Result<String, String> {
        k181::kaydet_gizli_sifrele(self, duz)
    }
    fn gizli_coz(&mut self, sifreli: &str) -> Result<String, String> {
        k181::kaydet_gizli_coz(self, sifreli)
    }

    fn totp_anahtari_uret(&mut self) -> Result<String, String> {
        k181::kaydet_totp_anahtari_uret(self)
    }

    fn totp_dogrula(&mut self, gizli: &str, kod: &str) -> bool {
        k181::kaydet_totp_dogrula(self, gizli, kod)
    }

    fn eposta_gonder(
        &mut self,
        b: Option<&str>,
        alici: &str,
        konu: &str,
        govde: &str,
    ) -> Result<i64, String> {
        k181::kaydet_eposta_gonder(self, b, alici, konu, govde)
    }

    fn gorsel_kucult(&mut self, kaynak: &str, hedef: &str, azami: i64) -> Result<i64, String> {
        k181::kaydet_gorsel_kucult(self, kaynak, hedef, azami)
    }

    fn eposta_mime_gonder(
        &mut self,
        _b: Option<&str>,
        _a: &str,
        _k: &str,
        _g: &str,
        _s: &crate::eposta::MimeGonderim<'_>,
    ) -> Result<i64, String> {
        self.hata = Some("MIME e-posta IO iz kaydında kapalı".into());
        Err("MIME e-posta IO iz kaydında kapalı".into())
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

    /// Operatör günlüğü program tarafından gözlenmez; replay'i etkilememesi
    /// için olay kaydedilmeden iletilir (K-324/ADR-157).
    fn istek_tanisini_gunlukle(&mut self, durum: u16, tani: &crate::tani::Tani) {
        self.ic.istek_tanisini_gunlukle(durum, tani);
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

    fn tamsayi_sonuc(&mut self, islem: &str, sonuc: Option<Vec<String>>) -> Result<i64, String> {
        let Some(alanlar) = sonuc else {
            return Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi uyuşmazlığı".into()));
        };
        tamsayi_sonuc_coz(&alanlar).unwrap_or_else(|hata| {
            self.bozuk_sonuc(islem, hata);
            Err(self
                .hata
                .clone()
                .unwrap_or_else(|| "IO izi sonucu bozuk".into()))
        })
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
