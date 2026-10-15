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
