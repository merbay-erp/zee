# ADR-051 — LSP tam-metin invalidation ölçeği

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-154, B-062

## Bağlam

LSP `textDocument/didChange` bugün değişen aralığı değil belgenin yeni tam
metnini alır. `Sunucu::belgeyi_guncelle` saklanan `String`i bütünüyle
değiştirir; `tanilari_yayinla` bu metni klonlayıp `kaynagi_tanilari` hattına
verir. Hat kaynağın tamamını yeniden sözcükler, ayrıştırır, isimleri çözer ve
checker üzerinden typed-HIR kanıtını yeniden kurar. Parse ağacı, çözümleme veya
HIR için belge sürümleri arasında yaşayan bir incremental cache yoktur.

Tek bir 2.000 satır metriği bu mimarinin hangi belge boyutunda kullanıcı
gecikmesine dönüştüğünü göstermiyordu. Optimizasyona başlamadan önce maliyet
eğrisi ve karar eşikleri sabitlenmelidir.

## Karar

1. Mevcut `lsp_degistir` metriği iş yükü değişmeden
   `lsp_degistir_2k` adını alır. Sürümlü geçmişteki eski sayılar da anlam
   kaybetmeden bu ada göçer.
2. Açık ve initialize edilmiş aynı in-process LSP yolunda 2.000, 5.000,
   10.000 ve 20.000 satırlık tam-metin `didChange` ayrı kimliklerle ölçülür.
   Her örneğin hazırlığında belge `didOpen` ile açılır; zaman penceresi yalnız
   `didChange` isteği ile tanı bildiriminin tamamlanmasını kapsar.
3. Uzun ölçek koşusu açık `--lsp-olcek` seçeneğidir. Shared CI bu seçeneği
   kullanır; hızlı yerel duman yalnız 2.000 satırı ölçer. Böylece normal test
   döngüsü yanlışlıkla dakikalarca uzatılmaz.
4. Kullanıcı algısı için önceden belirlenen p95 gözlem çizgileri 250 ms,
   500 ms ve 1 s'dir. Rapor her çizginin ilk aşıldığı sabit boyutu yazar.
   Bunlar bu aşamada hard CI geçiş/kalış kapısı veya optimizasyon başarısı
   değildir.
5. Bugünkü invalidation sınırı açıkça **belgenin tamamı + tam lexer/parser +
   tam çözümleme/checker + yeni typed-HIR kanıtı**dır. Incremental çalışma,
   ancak ayrı tasarım ve correctness kanıtıyla bu sınırı değiştirebilir.

## Reddedilen seçenekler

- **Ölçmeden incremental parser/checker yazmak:** en pahalı sınırı ve gerçek
  kırılma boyutunu bilmeden karmaşıklık ekler.
- **Yalnız parse süresini LSP değişiklik maliyeti saymak:** çözümleme, checker,
  HIR kanıtı, tanı üretimi ve protokol işleme maliyetini gizler.
- **Shared CI sayılarını hard eşik yapmak:** paylaşımlı runner gürültüsünü ürün
  regresyonu sanır. Bu seri gözlem ve kapasite kararı içindir.
- **Satır sayısını iş yükü tanımı olmadan değiştirmek:** tarihsel karşılaştırma
  anlamını bozar. Dört satırlık sabit üretim bloğu ve kimlikler sürümlüdür.

## Sonuçlar

- CI ve yerel tam koşu, editör değişiklik maliyetinin büyüme eğrisini aynı
  makine-okunur provenance sözleşmesiyle yayımlar.
- Exact temiz `59580cd0c0b2d66a1ff1f28e7e285bfa0858abab` uygulama
  commit'inde Apple M4 Pro üzerinde alınan 25 örnekli release tabanı şöyledir:
  2k p95 167,281 ms; 5k p95 1.081,746 ms; 10k p95 4.669,372 ms; 20k p95
  20.159,636 ms. 250, 500 ve 1.000 ms çizgilerinin üçü de ilk kez 5k'da
  aşılır. K-154/B-062 bu veriyle kapanır.
- Bu dilim grammar, runtime semantiği, kullanıcı tanıları, RFC ve normatif
  spec'i değiştirmez. Yalnız gözlem aracı, mimari gerçek ve CI kanıtı değişir.
