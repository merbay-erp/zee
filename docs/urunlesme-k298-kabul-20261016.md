# Ürünleşme — yerel kabul ve açık teknik işler raporu

> **Güncel durum:** K-298–K-305 dar kabulleri tamamlandı. K-306’nın temiz kaynak kaydı, kontrollü modelleri ve gerçek VM/EOF sahiplik denemeleri geçti; K-306/GB-115 açık; sahipli unit kaynak/test dilimi uygulandı; yeni exact-source native kabulü ve kapanış bekleniyor.
> **Kalan yerel adım:** Uygulanan minimal kaynak+test+belge diliminin tam15 kaynak kapısı ve exact temiz commit kaydı; ardından fresh dispatcher4/HTTP7 kabulü.
> **Sizden beklenenler:** Beş anonim tam proje ağacı ve kaynak commit’leri; fiziksel Windows ve minimum desteklenen macOS cihazları. İnsan çalışması, sağlayıcı/pilot ve yayın girdileri ilgili aşamalarında ayrıca listeleniyor. 39 maddenin tamamı kapanmış sayılmıyor.

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
Bağımsız tarihsel incelemede ikinci kapının 11 günlüğü, dış exit0/reaped
kaydı ve `a31f12a` commit'inin 164 exact dosyası doğrulandı. Commit adaptörünün
ayrı dış süreç makbuzu bulunamadı; bu dar lifecycle alanı **unverified**.
İlk başarısız sır taraması kaydı korundu. Audit, E/k304-related-audits-1
altındaki `K298-K299-FINAL-DOCS2-AUDIT.json` dosyasındadır.

K-302 TLS8/göç1, K-303 havuz1, K-304 direct web4 ve K-305 ayrı köprü
kabulleri kendi exact kaynaklarındadır; K-305 son11/temiz C11 kayıt zinciri
de tamamlandı. K-306 iki Linux kabul koşucusunun iptal/PG sahipliği
düzeltmesidir; C12 V3 dispatcher Cargo/libtest sonucu yapılandırılmış dispatcher4 kabulü değildir; HTTP çalışmadı. Yeni exact kaynakta dispatcher4/HTTP7 ve kapanış kapıları bekleniyor. Kalan beş PG tüketicisi, fiziksel platform
U2 ve gerçek ürün kabulü ayrı açık işlerdir.

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

Son belge/güvenlik kapısı **11/11 geçti** (69,865 sn); özgün controller
exit0/reaped. Dokuz belge `47812bfe8312d5a5a439931bf49a079a7b0b22db`
commit'iyle temiz kaydedildi. `E/k302-gates-1/PROOF.json` SHA-256
`5157f531e2518ac6305422ccb53ba2eb4af17f6488c72ca67f6fb18888ff5a5d`;
`E/k302-commit-1.json` SHA-256
`4d94e63ea38967df1e42817f1204d0509e0c22dc1a0186fcdec3176fdc14a715`.

Belge etki incelemesi: README, master plan, backlog, sürüm kapıları,
sürüm notları, PG rehberi, kanıt haritası ve ürünleşme planı güncellendi.
Normatif davranış değişmediğinden spec/RFC/ADR ve indeksleri, tanı kataloğu,
compiler/core-freeze/regression kayıtları, katman grafiği, test sayısı/faz
sahipliği değişmez. Performans iş yükü/ADR-045/049 ve ölçüm tarihçesi,
RC/soak tarihçesi, artefakt ve kurulum sözleşmesi de değişmez.

## K-303 — Mevcut PostgreSQL havuz kabulü

**Dar yerel kabul tamamlandı.** Temiz kaynak
`47812bfe8312d5a5a439931bf49a079a7b0b22db`, ağaç
`b24875e03028e55dd60b73116e67501b536f4ddb` üzerinde mevcut tek havuz
Cargo vakası **1 başarılı/0 başarısız/0 atlanan** ile geçti (4,38 sn).
Tüm kabul sürücüsü 14,745 sn; özgün dış controller 15,063 sn, exit0/reaped.

Ham çıktıdaki 16 metrik kaydı dört bağlantı doluyken beşinci isteğin
beklediğini, bağlantı dönüşünü ve beklenen tek timeout sayacını doğruladı.
Son durumda dört bağlantı boşta, kullanımda/bekleyen sıfırdı. Bu işlevsel
kabul, performans eşiği veya genel yük/SLA ölçümü değildir.

Tek özel PG16.15/TLS1.2 sunucusu özgün süreç sahibince normal SIGINT
kapanışıyla exit0/reaped; geçici veri/anahtar kökü kaldırıldı. Başlangıç/son
kaynak temiz ve eşit, 12 kaynak pini değişmedi. Ham loglar ve kapanış
bağımsız olarak doğrulandı. Production veya test kaynakları değişmedi.

| Özgün makbuz (E altında) | SHA-256 |
| --- | --- |
| `k303-pg-pool-accept-1/PROOF.json` | `07eb31f5bf6e226251e05bf8dcd4bdd7f9624c3ac0a7d70867b0a830493bf0f6` |
| `k303-pool-controller-1.json` | `ec1a7eaa406040b53e713292dcd821b90a9b9eccaf98eb0b4bf272c33b4e6042` |
| `k303-driver-evidence-1/K303-INDEPENDENT-REVIEW.json` | `07308869b2cd48a432a86906767602b45d2fedbc917a8c8ff5f148fe21092dcc` |

Bu kampanya bu kaynakta **25 explicit ignored sahibinden yalnız birini**
çalıştırdı; kalan24 bu kampanyada çalışmadı. K-302'nin C5 üzerindeki dokuz
vakasıyla aynı kaynak toplamı yapılmaz. Native metrik tüketicisinin sekiz
alt vakası, diğer PG tüketicileri, uzak CI/provider/COMMIT veya gerçek ürün
kabulü çıkarılmaz; P13/P36 genel kapsamı kısmi kalır.

Son belge/güvenlik kapısı **11/11 geçti** (70,759 sn); özgün controller
exit0/reaped. On iki belge `a5cdbc5b3d2c66685e48bd34c320b7b5e0db58ce`
commit'iyle temiz kaydedildi. `E/k303-gates-1/PROOF.json` SHA-256
`af9c17e0301f6981af99ba5444f313b951e1075e7cf35ddad08a26e07d6acde9`;
controller `15f238cc11ee8deb093694aa0c85ca8c6f1e73adbf61c358f930ce138d7c00da`;
`E/k303-commit-1.json` SHA-256
`9cf34be78d086481ea2164dafa2e9eee93264efb4ef45b4e0a0fb662cc960f4e`.
Bağımsız inceleme 11 günlük hash'ini, 12 belgeyi ve kaynak pinlerini doğruladı.
Spec/RFC/ADR normları ve indeksleri,
tanı kataloğu, compiler/core-freeze/regression kayıtları, katman grafiği,
test sayısı/faz sahipliği, performans/RC/soak tarihçesi ve kurulum sözleşmesi
incelendi; yalnız mevcut kabulün kaydı olduğundan değişmedi.

## K-304 — Owned web koşucusunda geç iptal: dar kabul tamamlandı

**K-304 ve GB-113 dar kapsamda kapalıdır.** Son vaka başarısı cleanup bekleyen
ara durumda tutulur; geçici dizin/command-owner kapanışı, proof yazımı ve son
need kontrolünde gözlenen iptal veya süre aşımı başarısız makbuz/nonzero
sonuçta korunur. Süreç çıktıktan sonraki sinyal için gözlem garantisi yoktur.

- Özgün kaynak altı kontrollü karşı örnekte hatalı `passed` üretti. İlk
  önerinin son need kontrolünde iki ek karşı örnek bulundu; her iki aşamanın
  başarısız günlükleri korunur. V2 sekiz olumsuz alt vaka ve normal kontrolü
  iki Python metodu altında geçti. Bu mock gerçek PG/HTTP veya OS sinyal
  teslimi kabulü değildir.
- Mevcut Cargo hazırlık sahibi **22 Python kontrolünü doğrulayarak geçti**.
  Kaynak öncesi **15/15 kapı geçti** (1030,946 sn): tam faz **998/1023 geçti,
  0 başarısız, 25 atlanan**, Clippy, katalog/tazelik, spec/fmt, koruklar,
  sır taraması ve sürekli güvenlik. Faz komutu 821,014 sn sürdü; bu hazırlık
  sonucu ayrı RC/soak/IDE kabulü değildir.
- Test edilen **19 dosya** temiz `abdc5cf36b4d55021df5edad08cee3a7ad953ebd`
  commit'iyle kaydedildi; ağaç `b44b4643b982f8063afedb1e75ada655cc2fe96d`.
  Kaynak kapısında 191 pin ve exact commit içerikleri doğrulandı.
- Bu exact temiz kaynakta mevcut **tek ignored Cargo sahibi** dört gerçek
  web senaryosuyla geçti: **1 başarılı/0 başarısız/0 atlanan**, 4,76 sn.
  Site snapshot, oturum/CSRF, commit/istek rollback ve fresh DB izolasyonu:
  toplam **19 HTTP isteği + 6 SQL gözlemi = 25 adım**; iki PG kümesinin
  kimlikleri farklı ve observer salt okunurdu.
- Native sürücü 21,643 sn; özgün dış controller **exit0/reaped**, 22,077 sn.
  Başlangıç/son kaynak temiz, 24 kaynak/fixture ve 17 runtime pini eşitti.
  BuildJSON `abdc5cf36b4d55021df5edad08cee3a7ad953ebd` / `dirty=false`;
  ikili SHA-256 `e250c7977806958bef836b8ee5f64934f1cb1fd096654e4848073ec6bddcfd34`.
- Dört vaka için worker/PG grupları ve son command grupları boş kaydedildi.
  Kanıt, kanonik foreground/Popen/WNOWAIT uygulaması ile bu makbuzlara
  dayanır; çocuk başına ham exit veya özel PG dizini kaldırılması bağımsız
  gözlem değildir. K-303'ün normal SIGINT/PG exit0 iddiası buraya taşınmaz.

| Özgün makbuz (E altında) | SHA-256 |
| --- | --- |
| `k304-source-gates-1/PROOF.json` | `ad51c13916f66c44c41eab151cf27f9c875a08056bf4b56a9f8c87d891f96abd` |
| `k304-source-gates-controller-1.json` | `3758abcfecbe13a904a1ad261285706f4c56ccd58bcab615c96356f6bac42a4d` |
| `k304-source-commit-1.json` | `322dadf7ef53555b77a1ce543f49a7eead3e99e430899582d2e02e4000f9805d` |
| `k304-source-commit-controller-1.json` | `f5afcf4a40e8e8f6cbec3868ad7cf1a40d25032d5f11a62cc3f56684bb50f6f4` |
| `k304-owned-web-accept-1/PROOF.json` | `c72fefacb2133310c49feeba069e6bff64b67f033eda075abd8ddb40e4042efc` |
| `k304-owned-web-controller-1.json` | `26489de1dd9af962f3b0e5fa43df7a2e3091676e6c2597edf5b5c8bcc60a25ed` |
| `k304-owned-web-accept-1/NATIVE-PROOF.json` | `9c183b97af91d70b0f879a2f13b9aa650bec84a113139f0b92fc901e5b13e8f5` |

Dar hazırlık makbuzu `E/k304-preparation-2/v2/K304-PREPARATION.json`, SHA-256
`742ea73df57fd0daeb8109915292475e9234ab836b8b5ae13551b4a29d5f7d8c`
ile ham eski/V1/V2 sonuçlarını bağlar. Dondurulmuş yedi makbuz haritası
`E/k304-acceptance-record-1.json`, SHA-256
`953d15fcd58c6fefdaefdec2f99034be5938640e5cbce9fe0e4caa0d00fbb315`.

Bu kaynakta 25 ignored sahibinden yalnız biri ayrıca çalıştırıldı; kalan24
bu kampanyada çalışmadı. Native alt vakalar Cargo toplamına eklenmez.
Ayrı `dene`/source-coverage sahibi, TLS/göç/havuz, gerçek ürün/platform ve
önceki RC/soak/IDE kabulleri bu sonuca taşınmaz. P22/P36 kısmi kalır.

Son **11/11 belge/güvenlik kapısı 67,041 sn içinde geçti**; özgün dış süreç
exit0/reaped. 15 belge, `95236cbe2d34f7497980ea3292b1f8451245a767`
commit'iyle (tree `5bce8917431491a2ee0276f7ef69771e5ec94418`) kaydedildi;
205 kaynak pini değişmedi. Bağımsız son inceleme de geçti.

| Son kayıt (E altında) | SHA-256 |
| --- | --- |
| `k304-final-gates-1/PROOF.json` | `3329e1dde366ba000c05d6c97f24beb6bd870ef87bb79737170b7c1fda4aa5a0` |
| `k304-final-gates-controller-1.json` | `fd00745d7344393a17681ce872646e225e346f5051d32bec82c5f059cb91041b` |
| `k304-final-commit-1.json` | `89c2258d3707ce94ddedf3d29c54e6e84a961d71d02fabeb0f7290a72eeae993` |
| `k304-final-commit-controller-1.json` | `fc437c54f33492ff1508c3dac36ffbc5a1e84a01b887b41889606b417d65b514` |
| `k304-final-audits-1/K304-FINAL-CLOSURE-INDEPENDENT-REVIEW.json` | `750863c0ad63ce71c5aac556cff4d0df582bb4fb2ddcf511f2e5d66e90b7a613` |

ADR-129/144, indeks, kit/köprü rehberleri, README/backlog/master/sürümler,
v1 kapıları, güvenlik/kanıt haritası ve 39-plan güncellendi. Faz açıklaması
ve üretilen faz belgesi yenilendi; Cargo sahipliği aynı. Üretim Rust,
grammar/API/tanı/katman/bağımlılık, sabit performans iş yükü, spec/RFC,
hata kataloğu, RC/soak/performans tarihçesi ve kurulum sözleşmesi değişmedi.

## K-305 — Ayrı köprü negatiflerinde iptal ve makbuz doğruluğu

**Dar kaynak/native ve son belge kabulü tamamlandı; K-305 / GB-114 kapalıdır.** Negatif test koşucusu erken flag-only
iptal kontrolü kurar; özgün çocuk/selector sahipliğini korur ve iptal ya da
kapanış hatasını başarılı makbuza dönüştürmez. Altı admission oracle,
üretim compiler, Supervisor ve gömülü kit davranışı değişmedi. Selector
hazırlığının iç try dışında olması tek başına kaçak kanıtı değildir;
gerçek kaçak süreç veya üretim HTTP/PG açığı gözlendiği ileri sürülmez.

Kontrollü hazırlıkta 15 Python metodu ve 27 senaryo geçti. Eski sürümün
17 yapısal eksiği, iki cleanup hatasında eksik makbuzu ve bir terminal süre
aşımındaki yanlış passed sonucu korunur. Eski dış owner exit1, yeni owner
exit0 ile reaped. Bu mock/handler kanıtı gerçek OS sinyal teslimi veya
native PG/HTTP sayısına eklenmez. Hazırlık kaydı
`E/k305-preparation-1/K305-PREPARATION.json`, SHA-256
`e445af8d0a82448488bd72d47e1360bba990751db0354f91061a93603c4918b1`.

- Kaynak kapısı **15/15** geçti; tam faz
  **998 başarılı / 0 başarısız /
  25 ignored**. Faz komutu 839,411 sn,
  kaynak kapısının toplamı 1045,134 sn. Clippy,
  katalog/tazelik, fmt, spec, koruklar ve sürekli güvenlik aynı kaynak
  değişikliğiyle doğrulandı; 194 kaynak pini değişmedi.
- Test edilmiş değişiklik temiz kaynak
  `40788a3d0a9f0d43083130bb0f062871861787db` (tree `9c1ea8dffa61d91f5946eff32a9c4cf1c124759e`) olarak kaydedildi.
  Üst kaynak K304 kapanış commit’i
  `95236cbe2d34f7497980ea3292b1f8451245a767` olarak korunur.
- Aynı temiz kaynakta mevcut ayrı CLI köprü Cargo sahibi
  **1 başarılı / 0 başarısız /
  0 ignored** sonucu verdi. Unit-before-native,
  dört native vaka, 19 HTTP isteği, altı SQL gözlemi ve altı admission
  negatifi doğrulandı. Unit ve dört worker coverage raporunun payda,
  vurulan ve eksik sonuçları ham edge kayıtlarından yeniden hesaplandı;
  unit ürün paydası 14. Native alt vakalar yeni Cargo sahibi değildir.
- Native sürücü 20,169 sn, özgün dış owner
  21,076 sn içinde exit0/reaped tamamlandı.
  Başlangıç/son temiz kaynak ve 205 pin aynı; 17 runtime pini ve 12 public
  dosyanın hashleri doğrulandı. FIFO yalnız lstat ile denetlendi;
  bağımsız kanıt incelemesinde private PG/proje içeriği okunmadı. Normal makbuz
  `signal_handlers_installed=true`, `cancelled=false` ve
  `owned_groups_closed=true` taşır. Handler restoration kanıtı ayrı
  hazırlıktadır; native makbuza eklenmiş sayılmaz.
- Native ikili SHA-256: `e3e0ab448f9ec9d4cab66d64b3212f3776a0f9d15378702a3bff0f003ea27885`.
  Native makbuzdaki BuildJSON/source/hash bağı korunur; sonraki belge
  komutlarının test ikilisini yeniden derlemesi shipping artifact kabulü
  veya yeni native koşusu sayılmaz.

Aşağıdaki yedi özgün makbuz exact SHA-256 ile bağlanır. Dondurulmuş kabul
kaydı `E/k305-acceptance-record-1.json`, SHA-256
`b3f8a459cae86d7747a14103632d09fb85febb2453d9917db5a1bb835a3c2dab`; kaynak/native bağımsız incelemesi
`/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k305-final-audits-1/INDEPENDENT-REVIEW.json`, SHA-256 `421facd52cc777ae0d34a480a3e971b2df6a6c63789eba64b05bcfb40d622d6e`.

| Makbuz (E altında) | SHA-256 |
| --- | --- |
| `k305-source-gates-1/PROOF.json` | `91021f9d62cf62c33abb26df9e39f5a8b8dcc8bfa15c8ddcd40b6ccd302b84ae` |
| `k305-source-gates-controller-1.json` | `f6010f305ae484c206ea4a7eadb54148a0ec654104692e707a2a7c126a6c9498` |
| `k305-source-commit-1.json` | `dfcdc24c6f7325a6f570cfc1841f6057ac691a7858a3b45f97c472be47043dfb` |
| `k305-source-commit-controller-1.json` | `281a21d30ded5341d5519b5472348a0e697178661a72003b8172cf1a84b786ca` |
| `k305-web-bridge-accept-1/PROOF.json` | `bcf9c95582ff0f60f193d0e5bb63ee54c5adc509803c3ef47ca19187c7914ac2` |
| `k305-web-bridge-controller-1.json` | `6fa55039301cd0c168331727a11df8361642758e3e17df15b40cced795de0891` |
| `k305-web-bridge-accept-1/public/bridge/results/proof.json` | `a9264792f9f8b81605d26bf4e161d6cd60b47b610caa98dfa068846a426dd196` |

Son **11 belge/güvenlik kapısı 11/11 geçti** (65,706493 sn; özgün dış owner 66,223813 sn, exit0/reaped). Test edilen 13 belge temiz kayıt commit’i `81d7b9e1fd7658113770e1961dc718971100c04b`, tree `c427adb1968385a98d97d34007421f7be735a845` ile kaydedildi. 213 değişmeyen kaynak pini, 11 ham günlük, 13 Git blob’u ve tam binary diff bağı bağımsız doğrulandı; commit dış owner da exit0/reaped tamamlandı.

| K-305 son kayıt makbuzu (E altında) | SHA-256 |
| --- | --- |
| `k305-final-gates-1/PROOF.json` | `97a9594cb33aa127cf68c4be0042742bdd8a5893cd2fd20a617f551cfbb9206f` |
| `k305-final-gates-controller-1.json` | `65c5ff28574d607c357eb180f051f4d0cf0942a633efaf826c32c0d136645cc9` |
| `k305-final-commit-1.json` | `f87fe4b4e75f31522ab158fd60847b12e21487415b27a64439d50f9ab1b9b607` |
| `k305-final-commit-controller-1.json` | `836220bff8aa5d11a3fff49a33d36cd880a287fae5084d0293bae88de0156e7c` |

Bağımsız son inceleme `E/k305-closure-audits-1/K305-FINAL-COMMIT-INDEPENDENT-REVIEW.json`, SHA-256 `f7a4890c29764c5943b8d4d5fb079010c1fb41e41ad5eec2b0dc853328f4b3a4`; arşiv manifesti `016c09c78a5384b18e7e8f206fb83887a0ef9161c5d173d1b166869e7ec6c138`. Bu tamamlanmış kapı yeni K-306 kaynak/belge değişikliğinin kapısı yerine geçmez.

Bu kaynakta 25 ignored sahibinden yalnız ayrı köprü sahibi ayrıca
çalıştırıldı; kalan 24 bu kampanyada çalışmadı. K304 direct kit, S3, TLS,
göç/havuz, RC/soak/IDE ve performans tarihçesi bu kaynağa aktarılmaz.
P22/P36 ve gerçek ürün/platform kapsamı kısmi kalır; bütün 39 madde kapanmaz.

ADR-144, köprü rehberi, indeks/kanıt haritası, GB-114,
backlog/master/sürümler/README/v1/39-plan ve faz kaynak/açıklaması birlikte
incelendi. Mevcut iki Cargo sahibi ve CLI birincil fazı korunur; Python
hazırlık senaryoları Rust toplamına eklenmez. Production Rust, Supervisor,
gömülü kit, grammar/API/tanı/spec/RFC, katman/bağımlılık, semantic regresyon,
performans iş yükü ve kurulum sözleşmesi değişmez.

## K-306 — Linux kabul koşucusu: açık

İki kabul koşucusunun canonical kök/erken handler, yapışkan iptal ve failed-body/tekrar-close doğruluğu; kısmi setup ve foreground PG sahipliği düzeltildi. Üretim compiler/dil/API değişmedi; iki mevcut ignored Cargo sahibi ve dispatcher4/HTTP7 oracle korundu.

**Kaynak:** `7d5e4b04391c82d2dec0a73a81d39de09357464e`; tree `803f6698e3926de68093634d40b531f1de4219a4`. Kaynak doğrulaması, ikinci denemenin geçen 12 komutu ile yalnız EOF boşluk düzeltmesinden sonra yeniden geçen beş komuta dayanır; 15 kapının bütünü yeniden çalışmış gösterilmez. Temiz `7d5e4b0` kaynak commit’i ve exact ağaç kayıtlıdır.

**Hazırlık ve gerçek inert ayrımı:** Kontrollü hazırlık Campaign16 ve PG18 yöntem/31 senaryoda geçti. Dış sahiplik hazırlığında core17, CLI9 ve runner5 modeli geçti; bu sayılar Cargo/native vaka toplamına eklenmez. Aynı16 eski Campaign modeli13 assertion failure üretirken aday0 failure verdi. Gerçek inert VM kabulünde normal komut ve EOF olumsuz yolu doğrulandı. EOF iç adımı `failed/control-eof` kaldı; özgün çocuk kapanıp reap edildi. VM normal SIGINT/exit0 ile kapandı, fallback kullanılmadı ve özel host runtime silindi. Bu iki inert prob, Linux native4+7 kabulünün yerine geçmez.

**Gerçek Linux kabulü:** C12 V3 dispatcher Cargo/libtest sahibi geçti; manager-end exit1/degraded nedeniyle kampanya başarısız kaldı, iç proof/günlük dışarı aktarılmadı ve HTTP çalışmadı. Yeni kaynakta gerçek dispatcher4/HTTP7 kabulü henüz yok. C12 VM1–3 başarısız geçmişi korunur; VM4 sonucu aşağıda kendi public kanıtıyla kayıtlıdır. Yeni kaynak ortam kabulü henüz yok. Önceki başarısız VM kampanyalarının özgün owner kapanışları kendi kayıtlarında korunur; yeni kaynakta native PG/çocuk/VM kabulü bekleniyor. Koşullu kaynak düzeltmesi bağımsız incelemesi native kabulü değildir; yeni kaynak ve native zincirinin bağımsız incelemesi ayrıca gerekir.

**Gerçek kampanya başarısızlıkları:** V1 ve V2 ayrı fresh VM denemelerinde 45 bootstrap ve yedi başlangıç kontrolü geçti; 53. komut olan native validator prepare exit1/reaped ile durdu. Her iki VM normal exit0 ile kapandı, bütün özgün sahipler reap edildi ve dış owner exit1/reaped kaldı. Derleme ve native dispatcher4/HTTP7 başlamadı. V1 başarısız prepare ham günlüğü dışarı aktarılmadığı için kök neden bilinmiyor; V2 sonucu V1 için varsayılmaz. V2 dışarı aktarılmış günlükte gerçek `FileNotFoundError: /run/user/501/bus` kaydı vardır; günlük SHA-256 `6d28927ed5c8e7dfd529edc0a3e91ad220dc1d025f25f55f9e29be419a247bf1`. V1/V2 başarısız kanıtları korunur.

**V3 gerçek kampanya — başarısız kapanış:** `E/k306-linux-campaign-3` public kaydında 48 bootstrap ve 18 build komutu geçti. Dispatcher için mevcut tek Cargo/libtest sahibi PASSED verdi (derleme 13,36 sn; libtest 509,65 sn); ardından üç source-end kontrolü geçti. `manager-end` exit1 ve degraded günlük sonucu nedeniyle kampanya başarısız kaldı. HTTP çalışmadı. Dispatcher iç proof/günlük aktarımı 17 adımlık fazın sonunu beklediğinden yapılmadı; bu Cargo sonucu yapılandırılmış dispatcher4 kabulü sayılmaz. Dış süre 872,0194029 sn; bütün özgün sahipler reap edildi, VM normal exit0 ile kapandı, fallback kullanılmadı, dış owner exit1/reaped kaldı ve runtime korundu. Başarısız unit için reset eksikliği yalnız doğrulanmamış hipotezdir; neden kesinleştirilmedi ve reset-failed/oracle atlama yapılmadı.

**Uygulanan kaynak takip dilimi — K-306/GB-115 açık:** VM4 gerçek sonucu: C12 üzerinde E/k306-linux-campaign-4 kaydında 48 bootstrap ve 18 build komutu geçti; dispatcher Cargo sahibi exit0/reaped ve dışarı aktarılmış dört senaryo geçti. Son failed-unit listesi tek sahipli zee-dispatch.service birimini gösterdi; manager-end exit1/degraded ile kampanya durdu, HTTP başlamadı. Host/guest özgün sahiplerin tamamı reap edildi, PostgreSQL ve VM normal exit0 ile kapandı, fallback kullanılmadı; dış owner exit1/reaped kaldı. Bu başarısız kampanya yeni kaynak kabulü veya K-306/GB-115 kapanışı değildir. Doğrulanmış sahipli unit kimliği: manager-failed-end.log içeriği exact `zee-dispatch.service not-found failed failed zee-dispatch.service`; dispatcher unknown/restart senaryosunun aynı kampanya tarafından kurulan birimidir. Kamu proof/log hash ve byte kayıtları T/K306-VM4-OWNED-UNIT-DIAGNOSIS-1.json bağımsız incelemesinde bağlıdır; foreign unit gözlenmedi. Public VM4 `manager-failed-end` kaydıyla kimliği doğrulanan kampanya unit’i için `Campaign.unlink` düzeltildi: mevcut exact stop+cgroup-empty ve symlink inode/hedef/kaynak hash kanıtından sonra yalnız `self.units` üyesi exact `p.name` için reset-failed çağrılır; ActiveState inactive olmalı ve bağlantı kimliği yeniden doğrulanmalıdır. Ardından mevcut unlink/daemon-reload/LoadState not-found zinciri sürer. Global veya foreign reset yasaktır; foreign failed unit genel manager kapısını yine kırar. Yapışkan failure/cancel sonucu, cgroup/path sahipliği, komut/cleanup bütçeleri ve dispatcher4/HTTP7 oracle değişmedi. Bu anlatım yalnız VM4 sahiplik kanıtı, root tarafından doğrulanmış kalıcı21+PG18 sonuçları ve aynı toplu dilimde kaynak kopyasının uygulanması altında geçerlidir.

**Kontrollü model sınırı:** Ayrı altı inert metot root tarafından exit0/reaped ile geçti: bir eski davranış karşı örneği ve beş olumlu metot; eski altı metodun başarısızlığı iddia edilmez. Mevcut 16 Campaign metodu korunup aynı preflight dosyasında ve aynı Cargo sahibi altında beş metot eklendi; toplam21. Kalıcı Campaign21 gerçek sonucu: E/k306-unit-reset-campaign21-controller-1.json ve hash bağlı günlük: 21 metot geçti; özgün owner observed_exit=exit=0, closed/reaped, hata ve iptal yok. Önceki16 metot korunur; beş yeni metot exact sahiplik, inactive durumu, foreign durumun korunması ve yapışkan iptal sınırlarını doğrular. Birleşik PG18 yöntem/31 senaryo gerçek sonucu: E/k306-unit-reset-pg18-controller-2.json ve hash bağlı günlük: 18 metot/31 senaryo geçti; özgün owner observed_exit=exit=0, closed/reaped, hata ve iptal yok. İlk denemenin iki eksik public tüketici fixture hatası korunur; yalnız exact C12 dispatcher/HTTP fixture kopyaları eklendi, oracle değişmedi. Kalıcı21 root kaydı `E/k306-unit-reset-campaign21-controller-1.json` içindedir; PG18 ikinci controller kaydı geçti. İlk PG18 denemesi yalnız iki public tüketici fixture’ı eksik olduğu için başarısız kaldı; exact C12 kopyaları T içine eklenerek tekrarlandı, oracle değişmedi. Bu kontrollü modeller ve koşullu peer incelemesi native düzeltme/kapanış kabulü değildir.

**Kaynak commit’i ve yeni native önkoşulları:** Minimal kaynak, kalıcı test ve açık durumlu14 belge aynı toplu kaynak takip dilimindedir. Tam15 kaynak kapısı `E/k306-source-gates-3/PROOF.json` ve controller3 bu aynı kaynak+test+belge commit’inin önkoşuludur; exact kaynak commit SHA’sı `E/k306-source-commit-2.json` içinde kaydedilir. Belge kendi gelecekteki hash/SHA’sını uydurmaz; bu yollar kapı/commit başarısı iddiası değildir. İki değişen test-harness dosyasına göre source214/source341 pin değerleri, transfer/arşiv ve native zinciri yeniden üretilir; dosya sayıları214/341 ve production175 kapsamı değişmez. Exact temiz kaynak commit’i sonrasında fresh dispatcher4/HTTP7 ve özgün owner/PG/VM kapanışı ayrıca gerekir. C12 kaynak kapılarının 12 tarihsel + 5 yeniden geçen komut birleşimi, VM1–3 başarısızlıkları ve C12 Cargo sonucu yeni kaynak/native kapılarının yerine geçmez. V3’te reset eksikliği hipotezi o başarısız kampanyanın tarihinde doğrulanmamış olarak kalır; VM4 public sonucu yukarıdaki ayrı alandır. Diğer beş PG tüketicisi açık kalır; P09/P13/P36 kısmi, P37 sürekli korunur.

**Son belge ve kapanış:** Bu kaynak+test+belge diliminin commit önkoşulu tam15 kaynak kapısı E/k306-source-gates-3/PROOF.json ve controller3 makbuzlarıdır; exact kaynak commit SHA’sı E/k306-source-commit-2.json ile izlenir. Bu metin kapı veya commit başarısı yerine geçmez. Fresh yeni-kaynak dispatcher4/HTTP7 ve son belge/güvenlik kapanışı ayrıca gerekir. K-306/GB-115 durumunun kapanışa çevrilmesi actual native11 iç vaka, özgün owner/PG/VM kapanışı ve son belge kapısının makbuzlarına bağlıdır. Kaynak takip ve final makbuzları yukarıdaki dış kayıtlarla izlenir; geçmiş kanıtlar yeni kaynak kabulüne taşınmaz.

**Korunan başarısız denemeler:** Kaynak kapısı1 katalog/tazelikte reddedildi; kapı2 ilk12 komutu geçtikten sonra support dosyasının sonundaki ek boş satır nedeniyle diff-check exit2 verdi. Yalnız EOF whitespace düzeltmesinden sonra hazırlık, katalog/tazelik, diff-check, secret-scan ve sürekli güvenlik yeniden geçti. İlk15 tam kapı baştan yeniden koşuldu denmez. İmajın ilk HTTP302 reddi korunur. İnert EOF probunun `failed/control-eof` iç sonucu bilerek korunur; dış kabul, bu olumsuz yolun özgün çocuk ve VM kapanışını doğruladı.

Tarihsel S3 ile K-298–K-305 sonuçları kendi kaynaklarında kalır; farklı SHA sonuçları toplanmaz. P09/P13/P36 kısmi, P37 sürekli; ürün/sağlayıcı benimsenmesi, genel exactly-once ve fiziksel platform U2 açık kalır. C1/P33 reddedilmiş kapsamı yeniden denenmez. Kullanıcıdan bu yerel kampanya için yeni dosya/cihaz istenmiyor.

### Ön hazırlık makbuzları — tarihsel kayıt

Aşağıdaki indirme/model makbuzları kendi aşamalarına aittir; tek başına guest kurulum veya native başarı iddiası taşımaz.

| K-306 hazırlık makbuzu (E altında) | SHA-256 |
| --- | --- |
| `k306-campaign-models-1/PROOF.json` | `045a65ae1ce84b5e11d16a5da6620d786d8f3ad2700403b00ffcbb89c03260e6` |
| `k306-campaign-models-controller-1.json` | `cda19bb2362a02ad843d8b4ff04385d40d8110939b07623dc91ad5d783901ccc` |
| `k306-combined-models-1/PROOF.json` | `f249dafb4fb00021a5f9e2c1864ce68c5486c89f5a0ef61a03a08471587e3d91` |
| `k306-combined-models-controller-1.json` | `d3613083eaf4bcc2c287cc8033aab6d7b82842e0e9872d089e13343356891540` |
| `k306-public-transfer-1/PROOF.json` | `6454ab6ab1b56d547342328fb537ad09f7bbd7344704b1573e4d4719fa2ef5cc` |
| `k306-public-transfer-controller-1.json` | `e1f1f5de57eb05654b6ae6b205dd1aeb4f08ad049b9e0061ebad768086635a35` |
| `k306-linux-rust-inputs-1/PROOF.json` | `6701dc6bb99ab551cb3f0281c0ab291206472a1f29ee99d17c02115f1e58105c` |
| `k306-linux-rust-components-1/PROOF.json` | `1fd6170e75af757aa7f4722a034aeccd735da5046d5643fac9305271bbb65b59` |
| `k306-rust-components-controller-1.json` | `1fce6b11f157879429c0ffe2d19b3c3a6debbf95f48cd7bca2e117c3ea3118f0` |
| `k306-linux-image-2/PROOF.json` | `29d7ae3ead5941f64fd20f085c44171aad5acf9d1f37c19cd4afd1fb34a86c8c` |
| `k306-linux-image-controller-2.json` | `92444499ca064f0a9fe09d962cdcb57d1595bbea23f7fb30ddf1d84f866f0fdb` |


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
| K-303 PG havuzu | Temiz `47812bf` üzerinde tek mevcut Cargo vakası ve 16 metrik kaydı geçti; normal PG kapanışı ve kaynak eşliği doğrulandı. |
| K-304 owned web | Hazırlık22, kaynak15 kapı ve temiz `abdc5cf` direct native4/19HTTP/6SQL geçti; GB-113 dar kapsamda kapalı. |
| K-305 ayrı köprü | Kaynak15 ve temiz kaynakta bridge/native4/19HTTP/6SQL/negatif6/coverage5; ardından son11 ve temiz C11 kayıt zinciri doğrulandı. GB-114 dar kapalı. |
| K-306 Linux koşucusu | Temiz7d5e4b0 kaynak12tarihsel+5yeni komutla doğrulandı; model17/9/5 ve gerçek inert2 geçti. C12 V3 dispatcher Cargo/libtest sonucu yapılandırılmış dispatcher4 kabulü değildir; HTTP çalışmadı. Yeni exact kaynakta dispatcher4/HTTP7 ve kapanış kapıları bekleniyor. GB-115 acik; son belge kapısı kayıt koşulu. |
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

25 explicit ignored Cargo sahibinin salt okunur envanteri
`E/k304-related-audits-1/P36-IGNORED25-INVENTORY.json` ve `.tsv` içinde kayıtlıdır.
IO15, web/güvenlik5, proje3, CLI1, LSP1 faz dağılımı; tam isim, komut/ortam,
geçmiş exact SHA ve somut engeller vardır. Bu envanter K-304 native koşusundan
önce hazırlanmıştır; K-304 direct kabulü ve K-305 ayrı köprü/son11/temiz C11
kapanışı bu rapordaki exact kaynaklarıyla okunur. S3 tarihsel eşleme yeni Cargo replay’i değildir. Beş PG
tüketicisinin sahiplik/iptal hazırlığı yerel açık iştir. İki Linux sahibinin
C12 kaynak hazırlığı kayıtlıdır; sahipliği VM4 ile doğrulanmış unit için
minimal kaynak ve kalıcı test düzeltmesi bu belge dilimiyle uygulandı.
Aynı kaynak+test+belge commit’i tam15 kaynak kapısı ve exact commit kaydına
bağlıdır; fresh yeni-kaynak dispatcher4/HTTP7 ve kapanış ayrıca açıktır.
C12 Cargo sonucu native kabul ve K-306/GB-115 kapanışı değildir.

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
| P09 — Kuyruk/dispatcher | Kısmi; S3 tarihsel. Temiz7d5e4b0 kaynak doğrulandı; C12 V3 Cargo/libtest geçti; iç proof/günlük dışarı aktarılmadığından yapılandırılmış dispatcher4 kabulü yok. | Gerçek ürün/sağlayıcı benimsenmesi ve genel exactly-once açık. Bu kaynak+test+belge diliminin commit önkoşulu tam15 kaynak kapısı E/k306-source-gates-3/PROOF.json ve controller3 makbuzlarıdır; exact kaynak commit SHA’sı E/k306-source-commit-2.json ile izlenir. Bu metin kapı veya commit başarısı yerine geçmez. Fresh yeni-kaynak dispatcher4/HTTP7 ve son belge/güvenlik kapanışı ayrıca gerekir. | Yerel dilim için yok; ürün için U1 + U5. |
| P10 — SMTP/PG hata sınırları | Kısmi; S3 fault11 tarihsel. | Canlı sağlayıcı, gerçek katalog ve ürün entegrasyonu açık. | U1 + U5. |
| P11 — PDF/FFI | Kısmi; temiz C2 PDF9 ve bağımsız incelemesi geçti, K-299 dar kapsamı kapandı. İlk C başarısızlığı korunur. | Gerçek katalog, genel FFI/sandbox ve yayın ayrı. | Ürün için U1 + U5. |
| P12 — Göç/ürün betikleri | Kısmi; dar yerel göç/rollback kanıtı var. | Gerçek üründeki Python betiğinin kaldırılması ve göç kabulü açık. | U1 + U5. |
| P13 — Sürekli HTTP servisi | Kısmi; K-303 havuz1 kendi47812bf kaynağında. Temiz7d5e4b0 için C12 V3 HTTP çalışmadı; yeni exact kaynakta HTTP7 bekleniyor. | Üretim/pilot benimsenmesi ve fiziksel platform açık. Bu kaynak+test+belge diliminin commit önkoşulu tam15 kaynak kapısı E/k306-source-gates-3/PROOF.json ve controller3 makbuzlarıdır; exact kaynak commit SHA’sı E/k306-source-commit-2.json ile izlenir. Bu metin kapı veya commit başarısı yerine geçmez. Fresh yeni-kaynak dispatcher4/HTTP7 ve son belge/güvenlik kapanışı ayrıca gerekir. | Ürün U1 + U5, platform U2. |
| P14 — Karma sürüm geçişi | Kısmi; temiz K-300 `549dbe6` native9/dört dalga/160 istek geçti. | Genel SLA, DB şema devri ve gerçek ürün kabulü açık. | Ürün için U1 + U5; platform için U2. |
| P15 — Parola/login | Kısmi; S3 parola/TLS kanıtı tarihsel. | Gerçek login, göç, sağlayıcı ve ürün kabulü açık. | U1 + U5. |
| P16 — Form/Turnstile | Kısmi; hermetik form testleri mevcut. | Gerçek staging ve tam ürün kabulü açık. | U1 + U5. |
| P17 — Depolama/IAM | **Ertelendi.** | Gerçek çok sunuculu ihtiyaç kanıtı olmadan yeni kapasite işi açılmıyor. | İhtiyaç doğarsa U1 + U5. |
| P18 — C1 tanı/quickfix | **Kısmi; araç incelemesi sınırı açık.** | Beş özel tanı/quickfix zinciri kabul edilmedi; lint bu kabulün yerine geçmez. | Ek izin/dosya isteğiyle kapanmaz; aşağıdaki araç sınırı. |
| P19 — LSP/editör | **Kısmi; K-298 dar stdio/RSS/RC/IDE kabulü kapandı.** S UTF-16 işlev kanıtı da var. | Artımlı HIR/tür, UI/gerçek ürün gecikmesi, SLA ve Windows/yayın açık. | Teknik kapsamın tümü girdi engeli değil; ürün U1, platform U2. |
| P20 — SQL modülleri | Kısmi; S3 native modül kabulü tarihsel. | Gerçek ürün kullanımı açık; DB+FS atomikliği ve genel extension point vaadi yok. | U1 + U5. |
| P21 — TR–EN/bileşen | Kısmi; S3 HTTP/Zee kabulü tarihsel. | Gerçek sayfa/ürün benimsenmesi açık; yerleşik çeviri kapasitesi eklenmedi. | U1 + U5. |
| P22 — Kaynak kapsamı/test köprüsü | Kısmi; K-304 direct native4 ve K-305 ayrı köprü/native4/negatif6 kendi kaynaklarında. K-305 son11/temiz C11 ile dar kapalı. | Gerçek ürün kabulü ve genel kapsam açık; farklı kaynak/coverage paydaları toplanmaz. | Yerel devam için yok; ürün için U1 + U5. |
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
| P36 — Test/ignored kapsamı | K-302 seçili9, K-303 havuz1, K-304 directweb1, K-305 bridge1 ayrı kaynaklardadır. K-306: K306/GB115acik;C12CargoPASSyapilandirilmisdispatcher4kabuludegil;yenikaynak4+7bekleniyor. | Farklı SHA sonuçları toplanmaz; diğer sahiplerin mevcut kaynak ve ortam bağımlılıkları ayrı açık kalır. | Yerel dilim için yok; gerçek cihaz/ürün/servis ayrıca girdi ister. |
| P37 — Sürekli kalite kapıları | Sürekli iş. K-306 kaynak12tarihsel+5yeni doğrulaması kayıtlı. Bu kaynak+test+belge diliminin commit önkoşulu tam15 kaynak kapısı E/k306-source-gates-3/PROOF.json ve controller3 makbuzlarıdır; exact kaynak commit SHA’sı E/k306-source-commit-2.json ile izlenir. Bu metin kapı veya commit başarısı yerine geçmez. Fresh yeni-kaynak dispatcher4/HTTP7 ve son belge/güvenlik kapanışı ayrıca gerekir. | Her yeni atomik dilim kendi kaynak/test/belge kapılarını tekrar sağlar. | Yerel işler için yok; bağlı dış kapılar ayrı. |
| P38 — Backend/WASM/self-hosting | **Ertelendi.** | Native WASM/backend/self-hosting gerçekleştirilmiş sayılmıyor; V1 ve ihtiyaç koşulları korunuyor. | Şimdi yok. |
| P39 — Lisans/yayın kararı | Kısmi karar: sınırlı pilot seçildi, lisans bekletildi. | Mevcut marka/site korunur; public/yayın kararı henüz verilmedi. | Zamanı gelince **U6.** |

K-298–K-305 dar kabulleri kendi kaynak/ölçüm/native/arşiv ve son belge
makbuzlarıyla tamamlandı. K-306/GB-115 açık; önceki kaynakların
RC/RSS sonuçları bu işe aktarılmaz. Bu kaynak+test+belge diliminin commit önkoşulu tam15 kaynak kapısı E/k306-source-gates-3/PROOF.json ve controller3 makbuzlarıdır; exact kaynak commit SHA’sı E/k306-source-commit-2.json ile izlenir. Bu metin kapı veya commit başarısı yerine geçmez. Fresh yeni-kaynak dispatcher4/HTTP7 ve son belge/güvenlik kapanışı ayrıca gerekir. P19'un artımlı HIR/tür ve UI gecikmesi ile tabloda kalan ürün
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
