# Lexer/parser fuzz rehberi

Bu rehber K-110/ADR-022'nin işletim sözleşmesidir. Amaç geçerli her UTF-8
kaynağın ya token/AST ya da Türkçe tanı üretmesi; process panic'i üretmemesidir.

## Kalıcı katmanlar

- `compiler/fuzz/fuzz_targets/lexer_parser.rs`: libFuzzer hedefi. Lexer başarılı
  olursa aynı token akışını normal ve hata-kurtarmalı parser'dan geçirir.
- `compiler/fuzz/corpus/lexer_parser/`: sekiz başlangıç girdisi. Geçerli
  programın yanında Unicode, emoji, combining im, girinti, sayı, virgül, metin
  ve iç içe blok saldırıları taşır.
- `compiler/fuzz/dictionaries/zee.dict`: Türkçe kalıpları ve kritik byte
  dizilerini mutation sözlüğüne verir.
- `compiler/tests/fuzz_korpusu_testi.rs`: stable ve bütün Tier-1 işletim
  sistemlerinde korpusu, 4.096 deterministik UTF-8 bileşimini ve 64 KiB uç
  örnekleri her `cargo test` koşusunda yeniden oynatır.

## Yerel koşu

Araç sürümleri politika gereği sabittir:

```sh
rustup toolchain install nightly-2026-08-31 --profile minimal
cargo +nightly-2026-08-31 install cargo-fuzz --version 0.13.2 --locked
cd compiler
cargo +nightly-2026-08-31 fuzz run lexer_parser fuzz/corpus/lexer_parser -- \
  -dict=fuzz/dictionaries/zee.dict -max_len=65536 -timeout=5
```

Kısa doğrulama için sona `-max_total_time=30`, uzun yerel çalışma için uygun
bir saniye bütçesi eklenir. Fuzzer'ın `DONE` ile ve sıfır koduyla bitmesi temiz
koşudur.

## Crash işlemi

1. `compiler/fuzz/artifacts/lexer_parser/` altındaki girdiyi aynı hedefe tek
   dosya olarak verip yeniden üret.
2. `cargo fuzz tmin lexer_parser <artifact>` ile girdiyi küçült.
3. Küçük girdiyi kalıcı korpusa ekle; davranış belirliyse ayrıca adı konmuş
   integration testi yaz.
4. Düzeltmeden sonra ana testleri, Clippy'yi, WASM'ı ve en az 30 saniyelik
   fuzz smoke koşusunu geçir.

Gece işinde korpus cache ile sonraki koşuya taşınır. Başarısız koşunun crash
girdisi GitHub artifact'ı olur; yalnız logda kalan ve tekrar üretilemeyen bulgu
kapatılmış sayılmaz.

## Sınırlar

Hedef `&str` aldığı için geçersiz UTF-8 byte dizileri burada değil dosya okuma
sınırında reddedilir. Kampanya girdisi 64 KiB ile sınırlıdır; bu bir dil dosyası
boyut sınırı değildir. Malformed, elle kurulmuş token/AST yapıları B-017'nin
invariant doğrulayıcısına aittir.
