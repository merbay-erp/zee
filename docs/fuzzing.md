# Compiler fuzz rehberi

Bu rehber K-110/ADR-022, K-111, K-140/ADR-037, K-142/ADR-039,
K-143/ADR-040 ve K-156/ADR-055'in ortak işletim
sözleşmesidir. Lexer/parser hedefi geçerli her UTF-8 kaynağın token/AST ya da
Türkçe tanı üretmesini; morfoloji hedefi geçerli her üretilmiş kök+ek zincirinin
aynı soyut çözüme dönmesini ve çoklu köklerin sessizce seçilmemesini;
`http_istegi` hedefi ise her byte dizisinin exact bir HTTP/1.x isteği ya da
fail-closed hata olmasını arar. `wasm_abi` hedefi kayıtlı/kayıt dışı pointer,
uzunluk, UTF-8 ve tampon bırakma dizilerini hasım native host olarak zorlar.

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
- `compiler/fuzz/fuzz_targets/http_istegi.rs`: ham `&[u8]` request-line,
  başlık ve gövdeyi byte parser'dan geçirir; kayıplı UTF-8 üretmez.
- `compiler/fuzz/corpus/http_istegi/`: geçerli GET/POST ile bare-LF,
  absolute-form ve duplicate Content-Length başlangıçlarını taşır. `.gitattributes`
  bu wire fixture'larını metin normalizasyonundan çıkarır; CRLF ve bare-LF
  ayrımı her platformda byte-byte korunur.
- `compiler/tests/http_istegi_testi.rs`: kalıcı HTTP korpusunu ve adlandırılmış
  CRLF/obs-fold/NUL/target/TE-CL/UTF-8 framing saldırılarını her ana testte
  yeniden oynatır.
- `compiler/fuzz/fuzz_targets/wasm_abi.rs`: kayıtlı girdinin yanında null,
  iç/kayıt dışı pointer, yanlış/taşkın uzunluk, keyfî UTF-8, kaynak/soru kipi
  ve yanlış+çift bırakma sıralarını yürütür; her canlı sonuç kaydının uzunluk
  önekini ve UTF-8 gövdesini doğrular.
- `compiler/fuzz/corpus/wasm_abi/`: ilk kontrol baytıyla geçerli kaynak, soru
  girdisi, Unicode, bozuk uzunluk ve boş çağrı yollarını başlatan beş byte
  tohumudur.
  `.gitattributes` bu kontrol baytlarını metin normalizasyonundan çıkarır.
- `compiler/tests/playground_testi.rs`: aynı ABI'yi native hostta adlandırılmış
  pointer/uzunluk, UTF-8, yanlış/çift bırakma, kaynak/soru sınır+bir ve bozuk
  çağrı sonrası tekrar kullanım regresyonlarıyla yürütür.
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
cargo +nightly-2026-08-31 fuzz run http_istegi fuzz/corpus/http_istegi -- \
  -dict=fuzz/dictionaries/zee.dict -max_len=81920 -timeout=5
cargo +nightly-2026-08-31 fuzz run wasm_abi fuzz/corpus/wasm_abi -- \
  -dict=fuzz/dictionaries/zee.dict -max_len=4097 -timeout=5
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

Gece işinde korpus cache ile sonraki koşuya taşınır; cache yalnız hızlandırma
katmanıdır. Her hedefin başarılı veya başarısız koşu sonu korpusu ayrıca 90
günlük `${hedef}-fuzz-corpus-${run_id}-${run_attempt}` artefaktına yüklenir.
Artefakt içindeki `manifest.tsv`; hedef/korpus, kaynak commit, run/attempt,
sabit araç sürümleri ve her seed'in göreli yol+SHA-256 özetini taşır. Başarısız
koşunun crash girdisi de ayrı 90 günlük artefakttır; yalnız logda kalan ve
tekrar üretilemeyen bulgu kapatılmış sayılmaz.

İndirilen korpus önce doğrulanır:

```sh
scripts/fuzz-korpus-artefakti-dogrula.sh lexer_parser \
  indirilen/zee-fuzz-corpus-lexer_parser
```

Yeni coverage seed'leri doğrudan repoya kopyalanmaz. Hedefin mevcut kaynak
korpusuyla birlikte `cargo fuzz cmin` ile küçültülür; yeni yolu koruyan en küçük
girdi anlamlı bir adla `compiler/fuzz/corpus/<hedef>/` altına alınır. Stable
replay veya adı konmuş regresyon testi eklenir ve normal code review'dan geçer.
Bu nedenle cache silinmesi öğrenimi yok etmez, fakat geçici mutation çıktısı da
otomatik olarak kalıcı dil kanıtına dönüşmez.

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
HTTP hedefi `&[u8]` aldığı için geçersiz UTF-8'i özellikle tarar. 81920 byte
fuzz sınırı production'daki 16 KiB başlık + 64 KiB gövde zarfıdır; daha büyük
socket girdisi parser tahsisinden önce 431/413 ile kesilir.
WASM ABI hedefinin ilk baytı çağrı ve kaynak/soru kipidir; kalan en çok 4 KiB
girdi byte'ıdır. Bu fuzz kampanya sınırıdır, ürünün kaynak boyu sözü değildir.
ABI katmanı tek
tamponu 16 MiB + dört bayt, sekiz canlı tamponu toplam 64 MiB ile sınırlar;
kaynak 8 MiB, soru girdisi 1 MiB/4.096 satırla kopya ve satır koleksiyonundan
önce ayrıca sınırlanır. Sınır+bir davranışı fuzz giriş boyuna bırakılmaz;
native ve gerçek wasm32 Node regresyonu her ana kapıda exact uygular.
Kaynak+soru kipli ilk K-143 kampanyası 1.709.869 çağrıyı 61 saniyede ihlalsiz
tamamlamıştır.

K-156 kalıcılık kapısı kampanya süresini büyütmez. Dört hedefte 30–60 dakikalık
Uzun release-candidate koşusu K-157/ADR-056 ile ayrı bir kapıdır. İlk doğrulanmış
tabanda dört hedef 30'ar dakika AddressSanitizer altında toplam 136.789.564
girdiyi crash/timeout/bulgu olmadan yürüttü; seçili Miri testleri 3/3 geçti.
Tekrar ve provenance sözleşmesi [RC fuzz rehberindedir](fuzz-rc.md).
