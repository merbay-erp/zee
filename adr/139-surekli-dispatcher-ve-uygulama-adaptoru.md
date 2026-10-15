# ADR-139 — K-282 sürekli dispatcher ve application-owned adapter

- **Durum:** uygulanmış aday; hazırlık testleri ve source review, yeni exact-clean Linux
kampanyasından ayrıdır. P09/B1'in genel daemon ve uygulama-owned effect consumer
sürtünmesi mevcut host araçları üzerinde karşılanır; compiler/grammar/API açılmaz.

K-268 SQL-only tick daemon içinde yeniden kullanılır; bilinen completion sonrası
monotonic aralık ve ayrı explicit OnCalendar usertimer eklenir. Lifetime service
kilidi bekleme aralarında da tektir. Unknown sonuçlar için kalıcı latch/restart
ret ve exit3 korunur. Bilinen completion sonrası görünür clear/fsync belirsizliği
exit4'tür; gizli rollback veya otomatik unknown retry yoktur.

Genel dış etkiyi sabit SMTP fixture'ı gibi gösterme yerine, ayrı opt-in pinlenmiş
application executable/argv/data/cwd ve strict bounded ack sözleşmesi kullanılır.
Adapter güvenilir uygulama kodudur, sandbox değildir. Host app ack'ini teslim,
SQL transaction veya exactly-once diye yeniden yorumlamaz. Provider protokolü ve
credential yetkisi uygulamaya aittir; host ambient sır veya arbitrary FD aktarmaz.
Durable guard Popen öncesindedir; nofollow/bounded snapshot, held-directory/proc
cwd, WNOWAIT cleanup ve fixed metadata sınırı korunur. Unknown guard için bu
sürüm otomatik/operator-asserted replay veya arbitrary deletion komutu sağlamaz.

[Rehber](../docs/dispatcher-service.md), host hazırlık21 ve ayrı rendered-unit/
SQL/calendar/adapter native koşucusu aynı dilimdedir. Yeni native kabul henüz
üretilmemiştir; eski finite/SMTP kanıtı yeni source'a mal edilmez. GB-097 kaydı,
backlog, plan, faz ve uyumluluk kayıtları aynı kaynak diliminde güncellendi.
Kalıcı benchmark veya compiler/LSP persistent buffer/cache değişmediği için bu
host işi tek başına core soak tetiklemez; bütün kaynak kapıları yine korunur.
