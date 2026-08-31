# ADR-002 — Parser stratejisi: elle yazılmış, yüklem-sonlu dağıtımlı iniş

- **Durum:** kabul
- **Tarih:** 31 Ağustos 2026

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

Pratt katmanı henüz yok: ifadeler kalıp-temelli ("yapılı kalıplar") okunuyor;
sembolik işleç önceliği diye bir şey olmadığından Pratt'a ihtiyaç doğmadı.

## Gerekçe

- Üreteçler Türkçe eklerin bağlamsal çözümünü (morfoloji aday-kök eşlemesi)
  ve "kalıplar alan-atamadan önce" gibi öncelik kurallarını ifade etmeyi
  zorlaştırırdı.
- Tanı kalitesi: her hata elle yazılmış Türkçe mesaj + öneri taşıyor
  (83 kod, katalog bekçili) — üreteç çıktısıyla ulaşılması zor.
- Maliyet: grammar tek kaynaktan (EBNF) üretilmiyor; RFC'ler + golden korpus
  + `bekle_dosya_sonu` düzeyinde el disiplini gerekiyor. Conformance suite
  (Faz 8) bu riski taşıyacak.

## Sonuçlar

Resmi EBNF, spesifikasyon dondurmasında (v1.0) parser'dan TÜRETİLECEK ve
differential testle doğrulanacak; parser'ın kendisi referans gerçekleme kalır.
