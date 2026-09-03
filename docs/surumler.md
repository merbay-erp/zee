# Sürüm notları

Her sürüm: ne eklendi, ne değişti, neyin sözü verildi. Kırıcı değişiklik
sessizce yapılamaz (master plan bölüm 23) — burada duyurulur.

## v0.7.0 — 1 Eylül 2026

**Tema: editör zekâsı — dil, düzenleyicide de Türkçe düşünüyor.**

(Üstteki K-069..K-072 kalemleri bu sürümündür: çıkış kodu, kitaplık
olgunlaşması, zengin doğrulamalar, morfolojili yeniden adlandırma.)

## Yolda (v0.8.0'a birikenler)

- **Akışlı binary upload ve dosya yaşam döngüsü** (K-163/F030,
  RFC-0027/ADR-061/spec-26): Native web adaptörü exact
  `application/octet-stream` gövdesini 16 MiB sınırında belleğe toplamadan
  `0600` temp dosyaya akıtır; SHA-256 ve boyutu aynı geçişte üretir. Yeni
  `atomik taşımayı dene`, `silmeyi dene`, `dosyaları listelemeyi dene` ve
  `sha256 özetini almayı dene` yüzeyleri beklenen arızayı C013 `Hata` değeriyle
  taşır, capability ve IO trace/replay sınırını korur. Çatlı'nın 970 LOC/4
  modül/10 test ürünü gerçek PostgreSQL 16.11 üzerinde normal upload→ready→
  delete akışını ve temp/metadata/rename/tombstone kesmelerinden restart
  yakınsamasını kanıtladı. Multipart, antivirüs, production TLS/pool ve nesne
  deposu hâlâ açık kapsamdadır.

- **Dosya yazma arızasında web worker survival** (K-163/F029): Çatlı medya +
  metadata protokolündeki gerçek dizin çakışması C013'ün yalnız isteği değil
  bütün worker'ı sonlandırdığını gösterdi. Web adaptörü artık C013'ü ayrıntı
  sızdırmayan 503'e dönüştürür, başarısız yazımı otomatik tekrarlamaz ve sonraki
  isteği kabul eder; CLI C013'ü korur. İlk ürün reproducer'ı `itwise-admin`
  `b3cee9a` commit'indedir. Binary upload, hash, fiziksel silme/orphan tarama ve
  production TLS/pool hâlâ açık kanıttır. Final gerçek koşuda C013 503 verdi,
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
  Çalışan ürün kanıtı `itwise-admin` deposundaki `a393128` commit'indedir.
  Bu yalnız localhost sslmode=disable profilidir; production TLS/pool sözü yoktur.

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
  F028 iki uçlu COMMIT-kaybı enjeksiyonunu kapattı; gerçek pool açık kanıttır.
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
  Production TLS/pool ve genel idempotency servisi hâlâ açık kapsamdadır.
  PostgreSQL geçişli bağımlılık kapanışı fuzz aracının ayrı Cargo.lock'una da
  işlendi; iki `cargo deny --locked` grafiği yeniden yeşildir.

- **Dogfood kanıt referans bütünlüğü** (B-073): Append-only ürün kaydı tekil
  slug, repo içi kanıt kökü, exact harici ürün commit'i ve durum taşır. CORE
  FREEZE koruğu artık dogfood değişikliğinin kayıtlı etkin ürüne, gerçek K-işine,
  ürün kökü altındaki Zee dosyasına ve ilgili karar belgesine bağlı olduğunu
  doğrular. Geçici Git testi kayıtsız ürün/iş, kök dışı dosya ve alakasız karar
  kaçışlarını reddeder. Reproducer'ın semantik ilgisi insan review'undadır.

- **İlk gerçek ürün dogfood'u başladı** (K-163): Çatlı/ITWISE Admin için temiz
  `codex/k-163-catli-dogfood` dalındaki `43171b9 → cef5ae3`, 495 satırlık üç
  Zee dikey dilimini kurdu. Duyuru, Argon2id yönetici oturumu/rol, CSRF CRUD,
  JSON site ayarları ve slug doğrulamalı taslak/yayında sayfa yaşam döngüsü
  7/7 hermetik testten ve gerçek TCP içerik+ayar+sayfa provasından geçti. İlk
  ürün sürtünmeleri ayrıca kaydedildi: P011 doğru çözüme götürdü; morfoloji ve
  zincirleme ergonomisi izleniyor; son kayıt silmede kaybolan boş CSV başlığı
  mevcut dille kapatılıp teste alındı. Dördüncü dilimde yerel PostgreSQL ve
  migration kanıtı da üretildi; K-163 production TLS/pool, medya ve süreli
  ergonomi kanıtı tamamlanmadığı için henüz kapanmadı.

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
