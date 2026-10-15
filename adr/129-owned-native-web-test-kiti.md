# ADR-129 — Owned native web test kiti (K-271/P22)

- **Durum:** kabul; dar Unix fixture kiti; ortak Cargo/final temiz kaynak kabulü ayrıca açık
- **Tarih:** 15 Ekim 2026
- **İlgili:** ADR-118/125/127; GB-087

P22/C5 mevcut internal Rust ToplayanIo testlerinden geliştiricinin gerçek CLI/HTTP projesine taşınabilen dar test-kit eksikliğidir. JSON scenario native subprocess, named private cookie jar, gerçek HTML CSRF capture, exact public body snapshot ve named SQL observer tanımlar. Yeni grammar/compiler/api::v1 HTTP injection açılmaz. ToplayanIo supported public facade değildir. Python/Rust scenario başarılarından Zee source/branch coverage çıkarılmaz.

Operator explicit compiler/build/SHA ve PG16.15 binary rootunu seçer; manifest executable/argv/shell/DSN/inline SQL seçmez. Fixture kökü trusted source'dur, sanal güvenilmeyen kod sandbox'ı değildir. Trusted pinned SQL fixture owner rolünde yürür; psql meta reddedilir ve server rolü superuser/CREATEROLE/CREATEDB değildir. App DML, bağımsız observer SELECT-only/default read-only; her vaka yeni owned PG data/system_identifier ve process group alır. Var olan loopback sslmode=disable geliştirme profili yalnız disposable local fixture içindir; P31 TLS/production kabulü diye gösterilmez.

COMMIT aynı vaka içinde kalıcıdır; başarısız SQL eylemi/savepoint geri alınır ve rota Sonuç/Hata'yı açık503'e çevirir. Farklı read-only role/session committed=1 ve failedwrite=0 görür. Sonraki kümenin baseline0'ı vaka izolasyonudur; HTTP istekleri üstünde dış rollback iddiası değildir. Dosya/session izolasyonu yeni proje kopyası ve cookie jar'dır.

Mevcut P23 worker-port/Supervisor route kullanılır; WNOWAIT leader grup cleanup bitene kadar tutulur. Serbest port seçimiyle bind arasında yarış başarısız koşudur, yabancı servis/PID edinilmez. HTTP Build kimliği exact beklenen ikiliyle eşlenir; native Host'un app receipt şartı gevşetilmez ve generic receipt üretilmez. Uygulama kaynak bytes değiştirilmez; yalnız DB fixture proje bildirimindeki tek sabit endpoint owned port'a bağlanır ve oluşan bildirim hash'i ayrı makbuzdadır.

Inventory/JSON/cases/steps/HTTP/process output ve monotonic süreler sınırlıdır. Cookie/CSRF/password/PHC/DSN/raw body ve private stderr makbuzda yer almaz; hata reason sabit alanlardır. Signal handler yalnız cancel flag kurar, owned cleanup finally üzerinden işler. POSIX owned süreç ve yerel PG kabulü Windows/minOS/canlı adoption değildir. P22 coverage, genel public HTTP injection, P08 gerçek anonim tam ürün korpusu ve P23 yeni kullanıcı sınırları açık kalır.

İlk Python aday incelemesinde generic argparse hata yolu argümanları echo edebiliyordu; ilk kaynak öncesi MaskedParser fixed ret ve actual unknown-argument canary alt-senaryosuyla kapatıldı. Son cleanup başarısızlığı her durumda failed makbuz üretir. Bu yeni host aracının aday düzeltmesidir; tarihsel compiler bug/regression SHA uydurulmaz.

Bağımsız ilk aday incelemesi suite hashinin sınırlı JSON okumasından önce FIFO üzerinde bloklanabildiğini buldu. İlk kaynak öncesi R1 düzeltmesi nofollow normal-dosya/64KiB snapshotını hem hash hem decode için aynı baytlara bağlar; identity başlangıç/FD/bitişte doğrulanır. Binary/PG executable hashleri64MiB, runtime yardımcı hashleri1MiB rol sınırında streamingdir. Global cancel/deadline preflight öncesindedir. Gerçek FIFO/oversize/symlink/dizin negatifleri dosyayı açmadan masked exit2 ve yeni çıktı oluşturmamayı doğrular; bu preflight kanıtı native uygulama kabulü değildir. Tarihsel compiler fixed_by/introduced_by kaydı uydurulmaz.
