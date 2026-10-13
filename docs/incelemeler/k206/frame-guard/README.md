# K-206 erken mesaj sınırı kabul tasarımı

**ETKİN DEĞİL.** `backend.patch` üretimde seçilen immutable protokol
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
