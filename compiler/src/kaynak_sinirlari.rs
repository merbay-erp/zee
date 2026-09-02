//! Derleyici, runtime ve araçlar için tek değişmez kaynak politikası.
//!
//! Değerler kullanıcı girdisinden değiştirilemez. Daha geniş bir profil ancak
//! ayrı, açık ve sürümlü bir ürün kararıyla eklenebilir; sessiz sınırsız geri
//! dönüş yoktur.

mod baglanti;

pub use baglanti::{baglanti_izni_al, BaglantiIzni};

use crate::tani::Tani;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KaynakSinirlari {
    kaynak_bayti: usize,
    toplam_kaynak_bayti: usize,
    kaynak_dosyasi: usize,
    token_sayisi: usize,
    cagri_derinligi: usize,
    calistirma_adimi: usize,
    koleksiyon_ogesi: usize,
    eszamanli_gorev: usize,
    calisma_heap_bayti: usize,
    metin_bayti: usize,
    ag_baglantisi: usize,
    cikti_bayti: usize,
    cikti_olayi: usize,
    dosya_okuma_bayti: usize,
    lsp_acik_belge: usize,
    lsp_toplam_belge_bayti: usize,
    lsp_yanit_bayti: usize,
}

impl KaynakSinirlari {
    pub const fn kaynak_bayti(self) -> usize {
        self.kaynak_bayti
    }

    pub const fn token_sayisi(self) -> usize {
        self.token_sayisi
    }

    pub const fn toplam_kaynak_bayti(self) -> usize {
        self.toplam_kaynak_bayti
    }

    pub const fn kaynak_dosyasi(self) -> usize {
        self.kaynak_dosyasi
    }

    pub const fn cagri_derinligi(self) -> usize {
        self.cagri_derinligi
    }

    pub const fn calistirma_adimi(self) -> usize {
        self.calistirma_adimi
    }

    pub const fn koleksiyon_ogesi(self) -> usize {
        self.koleksiyon_ogesi
    }

    pub const fn eszamanli_gorev(self) -> usize {
        self.eszamanli_gorev
    }

    pub const fn calisma_heap_bayti(self) -> usize {
        self.calisma_heap_bayti
    }

    pub const fn metin_bayti(self) -> usize {
        self.metin_bayti
    }

    pub const fn ag_baglantisi(self) -> usize {
        self.ag_baglantisi
    }

    pub const fn cikti_bayti(self) -> usize {
        self.cikti_bayti
    }

    pub const fn cikti_olayi(self) -> usize {
        self.cikti_olayi
    }

    pub const fn dosya_okuma_bayti(self) -> usize {
        self.dosya_okuma_bayti
    }

    pub const fn lsp_acik_belge(self) -> usize {
        self.lsp_acik_belge
    }

    pub const fn lsp_toplam_belge_bayti(self) -> usize {
        self.lsp_toplam_belge_bayti
    }

    pub const fn lsp_yanit_bayti(self) -> usize {
        self.lsp_yanit_bayti
    }
}

/// Resmî güvenli profil. Limit değişiklikleri kullanıcı programlarının
/// gözlemlenebilir davranışıdır ve sürüm notu/test güncellemesi ister.
pub const VARSAYILAN_KAYNAK_SINIRLARI: KaynakSinirlari = KaynakSinirlari {
    kaynak_bayti: 8 * 1024 * 1024,
    toplam_kaynak_bayti: 128 * 1024 * 1024,
    kaynak_dosyasi: 4_096,
    token_sayisi: 1_000_000,
    cagri_derinligi: 500,
    calistirma_adimi: 10_000_000,
    koleksiyon_ogesi: 1_000_000,
    eszamanli_gorev: 1_024,
    calisma_heap_bayti: 64 * 1024 * 1024,
    metin_bayti: 16 * 1024 * 1024,
    ag_baglantisi: 64,
    cikti_bayti: 16 * 1024 * 1024,
    cikti_olayi: 100_000,
    dosya_okuma_bayti: 16 * 1024 * 1024,
    lsp_acik_belge: 256,
    lsp_toplam_belge_bayti: 128 * 1024 * 1024,
    lsp_yanit_bayti: 8 * 1024 * 1024,
};

pub(crate) fn kaynak_boyutunu_denetle(kaynak: &str) -> Result<(), Tani> {
    let azami = VARSAYILAN_KAYNAK_SINIRLARI.kaynak_bayti();
    if kaynak.len() <= azami {
        return Ok(());
    }
    Err(Tani::yeni(
        "S045",
        format!(
            "Kaynak {} bayt; güvenli profil {} MiB sınırını aşıyor.",
            kaynak.len(),
            azami / 1024 / 1024
        ),
        1,
        1,
        1,
    )
    .onerili("Kaynağı sorumluluğu açık daha küçük birimlere böl.".into()))
}

pub(crate) fn token_sayisini_denetle(sayi: usize, satir: usize) -> Result<(), Tani> {
    let azami = VARSAYILAN_KAYNAK_SINIRLARI.token_sayisi();
    if sayi <= azami {
        return Ok(());
    }
    Err(Tani::yeni(
        "S045",
        format!(
            "Kaynak {} token; güvenli profil {} token sınırını aşıyor.",
            sayi, azami
        ),
        satir,
        1,
        1,
    )
    .onerili("Üretilmiş ya da çok büyük kaynağı daha küçük birimlere böl.".into()))
}

pub fn kaynak_dosyasi_oku(yol: &Path) -> io::Result<String> {
    sinirli_metin_oku(yol, VARSAYILAN_KAYNAK_SINIRLARI.kaynak_bayti())
}

pub fn veri_dosyasi_oku(yol: &Path) -> io::Result<String> {
    sinirli_metin_oku(yol, VARSAYILAN_KAYNAK_SINIRLARI.dosya_okuma_bayti())
}

pub fn veri_dosyasi_baytlarini_oku(yol: &Path) -> io::Result<Vec<u8>> {
    sinirli_bayt_oku(yol, VARSAYILAN_KAYNAK_SINIRLARI.dosya_okuma_bayti())
}

fn sinirli_metin_oku(yol: &Path, azami: usize) -> io::Result<String> {
    let baytlar = sinirli_bayt_oku(yol, azami)?;
    String::from_utf8(baytlar).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("`{}` geçerli UTF-8 metin değil", yol.display()),
        )
    })
}

fn sinirli_bayt_oku(yol: &Path, azami: usize) -> io::Result<Vec<u8>> {
    let dosya = std::fs::File::open(yol)?;
    let bildirilen = dosya.metadata()?.len();
    if bildirilen > azami as u64 {
        return Err(sinir_hatasi(yol, azami));
    }
    let mut baytlar = Vec::with_capacity(usize::try_from(bildirilen).unwrap_or(azami).min(azami));
    dosya
        .take(azami.saturating_add(1) as u64)
        .read_to_end(&mut baytlar)?;
    if baytlar.len() > azami {
        return Err(sinir_hatasi(yol, azami));
    }
    Ok(baytlar)
}

fn sinir_hatasi(yol: &Path, azami: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "`{}` {} MiB okuma sınırını aşıyor",
            yol.display(),
            azami / 1024 / 1024
        ),
    )
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn varsayilan_profil_kritik_limitleri_sifira_birakmaz() {
        let sinirlar = VARSAYILAN_KAYNAK_SINIRLARI;
        assert!(sinirlar.kaynak_bayti() > 0);
        assert!(sinirlar.toplam_kaynak_bayti() >= sinirlar.kaynak_bayti());
        assert!(sinirlar.kaynak_dosyasi() > 0);
        assert!(sinirlar.token_sayisi() > 0);
        assert!(sinirlar.cagri_derinligi() > 0);
        assert!(sinirlar.calistirma_adimi() > 0);
        assert!(sinirlar.koleksiyon_ogesi() > 0);
        assert!(sinirlar.eszamanli_gorev() > 0);
        assert!(sinirlar.calisma_heap_bayti() >= sinirlar.metin_bayti());
        assert!(sinirlar.ag_baglantisi() > 0);
        assert!(sinirlar.cikti_bayti() > 0 && sinirlar.cikti_olayi() > 0);
        assert!(sinirlar.lsp_acik_belge() > 0);
        assert!(sinirlar.lsp_toplam_belge_bayti() >= sinirlar.kaynak_bayti());
        assert!(sinirlar.lsp_yanit_bayti() > 0);
    }

    #[test]
    fn buyuk_bellek_kaynagi_s045_ile_reddedilir() {
        let kaynak = " ".repeat(VARSAYILAN_KAYNAK_SINIRLARI.kaynak_bayti() + 1);
        let hata = kaynak_boyutunu_denetle(&kaynak).expect_err("sınır aşımı");
        assert_eq!(hata.kod, "S045");
    }
}
