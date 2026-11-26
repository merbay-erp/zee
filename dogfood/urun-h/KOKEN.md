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
| `korpus/zm-f042-dkim-imza-yok.dil` | ZeeMail sürtünmesi ZM-F042 (ürün commit'i `850d86b494839a817bf2137fd36cebd9b3c880db`, `zeemail/docs/dogfood/zm-f042-yeniden-uretim.dil` SHA-256 `2ab1b5728933575d6693f709e470392accf70754c2c3de23cb80dde3f777f23b` blob `e732f3f6…`; `surtunmeler.md` satır 62; ürünün `\r\n` sabiti LF yazıldı, kalıp K-397 ile anlaşılan son biçimde) | S015; K-397 Dilim 2 (RFC-0044 B3) sonrası temiz derlenir |
| `korpus/zm-f046-dkim-govde-dosyadan.dil` | ZeeMail sürtünmesi ZM-F046 (ürün commit'i `d6705f1925da787c7ffba4b494e287f3ca34e02e`, gözlem `cb047c5`; `zeemail/docs/dogfood/surtunmeler.md` satır 66 SHA-256 `5c704d4ed9ba8c1aa7364376d6d723aef7697d96a75f3efdf197248c0b421aac` blob `53213249…`; `zm-f046-yeniden-uretim.dil` SHA-256 `2a8859482f06ef482d618b1d7751bf76ca3fc386486b3fad26042532643d577d` blob `aa0b8978…`, birebir) | S015; K-397 Dilim 2 (ADR-174 §8) sonrası temiz derlenir |
| `korpus/zm-f049-ekli-govde-dosyada.dil` | ZeeMail sürtünmesi ZM-F049 (ürün commit'i `e524850b080e6920df30760ac1803a51a5ed0a2c`; `zeemail/docs/dogfood/surtunmeler.md` blob `d9f9ce8eeb13af4df5dacc1fbc2a1fcef686d9b9`; etkilenen `disteslim-hizmet/kaynak/ana.dil` blob `8fe6693f8abae28f7488425fbad6bead5640fb27` (yalnız eksiz ileti imzalanıyor) ve `disteslim/kaynak/ana.dil` blob `28fba7bb395075ad1d2cffe6ac174a57ca53bc1c` (ek parçaları yalnız bağlantıya akıyor); uçtan uca `disteslim-deneme/dene.sh` blob `9543bf953c691874e1c0033729af9ab4889d71ab`) | kalıp yok; K-408 sonrası temiz |
| `korpus/zm-f043-dkim-dogrulama-yok.dil` | ZeeMail sürtünmesi ZM-F043 (ZM-F024/F025'e bağlı; ürün commit'i `1fe6a524dcf910aea3b26f2e2c68485d2fdeead1`, `zeemail/docs/dogfood/surtunmeler.md` SHA-256 `17fc1cad8a9db447a389bcc739bfeec4d54a4890cc528087d139f53c2c5a986c` blob `98e22df1…` satır 44/45/63; ürün yeniden üretimi `zm-f043-yeniden-uretim.dil` SHA-256 `169b9bafd51d5a373c1659eb6cbcaa8458b7da8a2a1701b03803296db753a65b`; alan adı anonim) | S015; K-390 (RFC-0044 B1b/B2b) sonrası temiz derlenir |
| `korpus/f004-cr-kacisi-yok.dil` | ürün reproducer'ı (çıplak CR) | S040 |
| `korpus/f001-yerel-ayarli-buyuk-harf.dil` | ürün reproducer'ı (harf katlama) | `MAİL FROM:<A@B.TEST>` |
| `korpus/f001-ascii-katlama-elle.dil` | ürünün ASCII metin yardımcısı | `MAIL FROM:<A@B.TEST>` |
| `korpus/zm-r004-uzunluk-oktet-saymaz.dil` | ürün reproducer'ı ZM-R004 (ürün commit'i `7f7b1fe48035ccff1fe4c78810b19e00234ca8ac`, blob `60e565e7…`; K-402 ikinci satırı ekledi) | `1\|2` (K-402 `utf8 bayt sayısı`) |
| `korpus/zm-f023-auth-plain.dil` | ürün commit'i `23af030781a40696f803f90709356f94c04631e4`, `motor/kaynak/smtp_gonderim.dil` (blob `9aba5391…`) AUTH PLAIN çözümünün küçültmesi; kullanıcı ve parola yer tutucu | S015; K-402 sonrası `AGF5…` ve 3 parça |
| `korpus/zm-f023-imap-literal-oktet.dil` | aynı commit, `imap/kaynak/ana.dil` (blob `25cf0e4e…`) literal bayt sayımı; metin ürünün ölçüm örneği | S015; K-402 sonrası `14\|18` |
| `korpus/zm-f023-rfc2047-emoji.dil` | aynı commit, `motor/kaynak/ileti_olustur.dil` (blob `40aefd2a…`) ve `kodlamalar.dil` (blob `3ee434b5…`) kod noktası/base64 tablosu; konu ürünün örneği | S015; K-402 sonrası `😀 Fatura`, 128512, gidiş-dönüş |
| `korpus/zm-r005-bellekte-sha256.dil` | ürün reproducer'ı ZM-R005 (aynı commit, blob `f994eaab…`) | S015; K-389 sonrası gerçek SHA-256 |
| `korpus/zm-f030-yumusak-parametre.dil` | ZeeMail sürtünmesi ZM-F030 (ürün commit `a180e0c3b38384ca1b27c324811db71dc0dbe8ce`, `imap/kaynak/ana.dil` blob `26863c63…`, SHA-256 `f48c444ea345452534b9015d9952b2fb449d3e5738489c886baf52d96b9413cf`; `klasörleri listele` `kalıbı` yerine `deseni`) | A001; K-398 (ADR-175) sonrası `doğru`, `yanlış`, `doğru` |
| `ek-cikarma/ana.dil` | ürün ek çıkarma kesiti (ZM-F034, ürün commit'i `4f8c67cf6de936d1f17a689598f65cdca2bb4918`) | S015; K-389 sonrası temiz |
| `paylasilan-depo/alici/ana.dil`, `paylasilan-depo/web/ana.dil` | ürün paylaşılan ek deposu kesiti (ZM-F038, ürün commit'i `35df031accc4d010ed1ca3db1f2c479df40f2916`) | P001; K-394 sonrası temiz |
| `imap-govde-base64/ana.dil` | ürün IMAP FETCH gövde akışı kesiti (ZM-F040, ürün commit'leri `171f6b2a8dc5fcbb2f42c4f769f94a8ced968eca`, `c455cc8d1780ade481adc44752700bf6c9fdef8c`) | S015; K-396 sonrası temiz |
| `takvim-yayini/ana.dil` | ürün webmail takvim yayını rotası kesiti (ZM-F048, ürün commit'i `b75db4c7a460652f24c5d96d60817593587fbc70`) | S037; K-406 sonrası temiz |

Reproducer'lar ürün kodundan hermetik küçültmedir; sunucu adı, alan adı,
adres, yerel yol ve ürünün geri kalan kaynağı taşınmaz (ADR-087). Etkilenen
ürün kaynakları SMTP oturum durum makinesi ve ASCII metin yardımcılarıdır.
Ürün kaydı compiler değişikliği getirmez; RFC-0039'un her alt RFC'si bu
kökü kanıt olarak anar.
