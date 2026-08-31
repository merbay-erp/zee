# A01 — İngilizce anahtar kelime karışımı

## Reddedilen

```
if yaş > 8:
    print("başlayabilir")
```

## Neden

Türkçe-first ilkesinin doğrudan ihlali. İngilizce keyword zorunluluğu, dilin
var oluş nedenini ortadan kaldırır. `print`, `if`, `for` kaynak kodda anahtar
kelime olarak geçmez (manifesto 1; v0.1 kabul kriteri: "kaynak kodda İngilizce
keyword yazmak gerekmez").

## Doğrusu

```
yaş 8 den büyükse
    "başlayabilir" yaz
```

## Beklenen tanı

Parser `if` gördüğünde öğretici hata: "Bu dilde koşul 'ise' kalıbıyla yazılır" +
öneri. (Sık yapılacak hata; hata kataloğuna girecek.)
