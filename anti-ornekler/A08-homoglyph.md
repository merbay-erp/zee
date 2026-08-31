# A08 — Homoglyph / confusable tanımlayıcı

## Reddedilen

```
sаyı 5 olsun
sayı 7 olsun
sayıyı yaz
```

İlk satırdaki `sаyı` içindeki "а" Kiril harfidir (U+0430); ekranda Latin
"a"dan ayırt edilemez.

## Neden

İki görünüşte özdeş, gerçekte farklı tanımlayıcı — hem öğrenciyi deliye
çevirir hem de supply-chain saldırı yüzeyidir (master plan bölüm 18).
Kaynak NFC'ye normalize edilir; confusable karışımı compiler uyarısı/hatası
üretir.

## Doğrusu

Tek alfabe: Türkçe/Latin. Karışık script tanımlayıcı ancak açık gerekçeyle
(FFI adı vb.) ve görünür işaretle kabul edilebilir.
