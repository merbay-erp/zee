# ADR-058 — Sürümlü public API facade'ı

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-159, B-069

## Karar

Rust gömme sözleşmesi `dil::api::v1` altında exact allowlist'tir. Kök legacy
işlevler ve modüller 1.0 öncesi uyumluluk için şimdilik erişilebilir; modüller
`#[doc(hidden)]` ile internal sınıflanır ve SemVer sözü sayılmaz. Test kapısı
yeni facade ihracını, açık internal işaret olmadan kök modülü ve çalışmayan
derleme/çalıştırma/tanı/biçim akışını reddeder.

## Sonuç

K-160 paket sahipliği refactor'u internal yolları public söz sanmadan
değiştirebilir. V1 tüketicisinin desteklenen yolu ve breaking-change sınırı
makinece görünürdür.
