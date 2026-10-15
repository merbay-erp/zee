# K-291 — Sahipli mixed-code revision dalgası

Bu araç generic deployer değildir. Aynı canonical anonymous proje/root, **tek**
parent-owned TCP listener ve aynı `.zee/web-durumu-v1.json` korunur. Eski worker
başlangıçta yüklenmiş programıyla çalışırken test-owned `ana.dil` atomik olarak
yeni source'a geçirilir. Host'un yalnız **yeni spawn** için expected source hash ve
application_revision alanları güncellenir; old process bellekteki eski AST'yi
kullanır. Native FD/ready/ACTIVATE ABI'si, default davranış veya compiler değişmez.
[ADR-149](../adr/149-mixed-revision-owned-wave.md).

Bu, runtime file-read davranışı/schema değişiklikleri/key rotation/DB migration
uyumluluğu veya evrensel production deployment işlemi değildir. Test aynı arayüzü
koruyan iki code revision davranışını gözler; farklı root'larla shared-state
varmış gibi yapmaz. Kaynak uygulama/data publication'ı güvenilir operator-owned
fixture işlemidir, hostile sameUID sandbox veya genel rollback transaction'ı
değildir. Hatalı yeni kaynakta eski byte'lar yalnız test-owned invalid source hash'i
hâlâ eşleşiyorsa geri yazılır; foreign/private proje üzerine yazılmaz.

```sh
python3 -I -B scripts/tests/mixed-revision-native.py \
  --repo /owned/exact-clean-clone --sha <full-SHA> \
  --dil /owned/sameSHA/dil --binary-sha256 <actual-SHA256> \
  --out /owned/private-parent/fresh-output
```

Unix/WNOWAIT, canonical fresh output ve current-owner0700 parent; exact temiz
repo ve dirtyfalse BuildJSON gerekir. Source/binary/runtime/helper pinleri başta
ve sonda karşılaştırılır. Caller360s external owned watchdog kullanmalıdır;
koşucunun cooperative300s süresi, request başına tek5s deadline'ı ve bounded
32KiB response/16KiB header sınırı vardır. Connect'ten sonra send öncesi deadline
tekrar kontrol edilir. HTTP retry yoktur. Source diagnostic/SQL/cookie/CSRF veya
business/event identity makbuza yazılmaz. Generated anonymous revision source
byte'ları ve hashleri dış output'ta korunur; bunlar ürün kodu değildir.

Dört load dalgası0→1→2→3→4'tür. Her dalgada yeni generation ready fakat henüz
aktif değilken eski worker'a `/slow` gider. Eski native stdout'unda gerçek
`entered` görülür; yeni ACTIVATE ve exact yeni revision yanıtı doğrulanır. Eski
istemcinin hâlâ inflight olduğu kontrol edilip yalnız owned Popen'a TERM verilir;
eski3s handler'ın exact eski body cevabı istemciye ulaşmalıdır. İki load istemcisinin ilk istekleri ACTIVATE öncesinde gerçek socket send ile
queued olduğunda yeni nesil aktive edilir; load TERM sınırı boyunca devam eder.
İki istemciyle40 istek/dalga, toplam160 istek; her status, body mismatch, refusal,
partial/framing ve deadline failure sayılır. Tek listener inode değişmez.
Load concurrency2 dışında ayrıca old admitted slow request ve bir new-revision
probe vardır; bunlar toplam eşzamanlı istek hesabından gizlenmez.
Her istek için farklı synthetic loopback client identity kullanılır; quota
artırılmaz veya state resetlenmez. Bu kontrollü kapasite sonucu saturation/SLA
veya evrensel sıfır kesinti sözü değildir. Başarısız koşunun sayacı da sonuçta
kalmalıdır; failed makbuz accepted diye sayılmaz.

Gerçek login/CSRF/cookie yeni code revision'larda çalışır. Logout/revoke ve
exhausted quota'nın ayrıca4→5 state kontrol revision'ında401/429 kaldığı gözlenir;
bu beşinci state rollover load içindeki dört rollover sayısına eklenmez.
Cookie/CSRF yalnız bellekte tutulur. Wrong expected apprevision ve wrongnonce
admission açmaz; current old revision kullanılabilir kalır. Bozuk yeni source
native-ready üretmez; host stderr DEVNULL'dur. Bu diagnostic suppression kontrolü
native early-quiet FD16 kapısının yerine geçmez. Aynı clean source ile mevcut
single-generation late-EOF/deadline1 kontrolü ayrıca çalışır; mixed-load isteği
olarak sayılmaz. Toplam9 ayrı vaka kaydı beklenir; bunlar Rust toplamına9 diye eklenmez.

Hazırlık8 actual short owned TCP/framing/refusal/snapshot ve expired-send
negatifidir; native compiler çalıştırmaz. Dedicated native9 bu dilimde henüz
çalıştırılmamıştır; eski S inheritedFD17/quiet16/deadline1 yeni mixedrevision
kabulü yerine konmaz. Yalnız root review ve yeni clean source/binary sonrasında
owned Linux/Mac koşusu yapılabilir. Listener ve tüm owned generation'lar finally
ile kapatılır; foreign sentinel yalnız kendi issued Popen yetkisiyle temizlenir.

`application_revision`ler anonymous fixture'a ait sentetik40hex etiketlerdir;
Git commit diye sunulmaz. Gerçek compiler/source S ayrı exact Git SHA, her
yayınlanan source'un gerçek SHA256'sı ayrıca kaydedilir.

Driver ham Cookie/CSRF değerini loglamaz. Native uygulamanın private `.zee`
runtime state'i synthetic oturum anahtarları/özetleri içerebilir; state korunması
bu testin amacı olsa da raw runtime dosyaları Git'e/evidence metadata alt kümesine
kopyalanmaz. Yalnız hash/fixed sonuçlar arşivlenir; output parent0700 kalır.
