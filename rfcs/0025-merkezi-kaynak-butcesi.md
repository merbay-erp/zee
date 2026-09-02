# RFC-0025 — Merkezî Kaynak Bütçesi

- **Durum:** **geçici kabul** (K-129 ilk ortak profil; B-025 heap/bağlantı
  dilimi açık)
- **Tarih:** 2 Eylül 2026
- **İlgili kayıtlar:** K-105, K-107, K-129, B-025, ADR-017/019/033, V1-P0-31
- **Gerçekleme:** `kaynak_sinirlari.rs`; lexer, proje yükleyici, runtime,
  kalıcı dosya, CLI ve LSP tüketicileri

## 1. Amaç

Zee programının hatası veya saldırgan girdisi host sürecinin sınırsız CPU,
bellek ya da çıktı tüketmesine dönüşmemelidir. Aynı resmî profil çocukların
yerel çalıştırıcısında, editörde ve profesyonel self-hosted kullanımda
öngörülebilir, Türkçe ve testlenebilir bir red davranışı vermelidir.

## 2. Tek politika

`VARSAYILAN_KAYNAK_SINIRLARI`, çalışma sırasında değiştirilemeyen tek değer
nesnesidir. Limit yükseltme bir ortam değişkeni, kaynak cümlesi veya paket alt
bağımlılığıyla yapılamaz. Gelecekte daha geniş bir profil gerekiyorsa adı,
sürümü, tehdit modeli ve üst sınırı ayrı RFC ile görünür olur.

## 3. K-129 tablosu

| Kaynak | Sınır | Red |
|---|---:|---|
| Tek `.dil` kaynağı | 8 MiB | S045 |
| Tek kaynak tokenı | 1.000.000 | S045 |
| Proje kaynak dosyası | 4.096 | P009 zincirinde görünür hata |
| Proje toplam kaynak metni | 128 MiB | P009 zincirinde görünür hata |
| Çağrı derinliği | 500 | C019 |
| Çalışma cümlesi adımı | 10.000.000 | C023 |
| Tek liste/sözlük | 1.000.000 öğe | C023 |
| Tek eşzamanlı grup | 1.024 görev | C023 |
| Çalışma/istek çıktısı | 16 MiB ve 100.000 olay | C023 |
| Dil veri dosyası okuması | 16 MiB | C012/C013 bağlamında görünür hata |
| LSP açık belge | 256 | S045 bildirimi |
| LSP toplam belge metni | 128 MiB | S045 bildirimi |
| LSP outbound mesajı | 8 MiB | JSON-RPC `-32001` |

## 4. Uygulama sırası

Boyutu disk metadata'sından bilinen dosya tahsis öncesi reddedilir. Dosya
okuma ayrıca `sınır+1` baytta durur; metadata ile okuma arasındaki büyüme de
sınırsız tahsise dönüşmez. Lexer tokenı eklemeden, runtime koleksiyon öğesini
veya görev future'ını kurmadan, çıktı ise IO adaptörüne verilmeden önce bütçe
denetlenir. Reddedilen LSP güncellemesi önceki belgeyi ve toplam sayacı korur.

## 5. Determinizm ve tanılar

Aynı profil ve aynı giriş, aynı sınırda aynı tanı kimliğini üretir. Sınırda
olan değer kabul, bir fazlası red olur. Uygulama kaynak kıtlığında sessiz veri
kesemez, işi eksik başarılı gösteremez veya host panic'e düşemez.

## 6. Açık işler

Canlı değer grafiğinin yaklaşık toplam byte/öğe muhasebesi, bütün metin üretim
operasyonları, toplam bağlantı sayısı ve eski domain limitlerinin tamamının
`KaynakSinirlari` içine taşınması B-025'te sürer. Cancellation invariant'ları
B-026'nın ayrı sözleşmesidir.
