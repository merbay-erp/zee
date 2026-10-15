# Yönetilen modül iskeleti — K-273/P20

Bu dar Unix host aracı yalnız kendisinin oluşturduğu yeni owned0700 proje kökünde
çalışır. Mevcut yabancı ürün/projeyi edinmez; özel `modul.py` veya metin ankrajı
çalıştırmaz. [ADR-131](../adr/131-yonetilen-modul-kompozisyonu.md)/GB-089,
işbirlikçi aynı kullanıcı profilini kaydeder. Hostile sameUID/root sandboxı,
genel extension API, DB lifecycle veya tam P20 kabulü değildir.

```sh
python3 scripts/modul-yoneticisi.py --root /owned/new-project --binary /trusted/dil init 18080
python3 scripts/modul-yoneticisi.py --root /owned/new-project --binary /trusted/dil install "$PWD/compiler/tests/fixtures/yonetilen-moduller/haber"
python3 scripts/modul-yoneticisi.py --root /owned/new-project --binary /trusted/dil remove haber
```

`init` mevcut kökü reddeder. `install` yalnız güvenilir yerel modül kaynağını alır;
metadata executable/argv/shell/callback seçmez. `zee-modul-1` exact envanter hashleri,
ID/version/core aralığı, yerel paket adı, route adapter, plain menu etiketi,
content-type adı ve çakışmayan SQL aralığını doğrular. Menü etiketi HTML escaping'den
geçer. Modül kaynakları güvenilir koddur; doğrulama kod sandboxı değildir.
Uzak registry/download veya transitive dependency installer yoktur.

## Değişmeyen kök ve journal

Proje kökü ve `.zee/appdata` yerinde kalır; uygulama verisi candidate projection'a
alınmaz veya nesil gibi swap edilmez. Owner/private mode, canonical/nofollow
parent ve receipt root dev/ino kimliği kontrol edilir. Aktif8 modül, modül başına
16 dosya/256KiB, projection64 dosya, active16 marker ve encoded journal/receipt
256KiB sınırındadır. SQL dosyaları yalnız immutable `archive/` baytlarıdır;
aktif migration dizinine konmaz ve araç tarafından **yürütülmez**. Kaldırma arşivi,
ID/namespace/range tombstone'u korur. Content-type kaydı CRUD depolaması değildir.

Mutasyon EX flock gerektirir. Candidate yeni private kopyada mevcut native CLI
`kilitle` ve `denetle` ile doğrulanır. Önce bounded/nofollow/hashlenmiş dar
`scripts/lifecycle_core.py` aynı okunmuş baytlardan yüklenir; source sha lineage
exact e76fe65b'dir. Bütün encoded old/new operation'lar, nested metadata,
projection ve live old-or-new durumları ilk yazıdan önce birlikte doğrulanır.
Yeni owned yol önceden var olamaz; appdata veya arbitrary user path whitelist'e
alınmaz. Mevcut SQL arşivinin değiştirilmesi reddedilir.

Prepare journal fsync'i live per-file replace/fsync'ten öncedir. Bu çok dosyalı
atomik DB transaction veya otomatik rollback değildir. Kısmi hatada journal kalır
ve launch kapanır; explicit `reconcile` yalnız exact old/new bayt durumlarından
ilerler, foreign mutation reddedilir. Receipt tamamlandıktan sonra journal unlink
görünür fakat son directory fsync başarısızsa exit4
`module-publication-visible-durability-unconfirmed` döner. Görünür tamamlanma ve
kalıcı başarı ayrıdır; false rollback/paused iddiası yoktur. Marker/offline clear
aynı yerel dayanıklılık sınırını taşır.

## Launch sahipliği ve offline inceleme

`launch` boyunca SH flock tutulur; mutasyon EX alamaz. İki SH holder testi iki
worker'ın ortak listener kabulü değildir. Popen öncesi root/projection/binary
bağlayan durable active marker yazılır. Prespawn hata veya parent SIGKILL marker'ı
korur; lock'ın serbest kalması child exit kanıtı değildir. Native çocuk outer
owned process grubunu devralır. Launcher yalnız kendi direct-child handle'ına
TERM3s/KILL2s/wait uygular; outer Supervisor retained leader yetkisiyle grubu
kapatır. PID discovery/adoption, service install veya cgroup algılama yoktur.
Başarılı child exit ve son source/binary pin denetimi marker clear'den öncedir.
Direct `dil` bypass, zero downtime ve genel sıcak kod reload bu profilde yoktur.

`reconcile-active` exact `zee-module-offline-review-1` nesnesini ister: schema,
root [dev,ino], exact marker filename→SHA256 ve
`ownership=owned-service-group-cleanup-confirmed`. Operatör beyanı önce durable
kaydedilir, sonra yalnız bu exact marker'lar silinir. Bu authenticated DB/cgroup
kanıtı değildir; aracı yabancı PID sorgulamaya veya rastgele silmeye yetkili kılmaz.
Partial clear kalan marker'larla konservatif blokta kalır; görünür clear sonrası
fsync hatası ayrı exit4'tür.

## Kanıt ve açık sınır

Normal Cargo wrapper, CARGO_BIN_EXE_dil ve ZEE_BUILD_SHA ile canonical Python23
vakasını sıfır atlamayla çalıştırır; Build/binary/runtime3/fixture10 hashleri başta
ve sonda eşlenir, raw proof target altında kalır. İki modül kur/kaldır/native
HTTP core/menu/content metadata, archive/appdata/root eşliği, TERM/SIGKILL,
kilitler, bounded helper/binary, symlink, malformed nested journal ve kontrollü
IO/fsync arızaları kapsanır. Actual23 içinde native ve fault alt-senaryoları vardır;
23 ayrı native uygulama/kaynak coverage yüzdesi değildir. Binary growth observed
limitinin endless-reader modeli gerçek128MiB büyüyen dosya deneyi diye sunulmaz.

İlk immutable dirty pilot ve taşınabilir adaptör kanıtları tarihsel kaynağında
kalır. Ortak Cargo/final exact clean kabulü ayrıca doğrulanır. Fiziksel powerloss,
Linux service/cgroup, DB SQL execute/applied-history/downmigration, gerçek özel
ürün benimsenmesi, P08 ve tam P20 kapanışı açıktır. Production compiler/grammar,
public Rust facade veya kalıcı compiler/LSP state değişmez.
