# ADR-035 — Protokol-kesin LSP JSON-RPC sınırı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-138, B-051, V1-P0-17

## Bağlam

K-107 LSP çerçevesini ve JSON derinlik/düğüm bütçesini sınırladı; K-132 bütün
çıktıyı tahsis sırasında bütçeledi. Buna rağmen mini ayrıştırıcı sayıları
`f64` olarak tutuyor, JSON sayı karakterlerini durumlarına bakmadan topluyor
ve aynı nesne anahtarını birden çok kez kabul ediyordu. Bozuk JSON sessizce
yutuluyor; geçerli JSON içindeki bozuk JSON-RPC zarfı parse hatasından
ayrılmıyordu. Sayısal istek kimliği `i64`'e çevrilirken kesilebiliyor veya
doyurulabiliyordu.

Bu sınır editörden gelen byte'ı semantic LSP işlemlerine açar. Belirsiz alan
seçimi, sessiz hata ve kimlik dönüşümü istemciyle sunucunun farklı mesajları
gördüğü bir protokol uyuşmazlığıdır.

## Karar

- JSON sayısı RFC 8259 durum makinesiyle çözülür: isteğe bağlı eksi, kurallı
  tamsayı, isteğe bağlı kesir ve üs. Baştaki sıfır, eksik kesir/üs, `NaN` ve
  `Infinity` reddedilir.
- Sayı binary float'a çevrilmez. Doğrulanmış kaynak lexeme'i özel
  `JsonSayisi` içinde kayıpsız saklanır; JSON-RPC kimliği aynı lexeme ile
  yazılır. Böylece geçerli büyük üsler de `Infinity` iç durumuna dönüşmez.
- Bir nesnede çözülmüş Unicode adı aynı olan iki alan reddedilir. `"method"`
  ile `"\u006dethod"` aynı anahtardır. İlk/son alanı seçme davranışı yoktur.
- UTF-8 olmayan ya da sözdizimsel bozuk gövde `id:null` ile `-32700 Parse
  error` üretir. Çerçeve okunabildiyse sunucu sonraki mesaja devam eder.
- Sözdizimsel olarak geçerli fakat tek-nesne LSP JSON-RPC zarfına uymayan
  değer `id:null` ile `-32600 Invalid Request` üretir. `jsonrpc` tam `"2.0"`,
  `method` metin, `params` varsa nesne/dizi, `id` varsa metin/sayı/null olmak
  zorundadır. LSP stdio profili batch mesaj kabul etmez.
- Bilinmeyen kimlikli yöntem `-32601`, geçersiz yöntem parametresi `-32602`
  alır. Kimliksiz bildirimlere yanıt verilmez; geçerli belge bildirimlerinin
  ayrı `publishDiagnostics` çıktısı bu kuralın dışı değildir, sunucudan gelen
  yeni bir bildirimdir.
- JSON ayrıştırma `lsp/json.rs`, bütçeli çıktı `lsp/cikti.rs` sahibindedir.
  Kök LSP dosyası semantic yönlendirmeyi taşır; modül sınırları satır bütçeli
  mimari testle korunur.

## Sonuçlar

- Sayısal kimlik kesilmez; geçerli JSON girdisinden finite olmayan Rust
  `f64` değeri veya JSON dışı sayısal çıktı üretilemez.
- Parse ve request katmanı ayrıldığı için editör sessiz timeout yerine kararlı
  standart hata alır.
- Yinelenen alanın güvenlik açısından hangi değer sayıldığı artık istemci,
  parser veya semantic handler sırasına bağlı değildir.
- Yeni JSON değeri/yazıcısı eklendiğinde RFC 8259 sayı korpusu, yinelenen alan,
  standart hata kodları, bildirim sessizliği ve bütün outbound gövdelerin
  yeniden ayrıştırılması birlikte korunmalıdır.

## Kanıt

`compiler/tests/lsp_testi.rs`; RFC sayı olumlu/olumsuz korpusunu ve
`serde_json` differential kararını, kaçışla eşdeğer duplicate anahtarı,
`-32700/-32600/-32601/-32602` ayrımını, UTF-8 olmayan gövdede devamı,
bildirim sessizliğini ve çok büyük/kesirli kimliğin kayıpsız dönüşünü sınar.
`compiler/tests/mimari_sinir_testi.rs` ayrıştırıcı/çıktı sahiplerini ve 300
satırlık JSON modül bütçesini korur.
