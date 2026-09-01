# RFC-0016 — Atomik Kalıcı Dosya Sözleşmesi

- **Durum:** **geçici kabul** (1 Eylül 2026 — K-084 gerçeklendi; macOS
  conformance kanıtı var, Windows/Linux CI matrisi tam kabul kapısıdır)
- **Tarih:** 1 Eylül 2026
- **İlgili kayıtlar:** K-019, K-084, V1-P0-04, RFC-0015 §4
- **Gerçekleme:** `kalici_dosya.rs`; `GercekIo::dosya_yaz`, paket kilidi ve
  biçimleyici yazma yolları; birim hata enjeksiyonu ve iki-süreç regresyonları

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

## 2. Gerçekleme modeli

- Aynı klasörde kalıcı `.zee-yazma-kilidi` dosyası açılır. Unix'te
  `flock`, Windows'ta `LockFileEx` kullanılır. Kilit dosyasını silmemek,
  bekleyen süreçlerin farklı inode/handle kilitleyip birbirini atlamasını
  önler.
- Yeni içerik hedefle aynı klasörde benzersiz `.zee-gecici-...` dosyasına
  yazılır. İçerik `sync_all` ile kalıcılaştırılır; eski izinler korunur.
- Unix'te atomik `rename`, Windows'ta replace + write-through
  `MoveFileExW` commit noktasıdır. Unix'te klasör de eşzamanlanır.
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

## 5. Elenen seçenekler

- **Doğrudan truncate:** hata/çöküş eski veriyi yok eder.
- **Yalnız O_APPEND:** tek çağrıda ofset yarışını azaltır ama kısmi yazma,
  dayanıklılık ve bütün-dosya güncellemesini çözmez.
- **Silinen lockfile:** bekleyen süreç eski kilit nesnesinde kalırken yeni süreç
  başka nesneyi kilitleyebilir. Kilit dosyası bu yüzden kalıcı ve gizlidir.
- **Dış crate:** v0'ın sıfır dış bağımlılık/tedarik zinciri kararını bozar
  (ADR-001); küçük yerel FFI sınırı daha denetlenebilirdir.

## Tam kabul kapısı

Windows, macOS ve Linux'ta aynı conformance paketi; güç-kesintisi/disk-dolu
hata enjeksiyonu; büyük dosya davranışı ve gözlemlenebilir performans sınırı.
