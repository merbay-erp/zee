//! K-391 (ADR-168): web isteğinin yorumlayıcı tarafındaki boyut denetimi.
//! Native adaptör bayt düzeyi zarfı uygular; burada aynı isteğin gövde ve
//! alan sayısı sınırı ikinci kez, adaptörün bildirdiği bütçeyle denetlenir.

use super::*;

/// K-391 (ADR-168): native adaptörün bu istekte uyguladığı gövde bütçesini
/// yorumlayıcının ikinci denetimine taşıyan iç çerez. İstemcinin aynı adlı
/// çerezi adaptörde atılır; uygulama çerezlerine hiç girmez.
pub(super) const IC_GOVDE_SINIRI: &str = "__zee_ic_govde_siniri";

/// K-391: adaptör varsayılan (64 KiB) dışında bir gövde bütçesi uyguladıysa
/// (rota bazlı sınır, multipart'tan yeniden kodlanan form) bunu iç çerezle
/// isteğe ekler. Varsayılan bütçede istek baytı değişmez.
pub fn govde_sinirini_istege_ekle(ham: &str, sinir: usize) -> String {
    if sinir == AZAMI_ISTEK_GOVDESI {
        return ham.to_string();
    }
    let (ilk_satir, cerez_satiri, govde) = istek_zarfi_parcala(ham);
    let ayrac = if cerez_satiri.is_empty() { "" } else { "; " };
    format!(
        "{ilk_satir}
çerez {cerez_satiri}{ayrac}{IC_GOVDE_SINIRI}={sinir}
{govde}"
    )
}

pub(super) fn istek_sinirlarini_denetle(ham: &str) -> Result<(), (u16, &'static str)> {
    let (ilk_satir, cerez_satiri, govde) = istek_zarfi_parcala(ham);
    let (yontem, hedef) = ilk_satir.split_once(' ').unwrap_or(("GET", ilk_satir));
    let (_, sorgu) = hedef.split_once('?').unwrap_or((hedef, ""));
    let sinir = cerez_satiri
        .split(';')
        .filter_map(|cift| cift.trim().split_once('='))
        .find(|(ad, _)| *ad == IC_GOVDE_SINIRI)
        .and_then(|(_, deger)| deger.parse::<usize>().ok())
        .unwrap_or(AZAMI_ISTEK_GOVDESI);
    if govde.len() > sinir {
        return Err((
            413,
            if sinir == AZAMI_ISTEK_GOVDESI {
                "istek gövdesi 64 KiB sınırını aşıyor"
            } else {
                "istek gövdesi rota sınırını aşıyor"
            },
        ));
    }
    let govde_kullanilir = matches!(
        yontem.to_ascii_uppercase().as_str(),
        "POST" | "PUT" | "PATCH" | "DELETE"
    );
    let alan_sayisi = sorgu.split('&').filter(|alan| !alan.is_empty()).count()
        + if govde_kullanilir {
            govde.split('&').filter(|alan| !alan.is_empty()).count()
        } else {
            0
        };
    if alan_sayisi > AZAMI_ISTEK_ALANI {
        return Err((413, "istek 100 alan sınırını aşıyor"));
    }
    Ok(())
}
