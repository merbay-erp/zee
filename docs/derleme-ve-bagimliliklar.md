# Derleme ve bağımlılık envanteri

K-191, 12 Ekim 2026. İnceleme tabanı `f3276b9bb4a286ca897bc45fd0aca7c622f1e5c6`;
manifest `compiler/Cargo.toml`, çözüm `compiler/Cargo.lock`, araç zinciri
`compiler/rust-toolchain.toml`. Bu sayfa mevcut kapasiteyi ve sonraki işleri
ayırır; native backend veya self-hosting tamamlanmış değildir.

## Üç ayrı bağımlılık sınırı

| İş | Mevcut gereksinim | Açık sınır |
|---|---|---|
| Bootstrap derleyicisini kaynaklardan üretmek | Rust 1.93.1, Cargo, platform linker/SDK, lock'taki Rust kitaplıkları | Rust araç zinciri gereklidir; vendor yalnız crate kaynaklarını sağlar |
| Zee programını hazır `dil` ile çalıştırmak | Platforma uygun `dil`, işletim sistemi kitaplıkları; kullanılan ürüne göre dosya/ağ/veritabanı | Rust/Cargo kurulumu gerekmez; ikilinin bütün sistemlerde statik olduğu iddia edilmez |
| Zee kaynağından bağımsız native/WASM çıktı üretmek | Henüz gerçeklenmedi | Şimdiki `kaynagi_derle` bağlı program üretir; ağaç yürüyen yorumlayıcı çalıştırır |

Playground'daki `dil.wasm`, bootstrap derleyicisinin Rust'tan WASM'e derlenmiş
halidir. Zee programının doğrudan WASM modülüne çevrilmesiyle aynı kapasite
değildir. Zee'nin kendi paket bağımlılıkları (`proje.dil`, exact uzak sürümler,
`proje.kilit` v3) Cargo bağımlılıkları değildir; sözleşmeleri
[spec/07](../spec/07-birimler.md) ve [spec/19](../spec/19-registry-metadata-guveni.md).

## Doğrudan Rust bağımlılıklarının sorumlulukları

| Hedef | Crate | Sorumluluk |
|---|---|---|
| Native ve WASM | `num-bigint`, `num-traits` | Kesin sayısal altyapı |
| Native | `argon2`, `getrandom`, `rpassword` | Parola özeti, OS rastgeleliği, terminalde gizli giriş |
| Native | `ed25519-dalek`, `sha2` | Yayın/metadata imzası ve özet |
| Native | `serde`, `serde_json` | Kapalı metadata şemaları ve bakım çıktıları |
| Native | `unicode-normalization` | NFC normalizasyonu |
| Native | `ureq` (rustls) | HTTPS outbound/registry taşıması |
| Native | `postgres`, `postgres-native-tls`, `native-tls` | PostgreSQL ve TLS adaptörü |
| Native | `r2d2`, `r2d2_postgres` | Sınırlı bağlantı havuzu |
| Unix | `libc` | İşletim sistemi/metadata adaptörleri |

Exact geçişli sürümler ve checksum'lar Cargo.lock'tan alınır; manifestteki
SemVer aralığı `--locked` çalışmada bağımlılık yükseltmez. Kriptografi, TLS
ve genel sayı altyapısını yeniden yazmak bağımsızlık hedefinin önkoşulu
değildir ([ADR-001](../adr/001-bootstrap-dili.md)).

`native-tls` nedeniyle native ikililerin platform kitaplıkları ayrıca
incelenmelidir. Crate SBOM'u linker/SDK veya sistemde yüklenen bütün kitaplıkları
listelediği anlamına gelmez. K-192 bu gerçek ikili sınırını ölçmeden
kurulumsuz/statik dağıtım sözü genişletilmez.

## Araç zinciri ve provenance düzeltmesi

K-191 öncesinde kökten başlatılan sürüm betiği host/rustc/sysroot bilgisini
varsayılan Rust'tan okurken klonları `compiler/` altındaki pinle derliyordu.
İnceleme makinesinde varsayılan 1.98.1, proje pini 1.93.1 idi.

Ortak `scripts/rust-arac-zinciri.sh` tek exact `X.Y.Z` pinini doğrular,
`RUSTUP_TOOLCHAIN` değerini o pine bağlar ve çalışan `rustc` sürümünü kontrol
eder. Sürüm üretimi pinini üretilen exact HEAD'den alır; `--kirli-kabul` yerel
pin değişikliğini provenance'a taşımaz. Vendor denetimi incelenen çalışma
ağacının pinini kullanır. Derleme, sysroot remap ve metadata aynı araç
zincirini kullanır. Eksik araç zinciri ve uyuşmayan rustc hata verir;
betik otomatik araç zinciri kurmaz.

`bash scripts/tests/rust-arac-zinciri-test.sh` farklı varsayılan, stable/nightly,
eksik/yinelenen pin, uyuşmayan rustc, kirli yerel pin, iki klon ve boş Cargo
home girişlerini sahte araçlarla sınar. Bu, betiklerin yönlendirme regresyonudur;
gerçek derleme ve çift-klon hash kanıtının yerini tutmaz.

## Kanıtı yeniden üretmek

```bash
cd compiler
rustc --version
cargo metadata --locked --offline --format-version 1
cargo tree --locked --offline --target wasm32-unknown-unknown --depth 1
cargo check --locked --target wasm32-unknown-unknown --lib
cd ..
bash scripts/tests/rust-arac-zinciri-test.sh
bash scripts/offline-vendor-denetle.sh
# Temiz exact commit'te gerçek iki-klon artefakt kanıtı:
bash scripts/surum-artefakti.sh
```

İlk kaynak edinimi ve Rust hedef bileşenlerinin kurulması ağ gerektirebilir;
boş Cargo home vendor kapısı önceden kurulu Rust/linker/SDK'yi içermez.

## Bağımlılık sırasıyla sonraki işler

1. **K-191 kapalı kapsam:** derleme/bağımlılık envanteri ve araç zinciri
   provenance tutarlılığı. Dil semantiği, crate sürümleri ve SBOM şeması değişmez.
2. **K-192 kısmen açık:** Linux/macOS/Windows release ikililerinde sistem kitaplığı
   envanteri, linker/SDK provenance sınırı ve Rust bulunmayan makinede kurulum
   provası. Mevcut kurulum tatbikatının compiler dev ortamından bağımsızlığı
   ayrıca değerlendirilir; saha kanıtı olmadan kapanmaz.
   [Yerel kanıt](derleme-bagimlilik-kaniti-2026-10-12.md) macOS release
   ikilisinin sistem kitaplıklarını ve Rust olmayan PATH ile çalışmasını
   ölçtü; kaynak kimliği kanonik depoya bağlandı. Üç platform tatbikatı, temiz Linux container runtime ve uzak exact kaynak
   erişimi kanıtlandı; temiz Windows/macOS ve minimum OS sınırları açıktır.
3. **K-193 açık:** native/WASM kod üretimi için gerçek ürün ihtiyacı, mevcut
   semantik korpusuyla eşdeğerlik ölçütleri ve backend karar verisi.
   [Backlog](oncelikli-backlog.md), CORE FREEZE ve insan kanıtı kapıları korunur;
   henüz backend seçilmez veya yeni CLI/grammar açılmaz.
4. **Self-hosting açık:** [ADR-008](../adr/008-self-hosting-asamalari.md)
   aşamalarına göre önce Zee kitaplığı, ardından derleyici parçaları;
   Stage 0 bootstrap güven kökü olarak korunur.

## K-192 otomatik çalışma ortamı kapısı

`python3 scripts/calisma-ortami-denetle.py --artefakt <surum> --kurulu <bin>
--rapor <rapor.json> --beklenen-sha <tam-sha>` (`Windows: python`) kurulu
`dil`/`dillsp` hash'lerini artefakt manifestiyle ve kaynak SHA'yı beklenen
commit ile karşılaştırır. Fark varsa ikili çalıştırmadan kapı kırılır.

Linux `readelf`, macOS `otool -L` ve Windows `dumpbin /DEPENDENTS` doğrudan
sistem bağımlılıklarını raporlar. Windows araç bulma
[Microsoft vswhere tarifine](https://github.com/microsoft/vswhere/wiki/Find-VC),
DLL listesi [resmî dumpbin sözleşmesine](https://learn.microsoft.com/en-us/cpp/build/reference/dependents)
dayanır. Python ve inceleme araçları kanıt üreticisidir; Zee programının
runtime bağımlılığı olarak sunulmaz.

Ayrı çocuk süreçlerinin PATH'i yalnız sistem dizinleridir; `rustc`, `cargo`,
`rustup` bulunursa prova reddedilir. `dil sürüm`, exact merhaba-dünya çıktısı
ve gerçek `dillsp` initialize/shutdown/exit yanıtları kontrol edilir.
`zee-calisma-ortami-1` raporu başarılı/başarısız durum, kaynak SHA, ikili
hash'leri, OS/mimari, doğrudan kitaplıklar, ABI header gözlemleri ve prova
sınırını taşır. Hata raporu başarı gibi sunulmaz.

`kurulum-tatbikati` bu kapıyı üç platformda kurulu ikili üzerinde çalıştırır;
rapor exact commit+OS adıyla 90 günlük ayrı artefakta gider. Sürüm artefakt
kümesi/SBOM şeması değişmez. Saf ret regresyonları
`scripts/tests/calisma-ortami-test.py` ile farklı kaynak, değiştirilmiş ikili
ve yinelenen checksum'ı çalıştırmadan reddetmeyi kanıtlar.

İlk macOS prova K-191'in exact release ikililerinde geçti. Güncel üç platform sonuçları aşağıdaki kalıcı deftere kaydedildi. Builder'da Rust'ın kurulu
olmaya devam etmesi, minimum OS ve gerçek linker/SDK üretim provenance'ı
ayrı açık sınırlardır; PATH yalıtımı bunların yerine geçmez.

İlk uzak tatbikat `3bf59d6dd4641ba91fa983048f051cbbda6b70d3` kaynağında
Linux/macOS workflow'u geçti; Windows raporundaki program/LSP sonucu geçti,
ama log satırının CP1252 kodlanması workflow'u kırdı. K-192 log özeti ASCII,
JSON raporu UTF-8 olarak ayrıldı; ret regresyonları `PYTHONIOENCODING=ascii:strict`
altında da çalışır. Güncel `30dcfae` kaynağında üç platform koşusu geçti.

Linux `dil` doğrudan `libssl.so.3`/`libcrypto.so.3` ister; Windows ikilileri
`VCRUNTIME140.dll` ve UCRT/API-set kitaplıklarını ister. Bu gerçek bağımlılıklar
"yalnız tek statik ikili" sözü değildir. DLL/ELF listesi runtime yükleme
başarısıyla birlikte değerlendirilir. Ayrı tatbikat artefaktı JSON ile o
koşunun mevcut `surum/` kümesini birlikte saklar; böylece indirilen byte'lar
rapor hash'leriyle doğrulanıp ayrı runtime ortamında tekrar sınanabilir.

## Üç platform kanıtı ve temiz Linux runtime

`30dcfaeec34f2bb992719f7a210bdf3138a226b0` kaynak commit'inin
[38238657801 tatbikatı](https://github.com/itwise-bilisim/zee/actions/runs/38238657801)
Linux/macOS/Windows'ta başarılıdır. Üç rapor ve byte özeti
[platform kanıt defterinde](derleme-platform-kaniti-v1.tsv) kalıcıdır;
`kurulum_testi` kaynak/hash, schema, ikili listesi, Rust'sız PATH program ve
LSP gözlemlerini denetler. Archive zip digest'leri GitHub artifact metadata'sıdır;
indirilen ikili byte'ları rapor/SHA256SUMS ile ayrıca eşleştirildi.

Aynı Linux artefaktı Ubuntu amd64
`sha256:f610ab94648195aa356059f5b41d6085c9d4d903c072430cdd1af7bdb646106b`
image'ının yeni container'ında `--read-only --network none` ile çalıştı.
Yalnız artefakt ve merhaba kaynağı read-only bağlandı; `rustc`/`cargo` paketleri
ve PATH'te `rustc`/`cargo`/`rustup` yoktur. Program exact çıktı, stdio LSP
initialize/shutdown/exit geçti. Runtime Ubuntu 24.04.5 x86_64; libc6
`2.39-0ubuntu8.9`, libgcc-s1 `14.2.0-4ubuntu2~24.04.1`, libssl3t64
`3.0.13-0ubuntu3.15`. macOS arm64 Docker hostunda amd64 emülasyonu kullanıldı;
bu performans ya da bare-metal minimum OS ölçümü değildir. Windows/macOS
temiz fiziksel makine ve minimum OS/linker-SDK provenance sınırları açıktır.

Kalıcı JSON raporları `.gitattributes` içinde binary olarak işaretlidir;
Windows CRLF dahil indirilen ham byte'lar Git satır sonu dönüşümünden korunur.
Defterdeki SHA-256 normalize edilmiş JSON metninin değil ham raporun özetidir.

## K-195 Windows dağıtım profili — CI kanıtı açık

Önceki `30dcfae` Windows artefaktı VCRUNTIME140/UCRT ister; bu tarihsel
kanıt yeniden yazılmaz. Yeni MSVC dağıtım tarifi statik CRT ve explicit
`--target <exact-host>` kullanır; yalnız `surum-artefakti.sh` release
üretiminde etkin, geliştirme/Cargo varsayılanı aynı kalır. Üç platform
kurulum tatbikatı aynı iki-klon scriptini çağırır. Profil provenance
`cargoArgs`/`crtLinkage`/`targetRustflags` alanlarıyla izlenir; Windows
runtime kapısı `static` kaydının dinamik CRT DLL importu taşımadığını da
kontrol eder. Sistem DLL bağımlılığı ve minimum OS sınırı sürer.

Rust bakım ikilisindeki değişiklik dil semantiği maintenance kapsamındadır;
yeni grammar, komut, tanı veya gömme API exportu yoktur. ADR-046/katman
fixture ve bağımlılık çevrim grafı yeni modül/kenar almaz. RFC/spec/hata
kataloğu normatif dil davranışı açısından incelendi ve değişmedi. Release
profilinin performans etkisi henüz ölçülmedi; ADR-045/049 uyarınca eski
benchmark satırları taşınmaz veya yeni baseline uydurulmaz.

Windows Git Bash üreticisi klon/Cargo home/sysroot yollarını `cygpath -m`
ile native rustc yol biçimine getirir; remap iki klonun farklı MSYS
yollarını ikiliye sızdırmamalıdır. Bu da çift-klon eş-hash kapısındadır.

Release bayrakları [Cargo encoded rustflags sözleşmesi](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags)
ile ayrı argümanlar halinde taşınır; boşluklu remap yolları parçalanmaz ve
caller encoded flags seçilen CRT/remap tarifini ezemez. Giriş regresyonu
caller bayrağıyla Windows tarifini de denetler.
