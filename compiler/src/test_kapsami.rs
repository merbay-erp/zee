//! Internal opt-in Zee source decision coverage (K284); not api::v1.
//!
//! Inventory belongs to the final borrowed AST. Addresses are lookup keys only,
//! never report identities; an unregistered clone is an invariant error.
use crate::agac::{Cumle, Ifade, Program};
use crate::tani::Tani;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::rc::Rc;

mod envanter;
pub const AZAMI_DAL: usize = 16_384;
pub const AZAMI_RAPOR: usize = 2 * 1024 * 1024;
pub const AZAMI_KAYNAK: usize = 256;

#[derive(Clone, Serialize)]
struct Dal {
    id: usize,
    kaynak: String,
    sahip_turu: String,
    sahip: String,
    satir: usize,
    karar: String,
    sira: usize,
    sonuc: bool,
    vurus: u64,
}

type Anahtar = (bool, usize, usize);
struct Toplayici {
    dallar: Vec<Dal>,
    dizin: HashMap<Anahtar, usize>,
    kaynaklar: BTreeMap<String, String>,
    hata: bool,
}
thread_local! {
    static ETKIN: RefCell<Option<Toplayici>> = const { RefCell::new(None) };
}

/// One scope, one final immutable AST, one runtime thread; never Send/Sync.
pub struct Nobetci<'a> {
    _program: &'a Program,
    _tek_is_parcasi: PhantomData<Rc<()>>,
    etkin: bool,
}

pub fn baslat<'a>(
    program: &'a Program,
    ana_koken: &str,
    kaynaklar: BTreeMap<String, String>,
) -> Result<Nobetci<'a>, String> {
    if ETKIN.with(|s| s.borrow().is_some()) {
        return Err("kapsam iç içe başlatılamaz".into());
    }
    if kaynaklar.len() > AZAMI_KAYNAK
        || !kaynaklar.contains_key(ana_koken)
        || kaynaklar.iter().any(|(k, v)| {
            k.len() > 4096 || v.len() != 64 || !v.bytes().all(|b| b.is_ascii_hexdigit())
        })
    {
        return Err("kapsam kaynak envanteri geçersiz".into());
    }
    let toplayici = envanter::kur(program, ana_koken, kaynaklar)?;
    let ust_sinir = serilestirilmis_boyut(&toplayici.dallar)?
        .saturating_add(serilestirilmis_boyut(&toplayici.kaynaklar)?)
        .saturating_add(toplayici.dallar.len().saturating_mul(32))
        .saturating_add(4096);
    // Actual escaped metadata, worst u64 counters and both missing-ID summaries.
    if ust_sinir > AZAMI_RAPOR {
        return Err("kapsam rapor bütçesi aşıldı".into());
    }
    ETKIN.with(|s| *s.borrow_mut() = Some(toplayici));
    Ok(Nobetci {
        _program: program,
        _tek_is_parcasi: PhantomData,
        etkin: true,
    })
}

// Count JSON without allocating an oversized temporary report or doing IO.
fn serilestirilmis_boyut(value: &impl Serialize) -> Result<usize, String> {
    struct Sayac(usize);
    impl std::io::Write for Sayac {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(bytes.len());
            if self.0 > AZAMI_RAPOR {
                return Err(std::io::Error::other("coverage report budget"));
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Sayac(0);
    serde_json::to_writer(&mut count, value)
        .map_err(|_| "kapsam rapor bütçesi aşıldı".to_string())?;
    Ok(count.0)
}

#[derive(Serialize)]
struct Ozet {
    payda: usize,
    vurulan: usize,
    eksik: Vec<usize>,
}
fn ozet(dallar: &[Dal], test: bool) -> Ozet {
    let secili = dallar.iter().filter(|d| (d.sahip_turu == "test") == test);
    let mut sonuc = Ozet {
        payda: 0,
        vurulan: 0,
        eksik: Vec::new(),
    };
    for dal in secili {
        sonuc.payda += 1;
        if dal.vurus > 0 {
            sonuc.vurulan += 1;
        } else {
            sonuc.eksik.push(dal.id);
        }
    }
    sonuc
}

impl Nobetci<'_> {
    pub fn bitir(mut self, basarili: bool) -> Result<serde_json::Value, String> {
        let sonuc = ETKIN
            .with(|s| s.borrow_mut().take())
            .ok_or("kapsam oturumu kayıp")?;
        self.etkin = false;
        if sonuc.hata {
            return Err("kapsam envanter dışı düğüm gözledi".into());
        }
        Ok(serde_json::json!({
            "schema":"zee-source-coverage-1", "complete":true,
            "execution_success":basarili, "profile":"source-decision-outcomes-1",
            "product":ozet(&sonuc.dallar,false), "tests":ozet(&sonuc.dallar,true),
            "sources":sonuc.kaynaklar,"edges":sonuc.dallar,
            "zero_denominator":"not-applicable", "native_host_branches_included":false
        }))
    }
}
impl Drop for Nobetci<'_> {
    fn drop(&mut self) {
        if self.etkin {
            ETKIN.with(|s| *s.borrow_mut() = None);
        }
    }
}

fn vur(anahtar: Anahtar, sonuc: bool, satir: usize) -> Result<(), Tani> {
    ETKIN.with(|s| {
        let mut yuva = s.borrow_mut();
        let Some(t) = yuva.as_mut() else {
            return Ok(());
        };
        if let Some(&ilk) = t.dizin.get(&anahtar) {
            let d = &mut t.dallar[ilk + usize::from(sonuc)];
            d.vurus = d.vurus.saturating_add(1);
            Ok(())
        } else {
            t.hata = true;
            Err(Tani::yeni(
                "C000",
                "Kapsam izleme düğümü kaynak envanterinde yok.".into(),
                satir,
                1,
                1,
            ))
        }
    })
}
pub(crate) fn cumle(c: &Cumle, sira: usize, sonuc: bool, satir: usize) -> Result<(), Tani> {
    vur((false, c as *const Cumle as usize, sira), sonuc, satir)
}
pub(crate) fn ifade(i: &Ifade, sira: usize, sonuc: bool, satir: usize) -> Result<(), Tani> {
    vur((true, i as *const Ifade as usize, sira), sonuc, satir)
}

/// Coverage derlemesi ortak kaynak kökeni kapsamını açar.
pub use crate::agac::KaynakKokenNobetcisi as DerlemeNobetci;
pub fn derleme_baslat(etkin: bool) -> DerlemeNobetci {
    crate::agac::kaynak_kokenlerini_izle(etkin)
}
