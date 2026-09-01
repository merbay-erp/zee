# 17 — Değer Semantiği ve Gezme

Normatif kaynak: RFC-0019. Tanı kodu: T053.

## Genel değer modeli (TANIMLI)

Bütün kullanıcı değerleri değer semantiğine sahiptir. Bir değerin bir ada
bağlanması, listeye/sözlüğe konması, işlem argümanı yapılması, döndürülmesi veya
görev snapshot'ına alınması gözlemlenebilir paylaşılan nesne kimliği oluşturmaz;
iç içe bileşenleriyle bağımsız bir değer kopyası üretir.

Bu kural TamSayı, Ondalık, Metin, Mantıksal, Liste, Sözlük, Yapı, Seçenek,
Sonuç, Hata, Tarih, Saat, Süre ve AğYanıtı için geçerlidir. v1'de sıradan bir
değer için referans eşitliği, adresi veya paylaşımlı değiştirilebilir alias yoktur.

## Liste gezmesi: değer-sonuç imleci (TANIMLI)

`her X için` ve `<listedeki> her X için` kaynak adı bir Liste olduğunda:

1. Kaynak bir kez değerlendirilir; giriş anındaki uzunluk, sıra ve öğeler
   gezme snapshot'ını oluşturur.
2. Her özgün `i` sırası için `X`, snapshot öğesinin bağımsız kopyasına bağlanır.
3. Gövde `X`i aynı türde yeniden bağlayabilir. `X` bir Yapıysa alan yazma
   onun yerel değerini değiştirir.
4. Tur normal tamamlanınca `X`in son değeri kaynak listenin aynı `i` sırasına
   geri yazılır. Alan yazma ve yeniden bağlama arasında sonuç farkı YOKTUR.
5. Gövde `döndür` üretirse dönüş ifadesi önce değerlendirilir, sonra o turun
   geri yazması tamamlanır, ardından dönüş yayılır.
6. Döngü adı RFC-0004 blok kapsamına tabidir.

Örnek:

```dil
sayılar 1, 2, 3 listesi olsun
her sayı için
    sayı 7 olsun
sayıların json metni yaz       # [7,7,7]
```

## Kaynağın sabitliği (ZORUNLU)

Bir Liste veya Sözlük kaynak olarak gezilirken, o kaynak ad üzerinde aşağıdaki
işlemler YASAKTIR ve derlemede T053 üretir:

- kaynağı yeniden bağlama;
- listeye ekleme veya listeden silme;
- sözlük değeri yazma veya sözlük anahtarı silme;
- aynı kaynak adıyla iç içe ikinci gezme.

T053 kaynak adını gösterir ve değişiklikleri ayrı koleksiyonda toplayıp gezme
sonrası uygulamayı önerir. Bu kural, kaynak adıyla morfolojik olarak çözülen
`sayılara`/`sayılardan` gibi yüzeylerde A002'den önce uygulanır.

Kaynağın daha önce alınmış değer kopyası başka addır ve bağımsızdır. Kopyayı
değiştirmek kaynak üzerinde etki oluşturmaz; iç içe çift gezme gerekiyorsa
ikinci adla açık kopya alınabilir.

## Sözlük gezmesi (TANIMLI)

Sözlük kaynağı giriş anındaki anahtarları ekleme sırasıyla snapshot alır.
Döngü adı her turda anahtarın Metin kopyasıdır; sözlüğe geri yazma yapılmaz.
Kaynak sabitliği ve T053 kuralları Liste ile aynıdır.

## Conformance gereklilikleri

Bir uyumlu gerçekleme en az şunları kanıtlamalıdır:

- Yapı alanı yazmanın kaynak Liste öğesine yansıması;
- döngü adını yeniden bağlamanın her liste uzunluğunda aynı sıraya yansıması;
- önceden alınmış liste/yapı kopyasının değişmemesi;
- ekle, sil, kaynak yeniden bağlama, sözlük yazma ve aynı-kaynak iç içe gezme
  için T053;
- T053'ün kaynak adı, konum ve eyleme dönük öneri taşıması;
- sözlük anahtar sırası ve geri-yazmasız davranışın korunması.
