# RFC-0016 — Atomik Kalıcı Dosya Sözleşmesi

- **Durum:** **geçici kabul** (K-084 içerik atomikliği, K-128 metadata
  sözleşmesi; güç-kesintisi/disk-dolu kampanyası tam kabul kapısıdır)
- **Tarih:** 1 Eylül 2026
- **İlgili kayıtlar:** K-019, K-084, K-128, B-048, V1-P0-04/V1-P0-30,
  RFC-0015 §4, ADR-032
- **Gerçekleme:** `kalici_dosya.rs`, `kalici_dosya/metadata.rs`;
  `GercekIo::dosya_yaz`, paket kilidi ve biçimleyici yazma yolları; birim hata
  enjeksiyonu, metadata ve iki-süreç regresyonları

## Özet

zee'ye dosya emanet eden bir program, süreç ya da makine beklenmedik anda
durduğunda hedefte yarım içerik görmemelidir. Aynı dosyaya iki zee süreci
yazdığında güncelleme sessizce kaybolmamalıdır. Bu RFC, mevcut
`"X" dosyasına ... yaz/ekle` yüzeyini değiştirmeden kalıcılık sözünü
profesyonel seviyeye çıkarır.

## 1. Sözleşme

1. **`yaz`** hedefin tamamını tek satır + satır sonuyla değiştirir.
2. **`ekle`** var olan içeriği korur ve yeni satırı sonuna ekler.
3. Okuyucu değişiklik sırasında yarım içerik GÖREMEZ: eski ya da yeni bütün
   sürümü görür.
4. Aynı klasördeki zee yazarları işletim sistemi kilidiyle sıralanır. İki
   başarılı `ekle` işleminin ikisi de dosyada bulunur. İki `yaz` işlemi
   için kilidi son alanın bütün değeri kazanır; baytlar karışmaz.
5. Commit noktasından önce hata olursa eski hedef korunur ve geçici dosya
   temizlenir. Süreç aniden biterse işletim sistemi kilidi otomatik bırakır.
6. Kilit beş saniyede alınamazsa işlem C013 yolundan görünür hatayla durur;
   güvenli olmayan kilitsiz yazmaya düşmez.
7. Var olan normal dosyanın platformca desteklenen erişim/güvenlik metadata'sı
   korunmadan commit yapılamaz. Sembolik bağ ve desteklenmeyen metadata
   platformu sessizce normal dosyaya/eksik etikete dönüştürülemez.

## 2. Gerçekleme modeli

- Aynı klasörde kalıcı `.zee-yazma-kilidi` dosyası açılır. Unix'te
  `flock`, Windows'ta `LockFileEx` kullanılır. Kilit dosyasını silmemek,
  bekleyen süreçlerin farklı inode/handle kilitleyip birbirini atlamasını
  önler.
- Yeni içerik hedefle aynı klasörde benzersiz `.zee-gecici-...` dosyasına
  yazılır. İçerik ve metadata committen önce tamamlanıp `sync_all` ile
  kalıcılaştırılır. Unix mode+uid+gid ortak tabandır. Linux görünür xattr'ları
  (POSIX ACL/security label dâhil) descriptor üzerinden ve 64 KiB ad/değer +
  1 MiB toplam bütçeyle; macOS ACL+xattr'ı `fcopyfile` ile taşır.
- Unix'te atomik `rename` commit noktasıdır ve klasör de eşzamanlanır. Var olan
  Windows hedefi DACL/security resource/named stream metadata'sını birleştiren,
  hata yoksayma bayraksız `ReplaceFileW` ve aynı klasör kurtarma yedeğini
  kullanır. İlk yaratma write-through `MoveFileExW` yoludur.
- `ekle`, aynı kilit altında oku + birleştir + atomik replace yapar. Bu,
  doğrudan append'in çöküşte yarım satır ve yarışta kayıp güncelleme riskini
  ortadan kaldırır.
- Atomik replace/kilit sözünü veremeyen platform yazmayı
  `Unsupported` hatasıyla reddeder; sessizce zayıf semantiğe geçmez.

## 3. Araç zinciri

`dil biçimle`, `proje.kilit` ve paket bildirimi geri-yükleme yolları da
aynı tek-dosya primitive'ini kullanır. Paket bildirimi + kilit gibi çok
dosyalı işlemler K-079'un doğrula-önce/geri-al sözleşmesini korur; birden fazla
kaynağı kapsayan uygulama transaction'ı bu RFC'nin iddiası değildir ve
RFC-0015 eylem modelinde kalır.

## 4. Kanıt

- replace öncesi enjekte edilen hata eski veriyi ve geçici dosya temizliğini;
- yarışan okuyucu yalnız eski/yeni 32 KiB görüntünün bütününü;
- iki iş parçacığı 80 yarışlı eklemenin kaybolmadığını;
- iki bağımsız `dil` süreci 80 satırın tamamını;
- Drop çalıştırmadan biten çocuk süreç işletim sistemi kilidinin salındığını;
- temel test ise `yaz` + `ekle` yüzeyinin geriye uyumunu sınar.
- macOS mode/uid/gid, binary xattr ve gerçek ACL'yi; Linux aynı descriptor
  tabanlı mode/uid/gid+xattr yolunu; Windows NTFS named stream'i platform CI
  testinde korur. Symlink hedef her Unix testinde fail-closed kalır.

## 5. Metadata kapsamı

İçerik değiştiği için Unix mtime/ctime eski değere döndürülmez. Linux'ta
çağıranın göremediği ayrıcalıklı `trusted.*`, Windows SACL/owner SID ve dosya
sistemine özel immutable bayraklar taşınabilir söz değildir. Okunup taşınması
gereken bir metadata yazılamıyorsa eski hedef korunarak hata verilir. Tier-1
dışı Unix'te var olan dosya replace'i ACL/xattr koruması kanıtlanmadığı için
`Unsupported` olur. Ayrıntılı platform kararı ADR-032'dedir.

## 6. Elenen seçenekler

- **Doğrudan truncate:** hata/çöküş eski veriyi yok eder.
- **Yalnız O_APPEND:** tek çağrıda ofset yarışını azaltır ama kısmi yazma,
  dayanıklılık ve bütün-dosya güncellemesini çözmez.
- **Silinen lockfile:** bekleyen süreç eski kilit nesnesinde kalırken yeni süreç
  başka nesneyi kilitleyebilir. Kilit dosyası bu yüzden kalıcı ve gizlidir.
- **Best-effort metadata kopyası:** ACL/security label kaybını başarılı yazma
  gibi gösterir. Desteklenen metadata tam taşınır veya commit yapılmaz.
- **Platform komutları:** locale, kurulum ve shell yüzeyine bağlıdır. Exact
  `libc` descriptor API'leri ve Windows sistem çağrısı denetlenebilir sınırdır.

## Tam kabul kapısı

Linux/macOS/Windows metadata conformance'ı K-128 ile CI'a bağlıdır. Kalan tam
kabul kapısı güç-kesintisi/disk-dolu hata enjeksiyonu, büyük dosya davranışı ve
gözlemlenebilir performans sınırıdır.
