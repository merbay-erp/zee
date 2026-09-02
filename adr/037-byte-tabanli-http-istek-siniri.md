# ADR-037 — Byte tabanlı HTTP/1.x istek sınırı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-140, B-053, RFC-0017, ADR-022, ADR-034, V1-P0-03

## Bağlam

Native web adaptörü başlık sonunu ham baytta arıyor, ardından başlığı `str`
olarak gevşek satır/parça işlemleriyle çözüyordu. Gövde
`String::from_utf8_lossy` ile dönüştürülüyor; bare-LF, obs-fold, NUL,
absolute-form hedef ve başlık alanı grameri tek bir parser sözleşmesinde
reddedilmiyordu. `Transfer-Encoding` ile duplicate `Content-Length` için yerel
kontroller bulunsa da framing kararı tek sahipli değildi ve mutation fuzz
hedefi yoktu.

Reverse proxy buffering ve bağlantı başına tek-worker sınırı etkiyi azaltır;
ancak farklı parser yorumları request smuggling için güvenilir bir temel
değildir. Backend'in kabul ettiği HTTP alt kümesi açık ve fail-closed olmalıdır.

## Karar

1. Native runtime, `http_istegi.rs` içindeki byte tabanlı parser'dan geçmeyen
   request-line, header veya gövdeyi Zee programına vermez.
2. Satırlar yalnız CRLF ile biter. Bare-LF, bare-CR, obs-fold, NUL, UTF-8 dışı
   veya denetim karakterli başlık ve geçersiz alan adı reddedilir.
3. Yöntem RFC token baytlarından oluşur. Yalnız `/` ile başlayan origin-form
   hedef ve `HTTP/1.0`/`HTTP/1.1` kabul edilir; absolute-form, authority-form,
   asterisk-form, fragment ve boşluklu request-line reddedilir.
4. `Transfer-Encoding` hiçbir yazımda desteklenmez. En çok bir
   `Content-Length` vardır; değer yalnız ondalık ASCII rakamdır. Gövde exact
   bu uzunlukta, NUL'suz ve UTF-8 olmak zorundadır. Eksik, fazla veya kayıplı
   dönüşüm yoktur.
5. 16 KiB başlık zarfı tahsis öncesi; 64 KiB gövde zarfı doğrulanmış
   `Content-Length` sonrasında uygulanır. Başlık ve gövde ayrı tamponlanır.
6. V1 sunucusu bağlantı başına tek request işler ve `Connection: close`
   döndürür. HTTP pipelining/chunked body söz değildir; ilk gövdenin ardından
   aynı okumada görülen ek bayt reddedilir.
7. Ayrı `http_istegi` libFuzzer hedefi ham `&[u8]` alır. CRLF, target, header,
   framing ve UTF-8 kararları kalıcı korpusla her ana testte de yeniden oynar.

## Reddedilen seçenekler

- **String'i kayıplı kurup sonra ayrıştırmak:** geçersiz baytı replacement
  karakterine dönüştürerek wire kimliğini değiştirir.
- **Aynı değerde duplicate Content-Length'i kabul etmek:** proxy/backend
  ayrımını büyütür; tek başlık daha dar ve denetlenebilir profildir.
- **Chunked body eklemek:** Stage 0 form yüzeyi için gereksiz yeni framing
  durumu ve smuggling alanı açar.
- **Yalnız reverse proxy'ye güvenmek:** backend parser'ı bağımsız güvenlik
  sınırıdır; yanlış proxy ayarı onu gevşetemez.

## Sonuçlar

- HTTP framing kararı socket akışından ve proxy başlık politikasından ayrı,
  260 satır bütçeli tek modüldedir.
- Geçerli istek programın beklediği aynı `YÖNTEM /yol` + UTF-8 gövde
  biçimine dönüşür; mevcut dil web semantiği değişmez.
- Bilerek desteklenmeyen HTTP biçimleri 400, başlık/gövde kaynak aşımı
  431/413 ve mutlak okuma süresi aşımı 408 olarak kalır.
- Fuzz crash'i küçültülüp korpusa ve adı konmuş regresyona alınmadan kapalı
  sayılamaz.
