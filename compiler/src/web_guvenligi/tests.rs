use super::*;

fn uretici() -> impl FnMut() -> Result<String, String> {
    let mut sira = 0;
    move || {
        sira += 1;
        Ok(format!("belirtec-{}", sira))
    }
}

fn istegi_baslat(depo: &mut WebGuvenligi, belirtec: Option<&str>, an_ms: i64) {
    if !depo.bekleyen.silinecekler.is_empty() || !depo.bekleyen.eklenecekler.is_empty() {
        depo.istegi_tamamla(an_ms).expect("önceki istek commit'i");
    }
    depo.istegi_baslat_kimlikle(belirtec, "yerel-test", an_ms)
        .expect("istek başlamalı");
}

#[test]
fn csrf_anonim_oturuma_baglanir_ve_sahte_deger_reddedilir() {
    let mut depo = WebGuvenligi::yeni();
    istegi_baslat(&mut depo, None, 0);
    let (csrf, yeni) = depo.csrf_belirteci(0, uretici()).expect("csrf");
    let yeni = yeni.expect("anonim oturum");
    istegi_baslat(&mut depo, Some(&yeni.belirtec), 1);
    assert!(depo
        .denetle(&RotaErisimi::HerkeseAcik, Some(&csrf), true, 1)
        .is_ok());
    assert_eq!(
        depo.denetle(&RotaErisimi::HerkeseAcik, Some("sahte"), true, 1)
            .unwrap_err()
            .durum,
        403
    );
}

#[test]
fn giris_kimligi_dondurur_rolu_ayirir_ve_cikis_iptal_eder() {
    let mut depo = WebGuvenligi::yeni();
    let mut uret = uretici();
    istegi_baslat(&mut depo, None, 0);
    let (_, anonim) = depo.csrf_belirteci(0, &mut uret).expect("csrf");
    let anonim = anonim.unwrap();
    let yeni = depo
        .oturum_ac("Mustafa".into(), "yönetici".into(), 1, &mut uret)
        .expect("giriş");
    assert_ne!(anonim.belirtec, yeni.belirtec, "session fixation");
    istegi_baslat(&mut depo, Some(&yeni.belirtec), 2);
    let bilgi = depo.oturum_bilgisi(2).expect("oturum bilgisi");
    assert_eq!(bilgi.kullanici, "Mustafa");
    assert_eq!(bilgi.rol, "yönetici");
    assert_eq!(bilgi.olusturma_ms, 1);
    assert_eq!(bilgi.son_erisim_ms, 2);
    assert!(depo
        .denetle(&RotaErisimi::Rol("yönetici".into()), None, false, 2)
        .is_ok());
    assert_eq!(
        depo.denetle(&RotaErisimi::Rol("okur".into()), None, false, 2)
            .unwrap_err()
            .durum,
        403
    );
    depo.oturum_kapat();
    assert_eq!(depo.oturum_bilgisi(2), None);
    assert_eq!(depo.oturum_sayisi().unwrap(), 0);
}

#[test]
fn kullanici_oturumlarini_sirsiz_listeler_ve_ozet_kimlikle_iptal_eder() {
    let mut depo = WebGuvenligi::yeni();
    istegi_baslat(&mut depo, None, 10);
    let yeni = depo
        .oturum_ac("Mustafa".into(), "yönetici".into(), 10, uretici())
        .unwrap();
    depo.istegi_tamamla(10).unwrap();

    let oturumlar = depo.kullanici_oturumlarini_listele("Mustafa", 11).unwrap();
    assert_eq!(oturumlar.len(), 1);
    assert_eq!(oturumlar[0].kimlik, anahtar(&yeni.belirtec));
    assert_ne!(oturumlar[0].kimlik, yeni.belirtec);
    assert_eq!(oturumlar[0].olusturma_ms, 10);
    assert!(depo
        .kullanici_oturumlarini_listele("Başka", 11)
        .unwrap()
        .is_empty());

    assert_eq!(depo.oturum_kimligini_iptal_et(&oturumlar[0].kimlik), Ok(1));
    depo.istegi_tamamla(11).unwrap();
    istegi_baslat(&mut depo, Some(&yeni.belirtec), 12);
    assert_eq!(depo.oturum_bilgisi(12), None);
    assert_eq!(depo.oturum_kimligini_iptal_et(&oturumlar[0].kimlik), Ok(0));
    assert!(depo.oturum_kimligini_iptal_et("ham-belirtec").is_err());
}

#[test]
fn erisim_mutlak_oturum_omrunu_uzatmaz() {
    let mut depo = WebGuvenligi::yeni();
    istegi_baslat(&mut depo, None, 0);
    let yeni = depo
        .oturum_ac("Zeynep".into(), "okur".into(), 0, uretici())
        .unwrap();
    istegi_baslat(
        &mut depo,
        Some(&yeni.belirtec),
        (OTURUM_OMRU_SANIYE - 1) * 1000,
    );
    assert!(depo
        .denetle(
            &RotaErisimi::Oturumlu,
            None,
            false,
            (OTURUM_OMRU_SANIYE - 1) * 1000,
        )
        .is_ok());
    istegi_baslat(
        &mut depo,
        Some(&yeni.belirtec),
        (OTURUM_OMRU_SANIYE + 1) * 1000,
    );
    assert_eq!(
        depo.denetle(
            &RotaErisimi::Oturumlu,
            None,
            false,
            (OTURUM_OMRU_SANIYE + 1) * 1000,
        )
        .unwrap_err()
        .durum,
        401
    );
}

#[test]
fn anonim_kota_en_uzun_suredir_kullanilmayani_tahliye_eder() {
    let mut depo = WebGuvenligi::sinirli(3, 2);
    let mut uret = uretici();
    istegi_baslat(&mut depo, None, 0);
    let (csrf_bir, bir) = depo.csrf_belirteci(0, &mut uret).unwrap();
    let bir = bir.unwrap();
    istegi_baslat(&mut depo, None, 1);
    let (csrf_iki, iki) = depo.csrf_belirteci(1, &mut uret).unwrap();
    let iki = iki.unwrap();

    istegi_baslat(&mut depo, Some(&bir.belirtec), 2);
    istegi_baslat(&mut depo, None, 3);
    let (csrf_uc, uc) = depo.csrf_belirteci(3, &mut uret).unwrap();
    let uc = uc.unwrap();
    assert_eq!(depo.oturum_sayisi().unwrap(), 2);

    istegi_baslat(&mut depo, Some(&iki.belirtec), 4);
    assert_eq!(
        depo.denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_iki), true, 4)
            .unwrap_err()
            .durum,
        403
    );
    istegi_baslat(&mut depo, Some(&bir.belirtec), 5);
    assert!(depo
        .denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_bir), true, 5)
        .is_ok());
    istegi_baslat(&mut depo, Some(&uc.belirtec), 6);
    assert!(depo
        .denetle(&RotaErisimi::HerkeseAcik, Some(&csrf_uc), true, 6)
        .is_ok());
}

#[test]
fn toplam_kota_doluyken_anonim_once_tahliye_edilir() {
    let mut depo = WebGuvenligi::sinirli(2, 2);
    let mut uret = uretici();
    istegi_baslat(&mut depo, None, 0);
    let (_, bir) = depo.csrf_belirteci(0, &mut uret).unwrap();
    let bir = bir.unwrap();
    istegi_baslat(&mut depo, None, 1);
    depo.csrf_belirteci(1, &mut uret).unwrap();
    istegi_baslat(&mut depo, None, 2);
    let giris = depo
        .oturum_ac("Zeynep".into(), "okur".into(), 2, &mut uret)
        .unwrap();
    assert_eq!(depo.oturum_sayisi().unwrap(), 2);

    istegi_baslat(&mut depo, Some(&bir.belirtec), 3);
    assert_eq!(
        depo.denetle(&RotaErisimi::HerkeseAcik, None, true, 3)
            .unwrap_err()
            .durum,
        403
    );
    istegi_baslat(&mut depo, Some(&giris.belirtec), 4);
    assert!(depo.denetle(&RotaErisimi::Oturumlu, None, false, 4).is_ok());
}

#[test]
fn yalniz_kimlikli_kayitlarla_dolu_depo_yeni_girisi_reddeder() {
    let mut depo = WebGuvenligi::sinirli(2, 1);
    let mut uret = uretici();
    istegi_baslat(&mut depo, None, 0);
    depo.oturum_ac("Bir".into(), "okur".into(), 0, &mut uret)
        .unwrap();
    istegi_baslat(&mut depo, None, 1);
    depo.oturum_ac("İki".into(), "okur".into(), 1, &mut uret)
        .unwrap();
    istegi_baslat(&mut depo, None, 2);
    let hata = depo
        .oturum_ac("Üç".into(), "okur".into(), 2, &mut uret)
        .unwrap_err();
    assert!(hata.contains("kapasitesi dolu"));
    assert_eq!(depo.oturum_sayisi().unwrap(), 2);
}

#[cfg(not(target_arch = "wasm32"))]
fn gecici_depo_yolu(ad: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SIRA: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir()
        .join(format!(
            "zee-web-depo-test-{}-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed),
            ad
        ))
        .join(".zee/web-durumu-v1.json")
}

#[cfg(not(target_arch = "wasm32"))]
fn gecici_depoyu_sil(yol: &std::path::Path) {
    if let Some(kok) = yol.parent().and_then(std::path::Path::parent) {
        let _ = std::fs::remove_dir_all(kok);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn bellek_oran_clone_atomik_yeniden_baslatma_ayri_ve_dosyaya_yazmaz() {
    let yol = gecici_depo_yolu("bellek-oran");
    let mut temel = WebGuvenligi::kalici_bellek_oranli(&yol);
    temel
        .istegi_baslat_kimlikle(None, "198.51.100.9", 0)
        .unwrap();
    let handles = (0..4)
        .map(|_| {
            let depo = temel.clone();
            std::thread::spawn(move || {
                (0..5)
                    .filter(|_| {
                        depo.rate_limit_artir(RateLimitTuru::Giris, "parola", 0)
                            .unwrap()
                            .izinli
                    })
                    .count()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .sum::<usize>(),
        5
    );
    assert!(
        !yol.exists(),
        "anonim istek ve oran artışı dosya oluşturmamalı"
    );
    let mut yeniden = WebGuvenligi::kalici_bellek_oranli(&yol);
    yeniden
        .istegi_baslat_kimlikle(None, "198.51.100.9", 0)
        .unwrap();
    assert!(
        yeniden
            .rate_limit_artir(RateLimitTuru::Giris, "parola", 0)
            .unwrap()
            .izinli
    );
    assert!(
        !temel
            .rate_limit_artir(RateLimitTuru::Giris, "parola", 1)
            .unwrap()
            .izinli
    );
    assert!(
        temel
            .rate_limit_artir(RateLimitTuru::Giris, "parola", 300_000)
            .unwrap()
            .izinli
    );
    // Gerçek oturum dosyası mevcutken de salt anonim oran sıcak yolu yazmaz.
    let yeni = temel
        .oturum_ac("Deniz".into(), "okur".into(), 300_000, uretici())
        .unwrap();
    temel.istegi_tamamla(300_000).unwrap();
    let once = std::fs::read(&yol).unwrap();
    let zaman = std::fs::metadata(&yol).unwrap().modified().unwrap();
    temel
        .istegi_baslat_kimlikle(None, "198.51.100.10", 300_001)
        .unwrap();
    temel
        .rate_limit_artir(RateLimitTuru::Ucnokta, "GET /", 300_001)
        .unwrap();
    assert_eq!(std::fs::read(&yol).unwrap(), once);
    assert_eq!(std::fs::metadata(&yol).unwrap().modified().unwrap(), zaman);
    yeniden
        .istegi_baslat_kimlikle(Some(&yeni.belirtec), "198.51.100.9", 300_002)
        .unwrap();
    assert!(yeniden.gelen_oturum.is_some());
    yeniden.oturum_kapat();
    yeniden.istegi_tamamla(300_002).unwrap();
    temel
        .istegi_baslat_kimlikle(Some(&yeni.belirtec), "198.51.100.9", 300_003)
        .unwrap();
    assert!(temel.gelen_oturum.is_none());
    std::fs::write(&yol, b"bozuk").unwrap();
    assert!(temel
        .istegi_baslat_kimlikle(Some(&yeni.belirtec), "198.51.100.9", 300_004)
        .is_err());
    gecici_depoyu_sil(&yol);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn kalici_depo_iki_worker_login_logout_restart_ve_expiry_paylasir() {
    let yol = gecici_depo_yolu("oturum");
    let mut a = WebGuvenligi::kalici(&yol);
    let mut b = WebGuvenligi::kalici(&yol);
    a.istegi_baslat_kimlikle(None, "203.0.113.1", 0).unwrap();
    let yeni = a
        .oturum_ac("Zeynep".into(), "yönetici".into(), 0, uretici())
        .unwrap();
    a.istegi_tamamla(0).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&yol).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(yol.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }

    b.istegi_baslat_kimlikle(Some(&yeni.belirtec), "203.0.113.1", 1)
        .unwrap();
    assert!(b
        .denetle(&RotaErisimi::Rol("yönetici".into()), None, false, 1)
        .is_ok());
    drop(b);

    let mut yeniden = WebGuvenligi::kalici(&yol);
    yeniden
        .istegi_baslat_kimlikle(Some(&yeni.belirtec), "203.0.113.1", 2)
        .unwrap();
    assert!(yeniden
        .denetle(&RotaErisimi::Oturumlu, None, false, 2)
        .is_ok());
    yeniden.oturum_kapat();
    yeniden.istegi_tamamla(2).unwrap();

    a.istegi_baslat_kimlikle(Some(&yeni.belirtec), "203.0.113.1", 3)
        .unwrap();
    assert_eq!(
        a.denetle(&RotaErisimi::Oturumlu, None, false, 3)
            .unwrap_err()
            .durum,
        401
    );

    let mut c = WebGuvenligi::kalici(&yol);
    c.istegi_baslat_kimlikle(None, "203.0.113.2", 10).unwrap();
    let dolacak = c
        .oturum_ac("Eliz".into(), "okur".into(), 10, uretici())
        .unwrap();
    c.istegi_tamamla(10).unwrap();
    let son = 10 + OTURUM_OMRU_SANIYE * 1_000;
    let mut d = WebGuvenligi::kalici(&yol);
    c.istegi_baslat_kimlikle(Some(&dolacak.belirtec), "203.0.113.2", son)
        .unwrap();
    d.istegi_baslat_kimlikle(Some(&dolacak.belirtec), "203.0.113.2", son)
        .unwrap();
    assert_eq!(
        c.denetle(&RotaErisimi::Oturumlu, None, false, son)
            .unwrap_err()
            .durum,
        401
    );
    assert_eq!(
        d.denetle(&RotaErisimi::Oturumlu, None, false, son)
            .unwrap_err()
            .durum,
        401
    );
    gecici_depoyu_sil(&yol);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn kalici_rate_limit_paralel_artisla_esigi_deldirmez() {
    let yol = gecici_depo_yolu("oran");
    let mut temel = WebGuvenligi::kalici(&yol);
    temel
        .istegi_baslat_kimlikle(None, "198.51.100.9", 0)
        .unwrap();
    // K-182: 4 yazar × 5 deneme = 20 > giriş oranı; kalıcı depo her artışı
    // fsync'li yazdığından 10×10 CI'ın yavaş diskinde 5 sn kilit sınırına
    // takılıyordu (ubuntu-latest). Paralel atomiklik daha az yazmayla da
    // kanıtlanır; kilit adaleti ve yavaş disk bütçesi K-183'te ölçülür.
    let mut isler = Vec::new();
    for _ in 0..4 {
        let depo = temel.clone();
        isler.push(std::thread::spawn(move || -> Result<usize, String> {
            let mut izinli = 0;
            for _ in 0..5 {
                if depo
                    .rate_limit_artir(RateLimitTuru::Giris, "parola", 0)?
                    .izinli
                {
                    izinli += 1;
                }
            }
            Ok(izinli)
        }));
    }
    let izinli = isler
        .into_iter()
        .map(|is| is.join().map_err(|_| "rate worker durdu".to_string())?)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .sum::<usize>();
    assert_eq!(
        izinli,
        crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
            .web()
            .giris_orani() as usize
    );
    gecici_depoyu_sil(&yol);
}

#[test]
fn ucnokta_csrf_ve_giris_pencereleri_ayri_ve_sureli_kalir() {
    let mut depo = WebGuvenligi::yeni();
    depo.istegi_baslat_kimlikle(None, "203.0.113.11", 0)
        .unwrap();
    let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.web();
    for _ in 0..sinirlar.ucnokta_orani() {
        assert!(
            depo.rate_limit_artir(RateLimitTuru::Ucnokta, "GET /", 0)
                .unwrap()
                .izinli
        );
    }
    assert!(
        !depo
            .rate_limit_artir(RateLimitTuru::Ucnokta, "GET /", 0)
            .unwrap()
            .izinli
    );
    assert!(
        depo.rate_limit_artir(RateLimitTuru::Ucnokta, "GET /başka", 0)
            .unwrap()
            .izinli
    );
    for _ in 0..sinirlar.csrf_orani() {
        assert!(
            depo.rate_limit_artir(RateLimitTuru::Csrf, "csrf", 0)
                .unwrap()
                .izinli
        );
    }
    assert!(
        !depo
            .rate_limit_artir(RateLimitTuru::Csrf, "csrf", 0)
            .unwrap()
            .izinli
    );
    assert!(
        depo.rate_limit_artir(
            RateLimitTuru::Ucnokta,
            "GET /",
            sinirlar.ucnokta_penceresi_saniye() * 1_000,
        )
        .unwrap()
        .izinli
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn kalici_oturum_commiti_yanit_hatasinda_kosullu_geri_alinir() {
    let yol = gecici_depo_yolu("geri-al");
    let mut a = WebGuvenligi::kalici(&yol);
    a.istegi_baslat_kimlikle(None, "192.0.2.1", 0).unwrap();
    let yeni = a
        .oturum_ac("Zeynep".into(), "okur".into(), 0, uretici())
        .unwrap();
    let commit = a.istegi_tamamla(0).unwrap();
    a.commiti_geri_al(commit).unwrap();

    let mut b = WebGuvenligi::kalici(&yol);
    b.istegi_baslat_kimlikle(Some(&yeni.belirtec), "192.0.2.1", 1)
        .unwrap();
    assert_eq!(
        b.denetle(&RotaErisimi::Oturumlu, None, false, 1)
            .unwrap_err()
            .durum,
        401
    );
    gecici_depoyu_sil(&yol);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn bozuk_kalici_depo_fail_closed_kalir() {
    let yol = gecici_depo_yolu("bozuk");
    std::fs::create_dir_all(yol.parent().unwrap()).unwrap();
    std::fs::write(&yol, b"{bozuk").unwrap();
    let mut depo = WebGuvenligi::kalici(&yol);
    let hata = depo
        .istegi_baslat_kimlikle(None, "192.0.2.1", 0)
        .unwrap_err();
    assert!(hata.contains("JSON"));
    gecici_depoyu_sil(&yol);
}

#[cfg(unix)]
#[test]
fn kalici_depo_sembolik_bagi_reddeder() {
    use std::os::unix::fs::symlink;
    let yol = gecici_depo_yolu("symlink");
    std::fs::create_dir_all(yol.parent().unwrap()).unwrap();
    let hedef = yol.parent().unwrap().join("hedef.json");
    std::fs::write(&hedef, b"{}").unwrap();
    symlink(&hedef, &yol).unwrap();
    let mut depo = WebGuvenligi::kalici(&yol);
    let hata = depo
        .istegi_baslat_kimlikle(None, "192.0.2.1", 0)
        .unwrap_err();
    assert!(hata.contains("sembolik bağ"));
    gecici_depoyu_sil(&yol);

    let dizin_yolu = gecici_depo_yolu("symlink-dizin");
    let kok = dizin_yolu.parent().unwrap().parent().unwrap();
    std::fs::create_dir_all(kok).unwrap();
    let hedef_dizin = kok.join("hedef-dizin");
    std::fs::create_dir(&hedef_dizin).unwrap();
    symlink(&hedef_dizin, dizin_yolu.parent().unwrap()).unwrap();
    let mut depo = WebGuvenligi::kalici(&dizin_yolu);
    let hata = depo
        .istegi_baslat_kimlikle(None, "192.0.2.1", 0)
        .unwrap_err();
    assert!(hata.contains("sembolik bağ"));
    gecici_depoyu_sil(&dizin_yolu);
}

#[test]
fn baslik_degerleri_enjeksiyona_kapali() {
    assert!(cerez_adi_gecerli("tema_1"));
    assert!(!cerez_adi_gecerli("x\r\nX-Test"));
    assert!(cerez_degeri_gecerli("abc-_123"));
    assert!(!cerez_degeri_gecerli("a; Secure"));
    assert!(yerel_yonlendirme_gecerli("/giris"));
    assert!(!yerel_yonlendirme_gecerli("https://kotu.example"));
    assert!(!yerel_yonlendirme_gecerli("//kotu.example"));
}
