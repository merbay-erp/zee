# A09 — "AI anlar" varsayımı

## Reddedilen

```
# aşağıda ne demek istediğimi anla ve uygun şekilde çalıştır
müşteri verilerini güzelce özetleyip önemli olanları vurgula
```

## Neden

Programın anlamı yorumlayıcının "anlayışına" bırakılamaz; aynı kaynak bugün ve
on yıl sonra, çevrimiçi ve çevrimdışı, aynı davranışı üretmelidir (manifesto
3-4, bölüm 28: araçlar kullanıcı kodunu buluta göndermez). AI editörde
yardımcıdır: kod üretir, açıklar, hata çözer — ama ürettiği kod da bu dilin
deterministik grammar'ından geçer.

## Doğrusu

"Özetle" ve "önemli" tanımlı işlemlere iner:

```
özet müşteriler için satış özetini hesapla olsun
özetteki her satır için
    satırın tutarı 10000 den büyükse
        satırı yaz
```
