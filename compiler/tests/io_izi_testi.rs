//! K-115 deterministik IO trace/replay davranış kanıtları.

use dil::yorumlayici::{
    calistir_io, EylemHataSinifi, EylemHatasi, GirdiCikti, IzKaydedenIo, IzYenidenOynatici,
    ToplayanIo, VeritabaniHatasi,
};

#[test]
fn dosya_yasam_dongusu_izi_fiziksel_etki_yapmadan_oynatilir() {
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.dosyalar.insert("temp/a.bin".into(), "abc".into());
    taban.dosyalar.insert("temp/orphan.bin".into(), "x".into());
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    assert_eq!(
        kaydeden.dosya_atomik_tasi("temp/a.bin", "medya/a.bin"),
        Ok(1)
    );
    assert_eq!(
        kaydeden.dosyalari_listele("temp"),
        Ok(vec!["temp/orphan.bin".into()])
    );
    let ozet = kaydeden.dosya_sha256("medya/a.bin").unwrap();
    assert_eq!(kaydeden.dosya_sil("medya/a.bin"), Ok(1));
    let iz = kaydeden.iz_metni().unwrap();

    let mut oynatici = IzYenidenOynatici::yeni(&iz).unwrap();
    assert_eq!(
        oynatici.dosya_atomik_tasi("temp/a.bin", "medya/a.bin"),
        Ok(1)
    );
    assert_eq!(
        oynatici.dosyalari_listele("temp"),
        Ok(vec!["temp/orphan.bin".into()])
    );
    assert_eq!(oynatici.dosya_sha256("medya/a.bin"), Ok(ozet));
    assert_eq!(oynatici.dosya_sil("medya/a.bin"), Ok(1));
    oynatici.bitir().unwrap();
}

#[test]
fn commit_sonucu_belirsiz_sinifi_io_izinde_korunur() {
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban
        .eylem_tamamla_sonuclari
        .push_back(Err(EylemHatasi::commit_sonucu_belirsiz(
            "COMMIT cevabı kayboldu",
        )));
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    kaydeden.eylem_baslat().expect("transaction başlamalı");
    let hata = kaydeden
        .eylem_tamamla()
        .expect_err("COMMIT sonucu belirsiz olmalı");
    assert_eq!(hata.sinif, EylemHataSinifi::CommitSonucuBelirsiz);

    let iz = kaydeden.iz_metni().expect("iz yazılmalı");
    assert!(iz.contains("636f6d6d69745f736f6e7563755f62656c697273697a"));
    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    oynatici.eylem_baslat().expect("başlangıç oynatılmalı");
    let hata = oynatici
        .eylem_tamamla()
        .expect_err("belirsiz sınıf oynatılmalı");
    assert_eq!(hata.sinif, EylemHataSinifi::CommitSonucuBelirsiz);
    oynatici.bitir().expect("iz bitmeli");
}

#[test]
fn program_gercek_io_izinden_dis_dunyasiz_aynen_oynatilir() {
    let kaynak = r#"
"Ad?" diye sor
yanıt yaz
zar 1 ile 6 arasında rastgele sayı olsun
zar yaz
"not.txt" dosyasına "miras" yaz
satırlar "not.txt" dosyasının satırları olsun
satırların ilki yaz
bugün bugünün tarihi olsun
bugünün yılı yaz
gelenler komut satırından gelenler olsun
her gelen için
    geleni yaz
cevap "https://ornek.dev/durum" adresinden gelen yanıt olsun
cevabın gövdesini yaz
kapı açıksa
    yeşil ışığı yak
"#;
    let program = dil::kaynagi_derle(kaynak).expect("iz programı derlenmeli");
    let mut taban = ToplayanIo::yeni(vec!["Zeynep".into()]);
    taban.rastgele_degerler.push_back(4);
    taban.argumanlar = vec!["bir".into(), "iki".into()];
    taban
        .http_yanitlari
        .insert("https://ornek.dev/durum".into(), (200, "çalışıyor".into()));
    taban.sensorler.insert("kapı".into(), true);

    let mut kaydeden = IzKaydedenIo::yeni(taban);
    calistir_io(&program, &mut kaydeden).expect("kayıt koşusu çalışmalı");
    let beklenen_cikti = kaydeden.ic.cikti.clone();
    let iz = kaydeden.iz_metni().expect("iz kanonik yazılmalı");
    assert!(kaydeden.olay_sayisi() >= 14, "iz dış etkileri kapsamalı");

    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    calistir_io(&program, &mut oynatici).expect("replay çalışmalı");
    assert_eq!(oynatici.cikti, beklenen_cikti);
    oynatici.bitir().expect("bütün iz tam sırada tüketilmeli");
}

#[test]
fn iz_byte_bicimi_surumlu_ve_kanoniktir() {
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.rastgele_degerler.push_back(4);
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    kaydeden.yazdir("A".into());
    assert_eq!(kaydeden.rastgele(1, 6), 4);

    assert_eq!(
        kaydeden.iz_metni().expect("iz yazılmalı"),
        "zee-io-izi\t1\n1\tyazdir\t1\t0\t41\n2\trastgele\t2\t1\t31\t36\t34\n"
    );
}

#[test]
fn replay_sira_arguman_ve_tam_tuketimde_fail_closed_durur() {
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.rastgele_degerler.push_back(4);
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    kaydeden.rastgele(1, 6);
    let iz = kaydeden.iz_metni().expect("iz yazılmalı");

    let mut farkli = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    assert_eq!(farkli.rastgele(1, 10), 1, "uyuşmazlıkta güvenli alt uç");
    assert!(farkli
        .bitir()
        .expect_err("argüman farkı reddedilmeli")
        .contains("argümanları"));

    let eksik = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    assert!(eksik
        .bitir()
        .expect_err("tüketilmeyen olay reddedilmeli")
        .contains("tamamlanmadı"));
}

#[test]
fn bozuk_ve_kanonik_olmayan_izler_yurutulmeden_reddedilir() {
    let vakalar = [
        "zee-io-izi\t2\n",
        "zee-io-izi\t1",
        "zee-io-izi\t1\n2\tyazdir\t1\t0\t41\n",
        "zee-io-izi\t1\n1\tbilinmeyen\t0\t0\n",
        "zee-io-izi\t1\n1\tyazdir\t1\t0\t4A\n",
        "zee-io-izi\t1\n01\tyazdir\t1\t0\t41\n",
        "zee-io-izi\t1\n1\trastgele\t2\t1\t31\t36\t3034\n",
    ];
    for iz in vakalar {
        assert!(IzYenidenOynatici::yeni(iz).is_err(), "reddedilmeli: {iz:?}");
    }
}

#[test]
fn parola_ve_ozet_ham_ya_da_hexlenmis_halde_ize_sizmaz() {
    let mut kaydeden = IzKaydedenIo::yeni(ToplayanIo::yeni(Vec::new()));
    assert!(!kaydeden.parola_dogrula("parola", "ozet"));
    let iz = kaydeden.iz_metni().expect("iz yazılmalı");

    assert!(
        !iz.contains("7061726f6c61"),
        "ham parola yalnız hexlenmemeli"
    );
    assert!(!iz.contains("6f7a6574"), "ham özet yalnız hexlenmemeli");

    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    assert!(!oynatici.parola_dogrula("parola", "ozet"));
    oynatici.bitir().expect("parmak izleri eşleşmeli");
}

#[test]
fn postgresql_okuma_yazma_ve_yapilandirilmis_hata_izden_aynen_oynatilir() {
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.postgresql.okuma_sonuclari.push_back(Ok(vec![vec![
        ("slug".into(), "anasayfa".into()),
        ("baslik".into(), "Zee".into()),
    ]]));
    taban
        .postgresql
        .degistirme_sonuclari
        .push_back(Err(VeritabaniHatasi {
            mesaj: "benzersiz alan çakıştı".into(),
            veri: vec![("sqlstate".into(), "23505".into())],
        }));
    let mut kaydeden = IzKaydedenIo::yeni(taban);
    let okuma = kaydeden
        .postgresql_oku("SELECT slug::text", &["anasayfa".into()])
        .expect("okuma kaydedilmeli");
    let hata = kaydeden
        .postgresql_degistir("INSERT INTO sayfa VALUES ($1)", &["anasayfa".into()])
        .expect_err("hata kaydedilmeli");
    assert_eq!(okuma[0][0].1, "anasayfa");
    assert_eq!(hata.veri[0].1, "23505");
    let iz = kaydeden.iz_metni().expect("iz yazılmalı");

    let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("iz okunmalı");
    assert_eq!(
        oynatici
            .postgresql_oku("SELECT slug::text", &["anasayfa".into()])
            .expect("okuma oynatılmalı"),
        okuma
    );
    assert_eq!(
        oynatici
            .postgresql_degistir("INSERT INTO sayfa VALUES ($1)", &["anasayfa".into()])
            .expect_err("hata oynatılmalı"),
        hata
    );
    oynatici.bitir().expect("bütün iz tüketilmeli");
}
