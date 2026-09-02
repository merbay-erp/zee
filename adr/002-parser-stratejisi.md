# ADR-002 — Parser stratejisi: elle yazılmış, yüklem-sonlu dağıtımlı iniş

- **Durum:** kabul
- **Tarih:** 31 Ağustos 2026
- **Revizyon:** 1 Eylül 2026 — K-097/RFC-0021 ile ifade katmanları bağlandı
- **Revizyon:** 1 Eylül 2026 — K-099/ADR-012 ile cümle ve ifade handler'ları
  fiziksel modüllere ayrıldı
- **Revizyon:** 2 Eylül 2026 — K-119 ile formatter'ın tam parser-token
  eşdeğerliği kalıcı kapıya bağlandı
- **Revizyon:** 2 Eylül 2026 — K-126/ADR-030 ile her AST ifadesinin kesin
  token aralığı zorunlu kılındı

## Bağlam

Parser üreteci (LALR/PEG) mi, elle yazım mı? Türkçenin yüklem-sonlu yapısı ve
"hata mesajları öğretici olacak" ilkesi belirleyiciydi.

## Karar

**Elle yazılmış recursive descent**, iki özgün uyarlamayla:

1. **Yüklem-sonlu dağıtım:** cümle türü satırın SON kelimesinden seçilir
   (`... yaz`, `... olsun`, `... tekrarla`). İngilizce dillerdeki
   "ilk anahtar kelimeye bak" kuralının aynadaki karşılığı — Türkçenin
   sözdizimine parser mimarisi düzeyinde uyum.
2. **Anahtar kelimesiz sözcükleyici:** kelimelerin anlamı tamamen konumdan
   gelir; `not`, `sayaç` gibi kelimeler serbest kalır (RFC-0002 §5).

3. **Katmanlı ifade bölgeleri:** Cümle dağıtımından sonra değer/koşul bölgesi
   primary → erişim/postfix → çağrı → aritmetik → birleştirme → karşılaştırma
   → boolean güç sırasına bağlıdır (RFC-0021/spec-20). Her kalıp bölgenin
   tamamını tüketir; kısmi eşleşme başka anlama düşmez.

Pratt katmanı bugün yoktur: ifadeler Türkçe tam-bölge kalıplarıyla okunur;
sembolik serbest infix operatör önceliği olmadığı için Pratt tek başına ek
değer sağlamaz. Böyle bir yüzey gelirse bu ADR yeniden açılır.

## Gerekçe

- Üreteçler Türkçe eklerin bağlamsal çözümünü (morfoloji aday-kök eşlemesi)
  ve "kalıplar alan-atamadan önce" gibi öncelik kurallarını ifade etmeyi
  zorlaştırırdı.
- Tanı kalitesi: her hata elle yazılmış Türkçe mesaj + öneri taşıyor
  (83 kod, katalog bekçili) — üreteç çıktısıyla ulaşılması zor.
- Maliyet: parser normatif grammar'dan üretilmiyor; spec/20 katman tablosu,
  RFC-0021 uzatma protokolü, golden/anti-example ve bağımsız ifade conformance
  testi bu riski taşır.

## Sonuçlar

Niyet EBNF'si ve normatif katman sırası artık RFC-0021/spec-20'dedir; parser
referans gerçekleme kalır. B-005/K-099'da cümle ve ifade handler'ları
ADR-012'nin fiziksel modüllerine ayrıldı; sınırlar bu katmanları izler ve
davranış conformance testiyle korunur.
Gelecekte ikinci compiler geldiğinde spec/20 + ortak korpus kaynak olur;
Rust fonksiyon sırası normatif kaynak sayılmaz.

K-126/ADR-030'da parser'ın ifade üretimi `ayristirici/kaynak.rs` kapısına
bağlandı. Her yaprak ve bileşik ifade tükettiği token bölgesinden ayrı,
Unicode karakteri tabanlı kesin kaynak zarfı alır; boş, ters, taşan veya çok
satırlı bölge sessizce satır tahminine düşmez. Yeni ifade kalıbı, bu zarfı ve
alt ifadelerin bağımsız aralıklarını korumadan tamamlanmış sayılmaz.

Resmî formatter yalnız görünüşü değiştirir. Biçimleme öncesi ve sonrasındaki
`TokenTur` dizisi; `SatirSonu`, `Girinti`, `Cikinti` ve `DosyaSonu` dahil
birebir aynı değilse C011 ile hiçbir çıktı yazılmaz. Parser semantic AST'yi
yalnız bu yapısal token dizisinden deterministik kurar; token konumları kaynak
haritasıdır, anlam seçimi değildir. K-119'un 33 programlık property kapısı,
her golden kaynağın dağınık-boşluk varyantını biçimler, iki tam parser izini
kıyaslar ve parser'ın iki tarafı da kabul ettiğini doğrular.
