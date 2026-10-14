# P13/B5 gerçek kod sınırları — başlangıç incelemesi ve takip

Aşağıdaki tablo temiz `965c3a979bf39ad575898785deaa4496c3384713`
başlangıcının incelemesidir. K-253 native SIGTERM drain sonradan
`ba719497669823033c665a6bdceeebde3bac8f3e` kaynak commit'inde uygulandı;
exact dogfood-change beyanı `d42f14747006aa0c50335d3e7c8ea0a1026141a8`
takip commit'indedir. Mevcut admitted handler'ın tamamlanması ve idle
kapanış native testlerle kapsanır; son temiz soak kabulü ayrı gereklidir.
Tablodaki başlangıç eksikleri güncel compiler davranışı diye okunmaz.

| İhtiyaç | Mevcut API ile gerçek durum | Minimal sonraki iş |
|---|---|---|
| Request count/duration/final status | Native accept/ham ret, rate429, handler, transaction commit ve write farklı private CLI sahiplerinde. Handler wrapper yalnız giriş/başarılı gövde sayabilir; final commit/status/early429 eksik kalır. Dil `an_ms` iç IO'dur; public dil ifadesi değildir. | Private runtime observer ve route cardinality/secret-safe sink; prehandler ret, actual commit failure ve write-error testleriyle eklenmeli. Proxy metrikleri bunun yerine geçmez. |
| Rate reject | Oran kararı handler'a girmeden `main.rs::web_istegini_baslat` içinde429 üretir. | Native karar sahibinde sabit bounded counter gerektirir. |
| DB pool metrics | `postgresql/havuz.rs::PostgresqlHavuzu.ic` private r2d2 pool. Uygulama SQL'i checkout waiter/idle/busy'yi göremez; pg_stat_activity server connection sayısıdır. | Private pool state/checkout timing export; başarılı/timeout/yeniden-bağlantı actual PG testleri. |
| SIGTERM nazik kapanış | Default native signal handler inflight isteği keser; external helper yalnız bekleyerek yeni native accept'i güvenle kapatamaz. | K-253 dış private native guard+poll adayı; current request drain, bütün backlog/P14 değildir. |
| SIGHUP config yenileme | Mevcut supervisor sıralı restart'ta CLI `zee.ayar` yeniden okunur; aynı pid'de reload yoktur. | Ayrı real3-case candidate mevcut davranışı kanıtlar: kota20→5, fresh workerPID/readiness, bozuk security failclosed. Transactional old-config rollback/inprocess reload açık. |

Bu işler yalnız harici insan/platform kanıtıyla bloklu değildir. Metrics ve
full transactional reload için gerçek production kodu ve güvenlik/state
sözleşmesi eksiktir. K-253 gerçek B5 provenance ile minimal corefreeze
sınırını karşılar; bütün B5 clean-server15dk, P14 ve product adoption
kabulünü kapatmaz. Metrik ve socket devri adayları kendi kaynak, test ve
karar zinciri kabul edilene kadar bu incelemenin eksiklerini kapatmış sayılmaz.
