# Owned native listener ve generation hazırlığı — K-259

[ADR-118](../adr/118-native-listener-nesil-hazirligi.md) zee-cekirdek B6 talebini
sınırlı Unix host profiliyle karşılar. [Anonim repo reproducer'ı](../dogfood/zee-cekirdek/socket-devri/README.md)
gerçek talebin source/hash eşliğini taşır; özel ürün gövdesi veya müşteri verisi
kopyalanmaz. İlk yerel birleşim adayı native17/quietFD16 ve ayrı geç EOF1
kanıtını taşıdı. Tarihsel [S3 kabulü](urunlesme-s3-kabul-20261015.md) temiz
kaynak kapısını ve ayrı K-291 mixed-code native9/160 istek dilimini kaydeder.
Bu sonuçlar başka kaynağa aktarılmaz; tam P14 ürün benimsenmesi, genel sıfır
hata/SLA ve üretim dağıtımı açık kalır.

## Tek gerçek listening socket

[Host API](../scripts/native-listener-host.py) bir owned loopback TCP LISTEN
socket'ini yaratır, original'ı wave boyunca tutar, child'a exact duplicate'i
`pass_fds` ile aktarır. Native worker yeniden bind, SO_REUSEPORT veya proxy relay
kullanmaz. Yabancı PID/port/service/descriptor keşfi veya benimsemesi yoktur.
Worker yalnız kendi duplicate'ini kapatır; bağlantı veya HTTP request replay edilmez.

Native proxy çalıştırmada kaynaktan önce dört seçenek verilir:
`--web-listener-fd`, `--web-ready-fd`, `--web-activate-fd`, `--web-generation`.
Explicit HTTPS proxy origin ve worker port da gerekir. Bu seçenekler shell'de
rastgele FD numarasıyla çalıştırılacak deployment reçetesi değildir; creator
host gerçek socket/control ownership'i sağlar. Kaynak sonrası aynı sözcükler
uygulama argv verisidir. [Kanonik test](../scripts/tests/native-listener-test.py)
gerçek Zee argv oracle'ıyla bu ayrımı denetler.

Quiet borrowed preflight config/kaynak tanısı oluşmadan stdio/role aliases'i
reddeder; config sonrasında ve ownership transferinde tekrar denetim vardır.
Socket TCP, AF_INET, exact127.0.0.1/port, LISTEN ve O_NONBLOCK; kontrol pipe'ları
FIFO read/write rolü, distinctfd ve canlı peer ister. FD_CLOEXEC ve tek owning
wrapper korunur; ikinci `sunucu başlat` aynı raw FD'yi tekrar edinemez. Darwin'de
SO_ACCEPTCONN getter ENOPROTOOPT olduğundan native SDK TCP_CONNECTION_INFO
TCPS_LISTEN durumu kullanılır. Opposite pipe ends farklı inode taşıdığından
own-PID SDK pipe_handle/peerhandle çifti de karşılaştırılır; değerler loglanmaz.
Bu macOS kanıtı diğer fiziksel platform kabulü olarak sunulmaz.

## Ready, ACTIVATE ve admitted stream

Native ready pipe tek bounded fixed JSON kaydıdır: exact owned Popen PID,
fresh generation nonce, port ve actual BuildJSON. Eski worker HTTP200 cevabı
bu channel'ın yerini tutamaz. App receipt ayrıca private generation'a ait fresh,
stable/nofollow exact üç alanlı dosyadır; expected frozen app revision ister.
Anonim helper yalnız kendi başlangıcını bildirir; DB/migration readiness için
[K251 uygulama probu](../adr/112-db-goc-hazirlik-probu.md) ayrıca gerekir.

Exact `ACTIVATE <nonce>\n` ve EOF gelmeden ilk accept yapılmaz. Kontrol okuması
nonblocking, absolute deadline, sinyal ve EINTR/WouldBlock retry sınırıyla bounded'dır.
Wrongnonce, emptyEOF, extra/duplicate/oversized control aktive etmez. İki
neslin ready örtüşmesinden sonra old worker'a TERM verilir; original socket açık
kalır. Successful accept bu inherited profile'da admission'dır; hemen sonrası
TERM'de stream mevcut read/write/handler sınırlarıyla tamamlanır. Gerçek TCP ve
SIGTERM ile zorlanmış postaccept race unit bunu kanıtlar. Default direct bind
K253 postaccept davranışı değişmez. WNOWAIT leader reservation grup cleanup
bitene kadar korunur; EOF yolunda blocking stderr diagnostic okunmaz.

## Shared JSON state ve metrik kompozisyonu

İki worker aynı canonical project/root ve `.zee/web-durumu-v1.json` atomic durumunu
kullanır; JSON dosyası reset/copy/delete edilmez, SQLite backend iddiası yoktur.
Gerçek login/formCSRF oturumu yeni generation'da çalıştı; revoked cookie overlap
ve restart sonrası401 aldı. Kota429 yeni generation'da korundu. Süreç belleği
quota backend'i inherited overlap profilinde reddedilir; restart reset'i gizlenmez.

[Native metric](native-metrics.md) FD'si ayrı owned nonblocking write-only pipe'tır.
Before/after listener seçenek sırası ve reader preclosed EPIPE ile birleşim
sınanır. Telemetry kaybı geçerli listener'ı durdurmaz; metric sink sahipliğinden
sonra kapalı writer FD yeniden listener ownership'e alınmaz. Control pipe'ları
aynı best-effort politika değildir. [Credential capability](../adr/119-credential-native-metrics-capability.md)
tek metrik profiliyle yalnız metadata pipe'ını aktarır. Ayrı K-267 typed
credential listener route aşağıdaki dar stage adayıdır; varsayılan doğrudan
native owned host ve noinherit sözleşmesi değişmez.

## Tekrarlanabilir sınırlı kabul

```sh
python3 -I -B scripts/tests/native-listener-test.py \
  --framework-root "$PWD" --dil <owned-binary> --output <fresh-output> \
  --metrics-composition
python3 -I -B scripts/tests/native-listener-preflight-test.py \
  --framework-root "$PWD" --dil <owned-binary> --output <other-fresh-output>
```

Koşucu verilen binary SHA256 ve actual BuildJSON'u kaydeder. Bu fixture dirty ikiliyi reddetmez; temiz kabul için ortak kampanya ayrıca exact source SHA ve dirty=false denetler. `--metrics-composition` olmadan üç
birleşim vaka atlandığından bu kısmi koşu full merged gate sayılamaz.
[Cargo wrapper](../compiler/tests/native_listener_testi.rs) native17, quietFD16 ve ayrı geç EOF1'i
exact zorunlu yürütür. Sayılar Python kampanyasıdır, Rust faz toplamına eklenmez.

Native17 lifecycle/activation/receipt/TERM, shared quota/revoke,1000 request,
owned native descendant cleanup, iki metric order/EPIPE, gerçek argv ve bozuk
`zee.ayar` reddini kapsar. FD16 stdio/control/metric aliases, UDP/nonLISTEN/
blocking/wildcard/wrongport/closed/stdio role ve `--ayar`/`--profil`-öncesi earlyquiet
sınırlarını içerir. Native1000 sonucu concurrency1/dört owned rollover ve ayrı
sentetik loopback istemcileridir; quota raise/reset,5xx/refusal/retry görülmedi.
Bu yük sınırları içindeki fixture sonucu public SLA veya saturation testi
olarak genellenmez. Baseline rate/argv correctness K262 ayrı kayıttır.

Sabit aday `2caa26e55e50bf35b4220e4f6d16ac7273f337159243b6e97c8f2443581adbf8`
BuildJSON'u17b29da1 taban/dirtytrue taşır. Finalclean ortak faz/native,900sn host,
1800sn soak, explicit ASAN RC/Miri, Windows/minOS ve gerçek ürün benimsemesi
henüz bu sayılardan kapanmaz. [Ürünleşme planı](urunlesme-plani.md) ve
[sürüm kapıları](v1-surum-kapilari.md) P13/P14/P25'i kısmi/açık sınırla izler.

Kanonik güvenlik sırası: atomik K-263 owned-host cleanup düzeltmesi GB-077 ile `ac077e31e761e401eedc5140a8035fd02f2a19ba` kaynak commitinde tamamlandı; K-262 dar CLI bugfix `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179` ve exact provenance `4d8edeb2a9f4453d2f28481ecd550bf8ba5f2032` ile kapandı. Bu ilk kaynak kaydında ortak temiz özellik kabulü bekleniyordu; sonraki tarihsel [S3 dar kabulü](urunlesme-s3-kabul-20261015.md) kendi kaynak kimliğiyle ayrı kaydedildi. S3 sonucu başka kaynakta yeniden koşulmuş sayılmaz. Kaynak `54a8d6dc0efb060ae5644b37546da40ad03ee035` ile commitlenen özellik dilimlerinin kayıtları K-257/GB-078, K-258/GB-079, K-259/GB-080 ve K-260/GB-081’dir; ADR-116–119 numaraları korunur. Önceki immutable dış-stage makbuzlarının geçici GB etiketleri tarihsel kanıttır, kanonik yeni kayıt kimliği değildir. Finalclean/900sn host/1800sn soak/RC kabulü bu revizyonda ileri sürülmez.

## Mutlak activation bütçesinin son EOF kontrolü

Commitlenmemiş ilk aday son poll turunda on saniyelik bütçe sonrasındaki EOF ile HTTP200 verebiliyordu. Poll artık yalnız kalan bütçeyi bekler; exact frame ve EOF sonrasında deadline/sinyal yeniden denetlenir. [Ayrı owned test](../scripts/tests/native-listener-deadline-test.py) EOF zamanını ilk byte tüketiminden hesaplanan üst sınırın sonrasına koyar; düzeltmede HTTP byte0, normal terminal0, tüm owned child reaped ve parent listener korunmuş olmalıdır. Bu tek vaka native17 veya FD16 toplamına eklenmez. Kaynak commitine girmemiş aday için tarihsel introduced_by/fixed_by SHA üretilmez; bulgu GB-080 altında ilk özellik değişikliğiyle birlikte kapanır.

Kalıcı vaka [native-listener-late-eof](../regression/security/native-listener-late-eof.dil), ilk özellik kaynak commit `54a8d6dc0efb060ae5644b37546da40ad03ee035` için exact fixed_by taşır; introduced_by=-, çünkü eski2caa bir Git commit değildi. Ayrı semantic test gerçek source bytes/helper/HTTP0/terminal0/owned cleanup doğrular. Native17, FD16 ve deadline1 kampanyaları ile Rust/corpus sayıları toplanmaz.


## K-267/P25 typed native listener bileşimi — dar Unix profil

Yeni `CredentialRoute` ctor listener/ready/activate ve host-only startup FIFO ile
fresh generation, explicit port/binary hash'i bağlar; optional designated metrik
kanalı korunur. Host+launcher ortak borrowed rol/alias denetimini secret okumadan
ve Popen öncesi kullanır. Launcher actual child PID/gen/hash makbuzunu yalnız
startup FIFO'ya tek sınırlı yazımla bırakır; native child bu FD'yi almaz. Native
ready bu trusted kayda ve app receipt'e bağlanır; host yalnız launcher Popen
ve WNOWAIT process group'unu sahiplenir, native PID edinmez. Varsayılan noinherit
ve native stdout/stderr secret-masking korunur. Yeni typed rol STDIO alias
retlerinde quiet olur. source sonrası argv yeni FD capability açmaz.

Gerçek kısa d53 aday koşusu Build dirty=true ile kaydedilmiştir; temiz final
source/normal faz/ortak kampanya kabulü değildir. Internal P25 bridge dar dilimi
kanıtlı adaydır; Windows/systemd/file-native sır gizliliği ve live adoption
açık kalır. [ADR-125](../adr/125-credential-native-listener-kompozisyonu.md)/GB-083 aynı atomik kaynakta bağlanır; normal Cargo/faz ve final temiz kaynak kabulü ayrıca yürütülür.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

S common native listener17, quiet FD16 ve deadline1 geçti. [Son rapor](urunlesme-final-kabul-20261015.md) bunları 900/1800/RC kanıtlarından ayrı sayar; genel kesintisiz deployment veya platform SLA’sı kurulmaz.
