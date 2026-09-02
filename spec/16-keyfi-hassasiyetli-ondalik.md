# 16 — Keyfî hassasiyetli Ondalık

Normatif kaynak: RFC-0013 (K-092 revizyonu), ADR-029/K-125. Durum:
**TANIMLI**.

## Tek kullanıcı türü

Ondalık, imzalı keyfî uzunlukta bir tam sayı katsayı ile onluk ölçek taşır:

`değer = katsayı × 10⁻ölçek`

Katsayı i64/i128 sınırına bağlı değildir. Kaynaktaki `3,14` katsayı `314`,
ölçek `2` olarak doğar. Sağdaki gereksiz sıfırlar atılır; runtime ölçeği en az
birdir. Bu nedenle `1,50` ile `1,5` aynı değerdir, Ondalık türündeki tam değer
ise `2,0` biçiminde görünür.

TamSayı ayrı i64 türü olarak kalır. TamSayıdan Ondalığa örtük genişleme
kayıpsızdır; ters yön yalnız `tam kısmı` veya `yuvarlanmışı` ile açıktır.

## Sabitler ve dönüşüm

- Türkçe ondalık virgülü bitişiktir: `3,14`, `-0,0001`.
- Tam ve kesir hanesi için dil-semantik bir dokuz-hane sınırı yoktur.
- Nokta ondalık ayracı değildir; S001 Türkçe virgülü önerir.
- `3 ,14` belirsizdir ve S033'tür.
- `yanıtın ondalığı` ile `ondalığını almayı dene` aynı keyfî hassasiyetli
  çekirdeği kullanır; isteğe bağlı bitişik `-` kabul edilir.
- Eski S032 başka anlama verilemez; ayrılmış tarihsel koddur.

## Aritmetik

| İşlem | Sözleşme |
|---|---|
| toplama / çıkarma | ortak ölçeğe kayıpsız hizalanır, sonuç tamdır |
| çarpma | katsayılar çarpılır, ölçekler toplanır, sonuç tamdır |
| sonlu bölme | sade paydanın asal çarpanları yalnız 2/5 ise sonuç tamdır |
| sonsuz bölme | 34 anlamlı haneye, yarımlar sıfırdan uzağa yuvarlanır |
| karşılaştırma | ölçekten bağımsız gerçek değer karşılaştırılır |

İkilik kayan nokta hiçbir adımda kullanılmaz. Şunlar kimliktir:

```text
0,1 + 0,2 = 0,3
1,0 / 8 = 0,125
1,50 = 1,5
```

## Sonsuz bölüm bağlamı

Ondalık açılım sonlu değilse sonuç tam 34 anlamlı hane hedefler. Ondalık
noktasından önceki sıfırlar ve ilk anlamlı haneden önceki sıfırlar sayılmaz:

```text
1,0 / 3  = 0,3333333333333333333333333333333333
10,0 / 3 = 3,333333333333333333333333333333333
```

35. hane yarım veya üstüyse 34. hane mutlak değerce büyür; negatifte de
yarımlar sıfırdan uzağa gider. Sonuç büyük ya da çok küçük olsa da aynı kural
geçerlidir. Bağlam platformdan ve çalışma kipinden bağımsızdır; sessiz küresel
hassasiyet ayarı yoktur.

## Gösterim ve sınırlar

- Normal basım Türkçe virgül, `json metni` JSON noktası kullanır; ikisi de
  bilimsel gösterime kendiliğinden geçmez.
- `kuruşlusu` ve `binlikli kuruşlusu` iki haneye aynı sıfırdan-uzağa kuralıyla
  yuvarlar; büyük katsayıyı daraltmaz.
- `tam kısmı` sıfıra doğru kırpar, `yuvarlanmışı` en yakına yuvarlar. Sonuç
  TamSayı i64'e sığmıyorsa C002 verir; Ondalık değer kaybolmaz.
- Süre milisaniyeye ve i64'e bilinçli dar bir türdür; büyük Ondalık süre S006
  verir. Bu, Ondalık kapasite sınırı değildir.
- Temsil ölçeği u32 metadata taşır. Gerçek programda kaynak/bellek sınırı çok
  önce gelir; metadata taşması C002 ailesidir ve sessiz yuvarlama yapmaz.

## FFI ve binary float sınırı

Ondalık için C `float`/`double`, binary32 veya binary64'e örtük ABI eşlemesi
YASAKTIR. Binary kayan nokta ikinci bir Zee kullanıcı türü değildir. Mevcut
Stage 0 FFI yüzeyi sunmaz ve bu değerleri `Tur`/`Deger` envanterinde taşımaz.

Gelecekteki FFI köprüsü ancak deklarasyon ve çağrıda görünür **kayıplı**
işaretiyle, temsil hatalarını `Sonuç` olarak taşıyarak açılabilir. IEEE 754
yuvarlama yönü, signed zero, subnormal, taşma, NaN/sonsuzluk ve
binary64→Ondalık kanonikleştirmesi RFC-0012'de tanımlanmadan gerçekleme
uyumlu değildir. TamSayı→Ondalık kayıpsız dil içi genişlemesi bu yasaktan
etkilenmez.

## K-092 conformance kanıtı

Regresyonlar; 39 kesir haneli sabit, 30+ haneli katsayı, tam `1/8`, 34
anlamlı haneli `10/3`, 34 basamak ölçekli küçük değer, negatif uzun metin
dönüşümü, değer karşılaştırması, JSON, para biçimi ve i64 daraltma taşmasını
kilitler. `ffi_sinir_testi.rs` ayrıca binary float'ın tür/değer envanterine
sızmamasını ve eski FFI taslak eşlemesinin geri gelmemesini korur. V1-P1-01
ile V1-P0-28 bu sözleşmeyle kapalıdır.
