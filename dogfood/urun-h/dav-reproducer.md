# K-364 — RFC-0040, ZM-F027 ve ZM-F050 reproducer'ları

Bu vakalar etkin `urun-h` ürününe (bkz. [KOKEN](KOKEN.md)) bağlıdır. Ürünün
web arayüzü (ZeeMail) ayrı ve özel depodadır; sunucu adı, alan adı, adres ve
yerel yol taşınmaz (ADR-087). Ürün sahibinin 26 Kasım 2026 kararı “önce
reproducer” idi: alt RFC kararı bu vakalardan sonra verilir (büyük olasılıkla
V1 sonrası). Bu bir dil değişikliği değildir.

Ürün commit'i `a39356ded33f0058aacaa0f3e93bd685c745a6ce` (26 Kasım 2026);
vakaların eklendiği commit `ef71d39faf79a7ff15ecb4af2fae3f351eb2a270`. İki
commit arasında dosyalar değişmedi. Depodaki dosyalar ürün dosyalarıyla
bayt bayt aynıdır.

| Depo dosyası | Ürün dosyası (`docs/dogfood/korpus-aday/`) | Git blob | SHA-256 | Tanı |
|---|---|---|---|---|
| `korpus/rfc0040-a-propfind.dil` | `rfc0040-a-propfind/ana.dil` | `66904b106d50a77172de1eaa062f8cddf7d93c87` | `157213d8c9e37a9b3bf8f99279261a1d2e0e79aaf38a52332ccfdfad336001ec` | S036 |
| `korpus/rfc0040-a-report.dil` | `rfc0040-a-report/ana.dil` | `f13647220ff47d0cd22c6e923bdaf61afaf12239` | `8f5fde0761a35517b8c35ba775910c94e78b8a28233841cced0340bf2e1157e5` | S036 |
| `korpus/rfc0040-b-depth-basligi.dil` | `rfc0040-b-depth-basligi/ana.dil` | `1ea1bbb001dea8daf97792b657f84349f75ae804` | `fba8b9645dac013ee2efb4d53c548ec521e148c50332e71bfb609fa83ddd33c4` | S015 |
| `korpus/rfc0040-c-207-xml.dil` | `rfc0040-c-207-xml/ana.dil` | `58f3312e5129a9997cbd6067e566c2e17522d432` | `7f7ea240e4415c643fb9393fb2fa9c8c49389c7fa19529d312540a56a797ff55` | S037 |
| `korpus/rfc0040-c-etag-basligi.dil` | `rfc0040-c-etag-basligi/ana.dil` | `c69765ee8e1fd524bfc538c92363501fda2e84a0` | `0b779c80756e615ecb769731fc3bbcb575cc47cb846dd89b5c1abeca22d77335` | S015 |
| `korpus/rfc0040-e-basic-kimlik.dil` | `rfc0040-e-basic-kimlik/ana.dil` | `c3a0e7294a144f45ae5b9785fff2065ef49bf839` | `f273b5a09f98e5fa2f2f1b41eaa6eff10d17ccd65206c5994e708b734e0236ae` | S015 |
| `korpus/rfc0040-f-kimlikli-makine.dil` | `rfc0040-f-kimlikli-makine/ana.dil` | `9fc7adbfbc9ae22f0bc118ee15fb5645aa57c19d` | `f5a8461eae44322c7b8f544d84db307ed49f06d663268da293ab23e0f5a2e137` | T050 |
| `korpus/rfc0040-f-oturumlu-ham-govde.dil` | `rfc0040-f-oturumlu-ham-govde/ana.dil` | `b4e2d1fdaec3fb5174569a1409693d3bfeee1bcf` | `410cb25ef4041ced9bbc19901a44659d05576eafa173936e798895b09354c1d8` | T062 |
| `korpus/rfc0040-f-put-makine.dil` | `rfc0040-f-put-makine/ana.dil` | `f18123cb1e1f920b66fe7e5d8d96a374ceb1acfe` | `fdaf46b38ee84977de6118e3fc9615316c224cf20a3c788ec31a52a183dee05e` | T060 |
| `korpus/zm-f027-istemci-ip.dil` | `zm-f027-istemci-ip/ana.dil` | `089005c2cd94ffbc337199a33a419595623036e3` | `224c9b7e0ceb467dfd56d3e6b2234be4dacced1454c0222490d5647b718470f3` | S015 |
| `zm-f050-dkim-16-ustu/proje.dil` | `zm-f050-dkim-16-ustu/proje.dil` | `23d4d5cf9532a376d9f79ecd01ddc82158e33883` | `c95b71226c72beb42e52d699d68a1b84400b222572cd9339dc8b84c3099ace6c` | P015 |
| `zm-f050-dkim-16-ustu/ana.dil` | `zm-f050-dkim-16-ustu/ana.dil` | `b4909b5c85a38a3c7d097498b3a1609e35290c0f` | `8ee141b465e8650cf9f63826b2ea53c705370693e59dfa490ccdd3b38586ff1a` | — |
| `zm-f050-dkim-ortak-credential/proje.dil` | `zm-f050-dkim-ortak-credential/proje.dil` | `2fb86627a8c0ba18797b045375500d2d4619b9ef` | `7a2a6091a02c3709d45fcffc780a2f8a43ad0898f2fe6f00e6eb07e98160efc8` | P015 |
| `zm-f050-dkim-ortak-credential/ana.dil` | `zm-f050-dkim-ortak-credential/ana.dil` | `b4909b5c85a38a3c7d097498b3a1609e35290c0f` | `8ee141b465e8650cf9f63826b2ea53c705370693e59dfa490ccdd3b38586ff1a` | — |

Ürün tarafındaki aday manifesti `korpus-aday.tsv` (blob
`5d0cac8babf65a99b87e5660f102e234a044ccb8`, SHA-256
`55d9e057f0c73058e87c281a03d63243caa1fca48bd2813932703c2f4beae772`) ve notlar
(`README.md`, blob `26d0884b58a24200f449b2bd579c9541d94d68d3`, SHA-256
`433c4c890e928d54631eaf4035eb5362a8ab9a7529f68b5f60e813576f0995a9`) depoya
alınmadı; satırlar `dogfood/korpus-v1.tsv` biçimine çevrildi.

## Nasıl sınanır

Tek dosyalı on vaka `denetle` kipindedir: kaynak tek dosya olarak derlenir
ve ürün kökü [`proje.dil`](proje.dil) politikasıyla denetlenir; yalnız tanı
kodu karşılaştırılır. Ürünün vaka başına `proje.dil` dosyaları (yetkinlik
`ağ-sunucusu`, `web-oturumu`, E'de ayrıca `kriptografi`) ürün kökü
politikasının alt kümesidir; ürün kökünde de aynı tanılar çıktığından ürün
politikası genişletilmedi ve klasör düzeni gerekmedi.

ZM-F050 tanısı proje bildiriminden (`proje.dil`) gelir, tek dosya
derlemesiyle görülemez. Bu iki vaka klasördür ve `proje` kipinde
`basarisiz` beklenir: giriş CLI ile aynı proje yükleyicisinden geçer, P015
yükleyiciden döner.

## Sürtünmeler

- **A (yöntem kaydı):** keşif PROPFIND, eşitleme REPORT ile yapılır; rota
  yöntemi kapalı kümede yok (S036).
- **B (istek başlığı):** `Depth` okunamıyor; `If-Match`/`If-None-Match`
  aynı durumda (S015). Ürünün ilk listesi bu üç başlıktır.
- **C (yanıt):** K-406 türlü metin yanıtı ile 207 durumu birlikte
  yazılamıyor (S037); yanıta ETag yazılamıyor (S015).
- **E (kimlik):** çerezsiz `Authorization: Basic` okunamıyor (S015).
  Okunsa bile istek başına Argon2id doğrulaması spec/12 §5 oran sınırına
  takılır; bu çalışma zamanı sürtünmesi burada sınanmaz.
- **F (CSRF muafiyeti):** makine uç noktası kimliksizdir (T050), yalnız
  POST'tadır (T060) ve ham gövde yalnız orada okunur (T062). Üç tanı da
  bilinçli güvenlik sınırıdır; vakalar onları gevşetme kararı değildir.
- **ZM-F027 (istemci IP'si):** web rotası istemci adresini göremiyor
  (S015). Form alanı olarak `istemci_ip` derlenir ama değerini saldırgan
  belirler; çözüm ADR-036 güvenilir vekil modeline göre çözülmüş ayrı bir
  yüzeydir.
- **ZM-F050 (DKIM anahtarları):** 17. alan için anahtar bildirilemiyor ve
  tek credential birden çok alana bağlanamıyor (P015, ADR-174 kararı).
  Ürünün bugünkü çözümü alan başına ayrı credential adıdır.

K-164 kataloğunda RFC-0040 vakaları ve ZM-F027 “web adaptörü yüzeyi eksik”
sınıfıdır (ZM-F022, ZM-F033, ZM-F048 ile aynı üründe tekrar; tasarım maddesi
K-364). ZM-F050 tek üründe, K-164 karşılığı yok. Ürünün sıra notu
[RFC-0040](../../rfcs/0040-webdav-caldav-carddav-adaptoru.md) açık
sorularındadır.
