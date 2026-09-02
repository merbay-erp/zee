# RFC-0025 — Merkezî Kaynak Bütçesi

- **Durum:** **geçici kabul** (K-129 ortak profil, K-130 değer/bağlantı
  zarfı, K-131 domain görünümü, K-132 sınırlı LSP JSON üretimi ve K-143
  playground ön-tahsis bütçesi gerçeklendi)
- **Tarih:** 2 Eylül 2026
- **İlgili kayıtlar:** K-105, K-107, K-129/K-130/K-131/K-132/K-143,
  B-025/B-056, ADR-017/019/033/040, V1-P0-31/V1-P1-11
- **Gerçekleme:** `kaynak_sinirlari.rs`; lexer, proje yükleyici, runtime,
  kalıcı dosya, CLI, LSP ve playground tüketicileri

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

## 3. K-129/K-130/K-131/K-132 tablosu

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
| Tek üretilen metin | 16 MiB | C024 |
| Çalışma/istek saklanan değer tahsisi | yaklaşık 64 MiB | C024 |
| Çalışma/istek çıktısı | 16 MiB ve 100.000 olay | C023 |
| Süreç genelinde inbound+outbound ağ bağlantısı | 64 | C018 veya HTTP 503 |
| Dil veri dosyası okuması | 16 MiB | C012/C013 bağlamında görünür hata |
| LSP açık belge | 256 | S045 bildirimi |
| LSP toplam belge metni | 128 MiB | S045 bildirimi |
| LSP outbound mesajı | 8 MiB | JSON-RPC `-32001` |
| Playground kaynak metni | 8 MiB | `PLAYGROUND SINIR HATASI` |
| Playground soru girdisi | 1 MiB ve 4.096 satır | `PLAYGROUND SINIR HATASI` |

## 4. Uygulama sırası

Boyutu disk metadata'sından bilinen dosya tahsis öncesi reddedilir. Dosya
okuma ayrıca `sınır+1` baytta durur; metadata ile okuma arasındaki büyüme de
sınırsız tahsise dönüşmez. Lexer tokenı eklemeden, runtime koleksiyon öğesini
veya görev future'ını kurmadan, çıktı ise IO adaptörüne verilmeden önce bütçe
denetlenir. Reddedilen LSP güncellemesi önceki belgeyi ve toplam sayacı korur.

K-130'da metin birleştirme/değiştirme, HTML kaçışı, değer metni, keyfî
hassasiyetli sayı/para, JSON ve CSV sonucu bütçeli yazıcıyla büyür; bilinen
büyüme tahsisten önce reddedilir.
Ortam/list/sözlük yazımları ve görev ortamı klonları iade edilmeyen,
muhafazakâr saklama fişleri tüketir. Bu fiş gerçek resident-memory ölçümü
değildir; yeniden kullanımda fazla sayarak üst sınırı güvenli tarafta tutar.
Inbound kabul ve outbound DNS/bağlantı girişleri süreç-geneli RAII izni alır;
bütün dönüş/hata yolları izni bırakır.

K-131 sayısal varsayılanları değiştirmeden HTTP/ağ, web oturumu, IO izi, LSP,
paket/registry, tanı ve kalıcı dosya domain görünümlerinde toplar. Tüketici
modüller geriye uyumlu sabit adlarını koruyabilir; sayısal değerin tek sahibi
`VARSAYILAN_KAYNAK_SINIRLARI` olmak ZORUNDADIR.

K-132 LSP yanıtını tam `String` kurulduktan sonra ölçmez. JSON zarfı, kimlik,
kaçışlı metin, diagnostics ve rename düzenlemeleri aynı `SinirliJson` içinde
her append öncesi 8 MiB bütçesinden düşer. Rename yalnız aralık planını tutar;
yeni metinler birer birer üretilir. Taşma kısmi gövde yayımlamadan kimlikli
istekte `-32001`, bildirimde sınırlı `window/logMessage` üretir.

K-143 playground için ayrı domain görünümü ekler. Native köprü kaynak/soru
byte boyunu ve soru satır sayısını sahipli satır tablosundan önce denetler.
ABI v3 limitleri hosta bildirir ve aşırı kayıtlı tamponu kopyalamadan reddeder.
Kanonik tarayıcı UTF-8 boyunu önceden hesaplayıp `encodeInto` ile exact WASM
tamponuna yazar; sınırsız ara byte dizisi veya sessiz kesme kurmaz.

## 5. Determinizm ve tanılar

Aynı profil ve aynı giriş, aynı sınırda aynı tanı kimliğini üretir. Sınırda
olan değer kabul, bir fazlası red olur. Uygulama kaynak kıtlığında sessiz veri
kesemez, işi eksik başarılı gösteremez veya host panic'e düşemez.

## 6. Tamamlanma ve ayrı takip

K-143 ile playground'a özgü açık allocation-order boşluğu da kapandı; B-025,
B-056 ve V1-P0-31 kapalıdır. RFC tam kabul yerine proje genelindeki usability
kabul politikasına uyarak geçici kabulde kalır. Cancellation ve duvar-saati
invariant'ları bu bütçe sözleşmesinin değil B-026'nın ayrı kapsamıdır.
