# ADR-085 — Görünüm bloğu: tipli HTML ağacı (`<ad> görünüm olsun`, K-212)

- **Durum:** kabul
- **Tarih:** 9 Eylül 2026
- **İlgili kayıt:** K-212, `dogfood/urun-e`, K-174 (syntax freeze), spec/28, K-051 (`html güvenlisi`)
- **Kanıt:** `compiler/tests/gorunum_testi.rs`

## Bağlam

Site E paneli formları `"<form …>" ile csrf ile "…"` birleştirmeleriyle
yazılıyordu (inceleme P1-6 / v3 "en belirgin ergonomi borcu"). Güvenlik `html
güvenlisi` ile korunuyordu ama kaçış *isteğe bağlı*, okunabilirlik düşük ve
etiket dengesi elle takip ediliyordu.

## Karar

- Yeni cümle: **`<ad> görünüm olsun`** + girintili blok. Her satır bir düğümdür:
  `etiket ad değer ad değer …` (öznitelikler ad-değer çiftleri; ad kelime ya da
  tırnaklı metin — `"data-modal"`), `"sabit metin"`, `metin <ifade>` (kaçışlı),
  `ham <ifade>` (kaçışsız, önceden güvenli kılınmış parça). Girinti çocukları
  verir; sonuç Metin'dir.
- **Kaçış varsayılandır:** metin düğümleri ve öznitelik değerleri her zaman
  HTML kaçışlı yazılır; yalnız `ham` ham girer. Mantıksal öznitelik `doğru`
  ise yalnız ad yazılır, `yanlış` ise öznitelik yok (bayrak). TamSayı değer
  metne çevrilir. Boş (void) öğeler kapanış almaz.
- Tanılar: S046 (satır biçimi), T055 (metin/öznitelik türü). Grammar yalnız
  `görünüm` kalıp kelimesiyle büyüdü; K-174 kaydı güncellendi.

## Sonuçlar

- Etiket dengesi ve kaçış derleyicide; XSS için `html güvenlisi` unutma
  sınıfı hata görünüm bloğunda mümkün değil.
- `ham` yalnız `siteSablonu` gibi zaten güvenli parçalar için kullanılır;
  gözden geçirmede `ham` satırları aranır.
