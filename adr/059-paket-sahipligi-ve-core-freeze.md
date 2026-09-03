# ADR-059 — Paket sahipliği ve core freeze

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-160, B-070

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
