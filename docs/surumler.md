# Sürüm notları

> `a57290f7a1c1fad07cda3f266b2a221f09aa6404` için [K-298 yerel kabulü](urunlesme-k298-kabul-20261016.md) dört stdio kohortunda gözlemsel p95 200 ms hedefini, macOS teslim/kurulum, ordinary UTF-16 ve dar IDE işlev kabulünü kaydeder. Aynı SHA'da ikinci RSS ve dört ASan/Miri3/tam RC kapısı geçti; K-298 dar kabulü kapalı, P19 kısmi kalır. Belge takip commit’i gerçek dış son kapı makbuzuna bağlıdır. [S4](urunlesme-s4-kabul-20261015.md) ve [S3](urunlesme-s3-kabul-20261015.md) kendi kaynaklarında tarihseldir; ürün, insan, fiziksel platform ve yayın kapıları açıktır.

Her sürüm: ne eklendi, ne değişti, neyin sözü verildi. Kırıcı değişiklik
sessizce yapılamaz (master plan bölüm 23) — burada duyurulur.

## v0.7.0 — 1 Eylül 2026

**Tema: editör zekâsı — dil, düzenleyicide de Türkçe düşünüyor.**

(Üstteki K-069..K-072 kalemleri bu sürümündür: çıkış kodu, kitaplık
olgunlaşması, zengin doğrulamalar, morfolojili yeniden adlandırma.)

## Yolda (v0.8.0'a birikenler)

- **K-304/GB-113 dar kapsamda kapalı:** owned web kabulünün geç iptal/deadline
  sonucu, hazırlık22/kaynak kapısı15 ve temiz direct native4 ile doğrulandı.
  Kullanıcı göçü yoktur; [kit ve ayrı köprü kabulü sınırı](owned-web-test-kiti.md).

- **K-306 hazırlığı / GB-115 açık:** kontrollü Campaign16 adayda geçti; foreground PG18 yöntem/31 kontrollü senaryo da geçti; tam kaynak kapıları ve temiz Linux4+7 bekleniyor. Üretim davranışı veya kullanıcı göçü değişmiyor; [kabul sınırı](dispatcher-service.md).

- **K-305/GB-114 dar kapsamda kapalı:** negatif bridge kabul koşucusunun erken
  iptal/owned cleanup ve final makbuz doğruluğu, hazırlık15/kaynak kapısı15 ve
  yeni temiz exact bridge Cargo1 ile unit kaynak kapsamı/native4/altı negatif
  üzerinden doğrulandı. Son11 belge kapısı kayıt commit önkoşuludur. Üretim
  grammar/API/kit/Supervisor davranışı ve kullanıcı kaynağı göçü değişmez;
  [kaynak ve kabul sınırı](dene-native-web-koprusu.md).

- **K-303 dar pool kabulü:** temiz `47812bf` üzerinde mevcut tek Cargo sahibi
  1/0/0, 16 metrik anlık görüntüsü, tek timeout ve normal PG kapanışı geçti.
  Üretim davranışı/test sayısı değişmez; K-302’nin `e6322db` üzerindeki
  TLS8/göç1 sonucu ayrı kalır. Native metrik tüketicisinin sekiz vakası
  burada yürütülmedi. Son 11 belge/güvenlik kapısı kayıt commit’inin
  önkoşuludur; [gerçek kapsam ve makbuzlar](urunlesme-k298-kabul-20261016.md).

- **K-302 yerel PG kabulü:** temiz `e6322db` için TLS8/göç1 geçti.
  Production/göç davranışı değişmez; [kaynak ve açık sınır](urunlesme-k298-kabul-20261016.md).

- **Worker inflight test doğruluğu (K-301, dar kapsam tamamlandı):** handler
  giriş işareti ve tüm sınırlı yanıt kontrolü, üçüncü gerçek worker koşusunda
  tek Cargo sahibi 1/0/0 ve wrapper'ın 25/0/0 doğrulamasıyla geçti. V3 normal
  `web-oturumu`/CSRF GET/çerez/form POST ve `eylem` kullanır; CSRF atlanmaz.
  Ayrı mock karşılaştırması 10 senaryo/20 yöntemdir; gerçek yardımcı worker3'te
  sınandı. İlk GET fixture'ının T045 reddi ve worker1 başarısızlığı ile ikinci
  koşunun yalnız inflight hatası (25 test/1 hata) korunur. İkincide assertion
  ayrıntısı yoktur; CSRF/çerez eksikliği kaynak incelemesidir, HTTP403 gözlemi
  değildir. Son 11 belge/güvenlik kapısının geçmesi kayıt commit'inin
  önkoşuludur. Üretim davranışı, 25 vaka/tek Cargo sahibi aynı; genel
  P34/drain/fault kapanışı veya kullanıcı göçü değildir.


- **Mixed-revision kabul koşucusu (K-300, ADR-149/GB-112 dar kapsamda kapalı):**
  Rust kökü kanonikleştirildi; erken iptal, kısmi host sahipliği ve cleanup
  iptalinin başarısız sonuçta korunması tamamlandı. Hazırlık13, C3 öncesi
  kaynak kapısı13/tam faz998/0/25 ve temiz C3
  `549dbe66a775db17875b47c67948245a6a8fbf60` üzerinde native9/dört dalga/160
  HTTP200 geçti. Son11 belge kapısının geçmesi takip commit'inin önkoşuludur.
  Üretim compiler/host ve kullanıcı kaynağı değişmez; göç gerekmez. P14 kısmi;
  eski RC/soak/IDE kabulü aktarılmaz. [Kapsam ve kabul sınırı](mixed-revision-wave.md).

- **Kabul koşucularının iptal temizliği (K-299, dar kaynak/native/arşiv kapsamı kapalı; toplu commit öncesi son belge kapısı gerekli):** PDF/service
  native test betiklerinde erken sinyal, yarım keygen/fixture ve nested owner
  kapanışı ele alındı. Stdlib PDF9/service10 ayrı; `8178920e03fed782a817d1864b8280fcb80812a3` üzerinde
  temiz C2 native2 iki Cargo sahibiyle geçti: signed PDF9 ve service-fault2; alt vakalar Cargo owner sayısına eklenmez; C2 öncesi gate4: 13 adım geçti; tam faz 998/1023 geçti, 0 başarısız, 25 atlanan GB-111: kapalı; toplu commit öncesi son belge kapısı gerekli.
  Üretim host/compiler ve kullanıcı kaynakları değişmedi; göç yoktur.
  [Dar kabul kaydı](kabul-kosuculari-yasam-dongusu-20261016.md).

- **LSP açık belge maliyeti (K-298, ADR-145):** aynı bildirimde açık dosyaların
  gerçek yol/tutarlı ödünç metin dizini paylaşılır; alias çatışması,
  disk/paket doğrulaması ve tanı davranışı korunur. Yeni ayar veya kaynak göçü
  yoktur. [Temiz kaynak gözlemi](urunlesme-k298-kabul-20261016.md) dört stdio kohortunda p95'i
  200 ms altında buldu; en ağır kohortta yeniden ölçülen eski/yeni değer
  204,880→189,886 ms (%7,32). Paylaşımlı gözlem hard eşik/SLA değildir;
  aynı kaynağın ikinci RSS ve RC kabulü geçti; K-298 dar kapsamı kapalıdır, genel P19 açıktır. Son belge kapısı takip commit’inin koşuludur.

- **LSP UTF-16 konumları (K-297, ADR-153):** emoji gibi BMP dışı karakterlerin
  ardından hover/definition/rename imleci ve tanı/düzenleme aralıkları doğru
  UTF-16 sütununu kullanır. Core scalar tanılar ve Zee kaynağı değişmez;
  kullanıcı göçü gerekmez, güncellenmiş dillsp yeniden başlatılır.
  [S4 yerel kabulü](urunlesme-s4-kabul-20261015.md) yeni exact kaynağın
  native stdio/IDE, RC/soak ve gözlemsel stdio gecikme kanıtını kaydeder.

- **LSP Türkçe dosya yolu (K-248, ADR-108):** yüzde kodlu UTF-8 klasör
  yolunun yanlış çözümlenmesi giderildi; yan birimler doğru klasörden
  yüklenir. Yeni URI platform sözleşmesi veya dil özelliği eklenmedi.

- **Resmî Zee sitesi (K-203, ADR-081):** `zeedili.com` HTML ve rotaları Zee
  ile üretir; on içerik sayfası, responsive/erişilebilir görünüm,
  canonical/OG/Twitter/JSON-LD, sitemap/robots, yerel statik ön kapı ve sunucu A
  systemd+Nginx dağıtımı eklendi. Production worker `text/html` üretir;
  kurucunun sağladığı standalone Zee sembolü, mavi/camgöbeği/lacivert palet
  ve favicon/PWA ikon ailesi dil kimliğine uyarlandı; ERP yazı kilidi ile
  sloganı ürün sınırını korumak için kullanılmadı.
  K-203/F003 ile site kod-öncelikli bir dil platformu olarak yeniden
  tasarlandı: ana sayfada aynı WASM derleyicisiyle çalışan deneme kutusu,
  yüklem-odaklı sözdizimi vurgusu, gerçek tanı ve `dil` komut yüzeyi,
  açık/koyu tema ve kenar menülü dil turu/öğrenme/belge sayfaları. Sitedeki
  her “Çalıştır” örneği gerçek derleyiciden geçer; 404 veren `blob/main`
  depo bağlantıları varsayılan `master` dalına düzeltildi.
  K-203/F005 ile görsel kimlik zee'nin resmî antrasit + altın logosuna,
  kağıt zemin ve İznik turkuazı/mercanına taşındı; yazı tipleri aynı
  origin'den sunuluyor. `/playground/` 15 derslik “30 dakikada Zee” akışına
  dönüştü; dersler Zee'de üretiliyor ve gerçek derleyicide doğrulanıyor.
  Site Selçuklu yıldızı, Kubadabad yıldız ve baklava deseni, İznik çini bordürü ve
  lale motifleriyle Türk görsel diline bağlandı; “Türkçe harfler” dersi
  `ı→I`, `i→İ` dönüşümünü gösteriyor. Depodaki 204 belge (kılavuz, spec, RFC,
  ADR) artık GitHub yerine sitede `/belge/` altında okunuyor; Markdown
  dönüştürücüsü de Zee ile yazıldı.
  K-203/F002 ile `/playground/` gerçek compiler WASM C ABI v3 ikilisini
  çalıştıran tam bir dil çalışma alanına dönüştü: altı örnek, düzenleyici,
  girdi/tohum, çıktı/tanı sekmeleri, kopyalama, indirme ve URL parçasıyla
  paylaşım vardır. Kaynak kodu cihazdan çıkmaz; binary SHA-256 ve ABI kapısı
  dağıtım öncesinde zorunludur.
  Nginx TLS/statik/HEAD normalizasyonu ve daha sıkı site CSP'sini uygulama
  mantığından ayırır. 7 Ekim 2026'da https://zeedili.com Cloudflare proxy'si
  ve DNS-01 Let's Encrypt sertifikasıyla public yayına girdi; uçlar
  `scripts/canli-dogrula.sh` ile doğrulandı. Dil/compiler davranışı değişmedi.

- **Kurulum metadata sahipliği (K-194, GB-032):** Kaynak `GIT_SHA` alanından
  manifest satırı enjekte edip kaldırma sırasında yabancı dosya sildirme
  yolu kapatıldı. Var olan `GIT_SHA` exact 40 küçük hex ister; hedef/veri
  yollarındaki TAB/CR/LF ve kanonik fiziksel yoldaki kontrol karakterleri
  kurulumdan önce reddedilir. Normal dosya sahipliği ve kaldırma korunur.

- **Çalışma ortamı kanıtı (K-192):** Kurulu release ikililerinin hash/SHA,
  doğrudan sistem kitaplıkları, Rust araçları PATH dışında exact program
  çıktısı ve stdio LSP initialize/shutdown/exit akışı ortak kapıda ölçülür.
  Üç platform tatbikatı başarılı/başarısız JSON raporunu exact commit+OS
  adıyla saklar; temiz makine/minimum OS/SDK provenance kapsamı açık kalır.

- **Kurulum hata geri alma (K-194):** Unix ve Windows kurulumları bütün
  ikilileri geçici alanda kopyalayıp doğrular; bozuk/eksik/yinelenen ikinci
  kayıt ilk ikiliyi manifestsiz bırakamaz. Yayınlama hatasında bu koşunun
  oluşturduğu dosyalar geri alınır, manifest en son yayımlanır. Üç platform
  gerçek CI kanıtı `38238657801` koşusunda geçti; ani güç kaybı uzlaştırması
  vaat edilmez.

- **Sürüm kaynak kimliği (K-192):** SBOM ve provenance, eski depo
  yönlendirmesi yerine kanonik `itwise-bilisim/zee` ve aynı exact commit'i
  kaydeder. Dil davranışı/artefakt şeması değişmez; uzak commit erişimi
  yayın öncesi ayrıca doğrulanır.

- **Derleme araç zinciri provenance tutarlılığı (K-191):** Sürüm ve offline
  vendor betikleri kayıtlı exact Rust pinini kullanır. Host/rustc/sysroot ve
  ikili üretimi aynı araç zincirindedir; farklı varsayılan Rust yanlış
  provenance üretemez. Sürüm pininin kaynağı exact HEAD'dir. Dil davranışı
  ve bağımlılık sürümleri değişmez; geçersiz/eksik pin hata verir.

- **Güvenli oturum listesi ve tekil iptal (K-188, ADR-079):** Zee ERP artık
  kullanıcıya ait etkin oturumları ham çerez sırrı olmadan SHA-256 özet kimlik
  ve zaman metadata'sıyla listeleyebilir; seçili kimliği ortak runtime deposunda
  worker/restart sınırları boyunca iptal edebilir. İki yeni `Sonuç` ifadesi
  eklendi; K-174 syntax freeze penceresi 30 Eylül–28 Ekim olarak yeniden başladı.

- **Migration işlev gövdesi ayrımı (K-187, ADR-078):** C026 transaction
  sınırı artık yalnız üst seviye SQL komutlarını tarar; PL/pgSQL ve
  dollar-quoted trigger gövdeleri yanlış `BEGIN` reddi üretmez.

- **Request-bound PostgreSQL tenant bağlamı (K-186, ADR-077):** Korumalı rota,
  doğrulanmış sunucu oturumu kullanıcısını bildirilen dar resolver işlevine
  bind eder; tenant UUID aynı leased bağlantıda transaction-local RLS ayarı
  olur. Public rota bağlamsızdır, hata rollback'tir ve uygulama eylemleri
  request transaction'ı içindeki savepoint'ler olarak kalır.

- **Doğrulanmış rota oturumu (K-185, ADR-076):** Gerçek Zee ERP tenant
  gereksinimiyle, sunucu oturumundaki kullanıcı ve rol yalnız rota içindeki
  `oturum` sözlüğünden okunur. İstemci girdisi bu sözlüğü dolduramaz; oturum
  sırrı ve CSRF açılmaz. Sahte iç alan ve rota dışı kapsam regresyonları ile
  Zee ERP dogfood reproducer'ı eklendi.

- **Public depo güvenliği (K-184):** IO iz kaydı yeni/eski hedefte yalnız
  sahibine erişimle atomik yazılır; eski geniş izinler taşınmaz. Public depo
  bildirim belgesi, hassas dosya ignore kuralları ve sabit Gitleaks CI taraması
  eklendi. Replay şeması değişmez; eski izler ayrıca değerlendirilmelidir.
- **Parola özeti, TOTP, SMTP ve görsel küçültme** (K-210, ADR-083,
  `dogfood/urun-e`): zee ile yazılan ilk müşteri sitesi Site E, legacy panel
  panelinin dört "desteklenmeyen" maddesini tekrar etti. `parolanın argon2
  özeti`, `yeni totp anahtarı` / `kod gizli anahtarıyla doğrulanıyorsa`,
  `alıcı konu gövde ile ileti göndermeyi dene` (geri alınamaz etki; proje
  bildirimi `e_posta_bağlantı_değişkeni`; smtp/smtps ağ hedefleri), aynı
  kalıbın `... ile bağlantı üzerinden ...` biçimi (SMTP bağlantısı değerden,
  panelden ayarlanabilir), `<` ile başlayan durum yanıtlarının `text/html`
  olarak gönderilmesi (panel hata sayfaları), `parolanın gizli şifrelenmişi` /
  `sifreli gizli çözmeyi dene` (XChaCha20-Poly1305, `gizli_anahtar_değişkeni`,
  `dil gizli-anahtar`), `Permissions-Policy` başlığı, rota birimleri
  (`<ad> rotalarını kullan`, K-211/ADR-084), görünüm bloğu (`<ad> görünüm
  olsun`, tipli HTML ağacı, K-212/ADR-085, S046/T055) ve `kaynak
  görselini hedef dosyasına N piksele küçültmeyi dene` (`image` crate) merkezi
  intrinsic kaydına girdi. Yeni tanılar C028/C029. K-174 freeze yüzey kaydı
  ürün sahibi kararıyla güncellendi.

- **İkinci gerçek ürün ve iki birim düzeltmesi** (K-164, ADR-072):
  `dogfood/kanit-ozeti` depo kayıt defterlerinden `docs/kanit-ozeti.md`
  üretir (toplu veri işleme iş yükü). Elmas birim içe alımı (`c→a`, `c→b→a`)
  artık A008 değildir; birimde doğan tanı kökenini taşır, CLI birim
  dosyasından alıntılar, LSP `kullan` satırına taşır, JSON kökenli tanıda
  `koken` alanı verir. Tanı kimlikleri ve geçerli program davranışı değişmedi.
- **Yürütme yığını 32 → 256 MiB rezervasyon** (K-178): debug profilde
  yorumlayıcı çerçevesi ~194 KiB/düzey ölçüldü; eski yığın debug CLI'yı 167.
  düzeyde C019 yerine doğal taşmayla düşürüyordu. Regresyon koşucusu ve
  derinlik testleri resmî yığında koşar; `regression/runtime/derinlik-499-resmi-yiginda.dil`.
- **Adil kalıcı yazma kilidi** (K-183, ADR-075): bekleyen yazar 5 ms
  yoklamak yerine çekirdek kuyruğunda bloklanır; kilidi ardışık yeniden alan
  yazar diğerini yavaş diskte 5 sn boyunca aç bırakamaz (ubuntu CI'da iki
  kez görüldü). Beş saniye sözü ve C013 değişmedi.
- **Üçüncü ürün: kelime avı** (K-179, ADR-074): etkileşimli CLI oyunu
  (`dogfood/kelime-avi`, 3 birim/7 test) K-164 sürtünmelerinin tekrarını ölçtü:
  2 yeni sınıf, 4 tekrar, 5 kılavuzla önlenen; compiler değişikliği yok,
  sıra erişimi (F010) iki üründe tekrar edince K-180 tasarım maddesi açıldı.
- **Zee–Rust–Go karşılaştırması** (K-165, ADR-073): aynı araç üç dilde bayt
  bayt aynı sayfayı üretir; ölçüler exact SHA tarihçesinde.

- **Kurulum-kaldırma ve sürüm runbook'u** (K-169, ADR-071): `scripts/kur.sh`
  ve `kur.ps1` artefaktı `SHA256SUMS` ile doğrulayıp kullanıcı klasörüne kurar,
  var olan dosyayı ezmez; `kaldir.sh`/`kaldir.ps1` yalnız manifestteki
  değişmemiş dosyaları siler. Üç platform tatbikat workflow'u etikette koşar;
  `docs/surum-runbook.md` sürüm günü sırasıdır. Dil davranışı değişmedi.

- **Tanı kalitesi kapısı ve üç tanı düzeltmesi** (K-166, ADR-070, ADR-024
  §8–9): golden'a 11 acemi hatası mutasyonu uygulayan `tani_kalitesi`
  öneri/işaret/gürültü ölçer. Düzeltmeler: işlem/yapı/`göre` başlığının S007'si
  girinti önerisi taşır; gövde dışına düşen `sayıyı al` satırı oturum kalıbı
  S043 yerine yol gösteren S004 verir; `dil denetle` ve LSP'de aynı tanımsız
  ad/yapı için A001/A007 bir kez yayımlanır. Geçerli programların davranışı
  değişmedi.

- **Uzun soak kapısı** (K-173, ADR-069): `soak` koşucusu golden korpusunu
  derleyip sabit programı yürütürken gerçek `dillsp` sürecine sürekli
  `didChange` gönderir; ısınma sonrası RSS büyümesi %10 ve 32 MiB'ı birlikte
  aşarsa kapı kalır. Tarihçe exact SHA ile yazılır; haftalık workflow 30 dk
  koşar. Dil davranışı değişmedi.

- **Tekrar üretilebilir sürüm artefaktı** (K-168, ADR-068): `dil` ve `dillsp`
  iki bağımsız temiz klonda sabit toolchain, `--remap-path-prefix` ve commit
  zamanı `SOURCE_DATE_EPOCH` ile bayt-bayt eş üretilir; eşit değilse artefakt
  yoktur. Çıktı SHA256SUMS, SPDX 3.0.1 SBOM (`Cargo.lock` bağımlılıkları),
  SLSA v1 provenance ve `zee-surum-imza-v1` Ed25519 imzası taşır;
  `surum-adayi` workflow'u `v*` etiketinde koşar. Dil davranışı değişmedi.

- **Dogfood korpusu** (K-172, ADR-067): `dogfood/` altındaki her `.dil`
  kaynağı ürün, K-işi/dilimi, kip ve exact çıktı/tanıyla manifestte; ilk 10
  vaka legacy panel'nın F007/F012/F030/F032 sürtünmelerinin reddedilen biçimini ve
  mevcut dildeki çözümünü ürün politikasıyla CI'da koşar. X-Zee-CSRF/çoklu
  Content-Type ve `__Host-` çerez nitelikleri gerçek adaptör testleriyle
  kanıtlandı (GB-020/GB-021 kapandı). Dil davranışı değişmedi.

- **Birleşik güvenlik sürüm kapısı** (K-170, ADR-066, SECURITY.md): K-105'ten
  K-176'ya 15 kapalı güvenlik bulgusu, 4 kabul edilen sınır ve 3 açık düşük/orta
  test boşluğu (ikisi K-172 ile kapandı) ardışık `GB-NNN` kaydında; açık kritik/yüksek bulgu CI'ı ve
  etiketi durdurur. `scripts/guvenlik-kapisi.sh --surekli` tedarik
  workflow'unda; `--surum-adayi` exact HEAD RC fuzz kanıtı, clippy ve tam test
  ister. İçerik güvenliği (multipart/antivirüs yok) GB-019 kabul kaydıdır.

- **Spec maddesi drift kapısı** (K-171, ADR-065): `spec/` altındaki 169
  normatif madde (ZORUNLU/YASAK/TANIMLI/AÇIK) parmak-izi kimliğiyle exact
  test işlevine bağlandı; 151 kanıtlı, 16 gerekçeli kısmi ve 2 açık madde
  `docs/spec-drift-raporu.md` içinde görünür. Metni değişen madde, kayıp test
  işlevi ve bayat rapor CI'ı durdurur. Dil davranışı değişmedi.

- **Uyumluluk ve deprecation politikası** (K-167, RFC-0028/ADR-064/spec-27):
  Kalıp kelimeleri, koşul yüklemleri, CLI komutları, `proje.kilit`/ZEP/yayın
  zarfı biçimleri, morfoloji/IO/eşzamanlılık profilleri, WASM ABI, Rust
  `api::v1` ve tanı şeması tek envanterde giriş sürümüyle donduruldu.
  Kaldırma yalnız ardışık `DEP-NNN` kaydı, bir alt sürüm serisi süre ve göç
  yoluyla olur; kaldırılan kelime mezar taşı kalır. Çalışma ağacı artık
  `0.8.0-dev` kimliği taşır (`dil sürüm`, LSP ve SBOM). **Kaldırılan:**
  internal `dil::tedarik` cephesi (DEP-004; göç `dil::artefakt_dogrulama`,
  desteklenen yol `dil::api::v1`). Zee kaynak programları etkilenmez.

- **Kritik işlev bölünmesi ve yerel kapı turu** (K-177, ADR-041): Uzak CI
  görmemiş 126 commit için bütün kapılar yerelde koşuldu; yalnız işlev eğilim
  kapısı kırıktı. Web host'un `istek_al` işlevi başlık okuma, gövde zarfı,
  akışlı gövde ve kimlik/oturum/oran yardımcılarına; PostgreSQL migration
  uygulaması dosya tarama ve DB uygulamaya bölündü. HTTP davranışı, tanılar
  ve dil semantiği değişmedi; eğilim tabanı K-177 incelemesiyle yenilendi.

- **Sabit HTTP durumlu yanıt ve K-163 kapanışı** (K-163/F032, ADR-063,
  RFC-0015/spec-11): Rota artık `<ifade> yanıtını <100..599 literal>
  durumuyla gönder` diyebilir; değişken/aralık dışı durum S037'dir ve eski
  yanıt 200 kalır. legacy panel gerçek TLS PostgreSQL kapanınca aynı worker'da
  readiness 503/liveness 200, backend dönüşünde readiness 200 verdi; eski
  bulunamayan/taslak içerik 200 workaround'u 404 ile emekli edildi. Ürün 1037
  Zee LOC/5 modül/12 teste ulaştı. Gerçek workspace LSP full-change p95
  41,416 ms; 10 dogfood commit'inde dokunulan dosya medyanı 5,5'tir. Exact
  ürün kanıtı `bb8e1ac`; K-163/V1-P0-36 kapalıdır.

- **PostgreSQL production TLS ve sınırlı havuz** (K-163/F031,
  ADR-062/spec-25): Proje manifesti secretsiz kanonik DNS/IP hedefi taşır;
  URL exact eşleşir, `hostaddr` ve örtük `prefer` reddedilir. `sslmode=require`
  yalnız bağlantı değişkenine bağlı PEM CA'yı güvenilir sayar, sistem köklerini
  kapatır, hostname ve en az TLS 1.2 doğrular. Worker başına 4 bağlantı, 2 sn
  checkout, 30 sn idle, 300 sn lifetime hedefi + en çok 30 sn bakım çevrimi ve
  checkout sağlık kontrolü vardır.
  Gerçek PG16.11 deneyi doğru CA/hostname, iki olumsuz sertifika vakası,
  exhaustion, stale replacement, idle/lifetime ve shutdown sınırlarını geçti.
  Managed-provider rotasyonu ile çok-worker toplam bütçe tatbikatı açık kalır.
  Exact ürün saha kaydı `977cd2a` commit'indedir.
  Ana+fuzz lock'ları aynı zincire güncellendi; 163 paketlik iki vendor üretimi
  `6adafc91…9456e` özetiyle eş ve boş Cargo home offline derlemesi yeşildir.

- **Akışlı binary upload ve dosya yaşam döngüsü** (K-163/F030,
  RFC-0027/ADR-061/spec-26): Native web adaptörü exact
  `application/octet-stream` gövdesini 16 MiB sınırında belleğe toplamadan
  `0600` temp dosyaya akıtır; SHA-256 ve boyutu aynı geçişte üretir. Yeni
  `atomik taşımayı dene`, `silmeyi dene`, `dosyaları listelemeyi dene` ve
  `sha256 özetini almayı dene` yüzeyleri beklenen arızayı C013 `Hata` değeriyle
  taşır, capability ve IO trace/replay sınırını korur. legacy panel'nın 970 LOC/4
  modül/10 test ürünü gerçek PostgreSQL 16.11 üzerinde normal upload→ready→
  delete akışını ve temp/metadata/rename/tombstone kesmelerinden restart
  yakınsamasını kanıtladı. Multipart, antivirüs ve nesne deposu hâlâ açık
  kapsamdadır; production TLS/pool F031 ile ayrıca kapandı.

- **Dosya yazma arızasında web worker survival** (K-163/F029): legacy panel medya +
  metadata protokolündeki gerçek dizin çakışması C013'ün yalnız isteği değil
  bütün worker'ı sonlandırdığını gösterdi. Web adaptörü artık C013'ü ayrıntı
  sızdırmayan 503'e dönüştürür, başarısız yazımı otomatik tekrarlamaz ve sonraki
  isteği kabul eder; CLI C013'ü korur. İlk ürün reproducer'ı `legacy-panel`
  `b3cee9a` commit'indedir. Binary upload, hash, fiziksel silme/orphan tarama ve
  production TLS/pool bu dilimde açık kanıttı ve F031 ile kapandı. Final gerçek koşuda C013 503 verdi,
  aynı worker sağlık isteğini kabul etti, metadata `hatalı`ya uzlaştırıldı;
  finalize reddi ise process restartından sonra `hazır` oldu. Exact sonuç ve
  temizlik ürünün `29d30c8` commit'indedir.

- **WASM/native bağımlılık sınırı:** Playground WASM hedefi native
  yayın/artefakt imzalama modüllerini artık derleme grafiğine almaz. Paket
  grafiğinin davranışsız modeli WASM'da serde gerektirmeden korunur; gerçek
  `wasm32-unknown-unknown` kapısı native-only bağımlılıklardan ayrıdır.

- **Yerel PostgreSQL dogfood yüzeyi** (K-163, RFC-0026/ADR-060/spec-25):
  Proje sırrı kaynakta tutmadan exact loopback hedef ve ortam değişkeni adı
  bildirir. Okuma/değiştirme extended-query TEXT parametrelerini SQL'den ayrı
  bind eder; C025 Hata değeri SQLSTATE/constraint/tabloyu korur. Değişiklikler
  eylem transaction/savepoint'ine, dil göçür ise advisory lock ve değişmez
  SHA-256 migration geçmişine bağlıdır. Gerçek PostgreSQL 16.11 ürün provası
  apply→skip→hash reddi, injection-benzeri değer, 23505 ve rollback'i geçti.
  Çalışan ürün kanıtı `legacy-panel` deposundaki `a393128` commit'indedir.
  Bu ilk kanıt yalnız localhost sslmode=disable profilidir; production
  TLS/pool sözü F031/ADR-062 ile sonradan eklenmiştir.

- **İlk PostgreSQL dayanıklılık kırılma noktası** (K-163): `43d04fc` ürün
  commit'inde PostgreSQL backend'i gerçek koşuda sonlandırıldı. Hiç kurulamayan
  bağlantı kapasite geri gelince süreç içinde iyileşirken öldürülmüş canlı
  client yeniden bağlanmıyordu. Ardındaki correctness değişikliği transaction
  dışı salt okumada kapalı client'ı atıp tek reconnect + tek SELECT tekrarı
  yapar; aynı PostgreSQL 16.11 provası process restart olmadan başarıya döndü.
  Exact PID/süre/temizlik ve negatif write kanıtı `78cce11` ürün commit'indedir.
  Transaction içi okuma, SQL reddi, write ve COMMIT tekrar edilmez; COMMIT
  bağlantı kaybı “sonuç belirsiz” olarak raporlanır. Write olumsuzu satır
  üretmedi fakat C021 ile süreci sonlandırdı. F027 write-path availability'yi,
  F028 iki uçlu COMMIT-kaybı enjeksiyonunu kapattı; gerçek pool F031'de kapandı.
  492 LOC maintenance tabanı 1000+ satır karşılaştırması için kaydedildi.

- **Write bağlantı kaybında worker survival** (K-163/F027): Web isteğindeki
  C021 transaction-boundary hatası artık istek taslağını geri alıp 503 üretir;
  PostgreSQL client'ı yeniden kullanılmaz ve worker sonraki isteği kabul eder.
  Gerçek backend-kill provasında başarısız write otomatik tekrar edilmedi, satır
  sayısı 0 kaldı; sonraki GET yeni backend PID'sine bağlandı ve sonraki bağımsız
  write 303 ile tek satır üretti. Exact saha kaydı `00a659a` ürün commit'indedir.
  Wire-level COMMIT ambiguity enjeksiyonu F028 ile kapandı.

- **COMMIT sonucu belirsiz sınıfı ve gerçek iki-uç provası** (K-163/F028):
  PostgreSQL `db.commit_unknown` verisi, yorumlayıcıdaki tipli transaction
  hatası ve kullanıcıya açık C027 aynı anlamı uçtan uca taşır. Web adaptörü
  “otomatik tekrar yok; uzlaştırma gerekli” 503'ü verir, worker yaşar; CLI
  C027'yi korur. Wire proxy COMMIT'i DB'ye iletmeden kestiğinde 0, COMMIT
  uygulanıp ReadyForQuery yanıtı yutulduğunda 1 satır gözledi; iki durumda da
  aynı 503 çıktı ve retry olmadı. Sonraki bağlantıda iş anahtarıyla uzlaştırma
  ve UNIQUE altında aynı anahtarlı bilinçli tekrar kayıt sayısını 1'de tuttu.
  Exact ürün kanıtı `8a12848` commit'indedir.
  Genel idempotency servisi hâlâ açık kapsamdadır; production TLS/pool F031'de kapandı.
  PostgreSQL geçişli bağımlılık kapanışı fuzz aracının ayrı Cargo.lock'una da
  işlendi; iki `cargo deny --locked` grafiği yeniden yeşildir.

- **Dogfood kanıt referans bütünlüğü** (B-073): Append-only ürün kaydı tekil
  slug, repo içi kanıt kökü, exact harici ürün commit'i ve durum taşır. CORE
  FREEZE koruğu artık dogfood değişikliğinin kayıtlı etkin ürüne, gerçek K-işine,
  ürün kökü altındaki Zee dosyasına ve ilgili karar belgesine bağlı olduğunu
  doğrular. Geçici Git testi kayıtsız ürün/iş, kök dışı dosya ve alakasız karar
  kaçışlarını reddeder. Reproducer'ın semantik ilgisi insan review'undadır.

- **İlk gerçek ürün dogfood'u başladı** (K-163): legacy panel/legacy Next.js yönetim paneli için temiz
  `codex/k-163-legacy-panel` dalındaki `43171b9 → cef5ae3`, 495 satırlık üç
  Zee dikey dilimini kurdu. Duyuru, Argon2id yönetici oturumu/rol, CSRF CRUD,
  JSON site ayarları ve slug doğrulamalı taslak/yayında sayfa yaşam döngüsü
  7/7 hermetik testten ve gerçek TCP içerik+ayar+sayfa provasından geçti. İlk
  ürün sürtünmeleri ayrıca kaydedildi: P011 doğru çözüme götürdü; morfoloji ve
  zincirleme ergonomisi izleniyor; son kayıt silmede kaybolan boş CSV başlığı
  mevcut dille kapatılıp teste alındı. Dördüncü dilimde yerel PostgreSQL ve
  migration kanıtı da üretildi. Medya F030, production TLS/pool F031 ile
  kapandı; F032 organik 1000+ LOC bakım skor kartını ve readiness/liveness
  ayrımını kapatarak K-163'ü tamamladı.

- **Executable CORE FREEZE** (K-160A, ADR-059): Yazılı politika artık her
  `compiler/src` commit'ini exact freeze sınıfına zorlayan CI kapısıdır.
  Semantic feature yalnız dogfood/security/correctness olabilir;
  `dogfood-change` ürün, K-işi, reproducer, etkilenen proje, minimalite ve
  ADR/spec/RFC'nin tamamını ister. Maintenance sınıfıyla kaçış ve geçmiş beyan
  yeniden yazımı geçici Git depo testinde reddedilir. Kapanış denetiminde
  bulunan üç Clippy borcu davranış değiştirmeyen adlandırılmış istek tipleri ve
  açık filter/map zinciriyle temizlendi. B-071 kapandı.

- **Paket sahipliği ve CORE FREEZE** (K-160, ADR-059): Davranışsız
  `paket_modeli`, bağımlılık resolver'ı, registry protokol/taşıması,
  `artefakt_dogrulama` güveni ve `yayin` orkestrasyonu fiziksel olarak ayrıldı.
  Registry artefaktı doğrulamaya devreder; resolver taşıma ve imza ayrıntısını
  bilmez. Son C001 kaldırıldı; production SCC ve izin sayısı sıfırdır. B-070
  kapandı. Yeni compiler özelliği gerçek dogfood ihtiyacı kanıtlanmadıkça
  varsayılan olarak reddedilir.

- **Sürümlü Rust public facade'ı** (K-159, ADR-058): Desteklenen gömme
  sözleşmesi `dil::api::v1` altında exact allowlist'tir. Kök legacy modüller
  internal sınıflanır; smoke ve sızıntı kapısı yeni export'u bilinçli inceleme
  olmadan reddeder. Breaking/minor/patch ve yan yana v2 göç politikası yazılıdır.
  B-069 kapandı.

- **Gerçek test blast radius'u** (K-158, ADR-057): Faz matrisi v2, tekil
  birincil sahipliği korurken kritik test seçicilerine isteğe bağlı çoklu
  `ek_kapsam` bağlar. Geçersiz metadata fail-closed reddedilir; kanonik belge
  ve dinamik CI raporu birincil grupları, çapraz gerçek kanıtı ve mimari aşağı
  akışı ayrı gösterir. B-068 kapandı; sırada K-159 public facade vardır.

- **Uzun fuzz RC kapısı** (K-157, ADR-056): Dört bağımsız hedef haftalık ve
  elle tetiklenen işte 60'ar dakika explicit AddressSanitizer altında koşar;
  log, sonuç, crash ve provenance'lı korpus 90 gün saklanır. İlk exact taban
  dört hedefte 30'ar dakika, toplam 136.789.564 yürütme, sıfır crash/timeout/
  sanitizer bulgusu ve 3/3 seçili Miri testiyle geçti. B-067 kapandı.

- **Cache dışı fuzz korpus kalıcılığı** (K-156, ADR-055): Dört gecelik hedefin
  koşu sonu coverage korpusu artık başarılı/başarısız her koşuda 90 günlük
  ayrı artefakttır. Manifest kaynak commit, run/attempt, sabit toolchain ve
  cargo-fuzz sürümüyle her seed'in göreli yolu+SHA-256 özetini taşır. Linux ve
  macOS uyumlu doğrulayıcı değiştirilmiş/ağaçtan kopmuş artefaktı reddeder;
  kalıcı kaynak seed'i `cmin`, stable replay ve review ister. B-066 kapandı.

- **Strict form URL kodlaması** (K-176,
  ADR-054): Sorgu ve form alanlarının adı/değeri artık tam iki hexadecimal
  haneli `%xx` ve çözüm sonrasında exact UTF-8 ister. Eksik/kuralsız kaçış ile
  geçersiz UTF-8 rota çalışmadan deterministik 400 üretir; kayıplı dönüşüm
  kaldırıldı. Hedefli web testi ve `web` kipli kalıcı semantic fixture,
  `32247c7…` exact uygulama SHA'sını değişiklik beyanıyla eşleştirir; B-065
  kapandı.

- **Semantic regresyon provenance zinciri** (K-155, ADR-052): 17 tarihsel
  vaka `regression/v2.tsv` içinde gerçek tam `fixed_by`, kanıtlıysa
  `introduced_by` ve `guaranteed_since=0.8.0-dev` taşır. Eski introduced
  commit'ler reproducer olmadan tahmin edilmedi; açık `-` yalnız kanıtlı ata
  SHA'ya tek yönlü zenginleştirilebilir. Manifest testi commit varlığı ve ata
  yönünü ve Git koruğu v1→v2 soy ağacını denetler. K-155A/ADR-053 ilk
  başlık-regex kapısını kaldırdı: `1c73298…` sonrasındaki her compiler kaynak
  commit'i mesajından bağımsız semantic sınıf, kanıt ve gerekçe taşır;
  `semantic-bugfix` exact fixture ister. B-063/B-064 kapandı; grammar, runtime,
  tanı anlamı, RFC ve normatif spec değişmedi.

- **LSP tam-metin değişiklik ölçeği** (K-154, ADR-051): Ölçüm koşucusu artık
  açık `--lsp-olcek` kipinde 2.000/5.000/10.000/20.000 satır `didChange`
  p50/p95 eğrisini ve önceden belirlenmiş 250/500/1000 ms p95 çizgilerinin
  ilk aşımını raporlar; shared CI bu uzun gözlemi artefakta ekler. Mevcut yol
  tüm belgeyi değiştirip klonlar ve lexer/parser/resolver/checker/typed-HIR
  hattını bütünüyle yeniden kurar; incremental cache yoktur ve henüz
  optimizasyon yapılmadı. Exact temiz `59580cd…` uygulama commit'indeki 25
  örnekte 2k/5k/10k/20k p95 sırasıyla 167,281 ms / 1.081,746 ms /
  4.669,372 ms / 20.159,636 ms; 250/500/1000 ms eşiklerinin üçü de ilk kez
  5k'da aşılır. B-062 kapandı. Grammar, runtime, tanı, RFC ve normatif spec
  değişmedi.

- **Gerçek LSP process cold-start ölçümü** (K-153, ADR-050): Eski
  `lsp_soguk` gerçekte aynı süreç engine initialize'ıydı ve
  `lsp_engine_initialize` olarak düzeltildi. Yeni `lsp_process_cold_start`
  işletim sistemi process spawn'ından başlar; piped stdio üzerinden çerçeveli
  initialize isteği ve tam capabilities yanıtına kadar ölçer. Gerçek
  `olcum`→`dillsp` entegrasyon testi Tier-1 faz matrisindedir, CI release
  ikilisini açık yolla verir. Exact temiz `a2693d6…` uygulama commit'indeki 25
  örnek process p50 1,557 ms/p95 1,997 ms; engine p50 542 ns/p95 625 ns
  tabanını verdi. B-061/V1-P1-13 kapandı.

- **Tekrarlanabilir benchmark provenance** (K-152, ADR-049): JSON ve tarihçe
  v2'ye yükseldi. Her kalıcı satır tam 40 haneli Git SHA, ayrı milestone,
  temiz çalışma ağacı, platform+OS, CPU, fiziksel RAM, Rust, release profili
  ve gerçek `sample_count`/`warmup_count`/örnekleme semantiğini taşır. Tarihçe
  HEAD'den farklı SHA veya kirli ağaçta üretilemez. K-148 sayıları değişmeden
  exact `df737f…` kaynak commit'ine göçtü; RSS satırı 1 örnek/0 ısınmalı tek
  süreç-tepe görüntüsüdür. Envanter 610 test, 96 numaralı belge ve 47 kabul
  ADR'dir; B-060 kapandı. Grammar, runtime, tanılar ve normatif spec değişmedi.

- **GitHub Actions immutable SHA pinleri** (K-151, ADR-048): `checkout`,
  `cache`, `upload-artifact` ve `rust-toolchain` kullanımlarının tamamı resmî
  ref'lerden doğrulanmış 40 haneli commit SHA'lara sabitlendi. Pin TSV'si
  workflow kullanımıyla birebir; hareketli/kısa/kayıt dışı ref fail-closed'dur.
  Haftalık Dependabot yalnız inceleme PR'ı açar, otomatik merge yoktur.
  Checkout credential saklamaz, token yetkisi `contents: read` ile sınırlıdır.
  Bir supply-chain regresyonuyla envanter 610 test, 95 numaralı belge ve 46
  kabul ADR'ye çıktı; B-059 kapandı. Dil davranışı ve normatif spec değişmedi.

- **Dependency cycle kapısı** (K-150/K-160, ADR-047/059): Production exact
  graph'ı sahipler-arası SCC için fail-closed denetlenir. Tanı↔kaynak
  bütçesi çevrimi bağımsız `tani_politikasi`; checker↔HIR çevrimi private
  `semantic_model` sahibiyle kırıldı. `cozumleyici::Tur` yolu uyumludur.
  K-160 son paket/registry/tedarik SCC'sini saf model, resolver, taşıma,
  verification ve yayın sahiplerine ayırdı; geçici C001 silindi. Yeni çevrim
  reddedilir; production SCC ve izin sayısı sıfırdır. B-058/B-070/V1-P0-33
  kapandı; grammar, runtime semantiği, tanılar ve normatif spec değişmedi.

- **Production katman yönü fail-closed** (K-149, ADR-046): Bütün production
  Rust ağacı 35 üst sahip ve on katmana ayrıldı. Sürümlü TSV her sahibin exact
  doğrudan bağımlılıklarını ve sorumluluğunu taşır; yeni/kayıp modül,
  eklenen/kaldırılan kenar ve izin dışı temel→adaptör yönü testte durur.
  Tarayıcı yorum/metin/test-only kodu ayırır, hedefe özgü production yollarını
  birleşik korur ve `crate`/`dil` kök takma adıyla graph gizlemeyi reddeder.
  İnceleme sırasında morfoloji→paket SHA-256 ve tedarik→runtime Gregoryen
  takvim terslikleri ortak `guvenlik`/`zaman` temel sahiplerine taşındı; eski
  runtime tarih API'si yeniden dışa aktarımla uyumludur. Üç katman testi ve
  bir takvim regresyonuyla kaynak envanteri 607 test, 93 numaralı belge ve 44
  kabul ADR'ye çıktı; B-057/V1-P0-32 kapandı. Dil sözdizimi, runtime
  semantiği, tanılar, RFC ve normatif spec değişmedi.

- **p50/p95 performans tarihçesi ve CI artefaktı** (K-148, ADR-045; K-152/
  ADR-049 provenance revizyonu): Eski altı
  iş yükü/tek medyan çıktısı; parse, resolver/checker+HIR kanıtı, tam
  kaynak→typed-HIR, runtime başlangıcı, 100 bin tur yürütme, LSP
  cold/open/change ve Unix tepe RSS olarak dokuz ayrı gözleme bölündü. İki
  ısınma+25 release turu ham örnek, min/max ve nearest-rank p50/p95 üretir.
  V2 JSON'u, Markdown raporu ve sürümlü TSV tarihçesi exact Git SHA,
  milestone, temiz ağaç, OS/CPU/RAM/Rust/release ve gerçek örnekleme bağlamını
  taşır; bozuk/yinelenen tarihçe fail-closed'dur.
  Linux CI sonucu summary ve 90 günlük indirilebilir artefakttır, gürültülü
  shared runner'da hard gate değildir. Eşik yalnız sabitlenmiş adanmış
  benchmark koşucusunda açık `--esik-yuzde` seçeneğiyle etkinleşir. Yedi
  koşucu birim testi ve bir mimari kablolama testiyle kaynak envanteri 603
  test, 92 numaralı belge ve 43 kabul ADR'ye çıktı; B-040/V1-P1-12 kapandı.
  Dil sözdizimi, çalışma semantiği, tanılar, RFC ve normatif spec değişmedi.

- **Sürümlü semantic regresyon korpusu** (K-147, ADR-044): Parser, checker,
  typed HIR, runtime, morphology, concurrency ve security için 17 tarihsel
  bug, `regression/` altında minimal `.dil` kaynaklarına indirildi. Sürümlü
  manifest her vakayı K-kimliği, birincil faz, çalışma kipi, kesin tanı spanı,
  program exit'i ve sıralı stdout ile bağlar. Ağaç birebirliği, 4 KiB/32 satır
  minimality sınırı ve gerçek faz gözlemleri tek veri-güdümlü testtedir;
  Git-tabanlı CI koruğu eski vaka+bug+yol üçlüsünün silinmesini engeller.
  K-146 raporuna ayrı 22. `Semantic regression` fazı eklendi. Typed-HIR
  runtime API'si çıkış kodunu kaybetmeden gözlenebilir oldu. B-039 kapandı;
  kaynak envanteri 595 test, 91 numaralı belge ve 42 kabul ADR'ye çıktı. Dil
  sözdizimi, tanı anlamları, RFC ve normatif spec değişmedi.

- **Faza özgü test matrisi** (K-146, ADR-043): Cargo'nun gerçekten derlediği
  lib/bin/integration/doctest envanteri 21 birincil faza tam sahipletildi.
  Sahipsiz veya yinelenen test, boş seçici, kayıp regression/fuzz/conformance
  yolu ve bayat matris fail-closed'dur. Linux, macOS ve Windows her biri kendi
  `cfg` envanterini faz faz çalıştırıp count, pass/fail/ignored ve duvar
  süresini job summary ile indirilebilir Markdown artefaktına yazar. Süre
  correctness/performance eşiği değildir; performans trendi B-040'ta kalır.
  Altı koşucu birim testi ve CI kablolama regresyonuyla kaynak envanteri 592
  test ve 41 kabul ADR'ye çıktı; B-038 kapandı. Dil semantiği, tanılar, RFC ve
  normatif spec değişmedi.

- **Kanonik Rust biçim kapısı** (K-145, ADR-042): 53 dosyada kalan 349
  `rustfmt` fark bloğu tek, davranışsız toplu dilimde temizlendi. Rust 1.93.1
  araç zinciri `rustfmt` bileşenini açıkça sabitler; üç platformlu CI artık
  `cargo fmt --all -- --check` ile yeni biçim borcunu reddeder. Fiziksel faz
  bütçeleri kanonik satır ölçüsüne, K-144 eğilim tabanı 49 kritik işleve bir
  kez yeniden alındı. Dil semantiği, tanılar, RFC ve normatif spec değişmedi;
  B-037 kapandı.

- **Kritik işlev büyüme/karmaşıklık eğilimi** (K-144, ADR-041): Sabit
  Rust/Clippy zinciri üretim `lib` ve `dil` ikilisindeki 48 kritik işlevi
  incelenmiş TSV tabanına bağlar. 80 satır veya 12 bilişsel karmaşıklık yalnız
  izlemeye giriş; satırda tabanın %10'u (+8..+24), karmaşıklıkta %20'si
  (+2..+5) gözden geçirme payıdır. Yeni/kayıp kritik işlev, pay aşımı veya
  bayat [eğilim raporu](islev-egilimi.md) Ubuntu CI'ı fail-closed durdurur;
  kaynak içi `allow`, `--force-warn` kapısını geçemez. Beş ölçüm/kimlik/eşik
  birim testi ve bir mimari kablolama testiyle envanter 585 test ve 39 kabul
  ADR'ye çıktı; B-035 kapandı. Dil semantiği, kullanıcı tanısı ve normatif
  spec değişmedi.

- **Temiz kaynak arşivi:** Paylaşım paketi artık çalışma klasöründen değil,
  yalnız commit edilmiş `HEAD` içeriğini alan `git archive` tabanlı
  `scripts/temiz-kaynak-arsivi.sh` kapısından üretilir. Build hedefleri, fuzz
  ikilileri, profiler çıktıları, Finder metadatası ve yerel ZIP'ler kök ignore
  politikasında açıkça dışarıdadır; kalıcı fuzz korpusu korunur. Sabit commit
  zamanı, arşiv içi dosya SHA-256 manifesti ve ZIP yan özeti aynı `HEAD` için
  byte-byte tekrar üretim ve bağımsız bütünlük denetimi sağlar.

- **Ortak production web durumu ve worker modeli**
  (K-137, RFC-0017/ADR-034/spec-12): `--web-proxy` oturum, revoke, expiry ve
  endpoint/CSRF/Argon2id oran pencerelerini proje kökündeki
  `.zee/web-durumu-v1.json` kalıcı deposunda süreçler arası atomik tutar.
  Proxy kimliği yalnız proxy'nin yeniden kurduğu tek-hop `Forwarded`
  başlığındaki kanonik IP'dir; XFF kimlik değildir. V1 process başına tek
  worker'dır; `--web-worker-port` ile aynı kaynağı çalıştıran N ayrı süreç
  reverse proxy arkasında aynı depoyu paylaşır. İki gerçek CLI sürecinde
  login, worker geçişi, restart, çapraz logout ve iki sürece dağıtılmış altıncı
  yanlış parola denemesinin Argon2id öncesi 429 olması kanıtlıdır. Bozuk
  depo/symlink/kapasite hatası fail-closed 503'tür. Envanter 547 test ve 32
  kabul ADR'ye çıktı; B-046 kapandı.

- **Protokol-kesin LSP JSON-RPC** (K-138, ADR-035/spec-24): Elle yazılmış
  JSON parser'ı RFC 8259 sayı durum makinesini kullanır; doğrulanmış sayı
  lexeme'ini `f64`'e çevirmeden kayıpsız saklar. Baştaki sıfır, eksik
  kesir/üs, `NaN/Infinity` ve çözülmüş adı yinelenen nesne alanı reddedilir.
  UTF-8/sözdizimi `-32700`, bozuk tek-nesne zarf `-32600`, bilinmeyen yöntem
  `-32601`, bozuk yöntem parametresi `-32602` üretir; kimliksiz bildirim
  response almaz. Geçerli çok büyük/kesirli `id` aynı lexeme ile döner ve tam
  okunmuş UTF-8 dışı gövde sunucuyu düşürmez. Ayrıştırıcı 300 satır bütçeli
  `lsp/json.rs` sahibine ayrıldı. Sayı differential, escaped duplicate,
  standart hata/notification, ardışık framing ve mimari regresyonlarıyla
  envanter 556 test ve
  33 kabul ADR'ye çıktı; B-051 kapandı.

- **Tek kanonik web proxy origin'i** (K-139, ADR-036/spec-12): CLI'a özel
  `GuvenliOrigin` parser'ı kaldırıldı. `--web-proxy`, `Host`, tek-hop
  `Forwarded host` ve unsafe `Origin`, outbound allowlist ile aynı
  `AgHedefi` DNS/IPv6/port kimliğine ayrıştırılır. Varsayılan HTTPS `:443` ve
  DNS harf farkı kanoniklenir; geçersiz etiket/IPv6/port, yol, sorgu,
  kullanıcı bilgisi ve HTTP fail-closed reddedilir. Production listener sabit
  `127.0.0.1`e bind eder ve kabul edilen socket peer'ini ayrıca loopback olarak
  doğrular. Origin olumluları/olumsuzları ve mimari drift koruğuyla envanter
  558 test ve 34 kabul ADR'ye çıktı; B-052 kapandı.

- **Byte tabanlı HTTP/1.x istek sınırı** (K-140, ADR-037/spec-11/12/24):
  Socket başlığı metne çevrilmeden CRLF, request-line, header adı/değeri ve
  framing olarak doğrulanır. Yalnız origin-form ve HTTP/1.0/1.1 kabul edilir;
  bare-LF/CR, obs-fold, NUL, absolute/authority/asterisk-form, fragment,
  Transfer-Encoding, duplicate/kuralsız Content-Length, fazla/eksik veya
  UTF-8 dışı gövde fail-closed'dur. Gövde tahsisi ancak CL 64 KiB sınırından
  geçince yapılır; kayıplı UTF-8 kaldırılmıştır. Yedi protokol ve mimari
  regresyon, beş kalıcı fuzz seed'i ve ayrı ham-byte libFuzzer hedefiyle
  envanter 566 test ve 35 kabul ADR'ye çıktı; B-053 kapandı.

- **Rust tedarik zinciri ve gerçek offline vendor kapısı**
  (K-141, ADR-038): Compiler ve fuzz bağımlılık grafikleri sabit
  `cargo-deny 0.20.2` ile push/PR ve günlük takvimde güncel RustSec, izinli
  SPDX, duplicate/wildcard ve yalnız crates.io kaynak politikasından geçer.
  Advisory ignore boştur; iki exact major-geçiş istisnası teknik gerekçelidir.
  CI'daki kilitsiz fallback kaldırıldı ve bütün Cargo komutları `--locked`
  oldu. İki lock'un ortak vendor ağacı iki kez üretilip sıralı yol+dosya
  SHA-256 manifestiyle eşitlenir; sonra boş Cargo home'da compiler ve fuzz
  gerçekten offline derlenir. Dört statik regresyonla envanter 570 test ve 36
  kabul ADR'ye çıktı; B-054 kapandı. `publish = false`, ürün lisansı seçilene
  kadar yanlışlıkla crates.io yayınına izin vermez; bağımlılık lisans listesi
  Zee'nin lisansı değildir.

- **Playground ön-tahsis girdi bütçesi** (K-143, ADR-040): Merkezî kaynak
  profili playground kaynağını 8 MiB, soru girdisini 1 MiB ve 4.096 satırla
  sınırlar. Native köprü byte/satır boyunu sahipli `Vec<String>` öncesinde;
  ABI v3 byte boyunu kayıtlı tamponu kopyalamadan reddeder. Tarayıcı limitleri
  WASM'den okur, UTF-8 boyunu ara byte dizisi kurmadan hesaplar ve
  `TextEncoder.encodeInto` ile doğrudan exact tampona yazar. Sınır/bir-fazlası
  native ve gerçek wasm32 Node testleri, kaynak+soru kipli beş kalıcı seed
  fuzzer'ı, 61 saniyede 1.709.869 çağrılık kampanya ve allocation-order mimari
  kapısıyla envanter 579 test ve 38 kabul ADR'ye çıktı; B-056/V1-P1-11
  kapandı. ABI v2 hostu v3 modülle bilinçli uyumsuzdur; depodaki tek dosyalık
  playground yeniden üretilmiştir.

- **Sürümlü ve hasım hosta dayanıklı WASM C ABI** (K-142, ADR-039): Eski
  kayıt dışı `slice/Vec::from_raw_parts` köprüsü kaldırıldı. ABI v2 yalnız
  modülün ayırdığı başlangıç pointer'ı ile exact boyu sahipli kopyaya alır;
  null/iç/yabancı pointer, taşkın boy, sonuç tamponunu girdi sayma ve geçersiz
  UTF-8 çekirdekten önce görünür hata olur. Sonuç toplamı ayrı kayıttan
  sorgulanır; yanlış veya çift bırakma allocator durumunu değiştirmez.
  Playground şablonu sürümü, linear-memory zarfını, uzunluk önekini ve strict
  UTF-8'i doğrulayıp bütün tamponları `finally` içinde bırakır. Beş yeni
  regresyon, dört byte seed, gecelik ayrı fuzz işi, gerçek wasm32 Node host
  round-trip'i ve 1.745.134 çağrılık ilk kampanyayla envanter 575 test ve 37
  kabul ADR'ye çıktı; B-055/V1-P1-10 kapandı. Bu v2 sahiplik tabanı K-143'ün
  ABI v3 ön-tahsis bütçesiyle genişletildi.

- **Normatif otorite ve v1 kapıları** (K-081, ADR-010): geçerli dilin kesin
  davranışını spec anlatır; RFC değişikliği yetkilendirir ama spec+conformance
  testi aynı değişiklikte güncellenmeden yürürlüğe girmez. Kaynak denetimli
  [v1.0 sürüm kapıları](v1-surum-kapilari.md) bağlayıcıdır. RFC-0006 ve
  RFC-0011 güncel gerçeklemeyle uzlaştırıldı; RFC-0015 production web/eylem
  sınırını taslağa aldı.
- **Deneysel web korkuluğu** (K-082): gerçek TCP sunucusu artık sıradan
  `dil çalıştır` ile açılmaz; yalnız açık `--deneysel-web` opt-in'i ve görünür
  production uyarısıyla localhost'ta çalışır. Panel örnekleri eğitim demosu
  olarak yeniden etiketlendi; giriş/kaydet/sil/çıkış durum değişiklikleri POST
  kontrolüne alındı. GET ile silmeme regression testidir.
- **Açık işlem imzası** (K-083): başlangıç için `sayıyı al` aynen kalır;
  public/paket API'si `sayıyı Ondalık olarak al` yazabilir. Açık gövde hiç
  çağrılmasa da denetlenir, imza çağrıyla değişmez. Liste/sözlük/Seçenek/Sonuç
  ve yapı tür yazımları desteklenir; TamSayı→Ondalık runtime değeri de
  genişler. T037/T038, golden 33 ve çağrı sırası permütasyonlarıyla 270 test.
- **Atomik kalıcı dosya** (K-084, RFC-0016): `dosyasına ... yaz/ekle`
  aynı klasörde geçici dosya + disk eşzamanlama + atomik replace kullanır.
  Unix `flock` / Windows `LockFileEx` süreç kilidi iki yazarın
  güncellemesini korur; ani süreç sonu kilidi bırakır. Biçimleyici, paket
  bildirimi ve `proje.kilit` tek-dosya yazmaları da aynı çekirdeğe taşındı.
  Hata enjeksiyonu, eşzamanlı okuyucu ve iki bağımsız CLI yazarıyla 276 test.
- **Gerçek son tarih iptali** (K-085, RFC-0011/spec-09): `içinde` artık
  gövdeyi bitirip geç kaldığını sonradan söylemez. Mutlak deadline
  blok/işlem/döngü sınırlarında denetlenir; `bekle` kalan süreye kırpılır,
  HTTP bağlantı/yazma/okuma tek bütçeyi kullanır. İç içe bloklarda yalnız
  deadline sahibi `yetişmezse` kolu çalışır; iptal sonrası yan etki yoktur.
  Ç001 iç nöbetçisiyle 123 katalog kodu ve toplam 279 test; playground'da
  ardışık sanal beklemeler de aynı son tarih anlamını taşır.
- **Tam public işlem sözleşmesi** (K-086, spec-10): yerel başlangıç
  işlemlerinde `sayıyı al` çıkarımı korunur; birim/paket işlemleri bütün
  parametrelerini `<ad> <Tür> olarak al` ve dönüşünü `<Tür> döndürür` ya da
  `değer döndürmez` ile açıkça bildirir. Dönüş türü gövdeyle ve bütün akış
  yollarıyla kanıtlanır (T039–T042). v1 public modeli bilinçli monomorfik
  kaynak ABI'sidir; kırıcı semver sınırı spec'te sabittir. Standart kitaplık
  bu sözleşmeye geçirildi; `liste_araclari` somut Ondalık ABI'si taşır,
  TamSayı listeleri çağrıda genişler ve uç sonuçları da `71,0` gibi Ondalık
  döner. Liste/özyineleme/çağrı sırası ve gerçek paket
  olumsuzuyla ve örtük yeniden-dışa-açma yasağıyla toplam 289 test, 127
  katalog kodu. V1-P0-01 kapandı.
- **Uygulama eylemi ve yöntemli web adaptörü** (K-087, RFC-0015/spec-11):
  açık imzalı `eylem` aynı çağrı sözdizimiyle web/CLI/görev/test bağlamında
  kullanılır; HTTP yanıtı/çerezi ve geri alınamayan ekran/girdi/donanım etkisi
  taşıyamaz. GET/HEAD'in doğrudan ve çağrı grafiğindeki dolaylı yazması T045,
  rota içi uygulama yazması T046'dır. Açık GET/HEAD/POST/PUT/PATCH/DELETE,
  404/405, 64 KiB+100 alan için 413 ve 30 saniye için 504 gerçeklendi. Her
  eylem çalışma hatası ya da başarısız Sonuçta geri alınan iç içe dosya
  savepoint'i taşır; IO desteği yoksa C021 ile fail-closed davranır. Gerçek TCP
  HEAD gövdesini bastırır. Geri alma, araya giren başka yazarın verisini
  ezmek yerine C021 verir. Hermetik ve gerçek CLI geri alma kanıtlarıyla 304
  test ve 134 katalog kodunda V1-P0-02
  kapandı. Çok-dosyalı süreç-çökmesi atomikliği bu sözün parçası değildir.
- **Production web güvenlik profili** (K-088, RFC-0017/spec-12): unsafe rota
  ilk satırda public/oturum/rol politikasını açıklar (T049/T050), zorunlu
  alanlar 400 ve otomatik synchronizer CSRF 403 kapısından geçer. Native
  runtime 256 bit OS CSPRNG belirteç üretir, oturum kimliğinin yalnız SHA-256
  özetini sunucuda tutar; girişte session+CSRF rotation, 30 dakika ömür,
  sunucu rolü ve logout revoke vardır. `dil parola-özeti` Argon2id PHC üretir.
  `--web-proxy https://host` loopback'teki HTTPS proxy için Host/proto/Origin
  doğrular; `__Host-` Secure+HttpOnly+SameSite=Lax çerez, HSTS/CSP ve CRLF/
  request-smuggling korkuluklarını uygular. 321 test ve 138 katalog koduyla
  V1-P0-03 kapandı.
- **Sürümlü morfoloji profili** (K-089, RFC-0018/spec-13): çözümleyici ve
  LSP'nin ayrı ek listeleri `zee-tr-1` tek kaynağında birleşti. İyelik ayrı
  soyut kimlikle iki katmanlı üretilir; düzenli kök×bütün tek/iki katman
  `üret→çöz` property'leri, ters ses değişimi ve A002 belirsizlik korpusu
  tablo snapshot'ını korur. Rename artık `fiyatıyla→elmasıyla` zincirini de
  giydirir. `proje.dil` profili sabitler (P011), `proje.kilit` v2 ana/paket
  profilini taşır; `dil morfoloji [kelime]` kararı görünür kılar. 333 test ve
  139 katalog koduyla V1-P1-02 kapandı.
- **Deterministik görev scheduler'ı** (K-090, RFC-0011/spec-14):
  `eşzamanlı olarak` görevleri artık kaynak sırasında bitiren sahte bir blok
  değildir; `hepsini bekle`, tek iş parçacıklı future scheduler'ında görevleri
  `bekle` noktalarında dönüşümlü ilerletir. Kaynak sırası bağlayıcıdır; 2 sn ve
  1 sn bekleyen görevler toplam 2 sn'de biter ve data race oluşmaz. İlk
  yönetilmemiş hata bekleyen kardeşleri sonraki yan etkileri başlamadan iptal
  eder; dış son tarih bütün görev ağacına yayılır. T051 her grubu aynı
  sözcüksel kapsamda tek join'e zorlar; T033 sonuç erişimini join sonrasına
  bırakır. Eylem transaction'ları savepoint sahipliği için atomik scheduler
  dilimidir. 343 test ve 140 katalog koduyla V1-P1-03 kapandı; çok çekirdekli
  paralellik v1 sözü değildir.
- **Yapılandırılmış Hata değeri** (K-091, RFC-0008/spec-15): `Sonuç<T>` hata
  tarafı artık kod, Türkçe mesaj, `Seçenek<Hata>` neden zinciri ve Metin
  sözlüğü verisi taşır. `hatanın kodu` `göre` ile eşlenir; neden `varsa` ile
  güvenle açılır; `hatanın json metni` bütün zinciri sabit anahtar sırasında
  verir. Eski `"..." hatasını döndür` `GENEL` koduyla aynı kullanıcı çıktısını
  korur; yerleşik denemeler kararlı kod üretir. S044/T052 olumsuzları,
  yeniden yayma ve geriye uyum regresyonlarıyla 350 test ve 142 katalog
  kodunda V1-P1-04 kapandı.
- **Keyfî hassasiyetli Ondalık** (K-092, RFC-0013/spec-16): ilk bootstrap'ın
  dokuz kesir hanesi ve i64/i128 katsayı sınırı kaldırıldı. Tek Ondalık türü
  keyfî uzunlukta katsayı+ölçek taşır; toplama/çıkarma/çarpma ve sonlu onluk
  bölüm tamdır. Sonsuz bölüm platformdan bağımsız 34 anlamlı haneye,
  yarımlar sıfırdan uzağa yuvarlanır. Uzun sabit ve negatif metin dönüşümü,
  büyük/küçük değer, exact `1/8`, `10/3` bağlamı, karşılaştırma, JSON/para ve
  i64 daraltma taşmasıyla 356 test yeşildir. S032 emekliye ayrıldı; katalog
  141 etkin + 1 ayrılmış kod taşır. Release ölçümünde 20 bin exact Ondalık
  toplaması 6,8 ms'dir; `BigInt` kapasite maliyeti ölçüm arşivinde görünürdür.
  V1-P1-01 kapandı.
- **Değer semantiği ve gezme imleci** (K-093, RFC-0019/spec-17): bütün
  kullanıcı değerleri derin kopyadır; gizli paylaşılan alias yoktur. Liste
  gezmesi öğeyi kopyalayıp turun sonunda aynı sıraya geri yazan değer-sonuç
  imlecidir; alan yazma ve yeniden bağlama aynı kalıcı sonucu verir. Gezilen
  kaynağı ekleme/silme/yeniden bağlama, sözlüğe yazma ve aynı kaynağı iç içe
  gezme T053 ile derlemede durur; doğal `sayılara` yüzeyi artık A002 yerine
  doğrudan bu tanıyı alır. Derin kopya, beş olumsuz ve 1–24 uzunluk property
  korpusuyla 360 test ve 142 etkin + 1 ayrılmış kod yeşildir. V1-P1-05'in
  makine tarafı tamamdır; gerçek çocuk/profesyonel usability sonucu gelmeden
  kapı dürüstçe açık kalır.
- **Tekrar üretilebilir paket yayını** (K-094, ADR-006/RFC-0020/spec-18):
  `dil anahtar üret` var olanı ezmeyen Unix 0600 Ed25519 yayıncı anahtarı ve
  yeni proje iskeletlerinde `*.zee-anahtar` Git dışlama koruması;
  `dil paketle` platform metadata'sı taşımayan sıralı `.zep`, SPDX 3.0.1
  JSON-LD SBOM, in-toto/SLSA v1 provenance ve üçünü ad+boyut+SHA-256 ile
  imzalayan `zee-yayin-v1` üretir. `SOURCE_DATE_EPOCH` ile dört dosya byte-byte
  yinelenir. İmza/paket/SBOM/provenance oynama, path traversal/fazladan byte,
  sembolik bağ ve yerel yol bağımlılığı fail-closed testlidir. P012 eklendi.
  Bu yayın öz-imzası tek başına registry güveni değildir; TUF tarzı eşik kök,
  targets/snapshot/timestamp, doğrulanmış cache, yanked ve duyuru tamamlanana
  kadar V1-P1-07 açık kalır. Doküman tazelik testiyle toplam 370 test yeşildir.
- **Registry metadata güven zinciri** (K-095, ADR-006/RFC-0020/spec-19): ağ
  dışı SHA-256 ile sabitlenen root, rol başına Ed25519 eşik, hem eski hem yeni
  root eşiğini isteyen ardışık rotasyon ve kanonik kapalı JSON zarfı çalışır.
  Tek güncelleme saatiyle timestamp→snapshot→targets sürüm/boyut/SHA-256
  bağları; kalıcı sürüm+aynı-sürüm-özeti rollback/equivocation koruması ve
  zincir tamamlanmadan ya da daha yeni sonuçtan sonra bayat sonuçla durumu
  uygulamayan transaction sınırı kuruldu. Exact
  hedef yayıncı yetkisi dört yayın dosyasını bağlar; yanked ve etkin kritik
  duyuru varsayılan reddedilir. Eşik/rotasyon, rollback/expiry, aynı sürümlü
  farklı içerik, mix-and-match, fast-forward zehirleme, kanonik/limit,
  bozuk kalıcı durum, RFC3339 takvim, yanlış yayıncı ve politika olumsuzlarıyla
  toplam 378 test;
  P013/P014 ile 144 etkin + 1 ayrılmış tanı yeşildir. Taşıma, kalıcı durum
  dosyası, doğrulanmış cache/offline ve CLI bitmeden V1-P1-07 açık kalır.
- **K-016 çağrı karar deneyi** (K-096, RFC-0006/spec-02): çalışan A yüzeyi
  nihai V1 kararı sayılmadan önce serbest üretim, kör A/B/C kartları, üç sıra
  grubuna dengeli dağıtım, çocuk/profesyonel alt grup eşikleri ve yedi teknik
  bağlam önden bağlandı. B güçlü çıkarsa ikinci çağrı sözdizimi eklenmeyecek;
  B-003/RFC-0021 expression grammar turuna dönülecek. Anonim katılımcı/özet
  şablonları ve kişisel veri koruması hazırlandı. Parser'ın eski tanım-sırası
  yorumu güncel ön-tarama/karşılıklı özyineleme gerçeğine düzeltildi. Dil
  yüzeyi değişmedi; 378 test tabanı korunuyor ve V1-P0-07 gerçek 10 çocuk +
  5 profesyonel sonucu gelene kadar açık kalıyor.
- **Katmanlı ifade grameri** (K-097, RFC-0021/spec-20/ADR-002): mevcut yüzey
  primary → erişim/postfix → çağrı → aritmetik → birleştirme → karşılaştırma
  → boolean güç sırasına ve tam bölge tüketimine bağlandı. `ile`/`ve` gibi
  bağlama göre ayrılan kelimelerin sahipliği, en uzun işlem adı, karışık
  `ve/veya` için S030, işlem-adı kuyruğunun postfix'i gölgelememesi, tam
  sıfır-argüman çağrısının korunması ve tanımsız katman birleşimi için
  fail-closed davranış normatifleşti. Böylece görünür `sayısı` işleminin
  `metnin sayısı` ifadesini yanlış S019'a çevirdiği çakışma, geçerli tam
  sıfır-argüman çağrılarını bozmadan kapandı. Yeni ifade özelliği katman,
  çakışma matrisi, AST/lowering,
  formatter ve olumlu/olumsuz conformance kanıtı olmadan parser'a dal
  ekleyemez. Sekiz bağımsız testle toplam 386 test yeşil; kullanıcı yüzeyi
  değişmeden V1-P0-08 kapandı. Fiziksel parser parçalama B-005/K-099'da
  tamamlandı.
- **Core AST intrinsic/yetkinlik sınırı** (K-098, ADR-011): HTTP, sensör,
  CSRF ve parola için dört alan-özel AST varyantı kaldırıldı. Mevcut Türkçe
  cümleler parser'da ad alanlı kararlı kimlik ve sıralı argüman taşıyan tek
  `Intrinsic` düğümüne indirilir. Tür imzası, gereken ağ/donanım/web oturumu/
  kriptografi yetkinliği ve statik etki merkezi kayıttadır; checker, etki
  çözümleyici ve runtime aynı kaydı tüketir. `kapı kapalıysa` ikinci intrinsic
  yerine genel olumsuzlamayı kullanır. Kimlik tekilliği, dört lowering ve iki
  tür olumsuzuyla yedi yeni test; mevcut HTTP/sensör/web/parola regresyonları
  dahil toplam 393 test yeşildir. Kaynak semantiği değişmeden B-004 ve
  V1-P0-09 kapandı; fiziksel handler ayrımı B-005/K-099'da, izin politikası
  B-023/K-127'de tamamlandı.
- **Derleyici fiziksel faz sınırları** (K-099, ADR-012): 2709 satırlık parser
  cümle/ifade, 3067 satırlık checker cümle/ifade/çağrı ve 3181 satırlık
  runtime cümle/ifade handler modüllerine ayrıldı. Kökler 1160/963/1965 satıra
  indi; alt modüller yalnız `pub(super)` görünür ve public Rust API değişmedi.
  Üç kaynak-mimari testi büyük handler'ların köke dönmesini ve ilan edilmiş
  faz bütçelerinin aşılmasını engeller. Hata kataloğu taraması yeni alt
  modülleri özyinelemeli kapsar. 393 davranış testi aynen korunup toplam 396
  test yeşil kaldı; kaynak semantiği değişmeden B-005/V1-P0-10 kapandı.
- **Checker semantik katmanları** (K-100, ADR-013): 963 satırlık checker kökü
  143 satırlık geçiş orkestrasyonuna indirildi. Türler, bağlam, sembol, akış,
  çağrı, public sözleşme, etki/yetkinlik ve dönüş/control-flow ayrı tek-sahipli
  modüllere taşındı; eski `eylem.rs` aynı davranışla `cozumleyici/etki.rs`
  oldu. Public `Tur`, `VeriTuru`, `SozlukDegerTuru` ve `ad_cozumle` API'si
  yeniden dışa aktarımla korundu. İki yeni sahiplik/API testiyle mimari test
  sayısı beşe, toplam test sayısı 398'e çıktı. Kaynak semantiği değişmeden
  B-006/V1-P0-11 kapandı; semantic ID ardılı B-010/K-101 ile tamamlandı.
- **Semantic kimlik modeli** (K-101, ADR-014): `YapiId`, `IslemId` ve
  `SymbolId` newtype'ları eklendi. `Tur::Yapi` çıplak `usize` yerine kimlik
  taşır; fiziksel yapı konumu ayrı dizinden çözülür. İşlem imzaları ve
  özyineleme kaydı `IslemId`, yerel sembol tablosu ad→(`SymbolId`, tür)
  kullanır. Checker çözülmüş değişken, yapı oluşturma ve işlem çağrısı AST
  düğümlerini ID ile bağlar; kaynak adını tanı/runtime geçişi için korur. Üç
  davranış ve bir mimari testle toplam 402 test yeşildir; kaynak semantiği
  değişmeden B-010/V1-P0-12 kapandı; faz modeli B-018/K-102 ile tamamlandı.
- **Tür güvenli derleyici fazları** (K-102, ADR-015): `KaynakMetni`,
  `TokenAkisi`, `AyristirilmisAst`, crate-içi `BaglanmamisProgram` ve yalnız
  başarılı checker'ın üretebildiği `BaglanmisProgram` eklendi. Standart
  denetle/çalıştır/dene hatları fazlı API ve `calistir_baglanmis[_io]`
  girişlerini kullanır; eski raw `Program` API'si yalnız uyumluluk adaptörüdür.
  Parsed AST'nin doğrudan yürütülemeyeceği compile-fail dahil dört yeni testle
  toplam 406 test yeşildir. Kaynak semantiği değişmeden B-018/V1-P0-13
  kapandı; ayrı typed HIR B-019 olarak açık kaldı.
- **Typed HIR çekirdeği** (K-103, ADR-016): checker'ın her denetlenmiş ifade
  için ürettiği tür ve `SymbolId`/`IslemId`/`YapiId` bağı ayrı
  `HirIfadeBilgisi` kaydına taşındı; `HirDugumId` program içi semantic düğüm
  kimliğidir. `BaglanmisProgram` artık zorunlu `HirProgram` sahibidir; AST
  tanı ve v0 uyumluluğu için salt-okunur kalır. İki davranış ve bir mimari
  testle toplam 409 test yeşildir. K-104 ardılı runtime geçişini tamamladı.
- **HIR-bağlı standart runtime** (K-104, ADR-016): `calistir_baglanmis[_io]`
  ve kaynak test hattı `CalistirmaProgrami::Hir` kullanır. Değişken erişimi/
  güncellemesi, işlem çağrısı ve yapı oluşturma yalnız `HirBagi` ile çözülür;
  HIR kolu kaynak adına geri düşmez. Eşzamanlı görev özgün ifade düğümünü
  ödünç alır. Kaynak adlarını bilerek bozan iki regresyon ve standard-hat
  mimari testi ve dönüşsüz çağrı HIR kanıtıyla toplam 413 test yeşildir;
  B-019/V1-P0-14 kapandı.
- **Sınırlı native ağ I/O** (K-105, ADR-017): HTTP istemcisi kaynakta deadline
  yoksa 30 saniyelik mutlak varsayılan bütçe kullanır; bütün adres denemesi,
  yazma ve her okuma aynı kalan süreyi tüketir. Başlıklar dahil wire yanıt
  8 MiB ile sınırlıdır. Yerel sunucu başlık+gövdeyi kabulden başlayan mutlak
  10 saniyede tamamlatır, bayt damlatmak süreyi yenilemez ve aşım 408 olur;
  yanıt socket'i de 10 saniye yazma zaman aşımı taşır. Üç loopback sınır
  testiyle toplam 416 test yeşildir; B-025'in ağ dilimi ve V1-P0-15 kapandı.
- **Sınırlı web oturum deposu** (K-106, ADR-018): process içi depo 4096
  toplam/1024 anonim kayıtla sınırlıdır. Süresi dolanlar önce silinir; kota
  dolunca en uzun süredir kullanılmayan anonim kayıt oluşturma sırasıyla
  deterministik tahliye edilir. Kimlikli oturum rastgele düşürülmez; yalnız
  kimlikli doluluk yeni girişi fail-closed reddeder. 10/30 dakikalık ömür
  mutlak ve kaymazdır. Üç yeni kota/tahliye testiyle toplam 419 test yeşildir;
  V1-P0-16 kapandı. Bu tarihsel çok süreç açığı daha sonra K-137/ADR-034 ile
  kapandı; process-local adaptör yalnız deneysel/öğretici kipte kaldı.
- **Sınırlı LSP girdisi ve sıkı JSON** (K-107, ADR-019): `dillsp` gelen
  çerçeveyi tahsis öncesi 8 KiB başlık/8 MiB gövdeyle ve tek `Content-Length`
  ile sınırlar. Mini JSON 128 iç içelik/100 bin düğüm bütçesi taşır; yanlış
  veya eksik surrogate çifti, tek düşük surrogate ve kaçışsız kontrol
  karakteri reddedilir. Üç framing ve dört parser testiyle toplam 426 test
  yeşildir; B-047/V1-P0-17 kapandı.
- **Zorunlu HIR kaynak aralığı** (K-108, ADR-020): her
  `HirIfadeBilgisi` kimlik+tür+bağın yanında zorunlu `HirKaynakAraligi`
  taşır. Değişkenlerde lexer'ın kesin satır/sütun/uzunluğu, diğer mevcut AST
  ifadelerinde sahte kesinlik üretmeyen kaynak satırı zarfı vardır; sıfır
  konum `NonZeroUsize` ile kurulamaz. Bir yeni davranış ve genişletilmiş HIR+
  mimari kanıtlarıyla toplam 427 test yeşildir; B-020/V1-P0-18 kapandı.
- **Production panic audit'i** (K-109, ADR-021): lexer/parser, formatter/LSP,
  checker/runtime, paket/arşiv/kalıcı IO ve CLI/ölçüm yollarındaki 46 doğrudan
  `unwrap`/`expect`/panic noktası sonuç veya kodlu tanıya çevrildi. Dört
  production crate kökü test dışında `unwrap`, `expect`, `panic!`,
  `unreachable!`, `todo!` ve `unimplemented!` kullanımını Clippy `deny` ile
  reddeder. `SymbolId`nin yapay 32-bit kapasite assertion'ı da kalktı. İki
  yeni regresyonla toplam 429 test yeşildir; B-014/V1-P0-19 kapandı.
- **Lexer/parser sürekli fuzz hattı** (K-110, ADR-022): geçerli UTF-8 kabul
  eden libFuzzer hedefi lexer çıktısını hem normal hem hata-kurtarmalı parser'a
  verir. Sekiz Unicode/girinti/sayı/virgül/blok tohumu ile Zee mutation
  sözlüğü, sabit nightly+cargo-fuzz gece işinde cache'li korpusu büyütür;
  crash girdisi artifact olarak korunur. Üç stable regresyon korpusu, 4.096
  deterministik UTF-8 bileşimini ve 64 KiB uçları her ana testte oynatır.
  İlk smoke 1.048.287 girdiyi crashesiz tamamladı; toplam 432 test yeşildir,
  B-015/V1-P0-20 kapandı.
- **Morfoloji property/fuzz sertleştirmesi** (K-111): `zee-tr-1` üret→çöz
  değişmezi 4.096 deterministik kökün bütün geçerli tek/iki katmanlarında;
  sessiz seçim yasağı üretilmiş 2.048 yüzeyin bütün adaylarıyla sınanır.
  `ğ/ö/ş/â/İ` NFC yazımları geçer, NFD ayrıştırmaları S029 verir. Ayrı
  libFuzzer hedefi byte girdiden geçerli Zee kökü üretir; gecelik matrix
  korpusu bağımsız büyütür. İlk smoke 527.966 girdiyi ihlalsiz tamamladı;
  üç yeni regresyonla toplam 435 test yeşildir, B-016/V1-P0-21 kapandı.
  K-105 HTTP timeout testindeki tek-read sahte sunucu yarışı da başlığın
  bütünüyle tüketilmesiyle giderildi; macOS RST kararsızlığı kaldırıldı.
- **AST/HIR invariant kapısı** (K-112, ADR-023): parser AST'sinde çözülmüş
  ad/semantic ID/checker işareti; bağlı programda eksik, yinelenen veya yetim
  HIR kaydı ve varyantla uyuşmayan semantic bağ debug/test faz çıkışında
  fail-closed C000 olur. Açık doğrulama API'si faz, yapısal yol, açıklama ve
  satır taşır. Kapı, özellik erişimi alan erişimine çevrilirken klonlanan alt
  düğümün eski HIR kaydını yetim bıraktığını buldu; dönüşüm artık kutuyu taşır
  ve beklenmeyen varyantta T016 döndürür. Üç integration ve iki HIR unit
  regresyonuyla toplam 440 test yeşildir; B-017/V1-P0-22 kapandı.
- **Parser hata kurtarma ve LSP tanı bütçesi** (K-113, ADR-024): kurtarmalı
  parser hatalı cümlede satır sonuna, varsa yalnız ona ait dengeli girinti
  gövdesinin sonuna senkronlanır. Sağlam kardeşler ebeveyn blokta kalır;
  fiziksel derinlik sonraki üst düzey tanıma sızmaz. Yapı alanı, `göre` kolu,
  eşzamanlı görev, `değilse` ve `yetişmezse` sonraki geçerli satırı korur.
  CLI/LSP parser+birim+checker tanıları kaynak sırasında ve belge başına en
  çok 20 kayıttır. Altı parser ve bir gerçek LSP regresyonuyla toplam 447 test
  yeşildir. Recovery yolunu da çalıştıran 31 saniyelik lexer/parser smoke'u
  977.014 mutation'ı çökmesiz tamamladı; B-021/V1-P0-23 kapandı.
- **Tanı kodu sürüm kimliği** (K-114, ADR-025): kaynak↔katalog kapısının
  yanına 145 etkin ve 3 ayrılmış kaydı kapsayan şema-1 fixture'ı geldi. Her
  kod aile uyumlu tekil semantik anahtar ve kanonik “Ne oldu” özeti taşır;
  aynı kodun sessiz anlam değişimi ya da emekli kodun yeniden kullanılması
  testi kırar. A004, C014 ve S032 açık mezar taşıdır. Bir fixture regresyonuyla
  toplam 448 test yeşildir; B-022/V1-P0-24 kapandı.
- **Deterministik IO izi ve replay** (K-115, RFC-0022, ADR-026): bütün 27
  `GirdiCikti` yöntemi işlem, argüman, sonuç ve tek küresel sırayla kanonik
  şema-1 izine alınabilir. `dil iz kaydet/oynat` gerçek koşuyu atomik kaydeder,
  yeniden oynatmada dosya/ağ/web/donanım etkilerini uygulamadan protokolü
  birebir doğrular ve artan olayı reddeder. Okuyucu 64 MiB/100.000 olay/4.096
  alan sınırını ve işlem-özel şemaları yürütmeden önce denetler. Parola ile PHC
  argümanları yalnız SHA-256 parmak izi taşır; izin geri kalanı özel veri kabul
  edilir. Beş çekirdek ve iki CLI regresyonuyla toplam 455 test yeşildir;
  B-027/V1-P0-25 kapandı.
- **Sürümlü deterministik IO profili** (K-116, RFC-0023, ADR-027): CLI ve
  playground'un kopya rastgele üreticileri tek `SurumluRastgele` sahibinde
  birleşti. `zee-io-1`; tohum karışımı+xorshift64* dizi snapshot'ını, yansız
  uçları-dahil eşlemeyi ve tam i64 aralığını bağlar. Hermetik takvim ile
  tekdüze saat ayrıldı; negatif/eski zaman geri gidemez, sanal bekleme takvimi
  oynatmaz. FIFO girdi/rastgele fallback'i ve sahte dosya/ağ/sensör çıktıları
  da conformance kapsamındadır. Beş regresyonla toplam 460 test yeşildir;
  B-028/V1-P0-26 kapandı.
- **Platformlar arası kanonik `.zep` ve saldırı korpusu** (K-117, RFC-0020,
  ADR-028): paket üreticisi dosya sistemi bileşenlerini Unicode 17.0 NFC'ye
  çevirir, normalizasyon çakışmasını reddeder; tüketici yalnız zaten kanonik
  yolu kabul eder. UTS #39 `/`, `\\`, `.`, `:` benzerleri, tam genişlikli
  ayraçlar ve görünmez bidi denetleyicileri fail-closed'dur. Türkçe Unicode
  dosya adlı sabit `.zep` fixture'ı Linux/macOS/Windows CI'da aynı testle,
  80 yol saldırısı ayrı kalıcı korpusla korunur. Üç yeni regresyonla toplam
  463 test yeşildir; B-030/B-031 ve V1-P1-08 kapandı.
- **Makine-okunur kanıt ve canlı depo sayıları** (K-118, ADR-010): 23 RFC,
  26 ADR ve 22 spec bölümü durum+test dosyaları+açık kapsam notuyla tek TSV'de
  birebir izlenir. Yeni/eksik/yinelenen belge veya olmayan test yolu tazelik
  testini kırar. `depo_sayilari` golden, Rust+doctest, tanı, RFC/ADR ve spec
  sayılarını gerçek dosyalardan üretir; README işaretli bloğundaki tek byte
  kayma CI hatasıdır. İki yeni regresyonla toplam 465 test yeşildir;
  B-043/B-044 kapandı.
- **Formatter parse-equivalence kapısı** (K-119, RFC-0021/ADR-002/spec-20):
  C011 artık `SatirSonu`, `Girinti`, `Cikinti` ve `DosyaSonu` dahil tam parser
  token dizisini korur. Sayısal 33 golden programın metin ve yorum içini
  bozmayan dağınık-boşluk varyantı biçimlenir; önce/sonra token izi eşit ve
  her iki kaynak parser tarafından kabul edilmiş olmalıdır. İki yeni
  regresyonla toplam 467 test yeşildir; B-042 kapandı.
- **Semantic LSP gezinme ve rename** (K-120, ADR-014/016): HIR artık her
  `SymbolId` için ilk tanım, yeniden atama ve okuma aralıklarını; işlem/yapı
  kullanımlarını `IslemId`/`YapiId` ile araçlara açar. Definition/rename aynı
  yazımlı ayrı kapsamları ayırır, çok kelimeli işlem adını bütün değiştirir ve
  morfolojiyi yalnız semantic hedef seçildikten sonra uygular. Hatalı veya
  A002 belirsiz belgede metin tahmini yapılmaz; dış birim tanımı için eksik
  tek-dosya rename'i üretilmez. Bir HIR ve yedi LSP regresyonuyla toplam 475
  test yeşildir; B-041 kapandı.
- **Sıra-bağımsız yerel çağrı çıkarımı** (K-121, RFC-0006/ADR-013): Yerel
  `<ad> al` işlemlerinin erişilebilir ana program ve test çağrıları önce kopya
  AST'de toplanır; parametre konumu başına kısıtlar değişmeli birleşimle nihai
  imzaya çevrilir. Asıl gövde, tanılar ve typed HIR yalnız bu imzayla üretilir.
  Dar↔geniş skaler, liste, çok parametre, iç içe çağrı grafiği, iç blok
  kurtarması ve iki yönlü T017 regresyonlarıyla toplam 481 test yeşildir;
  B-007 kapandı.
- **Immutable `zee-tr-1` uyumluluk kapısı** (K-122, RFC-0018/spec-13): Profil
  dökümü, 11 ham sınır yüzeyinin sıralı çözümleri ve 4.096 kökün yedi tek+altı
  iki katmanlı zincirindeki 53.248 üretim/çözüm vektörü kanonik SHA-256 kaydına
  bağlandı. Çalışan davranış değişirse test; yayımlanmış fixture değiştirilir,
  silinir veya yeniden adlandırılırsa Git-geçmişli CI koruğu kırılır.
  `dil morfoloji --uyumluluk` kaydı görünür kılar. Bir yeni regresyonla toplam
  482 test yeşildir; B-008 kapandı.
- **Derleyiciden bağımsız morfoloji conformance korpusu** (K-123,
  RFC-0018/spec-13): Kök `conformance/` alanındaki sürümlü JSON Schema ve
  `zee-tr-1` veri dosyası, Rust tür ya da fonksiyon adlarına bağlanmadan bütün
  ek tablosunu ve izinli iki katmanlı zincirleri yayımlar. 27 çözüm/karar
  vakası doğrudan eşleşme, A001/A002, sıralı bütün adaylar ve ses değişimlerini;
  21 üretim vakası bütün tek/iki katmanlı zincirlerle geçersiz zincirleri
  kapsar. Çalışan Rust motoru aynı veriyi tüketen regresyonda doğrulanır;
  JSON Schema ile korpus da Git-geçmişli immutable koruğa dahildir. Bir yeni
  regresyonla toplam 483 test yeşildir; B-009 kapandı.
- **Gözlenebilir eşzamanlılık uyumluluk profili** (K-124, RFC-0011/spec-14):
  `zee-esz-1`, scheduler'ın iç future/thread yapısını değil çıktı ve ortak IO
  sırasını, sanal süreyi, sonuç bağlarını, sonlanma kodunu ve iptal sonrası etki
  yokluğunu sürümler. JSON Schema altındaki 10 doğrudan Zee programı; ortam
  snapshot'ı, tembel başlangıç, eşit/farklı uyanış, çoklu tur, ortak dosya,
  iç görev ağacı, hata/son tarih/çıkış iptali ve atomik eylem rollback'ini
  taşır. Gelecekte çok çekirdek kullanımı yalnız aynı gözlemleri veren iç
  optimizasyon olabilir. Genel Git-tarih conformance koruğu yayımlanmış veriyi
  kilitler. Bir yeni regresyonla toplam 484 test yeşildir; B-011 kapandı.
- **Ondalık/binary float FFI sınırı** (K-125, ADR-029): RFC-0012'deki artık
  var olmayan `GerçekSayı↔double` eşlemesi kaldırıldı. Ondalık'ın C
  `float`/`double` ya da binary32/binary64'e örtük dönüşümü yasaktır;
  `Tur`/`Deger` envanteri yalnız onluk değeri taşır. Gelecekteki köprü hem
  deklarasyon hem çağrıda görünür `kayıplı` işareti, `Sonuç` ve normatif IEEE
  754 yuvarlama/özel-değer semantiği ister. FFI Faz 4/5 taslağı olarak kalır.
  Enum envanteri, eski yüzey reddi, exact `0,1+0,2=0,3` ve belge tazeliği için
  bir yeni regresyonla toplam 485 test yeşildir; B-012/V1-P0-28 kapandı.
- **Bütün AST ifadelerinde kesin kaynak aralığı** (K-126, ADR-030): Parser'ın
  her yaprak ve bileşik ifadesi sıfır olamayan `AstKaynakAraligi` taşıyan tek
  bir kaynak zarfındadır; alt düğümler kendi token bölgelerini ayrıca korur.
  Checker aralığı HIR'a birebir taşır ve eski `1:1` ifade tanısını gerçek
  düğüme yükseltir. Invariant spansiz/iç içe zarfı ve AST↔HIR uyuşmazlığını
  reddeder. Örtük çoğul kaynak döngü adının tokenına bağlanır; LSP aynı adlı
  argümanla işlem kuyruğunu kesin semantic aralıkta ayırır. Beş yeni
  regresyonla toplam 490 test yeşildir; B-050/V1-P0-18 tamamlandı.
- **Merkezî yetkinlik ve güvenli outbound** (K-127, RFC-0024/ADR-031/spec-23):
  `proje.dil` sekiz kararlı dış dünya yetkinliğini ve exact şema+host+port
  hedeflerini bildirir. Paket bu kümeyi aşamaz (P015); checker ana/test ve
  çağrılmayan işlem gövdelerini tarar, eksik izin veya sabit hedefi kesin
  aralıkta T054 yapar. Genel `PolitikaliIo` ile gerçek adaptör aynı kararı
  runtime'da yeniden uygular; proje dosyası `..`/mutlak/canonical symlink
  kaçışına kapalıdır. Elle yazılmış düz HTTP istemcisi exact `ureq 3.4.0` +
  rustls HTTPS'e taşındı: public ağ HTTPS, private/loopback ve düz HTTP ayrıca
  `yerel-ağ` onaylı; metadata/link-local ve IANA public olmayan özel-kullanım/
  geçiş önekleri her zaman kapalıdır. DNS sonrası bütün adresler denetlenir;
  redirect/proxy kapalı, 30 saniye ve 64 KiB+
  8 MiB zarfı korunur. On bir yeni net regresyonla toplam 501 test, 147 etkin +
  3 ayrılmış tanı ve 76 numaralı belge yeşildir; B-023/B-049/V1-P0-29 kapandı.

- **Atomik replace metadata koruması** (K-128, ADR-032, RFC-0016/spec-08):
  Var olan hedef artık yalnız normal dosyaysa değiştirilir; symlink sessizce
  normal dosyaya dönüştürülmez. Linux/macOS mode+uid+gid'yi korur. Linux
  görünür xattr namespace'lerini 64 KiB ad/değer ve 1 MiB toplam bütçeyle;
  macOS ACL+xattr/resource fork'u `fcopyfile` ile taşır. Windows mevcut hedefi
  DACL/security resource/named stream birleştiren, hata yoksaymayan
  `ReplaceFileW` ve aynı klasör kurtarma yedeğiyle değiştirir. Desteksiz veya
  taşınamayan metadata committen önce fail-closed olur. Beş yeni platform
  regresyonuyla envanter 506 test ve 77 numaralı belgeye çıktı; B-048 ve
  V1-P0-30 kapandı.

- **Merkezî kaynak güvenlik profili** (K-129, RFC-0025, ADR-033, spec-24):
  Tek değişmez `KaynakSinirlari`; 8 MiB kaynak/1 milyon token, 4.096 dosya ve
  128 MiB proje toplamı, mevcut C019/500 çağrı derinliği, 10 milyon adım,
  1 milyon koleksiyon öğesi, 1.024 görev, 16 MiB/100 bin çıktı ve 16 MiB
  bounded dosya okumasını bağlar. LSP 256 açık belge/128 MiB toplam metin ve
  8 MiB outbound zarf taşır; red eski belgeyi değiştirmez. S045/C023 ile iki
  yeni kararlı tanı ve yedi regresyon eklendi; envanter 513 test, 149 etkin +
  3 ayrılmış tanı ve 80 numaralı belgeye çıktı. B-025 toplam heap/metin ve
  bağlantı muhasebesi için kısmen açık kalır.

- **Bütçeli değer/metin ve bağlantı zarfı** (K-130, RFC-0025/ADR-033
  revizyonu): Tek metin, keyfî hassasiyetli sayı/para basımı dahil, 16 MiB'ta
  tahsis öncesi bütçelenir; ortam/koleksiyon yazımları ile görev ortamı
  klonları çalışma/istek başına iade edilmeyen yaklaşık
  64 MiB saklama zarfı tüketir. Süreç genelindeki inbound+outbound bağlantılar
  64 RAII izniyle sınırlıdır. Metin/değer aşımı append-only C024'tür. Dört
  yeni regresyonla envanter 517 test, 150 etkin + 3 ayrılmış tanı ve 80
  numaralı belgedir. B-025 yalnız eski domain sabitlerinin göçü için açıktır.

- **Kaynak limitlerinde tek sayısal sahip** (K-131, RFC-0025/ADR-033
  revizyonu): HTTP/ağ, web oturumu, IO izi, LSP, paket/registry, tanı,
  atomik metadata ve dosya kilit sürelerinin mevcut değerleri davranış
  değiştirmeden `KaynakSinirlari` domain görünümlerine taşındı. Geriye uyumlu
  yerel sabit adları yalnız ortak profile bağlı alias'tır. Bounded reader ve
  profil tanımları ayrı mimari bütçeli modüllere ayrıldı; merkezi sahiplik
  regresyonuyla envanter 518 teste çıktı. B-025, LSP outbound JSON'unu tam
  yanıt kurulmadan 8 MiB'ta kesen allocation-order dilimi için kısmen açıktır.
- **Tahsis sırasında bütçeli LSP JSON'u** (K-132, RFC-0025/ADR-033/spec-24):
  Initialize, diagnostics, completion, hover, definition, rename ve hata
  gövdeleri tek `SinirliJson` yazıcısına taşındı. Her append merkezî 8 MiB
  sınırından önce denetlenir; rename/diagnostics bütçesiz `Vec<String>` ve
  `join` kurmaz. Taşma `-32001` veya bounded `window/logMessage` olur; kısmi
  gövde yayımlanmaz. Üç yeni regresyonla envanter 521 teste çıktı ve
  B-025/V1-P0-31 kapandı.

- **Yan etki öncesi deadline kapıları** (K-133, RFC-0011/spec-09/spec-14):
  Çıktı/girdi, dosya, sunucu, web yanıtı/yönlendirmesi, çerez/oturum,
  eyleyici, CSRF, parola, rastgelelik ve `eylem` başlangıcı gerçek çağrının
  hemen önünde deadline'ı yeniden denetler. Görev HTTP'si scheduler'a sıra
  verdikten sonra kalan süreyi tazeler; süre dolmuşsa adaptör hiç çağrılmaz.
  İki sanal saat regresyonuyla envanter 523 teste çıktı. B-026 kısmen
  kapandı; web istek yanıtı+oturum transaction'ı K-134'e kaldı.

- **Transaction'lı web istek yaşam döngüsü**
  (K-134, RFC-0011/RFC-0017/spec-09/spec-11/spec-12): Rota seçimi ve yürütmesi
  180 satır bütçeli `yorumlayici/web_istek.rs` sahibine ayrıldı. İlk HTTP
  yanıtı/yönlendirmesi session ve çerez mutation'ıyla birlikte tamponlanır;
  yalnız başarılı gövde ve eksiksiz socket yazımında commit olur. Timeout,
  runtime/socket hatası veya yanıtsız rota hepsini geri alır. Dört yeni
  hermetik+gerçek TCP regresyonuyla envanter 527 teste çıktı; B-026 kapandı.

- **Registry taşıma, kalıcı durum ve doğrulanmış cache**
  (K-135, RFC-0020/ADR-006/spec-19): Yalnız HTTPS originli, redirect/proxy'siz
  ve DNS sonrası public-IP denetimli statik ayna; 64 ardışık root rotasyonu;
  sürümlü timestamp/snapshot/targets yolları ve hedef başına byte limiti
  çalışır. Tam metadata+yayınevi zinciri geçmeden cache/durum yayımlanmaz.
  Nesneler salt-okunur SHA-256 adreslidir; sürüm+özet durumu atomik CAS ile
  yarışan rollback'e kapalıdır. Offline kip son kabul zincirini duvar saati ve
  ağ olmadan yeniden doğrular; miss/bozulma P016'dır. Altı yeni regresyonla
  envanter 533 teste çıktı. B-029 exact manifest/kilit/CLI için kısmen açıktır.

- **Exact registry manifesti, kilit v3 ve CLI**
  (K-136, RFC-0009/RFC-0020/ADR-006/spec-07/spec-19): `proje.dil` HTTPS
  registry originini, ağ dışı root sürüm+SHA-256 pinini ve yalnız exact
  `ad@X.Y.Z` bağımlılıkları taşır. `dil ekle`, `dil kilitle
  [--çevrimdışı]` ve ağsız varsayılan `dil paketler [--yenile]` aynı
  doğrulanmış proje-local cache'i kullanır; normal derleme/LSP sessiz ağ
  açmaz. `proje.kilit` v3 root/rol/yayıncı/dört hedef/yanked/kritik kimliğini
  ve insan gerekçeli baypası sabitler. `.zep` kaynakları görünmez geçicide
  exact doğrulanıp atomik ve salt-okunur kurulur. Kritik kabul anahtarı güncel
  sıralı duyuru kümesini de taşır; yeni duyuru eski gerekçeyi kullanamaz.
  P017 ile altı regresyon eklenerek envanter 539 test, 152 etkin + 3 ayrılmış tanıya çıktı;
  B-029/V1-P1-07 kapandı.

- **Proje modeli** (K-076): geçerli zee sözdizimli `proje.dil` (`proje`,
  `sürüm`, `giriş`); `dil çalıştır/denetle/dene <klasör>`; `dil yeni`
  bildirimi hazır üretir. P001–P004 Türkçe proje tanıları. Doğrudan dosya
  kullanımı geriye uyumludur. Göreli dosya IO'su giriş klasörüne sabitlendi;
  `--güvenli` bayrağının program argümanına sızması da kapandı.
- **Yerel paketler ve kilit** (K-078): `yerel_bağımlılıklar` başka zee
  projelerini bağlar; `X paketini kullan` yalnız doğrudan bağımlılığı alır.
  Kaynak kökeni paket içi birimlerde korunur. `dil kilitle` geçişli grafiği
  göreli yol, sürüm, kenar ve SHA-256 `.dil` özetiyle deterministik
  `proje.kilit`e sabitler; eksik/bayat kilit P008'dir. Döngü, yinelenen ad,
  proje dışına çıkan sembolik bağ ve geçişli bağımlılığa gizli erişim testli.
- **Güvenli paket ekleme** (K-079): `dil ekle <yerel-yol> [proje]` yolu
  proje köküne göre taşınabilir biçime çevirir; yorumları koruyup listeyi
  sıralar. Yeni bağımlılık grafiği yazmadan önce bütünüyle doğrulanır. Başarılı
  işlem bildirimi ve kilidi birlikte günceller; yinelenen gerçek kök ikinci kez
  eklenmez, öz-bağımlılık P007'dir.
- **Paket grafiği ve güvenli kaldırma** (K-080): `dil paketler [proje]`
  doğrudan/geçişli paketleri sürüm, göreli yol ve kilit özetiyle listeler.
  `dil çıkar <paket> [proje]` yalnız doğrudan paketi kaldırır; kaynakta kalan
  `X paketini kullan` P010 ile işlemi değişiklik yapmadan durdurur. Başarıda
  bildirim/kilit birlikte güncellenir ve yazma hatasında geri alınır.
- **Proje çapında biçimleme** (K-077): `dil biçimle <klasör>` bütün `.dil`
  kaynaklarını deterministik yol sırasında biçimler; önce tamamını doğrular,
  tek hata varsa hiçbir dosyaya dokunmaz.

- **Gezmede yazma yansır** (K-074): `her kutu için / kutunun adedi ...`
  listeye geri yazılır — kopya tuzağı kapandı (TANIMLI).
- **Türk para yazımı** (K-075): `binlikli kuruşlusu` → "1.234.567,89".
- **Çerez silme** (K-073): `"oturum" çerezini sil` — Max-Age=0; panel
  çıkışı gerçek silmede.

## v0.6.0 — 1 Eylül 2026

**Tema: tip sistemi olgunlaştı, dil ayrıntıda medenileşti.**

- **Aralık iki yönde** (K-068): `5 ten 1 e kadar` geri sayar (sessiz
  boş dönüş tuzağı kapandı). roket.dil projesi.
- **Çıkış kodu** (K-069, K-024 kapanışı): `programı 1 ile bitir` —
  süreç kodu kabuğa gider (0–255, C020); CLI otomasyon kapısı.
- **Yeniden adlandırma** (K-072): dillsp + VS Code F2 — morfoloji
  farkındalıklı: ekler yeni köke Türkçe uyumla giydirilir (sayacı→puanı,
  renk→rengi). Metin/yorum dokunulmaz; tek katman ek kapsamı.
- **Doğrulamalar** (K-071): `içermeli`, genel `olmamalı`, çıplak `boş`
  atomu — test kültürü zenginleşti. Ölçüm: döngü izlemesi kapandı
  (v0.6.0 satırı; v0.2 tabanının altında).
- **Kitaplık** (K-070): `toplamını/ortalamasını hesapla` artık DAİMA
  Ondalık döner (deneysel-kırıcı; TamSayı listeleri genişlemeyle girer).
  Yeni gömülü birim: `sozluk_araclari` (`en çok geçeni bul`).

- **Para biçimi** (K-065): `tutarın kuruşlusu` — daima iki hane.
- **Sayısal genişleme çağrıda + imza terfisi** (K-067): Liste<TamSayı> →
  Liste<Ondalık> parametre; dar imza geniş argümanla terfi eder. Sözlük
  değerleri Ondalık olabilir. Kitaplığa medyan girdi.
- **Evrensel metin hali** (K-066): `değerin metni`; `dil belge` artık
  işlem açıklamalarını basar; playground'da Envanter vitrini.
- JSON okuma hoşgörüsü (K-063), alan-özellik gölgelemesi (K-064),
  envanter projesi; işlemden liste dönüşü testle sabitlendi; biçim
  hijyeni projeler+kitaplığa genişledi.

## v0.5.0 — 1 Eylül 2026

**Tema: kayıtlar ve koleksiyonlar — dil, veri işlerinin dili oldu.**

- **JSON okuma hoşgörüsü** (K-063): sayı/bool/null değerler Metin gelir
  (nokta → virgül); yalnız iç içe yapı C016.
- **Alan, özelliği gölgeler** (K-064): `ürünün adedi` alan okur —
  denetleyici yeniden yazımı. Envanter projesi eklendi (stok defteri).

- **Sıralama** (K-056): `sıralanmışı` — Metinler Türk alfabesi sırasıyla
  (ç, ğ, ı, i, ö, ş, ü yerli yerinde; TANIMLI); `tersi`.
- **Tarih farkı** (K-057): `X ile Y arasındaki günler` (işaretli).
- **Liste üyeliği + CSV yazma** (K-058): `sayılarda 5 varsa`;
  `tablonun csv metni`.
- **Morfoloji: iki katmanlı ek** (K-061): `kitabın fiyatıyla artır`
  (iyelik+araç zinciri) çözülür.
- **CSV hücreleri Metin** (K-062): isimli sütunlar birinci sınıf;
  sayı `değerin sayısı` ile bilinçli çevrilir. Golden 19 revize.
- **Yapı listeleri** (K-060): kayıt tabloları — `boş liste`ye yapı ekle,
  gez, alan oku; `json metni` nesne listesi üretir. (T027 belgesi düzeltildi:
  Ondalık alan zaten vardı.)
- **Silme** (K-059): `sayılardan 5 i sil` / `defterden "elma" yı sil` —
  yoksa sessiz (idempotent). Girişli panel demosuna silme akışı eklendi
  (dosya-satırı silme saf zee: süz + birleştir + yaz).

## v0.4.0 — 1 Eylül 2026

**Tema: dil, kurucunun gerçek projelerine hazırlanıyor — web ve metin.**

- **Metin dalgası** (K-053): `parçaları`, `birleşmişi`, `yerine ...
  değişmişi`, `kırpılmışı`, `harfleri`, `ile başlıyorsa/bitiyorsa` —
  harf düzeyine ilk iniş. Gömülü `metin_araclari` birimi (saf zee).
- **JSON yazma** (K-054): `değerin json metni` — sıra-korumalı,
  deterministik serileştirme.
- **Deneysel web uygulaması katmanı** (K-050..K-052, K-055): HTML servis, örtük
  `istek` sözlüğü (sorgu + POST form), `adresine yönlendir` (303),
  `html güvenlisi` (XSS), örtük `çerezler` + `çerezine yaz` (oturum),
  önekli rotalar. Kanıtlar: mini-site, panel-not-defteri, girisli-panel
  (parola+oturum mantığı SAF ZEE) — üçü de localhost'ta canlı + hermetik.
- **Sınır (değişmedi):** parola düz metin, HTTPS yok — internete açık
  üretim Faz 5 güvenlik dalgasını bekler.

### v0.3.0 sonrası küçükler

- **Ünsüz ikizleşmesi morfolojisi** (K-049): `üssü`, `affı`, `zammı`,
  `reddi` (sertleşmeyle) çözülür; matematik biriminin doğal `üssü al`
  parametresi geri geldi.
- **`dil belge <birim>`**: işlem başlıkları + test sayısı (RFC-0014 §8.2).
- Playground'a "Kitaplık (obeb)" örneği eklendi.
- **Deneysel zee web sitesi** (K-050): sunucu HTML'i text/html olarak servis
  eder; projeler/mini-site.dil — rotalar + stil + gömülü kitaplık hesabı.
- **Web uygulaması dalgası** (K-051): örtük `istek` sözlüğü (sorgu + POST
  form, UTF-8 yüzde çözümü), `adresine yönlendir` (303, S041),
  `html güvenlisi` (XSS kaçışlaması). Kanıt: panel-not-defteri projesi —
  formlu, dosyada saklayan, gizli yollu eğitim paneli (localhost'ta canlı +
  hermetik tam-döngü testi). Sınır: HTTPS/hash Faz 5'te.
- **Oturum kapısı** (K-052): örtük `çerezler` sözlüğü + `çerezine yaz`
  (Set-Cookie, HttpOnly). Oturum mantığı saf zee'de: girisli-panel projesi
  (parola → dosyada oturum kimliği → korumalı rotalar; sahte çerez reddi
  testli). Parola düz metin — hash Faz 5.

## v0.3.0 — 1 Eylül 2026

**Tema: dil günlük Türkçeye yaklaştı; standart kitaplığın tohumu atıldı.**

- **Gömülü standart kitaplık** (RFC-0014 taslak + çalışan prototip, K-048):
  `matematik` (mutlak, üs, tam karekök, obeb-Öklit, okek) ve
  `liste_araclari` (toplam, uçlar, Ondalık ortalama) — zee'yle yazıldı,
  ikiliye gömülü, playground dahil her yerde kurulumsuz; kendi test
  blokları CI'da. Çözüm: yerel klasör → gömülü.
- **Öğretmen rehberi** (docs/ogretmen-rehberi.md): internetsiz sınıf
  kurulumu, 10 oturumluk ders sırası, hata kültürü.
- Proje kitaplığı 10 projeye çıktı (kelime sayacı, gün sayar).

- **Mantıksal ad tek başına koşul** (K-044): `hazır ise` / `hazır değilse`.
- **Boş koleksiyon tür çıkarımı** (K-045): `boş liste`/`boş sözlük` ilk
  eklemeyle türlenir; metin listeleri ve metin sözlükleri artık kurulabilir.
  KIRICI: hiç eklenmemiş boş listenin `ilki` artık derleme hatası (T014;
  eskiden çalışma anında C007).
- **Çocuk modu** (K-047): `dil çalıştır --güvenli` — ağ/sunucu kapalı,
  dosyalar çalışma klasörüyle sınırlı; hata `dene` ile yönetilebilir.
- **Kalan işlemi** (K-046): `17 nin 5 e bölümünden kalanı` — okul kuralı,
  kalan hiç negatif olmaz.
- **Zamir n'si morfolojisi** (K-041): `bilgisayarın_zarından` çözülür.
- **VS Code**: elle yazılmış LSP istemcisi (npm'siz) — canlı tanılar,
  hover, tanıma git, tamamlama editörde. ADR-008 (self-hosting aşamaları) kabul.
- dillsp: hover (Türkçe açıklama) + tanıma git. Performans arşivi
  (docs/olcumler.md) ve `olcum` koşucusu. Proje kitaplığı 8 projeye çıktı.

## v0.2.0 — 31 Ağustos 2026

**Tema: dil derinleşti, derleyici tarayıcıya taşındı.**

### Yeni dil yüzeyi

- **Özyineleme** (K-035): işlem kendini ve karşılıklı olarak birbirini
  çağırabilir; tanım sırası serbest (adlar ön-taranır). Kural: temel durum
  özyinelemeli çağrıdan ÖNCE gelir (T035) — tür çıkarımı "o ana dek görülen
  dönüşler" üzerinden yapılır, dil iyi alışkanlığı kendisi öğretir.
  Çalışma zamanı derinlik sınırı 500 (C019, K-040: her platformda aynı).
- **Blok kapsamı** (K-034, RFC-0004 kapanışı): gövdede doğan ad gövdeyle
  ölür (döngü değişkeni dahil); dıştaki ada atama kalıcıdır; gölgeleme
  yapısal olarak yoktur.
- **Akış-duyarlı daraltma** (K-037, T036): `varsa` / `başarılıysa` /
  `başarısızsa` dalları ve `değilse` tersinmeleri içinde `değeri` / `hatası`
  erişimi statik güvenli; dal dışında korumasız erişim artık DERLEME
  hatasıdır. "Boş değeri açmak" hatası çalışma zamanından derleme zamanına
  taşındı; C008/C009 iç savunmaya dönüştü.
- **Metin kaçışları** (K-036): `\"`, `\\`, `\n`; bilinmeyen kaçış S040.
- **Negatif sayı sabitleri** (K-036): `-3`, `-3,14` — işaret rakama bitişik.
- **Çok-tokenli çağrı argümanları** (K-038): `tabanın tam kısmı için yuvarla`
  — her `ve`/`ile` dilimi tam bir ifade bölgesi.

### Playground (K-039)

- `playground/zee-playground.html`: derleyicinin tamamı WebAssembly olarak
  tek HTML dosyasında. Çift tıkla açılır; kurulum ve internet gerekmez.
- 7 hazır örnek, girdiler alanı, görünür tohum — **aynı tohum + aynı girdi
  = her zaman aynı çıktı** (determinizm tarayıcıda da geçerli).
- ADR-001 korunur: wasm-bindgen yok; elle yazılmış C-ABI köprüsü
  (`compiler/src/wasm_api.rs`), çekirdek doğal derlemede de testli.
- Yeniden üretim: `playground/olustur.sh`.

### Tanılar

- Yeni: T035 (temel durum önce), T036 (korumasız değer/hata erişimi),
  S040 (bilinmeyen kaçış), C019 (çağrı derinliği).
- Yeniden konumlanan: C008/C009 artık iç savunma (derlemede T036 yakalar).

### Mühendislik

- CI üç platformda yeşil; wasm hedefi de derlenir (ubuntu).
- CLI, işi 32 MB yığınlı iş parçacığında koşar (Windows ana iş parçacığı
  1 MB'dır); sınıra dokunan test kendi yığınını getirir (K-040).
- Toplam 154 hermetik test (`cargo test --no-fail-fast`), clippy temiz,
  uyarısız derleme.

### Kırıcı değişiklikler

- Korumasız `değeri`/`hatası` erişimi olan programlar artık derlenmez (T036).
  Düzeltme tanının önerisindedir: erişimi `varsa`/`başarılıysa`/`başarısızsa`
  dalına al. 32 golden programın hiçbiri etkilenmedi.
- Gövde içinde tanımlanan ada gövde dışından erişim artık A001 (K-034).
  Korpusta buna dayanan program yoktu.

## v0.1.0 — 31 Ağustos 2026

İlk sürüm — proje doğdu. 32/32 golden korpus; v0.1 kabul kriterlerinin
8/8'i (master plan bölüm 33). Tam yüzey listesi: [README](../README.md).
Etiketler: `baslangic` (ilk taahhüt), `dogum` (merhaba.dil çalıştı),
`v0.1.0` (kabul kriterleri kapandı).

K-195 geliştirme: Windows MSVC dağıtım tarifi statik CRT/explicit target taşır; üç platform tatbikatı çift-klon üreticisini kullanır. Provenance CRT/bayrak alanları eklendi; Windows dinamik CRT importu ret kapısı vardır. Exact `c7e3d04` üç platform CI kanıtı geçti; yayın etiketi üretilmedi.

K-195 ilk Windows çift-klon koşusu eş-hash kapısında kaldı; artefakt
üretilmedi. Yeni MSVC linker tarifi `/Brepro` ve provenance
`targetRustflags` alanını birlikte günceller; exact tekrar kanıtı gerekir.
Eş olmayan byte'lar ayrı `yayinlanamaz-surum-farki` tanısında saklanır,
sürüm/kurulum kümesine girmez. Benchmark tarihçesine yeni satır yazılmadı.

K-195 kapanma: `c7e3d04adf27905ade8b622d24bd76762123d2dc` kaynağındaki
[38244549913 tatbikatı](https://github.com/itwise-bilisim/zee/actions/runs/38244549913)
üç platformda geçti. Windows iki temiz klonda eş hash, dinamik CRT importu
yok ve kur/program/LSP/kaldır başarılı. İndirilen byte'lar metadata ile
ayrıca eşleştirildi; ham rapor/provenance kalıcı fixture'dadır. İlk Windows
reddi korunur. K-192 temiz Windows/macOS, minimum OS/linker-SDK kanıtı
ayrı açık kalır; native Zee backend ve self-hosting tamamlanmış değildir.

K-196 linker/SDK kayıt dilimi exact üç platform kanıtıyla kapandı: release iki klonun gerçek link
komutunu, program path/hash/version ve SDK referanslarını provenance'a
bağlar. Unix `-v`, MSVC `/VERBOSE:LIB` tarifte görünür; Python 3 build kanıtı
aracıdır, runtime bağımlılığı değildir. Eksik kanıt yayımlanmadan reddedilir.
Exact b153064 kaynağı/run 38252942210 üç platformda geçti; minimum OS/temiz Windows-macOS ve
bütün native C araçlarının kaydı tamamlanmış gösterilmez. Performans
baseline'ı/tarihçe satırı üretilmedi; ADR-045/049 kuralları korunur.

K-196 release kanıtı MSVC `/OUT:` anahtarıyla düz/hashli exe adlarını ve
Windows Kits verbose library yollarını tanır. Eksik linker kanıtı hâlâ
release kümesini reddeder; tanı artefaktında iki build logu korunur. Linux
tatbikatında yalnız o koşuya özel GitHub koşucusu seçilebilir. Dil davranışı
ve kurulum dosya kümesi değişmez; üç platform kapanma kanıtı açık kalır.

Windows MSVC build kimliğinde help exit davranışı yerine executable
FileVersion/ProductVersion metadata'sı kullanılır; `version_source`
provenance kaydında görünür. Başarısız/boş metadata release'i reddeder.

K-197 temiz Windows runtime provası exact kanıtıyla kapandı; exact release kaynağı/hash, immutable Nano Server image,
ContainerUser/network none/read-only bind ile program ve LSP denenir.
Nano Server program/LSP sonucu geçti; masaüstü/minimum OS ve bütün IO
desteği ayrı açıktır. Release/prova kaynak SHA'ları ayrı kaydedilir.
ADR-068/GB-034; spec/RFC/tanı/API/production Rust katmanları ve benchmark
tarihçesi değişmez.

K-197 kapanma: release c7e3d04, prova 27c0fa9/run 38253777857/job
114826470199. [Ham rapor](temiz-windows-runtime-kaniti-2026-10-12.json),
[defter](temiz-windows-platform-kaniti-v1.tsv); Nano Server 10.0.26100.33438
program/LSP geçti. Kurulum/kaldırma, desktop/minimum OS ve bütün IO ayrı.

K-196 kapanma: b153064/run 38252942210; iki klon eş hash ve gerçek
kur/program/LSP/kaldır üç platformda geçti. GCC 13.3.0, Apple Clang 21.0.0
+ SDK 26.5, MSVC FileVersion 14.51.36260.0 + SDK 10.0.26100.0 dört
link kaydında doğrulandı. Ham runtime/provenance ve artifact hash'leri
[platform defterinde](derleme-platform-kaniti-v1.tsv) korunur. K-192 minimum OS, bütün native C
araçları ve fiziksel temiz makine kanıtı ayrıca açıktır.

K-198 temiz Server Core kurulum/kaldırma dilimi exact kanıtla kapandı; exact source/
byte/script hash'leri, Türkçe boşluklu ContainerUser dizini, manifest
path/hash/source, program/LSP ve kaldırma yokluk kanıtı istenir. Gerçek
Windows koşusu geçti; desktop/minimum OS/fiziksel macOS ve bütün IO
tamamlanmış gösterilmez. ADR-068/GB-035. Spec/RFC/tanı/API, production
Rust katmanları, release dosya kümesi ve benchmark tarihçesi değişmez.

K-198 kapanma: release b153064/prova 33176cf/run 38255788989/job
114833388588; stock Server Core 10.0.26100.33438'de Rust kurulmadan
Türkçe dizine kur/program/LSP/kaldır geçti. [Ham rapor](temiz-windows-kurulum-kaniti-2026-10-12.json)
ve [kimlik/hash defteri](temiz-windows-platform-kaniti-v1.tsv) kalıcıdır.
ContainerUser/network none/read-only bind; host kernel, fiziksel/desktop
minimum OS ve tüm IO kapsamı ayrı açıktır.

K-199 Cargo compiler/wrapper pin düzeltmesi üç platform exact kanıtıyla kapandı: helper
exact kurulu rustc executable path/version'ını Cargo RUSTC'ye bağlar ve
wrapper/config seçimini boş değerle kapatır. Önce helper 1.93.1 bildirip
Cargo RUSTC ile 1.97.1 çağırabiliyordu. Resmî release/offline girişi
ambient compiler/wrapper seçimini korumaz; custom geliştirme wrapper
kendi Cargo çağrısıyla kullanılır. Gerçek küçük Cargo poisoned ENV/config
provası exact 22ea2b8/run 38259341995 üç platformda geçti. ADR-068/GB-036; dil spec/RFC/
tanı/API/production Rust kenarı ve release dosya kümesi değişmez.

K-199 kapanma: 22ea2b8/run 38259341995; Linux/macOS/Windows gerçek
Cargo/build-script/ikili Rust 1.93.1, empty wrappers ve temiz source
kanıtı [kalıcı defterde](rustc-pin-platform-kaniti-v1.tsv). Helper/prova/pin
hash'leri kaynakla ayrıca eşleşir. Tam Zee release ve bütün Cargo
profil/linker/native C ayarları bu dar probe'un kapsamı değildir.

K-200 macOS C/Rust deployment uyum kapısı kapalı: final minos 11.0
bildiren eski derlemede C nesnesi 27.0 idi. Resmî release exact Rust
default deployment'ını C/Rust'a ortak export eder; iki klon native .o
başlık/hash kanıtı provenance'a girer. Yerel sonrası 24/24 nesne 11.0
ve program/LSP geçti. Exact kaynak `44fba404c79762aff1698a07978c79ea069a59df`,
run 38263264133 üç platform iki-klon/kurulum/program/LSP/kaldırma kapısını
geçti; macOS a/b klonlarında ayrı ayrı 24 nesnenin minos değeri 11.0. Gerçek
macOS 11/minimum OS/tüm native alt süreç desteği değildir. ADR-068/GB-037;
spec/RFC/tanı/API/production Rust katmanı ve release dosya kümesi değişmez.

K-201 native C/archiver driver çağrı kaydı hazırdır; iki klonda compile ve
archive zorunlu, paket/argüman/executable path/hash ve sürüm sorgusu sonucu
provenance'a eklenir. Exact kaynak 25b6882/run 38349601833 üç platform release kanıtı geçti.
pkg-config/vcpkg, driver iç alt süreçleri ve minimum OS K-192'de ayrıca
açık kalır. Ayrıntı [derleme rehberinde](derleme-ve-bagimliliklar.md).

K-202: ana GitHub/CI deposu `merbay-erp/zee`; `itwise-bilisim/zee` yalnız
ayna ve repository Actions kapalıdır. SBOM/provenance yeni ana kaynak
URI kullanır; eski CI/fixture kanıtları tarihsel bağlantılarıyla korunur.

K-204: sır taramasının dört kamuya açık belge yanlış pozitifine yalnız
immutable fingerprint/kaynak satırı hash doğrulaması uygulandı; yeni
anahtar exit 1/REDACTED testi, default kurallar ve iki tarama korunur.
K-202 beyan sonrası türetilen kanıt özeti yenilendi.

K-207 başarılı ad çözümlemesini hash üyeliğine taşıdı; full ad listesi
yalnız A001 önerisinde üretilir, A002 ve exact-ad kararı değişmez. Oracle
26.624 karar, golden/morfoloji/LSP ve mimari kapıları geçti; exact önce/
sonra performans kanıtı açık. K-206 PG sonucu bellek deneyi GB-041 olarak
orta/açık kaydedildi; 16 MiB sonuç bütün driver belleği tavanı değildir.

K-207 ilk exact ölçümü: 29be85b → 94de420 aynı M4/Rust/release/25+2
koşulunda 20k p95 22.169,906 → 2.961,638 ms. Raw sample ve tarihçe
[ölçümlerde](olcumler.md). Kapsam maliyeti ve 1 sn hedefi açık;
incremental parser/cache veya genel platform SLA eklenmedi.


K-207 kapsam dilimi kapalı: temiz exact 9e80b69 release/25 örnek/2 ısınma
20k LSP p95 165,137 ms; aynı exact kaynakta 1800 sn/264.343 döngü soak
geçti. Derleyici RSS medyanı 8.144→8.160 KiB, LSP 19.088→17.568 KiB.
Ham örnek/çevre ve tarihçe [ölçümlerde](olcumler.md). 728/728 faz,
26.624 ad oracle ve 4.096 kapsam işlemi oracle karşılaştırması geçti.
K-207 KISMEN; full didChange ve incremental/cache kapasitesi açıktır.

K-206 native okuma akış toplamasına geçti: sonuç sınırından sonra çıktı
biriktirilmez; borrowed metin kopyalanmadan denetlenir. Hata önceliği ve
mevcut read retry/transaction/API korunur. Tek wire satırı/driver RSS
bütçesi GB-041'de açık; exact native kabul ve RSS kaydı aşağıdaki kapanışta saklıdır.

K-206 akış toplama dilimi kapalı: source f2ba654, temiz beyan/ölçüm
6463a7f; 731/731 faz ve gerçek native oracle/transaction kabulü geçti.
64×1 MiB sonuçta native süreç RSS 128,77→23,81 MiB, tek 64 MiB satırda
136,77→72,72 MiB; üç vakada sonraki sorgu başarılıdır. Bir örnek/sıfır
ısınma gözlemi, genel RSS garantisi değildir. [Ham kanıt](incelemeler/k206/README.md).
**K-206 KISMEN:** tek wire satırı/driver sınırı ve GB-041 açık kalır.

K-206 tedarik hazırlığı: upstream protokol Rust src byte'ları değişmeden
local patch kimliği compiler/fuzz lock ve doğrulanmış SPDX kaynak/tree
hash ile kaydedildi. GB-042 kapalı; GB-041/erken wire sınırı açık.
Offline vendor ve iki-clone release kabulü kapanış öncesi beklenir.

K-206 path patch iki-clone kabulünde reddedildi; yalnız kaynak kimliği
exact ana repo Git SHA pinine çevrildi. Protokol davranışı değişmedi.
SBOM yerel + Git blob tree hash eşitliği ve exact lock source ister;
v2 kayıt etkin, v1 tarihsel. Yeniden offline/release kanıtı beklenir.

K-206 dependency kimliği dilimi kapalı: source 9dae3e6, temiz beyan/
ikili üretimi ab8e97e. 735/735 faz, git pin/blob/tree SBOM, offline vendor
163 paket ve iki temiz klonda eş dil/dillsp SHA-256 geçti. Yerel macOS
kabulüdür; [ham kanıt](incelemeler/k206/source-identity/README.md).
Path kaynaklı başarısız iki-clone koşusu da saklanır. Protokol src byte'ları
upstream ile aynıdır; GB-041/32 MiB erken wire guard hâlâ açık kalır.

K-206 erken header kabul tasarımı: iki ayrı backend parser'ı kapsayan
typed 32 MiB aday yama bağımsız geçici crate'te 34/34 testle doğrulandı.
Aktif dependency pin değişmedi; üretimde mesaj sınırı etkin değildir.
Transaction durum temizliği, read retry engeli ve exact immutable kaynak
entegrasyonu açık; GB-041 kapatılmadı. [Kabul tasarımı](incelemeler/k206/frame-guard/README.md).

K-206 aday protokol kabulü clean c68af0f üzerinde kapandı: iki parser'da
34/34 bağımsız lib test, native 5 vaka, typed io cause ve kopan transaction
yazısının kalıcı olmaması doğrulandı. Ham JSON/log [kayıttadır](incelemeler/k206/frame-guard/README.md).
Bu yalnız adayın kabulüdür; aktif pin, Zee session/retry/RLS cleanup
entegrasyonu ve GB-041 açık kalır. Compiler fazı 735/735 geçti.

K-206 wire entegrasyonu: compiler/fuzz protokolü exact 1ffdb584 .2 kaynağına
pinlidir. İki parser'da 32 MiB length reddi, typed C025 verisi, read retry
engeli ve geçersiz transaction/request-local cleanup uygulanır. Gönderilmiş
COMMIT sonrası bağlantı kaybı C027 olarak korunur. Native loopback kabulü
okuma, büyük Notice üreten write, nested/RLS ve deferred COMMIT'i kapsar.
GB-041 bu kaynakta uygulandı; exact temiz kaynak, vendor/iki-clone ve
sürekli kapı kabulü henüz beklenir. Global RSS sözü verilmez.

K-206 wire security dilimi kapandı: source 7836b73, clean beyan/kanıt
a4e1045; native 13 vaka, 736/736 faz, offline vendor 163 ve iki-clone
release geçti. GB-041 kapalı; global RSS garantisi ve sürekli geniş
fault-proxy matrisi değildir. Exact ham kanıt
[buradadır](incelemeler/k206/frame-guard/README.md).

## K-208/K-209 — Zee çekirdek legacy giriş ve audit IP

Gerçek ürün talebi ve exact köken dogfood/zee-cekirdek altında korunur.
K-208 bcrypt $2a/$2b/$2y yalnız doğrulama (maliyet 4–14, 72 UTF-8 byte
sınırı), K-209 trusted `istemci adresi` Metin ifadesidir. Node vektör,
real worker proxy/socket/IPv6/spoofed XFF, CLI boş değer ve replay testleri
mevcuttur. RFC-0030 geçici kabul/ADR-082; yeni özet Argon2id kalır.
Ürün upgrade/migration/audit saklama işini kendi tarafında yapar.
K-174 syntax gözlem penceresi gerçek K-209 grammar değişikliği nedeniyle
2026-10-13–2026-11-10 olarak yeniden başladı; B-001/B-002 açık.
Tam faz, source beyanı ve vendor/release kapanışı beklenir.

K-208/K-209 kapandı: feature source 2c355ef, clean beyan/ikili kanıt
daaa3ea. 742/742 faz, Node crypto ve gerçek HTTP/CLI/trace kabulü,
167 paket offline vendor ve iki-clone binary eşitliği geçti.
[Ham kayıt](incelemeler/k208/README.md) yerel macOS kanıtıdır; V1
usability/RC/signed release ve sunucu dağıtımı değildir.

### ürün E özelliklerinin ana kaynak hattı — K-213

[ADR-086](../adr/086-site-hatlarinin-birlestirilmesi.md) mevcut ürün E
parola üretimi/TOTP/SMTP, rota birimi, görünüm, görsel ve gizli değer
yüzeylerini K-208/K-209 bcrypt/IP ile aynı hatta toplar. Ürün kabulü
dosya kontrolüyle değil bütün proje `dil dene .` ile alınır. Ham ürün E
şablonu normal katalog üretimini ister. Exact source/CI kapanışı beklenir;
canlı dağıtım ve otomatik parola göçü bu işte yapılmaz.

K-213 clean source kabulü: source e09493e, beyan/ikili/test 0531432;
769/769 faz, beş bütün proje 242/242, 187 paket offline vendor ve
macOS arm64 iki-clone eşliği geçti. Exact [kayıt](incelemeler/k213/README.md).
İlk Linux CI'ın test Git kimliği bağımlılığı düzeltildi; son uzak CI
kapanışı beklenir. Canlı dağıtım ve otomatik parola göçü açık kalır.

K-213 compiler gerçekleme/test/belge dilimi kapandı. Eski ürün E
sunucu hattı ile master aynı kaynak geçmişinde; ürün CI/sunucu pininin
bu kaynağa geçirilmesi ve canlı giriş/IP kabulü ürün tarafının işidir.
Otomatik bcrypt→Argon2 ürün göçü henüz uygulanmadı.

### K-214: Anonim açık depo kanıtı

Müşteri ve altyapı kimliği anonimleştirildi; yeni içerik hash politikasıyla
CI içinde denetlenir. Eski geçişler K-214 sırasında maskeli dökümle ayrı
karar bekledi; K-215 için kullanıcı temizliği onayladı. Tarihsel ölçümler
eski kaynak kimliklerini korur; yeniden açılış ve
GitHub PR önbelleği temizliği ayrıca onay/teyit ister. [Kayıt](incelemeler/k214/README.md).

### K-215: Önceki ürün geçişleri ve bağımsız yerel yama

K-214'te ayrı bırakılan geçişler kullanıcı onayıyla anonim kaynak soy ağacına
taşınır; eski fingerprint istisnaları kaldırılır ve ilk commit dahil tüm ata
geçmişi denetlenir. Compiler/fuzz PostgreSQL yaması path bağımlılığıdır;
derleme self Git deposu veya credential gerektirmez. SBOM gerçek build
commit'ini ve yerel paket baytlarını kayıtlı SHA-256 ile doğrular.
Tarihi ölçümler yeni kaynakta koşulmuş sayılmaz. GitHub PR refs/cache teyidi
ve yeniden açılış onayı açıktır; depo gizli kalır.

K-215 yerel makine doğrulaması tamam: 771/771 test, bütün yayımlanmış
ref geçmişinde sıfır gizlilik eşleşmesi ve iki temiz klon ikili eşitliği.
Güncel master uzak CI'ında altı iş hesap ödemesi/harcama limiti nedeniyle
başlamadı; test adımı sıfır. Linux/Windows ve uzak kapı kanıtı açık kalır.
[Hesap engeli kaydı](incelemeler/k215/ci-hesap-engeli.json) ve GitHub destek
teyidi tamamlanmadan yeniden açılış hazırlanmış sayılmaz.

### K-216: Git2 ve Gitsrv üzerinde koşular

Kullanıcı CI koşularını özel Git2/Gitsrv sistemine taşıdı. Ana GitHub kaynağı
merbay-erp/zee, Enterprise yalnız aynadır; ikisinde Actions kapalıdır.
Forgejo akışı tam Linux faz/kalite, gizlilik/sır, tedarik/offline vendor ve
WASM kapılarını exact seçili kaynakta çalıştırır; ilgisiz özel dal/etiket
almaz, PR kodu otomatik çalıştırılmaz. Uzak sonuç doğrulanınca kaydedilir;
Windows runtime kanıtı ve GitHub destek teyidi açık kalır.
[Karar](../adr/088-git2-gitsrv-kosulari.md).

İlk Linux koşusu #20'de 770/771 test geçti. Tarihsel karşılaştırma kanıtı
özgün ölçüm SHA'sını korur; anonimleştirilmiş kaynak soyu K-214/K-215 exact
eşlemesi ve HEAD atası şartıyla doğrulanır. Eski kaynak nesnelerini çekmek
gerekmez. Bozuk/çelişkili/ilgisiz eşleme hermetik testle reddedilir. Gitsrv
Go eşdeğerliğini doğrulanmış Go 1.26.1 aracıyla gerçekten çalıştırır.

## K-217 — Tek merkez ve dal incelemesi

Ürün talebi §2: site temiz içerik, issue formları, özel Forgejo MSVC
çapraz akışı ve incelenmiş action pinleri ana hatta uygulanır. Snapshot
compiler/CLI genişlemesi core freeze kanıt zinciri olmadığı için kapatılır.
Dal yedekleri özel; uzak silme ve CI teyidi henüz açıktır.
[Dal kararları](incelemeler/k217/README.md). Dil/spec/tanı/API değişmez.

### K-218 — Bellek oran backend'i

[ADR-089](../adr/089-bellek-oran-deposu.md) ile `--web-oran-deposu bellek`
güvenli proxy'de süreç başına oran sayacı seçer; varsayılan ortak dosya kotası
korunur. Oturum/revoke kalıcı paylaşılır. Çok worker ve restart ortak kota
vermez; proxy'de ortak limit gerekir. PostgreSQL ve §3/2 kota ayarı/tahliye
açıktır. Exact 1800 sn soak ve gerçek ürün yük kabulü kapanış kanıtına eklenir;
yalnız yerel ilk testlerle performans/sürüm hazır ilan edilmez.

K-217/K-218 kapanış (14.10.2026): §2 dalları temiz içerik/gerekçeli kapatma
ve exact lease silme ile tek master'da toplandı. Süreç başına bellek oran
seçimi gerçek ürün yükünde 54→2638 tek-worker ve 54→5079 iki-worker
istek/sn gözlemiyle, exact 1800 sn soak %0 RSS büyümesi ve Gitsrv #24
başarılı sürekli akışıyla kabul edildi. Kaynak 4e351d34, exact beyan 5a3cb270,
uzak kanıt 4077efe28; [tam kanıt](incelemeler/k218/README.md). Varsayılan
ortak kota ve kalıcı oturum paylaşımı korunur; explicit bellek seçiminde
proxy ortak saldırı bütçesi gerekir. Windows runtime/RC/usability ve
§3/2–5 açık kalır; bu değişiklik V1 hazır veya canlı yükseltme değildir.

### K-219 — Bellek oran ayarları

[ADR-090](../adr/090-web-oran-ayarlari.md) / RFC-0032 ile yalnız explicit
bellek backend’i için `--web-oran-siniri`, `--web-oran-kapasitesi` ve
`--web-oran-doluluk` eklenir. Varsayılan 100/60 sn, 32768 ve reddet;
en-eski yalnız uç nokta pencerelerini çıkarır, login/CSRF canlı sayaçları
korunur. Tahliye ile uç nokta kotası sıfırlanabilir; proxy ortak saldırı
bütçesi gerekir. Dosya backend’i ve kalıcı oturum paylaşımı değişmez.
Kod/test/spec aynı dilimdedir; exact 1800 sn soak, 786/786 yerel faz, ürün
kapasite yükü ve Gitsrv #26 kabulü [kanıt sayfasında](incelemeler/k219/README.md)
tamamlandı. Ürün sunucusu yükseltilmedi.

### K-220 — Ürün HTTP POST ve başlıklar

[ADR-091](../adr/091-http-post-baslik-ve-gizli-ortam.md) / RFC-0033: native
HTTPS GET/POST, başlık ve metin gövde, açık 1..30000 ms süre; Metin sözlüğü
Sonucu (`durum`, `gövde`). Ortam sırları yalnız explicit ana proje listesiyle
okunur; yeni sır taşıyan işlem iz kaydında/replay’de kapalıdır. Kanonik HTTP
JSON helper bool/metin ayrımını korur. Eski GET ve Rust facade değişmez;
WASM yeni native adaptörü açmaz. Ürün Turnstile helper ve yerel TLS testleri
aynı dilimde; tam faz/Gitsrv sonucu [kabulde](incelemeler/k220/README.md) kapanır.
Sunucu/ürün rotası ve otomatik kayıt/e-posta entegrasyonu ürün tarafındadır.

K-220 / K-174: exact gerçek ürün talebiyle üç contextual kalıp eklendi; syntax gözlem penceresi 14.10.2026–11.11.2026 olarak bilinçli yeniden başladı. Core freeze ve insan/RC kapıları sürer; V1 tamamlandı olarak gösterilmez.

K-220 CI kabul düzeltmesi: sentetik localhost PKCS12 kimliği exact hash/kaynak doğrulamasıyla sır taramasında tanınır; başka anahtarlar reddedilir. Dil/HTTP çalışma davranışı değişmez, Gitsrv tekrar kabulü beklenir.

K-220 makine kabulü: kaynak 488dc7d, exact CI 3d7ff93 / Gitsrv #29 başarılı; 801/801 yerel faz ve bağımsız kullanım rehberi temiz. Compiler kaynağı kapanışta değişmedi; canlı entegrasyon ve sunucu yükseltmesi ürün tarafında kalır.

### K-221 — Bilinmeyen seçenek tanısı

CLI seçenek bölgesindeki bilinmeyen `--` artık dosya okunamadı hatasına
dönüşmez; seçenek ve etkin sürümle exit 2 döner. `çalıştır` kaynak yolundan
sonraki program argümanları korunur. Eski ikili yeni runtime ayarlarını
kendiliğinden desteklemez; rollback ayar uyumu ayrı kalır.

### K-222–K-225 — Ürünleşme kanıtları ve pilot sınırı

Hermetik gerçek PostgreSQL TLS/NULL dedicated kapısı; anonim mevcut dilde
parola rehash/form helper kabulü; beş doğrudan spec testi ve sınırlı pilot
destek politikası eklendi. Lisans, insan/temiz fiziksel/minOS ve destek
teyidi açık; yeni compiler özelliği veya canlı migration iddia edilmez.

### K-226 — Exact build kimliği

`dil sürüm --json` source SHA/tarih/kanal/dirty verir; metin sürümü bunları
gösterir. LSP zeeBuild ve worker X-Zee-Build/startup aynı kayıttır. Tarih
source commit UTC günüdür; yeniden derleme duvar saatini ikiliye gömmez.

### K-229 — Yazmadan biçim denetimi

CI için `dil biçimle --denetle <dosya|proje>`: uygun biçim0, fark1; kaynak
içeriği/mtime/izin değişmez. Default biçimleme writer aynı şekilde sürer.
K-226 dirty işareti conservative tüm kaynak ağacı değişikliklerini kapsar.

### K-228 — SQL yazmadan göç planı

`dil göçür --durum|--kuru [proje]` bütün/bekleyen migration'ları gösterir.
Plan read-only snapshot'tır; tablo yokken metadata oluşturmaz. Default
apply tek transaction/advisory lock ile sürer; göç credential/SQL basılmaz.

K-223 native parola kabul koşucusu gerçek PostgreSQL üzerinde yedi vakayı
atlamadan denetler; strict rapor sözleşmesinin dokuz ret testi vardır.
[Koşucu ve C027/live sınırları](parola-goc-ci.md).

K-235 test kanıtı DB kaynak bütçelerini gerçek PostgreSQL'de tam eşik ve
bir fazla retle doğrular; production limitleri değişmez. İki kısmi spec
kaydı DB bütçeleri ve CLI/LSP/SBOM kimlik eşliğiyle kanıtlı oldu.

### K-234 — Projede sürümlü runtime ayarları

`dil çalıştır <proje-klasörü>` [zee.ayar](zee-ayar.md) profilini okur;
CLI override, unknown ordinary uyarı, unknown security ret. İlk destek
zee-ayar-1 yetenekli 0.8.0-dev; gerçek 0.7 rollback kabulü açık kalır.

### K-232 — Ekli SMTP e-posta

[Ekli kalıp](../rfcs/0034-ekli-eposta.md) native ortamda PDF/PNG/JPEG/WebP
readonly ek, Reply-To ve HTML/düz alternative sağlar. Dosya-okuma ve ağ
izinleri gerekir; IO izi ve WASM bu çağrıda başarı üretmez. Sır içermeyen
C029 ve DATA yanıt kaybında belirsiz teslim; otomatik retry yok. Queue/live
provider/ürün kabulü açık. Eski plain tel biçimi K-232'de korunur.

K-233 [pilot compiler seçici](derleyici-secici.md) exact yerel signed paketi
proje piniyle seçer; kurulum/indirme/apt/container/global shim değildir.
Config projede compiler capability guard ve fail-closed kontroller vardır.

### K-236 — SMTP hata ayrıntısında sır maskeleme

Eski plain ortam/explicit SMTP kalıplarının C029 hata mesajı artık sunucu
AUTH echo’sunu, URL/credential veya host ayrıntısını taşımaz; sabit güvenli
mesaj döner. Hata mesajını ayrıştıran uygulamalar C029 kodunu ve Sonuç
başarısını kullanmalıdır. İleti tel baytları, TLS/deadline ve retry yokluğu
korunur; başarısızlık teslim edilmedi garantisi değildir.
[ADR-099](../adr/099-smtp-hata-maskelemesi.md), gerçek native/CLI AUTH535
kabulü ve exact `smtp` regression aynı güvenlik dilimindedir.

### K-237 — SMTP yanıt büyümesinin okuma öncesinde kesilmesi

LF/EOF vermeyen sunucu artık mevcut 16 KiB SMTP yanıt sınırını String büyüterek aşamaz. Tam sınır ve geçerli tel baytları korunur; taşmada UTF-8 hata önceliği sınır reddindedir. K-236 güvenli hata maskesi aynı; ayar/grammar göçü veya retry yoktur. [ADR-100](../adr/100-smtp-yanit-okuma-siniri.md): sabit BufReader ön okuması, TLS/kernel/RSS ve eski plain timeout/MIME deadline bu payload sınırından ayrıdır.

### K-238 — HTTPS tedarik bağımlılığında rustls güvenlik yaması

Compiler ve fuzz kilitleri `rustls 0.23.46`ya yükseltildi; `ureq 3.4.0`,
kullanıcı kalıpları ve ayarlar değişmedi. Dört RustSec advisory için ignore
eklenmedi. [ADR-101](../adr/101-rustls-advisory-yamasi.md) standart ring
config ile upstream tetik koşullarını ayırır; dört ayrı ürün exploit'i
iddia edilmez. Yeni ikili/SBOM yeniden üretilecektir; tarihsel artifact ve
imza kayıtları bu yamaya aitmiş gibi değiştirilmez.

K-239 özel CI bağlantılarını yayınlanabilir inceleme raporundan ve yalnız
izole çalışma geçmişinden kaldırır; özgün kayıt dış kanıt paketinde korunur.
K-232 intrinsic kayıt testi 25 kimliği ve MIME Ağ/geri-alınamaz etkisini
bilinçli sayar. K-236 C029 kimliği korunur; fixture özeti yalnız teslim
sonucu belirsizliği ve hata maskelemesini açıklayan editoryal güncellemedir.
K-238 mevcut GET yürütmesi HTTP handler’a taşınarak 900 satır mimari sınırı
korunur; deadline, C018 ve tel davranışı aynı kalır.

### K-241 — Temiz sürüm klonunda doğru build kimliği

Sürüm üreticisinin kendi logları kaynak klonunun dışına taşındı. Her klon
derleme öncesi/sonrası temiz kalmalı ve gerçek `dil sürüm --json` exact
kaynak SHA'sı ile `dirty: false` bildirmelidir. Aynı hash'e sahip iki kirli
ikili veya değiştirilmiş klon artık çıktı oluşturamaz. Dil anlamı, build.rs
kirli-ağaç denetimi ve mevcut imza/platform sınırı korunur.
[ADR-102](../adr/102-surum-klonu-ve-build-kimligi.md).

### K-242 / K-243 — COMMIT fault kanıtı ve kaynak ZIP güvenilirliği

Native parola test koşucusu yedi TLS vakasına dört owned loopback kontrol
ve COMMIT fault vakası ekler; C027 ve tekrar etmeme davranışı korunur.
Salt SQLSTATE ile rollback sonucu çıkarılmaz. Strict skor kartı ve proxy
negatifleri normal Cargo kapısındadır; canlı ürün kabulü ayrı kalır.
Kaynak ZIP başlangıç SHA'sını tek kez çözer, Türkçe dosya adlarını NUL
listeyle okur, canonical depo dışı/metadata/kaynak çıktısını reddeder ve
yayın öncesi gerçek byte/comment/manifest eşliğini denetler. Yerel arşiv
aracı Python3 standart kitaplığını gerektirir. Dil/config göçü gerekmez.
[ADR-103](../adr/103-native-commit-belirsizligi-kaniti.md),
[ADR-104](../adr/104-exact-kaynak-arsivi-siniri.md).

K-244 Windows kaldırıcı receipt yollarını artık LiteralPath olarak okur,
hashler ve siler; `bin[1]` gibi adlar yabancı `bin1` dosyasını seçmez.
Checksum/Zorla/manifest sözleşmesi korunur. Altı gerçek geçici PowerShell
macOS senaryosu yerel kabul; fiziksel Windows/minOS kabulü ayrı kalır.
[ADR-071](../adr/071-kurulum-kaldirma-sozlesmesi.md).

K-245 sınırlı POSIX credential launcher, mevcut çalıştır argv ve proje
izinlerini korur;14 actual vaka Cargo kapısına bağlıdır. Tam P25 veya
Windows/systemd/canlı ürün kabulü değildir (ADR-105/GB-068).

K-246/P13 (ADR-106/GB-069) worker host identity+ayrı readiness ve sahipli
cleanup pilotudur; public launcher cleanup bug'ı temiz c67a69e üzerinde 25-vaka kabulüyle kapandı.
Tam request drain/reload/metrics ve P14 socket devir açıktır.
K-247/P09 (ADR-107/GB-070) dar idempotent SQL queue helper'ıdır;11 native
PG crash/COMMIT observer ve16 validator temiz c67a69e kabulündedir. Canlı ürün adoptionı, SMTPretry ve full P08 açık kalır.

K-249/P21 (ADR-109/GB-071) tek anonim TR–EN article fragmenti, plain
Metin slotları ve sabit sözlüktür. Dört kaynak testi, yedi native CLI
snapshot/XSS/ret senaryosu mevcut grammar/API ile temiz c67a69e kabulündedir. Genel layout/i18n ve gerçek ürün/P08 kabulü açıktır.

K-245–K-249 [temiz pilot kabulü](urunlesme-pilot-k245-k249.md)
`c67a69e6ed9c24c4a540a55eeaaa986a348a2de3` üzerinde 895/901 faz,
0 başarısız/6 dedicated atlama; ayrıca 28 PostgreSQL, credential14,
supervisor25 ve iki Python sürümünde TR–EN7+4 ile kanıtlıdır. İki temiz
klondan Mac ARM64 ikilileri eş ve dirty=false; yeni sır istisnası yoktur.
Lisans/insan/fiziksel/tam ürün/uzak CI/RC kapıları ve genel P09/P13/P14/
P21/P25 kapsamı açık kalır. Takip belge commit'i test kaynağı değildir.

### K-252 SMTP süre düzeltmesi

Native düz metin SMTP artık tek mutlak bütçeyi bağlantı ve SMTP/TLS okuma-yazmalarında korur. Yavaş parçalı yanıt süreyi yenilemez; C029/no-retry ve API aynıdır. Sistem DNS iptali yoktur. Göç gerektirmez; [ADR-110](../adr/110-smtp-mutlak-sure-butcesi.md).

## K-250 — PostgreSQL statement fault kanıtı

[ADR-111](../adr/111-postgresql-statement-fault-kaniti.md): gerçek 40P01 ve
40001, no-retry sayaçları, outer rollback onayı ve bağımsız durable observer
iki native vakayla sınanır. Dedicated TLS kapısı yedi vakadır; COMMIT
belirsizliği ADR-103/C027 olarak kalır. Çalışma ağacı native7/strict9 geçti;
temiz exact ortak kabul ayrıca gereklidir; provider/full fault matrisi açık.

## K-251 — DB ve göç hazırlık probu

[ADR-112](../adr/112-db-goc-hazirlik-probu.md), mevcut CLI ile uygulama
sahipli tek seferlik DB+migration gate üretir. Native13/strict10 yerel aday
kabulü vardır; compiler özelliği yoktur. DB/worker nesli değişince yeniden
prob gerekir; continuous/TTL, request metrics/drain/config reload ve P14
socket devri açık kalır. Gerçek ürün/platform adoptionı ve exact ortak temiz
kabul ayrıca gereklidir.

## K-253 — Native Unix graceful shutdown

[ADR-113](../adr/113-native-web-sigterm-drain.md), gerçek B5 ürün ihtiyacını
anonim native listener reproducer'ına bağlar. Yeni grammar/CLI/API yoktur;
SIGTERM mevcut admitted handler'ı bitirir, yeni admission durur ve idle
listener kapanır. Supervisor default5s korunur, max60s/explicit55s fırsatı
vardır; dış grace sonunda KILL/cleanup sürer. Native4/guard3 ve eski host25
stage kanıtıdır; exact kaynak/beyan ve final soak1800s kabulü ayrıca gerekir.
Bu semantic-change/dogfood-change'dir; bug fixed_by kaydı icat edilmez.
Metrics, Windows signal drain, tüm backlog/P14 socket/overlap ve adoption açıktır.

## K-254 — Ayrı SQL kira sahibi heartbeat

[ADR-114](../adr/114-ayri-sql-kira-heartbeat.md): mevcut SQL-only job satırı
handler'da kilitliyken ayrı lease satırı yenilenir. Lease30s/toplam120s/24renew;
immutable0002 geçişi, old-generation fencing ve mixedv1/v2 ret sınırı vardır.
Dedicated21 native+24validator stage kabulündedir; final exactclean ayrı.
Dispatcher/cron/SMTP non-idempotent etkiler ve gerçek katalog adoptionı açık.

## K-256 — TLS1.1 capable-positive kontrol

[ADR-115](../adr/115-tls11-capable-positive-kontrol.md): owned verifiedTLS1.1
positive connection+server gözlemi sonrasında native ret, dedicated8/strict14.
Yeni compiler özelliği/tanı yoktur; legacy cipher policy test contextiyle
sınırlıdır. Provider/full matrix/Windows/uzakCI ve final exactclean kabul ayrı.


## 15 Ekim 2026 temiz teknik kontrol noktası

[K-250–K-256 kontrol noktası](urunlesme-kontrol-noktasi-20261015.md)
exact temiz d42f147 üzerinde faz904/913 (0 başarısız/9 dedicated atlama),
ayrı PostgreSQL41, DB hazırlık13, native drain4, credential14, worker25 ve
iki Python sürümünde TR–EN7+4 kabulünü kaydeder. K-252 exact regression ve
beyan zinciriyle kapalıdır. K-253 mevcut admitted handler'ı SIGTERM sonrası
bitirir; final1800s soak hâlâ gereklidir. K-254 uzun handler heartbeat ve
K-256 verifiedTLS1.1 capable-positive ret kanıtı artık çalışma ağacı
adayı değil, bu exact kaynakta yerel kabuldür. Metrics/socket/dispatcher,
gerçek ürün, insan/platform ve RC/release kapıları açık kalır. Kaynak
kimliği daha sonraki belge veya özellik commit'lerine taşınmaz.

## K-262 — Mevcut web oran CLI argüman kayması

Temiz17b preexisting CLI bug kanıtlıdır: mevcut dört değerli oran seçeneği kaynak öncesi tüketilmediğinde APP_ARG yerine bellek görünür. K-262 dar kaynak öncesi skip onarımı, kaynak sonrası oran argümanları ve eski globalproxy sınırını korur. Yeni flag/grammar/export yoktur. Dar bugfix KAPALIDIR: kaynak commit `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179`, olumlu/olumsuz gerçek CLI testleri, ADR-121 ve `regression/v2.tsv` içindeki `web-oran-program-argumani` exact fixed_by kaydı semantic-bugfix/core-freeze correctness zincirini tamamlar. Final exact RC ve birleşik özellik kabul kapıları AÇIKTIR; bu kayıt sürüm veya ürün adoption kapanışı değildir.

## K-257–K-260 — v0.8.0-dev için sınırlı yerel aday profiller

Native proxy worker için `--web-metrik-fd` içeriksiz request ve gerçek PG pool
sayaçlarını inherited nonblocking FIFO'ya yollar; best effort'tür, endpoint veya
v1 facade export açmaz. Credential launcher tek `--native-metrics-fd` capability
ile bu metadata kanalını aktarır; default noinherit korunur. Socket control
FD'leri credential launcher üzerinden desteklenmez.

Native Unix host için dört kaynak-öncesi seçenek eklenir: `--web-listener-fd`,
`--web-ready-fd`, `--web-activate-fd`, `--web-generation`. Bunlar explicit proxy
origin/worker port, exact owned listening TCP socket ve fresh control/receipt
ister; arbitrary descriptor/stdio/role aliases sessiz exit2 ile kaynak/config
okunmadan reddedilir. Successful accept native inherited profile'da admission'dır;
TERM sonrası admitted stream tamamlanır. Default direct bind değişmez.
Süreç belleği quota seçimi inherited overlap'ta reddedilir; shared JSON resetlenmez.

`scripts/job-dispatcher.py` sabit SQL niyetini1..100 tick pilotuyla yürütür.
Belirsiz COMMIT/timeout/sinyal otomatik replay/fail yazımı yapmaz; üç bilinen
rollback denemesi deadletter'da durur. Log pipe doluysa bounded kayıt düşer,
SQL ilerlemesi bloklanmaz. Genel scheduler/cron veya dış SMTP/PDF effect açılmaz.

Listener17/FD16, metrics8, credential7+2 ve dispatcher11 çalışma ağacı aday
kanıtlarıdır. Finalclean/900sn/1800sn/ASAN RC ve gerçek ürün/platform kabulü bu
sürüm notundan üretilmez. [Dar kullanım](job-dispatcher.md),
[native listener](native-listener.md) ve ADR-116–119 açık kapsamı korur.

Güncel ek sınır: K263 owned-host cleanup dar düzeltmesi ac077e31 ile commitlendi; yukarıdaki yerel
aday makbuzları birleşik finalclean özellik kabulü değildir.
K262 dar source/argv düzeltmesi 7c56518c ve exact provenance4d8edeb2 ile kapalıdır; ortak özellik kabulü açıktır.

Kanonik güvenlik sırası: atomik K-263 owned-host cleanup düzeltmesi GB-077 ile `ac077e31e761e401eedc5140a8035fd02f2a19ba` kaynak commitinde tamamlandı; K-262 dar CLI bugfix `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179` ve exact provenance `4d8edeb2a9f4453d2f28481ecd550bf8ba5f2032` ile kapandı. Ortak temiz özellik kabulü ayrıca beklenmektedir. Kaynak `54a8d6dc0efb060ae5644b37546da40ad03ee035` ile commitlenen özellik dilimlerinin kayıtları K-257/GB-078, K-258/GB-079, K-259/GB-080 ve K-260/GB-081’dir; ADR-116–119 numaraları korunur. Önceki immutable dış-stage makbuzlarının geçici GB etiketleri tarihsel kanıttır, kanonik yeni kayıt kimliği değildir. Finalclean/900sn host/1800sn soak/RC kabulü bu revizyonda ileri sürülmez.

## K-263 — Sahipli supervisor EPERM geçişi

K-263/ADR122/GB077 dar host correctness adayıdır: gerçek sahipli worker EPERM/WNOWAITNone geçişi artık tek mutlakgrace deadline içinde heldleader üzerinde gözlenir. Persistent ambiguity/returncode-set/ECHILD failclosed, canlı descendant temizliği ve foreigngroup sınırı korunur. Ayrı8 lifecycle testi +Cargo wrapper1 mevcutworker25 sayısını değiştirmez. Compiler source/API/grammar, performans/soak ve K262/featurewave kapsamı değişmez. Dar host source commit ac077e31e761e401eedc5140a8035fd02f2a19ba ve 905/914 faz kapısı tamamlandı; birleşik özellik finalclean gate AÇIKTIR; tamamlanmış release/adoption iddiası yoktur.

## K-264 — Kritik işlevlerde dar sorumluluk incelemesi

[ADR-123](../adr/123-kritik-islev-secici-taban-incelemesi.md) dört yeni kayıt ve yalnız govde tabanını gerekçeli kabul eder. Bütün diğer satır değerleri/eski kimlik/global paylar korunur; gerçek islev_egilimi denetimi64 kayıtla geçmiştir. Kaynak davranışı değişmez; fullclean/RC/soak/ürün kapıları ayrı açıktır. Mevcut olmayan parser/MIME kapsamı varmış gibi gösterilmez.

K-257–260 birleşim adayında seri tam faz 924/936 geçti, başarısız0/atlanan12; kaynak+belge93 dosya koşu boyunca aynı hashte kaldı. Ayrı fmt/clippy, katalog1+tazelik6, spec214 ve işlev64 kapıları geçti; sırtaraması/sürekli güvenlik temizdir. İlk başarısız koşu (paralel Cargo binary-pin değişimi ve korpus ürün politikası receipt yetkisi eksikliği) dış immutable kayıtta korunur. Bu sonuç finalclean, dedicated DB/native, 900sn host,1800sn soak veya RC kabulü değildir.

K-257–260 kaynak commit `54a8d6dc0efb060ae5644b37546da40ad03ee035` öncesi tam faz926/938 geçti, başarısız0/atlanan12; dondurulan97 kaynak+belge aynı kaldı. K-259 minimal `native-listener-late-eof` vakası introduced_by=- ve exact fixed_by bu SHA ile takip provenance dalgasında kaydedilir; gerçek Unix replay ayrı nonempty testtedir, Unix dışı native vaka açıkça ignored kalır. Semantic-change/dogfood-change beyanları K-259/ADR-118 ve gerçek B5/B6 provenance zincirine bağlanır. Bu kaynak/provenance kapanışı finalclean/native900/soak1800/RC veya ürün adoption kapanışı değildir.

## K-265 — Exact kaynak dış RC kanıtı

[ADR-124](../adr/124-exact-kaynak-dis-rc-kaniti.md) sourceS/historyE self-reference doğruluk boşluğunu explicit dış immutable paketle ele alır; kaynak eşdeğerliği/bypass yoktur. İzole25 sentetik protokol+9 kısa ownedprocess test dilimi, tam4ASAN2Miri/rawhash/initialsourcecorpus/erkenHEADpin/streamingdizinbütçesi negatiflerini kapsar. K-265 kaynak commit’i `1367ce1a7d0281d8a7f3d9307021afbc452da15f` ile kayıtlıdır: kaynak öncesi tam faz928/940 geçti, başarısız0/atlanan12; son EOF boşluğu düzeltmesi sonrası25 protokol testi yeniden geçti. Final gerçek cleanS uzunASAN/Miri kampanyası,900/1800/release/ürün adoption kapıları açıktır. Kullanıcı AGENTS değiştirilmez, exactsourceS ve append-only arşivE ayrımı açık tutulur.

## K-267 — Typed credential/native listener bileşimi

[ADR-125](../adr/125-credential-native-listener-kompozisyonu.md) mevcut D3/P25 ve B5/B6 ürün ihtiyacını dar Unix host profilinde birleştirir. Credential okumadan all-or-none FD/rol/socket/port/argv doğrulaması, native child tarafından devralınmayan ayrı startup FIFO ile PID/generation/binary kimliği ve varsayılan noinherit korunur. İki normal Cargo vakası model/subprocess5 ile gerçek native crypto/overlap/metrics/admission/cleanup5 kampanyalarını ayırır; alt-senaryolar eşsiz toplam diye sayılmaz. İzole dirty d53 kısa aday kanıtı final temiz kaynak kabulü değildir. P25 bütünü, Windows/systemd/file-native sır aktarımı, canlı benimseme ve 900/1800/RC kapıları açıktır. GB-083 bilinçli capability sınırını kaydeder; yeni grammar/compiler/public Rust API eklenmez.

K-267 doğrulama bakımı: macOS nativeTLS platform hazırlığı, mevcut SMTP IO
iptal testinin ölçülen kısmından önce tamamlanır; 60 ms IO bütçesi, iki saniye
takılma koruması ve sessiz peer değişmez. Production davranışı, tanı, test
sayısı ve normatif sözleşme değişmedi. Önceki iki tam matrisin aynı SMTP
korumasında kalması korunur; K-267 son tam faz koşusu 930/942 geçti, 0 kaldı, 12 atlandı; koşu boyunca 40 kaynak dosyası değişmedi. Final temiz kaynak ve uzun kabul kampanyaları ayrı kalır.

### K-268–K-270 yerel pilot kapasitesi

K-268/ADR-126/GB-084 sonlu timer37 hazırlığı; K-269/ADR-127/GB-085 kopyalanabilir site/panel2 mevcut CLI/HTTP pilotu; K-270/ADR-128/GB-086 yerel import/proje launcher14 ve korunmuş selector25. Yeni üç normal Cargo vakası gerçek alt-senaryolarıyla ayrı sayılır. Native Linux/systemd/SQL timer, gerçek kullanıcı/live ürün, production publisher/platform ve final temiz kaynak kampanyaları açık; P09/P23/P07 kısmi kalır. Yeni compiler/src/grammar/facade/dependency veya spec normatif madde yoktur; spec214 ve önceki source54/c86 maintenance/provenance, K267 kayıtları korunur.

[Timer](../docs/sonlu-dispatcher-timer.md), [başlangıç](../sablonlar/README.md), [yerel araç](../docs/derleyici-yerel.md).

K-268–270 ortak çalışma ağacı kabulü: dondurulan 41 dosya koşu boyunca
değişmeden tam faz matrisi **933/945 geçti, 0 başarısız, 12 explicit ignored**
sonucunu verdi; faz süreleri toplamı 376.425 saniyedir. Timer37, kopyalanan
site/panel2 ve yerel araç14 (eski seçici25 korunarak) normal Cargo kapısındadır.
Fmt/clippy/işlev64/spec214/mimari/SCC/API/uyumluluk/katalog/tazelik/tedarik,
sürekli güvenlik, kanonik sır taraması ve semantic/core-freeze korukları geçti.
Bu kayıt temiz exact kaynak/native Linux/systemd, 900/1800 saniye, ASAN/Miri,
kurulum veya sürüm kabulü değildir; ilgili açık sınırlar korunur.

### K-271–K-274: dar native geliştirme ve host profilleri

Native web fixture kiti20 hazırlık ve ayrı owned PG/HTTP4 senaryosu sunar;
TR/EN liste-detay golden9HTTP/5Zee eski7+4 çıktısını korur. Yönetilen modül aracı
yalnız yeni owned scaffold/metadata/journal içindir, SQL migration yürütmez.
Supervisor `--readiness-ttl-seconds` verilmezse eski davranışı korur;
DB hazırlık `--rounds` default1, explicit finite2–1000 ve `--interval`2–60s'dir.
Config yenileme yalnız atomic `zee.ayar` replace +HUP ile yeni worker okumasıdır.
Grammar/public Rust API değişmez; çok ürün/platform/P08 ve final clean/uzun
kampanya kabulü bu kısa pilotlarla kapanmaz.

### K-275 — Sabit yerel PDF örneği

K-275/ADR-133/GB-091 sabit owned PDF host profili: mevcut Zee JSON örneği, exact ReportLab4.4.9/pypdf6.10.0/Pillow12.3.0, stdlib8 normal ve hostbackend8 explicitignored ayrımı. Sabit kaynak/font/notice, bounded job/output ve nofollow roller korunur. Finalclean S native emitter+backend8 ortak kohortu ayrı zorunludur; tarihsel dirty kanıt temiz S sayılmaz. P11/B3 Zee içinden invocation/resmi signedplugin, ürün/adoption/P08/platform ve proje lisansı açıktır; full900/1800/RC/soak/kurulum kabulü ileri sürülmez. Yeni grammar/compiler/dependency/FFI yoktur; global Python/Codex runtime kurulumu varsayılmaz.

### K-276–278: Linux taşıyıcı hazırlığı, native dış etki ve editör şablonu

K-276/[ADR-134](../adr/134-linux-ozel-paket-tasiyicisi.md) özel inert Linux taşıyıcı hazırlığıdır: normal wrapper
12 preflight +9 carrier model/negatifi yürütür. ELF header doğrulaması Linux
ABI/çalıştırma kanıtı değildir. İki temiz klon kanonik teslimi, final kaynak S
üzerinde gerçek .deb build/install/remove/user-prefix activation/container
kabulü ve yayıncı/lisans/imza/apt repository ayrı açıktır. Namespace/image
manifest önkoşulu tamamlanmış Zee container kabulüne dönüştürülmez.

K-277/[ADR-135](../adr/135-sabit-native-dis-etki-tuketicisi.md), mevcut SQL-only dispatcher'dan ayrı fixed owned
loopback native SQL→MIME SMTP→ack profilidir. Effect-start latch bilinen committed
olmadan SMTP başlatılmaz; owner/token/generation/lease korunur. Yalnız kanıtlı
pre-effect SQL reddi, sıfır send invocation ile toplam3/1–2sn backoff yapabilir.
C029/DATA/ackCOMMIT belirsizliği guard+exit3/manual incelemedir; SMTP otomatik
replay/exactly-once sözü yoktur. Bilinen completion sonrası görünür guard unlink
fakat final fsync hatası ayrı exit4'tür; olmayan guard yeni admission'ı engelliyor
denmez. Normal hazırlık8 ve explicit ignored native11 ayrı kapılardır. Kaynak
harness yalnız dört pinli fixture dosyasını kopyalar; ek SQL/.dil/FIFO alınmaz.
Tarihsel dirty aday kanıtı finalclean değildir. Gerçek B1/B2 katalog, sağlayıcı
Gmail/M365, K-275 PDF üretimiyle ürün entegrasyonu ve tam P09/P08 açıktır.

K-278/[ADR-136](../adr/136-jetbrains-yerel-lsp-sablonu.md) mevcut K-270 launcher'ını kullanan POSIX LSP4IJ veri
şablonudur; TextMate mevcut VS Code bundle'ından gelir. Npm bağımlılığı olmayan
elle yazılmış VS Code istemcisi için owned `dillsp` mutlak yolu açıkça seçilir;
IDE macro/tokenizer/kurulum/etkileşim/Windows kabulü henüz açık kalır. Şablon
installer/download/yeni grammar/capability/cache veya gerçek IDE kabulü eklemez.
[PDF/SMTP'den ayrı editör rehberi](../editors/jetbrains/README.md) bu sınırı korur.

### K-279: gerçek önceki küçük sürüm korpusu ve açık göç

[ADR-137](../adr/137-onceki-kucuk-surum-korpusu.md) gerçek v0.7.0 exact
`d08a947713ddc38d2163360c17d07fbffcb1213d` kaynağındaki33 golden girdisini
değişmeyen blob/SHA ile korur.32 directcheck eşliği ve15 saf runtime çıktısı
gerçek eski/yeni CLI'da sınandı; birim örneği K-086 publictype sözleşmesi için
yalnız birimde iki annotation satırıyla göç ister. Ham33 kaynak gizlice
yenilenmez. Normal iki Rust vakası49 modernCLI altkontrol+5 erken bootstrap
reddini yürütür;54 libtest veya gerçek anonim ürün/P08 kanıtı sayılmaz.

Eski CLI config capability JSON sunmadığı için mevcut BuildJSON/selector
reddi korunur; yeni `zee.ayar` için otomatik legacy rollback açılmaz. Native
çalışma altkümesi effect-free'dir; ağ/web/dosya/saat/rastgele/async/hardware
runtime örnekleri çalıştırılmaz. Sabit33 kaynakta provenance olarak duran bu örnekler
compiler bugfix/dogfood veya minOS/platform kabulü değildir.

Linux'ta e76 retained leader cleanup ölçümü komut başına yaklaşık3.001s'dir.
52 sınırlı komutun yaklaşık156s toplam bakım yükü için yalnız test harness
aggregate bütçesi120→240s seçildi; her native çağrı8s, owned cleanup/deadline/
capture, kaynak/output oracles ve production SLA/perf eşikleri değişmedi.
Bağımsız Mac49+5 replay ayrı; gerçek tam Linux ve temiz finalS kabulü ayrıca
gerekir. Tam P04/P05, ürün/kanal/platform/adoption açık kalır. [Rehber](onceki-kucuk-surum-korpusu.md).

K-271–K-279 commit öncesi birleşik kapısı (15 Ekim 2026): tam faz matrisi
**943/959 geçti, 0 başarısız, 16 explicit ignored**; faz süreleri toplamı
429.100 saniyedir. Koşu boyunca 1.477 kaynak/belge/fixture dosyası aynı hash'te
kaldı. Fmt, tüm hedeflerde Clippy, işlev64, spec214, katalog/tazelik,
katman/SCC/public API/uyumluluk/tedarik, gerçek Zee–Rust–Go çıktı eşliği,
sürekli güvenlik, semantic/core-freeze ve kanonik sır taraması geçti.
Üç yeni Rust test sarmalayıcısındaki sabit platform assertion'ı açık Unix
koşullu gövde ve desteklenmeyen platform panic'iyle kanonik biçime taşındı;
Unix vakaları ve üretim compiler davranışı değişmedi. İlk Clippy retleri
ayrı dış kayıtta korunur. Normal fazdaki16 dedicated atlama temiz kaynak
kampanyasında ayrıca yürütülmelidir; bu sonuç final temiz kaynak/native,
900 saniye işletim, 1800 saniye soak, ASan/Miri, paket/kurulum veya gerçek
ürün/platform/IDE/insan kabulü değildir. Yeni compiler/src, grammar, spec
normu, public Rust API veya production bağımlılık değişikliği yoktur.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

Geliştirme kaynağı S için [yerel teknik kabul](urunlesme-final-kabul-20261015.md) kaydedildi: gerçek native web/PDF/SMTP/PG profilleri, Linux taşıyıcı ve konteyner denemeleri, uzun koşular ve kaynak kimlikli teslimler. Kullanıcıya yeni production compiler davranışı veya yeni sürüm etiketi eklenmedi; önceki0.7 K-086 göçü ve legacy rollback reddi korunur.


### K-280: JetBrains tamamlama ve normal çıkış

15 Ekim 2026 owned IntelliJ IDEA2025.3.1/LSP4IJ0.21.0 POSIX profilinde
mevcut S8b49113 launcher ile gerçek completion isteği7, `sayi 7 ols` →
`sayi 7 olsun` uygulaması ve ardından boş LSP tanısı doğrulandı. Trace ayarı
uygulanırken shutdown5/yanıt/exit ayrıca görüldü. Normal Quit/Confirm Exit
sonrasında original süreç sahibi exit0 ve kapalı/toplanmış grubu kaydetti;
zorunlu sonlandırma gerekmedi. Son Quit protokol konsolu dışa aktarılmadı;
popup görünürlüğü ayrıca kanıtlanmadı. Önceki retler korunur. Dar IDE
etkileşim eksikliği kapalı; Windows/eklenti yayın kanalı/incremental/lint/
20k satır200ms ve tam P19 açık. [ADR-138](../adr/138-jetbrains-tamamlama-ve-duzenli-kapanis.md),
[kabul kaydı](incelemeler/urunlesme-20261014/k280-jetbrains/ACCEPTANCE.json).
Yeni kaynak dalgası bu S kanıtını kendi derleyici kabulü sayamaz.

## K-281–K-294 yeni teknik aday dalgası

Güncel kaynak/aday test/eksik kabul ayrımı [ikinci dalga kaydındadır](urunlesme-ikinci-dalga-20261015.md).
İmzalı PDF hostu, sürekli hizmetler, SQL modül yaşam döngüsü, kaynak kapsamı,
`dene` HTTP/DB köprüsü, editör eşitleme/cache/lint, TR–EN bileşimi ve içerik/medya
başlangıcı eklendi. Önceki S1/E1 kabulü bu yeni kaynağa aktarılmaz. K-294
regression follow-up ve temiz exact normal/dedicated/RC/Miri/soak kapıları
bitmeden dalga tamamlandı sayılmaz; C1/P33 araç retleri açık tutulur.

## K-295 — daemon unit'in explicit yürütme düzeltmesi

S2 `e35b617095e6a9d00915794d6adee90842238a5f` gerçek Linux koşusunda
daemon ExecStart yalnız --config taşıdığı için check-only kipinde exit0 dönmüş,
hiç SQL tick başlamamıştır (0/4; cleanup_errors boş, owned_units_released true).
Üretilen daemon artık explicit --run, OnCalendar hizmeti yalnız --once taşır.
Doğrudan CLI varsayılan preflight ve render'ın servis başlatmama sınırı değişmez.
Eski unit dosyaları yeniden --render ile üretilmelidir. Exact S2 kaynağına karşı
yeni regresyon kırmızı, düzeltmeyle hazırlık22 yeşildir; S3 gerçek Linux dört vaka kabulü geçti. Host correctness düzeltmesidir; compiler bug/grammar/tanı/API değişmez.

## K-296 — deduplikasyonlu state olayı yerine gerçek liveness oracle

S2 release kabulünde worker-loss vakası geçti; stale-gate vakası aynı ready=false
durumunda yeni state olayı beklediği için kırıldı. Supervisor state'i yalnız
readiness değişince yayımlar; üretim davranışı korunur. Native koşucu iki worker'ın
Build kimliği doğrulanmış HTTP200 liveness'ını doğrudan owned Supervisor probe
ile gözler. Startup reddi, hiç ready=true olayı olmaması, eski gate bayt/mtime
değişmezliği ve bütün owned handle kapanışları birlikte zorunlu kalır. Yalnız
test oracle düzeltilmiştir; native DNS veya production lifecycle genişlemez.
S3 gerçek native iki vaka kabulü geçti.

## S3 — ikinci dalga temiz kaynak kabulü

Kaynak `66e4149e53451111d153f47283c219079a8d1cab` üzerinde ikinci dalganın native senaryoları, macOS/Linux için iki temiz klondan teslim ve sekizer kurulum testi, gerçek JetBrains tamamlama/biçimlendirme/tanı/kapanış akışı, 1.800 saniyelik soak, 1 ve 41 açık belge ölçümü, dört ASan hedefi ve seçili Miri testleri geçti. Kanonik kanıt dışa aktarımı ve denetimi ile aynı kaynakta güvenlik kapısı geçti: tam Cargo koşusunda 989 başarılı, 0 başarısız, 25 varsayılan dışı vaka. Bu 25 vaka ayrıca çalıştırılan kampanyalara eşlendi; alt senaryolar Cargo toplamına eklenmez. GB-110 exact regression/native26/RC zinciriyle kapalıdır.

[S3 teknik kabulü](urunlesme-s3-kabul-20261015.md), sonuçları ve önceki başarısız denemeleri ayrı kaydeder. E commitleri belge ve kanıtları kaydeder. C1/K-288 ve P33/K-293 araç retleri; artımlı HIR/type, gerçek ürün/provenance/adoption, insan/takvim, fiziksel Windows/minOS ve yayın/lisans/destek kapıları açıktır. Bu kayıt tam V1 ya da public signed release kabulü değildir.
