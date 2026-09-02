# Semantic regresyon korpusu

K-147/ADR-044, düzeltilmiş bir Zee bug'ını genel test toplamında kaybolan bir
anı olmaktan çıkarır. K-155/ADR-052 bunu exact düzeltme commit'i ve garanti
sürümüyle kaynak soy ağacına bağlar. Her kayıt “hangi bug, hangi commit, hangi
sürüm, hangi faz, hangi küçük kaynak ve hangi kesin gözlem?” sorularını
makine-okunur cevaplar.

## Yerleşim

```text
regression/
├── v2.tsv
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
kapısını kapatır. CI ayrıca taban Git revizyonundaki vaka kimliği, K-kimliği,
dosya yolu ve provenance zincirinin silinmesini ya da başka anlama taşınmasını reddeder.
Kaynak/beklenti bilinçli bir dil göçü nedeniyle değişirse ilgili RFC/spec ve
sürüm notu aynı committe güncellenir; vaka soy ağacı korunur.

## Manifest sözleşmesi

`regression/v2.tsv` on iki sekmeli alandan oluşur:

1. küçük ASCII kebab-case vaka kimliği;
2. düzeltmenin `kararlar/gunluk.md` içinde var olan `K-NNN` bug kimliği;
3. düzeltmeyi taşıyan tam 40 haneli `fixed_by` Git SHA;
4. kanıtlandıysa tam `introduced_by` SHA, bilinmiyorsa `-`;
5. güvencenin başladığı SemVer serisi (`0.8.0-dev`);
6. birincil faz;
7. çalışma kipi;
8. beklenen tanı kodu, başarı için `-`;
9. kesin `satır:sütun:uzunluk`, başarı için `-`;
10. beklenen exit;
11. `\\n` ile ayrılmış sıralı çıktı, çıktı yoksa `-`;
12. depo köküne göre `.dil` yolu.

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
3. Davranış düzeltmesini ayrı, yeşil bir commit olarak al; tam SHA'yı kaydet.
4. `v2.tsv` satırında bug, `fixed_by`, kanıtlıysa `introduced_by`, garanti
   sürümü, faz, kip, tanı+span, exit ve çıktıyı kaydet.
5. Fixture/provenance commit'inde ilgili RFC/spec/ADR ve sürüm notunu güncelle.
6. Önce hedefli, sonra tam faz kapısını çalıştır:

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

Manifest testi tam SHA biçimini, commit varlığını, `fixed_by`→HEAD ve varsa
`introduced_by`→`fixed_by` ata yönünü denetler. İlk 17 tarihsel vakada
minimal reproducer düzeltme anında mevcut olmadığı için introduced commit
tahmin edilmemiş, açıkça `-` bırakılmıştır. Sonradan bisect ile kanıtlanan
değer yalnız `-`→ata SHA yönünde bir kez eklenebilir. Koruk ayrıca
`compiler/src` tarihindeki başlığı fix/bug/düzeltme bildiren her commit için
aynı karşılaştırma aralığında `fixed_by` sahibi yeni bir fixture ister.
`semantic_regresyon_korugu_testi`, geçici gerçek Git deposunda fixture'sız
bug-fix reddini, exact `fixed_by` ile kabulü ve provenance yeniden yazım
reddini uçtan uca çalıştırır; yalnız script metnine bakmakla yetinmez.

## Korpusların rolleri

- `regression/`: geçmişte düzeltilen somut Zee bug'larının küçük yeniden
  üretimleri;
- `golden/`: öğretici, kullanıcı yüzeyli uçtan uca programlar;
- `conformance/`: başka gerçeklemelerin de tüketebileceği sürümlü davranış
  profilleri;
- `compiler/fuzz/corpus/`: hasım veya üretilmiş girdi başlangıç tohumları.

Bu alanlar birbirinin yerine geçmez; K-146 faz raporu aralarındaki bağı aynı
satırda görünür tutar.
