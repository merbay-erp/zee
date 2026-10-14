# K-232 — Katalog PDF e-posta ihtiyacı

Bu anonim kesit aktif `zee-cekirdek` ürününün gerçek B2/P10 talebine bağlıdır.
Harici talep commit’i `cd5d12f0008620819e6167af5146e83503fd2b38`, talep dosyası
`docs/dil-talepleri/2026-10-14-zee-urunlesme.md`, dilim B2'dir. Gerçek etkilenen
Zee dosyası `moduller/pdf-katalog/kaynak/rota_sm_katalog.dil`; bu exact commit’te
ve salt okunur ürün kopyasında SHA-256
`b8721f8044ee976c3f1770bfe43d4788e0e962ecd6196be7845367c54d29fb59` eşleşir.
Özel ürün gövdesi, alan adı, müşteri verisi veya bağlantı sırrı taşınmaz.

[Reproducer](katalog-talep.dil), yerelde üretilen PDF’i mevcut SMTP ayarıyla
TR/EN düz metin ve HTML gövdesine ekler; yanıt adresini ayrıca taşır.
Mevcut HTTP helper yerel binary dosyayı UTF-8/Base64 API’si olmadan okuyamaz;
dosyayı dışarı açma varsayımı da ürün talebi değildir. Bu yüzden tek yedi
argümanlı intrinsic, genel binary türü/FFI/queue eklemekten küçük yüzeydir.
Karar [ADR-096](../../../adr/096-ekli-eposta-ve-mime.md),
[RFC-0034](../../../rfcs/0034-ekli-eposta.md) ve
[spec/23](../../../spec/23-yetkinlik-ve-outbound-guvenligi.md) zincirindedir.

```zee
sonuc alici konu duz html yanit ekler baglanti ile ekli ileti göndermeyi dene olsun
```

İlk beş ve son argüman Metin, `ekler` Metin listesidir. Boş HTML/yanıt adresi
ilgili opsiyonu kaldırır; boş ek listesi dosya açmaz. Boş bağlantı yalnız ana
projenin `e_posta_bağlantı_değişkeni` ayarını kullanır. SMTP URL’si kaynakta
veya bildirimde sır olarak saklanmaz. Ürün sahibi `ağ`, SMTP origin’i ve ekler
için `dosya-okuma` açar; loopback testte ayrıca `yerel-ağ` gerekir.
`dosya-yazma` gerekmez. Bu örnek standalone gönderim/deploy komutu değildir.

`compiler/tests/smtp_mime_testi.rs` bu exact dosyayı include ederek typed
lowering, yanlış alan/listeler, T048, C029 sır maskeleme, yetki/yol reddi,
iz yürütme/export/replay reddi, playground desteklenmeme, üst son tarih ve
gerçek sahte SMTP’de binary PDF/Reply-To/alternative gönderimini sınar.
Native codec birim testleri gerçek STARTTLS/SMTPS, güvenilmeyen sertifika,
timeout, uzun Unicode başlık, boyut, CRLF ve dot-stuffing kanıtını taşır.
Sentetik TLS kimliği yalnız test connector’ında güvenilirdir.

Canlı SMTP/M365/Gmail kabulü, özel ürün rota değişikliği ve staging
entegrasyonu yapılmadı. P09 kalıcı kuyruk/retry/outbox/cron, SMTPUTF8, genel
binary API, başka ek türleri ve ek içeriğinin kötü amaçlı dosya denetimi
**açıktır**. DATA başarı yanıtı kaybolursa sonuç belirsiz olabilir; otomatik
tekrar yoktur. PDF üretimi bu işin konusu değildir; üretilmiş dosya okunur.
