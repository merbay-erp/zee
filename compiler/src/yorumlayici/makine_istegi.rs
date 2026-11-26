//! K-405 (ADR-179): makine uç noktası isteğinin ham gövdesini form
//! ayrıştırmasından ve tarayıcı kimliğinden ayırır.

use super::*;

/// Rota ortamındaki ham gövde anahtarı; boşluk taşıdığı için kaynakta ad
/// olarak yazılamaz, yalnız `isteğin ham gövdesi` intrinsic'i okur.
pub(super) const HAM_GOVDE_ANAHTARI: &str = "isteğin ham gövdesi";

/// `Some((zarf, gövde))`: istek bir makine ucunun POST yoluna gidiyor. Zarf
/// yalnız istek satırıdır; çerez satırı (iç oturum alanları dahil) atılır,
/// gövde form olarak ayrıştırılmaz. Native adaptör aynı kümeyle gövdeyi
/// okumadan 413/415 verir; burada tavan ikinci kez denetlenir.
pub(super) fn ayir(
    program: &Program,
    ham: &str,
) -> Result<Option<(String, String)>, (u16, &'static str)> {
    let (ilk_satir, _, govde) = istek_zarfi_parcala(ham);
    let Some((yontem, hedef)) = ilk_satir.split_once(' ') else {
        return Ok(None);
    };
    let yol = hedef.trim().split('?').next().unwrap_or_default();
    if !yontem.eq_ignore_ascii_case("POST") || !program.makine_uc_noktasi_yollari().contains(yol) {
        return Ok(None);
    }
    if govde.len() > crate::agac::MAKINE_GOVDE_SINIRI {
        return Err((413, "makine uç noktası gövdesi 8 KiB sınırını aşıyor"));
    }
    Ok(Some((ilk_satir.to_string(), govde.to_string())))
}

/// Makine rotasının ortamına ham gövdeyi koyar. Makine sınıfı istek makine
/// olmayan bir rotaya (önekli yol) düşerse gövde verilmez; o rota CSRF
/// belirtecini bulamaz ve 403 ile kapanır.
pub(super) fn ortama_ekle(
    rota_ortami: &mut HashMap<String, Deger>,
    makine: bool,
    ham_govde: &Option<String>,
    satir: usize,
) -> Result<(), Tani> {
    if let (true, Some(govde)) = (makine, ham_govde) {
        let deger = Deger::Metin(govde.clone());
        ortama_yazma_butcesini_tuket(rota_ortami, HAM_GOVDE_ANAHTARI, &deger, satir)?;
        rota_ortami.insert(HAM_GOVDE_ANAHTARI.into(), deger);
    }
    Ok(())
}

/// `isteğin ham gövdesi`: yalnız makine rotası ortamında bulunur (T062).
pub(super) fn ham_govde(ortam: &HashMap<String, Deger>, satir: usize) -> Result<Deger, Tani> {
    ortam
        .get(HAM_GOVDE_ANAHTARI)
        .cloned()
        .ok_or_else(|| ic_hata(satir))
}
