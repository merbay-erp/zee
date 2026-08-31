# A10 — Türkçe keyword, İngilizce API yüzeyi

## Reddedilen

```
liste yeni ArrayList olsun
listeye push ile 5 ekle
uzunluk listenin length değeri olsun
str listenin toString sonucu olsun
```

## Neden

Anahtar kelimeler Türkçe olsa da API adları İngilizce kalırsa bariyer sadece
yer değiştirir; kullanıcı yine iki dil birden öğrenir. Standart kütüphane
yüzeyi Türkçedir (`Liste`, `ekle`, `adedi`, `metni`). Harici İngilizce API'ler
FFI sınırında kalır ve Türkçe wrapper ile sunulabilir (master plan bölüm 19).

## Doğrusu

```
sayılar boş liste olsun
sayılara 5 ekle
adet sayıların adedi olsun
metin sayıların metni olsun
```
