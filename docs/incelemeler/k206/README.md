# K-206 izole PostgreSQL inceleme kanıtı

`rss-once.json`, `probe.rs`, `driver.py` mevcut Zee adaptörünün temiz
29be85b kaynak deneyi; sonraki `rss-prototype.json` ve `contract-prototype.json`
9e80b69 kaynak ortamındaki **doğrudan postgres stream prototipidir**.
Üretim entegrasyonu, retry/commit_unknown ve bütün transaction matrisi
kanıtı değildir; GB-041 açıktır. RSS toplam süreçtir, sürücü tahsisi değildir.
Bir örnek/sıfır ısınma, aynı immutable PG 16.15 imajı ve release/Rust 1.93.1
kullanılır; K-207 process-RSS soak aynı masaüstünde eşzamanlı çalışmıştır.
Prototip bütün sonucu Vec'e toplamaz, ilk uygulama hatasından sonra sonucu
bırakır ve bağlantı sağlığı/hata önceliği için satırları tüketir. Tek büyük
wire satırı hâlâ sürücü belleğinde oluşur; global 16 MiB bellek sınırı yoktur.

`prototype-*` dosyaları `compiler/target/k206-stream-prototype/` konumuna
Cargo.toml, Cargo.lock, src/main.rs ve src/contracts.rs adlarıyla kopyalanır.
Driver'lar compiler çalışma dizininde `target/` altına konur. Ardından:

```bash
cd compiler
CARGO_TARGET_DIR=target cargo build --locked --offline --release \
  --manifest-path target/k206-stream-prototype/Cargo.toml
python3 target/prototype-rss-driver.py
python3 target/prototype-contract-driver.py
python3 target/tls-pool-driver.py
```

RSS kaydındaki hash yolları özgün `target/k206-*` adlarını taşır; yeniden
koşuda kayıt adları ve hash'ler yeni deneye aittir. Driver her koşuda yalnız
kendi UUID konteynerini oluşturur/kaldırır. Kullanıcı veritabanı kullanılmaz.

`tls-pool-long.json` ve log mevcut native havuz testinin 336,840 saniyelik
başarılı koşusudur: dört backend, checkout exhaustion, stale replacement,
300 saniyelik aktif lease ve 30 saniyelik bakım. `tls-negotiated.txt` aynı
geçici sunucudan TLS 1.2 gözlemidir. CA/private key dosyaları geçicidir ve
silinmiştir; yalnız CA fingerprint'i saklanır. İlk self-signed sunucu
kurulumunda handshake başarısız olmuş, ayrı CA ve localhost leaf ile pozitif
koşu geçmiştir; ilk kurulum bir negatif hostname/CA test kanıtı sayılmaz.

## K-206 üretim adaptörü kabulü

`rss-native-after.json` temiz exact 6463a7f kaynakta gerçek Zee native
adaptörünü ölçer; `probe.rs` öncekiyle bayt bayt aynı, hash kayıttadır.
64×1 MiB sonuçta RSS 135.020.544→24.969.216 bayt (128,77→23,81 MiB),
tek 64 MiB satırda 143.409.152→76.251.136 bayt (136,77→72,72 MiB).
Her üç vakada sonraki sorgu başarılıdır. Bu örnek=1/ısınma=0 gözlemidir;
kalıcı performans tabanı veya global RSS garantisi değildir.

`native-acceptance.json`/log aynı temiz kaynakta production `oku` yolunu
eski 4a4fd0d Vec oracle'ıyla karşılaştırır: 15 sorgu sınıfı, TEXT parameter
bind ve transaction limit hatası sonrası sağlık. Satır sınırı NULL/tür/
byte hatasını, sonradan gelen sunucu hatası satır/NULL hatasını geçersiz
kılar. Uygulama hataları read retry sınıfına taşınmaz; mevcut write/commit
ve recovery sözleşmesi değişmedi. Sürekli native fault proxy matrisi bu
tek testin tamamladığı bir iş diye gösterilmez. Tek wire satırı ve GB-041
**açıktır**.

Reproducer için `native-probe-Cargo.*` dosyalarını compiler/target/
k206-native-probe altında Cargo.* adlarıyla, `probe.rs` dosyasını bu
klasörün src/main.rs yoluna kopyala. Native driver'ları compiler/target
altına koy; compiler çalışma dizininde:

```bash
CARGO_TARGET_DIR=target cargo build --locked --offline --release \
  --manifest-path target/k206-native-probe/Cargo.toml
python3 target/native-acceptance-driver.py
python3 target/native-rss-driver.py
```

Yeni koşu kendi exact SHA/temizlik/çevre kaydını üretir. Konteynerler
UUID adıyla yalnız test tarafından oluşturulur ve kaldırılır.

Yerel dependency source kimliği/iki-clone kabulü [buradadır](source-identity/README.md).
Mesaj üst sınırı henüz etkin değildir; sıradaki security dilimi bağlantı
kapanması/transaction kaybını ayrı typed hata ve native testle ele almalıdır.

İki-parser typed erken sınırın **etkin olmayan** kabul tasarımı ve transaction
gözlemi [ayrı kayıttadır](frame-guard/README.md). Üretim entegrasyonu açıktır.
