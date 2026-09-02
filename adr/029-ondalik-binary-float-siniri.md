# ADR-029 — Ondalık ile binary float arasında örtük köprü yoktur

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili:** K-125, B-012, RFC-0012, RFC-0013, spec/16

## Bağlam

Zee'nin tek kesirli sayı türü keyfî hassasiyetli onluk `Ondalık`tır. Eski FFI
taslağı, artık dilde bulunmayan `GerçekSayı` adını C `double` ile doğrudan
eşleştiriyordu. Bu ifade hayata geçerse `0,1 + 0,2 = 0,3` gibi temel dil
sözlerinin yabancı çağrı sınırında görünmeden kaybolmasına izin verebilirdi.

Stage 0'da FFI ve binary kayan nokta değer türü yoktur. Buna rağmen gelecek
native backend tasarımının yanlış bir örtük köprü varsaymaması bugün
bağlanmalıdır.

## Karar

1. `Ondalık` için C ABI'de örtük `float`, `double`, binary32 veya binary64
   eşlemesi YASAKTIR.
2. Binary kayan nokta Zee'nin genel amaçlı kullanıcı tür envanterine girmez;
   yalnız gelecekteki işaretli FFI adaptörünün sınır temsilidir.
3. Gelecekte böyle bir köprü eklenirse dış bildirimde ve çağrı yüzeyinde
   görünür biçimde **kayıplı** olarak işaretlenir. Belirli bir değerin tam
   temsil edilebilmesi bu işlem sınıfını kayıpsız yapmaz.
4. Dönüşüm olağan `Ondalık` değeri gibi sessizce dönmez; taşma, sonlu olmayan
   binary değer ve tanımlanacak diğer temsil hatalarını taşıyan `Sonuç`
   sözleşmesine girer.
5. IEEE 754 yuvarlama yönü, signed zero, subnormal, taşma, NaN/sonsuzluk ve
   binary64→Ondalık kanonikleştirmesi ayrı RFC değişikliğiyle tanımlanmadan
   köprü gerçeklenemez.
6. Mevcut FFI taslağının uygulanabilir v1 tür listesi şimdilik yalnız kayıpsız
   temsil edilebilen türlerle sınırlıdır. FFI'nın kendisi Faz 4/5'e kadar dil
   yüzeyi değildir.

## Sonuçlar

- Profesyonel paketler C ekosistemine açılabilir, fakat sayısal hassasiyet
  kaybı sıradan bir işlem gibi gizlenemez.
- Para, ERP ve eğitim kodu FFI çağrısından sonra da bilinçsiz binary floating
  davranışı edinmez.
- İlk FFI gerçeklenmesi double desteği olmadan başlayabilir. Binary köprü daha
  sonra eklense bile kırıcı olmayan, açık bir opt-in olmak zorundadır.
- Dönüşüm yüzeyinin kesin Türkçe sözdizimi bu ADR'nin kararı değildir; golden
  örnek ve usability kapısı olmadan parser'a eklenmez.

## Kanıt

`compiler/tests/ffi_sinir_testi.rs`, checker/runtime değer envanterinde binary
float varyantı bulunmadığını, `tehlikeli/dış işlem` taslak yüzeyinin henüz
derlenmediğini, onluk aritmetiğin exact kaldığını ve RFC-0012'nin eski örtük
eşleme cümlesini yeniden taşımadığını doğrular.
