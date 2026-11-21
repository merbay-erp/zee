//! Süreçler arasında atomik, proje-yerel kalıcı web durumu.

use super::DepoDurumu;
use std::io;
use std::path::{Path, PathBuf};

pub(in crate::web_guvenligi) struct KaliciDepo {
    yol: PathBuf,
}

impl KaliciDepo {
    pub(super) const fn yeni(yol: PathBuf) -> Self {
        Self { yol }
    }

    /// K-353: durum kilit altında okunur, dönüştürülür ve yayımlanır. Eski
    /// iyimser döngü (kilitsiz okuma + CAS, 128 deneme) çok işçide yavaş
    /// süreci aç bırakıp isteği 503 ile reddediyordu.
    pub(super) fn guncelle<R, F>(&self, mut islem: F) -> Result<R, String>
    where
        F: FnMut(&mut DepoDurumu) -> Result<R, String>,
    {
        use crate::kalici_dosya::KilitliGuncellemeHatasi as Hata;
        self.hazirla()?;
        crate::kalici_dosya::kilit_altinda_guncelle(&self.yol, |onceki| {
            let mut durum = match &onceki {
                Some(baytlar) if !baytlar.is_empty() => {
                    serde_json::from_slice::<DepoDurumu>(baytlar)
                        .map_err(|hata| format!("kalıcı web deposu JSON'u geçersiz: {hata}"))?
                }
                Some(_) | None => DepoDurumu::default(),
            };
            self.dogrula(&durum)?;
            let sonuc = islem(&mut durum)?;
            durum.degisiklik_sirasi = durum.degisiklik_sirasi.saturating_add(1);
            self.dogrula(&durum)?;
            let yeni = serde_json::to_vec(&durum)
                .map_err(|hata| format!("kalıcı web deposu yazılamadı: {hata}"))?;
            Ok((yeni, sonuc))
        })
        .map_err(|hata| match hata {
            Hata::Okuma(hata) => format!("kalıcı web deposu okunamadı: {hata}"),
            Hata::Islem(hata) => hata,
            Hata::Yazma(hata) => format!("kalıcı web deposu yayımlanamadı: {hata}"),
        })
    }

    fn hazirla(&self) -> Result<(), String> {
        let ebeveyn = self.yol.parent().unwrap_or_else(|| Path::new("."));
        match std::fs::symlink_metadata(ebeveyn) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(
                    "kalıcı web depo klasörü normal dizin olmalı; sembolik bağ reddedildi".into(),
                );
            }
            Ok(_) => {}
            Err(hata) if hata.kind() == io::ErrorKind::NotFound => {
                std::fs::create_dir_all(ebeveyn)
                    .map_err(|hata| format!("kalıcı web depo klasörü açılamadı: {hata}"))?;
            }
            Err(hata) => return Err(format!("kalıcı web depo klasörü okunamadı: {hata}")),
        }
        match std::fs::symlink_metadata(&self.yol) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(
                    "kalıcı web deposu normal dosya olmalı; sembolik bağ reddedildi".into(),
                );
            }
            Ok(_) => {}
            Err(hata) if hata.kind() == io::ErrorKind::NotFound => {}
            Err(hata) => return Err(format!("kalıcı web depo türü okunamadı: {hata}")),
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(ebeveyn, std::fs::Permissions::from_mode(0o700))
                .map_err(|hata| format!("kalıcı web depo klasörü korunamadı: {hata}"))?;
            if !self.yol.exists() {
                use std::os::unix::fs::OpenOptionsExt;
                match std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .mode(0o600)
                    .open(&self.yol)
                {
                    Ok(_) => {}
                    Err(hata) if hata.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(hata) => {
                        return Err(format!("kalıcı web depo dosyası oluşturulamadı: {hata}"));
                    }
                }
            }
        }
        Ok(())
    }

    fn dogrula(&self, durum: &DepoDurumu) -> Result<(), String> {
        let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.web();
        if durum.bicim_surumu != 1 {
            return Err(format!(
                "kalıcı web deposu biçim sürümü desteklenmiyor: {}",
                durum.bicim_surumu
            ));
        }
        if durum.oturumlar.len() > sinirlar.oturum_sayisi()
            || durum.oranlar.len() > sinirlar.oran_anahtari_sayisi()
        {
            return Err("kalıcı web deposu kayıt sınırını aşıyor".into());
        }
        if durum
            .oturumlar
            .values()
            .filter(|oturum| oturum.kullanici.is_none())
            .count()
            > sinirlar.anonim_oturum_sayisi()
        {
            return Err("kalıcı web deposu anonim oturum sınırını aşıyor".into());
        }
        if durum
            .oturumlar
            .keys()
            .chain(durum.oranlar.keys())
            .any(|anahtar| {
                anahtar.len() != 64
                    || !anahtar
                        .bytes()
                        .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
            })
        {
            return Err("kalıcı web deposunda kanonik olmayan SHA-256 anahtarı var".into());
        }
        if durum.oturumlar.values().any(|oturum| {
            oturum.csrf.len() > 256
                || oturum
                    .kullanici
                    .as_ref()
                    .is_some_and(|deger| deger.len() > 1_024)
                || oturum.rol.as_ref().is_some_and(|deger| deger.len() > 1_024)
        }) {
            return Err("kalıcı web deposunda alan boyutu sınırı aşıldı".into());
        }
        Ok(())
    }
}
