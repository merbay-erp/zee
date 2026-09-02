# ADR-052 — Semantic regresyon provenance zinciri

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-155, B-063

## Bağlam

K-147'nin 17 vakalık korpusu bug kimliği, faz, kaynak ve gözlenebilir sonucu
kalıcılaştırdı; ancak “hangi commit düzeltti, hangi commit bozdu ve hangi Zee
sürümünden itibaren bu güvence var?” sorularını cevaplamıyordu. K-numarası bir
karar kaydıdır, exact kaynak revizyonu değildir. Ayrıca yeni bir compiler bug
düzeltmesinin fixture ekleme kuralı yalnız yazılı politikaydı.

## Karar

1. Kanonik manifest `regression/v2.tsv` olur. Her satır eski alanlara ek olarak
   tam 40 haneli `fixed_by`, biliniyorsa tam `introduced_by` (bilinmiyorsa
   açık `-`) ve SemVer biçimli `guaranteed_since` taşır.
2. İlk 17 vaka için `fixed_by`, ilgili K-kaydının gerçek düzeltme commit'ine
   bağlanır. Fixture'lar K-147 ile yayımlanmadan önce minimal reproducer
   bulunmadığından geriye dönük `introduced_by` tahmini yapılmaz; bilinmeyen
   veri sahte kesinlikle doldurulmaz. Garanti serisi `0.8.0-dev`dir.
3. Manifest testi SHA biçimini, commit varlığını, `fixed_by` revizyonunun HEAD
   atası olmasını ve varsa `introduced_by`→`fixed_by` tarih yönünü denetler.
   Garanti sürümü kanonik geliştirme serisiyle uyuşmalıdır.
4. Git tabanlı koruk v1→v2 göçünü okuyabilir; v2 yayımlandıktan sonra vaka,
   bug, dosya, fixed commit veya garanti sürümü yeniden yazılamaz.
   `introduced_by` yalnız `-` değerinden, `fixed_by` atası olduğu kanıtlanan
   tam SHA'ya bir kez zenginleştirilebilir; geri alınamaz veya değiştirilemez.
5. `compiler/src` altında düzeltme bildiren (`fix`, `bugfix`, `bug`, `duzelt`
   veya `düzelt`) her commit, aynı karşılaştırma aralığında `fixed_by` alanı o
   exact SHA olan yeni bir manifest satırı ister. Akış bu nedenle iki atomik
   adımdır: önce düzeltme commit'i; sonra minimal fixture+provenance commit'i.

## Reddedilen seçenekler

- **K-numarasını fixed commit saymak:** karar kimliğini kaynak kimliğiyle
  karıştırır ve checkout edilebilir provenance vermez.
- **Introduced commit'i tahmin etmek:** eski reproducer olmadan güvenilir
  bisect yapılamaz; yanlış soy ağacı bilinmeyen değerden daha zararlıdır.
- **Serbest metin sürüm notu:** makinece tekillik, biçim ve değişmezlik kapısı
  kurulamaz.
- **Bug fix ile aynı commit'in SHA'sını manifestte istemek:** commit kendi
  hash'ini önceden taşıyamaz; döngüsel ve üretilemez bir sözleşmedir.

## Sonuçlar

- “Bug düzeldi” ifadesi exact kaynak commit'i, kalıcı reproducer ve garanti
  serisi olmadan CI'dan geçemez.
- Gelecekte gerçek bir introduced commit bisect ile kanıtlanırsa `-` alanı
  tek yönlü ve ata denetimli biçimde zenginleştirilebilir; SHA geri alınamaz
  veya başka bir commit'le değiştirilemez.
- Grammar, runtime davranışı, tanı anlamları, RFC ve normatif spec değişmedi.
