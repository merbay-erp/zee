# ADR-034 — Kalıcı ortak web deposu ve tek-worker süreç modeli

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-137, B-046, RFC-0017, ADR-018, V1-P0-03

## Bağlam

ADR-018 process içi oturumları 4096 toplam/1024 anonim kayıtla sınırladı;
K-134 oturum ve çerez mutation'ını HTTP yanıt transaction'ına aldı. Buna
karşın process A'da açılan oturum process B'de görünmüyor, restart'ta
kayboluyor ve revoke worker'lar arasında yayılmıyordu. Endpoint, CSRF ve
özellikle CPU-pahalı Argon2id doğrulaması ortak atomik oran sınırı taşımıyordu.

Stage 0 HTTP sunucusu bir bağlantıyı kabul edip request'i okur, programı
çalıştırır, yanıtı tamamlar ve ancak sonra yeni bağlantı kabul eder. Bu
gerçeklik gizlenerek thread-pool sözü verilemez.

## Karar

Oturum runtime'ı saklama ayrıntısından `Depo` sınırıyla ayrılır. Sınır;
oturumu okuma, istek sonundaki ekle/sil mutation'ını atomik uygulama/geri alma,
süre dolumunu temizleme ve rate-limit sayacını artırma işlemlerini sunar.
Runtime doğrudan `HashMap` bilmez.

İki adaptör vardır:

1. Deneysel/öğretici web için süreç içinde kilitli bellek deposu.
2. `--web-proxy` production profili için proje kökündeki
   `.zee/web-durumu-v1.json` kalıcı ortak deposu.

Kalıcı adaptör her değişikliği süreçler arası dosya kilidi, eski byte
karşılaştırması ve atomik replace ile yayımlar. Yarışta güncel durum yeniden
okunur; 128 ardışık çakışma sonrası istek fail-closed kalır. Bozuk JSON,
bilinmeyen biçim sürümü, symlink, kayıt/alan sınırı veya IO hatası 503'tür.
Unix'te `.zee` 0700, durum dosyası 0600'dür. Oturum belirtecinin yalnız
SHA-256 özeti saklanır.

Oturum mutation'ı yanıt öncesi ortak depoya commit edilir. Socket yazımı
başarısızsa yalnız bu commit'in değiştirdiği kayıtlar beklenen yeni değerleri
taşıyorsa atomik geri alınır; başka süreçlerin ilgisiz kayıtları ezilmez.

Production oran profili kanonik istemci kimliğiyle şu sabit pencereleri
uygular:

- yöntem + sorgusuz path uç noktası: 60 saniyede 100 istek;
- CSRF doğrulaması: 60 saniyede 60 deneme;
- Argon2id parola doğrulaması: 300 saniyede 5 deneme.

Sayaç artışı ve pencere sonu aynı ortak CAS transaction'ındadır. Altıncı
parola doğrulaması Argon2id çalıştırılmadan 429 olur. En çok 32768 canlı oran
anahtarı tutulur; süresi dolanlar temizlenir, yalnız canlı kayıtlarla doluluk
yeni kimliği tahliye ederek eşiği delmek yerine fail-closed reddeder.

`--web-proxy` yalnız loopback proxy'ye güvenir. Proxy, istemciden gelen bütün
proxy başlıklarını silip tam bir `Forwarded: for=<IP>;proto=https;host=<host>`
başlığı kurar. Runtime tek header/tek halka ister, IP'yi `IpAddr` ile kanonikler
ve `X-Forwarded-For`ı kimlik kaynağı saymaz.

V1 server modeli bilinçli olarak **tek worker/thread per process**tir.
Concurrency, reverse proxy arkasında aynı proje kökünü/deposunu paylaşan N
ayrı `dil` süreciyle kurulur. Aynı kaynak dosyası her süreçte farklı loopback
porta `--web-worker-port N` ile bağlanır. Thread-pool veya aynı process içinde
eşzamanlı Zee request yürütümü V1 sözü değildir.

## Reddedilen seçenekler

- **Sticky session:** restart/revoke/rate-limit ortaklığını sağlamaz.
- **İstemci rolü/JWT:** anlık revoke ve anahtar yönetimi karmaşasını istemciye
  taşır.
- **Hemen thread-pool:** Zee runtime'ın gözlenebilir IO ve transaction
  determinizmini büyütür; V1 için N süreç daha dar ve denetlenebilir sınırdır.
- **Zorunlu Redis/harici servis:** bootstrap dağıtımını üçüncü bir servise
  bağlar. Çok-hostlu deployment ileride aynı depo sözleşmesinin ayrı adaptörü
  olabilir; mevcut garanti aynı güvenilir dosya sistemini paylaşan süreçlerdir.

## Sonuçlar

- Login, revoke, expiry ve rate-limit restart ile worker geçişinde korunur.
- Tek bağlantı bir worker'ı en çok mevcut request deadline'ları kadar meşgul
  edebilir; kapasite N process ve reverse-proxy buffering/timeout ayarıyla
  büyütülür.
- Operatör, loopback dışı bind'e izin veremez; `Forwarded` başlığını dış
  istemciden aynen geçiremez.
- Gerçek iki CLI süreci; login, çapraz authenticated GET, restart, çapraz
  logout ve ortak 429 akışını uçtan uca doğrular. On paralel depo kullanıcısı
  100 artışta eşik üstü tek isteğin kaçmadığını ayrıca kanıtlar.
