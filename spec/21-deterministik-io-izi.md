# 21 — Deterministik IO izi

Normatif kaynak: RFC-0022, ADR-026. Gerçekleme:
`yorumlayici/io_izi.rs` ve `dil iz`.

## Kayıt (TANIMLI)

`IzKaydedenIo`, `GirdiCikti` üzerinden tamamlanan her gözlenebilir IO çağrısını
tek küresel sırayla kaydeder. Olay; işlem adı, sıralı argüman alanları ve
sıralı sonuç alanları taşır. Host çağrısı tamamlanmadan olay tamamlanmış
sayılmaz. K-134 request commit/rollback kancaları bağımsız IO değildir; kayıt
sarmalayıcısı bunları olay eklemeden iç adaptöre geçirir.

`dil iz kaydet <iz> <program> [argümanlar]` gerçek koşuyu çalıştırır ve şema-1
izini atomik yazar. İz üretimi açık opt-in'dir.

## Replay (TANIMLI)

`IzYenidenOynatici` dış IO adaptörü kullanmaz. Her çağrı sıradaki olayın işlem
ve argümanlarıyla birebir eşleşmek ZORUNDADIR. Eşleşirse kayıtlı sonuç döner;
farkta koşu fail-closed başarısız olur. Başarılı koşu izin tamamını tüketir.

Replay dosya/ağ/web/PostgreSQL/transaction/donanım etkisini yeniden uygulamaz; bu etkilerin
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

İz; kullanıcı girdisi, dosya/ağ/veritabanı içeriği, argüman, çerez ve token taşıyabilir;
özel artefakttır ve varsayılan olarak Git dışında kalır. Parola ile PHC özeti
ham/geri çevrilebilir alan olamaz; yalnız ayrı SHA-256 parmak izleri kaydedilir.

CLI dosya kaydı yalnız sahibine erişimle yayımlanır: Unix `0600`, macOS
boş genişletilmiş ACL, Windows mirassız owner-rights DACL. Eski hedefin
geniş izinleri taşınmaz; symlink hedefi reddedilir. Bu davranış IO olaylarını
maskelemez ve gömme API’sinin döndürdüğü metnin saklama politikasını belirlemez.

## Sınır

Kaynak program iz içine gömülmez. Derleme/birim yükleme yürütme IO izi değildir.
Saat/rastgele tohum ve sanal zaman ilerleme algoritması spec/22'deki
`zee-io-1` profilidir. Şema-1 profil algoritmasını yeniden çalıştırmaz;
gözlenen çağrı, sonuç ve sırayı korur.

Şema-1'in dosya olayları `dosya_atomik_tasi`, `dosya_sil`,
`dosyalari_listele` ve `dosya_sha256` işlemlerini de kapsar. Argümanlar exact
göreli yollar; sonuçlar başarı etiketiyle sayı/liste/özet veya hata mesajıdır.
Replay hiçbir fiziksel taşıma, silme, listeleme ya da hashing yapmaz.
