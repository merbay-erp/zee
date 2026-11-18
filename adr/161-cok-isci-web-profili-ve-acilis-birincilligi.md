# ADR-161 — Resmî çok işçi web profili ve açılış birincilliği

- **Durum:** kabul
- **Tarih:** 18 Kasım 2026
- **İlgili kayıt:** K-336, K-337, RFC-0038, ADR-034 (production web
  profili), ADR-059 (core freeze), ADR-064 (dil yüzeyi), ADR-087 (açık depo),
  ADR-118 (native listener), ADR-157 (istek-yerel hata sınırı), spec/12 §7,
  spec/21, GB-132

## Bağlam

zee-cekirdek (exact ürün commit'i `ca3870d854e4567c47bb6748fe8573223c885798`)
talebi W2 (P1) ve W4 (P2). Ürün ölçümü: tek işçi, istek başına ~250 ms,
eşzamanlılık 1/3/10 → saniyede 4,03/4,02/4,01 istek, medyan 253/744/2.485 ms;
aynı kökten üç ayrı portlu süreç → saniyede 12,0 istek. Darboğaz süreç içi
sıralılık; kalıcı web deposu (`.zee/web-durumu-v1.json`, CAS) ve PG havuzu
süreçler arasında güvenli. Ürün açılışı (`cekirdek/kaynak/ana.dil`: yarım
medya/dosya uzlaştırması, `.zee/yuklemeler` sahipsiz dosya arındırması,
taşınan hesap aktarımı) her süreçte koşar; tek işçi yeniden başlarken diğerinin
yüklediği geçici dosyayı silebilir.

Core freeze (ADR-059) yeni yüzeyi varsayılan olarak reddeder. İhtiyaç gerçek
ürün dosyasında duruyor ve mevcut yüzeyle karşılanamıyor: süreç, grubun
birincili olduğunu bilemez; elle üç süreç açmak açılış sırası vermez.

## Karar

1. **W2 — süreçler arası, süreç içi değil.** `dil çalıştır --web-proxy … --web-worker-port P
   --web-isci-sayisi N` (1..64) yönetici süreç açar. Yönetici `127.0.0.1:P`'yi
   bir kez bağlar, `dup(2)` ile FD_CLOEXEC taşımayan tek kopyayı N işçiye
   miras bırakır (pre-fork); işçi dinleyiciyi `getsockname`/LISTEN/TCP
   doğrulamasıyla sahiplenir (ADR-118 doğrulayıcıları), hemen FD_CLOEXEC yapar.
   Rol ve descriptor çocuğa yalnız dört `ZEE_WEB_ISCI_*` ortam kaydıyla verilir;
   işçi bunları ilk iş okur ve ortamdan siler. Yönetici Zee yürütmez.
2. **Davranış eşliği.** Her işçi §5 tek süreç çalışma zamanıdır; oturum, CSRF,
   revoke ve varsayılan ortak oran deposu tek süreçle aynıdır (conformance
   testi). Explicit bellek oran deposu işçi başınadır; yönetici uyarır.
3. **W4 — birincillik süreç rolüdür, kilit değildir.** `birincil süreç`
   (intrinsic `web.birincil_surec`, `ağ-sunucusu`, Mantıksal): tek süreçte
   doğru; grupta yalnız ilk açılan birincil işçide doğru. Yönetici birincilin
   hazır kanalına (`sunucu başlat` sonrası tek bayt) kadar ikincilleri başlatmaz;
   yeniden başlatılan işçi her zaman ikincildir. Açılış bloğu bu yüzden ilk
   açılışta istek kabulünden önce ve yalnız bir kez koşar.
4. **İşletim.** Çıkan işçi ikincil olarak 250 ms→5 sn katlanan gecikmeyle döner;
   10 sn içinde art arda 10 çıkış grubu durdurur. SIGTERM/SIGINT işçilere
   SIGTERM olarak iletilir (kabul edilmiş istek tamamlanır, 40 sn sonra zorla).
   Yönetici kaybolursa işçiler `getppid` değişimini boşta yoklamada görüp yeni
   istek almadan çıkar. `--web-saglik-port` yalnız loopback `GET|HEAD /saglik`.
5. **Platform.** Yalnız Unix. Windows'ta `--web-isci-sayisi` çıkış 2 ile
   reddedilir; ayrı `--web-worker-port` süreçleri + proxy yolu sürer.

## Değerlendirilen seçenekler

- **(a) Süreç içi N istek işçisi.** Reddedildi: yorumlayıcı `Rc`/`RefCell`
  değerleri, tek `GirdiCikti` adaptörü, istek başı eylem/PG transaction'ı ve
  web oturum taslağı süreç başına tektir; `Send` olmayan durum iş parçacıkları
  arasında paylaşılamaz. Güvenli paralellik tüm çalışma zamanının yeniden
  yazımı ve yeni veri yarışı sınıfı demektir; core freeze ve risk/fayda bunu
  taşımaz. Süreç ayrımı bellek güvenliği ve istek yalıtımını bedavaya verir;
  ölçüm doğrusal ölçeği zaten gösterdi.
- **SO_REUSEPORT.** Reddedildi: macOS'ta yük dağıtmaz, Windows'ta yok; aynı
  kullanıcının yabancı süreci aynı porta bağlanıp bağlantı çalabilir.
- **N dinleyici + yönetici içi aktarım (relay).** Reddedildi: ek kopya, ek
  gecikme ve yöneticide HTTP ayrıştırması; ön proxy zaten var.
- **Açılış kilidi (dosya ya da PG danışma kilidi).** Reddedildi: eşzamanlı
  açılışı sıraya koyar ama yeniden başlayan tek işçinin uzlaştırmayı çalışan
  işçilerin ortasında yeniden koşmasını engellemez (W4 kanıtındaki asıl risk).
- **`açılışta bir kez` bloğu ya da `birincil süreç ise` koşul biçimi.**
  Reddedildi: yeni grammar/koşul yüklemi; değişkene bağlanan sıfır argümanlı
  ifade (`istemci adresi` K-209 kalıbıyla aynı yol) yeterli.
- **Ortam değişkeni okuma (`gizli ortam değerini okumayı dene`).** Reddedildi:
  gizli/izsiz yüzeyi rol bilgisi için kötüye kullanır, projeye adı yazdırır.

## Sonuçlar

- Ölçüm (gözlemsel, ADR-045/049 tabanı değil): exact `9e51868` temiz release,
  4 işçi, ~100 ms CPU rotası, eşzamanlılık 4 / tek istek tur medyanı 3,72
  (5 tur: 4,04–2,76; [kanıt](../docs/incelemeler/k336/README.md)).
- Yeni kalıp kelimeleri `birincil`, `süreç`; API `web.birincil_surec`,
  `io-izi.birincil_surec`; CLI profilleri `cli-web-isci-sayisi`,
  `cli-web-saglik-port` (ADR-064). Tanı kodu eklenmedi: CLI reddi çıkış 2 metni.
- Derleyici/LSP önbelleği ya da büyüyen tampon yok; yönetici sabit boyutlu
  yuva dizisi tutar, soak kapısı (ADR-069) gerektirmez.
- Güvenlik sınırı GB-132: devralınan descriptor doğrulanır, ortam silinir,
  sağlık uç noktası loopback ve PID listesi yalnız yerel operatöre.

## Açık

Windows dinleyici devri; dengeli dağılım (macOS); `--ayar` alanı; çok hostlu
grup; elle kurulan ayrı portlu süreçlerde birincillik (her biri kendini birincil
görür). Art arda çökme sınırı ve 40 sn zorla sonlandırma ayrı vaka ile
sınanmadı (spec madde kanıtı `kismi`).
