# ADR-065 — Spec maddesi düzeyinde drift kapısı

- **Durum:** kabul
- **Tarih:** 5 Eylül 2026
- **İlgili kayıt:** K-171, ADR-010, V1-P1-22

## Bağlam

K-118 kanıt haritası her RFC/ADR/spec belgesini test dosyalarına bağlar; fakat
belge düzeyinde. Bir spec bölümünde on normatif madde varken tek test dosyası
bütün bölümü "kanıtlı" gösterebilir; bir maddenin metni değişse harita
kırılmaz; bir test işlevi silinse belge yine kanıtlı görünür. Üçüncü dış
inceleme K-171 ile spec/RFC/ADR ↔ gerçekleme ↔ test arasındaki drift'in madde
düzeyinde raporlanmasını istedi.

## Karar

1. **Madde tanımı mekaniktir.** `spec/NN-*.md` içinde kod bloğu ve tablo
   dışında kalan; büyük harfli `ZORUNLU*`, `ZORUNDA*`, `YASAK*`, `TANIMLI*`
   ya da `AÇIK*` işaretçisi taşıyan her paragraf, liste öğesi ve başlık bir
   maddedir. `AÇIK değil` biçimindeki olumsuzlama karara bağlanmış davranış
   sayılır. Madde kimliği dosya adı + sadeleştirilmiş metnin FNV-1a parmak
   izidir; metin değişince kimlik değişir ve kanıt yeniden incelenir.
2. **Exact test işlevi kanıtı.** `docs/spec-madde-kaniti-v1.tsv` her maddeyi
   `kanitli|kismi|acik` durumu ve `<dosya>::<işlev>` seçicileriyle kaydeder.
   Seçici var olan, `#[test]` taşıyan bir dosyadaki gerçek işlev olmalıdır.
   `kismi` ve `acik` en az 20 karakterlik açık gerekçe ister; AÇIK madde test
   taşıyamaz, normatif madde `acik` olamaz.
3. **Fail-closed drift.** `spec_drift --denetle` kayıtsız madde, spec'ten
   kaybolmuş kayıt, bayat özet, durum uyuşmazlığı, kayıp test işlevi ve bayat
   `docs/spec-drift-raporu.md` için CI'ı durdurur. Rapor bölüm başına
   madde/kanıtlı/kısmi/açık sayımını ve bütün kısmi/açık gerekçeleri
   deterministik basar.
4. **Kapsam sınırı.** RFC ve ADR gerekçe belgesidir; belge düzeyi kanıt
   haritası (K-118) onlar için yeterlidir ve değişmez. Seçicinin maddeyi
   gerçekten kanıtladığı hükmü kod incelemesinde kalır; CI yalnız varlığı,
   tazeliği ve sınıflandırmayı doğrular.

## Reddedilen seçenekler

- **Elle numaralanmış madde kimlikleri:** ekleme/silme numaraları kaydırır ve
  metin değişikliği görünmez kalır.
- **Yalnız belge düzeyi harita:** on maddelik bölümün tek testle kanıtlı
  görünmesine izin verir.
- **Bütün cümleleri madde saymak:** işaretçisiz açıklama cümleleri gürültü
  üretir; spec README'nin sözleşme dili zaten dört işaretçiyi normatif kılar.

## Sonuçlar

- İlk taban 169 madde: 151 kanıtlı, 16 kısmi, 2 açık. Kısmi maddeler
  gerçek boşluğu adlandırır (S005/S007/T015/C005 doğrudan olumsuz vakası,
  UTF-8 olmayan kaynak reddi, `__Host-` nitelik testi, gerçek PostgreSQL
  gerektiren sınır/TLS vakaları, X-Zee-CSRF başlığı, sürüm kimliği eşitliği,
  ilk DEP kaldırmasının tanı kanıtı); K-166/K-172 bu listeden beslenir.
- Yeni spec maddesi aynı committe kanıt satırı ister; kaldırılan madde
  kaydını da götürür. `spec_drift --taslak` kayıtsız maddeleri basar.
- Kapı bakım ikilisidir; dil semantiği, tanı ve normatif metin değişmez.
