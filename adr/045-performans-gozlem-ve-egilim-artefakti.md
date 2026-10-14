# ADR-045 — Performans gözlemi ve tarihsel eğilim artefaktı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-148, B-040; K-152 provenance revizyonu ADR-049

## Bağlam

Eski `olcum` koşucusu altı kaba iş yükünün yalnız medyanını terminale
yazıyordu. Sonuçlar elle Markdown'a geçiriliyor; ham örnek, p95 kuyruk
gecikmesi, typed-HIR fazı, LSP yaşam döngüsü ve süreç tepe belleği
görünmüyordu. CI koşucusunun değişken makine yükü de tek bir sayıyı sürüm
kapısına çevirmeyi güvenilmez kılar. Derleyici büyürken performans
gerilemesini görünür tutmak gerekiyor; fakat gürültüyü correctness arızası
gibi sunmak da yanlış güven üretir.

## Karar

1. `olcum`, release profilde iki ısınmadan sonra varsayılan 25 örnek alır ve
   nearest-rank yöntemiyle p50/p95 ile min/max ve ham örnekleri üretir.
   `--hizli` yalnız yerel duman koşusudur; arşiv tabanı değildir.
2. Dokuz ayrı yüzey gözlenir: lexer+parser, hazır AST üzerinde
   resolver/checker+HIR kanıt toplama, tam kaynak→typed-HIR ön ucu, boş
   runtime başlangıcı, önceden derlenmiş yürütme, LSP engine initialize,
   `didOpen`, `didChange` ve Unix'te süreç tepe RSS'i. K-153/ADR-050 gerçek
   dillsp process cold-start'ını onuncu ayrı yüzey olarak ekler. Ölçüm adı ile kapsamı
   aynı JSON/Markdown kaydında açıkça yazılır; typecheck sonucu saf tür
   çıkarımıymış gibi adlandırılmaz.
3. K-152/ADR-049 ile `zee-performans-2` JSON ve
   `zee-performans-gecmisi-2` TSV'si tam Git SHA, ayrı milestone, temiz/kirli
   çalışma ağacı, OS/CPU/RAM/Rust/release profili ve ölçüm başına gerçek
   örnek/ısınma/örnekleme semantiğini taşır. Bozuk veya eksik provenance
   fail-closed'dur; ayrıntılı v2 sınırı ADR-049'un otoritesindedir.
4. Her Linux CI koşusu JSON, Markdown ve yeni satırları içeren TSV'yi job
   summary ile 90 günlük indirilebilir artefakta yazar. İzlenen tarihçe CI'da
   doğrudan değiştirilmez; yeni taban ancak aynı makinede alınan sonuç
   incelendikten sonra kod+belge commitine girer.
5. Paylaşımlı CI'da hard performans eşiği yoktur. K-152 sonrası yalnız exact
   platform+OS+CPU+RAM+Rust+profil eşleşmesi karşılaştırma tabanı olabilir.
   `--esik-yuzde`, ancak makinesi ve yükü sabitlenmiş
   adanmış benchmark koşucusunda bilinçli olarak verildiğinde p95 eşiğini
   süreç hatasına çevirir; karşılaştırılabilir taban yoksa fail-closed'dur.

## Reddedilen seçenekler

- **Yalnız medyanı terminale yazmak:** kuyruk gecikmesini, ham dağılımı ve
  sonraki koşuların makinece karşılaştırılabilir kanıtını kaybeder.
- **Shared GitHub koşucusunu hard gate yapmak:** komşu iş yükü ve runner
  seçimini ürün gerilemesi sanarak rastlantısal kırmızılar üretir.
- **Tek “derleme” süresi kullanmak:** parse, checker ve HIR maliyetlerinin
  birbirini gizlemesine; LSP ve runtime yüzeylerinin görünmez kalmasına yol
  açar.
- **CI sonucunu izlenen tarihçeye otomatik commit etmek:** güvenilmeyen ve
  incelenmemiş gürültüyü kalıcı taban yapar.

## Sonuçlar

- K-148 başlangıç tabanı Apple M4 Pro/macOS arm64/Rust 1.93.1 üzerinde 25
  turla dokuz ölçümü kaydeder. Sonraki exact-ortam koşuları p50/p95 ve yüzde
  eğilimini doğrudan raporlar.
- Performans artık test faz sürelerinden ayrı bir artefakttır: test süresi
  correctness çalışmasının duvar saatidir; bu kayıt sabit kullanıcı iş
  yüklerinin gözlemidir.
- Dil sözdizimi, çalışma semantiği, tanı kataloğu, RFC ve normatif spec
  değişmez. B-040 kapanır. Ardından K-149/ADR-046 bağımlılık yönü/katman
  mimarisi denetimini ayrı kapı olarak tamamlamıştır.
- K-152 sayısal K-148 tabanını değiştirmeden exact kaynak commit'ine bağladı;
  RSS satırının tek süreç-tepe görüntüsü olduğu ADR-049 ile görünürdür.

K-207 ad çözümleme maliyetini azaltır; sabit 2k/5k/10k/20k iş yükü ve
gerçek sample/warmup semantiği değişmez. Taban ve sonrası ayrı temiz exact
commit, aynı makine/araç zinciri/release profile ile ölçülür; yerel paylaşılmış
masaüstü gözlemidir. Raw sample/hash/çevre olmadan yeni tarihçe yazılmaz.

K-237/ADR-100 yanıt tahsisini 16 KiB + 1 bayt kapasiteli tek çağrı tamponuyla sınırlar; küçük yanıtın başlangıç tahsisi de değişir. Sabit benchmark korpusu SMTP taşımadığından iş yükü/örnek semantiği ve immutable tarihçe aynı kalır. Byte/capacity kanıtı p95/RSS dağılımı değildir; performans iyileşmesi veya yeni taban iddiası yoktur.

## K-255 — Gözlemsel P19 checkpoint

Temiz exact021bff47839092706c7698d360ae5face9cd2a19 release Rust1.93.1
macOS26.6.2/M4Pro/24GiB, mevcut harness25sample/2warmup ve RSS1/0;
[raw proof](../docs/incelemeler/urunlesme-20261014/k255-lsp-olcum/measurement.json)
tam metadata/dağılımı taşır. DidChange2k–20k p95 7.490–177.321ms'dir.
Komşu yerel işler vardır; hard eşik ve yeni adanmış baseline yoktur.
Kaynak klonu önce/sonra temiz; kayıt yalnız actual checkpoint'e aittir.
Gerçek P08 ürün200ms/cache kabulü açık, eski tarihçe/fixture semantiği korunur.
