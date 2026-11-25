# K-397 — Giden TCP ve istemci STARTTLS (ürün H giden teslim)

Bu anonim kesit, etkin `urun-h` ürününün giden teslim hizmetine
(`zeemail/disteslim`; bkz. [KOKEN](../KOKEN.md)) bağlıdır. Ürün ve işletim
sistemi depoları özeldir; sunucu adı, alan adı, adres ve yerel yol
taşınmaz (ADR-087).

ZM-F042 reproducer'ı kesin yüzeyden önce taslak söz dizimiyle yazıldı ve
provenance olarak durur; ZM-F041 reproducer'ı kesin söz dizimiyle birebir
aynıdır. Sürtünmeler ZM-F041 (giden TCP + STARTTLS, bu kesit) ve ZM-F042 (DKIM
imzası, K-397 Dilim 2).

| Exact ürün dosyası (ürün commit'i `850d86b494839a817bf2137fd36cebd9b3c880db`, 24 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Ürün reproducer'ı ZM-F041 (depoda birebir: [korpus/zm-f041-yeniden-uretim.dil](../korpus/zm-f041-yeniden-uretim.dil)) | `ddbb672d8d035e22c123609794d8e53c0fa07e80` | `562b8384379b45335ad8dab9ad5126d183437dedad54bb916e5d4619749006d8` |
| Ürün reproducer'ı ZM-F042 (DKIM imzası, Dilim 2) | `e732f3f639c0026c8b0bbf0b05b3508ac84358ea` | `2ab1b5728933575d6693f709e470392accf70754c2c3de23cb80dde3f777f23b` |
| Ürün giden SMTP durum makinesi, MX sırası, yeniden deneme (`smtp_istemci` birimi) | `df9fb029ad9e677dfeb9b6bc8bf3c458b2ae023a` | `7b8a3d8fab468fa5ad5257d4df2b5ab5bdbbbaa361b7faed22ae8c986504bd6d` |
| Ürün MTA-STS/DANE/TLS-RPT kararı; `doğrulandı` ve SPKI alanlarını tüketir (`giden_tls` birimi) | `42cafd5d7cb5bc8c2165f6444a937d17b21080c7` | `36086c685505e077c1b1aed8a8212f27bcde520f18b0fe9e8f3714bbec9fb2ea` |
| Ürün teslim planı ve sonucu (`dis_teslim` birimi) | `fceb1e1e68097a86a7e209d720d453a516e69536` | `421ab6e4b89f96ab0f4253a3c8ba3457eba3b7cbf3d52db8fa44fce98a57107f` |
| Ürün kuyruk sözleşmesi göçü (ürün dalı commit'i `87c223fa70f4d2a2f9e649af1d3ce80ca9d633ab`) | `b2b61bae722ec92517b702be2bb0e593788e1a6c` | `4104eca398b79334ea397025c4d82140bf64eda6763c2a249110f6c60c574341` |
| Ürün sürtünme kaydı | `b8b3237eb60f2d858cc57265b25cd852269078d1` | `e364868612ebda4fe887a8423a989069ff81a637408943475db8b262eb4173b2` |

## Sürtünme

Ürünün teslim motoru Zee'de yazılı: SMTP istemci durum makinesi, MX sırası,
yeniden deneme, MTA-STS/DANE/TLS-RPT kararı, DSN ve ileti kurma. Motor
başka kurumlara posta teslim edemiyordu, çünkü dil giden bağlantı olarak
yalnız iki yol sunuyordu: tam origin listeli HTTP(S) ve bildirilmiş
aktarıcıya K-210 SMTP gönderimi. Önceden bilinmeyen bir MX IP'sine 25
kapısından bağlanmak, o bağlantıda STARTTLS yapmak ve sertifika
doğrulamasını, DANE için de SPKI özetini görmek mümkün değildi. MTA-STS
ilkesinin adresi alıcı alanına bağlı olduğundan `ağ_hedefleri` ile
yazılamıyordu. K-164 kataloğunda "işletim yüzeyi eksik" sınıfıdır
(K-343/K-376/K-381 ile aynı sınıf).

## Minimal değişiklik

- [proje.dil](proje.dil) `ağ` yetkinliğini ve iki dar jokeri bildirir:
  `tcp://*:25` (yalnız 25, yalnız public IP) ve `https://mta-sts.*`
  (443, sabit yol).
- [ana.dil](ana.dil) ürün teslim motorunun ağ tarafının küçültülmüş
  hâlidir: bağlanma, karşılama, EHLO, STARTTLS ilanı, istemci TLS ve enforce
  kipinde `doğrulandı` denetimi, zarf, dosyadan gövde aktarımı, QUIT. Sabit
  MTA-STS URL'si derlemede joker kuralıyla denetlenir.
- Bağlanma ve TLS kalıpları bu değişiklikten önce S015 verirdi; bağlantı
  kalıpları ise `ağ-dinleyici` isterdi.
- Politika (hangi MX, enforce mi fırsatçı mı, DANE eşlemesi) ürünün Zee
  kodudur. Dil yalnız hedef ve IP sınırını (tek çözümleme, denetlenen IP'ye
  sabitleme) ve kriptografiyi taşır (ADR-173).
