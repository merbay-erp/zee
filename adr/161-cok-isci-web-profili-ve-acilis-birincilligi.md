# ADR-161 — Resmî çok işçi web profili ve açılış birincilliği

- **Durum:** kabul
- **Tarih:** 18 Kasım 2026
- **İlgili kayıt:** K-336, K-337, RFC-0038, ADR-034 (production web
  profili), ADR-059 (core freeze), ADR-064 (dil yüzeyi), ADR-087 (açık depo),
  ADR-118 (native listener), ADR-157 (istek-yerel hata sınırı), spec/12 §7,
  spec/21, GB-133

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
- Güvenlik sınırı GB-133: devralınan descriptor doğrulanır, ortam silinir,
  sağlık uç noktası loopback ve PID listesi yalnız yerel operatöre.

## Açık

Windows dinleyici devri; dengeli dağılım (macOS); çok hostlu
grup; elle kurulan ayrı portlu süreçlerde birincillik (her biri kendini birincil
görür).

## Ek — K-338: yeniden başlatma sınırı ve sağlık uç noktası kanıtı (19 Kasım 2026)

Açık kalan iki spec maddesi (art arda çökme durdurması, 40 sn zorla
sonlandırma; `/saglik` 503 ara durumları, HEAD ve 404) gerçek yönetici
sürecinde sınandı. Sınama bir sapma buldu: yuva sayacı `ardisik >
AZAMI_ARDISIK_COKUS` ile karşılaştırıldığından grup 10 değil 11. hızlı
çıkışta duruyor, günlük yine "art arda 10 kez" yazıyordu. Sözleşme spec/12'deki
"art arda 10 kez çıkan yuva grubu durdurur" metnidir; kod ona çekildi
(`>=`, günlük gerçek sayıyı yazar). Davranış yalnız sürekli çöken bir yuvanın
grubu bir yeniden başlatma (≈5 sn) erken durdurmasıdır; sağlıklı grupta fark
yoktur.

- Kanıt: `web_cok_surec_testi::isci_grubu_art_arda_hizli_cokuste_durur_ve_saglik_ara_durumlarini_bildirir`
  (ikincil `programı 3 ile bitir`: yuva tam 10 kez başlar, 9 yeniden başlatma,
  çıkış 1; birincil açılışı sürerken 503 `basliyor`, ikincil düşükken 503
  `eksik`, HEAD gövdesiz, `/baska`, `POST /saglik`, `/saglik/` 404) ve
  `…::isci_grubu_kapanista_bitmeyen_isciyi_40_sn_sonra_sonlandirir`
  (SIGSTOP ile dondurulan işçi SIGTERM'i işleyemez; diğer işçi hemen çıkar,
  sağlık 503 `kapaniyor`, yönetici ≥38 sn sonra donmuş işçiyi sonlandırıp
  çıkış 1 verir). Regression `cli/isci-grubu-ardisik-cokus`.
- Dil yüzeyi, grammar, tanı ve `dil::api::v1` değişmedi; güvenlik sınırı
  değil (GB satırı yok): yönetici yalnız erken durur.

## Ek — K-340: `zee.ayar` işçi grubu alanları (19 Kasım 2026)

zee-cekirdek (exact `abaca3aba0e90f57d26435c4befffa903346c425`) üretim web
ayarlarını K-234 talebiyle sürümlü `zee.ayar`'a taşıma yolundadır; işçi
sayısı ve sağlık portu yalnız CLI'da kaldığı için tek komutluk gruba geçen
ürün systemd `ExecStart` satırına yeniden bayrak yazmak zorundaydı
(`dogfood/zee-cekirdek/zee-ayar-isci`). Karar: ADR-097 `guvenlik` dalına
`isci-sayisi` (1..64) ve `saglik-port` (1..65535) eklenir; eşleme mevcut
`--web-isci-sayisi`/`--web-saglik-port` bayraklarıdır, kaynak öncesi açık
bayrak config'i ezer. `isci-sayisi` aynı config'te `proxy` + `worker-port`
ister (CLI portu bozuk config'i geçerli kılamaz, ADR-097 ilkesi).

- Güvenlik dalı seçildi: alan dinleme yüzeyini ve süreç sayısını değiştirir;
  olağan `ayarlar` dalı yalnız atlanabilir alanlar içindir. Eski okuyucu
  bilinmeyen güvenlik alanında çıkış 2 verir; sessizce tek süreçle açılmaz.
- İşçi argv'si config'ten bayrak almaz ama işçi aynı kökü açtığında
  `zee.ayar`'ı yeniden okur. `hazirla` işçi ortamı gördüğünde işçi
  bayraklarını ayıklayıp yok sayar; aksi hâlde işçi kendi grubunu açmaya
  çalışıp port çakışmasıyla çıkıyordu (test bunu kanıtlar).
- Reddedilen: `ayarlar` (olağan) dalı — güvenlik etkili alan sessizce
  atlanabilirdi; ayrı `--ayar` dosyası — K-234'te zaten tek dosya var;
  `proje.dil` alanı — dil yüzeyi/grammar sözleşmesi, CLI adaptörü yeterli.
- Kanıt: `ayar.rs::tests::isci_grubu_alanlari_eslenir_ve_bagimliliklari_zorunludur`,
  `cli_ayar_testi::guvenlik_tur_aralik_ve_ic_bagimlilikler_cli_override_ile_asillamaz`,
  `web_cok_surec_testi::isci_grubu_zee_ayar_ile_baslar_ve_cli_ezer` (config
  ile 3 işçi, yeniden başlatma 0; CLI `--web-isci-sayisi 2` ezer).
- Dil yüzeyi: `profil zee-ayar.isci-sayisi`, `zee-ayar.saglik-port`
  (ADR-064). Grammar, tanı kodu ve `dil::api::v1` değişmedi; yeni güvenlik
  sınırı yok (mevcut GB-133 descriptor/ortam kuralları aynı).
