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
  değişmez. K-153 gerçek process→stdio LSP cold-start metriğinin ayrı işidir.
- Dil grammar'ı, runtime davranışı, tanılar, RFC ve normatif spec değişmedi.
