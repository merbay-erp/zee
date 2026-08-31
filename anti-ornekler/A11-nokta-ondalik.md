# A11 — Nokta ile ondalık yazımı

## Reddedilen

```
pi 3.14 olsun
```

## Neden

Türkçede ondalık ayracı virgüldür; nokta yabancı yazımdır (manifesto 1,
RFC-0013 §1). Nokta kabul edilseydi `3,14` ile `3.14` iki ayrı doğru yol
olurdu — "tek doğru biçim" ilkesi bozulurdu.

## Doğrusu

```
pi 3,14 olsun
```

## Beklenen tanı

S001 — özel öneriyle: "Ondalık ayracı Türkçede virgüldür: 3.14 değil 3,14 yaz."
(`nokta_ondalik_yonlendirmesi` testi bu tanıyı sabitler.)
