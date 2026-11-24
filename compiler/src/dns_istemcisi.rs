//! K-390 (RFC-0044 B1, ADR-169): bildirilmiş loopback doğrulayıcı çözücüye
//! dar stub istemci. Önbellek yoktur (ilke 7); önbellek yerel çözücünündür.
//!
//! - Her sorgu `getrandom` ile 16 bit kimlik ve işletim sisteminin rastgele
//!   geçici kaynak kapısıyla gönderilir; soket çözücüye `connect` edilir.
//! - Kimliği, soru adı/türü/sınıfı eşleşmeyen ya da çözülemeyen paket atılır,
//!   bekleme son tarihe kadar sürer (zehirleme önlemi).
//! - `TC=1` yanıtında aynı sorgu TCP ile (≤ 64 KiB) yinelenir.
//! - Sorgu başına süre en çok 5 sn; çağıran daha kısa son tarih verebilir.
#![forbid(unsafe_code)]

use crate::dns_teli::{
    self, Cozum, DnsAyari, DnsHataTuru, DnsHatasi, DnssecKipi, Durum, KayitTuru, Yanit,
};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::{Duration, Instant};

/// Tek sorgunun en uzun süresi (RFC-0044 B1 sınırlar tablosu).
pub const SORGU_SURESI: Duration = Duration::from_secs(5);

fn kimlik_uret() -> Result<u16, DnsHatasi> {
    let mut bayt = [0u8; 2];
    getrandom::fill(&mut bayt)
        .map_err(|_| DnsHatasi::yeni(DnsHataTuru::Ag, "sorgu kimliği üretilemedi"))?;
    Ok(u16::from_be_bytes(bayt))
}

fn kalan(son: Instant) -> Result<Duration, DnsHatasi> {
    let simdi = Instant::now();
    if simdi >= son {
        return Err(DnsHatasi::yeni(
            DnsHataTuru::ZamanAsimi,
            "DNS sorgusu süresinde yanıtlanmadı",
        ));
    }
    Ok(son - simdi)
}

fn ag_hatasi(hata: std::io::Error) -> DnsHatasi {
    match hata.kind() {
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => DnsHatasi::yeni(
            DnsHataTuru::ZamanAsimi,
            "DNS sorgusu süresinde yanıtlanmadı",
        ),
        _ => DnsHatasi::yeni(DnsHataTuru::Ag, "yerel DNS çözücüsüne ulaşılamadı"),
    }
}

/// Paket bu sorgunun mu? Başlık/soru eşleşmeyen `None` (atılır); eşleşen
/// ama gövdesi bozuk ya da sınırı aşan paket hata olur.
fn eslesen(
    paket: &[u8],
    kimlik: u16,
    ad: &str,
    tur: KayitTuru,
) -> Option<Result<Yanit, DnsHatasi>> {
    if !dns_teli::basligi_eslesiyor(paket, kimlik, ad, tur) {
        return None;
    }
    Some(dns_teli::yanit_ayristir(paket).map_err(DnsHatasi::from))
}

fn udp_sorgula(
    cozucu: SocketAddr,
    paket: &[u8],
    kimlik: u16,
    ad: &str,
    tur: KayitTuru,
    son: Instant,
) -> Result<Yanit, DnsHatasi> {
    let yerel: SocketAddr = if cozucu.is_ipv4() {
        ([127, 0, 0, 1], 0).into()
    } else {
        (std::net::Ipv6Addr::LOCALHOST, 0).into()
    };
    let soket = UdpSocket::bind(yerel).map_err(ag_hatasi)?;
    soket.connect(cozucu).map_err(ag_hatasi)?;
    soket.send(paket).map_err(ag_hatasi)?;
    let mut tampon = vec![0u8; usize::from(dns_teli::UDP_TAMPONU) + 1];
    loop {
        soket
            .set_read_timeout(Some(kalan(son)?))
            .map_err(ag_hatasi)?;
        let uzunluk = soket.recv(&mut tampon).map_err(ag_hatasi)?;
        if uzunluk > usize::from(dns_teli::UDP_TAMPONU) {
            continue; // bildirilen tamponu aşan paket bu sorgunun yanıtı sayılmaz
        }
        if let Some(sonuc) = eslesen(&tampon[..uzunluk], kimlik, ad, tur) {
            return sonuc;
        }
    }
}

fn tcp_sorgula(
    cozucu: SocketAddr,
    paket: &[u8],
    kimlik: u16,
    ad: &str,
    tur: KayitTuru,
    son: Instant,
) -> Result<Yanit, DnsHatasi> {
    let mut akis = TcpStream::connect_timeout(&cozucu, kalan(son)?).map_err(ag_hatasi)?;
    let mut cerceve = (paket.len() as u16).to_be_bytes().to_vec();
    cerceve.extend_from_slice(paket);
    akis.set_write_timeout(Some(kalan(son)?))
        .map_err(ag_hatasi)?;
    akis.write_all(&cerceve).map_err(ag_hatasi)?;
    let mut oku = |tampon: &mut [u8]| -> Result<(), DnsHatasi> {
        let mut okunan = 0;
        while okunan < tampon.len() {
            akis.set_read_timeout(Some(kalan(son)?))
                .map_err(ag_hatasi)?;
            match akis.read(&mut tampon[okunan..]) {
                Ok(0) => {
                    return Err(DnsHatasi::yeni(
                        DnsHataTuru::BozukYanit,
                        "çözücü TCP yanıtını yarıda kesti",
                    ))
                }
                Ok(n) => okunan += n,
                Err(hata) if hata.kind() == std::io::ErrorKind::Interrupted => {}
                Err(hata) => return Err(ag_hatasi(hata)),
            }
        }
        Ok(())
    };
    let mut boy = [0u8; 2];
    oku(&mut boy)?;
    let mut govde = vec![0u8; usize::from(u16::from_be_bytes(boy))];
    oku(&mut govde)?;
    match eslesen(&govde, kimlik, ad, tur) {
        Some(Ok(yanit)) if yanit.kesik => Err(DnsHatasi::yeni(
            DnsHataTuru::BozukYanit,
            "TCP yanıtı da kesik",
        )),
        Some(sonuc) => sonuc,
        None => Err(DnsHatasi::yeni(
            DnsHataTuru::BozukYanit,
            "TCP yanıtı sorguyla eşleşmiyor",
        )),
    }
}

/// Tek sorgu. `azami` çağıranın kalan son tarihidir; 5 sn ile sınırlanır.
/// `doğrulanmış` yalnız `zorunlu` kipte AD bitini taşır.
pub fn sorgula(
    ayar: &DnsAyari,
    ad: &str,
    tur: KayitTuru,
    azami: Duration,
) -> Result<Cozum, DnsHatasi> {
    let ad = dns_teli::ad_dogrula(ad)?;
    let son = Instant::now() + azami.min(SORGU_SURESI);
    let kimlik = kimlik_uret()?;
    let paket = dns_teli::sorgu_kur(kimlik, &ad, tur)?;
    let mut yanit = udp_sorgula(ayar.cozucu, &paket, kimlik, &ad, tur, son)?;
    if yanit.kesik {
        yanit = tcp_sorgula(ayar.cozucu, &paket, kimlik, &ad, tur, son)?;
    }
    let mut cozum = dns_teli::coz(&yanit, &ad, tur)?;
    cozum.dogrulanmis &= ayar.dnssec == DnssecKipi::Zorunlu;
    Ok(cozum)
}

/// `dns_dnssec: "zorunlu"` açılış sınaması: kök `SOA` yanıtı `AD=1` taşımalı.
/// Başarısızlıkta süreç kaynak yürütülmeden durur (fail-closed).
pub fn dnssec_sina(ayar: &DnsAyari) -> Result<(), String> {
    let cozum = sorgula(ayar, ".", KayitTuru::Soa, SORGU_SURESI)
        .map_err(|hata| format!("çözücü sınanamadı ({}): {}", hata.tur.yazimi(), hata.mesaj))?;
    if cozum.durum != Durum::Var || !cozum.dogrulanmis {
        return Err(
            "çözücü kök bölgeyi doğrulanmış (AD=1) olarak yanıtlamadı; DNSSEC doğrulayan yerel çözücü gerekir"
                .into(),
        );
    }
    Ok(())
}
