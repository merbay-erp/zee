type OturumSatirlari = Vec<Vec<(String, String)>>;

pub(super) fn sonuc_yaz(sonuc: &Result<OturumSatirlari, String>) -> Vec<String> {
    match sonuc {
        Ok(satirlar) => match json_yaz(satirlar) {
            Some(json) => vec!["ok".into(), json],
            None => vec!["hata".into(), "web oturum listesi JSON'a yazılamadı".into()],
        },
        Err(hata) => vec!["hata".into(), hata.clone()],
    }
}

pub(super) fn sonuc_coz(alanlar: &[String]) -> Result<Result<OturumSatirlari, String>, String> {
    match alanlar {
        [etiket, json] if etiket == "ok" => json_coz(json)
            .map(Ok)
            .ok_or_else(|| "web oturum listesi sonucu geçerli JSON değil".into()),
        [etiket, hata] if etiket == "hata" => Ok(Err(hata.clone())),
        _ => Err("web oturum listesi sonuç şeması geçersiz".into()),
    }
}

// Oturum listesi yalnız yerel web çalışma zamanında oluşur; wasm playground çekirdeğinde
// serde_json bağımlılığı yoktur (Cargo.toml hedef koşulu), iz orada hata olarak okunur.
#[cfg(not(target_arch = "wasm32"))]
fn json_yaz(satirlar: &OturumSatirlari) -> Option<String> {
    serde_json::to_string(satirlar).ok()
}

#[cfg(target_arch = "wasm32")]
fn json_yaz(_: &OturumSatirlari) -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn json_coz(json: &str) -> Option<OturumSatirlari> {
    serde_json::from_str(json).ok()
}

#[cfg(target_arch = "wasm32")]
fn json_coz(_: &str) -> Option<OturumSatirlari> {
    None
}
