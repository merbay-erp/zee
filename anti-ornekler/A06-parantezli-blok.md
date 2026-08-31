# A06 — Girinti yerine parantezli blok

## Reddedilen

```
10 kez tekrarla {
    "Merhaba" yaz
}
```

## Neden

Blok yapısını girinti belirler (manifesto 5). İkinci bir blok mekanizması
(süslü parantez) hem gereksiz noktalama hem de "iki doğru yol" belirsizliği
getirir. Formatter idempotentliği de tek blok modeliyle kolay kalır.

## Doğrusu

```
10 kez tekrarla
    "Merhaba" yaz
```

## Not

Girintinin kesin kuralları (boşluk sayısı, tab davranışı, karışık girinti
hatası) RFC-0003'te sabitlenecek; korpus 4 boşluk kullanıyor.
