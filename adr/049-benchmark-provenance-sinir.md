# ADR-049 — Benchmark provenance ve örnekleme sınırı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-152, B-060

## Bağlam

K-148 performansı dokuz ayrı yüzeye, ham örneklere ve p50/p95 tarihçesine
ayırdı; fakat ilk TSV'deki `revizyon=K-148` gerçek Git kimliği değildi.
Milestone ile kaynak commit'i aynı alana sıkışmıştı. OS sürümü, fiziksel RAM ve
build profili satırda yoktu. Ayrıca bütün satırlar `tur=25` dediği halde süreç
tepe RSS'i gerçekte koşunun sonunda tek kez okunuyordu. Bu kayıtlar sayısal
olarak yararlı olsa da bağımsız tekrarlanabilirlik iddiası için eksikti.

## Karar

1. JSON şeması `zee-performans-2`, kalıcı TSV şeması
   `zee-performans-gecmisi-2` olur. Her tarihçe satırı tam 40 haneli küçük-hex
   `git_sha` ve bundan ayrı bir `milestone` taşır.
2. Kayıt düzeyi provenance `git_dirty=false`, platform/architecture, OS
   sürümü, CPU, fiziksel `ram_bytes`, tam `rustc` sürümü ve
   `build_profile=release` alanlarını zorunlu tutar. Aynı kayıt içindeki bu
   alanlar ayrışamaz.
3. Her ölçüm gerçek `sample_count`, `warmup_count` ve `sampling_semantics`
   taşır. Süre ölçümleri bağımsız turlardır. `tepe_bellek`, bütün koşunun
   sonunda alınan tek süreç `ru_maxrss` görüntüsüdür; yalnız 1 örnek, 0 ısınma,
   KiB ve p50=p95 ile geçerlidir.
4. `--gecmis-cikti` yalnız verilen SHA ölçülen HEAD ile exact aynıysa ve Git
   çalışma ağacı temizse üretilebilir. Kirli yerel ölçüm yapılabilir; rapor
   bunu `git_dirty=true` diye açıklar fakat kalıcı tarihçe yazamaz.
5. İlk K-148 kaydı, onu oluşturan exact kaynak ağacı
   `df737f643c4ee9c8525ce7e972660230e75f5f45`, ayrı K-148 milestone'u,
   kayıtlı makine/OS/RAM/Rust/release bağlamı ve gerçek RSS örnek semantiğiyle
   v2'ye göç eder. Sayısal gözlemler değiştirilmez.

## Reddedilen seçenekler

- **K-numarasını revizyon saymak:** bir kaynak ağacını checkout etmeye yetmez.
- **Kısa SHA kabul etmek:** depo büyüdükçe tekillik garantisi zayıflar ve kayıt
  tam provenance değildir.
- **Kirli ağacı sessizce HEAD'e bağlamak:** ölçülen kod commit'ten farklı
  olabilir.
- **RSS satırına 25 tur yazmak:** 25 bağımsız bellek örneği varmış izlenimi
  üretir.
- **Yalnız serbest metin makine notu:** alanların varlığını, türünü ve kayıt
  içi tutarlılığını fail-closed doğrulayamaz.

## Sonuçlar

- Her kalıcı benchmark satırı checkout edilebilir kaynak commit'i ve yeniden
  kurulabilir koşu ortamını taşır.
- JSON ve Markdown kirli/temiz Git durumunu görünür kılar; CI gerçek
  `GITHUB_SHA` ile ayrı `shared-ci` milestone'u verir.
- Eski TSV v1 bilinçli olarak reddedilir; sessiz veya eksik göç yoktur.
- Eşik politikasının shared CI'da gözlem, adanmış koşucuda açık seçenek olması
  değişmez. K-153/ADR-050 gerçek process→stdio LSP cold-start metriğini ayrı
  yüzey olarak eklemiştir; exact ilk tabanı ayrıca kaydedilecektir.
- Dil grammar'ı, runtime davranışı, tanılar, RFC ve normatif spec değişmedi.

K-237/ADR-100 helper byte/capacity sayacı ve native fixture son tarihi benchmark örnekleme/provenance kaydı değildir. Tahsis etkisi incelendi; sabit iş yükü/sample/warmup ve immutable tarihçe korunur. Yeni p95/RSS kaydı ancak temiz exact düzeltme SHA’sı ve tam makine/Rust/release/örnek metadata’sıyla ayrıca kabul edilir.

## K-255 — Gözlemsel P19 checkpoint

Temiz exact021bff47839092706c7698d360ae5face9cd2a19 release Rust1.93.1
macOS26.6.2/M4Pro/24GiB, mevcut harness25sample/2warmup ve RSS1/0;
[raw proof](../docs/incelemeler/urunlesme-20261014/k255-lsp-olcum/measurement.json)
tam metadata/dağılımı taşır. DidChange2k–20k p95 7.490–177.321ms'dir.
Komşu yerel işler vardır; hard eşik ve yeni adanmış baseline yoktur.
Kaynak klonu önce/sonra temiz; kayıt yalnız actual checkpoint'e aittir.
Gerçek P08 ürün200ms/cache kabulü açık, eski tarihçe/fixture semantiği korunur.

## K-257–K-260 performans ve kaynak etkisi incelemesi

K-257 opt-in request duration/counter ve actual pool observation durumunu sabit
22/8 diziler, doyumlu sayaçlar ve en çok512byte atomic-size nonblocking FIFO
snapshot ile ekler. K-259 kontrol/activation tamponları ve beklemeleri bounded;
K-258 logu PIPE_BUF/nonblocking, tick ve lease lifetime sınırlıdır. Busy/idle/wait
sayaçları veya1000 HTTP denemesi `olcum` bağımsız sample/warmup dağılımı değildir.
Sentetik1000 concurrency1/dört rollover kabulü p95/RSS iyileşmesi ya da genel SLA
kanıtı olarak yorumlanmaz; credential7+2 ve heartbeat35sn de benchmark değildir.

Sabit `olcum` fixture/sample/warmup semantiği bu dalgada değiştirilmedi. Yeni
makine/Rust/release/temiz exact kaynak ve raw dağılım kabulü olmadığından
`performans-gecmisi-v2.tsv` eski satırları ve baseline'ı korunur. Shared CI'da
`--esik-yuzde` kullanılmaz; yeni hard eşik açılmaz (ADR-045/049).
Native counter state ve uzun ömürlü CLI/runtime worker etki incelemesi yeni
exact source1800sn soak ister;900sn host işletim tatbikatı da ayrı bekler.
Önceki kaynak soak veya LSP177.321ms checkpoint'i yeni kaynak adına taşınmaz.
