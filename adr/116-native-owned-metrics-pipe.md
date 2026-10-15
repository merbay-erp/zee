# ADR-116 — Native sahipli sınırlı metrik pipe

- **Durum:** kabul, sınırlı native metadata profili
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-257, GB-078

Kaynak entegrasyonu olumlu/olumsuz testleriyle bu dalgadadır; final exact
commit kapıları ve soak kabulü ayrı kayıt gerektirir.

Ürün: zee-cekirdek P13/B5. Mevcut API handler/request status süresi ile actual PostgreSQL pool busy/idle/wait sahiplerine erişmiyor. Proxy probe ölçümü bu sahipleri karşılamaz. Minimal host sözleşmesi native runtime'ın fixed counters snapshot'ını host'un açıkça devrettiği inherited nonblocking FIFO'ya yollar. Yeni dil API/grammar/endpoint açılmaz; desteklenen Rust v1 facade değişmez.

Karar: native request owner final attempted status ve final transport-write outcome'u ayrı sayar. Havuz HandleEvent callback'i yalnız sayaç; actual state() callback dışında örneklenir. Snapshot512byte, tek atomic-size nonblocking write, bounded arrays/cardinality. CLI FD preflight private kaynak/config diagnostic öncesinde stdio alias'ı reddeder, ownership constructor tekrar doğrular. SIGPIPE disposition kurulmaz. Host bounded consumer two workers/restarts için aggregate üretir. Telemetry best effort ve payload content-free'dir; client delivery, continuously fresh gauges veya tüm P13 kabulü değildir.

Risk: explicit FD yanlış channel olabilir; dev/inode stdio alias, type/mode/owner/SIGPIPE checks fail closed. inherited ownership caller'ın aynı descriptor'u kullanmamasını gerektirir. PIPE_BUF boyutu doğrulanır; partialwrite bozulması kanalı kapatır. Backpressure request latency'yi pipe retry ile uzatamaz. Native cumulative saturation ve son snapshot kaybı sayaçları eksik bırakabilir, transport audit değildir. K253 signalstate aynı absolute read/write deadline ve graceful drain sözleşmesi korunur. Yeni dependency yok; küçük private libc unsafe sınırı docs/native-metrics.md'dedir.

Credential launcher kompozisyonu [ADR-119](119-credential-native-metrics-capability.md) ile tek designated write-only FD capability kullanır. Önceki typed rejection sınırı K-260 ile kapandı; genel descriptor veya credential yetkisi açılmaz. Kaynak/reproducer/olumlu-olumsuz test/spec/freeze beyanı aynı atomik root diliminde kapanır; finalexactSHA soak1800 zorunludur.
