//! RFC-0047 F/E (ADR-182 dilim C): `dav istemcisi gerekli` rota sınıfının
//! statik kapıları (T064/T065/T066) ve kimlik çözücü işleminin denetimi
//! (T067).
//!
//! DAV sınıfı CSRF/Origin kapısından muaftır; bunun bedeli rotanın çerez
//! oturumuna hiç dokunamamasıdır. Kimlik yalnız istekte açıkça taşınan Basic +
//! uygulama parolasından gelir ve `oturum` sözlüğüne istek yerel konur.

use super::onbellek::OnbellekIzi;
use super::{blok_bilgisi, etkileri_yay, makine};
use crate::agac::{Cumle, HttpYontemi, Ifade, IslemTuru, Program, RotaKipi};
use crate::tani::Tani;

pub(super) fn dav_rotasi(govde: &[Cumle]) -> bool {
    dav_mi(govde).is_some()
}

fn baslik_tanisi(satir: usize) -> Tani {
    Tani::yeni(
        "T068",
        "`isteğin \"<ad>\" başlığı` yalnız `dav istemcisi gerekli` rotasının kendi gövdesinde okunur.".into(),
        satir,
        1,
        1,
    )
    .onerili(
        "Başlığı DAV rotasında `derinlik isteğin \"Depth\" başlığı olsun` ile al ve işleme argüman olarak geçir."
            .into(),
    )
}

fn etag_tanisi(satir: usize, mesaj: &str) -> Tani {
    Tani::yeni("T069", mesaj.into(), satir, 1, 1).onerili(
        "ETag'i DAV rotasının kendi gövdesinde, yanıt cümlesinden önce bir kez `yanıta \"ETag\" başlığını etiket olarak ekle` ile yaz."
            .into(),
    )
}

/// İşlem/eylem gövdeleri ve rota dışı üst düzey cümleler DAV başlığı okuyamaz
/// (T068) ve ETag yazamaz (T069).
pub(super) fn rota_disi_izleri_denetle<'a>(
    izler: impl Iterator<Item = &'a OnbellekIzi>,
) -> Result<(), Tani> {
    for iz in izler {
        if let Some(satir) = iz.baslik {
            return Err(baslik_tanisi(satir));
        }
        if let Some(satir) = iz.etag {
            return Err(etag_tanisi(
                satir,
                "ETag yalnız `dav istemcisi gerekli` rotasının kendi gövdesinde yazılır.",
            ));
        }
    }
    Ok(())
}

fn dav_mi(govde: &[Cumle]) -> Option<usize> {
    match govde.first() {
        Some(Cumle::RotaPolitikasi {
            kip: RotaKipi::Dav,
            satir,
            ..
        }) => Some(*satir),
        _ => None,
    }
}

/// Rota başına DAV sınıfı kuralları; `rota` bir `IstekGeldiginde` cümlesidir.
pub(super) fn rotayi_denetle(
    rota: &Cumle,
    yontem: HttpYontemi,
    dogrudan: &OnbellekIzi,
    toplam: &OnbellekIzi,
) -> Result<(), Tani> {
    let Cumle::IstekGeldiginde {
        yol, govde, satir, ..
    } = rota
    else {
        return Ok(());
    };
    let Some(politika) = dav_mi(govde) else {
        if matches!(yontem, HttpYontemi::Propfind | HttpYontemi::Report) {
            return Err(Tani::yeni(
                "T064",
                format!(
                    "{} rotası yalnız `dav istemcisi gerekli` politikasıyla yazılır.",
                    yontem.yazimi()
                ),
                *satir,
                1,
                1,
            )
            .onerili(
                "Rotanın ilk satırına `dav istemcisi gerekli` yaz; PROPFIND ve REPORT kimlikli DAV istemci yöntemleridir, makine ucu ya da oturumlu rota olamaz."
                    .into(),
            ));
        }
        if let Some(satir) = dogrudan.baslik {
            return Err(baslik_tanisi(satir));
        }
        if let Some(satir) = dogrudan.etag {
            return Err(etag_tanisi(
                satir,
                "ETag yalnız `dav istemcisi gerekli` rotasında yazılır.",
            ));
        }
        return Ok(());
    };
    if let Some(satir) = dogrudan.etag_ikinci {
        return Err(etag_tanisi(satir, "DAV rotasında ETag bir kez yazılır."));
    }
    let bicim = if !yontem.dav_yontemi() {
        Some(format!(
            "{} rotası DAV istemci rotası olamaz; DAV yalnız GET, HEAD, PROPFIND, REPORT, PUT ve DELETE yöntemlerindedir.",
            yontem.yazimi()
        ))
    } else if !matches!(yol.turu(), Ifade::MetinSabiti(_)) {
        Some("DAV istemci rotasının yolu sabit bir metin (tam ya da önekli) olmalı.".into())
    } else {
        None
    };
    if let Some(mesaj) = bicim {
        return Err(Tani::yeni("T065", mesaj, politika, 1, 1).onerili(
            "Durum değiştiren DAV işini PUT ya da DELETE rotasına taşı; POST/PATCH tarayıcının çapraz kökenden sahteleyebildiği yöntemlerdir."
                .into(),
        ));
    }
    if let Some(dokunus) = toplam.dokunus() {
        return Err(Tani::yeni(
            "T066",
            "DAV istemci rotası çerez oturumu, CSRF belirteci ya da çerez kullanıyor; CSRF'siz rota tarayıcı kimliğine ulaşamaz."
                .into(),
            dokunus,
            1,
            1,
        )
        .onerili(
            "Kimliği rotada `oturumun \"kullanıcı\" değeri` ile oku; oturum açma, çerez ve CSRF işini ayrı, CSRF korumalı bir rotaya taşı."
                .into(),
        ));
    }
    Ok(())
}

fn cozucu_tanisi(mesaj: String, satir: usize) -> Tani {
    Tani::yeni("T067", mesaj, satir, 1, 1).onerili(
        "proje.dil içinde `dav_kimlik_çözücüsü \"kimlik_coz\" olsun` yaz; işlem `kullanıcıyı Metin olarak al`, `öneki Metin olarak al` parametreli, salt okuma olmalı ve `{özet, kullanıcı, rol, kapsam, kimlik}` sözlüğü (kayıt yoksa boş sözlük) döndürmeli."
            .into(),
    )
}

/// Proje kipi: DAV rotası varsa çözücü bildirilmiş olmalı; bildirilen çözücü
/// iki parametreli, değer döndüren, durum değiştirmeyen bir `işlem`dir.
pub(in crate::cozumleyici) fn cozucuyu_denetle(
    program: &Program,
    cozucu: Option<&str>,
) -> Result<(), Tani> {
    let dav_satiri = program.cumleler.iter().find_map(|cumle| match cumle {
        Cumle::IstekGeldiginde { govde, .. } => dav_mi(govde),
        _ => None,
    });
    let Some(ad) = cozucu else {
        return match dav_satiri {
            Some(satir) => Err(cozucu_tanisi(
                "`dav istemcisi gerekli` rotası var ama proje `dav_kimlik_çözücüsü` bildirmiyor."
                    .into(),
                satir,
            )),
            None => Ok(()),
        };
    };
    let satir = dav_satiri.unwrap_or(1);
    let Some(islem) = program.islemler.get(ad) else {
        return Err(cozucu_tanisi(
            format!("Kimlik çözücü \"{ad}\" programda tanımlı bir işlem değil."),
            satir,
        ));
    };
    if islem.tur != IslemTuru::Islem
        || islem.parametreler.len() != 2
        || islem.donus_turu_yazimi.as_deref() == Some("DeğerDöndürmez")
    {
        return Err(cozucu_tanisi(
            format!("Kimlik çözücü \"{ad}\" iki parametreli, değer döndüren bir `işlem` olmalı."),
            islem.satir,
        ));
    }
    let dogrudan = makine::islem_bilgileri(program, blok_bilgisi)?;
    let etkiler = etkileri_yay(&dogrudan);
    if etkiler
        .get(ad)
        .is_some_and(|etki| etki.durum_yazma || etki.web || etki.geri_alinamaz)
    {
        return Err(cozucu_tanisi(
            format!("Kimlik çözücü \"{ad}\" salt okuma olmalı; durum, web ya da geri alınamaz etki taşıyor."),
            islem.satir,
        ));
    }
    Ok(())
}
