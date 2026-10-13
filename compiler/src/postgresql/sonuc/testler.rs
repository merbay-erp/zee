use super::*;

#[test]
fn hata_sonrasi_kopyalama_durur_ve_satir_siniri_onceliklidir() {
    let mut t = Toplayici::default();
    t.kaydet(|_| Ok(vec![("a".into(), "saklanmaz".into())]));
    let ilk = hata("ilk hücre hatası");
    t.kaydet(|_| Err(ilk.clone()));
    assert!(t.sonuc.is_empty());
    for _ in 2..10_000 {
        t.kaydet(|_| panic!("hata sonrası hücre kopyalanamaz"));
    }
    assert_eq!(t.ilk_hata.as_ref(), Some(&ilk));
    t.kaydet(|_| panic!("satır sınırından sonra hücre kopyalanamaz"));
    assert_eq!(
        t.bitir(),
        Err(hata("PostgreSQL sonucu 10.000 satır sınırını aşıyor"))
    );
}

#[test]
fn sinira_kadar_satir_ve_sutun_sirasi_korunur() {
    let mut t = Toplayici::default();
    for i in 0..10_000 {
        t.kaydet(|toplam| {
            *toplam += 1;
            Ok(vec![
                ("a".into(), i.to_string()),
                ("a".into(), "ikinci".into()),
            ])
        });
    }
    assert_eq!(t.toplam, 10_000);
    let sonuc = t.bitir().unwrap();
    assert_eq!(sonuc.len(), 10_000);
    assert_eq!(
        sonuc[9_999],
        [("a".into(), "9999".into()), ("a".into(), "ikinci".into())]
    );
}

// Immutable 4a4fd0d Vec conversion oracle; query transport error precedes all cell checks.
fn eski(
    istemci: &mut Client,
    sorgu: &str,
    parametreler: &[String],
) -> crate::veritabani_modeli::VeritabaniOkumaSonucu {
    let baglar = super::super::metin_baglari(parametreler);
    let satirlar = istemci
        .query_typed(sorgu, &baglar)
        .map_err(|h| postgres_hatasi("PostgreSQL sorgusu başarısız", h))?;
    let sinirlar = crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.veritabani();
    if satirlar.len() > sinirlar.satir() {
        return Err(hata("PostgreSQL sonucu 10.000 satır sınırını aşıyor"));
    }
    let mut toplam = 0usize;
    let mut sonuc = Vec::with_capacity(satirlar.len());
    for satir in satirlar {
        if satir.len() > sinirlar.sutun() {
            return Err(hata("PostgreSQL sonucu 100 sütun sınırını aşıyor"));
        }
        let mut alanlar = Vec::with_capacity(satir.len());
        for (sira, sutun) in satir.columns().iter().enumerate() {
            if !matches!(
                *sutun.type_(),
                Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME
            ) {
                return Err(verili_hata(
                    format!(
                        "\"{}\" sütunu Metin değil; sorguda açık ::text dönüşümü kullan",
                        sutun.name()
                    ),
                    vec![("sutun".into(), sutun.name().into())],
                ));
            }
            let deger: Option<String> = satir
                .try_get(sira)
                .map_err(|h| postgres_hatasi("PostgreSQL sonuç sütunu okunamadı", h))?;
            let Some(deger) = deger else {
                return Err(verili_hata(
                    format!(
                        "\"{}\" sütunu NULL; sorguda COALESCE ile açık metin değeri seç",
                        sutun.name()
                    ),
                    vec![("sutun".into(), sutun.name().into())],
                ));
            };
            toplam = toplam.saturating_add(sutun.name().len() + deger.len());
            if toplam > sinirlar.sonuc_bayti() {
                return Err(hata("PostgreSQL sonucu 16 MiB sınırını aşıyor"));
            }
            alanlar.push((sutun.name().to_string(), deger));
        }
        sonuc.push(alanlar);
    }
    Ok(sonuc)
}

#[test]
fn izole_saha_eski_vec_ile_stream_hatalari_ve_transaction_sagligi_aynidir() {
    let Ok(hedef) = std::env::var("ZEE_POSTGRES_K206_HEDEFI") else {
        return;
    };
    let url = std::env::var("ZEE_POSTGRES_K206_URL").unwrap();
    let hedef = crate::veritabani_modeli::VeritabaniHedefi::ayristir(&hedef).unwrap();
    assert!(
        hedef.loopback_mi(),
        "bu sentetik test yalnız izole loopback sunucudadır"
    );
    let bildirim = crate::veritabani_modeli::VeritabaniBildirimi {
        hedef,
        baglanti_degiskeni: "ZEE_POSTGRES_K206_URL".into(),
        gocler: "unused".into(),
        istek_baglami_cozucusu: None,
    };
    let mut yeni = super::super::PostgresqlOturumu::baglan(&bildirim, 0).unwrap();
    let mut eski_istemci = Client::connect(&url, postgres::NoTls).unwrap();
    let mut sorgular = vec![
        "SELECT 'ok'::text AS value".to_string(),
        "SELECT NULL::text AS value WHERE FALSE".into(),
        "SELECT NULL::text AS value".into(),
        "SELECT 1::integer AS value".into(),
        "SELECT 'x'::varchar AS value, 'a'::char(2) AS padded, 'n'::name AS named".into(),
        "SELECT i::text AS value FROM generate_series(1,10000) i".into(),
        "SELECT i::text AS value FROM generate_series(1,10001) i".into(),
        "SELECT NULL::text AS value FROM generate_series(1,10001) i".into(),
        "SELECT i::integer AS value FROM generate_series(1,10001) i".into(),
        "SELECT repeat('x',1048576)::text AS payload FROM generate_series(1,64)".into(),
        "SELECT repeat('x',2048)::text AS payload FROM generate_series(1,10001)".into(),
        "SELECT CASE WHEN i=10 THEN (1/(i-i))::text ELSE NULL::text END AS value FROM generate_series(1,10) i".into(),
        "SELECT CASE WHEN i=10002 THEN (1/(i-i))::text ELSE 'x'::text END AS value FROM generate_series(1,10002) i".into(),
    ];
    for sutun in [100, 101] {
        sorgular.push(format!(
            "SELECT {}",
            (0..sutun)
                .map(|i| format!("'x'::text AS c{i}"))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    for sorgu in &sorgular {
        assert_eq!(
            yeni.oku(sorgu, &[]),
            eski(&mut eski_istemci, sorgu, &[]),
            "{sorgu}"
        );
        assert!(yeni.oku("SELECT 'ok'::text AS health", &[]).is_ok());
    }
    let parametre = vec!["x' OR true --".to_string()];
    assert_eq!(
        yeni.oku("SELECT $1::text AS value", &parametre),
        eski(&mut eski_istemci, "SELECT $1::text AS value", &parametre)
    );
    yeni.eylem_baslat().unwrap();
    eski_istemci.batch_execute("BEGIN").unwrap();
    let sorgu = "SELECT repeat('x',1048576)::text AS payload FROM generate_series(1,64)";
    assert_eq!(yeni.oku(sorgu, &[]), eski(&mut eski_istemci, sorgu, &[]));
    assert_eq!(
        yeni.oku("SELECT 'ok'::text AS health", &[]),
        eski(&mut eski_istemci, "SELECT 'ok'::text AS health", &[])
    );
    yeni.eylem_geri_al().unwrap();
    eski_istemci.batch_execute("ROLLBACK").unwrap();
}
