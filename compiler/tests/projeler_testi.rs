//! Proje kitaplığı (projeler/) regression testleri: kitaplıktaki her proje
//! "çalışır belge"dir — dil değişir de bir proje kırılırsa burası söyler.

use dil::{kaynagi_calistir, kaynagi_calistir_girdiyle, kaynagi_dene};

fn proje(ad: &str) -> String {
    std::fs::read_to_string(format!("../projeler/{}", ad))
        .unwrap_or_else(|_| panic!("{} okunamadı", ad))
}

#[test]
fn carpim_tablosu_tam() {
    let cikti = kaynagi_calistir(&proje("carpim-tablosu.dil")).expect("çalışmalı");
    // 10 başlık + 100 satır.
    assert_eq!(cikti.len(), 110);
    assert_eq!(cikti[0], "--- 1 tablosu ---");
    assert!(cikti.contains(&"7 x 8 = 56".to_string()));
    assert_eq!(cikti[109], "10 x 10 = 100");
}

#[test]
fn hikaye_cevaplari_dokur() {
    let girdiler = vec![
        "Zeynep".to_string(),
        "ejderha".to_string(),
        "Kapadokya".to_string(),
        "9".to_string(),
    ];
    let cikti =
        kaynagi_calistir_girdiyle(&proje("hikaye.dil"), girdiler).expect("çalışmalı");
    assert!(cikti.contains(&"Zeynep adında bir kahraman varmış.".to_string()));
    assert!(cikti.contains(&"Yol tam 9 gün sürmüş.".to_string()));
    assert!(cikti.contains(&"Uzun bir yolculukmuş doğrusu!".to_string()));
}

#[test]
fn quiz_puani_dogru_sayar() {
    let girdiler = vec!["Ankara".to_string(), "56".to_string(), "5".to_string()];
    let cikti = kaynagi_calistir_girdiyle(&proje("quiz.dil"), girdiler).expect("çalışmalı");
    assert!(cikti.contains(&"Puanın: 2 / 3".to_string()));
    assert!(cikti.contains(&"Çok iyi!".to_string()));

    let hepsi_dogru = vec!["Ankara".to_string(), "56".to_string(), "7".to_string()];
    let cikti =
        kaynagi_calistir_girdiyle(&proje("quiz.dil"), hepsi_dogru).expect("çalışmalı");
    assert!(cikti.contains(&"Puanın: 3 / 3".to_string()));
    assert!(cikti.contains(&"Mükemmel! Hepsini bildin!".to_string()));
}

#[test]
fn zar_oyunu_tutarli_biter() {
    // Rastgelelik hermetik IO'dan gelir: aynı ortamda aynı oyun. Yapısal
    // doğrulama: 3 el + özet; final üç sonuçtan biri; skor toplamı ≤ 3.
    let cikti = kaynagi_calistir(&proje("zar-oyunu.dil")).expect("çalışmalı");
    let eller = cikti.iter().filter(|s| s.starts_with("Senin zarın: ")).count();
    assert_eq!(eller, 3);
    let final_satiri = cikti.last().expect("çıktı boş olmamalı");
    assert!(
        final_satiri == "Oyunu SEN kazandın!"
            || final_satiri == "Oyunu bilgisayar kazandı"
            || final_satiri == "Oyun berabere bitti",
        "beklenmedik final: {final_satiri}"
    );
    let tekrar = kaynagi_calistir(&proje("zar-oyunu.dil")).expect("çalışmalı");
    assert_eq!(cikti, tekrar, "hermetik IO'da oyun deterministik olmalı");
}

#[test]
fn market_hesabi_ve_testi() {
    let cikti = kaynagi_calistir(&proje("market-listesi.dil")).expect("çalışmalı");
    assert!(cikti.contains(&"Ara toplam: 46,15 lira".to_string()));
    assert!(cikti.contains(&"KDV dahil: 55,38 lira".to_string()));

    let sonuclar = kaynagi_dene(&proje("market-listesi.dil")).expect("dene çalışmalı");
    assert_eq!(sonuclar.len(), 1);
    assert!(sonuclar[0].hata.is_none(), "kdv testi geçmeli");
}

#[test]
fn zamir_nsi_cozulur() {
    // K-041: iyelikli köke hâl eki zamir n'siyle bağlanır —
    // "bilgisayarın_zarı"na "-ndan" gelince kök yine bulunur.
    let kaynak = "\
bilgisayarın_zarı 3 olsun
5 bilgisayarın_zarından büyükse
    \"büyük\" yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["büyük"]);
}

#[test]
fn asal_avcisi_dogru_sayar() {
    let cikti = kaynagi_calistir(&proje("asal-sayilar.dil")).expect("çalışmalı");
    assert_eq!(cikti[0], "2 ile 50 arasındaki asallar:");
    assert_eq!(cikti.last().unwrap(), "Toplam 15 asal bulundu");
    assert!(cikti.contains(&"47".to_string()));
    assert!(!cikti.contains(&"49".to_string()), "49 = 7x7 asal değil");
}

#[test]
fn kumbara_hedefe_ulasir() {
    let cikti = kaynagi_calistir(&proje("kumbara.dil")).expect("çalışmalı");
    assert!(cikti.contains(&"Hafta 12: 285,0 lira".to_string()));
    assert_eq!(cikti.last().unwrap(), "250,0 liraya 12 haftada ulaştın!");
}

#[test]
fn gizli_dil_kelime_cevirir() {
    let girdiler = vec!["okul çok güzel bugün".to_string()];
    let cikti =
        kaynagi_calistir_girdiyle(&proje("gizli-dil.dil"), girdiler).expect("çalışmalı");
    assert!(cikti.contains(&"Gizli hali: balina fıstık yıldızlı bugün ".to_string()));
}

#[test]
fn kelime_sayaci_dogru_sayar() {
    let girdiler = vec!["bal tut bal ye bal".to_string()];
    let cikti =
        kaynagi_calistir_girdiyle(&proje("kelime-sayaci.dil"), girdiler).expect("çalışmalı");
    assert!(cikti.contains(&"bal: 3".to_string()));
    assert!(cikti.contains(&"tut: 1".to_string()));
}

#[test]
fn gun_sayar_sahte_takvimle() {
    // Hermetik saat: ToplayanIo 31 Ağustos 2026'da yaşar.
    let cikti = kaynagi_calistir(&proje("gun-sayar.dil")).expect("çalışmalı");
    assert!(cikti[0].starts_with("Bugün: "), "{}", cikti[0]);
    assert!(cikti.contains(&"Yıl: 2026".to_string()));
    assert!(cikti.iter().any(|s| s.starts_with("Yarın: ")), "{:?}", cikti);
}

#[test]
fn mini_site_html_uretir() {
    // Hermetik sunucu: sahte istek kuyruğu, gömülü kitaplık yükleyicisi.
    let mut yukleyici = |ad: &str| -> Result<String, String> {
        dil::gomulu_birim(ad).map(str::to_string).ok_or_else(|| "yok".into())
    };
    let program = dil::kaynagi_derle_birimlerle(&proje("mini-site.dil"), &mut yukleyici)
        .expect("derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    io.istekler = vec!["/".into(), "/obeb".into(), "/kayip".into()].into();
    dil::yorumlayici::calistir_io(&program, &mut io).expect("çalışmalı");

    assert_eq!(io.sunucu_yanitlari.len(), 3);
    let (_, ana) = &io.sunucu_yanitlari[0];
    assert!(ana.starts_with("<!doctype html>"), "{}", &ana[..40]);
    assert!(ana.contains("<h1>zee ile yapılmış site</h1>"));
    let (_, obeb) = &io.sunucu_yanitlari[1];
    assert!(obeb.contains("obebi: <b>12</b>"), "{}", obeb);
    assert!(io.sunucu_yanitlari[2].1.contains("aranan sayfa yok"));
}

#[test]
fn envanter_stok_defteri() {
    let cikti = kaynagi_calistir(&proje("envanter.dil")).expect("çalışmalı");
    assert!(cikti.contains(&"Toplam stok değeri: 1824,50 lira".to_string()));
    assert!(cikti.contains(&"Alfabetik: çay, şeker, un".to_string()), "{:?}", cikti);
}

#[test]
fn envanter_json_yedegi_yazilir() {
    let kaynak = proje("envanter.dil");
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    let mut io = dil::yorumlayici::ToplayanIo::yeni(Vec::new());
    dil::yorumlayici::calistir_io(&program, &mut io).expect("çalışmalı");
    let yedek = io.dosyalar.get("envanter.json").expect("yedek dosyası olmalı");
    assert!(yedek.starts_with("[{\"ad\":\"çay\""), "{}", &yedek[..40.min(yedek.len())]);
    assert!(yedek.contains("\"fiyat\":52.0"));
}
