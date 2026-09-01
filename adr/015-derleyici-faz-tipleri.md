# ADR-015 — Derleyici fazlarını Rust türleriyle görünür kılma

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-102, B-018, V1-P0-13

## Bağlam

ADR-012/013 fiziksel dosya ve semantik sahiplik sınırlarını kurdu; ADR-014
semantic kimlikleri depolamadan ayırdı. Ancak uçtan uca veri hâlâ çoğunlukla
`&str`, `Vec<Token>`, `Vec<Cumle>` ve `Program` olarak taşınıyordu. Bir API
imzasına bakarak verinin lexer öncesi mi, parser sonrası mı veya checker'dan
geçmiş mi olduğu anlaşılamıyordu. Özellikle raw `Program`, hem bağlanmamış AST
hem yürütülebilir program anlamında kullanılabiliyordu.

## Karar

`compiler/src/faz.rs` gerçek bootstrap hattının veri sınırlarını ayrı türlerle
temsil eder:

```text
KaynakMetni
  → TokenAkisi
  → AyristirilmisAst
  → hoist / BaglanmamisProgram
  → checker (ad çözümü + semantic ID + tür/akış/etki denetimi)
  → BaglanmisProgram
  → interpreter
```

`BaglanmamisProgram` yalnız crate içinde kurulabilir. `BaglanmisProgram`ın tek
üretim yolu başarılı checker geçişidir; immutable program görünümü ve eski
Rust API tüketicileri için bilinçli `into_program` adaptörü sunar.

Standart `kaynagi_denetle`, `kaynagi_calistir[_girdiyle]` ve `kaynagi_dene`
yolları fazlı derleme API'sini kullanır. Runtime'a yeni
`calistir_baglanmis[_io]` girişleri eklenir. Raw `Program` alan eski derleme ve
runtime fonksiyonları v0 Rust embedding uyumluluğu için korunur; kaynak API'si
bu adaptörlere ancak checker başarısından sonra iner.

## Açık sınır

Checker ad çözümü ile tür denetimini aynı AST-mutasyon geçişinde yapar. Bu ADR
anında `BaglanmisProgram` typed HIR değildi. K-103/ADR-016 sonradan her
denetlenmiş ifadeye açık tür/ID bağı veren HIR çekirdeğini zorunlu sahip yaptı;
K-104 standart runtime ve `dene` hattını bu bağlara geçirdi. Zorunlu span
K-108/ADR-020 ile sonradan tamamlandı. B-018'in sözü mevcut gerçek fazların
yanlış adlandırılmadan türlerde görünür olmasıdır.

K-113/ADR-024 kurtarmalı çoklu-tanı yolunda kısmi AST'nin cümle ve dengeli
girinti sınırlarında kurulmasını bağladı. Bu değer `AyristirilmisAst` olarak
kalır; hata varken `BaglanmisProgram` veya yürütme girişine yükseltilmez.

## Değişmezler

1. Parsed AST doğrudan `calistir_baglanmis[_io]` girişine verilemez.
2. `BaglanmisProgram` yalnız başarılı checker sonucunda üretilebilir.
3. Standart kaynak→çalıştır hattı raw `Program` üzerinden checker'ı atlayamaz.
4. Kurtarmalı çoklu-tanı yolu kısmen geçerli AST'yi yürütülebilir diye
   etiketlemez.
5. Eski API adaptörü kaynak semantiğini veya runtime sırasını değiştirmez.
6. Typed HIR adı, B-019 gerçek temsil kurulmadan kullanılamaz.

## Sonuçlar

- Kaynak, token, parsed AST, bağlanmamış ve bağlanmış program kodda ayrı
  isimlere ve API'lere sahiptir.
- `compile_fail` kanıtı parsed AST'nin fazlı runtime'a verilemediğini korur.
- İki davranış ve bir kaynak-mimari testi standart hat ile uyumluluk
  adaptörünü korur.
- Parser recovery regresyonları kısmi AST'nin faz etiketini aşmadan sağlam
  kardeşleri doğru ebeveyn blokta tuttuğunu korur.
- Zee kaynak dili değişmediği için yeni normatif spec gerekmez.
