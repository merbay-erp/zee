//! K-406 (ADR-163 eki, ZM-F048): `<ifade> metnini "<tür>" türünde gönder`.

use super::super::*;
use super::yanit_bicimi_tanisi;

/// K-406: türlü metin yanıtının S037 tanısı; öneri kapalı kümeyi listeler.
fn turlu_metin_tanisi(mesaj: String, satir: usize) -> Tani {
    Tani::yeni("S037", mesaj, satir, 1, 1).onerili(
        "Metin yanıtı türü yalnız \"text/plain\", \"text/csv\", \"text/calendar\" ya da \"application/xml\" olabilir; örnek: `takvim metnini \"text/calendar\" türünde gönder`. HTML/SVG/betik türleri yoktur, ikili içerik için dosya yanıtı kullanılır."
            .into(),
    )
}

impl Ayristirici {
    /// `… gönder` cümlesi: önce türlü metin biçimi, değilse yanıt biçimi.
    pub(in crate::ayristirici) fn gonder_ayristir(
        &self,
        tokenlar: Vec<Token>,
        satir: usize,
    ) -> Result<Cumle, Tani> {
        match self.turlu_metin_ayristir(&tokenlar, satir)? {
            Some(cumle) => Ok(cumle),
            None => self.yanit_ayristir(tokenlar, satir),
        }
    }

    /// K-406 (ADR-163 eki, ZM-F048): `<ifade> metnini "<tür>" türünde gönder`.
    /// Tür kaynakta görünen Metin literalidir ve kapalı `icerik_turu` kaydının
    /// metin satırlarından biri olmak zorundadır; aksi S037. Kalıp değilse
    /// `None` döner ve yanıt cümlesi eski biçimiyle ayrıştırılır.
    fn turlu_metin_ayristir(
        &self,
        tokenlar: &[Token],
        satir: usize,
    ) -> Result<Option<Cumle>, Tani> {
        // Son token `gönder`dir (çağıran seçti).
        let tokenlar = &tokenlar[..tokenlar.len().saturating_sub(1)];
        let n = tokenlar.len();
        if n < 3 || !kelime_mi(&tokenlar[n - 1], "türünde") {
            return Ok(None);
        }
        let Some(metnini) = tokenlar[..n - 1]
            .iter()
            .rposition(|t| kelime_mi(t, "metnini"))
        else {
            return Ok(None);
        };
        let tur = match &tokenlar[metnini + 1..n - 1] {
            [Token {
                tur: TokenTur::Metin(tur),
                ..
            }] => tur.clone(),
            _ => {
                return Err(turlu_metin_tanisi(
                    "Metin yanıtının türü kaynakta görünen tırnaklı bir içerik türüdür.".into(),
                    satir,
                ))
            }
        };
        if crate::icerik_turu::web_metin_turu(&tur).is_none() {
            return Err(turlu_metin_tanisi(
                format!("\"{tur}\" metin yanıtı için kayıtlı bir içerik türü değil."),
                satir,
            ));
        }
        if metnini == 0 {
            return Err(yanit_bicimi_tanisi(satir));
        }
        let deger = ile_ifadesi(&tokenlar[..metnini], satir, &self.islem_adlari)?;
        Ok(Some(Cumle::YanitGonder {
            deger,
            durum: None,
            tur: Some(tur),
            satir,
        }))
    }
}
