# Ürünleşme — yerel kabul ve açık teknik işler raporu

> 16 Ekim 2026. K-298–K-302 dar yerel kabul kapsamları tamamlandı.
> Genel ürünleşme/V1 ve yayın kapıları açık; belge takip commit’i aşağıdaki dış kapı makbuzlarına bağlıdır.

## K-300 — Karma sürüm koşucusu: dar kabul tamamlandı

**K-300 ve GB-112 kapalıdır.** B-001/B-002 gerçek insan verisini bekler;
B serisinin diğer makine işleri kapalıdır. P14/P36 kabul koşucusunda bulunan
somut arıza, yeni kapasite işinden önce kod+test+belgesiyle ele alındı.

Rust wrapper'ın `compiler/..` kökü kanonikleştirildi. Python koşucusu ilk Git
sorgusundan önce iptal handler'ı kurar; kısmi hazırlık ve Host listener'ı sahipli
`finally` temizliği kapsamındadır. Host kurulumu sırasında veya kapanışta gelen
iptal korunur; iptal edilen koşu başarılı makbuz üretemez. Üretim
compiler/Host/FD protokolü, grammar, zaman aşımı ve native doğrulama ölçütleri
aynıdır. S3 veya K-298 sonuçları bu yeni kaynağa aktarılmaz.

- Dört eski kaynak karşı örneği mock-only koşuda doğrulandı. Güncel hazırlık
  **13/13** geçti (0,217 sn); gerçek erken child iptali, kısmi Host temizliği
  ve kapanış/yayın sırasında iptal dahil. Bunlar native başarı sayısına eklenmez.
- Kaynak kapısı2 **13/13** geçti: tam faz **998/1023 başarılı, 0 başarısız,
  25 ayrı koşulan test**, Clippy, katalog/tazelik, spec, fmt, üç koruk, sır
  taraması ve sürekli güvenlik. Faz komutu 778,533 sn; tüm kapı sürücüsü
  959,067 sn. İlk koşu ADR durum başlığında durmuştu; biçim düzeltildi ve
  başarısız makbuz korundu.
- Test edilen 16 dosya temiz kaynak
  `549dbe66a775db17875b47c67948245a6a8fbf60` olarak kaydedildi; ağaç
  `833c8cfb7d841e4fd27bcd7f2395e287c1c3bc99`.
- Bu exact temiz kaynakta mevcut **bir ignored Cargo sahibi**, **dokuz native
  senaryo** ile geçti. Dört dalgada **160/160 HTTP200**, concurrency2; 5xx,
  retry, taşıma hatası ve body mismatch **0**. Her dalgada 40 istek, ACTIVATE
  öncesi iki kuyruk isteği ve eski TERM sonrasında 38 tamamlanan yanıt var.
  Eski başlamış isteğin tamamlanması, oturum/revoke/kota korunması ve üç ret
  senaryosu ayrıca doğrulandı; native alt vakalar yeni Cargo sahibi değildir.
- Native ikili SHA-256:
  `a0fa25443baa9b0ed2ac9a840b9b27eb5c3dd2fb42fb3eac3ddb878a12cedc01`.
  Başlangıç/son kaynak temiz, kaynak ve ikili pinleri eşit; controller
  **exit0/reaped**, 87,296 sn. Runtime `.zee` verisi okunmadı/arşivlenmedi.

[Türetilmiş kabul özeti](incelemeler/urunlesme-20261014/k300/ACCEPTANCE.json)
özgün dış makbuzları hash/baytlarıyla bağlar. `E`, yerel
`~/.codex/zee-evidence/2026-10-14-urunlesme` kanıt köküdür. Temel makbuzlar:

| Makbuz (E altında) | SHA-256 |
| --- | --- |
| `k300-source-gates-2/PROOF.json` | `5d942c15e9e693d9c6fe2fcc8d663678ebda62e0cffcace85cb1078cd4b27912` |
| `k300-source-commit.json` | `f7b9cc2be9a6fe2cf885a2c95765ee9a4398dec312e7dccb1c7408b21b825f95` |
| `k300-native-1/PROOF.json` | `499d44f433b95a0fcfefd8844f5d229f9cd69b01b2f06d8d9d3bfce7b02f399b` |
| `k300-native-controller-1.json` | `a3741c28684d8ffc99c94e3dba2f0cb21e83a946d3faa7b5755427cfa1a59ae0` |

Kapanışın son belge/güvenlik kapısı **11/11** geçti (74,805 sn; özgün
controller exit0/reaped). Belgeler `7093e0c7d668ff5845cbe513816aba7e64da93fe`
commit'iyle temiz kaydedildi. Dış `k300-final-docs-1/PROOF.json` SHA-256
`1c227d67e1e84ba62cc176e8f27e1a210f5b6d7739a56bc0a39006be81f352c8`;
`k300-final-document-commit.json`
`cd652353fe6880144d61d328d57bdbb22a8be68875b5b064dddd0600d80e766f`.
[Koşucu sözleşmesi](mixed-revision-wave.md) ve [ADR-149](../adr/149-mixed-revision-owned-wave.md)
dar kapsamı taşır. P14 genel SLA, DB şema devri, ürün ve platform kabulü
kısmi kalır; yeni kaynak RC/soak/IDE kabulü ileri sürülmez.

Önceki K-298/K-299 belge dalgası `a31f12a41389ad1d447d5f17df6f7be9f1ae75f4`
ile kaydedilmişti: kendi son belge/güvenlik kapısı 11/11 ve özgün controller
exit0/reaped. Bu tarihsel kapı K-300 belgelerinin yerine geçmez.

K-302 ile özel PostgreSQL16.15 ortamı yeniden kuruldu ve TLS8/göç1
kabulü tamamlandı. Sonraki yerel inceleme, aynı ortamda mevcut havuz ve
uygulama kabul testleridir; ardından özel Linux ortamının yeniden kurulması
gelir. Bunlar kullanıcı cihazı/verisi bekleme gerekçesi değildir; fiziksel
platform U2 ayrıdır.

## K-301 — Worker testinde gerçek giriş ve tam yanıt kanıtı

**Dar test doğruluğu işi tamamlandı.** Önceki test handler başlamadan gelen
EOF'yi veya ilk TCP parçasında bulunmayan `bitti` gövdesini geçirebiliyordu.
İlgili fixture artık gerçek cookie/CSRF hazırlığından sonra POST/eylem üzerinden
exact `entered\n` işareti üretir. Test işareti doğruladıktan sonra kapatır;
yanıtı EOF/reset'e kadar bir saniye/16 KiB sınırında toplar. GET salt okuma,
eylem, oturum ve CSRF kuralları aynen korunur.

- Son V3 kaynak SHA-256 `15abd88acb28a13c89bbee1d248287953b95d47544594b350628d770dc86d71e`.
  Mevcut **tek Cargo sahibi geçti** (78,28 sn); wrapper aynı **25 Python
  vaka/0 hata/0 atlama**, sekiz native vaka ve iki credential bileşimini
  zorunlu doğrular. Başarılı Cargo günlüğü iç Python çıktısını ayrıca basmaz.
- Özgün controller **exit0/reaped**, 79,284 sn. Önce/sonra 11 kaynak pini
  eşittir. Bu çalışma `7093e0c7d668ff5845cbe513816aba7e64da93fe` tabanı
  üzerindeki test/belge değişikliğinin olağan Cargo kabulüdür; temiz sürüm
  artefaktı veya yeni kaynak RC/soak/IDE kabulü değildir.
- V3'ün 10 kontrollü senaryosunda 20 eski/yeni metot karşılaştırması geçti;
  eksik/yanlış giriş, parçalı tamamlanmış gövde ve timeout olumsuzları kapsanır.
  HTTP/CSRF yardımcısı mock karşılaştırmada taklittir; gerçek yolu worker3
  koşusunda geçti. Varsayılan fixture ve diğer metotların AST eşliği, aynı25
  metot ve tek Cargo sahibi doğrulandı. Yeni/taşınan test sahibi yoktur;
  K-300'ün 998/0/25 tam faz sonucu K-301'e aktarılmaz.
- İlk iki gerçek koşu 25 vakada bir inflight hatasıyla reddedildi ve korundu.
  İlk fixture'ın GET içinde yazması bağımsız native denetimde T045 verdi.
  İkinci fixture'daki CSRF önkoşulu eksikliği kaynak/spec incelemesinde bulundu;
  günlük assertion ayrıntısı vermediği için gözlenmiş403 veya eksik işaret
  sonucu ileri sürülmez. Son koşu doğru POST/eylem/oturum/CSRF akışıyla geçti.

| Özgün makbuz (E altında) | SHA-256 |
| --- | --- |
| `k301-worker-inputs-3.json` | `589ea3e5a1b73b387a0cdfb757864e4ff919fa4dd2fa5b31c6d0dee341f41626` |
| `k301-worker-owner-3.log` | `faf9053769e1e1bfd8ee4803eb9495cf454198e5061f04c2d08e15c7c3bbdb7b` |
| `k301-worker-owner-controller-3.json` | `9c6b0e793226b3e169a077e5b6484af9c4bc9496e0031465af29dde94a27437d` |

Mock V3 `PROOF.json` SHA-256
`cad3219d9993df0e8672ea9c60b4cf99612e20c1fa63ebd8e2b0d2f7967f5c19`;
kalıcı kopyası `E/k301-mock-evidence-3/PROOF.json` altındadır.
Bu kod ve belge grubunun **11/11** son kapısı geçti (74,072 sn), özgün
controller exit0/reaped. Tam test edilen 11 dosya
`e6322dbc0424a54f777a7b14f65d78712cfced1a` commit'iyle temiz kaydedildi.
`E/k301-gates-1/PROOF.json` SHA-256
`83b38764505e6a36fa465936466b5e64bd8ae970c5e6db4c2c8d479d51295fe8`;
`E/k301-commit-1.json` SHA-256
`cfc5d923ca529c7a127077f7b78434fdf5517792f01d3113f564fd607473f6bf`.
Üretim compiler/Host davranışı değişmedi. Genel P34, native DNS, tam fault,
drain ve SLA kapıları kısmi kalır. Bu yerel iş için kullanıcı girdisi gerekmedi.
[Host rehberi](worker-supervisor.md) ve K-301 backlog kaydı aynı kapsamı taşır.

## K-302 — PostgreSQL ortamı ve mevcut TLS/göç kabulü

**Dar yerel kabul tamamlandı.** Temiz kaynak
`e6322dbc0424a54f777a7b14f65d78712cfced1a`, ağaç
`df91f02403c9f11f7e2e6a073b93bd20ce95e4f2` üzerinde mevcut **sekiz TLS ve
bir göç testi geçti: dokuz başarılı, sıfır başarısız, sıfır atlanan**.
Production, test kaynakları, grammar ve API değişmedi.

- Silinmiş geçici PostgreSQL yerine resmi 16.15 kaynak arşivi SHA-256
  `c1575341fa7bd40f5274ea465b34390f4dc64cdd0770af327005caaeb9f6b7ed`
  doğrulanarak ayrı özel dizine derlendi. Mevcut OpenSSL3.6.4 ve gerçek
  libpq bağlantıları kaydedildi; küresel paket veya servis değiştirilmedi.
  İkinci derleme 19 komutla geçti (97,178 sn); bu derleme tek başına Zee
  kabulü değildir.
- İlk derleme `utils/errcodes.h` bulunamadığı için make exit2 ile durdu ve
  başarısız makbuzu korundu. Arşivi açan geçici araç yayıncı zaman damgalarını
  korumamıştı. Araç düzeltildi; aynı arşiv, araç zinciri ve derleme bayrakları
  ile bütün 8.482 dosyanın zaman damgası doğrulanarak ikinci koşu geçti.
- Native kabul 35,622 sn sürdü. TLS1.2 olumlu bağlantı; CA/hostname/NULL,
  bütçe sınırları, gerçek 40P01/40001 statement hataları ve erişilebilir
  TLS1.1 sunucusunun reddi geçti. TLS1.1 olumlu kontrolü protokol, sertifika
  ve hostname doğrulamasını gerçekten tamamladı; yalnız izole eski-protokol
  fixture'ıdır. Göç vakası mevcut rol/checksum/atomik uygulama sınırını geçti.
- Başlangıç/son kaynak temiz ve eşit; 14 kaynak pini değişmedi. Üç geçici
  PostgreSQL sunucusu özgün süreç sahipliği üzerinden normal kapandı,
  exit0/reaped; özel veri dizini kaldırıldı. Dış controller exit0/reaped
  (36,132 sn). Üç ilk readiness denemesinin exit2 sonucu tekrar kaydıdır;
  bunlar test başarısızlığı ya da gizlenmiş Cargo atlaması değildir.
- Bu kampanya mevcut **25 explicit ignored Cargo vakasının seçili dokuzunu**
  bu kaynakta çalıştırdı; **kalan16 bu kampanyada çalışmadı**. Eski S/C2/K-300
  sonuçları bu kaynağa toplanmaz. Tam faz, RC/soak/IDE, canonical `pg_ctl`
  koşucusunun yaşam döngüsü, sağlayıcı/COMMIT fault veya gerçek ürün kabulü
  çıkarılmaz. P31/P36 genel sınırları açık kalır.

| Özgün makbuz (E altında) | SHA-256 |
| --- | --- |
| `k302-pg16-runtime/PROOF.json` — ilk başarısız derleme | `081bc3a367c2fcc9545b682aac579fc708c2cb8ca4f1cdfeda83fc20866c895b` |
| `k302-pg16-runtime-2/PROOF.json` — başarılı derleme | `66283e9b683e7b3d82e522656010fad93b207799c86c177710f9c177eced74a2` |
| `k302-build-controller-2.json` | `3f4ebeafed7c3da3e10ed9aa2389b840d01bb6719594d27f62d995d5464d7ac8` |
| `k302-pg-accept-1/PROOF.json` | `47f27bb589efb84304913b417ae89f2662e7a7d0469e343785c3509fd93e2050` |
| `k302-accept-controller-1.json` | `2beb111c485c5ceb4337c96cb655bc3494c3911eeeebbb684a18f65e731374bb` |

Son belge/güvenlik kapısı kayıt commit'inin önkoşuludur; gerçek sonuç
`E/k302-gates-1/PROOF.json` ve özgün `k302-gates-controller-1.json`, kayıt
ise `E/k302-commit-1.json` üzerinden izlenir. Bu belgeler kendi sonraki
commit kimliğini veya henüz çalışmamış kapı başarısını ileri sürmez.

Belge etki incelemesi: README, master plan, backlog, sürüm kapıları,
sürüm notları, PG rehberi, kanıt haritası ve ürünleşme planı güncellendi.
Normatif davranış değişmediğinden spec/RFC/ADR ve indeksleri, tanı kataloğu,
compiler/core-freeze/regression kayıtları, katman grafiği, test sayısı/faz
sahipliği değişmez. Performans iş yükü/ADR-045/049 ve ölçüm tarihçesi,
RC/soak tarihçesi, artefakt ve kurulum sözleşmesi de değişmez.

## Sonuç özeti

| Alan | Doğrulanmış durum |
| --- | --- |
| P19 sabit yerel stdio hedefi | Dört kohort p95 <200 ms; en ağır kohort189,885667 ms. P19 bütün olarak kısmi. |
| macOS teslim ve kurulum | İki temiz klonda eş ikili, sourceZIP/bundle ve kurulum/kaldırma8 geçti. |
| UTF-16 ve gerçek IDE | Native stdio10 ve aşağıda sınırları verilen yerel IntelliJ işlev kabulü geçti. |
| Uzun süreli RSS soak | S kaynağında ikinci koşu geçti: 1801,116786 sn, 360 örnek; compiler büyümesi %0, LSP +1024 KiB ve yukarı yuvarlanmış %6. |
| Release-candidate | S kaynağında dört explicit ASan hedefi 1801’er sn, Miri 2+1 ve tam RC güvenlik kapısı geçti: 131 Cargo grubu, 998 geçti/0 başarısız/25 atlanan. |
| K-299 kabul betikleri | C2 kaynak zincirinde 13 kapı (998/0/25) ve sürekli güvenlik; temiz C2 native PDF9/service2 geçti. Küçük kanıt arşivi doğrulandı; GB-111 dar kapanışı bu belge dalgasında kaydedilir. İlk başarısızlıklar korunur. |
| K-300 karma sürüm | Hazırlık13, kaynak kapıları13 (998/0/25) ve temiz `549dbe6` native9/160istek/dört dalga geçti; GB-112 dar kapsamda kapalı. |
| K-301 worker test doğruluğu | Son fixture ile mevcut25 Python vaka/tek Cargo sahibi geçti; mock10/20 ve kaynak pinleri doğrulandı. İlk iki ret korundu. |
| K-302 PG/TLS/göç | Temiz `e6322db` üzerinde TLS8/göç1, sıfır başarısız/atlanan; üç özel sunucunun normal kapanışı ve kaynak eşliği doğrulandı. |
| Genel ürünleşme/V1 | Bütün39 madde kapanmadı; aşağıdaki dış girdiler, takvim ve teknik kapsam açık. |

Compiler bakım commit'i `bb4ed951ce5e8e9c1c21d2c58f75cd4ef51d6c59`, ağaç
`59206d3fb9520e17522bba2a031d52f891ec42e0`.
Gerçek yeni kabul/ölçüm kaynağı `a57290f7a1c1fad07cda3f266b2a221f09aa6404`, ağaç
`20644b4b6dc735a8cda8a24b7c064d28f12498d4`. İkinci commit maintenance/provenance
kayıtlarını taşır; compiler bakımının kimliğiyle karıştırılmaz. Bu raporu
kaydedecek belge takip commit'i ölçülen kaynağın yerine geçmez.

## Doğrulanan dar sonuç

Yeni exact kaynakta 20.081 satır/40 birimlik sabit yerel stdio iş yükünün
**dört kohortunda gözlenen p95, 200 ms hedefini karşıladı**. En ağır lint
açık/41 belge kohortunda yeniden ölçülen eski kaynağın p95'i **204,880125 ms**,
K-298 kaynağının p95'i **189,885667 ms**: bu koşu çiftinde **%7,32**
düşüş. Yeni kohortun ölçülen maksimumu **190,114958 ms**.

Bu, P19'un bu iş yükündeki yerel stdio p95 hedefinin dar gözlemsel kabulüdür; genel P19/V1
kapanışı değildir. Paylaşımlı makine gözlemidir; hard eşik veya adanmış
benchmark tabanı uygulanmadı. Sonuç tek koşu çiftinin ölçülen farkını verir;
genel hızlanma, istatistiksel kararlılık, her makinede 200 ms veya SLA sözü
vermez. Kalıcı `performans-gecmisi-v2.tsv` tabanı oluşturulmadı.

K-298, bildirim ömürlü gerçek yol → tutarlı ödünç metin diziniyle aynı açık
belge yolunun yinelenen çözümünü azaltır. Eş ad çelişkisi, disk/paket/lock
kontrolleri ve sonraki bildirimde yeniden değerlendirme korunur. Yeni kalıcı
önbellek veya artımlı HIR/tür kapasitesi eklenmedi. Üç alias sıralaması/lint
kipleri ve Unix symlink hedefinin sonraki bildirimde yenilenmesi için iki
koruyucu test kaynak değişikliğiyle birlikte yer aldı.

## Yeniden ölçülen eski/yeni stdio karşılaştırması

Önce: `4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2` / `6c4982688f5105a67c025815cbbb72c07fcea7f8`.
K-298: `a57290f7a1c1fad07cda3f266b2a221f09aa6404` / `20644b4b6dc735a8cda8a24b7c064d28f12498d4`.
Her iki koşuda aynı driver ve aynı fixture dosyalarının SHA-256 kümesi kullanıldı.

| Lint | Açık belge | Önce p50 ms | Önce p95 ms | Önce max ms | K-298 p50 ms | K-298 p95 ms | K-298 max ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| kapalı | 1 | 15,281333 | 16,024958 | 16,038833 | 14,936291 | 15,597958 | 15,768666 |
| kapalı | 41 | 96,558000 | 97,795333 | 98,900166 | 89,578375 | 91,896500 | 94,612625 |
| açık | 1 | 36,560084 | 37,885125 | 37,964250 | 36,136750 | 37,204458 | 37,387791 |
| açık | 41 | 202,703958 | 204,880125 | 205,722375 | 186,008333 | 189,885667 | 190,114958 |

Her kohort yeni LSP süreci, **5 saklanan ısınma + 20 gerçek ölçüm** içerir.
p50 sıralı 20 örneğin sıfır tabanlı 9. elemanı, p95 nearest-rank 18. elemanı,
max 19. elemanıdır. Yukarıdaki sayılar hamdan yeniden kurulan edit zamanlarıyla
ve makbuzdaki sample dizileriyle birebir doğrulandı. Her kaynakta toplam 80
ölçülen/20 ısınma edit ve 84 açma işlemi bağımsız verifier tarafından yeniden
kuruldu; her kaynak koşusunun 63 dosyalık ham/metaveri manifesti de hash/byte ile kontrol edildi.

Ölçüm aralığı ilk başarılı `os.write` çağrısından son tam `publishDiagnostics`
çerçevesinin okunmasına kadardır. İstemci zamanlaması ve ara çerçeve ayrıştırması
gözlemi etkiler; gönderim öncesi JSON/framing hazırlığı ve son çerçevenin sonraki
doğrulaması aralık dışındadır. Statik completion bariyeri ölçüm dışındadır.
Açma tanı sürelerinin toplamı tam süreç cold-start/duvar süresi sayılmaz.
Lint kapalıda 0, açıkta 1 tanı; açık belge sayısı kadar tanı bildirimi doğrulandı.

Makine: macOS 26.6.2, Apple M4 Pro/14 CPU, 24 GiB RAM, arm64;
Rust 1.93.1, Python 3.13.3. Kanonik release profili
`cargo build --locked --release --bin dil --bin dillsp`, aynı hedef/bayrak
ve remap ayarlarıdır. Her iki makbuz `shared_machine=true`,
`unowned_machine_work=not-assessed`, `OS_disk_cache_reset=false` taşır.
Sahipli ağır derleme/test controller'ları kapandıktan sonra en az 60 saniye bekleme
kaydı vardır; eski ve yeni ölçüm sırayla yürütüldü. Başka görevlerin yükü
ölçülmüş veya makine adanmış sayılmaz. Her kaynağın dört süreci de normal shutdown/exit0
ile özgün sahipleri tarafından toplandı; stderr boştu.

Özgün S4 raporundaki lint açık/41 p95 **223,697750 ms** tarihsel gözlemdir.
Buradaki %7,32 hesabının tabanı yeni alınan **204,880125 ms** ölçümüdür;
eski S4 değeri bu eşleştirilmiş koşu çiftinin tabanı yerine kullanılmadı.

## İkili, kaynak ve bağımsız denetim bağı

| Kaynak | CLI SHA-256 | LSP SHA-256 |
|---|---|---|
| Eski 4b7dd76 | `63d8bd1e2676600e3715979320415b4753680b894299e3c4daa73bb9c6eb6ec9` | `c9989583f8d25ce5dc81ed1aa804f85376022ed54cf635e1b701e3dcf6bfd288` |
| Yeni a57290f | `c597b9c1d0ec11e59ab0b3cc4701f1e21a37bcec67b37d4d52f983ddf69d56e2` | `fc4391569ed45bccc11cb424f1aeb7abaea7133170d8d55f1df95420360302f1` |

Her iki CLI BuildJSON ve native LSP initialize kimliği kendi exact SHA'sını
ve `dirty=false` değerini verdi. Temiz kaynak SHA/tree start/end ve ikili
hashleri değişmedi. Ölçüm driver SHA-256:
`e3916314960605b94f3d6cfac30a48576bbf2067e23f905a94bc5fced03d9ab5`;
bağımsız raw verifier SHA-256:
`5dea95761414bc09869ff817b662f10b7f336ef9469310f41fb1ae0495010eaa`. İki bağımsız review da kendi ölçüm proof
hashine bağlı `passed`, `verified_cohorts=4`, `raw_reconstruction=true`,
`no_new_processes=true` makbuzu verdi. `threshold_ms=null`,
`hard_200ms_gate_applied=false`, `observation_acceptance_only=true` korundu.

## Yeni kaynakta doğrulanan diğer dar kabuller

- **macOS teslim:** kanonik iki temiz klonda CLI/LSP hashleri eş;
  kaynak ZIP çifti eş, bundle restore/fsck ve SBOM/provenance mevcut.
  İkili imzasızdır ve yayımlanmadı. Native-deployment/build-environment kayıtları
  geçti; fiziksel Windows/minimum OS veya yeni Linux kabulü sayılmaz.
- **Kurulum/kaldırma:** aynı teslimle 8 sahipli prefix vakası geçti:
  doğru kurulum/BuildJSON/LSP, tekrar kurulumun reddi, değişmiş kurulumun
  kaldırılmadan reddi, yabancı dosyayı koruyan manifestli kaldırma,
  tekrar kaldırmanın reddi, yeniden kurulum/kaldırma, bozuk ikili ve bozuk
  checksum reddi. Controller normal exit0/reaped; orijinal artefakt değişmedi.
- **Ordinary UTF-16 native stdio:**10/10 LF/CRLF; tek/çok emoji sonrası
  hover/definition/rename/didChange, geçersiz surrogate/dış konum sorguları,
  iki emoji sonrası A001 `[0:11,0:16)` ve atomik değişiklik reddi geçti.
  Normal shutdown sonucu null, exit0/reaped, stderr 0. Bu gerçek IDE/UI
  kabulünden ayrıdır; K-297'nin yeni exact kaynaktaki correctness tazeliğidir.
- **İki ek Cargo sahibi:** fixed host PDF backend wrapper'ı ve mevcut
  `lsp_artimli_testi::yirmi_bin_satir_proje_duzenleme_olcumu` ayrıca geçti.
  PDF wrapper kaynakla pinlenmiş 8 iç vakayı doğrular; iç unittest stderr'i
  dışa aktarılmadığından 8 ayrı ham test logu var denmez. Saklanan PDF
  artefaktları bağımsız yeniden incelendi. ReportLab 4.4.9/pypdf 6.10.0/Pillow 12.3.0
  runtime kimliği kayıtlıdır. Signed PDF, native Zee emitter ve native
  worker/service-fault çalıştırılmadı.

Eski in-process LSP testinin lint kapalı 1/41 belge p50/p95/max değerleri sırasıyla
16,982666/17,896750/17,898792 ve 99,328708/103,663000/104,356792 ms'dir.
Bu koşu delivery derlemesiyle çakıştığından **yalnız işlevsel owner kabulü**
olarak kaydedilir; stdio 200 ms veya hızlanma hesabına alınmaz. Kohort başına 20
örnek vardır; 5 ısınmanın ham örnekleri bu eski testte saklanmaz. Backend 8 ve
kohortlar Cargo vaka sayısına eklenmez: burada tam **iki Cargo sahibi** vardır.

## Gerçek IDE — dar yerel işlev kabulü

Doğrudan UI/trace gözlemine göre IntelliJ IDEA 2025.3.1
(`IU-253.29346.138`) ve LSP4IJ ile aynı a57290f kaynağın proje launcher'ı
kullanıldı. `PREPARATION.json`/`LAUNCHER-PREPARATION.json` kaynak, teslim ve
ikili hashlerini bağlar. Launcher için sentetik imza kullanıldı; geçici özel
anahtarın silindiği kayıtlıdır. Gerçek release imzalanmadı/yayımlanmadı.

İlk oturumda **03:12:50.913** trace Apply işlemi LSP4IJ settings EDT exception
verdi; hata raporu gönderilmedi. Normal Quit sonrası ikinci oturumda aşağıdaki
olumlu/olumsuz gözlemler alındı. Saatler 16 Ekim 2026 Europe/Istanbul'dur.

| Saat / istek | Doğrudan gözlenen sonuç |
|---|---|
| 03:13:20 initialize | `lint=true`, UTF-16, `dillsp 0.8.0-dev`, incremental sync=2. |
| 03:13:40 completion id2, `0:15` | UI trace JSON'da 111 aday sayıldı; `ols`→`olsun` için `un` eklendi. Completion popup ayrıca kaydedilmedi. |
| İlk definition denemesi id3, `1:13` | Exclusive-end sorgusu null; IDE fallback id4 `0:1` sorgusu `[0:0,0:4)` döndürdü. Bu yanlış cursor denemesi doğru konum testi yerine sayılmadı. |
| 03:14:01 definition id5, `1:12` | Açıkça doğru konum sorgusu `[0:0,0:4)` döndürdü. |
| 03:14:05 hover id6, `1:12` | `Tanım (satır 1): isim "Ayşe" olsun`; popup görünür. |
| 03:14:22 rename id8, `1:12`, newName=`ad` | Exact `[0:0,0:4)` ve `[1:9,1:13)` aralıkları; editor `ad "Ayşe" olsun\n"😀" ile ad yaz\n`; LSP diagnostics `[]`. |
| 03:14:28 ordinary tanı | `"😀😀" ile eksik yaz\n` girdisinde A001 `[0:11,0:16)`. |
| 03:14:34 format id9 | tabSize=4/insertSpaces=true sonucu `isim "Ayşe" olsun\n"😀😀" ile isim yaz\n`; editor eşit, LSP diagnostics `[]`. |

Dört IDE spelling uyarısı LSP tanılarından ayrıdır. IDE Stop action/context-menu
denemelerinde görünür shutdown/exit üretilmedi. **Bu dalgada ayrı IDE shutdown
trace'i yakalanmadı.** Son normal Quit geçti: hem `CLOSED.json` hem
`restart-CLOSED.json` observed_exit=exit0 ve özgün sahibin reaped=true kaydını
taşır. Bu süreç kapanışı JSON-RPC shutdown trace'i diye sunulmaz; native
UTF-16 stdio'daki normal protokol kapanışı ayrı kanıttır. S4 shutdown trace'i
bu kaynağa kopyalanmaz.

Bu bölüm doğrudan UI gözleminin küratörlü kaydıdır.
`UI-OBSERVATIONS.json` ve `ACCEPTANCE.json` kaynak, hazırlık/controller ve
kapanış dosyalarının hashlerine bağlıdır; `passed-narrow-local-IDE` sonucu
doğrulandı. Tam ham trace dışa aktarımı değildir. Bu belge incelemesi UI
testini bağımsız yeniden yürütmedi; kaydedilen gözlemleri ve hash bağlarını
denetledi.
UI trace'in exact BuildID gösterdiği iddia edilmez; kaynak bağı launcher,
kanonik teslim ve native BuildJSON makbuzlarıdır. Genel UI gecikmesi,
Windows/minimum OS veya gerçek ürün SLA kabulü değildir.

## Kaynak kapıları ve korunan başarısız deneme

İlk kaynak kapısında faz matrisi **998/1023 geçti, 0 kaldı, 25 atlandı** ve
clippy geçti; sonra üretilmiş `islev-egilimi` raporunun bayatlığı toplam
makbuzu `failed` yaptı. İlk makbuz korunur; tamamı başarılı diye yeniden
etiketlenmez. Kanonik rapor yenilendi; ikinci kaynak kapısının remaining
kontrolleri, kaynak pin bağı ve maintenance/provenance kapıları geçti.
Compiler kaynak commit'i ve kabul SHA'sı bu zincirle ayrılır.

Kaynak hazırlık matrisindeki 998/0/25 ile son RC kapısındaki 131 Cargo grubunun
998/0/25 sonucu ayrı gerçek koşulardır; birbirinin yerine yazılmaz veya toplanmaz.
RC'de atlanan 25 sahibin ikisi S kaynağında ayrıca kabul edildi: fixed PDF backend
ve eski in-process LSP. Kalan 23 için bu S dalgasında ek kabul iddia edilmez.
C2 üzerindeki signed PDF/service-fault iki sahibi kendi kaynağında ayrı kalır.

## K-299 — kabul betiklerinin iptal temizliği

K-299, PDF ve service-fault test koşucularının erken iptal/yarım kurulum/cleanup
sınırını düzeltir; production compiler, PDF host ve worker-supervisor değişmez.
İki normal Cargo sahibindeki Python alt vaka sayıları PDF9/service10'dur; yeni
beş Python alt vaka Cargo toplamına eklenmez. Signed PDF9 ve native service2
ayrı temiz K-299 C2 kaynağında kabul edildi. Tarihsel ilk C koşusunda PDF9 ayrı
geçti; service-fault iç kampanya başlamadan reddedildi ve birleşik native1
başarısız kaldı.

İlk tam kapı996/1023 başarılı,2 başarısız,25 ignored; ikinci kapı997/1023
başarılı,1 başarısız,25 ignored olarak korunur. İlkinde compiler/target kökü
ve nested Rust dogfood ikilisi bulunamadı. İkincide yalnız nested Rust dogfood
ikilisinin beklediği paket-local target yolu kaldı. Özel CARGO_TARGET_DIR
zorlaması kaldırıldı; test beklentisi veya kaynak kodu bu iki hata için
gevşetilmedi. Üçüncü gerçek faz matrisi **998/1023 başarılı,0 başarısız,25 ignored**.
Biçim, sayılar, kanıt özeti, clippy, işlev eğilimi, spec-drift, katalog/tazelik,
üç koruk, diff ve sır taramasıyla toplam13 owned adım ve outer exit0/reap geçti.

Bu ortak kapılar ve sürekli güvenlik kapısı precommit aday overlay üzerindedir.
Güvenlikte compiler+fuzz için iki cargo-deny kontrolü, stable fuzz derlemesi ve
5+11=16 libtest vakası geçti; bu16 vaka998 normal test toplamına ayrıca eklenmez.
İki dış controller normal exit0/reaped kaydı taşır. Gate/log/source pinleri ve
19 dosyanın commit blobları bağımsız olarak eşlendi.

İlk K-299 kaynağı C `90fe778e86b9de28046f0f7bd77b47df138a2212`, ağaç
`3bb1340d68a1073bb9cbda71ce943c222959db24`; parent K-298 kabul kaynağıdır.
Bu C kaynağındaki ilk native kampanyada PDF Cargo seçicisi, dokuz iç vaka ve
bağımsız PDF incelemesi geçti. Service-fault Cargo seçicisi exit101 ile kaldı;
libtest süresi0,04 saniye, Cargo komutunun toplam süresi0,629197 saniyedir.
Rust wrapper `CARGO_MANIFEST_DIR.join("..")` ile `compiler/..` içeren bir depo
yolu gönderdi; Python Campaign'in `p.resolve() == p` kontrolü bu değeri iç
hazırlıktan önce reddetti. Service iç proof oluşmadı ve iki native worker/fault
vakası yürütülmüş sayılmaz. Bu, üç saniyelik worker hazırlık eşiği veya timeout
değildir.

Birleşik proof `failed`, özgün controller observed_exit=exit2/reaped=true olarak
korunur. `source_end_clean=false`, başarılı son kaynak kontrolüne ulaşılmayan
alandır; tek başına kirli depo gözlemi değildir. PDF9 ayrı C başarısı, bütün
native1 kabulünü veya sonraki kaynağın kabulünü sağlamaz.

Takip düzeltmesi yalnız Rust test wrapper depo kökünü kanonikleştirdi; Python
sınırı, `compiler/src`, üretim worker/host davranışı ve Cargo owner sayısı
değişmedi. Gerçek yeni kaynak **C2 = `8178920e03fed782a817d1864b8280fcb80812a3`**,
ağaç **`1eccf96d1a407880bf161c55bdd55c4d9a144866`**, parent ilk C'dir.

Dördüncü kaynak/belge kapısı **13 adım**, tam matris **998/1023 geçti,
0 başarısız, 25 ignored**; ayrı `bash scripts/guvenlik-kapisi.sh --surekli`
koşusu da geçti. İki cargo-deny kontrolü, stable fuzz derlemesi ve 5+11=16
libtest vakası bu güvenlik koşusuna aittir; 998 toplamına eklenmez.
Gate4/security2, ilk C üstündeki exact 10 dosyalık aday farkında koştu; commit
makbuzu bu denenmiş içerikleri C2'ye bağlar. Bunlar temiz C2'de tekrar edilmiş
kapılar diye sunulmaz. İki özgün controller normal exit0/reaped kapandı.

Ardından **temiz exact C2 üzerinde native2 geçti**: iki Cargo sahibi, signed
PDF **9** ve service-fault **2** iç vaka. Toplam **20 gerçek komut**, proof dahil
**78 public kanıt dosyası / 611.082 bayt**, başlangıç/bitiş source/tree/temizlik,
CLI ve iç BuildJSON kimlikleri bağımsız incelemelerle eşlendi. Her gerçek komut
ve özgün dış controller exit0 ve reap ile kapandı. Bunlar aynı SHA'dan üretilen
release test ikilileridir; kanonik shipping artefaktı kabulü değildir.

Bağımsız inceleme PDF metin/sayfa/font/görselini ve service fixture'larını
ayrıca doğrular. İç anahtar temizliği ve worker/sentinel yaşam döngüsü
iddiaları hash ile bağlı test koşucusu kanıtına ve gerçek Cargo başarısına
dayanır; gizli çalışma dizinleri veya anahtarlar incelenmedi. İç PDF stderr ve
worker zaman çizelgelerinin tümü public kanıtta bulunmaz. Dış controller'ın
normal kapanışı doğrulandı; 330 saniyelik dış iptal payı makbuz şemasından
bağımsız olarak kanıtlanmış sayılmaz.

C2 birleşik inceleme `K299-C2-ACCEPTANCE-REVIEW.json` SHA-256
`63a12aa14770c89352ac7afc11aae2a6f3662cc6fe70c3157ba037afd69f070b`,
native bağımsız inceleme `K299-NATIVE2-INDEPENDENT-REVIEW.json`
`62583632dc7caa069db226e7d43909a96d0541394561c981aac419f2a2631cef`.
Dış E native proof `p19-k299-native-2/PROOF.json`
`bfca13b42cabd6ff44e4b18b1888b96234644a6001b8bb871116943c1eb148e5`,
controller `p19-k299-native-controller-2.json`
`8bc6ed00260829577ba6a414adb0039f88c258855d8229b8d2c311255caff17c`.

**K-299'un dar kaynak/native/arşiv kapsamı kapalıdır; GB-111 kapanışı bu belge
dalgasına dahildir.** Toplu commit öncesi son belge kapısı ayrıca zorunludur.
İlk C native1 başarısızlığı kendi durumuyla korunur. K-298'in S üzerindeki
RC/soak/IDE/stdio sonuçları C2'ye aktarılmaz; genel P11/P34, native DNS, tüm
fault matrisi veya ürün/platform kabulü kapanmış sayılmaz.

## Korunan ilk uzun koşu ve saat ayrışması

İlk RC controller'ının duvar süresi16904,186977 sn, monotonic süresi1703,131386 sn;
ilk soak controller'ının sırasıyla17145,759184 ve1944,704717 sn ölçüldü.
İki fark da yaklaşık15201,055 sn'dir. Uyku veya saat değişikliği olasılığı bu
kayıtlarla tek başına ayırt edilmedi; kök neden kesinleştirilmedi.

İlk RC **başarısızdır**: lexer_parser timeout girdisi kaydetti; dört hedef/Miri
kapısı tamamlanmadı ve kalan işler özgün sahibin cooperative iptaliyle toplandı.
Timeout bulgusu silinmedi ve saat farkı gerekçesiyle compiler dışı ilan edilmedi.
Saklanan girdi330 bayt, SHA-256
`c64bc23f74a8683048a8b7897a5a34cc2bfb2b652ce4e8cd22dc6bfadd852888`.
Pinned libfuzzer/arbitrary kaynaklarının statik incelemesine göre hedefe geçerli
90 baytlık UTF-8 öneki aktarılır; compiler'ın ham330 baytı işlediği söylenmez.

İlk tekrar yanlış çalışma dizini nedeniyle teste ulaşmadan başarısız oldu.
Doğru dizindeki ikinci tekrar aynı exact kaynak/nightly/ASan ile100 sabit girdi
çalıştırmasını15 ms'de tamamladı, exit0/reaped; timeout yeniden oluşmadı.
Bu tekrar yeni fuzz kapsamı veya compiler hatası yoktur kanıtı değildir.
Cargo fuzz tekrar sırasında test ikilisini yeniden derledi: ilk kayıttaki
SHA-256 `82d01a852448fb7e280448382751a61cc82576e060397dbdb9af691ed9b13a08`,
sonraki ikilinin SHA-256'sı
`ceacd22703793e391042da7ccd216058c14d8cbccb6692d97006fb4bbf0bae76`.
İlk log/proof/girdi/manifestler korunur; aynı ikiliyle tekrar veya ilk build-cache
ikilisinin baytlarının korunduğu ileri sürülmez. Girdi korpusa alınmadı.

İlk soak'ın kendi makbuzu **passed** ve actual süre1800,915283 sn'dir;
compiler/LSP RSS eşiğini geçer, normal exit0/reaped kaydı vardır. Sonradan yazılan
iptal dosyası bu bitmiş koşuyu durdurmuş sayılmaz. Ancak saat ayrışması nedeniyle
bu koşu final kesintisiz kabul olarak seçilmedi. İkinci koşu ayrı dizin ve özgün
süreç sahibiyle başlatıldı; ilk makbuzun durumu değiştirilmedi.

## İkinci RSS koşusunun kabulü

Aynı temiz S kaynağında ikinci koşu **geçti**: istenen 1800 saniyeye karşılık
ölçülen süre **1801,116786 saniye**, ham örnek sayısı **360**; compiler ve LSP
ayrı ayrı **24.072 dönem** tamamladı. Isınma sonrası ilk ve son pencerenin üst
medyanları compiler için **20.112 → 20.016 KiB**, LSP için **19.776 → 20.800 KiB**.
Mevcut formülde azalma sıfıra sınırlanır: compiler büyümesi **0 KiB/%0**;
LSP büyümesi **1024 KiB**, yukarı yuvarlanmış **%6**. Red sınırı olan **%10 ve
32 MiB'ın birlikte aşılması** oluşmadı. Kaynak/tree, ikili pinleri ve temiz
başlangıç/bitiş mevcut doğrulayıcıyla bağlıdır; özgün controller normal
**exit0/reaped** kapandı. Ham 360 örnekten medyanlar, dönemler, eşik ve dosya
hashleri bağımsız salt okunur incelemede yeniden doğrulandı.

Ek saat denetimi **passed, 0 sorun** verdi. Soak komutunun UTC ve monotonic
süre farkı yaklaşık **0,093270 saniye**, controller farkı **0,092673 saniye**;
ikisi de 5 saniyelik tolerans içindedir. Bu ek denetim kayıtlı uç saatlerin
tutarlılığını doğrular; kesintisiz CPU etkinliği veya işletim sistemi uyku
kaydı gözlemi değildir. İlk koşunun saat ayrışması ve özgün sonucu korunur.

Dış E kanıtları: `p19-k298-a57290f-soak-2-result.json`
SHA-256 `727f217adb019b15b652e7b8f997987f49455833fbdee2ab39886c33d32dae0b`,
`p19-k298-a57290f-soak-2/PROOF.json`
`f333bdf3a7130b50d33f3a981a1074485c78e64d9507ab59b544b3f8dcb30a3c`,
`p19-k298-a57290f-soak-2-controller.json`
`7989661a94812c9486b5de7b3709ae3f920ef001bbb63853fd6b62861fe3ea5d`.
[Saat kabul kaydı](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-soak-2/continuous-clock-review.json) ve
[bağımsız ham örnek incelemesi](incelemeler/urunlesme-20261014/k298-a57290f/reviews/p19-k298-a57290f-soak-2/independent-metadata-review.json)
küçük arşivdedir. Gerçek 17 sütunlu satır, `docs/soak-gecmisi-v1.tsv`
tarihçesindeki bu S kaynağı kabulünün dayanağıdır.

## İkinci RC ve tam güvenlik kapısı

Aynı temiz S kaynağında dört hedef explicit AddressSanitizer ile geçti.
Her hedefte istenen süre 1800 saniye, libFuzzer'ın ham son istatistiği 1801 saniyedir.

| Hedef | Çalıştırma | libFuzzer süre, sn | Sahipli komutun monotonic süresi, sn |
| --- | ---: | ---: | ---: |
| lexer_parser | 6.492.608 | 1801 | 1804,039 |
| morfoloji | 7.638.426 | 1801 | 1803,006 |
| http_istegi | 10.696.127 | 1801 | 1803,015 |
| wasm_abi | 8.190.086 | 1801 | 1802,708 |

Toplam **33.017.247 çalıştırma**; kayıtlı crash, timeout veya sanitizer bulgusu yok.
Ondalık **2** ve zaman **1** olmak üzere **3 seçili Miri testi** geçti.
`nightly-2026-08-31`/cargo-fuzz 0.13.2, macOS 26.6.2 ve Apple M4 Pro ortamı
makbuzda sabittir. Derleme kimlikleri ve temiz kaynak başlangıç/bitişi kayıtlıdır.

Kanonik export, check-before, `guvenlik-kapisi.sh --surum-adayi` ve check-after
geçti. Tam Cargo bölümünde **131 grup, 998 başarılı, 0 başarısız, 25 atlanan**
var; bu sayılar ham logdan bağımsız çıkarıldı. Sürekli güvenlik alt grubunun
16 testi eklendiğinde güvenlik logunun tamamı 133 grup/1014 başarılıdır;
normal test toplamı 998 olarak korunur. Miri ve native alt vakalar bu toplama
eklenmez. Kampanya ve post-gate özgün controller'ları normal exit0/reaped kapandı.

Dış RC bundle manifesti **7989 dosya / 11.905.849 bayt** indeksler; SHA-256
`c507496d27e627f9f36c5cc89439b21b411c72f57f79778ee14130313d5b1e10`.
Dört gerçek kanonik integrity/policy kontrolü bundle kapsamını bağlar.
Bağımsız inceleme seçili küçük ham logları ve makbuz hashlerini yeniden
karşılaştırdı; büyük corpus/ikili payload'larını yeniden taradığı iddia edilmez.

Ek saat kabulü **passed, 0 sorun**: kampanya controller'ının duvar−monotonic
farkı **0,103752 saniye**, son kapının **0,004036 saniye**. Bu, 5 saniyelik
pratik tolerans içindeki uç saat tutarlılığıdır. Hedef başına duvar saati çifti
bulunmadığından bireysel saat/CPU sürekliliği veya hiç uyku olmadığı kanıtlanmaz.
İlk RC başarısızlığı ve ilk soak'ın dışlanan sürekli kabulü tarihsel kalır.

[Bağımsız RC incelemesi](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-rc-2/independent-raw-review.json) ve [ek saat kabulü](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-rc-2/continuous-clock-review.json)
source S, kampanya, kapı, controller ve manifest bağlarını birlikte doğrular.

## Son belge kapısı ve takip commit’i

K-298'in dar bakım/ölçüm/RC/RSS/IDE kabulü ve K-299'un dar kaynak/native/arşiv
kapsamı tamamlandı. İlk son belge kapısında dokuz kontrol geçti; onuncu
`sır taraması` adımı exit1 ile başarısız oldu ve son `--surekli` güvenlik
adımı çalıştırılmadı. Özgün controller exit1 ile normal kapandı ve reap edildi.
İlk başarısız koşunun dış makbuzu
`/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-final-docs-1/PROOF.json`
ve controller kaydı
`/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-final-docs-controller-1.json`
korunur. Bu belge kapısı sonucu S kaynağının geçen RC/RSS veya ayrı C2 native
kabulünü değiştirmez. Bildirilen 25 eşleşmenin tamamı, 13 public JSON dosyasında üç belge sürümü, iki eski tarama günlüğü ve güvenlik kapısı betiğinin gerçek baytlarından yeniden hesaplanan altı SHA-256 özetine bağlandı. Bu eşleşmeler gizli kimlik bilgisi değildir; inceleme yalnız bildirilen eşleşmeleri sınıflandırır, tam bir bağımsız sır taraması yapmaz ve başarısız kapıyı geçmiş saymaz. Sınıflandırmanın dış kaydı `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-archive-false-positive-classification.json` yolundadır.

İkinci son belge/sürekli güvenlik koşusu, birleştirilmiş belge ve şeffaf dış
referans görünümünü aynı 11 kontrolle denetler. Toplu commit yalnız aşağıdaki
gerçek dış makbuzlar başarılı olup özgün controller normal kapandıktan sonra alınır:

- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-final-docs-2/PROOF.json`
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-final-docs-controller-2.json`

Bu kapsam koşulu gelecekteki kapının sonucunu önceden ilan etmez. Makbuz,
hazırlanmış belge/arşiv snapshot'ını denetler; kendi hashini arşive veya sonradan
bu dosyaya ekleyerek döngü kurulmaz. Gerçek sonuç teslim mesajında dış makbuzuyla
bildirilir; test edilen belge baytları sonuç eklemek için değiştirilmez.

## Açık ürün ve yerel sınırlar

P19 **kısmi** kalır: artımlı HIR/tür analizi, gerçek ürün ve UI gecikmesi,
genel SLA, Windows/eklenti yayını açık. [Editor normu](../spec/30-editor-analizi-ve-lint.md)
yeni temiz ölçüm ve uzun süreli RSS kanıtını birlikte incelemeyi ister;
S kaynağının dar stdio ve ikinci RSS kabulü artık birlikte vardır. Tam P19
veya 39 maddenin tümü kapanmaz.

Kalanların tümü kullanıcı girdisi değildir. Signed PDF ve worker kabul
zincirlerinin cooperative iptal/finally/özgün süreç kapanışı K-299 ayrı kaynak
işinde ele alındı. İlk native1'de ortaya çıkan kanonik test-wrapper kökü
düzeltmesi C2'de kabul edildi ve dar arşiv kapsamıyla kapandı. Bu değişim
K-298 donuk S kaynağına gizlice eklenmedi.
Bu dalgada yeni dış IDE hazırlığı kullanıldı; eski hazırlayıcının lifecycle
bulgu kaydı tarihsel olarak korunur. İki normal Quit makbuzu ve
küratörlü UI kabul bağlantısı yukarıdaki dar kabulde doğrulandı. C1/K-288 ve P33/K-293 otomatik araç incelemesi retleri
korunur; yeniden deneme veya alternatif araç yoluna girilmedi.

Beş tam anonim proje henüz hazır değil; fiziksel cihaz yok. Gerçek 15 insan
çalışması, ürün provenance/staging/adoption, GitHub destek yanıtı,
11 Kasım 2026 syntax gözlem penceresi ve yayın/imza/lisans sınırları
[kullanıcı girdileri listesinde](urunlesme-kullanici-girdileri.md) ayrı kalır.
Mevcut kullanıcı teyitleri tekrar sorulmaz. [S4](urunlesme-s4-kabul-20261015.md)
kendi 4b7dd76; [S3](urunlesme-s3-kabul-20261015.md) native/Linux/PG kabulü
kendi 66e4149 kaynağındadır. Yeni ölçüm bunları a57290f'ye taşımaz.

## Belge etki incelemesi ve kanıt kaydı

Bakım semantic/grammar/API/tanı/bağımlılık davranışını değiştirmediği için
spec normları, RFC indeksi, hata kataloğu, public API ve katman/çevrim
fixture'larında yeni sözleşme gerekmez. ADR-145 ile indeksi, ölçümler,
editor analizi, backlog, 39 madde planı, master/v1/sürümler/README ve kullanıcı
girdileri bu belge dalgasında gerçek dar sonucu izler. ADR-045/049 etkisi
incelendi; paylaşımlı ölçümden adanmış baseline üretilmez. Son belge kapısının
sonucu yukarıdaki dış makbuzdan izlenir; arşiv ve rapor kendisinin sonraki
kapı sonucunu içermez.

Dış kanıt kökü E: `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme`.
Aşağıdaki hashler tamamlanmış makbuz dosyalarından hesaplandı. Seçili küçük
[arşiv](incelemeler/urunlesme-20261014/k298-a57290f/README.md),
[manifest](incelemeler/urunlesme-20261014/k298-a57290f/CURRENT-ARCHIVE-MANIFEST.json)
ve [ham kapsam](incelemeler/urunlesme-20261014/k298-a57290f/RAW-SCOPE.json)
bu kaynak kimliğine bağlıdır. Özgün küçük seçki dış kanıt snapshot’ı
`/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-original-small-archive` altında **137 dosya / 2.101.349 bayt**,
**133 seçili kayıt ve 270 hash bağı**, manifest SHA-256
`6e13b044becd1aee8c4d021bc47e3141c132b295c9e3aec8f6a2679016a1c265` ile
korunur. Depodaki şeffaf dış referans görünümü **137 dosya /
1.599.851 bayt** içerir; kendi manifest SHA-256 değeri
`aafb1ac5dfbe2c552f4beab0a5c7dd6245d4b21e3e4403229a1adae0f26edd2f` ve kapsamı özgün seçkiden ayrıdır.
Bu görünümde 120 dosya özgün baytlarıyla, 13 dosya okunabilir dış referans olarak bulunur; README, RAW-SCOPE, SELECTION ve manifest yeniden üretilmiştir. Her dış referans, korunmuş özgün dosyanın açık yolunu, SHA-256 değerini ve bayt sayısını verir. Özgün 133 kayıt ve 270 hash bağının tümü bu görünümün kendi içinde bulunmaz; bu bağlar dış özgün seçki üzerinden doğrulanır. Tarayıcı kuralı veya ignore istisnası eklenmez, özgün içerik kodlanarak ya da gizlenerek temsil edilmez.

S kaynağının source/delivery/install/UTF-16/stdio/extra/IDE/soak/RC kabulü ile
ayrı C2 kanıtları ve başarısız denemeler kaynak rolleri korunarak izlenir.
Aşağıdaki SHA-256 değerleri özgün makbuz baytlarına aittir; bir bağlantı dış
referans kaydına açılıyorsa o kaydın dosya hash'i olarak yorumlanmaz. Dış kaynak
makbuzlarının erişilebilirliği yerel kanıt köküne bağlıdır. Büyük ham
stream/frame/log, ikili, PDF ve corpus dış kapsamda kalır; bu görünüm tam ve
bağımsız taşınabilir ham kanıt arşivi değildir. Son belge kapısı makbuzu bu
arşivin dışındadır.

| E altındaki makbuz | SHA-256 |
|---|---|
| [p19-k298-accepted-source.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-accepted-source.json) | `2fc817b7d6fd87927b7ead01db6e3a8d08931138ce3164f5edb1bddf4e1276e1` |
| [p19-k298-source-commit.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-source-commit.json) | `38e1effcbdf0acdd737b7195476375421df70873b15a5420e29309b4f59e00c5` |
| [p19-k298-source-gates-1/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-source-gates-1/PROOF.json) | `6dc8cfd667035af5a516e51c97520274fc602930c4a51f0933e88945cc074a78` |
| [p19-k298-source-gates-2/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-source-gates-2/PROOF.json) | `6144b5a628781b5be67a87ff876650e026e23e2bcc07f5f340c552cadb2a3ed7` |
| [p19-k298-provenance-gates-1/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-provenance-gates-1/PROOF.json) | `8e52fdb957e23edb386b7ec99e6a6b1537e2ba965462a00f5ef7d03a05a127d3` |
| [p19-k298-baseline-stdio-1/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-baseline-stdio-1/PROOF.json) | `c21ede269e48dbceac2d471c38d639eedf4885b0a9c0946a06b44d31df9ae3fc` |
| [p19-k298-baseline-stdio-1-independent-review/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-baseline-stdio-1-independent-review/PROOF.json) | `11b04c7e182dcd7c155f4774e74ef8cef06c97f61eb112303d84ba9ecbce34ae` |
| [p19-k298-candidate-stdio-1/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-candidate-stdio-1/PROOF.json) | `814ecdaef008475cd08e3b34e56cf1de8e25a08935566634de7a3c434c319aa7` |
| [p19-k298-candidate-stdio-1-independent-review/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-candidate-stdio-1-independent-review/PROOF.json) | `e171fdec24a381581385724aee7a1ffffd16117976564003b7bce78d0c02b96e` |
| [p19-k298-a57290f-macos-delivery/DELIVERY.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-macos-delivery/DELIVERY.json) | `30c3e97dd2ff1831cc9abc97d9b5197dc5f64021e4efd16afd9899f28671cdcb` |
| [p19-k298-a57290f-macos-install/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-macos-install/PROOF.json) | `aeb03c8dda60a52ee1dbfbe92702d5694da27b7c3aafc7451363366c68c1f17e` |
| [p19-k298-a57290f-utf16-stdio/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-utf16-stdio/PROOF.json) | `7e0d18a483233ae09dca0aec0321abd223d90b975239f5c73acee0fa4b09fe92` |
| [p19-k298-extra-1/PROOF.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-extra-1/PROOF.json) | `41d5fad656de902c3532ab85afd10da4521cbaadce8231db81353b624a71ce47` |
| [p19-k298-extra-result.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-extra-result.json) | `c0946c7c26044a4e03ba0c6c614f83c56af29a1ced8a6a5ccf7e098ae7fc9bbb` |

Makbuzların manifest/pin/sample/closure bağları ayrıca salt okunur incelendi.
Bu belge incelemesi UI testini yeniden yürütmedi; küratörlü gözlemler ve
kaynak/ikili/kapanış hashleri esas alındı.

IDE hazırlık/kapanış makbuzları (E/p19-k298-a57290f-owned-ide altında):

| Makbuz | SHA-256 |
|---|---|
| [PREPARATION.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-owned-ide/PREPARATION.json) | `78f747adb481bb1028388dad06d91f0c6a7efefd206d3d59d505d4eba2b33972` |
| [LAUNCHER-PREPARATION.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-owned-ide/LAUNCHER-PREPARATION.json) | `a7fa4f936267f0d90b9d6f42ac9b2856d1c5cde67e389615e134c825dbef7cf1` |
| [CLOSED.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-owned-ide/CLOSED.json) | `539a6050047e5fc46f2adb6774869bd27d7c9134c1443d4483b99404a696684d` |
| [restart-CLOSED.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-owned-ide/restart-CLOSED.json) | `bc2d0cb8b3022c69f1d79b35b74d07aba0f97eef2ff67540db8d5b1534962950` |
| [UI-OBSERVATIONS.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-owned-ide/UI-OBSERVATIONS.json) | `f673b12b93978f432481bd3357890d9eb60dba9dc19ce267fe79c5de3e29cb85` |
| [ACCEPTANCE.json](incelemeler/urunlesme-20261014/k298-a57290f/receipts/p19-k298-a57290f-owned-ide/ACCEPTANCE.json) | `ca7f4a6b66feda3dc29f2e5b6558364b2dae4cdd1239b3ca414325071d534cff` |

<!-- BEGIN GENERATED OPEN ITEMS APPENDIX -->
## 39 maddenin açık sınırları ve kullanıcı girdileri

Sizden gereken somut girdiler aşağıdaki altı gruptur. Beş proje ve cihazın
henüz hazır olmadığı, sınırlı pilotun seçildiği ve lisansın bekletildiği mevcut
bilgidir; bunlar yeniden karar talebi değildir.

| Kod | Hazır olduğunda gereken |
| --- | --- |
| U1 | **Beş tam anonim proje + çekirdek kökü:** yerel paket yolları, exact ürün commit/provenance bağı, compiler pini, giriş kökü, kurulum ve `denetle`/`dene` komutları, beklenen sonuçlar ve anonim servis/fixture bilgisi. Küçük örnekler tam proje yerine sayılmaz. |
| U2 | **Temiz fiziksel Windows ve minimum OS macOS:** cihaz erişimi veya exact kaynak/ikili/OS bilgisiyle kurulum, program, LSP ve kaldırma kayıtları. Geliştirici Mac ve VM/container bu kabul değildir. |
| U3 | **8–14 yaş aralığında 10 öğrenci/çocuk + 5 profesyonel:** hazırlanmış protokole uygun gerçek anonim katılımcı formları ve oturum özeti. |
| U4 | **GitHub destek:** [hazır destek talebinin](incelemeler/k215/github-destek-talebi.md) tarafınızdan gönderilmesi; talep kimliği, tarih ve PR refs/cache/Actions kopyalarının kapsamını açıklayan yanıt. Gönderilmiş olması temizlik teyidi değildir. |
| U5 | **Seçilmiş pilot/staging:** ürün, hedef teknik dilim, sağlayıcı/test ortamı, izin verilen işlemler, beklenen davranış ve göç/geri dönüş kabulü. Canlı sırlar bu rapora yazılmaz. |
| U6 | **Yayın aşamasında:** lisans, yayıncı ve imza güveni, dağıtım kanalı ve belirli yayın işleminin yetkisi. Bugün yeni bir genel yayın veya SLA kararı gerekmiyor. |

| ID / konu | Şimdiki durum | Açık kalan sınır / sonraki iş | Kullanıcı girdisi |
| --- | --- | --- | --- |
| P01 — Kaynak/geri yükleme | S üzerinde yerel kaynak arşivi kabulü var. | Uzak CI ve canlı artefakt kanıtı ayrı; bütün platformlar kapanmadı. | Şimdilik yok. |
| P02 — Kanonik ikili | S macOS teslimi ve provenance kabulü var. | Yeni kaynak Linux kabulü, fiziksel platform ve gerçek yayın ayrı. | U2, zamanı gelince U6. |
| P03 — CLI çıkış/argüman | Dar CLI düzeltmesi ve regresyon kaydı mevcut. | Bu incelemede bu dar kapsam için yeni açık iş saptanmadı; genel CLI/ürün kabulü çıkarılmaz. | Yok. |
| P04 — Önceki sürüm geçişi | Kısmi; önceki 0.7 korpusu tarihsel kanıtlı. | Tam ürün/kanal geçişi ve eski CLI sınırları açık. | U1. |
| P05 — Uyumluluk | Kısmi; yüzey/deprecation kapıları mevcut. | Tam ürün uyumluluğu, K-086 geçişi ve yayın kanalı kabulü açık. | U1, sonra U6. |
| P06 — Teslim/kurulum | S macOS teslimi ve kurulum8 kabulü var. | Yeni kaynak Linux/native kabulü, fiziksel platform ve yayın açık. | U2, sonra U6. |
| P07 — Proje launcher/seçici | Kısmi; yerel launcher ve tarihsel yönetim testleri var. | Gerçek yayıncı güveni, indirme/global PATH, Windows ve tam sürüm yöneticisi kapsamı açık. | U2, U6. |
| P08 — Gerçek ürün korpusu | Beş tam anonim proje bekleniyor. | Ürün başına gerçek `denetle`/`dene` N/N, süre/RSS kabulü yapılacak. | **U1.** |
| P09 — Kuyruk/dispatcher | Kısmi; S3 native kanıtı tarihsel. | Gerçek ürün ve sağlayıcı benimsenmesi; genel exactly-once garantisi yok. | U1 + U5. |
| P10 — SMTP/PG hata sınırları | Kısmi; S3 fault11 tarihsel. | Canlı sağlayıcı, gerçek katalog ve ürün entegrasyonu açık. | U1 + U5. |
| P11 — PDF/FFI | Kısmi; temiz C2 PDF9 ve bağımsız incelemesi geçti, K-299 dar kapsamı kapandı. İlk C başarısızlığı korunur. | Gerçek katalog, genel FFI/sandbox ve yayın ayrı. | Ürün için U1 + U5. |
| P12 — Göç/ürün betikleri | Kısmi; dar yerel göç/rollback kanıtı var. | Gerçek üründeki Python betiğinin kaldırılması ve göç kabulü açık. | U1 + U5. |
| P13 — Sürekli HTTP servisi | Kısmi; S3 native HTTP kanıtı tarihsel. | Üretim/pilot benimsenmesi ve platform kabulü açık. | U1 + U5; platform için U2. |
| P14 — Karma sürüm geçişi | Kısmi; temiz K-300 `549dbe6` native9/dört dalga/160 istek geçti. | Genel SLA, DB şema devri ve gerçek ürün kabulü açık. | Ürün için U1 + U5; platform için U2. |
| P15 — Parola/login | Kısmi; S3 parola/TLS kanıtı tarihsel. | Gerçek login, göç, sağlayıcı ve ürün kabulü açık. | U1 + U5. |
| P16 — Form/Turnstile | Kısmi; hermetik form testleri mevcut. | Gerçek staging ve tam ürün kabulü açık. | U1 + U5. |
| P17 — Depolama/IAM | **Ertelendi.** | Gerçek çok sunuculu ihtiyaç kanıtı olmadan yeni kapasite işi açılmıyor. | İhtiyaç doğarsa U1 + U5. |
| P18 — C1 tanı/quickfix | **Kısmi; araç incelemesi sınırı açık.** | Beş özel tanı/quickfix zinciri kabul edilmedi; lint bu kabulün yerine geçmez. | Ek izin/dosya isteğiyle kapanmaz; aşağıdaki araç sınırı. |
| P19 — LSP/editör | **Kısmi; K-298 dar stdio/RSS/RC/IDE kabulü kapandı.** S UTF-16 işlev kanıtı da var. | Artımlı HIR/tür, UI/gerçek ürün gecikmesi, SLA ve Windows/yayın açık. | Teknik kapsamın tümü girdi engeli değil; ürün U1, platform U2. |
| P20 — SQL modülleri | Kısmi; S3 native modül kabulü tarihsel. | Gerçek ürün kullanımı açık; DB+FS atomikliği ve genel extension point vaadi yok. | U1 + U5. |
| P21 — TR–EN/bileşen | Kısmi; S3 HTTP/Zee kabulü tarihsel. | Gerçek sayfa/ürün benimsenmesi açık; yerleşik çeviri kapasitesi eklenmedi. | U1 + U5. |
| P22 — Kaynak kapsamı/test köprüsü | Kısmi; S3 dar kapsam kanıtı tarihsel. | Gerçek ürün test kapsamı; fresh DB bütün HTTP işlemlerinin rollback'i değildir. | U1 + U5. |
| P23 — İçerik/starter | Kısmi; S3 PG/HTTPS kabulü tarihsel. | Gerçek starter ürünü, insan, üretim ve platform kabulü açık. | U1 + U3 + U5; platform U2. |
| P24 — Destek politikası | Sınırlı pilot belgesi kabul edilmiş. | Güvenilir yayın imzası, gerçek yayın ve bildirim süreçleri ayrı. | Zamanı gelince U6. |
| P25 — Sır/credential yaşam döngüsü | Kısmi; önceki dar owned profiller kanıtlı. | Gerçek ürün/sağlayıcı, Windows ve tam sır yaşam döngüsü kabulü açık. | U1 + U5; platform U2. |
| P26 — HTML içe alma | **Ertelendi.** | Gerçek HTML import ürünü olmadan temizleyici/API geliştirilmiyor. | İhtiyaç doğarsa U1. |
| P27 — İnsan çalışması | Gerçek katılımcı kanıtı yok. | Hazır kitin 15 gerçek kişiyle uygulanması ve sonuçların değerlendirilmesi gerekiyor. | **U3.** |
| P28 — Syntax freeze | **Takvim kapısı açık.** | **14 Ekim–11 Kasım 2026** sıfır-syntax gözlem penceresi gerçekten tamamlanmalı. | Belge teslimi değil; tarih + gözlem koşulu. |
| P29 — Fiziksel platform | Temiz fiziksel cihaz kanıtı yok. | Windows ve temiz/minimum OS macOS kurulumu/çalışması/LSP/kaldırma kabulü gerekiyor. | **U2.** |
| P30 — Sürüm adayı | Kısmi; S RC2 dört ASan/Miri3/tam kapıyla geçti. S4 tarihsel. | Ürün, insan, platform, takvim, imza ve yayın kapıları ayrıca gerekli. | U1–U3, U5–U6; P28 takvimi. |
| P31 — PG/TLS/göç | K-302 temiz `e6322db` üzerinde TLS8/göç1 geçti; S3 kendi kaynağında tarihsel. | Uzak CI ve gerçek ürün kabulü açık. | Yerel K-302 için yok; ürün için U1 + U5. |
| P32 — GitHub kopya teyidi | Harici yanıt yok. | PR refs/cache/Actions temizleme kapsamının GitHub tarafından teyidi gerekiyor. | **U4.** |
| P33 — Güvenlik kanıt matrisi | **Kısmi; salt okunur matris mevcut.** | Koordineli reproducer reddi nedeniyle tam kapsam açık; istismar gözlenmiş denmiyor. | Ek izinle kapanmaz; aşağıdaki araç sınırı. |
| P34 — Worker/fault | Kısmi; temiz C2 service2 ve K-299 dar bakım kabulü geçti. S3 ve başarısız ilk C kayıtları ayrı. | K-301 yerel test doğruluğu kapalı: worker25/tekCargo ve mock10/20 geçti. Native DNS, tam fault matrisi ve gerçek ürün benimsenmesi açık. | K-301 için yok; gerçek ürün için U1 + U5. |
| P35 — Ergonomi/gramer | **Ertelendi; RFC aday.** | İnsan çalışması, tekrarlanan ürün ihtiyacı ve freeze kapısı olmadan yeni syntax yok. | U1 + U3; ayrıca P28. |
| P36 — Test/ignored kapsamı | S/C2/K-300 sonuçları kendi kaynaklarında; K-302 temiz `e6322db` seçili9/25 geçti. | K-302 kampanyasında kalan16 ayrıca çalışmadı; farklı kaynakların sayıları birleştirilmez. | Yerel PG devamı için yok; diğer kabul kapsamına göre cihaz/ürün/servis girdisi gerekebilir. |
| P37 — Sürekli kalite kapıları | **Sürekli iş; tek seferde kapanmaz.** Bu dalganın dar kanıt/arşiv kapsamı hazır. | Her değişiklikte ilgili kapılar; belge takip commit’i gerçek dış kapı makbuzuna bağlıdır. | Yerel işler için yok; bağlı dış kapılar ayrı. |
| P38 — Backend/WASM/self-hosting | **Ertelendi.** | Native WASM/backend/self-hosting gerçekleştirilmiş sayılmıyor; V1 ve ihtiyaç koşulları korunuyor. | Şimdi yok. |
| P39 — Lisans/yayın kararı | Kısmi karar: sınırlı pilot seçildi, lisans bekletildi. | Mevcut marka/site korunur; public/yayın kararı henüz verilmedi. | Zamanı gelince **U6.** |

Bu dalganın kaynak, ölçüm, RC, RSS, native ve küçük arşiv kabulleri tamamlandı.
Birleştirilmiş belgelerin son kapısı ve takip commit’i gerçek dış makbuzla
izlenir. P19'un artımlı HIR/tür ve UI gecikmesi ile tabloda kalan ürün
kapasiteleri ayrıca teknik işlerdir; bütün kalanlar kullanıcı girdisi
eksikliği değildir. 39 maddenin tamamı kapalı olarak işaretlenmez.

**Araç sınırı:** Önceki otomatik onay incelemesi C1/K-288'in beş özel
tanı/quickfix çalışmasını ve P33/K-293'ün koordineli güvenlik reproducerını
reddetti. Bu kapsamlar açık tutuldu; burada yeniden denenmedi veya başka
araç/ajan yoluna yönlendirilmedi.

Kaynak liste: `docs/urunlesme-plan-v1.tsv` P01–P39; teslim ayrıntıları:
`docs/urunlesme-kullanici-girdileri.md`. Bu tablo raporda belirtilen kaynak ve makbuzlara dayanır; son belge kapısının
sonucu yukarıdaki dış makbuzdan izlenir.
<!-- END GENERATED OPEN ITEMS APPENDIX -->
