use dil::cozumleyici::{ad_cozumle, Tur};
use dil::morfoloji::{
    cozumleri_bul, ek_uydur, ek_zinciri_uydur, kok_adaylari, profil_dokumu, SoyutEk, EK_TABLOSU,
    MAKSIMUM_EK_KATMANI, MORFOLOJI_PROFILI, MORFOLOJI_SURUMU,
};
use std::collections::{HashMap, HashSet};
use std::process::Command;

const KOKLER: &[&str] = &[
    "sayı", "elma", "araba", "kedi", "puan", "okul", "göl", "çocuk", "kitap", "yurt", "ağaç",
    "kanat", "renk", "top", "kâr",
];

const TEK_EKLER: &[SoyutEk] = &[
    SoyutEk::Belirtme,
    SoyutEk::Tamlayan,
    SoyutEk::Yonelme,
    SoyutEk::Ayrilma,
    SoyutEk::Bulunma,
    SoyutEk::Arac,
    SoyutEk::CogulYonelme,
];

const DIS_EKLER: &[SoyutEk] = &[
    SoyutEk::Belirtme,
    SoyutEk::Tamlayan,
    SoyutEk::Yonelme,
    SoyutEk::Ayrilma,
    SoyutEk::Bulunma,
    SoyutEk::Arac,
];

#[test]
fn profil_surumu_ve_ek_tablosu_snapshot_ile_kilitlidir() {
    assert_eq!(MORFOLOJI_PROFILI, "zee-tr-1");
    assert_eq!(MORFOLOJI_SURUMU, 1);
    assert_eq!(MAKSIMUM_EK_KATMANI, 2);
    assert_eq!(profil_dokumu(), include_str!("morfoloji_v1.snapshot"));

    let mut kimlikler = HashSet::new();
    for tanim in EK_TABLOSU {
        assert!(
            kimlikler.insert(tanim.ek),
            "soyut ek iki kez tanımlı: {:?}",
            tanim.ek
        );
        assert!(!tanim.yuzeyler.is_empty(), "{:?} yüzeysiz olamaz", tanim.ek);
        let mut yuzeyler = HashSet::new();
        for yuzey in tanim.yuzeyler {
            assert!(
                yuzeyler.insert(*yuzey),
                "{:?} içinde yinelenen yüzey: {}",
                tanim.ek,
                yuzey
            );
        }
    }
}

#[test]
fn uretilen_her_tek_ek_ayni_koke_ve_ek_kimligine_cozulur() {
    for kok in KOKLER {
        for &ek in TEK_EKLER {
            let yuzey = ek_uydur(kok, ek);
            let cozumler = cozumleri_bul(&yuzey);
            assert!(
                cozumler
                    .iter()
                    .any(|cozum| cozum.kok == *kok && cozum.ekler == [ek]),
                "{} + {:?} = {} geri çözülemedi: {:?}",
                kok,
                ek,
                yuzey,
                cozumler
            );
        }
    }
}

#[test]
fn uretilen_her_iyelik_zinciri_iki_katmanla_geri_cozulur() {
    for kok in KOKLER {
        for &dis in DIS_EKLER {
            let ekler = [SoyutEk::Iyelik, dis];
            let yuzey = ek_zinciri_uydur(kok, &ekler).expect("v1 iyelik zinciri");
            let cozumler = cozumleri_bul(&yuzey);
            assert!(
                cozumler
                    .iter()
                    .any(|cozum| cozum.kok == *kok && cozum.ekler == ekler),
                "{} + {:?} = {} geri çözülemedi: {:?}",
                kok,
                ekler,
                yuzey,
                cozumler
            );
        }
    }
}

#[test]
fn kanonik_uretim_ornekleri_profil_icinde_degismez() {
    let tek = [
        ("elma", SoyutEk::Belirtme, "elmayı"),
        ("elma", SoyutEk::Tamlayan, "elmanın"),
        ("elma", SoyutEk::Yonelme, "elmaya"),
        ("elma", SoyutEk::Ayrilma, "elmadan"),
        ("elma", SoyutEk::Bulunma, "elmada"),
        ("elma", SoyutEk::Arac, "elmayla"),
        ("elma", SoyutEk::CogulYonelme, "elmalara"),
        ("kitap", SoyutEk::Belirtme, "kitabı"),
        ("kitap", SoyutEk::Tamlayan, "kitabın"),
        ("kitap", SoyutEk::Yonelme, "kitaba"),
        ("kitap", SoyutEk::Ayrilma, "kitaptan"),
        ("kitap", SoyutEk::Bulunma, "kitapta"),
        ("kitap", SoyutEk::Arac, "kitapla"),
        ("renk", SoyutEk::Belirtme, "rengi"),
        ("renk", SoyutEk::Tamlayan, "rengin"),
        ("renk", SoyutEk::Yonelme, "renge"),
        ("göl", SoyutEk::Belirtme, "gölü"),
        ("top", SoyutEk::Belirtme, "topu"),
    ];
    for (kok, ek, beklenen) in tek {
        assert_eq!(ek_uydur(kok, ek), beklenen, "{} + {:?}", kok, ek);
    }

    let zincirler = [
        ("elma", SoyutEk::Belirtme, "elmasını"),
        ("elma", SoyutEk::Tamlayan, "elmasının"),
        ("elma", SoyutEk::Yonelme, "elmasına"),
        ("elma", SoyutEk::Ayrilma, "elmasından"),
        ("elma", SoyutEk::Bulunma, "elmasında"),
        ("elma", SoyutEk::Arac, "elmasıyla"),
        ("kitap", SoyutEk::Belirtme, "kitabını"),
        ("kitap", SoyutEk::Tamlayan, "kitabının"),
        ("kitap", SoyutEk::Yonelme, "kitabına"),
        ("kitap", SoyutEk::Ayrilma, "kitabından"),
        ("kitap", SoyutEk::Bulunma, "kitabında"),
        ("kitap", SoyutEk::Arac, "kitabıyla"),
    ];
    for (kok, dis, beklenen) in zincirler {
        assert_eq!(
            ek_zinciri_uydur(kok, &[SoyutEk::Iyelik, dis]).as_deref(),
            Some(beklenen),
            "{} + iyelik + {:?}",
            kok,
            dis
        );
    }
}

#[test]
fn ters_ses_degisimi_korpusu_koku_korur() {
    let ornekler = [
        ("sayacı", "sayaç", SoyutEk::Belirtme),
        ("kitabı", "kitap", SoyutEk::Belirtme),
        ("rengi", "renk", SoyutEk::Belirtme),
        ("üssü", "üs", SoyutEk::Belirtme),
        ("affı", "af", SoyutEk::Belirtme),
        ("reddi", "ret", SoyutEk::Belirtme),
        ("tıbbı", "tıp", SoyutEk::Belirtme),
        ("şekli", "şekil", SoyutEk::Belirtme),
        ("burnu", "burun", SoyutEk::Belirtme),
        ("oğlu", "oğul", SoyutEk::Belirtme),
    ];
    for (yuzey, kok, ek) in ornekler {
        assert!(
            cozumleri_bul(yuzey)
                .iter()
                .any(|cozum| cozum.kok == kok && cozum.ekler == [ek]),
            "{} → {} + {:?}",
            yuzey,
            kok,
            ek
        );
        let kanonik = ek_uydur(kok, ek);
        assert!(kok_adaylari(&kanonik).iter().any(|aday| aday == kok));
    }
}

#[test]
fn belirsizlik_korpusu_tahmin_edilmeden_a002_olur() {
    let korpus: &[(&str, &[&str])] = &[
        ("payı", &["pa", "pay"]),
        ("sayacı", &["sayac", "sayaç"]),
        ("fiyatıyla", &["fiyatı", "fiyat"]),
        ("zarından", &["zarı", "zar"]),
    ];

    for (yuzey, kokler) in korpus {
        let adaylar = kok_adaylari(yuzey);
        for kok in *kokler {
            assert!(
                adaylar.iter().any(|aday| aday == kok),
                "{} içinde {} yok: {:?}",
                yuzey,
                kok,
                adaylar
            );
        }
        let ortam = kokler
            .iter()
            .map(|kok| ((*kok).to_string(), Tur::TamSayi))
            .collect::<HashMap<_, _>>();
        let hata = ad_cozumle(yuzey, &ortam, 1, 1, yuzey.chars().count())
            .expect_err("belirsizlik sessizce seçilemez");
        assert_eq!(hata.kod, "A002", "{}: {}", yuzey, hata);
    }
}

#[test]
fn ek_tablosundaki_her_yapisal_sonek_cakismasi_a002_olabilir() {
    let dis_yuzeyler = EK_TABLOSU
        .iter()
        .filter(|tanim| tanim.ek != SoyutEk::Iyelik)
        .flat_map(|tanim| tanim.yuzeyler.iter().copied())
        .collect::<Vec<_>>();
    let mut sinanan = 0usize;

    for &uzun in &dis_yuzeyler {
        for &kisa in &dis_yuzeyler {
            if uzun == kisa || !uzun.ends_with(kisa) {
                continue;
            }
            let ara = uzun.strip_suffix(kisa).expect("sonek ilişkisi");
            let birinci = "mavi".to_string();
            let ikinci = format!("mavi{}", ara);
            if birinci == ikinci {
                continue;
            }
            let yuzey = format!("mavi{}", uzun);
            let adaylar = kok_adaylari(&yuzey);
            assert!(adaylar.contains(&birinci), "{} → {:?}", yuzey, adaylar);
            assert!(adaylar.contains(&ikinci), "{} → {:?}", yuzey, adaylar);

            let ortam = [birinci, ikinci]
                .into_iter()
                .map(|kok| (kok, Tur::TamSayi))
                .collect::<HashMap<_, _>>();
            assert_eq!(
                ad_cozumle(&yuzey, &ortam, 1, 1, yuzey.chars().count())
                    .expect_err("yapısal çakışma seçilemez")
                    .kod,
                "A002"
            );
            sinanan += 1;
        }
    }
    assert!(sinanan > 0, "profil en az bir yapısal çakışma taşımalı");
}

#[test]
fn gecersiz_ve_uc_katmanli_zincir_uretilmez() {
    assert!(ek_zinciri_uydur("fiyat", &[]).is_none());
    assert!(ek_zinciri_uydur("fiyat", &[SoyutEk::Arac, SoyutEk::Belirtme]).is_none());
    assert!(
        ek_zinciri_uydur("fiyat", &[SoyutEk::Iyelik, SoyutEk::Arac, SoyutEk::Yonelme]).is_none()
    );
}

#[test]
fn cli_profili_ve_belirsiz_cozumleri_gorunur_kilar() {
    let profil = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("morfoloji")
        .output()
        .expect("morfoloji profili");
    assert!(profil.status.success());
    assert_eq!(
        String::from_utf8(profil.stdout).expect("utf8"),
        include_str!("morfoloji_v1.snapshot")
    );

    let cozum = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["morfoloji", "sayacı"])
        .output()
        .expect("morfoloji çözümü");
    assert!(cozum.status.success());
    let stdout = String::from_utf8(cozum.stdout).expect("utf8");
    assert!(stdout.contains("- sayac + belirtme"), "{}", stdout);
    assert!(stdout.contains("- sayaç + belirtme"), "{}", stdout);
}
