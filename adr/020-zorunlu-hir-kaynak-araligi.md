# ADR-020 — HIR düğümlerinde zorunlu kaynak aralığı

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-108, B-020, V1-P0-18

## Bağlam

ADR-016 her denetlenmiş ifadeyi kimlik, tür ve semantic bağ taşıyan typed HIR
kaydına dönüştürdü. Ancak bu kayıt kaynak kökenini yapısal olarak zorunlu
kılmıyordu. Sonraki tanı, optimizasyon veya LSP geçişi bir semantic düğümü
kaynağa bağlamak için AST varyantını yeniden incelemek ya da konum uydurmak
zorunda kalabilirdi.

Bugünkü bootstrap AST'si bütün ifade varyantlarında aynı konum hassasiyetini
taşımaz. Değişken düğümü lexer tokenının satır, sütun ve uzunluğunu korur;
diğer birçok düğüm yalnız sahibi olan cümlenin satırıyla ilişkilendirilebilir.
Eksik bilgiyi sahte `sütun=1, uzunluk=1` değerine çevirmek kaliteli bir
sözleşme değildir.

## Karar

Her `HirIfadeBilgisi`, kurucusunda zorunlu bir `HirKaynakAraligi` alır. Alan
sonradan doldurulan `Option` değildir; konumsuz HIR ifadesi kurulamaz.

Kaynak aralığı hassasiyetini iki açık varyantla taşır:

- `Kesin { satir, sutun, uzunluk }`: lexer tokenından gelen tam tek-satır
  aralığı,
- `Satir { satir }`: düğümün tamamını kapsayan kaynak satırı zarfı.

Bütün sayısal bileşenler `NonZeroUsize`dır. Checker, değişken ifadelerinde
korunan token aralığını; diğer mevcut AST ifadelerinde cümle satırı zarfını
kaydeder. Değer döndürmeyen çağrı cümleleri de aynı zorunlu alanı taşır.
Belirsiz konuma sahte kesinlik verilmez.

## Değişmezler

1. `HirIfadeBilgisi` kaynak aralığı olmadan kurulamaz.
2. Kaynak satırı, sütunu ve uzunluğu sıfır olamaz.
3. Kesin bilgi varsa daha kaba satır zarfına düşürülmez.
4. Kesin bilgi yoksa uydurma sütun veya uzunluk yazılmaz.
5. Kaynak aralığı semantic kimlik, tür ve bağ ile aynı checker geçişinde
   üretilir.

## Sonuçlar

- B-020 ve V1-P0-18 kapanır; her mevcut semantic HIR düğümünün kaynak kökeni
  yapısal zorunluluktur.
- `hir/kaynak.rs` bu modeli ayrı sahiplenir; HIR kök dosyasının mimari satır
  bütçesi korunur.
- Değişken için kesin konumu ve bileşik ifade için satır zarfını doğrulayan
  HIR kanıtı vardır; mimari test kaynak-aralığı tipinin varlığını korur.
- Bütün AST varyantlarına kesin token aralığı yaymak ayrı tanı hassasiyeti işi
  B-050'dir; bu iş HIR'da konumsuz düğüm oluşturma izni vermez.
- K-112/ADR-023 bağlı her AST ifadesinin bu zorunlu aralığı taşıyan tam bir
  HIR kaydıyla eşleşmesini ayrıca yürütülebilir invariant yapar.
- Zee kaynak semantiği değişmediğinden yeni normatif dil spec'i gerekmez.
