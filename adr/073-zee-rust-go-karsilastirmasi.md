# ADR-073 — Aynı aracın Zee, Rust ve Go gerçeklemeleriyle veri temelli karşılaştırma

- **Durum:** kabul
- **Tarih:** 6 Eylül 2026
- **İlgili kayıt:** K-165, K-164, ADR-072, V1-P1-30

## Bağlam

Üçüncü dış inceleme K-165 ile "aynı uygulamanın Zee–Go/Rust veri temelli
dogfood karşılaştırması"nı istedi: Zee'nin bir işi kaç satırda, kaç testle,
ne kadar sürede ve hangi sürtünmeyle yaptığı sayıyla söylenmeli, izlenimle
değil. K-164'ün kanıt özeti aracı bunun için uygundur: dokuz kayıt defteri,
sayım, Türk alfabesi sıralaması, Markdown üretimi — küçük ama gerçek.

## Karar

1. **Eşdeğerlik önce.** `dogfood/kanit-ozeti-karsilastirma/rust` ve `/go`
   aynı sayfayı bayt bayt üretmek zorundadır (`dogfood_karsilastirma_testi`;
   Go yalnız araç zinciri varsa). Sapma varsa ölçüm anlamsızdır.
2. **Ölçüler.** `scripts/dogfood-karsilastirma.sh` temiz ağaçta exact Git
   SHA ile şunları `docs/dogfood-karsilastirma-v1.tsv`ye yazar: kaynak satırı
   (boş/yorum dışı, testler dahil), birim testi sayısı, derleme süresi (Zee
   için `dil` release derlemesi), çalışma süresi medyanı (Zee: derle+yürüt),
   tepe RSS ve korpusa giren sürtünme vakası. Kaynak satırı ve test sayısı
   testte yeniden hesaplanır; süre/RSS makineye bağlıdır, kapı değildir.
3. **Yorum kuralı.** Karşılaştırma belgesi sayıyı yazar, üstünlük iddiası
   yazmaz. Zee'nin kaybettiği yerler (çalışma süresi, sürtünme) açıkça
   listelenir; bunlar backlog girdisidir.

## Reddedilen seçenekler

- **Yalnız satır sayısı karşılaştırmak:** okunabilirlik iddiasını
  kanıtlamaz; test, süre ve sürtünme birlikte okunur.
- **Farklı bir uygulama seçmek:** üç dilde aynı sürtünmeyi yaşamak için
  aynı girdi/çıktı sözleşmesi gerekir; kanıt özeti bunu verir.
- **Benchmark eşiği koymak:** shared runner gürültülüdür (ADR-021 kuralı);
  süre yayımlanır, eşiklenmez.

## Sonuçlar

- İlk kayıt ve tablo `docs/dogfood-karsilastirma.md` içindedir.
- Zee'nin derle+yürüt süresi derlenmiş ikililerden yüksektir; yorumlayıcı
  tasarımının bilinen bedelidir ve V1 sözü değildir.
- Sürtünme sayısı ürün başına değil dil başına okunur: Zee'de 16 (K-164),
  Rust/Go'da bu iş için kaydedilen sürtünme yoktur.
