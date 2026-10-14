# K-253/B5 anonim native nazik kapanış reproducer'ı

Harici ürün exact commit'i `cd5d12f0008620819e6167af5146e83503fd2b38`.
Talep `docs/dil-talepleri/2026-10-14-zee-urunlesme.md`, B5, satır99–106.
UTF-8 NFC, `splitlines`, LF ile birleştirme ve son LF normalizasyonuyla
kesit SHA-256 `0466b376c023f719b7788fae3655226577b490de1289c1010337f10a67422971`.
Etkilenen harici native listener kaynağı `cekirdek/kaynak/ana.dil`, byte SHA-256
`f0a65a0b1272a717728db4050fd450fd85900434ed6d5309bdcb9bbf9f138619`.
Özel ürün kaynak gövdesi, etiket, URL veya credential kopyalanmaz.

Anonim kaynak gerçek B5 SIGTERM'de nazik kapanış ihtiyacını tek kabul edilmiş
GET handler'a indirger. `entered` yalnız test entry token'ıdır, production
request logu değildir. Bekleme mevcut handler süresini görünür kılar.
Önceki açık kesinti sınırı yeni davranışla genişletilir; semantic bugfix
veya eski davranışa regression provenance icat edilmez.

Normal istek üç saniyede200/completed verir. SIGTERM entry sonrasında
verildiğinde yeni native sözleşme aynı200yanıtını bitirir ve exit0 olur.
TERM'den sonra sıradaki handler kabul edilmez; idle listener sınırlı poll
ile kapanır. Kuyruktaki/backlog bütün istekleri drain etme, socket devri,
zero downtime veya P14 kabulü yoktur. Gerçek ürün adoption ve requirement
ile reproducer eşliği insan review'unda ayrıca doğrulanır.

K-164 sürtünme kataloğu karşılaştırıldı: mevcut kaynakta yeni grammar, sıra
erişimi veya generic FFI ihtiyacı yoktur; B5 native lifecycle ihtiyacı
CLI adaptörünün sahipli sinyal davranışına indirgenir.
