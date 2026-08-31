# zee dil spesifikasyonu (normatif çekirdek)

Bu klasör dilin **normatif** tanımıdır (master plan bölüm 23): bir cümlenin
geçerli olup olmadığı ve ne anlama geldiği konusunda son söz buradadır ve
buradan bağlanan RFC'lerdedir. Golden korpus ile regression testleri bu
tanımın yürütülebilir eşleniğidir — spec ile test çelişirse bu bir hatadır
ve hangisinin düzeltileceği RFC ile karara bağlanır.

## Sözleşme dili

- **ZORUNLU** — uyulmazsa program geçersizdir; derleyici tanı üretir.
- **YASAK** — dilde yeri yoktur; derleyici tanı üretir.
- **TANIMLI** — davranış budur, başka gerçekleme de aynı davranışı vermek
  zorundadır (determinizm sözü, RFC-0001).
- **AÇIK** — henüz karara bağlanmadı; bağlandığında RFC + sürüm notu ister.

Kırıcı değişiklik sessizce yapılamaz: davranış değişikliği RFC'den geçer ve
[docs/surumler.md](../docs/surumler.md)'de duyurulur.

## Bölümler

| Bölüm | Kapsam | Normatif kaynaklar |
|---|---|---|
| [01 — Sözcükleme](01-sozcukleme.md) | alfabe, tokenlar, kaçışlar, sayılar | RFC-0002, RFC-0013 |
| [02 — Dizim](02-dizim.md) | satır/blok yapısı, yüklem-sonlu dağıtım | RFC-0003, RFC-0006 |
| [03 — Adlar ve kapsam](03-adlar-ve-kapsam.md) | morfolojik çözüm, blok kapsamı | RFC-0004, K-011, K-034, K-041 |
| [04 — Türler](04-turler.md) | tür envanteri, birleşim, daraltma | RFC-0007, RFC-0008, RFC-0013 |
| [05 — Değerlendirme](05-degerlendirme.md) | yürütme sırası, taşma, determinizm | RFC-0001 §7, ADR-003 |
| [06 — Hata modeli](06-hata-modeli.md) | tanı sözleşmesi, Seçenek/Sonuç, test | RFC-0008, RFC-0010 |
| [07 — Birimler](07-birimler.md) | birim çözümü ve kapsülleme | RFC-0009 §2 |

## Faza bağlı — henüz spec dışı

Eşzamanlılık/iptalin tam anlamı (RFC-0011 yüzeyi geçici kabul; gerçek
paralellik Faz 5), FFI/ABI (RFC-0012, Faz 4/5), standart kütüphane kararlılık
politikası, paket çözümü (RFC-0009 §3), deprecation/edition modeli.
Bu başlıklar karara bağlandıkça buraya bölüm olarak eklenir.
