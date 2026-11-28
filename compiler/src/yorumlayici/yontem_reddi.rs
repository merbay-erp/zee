//! RFC-0047 dilim A (ADR-182): kayıt dışı yönteme 501, eşleşmeyen yönteme
//! 405 + kanonik `Allow`; bütün rota sınıflarında aynı. GET'e izin veren yol
//! HEAD'i örtük destekler (RFC 9110 §9.3.2).

use super::*;

/// RFC-0047 A (ADR-182): kapalı kayıtta olmayan yöntem yoldan bağımsız
/// 501'dir (RFC 9110 §15.6.2); rota ve oturum işi hiç başlamaz.
pub(super) fn kayitli_yontem<T>(
    istek: (String, String, T, T),
) -> Result<(String, String, T, T), (u16, &'static str)> {
    match HttpYontemi::ayristir(&istek.0) {
        Some(_) => Ok(istek),
        None => Err((501, "bu HTTP yöntemi desteklenmiyor")),
    }
}

/// Hiçbir rota eşleşmedi. RFC-0047 A (ADR-182): yol eşleştiyse 405, o yolda
/// tanımlı yöntemlerin kanonik `Allow` değeriyle (RFC 9110 §15.5.6), her rota
/// sınıfında; yol da eşleşmediyse 404.
pub(super) fn eslesmeyen_istek(io: &mut dyn GirdiCikti, izinli: &[HttpYontemi], yol: &str) {
    if izinli.is_empty() {
        io.durum_yaniti_gonder(404, &format!("aranan sayfa yok: {}", yol));
    } else {
        io.yontem_reddi_gonder(
            &HttpYontemi::allow_degeri(izinli),
            "bu adres istenen HTTP yöntemini kabul etmiyor",
        );
    }
}

/// RFC 9110 §9.3.2: GET'e izin veren yol HEAD'i örtük olarak destekler.
/// HEAD isteği için o yolda açık HEAD rotası yoksa ve GET rotası varsa
/// eşleştirme GET ile yapılır; GET rotası salt okumadır (T045), yanıt
/// gövdesini adaptör HEAD için yazmaz. Açık HEAD rotası önceliklidir.
pub(super) fn etkin_yontem(
    program: CalistirmaProgrami<'_>,
    io: &mut dyn GirdiCikti,
    gelen: &str,
    yol: &str,
) -> Result<String, Tani> {
    if gelen != HttpYontemi::Head.yazimi() {
        return Ok(gelen.to_string());
    }
    let mut get_var = false;
    for cumle in &program.program().cumleler {
        let Cumle::IstekGeldiginde {
            yontem,
            yol: kayitli,
            onekli,
            satir,
            ..
        } = cumle
        else {
            continue;
        };
        let kayitli = metne_sinirli(
            &degerlendir(kayitli, &HashMap::new(), program, io, 0, *satir)?,
            *satir,
        )?;
        let uydu = if *onekli {
            yol.starts_with(&kayitli)
        } else {
            kayitli == yol
        };
        match yontem.unwrap_or(HttpYontemi::Get) {
            HttpYontemi::Head if uydu => return Ok(gelen.to_string()),
            HttpYontemi::Get if uydu => get_var = true,
            _ => {}
        }
    }
    Ok(if get_var {
        HttpYontemi::Get
    } else {
        HttpYontemi::Head
    }
    .yazimi()
    .to_string())
}
