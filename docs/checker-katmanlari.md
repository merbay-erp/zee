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

- B-007, çıkarımlı yerel işlem imzasını çağrı sırasından bağımsız yapacaktır.
- B-010/K-101, yapı/işlem/sembol kimliklerini kararlı newtype'lara taşıdı;
  ayrıntılı sahiplik [semantic kimlik rehberindedir](semantic-kimlik-modeli.md).
- B-018/K-102 kaynak→token→parsed→bağlı program fazlarını türledi. K-103,
  checker sonucunu ayrı Typed HIR tür/bağ kaydına taşıdı; B-019'un kalan
  dilimi runtime'ın kaynak adıyla semantic karar vermesini bitirecektir.
- B-017, katmanlar arası AST/HIR değişmezlerini debug/test aşamasında ayrıca
  doğrulayacaktır.

B-007/B-017 açık, B-019 kısmen kalır; K-101 semantic ID, K-102 faz tipi ve
K-103 typed HIR üretim temelini kapatır.
