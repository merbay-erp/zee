# ADR-112 — Uygulama sahipli DB ve göç hazırlık probu

- **Durum:** kabul, dar host yardımcısı
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-251/P13/B5; GB-073

## Bağlam ve karar

Exact ürün talebi `cd5d12f0008620819e6167af5146e83503fd2b38` içindeki B5
readiness gereksinimini belirtir; talep provenance/hash ortak ürünleşme
kaydındadır. K-246'nın native başlangıç dosyası DB/göç hazırlığı değildir.
Yeni compiler özelliği açılmadan mevcut `göçür --durum` ve `çalıştır` CLI'ları
uygulama sahipli tek seferlik helper ile birleştirilir.

Göç makbuzu ve checksum'ların tamamı uygulanmış olmalı; ayrıca uygulamanın
sınırlı actual schema/revision SELECT probu sabit başarı token'ı vermelidir.
Role readonly yetkisi uygulama/DB operatörünün sorumluluğudur. Native kanıt
ayrı admin ve readonly role ile yalnız owned anonim loopback cluster kurar.
Helper güvenilir proje+contract/binary snapshot'ı, exact temiz build kimliği,
monoton ortak deadline, sınırlı pipe, kill/wait ve private atomic gate ister.
Geçerli gate edinildiğinde önce false; üç kontrol geçince true yazılır.

## Kabul ve güvenlik sınırı

Native13 vaka gerçek PostgreSQL16.15 stop/restore, pending/checksum/schema/
revision drift, iki native worker ve HUP ile liveness200/readiness ayrımını
kanıtlar. Strict10 vaka gate ownership/lock/parent değişimi ve child süre/
çıktı sınırını kanıtlar. Raw DB/child hata ve connection URL log'a çıkmaz.

Gate edinilemeyen çağrı eski dosyayı değiştiremez. Helper once çalışır;
DB değişimi/stop/recovery ve worker başlangıç/HUP sonrasında yeniden çağrı
zorunluluğu operatör sözleşmesidir. Sürekli izleme/TTL/SIGKILL invalidation
ve bütün DDL eşliği yoktur; eski başarılı dosya yeniden prob edilene kadar
kalabilir. Request/DB metrics, drain/reload, socket handoff/overlap,
sıfır kesinti ve gerçek ürün/platform adoption'ı açık kalır.

## Etki incelemesi

Spec/RFC/compiler grammar/tanı/public API/ABI veya production Rust bağımlılık
ve katman/SCC değişmez; core freeze genişletilmez. Yeni JSON host sözleşmesi
production dil yüzeyi değildir. Host docs/phase/kanıt/backlog/güvenlik/sürüm
ve README durumları toplu committe birlikte incelenir. Sabit benchmark veya
uzun ömürlü compiler state eklenmediğinden ADR-045/049/046 ve soak/RC geçmişi
bu adayla geçmiş kabul gibi değiştirilmez. Final exact temiz source native
kampanyası ayrı kayıt ister.
