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
negatifidir; native compiler çalıştırmaz. İlk hazırlıkta bekleyen dedicated
native9, tarihsel [S3 kaynağında](urunlesme-s3-kabul-20261015.md) dört dalga ve
160 istekle geçti. Eski inheritedFD17/quiet16/deadline1 bu mixed-revision
kabulünün yerine konmaz. S3 sonucu başka bir kaynakta yeniden koşulmuş sayılmaz;
yeni kabul root review ve exact temiz source/binary sonrasında sahipli
Linux/Mac koşusuna bağlanır. Listener ve tüm owned generation'lar finally
ile kapatılır; foreign sentinel yalnız kendi issued Popen yetkisiyle temizlenir.

`application_revision`ler anonymous fixture'a ait sentetik40hex etiketlerdir;
Git commit diye sunulmaz. Gerçek compiler/source S ayrı exact Git SHA, her
yayınlanan source'un gerçek SHA256'sı ayrıca kaydedilir.

Driver ham Cookie/CSRF değerini loglamaz. Native uygulamanın private `.zee`
runtime state'i synthetic oturum anahtarları/özetleri içerebilir; state korunması
bu testin amacı olsa da raw runtime dosyaları Git'e/evidence metadata alt kümesine
kopyalanmaz. Yalnız hash/fixed sonuçlar arşivlenir; output parent0700 kalır.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-300 — Kabul koşucusunun kök yolu ve iptal doğruluğu

**Dar kaynak/native kabulü kapalı; GB-112 kapalı.** K-300 hazırlık13 ve
C3 `549dbe66a775db17875b47c67948245a6a8fbf60` üzerinde ayrı temiz native9 kabulü
geçti. Son 11 belge kapısının geçmesi takip commit'inin önkoşuludur.
K-291'in tarihsel hazırlık8 ve S3 native9 sonucu kendi kaynağında korunur.

Rust test sarmalayıcısının `compiler/..` kökü Python koşucusunun canonical yol
önkoşulunda reddedilir; bu ret native dalgalar başlamadan oluşur. K-300 bu kökü
kanonikleştirir. İlk `clean_exact` çocuk komutundan önce iptal handler'ı kurulur;
kısmi hazırlık da aynı finally temizliği kapsamına alınır. Kısmen kurulmuş host,
oluşturulduğu anda açıkça koşucuya sahiplenilir; listener veya çocuk oluşturma
başarısız olsa da temizliği yalnız bu sahiplik üzerinden yapılır.

Cleanup, gerekli owned kapanışı sürdürebilir; fakat gelen iptal kalıcı olarak
kaydedilir ve sonuç `passed` olamaz. Önceden geçen vaka sonuçları, cleanup
sırasındaki iptali başarıya çeviremez. Özgün Popen/handle yetkisi korunur;
PID bulup benimseme veya yabancı süreç temizleme eklenmez. Mevcut native9,
dört dalga/160 istek, no-retry ve state koruma doğrulama koşulları değişmez.

Yeni Python hazırlık alt vakaları aynı `mixed_revision_hazirlik` Cargo sahibinin
içindedir; yeni Rust test sahibi değildir. `mixed_revision_native` ayrı ve
explicit ignored kalır. Son V2 hazırlığı 13/13 geçti (0,217 sn); özgün
controller exit0/reaped. `k300-preparation-2.log` ve
`k300-preparation-controller-2.json`, 16 Ekim yerel kanıt kökündedir. Dört
eski kaynak karşı örneği mock-only koşuda doğrulandı; native başarı değildir.
Hazırlık başarısı native9 kabulü yerine yazılmaz.

C3 ağaç kimliği `833c8cfb7d841e4fd27bcd7f2395e287c1c3bc99` üzerinde
`k300-native-1` ayrı olarak geçti: mevcut tek ignored Cargo sahibi, dokuz iç
vaka ve dört dalgada 160/160 HTTP200; 5xx, retry, body mismatch ve failure
sayısı sıfırdır. Her dalgada 40 istek, ACTIVATE öncesinde kuyrukta iki istek
ve eski TERM sonrasında tamamlanan 38 yanıt kaydedildi. Özgün dış controller
87,295 saniyede exit0/reaped kapandı; başlangıç/son kaynak temizliği, source
pinleri ve ikili kimliği eşti. Private `.zee` durum dosyaları okunmadı.

C3 commit'inden önce aynı kaynak değişikliklerini içeren a31 tabanlı overlay
için `k300-source-gates-2` 13/13 geçti: tam faz 998 başarılı, 0 başarısız,
25 atlanan (778,533 sn); Clippy ve sürekli güvenlik dahil tüm kapı 959,067 sn.
Bu kaynak kapısı temiz C3 native kampanyasından ayrı kayıttır. İlk kapının
ADR durum etiketi metadata hatasıyla başarısız sonucu korunur; başarıya
çevrilmez. [16 Ekim kabul raporu](urunlesme-k298-kabul-20261016.md) exact
makbuzları bağlar. Son belge/güvenlik 11 kapısının sonucu ayrıca dış makbuzla doğrulanır.

Etki incelemesi: değişiklik test sarmalayıcısı ve kabul koşucusuyla sınırlıdır.
Üretim compiler, native host, grammar/API/tanı, spec/RFC normları, katman ve
bağımlılık grafiği, performans iş yükü veya kalıcı durum değişmez. Bu nedenle
hata kataloğu, semantic regression/compiler beyanları, katman fixture'ı ve
performans/soak tarihçesi için yeni davranış kaydı gerekmez. Mevcut spec/RFC,
ADR, faz ve belge tazelik kapıları yine incelenir; Rust biçimi ve tam faz kapısı
bu test değişikliğinin kabulüne dahildir. Yeni kaynakta koşulmamış RC/soak/IDE
ve S/C2/S3 kanıtları aktarılmaz. P14 genel SLA, DB şema devri, ürün ve platform
kapsamıyla kısmi kalır; C1/K-288 ve P33/K-293 retleri bu çalışmanın dışındadır.
