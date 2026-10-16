# K-282 — Sürekli dispatcher ve application-owned adapter

K-268 sonlu SQL tick ve K-277 sabit SMTP fixture'ı korunur. Yeni
[`dispatcher-service.py`](../scripts/dispatcher-service.py) açık `--run` ile
beklemeden tekrar başlayan daemon değildir: bilinen tamamlanmadan **sonra**
1–3600 saniye monotonic aralık bekler. Kaçırılmış zamanları backfill etmez.
Varsayılan yalnız preflight'tır; `--once` tek aktivasyondur. `--render` dosya
üretir; systemctl/link/start/enable/linger çalıştırmaz. Gerçek ürün veya provider
adoptionı, SMTP exactly-once teslim ve genel cron platformları kapanmaz.

[ADR-139](../adr/139-surekli-dispatcher-ve-uygulama-adaptoru.md) iki explicit
profili ayırır. JSON config canonical/current-owner0600/single-link normal dosya,
16KiB; parent/state current-owner0700 ve symlink olmayan namespace'tir.
Duplicate/unknown JSON alanları, nonfinite değerler ve bilinmeyen CLI reddedilir.
Python >=3.11, Unix WNOWAIT; gerçek yürütme yalnız Linux `/proc` profili ister.

Ortak exact alanlar: `schema=zee-dispatch-service-1`,
`kind=sql-intent|application-adapter`, `repo`, `source_sha`, `state`,
`interval_seconds`. Repo HEAD tam40hex ve temiz, state repo dışında olmalıdır.
Bütün config/kod/ikili/input pinleri her tick öncesi ve başarılı tick sonunda
kontrol edilir. Örnek hash veya fabrikasyon clean-source kaydı verilmez.

## SQL profili

Ek alanlar `timer_config` ve `timer_config_sha256`dir. Aynı temiz repo/S'ye ait
[K-268 config](sonlu-dispatcher-timer.md) ve mevcut `--ticks 1` çağrısı yeniden
kullanılır; queue/reaper/lease/generation/known-fail politikası değiştirilmez.
Daemon'ın `service.lock` kilidi beklemeler dahil bütün ömrü boyunca tutulur;
her tick ayrıca K-268 state kilidini ve fsynced inflight protokolünü kullanır.
Kilit çatışması75, preflight2, unknown/pause3, bilinen tamamlanmanın görünür fakat
son fsync'i belirsiz yayını4'tür. Bunlar native/provider exit kodları değildir.

## Application-owned adapter profili

Ek exact alanlar: `executable`, `executable_sha256`, `cwd`, `data`,
`data_sha256`, `argv`, `timeout_seconds`, `contract=zee-app-dispatch-result-1`.
Executable açık ve executable-bit'li güvenilir uygulama kodudur; sandbox değildir.
Shell kullanılmaz. `argv` en çok16 sabit/control-free argüman, her biri256bayt;
config içine credential/değer koyulmaz. Program tam olarak
`[private_executable_snapshot, *argv, --input, private_data_snapshot]` ile çağrılır.
Executable ve input aynı bounded okuma sırasında hashlenip owned tmp
O_EXCL dosyalarına kopyalanır; child orijinal değiştirilebilir input yolunu almaz.
Başarıdan sonra snapshot hashleri tekrar doğrulanır ve yalnız bu iki issued dosya
kaldırılır. Unknown sonuçta en çok128MiB+1MiB snapshot ve latch korunabilir;
yeni child admission olmadığı için aynı state içinde sınırsız snapshot birikmez. Payload executable, shell veya
handler seçmez; data1MiB ve executable128MiB observed/nofollow regular snapshot
hash'i ile pinlidir. Cwd owner/nonwritable normal dizindir; held directory FD'ye
ait `/proc/<ownPID>/fd/<ownFD>` alias'ı kullanılır. State/tmp FD'leri cleanup
sonuna kadar korunur; yalnız lokalde açılmış descriptor/PID'ler türetilir.

Çocuk env'i yalnız PATH/locale ve own TMPDIR'dir; ambient credential veya
arbitrary FD mirası yoktur. Credential/provider composition uygulamanın açık
owned yetkisi ve ayrı protokolüdür; bu profil gizlice sır bulmaz/forward etmez.
1–165sn mutlak timeout,4096bayt birleşik output sınırı ve TERM/KILL/reap vardır.
Çocuk stdin kapalıdır. Stdout+stderr **birlikte** yalnız şu exact JSON olmalıdır:

```json
{"schema":"zee-app-dispatch-result-1","outcome":"completed","completed":1}
```

`outcome=idle` yalnız `completed=0`; completed sayısı gerçek integer1–100'dür.
Bu uygulama-owned ack'tir: host queue kaydı/SMTP delivery/DB COMMIT doğrulamış
sayılmaz. Adapter kendi durable lease/fencing/idempotency/ack protokolüne
sahiptir. İş sayısı ve sabit outcome dışında business identity, payload, argv,
SQL veya child output kaydedilmez. `last.json` bounded sabit metadata'dır.

Fsync edilmiş `inflight.json`, **Popen'dan önce** kurulur. Nonzero, timeout,
malformed ack, TERM/SIGKILL, output overflow veya pin/cleanup hatası latch'i
tutar; sonraki tick ve restart yeni adapter başlatmaz. Bilinen exit0+strict ack,
cleanup ve postpin sonrasında durable last yazılır ve exact latch kaldırılır.
Son directory fsync hatası exit4/görünür clear'dır; false rollback/pause iddiası
yoktur. Adapter protokolü K-268 reconciliation JSON'uyla temizlenmez.

Manuel adapter reconciliation: owned service'i durdur, cgroup boşluğunu ve
exact latch/config/executable/data hashlerini arşivle; gerçek uygulama ledger'ı
ve provider ack üzerinden durumu incele. Unknown ise latch kalır. Bu sürümde
adapter guard'ını kaldıran/replay yaptıran bir komut yoktur; başka state yolu
vermek geçmiş işi güvenli biçimde kurtarmış sayılmaz. Yeni protokol/işlem ancak
ayrı uygulama-owned review ile tasarlanır; arbitrary state/lock silinmez.

## User service/calendar profili

```sh
python3 -I -B scripts/dispatcher-service.py --config /owned/config.json --render /owned/fresh-units
python3 -I -B scripts/dispatcher-service.py --config /owned/config.json --render /owned/fresh-calendar --calendar '*-*-* *:*:00'
```

İlk çıktı `zee-dispatch.service` sürekli Type=simple; ikinci çıktı aynı isimli
`--once` service ve `zee-dispatch.timer`dir. İsimler değiştirilmez; mevcut aynı
isimli unit/resource asla benimsenmez. Calendar systemd OnCalendar sözdizimidir;
newline/unit substitution reddedilir, gerçek `systemd-analyze calendar` ve
`--user verify` kabulü ayrıca gerekir. `Persistent=false`: kaçırılmış activation
ve DST/backfill politikası icat edilmez. Cron-format parser/crontab installer
veya provider scheduler değildir. Mevcut cron yöneticisi açık `--once` çağrısını
kullanabilir; bu sürüm cron kurmaz veya bütün cron ortamlarını kabul etmez.

Restart=on-failure yalnız2/3/4/75 dışındaki crash'ler için; crash sonrası durable
latch yeniden admission'ı kapatır. KillMode=mixed, stop10s ve son cgroup KILL
vardır; SIGKILL sırasında Python finally çalışacağı iddia edilmez.

Hazırlık testleri gerçek kısa Python çocuklarıyla22 vaka çalıştırır; Mac `/proc`
çözümlemesi açıkça modellenir. Ayrı native koşucu, temiz exactS+native binary+
PG16.15+Linux user manager ister: iki native SQL tick, gerçek calendar tick,
TERM/unknown sonrasında explicit restart/no replay ve gerçek Zee yorumlayıcısını
çağıran application adapter. Bu belgede bu yeni kampanya **çalıştırıldı denmez**.
Geçmiş S finite timer2/SMTP11 kabulü bu yeni source için kabul yerine geçmez.

K-282 son hazırlık düzeltmesinde PostgreSQL/cert/query/build çocukları aynı
sahipli zaman ve çıktı bütçesine alındı. Fixture dosyaları açık kaynak listesiyle
kopyalanıp başlangıç/son SHA ile doğrulanır; her komut ayrı artan kimlikli
kayıt taşır. Kapanmayan PG durumunda kanıt dizini korunur. Bu hazırlık
sonuçları Linux native4 kabulünün yerine geçmez.

## K-295 — daemon unit'in explicit yürütme düzeltmesi

S2 `e35b617095e6a9d00915794d6adee90842238a5f` gerçek Linux koşusunda
daemon ExecStart yalnız --config taşıdığı için check-only kipinde exit0 dönmüş,
hiç SQL tick başlamamıştır (0/4; cleanup_errors boş, owned_units_released true).
Üretilen daemon artık explicit --run, OnCalendar hizmeti yalnız --once taşır.
Doğrudan CLI varsayılan preflight ve render'ın servis başlatmama sınırı değişmez.
Eski unit dosyaları yeniden --render ile üretilmelidir. Exact S2 kaynağına karşı
yeni regresyon kırmızı, düzeltmeyle hazırlık22 yeşildir; S3 gerçek Linux dört vaka kabulü geçti. Host correctness düzeltmesidir; compiler bug/grammar/tanı/API değişmez.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-306 — Linux kabul koşucusu: açık

İki Rust wrapper’ın canonical repo kökü, ilk çocuk öncesi handler, yapışkan iptal/başarısızlık ve kısmi cleanup düzeltildi. Foreground PostgreSQL özgün Popen adaptörü yalnız dispatcher ve HTTP kabul tüketicilerine bağlıdır. Üretim davranışı ve mevcut4/7 oracle değişmedi.

Kaynak doğrulaması, ikinci denemenin geçen 12 komutu ile yalnız EOF boşluk düzeltmesinden sonra yeniden geçen beş komuta dayanır; 15 kapının bütünü yeniden çalışmış gösterilmez. Temiz `7d5e4b0` kaynak commit’i ve exact ağaç kayıtlıdır. Kontrollü hazırlık Campaign16 ve PG18 yöntem/31 senaryoda geçti. Dış sahiplik hazırlığında core17, CLI9 ve runner5 modeli geçti; bu sayılar Cargo/native vaka toplamına eklenmez.

C12 V3/V4 başarısız kampanyaları tarihsel kalır. S13 tam15 kapısı ve temiz aec09c1 commit’i gerçek; VM5 dispatcher2/4 sonrası sahipli timer reset hatasıyla başarısız oldu, HTTP çalışmadı. Takip2 kaynak/test düzeltmesinin fresh VM6 native4/7 kabulü henüz yok. Önceki başarısız VM kampanyalarının özgün owner kapanışları kendi kayıtlarında korunur; yeni kaynakta native PG/çocuk/VM kabulü bekleniyor. Koşullu kaynak düzeltmesi bağımsız incelemesi native kabulü değildir; yeni kaynak ve native zincirinin bağımsız incelemesi ayrıca gerekir.

Tarihsel S3 ile K-298–K-305 sonuçları kendi kaynaklarında kalır; farklı SHA sonuçları toplanmaz. P09/P13/P36 kısmi, P37 sürekli; ürün/sağlayıcı benimsenmesi, genel exactly-once ve fiziksel platform U2 açık kalır. C1/P33 reddedilmiş kapsamı yeniden denenmez. S13 kaynak tam15 kapısı gerçek998/0/25 tam faz ile geçti ve temiz aec09c1 kaynak commit’i E/k306-source-commit-2.json ile kayıtlıdır; bu kaynakta VM5 başarısız kaldı. Takip2 kaynak+test+belge commit’i tam15 kaynak kapısı E/k306-source-gates-4/PROOF.json/controller4 ve exact E/k306-source-commit-3.json/controller3 kayıtlarına bağlıdır; bu yollar gelecekteki başarı iddiası değildir. Fresh yeni-kaynak VM6 dispatcher4/HTTP7 ve kapanış kapıları ayrıca gerekir. Ayrıntı ve başarısız deneme geçmişi [ortak kabul raporu](urunlesme-k298-kabul-20261016.md).

Bilinçli etki incelemesi: production Rust/bağımlılık/katman, grammar/API/ABI, tanı, normatif spec/RFC ve uyumluluk yüzeyi değişmez. Compiler/LSP kalıcı durum veya sabit performans iş yükü eklenmedi; ADR045/049/046/069, performans/soak tarihçesi ve compiler regression/beyanları etkilenmez.
