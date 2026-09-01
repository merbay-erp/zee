use super::*;

/// Özellik sözdizimi bir yapı alanına çözüldüğünde alt ifade kutusunu taşır.
/// Klonlama, eski alt düğüm adresinin HIR kaydını yetim bırakırdı.
pub(super) fn ozelligi_alana_donustur(
    ifade: &mut Ifade,
    alan: String,
    satir: usize,
) -> Result<(), Tani> {
    let eski = std::mem::replace(ifade, Ifade::BosListe);
    let Ifade::Ozellik { nesne, .. } = eski else {
        return Err(ic_tutarlilik_hatasi(
            "Özellik→alan dönüşümünde AST varyantı kayboldu",
            satir,
        ));
    };
    *ifade = Ifade::AlanErisim { nesne, alan };
    Ok(())
}
