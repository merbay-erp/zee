# ADR-076 — Rota birimleri: `<ad> rotalarını kullan` (K-182)

**Tarih:** 9 Eylül 2026 · **Durum:** kabul · **Kanıt:** `compiler/tests/birim_testi.rs`

## Bağlam

Site E dogfood'unun `ana.dil` dosyası 145 KB'a ulaştı: public site, kimlik,
panel CMS, medya, yayın, ayarlar, e-posta ve güvenlik rotaları tek dosyada.
RFC-0009 birim modeli yalnız **tanımları** (işlem, yapı, test) dışa verir; üst
düzey rota cümleleri kapsüllüdür. Bu yüzden rotalar dosyalara bölünemiyordu
(inceleme P1-5).

## Karar

- Yeni kullanım türü **`<ad> rotalarını kullan`** (`KullanimTuru::Rota`):
  birimin üst düzey `... adresine istek geldiğinde` cümleleri bu satırın
  **yerine, birimdeki sırasıyla** ana programa katılır. Birimin tanımları da
  `birimini kullan` gibi alınır (elmas/çakışma kuralları aynı, A008/A009).
- Rota sırası anlamlıdır (ilk eşleşen rota kazanır); bu yüzden `rotalarını
  kullan` satırının konumu ana dosyada rota sırasını belirler — `GET "/" önekli`
  gibi yakalayıcı rotalar en sonda kalır.
- `birimini kullan` rota taşıyan bir dosyayı yine yalnız tanımlarıyla alır
  (kapsülleme korunur). Rotasız bir birim `rotalarını kullan` ile alınamaz
  (A010).
- Yeni kalıp kelimesi `rotalarını`; K-174 yüzey kaydı ürün sahibi kararıyla
  güncellendi.

## Sonuçlar

- ürün E paneli rotaları `kaynak/rotalar/*.dil` altında bölümlenir; `ana.dil`
  sunucu başlatma, sıra ve yakalayıcı rotayı taşır.
- Yetkinlik/etki çözümlemesi birleşik program üzerinde çalıştığı için rota
  birimleri ayrı bir güvenlik sınırı değildir; sınır proje bildiriminde kalır.
