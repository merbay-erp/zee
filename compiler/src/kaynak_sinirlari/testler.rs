use super::*;

#[test]
fn varsayilan_profil_kritik_limitleri_sifira_birakmaz() {
    let sinirlar = VARSAYILAN_KAYNAK_SINIRLARI;
    assert!(sinirlar.kaynak_bayti() > 0);
    assert!(sinirlar.toplam_kaynak_bayti() >= sinirlar.kaynak_bayti());
    assert!(sinirlar.kaynak_dosyasi() > 0);
    assert!(sinirlar.token_sayisi() > 0);
    assert!(sinirlar.cagri_derinligi() > 0);
    assert!(sinirlar.calistirma_yigin_bayti() > 0);
    assert!(sinirlar.calistirma_adimi() > 0);
    assert!(sinirlar.koleksiyon_ogesi() > 0);
    assert!(sinirlar.eszamanli_gorev() > 0);
    assert!(sinirlar.calisma_heap_bayti() >= sinirlar.metin_bayti());
    assert!(sinirlar.ag_baglantisi() > 0);
    assert!(sinirlar.cikti_bayti() > 0 && sinirlar.cikti_olayi() > 0);
    assert!(sinirlar.lsp_acik_belge() > 0);
    assert!(sinirlar.lsp_toplam_belge_bayti() >= sinirlar.kaynak_bayti());
    assert!(sinirlar.lsp_yanit_bayti() > 0);
    assert!(sinirlar.ag().yanit_bayti() > sinirlar.ag().baslik_bayti());
    assert!(sinirlar.http().istek_govde_bayti() > sinirlar.http().istek_baslik_bayti());
    assert!(sinirlar.http().calistirma_zaman_asimi_ms() > 0);
    assert!(sinirlar.web().anonim_oturum_sayisi() <= sinirlar.web().oturum_sayisi());
    assert!(sinirlar.web().anonim_oturum_omru_saniye() <= sinirlar.web().oturum_omru_saniye());
    assert!(sinirlar.web().oran_anahtari_sayisi() > sinirlar.web().oturum_sayisi());
    assert!(sinirlar.web().ucnokta_orani() > sinirlar.web().giris_orani());
    assert!(sinirlar.web().csrf_orani() >= sinirlar.web().giris_orani());
    assert!(sinirlar.web().ucnokta_penceresi_saniye() > 0);
    assert!(sinirlar.web().csrf_penceresi_saniye() > 0);
    assert!(sinirlar.web().giris_penceresi_saniye() > 0);
    assert!(sinirlar.io_izi().alan() <= sinirlar.io_izi().olay());
    assert!(sinirlar.lsp().yanit_bayti() <= sinirlar.lsp().govde_bayti());
    assert!(sinirlar.playground().kaynak_bayti() <= sinirlar.kaynak_bayti());
    assert!(sinirlar.playground().girdi_bayti() < sinirlar.metin_bayti());
    assert!(sinirlar.playground().girdi_satiri() <= sinirlar.koleksiyon_ogesi());
    assert!(sinirlar.paket().dosya_bayti() <= sinirlar.paket().paket_bayti());
    assert!(sinirlar.paket().yayin_bayti() <= sinirlar.paket().paket_bayti());
    assert!(sinirlar.registry().yayin_bayti() <= sinirlar.registry().targets_bayti() as u64);
    assert!(sinirlar.registry().kok_rotasyonu() > 0);
    assert!(sinirlar.tani().sayi() > 0);
    assert!(sinirlar.metadata().deger_bayti() <= sinirlar.metadata().toplam_bayti());
    assert!(sinirlar.veritabani().sorgu_bayti() > 0);
    assert!(sinirlar.veritabani().parametre() > 0);
    assert!(sinirlar.veritabani().parametre_bayti() <= sinirlar.metin_bayti());
    assert!(sinirlar.veritabani().satir() > 0 && sinirlar.veritabani().sutun() > 0);
    assert!(sinirlar.veritabani().sonuc_bayti() <= sinirlar.calisma_heap_bayti());
    assert!(sinirlar.veritabani().goc_dosyasi_bayti() <= sinirlar.veritabani().goc_toplami_bayti());
    assert!(
        sinirlar.kalici_dosya().kilit_yeniden_dene_ms()
            < sinirlar.kalici_dosya().kilit_bekleme_ms()
    );
}

#[test]
fn buyuk_bellek_kaynagi_s045_ile_reddedilir() {
    let kaynak = " ".repeat(VARSAYILAN_KAYNAK_SINIRLARI.kaynak_bayti() + 1);
    let hata = kaynak_boyutunu_denetle(&kaynak).expect_err("sınır aşımı");
    assert_eq!(hata.kod, "S045");
}
