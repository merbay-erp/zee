# ADR-032 — Atomik replace metadata koruma politikası

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-128, B-048, V1-P0-30, RFC-0016, spec/08
- **Karar sahipleri:** çekirdek ekip

## Bağlam

K-084 içeriği aynı klasörde yeni inode'a yazıp atomik replace yapıyor ve eski
dosyanın yalnız izin bitlerini kopyalıyordu. Bu, Unix owner/group ve ACL/xattr;
Windows DACL, security resource attribute ve named stream bilgisini sessizce
kaybettirebilirdi. Güvenlik etiketi kaybı, byte içeriği doğru olsa bile dosyanın
erişim sınırını değiştirebilir.

Metadata, platformların aynı kavramı aynı API ile sunmadığı bir alandır.
Taşınabilir görünen ama bazı hedeflerde sessizce metadata düşüren tek bir
"best effort" davranışı kabul edilmemiştir.

## Karar

1. Var olan hedef yalnız normal dosyaysa değiştirilir. Sembolik bağ ve başka
   dosya türleri `InvalidInput` ile, eski hedefe dokunmadan reddedilir.
2. Linux/macOS'ta eski normal dosya açık descriptor ile yakalanır. Yeni inode
   aynı uid/gid ve mode'u alamazsa commit yapılmaz. İçerik değiştiği için
   mtime/ctime eski değere döndürülmez.
3. Linux'ta descriptor tabanlı `flistxattr/fgetxattr/fsetxattr` bütün görünür
   namespace'leri taşır. POSIX ACL, SELinux etiketi ve file capability bu
   dosya sisteminde xattr olarak görünüyorsa aynı yoldan geçer. Ad listesi ve
   tek değer 64 KiB, bütün değerler 1 MiB ile sınırlıdır. Okunabilen metadata
   yazılamazsa işlem fail-closed durur.
4. macOS'ta `fcopyfile(COPYFILE_ACL | COPYFILE_XATTR)` ACL, extended attribute,
   quarantine/Finder bilgisi ve resource fork'u descriptor'lar arasında
   taşır; data/stat zamanı kopyalanmaz. API hatası committen öncedir.
5. Windows'ta var olan hedef `ReplaceFileW` ile, metadata/ACL merge hatalarını
   yoksayan bayraklar olmadan değiştirilir. DACL, security resource attribute,
   encryption/compression ve replacement'ta bulunmayan named stream'ler
   korunur. Aynı klasörde kurtarma yedeği kullanılır; nadir kısmi taşıma hatası
   eski dosyayı atomik `MoveFileExW` ile geri kurmayı dener. Yeni hedef ilk kez
   yaratılıyorsa write-through `MoveFileExW` yolu korunur.
6. Tier-1 dışındaki Unix platformlarında var olan dosyanın ACL/xattr'ı güvenle
   taşınamadığından replace `Unsupported` olur. Sessiz metadata kaybı, geniş
   uyumluluktan daha kötü kabul edilir.

## Açık sınır

Linux `listxattr`, çağıranın göremediği `trusted.*` bilgisini listelemeyebilir;
bu nedenle ayrıcalıklı/gizli inode metadata'sını sıradan kullanıcı sürecinde
koruma sözü verilmez. Bu tür yönetilen dosyalar deployment aracınca yerinde
güncellenmeli veya Zee işlemi açıkça reddedilmelidir. Windows SACL/owner SID ve
dosya sistemine özel immutable bayraklar da bu ADR'nin taşınabilir sözü
değildir. Verilen söz ve verilmeyen söz spec/08'de açık kalır.

## Kanıt

- macOS: mode+uid+gid, binary xattr ve gerçek extended ACL replace sonrasında
  korunur;
- Linux/macOS: sembolik bağ fail-closed kalır; Linux xattr yolu aynı regresyonu
  taşır ve üç kaynak bütçesine tabidir;
- Windows: NTFS named stream testi yalnız `ReplaceFileW` metadata merge'iyle
  geçebilir;
- eski atomiklik, hata enjeksiyonu, iki yazar ve ani süreç sonu testleri aynen
  korunur; metadata aktarımı bu sözleri zayıflatamaz.

Tier-1 CI matrisi Linux, macOS ve Windows'ta aynı kaynak testlerini çalıştırır.

## Platform kaynakları

- [Linux xattr(7)](https://man7.org/linux/man-pages/man7/xattr.7.html)
- [Apple copyfile(3)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/copyfile.3.html)
- [Microsoft ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew)
