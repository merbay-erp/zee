# A02 — Çeviri makyajı (if→eğer katmanı)

## Reddedilen

```
eğer (yaş >= 8) {
    yaz("başlayabilir");
} yoksa {
    yaz("oyun zamanı");
}
```

## Neden

Bu, İngilizce dil iskeletine Türkçe etiket yapıştırmaktır: fiil-önce çağrı
(`yaz(...)`), süslü parantez, noktalı virgül, sembolik karşılaştırma aynen
durur. Manifesto 2: "çeviri dili değil". Türkçenin nesne→eylem akışı ve
yüklem-sonlu yapısı korunmalıdır.

## Doğrusu

```
yaş 8 veya daha büyükse
    "başlayabilir" yaz
değilse
    "oyun zamanı" yaz
```
