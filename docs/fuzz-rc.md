# Fuzz release-candidate kapısı

K-157/ADR-056, günlük beş dakikalık fuzz smoke ile release-candidate soak
kanıtını ayırır. `.github/workflows/fuzz-rc.yml` haftalık ve elle başlatılan
koşuda dört hedefi paralel, hedef başına 3.600 saniye AddressSanitizer altında
çalıştırır.

## Geçme koşulu

- `lexer_parser`, `morfoloji`, `http_istegi`, `wasm_abi` aynı Git commit'inde
  30–60 dakika çalışmış olmalı;
- dört exit kodu da sıfır, crash/sanitizer bulgusu ve tek-girdi timeout'u sıfır
  olmalı;
- log, sonuç TSV'si, K-156 SHA-256 korpus manifesti ve varsa crash girdisi 90
  günlük hedef artefaktında bulunmalı;
- `ondalik::testler::` ve `zaman::testler::` sabit nightly Miri altında yeşil
  olmalı.

Bir hedefin kırmızı olması diğer matrix hedeflerini iptal etmez; fakat RC
kapısının tamamını kırmızı yapar. Eski başka bir commit'in yeşili yeni adayın
yerine kullanılamaz.

## Yerel tekrar

Kaynak korpusun doğrudan büyümesini önlemek için önce geçici bir klasöre
kopyala, sonra explicit sanitizer ve zaman bütçesiyle çalıştır:

```sh
cd compiler
mkdir -p target/fuzz-rc/lexer_parser/corpus
cp -R fuzz/corpus/lexer_parser/. target/fuzz-rc/lexer_parser/corpus/
cargo +nightly-2026-08-31 fuzz run --sanitizer address \
  lexer_parser target/fuzz-rc/lexer_parser/corpus -- \
  -dict=fuzz/dictionaries/zee.dict -max_total_time=1800 \
  -max_len=65536 -timeout=5 -print_final_stats=1
```

Diğer hedeflerin exact `max_len` değerleri workflow matrix'inde ve
[fuzz rehberinde](fuzzing.md) tek tek görünür. Kampanya korpusu K-156
doğrulama+`cmin`+stable replay+review akışı olmadan repoya alınmaz.

## Sonuç kaydı

Kanonik append-only kayıt [fuzz RC tarihçesidir](fuzz-rc-gecmisi-v1.tsv).
`actual_seconds` libFuzzer final süresi, `executions` yürütülen girdi sayısıdır.
`crashes=0`, süreç exit'i ve artefakt logu birlikte değerlendirilir; yalnız
yüksek yürütme sayısı güvenlik kanıtı değildir.

## İlk doğrulanmış taban

3 Eylül 2026'da `b75072beac1a54125f7cd76c51965accc4d9526f` üzerinde dört
hedefin her biri 1.801 saniye AddressSanitizer altında çalıştı. Toplam
136.789.564 girdi yürütüldü; crash, timeout ve sanitizer bulgusu sıfırdı.
`lexer_parser`, `morfoloji`, `http_istegi`, `wasm_abi` sırasıyla 13.034.402,
19.622.858, 85.659.451 ve 18.472.853 yürütme yaptı. Seçili ondalık ve zaman
Miri testleri 3/3 geçti. Donanım ve araç zincirinin exact kaydı tarihçe
dosyasındadır; bu taban farklı bir commit'in RC kanıtı yerine kullanılamaz.
