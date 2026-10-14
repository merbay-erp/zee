# ADR-114 — Ayrı SQL kira sahibi heartbeat'i

- **Durum:** kabul, dar SQL-only heartbeat pilotu
- **Tarih:** 15 Ekim 2026
- Ürün kanıtı: zee-cekirdek cd5d12f B1; mevcut K-247 exact katalog hash zinciri
- Önceki karar: ADR-107; C027 sınırı ADR-103

SQL handler jobs satırını transaction boyunca kilitler. Aynı satırdaki lease
expiry'yi heartbeat'in güncellemesi kilitte bekler. Ayrı zee_job_leases satırı
lease'nin generation/token/owner, server expiry ve renewal cap bilgisinin
sahibidir; jobs satırı yaşam döngüsü ve SQL handler fencing sahibi kalır.

Yeni claim capacity→job→lease sırasıyla aynı eylem transaction'ında yürür.
Heartbeat yalnız lease satırını günceller, job durumunu MVCC salt okur; handler
job satırını tutup lease'yi salt okur. Reaper expired job satırlarını
FOR UPDATE OF job SKIP LOCKED seçer. Doğru token/owner/generation ve henüz
geçerli expiry olmadan yenileme yoktur. Lease 30s, toplam server-clock tavanı
120s ve en çok24 yenilemedir. Var olan migration0001 değişmez;0002 eski
lease'lerin ömrünü uzatmadan owner satırlarına geçirir. Karma v1/v2 worker
kullanımı kabul edilmez; eski job expiry alanı v2'de yetki kaynağı değildir.

Upsert yalnız yeni claim generation'ı için yapılır. Önceki reserved eski
satırın token'ı tekrar kullanılsa bile o satırın geçmiş expiry'si canlandırılmaz.
Dış ilk aday bunu sağlayamıyordu; actual before2live/revived→after1live/expired
kontrolü korundu. Bu dış aday düzeltmesi shipped compiler bug değildir.

Dış actual PostgreSQL16.15/Rust1.93.1 adayında11mevcut+10yeni=21vaka geçti.
35.050s job-lock altındaki handler boyunca7 heartbeat commit; ilk30s deadline
sonrasında geçerli kira, bir kalıcı etki/done görüldü. Doğal30s expired kilitli
handler reaper'ı bloke etmedi;SIGKILL/barrierrelease sonrası backend gerçekten
sona erince reaper1 ve fresh generation geldi. Eski heartbeat/handler reddi,
attempt2/effect1,3attempt/deadletter, heartbeat COMMIT before/after C027 ve
independent renewal0/1 observer kanıtları vardır. Bu dirty source kimlikli
dış aşamadır; clean exact acceptance değildir.

Completion yetkisi statement anındaki lease kontrolüdür; COMMIT-time clock
ve genel hard wall-clock sözü eklenmez. MVCC snapshot'ıyla completion yarışında
heartbeat1dönüp lease satırı uzayabilir; donejob'a yeni SQL yetkisi/slot verilmez.
Expired SQL handler hâlen process olarak yaşayabilir; kira sayacı fiziksel
process concurrency veya non-idempotent dış IO garantisi değildir. PostgreSQL
blocked query'nin TLS disconnect'i görmesi barrierrelease bekleyebilir;
backendgone görülmeden genel rollback sonucu çıkarılmaz.

C027 sonrası heartbeat/claim/handler yazısı tekrar edilmez. Yeni bağlantıda
salt reconciliation uygulanır. SMTP/PDF/HTTP external effects, scheduled cron,
calendar/DST ve gerçek katalog Python kaldırma kabulü yoktur. Operasyonel
sabit dispatcher bu adayda hâlen uygulanmamıştır. Compiler/grammar/public API,
production dependency graph ve toolchain değişmez. GB-075 ayrı kira sahibi, MVCC/completion ve C027 sınırını kaydeder.
