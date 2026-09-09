# 08 — Kalıcı dosya IO

Normatif kaynak: RFC-0016, ADR-032. Durum: **TANIMLI** (K-084/K-128).

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

## Metadata ve hedef türü

Var olan hedef normal dosya olmak ZORUNDADIR. Sembolik bağ veya başka dosya
türü `InvalidInput` ile, eski nesne değiştirilmeden reddedilir.

- Linux/macOS mode, uid ve gid'yi korumak ZORUNDADIR.
- Linux görünür xattr namespace'lerini descriptor üzerinden taşımalıdır;
  POSIX ACL/security label bir xattr olarak görünüyorsa bu kümeye dâhildir.
  Ad listesi ve tek değer 64 KiB, bütün değerler 1 MiB'ı AŞAMAZ.
- macOS ACL ve xattr `fcopyfile` metadata yolu ile taşınmalıdır.
- Windows var olan hedefte hata-yoksaymasız `ReplaceFileW` kullanmalı; DACL,
  security resource attribute, encryption/compression ve replacement'ta
  bulunmayan named stream'leri korumalıdır. Kısmi taşıma hatasında aynı klasör
  yedeğinden eski hedef geri kurulmaya çalışılmalıdır.
- Bu kapsamın uygulanamadığı destekli platform hata vermek ZORUNDADIR;
  best-effort metadata düşürme YASAKTIR.

Windows'ta `ReplaceFileW` hedefi iki yeniden adlandırmayla değiştirir; aradaki
anda yolu açan okuyucu dosyayı bulamayabilir. Resmî okuma yolu bu pencerede
kısa ve sınırlı süre yeniden dener (K-182); böylece okuyucu yine eski ya da
yeni bütün içeriği görür.

İçerik değişikliğinde Unix mtime/ctime korunmaz. Görünmeyen Linux `trusted.*`,
Windows SACL/owner SID ve dosya sistemine özel immutable bayraklar verilmiş söz
değildir. Bunlar için yönetilen deployment aracı gerekir.

Resmî runtime aynı klasördeki `.zee-yazma-kilidi` adını kendine ayırır.
Programlar `.zee-` önekli çalışma dosyalarını kullanıcı verisi olarak
kullanmamalıdır.

Bu söz tek bir dosyanın commit sınırıdır. Birden çok dosya/veritabanı kaydını
tek iş kuralı olarak değiştiren transaction, RFC-0015'in uygulama eylemi
sınırında ayrıca tanımlanacaktır.

## Binary yaşam döngüsü

K-163/F030'un no-clobber atomik taşıma ve idempotent silme kuralları
[spec/26](26-binary-yukleme-ve-dosya-yasam-dongusu.md) bölümündedir. Taşıma
aynı dosya sistemi rename görünürlük sınırını, iki işlem de kalıcı dizin
eşzamanlamasını ve süreçler-arası hedef kilidini kullanır. Bunlar DB ile
dağıtık commit sözü vermez.
