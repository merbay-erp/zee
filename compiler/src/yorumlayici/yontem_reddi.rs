//! RFC-0047 dilim A (ADR-182): kayıt dışı yönteme 501, eşleşmeyen yönteme
//! 405 + kanonik `Allow`; bütün rota sınıflarında aynı.

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
