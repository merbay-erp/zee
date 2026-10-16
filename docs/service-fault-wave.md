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

Normal hazırlık10, fresh owned Python child içinde `socket.getaddrinfo` bağımlılığını
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

S3 native2 tarihsel kabulü aşağıda kayıtlıdır; K-299 sonrası yeni exact native2
beklenir. Hazırlık10 başarıları native vaka sayısı değildir. Component log/probe
turlarında120s cooperative deadline vardır. Dış özgün owner iş timeout
bütçesinden ayrı en az45s TERM kapanış payı ayırır; mevcut direct watchdog330s
kullanır. Kısa grace taşıyan ara H.run wrapper kullanılmaz. Yeni temiz kaynak ve
aynı SHA ikili gerçek kampanyanın ayrı önkoşuludur.

Özel fixture BuildJSON kontrolü yalnız sahipli ve bütçeli komut adaptöründen
geçer; genel helper değiştirilmez. Kaynak listesi exact SHA ile doğrulanır;
kısmi load sayaçları bir oracle başarısız olduğunda da korunur.

## K-296 — deduplikasyonlu state olayı yerine gerçek liveness oracle

S2 release kabulünde worker-loss vakası geçti; stale-gate vakası aynı ready=false
durumunda yeni state olayı beklediği için kırıldı. Supervisor state'i yalnız
readiness değişince yayımlar; üretim davranışı korunur. Native koşucu iki worker'ın
Build kimliği doğrulanmış HTTP200 liveness'ını doğrudan owned Supervisor probe
ile gözler. Startup reddi, hiç ready=true olayı olmaması, eski gate bayt/mtime
değişmezliği ve bütün owned handle kapanışları birlikte zorunlu kalır. Yalnız
test oracle düzeltilmiştir; native DNS veya production lifecycle genişlemez.
S3 temiz native kabulünde iki vaka geçti.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-299 — Erken hazırlık ve cleanup iptali

Campaign kurucusu child açmaz; prepare ilk owned kaynak sorgusu öncesi handler
kurar. Başarısız borrowed Fixture kurucusunun kendi temporary'si temizlenir.
Cleanup sırasında iptal bayrağı korunur ve failed receipt üretir; özgün sentinel
ve worker handle'ları dışında süreç aranmaz. Üretim Supervisor/native DNS
davranışı değişmez. Üç yeni kısa vaka ile hazırlık10 geçti; yeni native2 ve ortak
kapılar beklenir. [GB-111 ve kaynak/koşu kaydı](kabul-kosuculari-yasam-dongusu-20261016.md)
PDF9/service10 hazırlığını native9/2 kabulünden ayrı izler.


İlk C native1 denemesinde Rust wrapper’ın `compiler/..` depo yolu Campaign
kanonik yol koşulunu sağlamadı; service iç proof oluşmadan Cargo seçicisi
kaldı (libtest0,04 saniye, Cargo komutu0,629197 saniye). Üç saniyelik worker
hazırlık sınırı veya timeout değildir. Dar takip, Rust test wrapper `root()`
kanonikleştirmesidir; yeni Cargo owner veya worker davranışı eklenmez.
Birleşik native1 failed kalır; PDF9 ayrı C başarısı takip kaynağına
taşınmaz. Yeni temiz kaynakta ortak kapılar/native2 kabulü henüz yoktur.