# Production katman ve bağımlılık yönü

K-149/ADR-046, fiziksel satır bütçesinin yanına gerçek modül bağımlılığı
kapısını koyar. Amaç yalnız küçük dosya değildir: parser semantic/runtime
ayrıntısını, checker runtime/adaptör ayrıntısını, runtime parser/checker/LSP
ayrıntısını ve LSP compiler iç geçişlerini doğrudan bilmemelidir.

## Katman yönü

| Katman | Doğrudan bağımlanabileceği katmanlar |
|---|---|
| `temel` | `temel` |
| `model` | `temel`, `model` |
| `altyapi` | `temel`, `model`, `altyapi` |
| `sozdizimi` | `temel`, `model`, `altyapi`, `sozdizimi` |
| `semantik` | önceki dört katman + `semantik` |
| `proje` | temel→semantik + `proje` |
| `web` | temel/model/altyapı + proje + `web` |
| `runtime` | bütün ürün alt katmanları + `runtime` |
| `adapter` | ürün katmanlarının tamamı ve adaptörler |
| `muhendislik` | bütün katmanlar; ürün kodu buna bağımlanamaz |

Bu yön tek tek Rust dosyalarının sırasını değil sorumluluk seviyesini anlatır.
Aynı `proje` kümesindeki paket/registry/tedarik koreografisi bu kararla
çevrimsiz ilan edilmez; mevcut doğrudan kenarlar ayrıca exact tabanda
görünürdür. Katmanlar arası ters kenar ise tabana yazılsa bile reddedilir.

## Makine-okunur sahiplik

[`katman-mimarisi-v1.tsv`](../compiler/tests/fixtures/katman-mimarisi-v1.tsv)
35 production sahibinin katmanını, bütün doğrudan iç bağımlılıklarını ve tek
cümlelik sorumluluğunu taşır. Kapsam şunların tamamıdır:

- `lib.rs`, bütün kök kitaplık modülleri ve alt modülleri;
- `main.rs` ile `cli/` tek `dil_cli` adaptörü;
- `dillsp` ürün adaptörü;
- depo sayısı, faz matrisi, işlev eğilimi ve performans gözlemi mühendislik
  ikilileri.

Yeni `.rs` dosyası mevcut üst sahibine girer. Yeni kök modül veya ikili yeni
sahip satırı ister. Test-only `testler.rs` dosyaları ile `#[cfg(test)]`
öğeleri production kenarı sayılmaz; hedefe özgü production kodlarının birleşik
kenarları ise bütün Tier-1 platformlar için korunur.

`katman_mimarisi_testi` Rust yorumlarını, iç içe blok yorumlarını, normal/byte/
raw metinleri ve karakter sabitlerini token akışından çıkarır; yaşam sürelerini
kodla karıştırmaz. `crate::`, `dil::`, toplu `use` ve doğrudan bilinen modül
yollarını toplar. `crate`/`dil` kökünü takma adla gizlemek reddir. Kök public
derleme, proje-yükleme ve runtime sembolleri kavramsal sahiplerine eşlenir;
bilinmeyen kök sembolü `lib` cephesine gider ve aşağı katmanda ters kenar olur.

Kapı üç ayrı drift'i durdurur:

1. kaynak ağacındaki sahiplerle fixture birebir değilse yeni/kayıp modül;
2. bulunan doğrudan kenarlar exact, sıralı bağımlılık tabanından farklıysa
   eklenen veya kaldırılan bağımlılık;
3. exact taban güncellense bile kaynak→hedef katmanı izinli yönde değilse
   ters katman bağımlılığı.

## K-149'da temizlenen iki ters kenar

- FIPS 180-4 `sha256_hex`, paket modülünden temel `guvenlik` sahibine taşındı.
  Böylece morfoloji profil parmak izi artık `sozdizimi → proje/paket` ters
  bağımlılığı kurmaz; paket, registry, tedarik, web ve IO izi aynı temel ilkeyi
  tüketir.
- Gregoryen gün↔tarih algoritması runtime'dan yeni temel `zaman` sahibine
  taşındı. Tedarik zinciri RFC 3339 zamanı için runtime'a bağımlanmaz;
  yorumlayıcı ve CLI aynı yardımcıyı kullanır. Eski
  `yorumlayici::gunlerden_tarih_utc` yolu geriye uyumlu yeniden dışa aktarılır.

Güncel graph ayrıca şu sınırları makinece gösterir: `ayristirici` semantic/HIR/
runtime'a; `cozumleyici` runtime/LSP'ye; `yorumlayici` lexer/parser/checker/LSP/
WASM'a; `lsp` lexer/parser/checker/runtime'a doğrudan bağımlı değildir.

## Değişiklik protokolü

```bash
cd compiler
cargo test --locked --test katman_mimarisi_testi
cargo run --locked --bin faz_test_matrisi -- --denetle \
  --rapor target/faz-test-matrisi.md
```

Bir bağımlılık farkında fixture körlemesine güncellenmez. Önce sorumluluğun
mevcut sahibinde kalıp kalmadığı, daha aşağı bir ortak ilkeye taşınıp
taşınamayacağı ve katman yönü değerlendirilir. Kabul edilen yeni kenar aynı
committe ADR/rehber etkisi ve ilgili davranış testleriyle girer. Kaldırılan
kenar da tabandan silinir; artık kullanılmayan bağımlılık gelecekteki kod için
sessiz izin olarak bırakılmaz.

Bu kapı Rust tip denetiminin veya semantic testlerin yerine geçmez. Makro ile
üretilmiş dinamik adları tam bir Rust semantic graph'ı olarak iddia etmez;
production kaynakta açık modül yollarının değişim kapısıdır. Tam faz matrisi,
Clippy, rustdoc, WASM ve ilgili RFC/spec kanıtları ayrıca zorunludur.
