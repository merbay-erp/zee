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

## Dört gerçek kabul vakası

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

Default Cargo/faz koşusunda bu dört vaka `ignored` görünür: her platforma
Docker/PostgreSQL gereksinimi yüklenmez. Dedicated sürekli Linux kapısı
aynı dört vakayı `--ignored` ile çalıştırır ve exact isimlerin her birinde
`ok`, yalnız bir özet, **4 passed / 0 failed / 0 ignored** ister. Eksik
fixture ortamı paniktir; erken `return` veya başarı sayılmaz. Sıfır test,
yanlış selector, ignored vaka ve başarısız exit için koşucu negatif testleri
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
