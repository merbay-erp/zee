# ADR-044 — Sürümlü semantic regresyon korpusu

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-147, B-039

## Bağlam

Zee'nin faz testleri davranış alanlarını geniş biçimde koruyor; ancak altı ay
önce düzeltilen tek bir bug'ın hangi küçük kaynakla, hangi fazda ve hangi
gözlemle kapatıldığını bağımsız bir veri kaydı olarak söylemiyordu. Rust test
gövdesine gömülü büyük kaynaklar ve yalnız tanı koduna bakan assertion'lar,
tanının yer değiştirmesini veya bir runtime çıkışının sessizce değişmesini
göstermeyebilir. Sayı büyüdükçe geçmiş arızanın kimliği genel yeşil toplamın
içinde kaybolur.

## Karar

1. Kök `regression/` ağacı parser, checker, typed HIR, runtime, morphology,
   concurrency ve security fazlarına göre ayrılır. Her vaka tek, küçük `.dil`
   dosyasıdır; dosya adı kararlı vaka kimliğiyle aynıdır.
2. `regression/v1.tsv` (K-155/ADR-052 ile `v2.tsv`ye yükseltildi); vaka ve
   `K-NNN` bug kimliğini, birincil fazı, yürütme
   kipini, beklenen tanı kodu ile kesin `satır:sütun:uzunluk` aralığını,
   süreç/program çıkışını, sıralı çıktıyı ve dosya yolunu tek satırda taşır.
   K-kimliğinin karar günlüğünde gerçek bir kaydı bulunmak zorundadır.
3. Veri-güdümlü test manifest ile ağaç arasında birebir sahiplik ister. Kayıp,
   yinelenen veya sahipsiz fixture; bilinmeyen faz/kip; tanısız span; 4 KiB ya
   da 32 dolu satır minimality sınırı ihlali fail-closed'dur.
4. Vaka kipleri gerçek faz yüzeylerini kullanır: parser, kurtarmalı çoklu tanı,
   tam compile, açık HIR invariantı, typed-HIR runtime ve kapalı yetkinlik
   denetimi. Runtime sonucu hem kısmi/tam stdout'u hem gerçek program exit'ini
   karşılaştırır. Bunun için typed-HIR yüzeyine çıkış kodunu kaybetmeyen
   `calistir_baglanmis_io_kodla` eklenir.
5. Her yeni compiler bug düzeltmesi, aynı committe minimal fixture ve metadata
   satırı bırakır. Git-tabanlı CI koruğu tabandaki vaka kimliği+bug kimliği+yol
   üçlüsünün silinmesini veya yeniden kullanılmasını reddeder. Beklenti ya da
   kaynak bilinçli değişirse ilgili RFC/spec göç kararı aynı committe gerekir.
6. Korpus K-146 matrisinde ayrı `Semantic regression` fazıdır. Böylece genel
   Cargo toplamından bağımsız count/pass/fail ve süre ile raporlanır; ilgili
   üretim fazları da onu aşağı akış etkisi olarak ilan eder.

## Reddedilen seçenekler

- **Yalnız Rust test adını bug kimliği saymak:** kullanıcı kaynak örneğini ve
  beklenen span/exit sözleşmesini makine-okunur veriden çıkarır.
- **Büyük golden programları regresyon korpusu saymak:** uçtan uca güven verir
  ama tek hatanın en küçük yeniden üretimini ve sahibi olan fazı gizler.
- **Sadece hata kodunu karşılaştırmak:** doğru kodun yanlış tokena taşınmasını
  ve başarılı programın çıktı/exit drift'ini yakalamaz.
- **Fixture içine serbest yorum metadata'sı koymak:** bütün kayıtları tek
  tabloda sorgulamayı ve ağaçla birebirlik denetimini zorlaştırır.

## Sonuçlar

- Başlangıç tabanı 17 tarihsel vakayı yedi semantic fazda kalıcılaştırır; vaka
  soy ağacı sonraki commitlerde sessizce budanamaz.
- Bir regresyon artık `K-NNN / vaka` kimliğiyle tanı, span, exit veya çıktı
  farkını doğrudan gösterir.
- Korpus conformance profillerinin yerine geçmez: conformance bağımsız
  gerçeklemeler arası yayımlanmış söz, regression ise geçmiş Zee bug'ının
  küçük ve büyüyebilen yerel yeniden üretimidir.
- Dil sözdizimi, tanı anlamları ve normatif spec bu kararla değişmez.
