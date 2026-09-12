# Deterministik IO izi rehberi

IO izi, gerçek bir Zee koşusundaki dış dünya konuşmasını kaydedip aynı programı
sonradan dosya, ağ, klavye, saat veya sensöre dokunmadan yeniden oynatır. K-115,
RFC-0022 ve ADR-026'nın kullanıcı yüzeyidir.

## Kayıt

```sh
cd compiler
cargo run -- iz kaydet ../kosu.zee-io-izi ../program.dil argümanlar...
```

Program normal çalışır; ekran ve istemler canlıdır. Koşu sonunda iz atomik
yazılır. Program argümanları da IO olayı olarak kayda girer.

## Yeniden oynatma

```sh
cargo run -- iz oynat ../kosu.zee-io-izi ../program.dil
```

Replay dış dünyaya erişmez. Program aynı IO işlemlerini aynı sırada ve aynı
argümanlarla isterse kayıtlı sonuçları alır. Kaynak değişikliği farklı URL,
dosya yolu, rastgele aralık, çıktı veya etki üretirse koşu “IO replay
uyuşmazlığı” ile durur. Replay'e ayrıca program argümanı verilmez; izden gelir.

Çıktı, bütün izin eksiksiz eşleştiği doğrulanmadan basılmaz. Böylece yarım
uyuşan bir koşu başarılı görüntüsü vermez.

## İz ne yapmaz?

- Kaynak kodu taşımaz; aynı program/proje ayrıca verilmelidir.
- Dosya yazma, HTTP, çerez, oturum, transaction veya donanım etkisini yeniden
  uygulamaz; yalnız aynı çağrının yapıldığını doğrular.
- Yedekleme, güvenlik yetkisi veya production audit logu değildir.
- Saat/rastgele algoritmasını yeniden çalıştırmaz; kayıtlı sonuçları kullanır.
  Etkin algoritma ve sanal saat [deterministik IO profilinde](deterministik-io-profili.md)
  `zee-io-1` adıyla ayrıca sürümlüdür.
- K-134 web request commit/rollback kancalarını ayrı olay yapmaz; bunlar
  kaydedilmiş yanıt/oturum çağrılarını hostta görünür kılan iç sahiplik
  sınırıdır ve replay sırasında dış etki uygulamaz.

## Gizlilik

`*.zee-io-izi` dosyaları Git tarafından varsayılan olarak yok sayılır. Buna
rağmen iz; yazdığınız cevapları, dosya/ağ gövdelerini, komut argümanlarını,
çerezleri veya tokenları taşıyabilir. Hex alanlar şifreli değildir. İzleri
özel veri gibi saklayın ve paylaşmadan önce inceleyin.

Parola doğrulamasında parola ve PHC özeti ham ya da yalnız hexlenmiş tutulmaz;
SHA-256 parmak iziyle eşleşme doğrulanır. Bu koruma izin geri kalanını anonim
yapmaz.

## Format ve sınırlar

Şema-1 başlığı `zee-io-izi<TAB>1`dir. Her olay 1 tabanlı kesintisiz sıra,
işlem, argüman sayısı, sonuç sayısı ve küçük harfli hex UTF-8 alanları taşır.
64 MiB, 100.000 olay ve olay başına 4.096 alan sınırı vardır. Bilinmeyen
sürüm/işlem, bozuk hex, sıra boşluğu veya işlem-özel şema hatası program
yürütülmeden reddedilir.

Kütüphane embedding'i için `IzKaydedenIo<T>` ve `IzYenidenOynatici` public
API'dir. Kayıt sarmalayıcısını politika/güvenlik adaptörünün en dışına koyun;
böylece programın gerçekten gördüğü sonuç izlenir.

### Özel dosya yazımı (K-184)

CLI iz kaydı Unix'te geçici dosyayı `0600` ile açar; macOS'ta `openx_np` ile boş ve mirassız ACL'yi
ilk açılışta uygular. Windows'ta dosya ilk oluşturulurken
miras kapalı, yalnız nesne sahibine tam erişim veren DACL kullanılır.
Atomik değiştirme eski hedefin geniş izinlerini veya ACL'sini yeni ize taşımaz.
Symlink ve normal dosya olmayan hedefler reddedilir. Genel dosya yazımının
metadata koruma sözleşmesi değişmez. Erişim kısıtlaması sağlanamıyorsa iz
kaydedilmez; bu durum kayıttan önce çalışmış programın etkilerini geri almaz.

Bu koruma şifreleme veya anonimleştirme değildir. Mevcut eski izler kendiliğinden
taranmaz; yeniden kaydedilen hedef daraltılır. Başka kullanıcının önceden
aldığı kopyalar veya açık dosya tanıtıcıları geri alınamaz. Güvenilir bir
kullanıcı dizini kullanın; root/Administrator erişimi kapsam dışıdır.

Paylaşım için gerçek sırlar yerine sentetik veriyle yeniden kayıt üretin.
Replay, alanların birebir eşleşmesine dayandığından genel amaçlı maskeleme
bu dosya biçiminde otomatik uygulanmaz. İzleri CI loglarına veya artefaktlarına
koymayın; `iz_metni()` kullanan gömme uygulaması saklama politikasını kendisi uygular.
