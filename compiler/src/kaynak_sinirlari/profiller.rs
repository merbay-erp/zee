//! Ortak politikanın domain görünümleri.
//!
//! Sayısal varsayılanların tek sahibi `VARSAYILAN_KAYNAK_SINIRLARI`dır;
//! tüketici modüller yalnız bu değişmez görünümleri okur.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgSinirlari {
    pub(super) zaman_asimi_ms: i64,
    pub(super) yanit_bayti: usize,
    pub(super) baslik_bayti: usize,
}

impl AgSinirlari {
    pub const fn zaman_asimi_ms(self) -> i64 {
        self.zaman_asimi_ms
    }

    pub const fn yanit_bayti(self) -> usize {
        self.yanit_bayti
    }

    pub const fn baslik_bayti(self) -> usize {
        self.baslik_bayti
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpSinirlari {
    pub(super) istek_okuma_saniyesi: u64,
    pub(super) calistirma_zaman_asimi_ms: i64,
    pub(super) istek_baslik_bayti: usize,
    pub(super) istek_govde_bayti: usize,
    pub(super) istek_alani: usize,
}

impl HttpSinirlari {
    pub const fn istek_okuma_saniyesi(self) -> u64 {
        self.istek_okuma_saniyesi
    }

    pub const fn calistirma_zaman_asimi_ms(self) -> i64 {
        self.calistirma_zaman_asimi_ms
    }

    pub const fn istek_baslik_bayti(self) -> usize {
        self.istek_baslik_bayti
    }

    pub const fn istek_govde_bayti(self) -> usize {
        self.istek_govde_bayti
    }

    pub const fn istek_alani(self) -> usize {
        self.istek_alani
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KaliciDosyaSinirlari {
    pub(super) kilit_bekleme_ms: u64,
    pub(super) kilit_yeniden_dene_ms: u64,
}

impl KaliciDosyaSinirlari {
    pub const fn kilit_bekleme_ms(self) -> u64 {
        self.kilit_bekleme_ms
    }

    pub const fn kilit_yeniden_dene_ms(self) -> u64 {
        self.kilit_yeniden_dene_ms
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoIziSinirlari {
    pub(super) bayt: usize,
    pub(super) olay: usize,
    pub(super) alan: usize,
}

impl IoIziSinirlari {
    pub const fn bayt(self) -> usize {
        self.bayt
    }

    pub const fn olay(self) -> usize {
        self.olay
    }

    pub const fn alan(self) -> usize {
        self.alan
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspSinirlari {
    pub(super) baslik_bayti: usize,
    pub(super) govde_bayti: usize,
    pub(super) json_derinligi: usize,
    pub(super) json_dugumu: usize,
    pub(super) acik_belge: usize,
    pub(super) toplam_belge_bayti: usize,
    pub(super) yanit_bayti: usize,
}

impl LspSinirlari {
    pub const fn baslik_bayti(self) -> usize {
        self.baslik_bayti
    }

    pub const fn govde_bayti(self) -> usize {
        self.govde_bayti
    }

    pub const fn json_derinligi(self) -> usize {
        self.json_derinligi
    }

    pub const fn json_dugumu(self) -> usize {
        self.json_dugumu
    }

    pub const fn acik_belge(self) -> usize {
        self.acik_belge
    }

    pub const fn toplam_belge_bayti(self) -> usize {
        self.toplam_belge_bayti
    }

    pub const fn yanit_bayti(self) -> usize {
        self.yanit_bayti
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaygroundSinirlari {
    pub(super) kaynak_bayti: usize,
    pub(super) girdi_bayti: usize,
    pub(super) girdi_satiri: usize,
}

impl PlaygroundSinirlari {
    pub const fn kaynak_bayti(self) -> usize {
        self.kaynak_bayti
    }

    pub const fn girdi_bayti(self) -> usize {
        self.girdi_bayti
    }

    pub const fn girdi_satiri(self) -> usize {
        self.girdi_satiri
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaketSinirlari {
    pub(super) paket_bayti: usize,
    pub(super) dosya_bayti: usize,
    pub(super) dosya_sayisi: usize,
    pub(super) yol_bayti: usize,
    pub(super) yayin_bayti: usize,
}

impl PaketSinirlari {
    pub const fn paket_bayti(self) -> usize {
        self.paket_bayti
    }

    pub const fn dosya_bayti(self) -> usize {
        self.dosya_bayti
    }

    pub const fn dosya_sayisi(self) -> usize {
        self.dosya_sayisi
    }

    pub const fn yol_bayti(self) -> usize {
        self.yol_bayti
    }

    pub const fn yayin_bayti(self) -> usize {
        self.yayin_bayti
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistrySinirlari {
    pub(super) kok_bayti: usize,
    pub(super) timestamp_bayti: usize,
    pub(super) snapshot_bayti: usize,
    pub(super) targets_bayti: usize,
    pub(super) anahtar_sayisi: usize,
    pub(super) imza_sayisi: usize,
    pub(super) hedef_sayisi: usize,
    pub(super) duyuru_sayisi: usize,
    pub(super) arsiv_bayti: u64,
    pub(super) sbom_bayti: u64,
    pub(super) provenance_bayti: u64,
    pub(super) yayin_bayti: u64,
    pub(super) kok_rotasyonu: usize,
}

impl RegistrySinirlari {
    pub const fn kok_bayti(self) -> usize {
        self.kok_bayti
    }

    pub const fn timestamp_bayti(self) -> usize {
        self.timestamp_bayti
    }

    pub const fn snapshot_bayti(self) -> usize {
        self.snapshot_bayti
    }

    pub const fn targets_bayti(self) -> usize {
        self.targets_bayti
    }

    pub const fn anahtar_sayisi(self) -> usize {
        self.anahtar_sayisi
    }

    pub const fn imza_sayisi(self) -> usize {
        self.imza_sayisi
    }

    pub const fn hedef_sayisi(self) -> usize {
        self.hedef_sayisi
    }

    pub const fn duyuru_sayisi(self) -> usize {
        self.duyuru_sayisi
    }

    pub const fn arsiv_bayti(self) -> u64 {
        self.arsiv_bayti
    }

    pub const fn sbom_bayti(self) -> u64 {
        self.sbom_bayti
    }

    pub const fn provenance_bayti(self) -> u64 {
        self.provenance_bayti
    }

    pub const fn yayin_bayti(self) -> u64 {
        self.yayin_bayti
    }

    pub const fn kok_rotasyonu(self) -> usize {
        self.kok_rotasyonu
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaniSinirlari {
    pub(super) sayi: usize,
}

impl TaniSinirlari {
    pub const fn sayi(self) -> usize {
        self.sayi
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataSinirlari {
    pub(super) ad_listesi_bayti: usize,
    pub(super) deger_bayti: usize,
    pub(super) toplam_bayti: usize,
}

impl MetadataSinirlari {
    pub const fn ad_listesi_bayti(self) -> usize {
        self.ad_listesi_bayti
    }

    pub const fn deger_bayti(self) -> usize {
        self.deger_bayti
    }

    pub const fn toplam_bayti(self) -> usize {
        self.toplam_bayti
    }
}
