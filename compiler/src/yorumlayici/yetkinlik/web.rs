//! Existing host policy gates and request metadata (K-209).
use super::*;

pub(super) fn rota_denetle<T: GirdiCikti>(
    io: &mut PolitikaliIo<T>,
    erisim: &RotaErisimi,
    csrf: Option<&str>,
    csrf_gerekli: bool,
) -> Result<(), WebReddi> {
    io.web_gerektir().map_err(|_| WebReddi {
        durum: 503,
        mesaj: "çalışma politikası ağ sunucusunu açmıyor",
    })?;
    io.ic.rota_guvenligini_denetle(erisim, csrf, csrf_gerekli)
}

pub(super) fn istemci_adresi<T: GirdiCikti>(io: &mut PolitikaliIo<T>) -> String {
    if io.politika.izin_verir(Yetkinlik::AgSunucusu) {
        io.ic.istemci_adresi()
    } else {
        String::new()
    }
}

pub(super) fn istek_al<T: GirdiCikti>(io: &mut PolitikaliIo<T>) -> Option<String> {
    io.politika
        .izin_verir(Yetkinlik::AgSunucusu)
        .then(|| io.ic.istek_al())
        .flatten()
}
