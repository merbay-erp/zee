# ürün H ürün kökeni — K-355

Harici ürün: ürün deposu (özel), bütünüyle zee ile yazılması hedeflenen
e-posta sunucusu (SMTP/IMAP). Ürün sahibi ayrı Rust/Go servis motorunu
reddetti; ağ yetkinliği eksikleri [RFC-0039](../../rfcs/0039-ag-sunucusu-yetkinligi.md)
çatı taslağındadır.

Exact ürün commit'i: `91fcfc76a81f08b7551bf0e78c35c972d376e695`
(21 Kasım 2026; ilk ürün commit'i `d15e27ce5ac4017c73874684a31178e09aa7b9c3`).
Bu commit'te ürün Zee kaynağı saf SMTP komut durum makinesi (soket yok),
ASCII metin yardımcıları ve stdio deneme aracıdır; durum makinesi testleri
`dil dene` ile 19/19 geçer (dil `357a93c`, 21 Kasım 2026). Ürünün kendi
sürtünme kaydı on madde taşır; bu klasördeki adlar `K-355/F001–F010`
dilimleridir.

| Vaka | Ürün kaynağı | Bugünkü sonuç |
|---|---|---|
| `korpus/f009-dinleyici-yok.dil` | ürün reproducer'ı (dinleyici) | S036; K-381 sonrası temiz derlenir |
| `korpus/zm-r006-mx-sorgusu-yok.dil` | ZeeMail korpusu ZM-R006 (ürün commit `7f7b1fe48035ccff1fe4c78810b19e00234ca8ac`, SHA-256 `b645c2230562c1cd37e4eae221b916ad75a69f801946038753036e5a50f97d08`; alan adı anonim) | S015; K-390 (RFC-0044 B1a) sonrası temiz derlenir |
| `korpus/f004-cr-kacisi-yok.dil` | ürün reproducer'ı (çıplak CR) | S040 |
| `korpus/f001-yerel-ayarli-buyuk-harf.dil` | ürün reproducer'ı (harf katlama) | `MAİL FROM:<A@B.TEST>` |
| `korpus/f001-ascii-katlama-elle.dil` | ürünün ASCII metin yardımcısı | `MAIL FROM:<A@B.TEST>` |
| `korpus/zm-r004-uzunluk-oktet-saymaz.dil` | ürün reproducer'ı ZM-R004 (ürün commit'i `7f7b1fe48035ccff1fe4c78810b19e00234ca8ac`, blob `60e565e7…`) | `1` (Dilim 2 açık) |
| `korpus/zm-r005-bellekte-sha256.dil` | ürün reproducer'ı ZM-R005 (aynı commit, blob `f994eaab…`) | S015; K-389 sonrası gerçek SHA-256 |
| `korpus/zm-f030-yumusak-parametre.dil` | ZeeMail sürtünmesi ZM-F030 (ürün commit `a180e0c3b38384ca1b27c324811db71dc0dbe8ce`, `imap/kaynak/ana.dil` blob `26863c63…`, SHA-256 `f48c444ea345452534b9015d9952b2fb449d3e5738489c886baf52d96b9413cf`; `klasörleri listele` `kalıbı` yerine `deseni`) | A001; K-398 (ADR-175) sonrası `doğru`, `yanlış`, `doğru` |
| `ek-cikarma/ana.dil` | ürün ek çıkarma kesiti (ZM-F034, ürün commit'i `4f8c67cf6de936d1f17a689598f65cdca2bb4918`) | S015; K-389 sonrası temiz |
| `paylasilan-depo/alici/ana.dil`, `paylasilan-depo/web/ana.dil` | ürün paylaşılan ek deposu kesiti (ZM-F038, ürün commit'i `35df031accc4d010ed1ca3db1f2c479df40f2916`) | P001; K-394 sonrası temiz |
| `imap-govde-base64/ana.dil` | ürün IMAP FETCH gövde akışı kesiti (ZM-F040, ürün commit'leri `171f6b2a8dc5fcbb2f42c4f769f94a8ced968eca`, `c455cc8d1780ade481adc44752700bf6c9fdef8c`) | S015; K-396 sonrası temiz |

Reproducer'lar ürün kodundan hermetik küçültmedir; sunucu adı, alan adı,
adres, yerel yol ve ürünün geri kalan kaynağı taşınmaz (ADR-087). Etkilenen
ürün kaynakları SMTP oturum durum makinesi ve ASCII metin yardımcılarıdır.
Ürün kaydı compiler değişikliği getirmez; RFC-0039'un her alt RFC'si bu
kökü kanıt olarak anar.
