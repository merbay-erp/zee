# 26 — Binary yükleme ve dosya yaşam döngüsü

Bu bölüm K-163/F030 dogfood gereksiniminin açtığı native web ve kalıcı dosya
sözleşmesini tanımlar (RFC-0027, ADR-061).

## Binary HTTP gövdesi

`Content-Type: application/octet-stream` taşıyan native HTTP isteği en çok
16 MiB olabilir. Adaptör gövdeyi bütünüyle belleğe ALMAZ; 16 KiB parçalarla
proje kökündeki `.zee/yuklemeler` dizininde `0600` izinli, benzersiz `.parca`
dosyasına yazar ve SHA-256'yı aynı akışta hesaplar. Başarılı framing sonunda
rota isteği şu alanları görür:

- `yukleme_gecici_yolu`: proje köküne göre güvenli geçici yol;
- `yukleme_sha256`: küçük harfli 64 haneli SHA-256;
- `yukleme_bayti`: onluk byte sayısı.

Unsafe rota CSRF belirtecini tek `X-Zee-CSRF` başlığından alır. Birden çok
`Content-Type`, sınırı aşan `Content-Length`, kısa/fazla gövde ve okuma son
tarihi mevcut fail-closed HTTP reddini korur. Kesilmiş akışın kısmi dosyası
silinmek ZORUNDA DEĞİLDİR; restart uzlaştırıcısının görebileceği orphan'dır.

## Beklenen dosya işlemleri

```text
kaynak dosyasını hedef yoluna atomik taşımayı dene
yol dosyasını silmeyi dene
dizin yolundaki dosyaları listelemeyi dene
yol dosyasının sha256 özetini almayı dene
```

İlk iki ifade `TamSayı sonucu`, listeleme `Metin listesi sonucu`, özetleme
`Metin sonucu` döndürür. Beklenen IO arızaları C013 kodlu `Hata` değeridir;
worker'ı düşürmez. Taşıma var olan hedefi ASLA ezmez ve aynı dosya sistemi
içindeki rename görünürlük sınırını kullanır. Silme bulunmayan hedefte başarılı
`0`, sildiğinde `1` döndürür. Listeleme yalnız dizinin doğrudan normal
dosyalarını, göreli ve sıralı verir; symlink/dizin döndürmez. Özetleme en çok
16 MiB'ı akışla okur.

Bu işlemler sırasıyla `dosya-yazma`/`dosya-okuma` yetkinliklerine bağlıdır,
IO trace/replay'de exact argüman ve sonuçla yer alır ve eylem savepoint'ini
korur. Aynı eylem içinde PostgreSQL ve dosya yazımı yine YASAKTIR.

## Açık sınır

Dosya sistemi ile PostgreSQL arasında dağıtık ACID sözü yoktur. Ürün
`hazırlanıyor → hazır/hatalı` ile `siliniyor` durumlarını, iş anahtarını ve
restart uzlaştırmasını kurmak zorundadır. Multipart/form-data, antivirüs/içerik
türü doğrulaması, production TLS/proxy yük limiti ve nesne deposu bu bölümün
sözü değildir.
