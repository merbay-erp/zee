# 01 — Sözcükleme

Normatif kaynak: RFC-0002 (geçici kabul), RFC-0013. Tanı kodları:
[hata-katalogu](../docs/hata-katalogu.md) S bölümü.

## Kaynak metin

- Kodlama **ZORUNLU** UTF-8'dir.
- Tanımlayıcı alfabesi: Türkçe/Latin harfler (şapkalılar dahil: â î û),
  rakamlar ve alt çizgi; ilk karakter harf olmalıdır.
- Türkçe/Latin dışı, Türkçe harfe benzeyen karakter **YASAK** (S028 —
  homoglyph koruması; tanı kod noktasını gösterir).
- Birleştirici imler (U+0300–U+036F) **YASAK** (S029): `ğ` birleşik
  karakterdir, `g`+˘ değildir. v0 kuralı budur; tam NFC AÇIK (RFC-0002 §6).
- Anahtar kelime YOKTUR: hiçbir kelime kullanıcıdan esirgenmez; anlam,
  cümle kalıbından çıkar (bkz. 02 — Dizim).

## Tokenlar

| Token | Biçim | Kural |
|---|---|---|
| Metin | `"..."` | Kapanmayan tırnak S002. Kaçışlar: `\"` `\\` `\n`; başkası S040 |
| TamSayı | rakamlar, isteğe bağlı bitişik `-` | i64 dışına taşan sabit S006 |
| Ondalık | `3,14` — virgül **bitişik** | tam ve kesir haneleri keyfî uzunluktadır; kaynak boyuyla sınırlıdır |
| Kelime | tanımlayıcı alfabesi | ekleriyle birlikte tek token |
| Virgül | `,` | liste ayracı — sonrasında boşluk beklenir |

## Bitişik virgül kuralı (TANIMLI — RFC-0013)

`3,14` tek Ondalık sabittir; `3, 14` iki öğedir. `3 ,14` gibi
boşluk-virgül-rakam dizisi belirsizdir ve **YASAK**tır (S033): tanı iki
düzeltmeyi de önerir. Negatif sabitte işaret rakama bitişiktir: `-3,14`
tek sabittir ve işaret gövdeye bir kez uygulanır.

Ondalık sabit katsayısı TamSayı'nın i64 sınırına bağlı değildir:
`123456789012345678901234567890,12` geçerlidir. Eski dokuz kesir hanesi
sınırı K-092 ile kaldırılmış, S032 yeniden kullanılmamak üzere ayrılmıştır.

## Girinti tokenları

Satır sonu, girinti ve çıkıntı sözcükleyici tarafından üretilir; kurallar
02 — Dizim'dedir. Girintide sekme **YASAK** (S003).

## Sağlamlık doğrulaması

K-110/ADR-022 fuzz hedefi, 64 KiB'a kadar geçerli UTF-8 mutation girdilerini
lexer ve iki parser yolunda yürütür. Unicode/homoglyph, emoji, combining im,
CRLF, sekme/girinti, metin kaçışı, dev sayı ve virgül korpusu her ana testte de
yeniden oynatılır. Kabul edilen sonuç token+AST veya kodlu tanıdır; process
panic'i kabul edilmez. İşletim ayrıntıları [fuzz rehberindedir](../docs/fuzzing.md).
