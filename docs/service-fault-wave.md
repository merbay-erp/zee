# K-292 — Tek native worker kaybı ve stale readiness sınırı

P34'ün genel fault matrisi kapanmaz. [K-283](http-service.md) whole-unit SIGKILL
vakası tek worker'ın kaybını veya native DNS iptalini kanıtlamaz. Bu ayrı dilim
[ADR-150](../adr/150-native-worker-fault-ve-kontrollu-resolver.md) ile iki gerçek
native vakayı mevcut Supervisor'ın **aynı process içindeki own Popen handle'ları**
üzerinde sınar; PID keşfi, ps/proc adoption veya global resolver değişikliği yoktur.

```sh
python3 -I -B scripts/tests/service-fault-native.py \
  --repo /owned/exact-clean-clone --sha <full-SHA> \
  --dil /owned/sameSHA/dil --binary-sha256 <actual-SHA256> \
  --out /owned/private-parent/fresh-output
```

Exact temiz kaynak/dirtyfalse ikili, Unix/WNOWAIT ve owned0700 output parent
zorunludur. İki gerçek native worker health200+fresh application gate gösterdikten
sonra yalnız listede kayıtlı bir worker'a SIGKILL verilir. Canonical live kontrolü
worker_exit bulmalı, sibling dahil tüm owned grupları reap etmeli ve portları
bırakmalıdır. Ayrı foreign sentinel canlı kalmalıdır; sentinel cleanup sonunda
yalnız ilk issued handle ile kapatılır. Bu component vakası systemd/cgroup kabulü
değildir; outer source process owner'ı zaten bütün native handles'ı tutar.

İkinci vakada yeni liveness-only iki worker gerçek HTTP200 verir fakat owner
oluşmadan önce yazılmış true gate'i kullanamaz. Startup reddi, zero ready=true,
healthy_workers=2, eski dosyanın byte/mtime değişmemesi ve owned cleanup birlikte
istenir. Sahte false dosya, hiç çalışmamış HTTP veya eski generation'ın cevabı
bu oracle'ın yerine geçmez. Kaynak/helper/generated fixture/binary pinleri
başlangıç/sonda kaydedilir; private body/credential/argv makbuza yazılmaz.

## Kontrollü resolver-call stall: ayrı hazırlık sınırı

Normal hazırlık7, fresh owned Python child içinde `socket.getaddrinfo` bağımlılığını
bilerek stall eden **test-only** fixture kullanır. Gerçek timeout ve SIGTERM flag
iptali DB helper'ın child kill/wait/pipe-close zincirini sınar; marker'a gerçekten
resolver çağrı noktasında ulaşıldığı gözlenir. Global /etc/resolv.conf, DNS service,
LD_PRELOAD, kernel ayarı, yeni dependency veya production resolver değişmez.
Cancellation-before-Popen ve selector setup fault da gerçek kısa owned child ile
ayrı kontrol edilir. Unknown CLI fixed masked exit2'dir.

Bu fixture native libc DNS veya gerçek UDP/TCP DNS yanıtı değildir. Özellikle
`compiler/src/eposta/mime.rs` sistem DNS'inin iptal edilemediğini açıkça belirtir;
Python stub kanıtıyla bu kapasite kapatılmaz. Native resolver/outer-process DNS
fault kabulü ayrı açık kalır. Bu dilim yeni compiler özelliği veya keyfi resolver
hook'u açmaz; full fault/RSS/provider/retry güveni iddiası yoktur.

Native2 henüz çalıştırılmadı; hazırlık7 başarıları native vaka sayısı değildir.
Caller180s external owned watchdog uygulamalıdır; component log/probe turlarında
120s cooperative deadline vardır; root review,
yeni clean kaynak ve aynı SHA ikili olmadan gerçek kampanya dispatch edilmez.

Özel fixture BuildJSON kontrolü yalnız sahipli ve bütçeli komut adaptöründen
geçer; genel helper değiştirilmez. Kaynak listesi exact SHA ile doğrulanır;
kısmi load sayaçları bir oracle başarısız olduğunda da korunur.
