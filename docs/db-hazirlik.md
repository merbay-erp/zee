# K-251: uygulama sahipli DB ve göç hazırlık probu

[Host yardımcısı](../scripts/db-hazirlik.py), mevcut `göçür --durum` ve
`çalıştır` CLI'larını kullanarak [worker kapısına](worker-supervisor.md)
uygulama revision'ına bağlı sonuç yazar. Compiler özelliği değildir.
Python 3.9+ ve POSIX flock/dirfd/O_NOFOLLOW gerekir.

## Sözleşme ve çağrı

Operatör, anonim [probe örneğini](../compiler/tests/fixtures/db-readiness/probe/proje.dil)
kendi uygulamasının gerçek sınırlı SQL sözleşmesine uyarlar. Örneğin iki
kolonun ad/tür/NOT NULL sırasını, primary key'i, tek satırı ve sabit revision'ı
kontrol eder. Bütün DDL/index/constraint/default/extension eşliği değildir.
Migration makbuzunun eşliği tek başına gerçek DDL eşliği sayılmaz.

Probe proje kökü kanonik, mutlak ve diğer kullanıcıların yazmasına kapalı
olmalıdır. JSON contract tam dört alan taşır: `schema` değeri
`zee-db-readiness-contract-1`, `application_revision` (40 küçük hex),
`files` (3–32 exact göreli yol+SHA-256) ve `migrations` (1–16 sıralı
version/name/SHA-256). Dosya toplamı en çok 256 KiB'tır; bütün proje dosyaları
listelenir. Migration adı pozitif version ile başlayan `.sql` adıdır.

```json
{"schema":"zee-db-readiness-contract-1","application_revision":"2222222222222222222222222222222222222222","files":[{"path":"proje.dil","sha256":"DOSYA_SHA256"},{"path":"kaynak/hazirlik.dil","sha256":"DOSYA_SHA256"},{"path":"gocler/0001_hazirlik.sql","sha256":"DOSYA_SHA256"}],"migrations":[{"version":1,"name":"0001_hazirlik.sql","sha256":"DOSYA_SHA256"}]}
```

Yer tutucular operatörün doğruladığı exact SHA-256/SHA değerleriyle değiştirilir.
Contract SHA'sı güvenilir deployment kaydından alınır; dosyanın yanında aynı
aktörün yazdığı hash kaynak otoritesi veya imza değildir. Probe kaynakları
uygulama tarafından güvenilir olmalıdır. Helper readonly SQL yetkisini
kendisi kurmaz; bağlantı rolünün yazma/DDL yetkisi olmamalı ve PostgreSQL
`default_transaction_read_only=on` kullanmalıdır. Native kampanya bu rolü
ayrı admin'den kurar, hazırlıkta yalnız SELECT çalıştırır.

```sh
python3 -I -B scripts/db-hazirlik.py \
  --dil /opt/zee/bin/dil --project /srv/probe \
  --worker-project /srv/uygulama --gate /srv/uygulama/.zee/hazirlik.json \
  --expected-source DERLEYICI_SHA --application-revision UYGULAMA_SHA \
  --contract /srv/deployment/readiness-contract.json \
  --contract-sha256 CONTRACT_SHA256 --deadline 8
```

DB URL ve TLS CA, probe `proje.dil` kaydının mevcut ortam değişkenlerinden
okunur; argv/log'a kopyalanmaz. Helper exact temiz build kimliğini ister;
`--pilot`/dirty bypass'ı yoktur. İkili ve proje private snapshot'ına alınır,
başlangıç/son metadata+hash eşliği kontrol edilir. Üç child çağrısı toplam
0.1–15 saniye monoton deadline ve her stdout/stderr için 64 KiB sınırı
paylaşır; süre/çıktı aşımında doğrudan child kill/wait yapılır.

Geçerli gate edinildikten hemen sonra `ready:false` atomik yazılır. Lock
aynı kapıya ikinci helper'ı reddeder; hedef symlink/hardlink ve public-write
regular dosya kabul edilmez. Sonra exact temiz BuildJSON, tüm göçlerin
uygulanmış checksum eşliği ve probe'un yalnız `zee-db-ready-1` satırıyla
exit0 vermesi birlikte `ready:true` üretir. Raw child çıktı/hata stdout'a
aktarılmaz. Host özeti yalnız fixed schema/revision/ready/reason alanıdır.
Ret exit1 (migration/schema/timeout/output) veya exit2 (input/gate/identity)
verir. Gate edinilemeyen geçersiz çağrı var olan kapıyı kapatamaz.

## Yaşam döngüsü ve sınırlar

Bu tek seferlik probdur. Supervisor başladıktan/her HUP neslinden sonra,
DB stop/restore, göç veya schema değişiminden sonra tekrar çağrılmalıdır.
Sürekli DB izleme, TTL, helper SIGKILL'inden sonra kapıyı otomatik kapatma,
DB bozulmasının anında görülmesi garantisi yoktur. Önceki başarılı dosya
helper yeniden çağrılana kadar durabilir; supervisor nesil tazeliği yalnız
başlangıç/HUP sınırını kapatır. Request/DB metrics, inflight drain, config
reload, P14 socket devri/overlap ve sıfır kesinti bu dilimle tamamlanmaz.

## Yerel kanıtı tekrar çalıştırma

Yalnız kendi anonim loopback PostgreSQL16.15 runtime'ı ve temiz exact native
ikili kullanılır; mevcut müşteri DB'si/URL'si kabul için kullanılmaz:

```sh
python3 -I -B scripts/tests/db-hazirlik-strict-test.py
python3 -I -B scripts/tests/db-hazirlik-test.py \
  --dil /ABSOLUTE/CLEAN/dil --framework-root /ABSOLUTE/zee \
  --postgres-bin-dir /ABSOLUTE/owned-pg16/bin \
  --output /ABSOLUTE/NEW/evidence
```

Native13 vaka eksik ledger, stale başlangıç kapısı, iki worker'la olumlu
hazırlık, pending migration, checksum drift, actual schema drift, revision
uyuşmazlığı, schema recovery, DB stop, yeni cluster'a logical restore,
HUP sonrası fresh gate ve SQL hatası maskesini sınar. Stop/schema drift
sırasında koşulsuz liveness200 devam eder; aggregate readiness ayrıdır.
Probe'un committed state'i değiştirmediği ve owned cluster/worker cleanup
ayrıca doğrulanır. Strict10 test atomic false/true, symlink/hardlink/public
write/parent replacement/lock, child deadline ve bounded-output retlerini
sınar. Yerel geliştirici Mac kanıtı fiziksel platform/ürün adoption kabulü
değildir; yeni exact temiz kaynak kabulü ayrıca alınır.
