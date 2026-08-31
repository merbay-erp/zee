# A07 — Büyük/küçük harf ve İ/ı tuzakları

## Reddedilen (aynı programda)

```
Yaş 10 olsun
yaş 12 olsun
ISIM "Ayşe" olsun
isim ISIM olsun
```

## Neden

`Yaş` ve `yaş` farklı iki değişkense çocuk için görünmez bir tuzak doğar.
Türkçede ayrıca İ/i ve I/ı büyütme-küçültme kuralları İngilizce'den farklıdır
(`isim`in büyüğü `İSİM`dir, `ISIM` değil). Spesifikasyon büyük/küçük harf
davranışını Türkçe kurallarla kesin tanımlamalı (master plan bölüm 4); yalnız
büyük/küçük farkıyla ayrışan tanımlayıcılar en azından uyarı almalıdır.

## Doğrusu

Aynı kavrama tek yazım: `yaş`, `isim`. Tür adları büyük başlar (`Öğrenci`,
`Sonuç`) — değer adlarından bu düzeyde ayrışma kabul edilebilir (K-014 bulgu 3).
