# 21 — Deterministik IO izi

Normatif kaynak: RFC-0022, ADR-026. Gerçekleme:
`yorumlayici/io_izi.rs` ve `dil iz`.

## Kayıt (TANIMLI)

`IzKaydedenIo`, `GirdiCikti` üzerinden tamamlanan her çağrıyı tek küresel
sırayla kaydeder. Olay; işlem adı, sıralı argüman alanları ve sıralı sonuç
alanları taşır. Host çağrısı tamamlanmadan olay tamamlanmış sayılmaz.

`dil iz kaydet <iz> <program> [argümanlar]` gerçek koşuyu çalıştırır ve şema-1
izini atomik yazar. İz üretimi açık opt-in'dir.

## Replay (TANIMLI)

`IzYenidenOynatici` dış IO adaptörü kullanmaz. Her çağrı sıradaki olayın işlem
ve argümanlarıyla birebir eşleşmek ZORUNDADIR. Eşleşirse kayıtlı sonuç döner;
farkta koşu fail-closed başarısız olur. Başarılı koşu izin tamamını tüketir.

Replay dosya/ağ/web/transaction/donanım etkisini yeniden uygulamaz; bu etkilerin
aynı protokolle istendiğini doğrular. `dil iz oynat` kullanıcı çıktısını ancak
bütün iz eşleştiğinde gösterir.

## Şema-1 byte biçimi (TANIMLI)

- Başlık: `zee-io-izi<TAB>1<LF>`.
- Olay: `sıra, işlem, argüman_sayısı, sonuç_sayısı, alanlar`; ayraç TAB,
  bitiş LF'dir.
- Sıra 1 tabanlı ve kesintisizdir. Sayılar kanonik onluktur.
- Alanlar, UTF-8 metnin küçük harfli hex yazımıdır. Hex şifreleme değildir.
- İşlem sözlüğü ve işlem-özel alan/tag şemaları kapalıdır.
- En çok 64 MiB, 100.000 olay ve olay başına 4.096 alan kabul edilir.
- Bilinmeyen sürüm/işlem, sıra boşluğu, büyük/bozuk/non-kanonik alan yürütme
  öncesinde reddedilir.

Şema, işlem sözlüğü veya alan semantiğindeki kırıcı değişiklik yeni başlık
sürümü gerektirir.

## Gizlilik (TANIMLI)

İz; kullanıcı girdisi, dosya/ağ içeriği, argüman, çerez ve token taşıyabilir;
özel artefakttır ve varsayılan olarak Git dışında kalır. Parola ile PHC özeti
ham/geri çevrilebilir alan olamaz; yalnız ayrı SHA-256 parmak izleri kaydedilir.

## Sınır

Kaynak program iz içine gömülmez. Derleme/birim yükleme yürütme IO izi değildir.
Saat/rastgele tohum ve sanal zaman ilerleme algoritmasının sürüm sözleşmesi
B-028 kapsamındadır; şema-1 bugün gözlenen çağrı, sonuç ve sırayı korur.
