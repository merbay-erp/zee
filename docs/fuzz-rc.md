# Fuzz release-candidate kapısı

K-157/ADR-056, günlük beş dakikalık fuzz smoke ile release-candidate soak
kanıtını ayırır. `.github/workflows/fuzz-rc.yml` haftalık ve elle başlatılan
koşuda sekiz hedefi paralel, hedef başına 3.600 saniye AddressSanitizer altında
çalıştırır. `dns_yaniti` (K-390/ADR-169) 24 Kasım 2026'da eklendi; bu tarihten
önceki RC kayıtları dört hedeflidir. `dkim_imza_girdisi` (ADR-174) ve `x509_spki`
(ADR-173) K-411 ile, `kodlama` (RFC-0043, ADR-167; ZeeMail'de ağdan gelen
MIME/QP/AUTH girdisinin katı çözücüleri) K-412 ile 26 Kasım 2026'da eklendi; bu
tarihten sonraki RC adayı sekiz hedefin hepsini ister.
`psl` (K-418, RFC-0044 B1b, ADR-177; kamu son ek listesi ve kurumsal alan)
27 Kasım 2026'dan beri yalnız günlük `fuzz.yml` koşusundadır; RC kapısına
alınması ayrı kullanıcı kararıdır ve bu karar olmadan RC listesi sekiz hedef
kalır.

## Geçme koşulu

- `lexer_parser`, `morfoloji`, `http_istegi`, `dns_yaniti`, `dkim_imza_girdisi`,
  `x509_spki`, `kodlama`, `wasm_abi` aynı Git commit'inde
  30–60 dakika çalışmış olmalı;
- sekiz exit kodu da sıfır, crash/sanitizer bulgusu ve tek-girdi timeout'u sıfır
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

## İlk yedi hedefli kampanya (26 Kasım 2026, K-417)

`b3b9d752c417b2f4db63cfdfb4cf5d65a448408c` üzerinde iki koşu yapıldı:
yerel kanonik üretici (`scripts/rc-kampanyasi.py`, macOS arm64, hedef başına
1.800 sn) ve GitHub `fuzz-rc` koşusu 45838195765 (ubuntu-26.04, hedef başına
3.600 sn). Altı hedef ve iki Miri grubu iki ortamda da yeşil. `dkim_imza_girdisi`
iki ortamda da **kaldı**. Neden kâhin hatası, ürün kodu değil (K-417):
`tests/ortak/dkim_ozellikleri.rs` girdinin `b=` ile bitmesini istiyordu.
RFC 6376 §3.2 adla `=` arasında FWS'ye izin verir (`b\t=`) ve
`imza_girdisini_denetle` bunu doğru kabul eder.

- GitHub crash: 430 bayt, 57. saniye, SHA-256
  `9d3cf17d25536afd3e48b1614beba2e70fb5d302b72f0b29037cae7d269cb8bc`.
- Yerel crash: 382 bayt, SHA-256
  `d45d9358b405c11d0ae9554d05f0163c41a9e800413409dae411ca6f9c6e14a6`.
- Yerel `dkim` satırındaki süre süreç süresidir; libFuzzer crash'te final
  süre basmaz.

Kırmızı satırlar tarihçede olduğu gibi kalır. GitHub satırlarında CPU modeli
ve RAM `-` yazılıdır; koşu günlüğü bunları kaydetmez. RC kapısı exact SHA'da
bütün hedefleri ister (`scripts/guvenlik-kapisi.sh`), bu yüzden bu commit
RC adayı olamaz. K-417 ve K-412 birleşince yeni master'da sekiz hedef birlikte yeniden
koşulur.

## İlk sekiz hedefli yeşil kampanya (26 Kasım 2026)

K-417 ve K-412 birleştikten sonra `9c003fff7c5d496eb32b5c8fa634dea2e043d99a`
üzerinde sekiz hedef birlikte koşuldu. Yerel kanonik üretici sonucu
`passed-local-technical-campaign`: hedef başına 1.801 sn, toplam 65.479.628
yürütme. GitHub `fuzz-rc` koşusu 45867659484 de yeşil: hedef başına 3.601 sn,
toplam 3.586.383.968 yürütme. İki ortamda da crash, timeout ve sanitizer bulgusu
sıfır. İki Miri grubu (3 test) iki ortamda da geçti. `dkim_imza_girdisi`
K-417 tohumuyla temiz. Bu SHA için `scripts/guvenlik-kapisi.sh` RC fuzz adımı
sekiz hedefte de karşılanır. Kapının clippy, tam test ve koruk adımları ayrıca
koşulur; bu kayıt sürüm etiketi değildir.

## Sekiz hedefli yeniden koşu (27 Kasım 2026)

Master `90e14b2c`'den ilerlediği için kampanya güncel HEAD
`7ff829ad6e09f866b27c4ae68be679c00dd77088` üzerinde yeniden koşuldu; `fuzz-rc`
iş akışı exact SHA girdisi almaz, yalnız master'da koşar. Yerel kanonik üretici
sonucu `passed-local-technical-campaign`: hedef başına 1.801 sn, toplam
64.188.073 yürütme, kaynak kampanya boyunca temiz ve değişmedi. GitHub
`fuzz-rc` koşusu 45945292581 de yeşil (9/9 iş): hedef başına 3.601 sn, toplam
2.067.613.199 yürütme. İki ortamda da crash, timeout ve sanitizer bulgusu sıfır;
iki Miri grubu (3 test) geçti. Bu SHA için `scripts/guvenlik-kapisi.sh` RC fuzz
adımı sekiz hedefte karşılanır. K-418 `psl` hedefi bu SHA'dan sonra yalnız
gecelik fuzz'a eklendi; RC kapısı hâlâ sekiz hedeflidir. Clippy, tam test ve
koruk adımları ayrıca koşulur; bu kayıt sürüm etiketi değildir.

## K-265 explicit dış RC kanıtı

[Exact kaynak dış kanıt rehberi](rc-kaniti.md) ve [ADR-124](../adr/124-exact-kaynak-dis-rc-kaniti.md) temiz S’ye bağlı dış paketi tanımlar. Explicit geçersiz paket fallback yapmaz; etiket/build S’de, append-only arşiv commit’i E’de S kimliğini korur. Default tracked yolun self-reference sınırı açıkça korunur. Gerçek uzun kampanya henüz bu kısa entegrasyon testlerinden tamamlanmış sayılamaz.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## S4 kaynak kabulü

[S4 ayrı kaynak kaydı](urunlesme-s4-kabul-20261015.md) `4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2`
üzerindeki güncel yerel kanıtı ve açık sınırları gösterir. Önceki S3 sonuçları
tarihsel kalır; genel ürün/platform/yayın veya tam P19 kabulü değildir.
