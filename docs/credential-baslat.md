# Credential dosyasından mevcut Zee CLI'ını başlatma

K-245/P25, gerçek `zee-cekirdek` D3 talebinin dar host araç dilimidir.
`scripts/credential-baslat.py`, credential dosyalarını doğrulayıp yalnız
başlattığı çocuğun ortamına aktarır. Yeni compiler komutu, grammar, export,
proje yetkinliği veya `zee.ayar` alanı eklemez. Ürünün mevcut SMTP, PostgreSQL,
gizli anahtar ve HTTP ortam bağları kullanılır; canlı ürün kabulü yapılmadı.
[Anonim talep/provenance](../dogfood/zee-cekirdek/credential/README.md).

## Açık çağrı ve ortam sınırı

Araç Python 3 standart kitaplığı ve POSIX `dir_fd`, `O_DIRECTORY`,
`O_NOFOLLOW`, `O_NONBLOCK`, `O_CLOEXEC` desteği ister. Hostun güvendiği
Python runtime'ını, scripti ve Zee ikilisini explicit mutlak yollarla seçin:

```sh
/abs/python3 -I -B /abs/scripts/credential-baslat.py \
  --credential-root /run/secrets/zee-private \
  --env UYGULAMA_SMTP_URL=smtp-url \
  --env UYGULAMA_DATABASE_URL=database-url \
  --dil /abs/dil -- çalıştır /abs/proje
```

Launcher verilen ikiliyi imza/pin/build kimliğiyle authenticate etmez;
hostun daha önce seçip doğruladığı trusted executable'ı çağırır. İmzalı
compiler seçiminin sınırı mevcut `derleyici-secici.py` aracında kalır.

systemd için kök yol açıkça `--credential-root "$CREDENTIALS_DIRECTORY"`
olabilir; araç kendiliğinden credential/ev/proje/üst dizin aramaz. systemd
credential'ları servis aktivasyonunda edinir, servis ömründe değişmez tutar
ve `$CREDENTIALS_DIRECTORY` altındaki dosyalardan sunar.
[Birincil systemd sözleşmesi](https://systemd.io/CREDENTIALS/).

En fazla 16 tekil `ENV=BASENAME` eşlemesi kabul edilir. Ortam adı tekil,
en fazla 64 ASCII karakterli `A-Z`, `0-9`, `_` biçimindedir; ilk karakter
rakam değildir. Basename en fazla 64 ASCII karakterdir, alfanumerik veya
`_` ile başlar; kalan karakterler alfanumerik, `_`, `.`, `-` olabilir.
Yol ayıracı, traversal, aynı dosyanın iki eşlemeye bağlanması ve yinelenen
launcher seçenekleri reddedilir.

`PATH`, `HOME`, `CODEX_HOME`, `CREDENTIALS_DIRECTORY`, loader/interpreter,
locale, shell ve OS denetim değişkenleri credential hedefi olamaz. Eşlenmiş
ortam adı caller'da zaten varsa boş değer dahil reddedilir; dosya/env
önceliği veya dosya hatasında ambient fallback yoktur. İlgisiz uygulama
ortamı korunur; çocuk için `LD_*`, `DYLD_*`, `PYTHON*` loader değişkenleri
ayıklanır. Bu işlem daha önce başlatılmış Python runtime'ının güvenilirliğini
kanıtlamaz; runtime ve caller ortamı host işletmecisinin güven sınırıdır.

`--` sonrası mevcut `çalıştır`/`calistir` argv vektörü değiştirilmeden Zee'ye
taşınır. Shell, `eval`, dotenv veya değer genişletmesi yoktur. CLI/config
önceliği Zee'de kalır. Host eşlemesi ana projenin
`http_gizli_değişkenleri`, SMTP, DB, kriptografi veya ağ hedefi izinlerini
açmaz; yetkiyi yine native CLI denetler. `iz` ve diğer compiler komutları bu
dar launcher çağrısının dışında kalır.

## Dosya ve süreç sözleşmesi

Credential kökünün bütün bileşenleri açık üst directory descriptor'ına göre
nofollow açılır. Kök gerçek directory, sahibi effective UID veya root,
POSIX modu `0500`/`0700` olmalıdır. Dosya köke bağlı basename ile read-only,
nofollow/nonblocking/close-on-exec açılır; gerçek regular file, tek hard-link,
sahibi effective UID veya root ve modu `0400`/`0600` olmalıdır. Symlink, FIFO, socket, cihaz, directory,
hard-link ve fazla açık izinler reddedilir. Genel `/run/secrets` mount'ındaki
`0444` dosya bu private-mode profiline uymaz; uygun sahip/izinle hazırlanmış
kök ayrıca seçilmelidir.

Her dosya 1–65.536 bayt, bütün dosyalar toplam en fazla 65.536 bayttır.
Okuma sınırı allocation'dan önce uygulanır. Profil URL, anahtar ve bearer
için tek satır UTF-8 scalar'dır: BOM, NUL, CR/LF ve geçersiz UTF-8 reddedilir.
Otomatik trim yoktur; kalan boşluklar aynen korunur. Credential hazırlarken
sona newline eklemeyin. Multiline PEM/binary credential bu dilimde yoktur.

Stat/open/read/post-stat ve bütün eşlemelerin son kontrolü; device/inode,
mod/UID/link sayısı, boyut, mtime/ctime kimliğini karşılaştırır. Root path
yeniden anchored açılıp aynı kimlikte olmalıdır. Tüm dosyalar doğrulanmadan
hiçbir çocuk başlatılmaz; descriptor'lar child-start'tan önce kapanır.
Araç credential içeriğini, izinini veya kaynak dosyalarını yazmaz; dosya
sistemi read-atime güncellemesi bu ifadenin dışındadır.

Çocuk stdin/stdout/stderr'i `DEVNULL`'a bağlıdır. Bilerek sır basan çocuğun
stream'i de host çıktısına veya JSON'a taşınmaz. Launcher yalnız şu exact
alanları yazar; credential adı/yolu/değeri, argv, exception veya raw stream
yer almaz:

```json
{"schema":"zee-credential-baslat-1","source":"child","exit":0}
```

Host doğrulama/başlatma reddi `source: launcher`, exit 2'dir; stderr boş kalır.
Çocuk başarı/ret exit'i aynen aktarılır; POSIX signal çıkışı `128 + signal`
ile gösterilir. Interrupt 130'dur. Kaynak/çocuk akışını kapatmak, çocuk
uygulamanın kendi dosya/ağ yazımlarına genel taint kontrolü sağlamaz.

Bir çocuk, shell'siz ve yeni session/process group açılmadan beklenir.
Restart, sinyal/grup yaşam döngüsü ve servis işletimi host supervisor'a
aittir; wrapper SIGTERM alan bütün grup adına supervisor davranışı
uydurmaz. Credential'lar launch snapshot'ıdır. Dosya değişimi yaşayan
çocuğun env'sini değiştirmez; yeni değer için süreç yeniden başlatılır.
Runtime DB havuzları/anahtar rotasyonu ve canlı reload bu dilimde yoktur.

## Kanıt ve açık sınırlar

`scripts/tests/credential-baslat-test.py` gerçek POSIX dosya/süreç kullanır:
normal/Unicode argv, secret stdout+stderr bastırma, exit aktarımı,
private-mode kabul/ret, symlink/hard-link/FIFO, exact dosya/toplam/eşleme
sınırı, bozuk text, ambient collision/fallback reddi, open/read/root yarışı,
descriptor cleanup ve restart snapshot'ı. Stat'tan sonra writersız FIFO
değişimi ayrı üç saniyelik hang guard ile sınanır; genel child hang guard
yalnız fixture'ın sahibi olduğu process group'u sonlandırır.

İki vaka gerçek native `dil` ikilisini zorunlu kullanır: ana proje HTTP ortam
allowlist'i açık/kapalı iken sonuç değişir; anonim dosyadan gelen SMTP URL'si
yalnız owned loopback fixture'a bir AUTH ve bir DATA ulaştırır. SMTP olumlu
fixture açık `yerel-ağ` izni taşır. Test ilk hali bu izni atladığında native
guard bağlantıyı reddetti; ürün/compiler bug'ı olarak sınıflandırılmadı.

```sh
ZEE_CREDENTIAL_TEST_DIL=/abs/dil /abs/python3 -I -B scripts/tests/credential-baslat-test.py
```

Bu aday macOS POSIX/native fixture kanıtıdır; exact temiz kaynak kabulü ayrı
final checkpoint'te bağlanır. Windows fail-closed unsupported'dır; DACL,
reparse/handle ve physical Windows kabulü yoktur. Linux/systemd gerçek servis
kurulumu, minimum OS, canlı ürün ve dış provider kabulü ayrıca açıktır.
POSIX mode denetimi extended ACL incelemesi veya aynı UID/root saldırganına
OS sandbox garantisi değildir. NFS/özel dosya sistemi hard wall deadline'ı,
metadata'yı geri yükleyen saldırgan ve çoklu credential'ların atomik tek
generation snapshot'ı vaat edilmez.

File→env aktarımı systemd credential'ın süreç ağacına otomatik miras
bırakılmama avantajını azaltır: değer native çocuğun env/belleğinde yaşar ve
o çocuk alt süreçlerine aktarabilir. Python/env belleği için garantili
zeroization yoktur. Bu nedenle P25 bütünü veya file-native compiler secret
API'si tamamlanmış gösterilemez; bu dilim explicit host bridge ve restart
sözleşmesidir. [Systemd güven sınırı](https://systemd.io/CREDENTIALS/).
