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

## K-307 — kabul harness’i iptal ve yayın doğruluğu: açık

K-307 / GB-116 düşük ve açıktır. `scripts/tests/modul-sql-native.py` cleanup callback’inin exact AST modeli, fake cluster.close içinden fake TERM handler çağrıldığında yeni iptalin kaybolduğunu gösterdi: tek beklenen assertion failure, özgün owner exit1/reaped. Gerçek sinyal, OS/PG süreci veya sızıntı çalıştırılmadı; gerçek PG kaçağı kanıtı değildir.

Düzeltme kapsamı ayrı yapışkan iptal ledger’ı; cleanup için yalnız H iptal bayrağının geçici maskelenmesi ve her yolda geri yüklenmesi; korumalı kısmi ilk handler kurulumu; tüm handler geri yüklemeleri; ExitStack ve geçici dizin cleanup’ından sonra passed kararı; yayın sırasında iptal/deadline/failure kontrolü ve birincil hatanın korunmasıdır. Diagnostic veya bir handler restore hatası kalan geri yüklemeleri atlayamaz. Mevcut600sn kampanya kabul sınırı ve dokuz native SQL oracle değişmez.

T adayV4 inert13 ve T adayV5 birleşik23 yöntemi gerçek özgün owner makbuzlarıyla geçti; baseline10 ayrıca geçti. Eski10 ordinary metot korunup13 inert yöntem aynı `compiler/tests/modul_sql_testi.rs::modul_sql_stdlib_sozlesmesi` sahibi altında birleştirildi. Bu controlled sonuç kaynak veya fresh native kabulü değildir; Kaynak/test/belge uygulaması bu atomik dilimin parçasıdır. Ayrı ignored sahibi `modul_sql_native_crud_goc_yasam_dongusu` ve dokuz alt native vaka korunur; Python yöntem sayısı Cargo/native toplamına eklenmez. Kaynak+test+belge aynı atomik commit, gerekli katalog/tazelik/fmt/faz ve kaynak kapıları, exact temiz kaynak, fresh native9 ve son kapılar kapanış önkoşuludur.

Bu dar harness işi `TLS.LocalCluster` içindeki pg_ctl ile başlatılan detached PostgreSQL ve immediate-stop sahipliğini foreground özgün Popen/normal kapanışa dönüştürmez. Kalan beş PG tüketicisi, genel P20/adoption/platform, DB+FS atomikliği ve unknown no-replay sınırı açıktır. Üretim compiler175/bağımlılık/katman, grammar/API/ABI, normatif spec/RFC, kullanıcı tanısı/katalog kodu, sabit performans iş yükü veya compiler/LSP kalıcı durum değişmez; yeni özellik/RC/soak/IDE kabulü aktarılmaz.

Gerçek kontrollü sonuçlar ayrı makbuzlarla kayıtlıdır: baseline10, adayV4 inert13 ve adayV5 birleşik23 yöntemi geçti; her özgün owner observed_exit=exit=0/closed/reaped/errors[]/cancelled[]. Birleşik23, korunan10 ordinary ve eklenen13 inert yöntemdir; Cargo/native sayısı değildir. Bu kaynak/test/belge dilimi atomik uygulama kapsamındadır; kaynak kapıları, exact temiz kaynak ve fresh native9/son kapı kabulü bekleniyor.
