# 04 — Türler

Normatif kaynak: RFC-0007, RFC-0008, RFC-0013 (üçü de geçici kabul).
Tanı kodları: T bölümü.

## Tür envanteri

TamSayı (i64) · Ondalık (onluk: gövde × 10⁻ᵏ, k ≤ 9) · Metin · Mantıksal ·
Liste\<T\> · Sözlük\<Metin, T\> (ekleme sırası korunur) · Seçenek\<T\> ·
Sonuç\<T\> · Tarih · Saat · Süre · yapı türleri · görev/ağ yanıtı (yüzeyleri
RFC-0011/golden 24-27'de).

## Değişmezlik ve çıkarım (TANIMLI)

- Tür bildirimi ifadelerde yazılmaz; tür ilk değerden çıkarılır. Yapı
  alanları istisnadır: public yüzeyde tür **ZORUNLU** yazılır (RFC-0007).
- Bir adın türü sonradan **değişemez** (T002); yeni anlam yeni ad ister.
- Bir listenin bütün öğeleri aynı türdendir (T011); sözlük değer türü
  sözlüğün türüne uyar (T021).

## Boş koleksiyon çıkarımı (TANIMLI — K-045)

`boş liste` / `boş sözlük` belirsiz öğe/değer türüyle doğar; **ilk**
ekleme/atama türü somutlar ve bağlamı günceller. Belirsizken okuma
(`ilki`, gezme, `değeri`) derleme hatasıdır. Boş sabit, somut eşiyle
yeniden atamada iki yönde uzlaşır — bu T002 sayılmaz. Mantıksal bir ad
tek başına koşuldur: `hazır ise` (K-044); Mantıksal olmayan ad T005.

## Sayısal genişleme (TANIMLI)

TamSayı, Ondalık beklenen yerde kendiliğinden Ondalığa genişler. Tersi
örtük DEĞİLDİR: Ondalıktan tam sayıya `tam kısmı` (sıfıra doğru) ya da
`yuvarlanmışı` (yarımlar sıfırdan uzağa) ile bilinçli inilir.

## İşlem imzaları (TANIMLI — v0 monomorfizmi)

İşlemin parametre ve dönüş türleri **ilk çağrıda** sabitlenir; sonraki
çağrılar imzaya uymak **ZORUNLU**dur (T015 sayı, T017 tür). Özyinelemeli
çağrının türü "o ana dek görülen dönüşlerden" çıkarılır; bu yüzden temel
durum özyinelemeli çağrıdan önce en az bir dönüş vermiş olmalıdır (T035).

## Dönüş birleşimi (TANIMLI — RFC-0008)

Bir işlemin dönüş kümesi birleşir:

- {T} → T
- {T, `yok`} → Seçenek\<T\> (yalnız `yok` **YASAK**: içi belirlenemez, T018)
- {T, `hatasını döndür`} → Sonuç\<T\> (yalnız hata **YASAK**, T018)
- İki farklı değer türü **YASAK** (T018).

Başarı dalları otomatik sarmalanır: `sayıyı döndür`, Sonuç\<TamSayı\>
işlemde başarı olarak sarılır.

## Akış-duyarlı daraltma (TANIMLI — K-037)

`X varsa` / `X başarılıysa` / `X başarısızsa` kollarında ve bunların
`değilse` tersinmelerinde `X in değeri` / `X in hatası` erişimi güvenlidir.
Bu dalların DIŞINDA korumasız erişim **YASAK**tır (T036 — derleme hatası).
Daraltma bilinçli olarak dardır: tek koşullu kol + `değilse`; tam veri
akışı analizi yoktur (anlaşılabilirlik ilkesi). C008/C009 çalışma zamanı
kodları yalnız iç savunmadır; kullanıcı programı onları göremez.
