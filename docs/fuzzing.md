# Compiler fuzz rehberi

Bu rehber K-110/ADR-022 ve K-111'in ortak işletim sözleşmesidir. Lexer/parser
hedefi geçerli her UTF-8 kaynağın token/AST ya da Türkçe tanı üretmesini;
morfoloji hedefi geçerli her üretilmiş kök+ek zincirinin aynı soyut çözüme
dönmesini ve çoklu köklerin sessizce seçilmemesini arar.

## Kalıcı katmanlar

- `compiler/fuzz/fuzz_targets/lexer_parser.rs`: libFuzzer hedefi. Lexer başarılı
  olursa aynı token akışını normal ve K-113/ADR-024 cümle+girinti
  senkronizasyonlu hata-kurtarmalı parser'dan geçirir.
- `compiler/fuzz/corpus/lexer_parser/`: sekiz başlangıç girdisi. Geçerli
  programın yanında Unicode, emoji, combining im, girinti, sayı, virgül, metin
  ve iç içe blok saldırıları taşır.
- `compiler/fuzz/fuzz_targets/morfoloji.rs`: geçerli Zee kökünü byte girdiden
  yüksek verimle üretir; seçilmiş tek/iki katmanlı eki giydirip üret→çöz
  değişmezini ve bütün adaylarla A001/A002 kararını denetler.
- `compiler/fuzz/corpus/morfoloji/`: düz, Türkçe, yumuşama ve uzun tanımlayıcı
  sınıflarını başlatan dört tohumdur.
- `compiler/fuzz/dictionaries/zee.dict`: Türkçe kalıpları ve kritik byte
  dizilerini mutation sözlüğüne verir.
- `compiler/tests/fuzz_korpusu_testi.rs`: stable ve bütün Tier-1 işletim
  sistemlerinde korpusu, 4.096 deterministik UTF-8 bileşimini ve 64 KiB uç
  örnekleri her `cargo test` koşusunda yeniden oynatır.
- `compiler/tests/morfoloji_testi.rs`: 4.096 deterministik kök × bütün geçerli
  ek zincirlerini, 2.048 bütün-aday belirsizlik vakasını ve NFC/NFD lexical
  sınırını her ana testte yeniden oynatır.

## Yerel koşu

Araç sürümleri politika gereği sabittir:

```sh
rustup toolchain install nightly-2026-08-31 --profile minimal
cargo +nightly-2026-08-31 install cargo-fuzz --version 0.13.2 --locked
cd compiler
cargo +nightly-2026-08-31 fuzz run lexer_parser fuzz/corpus/lexer_parser -- \
  -dict=fuzz/dictionaries/zee.dict -max_len=65536 -timeout=5
cargo +nightly-2026-08-31 fuzz run morfoloji fuzz/corpus/morfoloji -- \
  -dict=fuzz/dictionaries/zee.dict -max_len=128 -timeout=5
```

Kısa doğrulama için sona `-max_total_time=30`, uzun yerel çalışma için uygun
bir saniye bütçesi eklenir. Fuzzer'ın `DONE` ile ve sıfır koduyla bitmesi temiz
koşudur.

## Crash işlemi

1. `compiler/fuzz/artifacts/<hedef>/` altındaki girdiyi aynı hedefe tek dosya
   olarak verip yeniden üret.
2. `cargo fuzz tmin <hedef> <artifact>` ile girdiyi küçült.
3. Küçük girdiyi kalıcı korpusa ekle; davranış belirliyse ayrıca adı konmuş
   integration testi yaz.
4. Düzeltmeden sonra ana testleri, Clippy'yi, WASM'ı ve en az 30 saniyelik
   fuzz smoke koşusunu geçir.

Gece işinde korpus cache ile sonraki koşuya taşınır. Başarısız koşunun crash
girdisi GitHub artifact'ı olur; yalnız logda kalan ve tekrar üretilemeyen bulgu
kapatılmış sayılmaz.

Derlenmiş `compiler/fuzz/target/`, crash `artifacts/` ve coverage çıktıları
kaynak arşivine girmez. Kalıcı korpus ile fuzz kaynakları korunur; paylaşılacak
proje kopyası çalışma klasörünü sıkıştırmak yerine
[temiz kaynak arşivi](temiz-kaynak-arsivi.md) komutuyla üretilir.

## Sınırlar

Lexer/parser hedefi `&str` aldığı için geçersiz UTF-8 byte dizileri burada değil
dosya okuma sınırında reddedilir. 64 KiB sınırı bir dil dosyası boyut sınırı
değildir. Morfoloji hedefinin 128 byte girdisi en çok 64 kod noktalı geçerli
kök üretir; bu tanımlayıcı uzunluğu sınırı değildir. Ayrıştırılmış Unicode
biçimleri kaynak lexer'ında S029'dur. Malformed, elle kurulmuş token/AST
yapıları B-017/K-112'nin tamamladığı
[AST/HIR invariant doğrulayıcısına](ast-hir-invariantleri.md) aittir.
Kurtarmalı parser'ın kardeş/kapsam sahipliği ve 20 tanı bütçesi
[parser kurtarma rehberinde](parser-hata-kurtarma.md) bağlanır.
