# ADR-032 — Atomik replace metadata koruma politikası

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-128, B-048, V1-P0-30, RFC-0016, spec/08, K-348
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

## Ek — K-348: yarım kalmış `ReplaceFileW` kurtarması (19 Kasım 2026)

**Bağlam.** K-346'nın çok süreçli regresyonu Windows'ta aralıklı olarak
`ortak.txt` hiç yokken düştü. Teşhis: `ReplaceFileW` hedefi iki adımda
değiştirir (eski hedef → `<geçici>.zee-yedek`, sonra geçici → hedef). Yazar
arada ölürse hedef yolda dosya kalmaz. Windows CI'da yazar süreci değiştirme
döngüsünde TerminateProcess ile öldürüldüğünde 400 turun 16'sında (%4) hedef
yoktu; Linux'ta 0/400 (POSIX rename tek adımdır). Her vakada eski bütün yedekte,
eksiksiz yeni içerik eş geçicide duruyordu. Sonuç gerçek veri kaybıydı:
okuyucu dosyayı bulamıyor, sonraki ekleme hedefi boş sayıp bütün önceki
satırları siliyordu. Üretimde işçi çökmesi, servis durdurma ya da güç kaybı
aynı durumu üretir.

**Karar.** `ReplaceFileW` (metadata merge sözü bu ADR'nindir) korunur ve
yarım durum kilit altında kurtarılır:

1. Klasörün yazma kilidini alan her zee işlemi, hedef yoksa ona ait en yeni
   `.<ad>.zee-gecici-*.zee-yedek` yedeğini hedefe geri adlandırır; eş geçici
   (yayımlanmamış yeni içerik) ve bayat yedekler silinir. Kilit tutulurken
   başka zee değiştirmesi süremeyeceğinden iz kesin olarak ölü bir yazarındır.
2. Geri yükleme yönü eski içeriktir: yazar başarı bildirmemiştir ve spec/08
   "committen önceki hata eski hedefi değiştiremez" der; yedek özgün nesne
   olduğu için DACL/stream metadata'sı da aynen döner. İleri sarma (geçiciyi
   yayımlamak) metadata merge'i yapılmamış nesneyi hedef yapabilirdi.
3. Kilitsiz resmî okuyucu dosyayı bulamazsa ve yedek varsa yazma kilidini
   paylaşımlı alıp önce hedefi, yoksa en yeni yedeği okur; böylece okuyucu
   ilk yazara kadar beklemeden eski bütünü görür.
4. Hedefi silen işlem bayat yedekleri de siler; aksi hâlde silinmiş içerik
   sonraki yazımda dirilirdi.

Kurtarma yalnız Windows'ta etkindir; POSIX bu durumu üretmez ve yeni dosya
oluştururken klasör taraması maliyeti eklenmez.

**Reddedilen.** `MoveFileExW(REPLACE_EXISTING)` tek adımlıdır ama DACL,
security attribute ve named stream merge sözünü taşımaz. Okuyucunun yedeği
kendisinin geri koyması paylaşımlı kilitle yapılamaz ve okuma yolunu yazıcıya
dönüştürürdü. Zee dışı bir sürecin hedefi silmesiyle aynı anda duran bayat
yedek (ikinci adım ile yedeğin silinmesi arasında ölen yazar) içeriği
diriltebilir; bu dar pencere açık sınırdır.

**Kanıt.** Windows CI teşhis dalında 600 öldürme turu: hedefin yok kaldığı
turlarda okuma ve ekleme eski bütünü korudu, kurtarılamayan 0. Regression
`coken-degistirme-kurtarma` (`native-coken-degistirme`, Windows; düzeltmesiz
`okunamadı` ve dosya `b-1`e iner), `yarim_degistirme_eski_butunu_geri_koyar_ve_izleri_siler`,
`hedefe_ait_yedekler_en_yeni_once_ve_gecici_esiyle_bulunur`; K-346 çok süreç
regresyonu Windows'ta 25 tekrar. GB-139.

## Ek — K-351: adım bağlamı ve silinemeyen yedek (19 Kasım 2026)

**Bağlam.** Windows CI'da `iki_yazar_satir_kaybetmez` bir kez düştü: IO test
grubu olağan ~2 sn yerine 16,4 sn sürdü (paylaşımlı CI disk durması, ADR-075 /
K-217 sınıfı); yazar a 5 sn kilit bütçesinde `TimedOut`, yazar b
`atomik_satir_yaz`'da `NotFound (os 2)` aldı. 600 turluk yük altı A/B
yeniden üretmedi. Hata yalnız sistem kodunu taşıdığı için `ReplaceFileW`,
ilk yerleştirme `MoveFileExW`, yedek silme ya da kurtarmadan hangisinden
geldiği bilinemedi. İnceleme ayrıca başarılı `ReplaceFileW` sonrası
`return remove_file(yedek)` satırını buldu: hedef yeni bütün olduğu hâlde
yedek silinemezse yazım hata olarak bildiriliyordu. Çağıran (ör. `eylem`)
yazılmış veriyi yazılmamış sanıp yeniden deneyebilir ya da yanlış geri
alma kararı verebilirdi; yedek de bu durumda kalıp K-348'in açık sınırındaki
diriltme penceresini besliyordu.

**Karar.**

1. Kalıcı yazım zincirinin her adımı hatasını adla sarar (`kalici_dosya::adim`);
   `io::ErrorKind` korunur, özgün hata `source()` ve iletide kalır. İlk
   yerleştirme `MoveFileExW` hatası önceki `ReplaceFileW` kodunu da yazar.
   Ham `raw_os_error` artık dış hatada değil iç kaynaktadır; zee içinde
   yazım hatasının ham koduna bakan çağıran yoktur.
2. Başarılı `ReplaceFileW` sonrası yedek silme en iyi çabadır; hatası yazımı
   düşürmez.
3. Windows'ta yazma kilidini alan işlem hedef varken duran yedekleri siler
   (hedef varken yedek kesin olarak başarılı yazımın artığıdır; `ReplaceFileW`
   hedefi yalnız iki adımının arasında yok bırakır). Bu, her Windows kilit
   alımında hedef klasörüne bir dizin listesi ekler; maliyet `fsync`'li
   değiştirmenin yanında küçüktür ve yalnız Windows'tadır. K-348'in açık
   sınırı daralır: artık yedek ancak sonraki zee işlemine kadar yaşar.
4. `ReplaceFileW` 2/3 (bulunamadı) ile düştüğünde eski kod yedeği koşulsuz
   siliyor ve `MoveFileExW(REPLACE_EXISTING)` ile ilk yerleştirmeye
   geçiyordu. `ReplaceFileW` eski hedefi yedeğe taşıdıktan sonra düştüyse
   eski bütün siliniyor ve geçici metadata merge'siz yayımlanıyordu; hedef
   varken gelen 2/3'te de hedef merge'siz eziliyordu. Artık yedek varsa
   (1177 gibi) eski bütün yedekten geri kurulur ve hata döner; ilk
   yerleştirme yalnız hedef ve yedek yokken, `REPLACE_EXISTING` olmadan
   yapılır. CI'daki `NotFound`un bu yoldan gelmiş olabileceği dışlanamadı.
5. Test derlemesinde aynı adım noktaları iş parçacığına özel hata/gecikme
   enjeksiyonu kabul eder; üretim ikilisinde enjeksiyon yüzeyi yoktur.

**Reddedilen.** Süreç içi "artık yedek var" bayrağı ya da listesi: başka
süreçte kalan artığı görmez ve uzun ömürlü durum ekler. Yedek silmede
yeniden deneme döngüsü: dış süreç tutamacı ne zaman bırakacağını bilmez,
yazımı gereksiz bekletir. Üretimde ortam değişkeniyle hata enjeksiyonu:
güvenlik yüzeyi açar.

**Kanıt.** `kalici_dosya::adim::tests` (adım bağlamı ve tür korunur; her
adımın enjekte hatası eski bütünü bozmaz; disk durması yeniden üretimi:
değiştirmede duran yazar bekleyeni `TimedOut` + `yazma kilidi` adımıyla düşürür,
kendi satırını kaybetmez; Windows'ta yedek silinemezse yazım başarılı ve
artık sonraki yazımda temizlenir; hedef varken `ReplaceFileW` 2 ilk
yerleştirmeye sapmaz; kısmi replace sonrası 2 eski bütünü yedekten geri
kurar), `yarim_degistirme_eski_butunu_geri_koyar_ve_izleri_siler`
artık yedek dalı, regression `yedek-artigi-diriltmez`
(`native-yedek-artigi`, Windows). `NotFound` kaynağı yeniden üretilemedi;
sonraki tekrarda ileti adımı gösterecek. GB-140.

## Platform kaynakları

- [Linux xattr(7)](https://man7.org/linux/man-pages/man7/xattr.7.html)
- [Apple copyfile(3)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/copyfile.3.html)
- [Microsoft ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew)
