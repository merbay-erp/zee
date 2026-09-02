//! Derleyici, runtime ve araçlar için tek değişmez kaynak politikası.
//!
//! Değerler kullanıcı girdisinden değiştirilemez. Daha geniş bir profil ancak
//! ayrı, açık ve sürümlü bir ürün kararıyla eklenebilir; sessiz sınırsız geri
//! dönüş yoktur.

mod baglanti;
mod okuma;
mod profiller;

pub use baglanti::{baglanti_izni_al, BaglantiIzni};
pub(crate) use okuma::{kaynak_boyutunu_denetle, token_sayisini_denetle};
pub use okuma::{kaynak_dosyasi_oku, veri_dosyasi_baytlarini_oku, veri_dosyasi_oku};
pub use profiller::{
    AgSinirlari, HttpSinirlari, IoIziSinirlari, KaliciDosyaSinirlari, LspSinirlari,
    MetadataSinirlari, PaketSinirlari, RegistrySinirlari, TaniSinirlari, WebSinirlari,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KaynakSinirlari {
    kaynak_bayti: usize,
    toplam_kaynak_bayti: usize,
    kaynak_dosyasi: usize,
    token_sayisi: usize,
    cagri_derinligi: usize,
    calistirma_yigin_bayti: usize,
    calistirma_adimi: usize,
    koleksiyon_ogesi: usize,
    eszamanli_gorev: usize,
    calisma_heap_bayti: usize,
    metin_bayti: usize,
    ag_baglantisi: usize,
    cikti_bayti: usize,
    cikti_olayi: usize,
    dosya_okuma_bayti: usize,
    ag: AgSinirlari,
    http: HttpSinirlari,
    web: WebSinirlari,
    io_izi: IoIziSinirlari,
    lsp: LspSinirlari,
    paket: PaketSinirlari,
    registry: RegistrySinirlari,
    tani: TaniSinirlari,
    metadata: MetadataSinirlari,
    kalici_dosya: KaliciDosyaSinirlari,
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

    pub const fn calistirma_yigin_bayti(self) -> usize {
        self.calistirma_yigin_bayti
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
        self.lsp.acik_belge()
    }

    pub const fn lsp_toplam_belge_bayti(self) -> usize {
        self.lsp.toplam_belge_bayti()
    }

    pub const fn lsp_yanit_bayti(self) -> usize {
        self.lsp.yanit_bayti()
    }

    pub const fn ag(self) -> AgSinirlari {
        self.ag
    }

    pub const fn http(self) -> HttpSinirlari {
        self.http
    }

    pub const fn web(self) -> WebSinirlari {
        self.web
    }

    pub const fn io_izi(self) -> IoIziSinirlari {
        self.io_izi
    }

    pub const fn lsp(self) -> LspSinirlari {
        self.lsp
    }

    pub const fn paket(self) -> PaketSinirlari {
        self.paket
    }

    pub const fn registry(self) -> RegistrySinirlari {
        self.registry
    }

    pub const fn tani(self) -> TaniSinirlari {
        self.tani
    }

    pub const fn metadata(self) -> MetadataSinirlari {
        self.metadata
    }

    pub const fn kalici_dosya(self) -> KaliciDosyaSinirlari {
        self.kalici_dosya
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
    calistirma_yigin_bayti: 32 * 1024 * 1024,
    calistirma_adimi: 10_000_000,
    koleksiyon_ogesi: 1_000_000,
    eszamanli_gorev: 1_024,
    calisma_heap_bayti: 64 * 1024 * 1024,
    metin_bayti: 16 * 1024 * 1024,
    ag_baglantisi: 64,
    cikti_bayti: 16 * 1024 * 1024,
    cikti_olayi: 100_000,
    dosya_okuma_bayti: 16 * 1024 * 1024,
    ag: AgSinirlari {
        zaman_asimi_ms: 30_000,
        yanit_bayti: 8 * 1024 * 1024,
        baslik_bayti: 64 * 1024,
    },
    http: HttpSinirlari {
        istek_okuma_saniyesi: 10,
        calistirma_zaman_asimi_ms: 30_000,
        istek_baslik_bayti: 16 * 1024,
        istek_govde_bayti: 64 * 1024,
        istek_alani: 100,
    },
    web: WebSinirlari {
        oturum_omru_saniye: 30 * 60,
        anonim_oturum_omru_saniye: 10 * 60,
        oturum_sayisi: 4_096,
        anonim_oturum_sayisi: 1_024,
    },
    io_izi: IoIziSinirlari {
        bayt: 64 * 1024 * 1024,
        olay: 100_000,
        alan: 4_096,
    },
    lsp: LspSinirlari {
        baslik_bayti: 8 * 1024,
        govde_bayti: 8 * 1024 * 1024,
        json_derinligi: 128,
        json_dugumu: 100_000,
        acik_belge: 256,
        toplam_belge_bayti: 128 * 1024 * 1024,
        yanit_bayti: 8 * 1024 * 1024,
    },
    paket: PaketSinirlari {
        paket_bayti: 64 * 1024 * 1024,
        dosya_bayti: 16 * 1024 * 1024,
        dosya_sayisi: 10_000,
        yol_bayti: 1_024,
        yayin_bayti: 1024 * 1024,
    },
    registry: RegistrySinirlari {
        kok_bayti: 1024 * 1024,
        timestamp_bayti: 64 * 1024,
        snapshot_bayti: 1024 * 1024,
        targets_bayti: 8 * 1024 * 1024,
        anahtar_sayisi: 256,
        imza_sayisi: 256,
        hedef_sayisi: 100_000,
        duyuru_sayisi: 100_000,
        arsiv_bayti: 64 * 1024 * 1024,
        sbom_bayti: 8 * 1024 * 1024,
        provenance_bayti: 8 * 1024 * 1024,
        yayin_bayti: 1024 * 1024,
    },
    tani: TaniSinirlari { sayi: 20 },
    metadata: MetadataSinirlari {
        ad_listesi_bayti: 64 * 1024,
        deger_bayti: 64 * 1024,
        toplam_bayti: 1024 * 1024,
    },
    kalici_dosya: KaliciDosyaSinirlari {
        kilit_bekleme_ms: 5_000,
        kilit_yeniden_dene_ms: 5,
    },
};

#[cfg(test)]
mod testler;
