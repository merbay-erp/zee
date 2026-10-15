//! Exact kaynak/köken anahtarlı, oturum ömürlü ve ölçülmüş tahsis bütçeli LRU.
use crate::sozcukleyici::{sozcukle, Token, TokenTur};
use crate::tani::Tani;
use std::collections::VecDeque;

/// Henüz bağlanmamış yaprak AST'leri semantic sonucu saklamaz. Bütçe token
/// sayısı ve kaynak baytıdır; AST için toplam RSS/bayt sınırı iddia edilmez.
const YAPRAK_AZAMI_TOKEN: usize = 128_000;
const YAPRAK_AZAMI_KAYNAK: usize = 1024 * 1024;
struct Yaprak {
    koken: String,
    kaynak: String,
    token_sayisi: usize,
    cumleler: Vec<crate::agac::Cumle>,
}
#[derive(Default)]
pub(crate) struct AnalizOturumu {
    sozcukleme: SozcuklemeOturumu,
    yapraklar: VecDeque<Yaprak>,
    yaprak_tokenlari: usize,
    yaprak_kaynak_bayti: usize,
}
impl AnalizOturumu {
    pub(crate) fn sozcukle(
        &mut self,
        kaynak: &str,
        koken: Option<&str>,
    ) -> Result<Vec<Token>, Tani> {
        self.sozcukleme.sozcukle(kaynak, koken)
    }
    pub(crate) fn temizle(&mut self) {
        self.sozcukleme.temizle();
        self.yapraklar = VecDeque::new();
        self.yaprak_tokenlari = 0;
        self.yaprak_kaynak_bayti = 0;
    }
    pub(crate) fn ayristir_yapragi(
        &mut self,
        tokenlar: super::TokenAkisi,
        kaynak: &str,
        koken: Option<&str>,
    ) -> Result<Vec<crate::agac::Cumle>, Tani> {
        let koken = koken.unwrap_or("");
        if let Some(g) = self
            .yapraklar
            .iter()
            .position(|g| g.koken == koken && g.kaynak == kaynak)
            .and_then(|i| self.yapraklar.remove(i))
        {
            let cumleler = g.cumleler.clone();
            self.yapraklar.push_back(g);
            return Ok(cumleler);
        }
        if let Some(g) = self
            .yapraklar
            .iter()
            .position(|g| g.koken == koken)
            .and_then(|i| self.yapraklar.remove(i))
        {
            self.yaprak_tokenlari -= g.token_sayisi;
            self.yaprak_kaynak_bayti -= g.kaynak.len() + g.koken.len();
        }
        let token_sayisi = tokenlar.tokenlar().len();
        let cumleler = tokenlar.ayristir(Vec::new())?.into_cumleler();
        let kaynak_bayti = kaynak.len().saturating_add(koken.len());
        if token_sayisi <= YAPRAK_AZAMI_TOKEN && kaynak_bayti <= YAPRAK_AZAMI_KAYNAK {
            while self.yaprak_tokenlari + token_sayisi > YAPRAK_AZAMI_TOKEN
                || self.yaprak_kaynak_bayti + kaynak_bayti > YAPRAK_AZAMI_KAYNAK
                || self.yapraklar.len() >= AZAMI_GIRDI
            {
                let Some(eski) = self.yapraklar.pop_front() else {
                    break;
                };
                self.yaprak_tokenlari -= eski.token_sayisi;
                self.yaprak_kaynak_bayti -= eski.kaynak.len() + eski.koken.len();
            }
            self.yapraklar.push_back(Yaprak {
                koken: koken.into(),
                kaynak: kaynak.into(),
                token_sayisi,
                cumleler: cumleler.clone(),
            });
            self.yaprak_tokenlari += token_sayisi;
            self.yaprak_kaynak_bayti += kaynak_bayti;
        }
        Ok(cumleler)
    }
}

const AZAMI_BAYT: usize = 8 * 1024 * 1024;
const AZAMI_GIRDI: usize = 64;
struct Girdi {
    koken: String,
    kaynak: String,
    tokenlar: Vec<Token>,
    bayt: usize,
}
#[derive(Default)]
pub(crate) struct SozcuklemeOturumu {
    girdiler: VecDeque<Girdi>,
    bayt: usize,
    #[cfg(test)]
    isabet: usize,
}
impl SozcuklemeOturumu {
    pub(crate) fn sozcukle(
        &mut self,
        kaynak: &str,
        koken: Option<&str>,
    ) -> Result<Vec<Token>, Tani> {
        let koken = koken.unwrap_or("");
        if let Some(g) = self
            .girdiler
            .iter()
            .position(|g| g.koken == koken && g.kaynak == kaynak)
            .and_then(|i| self.girdiler.remove(i))
        {
            let tokenlar = g.tokenlar.clone();
            self.girdiler.push_back(g);
            #[cfg(test)]
            {
                self.isabet += 1;
            }
            return Ok(tokenlar);
        }
        // Aynı kökenin eski metni artık tutulmaz; hatalı yeni kaynak da eski sonucu kullanamaz.
        if let Some(g) = self
            .girdiler
            .iter()
            .position(|g| g.koken == koken)
            .and_then(|i| self.girdiler.remove(i))
        {
            self.bayt -= g.bayt;
        }
        let tokenlar = sozcukle(kaynak)?;
        let mut g = Girdi {
            koken: koken.to_string(),
            kaynak: kaynak.to_string(),
            tokenlar: tokenlar.clone(),
            bayt: 0,
        };
        g.bayt = g.koken.capacity()
            + g.kaynak.capacity()
            + g.tokenlar.capacity() * std::mem::size_of::<Token>()
            + g.tokenlar
                .iter()
                .map(|t| match &t.tur {
                    TokenTur::Metin(s)
                    | TokenTur::Kelime(s)
                    | TokenTur::Ondalik { govde: s, .. } => s.capacity(),
                    _ => 0,
                })
                .sum::<usize>();
        if self.girdiler.capacity() == 0 {
            self.girdiler.reserve_exact(AZAMI_GIRDI);
        }
        let yuva_bayti = self.girdiler.capacity() * std::mem::size_of::<Girdi>();
        if g.bayt.saturating_add(yuva_bayti) <= AZAMI_BAYT {
            while self.bayt + yuva_bayti + g.bayt > AZAMI_BAYT || self.girdiler.len() >= AZAMI_GIRDI
            {
                let Some(eski) = self.girdiler.pop_front() else {
                    break;
                };
                self.bayt -= eski.bayt;
            }
            self.bayt += g.bayt;
            self.girdiler.push_back(g);
        }
        Ok(tokenlar)
    }
    pub(crate) fn temizle(&mut self) {
        self.girdiler = VecDeque::new();
        self.bayt = 0;
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    #[test]
    fn exact_koken_metin_ve_kopya_yalitimi() {
        let mut o = SozcuklemeOturumu::default();
        let mut t = o.sozcukle("x 1 olsun\n", Some("a")).unwrap();
        t.clear();
        assert!(!o.sozcukle("x 1 olsun\n", Some("a")).unwrap().is_empty());
        assert_eq!(o.isabet, 1);
        o.sozcukle("x 1 olsun\n", Some("b")).unwrap();
        assert_eq!(o.isabet, 1);
        o.sozcukle("x 2 olsun\n", Some("a")).unwrap();
        assert_eq!(o.girdiler.len(), 2);
        assert!(o.sozcukle("\"bozuk", Some("a")).is_err());
        assert_eq!(o.girdiler.len(), 1);
        o.temizle();
        assert_eq!((o.girdiler.len(), o.bayt), (0, 0));
    }
    #[test]
    fn lru_adet_ve_tahsis_butcesi_sinirli() {
        let mut o = SozcuklemeOturumu::default();
        for i in 0..300 {
            o.sozcukle(&format!("x {i} olsun\n"), Some(&format!("unit{i}")))
                .unwrap();
            assert!(o.girdiler.len() <= AZAMI_GIRDI);
            assert!(o.bayt + o.girdiler.capacity() * std::mem::size_of::<Girdi>() <= AZAMI_BAYT);
        }
        assert_eq!(o.girdiler.front().unwrap().koken, "unit236");
        assert_eq!(o.bayt, o.girdiler.iter().map(|g| g.bayt).sum::<usize>());
    }

    #[test]
    fn yaprak_ast_kopyasi_yalitilir_ve_butceler_eski_metni_tutmaz() {
        let mut o = AnalizOturumu::default();
        for i in 0..100 {
            let kaynak = format!("x {i} olsun\n");
            let tokenlar = super::super::KaynakMetni::yeni(&kaynak).sozcukle().unwrap();
            let mut ast = o
                .ayristir_yapragi(tokenlar, &kaynak, Some(&format!("birim{i}")))
                .unwrap();
            ast.clear();
            assert!(o.yapraklar.len() <= 64);
            assert!(o.yaprak_tokenlari <= YAPRAK_AZAMI_TOKEN);
            assert!(o.yaprak_kaynak_bayti <= YAPRAK_AZAMI_KAYNAK);
        }
        let kaynak = "x 99 olsun\n";
        let t = super::super::KaynakMetni::yeni(kaynak).sozcukle().unwrap();
        assert!(!o
            .ayristir_yapragi(t, kaynak, Some("birim99"))
            .unwrap()
            .is_empty());
        let yeni = "x 100 olsun\n";
        let t = super::super::KaynakMetni::yeni(yeni).sozcukle().unwrap();
        o.ayristir_yapragi(t, yeni, Some("birim99")).unwrap();
        assert_eq!(
            o.yapraklar.iter().filter(|g| g.koken == "birim99").count(),
            1
        );
        assert_eq!(
            o.yaprak_tokenlari,
            o.yapraklar.iter().map(|g| g.token_sayisi).sum::<usize>()
        );
        o.temizle();
        assert!(o.yapraklar.is_empty() && o.sozcukleme.girdiler.is_empty());
        assert_eq!((o.yaprak_tokenlari, o.yaprak_kaynak_bayti), (0, 0));
    }
}
