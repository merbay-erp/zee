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
| `korpus/f004-cr-kacisi-yok.dil` | ürün reproducer'ı (çıplak CR) | S040 |
| `korpus/f001-yerel-ayarli-buyuk-harf.dil` | ürün reproducer'ı (harf katlama) | `MAİL FROM:<A@B.TEST>` |
| `korpus/f001-ascii-katlama-elle.dil` | ürünün ASCII metin yardımcısı | `MAIL FROM:<A@B.TEST>` |

Reproducer'lar ürün kodundan hermetik küçültmedir; sunucu adı, alan adı,
adres, yerel yol ve ürünün geri kalan kaynağı taşınmaz (ADR-087). Etkilenen
ürün kaynakları SMTP oturum durum makinesi ve ASCII metin yardımcılarıdır.
Ürün kaydı compiler değişikliği getirmez; RFC-0039'un her alt RFC'si bu
kökü kanıt olarak anar.
