type OturumSatirlari = Vec<Vec<(String, String)>>;

pub(super) fn sonuc_yaz(sonuc: &Result<OturumSatirlari, String>) -> Vec<String> {
    match sonuc {
        Ok(satirlar) => vec![
            "ok".into(),
            serde_json::to_string(satirlar).expect("metin oturum satırları JSON'a yazılabilir"),
        ],
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

pub(super) fn sonuc_coz(alanlar: &[String]) -> Result<Result<OturumSatirlari, String>, String> {
    match alanlar {
        [etiket, json] if etiket == "ok" => serde_json::from_str(json)
            .map(Ok)
            .map_err(|_| "web oturum listesi sonucu geçerli JSON değil".into()),
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        _ => Err("web oturum listesi sonuç şeması geçersiz".into()),
    }
}
