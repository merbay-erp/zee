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

pub(super) fn oturumlari_listele<T: GirdiCikti>(
    io: &mut PolitikaliIo<T>,
    kullanici: &str,
) -> Result<Vec<Vec<(String, String)>>, String> {
    io.web_oturumu_gerektir()?;
    io.ic.kullanici_oturumlarini_listele(kullanici)
}

pub(super) fn oturumu_iptal_et<T: GirdiCikti>(
    io: &mut PolitikaliIo<T>,
    kimlik: &str,
) -> Result<i64, String> {
    io.web_oturumu_gerektir()?;
    io.ic.oturum_kimligini_iptal_et(kimlik)
}
