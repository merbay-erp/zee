//! K-397 Dilim 2 (RFC-0044 B3, ADR-174): `dkim_anahtarları` özel
//! anahtarlarının açılış yüklemesi.
//!
//! Private CLI adapter. Her anahtar bildirimdeki credential adıyla K-339
//! sözleşmesinden (önce systemd credential dizini, yoksa ortam; ikisinde
//! birden → ret) okunur, PKCS#8 PEM olarak çözülür ve algoritma/bit sınırı
//! denetlenir. Hata kaynak yürütülmeden çıkış 2'dir ve yalnız alan ile
//! seçiciyi anar; PEM metni okunduktan sonra sıfırlanır.
//!
//! K-413 (ADR-174 §9): proje `dns` yetkinliği ve çözücü bildiriyorsa her
//! anahtarın `<s>._domainkey.<d>` TXT kaydı açılışta bir kez sorulur.
//! Yayımlanan `p=` özel anahtara uymuyorsa ya da iptal edilmişse yalnız o
//! anahtar devre dışı kalır ve stderr'e `HATA:` yazılır; süreç ve öteki
//! anahtarlar çalışır (çok kiracılı kurulumda bir kiracının DNS'i hizmeti
//! durduramaz). Kayıt yoksa ya da DNS geçici hata verirse `UYARI:` yazılır,
//! anahtar etkin kalır.

use dil::dkim_imza::{DkimAnahtarDeposu, DkimImzaAnahtari, AZAMI_PEM_BAYTI};
use dil::dkim_kripto::DkimAlgoritmasi;
use dil::guvenlik::sir_dizini::{gizli_deger_oku, SatirProfili};
use std::sync::OnceLock;

/// Süreçte tek proje; açılışta bir kez kurulur.
pub(crate) static DKIM_ANAHTARLARI: OnceLock<DkimAnahtarDeposu> = OnceLock::new();

/// Bildirilen her anahtarı okur ve doğrular; biri bile kurulamazsa süreç başlamaz.
pub(crate) fn hazirla(
    bildirimler: &[dil::proje::DkimAnahtarBildirimi],
    dns: Option<&dil::dns_teli::DnsAyari>,
) -> Result<(), String> {
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
    if let Some(ayar) = dns {
        for tani in depo.yayimlananlarla_denetle(|ad| txt_sorgula(ayar, ad)) {
            eprintln!("{tani}");
        }
    }
    DKIM_ANAHTARLARI
        .set(depo)
        .map_err(|_| "DKIM anahtarları iki kez kuruldu".to_string())
}

/// K-413: tek TXT sorgusu (K-390 B1a istemcisi, 5 sn); hata yalnız türdür.
fn txt_sorgula(ayar: &dil::dns_teli::DnsAyari, ad: &str) -> Result<Vec<Vec<u8>>, String> {
    use dil::dns_teli::{KayitTuru, KayitVerisi};
    let sure = std::time::Duration::from_secs(5);
    let cozum = dil::dns_istemcisi::sorgula(ayar, ad, KayitTuru::Txt, sure)
        .map_err(|hata| hata.tur.yazimi().to_string())?;
    Ok(cozum
        .kayitlar
        .into_iter()
        .filter_map(|kayit| match kayit.veri {
            KayitVerisi::Txt(txt) => Some(txt),
            _ => None,
        })
        .collect())
}
