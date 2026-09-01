# ADR-011 — Çekirdek AST ile alan adaptörleri arasında intrinsic/yetkinlik sınırı

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-098, B-004, V1-P0-09

## Bağlam

Bootstrap AST'si HTTP istemcisi, sensör okuması, CSRF belirteci ve parola
doğrulaması için ayrı ifade varyantları taşıyordu. Bunlar çalışan ve testli
kullanıcı yüzeyleridir; fakat ağ, donanım, web oturumu ve kriptografi gibi
alan ayrıntılarının dilin çekirdek ağaç şemasına yerleşmesi iki sorun doğurur:

1. her yeni profesyonel adaptör lexer/parser dışında AST, tür denetleyici,
   etki çözümleyici ve runtime'ın kapalı eşleşmelerini de büyütür;
2. kaynağın ne söylediği ile çalıştırma ortamının hangi yetkinliğe ihtiyaç
   duyduğu aynı kavrama dönüşür; güvenlik politikası merkezi kurulamaz.

Kaynak sözdizimini değiştirmek bu mimari borcun çözümü değildir. Çocuk için
okunur mevcut cümlelerle profesyonel runtime genişlemesi birbirinden bağımsız
kalmalıdır.

## Karar

Çekirdek AST, alan başına varyant yerine yalnız şu genel düğümü taşır:

```text
Intrinsic { kimlik, argumanlar }
```

Parser mevcut Türkçe yüzeyi kararlı, ad alanlı bir kimliğe ve sıralı argüman
listesine indirir. Merkezi intrinsic kaydı her kimliğin:

- argüman ve dönüş türünü,
- gereken yetkinliği,
- statik etki sınıfını,
- tür hatası açıklamasını

tek yerde tutar. Tür denetleyici bu imzayı, etki çözümleyici bu etkiyi,
yorumlayıcı da aynı kimliği kullanır. Bilinmeyen kimlik sessizce çalışmaz.

İlk kararlı kimlikler:

| Kimlik | Kaynak yüzeyi | Yetkinlik | Etki |
|---|---|---|---|
| `ag.http_getir` | `"..." adresinden gelen yanıt` | ağ | dış okuma |
| `donanim.sensor_acik_mi` | `kapı açıksa` | donanım | dış okuma |
| `web.csrf_belirteci` | `csrf belirteci` | web oturumu | web adaptörü |
| `guvenlik.parola_dogrula` | `parola özet ile doğrulanıyorsa` | kriptografi | saf |

`kapı kapalıysa`, ikinci bir alan intrinsic'i değildir; genel `Değil`
ifadesinin sensör intrinsic'ini sarmalamasıdır. Böylece olumsuzluk semantiği
tek yerde kalır.

## Değişmezler

1. Kullanıcı kaynakları ve tanıları geriye uyumlu kalır; lowering iç mimaridir.
2. Kimlikler ad alanlı ve tekildir; yeniden adlandırma derleyici içi ABI
   değişikliği sayılır.
3. AST yetkinlik politikası taşımaz. Kayıt gereksinimi bildirir; proje/çalışma
   profili izni B-023'ün ayrı güvenlik kararıdır.
4. Yeni intrinsic; kayıt + parser lowering + tür + etki + runtime + olumlu ve
   olumsuz conformance kanıtı olmadan eklenemez.
5. Yeni bir kullanıcı cümlesi yine ilgili RFC/spec değişikliğini ister. Bu ADR
   dil yüzeyi ekleme yetkisi vermez.

## Sonuçlar

- Core AST dört alan varyantından kurtuldu; yeni adaptörler tek genel düğümle
  taşınabilir.
- Tür/yetkinlik/etki metadatası `compiler/src/intrinsic.rs` içinde tek kaynak
  oldu. B-006 katmanlaştırması ve B-023 izin modeli artık bu kayıt üstünde
  ilerleyebilir.
- Runtime merkezi kimlik dağıtımı yapar. Handler'ların fiziksel modül
  sınırları B-005/K-099 ve ADR-012 ile davranış değişmeden ayrılmıştır.
- Kaynak davranışı değişmediği için yeni normatif dil spec'i açılmadı; mevcut
  RFC/spec cümle anlamları geçerlidir. Bu ADR yalnız derleyici mimarisini bağlar.

Yedi bağımsız lowering/imza testi, mevcut HTTP/sensör/web/parola regresyonları
ve tam test paketi kararı korur.
