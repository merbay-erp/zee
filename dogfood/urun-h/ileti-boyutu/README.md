# K-391 — İleti yazma formu boyutu (ürün H web arayüzü, ADR-168 Dilim 1)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürünün web arayüzü (ZeeMail) ayrı ve özel depodadır; sunucu adı, alan adı,
adres ve yerel yol taşınmaz (ADR-087).

| Exact ürün dosyası (ürün commit'i `57bd016d6df765b9fe4ecef664cf4491785a9956`, 24 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F036`, `ZM-F037` satırları) | `49da5c7bea612dfafac3ed9f8b6709f51105b47e` | `3df29993d6e2152f8384f63d138dd8dc3a4d36dbcd848396ee4b53aab1bb4a37` |
| İleti rotaları (yazma formu urlencoded, `maxlength=30000`; `POST /posta/gonder` 30 000 harf denetimi) | `0dc025eab3bdfe7fd8ea289eb5e724d4c1246a6e` | `53855291b64d9b300c14ffafae985c316cc23f928a659bfb45149809f6eed942` |

## Sürtünme (ZM-F037)

Web gövdesi 64 KiB'tır (spec/11 §5). Yazma formu
`application/x-www-form-urlencoded` gönderir; tarayıcı her Türkçe harfi
(`ı` → `%C4%B1`) altı bayta çevirir. 25 000 harflik Türkçe-yoğun ileti rota
çalışmadan ham 413 alıyor, kullanıcı taslağını kaybedebiliyordu; ürün yazma
alanını 30 000 harfle sınırladı. K-378 multipart yolu dosyasız formu ve 8 KiB
üstü metin alanını reddettiği için çözüm değildi. K-164 kataloğunda karşılığı
“web adaptörü yüzeyi eksik” (ZM-F022, ZM-F033 ile aynı sınıf).

## Kesit ([ana.dil](ana.dil))

Yazma formu `enctype=multipart/form-data` ile gönderilir; metin alanları
sıradan form alanıdır ve Türkçe metin şişmez. `gövde_aşımı_rotası`
bildirimi gövde sınırı aşıldığında salt okuma `GET /posta/cok-buyuk`
sayfasını 413 ile döndürür. Değişiklik öncesi bildirim P001 verirdi.
Gerçek HTTP davranışı `govde_siniri_testi.rs` ile sınanır. ZM-F036 (tek SQL
parametresi 64 KiB) ADR-168 Dilim 2'dedir (açık).
