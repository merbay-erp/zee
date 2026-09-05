# ADR-059 — Paket sahipliği ve core freeze

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **Revizyon:** 3 Eylül 2026 — K-160A/B-071 executable freeze beyanı eklendi.
- **İlgili kayıt:** K-160, K-160A, B-070, B-071

## Karar

- `paket_modeli`, paylaşılan davranışsız veri sözleşmesidir ve production
  modülüne bağımlı olamaz.
- `artefakt_dogrulama`, arşiv kurulumu, imza, SBOM ve provenance güveninin
  sahibidir; paket çözümü, registry taşıması veya yayın orkestrasyonunu çağıramaz.
- `registry`, protokol, HTTPS taşıma, metadata zinciri ve cache sahibidir;
  artefaktı verification'a devreder.
- `paket`, bağımlılık grafiği, exact sürüm ve kilit kararlarını sahiplenir.
- `yayin`, doğrulanmış proje grafiğini artefakt üretimine bağlayan üst seviye
  orkestrasyondur. Eski `tedarik` yolu yalnız geçici verification cephesidir.

Production SCC ve izin sayısı sıfırdır; C001 silinmiştir. Mimari test yasak
geri kenarları ayrıca korur.

## Core freeze

> Yeni compiler özelliği varsayılan olarak reddedilir; gerçek dogfood ihtiyacı
> kanıtlanmadıkça core genişletilmez.

Dogfood bugfix ile security/correctness işleri kabul edilir. API kırılması
gerçek ürün kanıtı, grammar değişikliği çok yüksek eşik ister. Ürün ihtiyacı →
minimal değişiklik → regression → ADR/spec zinciri kurulmadan merge edilmez.

K-160A bu politikayı [`core-freeze-korugu.sh`](../scripts/core-freeze-korugu.sh)
ile yürütülebilir yaptı. Her yeni compiler kaynak commit'i freeze sınıfını
exact SHA ile bildirir. Semantic değişiklik yalnız dogfood/security/correctness
olabilir; dogfood sınıfı ürün, K-işi, reproducer, etkilenen proje, minimalite
ve normatif kararın tamamını ister. Eski beyan tabana göre yeniden yazılamaz.

Eski `tedarik` cephesinin production tüketicisi yoktur; yalnız 1.0 öncesi iç
uyumluluk adıdır. Silme/deprecation kararı K-167 compatibility policy içinde
verilecek, bu P2 borç yeni compiler çalışmasını veya dogfood'u öne çekmez.

> Güncelleme (K-167, 5 Eylül 2026): cephe ADR-064 ile DEP-004 kaydı altında
> kaldırıldı; kurulum katmanı `artefakt_dogrulama/kurulum.rs` altındadır.
