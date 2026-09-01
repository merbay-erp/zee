# 22 — Deterministik IO profili

Normatif kaynak: RFC-0023, ADR-027. Etkin profil: **`zee-io-1`**.

## Rastgelelik (TANIMLI)

`rastgele sayı` uçları dahil aralıktadır. Ters aralık C006'dır. `zee-io-1`,
`u64` tohumu `0x5EED2EE5` ile XOR'lar; her çekilişte durumu OR 1 yaptıktan
sonra xorshift64 `(12,25,27)` adımlarını ve
`0x2545F4914F6CDD1D` çarpanını uygular. Aralık eşlemesi reddetme eşiğiyle
modulo yanlılığını kaldırır; tam `i64` uzayı geçerlidir.

Conformance vektörü: tohum 7 ile altı `[1,100]` çağrısı sırasıyla
`47, 29, 69, 71, 26, 60` verir. Bu üreteç kriptografik DEĞİLDİR; güvenlik
belirteçlerinde işletim sistemi CSPRNG'si ZORUNLUDUR.

## Takvim ve tekdüze saat (TANIMLI)

`simdi` UTC takvim bileşenlerini verir. `an_ms` süre/deadline için ayrı,
geriye gitmeyen tekdüze saattir. Hermetik profilde takvim varsayılanı
`2026-08-31 14:30`, tekdüze başlangıç 0'dır. `bekle_ms(n)` tekdüze anı
`max(n,0)` kadar saturating ilerletir; takvimi değiştirmez. Enjekte edilen
anlar FIFO tüketilir ve mevcut andan küçükse saat geri gitmez.

Scheduler ortak sanal saati en yakın uyanışa taşır; eşzamanlı beklemeler
toplanmaz (spec/14). Gerçek adaptör UTC sistem zamanı, süreç `Instant`ı ve
host uykusunu kullanır.

## Hermetik adaptör (TANIMLI)

- Girdi ve hazırlanmış rastgele değerler FIFO'dur. Rastgele değer aralığa
  kırpılır; kuyruk boşsa alt uçtur.
- `sor` önce istemi çıktıya ekler. Çıktı ve argüman sırası korunur.
- Sahte dosya baştan yazma/ekleme ayrımını korur ve her yazıya LF ekler.
- HTTP exact URL, dosya exact yol anahtarıyla eşleşir.
- Bildirilmemiş sensör kapalıdır; ışık çıktısı
  `[ışık] <ad> yandı|söndü` biçimindedir.
- Playground görünür tohumu etkin profile verir; ağı/sunucuyu açmaz,
  dosyaları RAM'de ve beklemeyi sanal tutar.

Profilin bu gözlemlerinden birini kıran değişiklik yeni profil kimliği, RFC,
spec ve conformance vektörü ZORUNLU kılar.

## IO iziyle ilişki

RFC-0022/spec-21 izi rastgele ve saat sonuçlarını açık olay olarak taşır.
Replay etkin profilin yeniden hesaplamasını değil kayıtlı sonucu kullanır;
böylece iz, profil sürümünden bağımsız dış etkisiz tekrar üretim kanıtıdır.
