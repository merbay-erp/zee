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

## K-309 — job dispatcher harness cleanup ve özgün PG sahibi: açık

K-309 / GB-118 açık: Job SQL dispatcher native kabul harness’inin cleanup kısa devresi ve özgün foreground PostgreSQL sahipliği için dar correctness dilimi hazırlanır. Exact child-free AST karşı örneğinde fake probe.close hatası cluster.close ve proof yazımı denemesini atladı; tek beklenen failure/exit1/reaped kaydı vardır. Gerçek değişmeyen ordinary mask baseline2 ayrı koşuda2test/0.099sn PASS/exit0/reaped verdi; yeni focused/combined/native kabulü değildir. Bu, gerçek OS/PG sızıntısı veya diskte kalmış PASS gözlemi değildir. Ayrı standalone44 ve combined3 kontrollü koşuları PASS/exit0/reaped verdi; tam kaynak15, temiz exact kaynak, fresh mevcut native11 ve final kabulü bekler; job dahil üç PG tüketicisi ve genel P09 kısmi kalır.

Kaynak/test/bu OPEN belgeler tek atomik dilim kapsamındadır; kontrollü hazırlık kendi başına tam kaynak veya native kabulü değildir. Kaynak incelemesinin doğruladığı dar düzeltme; ayrı yapışkan iptal/failure kaydı, cleanup maskesi ve garantili restore, kısmi handler/callback kurulumu ve ilk hata nesnesinin korunması, bağımsız cleanup callback denemeleri ve cleanup sonrası publication guard’ıdır. Gerçek probe/cluster/child/proxy nesneleri yalnız kendi özgün sahiplikleriyle kapatılmalıdır; yabancı PID/port/root benimsenmez. Shared `native-pg-owner.py` helper’ı exact6e5f baytlarında kalır; job adapter’ının TLS ve özgün foreground PG/readiness uyumu kaynak+inert model incelemesinde doğrulandı. Gerçek standalone44/combined3 kontrollü sonuçları ayrı kayıttadır; native11 kabulü değildir.

**Kaynak ve kontrollü hazırlık:** Proposal3 dört harness/test kaynağı bağımsız static incelemede doğrulandı: korumalı handler kurulumu/restore, sticky iptal/ilk hata, bağımsız cleanup ve publication, özgün foreground PG/readiness adapter’ı. Mevcut11native vaka/32require assertion/35s heartbeat döngüsü ve TLSrequire/scopedCOMMIT ayrımı korundu. Bu kaynak/test/OPEN belge dilimi aynı atomik uygulama kapsamındadır; tam kaynak veya native kabul sonucu değildir. Gerçek standalone preflight44 (genericPG18+job-specific26) PASS/exit0/reaped, unittest0.331sn; yalnız inert sahiplik/cleanup/iptal/publication modelleridir, gerçek PG/native11 değildir. Değişmeyen mask baseline2/.099sn ayrı tarihsel kabul olarak korunur. Gerçek fixture2 combined ordinary3 PASS/exit0/reaped, unittest0.511sn: eski mask2+tek nested44 wrapper. İlk fixture1 koşusu3test/2maskfailure/exit1/reaped kaldı; root üç public production import bağımlılığını fixture’a ekledi, Proposal3 baytları değişmedi. Bu eksik fixture hazırlığı kaynak/native kusuru veya OS/PG sızıntısı değildir.44/3 sayıları toplanıp Cargo/native vaka yapılmaz. Exact Proposal3 `/private/tmp/zee-k300/k309-job-proposal-3/PREPARATION.json` SHA256 `2096a3744a2989830b6833748b7382dfd9f290a77fcd793f0139a2df250e6cbe`; static peer `29eed668c09d318f258d78a167df9525c6cc50a9af41161465ac475d91ee3070`; actual controlled peer `22ee3e73afa325c3aa2b5caacad725ac32d5f859ef42b6cf0c70029296268559`; fixture2 `a7c866f4914d02db9b17b4b118a66bee6e9d78c48ac4b16844c84c3b33b8e309`. Dört source pini: `scripts/tests/job-dispatcher-native-test.py`=`ef6a3ab8f990c3148a09d3234fe8b8075b9fd12083742059139df2599be5f49d`, `scripts/tests/job-dispatcher-mask-test.py`=`92357774ae9a09d55a366d844a33711564df5a924c237bdb9051135529a4b21a`, `scripts/tests/job-pg-owner-preflight-test.py`=`158af9ca7f2a97d4499c36c3af5ed5befc73380195da98bdf30d1a76b80d630f`, `compiler/tests/job_dispatcher_testi.rs`=`bda79d9307573127ee62eff6931b5dad363bbf4f2b6c3c73052d00c5f8d5a3d1`. Baseline/odaklı saf modeller/ordinary wrapper/native11 ayrı kanıt kapsamlarıdır; Python yöntemleri Cargo veya native vaka toplamına eklenmez.

**Korunan native sözleşme:** Mevcut11 vaka ve assertion’lar, gerçek35s sabit handler sırasında bağımsız lease heartbeat/yenilemeler ve eski30s süresinin aşılması korunacaktır. CLI/gerçek PG yolu TLSrequire ve owned CA/credential sınırında kalır; sslmode=disable yalnız mevcut owned loopback host-component COMMIT belirsizlik fault fixture’ına aittir, varsayılan CLI profili değildir. Belirsiz COMMIT/sinyal/timeout sonrası replay veya fail yazımı yoktur; SQL durable-intent, no-replay ve unknown sınırı genişletilmez. K306 dispatcher4/HTTP7, K307 SQL9, K308 starter14 ve tarihsel job11 sonuçları yeni K309 kaynağına aktarılmaz.

**Açık kabul zinciri:** actual kaynak+test+OPEN docs aynı batch, tam kaynak15 ve özgün owner makbuzları, temiz tested-source commit; ardından aynı clean SHA/tree üzerinde fresh canonical mevcut job native11, özgün PG/readiness/child/proxy ve dış owner yaşam döngüsü public kanıtı ile root/bağımsız peer; exact final belge overlay’inde final kapılar ve temiz kapanış commit’i gerekir. Kontrollü44/3 sonuçları kayıtlıdır; gelecekteki temiz SHA/tree, gerçek normal PG kapanışı, source15 veya native11 başarısı henüz bilinmiyor ve yazılmıyor. Native11’in35s/TLS/COMMIT oracleları gevşetilemez. Bu taslak GB118’i kapatmaz.

Job, K308 sonrası halen açık üç PG tüketicisinden biridir; ancak gerçek dar kapanışından sonra kalan metrics/effect iki tüketicisinin kabulü ayrı olacaktır. 39 plan statüsü mevcut değerlerinde korunur; P09/P13/P20/P23/P36 kısmi ve P37 sürekli statükosu değişmez. Ürün/adoption/provider/fiziksel platform/dış proje girdileri, RC/soak/IDE ve C1/K288/P33/K293 ret sınırları açık kalır. Yeni compiler özelliği veya katalog SMTP/PDF effect-service kabulü yoktur.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans ve compiler/LSP kalıcı durum değişmez. Ürün komutları/göç yolu değişmediğinden `docs/job-dispatcher.md`, ADR-114/126/139 ve mevcut kaynak-pinli rehberler bilinçli unchanged tutulur; ADR-117 dar harness güvenlik sınırını ve mevcut SQL-only pilot durumunu taşır. PHC/parola/cookie/request/cert/key/DSN/credential özel baytları veya hash’leri public kayda alınmaz; private runtime veya eski PID/kök aranmaz ve benimsenmez.
