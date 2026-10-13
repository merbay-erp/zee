//! Bounded canonical client identity trace schema (K-209).
use super::{alan_sayisi, IzOlay};

pub(super) fn semayi_denetle(olay: &IzOlay) -> Result<(), String> {
    alan_sayisi(olay, 0, 1)?;
    let value = &olay.sonuc[0];
    if value.is_empty()
        || value
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.to_string() == *value)
    {
        Ok(())
    } else {
        Err("istemci adresi boş veya kanonik IP olmalı".into())
    }
}

pub(super) fn kaydet<T: super::GirdiCikti>(io: &mut super::IzKaydedenIo<T>) -> String {
    let value = io.ic.istemci_adresi();
    io.kaydet("istemci_adresi", Vec::new(), vec![value.clone()]);
    value
}

pub(super) fn yeniden_oynat(io: &mut super::IzYenidenOynatici) -> String {
    io.siradaki("istemci_adresi", Vec::new())
        .and_then(|v| v.first().cloned())
        .unwrap_or_default()
}
