# Hermetik gerçek PostgreSQL TLS kabulü

K-222, GB-022 ve ADR-062 için `scripts/postgresql-tls-ci.py` kendi geçici
PostgreSQL kümesini, bir günlük deney CA/leaf anahtarını ve ayrı yanlış CA'yı
üretir. Üretim URL'si, mevcut cluster, ürün verisi ve secrets kullanılmaz.
Sunucu yalnız loopback'e açılır; TLS 1.2 zorunlu ve plaintext HBA reddidir.
Leaf yalnız `DNS:localhost` SAN taşır; aynı sunucuya `127.0.0.1` ile erişim
olumsuz hostname vakasıdır.

## Tek komutla yeniden koşma

Docker Engine ve OpenSSL hazırken depo kökünde:

```bash
python3 scripts/postgresql-tls-ci.py
```

Docker yolu resmî `postgres:16.15-bookworm` multiarch OCI index'ini exact
`sha256:0ea6700a3b4f0ae6ce746519073558aed4d88a79d8d07622a9a644946c7319c4`
ile seçer. Mutable tag veya mevcut kullanıcı konteyneri seçilmez. Her koşu
kendi UUID konteynerini oluşturur ve `finally` içinde kaldırır. CA/key
yalnız geçici dizindedir ve koşu sonunda silinir; repoya eklenmez.

Docker socket'i olmayan izole Linux CI job'ında native PostgreSQL araçlarıyla:

```bash
python3 scripts/postgresql-tls-ci.py --engine local --postgres-bin-dir /usr/lib/postgresql/15/bin
```

Bu yol yeni `initdb` veri dizini ve boş loopback portu kullanır. Root CI job'ı
yalnız oluşturduğu dizini `postgres` kullanıcısına verir ve `runuser` ile
postmaster'ı çalıştırır; kullanıcı hesabı veya mevcut küme değiştirilmez.
Kapanışta kendi `pg_ctl` süreci durdurulur. Aktif özel Forgejo işindeki
Debian Bookworm `postgresql-15` paketi ve GitHub Ubuntu şablonundaki
`postgresql-16` paketi güvenlik güncellemeleri alabilir; exact gözlemlenen
sunucu sürümü her raporda kaydedilir, apt sürümü sabit taban diye sunulmaz.
GitHub Actions'ın açık/kapalı olması bu şablon değişikliğiyle değiştirilmez.

## Yedi gerçek kabul vakası

`compiler/tests/postgresql_testi.rs` içindeki `izole_tls_*` vakaları:

1. Doğru CA ve localhost SAN ile production adaptörü bağlanır; gerçek
   `pg_stat_ssl` kaydı `ssl=true`, `TLSv1.2` döndürür.
2. Aynı erişilebilir sunucu başka CA köküyle reddedilir; doğru oturum önce ve
   sonra çalışır.
3. Aynı sunucu SAN dışında kalan `127.0.0.1` kimliğiyle reddedilir. Koşucu
   ayrıca bu adreste psql/TLS connectivity preflight yapar; bağlantı reddi
   erişilemeyen endpoint ile karıştırılmaz.
4. Gerçek `NULL::text` ve integer sütunları exact mevcut hata/`sutun`
   verisiyle reddedilir; aynı transaction'da sonraki sorgu ve commit başarılıdır.
   Açık `COALESCE` ve `::text` olumlu yolları da kabul edilir.
5. K-235 sorgu/parametre/satır/sütun/sonuç ve migration bütçelerini tam
   eşik ve bir fazlasında sınar; ret sonrası transaction sağlığı korunur.

6. K-250 iki gerçek TLS oturumuna ters satır kilitleri aldırır. Döngü gerçek
   `40P01` üretir; bir actor reddedilir, diğeri ilerler. Her actor iki UPDATE
   denediği için rollback'ten etkilenmeyen BEFORE STATEMENT sequence toplam
   dört yazı girişimi görür. Açık outer ROLLBACK onaylarından sonra bağımsız
   observer kalıcı toplamı sıfır görür; farklı backend'de yeni bağlantı sağlıklıdır.
7. K-250 SERIALIZABLE oturumun acknowledged snapshot okumasından sonra başka
   TLS oturum aynı satırı değiştirip commit eder. Eski snapshot'ın UPDATE'i
   gerçek `40001` verir. Victim'ın önceki marker yazısı ile reddedilen UPDATE'i
   toplam iki sequence girişimidir; otomatik replay yoktur. Açık ROLLBACK
   sonrası diğer commit kalır, victim marker'ı sıfırdır ve server oturumu idle'dır.
   Yeni bağımsız backend aynı adaptörle sağlık sorgusunu tamamlar.

Bu iki vaka statement hatalarını sınar. Kira/request transaction'ının veya
COMMIT'in sonucunu SQLSTATE'ten türetmez. Stock PostgreSQL fixture'ının açık
ROLLBACK onayı ve bağımsız kalıcı gözlemi vardır; bütün extension/provider
hata yolları için genel rollback garantisi değildir. ADR-103'ün C027 ve
COMMIT belirsizliği kararı korunur. Gönderilmiş COMMIT fault kampanyası
`scripts/parola-goc-ci.py` içindeki ayrı on bir vakadır.

Fault actor'ları için statement timeout 15 saniye, lock timeout 12 saniye,
deadlock detector 200 ms'dir. Deadlock worker kapıları ve ana kontrol kanalı
20 saniyelik bounded acknowledged bariyer kullanır. Yeni kaynakta timeout
success diye kabul edilmez; exact SQLSTATE ve gözlenen sonuçlar gerekir.

Default Cargo/faz koşusunda bu yedi vaka `ignored` görünür: her platforma
Docker/PostgreSQL gereksinimi yüklenmez. Dedicated sürekli Linux kapısı
aynı yedi vakayı `--ignored` ile çalıştırır ve exact isimlerin her birinde
`ok`, yalnız bir özet, **7 passed / 0 failed / 0 ignored** ister. Eksik
fixture ortamı paniktir; erken `return` veya başarı sayılmaz. Sıfır test,
yanlış selector, ignored vaka, duplicate/unexpected isim, bool exit ve yanlış
SQLSTATE/attempt/kalıcı durum/health gözlemi için dokuz koşucu kabul/ret testi
`scripts/tests/postgresql-tls-ci-test.py` içindedir. Faz sahibi IO'dur;
mevcut sahte-adaptör dil vakalarının yürüttüğü çekirdek fazlar aynı grubun
`ek_kapsam` kaydındadır.

## Kimlik ve açık sınır

Varsayılan çıktı `compiler/target/postgresql-tls-ci.json` ve `.log`dur.
Rapor HEAD, ağacın temizliği, test/koşucu SHA-256'sı, UTC başlangıç,
host/Rust/sunucu sürümü, imaj veya native araç kimliği, sentetik CA
fingerprint'i ve gerçek libtest exit'ini taşır. Kirli ağaçta koşu exact
commit kabulü sayılmaz. Sürekli CI bunları kaynak SHA'sına bağlı faz
artefaktına ekler; workflow kablosu uzak koşunun geçtiğinin kanıtı değildir.

Yerel geliştirme kabulünde macOS ARM64 üzerinde Docker PostgreSQL 16.15
ve native Homebrew PostgreSQL 14.18 ile dört vaka sıfır atlamayla geçti.
Bu, aynı commit'in uzak Linux CI kabulü veya desteklenen minimum PostgreSQL
sürümü sözü değildir; source-clean alanı ve exact dosya hash'leri rapordadır.

TLS 1.1 sunucusuna karşı bağımsız downgrade reddi, managed-provider CA
rotasyonu, üretim cluster'ı, proxy/load-balancer ve çok-worker bütçe provası
bu kapının kanıtı değildir. K-173'ün soak/platform açık işleri kapanmaz.
GB-022'nin yalnız env'e bağlı ve CI fixture'ı olmayan hostname/CA/NULL
boşluğu kapanır; spec/25'in TLS protokol tabanı kanıtı bağımsız negatif
downgrade deneyi olmadığı için kısmi kalır.

K-235 kaynak bütçesi vakasının tarihsel dedicated kabulü **5/5** olarak kalır.
K-250 sonrası güncel dedicated kapı **7/7, sıfır ignored** ister. Gerçek SQL sorgu/parametre/satır/sütun/sonuç ve migration dosya/toplam
bütçeleri tam eşik ve bir fazlasında sınanır. Sonuç bayt bütçesi sütun adı
baytlarını da sayar; ret sonrası aynı transaction sağlık ve commit korunur.
K-222'nin tarihsel 4/4 kayıtları değişmez; yeni exact kaynak CI kabulü ayrıdır.

K-250 adayında PostgreSQL 16.15 resmi kaynaktan yapılmış sahipli yerel runtime,
Rust 1.93.1 ve `965c3a9` tabanlı kirli test çalışma ağacıyla 7/7 geçti.
Bu sonuç temiz commit kabulü değildir. Test/binary/input hashleri ve ilk raw
kanıt ayrı tutulur; integration sonrası yeni temiz exact kaynakta tekrar koşu
ve phase sayısı yenilenmesi gerekir. Yeni compiler davranışı, retry,
SQLSTATE allowlist'i, desteklenen minimum PG sürümü, geniş fault matrisi veya
uzak CI/platform/provider kabulü eklenmez. Üretim Rust kaynağı değişmedi.

## K-256 güncel kapı — sekiz native vaka

K-250'nin yedi vakası korunur; ek nativeTLS1.1 ret vakasından önce ayrı owned
legacy cluster'a independent Python/OpenSSL client doğru CA/localhostSAN ile
TLS1.1 bağlanır, pg_stat_ssl protokolü ayrıca doğrular. Legacy seclevel0 yalnız
ikinci fixture'dadır; native connector minTLS1.2 değişmez. Strict14 vakası
sahte positivecontrol ve expiry sonrası send'i reddeder; normal Cargo wrapper
bu14 testi yürütür. Tarihsel dört/beş/yedi vaka kayıtları kendi SHA'sına aittir.
Final clean exact acceptance ayrıca gerekir; [ADR-115](../adr/115-tls11-capable-positive-kontrol.md).

## K-302 — Güncel kaynakta yerel TLS ve göç kabulü

Temiz `e6322dbc0424a54f777a7b14f65d78712cfced1a` üzerinde sahipli PG16.15 runtime ile mevcut sekiz TLS
ve bir göç vakası sıfır başarısız/atlamayla geçti. TLS1.1 erişilebilir olumlu
kontrolü ile native ret, CA/hostname/NULL, bütçe, 40P01/40001 ve göçün
rol/checksum/atomik sınırları mevcut testlerce doğrulandı. Runtime derlemesi
ve Zee kabulü ayrı makbuzlardır; kaynak/pin/owned cleanup ve exact sonuçlar
[ortak rapordadır](urunlesme-k298-kabul-20261016.md). Bu, o kaynakta seçili
9/25 explicit ignored vakanın dar kabulüdür; tam fazın yeniden çalıştığı,
uzak CI/provider/ürün veya ASan/soak kabulü anlamına gelmez.
İlk runtime derlemesinin `utils/errcodes.h` hatası ve başarısız makbuzu
korunur. Yayımlanan kaynak zaman damgaları korunan ikinci derleme geçti
(19 komut, 97,178 saniye); yeni TLS/göç kabulü 35,622 saniyedir. Üç PG süreci
normal exit0/reap ile kapandı ve geçici özel küme dizini kaldırıldı.
Son belge/güvenlik kapısı kayıt commit’inin önkoşuludur; sonuç ortak raporun dış makbuzundan izlenir.
