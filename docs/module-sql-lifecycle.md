# Sahipli modül SQL yaşam döngüsü

K-285 / ADR-143 mevcut K-273 modül yöneticisini byte-identical helper olarak
kullanır. Yeni `scripts/modul-sql-yoneticisi.py` ayrı opt-in profildir. Eski
`modul-yoneticisi.py` ile SQL execute/CRUD yetkisi açılmaz. Compiler grammar,
Rust API, genel extension ABI veya RFC-0012 FFI gerçeklenmez.

## Şema ve sorgular

`zee-sql-module-1` tanımı id, label ve 1–8 alan taşır. Alan türü `text`,
`integer` veya `boolean`; SQL identifier yalnız küçük ASCII harf/rakam/alt çizgi,
en çok24 karakterdir. Ayrılmış id/state/schema ve tekrar alan reddedilir.
Kullanıcı değerleri SQL metnine girmez: mevcut Zee PostgreSQL parametreleri ve
sabit cast'ler kullanılır. Metin4096UTF8 byte, kimlik128 karakter, integer signed64
ve boolean gerçek JSON boolean ile sınırlıdır. Liste en çok100 satır döndürür. Yazı receipt’i gerçek affected0/1 sayısını taşır; bulunmayan update/delete değişmiş satır gibi sunulmaz.

Aynı uygulama kökünde modül registry/menü/içerik-türü ve GET liste adaptörü
oluşur; uygulama manifesti açık veritabanı yetkisi taşır. CLI `create`, `read`,
`update`, `delete`, `list` aynı şemanın gerçek Zee eylem/sorgularını çalıştırır.
Mutasyon HTTP endpoint'i/CSRF/auth katmanı eklenmez. Native HTTP yanıtı HTML olduğundan GET adaptörü JSON metnini HTML kaçışından geçirip sabit `<pre>` içinde sunar. CLI read/list stdout'u ayrı gerçek JSON verisidir; HTML-typed cevaba raw JSON koyulmaz.

Profil `zee-sql-module-profile-1`: açık loopback PostgreSQL target ve sabit
`ZEE_MODULE_DATABASE_URL` adı. DSN ve CA yalnız environment'tadır; profil ve
hata çıktısına parola kopyalanmaz. Operator yeni ve kendisine ait boş veritabanı
sağlar; ilk up public relation yokluğunu sorgular. Araç mevcut yabancı tabloyu
benimsemez; namespace çakışmasında CREATE transaction'ı geri alınır. Bu localhost
adresinden mülkiyet veya hostile same-UID sandbox sonucu çıkarılmaz.

## Göç ve veri koruma

Her up/down/reapply yeni pozitif sıralı migration ekler. `dil göçür` mevcut
transaction ve `_zee_gocleri` SHA-256 tarihçesini uygular. Uygulanmış kaynak
silinmez/yeniden yazılmaz; schema fingerprint ve gerçek sütun/type/not-null
metadata'sı denetlenir. İlk up CREATE; down active tabloyu archive adına SQL
ALTER TABLE ile taşır; reapply aynı tabloyu geri taşır. Satırlar korunur.
Down destructive drop/uninstall değildir. Bu profilde herhangi bir modül down
iken managed HTTP launch kapalıdır; diğer CLI modüllerinin durumu ayrı kalır.
Terminal kaldırma, schema evrimi ve keyfi kullanıcı migration SQL'i açılmaz.

DB ve dosya sistemi ortak atomik transaction değildir. EX lock ve active
marker koruması canlı managed host sırasında değişikliği reddeder. DB çağrısı
öncesinde durable `sql-pending.json` yazılır; bilinmeyen commit, hata veya kısmi
projection admission'ı kapalı tutar. `reconcile` göçü yeniden çalıştırmaz:
readonly gerçek metadata + exact bütün applied ledger hash'leri doğrulanınca
hazır projection journal'ını tamamlar. Uygulanmamış/başarısız göç pending'i
otomatik silinmez veya yeniden denenmez.

CRUD yazısında da ayrı durable `sql-write-pending.json` önce yazılır. Native
komut/receipt belirsizse yeni CRUD/launch kapalıdır. `reconcile-write --data`
operator review'u exact pending SHA ve yeni readonly satır gözlemiyle eşleştirir;
`accept-observed-state-no-replay` kararı yazıyı tekrar çalıştırmaz. Bu gözlem
hangi denemenin etkiyi ürettiğini veya exactly-once ispatlamaz. Post-unlink
fsync hatası exit4 görünür tamamlanma/durability belirsizliğidir; sahte paused
veya rollback raporu değildir.

## Kullanım

```sh
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil --profile /absolute/profile.json init 18080
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil up golden/module-sql/news.json
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil --data /absolute/new-item.json create news
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil list news
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil down news
python3 scripts/modul-sql-yoneticisi.py --root /absolute/fresh-app --binary /absolute/dil reapply news
```

Hata sabit `sql-module-operation-rejected`, exit2; görünür tamamlanma/durability
belirsizliği ayrı sabit ileti/exit4. SQL/DSN/body/path echo edilmez. Başarılı
read/list kullanıcının talep ettiği kendi verisini stdout'a yazar.
Her native işlem private generated Zee project, exact binary başlangıç/son
hash kontrolü ve mevcut bounded lifecycle owner ile çalışır. İşlem başına
kilitle15s/native20s ve capture64KiB vardır; tüm çok adımlı transition için
sert toplam wall-time veya powerloss finally garantisi verilmez. İptal
bayrağı ve held WNOWAIT TERM/KILL/reap sınırı eski helper'dan gelir.

## Kanıt kapsamı

Normal stdlib contract testleri ve ayrı ignored fresh-PG native kampanyası
vardır. Native iki farklı şema, gerçek CRUD/HTTP liste, down/up veri korunumu,
applied drift/collision, commit sonrası kayıp receipt ve DB/FS publication fault
ile readonly recovery'yi ölçer. Alt senaryolar Cargo test toplamını çoğaltmaz.
Örnekler golden tasarım fixture'ıdır; gerçek ürün P08/adoption veya müşteri
migration'ı diye dogfood korpusuna kayıt edilmez. Yeni exact source için clean-S
kampanyası ve root gate'leri ayrıca gerekir.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-307 — foreground PostgreSQL takip2: açık

K-307 / GB-116 düşük ve açıktır. İlk sticky cleanup iptali/yayın dilimindeki exact AST fakeTERM/fakecluster.close karşı örneği tek beklenen failure/exit1/reaped verdi; gerçek OS/PG sızıntısı gözlemi değildi. Baseline10, V4inert13 ve V5birleşik23 controlled sonuçları kendi makbuzlarıyla korunur; ilk kaynak15 kabulü yalnız kendi tam koşu/owner ve temiz commit makbuzlarıyla değerlendirilir.

Bu foreground takip2 anlatımı yalnız reviewed beş kaynak dosyasının aynı atomik kaynak/test/belge diliminde uygulanması altında geçerlidir. Yeni test-only `scripts/tests/native-pg-owner.py` ve preflight dosyası, modül SQL native harness’i ile ordinary/Rust wrapper aynı dilimdedir. SQL harness’i `TLS.LocalCluster` yerine `OwnedLocalCluster` foreground sahibi ve `SqlPgCampaign` adapter’ını kullanır. Yeni PostgreSQL özgün Popen nesnesi korunur; readiness çocuklarının özgün sahipliği ve data-directory/version kimliği doğrulanır. Normal kapanış için aynı özgün PostgreSQL sahibine SIGINT, observed_exit=exit=0/reap ve fallback_used=false gerekir. Belirsiz sahip veya private root kimlik değişiminde private kök silinmez; retained durum sonradan benimsenmez. Bu normal kapanış gerçek PG üzerinde henüz ölçülmüş değildir; helper’ın kabul gereğidir.

İlk yapışkan ledger, cleanup sırasında H maskesi, garantili restore, korumalı handler kurulumu, ExitStack sonrası passed ve publication iptal/deadline/failure denetimleri korunur. SqlPgCampaign cleanup90 sınırı kabul600 sınırını uzatmaz; dokuz SQL native oracle, pinler, pending/unknown no-replay, SQL/DB+FS ve üretim semantiği değişmez. Native Rust wrapper ayrıca foreground PG kapanış/readiness sonucunu ister; bu kontrolün varlığı fresh native PASS değildir. `ZEE_MODULE_SQL_NATIVE_OUT` yalnız internal ignored testin taze mutlak çıktı yolu seçimidir; belirtilmediğinde mevcut0700 geçici dizin varsayılanı korunur. Ürün CLI/API kapasitesi açılmaz.

Gerçek kontrollü hazırlık: aynı helper/preflight baytları üzerinde standalone foreground23 ve takip2 birleşik ordinary24 geçti; özgün owner observed_exit=exit=0/closed/reaped/errors[]/cancelled[]. Standalone23 önceki Tproposal1 yolunda çalıştı; actual hedef/preflight hashleri Tproposal2 ile birebirdir. Birleşik24, önceki23 yöntemi koruyup yeni foreground preflight23’ü içten çalıştıran bir ordinary yöntem ekler; bunlar47 veya yeni Cargo/native toplamı değildir. Mevcut ordinary Cargo sahibi ve ignored native sahibi/dokuz vaka korunur.

SQL foreground bağlantısı bu takip dilimindedir; diğer dört PG tüketicisinin foreground dönüşümü bu dilime dahil değildir ve açık kalır. SQL’in kendi fresh native9 kabulü de bekler; beş PG tüketicisinin tamamı kapalı veya tüm sahiplik kabulü geçti denmez. Genel P20 kısmi, ürün/adoption/platform ve C1/K288/P33/K293 retleri korunur. Önce ilk kaynak15/temiz kaynak commit’i gerçek makbuzlarıyla kapanmalı; sonra takip2 beş kaynak+test+12 belge atomik uygulanıp kendi kaynak kapıları/temiz exact commit’i, ayrı fresh native9 ve son kapılar sağlanmalıdır. Sonraki kaynak SHA/tree henüz bilinmiyor; burada üretilmez.

Etki incelemesi: production Rust175, bağımlılık/katman, grammar/API/ABI, normatif spec/RFC, kullanıcı tanısı/katalog kodu, sabit performans iş yükü ve compiler/LSP kalıcı durum değişmez. Test-only helper/adapter ve native output/proof kontrolleri kullanıcı ürün kapasitesi açmaz; RC/soak/IDE veya geçmiş SHA sonuçları aktarılmaz.
