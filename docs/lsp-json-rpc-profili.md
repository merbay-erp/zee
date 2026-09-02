# LSP JSON-RPC bakım profili

Bu belge `dillsp` giriş sınırını değiştirenler için kısa uygulama rehberidir.
Normatif karar ADR-019 ve ADR-035, ortak kaynak zarfı spec/24'tür.

## Katmanlar

1. `dillsp` tek `Content-Length` çerçevesini byte olarak ve tahsis öncesi
   sınırlar. Başlık yeniden eşleştirilemiyorsa process fail-closed kapanır.
2. Tam okunmuş gövde UTF-8 ve RFC 8259 JSON olarak çözülür. UTF-8/sözdizimi
   hatası `-32700` üretir; sonraki sağlam çerçeve okunabilir.
3. JSON değeri tek-nesne LSP JSON-RPC zarfı olarak doğrulanır. Zarf hatası
   `-32600`, bilinmeyen yöntem `-32601`, yöntem parametresi hatası `-32602`dir.
4. Semantic handler yalnız doğrulanmış `method`, `params` ve kayıpsız `id`
   görür. Yanıt ve sunucu bildirimi 8 MiB bütçeli yazıcıdan çıkar.

## Değişmezler

- Sayı parser'ı genel `parse::<f64>()` kullanmaz. RFC 8259 lexeme'i doğrulanır
  ve kayıpsız tutulur; `NaN`/`Infinity` JSON değildir.
- Nesne alanları çözüldükten sonra tekildir. Unicode kaçışı duplicate anahtar
  denetimini atlayamaz.
- `jsonrpc` yalnız `"2.0"`; `method` yalnız metindir. `params` varsa nesne veya
  dizi, `id` varsa metin, sayı ya da null'dır.
- LSP stdio profili bir çerçevede tek request/notification taşır; JSON-RPC
  batch dizisi `Invalid Request`tir.
- Kimliksiz notification için result/error yanıtı yoktur. Sunucunun
  `publishDiagnostics` notification'ı bağımsız outbound mesajdır.
- Sayısal LSP position alanı kesir, üs veya eksi işareti taşıyamaz ve `usize`
  dönüşümü taşarsa parametre geçersizdir.

## Bir değişiklik nasıl kanıtlanır?

- Geçerli/geçersiz sayı çiftini RFC korpusuna ve differential teste ekle.
- Yeni zarf kuralı için parse error ile invalid request'i ayrı sınayan vaka
  yaz; duplicate alanın escaped eşdeğerini de düşün.
- Yeni request handler'ı kimliksiz bildirimde yanıt üretmemeli; bozuk params
  için kimlikli isteği bekletmemeli.
- Bütün yeni outbound yolları yeniden `json_coz` ile ayrıştır ve 8 MiB
  bütçesini koru.
- `cargo test --test lsp_testi --test mimari_sinir_testi --bin dillsp` ve
  `cargo clippy --all-targets --all-features -- -D warnings` kapılarını geçir.
