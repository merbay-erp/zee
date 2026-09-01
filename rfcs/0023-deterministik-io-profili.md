# RFC-0023 — Sürümlü deterministik IO profili

- **Durum:** geçici kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıtlar:** K-116, B-028, V1-P0-26
- **Gerçekleme:** `yorumlayici/io_profili.rs`, `ToplayanIo`, `PlaygroundIo`
- **Profil kimliği:** `zee-io-1`

## Özet

Zee'nin aynı program+aynı IO dünyası sözünü oluşturan rastgele tohum,
takvim/tekdüze saat ve hermetik adaptör davranışı artık isimli bir uyumluluk
profilidir. `zee-io-1`; tohumdan rastgele diziye geçişi, uçları dahil aralık
eşlemesini, sanal zaman ilerlemesini ve test/playground IO'sunun gözlenebilir
varsayılanlarını sabitler. Bu davranışlardan birini kıran değişiklik sessizce
yapılamaz; yeni profil kimliği ister.

## 1. Tohumlu rastgelelik (bağlayıcı)

`SurumluRastgele::yeni(tohum)` sıfır dahil bütün `u64` tohumları kabul eder.
`zee-io-1` durumu ve her çekilişi şu sırayla tanımlar:

1. başlangıç durumu `tohum XOR 0x5EED2EE5` olur;
2. çekiliş başında durum `durum OR 1` ile sıfır kilidinden çıkarılır;
3. sırasıyla `x ^= x >> 12`, `x ^= x << 25`, `x ^= x >> 27` uygulanır;
4. durum bu `x` olur, çıktı `x × 0x2545F4914F6CDD1D` işleminin `u64`
   taşmalı sonucudur;
5. uçları dahil `[alt, üst]` aralığına eşleme modulo yanlılığını reddetme
   eşiğiyle kaldırır. Tam `i64::MIN..=i64::MAX` uzayı taşmadan desteklenir.

Tohum ve çağrı/argüman sırası aynıysa dizi her hedefte aynıdır. `tohum=7` ve
ardışık altı `[1,100]` çağrısı `47, 29, 69, 71, 26, 60` üretir; bu conformance
vektörüdür. Ters aralık dil hattında C006'dır.

Bu üreteç kriptografik değildir. Oturum, CSRF, paket anahtarı veya başka
güvenlik belirteçleri işletim sistemi CSPRNG katmanını kullanmaya devam eder.

## 2. Saatler ve bekleme (bağlayıcı)

Takvim zamanı ile tekdüze an ayrı kaynaklardır:

- `simdi`, yıl/ay/gün/saat/dakika UTC takvim görünümüdür.
- `an_ms`, yalnız süre/deadline hesabında kullanılan geriye gitmeyen tekdüze
  milisaniyedir. Takvim saatine dönüştürülemez.
- Hermetik ortamda tekdüze an 0'dan başlar. `bekle_ms(n)`, `max(n,0)` kadar
  saturating ilerletir; takvim zamanını değiştirmez.
- Enjekte edilen an kuyruğu FIFO tüketilir fakat önceki andan küçük değer
  saati geri götürmez.
- Yapılandırılmış scheduler, hazır görev kalmadığında ortak anı en yakın
  uyanışa taşır. Eşzamanlı 2 ve 1 saniyelik beklemeler 3 değil 2 saniye eder.

Gerçek adaptör takvim için sistem UTC'sini, tekdüze an için süreç başlangıç
`Instant`ını ve bekleme için host uykusunu kullanır. Bunların gözlenen sonuçları
RFC-0022 izinde kaydedilerek tekrar üretilebilir.

## 3. Hermetik IO davranışı (bağlayıcı)

`ToplayanIo` ve playground profili şu gözlenebilir sözleri taşır:

1. Varsayılan takvim `2026-08-31 14:30 UTC`, tekdüze an `0`dır.
2. Girdi ve rastgele değer kuyrukları FIFO'dur. Rastgele kuyruk değeri aralığa
   kırpılır; kuyruk boşsa alt uç kullanılır.
3. `sor`, istemi çıktıya önce ekler; sonra sıradaki girdiyi veya `yok`u verir.
4. Sahte dosya yazımı baştan yazmada eski içeriği temizler, eklemede korur ve
   her yazıda bir LF ekler. Okuma exact yol anahtarıyla yapılır.
5. Argümanlar eklenme sırasını; HTTP yanıtı exact URL eşlemesini korur.
6. Bildirilmemiş sensör kapalıdır. Işık çıktısı `[ışık] <ad> yandı|söndü`
   biçiminde normal çıktı sırasına girer.
7. Playground ağı/sunucuyu açmaz, dosyayı RAM'de tutar, görünür tohumu
   `zee-io-1`e verir ve sanal beklemeyle tarayıcı iş parçacığını uyutmaz.

Web oturum/CSRF, transaction ve scheduler ayrıntıları kendi spec'lerinde
kalır; bu profil onlarla çelişmeyen zaman ve adaptör temelini sürümler.

## 4. Sürümleme ve kanıt (bağlayıcı)

Profil kimliği `DETERMINISTIK_IO_PROFILI` public sabitidir. Tohum karışımı,
PRNG adımı, aralık eşlemesi, varsayılan takvim, sanal saat ilerlemesi veya
yukarıdaki hermetik gözlemlerden biri değişirse snapshot testini güncellemek
tek başına yeterli değildir: yeni profil adı ve geçiş RFC'si gerekir.

RFC-0022 izleri profil algoritmasına güvenmez; rastgele/saat sonuçlarını olay
olarak taşır. Bu nedenle şema-1 iz replay'i `zee-io-1` sonrasında da kaydedilen
sonuçları birebir kullanır.

## Dört soru süzgeci

Doğal ✓ (tohum görünür, saat ayrımı öğretilebilir) · Deterministik ✓ (isimli
vektör ve zaman algoritması) · Öğrenilebilir ✓ (FIFO/sanal saat) ·
Savunulabilir ✓ (tam i64, yansız aralık, CSPRNG ayrımı, sürüm kapısı).
