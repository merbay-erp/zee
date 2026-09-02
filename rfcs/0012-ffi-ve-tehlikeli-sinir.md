# RFC-0012 — FFI ve Tehlikeli Sınır

- **Durum:** taslak (tasarım — gerçekleme Faz 4/5; K-125/ADR-029 sayısal
  sınırı bağladı)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-125, ADR-029, B-012; master plan bölüm 19;
  anti-örnek A10
- **Gerçekleme:** yok

## Özet

Dış dünya köprüsü C ABI'dir; güvensiz sınır görünür Türkçe kelimeyle
işaretlenir: **tehlikeli**. Sıradan kullanıcı FFI'yı hiç görmez — paketler
sarar (A10 ilkesi: İngilizce API yüzeyi Türkçe sarmalayıcıyla sunulur).

## 1. Katman modeli

1. **Kullanıcı katmanı:** yalnız Türkçe stdlib/paket API'leri. FFI yok.
2. **Paket yazarı katmanı:** `tehlikeli` bloklarında dış bildirimler; paket
   bunları güvenli Türkçe işlemlerle sarar ve YALNIZ sarılmış yüzeyi dışa açar.
3. **Derleyici katmanı:** C ABI çağrı üretimi (Faz 4 native backend'le gelir;
   yorumlayıcı döneminde FFI yoktur — bilinçli erteleme).

## 2. Yüzey taslağı

```
tehlikeli
    dış işlem c_toplam
        adı "add" olsun
        kütüphanesi "libhesap" olsun
        girdileri TamSayı ve TamSayı olsun
        çıktısı TamSayı olsun

işlem güvenli toplam
    a al
    b al
    sonucu a ve b ile c_toplam olsun    # tehlikeli çağrı yalnız burada
    sonucu döndür
```

Kurallar (öneri):

- `tehlikeli` blok dışında dış bildirim ve çağrı derleme hatasıdır; blok,
  koddaki görünür "burada dikkat" işaretidir (bölüm 18: unsafe açık sınır).
- Dış adlar (`"add"`) metin sabitidir — RFC-0002 tanımlayıcı kuralları dış
  ada uygulanMAZ (İngilizce ad kaynakta yalnız tırnak içinde yaşar).
- Tür köprüsü taslağının kayıpsız çekirdeği: TamSayı↔C `int64_t` ve
  Metin↔uzunluğu açık UTF-8 salt-okunur görünüm. Sahiplik GEÇMEZ: dilin
  değerleri C'ye ödünç verilir; C'den dönen bellek sahipliği paket yazarının
  sorumluluğunda ve `tehlikeli` içinde kalır.
- `Ondalık` için C `float`, `double`, binary32 veya binary64'e örtük eşleme
  YASAKTIR. Zee'nin değer/tür envanterinde ikinci bir `GerçekSayı` ya da
  binary kayan nokta türü yoktur.
- Çökme sınırı: dış çağrıdaki çökme dilin güvence alanı dışındadır; tanı
  üretilemez — bu yüzden 1. katman kullanıcısına asla doğrudan FFI verilmez.

## 2.1 Ondalık ve binary kayan nokta

Binary kayan nokta desteği ilk FFI gerçeklenmesinin zorunlu parçası değildir.
İleride eklenirse aşağıdaki kararlar bağlayıcıdır (ADR-029):

1. Dış bildirim ve çağrı, dönüşümü görünür **kayıplı** sınır olarak işaretler;
   sözdiziminin kesin biçimi ayrıca golden/usability kararı ister.
2. Dönüşüm sıradan bir örtük genişleme değildir ve `Sonuç` sözleşmesi taşır.
   Taşma ile sonlu olmayan binary değerler sessizce Ondalık/sonsuzluk olamaz.
3. IEEE 754 yuvarlama yönü, signed zero, subnormal, NaN/sonsuzluk ve
   binary64→Ondalık kanonik biçimi bu RFC'de normatifleştirilmeden gerçekleme
   kabul edilmez.
4. Tek bir değerin binary64'te tam temsil edilebilmesi işlemin sınıfını
   kayıpsız yapmaz. Açık `kayıplı` işareti API sözleşmesinde kalır.

Bu karar TamSayı→Ondalık kayıpsız dil içi genişlemesini etkilemez. `Ondalık`,
RFC-0013/spec-16'daki keyfî hassasiyetli onluk anlamını FFI sınırına kadar
korur.

## 3. Güvenlik bağları (bölüm 18)

- `tehlikeli` içeren paket, registry'de görünür şekilde işaretlenir; kurulum
  onayı ister (çocuk/öğretmen modunda varsayılan: reddet).
- Native ikili taşıyan paketlerde OS/mimari metadata + checksum zorunlu.

## 4. Açık sorular

1. `tehlikeli` mi `güvensiz` mi — kelime usability'ye gider ("tehlikeli"
   çocuk için daha caydırıcı, tercih bu yönde).
2. Callback'ler (C→dil çağrısı) v1'de yok; olacaksa yeniden giriş kuralları.
3. WASM hedefinde FFI'nın karşılığı (host fonksiyon ithali) — playground
   sandbox'ıyla ilişkisi (bölüm 15: client-side WASM).
4. Rust/C++ üreteç-adaptörleri (bölüm 19) — ayrı araç RFC'si.
5. `kayıplı` dönüşümün tam Türkçe deklarasyon/çağrı yüzeyi ve IEEE 754
   ayrıntıları — ADR-029 sınırı içinde ayrı RFC revizyonu.

## Dört soru süzgeci

Doğal ✓ ("tehlikeli" kendini açıklar) · Deterministik ✓ (sınır sözdizimsel,
kaçış yok) · Öğrenilebilir ✓ (kullanıcı katmanı FFI'yı hiç görmez) ·
Savunulabilir ✓ (C ABI evrensel köprü; işaretli paket + onay zinciri).

## Korpus etkisi

FFI gerçeklenmediği için çalıştırılabilir dış çağrı korpusu henüz yoktur.
`ffi_sinir_testi.rs`; çekirdek `Tur`/`Deger` envanterine binary float
sızmadığını, eski taslak yüzeyinin derlenmediğini, Ondalık aritmetiğinin exact
kaldığını ve bu RFC'deki açık `kayıplı`/`Sonuç` kapısının bayatlamadığını
korur. Gerçek FFI ilk kez açılırken olumlu/olumsuz platform conformance korpusu
aynı değişiklikte zorunludur.
