# K-206 mesaj sınırı: aday geçmişi ve native entegrasyon

**Tarihsel aday kaydı (c68af0f).** `backend.patch` dosyası üretimde seçilen immutable protokol
kaynağını değiştirmez. Compiler/fuzz lock, v2 yama kaydı ve release SBOM
hâlâ c4e9d036'nın değiştirilmemiş upstream kodunu seçer. GB-041 açık,
K-206 kısmi kalır. Bu dizin, bir sonraki security entegrasyonunun
tekrar çalıştırılabilir kabul tasarımıdır.

Yama iki bağımsız girişe uygulanır: `Header::parse` ve `Message::parse`.
Beş baytlık başlıktaki length alanı 32 MiB'ı aşarsa, gövde beklenmeden
ve `BytesMut::reserve` çağrılmadan typed `FrameLimitExceeded` döner.
Length dört baytlık length alanını içerir, bir baytlık tag'i içermez.
Bütün backend tag'leri kapsanır. Bu sınır TEXT sonuç bütçesi olan 16 MiB,
frontend paket boyutu, TLS record boyutu veya süreç RSS tavanı değildir.
32 MiB TEXT hücresi DataRow overhead'i yüzünden bu sınırı aşar.

Beş yeni saf vaka: parçalı başlık; exact limit/limit+1; iki parser'ın
aynı sınırı ve doğrudan parser'da değişmeyen tampon kapasitesi;
bozuk length'in eski hata türleri; normal mesajın doğru tüketilmesi.
Upstream'in 29 testiyle toplam 34 lib testi geçer. Bunlar production
compiler faz sayısına eklenmez; bağımsız tedarik/protokol kabul koşusudur.
`verify.py`, exact upstream tree hash'ini doğrular, yamayı fuzz=0 ile
geçici kopyaya uygular, locked/offline test + clippy + rustfmt çalıştırır,
üretim kaynağının değişmediğini denetler ve kendi dizinini temizler.
Yama sıfır bağlam satırlıdır; taban eşitliği exact tree hash ile sağlanır,
diff bağlamının boş satırları whitespace borcu oluşturmaz:

```bash
python3 docs/incelemeler/k206/frame-guard/verify.py
```

`probe.rs` gerçek postgres 0.19.14 sürücüsünde typed hatayı denetler.
`std::io::Error::get_ref()` incelenmeden yalnız `Error::source()` zinciri
taramak içteki custom hata türünü bulmaz; ilk deney bu yüzden başarısız
oldu, probe bu durumu explicit kapsar. Mesaj metninden sınıf tahmini yoktur.
1/16 MiB satırda okuma ve sonraki sağlık sorgusu geçer; 32/64 MiB satırda
limit cause, kapalı bağlantı ve başarısız sonraki sorgu beklenir.
64 MiB transaction vakasında INSERT'in başka bağlantıdan sayısı sıfırdır;
kapalı bağlantıdaki ROLLBACK yanıtı alınamaz. **Yerel Zee transaction durumunun
böyle bir durumda temizlenmesi veya yeniden denemenin engellenmesi bu
standalone probe tarafından gerçekleştirilmez.** Üretime geçiş kapısı budur.

Native reproducer: kaynak klasörünü compiler/target/k206-typed-frame-source
altına kopyala ve backend.patch'i `patch --batch --fuzz=0 -p1` ile uygula.
probe-Cargo.* dosyalarını compiler/target/k206-typed-frame-probe altında
Cargo.* adlarıyla, probe.rs dosyasını src/main.rs olarak yerleştir. Compiler
çalışma dizininde Rust 1.93.1 ile:

```bash
CARGO_TARGET_DIR=target cargo build --release --locked --offline \
  --manifest-path target/k206-typed-frame-probe/Cargo.toml
python3 ../docs/incelemeler/k206/frame-guard/native-driver.py
```

Native driver temiz exact HEAD ister; yalnız UUID geçici loopback PostgreSQL
16.15 konteynerini oluşturup finally ile kaldırır. Başka DB/konteynere dokunmaz.
Bir örnek/sıfır ısınma RSS gözlemi kalıcı benchmark veya hard eşik değildir.
Sonuç ignored `compiler/target/k206-typed-frame-prototype.json` yoluna gider;
exact kapanışın ham kanıtı takip eden committe bu dizine eklenir.

Etki incelemesi: çalışan grammar/spec, tanı/API/ABI, Rust export, owner/SCC,
release ikilileri/SBOM/provenance ve soak davranışı değişmez. Normatif spec
kanıtı, dil yüzeyi ve faz matrisi değiştirilmez. ADR-060 sınırın açık oluşunu,
backlog/master plan/V1 kapıları/sürüm notu tamamlanmamış entegrasyonu belirtir.
Yeni grammar/RFC kararı yoktur. Root README özellik düzeyi yükseltilmez;
compiler sayıları bu bağımsız deney testleriyle şişirilmez. Performans tarihçesi,
ADR-045/049 ve ADR-046 fixture'ı yeni kaynak/kalıcı iş yükü olmadığından değişmez.

## Üretim entegrasyonu için açık kabul matrisi

Bu tablo taslaktır; mevcut Zee davranışı olarak sunulmaz.

| Durum | Entegrasyonda doğrulanacak sonuç |
| --- | --- |
| Transaction dışında limit cause | Normal kapalı bağlantı read retry yoluna girmez; hata döner, sonraki ayrı okuma yeni checkout ile yapılabilir. |
| Açık veya iç içe transaction'da limit cause | Lease bırakılır, yerel transaction geçersiz işaretlenir; okuma/yazma/BEGIN/COMMIT yeni bağlantıya taşınmaz. |
| Geçersiz transaction geri alma | Yerel derinlik ve request-local bağlam temizlenir; ROLLBACK onayı alınmış gibi başarı döndürülmez. Yeni transaction ayrıca başlatılır. |
| RLS request bağlamında limit | Eski tenant bağlamı yeniden checkout ile sürdürülmez; cleanup sonrası yeni request RLS kurulumu gerekir. |
| COMMIT gerçekten gönderildi, sonra bağlantı koptu | Mevcut `db.commit_unknown`/C027 davranışı korunur; typed cause olması kesin rollback sonucu iddiasına dönüşmez. |
| Limitten önce uygulama sonucu hatası, sonra wire kopması | Transport hatasının mevcut önceliği korunur; uygulama limiti diye bağlantı tekrar kullanılmaz. |
| 64×1 MiB toplam sonuç limiti | Her frame wire sınırının altında; mevcut 16 MiB uygulama limiti, akış tüketimi ve sonraki sağlık korunur. |

Entegrasyon, native okuma/yazma/nested/RLS/commit testleriyle birlikte
spec-25, C025 hata verisi, uyumluluk yüzeyi, exact kaynak pin/tree kaydı,
offline vendor ve iki-clone artefakt kanıtını aynı dilimde ele almalıdır.
Yalnız Header yamasını açıp bu matrisi atlamak GB-041'i kapatmaz.

## Temiz kaynak kapanışı

Exact `c68af0fa071fa988e5b38757bacf6bd24b99e37f` temiz kaynakta [34-test ham koşusu](unit.log)
ve [native 5-vaka JSON](native.json) geçti. 32/64 MiB reddinde ölçülen
RSS 6.750.208 bayttır (yaklaşık 6,44 MiB); transaction INSERT sayısı
sıfırdır. Bir örnek/sıfır ısınma, izole NoTLS loopback, macOS arm64
kabulüdür. Native JSON input/ikili hash, Rust ve OS kimliğini taşır.
735/735 mevcut compiler fazı, katalog/tazelik/fmt, güvenlik ve sır
kapıları geçti. Bu deney RC fuzz/Miri, imzalı release veya üç-platform
wire entegrasyonu kanıtı değildir. GB-041 açık; aday üretimde etkin değildir.

Hazırlık kopyası .2 guard taşımak üzere güncellendiğinde `verify.py`
patch tabanını exact c4e9d036 Git bloblarından yeniden oluşturur; güncel
hazırlık ağacını upstream sanmaz. Aktif pin taşınana kadar üretim .1 kalır.

## Etkin Zee entegrasyonu

Compiler/fuzz artık exact `1ffdb5846ef1f5022bbf3c695d6c70abcde85364`
Git kaynağından .2 protokolünü seçer; v2 tree hash
`dcddfbcde306fcf096dc0785c2ffa6338ef14ae63eaf624ce3eb783d851f08f5`tir.
C025 typed hata verisi ve geçersiz transaction cleanup uygulanmıştır.
`zee-native-driver.py` gerçek root adaptörünün 13 testini, eski stream
oracle dahil, çalıştırır. Volatile sunucu sequence'i read'in bir kez
çalıştığını denetler; Notice/write, nested/RLS cleanup ve gönderilmiş
COMMIT belirsizliği ayrıca kapsanır. Opt-in env olmadan CI bu iki native
saha testinde bağlantı kurmaz; ayrı izole koşu ham kanıtı gereklidir.

`zee-probe-Cargo.*`, önceki native RSS probe kaynak byte'larını koruyup
yeni Git patch'i standalone tüketiciye açıkça bağlar. Root patch'i
tüketici manifestine miras kalıyor diye varsayılmaz. `zee-rss-driver.py`
aynı 1×1 MiB, 64×1 MiB, 1×64 MiB vakalarını ölçer. Ham exact temiz
source/native/RSS/release kapanışı takip eden committe kaydedilir.
İlk bölümdeki etkin değil ifadesi eski adayın tarihidir; güncel pin
üretim kaynak kodunda etkindir. Dağıtım/RC etiketi yayımlanmaz.

## Etkin kaynak kapanışı

Clean exact `a4e104589477e0e786ff126513b6bdd6d1228547` üzerinde [13 native test](zee-native.json),
[ham test logu](zee-native.log), [aynı probe ile native RSS](zee-rss.json),
[163 paket offline vendor](vendor.log) ve [iki-clone release kanıtı](release/accepted.log)
geçti. Bir 64 MiB satır native RSS 7.077.888 bayttır (6,75 MiB);
sonraki bağımsız sorgu yeni checkout ile sağlıklıdır. 64×1 MiB vakasında
uygulama limiti/akış tüketimi de korunur. Samples=1/warmup=0 gözlemidir.
735→736 gerçek platform faz vakası geçti; static Rust+doctest 738'dir.
Yeni source için yerel macOS kabulüdür; üç-platform RC/fuzz/Miri veya
signed release değildir. GB-041 tanımlı tek-frame tehdidi kapandı;
global RSS ve sürekli geniş fault-proxy matrisi sözü verilmez.
