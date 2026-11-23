//! K-378 (RFC-0041 katman B): JS'siz tarayıcı yüklemesi için isteğe bağlı
//! `multipart/form-data` yolu. Bildirimsiz projede multipart istek bugünkü
//! gibi davranır; açılınca CSRF form alanından okunduğu için `web-oturumu`
//! ZORUNLUDUR.
use super::*;

pub(super) fn alani_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
) -> Result<bool, Tani> {
    let Some((ifade, satir)) = alanlar.remove("çok_parçalı_yükleme") else {
        return Ok(false);
    };
    let Ifade::MantiksalSabiti(acik) = ifade.turu() else {
        return Err(proje_hatasi(
            "P001",
            "\"çok_parçalı_yükleme\" alanı doğru ya da yanlış olmalı.",
            satir,
            "Örnek: çok_parçalı_yükleme doğru olsun",
        ));
    };
    if *acik && !yetkinlik_yazimlari.iter().any(|y| y == "web-oturumu") {
        return Err(proje_hatasi(
            "P015",
            "Çok parçalı form yüklemesi açık ama `web-oturumu` yetkinliği açık değil.",
            satir,
            "Yükleme CSRF belirtecini oturumdan doğrular; yetkinliklere \"web-oturumu\" ve \"ağ-sunucusu\" ekle.",
        ));
    }
    Ok(*acik)
}
