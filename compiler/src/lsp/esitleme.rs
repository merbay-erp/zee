//! LSP UTF-16 değişikliklerini önce özel adayda doğrula, sonra atomik yayınla.

use super::{belge_uri, Json};
use std::collections::HashMap;

fn sayi(deger: &Json) -> Option<usize> {
    let Json::Sayi(sayi) = deger else { return None };
    sayi.ham().parse().ok()
}

/// CRLF tek satır sonudur. Surrogate çiftinin ortasına konum kabul edilmez.
fn konum(metin: &str, deger: &Json) -> Option<usize> {
    let satir = sayi(deger.alan("line")?)?;
    let sutun = sayi(deger.alan("character")?)?;
    let mut bas = 0;
    for _ in 0..satir {
        bas += metin.get(bas..)?.find('\n')? + 1;
    }
    let son = metin[bas..].find('\n').map_or(metin.len(), |n| bas + n);
    let son = if son > bas && metin.as_bytes()[son - 1] == b'\r' {
        son - 1
    } else {
        son
    };
    let mut birim = 0;
    for (bayt, harf) in metin[bas..son].char_indices() {
        if birim == sutun {
            return Some(bas + bayt);
        }
        birim += harf.len_utf16();
        if birim > sutun {
            return None;
        }
    }
    (birim == sutun).then_some(son)
}

pub(super) fn degisim_parametreleri(
    mesaj: &Json,
    belgeler: &HashMap<String, String>,
) -> Option<(String, String)> {
    let uri = belge_uri(mesaj)?;
    let Json::Dizi(ogeler) = mesaj.alan("params")?.alan("contentChanges")? else {
        return None;
    };
    if ogeler.is_empty() {
        return None;
    }
    // Ranged edit requires an open document; a full replacement retains legacy behavior.
    let mut aday = belgeler.get(&uri).cloned();
    let azami = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.kaynak_bayti();
    for oge in ogeler {
        let yeni = oge.alan("text")?.metin()?;
        match oge.alan("range") {
            None => {
                aday = Some(yeni.to_string());
            }
            Some(aralik) => {
                let metin = aday.as_mut()?;
                let bas = konum(metin, aralik.alan("start")?)?;
                let son = konum(metin, aralik.alan("end")?)?;
                if bas > son {
                    return None;
                }
                if let Some(uzunluk) = oge.alan("rangeLength") {
                    if sayi(uzunluk)? != metin[bas..son].encode_utf16().count() {
                        return None;
                    }
                }
                let boy = metin
                    .len()
                    .checked_sub(son - bas)?
                    .checked_add(yeni.len())?;
                if boy > azami {
                    return None;
                }
                metin.replace_range(bas..son, yeni);
            }
        }
    }
    Some((uri, aday?))
}

/// Bütün belgenin kanonik biçimini UTF-16 aralıklı tek düzenleme olarak yazar.
pub(super) fn bicim_yaniti(
    kimlik: Option<&Json>,
    metin: Option<&str>,
) -> super::CiktiSonucu<String> {
    super::yanit_uret(kimlik, |yazici| {
        let Some(metin) = metin else {
            return yazici.ham("null");
        };
        let Ok(bicimli) = crate::bicimleyici::bicimle(metin) else {
            return yazici.ham("null");
        };
        if bicimli == metin {
            return yazici.ham("[]");
        }
        let satir = metin.bytes().filter(|b| *b == b'\n').count();
        let sutun = metin
            .rsplit('\n')
            .next()
            .unwrap_or("")
            .encode_utf16()
            .count();
        yazici.ham("[{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":")?;
        yazici.ham(&satir.to_string())?;
        yazici.ham(",\"character\":")?;
        yazici.ham(&sutun.to_string())?;
        yazici.ham("}},\"newText\":")?;
        yazici.metin(&bicimli)?;
        yazici.ham("}]")
    })
}
