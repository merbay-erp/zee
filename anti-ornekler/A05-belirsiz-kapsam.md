# A05 — Belirsiz kapsam ve iki okunuşlu cümle

## Reddedilen

```
küçük sayıları ve harfleri yaz
toplamı 5 ile 3 ün çarpımıyla artır
```

## Neden

İlk satırda "küçük" neyi niteliyor — yalnız sayıları mı, harfleri de mi?
İkincisinde artış miktarı "5 ile (3 ün çarpımı)" mı, "(5 ile 3) ün çarpımı" mı?
Aynı kaynak iki farklı AST'ye gidebiliyorsa dilde yeri yoktur (manifesto 3).
Grammar bu tür yapıları ya tek okunuşa bağlamalı ya da reddetmelidir;
reddederken "iki türlü anlaşılabilir, şöyle ayır" diyen tanı verilmelidir.

## Doğrusu (tek okunuşa ayrılmış)

```
küçük sayıları yaz
harfleri yaz

çarpım 5 ile 3 ün çarpımı olsun
toplamı çarpımla artır
```
