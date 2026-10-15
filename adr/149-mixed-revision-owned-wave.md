# ADR-149 — K-291 mixed-code revision için ayrı gerçek kanıt

- **Durum:** test/host doğrulama adayı; yeni native kampanya henüz yürütülmedi.
P14/B6'da inherited listener17 sabit application revision üzerinde kanıttı;
mixed-code yayınını veya evrensel kesintisizlik/SLA'yı kanıtlamıyordu.

Compiler/FD protocol genişletilmez. Yeni dedicated koşucu aynı owned project/root
ve atomic shared JSON state üzerinde eski loaded AST ve yeni source hash/revision
ile gerçek native process'leri birlikte yürütür. Yayın trusted anonymous fixture
source değişimidir, production artifact/DB schema deployment API'si değildir.
Ready/ACTIVATE fresh nonce korunur. Old admitted handler entry, new marker yanıtı,
TERM sonrası old exact body ve yeni generation'ın state/cookie/revoke/quota
koruması ayrı oracledır. Dört load wave/160istek/concurrency2 sonuçları request
retry veya quota reset/raise ile düzeltilmez. İlave state rollover ve mevcut
single-generation deadline kontrolü load sayısına katılmaz.

[Rehber](../docs/mixed-revision-wave.md), hazırlık8 ve dedicated native9 aynı
kaynak dilimindedir. Eski sabit-revision17 yeni kabul yerine geçmez. GB-107,
backlog, faz, uyumluluk ve makine planı ortak kayıtları güncellendi; compiler source/cache/API/grammar veya core soak semantiği
bu test diliminde değişmez. P14 gerçek ürün/şema/platform ve genel SLA açıktır.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.
