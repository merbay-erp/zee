//! RFC-0047 E/F (ADR-182 dilim C): native web adaptörünün DAV istemci yolu.
//!
//! `dav istemcisi gerekli` rotasına giden istek gövde, oturum ve Origin
//! işinden önce derlenen DAV yol kümesinden tanınır. İstek çerezi okunmaz,
//! oturum anonim başlar, yanıt hiçbir koşulda `Set-Cookie` taşımaz; CSRF
//! aranmaz. Buna karşılık `Sec-Fetch-Site: cross-site` ve yabancı `Origin`
//! rota çalışmadan 403'tür. Kimlik yalnız `Authorization: Basic` + uygulama
//! parolasıdır: başlık burada çözülür, gizli kısım bu süreçte kalır ve
//! yorumlayıcıya yalnız kullanıcı adı ile arama öneki verilir. Üretim
//! profilinde (`--web-proxy`) DAV rotaları yalnız ayrı `dav_kökeni`
//! origin'inde sunulur. Bu dilimde gövde okunur ve atılır (ham DAV gövdesi
//! RFC-0047 dilim D'dir).

use super::*;
use dil::guvenlik::uygulama_parolasi as up;

/// Bu dilimde DAV gövdesinin okunup atıldığı tavan (spec/11 §5 varsayılanı).
const DAV_GOVDE_SINIRI: usize = 64 * 1024;

/// Geçerli isteğin DAV kimlik durumu. `kullanici` yoksa başlık eksik ya da
/// bozuktur; `parola` biçim kapısından geçen (önek, gizli) çiftidir.
#[derive(Default)]
pub(super) struct DavIstegi {
    kullanici: Option<String>,
    parola: Option<(String, String)>,
}

impl GercekIo {
    /// Sorgusuz yol derlenen DAV yol kümesinde (tam ya da önekli) ve yöntem
    /// DAV yöntemlerinden biri (OPTIONS dahil).
    pub(super) fn dav_istegi_mi(&self, baslik: &HttpIstekBasligi<'_>) -> bool {
        let yol = baslik.hedef().split('?').next().unwrap_or_default();
        let yontem = baslik.yontem().to_ascii_uppercase();
        let dav_yontemi = yontem == "OPTIONS"
            || dil::agac::HttpYontemi::ayristir(&yontem).is_some_and(|y| y.dav_yontemi());
        dav_yontemi
            && self
                .guvenilir_betikler
                .dav_yollari
                .get()
                .is_some_and(|yollar| {
                    yollar.iter().any(|(kayitli, onekli)| {
                        if *onekli {
                            yol.starts_with(kayitli.as_str())
                        } else {
                            yol == kayitli
                        }
                    })
                })
    }

    /// Üretim profilinde DAV rotaları ayrı origin ister (ADR-182 soru 8).
    pub(super) fn dav_kokenini_denetle(&self) -> Result<(), String> {
        let var = self
            .guvenilir_betikler
            .dav_yollari
            .get()
            .is_some_and(|yollar| !yollar.is_empty());
        let Some(ana) = self.guvenli_proxy_origin() else {
            return Ok(());
        };
        match &self.dav_kokeni {
            _ if !var => Ok(()),
            None => Err("üretim profilinde `dav istemcisi gerekli` rotaları ayrı bir origin ister; proje.dil içinde `dav_kökeni \"https://dav.<alan>\" olsun` bildir".into()),
            Some(dav) if dav == ana => Err("`dav_kökeni` web origin'inden farklı olmalı; çerezli rotalarla aynı origin'de DAV sunulmaz".into()),
            Some(_) => Ok(()),
        }
    }

    /// Proxy denetiminin bu istek için beklediği origin: DAV isteğinde
    /// `dav_kökeni`, aksi hâlde web origin'i.
    pub(super) fn beklenen_origin(&self) -> Option<&AgHedefi> {
        let ana = self.guvenli_proxy_origin()?;
        Some(match (&self.dav_istegi, &self.dav_kokeni) {
            (Some(_), Some(dav)) => dav,
            _ => ana,
        })
    }

    /// `Sec-Fetch-Site: cross-site` ya da yabancı `Origin` 403; ardından
    /// Basic çözümü. `false`: yanıt gönderildi.
    fn dav_on_kapisi(
        &mut self,
        akis: &mut web_metrikleri::Akis,
        baslik: &HttpIstekBasligi<'_>,
        https: bool,
    ) -> bool {
        let head = baslik.yontem().eq_ignore_ascii_case("HEAD");
        let capraz = baslik
            .baslik_degerleri("Sec-Fetch-Site")
            .iter()
            .any(|deger| deger.trim().eq_ignore_ascii_case("cross-site"));
        let beklenen = match self.beklenen_origin() {
            Some(origin) => origin.yazimi(),
            None => format!("http://{}", tek_http_basligi(baslik, "Host").unwrap_or("")),
        };
        let yabanci_origin = baslik
            .baslik_degerleri("Origin")
            .iter()
            .any(|deger| !deger.trim().eq_ignore_ascii_case(&beklenen));
        if capraz || yabanci_origin {
            ham_http_hatasi_gonder(
                akis,
                403,
                "DAV istemci rotası çapraz kökenli tarayıcı isteğini kabul etmez",
                head,
                https,
            );
            return false;
        }
        let kimlik = match baslik.baslik_degerleri("Authorization").as_slice() {
            [deger] => up::basic_coz(deger),
            _ => None,
        };
        self.dav_istegi = Some(match kimlik {
            Some((kullanici, parola)) => DavIstegi {
                kullanici: Some(kullanici),
                parola: up::parolayi_coz(&parola),
            },
            None => DavIstegi::default(),
        });
        true
    }

    /// DAV isteğinin zarfı: istek satırı ve açık boş çerez satırı. Gövde bu
    /// dilimde okunur ve atılır. Yanıt yazıldıysa `None`.
    pub(super) fn dav_zarfini_kur(
        &mut self,
        akis: &mut web_metrikleri::Akis,
        baslik: &HttpIstekBasligi<'_>,
        ilk_govde: &[u8],
        es: std::net::SocketAddr,
        son_tarih: std::time::Instant,
        https: bool,
    ) -> Option<(String, Option<web_yukleme::IstekYuklemesi>, usize)> {
        if !self.dav_on_kapisi(akis, baslik, https) {
            return None;
        }
        let oturum_cerez_adi = self.oturum_cerez_adi().to_string();
        let head = baslik.yontem().eq_ignore_ascii_case("HEAD");
        match istek_govdesini_oku(
            akis,
            &self.kok,
            baslik,
            ilk_govde,
            son_tarih,
            https,
            &oturum_cerez_adi,
            DAV_GOVDE_SINIRI,
        ) {
            Ok(_) => {}
            Err(GovdeReddi::Gonderildi) => return None,
            Err(red) => {
                red.ham_gonder(akis, head, https);
                return None;
            }
        }
        self.web_istegini_baslat(akis, baslik, es, https)?;
        let zarf = format!("{} {}\nçerez \n", baslik.yontem(), baslik.hedef());
        Some((zarf, None, DAV_GOVDE_SINIRI))
    }

    /// Bildirilen HMAC anahtarları; okunamayan anahtar atlanır (o kimlikli
    /// kayıtlar doğrulanmaz, fail-closed).
    fn dav_anahtarlari(&self) -> Vec<(String, [u8; 32])> {
        self.uygulama_parolasi_anahtarlari
            .iter()
            .filter_map(|bildirim| {
                let hex = gizli_degeri_oku(&bildirim.credential, 4096, TekSatir).ok()?;
                let anahtar = dil::guvenlik::gizli::anahtar_coz(&hex).ok()?;
                Some((bildirim.kimlik.clone(), anahtar))
            })
            .collect()
    }

    pub(super) fn dav_kimligi_ver(&self) -> Option<dil::yorumlayici::DavKimligi> {
        let istek = self.dav_istegi.as_ref()?;
        Some(dil::yorumlayici::DavKimligi {
            kullanici: istek.kullanici.clone()?,
            onek: istek.parola.as_ref().map(|(onek, _)| onek.clone()),
            cozucu: self.dav_kimlik_cozucusu.clone(),
        })
    }

    /// Sabit zamanlı özet karşılaştırması; kayıt ya da biçimli parola yoksa
    /// aynı maliyette sahte karşılaştırma ve `false`.
    pub(super) fn dav_ozetini_dogrula(&self, ozet: Option<&str>) -> bool {
        if !self
            .politika
            .izin_verir(dil::yetkinlik::Yetkinlik::Kriptografi)
        {
            return false;
        }
        let parola = self.dav_istegi.as_ref().and_then(|i| i.parola.as_ref());
        let sahte = ("AAAAAAAA".to_string(), "A".repeat(up::GIZLI_KARAKTERI));
        let (onek, gizli) = parola.unwrap_or(&sahte);
        up::ozet_dogrula(&self.dav_anahtarlari(), onek, gizli, ozet) && parola.is_some()
    }

    /// RFC-0047 §7 kovaları; depo hatası fail-closed 429'dur.
    pub(super) fn dav_kovasini_isle(
        &self,
        kova: dil::yorumlayici::DavKova,
        anahtar: &str,
    ) -> Result<(), i64> {
        use dil::web_guvenligi::RateLimitTuru::{DavBasarili, DavBasarisiz, DavIstemci};
        use dil::yorumlayici::DavKova;
        let an = web_duvar_saati_ms();
        let guv = &self.web_guvenligi;
        let kararlar = match kova {
            DavKova::Bak => vec![
                guv.rate_limit_bak(DavBasarisiz, anahtar, an),
                guv.rate_limit_bak(DavIstemci, "", an),
            ],
            DavKova::Basarisiz => vec![
                guv.rate_limit_artir(DavBasarisiz, anahtar, an),
                guv.rate_limit_artir(DavIstemci, "", an),
            ],
            DavKova::Basarili => vec![guv.rate_limit_artir(DavBasarili, anahtar, an)],
        };
        let mut bekle = None;
        for karar in kararlar {
            match karar {
                Ok(karar) if karar.izinli => {}
                Ok(karar) => bekle = Some(karar.yeniden_dene_saniye.max(1)),
                Err(_) => bekle = Some(1),
            }
        }
        if bekle.is_some() {
            if let Some(sink) = &self.metrik {
                sink.rate_reject();
            }
        }
        bekle.map_or(Ok(()), Err)
    }

    /// 401 ek başlığı: `WWW-Authenticate: Basic` ve kanonik origin realm'i.
    pub(super) fn dav_realm_basligi(&self) -> String {
        let realm = self
            .beklenen_origin()
            .map(|origin| origin.yazimi())
            .unwrap_or_else(|| "zee".into());
        format!("WWW-Authenticate: Basic realm=\"{realm}\", charset=\"UTF-8\"\r\n")
    }
}
