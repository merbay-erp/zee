# ADR-022 — Lexer/parser fuzz politikası

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **Revizyon:** 2 Eylül 2026 — K-140/ADR-037 ham byte HTTP hedefini ekledi.
- **Revizyon:** 3 Eylül 2026 — K-156/ADR-055 cache dışı provenance'lı korpus artefaktını ekledi.
- **İlgili kayıt:** K-110, K-140, K-156, B-015, B-053, B-066, V1-P0-20
- **İlgili yüzey:** `compiler/fuzz/`, `compiler/tests/fuzz_korpusu_testi.rs`

## Bağlam

Production panic lintleri doğrudan `unwrap`/`expect`/panic makrolarını
engeller; ancak geçerli UTF-8 dizilerinin lexer ve recursive-descent parser'da
beklenmeyen bir indexing, taşma veya kontrol akışı paniği doğurmadığını tek
başına kanıtlamaz. Örnek testler de mutation kaynaklı bileşimleri aramaz.

## Karar

1. Tek libFuzzer hedefi `&str` kabul eder; UTF-8 olmayan byte dizileri dil
   kaynağı sayılmaz ve kaynak okuma sınırının sorumluluğundadır.
2. Başarılı lexer çıktısı hem normal hem hata-kurtarmalı parser API'sine verilir.
   Kurtarmalı yolun cümle/girinti senkronizasyonu K-113/ADR-024 ile ayrıca
   bağlanmıştır. Lexer tanısı geçerli ve beklenen bir sonuçtur; fuzz crash'i
   değildir.
3. Kalıcı başlangıç korpusu Unicode/homoglyph, emoji, birleştirici im, CRLF,
   sekme/girinti, metin kaçışı, büyük sayı, ondalık virgül ve blokları kapsar.
   Zee kalıp sözlüğü mutation'ı anlamlı tokenlara yönlendirir.
4. Her normal test koşusunda kalıcı korpus, 4.096 deterministik üretilmiş UTF-8
   kaynak ve 64 KiB büyük sayı/virgül saldırıları yeniden oynatılır.
5. Gece işi, sabit `nightly-2026-08-31` ve `cargo-fuzz 0.13.2` ile beş dakika
   koşar. Girdi başına üst sınır 64 KiB, timeout beş saniyedir. Korpus CI cache'i
   ile büyür; crash girdisi artifact olarak saklanır.
6. Her doğrulanmış crash önce küçültülür, sonra kalıcı korpusa ve mümkünse adı
   konmuş bir regresyon testine eklenmeden düzeltilmiş sayılmaz.
7. K-140'ta bu politika native web sınırına genişler. `http_istegi` hedefi
   `&[u8]` alır; request-line/header CRLF, target biçimi, TE/CL framing,
   NUL/UTF-8 ve exact gövde kararını mutasyona açar. HTTP korpusu geçerli ve
   fail-closed girdileri ana testte de yeniden oynatır.
8. K-156'da dört hedefin koşu sonu korpusu cache'ten bağımsız, SHA-256
   manifestli ve kaynak commit/run provenance'lı 90 günlük artefakt olur.
   Coverage seed'i ancak doğrulama, küçültme, stable replay ve insan review'u
   sonrasında kalıcı kaynak korpusuna girer.

## Sonuçlar

- Parser'ın kullanıcı girdisine karşı panic-free sözü örneklerden bağımsız,
  sürekli mutation aramasıyla denetlenir.
- Fuzz koşusunun zamana bağlı kanıt olduğu açık kalır; “bütün diziler biçimsel
  olarak ispatlandı” iddiası yapılmaz.
- 64 KiB fuzz sınırı compiler'ın daha büyük dosyaları kabul etmesini yasaklamaz;
  daha büyük kaynakların bellek/zaman bütçesi ayrı resource-hardening işidir.

## İlk kanıt

Apple ARM64 üzerinde aynı sabit araç zinciriyle 31 saniyelik yerel smoke koşusu
1.048.287 giriş yürüttü; crash, panic ve timeout üretmedi. Üç kalıcı regresyon
testiyle ana test toplamı 432'ye çıktı.
