//! K-386 (RFC-0029): listede 1 tabanlı sıra erişimi kalıpları ve kalıba
//! oturmayan sıra eki için öğretici S049.

use super::*;

/// RFC-0029 (K-386) sıra erişimi: `<liste>nin <sıra>. öğesi` (4 token) ve
/// `<liste>nin <sıra>. öğesini almayı dene` (6 token). Liste ve sıra tek
/// tokendir; tamlayan eki ada bitişiktir ve çözümleyicide ayıklanır.
pub(super) fn kalip(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let n = tokenlar.len();
    if !(n == 4 || n == 6) || tokenlar[2].tur != TokenTur::SiraEki {
        return Ok(None);
    }
    let kelime = |i: usize| match &tokenlar[i].tur {
        TokenTur::Kelime(k) => Some(k.as_str()),
        _ => None,
    };
    let dene = match n {
        4 if kelime(3) == Some("öğesi") => false,
        6 if kelime(3) == Some("öğesini")
            && kelime(4) == Some("almayı")
            && kelime(5) == Some("dene") =>
        {
            true
        }
        _ => return Ok(None),
    };
    let liste = Box::new(tekil_ifade(tokenlar[0].clone())?);
    let sira = Box::new(tekil_ifade(tokenlar[1].clone())?);
    Ok(Some(Ifade::ListeOgesi { liste, sira, dene }))
}

/// Sıra eki taşıyan ama RFC-0029 kalıbına oturmayan bölge için öğretici S049.
pub(super) fn kalip_disi_tanisi(tokenlar: &[Token]) -> Option<Tani> {
    let konum = tokenlar.iter().position(|t| t.tur == TokenTur::SiraEki)?;
    let nokta = &tokenlar[konum];
    let (mesaj, oneri) = match konum {
        // `harflerin 2. öğesi "x" olsun`: ad konumu `harflerin`i yuttu, değer
        // bölgesi sırayla başlıyor — ya liste eksik ya da sıra ile yazma denendi.
        1 => (
            "Sıradan önce tamlayan ekli liste adı gelmeli; sıra ile öğe YAZMA bu sürümde yoktur.",
            "Okuma: x harflerin 2. öğesi olsun — öğeyi değiştirmek için gezme yazmasını kullan (her harf için … harf \"x\" olsun) ya da yeni liste kur (RFC-0029)",
        ),
        2 => (
            "Sıra erişimi tanınmadı: kalıp \"<liste>nin <sıra>. öğesi\" biçimindedir.",
            "Örnek: harflerin 3. öğesi · harflerin sıra. öğesi · harflerin sıra. öğesini almayı dene",
        ),
        _ => (
            "Sıra tek bir ad ya da sayı olmalı ve listenin hemen ardından gelmeli.",
            "Hesaplanan sırayı önce bir ada ver: konum sıra ile 1 in toplamı olsun — sonra: harflerin konum. öğesi",
        ),
    };
    Some(
        Tani::yeni(
            "S049",
            mesaj.into(),
            nokta.satir,
            nokta.sutun,
            nokta.uzunluk,
        )
        .onerili(oneri.into()),
    )
}
