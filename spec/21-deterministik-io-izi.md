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
`dosyalari_listele`, `dosya_sha256` ve (K-333) `dosya_ilk_baytlari`
işlemlerini de kapsar. Argümanlar exact göreli yollar (imza okumasında ayrıca
onluk bayt sayısı); sonuçlar başarı etiketiyle sayı/liste/özet/onaltılık imza
veya hata mesajıdır. Replay hiçbir fiziksel taşıma, silme, listeleme, hashing
ya da okuma yapmaz.

ZORUNLU (K-389, ADR-167): `dosya_araligi_oku` olayı `[yol, konum, adet]`
argümanı ve `ok,<okunan bayt>,<sha256> | hata,<mesaj>` sonucu,
`dosyaya_bayt_yaz` olayı `[yol, yaz|ekle, <bayt>, <sha256>]` argümanı ve
`ok,<hedef sha256>,<toplam bayt> | hata,<mesaj>` sonucu taşır; ikisi de
dosya içeriğini ya da kodlanmış girdiyi izde TAŞIYAMAZ. Yazım olayı replay'de
kayıtlı sonucu döndürür ve dosyaya dokunmaz; başarılı aralıklı okuma içerik
üretemediği için replay iz uyuşmazlığıyla fail-closed biter. Mevcut
`dosya_oku` olayı değişmez.

K-337 `birincil_surec` olayı argümansızdır ve tek `0`/`1` sonucu taşır; replay
yönetici ya da işçi grubu kurmaz, kayıtlı rolü döndürür.

K-390 `dns_sorgula` olayı (spec/31 §1) argüman olarak kanonik soru adını ve
tür yazımını (`a`, `aaaa`, `mx`, `txt`, `tlsa`, `soa`) taşır; son tarih
bütçesi kaydedilmez. Sonuç `ok, durum, doğrulanmış(0/1), kanonik_ad` ve
etiketli sabit alanlı kayıtlar (A/AAAA/TXT/TLSA verisi onaltılık) ya da
`hata, tür, mesaj`tır; `var` durumu kayıtsız, diğer durumlar kayıtlı olamaz.
Replay hiçbir DNS paketi göndermez. Ekleme mevcut şema-1 izlerini bozmaz.
