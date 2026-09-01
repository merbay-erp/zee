//! Ad çözümleme + tür denetimi.
//!
//! Ad çözümleme, K-011'deki kuralı uygular: tanımlayıcılara ekler bitişik
//! yazılır ("sayıyı", "toplamı", "sayacı"). Çözüm TAHMİNLE DEĞİL, aday kök
//! üretip kapsamdaki tanımlı adlarla eşleyerek yapılır — birden çok aday
//! eşleşirse bu bir hatadır (determinizm, manifesto 3).
//!
//! Ünsüz yumuşamasının geri çevrimi desteklenir: "sayacı" → "sayac" → "sayaç".
//!
//! B-006 katman sırası: etki/yetkinlik → tür yazımları → açık sözleşmeler →
//! cümle/ifade denetimi. Cümle katmanı sembol ve akış hizmetlerini, çağrı
//! katmanı dönüş/control-flow hizmetini kullanır; katman sahipliği aşağıdaki
//! fiziksel modüllerde tekildir.

mod akis;
mod ast_donusum;
mod baglam;
mod cagri;
mod cumle;
mod donus;
mod etki;
mod ifade;
mod sembol;
mod sozlesme;
mod turler;

use self::akis::{
    bekleyen_gorev_olmadigini_denetle, daraltma_cikar, daraltma_cikar_geri, daraltma_ekle,
    gezilen_hedefi_denetle, gezilen_koleksiyonu_degistirme_tanisi, gorev_kapsami_tanisi,
    nesne_adi,
};
use self::baglam::{Baglam, Imza, ImzaKaydi};
use self::cagri::cagri_denetle;
use self::cumle::blok_denetle;
use self::donus::{blok_kesin_sonlanir, donusleri_birlestir, donusleri_sarmala};
use self::ifade::{hir_ifadesi_kaydet, ifade_denetle};
use self::sembol::{
    alan_cozumle, kapsam_baslat, kapsam_bitir, sembol_cozumle, SembolTablosu,
};
pub use self::sembol::ad_cozumle;
use self::sozlesme::{acik_islemleri_denetle, acik_parametre_turleri, bildirilmis_donus_turu};
use self::turler::{
    alan_turu, bos_koleksiyon_uzlasi, intrinsic_turunu_cevir, parametre_turu, veri_turu_yap,
    yapi_turu_tanilari,
};
pub use self::turler::{SozlukDegerTuru, Tur, VeriTuru};

use crate::agac::{Cumle, Ifade, Islec, Islem, Ozellik, Program, Yapi};
use crate::intrinsic;
use crate::kimlik::{IslemId, SymbolId, YapiId};

use crate::tani::Tani;
use std::collections::HashMap;

fn ic_tutarlilik_hatasi(mesaj: impl Into<String>, satir: usize) -> Tani {
    Tani::yeni(
        "T016",
        format!("{} — derleyici iç hatası olabilir, bildir.", mesaj.into()),
        satir.max(1),
        1,
        1,
    )
}

fn hir_kaynak_hatasi(satir: usize) -> Tani {
    ic_tutarlilik_hatasi("Semantic HIR düğümünün kaynak aralığı kurulamadı", satir)
}

/// Çoklu denetim (RFC-0010 §3.1): üst düzey cümle başına hata toplanır;
/// bir cümlenin hatası sonrakilerin denetimini durdurmaz. LSP/denetle --json
/// bu görünümü kullanır; derleme (çalıştır) ilk tanıda durur.
pub fn denetle_coklu(program: &mut Program) -> Vec<Tani> {
    let mut tanilar = Vec::new();
    if let Err(tani) = etki::denetle(program) {
        tanilar.push(tani);
    }
    let mut ortam = SembolTablosu::yeni(0);
    tanilar.extend(yapi_turu_tanilari(&program.yapilar));
    let mut baglam = Baglam::yeni(
        std::mem::take(&mut program.islemler),
        program.yapilar.clone(),
    );
    if let Err(tani) = acik_islemleri_denetle(&mut baglam) {
        tanilar.push(tani);
        program.islemler = baglam.islemler;
        return tanilar;
    }
    let mut bas = 0;
    while bas < program.cumleler.len() {
        // Çoklu tanı geçişi normalde cümle cümle toparlanır. Görev bildirimi
        // ile join ise tek sözcüksel kanıttır; ikisini aynı dilimle denetle.
        let mut son = bas + 1;
        if matches!(program.cumleler[bas], Cumle::Eszamanli { .. }) {
            while son < program.cumleler.len() {
                let join = matches!(program.cumleler[son], Cumle::HepsiniBekle { .. });
                son += 1;
                if join {
                    break;
                }
            }
        }
        let onceki_bekleyenler = baglam.bekleyen_gorevler.clone();
        if let Err(tani) =
            blok_denetle(&mut program.cumleler[bas..son], &mut ortam, &mut baglam)
        {
            tanilar.push(tani);
            baglam.bekleyen_gorevler = onceki_bekleyenler;
            if tanilar.len() >= 20 {
                break;
            }
        }
        bas = son;
    }
    for (test_indeksi, test) in program.testler.iter_mut().enumerate() {
        let mut test_ortami = SembolTablosu::yeni(baglam.test_kapsami(test_indeksi));
        if let Err(tani) = blok_denetle(&mut test.govde, &mut test_ortami, &mut baglam) {
            tanilar.push(tani);
            if tanilar.len() >= 20 {
                break;
            }
        }
    }
    program.islemler = baglam.islemler;
    tanilar
}

/// Programı yerinde çözümler ve tür denetiminden geçirir.
///
/// Çıkarımlı işlemler ilk çağrı argümanlarıyla; açık imzalı işlemler ise
/// tanım sözleşmesiyle çağrı beklemeden denetlenir (K-083).
pub fn denetle(program: &mut Program) -> Result<(), Tani> {
    denetle_ve_hir_bilgisi(program).map(|_| ())
}

/// Programı denetler ve başarılı geçişin typed HIR lowering bilgisini verir.
pub(crate) fn denetle_ve_hir_bilgisi(
    program: &mut Program,
) -> Result<crate::hir::HirOlusturmaBilgisi, Tani> {
    etki::denetle(program)?;
    let mut ortam = SembolTablosu::yeni(0);
    if let Some(tani) = yapi_turu_tanilari(&program.yapilar).into_iter().next() {
        return Err(tani);
    }
    let mut baglam = Baglam::yeni(
        std::mem::take(&mut program.islemler),
        program.yapilar.clone(),
    );
    let mut sonuc = acik_islemleri_denetle(&mut baglam);
    if sonuc.is_ok() {
        sonuc = blok_denetle(&mut program.cumleler, &mut ortam, &mut baglam);
    }

    // Testler ana programdan bağımsız, taze ortamda denetlenir.
    if sonuc.is_ok() {
        for (test_indeksi, test) in program.testler.iter_mut().enumerate() {
            let mut test_ortami = SembolTablosu::yeni(baglam.test_kapsami(test_indeksi));
            sonuc = blok_denetle(&mut test.govde, &mut test_ortami, &mut baglam);
            if sonuc.is_err() {
                break;
            }
        }
    }

    let hir_bilgisi = sonuc.as_ref().ok().map(|_| baglam.hir_bilgisi());
    program.islemler = baglam.islemler;
    sonuc?;
    hir_bilgisi.ok_or_else(|| ic_tutarlilik_hatasi("Başarılı checker HIR bilgisi üretmedi", 1))
}
