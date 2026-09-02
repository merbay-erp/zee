# ADR-038 — Rust tedarik zinciri ve offline vendor kapısı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-141, B-054, V1-P1-09

## Bağlam

Derleyicinin `Cargo.lock` dosyası ve crates.io checksum'ları vardı; ancak CI,
kilitli test başarısız olunca kilitsiz teste düşebiliyordu. RustSec duyurusu,
lisans, yinelenen sürüm, wildcard veya bilinmeyen registry/Git kaynağı için
fail-closed kapı yoktu. İnternetsiz derleme yalnız yerel Cargo cache'iyle
denenmiş, aynı kilitten üretilen vendor içeriğinin kararlılığı kanıtlanmamıştı.

Zee'nin kendi kod lisansı henüz hukuk/topluluk kararı bekler. Bu karar,
bağımlılık lisans kabul kümesiyle karıştırılamaz ve otomatik seçilemez.

## Karar

1. Bootstrap compiler ve fuzz paketi `publish = false` kalır. Proje lisansı
   seçilene kadar crates.io yayını yapılamaz; bağımlılık politikası Zee'ye bir
   lisans atamaz.
2. `Cargo.lock` şema 4 dosyaları izlenir. Her registry girdisi yalnız crates.io
   kaynağı ve 64 küçük-hex SHA-256 checksum taşır. CI'daki bütün normal Cargo
   komutları `--locked` kullanır; kilitsiz fallback yasaktır.
3. `cargo-deny 0.20.2`, push/pull request ve günlük takvimde hem compiler hem
   fuzz grafiğini bütün hedefler için tarar. RustSec ignore kümesi boştur;
   bilinmeyen registry/Git, wildcard ve gerekçesiz politika istisnası hatadır.
4. Kabul edilen bağımlılık lisansları `deny.toml` içindeki dar izin kümesidir.
   Bu statik tarama hukuk görüşü değildir; yeni lisans açık inceleme olmadan
   kümeye eklenemez.
5. Çoklu sürüm varsayılan olarak reddedilir. Yalnız exact sürüme bağlı,
   gerekçeli `getrandom 0.2.17` ve `syn 2.0.119` geçiş istisnaları vardır.
   İstisna artık grafikte yoksa `-D warnings` kapıyı kırar ve kayıt silinir.
6. `offline-vendor-denetle.sh`, iki lock'u birlikte `--locked
   --versioned-dirs` ile iki geçici vendor ağacına açar. Sıralı yol+dosya
   SHA-256 manifestleri eşit değilse durur. Ardından boş `CARGO_HOME`, ayrı
   target ve `--offline --locked` ile compiler'ın bütün hedeflerini ve fuzz
   ikililerini derler; iki lock'un da değişmediğini doğrular.
7. Vendor ağacı Git'e alınmaz. Her release adayı temiz ortamda bu kapıyı koşar;
   gerekiyorsa üretilen vendor ağacı, o release'in lock dosyaları ve manifest
   özetiyle birlikte ayrı offline kaynak artefaktı olarak yayımlanır.

## Reddedilen seçenekler

- **Yalnız `cargo audit`:** advisory tarar; lisans, wildcard, duplicate ve
  kaynak kökeni politikasını tek kapıda toplamaz.
- **Kilit bozulunca kilitsiz çözümleme:** CI'yı yeşil gösterebilir ama
  incelenmemiş bağımlılık grafiği üretir.
- **Vendor ağacını kalıcı olarak Git'e almak:** kaynak tarihini binlerce üçüncü
  taraf dosyasıyla büyütür. Lock+checksum ve release-time offline artefaktı aynı
  kanıtı daha dar sahiplikle verir.
- **Zee lisansını bağımlılık allowlist'inden çıkarsamak:** ürün lisansı hukuki
  ve yönetişimsel karardır; araç bunu veremez.

## Sonuçlar

- Güncel advisory verisi bilinçli olarak hareketlidir; yeni RustSec kaydı
  değişmeyen commit'i kırabilir ve bu güvenlik uyarısıdır, tekrar üretim hatası
  değildir.
- Araç sürümü, Rust sürümü, dependency graph ve kaynak checksum'ları sabittir.
  Bağımlılık yükseltmesi lock, deny sonucu, offline vendor kanıtı, test ve belge
  güncellemesini aynı committe ister.
- K-141 dört statik regresyon, gerçek iki-grafik `cargo deny` koşusu ve iki
  vendor üretimi + boş-cache offline derlemeyle B-054'ü kapatır.

## K-151 tamamlayıcı karar

ADR-048 Cargo bağımlılık zincirinden ayrı olarak bütün GitHub Actions
`uses:` referanslarını immutable commit SHA'ya ve makine-okunur pin kaydına
bağladı. Haftalık Dependabot yalnız güncelleme PR'ı açar; otomatik merge yoktur.
