# 28 — Görünüm bloğu (TANIMLI — K-212, ADR-085)

`<ad> görünüm olsun` cümlesi girintili bir HTML ağacını Metin'e derler.

## Sözdizimi

```
form_html görünüm olsun
    form method "post" action adres class "editor-card"
        input type "hidden" name "_csrf" value csrf
        label
            "E-posta"
            input type "email" name "eposta" value eposta required zorunlu
        p class "muted"
            metin aciklama
        ham parca
```

- **Öğe satırı:** `etiket (ad değer)*`. Etiket ASCII harfle başlar, harf/rakam
  içerir ve küçük harfe çevrilir. Öznitelik adı kelime ya da tırnaklı metindir
  (`"data-modal"`, `"aria-label"`); harf, rakam, `-`, `_`, `:` içerebilir.
  Değer tek bir ifadedir (ad, sabit metin, sayı, `doğru`/`yanlış`).
- **Metin satırı:** `"sabit"` ya da `metin <ifade>`; kaçışlıdır.
- **Ham satırı:** `ham <ifade>`; kaçışsız. Yalnız zaten güvenli parçalar için.
- **Çocuklar:** öğe satırının altındaki girintili satırlar. Metin/ham satırının
  çocuğu olamaz (S046).

## Anlam

- Sonuç `Metin`; ad daha önce başka türle tanımlıysa T002.
- Öznitelik değeri Metin ise `ad="kaçışlı"`, TamSayı ise `ad="n"`, Mantıksal
  `doğru` ise yalnız `ad`, `yanlış` ise yok; başka tür T055.
- Metin/ham ifadeleri Metin olmalı (T055).
- Kaçış tablosu `html güvenlisi` ile aynıdır (`& < > " '`).
- Boş öğeler (`area base br col embed hr img input link meta param source track
  wbr`) kapanış etiketi almaz.
- Blok hermetiktir: yalnız ifade değerlendirmesi yapar; IO yoktur.

## Tanılar

- S046 görünüm satırı biçimi (boş blok, tek değer, çift olmayan öznitelik,
  geçersiz ad, beklenmeyen girinti).
- T055 görünüm metni/özniteliği tür hatası.

TANIMLI: Görünüm hedefi HIR sembol tanımı ve yazımı taşır; aynı ada
yeniden atama kimliği değiştirmez. Farklı rota birimlerinin aynı yerel
satır numarası ayrı HIR kapsamları üretir (K-213/ADR-086).
