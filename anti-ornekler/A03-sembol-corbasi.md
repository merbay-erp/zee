# A03 — Sembol çorbası

## Reddedilen

```
sonuç = (a!=b && c>=d) || !e ? x : y;
```

## Neden

`!=`, `&&`, `||`, `!`, `?:` başlangıç kullanıcısının önündeki asıl bariyerdir.
Manifesto 5: noktalama minimum. Mantık kelimelerle yazılır: "ve", "veya",
"değil", "eşit değilse".

## Doğrusu

```
a b ye eşit değilse ve c d den küçük değilse
    sonuç x olsun
değilse e doğru değilse
    sonuç x olsun
değilse
    sonuç y olsun
```

## Not

Ternary'nin kısa Türkçe karşılığı olup olmayacağı açık tasarım sorusudur;
"okunabilirlikten kısalık çıkarma" ilkesiyle değerlendirilecek.
