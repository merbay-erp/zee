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
2. **K-192 açık:** Linux/macOS/Windows release ikililerinde sistem kitaplığı
   envanteri, linker/SDK provenance sınırı ve Rust bulunmayan makinede kurulum
   provası. Mevcut kurulum tatbikatının compiler dev ortamından bağımsızlığı
   ayrıca değerlendirilir; saha kanıtı olmadan kapanmaz.
3. **K-193 açık:** native/WASM kod üretimi için gerçek ürün ihtiyacı, mevcut
   semantik korpusuyla eşdeğerlik ölçütleri ve backend karar verisi.
   [Backlog](oncelikli-backlog.md), CORE FREEZE ve insan kanıtı kapıları korunur;
   henüz backend seçilmez veya yeni CLI/grammar açılmaz.
4. **Self-hosting açık:** [ADR-008](../adr/008-self-hosting-asamalari.md)
   aşamalarına göre önce Zee kitaplığı, ardından derleyici parçaları;
   Stage 0 bootstrap güven kökü olarak korunur.
