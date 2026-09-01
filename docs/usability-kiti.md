# Usability oturum kiti

Hedef: master planın Hafta 12 çıktısı — **10 öğrenci (8–14 yaş) + 5
profesyonel** ile korpusu sınamak. Bu kit, oturumu yürütecek kişiye (sana)
her şeyi hazır verir. Sonuçların bağlandığı karar: **RFC-0006 onay kapısı**
(K-032), **RFC-0019 gezme/değer semantiği onay kapısı** (K-093) ve
K-010/K-013 doğallık doğrulamaları.

## Oturum düzeni (kişi başı ~25 dakika)

1. **Isınma (2 dk):** "Bilgisayara Türkçe komut veren bir dil deniyoruz.
   Doğru cevap yok; takıldığın her yer bizim hatamız, senin değil."
2. **Sesli okuma (8 dk):** Aşağıdaki programları KAĞITTAN sesli okut.
   Kural: satırı önce okusun, sonra "sence bu ne yapar?" — cevabı YAZ.
3. **Tahmin görevleri (8 dk):** Program çıktısını tahmin ettir (aşağıda).
4. **Yazma görevi (5 dk):** Küçük bir işi dilde yazmayı DENESİN (kağıtla).
5. **Kapanış (2 dk):** "En garip gelen satır hangisiydi?" — birebir not al.

## Okutulacak programlar (sırayla)

| Sıra | Program | Neyi sınıyor |
|---|---|---|
| 1 | golden/01, 02, 05 | temel akış, ise/değilse (K-005) |
| 2 | golden/06, 08 | döngüler, örtük çoğul `her sayı için` (K-013) |
| 3 | **golden/12 + 14** | **K-016: `notlar için ortalamayı hesapla olsun` — ANA SORU** |
| 4 | golden/23, 32 | göre-eşleştirme, ondalık `3,14` (RFC-0013) |
| 5 | golden/07 | girdi + koşul zinciri (yalnız profesyonellere: 26 da) |

## K-016 özel protokolü (kritik)

Golden 12'deki çağrı satırını okuttuktan sonra iki kartı göster, hangisi
"daha doğal" sor ve NEDENİNİ yazdır:

- **Kart A:** `ortalama notlar için ortalamayı hesapla olsun`
- **Kart B:** `notlar için ortalamayı hesapla, sonucu ortalama olsun`

Sayım kuralı (önceden taahhüt — sonuca göre eğilme): 15 kişiden **10+**
B derse RFC-0006 revize edilir (B, dönüş değerli çağrılar için eklenir);
aksi halde geçici kabul (A) kesinleşir.

## K-093 gezme/değer kartları (kritik)

Önce hiçbir kural öğretmeden Kart G1 ve G2'nin son satır çıktısını ayrı ayrı
tahmin ettir. “Neden?” cevabını birebir yaz; doğru/yanlış deme. Ardından yalnız
şu tek cümleyi oku: **“Her öğe için dediğinde, döngü adının tur sonundaki
değeri listedeki aynı yere geri konur; listenin bir kopyası ayrı değerdir.”**
Kartları yeniden tahmin ettir.

**Kart G1 — alan yazma**

```dil
yapı Kutu
    adet TamSayı

kutular boş liste olsun
bir yeni Kutu olsun
birin adedi 1 olsun
kutulara biri ekle
her kutu için
    kutunun adedi 9 olsun
kutuların json metni yaz
```

Seçenekler: A `[{"adet":1}]` · B `[{"adet":9}]`

**Kart G2 — yeniden bağlama**

```dil
sayılar 1, 2, 3 listesi olsun
her sayı için
    sayı 7 olsun
sayıların json metni yaz
```

Seçenekler: A `[1,2,3]` · B `[7,7,7]`

**Kart G3 — kopya (yalnız profesyoneller; çocuk isterse göster)**

```dil
sayılar 1, 2 listesi olsun
yedek sayılar olsun
her sayı için
    sayı 7 olsun
yedeğin json metni yaz
```

Seçenekler: A `[1,2]` · B `[7,7]`

Önceden taahhüt edilen karar eşiği:

- Öğretmeden önce 15 kişinin **en az 10'u** G1 ve G2'de B/B diyorsa ve tek
  cümleden sonra **en az 13'ü** B/B diyorsa RFC-0019'un değer-sonuç modeli
  usability onayı alır.
- Öğretmeden önce en az 10 kişi G1=B, G2=A diyorsa “alan yazma yansır,
  yeniden bağlama yansımaz” alternatifi RFC'ye geri açılır; kod değiştirilmeden
  önce ikinci karşılaştırmalı oturum yapılır.
- İki eşik de oluşmazsa V1-P1-05 açık kalır; veri çoğaltılır, sonuç zorlanmaz.
- Profesyonellerin G3'te çoğunluğu B beklerse alias açıklaması ayrıca yeniden
  tasarlanır; bu sonuç tek başına paylaşılan referansı dile eklemez.

## Görev kartları

**Çocuk yazma görevi:** "Yaşını soran, 10'dan büyükse 'abisin/ablasın',
değilse 'kardeşsin' diyen programı yaz." (Beklenen kalıplar: diye sor,
yanıtın sayısı, ise/değilse.)

**Profesyonel yazma görevi:** "Bir liste sayının ortalamasını alan işlemi
tanımla ve çağır; sıfır bölme durumunu hatasını döndür ile ele al."

## Kayıt formu (kişi başı bir kopya)

```
Yaş/rol: ____   Tarih: ____
Sesli okumada takılan satırlar (birebir): ____
Yanlış tahmin edilen çıktılar (program + beklenen/dediği): ____
K-016 kartı: A / B — nedeni: ____
K-093 G1 ilk/öğretim sonrası: ____ / ____ — nedeni: ____
K-093 G2 ilk/öğretim sonrası: ____ / ____ — nedeni: ____
K-093 G3 (gösterildiyse): ____ — nedeni: ____
Yazma görevinde icat ettiği sözdizimi (ALTIN DEĞERİNDE — birebir): ____
"En garip satır": ____
Dört soru puanı (1-5): doğal __ / anlaşılır __ / tekrar ister mi __
```

## Sonuçların işlenmesi

1. Formları `docs/usability-sonuclari/` klasörüne tarih adıyla koy
   (`2026-09-XX-oturum-N.md`).
2. Her takılma bir günlük kaydına (K-0xx) dönüşür; kalıp icatları RFC
   alternatifi olarak kaydedilir.
3. K-016 sayımı RFC-0006'nın Durum satırına; G1/G2/G3 sayımı RFC-0019'a ve
   V1-P1-05'e işlenir. Ham sayı olmadan hiçbir kapı kapatılmaz.

> İlk pilot için en doğru ilk katılımcı bellidir: dile adını veren kişi.
> "Merhaba! Bu zee projesi." satırını ilk o okusun.
