# ADR-123 — Kritik işlevlerde seçici sorumluluk tabanı

- **Durum:** kabul, K-264 dar bakım incelemesi
- **Tarih:** 15 Ekim 2026
- **İlgili karar:** ADR-041; K-252/K-253/K-257/K-259

ADR-041 giriş eşiği ve büyüme payları değişmez. Yeni kritik işlev veya pay aşımı
otomatik refactor ya da toplu taban yenileme değildir: sorumluluk, yetki sınırı
ve gerçek test kapsamı birlikte incelenir. [Beş işlev incelemesi](../docs/incelemeler/urunlesme-20261014/k264-function-review/README.md)
ve bağımsız kontrol, bu dalgada dört yeni kayıt ve bir güncellemeyi gerekçelendirir.

`gonder_ic` tek SMTP oturumunun bütçe ve sırasını; `govde` kaynak tanısından önce
capability doğrulama, config, mod seçimi ve owning constructor sırasını;
`istek_al` kabul/admission ve bounded HTTP okuma sınırını; `web_modunu_ayikla_temel`
eski proxy profilini; `web_listener::extract` dört borrowed rolün kayıt çözümünü
tek sorumluluk olarak tutar. Ayrı yardımcılar framing, config, transport,
listener install ve handler/transaction işlerini taşır. Bu inceleme yeni dil
semantiği veya private capability yetkisi açmaz.

Yalnız exact beş satır kabul edilir: 87/7, 150/17, 113/12, 90/4 ve 104/8.
Diğer satır değerleri, eski `web_modunu_ayikla#1` kaydı ve yüzde/min/max büyüme
payları korunur. Sonraki büyüme yeniden normal kapıya tabidir. Küçük CLI wrapper
ayrıştırması değerlendirildi; güvenlik sırasının diagnostic/return sahipliğini
çoğaltacağı için bu atomik dalgada zorunlu görülmedi.

Kanıt kapsamı sınırlandırılır: MIME pozitifleri legacy `gonder_ic` doğrudan
başarı kanıtı değildir. CLI profile parser'ın partial/repeated/nonnumeric/invalid
generation durumları için mevcut suites'te doğrudan test gösterilmedi; ACTIVATE
nonce negatifleri ayrı `Control.wait` sınırını doğrular. Parser doğrulamaları
kaynak incelemesindedir; actual FD/config/argv ve native başarılı profiller
ayrı kampanyalarda yürür. Yeni test varmış gibi kayıt açılmaz.

Manuel beş satır farkı sonrasında kanonik rapor üretildi ve gerçek
`islev_egilimi --denetle` 64 kritik kayıtla geçti. Tabanın geçmesi tek başına
inceleme kanıtı değildir. Temiz ortak kaynak, RC, soak ve ürün benimsemesi
bu bakım kararından kapanmaz.
