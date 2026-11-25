# K-389 — Gelen iletiden ek çıkarma (ürün H posta motoru, RFC-0043 Dilim 1)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürün deposu özeldir; sunucu adı, alan adı, adres ve yerel yol taşınmaz
(ADR-087).

| Exact ürün dosyası (ürün commit'i `4f8c67cf6de936d1f17a689598f65cdca2bb4918`, 24 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F023`, `ZM-F024`, `ZM-F031`, `ZM-F034` satırları) | `49da5c7bea612dfafac3ed9f8b6709f51105b47e` | `3df29993d6e2152f8384f63d138dd8dc3a4d36dbcd848396ee4b53aab1bb4a37` |
| `korpus/zm-r004-uzunluk-oktet-saymaz.dil` (ürün commit'i `7f7b1fe48035ccff1fe4c78810b19e00234ca8ac`) | `60e565e786cb2dae8206d38d889b4ead973169e5` | `77adcdc0e727bf1f56dace255f337b70182f16ff450a4c795ee2571950acaa4c` |
| `korpus/zm-r005-bellekte-sha256-yok.dil` (ürün commit'i `7f7b1fe48035ccff1fe4c78810b19e00234ca8ac`) | `f994eaaba2a1ab343f477a8ce2ee66ba81f3cf14` | `cd268588fe23277b5a8adc541ded1a5776e4d75dfee178ddb2734b3bd400a1db` |

## Sürtünme (ZM-F034, ZM-F024)

K-384 (A2c) DATA gövdesini ve IMAP literal'ini `yol`/`sha256`/`bayt`
tutamaklı blok dosyasına alıyordu; ama ürün bu dosyayı parça parça `Metin`
olarak geri okuyamıyor, MIME parçasının base64/quoted-printable gövdesini
bayt olarak ek dosyasına yazamıyor, DKIM gövde özeti için bellekteki metnin
SHA-256'sını alamıyordu. Etki: gelen ekler webmail'e çıkarılamıyor, ekli
gönderim `554` ile düşüyor, büyük `APPEND` 64 KB'ta takılıyor.
`dosyasını okumayı dene` 8bit gövdede başarısızdı. K-164 kataloğunda sınıf
"yerleşik yüzey eksik"; aynı yapı taşı (base64) derleyicide üç kez iç
yardımcı olarak yazılmış, RFC-0044 DKIM yolu da istiyor.

## Kesit ([ana.dil](ana.dil))

Bağlantı bloğu DATA'yı A2c ile dosyaya alır; `başlık bölümünü oku` ilk
64 KiB'ı katı UTF-8 okur, `eki akıt` ek gövdesini 78 baytlık satırların
1 MiB altındaki en büyük katıyla okuyup `base64 çözerek … eklemeyi dene`
ile ek dosyasına akıtır, `metin gövdesini çöz` QP gövdeyi yazar, `sekiz bit
parçayı kodla` 8bit parçayı base64 Metin olarak alır, `dkim gövde özeti`
bellekte SHA-256 verir. Değişiklik öncesi bu kaynak `S015` verir (kalıp
yok); şimdi ürün politikasıyla (`dosya-yazma` eklendi) `denetle` temizdir.
Bayt-kesinlik, tavan, no-clobber, yol geçişi, içeriksiz iz ve RFC-0041
dosya yanıtı bağlantısı `compiler/tests/kodlanmis_dosya_testi.rs` ile
gerçek `dil` sürecinde sınanır.

Açık: UTF-8 bayt sayısı (`uzunluğu` harf sayar; ZM-R004 bugün `1`),
metin↔base64 ve kod noktası RFC-0043 Dilim 2'dir. Ürünün ZM-R005 testindeki
yer tutucu özet gerçek SHA-256 değildir; depo içi aynası gerçek değeri
bekler.

K-395 (ZM-F039, ürün commit'i `8ee251495170139b1416919e75d76954065a3c4c`;
sürtünme kaydı blob `37c3374a7f74911218364893aa7d548729481e97`, SHA-256
`ef907c3a4d01a645189015089bc618c1b363d555c58e5f831ee816484556805b`): 25 MB
iletideki ~18 MB'lık ek bu kesitteki `eki akıt` döngüsüyle çıkarılır; hedef
tavanı 16 MiB'tan 32 MiB'a çıktı. Tam sınır ve +1 bayt `kodlanmis_dosya_testi`
`eklemeyi_dene_hedef_tavani_32_mib_tam_sinirda` ile.

