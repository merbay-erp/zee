//! LSP JSON gövdelerini tahsis sırasında kaynak bütçesi içinde üretir.

use std::fmt::{self, Arguments, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CiktiTasmasi;

pub(super) type CiktiSonucu<T> = Result<T, CiktiTasmasi>;

pub(super) struct SinirliJson {
    govde: String,
    azami_bayt: usize,
}

impl SinirliJson {
    pub(super) fn yeni() -> Self {
        Self::sinirli(crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.lsp_yanit_bayti())
    }

    fn sinirli(azami_bayt: usize) -> Self {
        Self {
            govde: String::with_capacity(azami_bayt.min(4 * 1024)),
            azami_bayt,
        }
    }

    pub(super) fn ham(&mut self, parca: &str) -> CiktiSonucu<()> {
        let yeni_boyut = self
            .govde
            .len()
            .checked_add(parca.len())
            .ok_or(CiktiTasmasi)?;
        if yeni_boyut > self.azami_bayt {
            return Err(CiktiTasmasi);
        }
        self.govde.push_str(parca);
        Ok(())
    }

    pub(super) fn bicimle(&mut self, parcalar: Arguments<'_>) -> CiktiSonucu<()> {
        fmt::write(self, parcalar).map_err(|_| CiktiTasmasi)
    }

    pub(super) fn metin(&mut self, metin: &str) -> CiktiSonucu<()> {
        self.ham("\"")?;
        for karakter in metin.chars() {
            match karakter {
                '"' => self.ham("\\\"")?,
                '\\' => self.ham("\\\\")?,
                '\n' => self.ham("\\n")?,
                '\r' => self.ham("\\r")?,
                '\t' => self.ham("\\t")?,
                k if (k as u32) < 0x20 => self.bicimle(format_args!("\\u{:04x}", k as u32))?,
                k => {
                    let mut baytlar = [0u8; 4];
                    self.ham(k.encode_utf8(&mut baytlar))?;
                }
            }
        }
        self.ham("\"")
    }

    pub(super) fn bitir(self) -> String {
        self.govde
    }
}

impl Write for SinirliJson {
    fn write_str(&mut self, metin: &str) -> fmt::Result {
        self.ham(metin).map_err(|_| fmt::Error)
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn ham_parca_tahsis_edilmeden_once_reddedilir() {
        let mut yazici = SinirliJson::sinirli(4);
        yazici.ham("1234").expect("tam sınıra sığmalı");
        assert_eq!(yazici.ham("5"), Err(CiktiTasmasi));
        assert_eq!(yazici.govde, "1234");
    }

    #[test]
    fn json_kacisi_genislerken_butceyi_gecemez() {
        let mut yazici = SinirliJson::sinirli(7);
        assert_eq!(yazici.metin("\u{0001}"), Err(CiktiTasmasi));
        assert!(yazici.govde.len() <= 7);
    }
}
