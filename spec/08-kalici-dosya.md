# 08 — Kalıcı dosya IO

Normatif kaynak: RFC-0016. Durum: **TANIMLI** (K-084).

## Satır yazma

`"<yol>" dosyasına <değer> yaz`, değerin resmî metin gösterimini ve ardından
tek `\n` baytını hedefin **bütün yeni içeriği** yapar.

`"<yol>" dosyasına <değer> ekle`, var olan baytları değiştirmeden aynı satırı
sonuna ekler. Hedef yoksa iki biçim de dosyayı oluşturur.

## Atomiklik ve yarış

- Başarılı tek-dosya değişikliği **atomiktir**: eşzamanlı okuyucu eski ya da
  yeni bütün içeriği görür; ara/geçici içerik göremez.
- Aynı klasördeki zee yazma işlemleri süreçler arasında **sıralıdır**. Başarılı
  iki eklemenin hiçbiri kaybolamaz. Örtüşen iki baştan yazmada son kilit
  sahibinin bütün içeriği kalır; parçalar karışamaz.
- Committen önceki hata eski hedefi değiştiremez. Geçici içerik temizlenir.
  Yazarı taşıyan süreç aniden sonlansa da işletim sistemi kilidi salınır.
- Dayanıklılık/kilit primitive'leri bulunmayan platform işlemi reddetmek
  ZORUNDADIR; daha zayıf sessiz fallback YASAKTIR.
- Beş saniyede alınamayan yazma kilidi hata sayılır ve dosya korunur (C013).

Resmî runtime aynı klasördeki `.zee-yazma-kilidi` adını kendine ayırır.
Programlar `.zee-` önekli çalışma dosyalarını kullanıcı verisi olarak
kullanmamalıdır.

Bu söz tek bir dosyanın commit sınırıdır. Birden çok dosya/veritabanı kaydını
tek iş kuralı olarak değiştiren transaction, RFC-0015'in uygulama eylemi
sınırında ayrıca tanımlanacaktır.
