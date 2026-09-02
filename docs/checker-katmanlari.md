# Checker katmanları

Bu belge ADR-013'ün uygulama rehberidir. Bir checker değişikliğinin nereye ait
olduğunu ve hangi komşu katmanlarla konuşabileceğini tanımlar.

## Geçiş sırası

```text
Program AST
   │
   ├─ etki       web/uygulama etkisi + intrinsic yetkinlik ihtiyacı
   ├─ turler     tür yazımları ve yapı alanları
   ├─ sozlesme   public parametre/dönüş sözleşmeleri
   └─ cumle / ifade
        ├─ sembol   ad, alan, kapsam
        ├─ akis     daraltma, gezme, görev sahipliği
        ├─ cagri    imza çıkarımı ve çağrı uyumu
        ├─ kaynak   ifade tanısını kesin AST/HIR aralığına bağlama
        └─ donus    kesin sonlanma ve dönüş türü birleşimi
```

`cozumleyici.rs` bu sıranın orkestratörüdür; semantik kural sahibi değildir.
Tek-tanı ve çoklu-tanı girişleri aynı katmanları aynı sırada kullanır.

## Sahiplik tablosu

| Değişiklik | Sahip modül | Birlikte doğrulanacak alan |
|---|---|---|
| yeni temel/kapsayıcı tür | `turler` | ifade checker, runtime değer, spec |
| ad/alan çözümü | `sembol` | morfoloji profili, A001/A002/T028 |
| dal içi tür daraltma | `akis` | cümle handler, olumlu/olumsuz flow testi |
| işlem argüman uyumu | `cagri` | `sozlesme`, RFC-0006/spec-10 |
| public API imzası | `sozlesme` | paket/birim testleri ve semver sınırı |
| web/uygulama/intrinsic etkisi | `etki` | ADR-011, T044–T050 |
| dönüş yolu ve birleşimi | `donus` | Seçenek/Sonuç, T018/T041/T042 |
| ifade tanı konumu | `kaynak` | ADR-030, AST/HIR aralık eşliği |
| AST varyantı denetimi | `cumle` / `ifade` | ilgili hizmet katmanı ve tanı |

## Katman kuralları

- İç hizmetler `pub(super)` kalır; crate public API'si yalnız kökten açıkça
  yeniden dışa aktarılır.
- `Tur` içine akış durumu, yetkinlik izni veya sembol kimliği gömülmez.
- `cumle` ve `ifade` handler'ları ortak kuralı kopyalamaz; sahibini çağırır.
- Etki denetimi tür hatası yüzünden atlanmaz; bağımsız fail-closed geçiştir.
- Bütçesi dolan modül için sayı sessizce yükseltilmez. Sorumluluk gerçekten
  yeniyse adlandırılmış katman açılır; değilse ortak kural çıkarılır.

## Ardıl işler

- B-007/K-121, kopya AST'deki erişilebilir çağrı kısıtlarını `cikarim`
  katmanında birleştirir. Keşif modu iç bloktaki geçici tür hatasında da
  sonraki kısıtları toplamayı sürdürür; asıl AST/HIR yalnız nihai imzayla
  denetlenir.
- B-010/K-101, yapı/işlem/sembol kimliklerini kararlı newtype'lara taşıdı;
  ayrıntılı sahiplik [semantic kimlik rehberindedir](semantic-kimlik-modeli.md).
- B-018/K-102 kaynak→token→parsed→bağlı program fazlarını türledi. K-103,
  checker sonucunu ayrı Typed HIR tür/bağ kaydına taşıdı; K-104 runtime'ın
  kaynak adıyla semantic karar vermesini bitirdi ve B-019'u kapattı.
- B-017/K-112, katmanlar arası AST/HIR değişmezlerini debug/test aşamasında
  [ayrı kapıyla](ast-hir-invariantleri.md) doğrular; bu katmanlar semantic
  kuralları üretir, invariant modülü faz çıktısının çapraz tamlığını denetler.
- B-021/K-113 çoklu-tanı parser recovery'sini cümle/girinti sınırında
  tamamladı. Checker kısmi AST'de cümle başına sürer; birleşik çıktı kaynak
  sırasında ve ortak 20 tanı bütçesindedir.
- B-050/K-126 AST ifadelerinin kesin kaynağını HIR'a birebir taşır;
  `kaynak` katmanı yalnız eski kaba ifade tanısını ilgili kesin aralığa
  yükseltir, tür veya semantic kimlik seçmez.

B-007 K-121 ile kapandı; K-101 semantic ID, K-102 faz tipi, K-103 typed HIR
üretimi, K-104 runtime tüketimi ve K-112 fazlar arası doğrulama temelini korur.
