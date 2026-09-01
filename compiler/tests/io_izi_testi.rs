//! K-115 deterministik IO trace/replay davranış kanıtları.

use dil::yorumlayici::{calistir_io, GirdiCikti, IzKaydedenIo, IzYenidenOynatici, ToplayanIo};

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
