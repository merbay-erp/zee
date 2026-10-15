# Kopyalanabilir site ve panel başlangıcı

Bunlar mevcut Zee dilinde yerel pilot başlangıçlarıdır. [site](site/ana.dil)
üç salt-okunur sayfa; [panel](panel/ana.dil) parola doğrulama, rol, not kaydı,
CSRF ve çıkış akışını içerir. Veritabanı, canlı kullanıcı, HTTPS kurulumu veya
production dağıtımı içermez. Kaynak `projeler/mini-site.dil` ve
`projeler/girisli-panel.dil` örneklerinin proje bildirimli kopyasıdır.

Depo kökünden, yeni bir hedef seçerek:

```sh
cp -R sablonlar/site /abs/yeni-site
# Panel için bunun yerine sablonlar/panel kopyalanır.
dil kilitle /abs/yeni-site
dil denetle /abs/yeni-site
dil dene /abs/yeni-site
dil çalıştır --deneysel-web /abs/yeni-site
```

Site `http://127.0.0.1:8090`, panel `http://127.0.0.1:8092` adresindedir.
`--deneysel-web` yerel deneme içindir. Ürünleştirme için mevcut
[web profili](../docs/web-production-profili.md) ve
[owned host](../docs/worker-supervisor.md) rehberleri kullanılır.

## Panel parolasını kur

Panel kopyasının kökünde Bash ile bir kez çalıştır:

```bash
(
  set -euo pipefail
  set -C
  umask 077
  dil parola-özeti | tr -d '\n' > panel-phc
)
```

Mevcut CLI parolayı terminalde göstermeden iki kez sorar. Dosyada parola
metni değil Argon2id PHC özeti bulunur; son newline kaldırılır. `set -C`
var olan dosyanın üzerine yazmaz, `umask` yeni dosyayı 0600 yapar. CLI hata
verirse komut başarısızdır; kalan boş dosya giriş açmaz. Parola değişimi
özel dosyanın bilinçli yenilenmesidir; bu başlangıç otomatik parola kurtarma
veya kullanıcı yönetimi değildir.

Panel her giriş isteğinde kendi proje kökündeki `panel-phc` dosyasını okur.
Eksik/bozuk özet giriş vermez; sabit demo parolası yoktur. Dosya yalnız
operatörün eriştiği yerel credential verisidir; source kontrolüne veya
paylaşılan IO/debug çıktısına konmaz. `.gitignore` özet ve not dosyasını
hariç tutar. 0600, hazırlık adımının sonucudur; normal dosya
okuması credential launcher’ın özel nofollow/tek-link/izin ve secret IO
redaction protokolüne eşdeğer değildir. Bu dosya profili launcher’ın dosyadan env aktarımı değildir;
şablon ilgisiz outbound ağ hedefi veya ortam okuma yetkisi açmaz. Notlar aynı
kopyanın `notlar.txt` dosyasındadır; iki süreçte eşzamanlı not güncellemesi,
yedekleme ve veri göçü bu başlangıcın kabul kapsamı değildir.

`proje.kilit` bağımlılık/kaynak kilididir; compiler sürümü seçmez. Exact
compiler seçimi için [seçici](../docs/derleyici-secici.md) kullanılır.
Kopyadan sonra kaynak değişince `dil kilitle` bilinçli olarak yenilenir.
Gerçek yeni kullanıcı oturumu ve fiziksel Windows/minOS kurulumu ayrıca
bekleyen ürünleşme kanıtıdır; native test bu insan/platform kapılarını kapatmaz.
