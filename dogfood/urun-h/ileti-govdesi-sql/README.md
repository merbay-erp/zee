# K-391 — İleti gövdesinin tek SQL parametresi (ürün H motoru, ADR-168 Dilim 2)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürün ayrı ve özel depodadır; sunucu adı, alan adı, adres ve yerel yol
taşınmaz (ADR-087).

| Exact ürün dosyası (ürün commit'i `57bd016d6df765b9fe4ecef664cf4491785a9956`, 24 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F036` satırı) | `49da5c7bea612dfafac3ed9f8b6709f51105b47e` | `3df29993d6e2152f8384f63d138dd8dc3a4d36dbcd848396ee4b53aab1bb4a37` |
| Motor yardımcıları (`metni parçalara böl`: gövde 20 000 harflik parçalara bölünüp SQL'de `\|\|` ile birleştiriliyor) | `fb2b6ec2384182dbaebc13fc57bf1e1be7da1e82` | `f340f807bef56679a1702bc6cc1e41f5c68bd2d70656ee3884f9cd9d026824bb` |

## Sürtünme (ZM-F036)

spec/25 tek SQL parametresini 64 KiB ile sınırlıyordu; 64 KB üstü ileti
gövdesi tek parametreyle kaydedilemiyor, ürün gövdeyi ve HTML'i parçalara
bölüp SQL'de birleştiriyordu. K-164 kataloğunda karşılığı “kaynak sınırı
ürün ölçeğinde dar”.

## Kesit ([ana.dil](ana.dil))

`veritabanı_parametre_sınırı 1048576 olsun` ile gövde tek parametreyle
gider; bütün parametrelerin toplamı 6 400 KiB'ı aşamaz. Değişiklik öncesi
bildirim P001 verirdi. Tavan ve toplam
`postgresql.rs::parametre_tavani_varsayilan_ve_proje_yukseltmesi_toplamla_sinirli`
ile, alan kuralı `proje_testi.rs` ile sınanır.
