# A04 — Serbest doğal dil / niyet tahmini

## Reddedilen

```
yaş sekizden büyükse falan çocuğa güzel bir selam ver bence
mümkünse listedeki büyükçe sayıları göster
```

## Neden

"falan", "bence", "mümkünse", "büyükçe" — bunlar niyet ifadeleridir, talimat
değildir. Bunları çalıştırmak ancak tahminle (AI ile) olur; manifesto 4: AI
semantiğin parçası değildir, compiler hiçbir cümleyi tahmin etmez. v1 kontrollü
Türkçedir: desteklenen kalıplar grammar'da sayılıdır.

## Doğrusu

```
yaş 8 den büyükse
    "Merhaba" yaz

her sayı için
    sayı 100 den büyükse
        sayıyı yaz
```
