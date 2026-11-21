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
grup; ağ dosya sisteminde süreç kilidi. Elle kurulan ayrı portlu süreçlerde
tek makinede birincillik K-341 ekiyle kapandı.

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

## Ek — K-341: ayrı portlu süreçlerde birincillik (19 Kasım 2026)

K-337 birincilliği yalnız resmî grupta yöneticinin verdiği role bağladı.
Windows'ta grup yoktur ve ürün ölçeği Unix'te de elle açılan ayrı
`--web-worker-port` süreçleriyle alınabilir; bu kurulumda rol kaydı
olmadığından her süreç `birincil süreç`i `doğru` görüyordu. Sonuç W4'teki
asıl risktir: ölen süreç yerine açılan süreç açılış uzlaştırmasını ve sahipsiz
geçici dosya taramasını yaşayan süreçlerin yüklemeleri ortasında yeniden koşar;
eşzamanlı açılan iki süreç de açılışı paralel koşar. Gerçek süreç testi bunu
gösterdi (iki açılış başladı/bitti, ikincil kaydı yok). GB-136.

Karar: `--web-worker-port` süreci `birincil süreç`i ilk değerlendirdiğinde
dosya sınırı kökünde iki çalışma zamanı kilidi kullanır.
`.zee/birincil-acilis.kilit` dışlayıcı alınır (tutuluyorsa beklenir) ve kararı
sıraya koyar. Kilit altında `.zee/birincil-canli.kilit` dışlayıcı olarak
denenir: başarılıysa kökte canlı katılımcı yoktur. Her süreç sonra canlı
kilidi ömrü boyunca paylaşımlı tutar. Grup rolü ikincilse sonuç yine
`yanlış`tır. Birincil açılış kilidini `sunucu başlat`a (hazır kanalıyla aynı
an) dek tutar, ikincil hemen bırakır. Değer `sunucu başlat` sonrasında ilk kez
soruluyorsa açılış kilidi tutulmaz. Kilit Rust `File::lock` ailesidir (Unix
`flock`, Windows `LockFileEx`): süreç ölünce çekirdek bırakır, bayat kilit
dosyası anlamsızdır. Kilit dosyaları 0600, Unix'te `O_NOFOLLOW` ile açılır;
normal dosya olmayan yol reddedilir. Kurulamayan kilit stderr'e yazılır ve
süreç `yanlış` görür; güvenli yön açılışı atlamaktır.

K-337 ile uyum: K-337 "açılış kilidi"ni, eşzamanlı açılışı sıraya koyduğu
ama yeniden başlayan tek işçinin uzlaştırmayı çalışan işçilerin ortasında
koşmasını engellemediği için reddetti. K-341'in belirleyici kilidi açılış
kilidi değil, canlılık kilidi olan `birincil-canli.kilit`tir. Yaşayan her
katılımcı onu paylaşımlı tuttuğundan yeniden başlayan süreç `yanlış` görür;
grubun "yeniden başlatılan işçi ikincildir" kuralının ayrı portlardaki
karşılığıdır. Açılış kilidi yalnız sıralama içindir: "yalnızım" sınaması ile
paylaşımlı kayıt arasına başka süreç girmez ve ikinci süreç birincilin açılışı
bitmeden karar vermez. Gramer reddi aynen geçerlidir: yeni sözdizimi, koşul
biçimi ya da API yoktur. Mevcut `birincil süreç` ifadesinin değeri doğru
hesaplanır. Grupta sonuç değişmez: ilk birincil işçi kökte yalnızdır,
ikinciller ve yeniden başlatılanlar rol gereği `yanlış` görür.

- Reddedilen: süreç kimliği (PID) dosyası. Bayat PID yeniden kullanılabilir,
  yaşam denetimi platforma özgüdür. PG danışma kilidi: veritabanı
  kullanmayan projede yoktur, bağlantı kopunca sessizce düşer. Yalnız
  açılış kilidi: K-337 gerekçesiyle yetersizdir.
- Sınır: yalnız `birincil süreç`i değerlendiren süreçler katılımcıdır. Kilit
  tek makinede geçerlidir; NFS/SMB ve çok hostlu kurulum kapsam dışıdır.
  Birincilin açılışı sürdükçe ikinciller bekler (sağlık/zaman aşımı ürünün
  süreç yöneticisindedir).
- Kanıt: `web_cok_surec_testi::ayri_portlu_surecler_tek_birincil_secer_ve_acilisi_bekler`
  (Unix ve Windows CI; birincilin açılışı sırasında açılan süreç bekler ve
  ikincil görür, birincil öldürülüp diğeri yaşarken aynı kapıda açılan süreç
  ikincildir, kök boşalınca yeni süreç birincildir; düzeltmesiz kaynakta iki
  açılış koşar), `web_isci_grubu::tests::birincillik_kilidi_*`, regression
  `cli/ayri-port-birincillik`.
- Dil yüzeyi, grammar, tanı kodu ve `dil::api::v1` değişmedi. Yeni çalışma
  zamanı durum dosyaları `.zee/birincil-*.kilit` içerik taşımaz. Derleyici/LSP
  önbelleği yok; soak gerekmez.

## Ek — K-345: sağlık bağlantısının işçiye mirası (19 Kasım 2026)

K-344 yük sınamasında macOS'ta sağlık isteği yanıtı tam alıp EOF'u
beklerken asılı kalıyordu. `lsof` gösterdi ki bağlantının sunucu ucunu
yönetici değil bir işçi PID'i tutuyordu. Neden: sağlık uç noktası ayrı iş
parçacığında `accept` yapar; accept4 olmayan platformlarda std
`TcpListener::accept` FD_CLOEXEC'i ayrı bir adımda koyar. Aradaki anda ana
iş parçacığının `posix_spawn` ile başlattığı işçi soketi miras alır;
yönetici kapatınca FIN gitmez, istemci işçi ölene dek bekler ve işçi
yöneticiye ait bir soketi taşır (GB-138). Linux'ta `accept4(SOCK_CLOEXEC)`
atomiktir; hata orada oluşmaz.

Yöneticinin fd üretim noktaları tarandı: ortak dinleyici, `dup` kopyası
(bilerek miras), sağlık dinleyicisi ve hazır borusu ana iş parçacığında,
işçi başlatmadan önce ya da başlatmayla aynı iş parçacığında açılır;
std `Command` Linux'ta `pipe2(O_CLOEXEC)` kullanır, macOS'ta ek fd açmaz.
Başka iş parçacığında fd üreten tek yer sağlık kabulüdür.

Karar: (1) sağlık dinleyicisi tıkanmasızdır; bağlantı `poll` ile hazır
olunca kabul ve CLOEXEC bir okuma kilidi altında yapılır, işçi başlatma
aynı kilidin yazma tarafını tutar (platformdan bağımsız sıralama).
(2) Yanıt yazıldıktan sonra bağlantı `shutdown(Both)` ile kapatılır; kopya
herhangi bir yoldan kalsa da FIN hemen gider.

- Reddedilen: `POSIX_SPAWN_CLOEXEC_DEFAULT` ile macOS'a özel spawn. Yalnız
  açıkça devredilen fd'leri geçirmesi en geniş korumadır, ama std
  `process::Child` bir `posix_spawn` PID'inden kurulamaz. Yuva defteri
  (`try_wait`/`kill`/`wait`) platforma özgü güvensiz kodla yeniden yazılmak
  zorunda kalırdı. Tarama, tek eşzamanlı üreticinin sağlık kabulü olduğunu
  gösterdi. Yöneticiye yeni bir iş parçacığı ya da fd üreticisi eklenirse bu
  karar yeniden değerlendirilir. Yalnız `shutdown`: belirti kalkar ama işçi
  yine yöneticinin soketini taşır.
- Kanıt: `web_isci_grubu::unix::testler::saglik_yaniti_kopya_acikken_de_baglantiyi_bitirir`
  (sunucu ucu `dup` ile açık tutulurken istemci 3 sn içinde EOF alır;
  düzeltmesiz kodda zaman aşımı). Regression `cli/isci-grubu-saglik-mirasi`
  (32 işçili üç grup, yoklama, işçi PID'lerinin tuttuğu sağlık soketi
  macOS'ta `lsof`, Linux'ta `/proc` ile sayılır). Yerel ölçüm (macOS, 40 grup ×
  32 işçi, ≈2 M istek): düzeltmesiz 8 miras soket, düzeltmeyle 0;
  düzeltmeyle ≈1,3 M istekte asılı yanıt 0.
- Dil yüzeyi, grammar, tanı ve `dil::api::v1` değişmedi.
## Ek — K-343: systemd soketiyle kesintisiz yeniden başlatma (19 Kasım 2026)

zee-cekirdek B6 talebi (exact `abaca3aba0e90f57d26435c4befffa903346c425`,
`dogfood/zee-cekirdek/systemd-soketi`): her dağıtım worker'ı yeniden
başlatıyor, birkaç saniyelik bağlantı reddi oluşuyordu; kabul ölçütü "yük
altındaki yeniden başlatmada 5xx/bağlantı reddi yok". K-336 grubunun
yöneticisi dinleyiciyi kendisi bağladığından `systemctl restart` aralığı yine
reddedilen bağlantıydı; ADR-118 sahipli host devri bu profili kapsamaz.

Karar: açık opt-in `--web-systemd-soketi` / `zee.ayar`
`guvenlik.systemd-soketi: true`. Yönetici bağlamaz; sd_listen_fds
sözleşmesiyle (LISTEN_PID = kendi PID'i, LISTEN_FDS = 1, fd 3) systemd'nin
tuttuğu soketi devralır, GB-133 doğrulayıcısıyla (`adopt_group`:
TCP/AF_INET/127.0.0.1:worker-port/LISTEN, stdio değil) sınar, `LISTEN_*`
kayıtlarını her durumda siler; ardından bugünkü `dup` → işçi devri aynen
işler. Devir yok/geçersiz → çıkış 2 ve `.socket` düzeltme önerisi; sessiz
bind yok. Yeniden başlatmada soket systemd'de kalır, eski yönetici K-336
boşaltmasıyla çıkar, yeni yönetici aynı soketi alır; aradaki bağlantılar
kuyrukta bekler. Belge `docs/cok-isci-systemd-soketi.md` kopyalanabilir
`.socket`/`.service` şablonlarını (Accept=no, KillMode=mixed,
TimeoutStopSec=50 > 40 sn boşaltma) taşır; CI ölçümü birimleri bu
bloklardan üretir.

- Reddedilen: LISTEN_FDS'i kendiliğinden algılamak (ortamdan gelen fd'yi
  sessizce sahiplenmek; opt-in K-064 yüzeyi açıkça kaydedilir).
  SO_REUSEPORT örtüşmesi: ana kararda reddedildi (macOS dağıtmaz, yabancı
  süreç bağlanabilir). Yöneticiden yöneticiye re-exec devri: ek protokol,
  systemd zaten soketi tutuyor; açık kalır. Bind'e geri dönüş: yanlış
  yapılandırılmış birim fark edilmeden kesintili çalışırdı.
- Kanıt: `web_cok_surec_testi::isci_grubu_systemd_soketiyle_yeniden_baslatmada_istek_kaybetmez`
  (soketi test tutar; dört istemci, üç SIGTERM/yeniden başlatma, 0 başarısız
  istek, yeniden başlatma başına en uzun istek ≈0,2 sn, yerel macOS),
  `…::isci_grubu_systemd_soketi_eksik_ya_da_gecersizse_oneriyle_reddeder`
  (devir yok, başka PID, LISTEN_FDS=2, yanlış kapı, grup yok → çıkış 2),
  `cli_ayar_testi::systemd_soketi_ayardan_yoneticiye_ulasir_ve_devirsiz_oneriyle_reddeder`,
  `ayar.rs`/`web_isci_grubu.rs` birim testleri, CI ubuntu-24.04 gerçek
  systemd ölçümü (`scripts/systemd-soket-olcumu.py`: beş `systemctl restart`,
  dört istemci, başarısız 0 kapısı, yeniden başlatma başına en uzun istek
  raporu).
- Dil yüzeyi: `profil cli-web-systemd-soketi`, `profil zee-ayar.systemd-soketi`
  (ADR-064). Grammar, tanı kodu ve `dil::api::v1` değişmedi. Güvenlik sınırı
  GB-140.

## Ek — K-360: kabul ile SIGTERM denetimi arasındaki bağlantı (21 Kasım 2026)

K-353 sonrası kalan sıfır kayıp boşluğu: işçinin istek döngüsü kabul sonrası
kapanış denetiminde yalnız ADR-118 native profilini "paylaşılan dinleyici"
sayıyordu. K-336 grup işçisi (`adopt_group`) dinleyicinin yalnız bir
kopyasını tutar, ama kod onu tek süreçli doğrudan bind gibi ele alıyordu:
kabul ile denetim arasına SIGTERM düşerse kabul edilmiş akış düşürülüyordu.
Tek süreçte bu doğruydu (dinleyici de kapanır, K-253); grupta ortak
dinleyici yöneticide/systemd'de açık kaldığından bağlantı yalnız o istek
için kayboluyordu. Pencere birkaç mikrosaniyedir; K-343 ölçümünde
gözlenmedi.

Karar: devralınan her dinleyici (native profil ve işçi grubu) paylaşılan
sayılır. Paylaşılan dinleyicide başarılı kabul admission noktasıdır:
kabul edilen istek mevcut bütçelerle yanıtlanır, bir sonraki `istek_al`
döngü başı denetimi kabulü durdurur ve işçinin kopyasını kapatır. Doğrudan
bind tek süreç K-253 sınırını korur.

- Reddedilen: kabul edilen akışı yöneticiye geri devretmek (yeni fd
  kanalı/protokol, aynı sonucu daha geniş yüzeyle verir); SIGTERM'i
  kabulden önce maskeleyip sonra açmak (her istekte iki sistem çağrısı,
  pencereyi kaldırmaz yalnız taşır).
- Kanıt: `web_profili_testleri::grup_iscisi_kabul_sonrasi_term_istegi_tamamlar`
  (gerçek `sunucu_kur` grup devralma yolu, test kancasıyla kabulden hemen
  sonra gerçek SIGTERM; düzeltme öncesi "kabul edilen grup isteği
  düşürüldü"), regression `cli/isci-kabul-term-penceresi`
  (`native-isci-kabul-term`, Linux: harness ptrace ile işçiyi başarılı
  `accept4` dönüşünde durdurur, SIGTERM'in ana iş parçacığında işlendiğini
  görünce bırakır; eski `kayıp 3`, yeni `korundu 3`), Linux systemd soket
  ölçümü tekrarları.
- Dil yüzeyi, grammar, tanı ve `dil::api::v1` değişmedi. Güvenlik sınırı
  GB-143.
