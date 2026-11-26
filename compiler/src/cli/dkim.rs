//! K-397 Dilim 2 (RFC-0044 B3, ADR-174): `dkim_anahtarları` özel
//! anahtarlarının açılış yüklemesi.
//!
//! Private CLI adapter. Her anahtar bildirimdeki credential adıyla K-339
//! sözleşmesinden (önce systemd credential dizini, yoksa ortam; ikisinde
//! birden → ret) okunur, PKCS#8 PEM olarak çözülür ve algoritma/bit sınırı
//! denetlenir. Hata kaynak yürütülmeden çıkış 2'dir ve yalnız alan ile
//! seçiciyi anar; PEM metni okunduktan sonra sıfırlanır.

use dil::dkim_imza::{DkimAnahtarDeposu, DkimImzaAnahtari, AZAMI_PEM_BAYTI};
use dil::dkim_kripto::DkimAlgoritmasi;
use dil::guvenlik::sir_dizini::{gizli_deger_oku, SatirProfili};
use std::sync::OnceLock;

/// Süreçte tek proje; açılışta bir kez kurulur.
pub(crate) static DKIM_ANAHTARLARI: OnceLock<DkimAnahtarDeposu> = OnceLock::new();

/// Bildirilen her anahtarı okur ve doğrular; biri bile kurulamazsa süreç başlamaz.
pub(crate) fn hazirla(bildirimler: &[dil::proje::DkimAnahtarBildirimi]) -> Result<(), String> {
    if bildirimler.is_empty() {
        return Ok(());
    }
    let mut depo = DkimAnahtarDeposu::default();
    for bildirim in bildirimler {
        let kimlik = format!("DKIM anahtarı ({} / {})", bildirim.alan, bildirim.secici);
        let algoritma = DkimAlgoritmasi::ayristir(&bildirim.algoritma)
            .map_err(|hata| format!("{kimlik}: {}", hata.mesaji()))?;
        let pem = gizli_deger_oku(
            &bildirim.credential,
            AZAMI_PEM_BAYTI,
            SatirProfili::CokSatir,
        )
        .map_err(|hata| format!("{kimlik} okunamadı: {}", hata.neden()))?;
        let mut pem = pem.into_bytes();
        let sonuc = std::str::from_utf8(&pem)
            .map_err(|_| "PEM UTF-8 değil")
            .and_then(|metin| {
                DkimImzaAnahtari::pemden(&bildirim.alan, &bildirim.secici, algoritma, metin)
            });
        pem.fill(0);
        std::hint::black_box(&pem);
        depo.ekle(
            &bildirim.credential,
            sonuc.map_err(|neden| format!("{kimlik}: {neden}"))?,
        );
    }
    DKIM_ANAHTARLARI
        .set(depo)
        .map_err(|_| "DKIM anahtarları iki kez kuruldu".to_string())
}
