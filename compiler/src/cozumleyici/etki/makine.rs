//! K-405 makine uç noktası denetimi (T060/T061/T062, ADR-179).
//!
//! Makine ucu CSRF/Origin kapısından muaftır; bunun bedeli rotanın tarayıcı
//! kimliğine hiç ulaşamamasıdır. Oturum/CSRF/çerez yüzeyi önbelleklenebilir
//! rotayla (K-335) aynı izle, çağrılan işlemler dahil derlemede reddedilir.

use super::onbellek::{onbellekli_rotayi_denetle, OnbellekIzi};
use super::Bilgi;
use crate::agac::{Cumle, HttpYontemi, Ifade, Program, RotaErisimi, RotaKipi};
use crate::tani::Tani;
use std::collections::HashMap;

fn makine_mi(govde: &[Cumle]) -> Option<usize> {
    match govde.first() {
        Some(Cumle::RotaPolitikasi {
            kip: RotaKipi::Makine,
            satir,
            ..
        }) => Some(*satir),
        _ => None,
    }
}

fn ham_govde_tanisi(satir: usize) -> Tani {
    Tani::yeni(
        "T062",
        "`isteğin ham gövdesi` yalnız makine uç noktası rotasının kendi gövdesinde okunur.".into(),
        satir,
        1,
        1,
    )
    .onerili(
        "Gövdeyi rotada `gövde isteğin ham gövdesi olsun` ile al ve işleme argüman olarak geçir; rotanın ilk satırı `herkese açık makine uç noktası` olmalı."
            .into(),
    )
}

/// Rota başına K-335 önbellek (T056/T057) ve K-405 makine (T060/T061/T062)
/// kip denetimi; `rota` bir `IstekGeldiginde` cümlesidir.
pub(super) fn ozel_kipleri_denetle(
    rota: &Cumle,
    yontem: HttpYontemi,
    dogrudan: &OnbellekIzi,
    toplam: &OnbellekIzi,
) -> Result<(), Tani> {
    let Cumle::IstekGeldiginde {
        yol, onekli, govde, ..
    } = rota
    else {
        return Ok(());
    };
    uygulama_parolasi_yerini_denetle(yontem, govde, toplam)?;
    onbellekli_rotayi_denetle(yontem, govde, dogrudan, toplam)?;
    let Some(satir) = makine_mi(govde) else {
        return match dogrudan.ham_govde() {
            Some(satir) => Err(ham_govde_tanisi(satir)),
            None => Ok(()),
        };
    };
    let bicim = if yontem != HttpYontemi::Post {
        Some(format!(
            "{} rotası makine uç noktası olamaz; makine ucu yalnız POST rotasıdır.",
            yontem.yazimi()
        ))
    } else if *onekli || !matches!(yol.turu(), Ifade::MetinSabiti(_)) {
        Some("Makine uç noktasının yolu sabit, öneksiz bir metin olmalı.".to_string())
    } else {
        None
    };
    if let Some(mesaj) = bicim {
        return Err(Tani::yeni("T060", mesaj, satir, 1, 1).onerili(
            "Makine ucunu `POST \"/tam/yol\" isteği gelince` biçiminde sabit yollu ayrı bir rota yap."
                .into(),
        ));
    }
    if let Some(dokunus) = OnbellekIzi::oturum_dokunusu(dogrudan, toplam) {
        return Err(Tani::yeni(
            "T061",
            "Makine uç noktası oturum, CSRF belirteci ya da çerez kullanıyor; CSRF'siz rota tarayıcı kimliğine ulaşamaz."
                .into(),
            dokunus,
            1,
            1,
        )
        .onerili(
            "Oturuma bağlı işi ayrı, CSRF korumalı bir rotaya taşı; makine ucu yalnız isteğin kendisiyle çalışır."
                .into(),
        ));
    }
    Ok(())
}

/// RFC-0047 E (ADR-182): `yeni uygulama parolası` yalnız oturumlu ya da
/// yetkili, durum değiştiren (CSRF korumalı) rotada — gövdesinde ya da
/// geçişli çağırdığı işlemde — üretilir (T063). Rota dışı kod (CLI) serbesttir.
fn uygulama_parolasi_yerini_denetle(
    yontem: HttpYontemi,
    govde: &[Cumle],
    toplam: &OnbellekIzi,
) -> Result<(), Tani> {
    let Some(satir) = toplam.uygulama_parolasi else {
        return Ok(());
    };
    let herkese_acik = !matches!(
        govde.first(),
        Some(Cumle::RotaPolitikasi {
            erisim: RotaErisimi::Oturumlu | RotaErisimi::Rol(_),
            ..
        })
    );
    if !herkese_acik && !yontem.guvenli() {
        return Ok(());
    }
    Err(Tani::yeni(
        "T063",
        format!(
            "`yeni uygulama parolası` {} rotasında üretilemez; parola yalnız oturumlu, durum değiştiren rotada üretilir.",
            if herkese_acik { "herkese açık" } else { yontem.yazimi() }
        ),
        satir,
        1,
        1,
    )
    .onerili(
        "Parolayı `\"kullanıcı\" yetkisi gerekli` ile başlayan bir POST rotasında üret (CSRF korumalı form); GET, herkese açık ve makine ucu rotası üretemez."
            .into(),
    ))
}

/// İşlemlerin doğrudan etki bilgisi. İşlem/eylem gövdeleri ve rota dışı üst
/// düzey cümleler `isteğin ham gövdesi` okuyamaz (T062).
pub(super) fn islem_bilgileri(
    program: &Program,
    blok_bilgisi: fn(&[Cumle]) -> Bilgi,
) -> Result<HashMap<String, Bilgi>, Tani> {
    let dogrudan = program
        .islemler
        .iter()
        .map(|(ad, islem)| (ad.clone(), blok_bilgisi(&islem.govde)))
        .collect::<HashMap<_, _>>();
    let mut adlar = dogrudan.keys().collect::<Vec<_>>();
    adlar.sort();
    let islemde = adlar
        .into_iter()
        .find_map(|ad| dogrudan[ad].onbellek.ham_govde());
    let rota_disi = program
        .cumleler
        .iter()
        .filter(|cumle| !matches!(cumle, Cumle::IstekGeldiginde { .. }))
        .cloned()
        .collect::<Vec<_>>();
    match islemde.or_else(|| blok_bilgisi(&rota_disi).onbellek.ham_govde()) {
        Some(satir) => Err(ham_govde_tanisi(satir)),
        None => Ok(dogrudan),
    }
}
