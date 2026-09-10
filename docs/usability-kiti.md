# Usability oturum kiti

Hedef: master planın Hafta 12 çıktısı — **10 öğrenci (8–14 yaş) + 5
profesyonel** ile korpusu sınamak. Bu kit, oturumu yürütecek kişiye (sana)
her şeyi hazır verir. Sonuçların bağlandığı karar: **RFC-0006 onay kapısı**
(K-032), **RFC-0019 gezme/değer semantiği onay kapısı** (K-093) ve
K-010/K-013 doğallık doğrulamaları.

K-016'nın teknik bağlamları, kör adayları ve önceden taahhütlü karar eşiği
[K-016 karar paketinde](k016-cagri-karar-paketi.md) bağlayıcıdır. Bu kit yalnız
oturum uygulamasıdır; sonuç geldikten sonra eşik değiştirilmez.

## Oturum düzeni (kişi başı ~33–38 dakika)

1. **Isınma (2 dk):** "Bilgisayara Türkçe komut veren bir dil deniyoruz.
   Doğru cevap yok; takıldığın her yer bizim hatamız, senin değil."
2. **K-016 serbest üretim (3 dk):** Hiç çağrı örneği göstermeden iki görevi
   yazdır; satırı düzeltme veya Zee kuralı öğretme.
3. **K-016 kör kartlar (8 dk):** Katılımcıya atanmış sırayla üç kartı uygula.
4. **Sesli okuma (7 dk):** Aşağıdaki programları KAĞITTAN sesli okut.
   Kural: satırı önce okusun, sonra "sence bu ne yapar?" — cevabı YAZ.
5. **K-180 sıra erişimi kartı (3 dk):** serbest üretim, kör tercih, taban
   sorusu (RFC-0029; eşiği aşağıda).
6. **K-093 tahmin kartları (7 dk):** İlk tahmin, tek cümlelik öğretim, ikinci
   tahmin; G3 yalnız profesyonel/isteyen çocuk.
7. **Genel yazma görevi (5 dk):** Küçük bir işi dilde yazmayı DENESİN.
8. **Kapanış (2 dk):** "En garip gelen satır hangisiydi?" — birebir not al.

## Okutulacak programlar (sırayla)

| Sıra | Program | Neyi sınıyor |
|---|---|---|
| 1 | golden/01, 02, 05 | temel akış, ise/değilse (K-005) |
| 2 | golden/06, 08 | döngüler, örtük çoğul `her sayı için` (K-013) |
| 3 | **golden/12 + 14** | K-016 kartları tamamlandıktan sonra çağrıyı gerçek programda doğrulama |
| 4 | golden/23, 32 | göre-eşleştirme, ondalık `3,14` (RFC-0013) |
| 5 | golden/07 | girdi + koşul zinciri (yalnız profesyonellere: 26 da) |

## K-016 özel protokolü (kritik)

Katılımcı golden/12, golden/14 veya başka bir çağrı satırı görmeden şunları
kağıda yazmayı dener:

1. “`notlar` listesinin ortalamasını hesaplayan işlemi çağır ve sonucu
   `ortalama` adlı değere bağla.”
2. “`selamla` işlemini `\"Ayşe\"` ve `10` ile çağır; bu işlem değer döndürmüyor.”

İcat ettiği yazımı birebir kaydet. Sonra anonim kimliğine göre atanmış kart
sırasını uygula: beş kişi A→B→C, beş kişi B→C→A, beş kişi C→A→B. Kartta
“mevcut/önerilen” yazmaz:

- **Kart A:** `ortalama notlar için ortalamayı hesapla olsun`
- **Kart B:** `notlar için ortalamayı hesapla, sonucu ortalama olsun`
- **Kart C:** `ortalama (notlar için ortalamayı hesapla) olsun`

Her kart için önce “Bu ne yapar?”, sonra “Bu çağrının sonucunu başka bir
işleme doğrudan vermek istersen nasıl yazarsın?” sor. Cevaptan sonra yalnız o
kartın tek cümlelik kuralını oku ve yeni adlarla tekrar sor. Doğallık,
anlaşılırlık ve “yazarken seçerim” puanlarını ayrı al; “hiçbiri, ben şöyle
yazardım” her zaman serbesttir.

Uygulayıcının okuyacağı öğretim cümleleri sabittir:

- A: “Sonuç adı başta, çağrı ortada; satır `olsun` ile biter.”
- B: “Önce çağrı yazılır; `sonucu ... olsun` bölümü dönen değere ad verir.”
- C: “Sonuç adı başta; çağrı parantezin içinde, satır `olsun` ile biter.”

Öğretim sonrası üç kartta da yeni görev aynıdır: “`fiyatlar` listesinin
medyanını hesapla ve sonucu `medyan` adına bağla.” Doğru zihinsel model,
katılımcının hem çağrılan işlemi hem bağlanan sonuç adını doğru göstermesidir;
yazım kusuru ayrıca kaydedilir ama anlam puanına gizlice eklenmez.

Karar basit A/B çoğunluğuyla verilmez. Toplam ve çocuk/profesyonel alt grup
eşikleri, yedi teknik bağlam ve B'nin tek başına iç içe ifade vermemesi
[karar paketinin §5'inde](k016-cagri-karar-paketi.md) önceden taahhütlüdür.
Kart B güçlü çıkarsa doğrudan ikinci sözdizimi eklenmez; K-097 ile dondurulan
RFC-0021 expression grammar mimarisi yeniden açılır. Ham veri olmadan A da
kesinleşmiş sayılmaz.

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

## K-180 sıra erişimi kartı (RFC-0029; K-016/K-093 eşiklerinden bağımsız)

K-016 ve K-093 protokolleri donuktur; bu kart onlara dokunmadan oturuma
**3 dakikalık ek adım** olarak girer (sesli okumadan sonra, K-093'ten önce).
Amaç: listede N'inci öğeye erişim için hangi Türkçe yüzeyin kendiliğinden
yazıldığını ve hangi sayma tabanının beklendiğini ölçmek.

1. **Serbest üretim (öğretmeden):** "Elinde `kelimeler` diye bir liste var;
   üçüncü kelimeyi ekrana yazdıran tek satırı yaz." Yazılanı birebir kaydet;
   düzeltme yok.
2. **Kör tercih:** Üç satırı karışık sırayla göster, "sana en doğal geleni
   seç, neden?" — cevabı yaz.

```dil
kelimelerin 3. öğesi yaz
kelimelerin üçüncü öğesi yaz
kelimelerin sıra. öğesi yaz
```

3. **Taban sorusu:** "`elma, armut, kiraz` listesinde `2. öğesi` sence
   hangisi?" — armut (1 tabanlı) mı kiraz (0 tabanlı) mı, birebir yaz.

Önceden taahhüt edilen karar eşiği:

- 15 kişinin **en az 10'u** serbest üretimde ya da kör tercihte aynı yüzeyi
  yazıyor/seçiyorsa o aday RFC-0029'un önerilen yüzeyi olur; taban sorusunda
  en az 12 kişi armut diyorsa 1 tabanı kesinleşir.
- Eşik oluşmazsa RFC-0029 taslak kalır ve ikinci karşılaştırmalı oturum
  yapılır; freeze içinde ve insan verisi olmadan yüzey seçilmez.

## Görev kartları

**Çocuk yazma görevi:** "Yaşını soran, 10'dan büyükse 'abisin/ablasın',
değilse 'kardeşsin' diyen programı yaz." (Beklenen kalıplar: diye sor,
yanıtın sayısı, ise/değilse.)

**Profesyonel yazma görevi:** "Bir liste sayının ortalamasını alan işlemi
tanımla ve çağır; sıfır bölme durumunu hatasını döndür ile ele al."

## Kayıt formu (kişi başı bir kopya)

```
Anonim kimlik C__/P__: ____   Tarih: ____   Kart sırası: ____
Onam/izin süreci: ____
K-016 gösterim öncesi bağlama yazımı (birebir): ____
K-016 gösterim öncesi cümle çağrısı yazımı (birebir): ____
Sesli okumada takılan satırlar (birebir): ____
Yanlış tahmin edilen çıktılar (program + beklenen/dediği): ____
K-016 A ilk/öğretim sonrası + iç içe kullanım + 3 puan: ____
K-016 B ilk/öğretim sonrası + iç içe kullanım + 3 puan: ____
K-016 C ilk/öğretim sonrası + iç içe kullanım + 3 puan: ____
K-016 son doğal/anlaşılır/yazma tercihi + birebir nedeni: ____
K-093 G1 ilk/öğretim sonrası: ____ / ____ — nedeni: ____
K-093 G2 ilk/öğretim sonrası: ____ / ____ — nedeni: ____
K-093 G3 (gösterildiyse): ____ — nedeni: ____
Yazma görevinde icat ettiği sözdizimi (ALTIN DEĞERİNDE — birebir): ____
"En garip satır": ____
Dört soru puanı (1-5): doğal __ / anlaşılır __ / tekrar ister mi __
```

## Sonuçların işlenmesi

1. [Anonim katılımcı şablonunu](usability-sonuclari/katilimci-sablonu.md)
   `docs/usability-sonuclari/YYYY-MM-DD-C01.md` gibi kopyala; isim, okul,
   e-posta, ses/video veya başka kişisel bilgi commit etme.
2. Her takılma bir günlük kaydına (K-0xx) dönüşür; kalıp icatları RFC
   alternatifi olarak kaydedilir.
3. On beş formdan sonra [özet şablonunu](usability-sonuclari/ozet-sablonu.md)
   doldur. K-016 sayımı RFC-0006'nın Durum satırına; G1/G2/G3 sayımı
   RFC-0019'a ve V1-P1-05'e işlenir. Ham sayı olmadan hiçbir kapı kapatılmaz.

> Dile adını veren kişi ilk sıcak/formative pilot olabilir:
> “Merhaba! Bu zee projesi.” satırını ilk o okusun. Dört yaşındaki bu pilot,
> 8–14 yaş karar örneklemine sayılmaz; protokolü değiştirirse yeni sürüm gerçek
> sayım başlamadan dondurulur.
