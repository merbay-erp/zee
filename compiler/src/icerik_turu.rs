//! K-378 (RFC-0041, ADR-163): web dosya yanıtı ve SMTP eki için ortak, kapalı
//! içerik türü kaydı ile dosya adı/`Content-Disposition` kodlaması.
//!
//! Kayıt tek tablodur: her satır kanonik kısa adı, MIME türünü, uzantıları,
//! metin olup olmadığını, isteğe bağlı içerik imzasını ve hangi yüzeyde
//! (web dosya yanıtı, SMTP varsayılanı, SMTP adayı) kullanılabileceğini taşır.
//! RFC-0040 katman C'nin metin sınıfı ileride bu tabloya satır ekler; ayrı
//! kayıt açılmaz. Tabloda olmayan tür hiçbir yüzeyde yazılamaz. HTML, SVG,
//! XHTML, betik, yürütülebilir ve makro taşıyan türler bilinçli olarak yoktur.

/// Tek içerik türü satırı.
#[derive(Debug, PartialEq, Eq)]
pub struct IcerikTuru {
    /// Kanonik kısa ad; proje bildiriminde (`e_posta_ek_türleri`) bu yazılır.
    pub ad: &'static str,
    /// Küçük harf MIME türü (parametresiz).
    pub mime: &'static str,
    /// Küçük harf dosya uzantıları; ilki kanoniktir. Boşsa SMTP'de seçilemez.
    pub uzantilar: &'static [&'static str],
    /// `text/*`: başlıkta `charset=utf-8`, SMTP ekinde UTF-8 strict denetim.
    pub metin: bool,
    /// Varsa SMTP ekinin ilk baytları bununla başlamak zorundadır.
    pub imza: Option<&'static [u8]>,
    /// Bildirimsiz projede SMTP eki olarak kabul edilir (`zee-smtp-mime-1`).
    pub smtp_varsayilan: bool,
    /// `e_posta_ek_türleri` bildirimiyle açılabilir.
    pub smtp_aday: bool,
    /// Rota dosya yanıtında (`... ek olarak göndermeyi dene`) kullanılabilir.
    pub web_dosya: bool,
}

const ZIP_IMZASI: &[u8] = b"PK\x03\x04";

/// Kapalı kayıt. Sıra K-167 dil yüzeyi fixture'ındaki sıradır.
pub const TURLER: &[IcerikTuru] = &[
    IcerikTuru {
        ad: "pdf",
        mime: "application/pdf",
        uzantilar: &["pdf"],
        metin: false,
        imza: None,
        smtp_varsayilan: true,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "png",
        mime: "image/png",
        uzantilar: &["png"],
        metin: false,
        imza: None,
        smtp_varsayilan: true,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "jpeg",
        mime: "image/jpeg",
        uzantilar: &["jpg", "jpeg"],
        metin: false,
        imza: None,
        smtp_varsayilan: true,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "webp",
        mime: "image/webp",
        uzantilar: &["webp"],
        metin: false,
        imza: None,
        smtp_varsayilan: true,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "txt",
        mime: "text/plain",
        uzantilar: &["txt"],
        metin: true,
        imza: None,
        smtp_varsayilan: false,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "csv",
        mime: "text/csv",
        uzantilar: &["csv"],
        metin: true,
        imza: None,
        smtp_varsayilan: false,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "ics",
        mime: "text/calendar",
        uzantilar: &["ics"],
        metin: true,
        imza: None,
        smtp_varsayilan: false,
        smtp_aday: true,
        web_dosya: true,
    },
    // RFC-0047 C (ADR-182): CardDAV kişi kartı (RFC 6350/6352). Yalnız web
    // yüzeyi (metin yanıtı ve dosya yanıtı); SMTP adayı değildir.
    IcerikTuru {
        ad: "vcf",
        mime: "text/vcard",
        uzantilar: &["vcf"],
        metin: true,
        imza: None,
        smtp_varsayilan: false,
        smtp_aday: false,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "docx",
        mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        uzantilar: &["docx"],
        metin: false,
        imza: Some(ZIP_IMZASI),
        smtp_varsayilan: false,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "xlsx",
        mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        uzantilar: &["xlsx"],
        metin: false,
        imza: Some(ZIP_IMZASI),
        smtp_varsayilan: false,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "pptx",
        mime: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        uzantilar: &["pptx"],
        metin: false,
        imza: Some(ZIP_IMZASI),
        smtp_varsayilan: false,
        smtp_aday: true,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "zip",
        mime: "application/zip",
        uzantilar: &["zip"],
        metin: false,
        imza: Some(ZIP_IMZASI),
        smtp_varsayilan: false,
        smtp_aday: true,
        web_dosya: true,
    },
    // K-385 (ADR-163 eki, ZM-F033): yalnız web dosya yanıtı. E-posta
    // istemcisi autoconfig/autodiscover XML'i ve Apple yapılandırma profili.
    // Ayrıştırılmaz, imza denetimi yoktur; `attachment` + sandbox CSP +
    // `nosniff` diğer satırlarla aynıdır. `charset` eklenmez: XML kodlamasını
    // kendi bildirimi taşır (RFC 7303), imzalı profil ikili CMS'tir.
    IcerikTuru {
        ad: "xml",
        mime: "application/xml",
        uzantilar: &["xml"],
        metin: false,
        imza: None,
        smtp_varsayilan: false,
        smtp_aday: false,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "mobileconfig",
        mime: "application/x-apple-aspen-config",
        uzantilar: &["mobileconfig"],
        metin: false,
        imza: None,
        smtp_varsayilan: false,
        smtp_aday: false,
        web_dosya: true,
    },
    IcerikTuru {
        ad: "ikili",
        mime: "application/octet-stream",
        uzantilar: &[],
        metin: false,
        imza: None,
        smtp_varsayilan: false,
        smtp_aday: false,
        web_dosya: true,
    },
];

/// Web dosya yanıtının bayt tavanı: K-401 ile çözerek yazma hedef tavanına
/// hizalı 32 MiB (yükleme tavanı 16 MiB kalır). Yanıt 16 KiB parçalarla
/// akar; tavan bellek tepesini değil yanıt süresini/bant genişliğini sınırlar.
pub const WEB_DOSYA_YANITI_BAYTI: u64 = 32 * 1024 * 1024;

/// Dosya adının (UTF-8) azami bayt uzunluğu.
pub const DOSYA_ADI_AZAMI_BAYT: usize = 255;

/// Kanonik kısa adla satır.
pub fn adla(ad: &str) -> Option<&'static IcerikTuru> {
    TURLER.iter().find(|tur| tur.ad == ad)
}

/// Rota dosya yanıtında kullanılabilen türü exact (küçük harf, parametresiz)
/// MIME yazımıyla bulur.
pub fn web_dosya_turu(mime: &str) -> Option<&'static IcerikTuru> {
    TURLER.iter().find(|tur| tur.web_dosya && tur.mime == mime)
}

/// K-406 (ADR-163 eki, ZM-F048): bellekteki `Metin`in rota yanıtı olarak
/// verilebileceği kayıt satırları. Yalnız UTF-8 metin taşıyan, tarayıcıda
/// betik yürütmeyen dosya sınıfı türleri; ikili türler, `octet-stream` ve
/// Apple profili (imzalı hâli ikili CMS'tir) bilinçli olarak dışarıdadır.
/// RFC-0047 C (ADR-182) `vcf` (text/vcard) satırını ekler.
pub const WEB_METIN_TURLERI: &[&str] = &["txt", "csv", "ics", "vcf", "xml"];

/// `<metin> metnini "<tür>" türünde gönder` için exact (küçük harf,
/// parametresiz) MIME yazımıyla satır.
pub fn web_metin_turu(mime: &str) -> Option<&'static IcerikTuru> {
    web_dosya_turu(mime).filter(|tur| WEB_METIN_TURLERI.contains(&tur.ad))
}

/// Dosya adının son uzantısına göre SMTP'de seçilebilir satır (büyük/küçük
/// harf duyarsız). İzin kararı çağıranın kümesindedir.
pub fn uzantidan(ad: &str) -> Option<&'static IcerikTuru> {
    let uzanti = ad.rsplit_once('.')?.1.to_ascii_lowercase();
    TURLER
        .iter()
        .find(|tur| tur.uzantilar.contains(&uzanti.as_str()))
}

/// `Content-Type` başlık değeri; metin türlerinde `charset=utf-8` eklenir.
pub fn baslik_degeri(tur: &IcerikTuru) -> String {
    if tur.metin {
        format!("{}; charset=utf-8", tur.mime)
    } else {
        tur.mime.to_string()
    }
}

/// Unicode yön denetimi: görünen adı tersine çevirerek uzantı sahteciliği
/// (`fatura\u{202E}fdp.exe`) yapar.
fn yon_denetimi_mi(c: char) -> bool {
    matches!(
        c,
        '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

/// Kullanıcıya gösterilecek dosya adını doğrular (web yanıtı ve multipart).
/// Boş, `.`/`..`, yol ayracı, NUL ve diğer denetim karakterleri (CR/LF dahil),
/// çift tırnak, Unicode yön denetimi ve 255 baytı aşan ad reddedilir.
pub fn dosya_adini_denetle(ad: &str) -> Result<(), &'static str> {
    if ad.is_empty() || matches!(ad, "." | "..") {
        return Err("dosya adı boş ya da `.`/`..` olamaz");
    }
    if ad.len() > DOSYA_ADI_AZAMI_BAYT {
        return Err("dosya adı 255 baytı aşıyor");
    }
    if ad.contains(['/', '\\']) {
        return Err("dosya adı yol ayracı içeremez");
    }
    if ad.chars().any(char::is_control) {
        return Err("dosya adı denetim karakteri (CR/LF dahil) içeremez");
    }
    if ad.contains('"') {
        return Err("dosya adı çift tırnak içeremez");
    }
    if ad.chars().any(yon_denetimi_mi) {
        return Err("dosya adı Unicode yön denetim karakteri içeremez");
    }
    Ok(())
}

/// RFC 8187 `ext-value` yüzde kodlaması: yalnız `[A-Za-z0-9._-]` düz kalır.
pub fn rfc8187_kodla(ad: &str) -> String {
    let mut sonuc = String::with_capacity(ad.len());
    for bayt in ad.bytes() {
        if bayt.is_ascii_alphanumeric() || matches!(bayt, b'.' | b'_' | b'-') {
            sonuc.push(char::from(bayt));
        } else {
            sonuc.push_str(&format!("%{bayt:02X}"));
        }
    }
    sonuc
}

/// `filename="…"` ASCII geri düşüşü: ASCII dışı ve `%`, `"`, `\`, `;`
/// karakterleri `_` olur.
pub fn ascii_geri_dusus(ad: &str) -> String {
    ad.chars()
        .map(|c| {
            if c.is_ascii() && !c.is_ascii_control() && !matches!(c, '%' | '"' | '\\' | ';') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Doğrulanmış ad için `Content-Disposition: attachment` değeri (RFC 6266
/// §4.3): ASCII geri düşüş ve UTF-8 `filename*`. Başlığı yalnız çalışma
/// zamanı kurar; Zee kodu başlık değeri yazamaz.
pub fn ek_disposition(ad: &str) -> Result<String, &'static str> {
    dosya_adini_denetle(ad)?;
    Ok(format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        ascii_geri_dusus(ad),
        rfc8187_kodla(ad)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kayit_tekil_ve_yasak_turler_yok() {
        let mut adlar = std::collections::BTreeSet::new();
        let mut mimeler = std::collections::BTreeSet::new();
        for tur in TURLER {
            assert!(adlar.insert(tur.ad), "yinelenen ad {}", tur.ad);
            assert!(mimeler.insert(tur.mime), "yinelenen mime {}", tur.mime);
            assert_eq!(tur.mime, tur.mime.to_ascii_lowercase());
            assert!(!tur.smtp_varsayilan || tur.smtp_aday);
            assert!(!tur.smtp_aday || !tur.uzantilar.is_empty());
        }
        for yasak in [
            "text/html",
            "image/svg+xml",
            "application/xhtml+xml",
            "application/javascript",
            "text/javascript",
            "application/x-msdownload",
            "application/vnd.ms-word.document.macroenabled.12",
        ] {
            assert!(web_dosya_turu(yasak).is_none(), "{yasak}");
        }
        for uzanti in [
            "a.exe", "a.html", "a.svg", "a.js", "a.docm", "a.xlsm", "a.doc", "a.bat", "a.ps1",
            "a.lnk", "a.iso",
        ] {
            assert!(uzantidan(uzanti).is_none(), "{uzanti}");
        }
        let varsayilan: Vec<_> = TURLER
            .iter()
            .filter(|tur| tur.smtp_varsayilan)
            .map(|tur| tur.ad)
            .collect();
        assert_eq!(varsayilan, ["pdf", "png", "jpeg", "webp"]);
    }

    #[test]
    fn web_turu_yalniz_exact_kucuk_harf_ve_parametresiz_eslesir() {
        assert_eq!(web_dosya_turu("application/pdf").map(|t| t.ad), Some("pdf"));
        assert!(web_dosya_turu("Application/PDF").is_none());
        assert!(web_dosya_turu("application/pdf; charset=utf-8").is_none());
        assert!(web_dosya_turu("text/plain\r\nX-Evil: 1").is_none());
        assert_eq!(
            baslik_degeri(adla("csv").expect("csv")),
            "text/csv; charset=utf-8"
        );
        assert_eq!(baslik_degeri(adla("zip").expect("zip")), "application/zip");
        assert_eq!(
            baslik_degeri(web_dosya_turu("application/xml").expect("xml")),
            "application/xml"
        );
        assert_eq!(
            web_dosya_turu("application/x-apple-aspen-config").map(|t| t.ad),
            Some("mobileconfig")
        );
        // `text/xml` takma adı ve parametreli/büyük harfli yazımlar açılmaz.
        for kotu in [
            "text/xml",
            "Application/XML",
            "application/xml; charset=utf-8",
            "application/x-apple-aspen-config; charset=utf-8",
            "application/rss+xml",
            "application/xslt+xml",
        ] {
            assert!(web_dosya_turu(kotu).is_none(), "{kotu}");
        }
        // Web'e özgü satırlar SMTP adayı değildir; uzantıdan SMTP seçimi yok.
        for ad in ["xml", "mobileconfig", "vcf"] {
            let tur = adla(ad).expect(ad);
            assert!(tur.web_dosya && !tur.smtp_aday && !tur.smtp_varsayilan);
        }
        assert_eq!(uzantidan("Rapor.DOCX").map(|t| t.ad), Some("docx"));
        assert_eq!(uzantidan("foto.jpg").map(|t| t.ad), Some("jpeg"));
        assert!(uzantidan("uzantisiz").is_none());
    }

    #[test]
    fn web_metin_turu_yalniz_metin_satirlarini_acar() {
        for (mime, baslik) in [
            ("text/plain", "text/plain; charset=utf-8"),
            ("text/csv", "text/csv; charset=utf-8"),
            ("text/calendar", "text/calendar; charset=utf-8"),
            ("text/vcard", "text/vcard; charset=utf-8"),
            ("application/xml", "application/xml"),
        ] {
            let tur = web_metin_turu(mime).expect(mime);
            assert_eq!(baslik_degeri(tur), baslik);
        }
        for kotu in [
            "text/html",
            "image/svg+xml",
            "application/xhtml+xml",
            "text/javascript",
            "application/javascript",
            "application/octet-stream",
            "application/pdf",
            "image/png",
            "application/zip",
            "application/x-apple-aspen-config",
            "Text/Calendar",
            "text/calendar; charset=utf-8",
            "text/xml",
        ] {
            assert!(web_metin_turu(kotu).is_none(), "{kotu}");
        }
        for ad in WEB_METIN_TURLERI {
            assert!(adla(ad).is_some_and(|tur| tur.web_dosya), "{ad}");
        }
    }

    #[test]
    fn dosya_adi_enjeksiyon_ve_yon_karakterlerini_reddeder() {
        for kotu in [
            "",
            ".",
            "..",
            "a/b.pdf",
            "a\\b.pdf",
            "a\r\nSet-Cookie: x=1.pdf",
            "a\nb.pdf",
            "a\0.pdf",
            "a\".pdf",
            "fatura\u{202E}fdp.exe",
            "a\u{2066}.pdf",
            "a\u{7f}.pdf",
            "a\u{85}.pdf",
        ] {
            assert!(dosya_adini_denetle(kotu).is_err(), "{kotu:?}");
            assert!(ek_disposition(kotu).is_err(), "{kotu:?}");
        }
        assert!(dosya_adini_denetle(&"a".repeat(255)).is_ok());
        assert!(dosya_adini_denetle(&"a".repeat(256)).is_err());
    }

    #[test]
    fn disposition_rfc6266_ascii_geri_dusus_ve_filename_yildiz() {
        assert_eq!(
            ek_disposition("Fatura Ekim; 100%.pdf").expect("geçerli"),
            "attachment; filename=\"Fatura Ekim_ 100_.pdf\"; filename*=UTF-8''Fatura%20Ekim%3B%20100%25.pdf"
        );
        assert_eq!(
            ek_disposition("çizelge-ğüşİ.xlsx").expect("geçerli"),
            "attachment; filename=\"_izelge-____.xlsx\"; filename*=UTF-8''%C3%A7izelge-%C4%9F%C3%BC%C5%9F%C4%B0.xlsx"
        );
    }
}
