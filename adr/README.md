# ADR süreci

Güvenlik/mimari sınır değişikliği ADR ister (master plan bölüm 25).
Şablon: [000-sablon.md](000-sablon.md).

## Planlanan ilk ADR'ler (master plan bölüm 38)

| No | Başlık | Durum |
|---|---|---|
| ADR-001 | [Bootstrap dili: Rust + küçük/kilitli bağımlılık yüzeyi](001-bootstrap-dili.md) | **kabul** (K-088/K-092/K-117/K-192 revizyonu) |
| ADR-002 | [Parser: elle yazılmış, yüklem-sonlu dağıtım](002-parser-stratejisi.md) | **kabul** (K-097 katmanlar, K-119 formatter eşdeğerliği) |
| ADR-003 | [İlk yürütme: ağaç-yürüyen yorumlayıcı + IO soyutlaması](003-ilk-yurutme-modeli.md) | **kabul** |
| ADR-004 | Bellek yönetimi prototip kararı (GC / ARC benchmark) | bekliyor |
| ADR-005 | Native backend seçimi (Cranelift / LLVM) | bekliyor — Faz 4 |
| ADR-006 | [Paket registry güven modeli](006-paket-registry-guven-modeli.md) | **kabul/gerçeklenmiş** (K-094 yayın; K-095 metadata; K-117 kanonik yol; K-135 taşıma/cache/offline; K-136 exact proje/kilit/CLI) |
| ADR-007 | [Telemetri ve gizlilik: araçlar veri toplamaz](007-telemetri-ve-gizlilik.md) | **kabul** |
| ADR-008 | [Self-hosting aşamaları ve geçiş kapıları](008-self-hosting-asamalari.md) | **kabul** |
| ADR-009 | [Dilin adı: zee](009-dil-adi.md) | **kabul** (kurucu yetki devriyle) |
| ADR-010 | [Normatif otorite ve değişiklik bütünlüğü](010-normatif-otorite-ve-degisiklik-butunlugu.md) | **kabul** (K-118 kanıt/sayı tazelik kapısı) |
| ADR-011 | [Core AST intrinsic/yetkinlik sınırı](011-intrinsic-yetkinlik-siniri.md) | **kabul** (K-098/B-004) |
| ADR-012 | [Derleyici fiziksel faz modülleri](012-derleyici-faz-modulleri.md) | **kabul** (K-099/B-005) |
| ADR-013 | [Checker semantik katmanları](013-checker-katmanlari.md) | **kabul** (K-100 katmanlar, K-121 çağrı çıkarımı, K-207 ad maliyeti) |
| ADR-014 | [Semantic kimlikler](014-semantic-kimlikler.md) | **kabul** (K-101 temel, K-120 LSP tüketimi) |
| ADR-015 | [Derleyici faz tipleri](015-derleyici-faz-tipleri.md) | **kabul** (K-102/B-018, K-121 checker-içi keşif sınırı) |
| ADR-016 | [Typed HIR çekirdeği](016-typed-hir-cekirdegi.md) | **kabul** (K-103/K-104 runtime, K-120 LSP, K-121 nihai çıkarım) |
| ADR-017 | [Native ağ I/O kaynak sınırları](017-native-ag-kaynak-sinirlari.md) | **kabul** (K-105/B-025) |
| ADR-018 | [Sınırlı web oturum deposu](018-sinirli-web-oturum-deposu.md) | **kabul** (K-106; production uzantısı K-137/ADR-034) |
| ADR-019 | [LSP girdi sınırları](019-lsp-girdi-sinirlari.md) | **kabul** (K-107/B-047) |
| ADR-020 | [Zorunlu HIR kaynak aralığı](020-zorunlu-hir-kaynak-araligi.md) | **kabul** (K-108/B-020) |
| ADR-021 | [Production panic politikası](021-production-panic-politikasi.md) | **kabul** (K-109/B-014) |
| ADR-022 | [Lexer/parser fuzz politikası](022-lexer-parser-fuzz-politikasi.md) | **kabul** (K-110/B-015) |
| ADR-023 | [AST/HIR invariant doğrulama politikası](023-ast-hir-invariant-politikasi.md) | **kabul** (K-112/B-017) |
| ADR-024 | [Parser hata kurtarma politikası](024-parser-hata-kurtarma-politikasi.md) | **kabul** (K-113/B-021; K-166 §8–9 gürültü revizyonu) |
| ADR-025 | [Tanı kimliği kararlılık politikası](025-tani-kimligi-kararlilik-politikasi.md) | **kabul** (K-114/B-022) |
| ADR-026 | [Deterministik IO izi mimarisi](026-deterministik-io-izi-mimarisi.md) | **kabul** (K-115/B-027) |
| ADR-027 | [Sürümlü deterministik IO profili](027-surumlu-deterministik-io-profili.md) | **kabul** (K-116/B-028) |
| ADR-028 | [Kanonik `.zep` yol ve Unicode güvenlik profili](028-kanonik-zep-yol-profili.md) | **kabul** (K-117/B-030/B-031) |
| ADR-029 | [Ondalık ile binary float arasında örtük köprü yoktur](029-ondalik-binary-float-siniri.md) | **kabul** (K-125/B-012) |
| ADR-030 | [Bütün AST ifadelerinde kesin kaynak aralığı](030-kesin-ast-kaynak-araliklari.md) | **kabul** (K-126/B-050) |
| ADR-031 | [Merkezî yetkinlik politikası ve native outbound istemci](031-merkezi-yetkinlik-ve-outbound-istemci.md) | **kabul** (K-127/B-023/B-049) |
| ADR-032 | [Atomik replace metadata koruma politikası](032-atomik-replace-metadata-politikasi.md) | **kabul** (K-128/B-048) |
| ADR-033 | [Merkezî ve değişmez kaynak bütçesi](033-merkezi-kaynak-butcesi.md) | **kabul** (K-129/K-130/K-131/K-132; B-025 kapalı) |
| ADR-034 | [Kalıcı ortak web deposu ve tek-worker süreç modeli](034-web-ortak-depo-ve-worker-modeli.md) | **kabul** (K-137/B-046) |
| ADR-035 | [Protokol-kesin LSP JSON-RPC sınırı](035-protokol-kesin-lsp-json-rpc.md) | **kabul** (K-138/B-051) |
| ADR-036 | [Kanonik web proxy origin sınırı](036-kanonik-web-proxy-origin-siniri.md) | **kabul** (K-139/B-052) |
| ADR-037 | [Byte tabanlı HTTP/1.x istek sınırı](037-byte-tabanli-http-istek-siniri.md) | **kabul** (K-140/B-053) |
| ADR-038 | [Rust tedarik zinciri ve offline vendor kapısı](038-rust-tedarik-zinciri-kapisi.md) | **kabul** (K-141/B-054) |
| ADR-039 | [Hasım hosta karşı sürümlü WASM C ABI](039-hasim-hosta-karsi-wasm-c-abi.md) | **kabul** (K-142/B-055) |
| ADR-040 | [Playground girdi ön-tahsis bütçesi](040-playground-girdi-on-tahsis-butcesi.md) | **kabul** (K-143/B-056) |
| ADR-041 | [Kritik işlev boyutu ve karmaşıklık eğilim kapısı](041-kritik-islev-egilim-kapisi.md) | **kabul** (K-144/B-035) |
| ADR-042 | [Kanonik Rust biçim kapısı](042-kanonik-rust-bicim-kapisi.md) | **kabul** (K-145/B-037) |
| ADR-043 | [Faza özgü ve tam sahipli test matrisi](043-faza-ozgu-test-matrisi.md) | **kabul** (K-146/B-038) |
| ADR-044 | [Sürümlü semantic regresyon korpusu](044-surumlu-semantic-regresyon-korpusu.md) | **kabul** (K-147/B-039) |
| ADR-045 | [Performans gözlemi ve tarihsel eğilim artefaktı](045-performans-gozlem-ve-egilim-artefakti.md) | **kabul** (K-148/B-040; K-152 provenance revizyonu ADR-049) |
| ADR-046 | [Production katman sahipliği ve bağımlılık yönü](046-production-katman-bagimlilik-yonu.md) | **kabul** (K-149/B-057) |
| ADR-047 | [Production bağımlılık çevrimi kapısı](047-production-bagimlilik-cevrimi-kapisi.md) | **kabul** (K-150/B-058) |
| ADR-048 | [GitHub Actions immutable SHA pinleme](048-github-actions-immutable-sha-pinleme.md) | **kabul** (K-151/B-059) |
| ADR-049 | [Benchmark provenance ve örnekleme sınırı](049-benchmark-provenance-sinir.md) | **kabul** (K-152/B-060) |
| ADR-050 | [Gerçek LSP process cold-start ölçümü](050-gercek-lsp-cold-start-olcumu.md) | **kabul** (K-153/B-061) |
| ADR-051 | [LSP tam-metin invalidation ölçeği](051-lsp-tam-metin-invalidation-olcegi.md) | **kabul** (K-154/B-062) |
| ADR-052 | [Semantic regresyon provenance zinciri](052-semantic-regresyon-provenance.md) | **kabul** (K-155/B-063) |
| ADR-053 | [Compiler semantic değişiklik beyanı](053-compiler-semantic-degisiklik-beyani.md) | **kabul** (K-155A/B-064; ADR-052 kapı revizyonu) |
| ADR-054 | [Strict form URL kodlaması](054-strict-form-url-kodlamasi.md) | **kabul** (K-176/B-065) |
| ADR-055 | [Fuzz korpus öğreniminin kalıcılığı](055-fuzz-korpus-kaliciligi.md) | **kabul** (K-156/B-066) |
| ADR-056 | [Uzun fuzz release-candidate kapısı](056-uzun-fuzz-rc-kapisi.md) | **kabul** (K-157/B-067) |
| ADR-057 | [Seçici düzeyinde çoklu test kapsamı](057-coklu-test-kapsami.md) | **kabul** (K-158/B-068) |
| ADR-058 | [Sürümlü public API facade'ı](058-public-api-facade.md) | **kabul** (K-159/B-069) |
| ADR-059 | [Paket sahipliği ve core freeze](059-paket-sahipligi-ve-core-freeze.md) | **kabul** (K-160/K-160A, B-070/B-071) |
| ADR-060 | [PostgreSQL adaptör sahipliği ve güven sınırı](060-postgresql-adaptor-siniri.md) | **kabul** (K-163/B-073) |
| ADR-061 | [Binary yükleme ve dosya yaşam döngüsü sahipliği](061-binary-yukleme-sahipligi.md) | **kabul** (K-163/F030) |
| ADR-062 | [PostgreSQL TLS ve sınırlı havuz sahipliği](062-postgresql-tls-ve-havuz.md) | **kabul** (K-163/F031) |
| ADR-063 | [Sabit HTTP durumlu yanıt sınırı](063-sabit-http-durumlu-yanit.md) | **kabul** (K-163/F014/F032) |
| ADR-064 | [Yürütülebilir uyumluluk ve deprecation kapısı](064-uyumluluk-kapisi.md) | **kabul** (K-167/B-072) |
| ADR-065 | [Spec maddesi düzeyinde drift kapısı](065-spec-madde-drift-kapisi.md) | **kabul** (K-171) |
| ADR-066 | [Birleşik güvenlik sürüm kapısı](066-birlesik-guvenlik-surum-kapisi.md) | **kabul** (K-170/K-204) |
| ADR-067 | [Yalnız dogfood boşluklarından büyüyen korpus](067-dogfood-korpusu.md) | **kabul** (K-172) |
| ADR-068 | [Tekrar üretilebilir sürüm artefaktı](068-tekrar-uretilebilir-surum.md) | **kabul** (K-168/K-191/K-195/K-196/K-197/K-198/K-199/K-200/K-201/K-202) |
| ADR-069 | [Uzun soak ve kaynak sızıntısı kapısı](069-uzun-soak-kapisi.md) | **kabul** (K-173) |
| ADR-070 | [Tanı kalitesi kapısı](070-tani-kalitesi-kapisi.md) | **kabul** (K-166) |
| ADR-071 | [Kurulum, kaldırma ve sürüm runbook sözleşmesi](071-kurulum-kaldirma-sozlesmesi.md) | **kabul** (K-169) |
| ADR-072 | [İkinci gerçek ürün; elmas birim içe alımı ve birim kökenli tanılar](072-ikinci-urun-ve-birim-kokenli-tanilar.md) | **kabul** (K-164) |
| ADR-073 | [Aynı aracın Zee, Rust ve Go gerçeklemeleriyle karşılaştırma](073-zee-rust-go-karsilastirmasi.md) | **kabul** (K-165) |
| ADR-074 | [Üçüncü ürün: sürtünme tekrarı dil tasarımının sinyalidir](074-ucuncu-urun-ve-surtunme-tekrari.md) | **kabul** (K-179) |
| ADR-075 | [Adil kalıcı yazma kilidi: bekleyen yazar kuyrukta bloklanır](075-adil-kalici-yazma-kilidi.md) | **kabul** (K-183) |
| ADR-076 | [Doğrulanmış oturum bilgisi rota ortamında okunur](076-dogrulanmis-oturum-bilgisi.md) | **kabul** (K-185/GB-026) |
| ADR-077 | [Request-bound PostgreSQL tenant bağlamı](077-request-bound-postgresql-tenant-baglami.md) | **kabul** (K-186/GB-027) |
| ADR-078 | [Migration SQL seviye ayrıştırması](078-migration-sql-seviye-ayristirma.md) | **kabul** (K-187) |
| ADR-079 | [ERP için güvenli oturum listeleme ve tekil iptal](079-erp-oturum-listeleme-ve-iptal.md) | **kabul** (K-188/GB-028) |
| ADR-081 | [Resmî site ve üretim sınırı](081-resmi-site-ve-uretim-siniri.md) | **kabul** (K-203/GB-038) |
| ADR-082 | [Legacy giriş ve kanonik istemci kimliği](082-legacy-giris-ve-kanonik-istemci-kimligi.md) | **kabul** (K-208/K-209) |
| ADR-083 | [Site E dogfood'u: parola özeti, TOTP, SMTP ve görsel küçültme](083-urun-e-kriptografi-iletisim-gorsel.md) | **kabul** |
| ADR-084 | [Rota birimleri: `<ad> rotalarını kullan` (K-211)](084-rota-birimleri.md) | **kabul** |
| ADR-085 | [Görünüm bloğu: tipli HTML ağacı (K-212)](085-gorunum-blogu.md) | **kabul** |
| ADR-086 | [Site derleyici hatlarının birleştirilmesi](086-site-hatlarinin-birlestirilmesi.md) | **kabul** (K-213) |
| ADR-087 | [Açık depoda anonim ürün kanıtı](087-acik-depo-urun-gizliligi.md) | **kabul** (K-214) |
| ADR-088 | [Git2 üzerinde Gitsrv CI](088-git2-gitsrv-kosulari.md) | **kabul** (K-216) |
| ADR-089 | [Süreç başına bellek oran deposu](089-bellek-oran-deposu.md) | **kabul** (K-218) |
| ADR-090 | [Bellek oran ayarı ve korunmuş tahliye](090-web-oran-ayarlari.md) | **kabul** (K-219) |
| ADR-091 | [HTTP POST, başlık ve gizli ortam sınırı](091-http-post-baslik-ve-gizli-ortam.md) | **kabul** (K-220) |
| ADR-092 | [Bilinmeyen CLI seçeneği reddi](092-cli-bilinmeyen-secenek-reddi.md) | **kabul** (K-221) |
| ADR-093 | [Deterministik compiler build kimliği](093-deterministik-build-kimligi.md) | **kabul** (K-226) |
| ADR-094 | [Yazmadan biçim denetimi](094-bicimleyici-yazmadan-denetim.md) | **kabul** (K-229) |
| ADR-095 | [PostgreSQL göç durum/kuru planı](095-postgresql-goc-plani.md) | **kabul** (K-228) |
| ADR-096 | [Ekli e-posta ve MIME sınırı](096-ekli-eposta-ve-mime.md) | **kabul** (K-232) |
| ADR-097 | [Sürümlü runtime ayarları](097-surumlu-runtime-ayarlari.md) | **kabul** (K-234) |
| ADR-098 | [Yerel exact compiler seçici](098-yerel-derleyici-secici.md) | **kabul** (K-233) |
| ADR-099 | [SMTP hata maskelemesi](099-smtp-hata-maskelemesi.md) | **kabul** (K-236/GB-060) |
| ADR-100 | [SMTP yanıt okuma sınırı](100-smtp-yanit-okuma-siniri.md) | **kabul** (K-237/GB-061) |
| ADR-101 | [rustls advisory güvenlik yaması](101-rustls-advisory-yamasi.md) | **kabul** (K-238/GB-062) |
| ADR-102 | [Sürüm klonu ve build kimliği](102-surum-klonu-ve-build-kimligi.md) | **kabul** (K-241/GB-064) |
| ADR-103 | [Native COMMIT belirsizliği kanıtı](103-native-commit-belirsizligi-kaniti.md) | **kabul** (K-242/GB-065) |
| ADR-104 | [Exact kaynak arşivi sınırı](104-exact-kaynak-arsivi-siniri.md) | **kabul** (K-243/GB-066) |
| ADR-105 | [Credential host köprüsü](105-credential-host-koprusu.md) | **kabul, sınırlı POSIX pilot** (K-245/GB-068) |
| ADR-106 | [Worker host yaşam döngüsü](106-worker-host-yasam-dongusu.md) | **kabul, sınırlı POSIX pilot** (K-246/GB-069) |
| ADR-107 | [Kalıcı SQL iş kuyruğu pilotu](107-kalici-sql-is-kuyrugu-pilotu.md) | **kabul, dar SQL prototipi** (K-247/GB-070) |
| ADR-108 | [LSP URI UTF-8 yolu](108-lsp-uri-utf8-yolu.md) | **kabul** (K-248) |
| ADR-109 | [TR–EN plain slot yardımcısı](109-tr-en-plain-slot-yardimcisi.md) | **kabul, dar uygulama yardımcısı** (K-249/GB-071) |
| ADR-110 | [SMTP mutlak süre bütçesi](110-smtp-mutlak-sure-butcesi.md) | **kabul** (K-252/GB-072) |
| ADR-111 | [PostgreSQL statement fault kanıtı](111-postgresql-statement-fault-kaniti.md) | **kabul, dar yerel pilot** (K-250) |
| ADR-112 | [DB göç hazırlık probu](112-db-goc-hazirlik-probu.md) | **kabul, dar yerel pilot** (K-251/GB-073) |
| ADR-113 | [Native Unix SIGTERM drain](113-native-web-sigterm-drain.md) | **kabul, sınırlı Unix yaşam döngüsü** (K-253) |
| ADR-114 | [Ayrı SQL kira heartbeat](114-ayri-sql-kira-heartbeat.md) | **kabul, dar SQL-only pilot** (K-254/GB-075) |
| ADR-115 | [TLS1.1 capable-positive kontrol](115-tls11-capable-positive-kontrol.md) | **kabul, owned fixture** (K-256/GB-076) |
| ADR-116 | [Native sahipli metrik pipe](116-native-owned-metrics-pipe.md) | **kabul, sınırlı metadata profili** (K-257/GB-078) ; K310/GB119 metrics harness cleanup/özgün PG-FD dilimi açık; expected1failure ayrı, kontrollü standalone52/combined16 geçti; source15/clean/native8+distinct nestedlib/final kabulü bekler; P13kısmi |
| ADR-117 | [Sınırlı SQL dispatcher](117-sinirli-sql-dispatcher.md) | **kabul, SQL-only host pilotu** (K-258/GB-079); K309/GB118 exact kaynak d3d6a9044f1646871954e0933fe1fe43ecd5ac96 dar kapsamda kapalı; tam15/temiz source+tekCargo/job11/frozen kabulü ve exact final11/temiz kapanış commit makbuzları zorunlu; genelP09kısmi |
| ADR-118 | [Native listener nesil hazırlığı](118-native-listener-nesil-hazirligi.md) | **kabul, sınırlı Unix host profili** (K-259/GB-080) |
| ADR-119 | [Credential native metrik capability](119-credential-native-metrics-capability.md) | **kabul, tek POSIX metadata capability** (K-260/GB-081) |
| ADR-121 | [Web oran CLI program argümanları](121-web-oran-cli-program-argumanlari.md) | **kabul, exact source/provenance bugfix** (K-262) |
| ADR-122 | [Supervisor sahipli EPERM geçişi](122-supervisor-sahipli-eperm-gecisi.md) | **kabul, dar POSIX host correctness adayı** (K-263/GB-077) |
| ADR-123 | [Kritik işlev seçici taban incelemesi](123-kritik-islev-secici-taban-incelemesi.md) | **kabul, dar bakım incelemesi** (K-264) |
| ADR-124 | [Exact kaynakta dış RC kanıtı](124-exact-kaynak-dis-rc-kaniti.md) | **kabul; cleanS canonicalexport+sameS RC gate geçti; GB082 dar kapalı** (K-265) |
| ADR-125 | [Typed credential native listener bileşimi](125-credential-native-listener-kompozisyonu.md) | **kabul; temiz S typedcredential/listener model5+native5 dar Unix kabulü; fullP25/ürün/platform açık** (K-267/GB-083) |
| ADR-126 | [Sonlu SQL dispatcher timer ve belirsizlik kilidi](126-sonlu-sql-dispatcher-timer.md) | **kabul; S actual ownedLinux timer2, fullP09/ürün açık** (K-268/GB-084) |
| ADR-127 | [Kopyalanabilir site ve panel başlangıcı](127-kopyalanabilir-site-panel-baslangici.md) | **kabul; temiz S site/panel CLI+nativeHTTP dar kabulü; gerçek kullanıcı/fullP23/platform açık** (K-269/GB-085) |
| ADR-128 | [Yerel compiler import ve proje launcher](128-yerel-compiler-import-ve-proje-launcher.md) | **kabul, yerel pilot; publisher/platform/live açık** (K-270/GB-086) |
| ADR-129 | [Owned native web test kiti](129-owned-native-web-test-kiti.md) | **kabul; K-304/GB-113 dar kaynak/native4 kapsamı kapalı; genel P22/ürün/platform açık** (K-271/K-304; son belge kapısı kayıt commit koşulu) |
| ADR-130 | [Sabit TR/EN liste-detay HTTP golden](130-sabit-tr-en-liste-detay-http-golden.md) | **kabul; temiz S Python3.9/3.13 fixed TR/EN HTTPgolden kabulü; ürün/platform/fullP21 açık** (K-272/GB-088) |
| ADR-131 | [Yönetilen proje modül kompozisyonu](131-yonetilen-modul-kompozisyonu.md) | **kabul; temiz S normal Cargo managedscaffold dar kabulü; DB lifecycle/adoption/platform açık** (K-273/GB-089) |
| ADR-132 | [Süreli hazırlık ve kontrollü config yenileme](132-sureli-hazirlik-ve-kontrollu-config-yenileme.md) | **kabul; temiz S readinessPG15+TTL5 dar Unix kabulü; sürekli hizmet/adoption/platform açık** (K-274/GB-090) |
| ADR-133 | [Sabit owned host PDF backend](133-sabit-host-pdf-backend.md) | **kabul; S nativeemitter+hostPDF8, fullP11/ürün açık** (K-275/GB-091) |
| ADR-134 | [Linux özel paket taşıyıcısı](134-linux-ozel-paket-tasiyicisi.md) | **kabul; S ownedLinux delivery/install8/carrier4/container6, GB092 dar kapalı** (K-276/GB-092) |
| ADR-135 | [Sabit native dış etki tüketicisi](135-sabit-native-dis-etki-tuketicisi.md) | **kabul; S ownedloopback11, fullP09/ürün açık** (K-277/GB-093) |
| ADR-136 | [JetBrains yerel LSP şablonu](136-jetbrains-yerel-lsp-sablonu.md) | **a57290f dar IDE işlev kabulü; ayrı shutdown trace yok; popup/UI gecikmesi/Windows/yayın/tam P19 açık** (K-278/K-280/K-297/K-298/GB-094) |
| ADR-137 | [Önceki küçük sürüm korpusu](137-onceki-kucuk-surum-korpusu.md) | **kabul; S Mac/Linux modern49+preflight5, fullP04P05 açık** (K-279/GB-095) |
| ADR-138 | [JetBrains tamamlama ve düzenli kapanış](138-jetbrains-tamamlama-ve-duzenli-kapanis.md) | **kabul; POSIX macOS dar profil, tam P19 açık** (K-280/GB-096) |
| ADR-139 | [K-282 sürekli dispatcher ve application-owned adapter](139-surekli-dispatcher-ve-uygulama-adaptoru.md) | **tarihsel S3 dar yerel kabul; ürün/provider/platform kapsamı açık** (K-282/K-295); K-306/GB-115 dar kapsamda kapalı; tarihsel C12 kaynak hazırlığı ve S13/VM5 tanısı korunur; yeni exact kaynakta failed-only reset düzeltmesi; yeni exact 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa kaynakta2Cargo/dispatcher4+HTTP7; dar kapanış exact son11/temiz commit makbuzlarıyla bağlı; son belge kapısı kayıt koşulu |
| ADR-140 | [K-283 sürekli HTTP user service](140-surekli-http-user-service.md) | **S3 dar yerel kabul; üretim/ürün/platform kapsamı açık** (K-283); K-306/GB-115 dar kapsamda kapalı; tarihsel C12 kaynak hazırlığı ve S13/VM5 tanısı korunur; yeni exact kaynakta failed-only reset düzeltmesi; yeni exact 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa kaynakta2Cargo/dispatcher4+HTTP7; dar kapanış exact son11/temiz commit makbuzlarıyla bağlı; son belge kapısı kayıt koşulu |
| ADR-141 | [Purpose-bound imzalı PDF stdio host (K-281/P11)](141-imzali-pdf-stdio-host.md) | **S3 tarihsel; K-299 dar kaynak/native/arşiv kapsamı kapalı; toplu commit öncesi son belge kapısı gerekli, stdlib9 ve C2 native9 ayrı; ürün/FFI/yayın açık** (K-281/K-299/GB-111) |
| ADR-142 | [Opt-in Zee kaynak karar kapsamı](142-zee-kaynak-dal-kapsami.md) | **S3 dar yerel kabul; genel kapsam/ürün kabulü açık** (K-284) |
| ADR-143 | [Sahipli modül SQL yaşam döngüsü](143-sahipli-modul-sql-yasam-dongusu.md) | **S3 dar yerel kabul; DB+FS atomikliği/ürün kapsamı açık** (K-285); K-307/GB-116 dar kapsamda kapalı; Exact ikinci kaynakta tek ignored Cargo/SQL9; kaynak344 ve özgün foreground PG/outer owner/raw frozen kabulü doğrulandı; diğer dört PG/genel P20 kısmi; son11/tested-overlay temiz kayıt commit makbuzları kapanış koşulu |
| ADR-144 | [dene içinde sabit native web test köprüsü](144-dene-native-web-test-koprusu.md) | **K-305/GB-114 dar negatif-harness ve temiz bridge unit/native4/altı negatif kabulü kapalı; S3 tarihsel, K-304 direct kit ayrı; genel kapsam kısmi** (K-286/K-304/K-305; son 11 belge kapısı kayıt commit önkoşulu) |
| ADR-145 | [Artımlı editör analizi ve lint](145-artimli-editor-analizi-ve-lint.md) | **K-298 stdio p95 hedefi dar gözlemsel kabul; aynı SHA ikinci RSS/RC geçti, dar K-298 kapalı; HIR/type ve ürün açık** (K-287/K-298/GB-103) |
| ADR-147 | [Uygulamaya ait kapalı bileşen ve TR/EN katalog bileşimi](147-composed-tr-en.md) | **S3 dar yerel kabul; gerçek ürün benimsenmesi açık** (K-289) |
| ADR-148 | [İçerik, yönetim ve medya başlangıcı](148-icerik-yonetim-medya-baslangici.md) | **S3 dar yerel kabul; insan/üretim/platform/ürün kapsamı açık** (K-290); K308/GB117 exact kaynak 05e12f748503339ac18e92dabbe498c49437131e dar kapsamda kapalı; tam15/temiz source+tekCargo/native14/frozen9 kabulü ve exact final11/temiz kapanış commit makbuzları zorunlu; P23 kısmi |
| ADR-149 | [K-291 mixed-code revision için ayrı gerçek kanıt](149-mixed-revision-owned-wave.md) | **S3 tarihsel; K-300 dar kaynak/native kabulü ve GB-112 kapalı; hazırlık13, kaynak kapısı13 ve temiz C3 native9 geçti; son belge kapısı takip commit koşulu, P14 kısmi** (K-291/K-300) |
| ADR-150 | [K-292 native worker fault ve kontrollü resolver-call hazırlığı](150-native-worker-fault-ve-kontrollu-resolver.md) | **S3 tarihsel; K-299 dar kaynak/native/arşiv kapsamı kapalı; toplu commit öncesi son belge kapısı gerekli, stdlib10 ve C2 native2 ayrı; native DNS/tam fault açık** (K-292/K-299/GB-111) |
| ADR-152 | [Metric-only stdio alias için erken sessiz ret](152-native-metrik-stdio-sessiz-ret.md) | **S3 dar yerel kabul; K-294/GB-110 dar kapsamda kapalı** (K-294) |
| [080 — Özel Gitsrv CI ve Tier-1 kanıt sınırı](080-tier1-kanit-kaynaklari.md) | kabul | K-217 özel CI ve çapraz/gerçek runtime ayrımı |

| ADR-153 | [LSP sınırında UTF-16 konumları](153-lsp-utf16-konum-siniri.md) | **K-297 exact fixed_by kapalı; a57290f native UTF-16/IDE tazeliği; genel P19 açık** (K-297/K-298) |
