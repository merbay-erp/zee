# 05 — Değerlendirme

Normatif kaynak: RFC-0001 §7 (determinizm), ADR-003 (yürütme modeli),
RFC-0013 (ondalık aritmetik), RFC-0022/ADR-026 (IO izi), RFC-0023/ADR-027
(`zee-io-1`). Tanı kodları: C bölümü.

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

Gerçek bir koşunun bu IO dünyası `dil iz kaydet` ile sürümlü, kanonik bir
protokol olarak kaydedilebilir; `dil iz oynat` aynı çağrı/argüman sırasını dış
etki uygulamadan yeniden yürütür. İlk fark ve tüketilmeyen olay hatadır. Tam
byte biçimi, bütçeler ve gizlilik sınırı [spec/21](21-deterministik-io-izi.md)
içindedir. Rastgele tohum algoritması ile sanal zaman ilerlemesinin taşınabilir
sürüm semantiği [spec/22](22-deterministik-io-profili.md) içindeki
`zee-io-1` profiliyle tanımlıdır.

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

## Değer semantiği, gezme ve yazma (TANIMLI — K-093)

Bütün kullanıcı değerleri iç içe bileşenleriyle bağımsız değer kopyalarıdır;
gizli paylaşılan nesne kimliği yoktur. Liste gezmesi değer-sonuç imlecidir:
öğe kopyalanır, gövde çalışır, döngü adının son değeri aynı sıraya geri yazılır.
Alan yazma ve yeniden bağlama bu nedenle kalıcıdır. Gezilen koleksiyonun
kendisini büyütme/silme/yeniden bağlama ya da aynı kaynağı iç içe gezme
T053'tür. Sözlük gezmesi snapshot anahtarları verir ve geri yazmaz. Tam
normatif algoritma ve alias sınırı spec/17'dedir.

## Çağrı derinliği (TANIMLI — K-040)

Çağrı derinliği 500 ile sınırlıdır; aşımı C019 Türkçe tanısıdır. Sınır her
platformda AYNIDIR ve dilin kendi tanısıyla karşılanır — altındaki
makinenin yığın taşmasıyla değil. Sınır, en dar hedef platforma (tarayıcı
motoru çağrı yığını) paylı seçilmiştir.

Resmî CLI ve test koşucuları yürütmeyi `calistirma_yigin_bayti` (256 MiB
rezervasyon) büyüklüğünde ayrı bir iş parçacığında koşar. K-178 ölçümü (9
Eylül 2026): yorumlayıcı çerçevesi debug profilde düzey başına ~194 KiB,
release'te ~11 KiB; macOS arm64 ve Linux aarch64 birebir aynıdır. Bağlayıcı
kanıt `compiler/tests/ozyineleme_testi.rs` ve
`regression/runtime/derinlik-499-resmi-yiginda.dil` vakasıdır.

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
