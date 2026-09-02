# Semantic regresyon korpusu

K-147/ADR-044, düzeltilmiş bir Zee bug'ını genel test toplamında kaybolan bir
anı olmaktan çıkarır. Her kayıt “hangi bug, hangi faz, hangi küçük kaynak ve
hangi kesin gözlem?” sorularını makine-okunur cevaplar.

## Yerleşim

```text
regression/
├── v1.tsv
├── parser/
├── checker/
├── hir/
├── runtime/
├── morphology/
├── concurrency/
└── security/
```

Başlangıç tabanı yedi fazda 17 vaka taşır. Her `.dil` dosyası manifestte tam
bir kez bulunur; manifest dışı veya iki kez sahiplenilmiş kaynak test
kapısını kapatır. CI ayrıca taban Git revizyonundaki vaka kimliği, K-kimliği
ve dosya yolu üçlüsünün silinmesini ya da başka anlama taşınmasını reddeder.
Kaynak/beklenti bilinçli bir dil göçü nedeniyle değişirse ilgili RFC/spec ve
sürüm notu aynı committe güncellenir; vaka soy ağacı korunur.

## Manifest sözleşmesi

`regression/v1.tsv` dokuz sekmeli alandan oluşur:

1. küçük ASCII kebab-case vaka kimliği;
2. düzeltmenin `kararlar/gunluk.md` içinde var olan `K-NNN` bug kimliği;
3. birincil faz;
4. çalışma kipi;
5. beklenen tanı kodu, başarı için `-`;
6. kesin `satır:sütun:uzunluk`, başarı için `-`;
7. beklenen exit;
8. `\\n` ile ayrılmış sıralı çıktı, çıktı yoksa `-`;
9. depo köküne göre `.dil` yolu.

Tanı ile span birlikte bulunur. Derleme, parser, policy veya runtime hatası
`exit=1`; normal runtime kendi `programı N ile bitir` değerini taşır. Başarılı
programda stdout satır sırası byte düzeyinde sabittir.

## Kipler

| Kip | Çalışan sınır |
|---|---|
| `parser` | Lexer ve fail-fast parser |
| `diagnostics` | Kurtarmalı parser + çoklu checker tanı yüzeyi; vaka tam bir tanı ister |
| `compile` | Resolver, checker ve typed-HIR üretimi |
| `hir` | Compile sonrası açık bağlı-HIR invariant denetimi |
| `run` | Invariantı geçmiş typed HIR ile runtime; stdout ve program exit'i korunur |
| `capability` | Compile sonrası kapalı yetkinlik politikası |

## Yeni bug düzeltme protokolü

1. Arızayı en az satırlı, bağımsız `.dil` kaynağına indir.
2. Kaynağı sahibi olan faz klasörüne vaka kimliğiyle koy.
3. `v1.tsv` satırında bug, faz, kip, tanı+span, exit ve çıktıyı kaydet.
4. Aynı committe davranış testi, ilgili RFC/spec/ADR ve sürüm notunu güncelle.
5. Önce hedefli, sonra tam faz kapısını çalıştır:

```bash
cd compiler
cargo test --locked --test semantic_regresyon_korpusu_testi
cargo run --locked --bin faz_test_matrisi -- --denetle \
  --rapor target/faz-test-matrisi.md
bash ../scripts/semantic-regresyon-korugu.sh HEAD~1
```

Fixture en çok 4 KiB ve 32 dolu satırdır; CR, tab ve eksik son LF reddedilir.
Bu sınır bir kod golfü hedefi değil, tek arızayı açıklayan küçük örneği koruma
kapısıdır. Daha büyük senaryo golden veya conformance korpusuna aittir.

## Korpusların rolleri

- `regression/`: geçmişte düzeltilen somut Zee bug'larının küçük yeniden
  üretimleri;
- `golden/`: öğretici, kullanıcı yüzeyli uçtan uca programlar;
- `conformance/`: başka gerçeklemelerin de tüketebileceği sürümlü davranış
  profilleri;
- `compiler/fuzz/corpus/`: hasım veya üretilmiş girdi başlangıç tohumları.

Bu alanlar birbirinin yerine geçmez; K-146 faz raporu aralarındaki bağı aynı
satırda görünür tutar.
