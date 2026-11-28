//! K-406 (ADR-163 eki, ZM-F048): `<ifade> metnini "<tür>" türünde gönder`;
//! RFC-0047 C (ADR-182): durumlu kardeşi `… türünde <durum> durumuyla gönder`.

use super::super::*;
use super::yanit_bicimi_tanisi;

/// K-406: türlü metin yanıtının S037 tanısı; öneri kapalı kümeyi listeler.
fn turlu_metin_tanisi(mesaj: String, satir: usize) -> Tani {
    Tani::yeni("S037", mesaj, satir, 1, 1).onerili(
        "Metin yanıtı türü yalnız \"text/plain\", \"text/csv\", \"text/calendar\", \"text/vcard\" ya da \"application/xml\" olabilir; örnek: `takvim metnini \"text/calendar\" türünde gönder`. HTML/SVG/betik türleri yoktur, ikili içerik için dosya yanıtı kullanılır."
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
        let mut tokenlar = &tokenlar[..tokenlar.len().saturating_sub(1)];
        let durum = match self.turlu_durumu(tokenlar, satir)? {
            Some(durum) => {
                tokenlar = &tokenlar[..tokenlar.len() - 2];
                Some(durum)
            }
            None => None,
        };
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
            durum,
            tur: Some(tur),
            satir,
        }))
    }

    /// RFC-0047 C (ADR-182): `… türünde <durum> durumuyla` son eki. Durum
    /// kaynakta görünen TamSayı literalidir ve kapalı
    /// `TURLU_YANIT_DURUMLARI` kümesinden olmak zorundadır; aksi S037.
    /// `None`, son ekin türlü biçime ait olmadığını söyler.
    fn turlu_durumu(&self, tokenlar: &[Token], satir: usize) -> Result<Option<u16>, Tani> {
        let n = tokenlar.len();
        if n < 3
            || !kelime_mi(&tokenlar[n - 1], "durumuyla")
            || !kelime_mi(&tokenlar[n - 3], "türünde")
        {
            return Ok(None);
        }
        match tokenlar[n - 2].tur {
            TokenTur::TamSayi(kod)
                if u16::try_from(kod)
                    .is_ok_and(|kod| crate::agac::TURLU_YANIT_DURUMLARI.contains(&kod)) =>
            {
                Ok(Some(kod as u16))
            }
            _ => Err(Tani::yeni(
                "S037",
                "Türlü metin yanıtının durumu kapalı kümeden sabit bir TamSayı olmalı.".into(),
                satir,
                1,
                1,
            )
            .onerili(
                "Durum yalnız 200, 201, 207, 400, 403, 404, 409, 412 ya da 415 olabilir; örnek: `xml metnini \"application/xml\" türünde 207 durumuyla gönder`. Gövdesiz yanıt için `\"\" yanıtını 204 durumuyla gönder` kullanılır."
                    .into(),
            )),
        }
    }
}
