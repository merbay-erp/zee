# 05 — Değerlendirme

Normatif kaynak: RFC-0001 §7 (determinizm), ADR-003 (yürütme modeli),
RFC-0013 (ondalık aritmetik). Tanı kodları: C bölümü.

## Yürütme modeli (TANIMLI)

Program, cümlelerin yazılı sırasıyla yürütülür. Bir cümlenin içindeki
ifadeler tek biçimde ayrışır ve alt ifadeleri bir kez değerlendirilir;
gözlemlenebilir yan etkiler (yazma, sorma, dosya, ağ) kaynak sırasını izler.

## Determinizm sözü (TANIMLI — RFC-0001 §7)

Aynı program + aynı girdiler + aynı IO dünyası (tohum, saat, dosyalar,
ağ cevapları) = **her platformda aynı çıktı**. Bunu mümkün kılan kural:
zaman, rastgelelik, dosya, ağ, sensör ve an ölçümü dahil BÜTÜN dış dünya
IO soyutlamasının arkasındadır; dil çekirdeğinde gizli kaynak yoktur.
Playground'da tohum görünürdür; testlerde IO dünyası tamamen sahtedir.

## Sayısal anlam (TANIMLI)

- TamSayı i64'tür. Taşma sessizce **sarmalanmaz**: taşan işlem C002 verir.
- Sıfıra bölme C003. TamSayı bölmesi tam bölümdür (kalan atılır).
- Kalan (`bölümünden kalanı`, K-046) yalnız TamSayılar arasındadır ve okul
  kuralına uyar: sonuç DAİMA 0 ≤ kalan < |bölen| (Öklit kalanı). Sıfıra
  kalan C003.
- Ondalık, keyfî uzunlukta imzalı katsayı ve onluk ölçek taşır: gövde ×
  10⁻ᵏ. Toplama, çıkarma, çarpma ve sonlu onluk bölme TAMDIR; ikilik kayan
  nokta HİÇBİR aşamada kullanılmaz. `0,1 + 0,2 = 0,3` kimliktir.
- Sonsuz açılımlı Ondalık bölme 34 anlamlı haneye, yarımlar sıfırdan uzağa
  yuvarlanır. Bu tek yuvarlama noktası ve örnekleri spec/16'da tanımlıdır.
- Yuvarlama: yarımlar sıfırdan uzağa (`2,5 → 3`, `-2,5 → -3`).

## Gezme ve yazma (TANIMLI — K-074)

`her X için` gezmesi öğenin KOPYASINI bağlar; ancak kaynak bir ad
olduğundan (dilbilgisi gereği hep öyledir) her turun sonunda döngü
değişkeninin son değeri listedeki öğeye GERİ YAZILIR — gövdedeki alan
değişikliği kalıcıdır. Sözlük gezmesi anahtarları verir; yansıma yoktur.

## Çağrı derinliği (TANIMLI — K-040)

Çağrı derinliği 500 ile sınırlıdır; aşımı C019 Türkçe tanısıdır. Sınır her
platformda AYNIDIR ve dilin kendi tanısıyla karşılanır — altındaki
makinenin yığın taşmasıyla değil. Sınır, en dar hedef platforma (tarayıcı
motoru çağrı yığını) paylı seçilmiştir.

## Program sonlanması (TANIMLI — K-069)

`programı bitir` = çıkış kodu 0; `programı <n> ile bitir` süreç çıkış
kodunu belirler. Kod TamSayı (T034) ve 0–255 aralığında (C020) olmak
**ZORUNDA**dır; CLI bu kodu işletim sistemine aynen iletir.

## Girdi bitişi (TANIMLI)

Etkileşimsiz koşuda `diye sor` için girdi kalmadıysa C005. `programı bitir`
programı o noktada, o ana dek üretilmiş çıktıyla sonlandırır.

## Çalışma zamanı tanıları

Çalışma hatası da tanı sözleşmesine uyar (kod + mesaj + konum + öneri) ve
programı durdurur. "Beklenen" hatalar (dosya yok, çevrilemeyen metin...)
için Sonuç üreten `... dene` biçimleri **TANIMLI** yoldur — bkz. 06.
