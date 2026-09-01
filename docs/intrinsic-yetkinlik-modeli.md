# Intrinsic ve yetkinlik modeli

Bu belge ADR-011'in uygulama rehberidir. Kullanıcıların yazdığı Türkçe
cümleleri değiştirmez; derleyiciye yeni bir profesyonel adaptör eklerken core
AST'nin alan ayrıntılarıyla büyümemesini sağlar.

## Boru hattındaki yeri

```text
Türkçe kaynak
    ↓ parser lowering
Intrinsic { kararlı kimlik, sıralı argümanlar }
    ↓ merkezi kayıt
tür imzası + gereken yetkinlik + statik etki
    ↓
checker / etki çözümleyici / runtime adaptörü
```

Tek gerçek kayıt `compiler/src/intrinsic.rs` dosyasındadır. AST yalnız kimlik
ve argümanları bilir; HTTP, sensör, CSRF ya da parola adına özel bir varyant
taşımaz.

## Bugünkü kayıtlar

| Kimlik | Argümanlar → dönüş | Yetkinlik | Etki |
|---|---|---|---|
| `ag.http_getir` | `Metin → AğYanıtı` | `Ag` | `DisOkuma` |
| `donanim.sensor_acik_mi` | `Metin → Mantıksal` | `Donanim` | `DisOkuma` |
| `web.csrf_belirteci` | `() → Metin` | `WebOturumu` | `WebAdaptoru` |
| `guvenlik.parola_dogrula` | `(Metin, Metin) → Mantıksal` | `Kriptografi` | `Saf` |

Yetkinlik bir izin değildir. Çalıştırma profilinin hangi yetkinliklere izin
verdiği B-023'te kararlaştırılacaktır; bu katman yalnız ihtiyacı dürüstçe
bildirir. Örneğin çocuk modu ağ çağrısını bugün `GuvenliIo` sınırında reddeder.

## Yeni intrinsic ekleme protokolü

1. Kullanıcı yüzeyi yeniyse önce ifade katmanı/çakışma analizi ve ilgili
   RFC/spec değişikliği hazırlanır. Yalnız iç lowering değişiyorsa ADR etkisi
   değerlendirilir.
2. Ad alanlı, anlamı kararlı kimlik; tür imzası, yetkinlik ve etki merkezi
   kayda birlikte eklenir.
3. Parser yalnız kaynak cümlesini genel AST düğümüne indirir. Genel `Değil`,
   aritmetik veya erişim düğümüyle anlatılabilen davranış için ikinci bir
   intrinsic üretilmez.
4. Checker ve etki çözümleyici kayıt metadatasını tüketir. Aynı imza/etki
   başka bir eşleşme dalında ikinci kez tanımlanmaz.
5. Runtime kararlı kimliği uygun IO/adaptör sınırına dağıtır; bilinmeyen kimlik
   fail-closed kalır.
6. En az bir lowering, bir tür olumsuzu ve davranış regresyonu eklenir. Kimlik
   tekilliği kayıt testinde korunur.
7. Backlog, karar günlüğü, sürüm notu, v1 kapısı ve ilgili indeksler aynı
   committe güncellenir.

## Sınırlar

- Bu model genel kullanıcı işlemlerinin yerine geçmez; intrinsic derleyicinin
  güvenilir iç ABI'sidir.
- Paketlerin keyfî intrinsic kimliği üretmesine izin verilmez.
- Kimliğin kayıtlı olması adaptörün her ortamda açık olduğu anlamına gelmez.
- Runtime handler'larının modüllere ayrılması B-005, checker fazlarının
  ayrılması B-006, proje/paket yetkinlik izinleri B-023 kapsamındadır.
