# ADR-117 — Sınırlı SQL dispatcher sahipliği ve log basıncı

- **Durum:** kabul, sınırlı SQL-only host pilotu
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-258, GB-079

Ürün talebi B1. Tam P09 kapanışı değildir.

Mevcut K246 süreç grubu sahibi doğrudan kullanılır; WNOWAIT leader PID yetkisini
alt grubun kapatılmasına kadar korur. Başarısız leader çıkışı, stdout'u açık
bırakan alt süreç bulunsa da heartbeat veya yeni işi sürdürmez. Yalnız verilen
Popen handles kapatılır; yabancı port/PID benimsenmez.

Mevcut K245 credential okuyucusu yalnız açık eşlenmiş değerleri, özel dosya kökü
ve nofollow denetimiyle aktarır. CA PEM ayrı sınırlı güven yapılandırmasıdır.
Log yazımı mevcut K246 PIPE_BUF sınırı ve nonblocking writer ile atomik kayıttır;
dolu pipe kaydı düşürür ve sayaçla ölçer; SQL ilerlemesini durdurmaz.

Handler yalnız mevcut SQL niyet tablosuna sabit sorgu ve veri yazar. Belirsiz
COMMIT/sinyal/timeout sonrası replay veya fail yazımı yapılmaz. Bilinen rollback
sonucu üç deneme/deadletter sınırına gider. 1..100 tick pilotu daemon/cron değildir.

Gerçek11 vaka native-proof.json içinde; final temiz kaynak kabulü ve gerçek ürün
handler benimsemesi açık kalır. Spec/RFC/ADR indeksleri, sürümler, güvenlik defteri,
backlog, korpus ve faz kaydı ortak entegrasyon dalgasında root sorumluluğundadır.

Review revision2: normal argparse invalidargument leaked literal argv to stderr;
MaskedParser.error now routes to fixed rejected JSON/exit2, disables abbreviations.
Actual --ticks canary and unknownoption canary both stderr0/noargvusage/oneJSON.
Review1 historical manifests/source/proof retained; new exact source native11 passed.
