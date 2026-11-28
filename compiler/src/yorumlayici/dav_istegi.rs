//! RFC-0047 B/D (ADR-182 dilim D, K-364): DAV isteğinin ham gövdesi ve
//! kapalı başlık listesi (`Depth`, `If-Match`, `If-None-Match`).
//!
//! Gövde form olarak çözülmez: DAV yoluna giden zarfın gövdesi ayrılır ve
//! yalnız DAV rotasının `isteğin ham gövdesi` ifadesine verilir. Başlıklar
//! adaptörden ham gelir; burada bir kez, rota çalışmadan doğrulanır
//! (geçersiz değer 400, PROPFIND'de sonsuz/eksik `Depth` 403
//! `propfind-finite-depth`). Koşul değerlendirmesi (`If-Match` ↔ kayıt ETag'i,
//! 412) ürünün Zee kodundadır.

use super::*;

/// PUT gövde tavanı (ürün görüşü, soru 7): `text/calendar` / `text/vcard`.
pub const DAV_PUT_GOVDE_SINIRI: usize = 1024 * 1024;
/// PROPFIND ve REPORT gövde tavanı.
pub const DAV_XML_GOVDE_SINIRI: usize = 256 * 1024;
/// Tek başlık değerinin bayt tavanı.
const BASLIK_DEGERI_AZAMI: usize = 1024;
/// `If-Match` / `If-None-Match` listesinin en çok etiket sayısı.
const KOSUL_ETIKETI_AZAMI: usize = 16;

/// RFC 4918 §9.1: sonsuz derinlik reddinin sabit gövdesi.
pub const PROPFIND_SONLU_DERINLIK: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<d:error xmlns:d=\"DAV:\"><d:propfind-finite-depth/></d:error>\n";

/// Yöntem başına gövde tavanı; `None` gövdenin yasak olduğunu söyler.
pub fn govde_siniri(yontem: HttpYontemi) -> Option<usize> {
    match yontem {
        HttpYontemi::Put => Some(DAV_PUT_GOVDE_SINIRI),
        HttpYontemi::Propfind | HttpYontemi::Report => Some(DAV_XML_GOVDE_SINIRI),
        _ => None,
    }
}

/// DAV yoluna giden isteğin gövdesini zarftan ayırır (çerez satırı atılır).
pub(super) fn ayir(
    program: &Program,
    ilk_satir: &str,
    govde: &str,
) -> Result<Option<(String, String)>, (u16, &'static str)> {
    let Some((yontem, hedef)) = ilk_satir.split_once(' ') else {
        return Ok(None);
    };
    let yol = hedef.trim().split('?').next().unwrap_or_default();
    let Some(yontem) = HttpYontemi::ayristir(&yontem.to_ascii_uppercase()) else {
        return Ok(None);
    };
    let dav_yolu = program.dav_yollari().iter().any(|(kayitli, onekli)| {
        if *onekli {
            yol.starts_with(kayitli.as_str())
        } else {
            yol == kayitli
        }
    });
    if !dav_yolu || !yontem.dav_yontemi() {
        return Ok(None);
    }
    match (govde_siniri(yontem), govde.is_empty()) {
        (None, false) => return Err((400, "bu DAV yöntemi istek gövdesi taşıyamaz")),
        (Some(sinir), _) if govde.len() > sinir => {
            return Err((413, "DAV istek gövdesi yöntem tavanını aşıyor"))
        }
        (Some(_), true) if matches!(yontem, HttpYontemi::Put | HttpYontemi::Report) => {
            return Err((400, "PUT ve REPORT istek gövdesi ister"))
        }
        _ => {}
    }
    Ok(Some((ilk_satir.to_string(), govde.to_string())))
}

/// Doğrulanmış başlıklar.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct DavBasliklari {
    pub(super) derinlik: Option<String>,
    pub(super) if_match: Vec<String>,
    pub(super) if_none_match: Vec<String>,
}

fn entity_tag_mi(etiket: &str) -> bool {
    let govde = etiket.strip_prefix("W/").unwrap_or(etiket);
    govde.len() >= 2
        && govde.starts_with('"')
        && govde.ends_with('"')
        && govde.as_bytes()[1..govde.len() - 1]
            .iter()
            .all(|b| *b == 0x21 || (0x23..=0x7e).contains(b))
}

fn kosul_listesi(deger: &str) -> Option<Vec<String>> {
    if deger.trim() == "*" {
        return Some(vec!["*".into()]);
    }
    let etiketler: Vec<String> = deger.split(',').map(|e| e.trim().to_string()).collect();
    (etiketler.len() <= KOSUL_ETIKETI_AZAMI && etiketler.iter().all(|e| entity_tag_mi(e)))
        .then_some(etiketler)
}

/// Ham başlıkları doğrular. `Err(400)` tekrarlanan ya da geçersiz değer,
/// `Err(403)` PROPFIND'de sonsuz/eksik `Depth`tir. REPORT'ta eksik `Depth`
/// `0` sayılır (RFC 3253 §3.6).
pub(super) fn basliklari_dogrula(
    yontem: HttpYontemi,
    ham: &[(String, String)],
) -> Result<DavBasliklari, u16> {
    let mut sonuc = DavBasliklari::default();
    let mut gorulen = Vec::new();
    for (ad, deger) in ham {
        let ad = ad.to_ascii_lowercase();
        let gorunur = deger.len() <= BASLIK_DEGERI_AZAMI
            && deger.bytes().all(|b| b == b' ' || b.is_ascii_graphic());
        if gorulen.contains(&ad) || !gorunur {
            return Err(400);
        }
        gorulen.push(ad.clone());
        match ad.as_str() {
            "depth" => {
                let deger = deger.trim().to_ascii_lowercase();
                if !matches!(deger.as_str(), "0" | "1" | "infinity") {
                    return Err(400);
                }
                sonuc.derinlik = Some(deger);
            }
            "if-match" => sonuc.if_match = kosul_listesi(deger).ok_or(400u16)?,
            "if-none-match" => sonuc.if_none_match = kosul_listesi(deger).ok_or(400u16)?,
            _ => {}
        }
    }
    match (yontem, sonuc.derinlik.as_deref()) {
        (HttpYontemi::Propfind, None | Some("infinity")) => return Err(403),
        (HttpYontemi::Report, None) => sonuc.derinlik = Some("0".into()),
        _ => {}
    }
    Ok(sonuc)
}

/// Rota öncesi doğrulama; `false`: yanıt (400/403) gönderildi.
pub(super) fn on_denetim(io: &mut dyn GirdiCikti, yontem: HttpYontemi) -> bool {
    match basliklari_dogrula(yontem, &io.dav_istek_basliklari()) {
        Ok(_) => true,
        Err(403) => {
            io.turlu_durumlu_yanit_gonder("application/xml", 403, PROPFIND_SONLU_DERINLIK);
            false
        }
        Err(durum) => {
            io.durum_yaniti_gonder(durum, "DAV istek başlığı geçersiz");
            false
        }
    }
}

/// `isteğin ham gövdesi` ve DAV başlık intrinsic'leri.
pub(super) const INTRINSICLER: [&str; 3] = [
    crate::intrinsic::WEB_HAM_GOVDE,
    crate::intrinsic::WEB_ISTEK_DERINLIGI,
    crate::intrinsic::WEB_ISTEK_KOSULU,
];

pub(super) fn intrinsic_degerlendir(
    kimlik: &str,
    degerler: &[Deger],
    ortam: &HashMap<String, Deger>,
    io: &mut dyn GirdiCikti,
    satir: usize,
) -> Result<Deger, Tani> {
    if kimlik == crate::intrinsic::WEB_HAM_GOVDE {
        return super::makine_istegi::ham_govde(ortam, satir);
    }
    // Ön denetimden geçmiş başlıklar rotanın yöntemiyle yeniden normalize
    // edilir (REPORT'ta eksik Depth `0`); yöntem `istek` sözlüğündedir.
    let yontem = match ortam.get("istek") {
        Some(Deger::Sozluk(alanlar)) => alanlar.iter().find_map(|(ad, deger)| match deger {
            Deger::Metin(yontem) if ad == "yöntem" => HttpYontemi::ayristir(yontem),
            _ => None,
        }),
        _ => None,
    };
    let basliklar = basliklari_dogrula(
        yontem.unwrap_or(HttpYontemi::Get),
        &io.dav_istek_basliklari(),
    )
    .map_err(|_| ic_hata(satir))?;
    Ok(match (kimlik, degerler) {
        (crate::intrinsic::WEB_ISTEK_DERINLIGI, []) => {
            Deger::Metin(basliklar.derinlik.unwrap_or_default())
        }
        (crate::intrinsic::WEB_ISTEK_KOSULU, [Deger::Metin(ad)]) => {
            let liste = if ad.eq_ignore_ascii_case("if-match") {
                basliklar.if_match
            } else {
                basliklar.if_none_match
            };
            Deger::Liste(liste.into_iter().map(Deger::Metin).collect())
        }
        _ => return Err(ic_hata(satir)),
    })
}

#[cfg(test)]
mod testler {
    use super::*;

    fn b(ciftler: &[(&str, &str)]) -> Vec<(String, String)> {
        ciftler
            .iter()
            .map(|(a, d)| (a.to_string(), d.to_string()))
            .collect()
    }

    #[test]
    fn depth_kurallari_propfind_report() {
        use HttpYontemi::{Propfind, Put, Report};
        assert_eq!(basliklari_dogrula(Propfind, &b(&[])), Err(403));
        assert_eq!(
            basliklari_dogrula(Propfind, &b(&[("Depth", "infinity")])),
            Err(403)
        );
        assert_eq!(
            basliklari_dogrula(Propfind, &b(&[("depth", "1")]))
                .unwrap()
                .derinlik
                .as_deref(),
            Some("1")
        );
        assert_eq!(
            basliklari_dogrula(Report, &b(&[]))
                .unwrap()
                .derinlik
                .as_deref(),
            Some("0")
        );
        assert_eq!(basliklari_dogrula(Put, &b(&[])).unwrap().derinlik, None);
        for kotu in ["2", "", "-1", "sonsuz", "1,0"] {
            assert_eq!(
                basliklari_dogrula(Propfind, &b(&[("Depth", kotu)])),
                Err(400),
                "{kotu}"
            );
        }
        assert_eq!(
            basliklari_dogrula(Propfind, &b(&[("Depth", "0"), ("DEPTH", "1")])),
            Err(400),
            "tekrar"
        );
    }

    #[test]
    fn kosul_listesi_yildiz_etiketler_ve_red() {
        let p = |deger: &str| basliklari_dogrula(HttpYontemi::Put, &b(&[("If-Match", deger)]));
        assert_eq!(p("*").unwrap().if_match, ["*"]);
        assert_eq!(
            p("\"a\", W/\"b\",\"c\"").unwrap().if_match,
            ["\"a\"", "W/\"b\"", "\"c\""]
        );
        let on_alti = vec!["\"x\""; 16].join(", ");
        assert_eq!(p(&on_alti).unwrap().if_match.len(), 16);
        let on_yedi = vec!["\"x\""; 17].join(", ");
        for kotu in [
            "a",
            "\"a",
            "\"a\"b\"",
            "*, \"a\"",
            "",
            &on_yedi,
            "\"\u{e7}\"",
        ] {
            assert_eq!(p(kotu), Err(400), "{kotu}");
        }
        let n = basliklari_dogrula(HttpYontemi::Put, &b(&[("If-None-Match", "*")])).unwrap();
        assert_eq!(n.if_none_match, ["*"]);
        assert!(n.if_match.is_empty());
        let uzun = format!("\"{}\"", "a".repeat(1100));
        assert_eq!(p(&uzun), Err(400));
    }

    #[test]
    fn govde_tavanlari() {
        assert_eq!(govde_siniri(HttpYontemi::Put), Some(1024 * 1024));
        assert_eq!(govde_siniri(HttpYontemi::Report), Some(256 * 1024));
        assert_eq!(govde_siniri(HttpYontemi::Propfind), Some(256 * 1024));
        assert_eq!(govde_siniri(HttpYontemi::Get), None);
        assert_eq!(govde_siniri(HttpYontemi::Delete), None);
    }
}
