# Ürünleşme — yerel kabul ve açık teknik işler raporu

## K-311 — effect harness lifecycle: açık

K-311 / GB-120 düşük ve açık: effect kabul harness’inin OperationProxy.close yolunda tek socket-close hatası kalan özgün thread ve COMMIT sahibi cleanup çağrılarını atlayabiliyordu. Child-free karşı örnekte1 beklenen failure/exit1/reaped doğrulandı; bu gerçek OS/PG/SMTP sızıntısı veya diskte PASS gözlemi değildir.

Düzeltme resource-free constructor/start öncesi registration, partial allocation retained ownership, kalan cleanup denemeleri/ilk hata kimliği, yapışkan iptal/handler restore ve failed-only publication sağlar. Work180 ve tek cleanup90 mutlak bütçeleri uzatılmaz; H/WNOWAIT özgün sahipliği, foreign kaynaklara dokunmama, normal foreground PG SIGINT0/reaped/noFallback/readiness/SHOW identity ve private-root tutma sınırları korunur. Bu normal PG yollarının gerçekten kabulü gelecekteki exact-source native kampanyasının konusudur.

PG profili explicit plaintext loopback, TLS kapalı/startup196608; TLS6e5 shared helper’a çevrilmez. SQL/SMTP latch, C029/DATA/ackCOMMIT belirsizliği manualpause/no-replay; native11 gözlemi/60need ve14 sabit source-role değişmez. İç helper ayrıca fixedpin ile tüketilir. Production175/compiler/src/H/ürün API/ABI/grammar/performance değişmez. ZEE_EFFECT_NATIVE_OUT yalnız internal ignored native test için fresh absolute output seçimidir; canonical parent/NotFound kontrolü ve default hedef yolu korunur, ürün CLI kapasitesi değildir.

Child-free1/0.003sn expectedFAIL/exit1/reaped; actualpeer812690ff68322169e9fa96770f397a503418f33c5100b803a432e62459cfdcb9. Yalnız stop.set/owned-sockets.close gözlendi; sonraki beş cleanup eylemi atlandı ve ilk hata korundu. Tarihsel ad-hoc UNVERIFIED kaydı geriye dönük kapanmaz.

Eski baseline8 attempt2:8/0.947sn PASS; peerab299b4506236e838d97de88a87271bdae42b005e17c8a1cd2b4430fda371b1f. İlk precommand driver KeyError ayrı başarısız hazırlıktır. Ordinary9 fixture2:9/1.101sn PASS/özgün owner0/reaped/1.1660061669535935sn; peerc3acfa3ac563a1ad8f91bac982bc29ad3805bdeb59e6fa0605792e509da8ec9d. Fixture1 setup/missing-script hatası testlerden önceydi; source kusuru veya testFAIL değildir.

Lifecycle27:27/0.081sn PASS, peer7cadf3e0bfaaf1a2c08c52d36bb906cd47bd2c4cacd0ad81b01b8049ae2b1d85. Aynı lifecycle27+PG18’i seçen preflight45:45/0.153sn PASS/özgün owner0/reaped/0.27195924997795373sn; peerbcf933e37f25472a49d419ab9415a93e0e89fbe0ce796361a1a17765a216876a. Ordinary9 nested45’i yeniden toplamlatmaz;45/27/18 native veya yeni Cargo sahibi değildir.

Standalone PG18:18/0.051sn PASS, peera6268fd4a1e08e23859d97f8eddf3f02c646f55e638c12da9fcd26f4f37cfcf4. Fake command/readiness/SIGINT/FD/uncertainty modeli; gerçek PG/native yürütümü değildir.

Kaynak/test/belge aynı atomik dilimdedir. Son beş kaynak pininin tam envanteri:

| Kaynak/test dosyası | SHA-256 | Bayt |
| --- | --- | ---: |
| `scripts/tests/effect-dispatcher-native-test.py` | `c4a91293f32d8f7463a922f15928c70cd4e058d3e0ca7a922966774f8c1bcf1f` | 46884 |
| `scripts/tests/effect-dispatcher-test.py` | `3a95add8341e83b934874cab497f64589fc2c2ca99debaa2fb4c9fdcfd7cd0f5` | 11846 |
| `scripts/tests/effect-pg-owner.py` | `37e72f521260f5dc039840bd413efa405164d0a86ea5601e4c5abfd820e0611e` | 21768 |
| `scripts/tests/effect-pg-owner-preflight-test.py` | `922d58f149c5bce69d9ef66c6523a61689b0c1c7ba842ae84becbbb826f51cfa` | 41082 |
| `compiler/tests/native_effect_consumer_testi.rs` | `8e8d3d23016dfefe1d108617602ede6fc920ec4d2d4601a5c0c16dbcf0024fa9` | 4482 |

Rust8e8d output tasarımı45fed bağımsız incelemesine bağlıdır; rootaudit5a788 format/check0-empty ve byteexact sonucunu kaydeder, ayrı formatter raw/owner makbuzu sunulmadığından bu sınırlı root attestation’dır. Prospective source union358/175 için k311-source-pins-DRAFT-1 manifesti türetildi; bu taslak pin envanteri source15/temiz kaynak/native/final kabulü değildir ve frozen değildir.14role içindeki docs/native-effect-consumer.md ürün komutları/semantik değişmediği için kaynak-pinli olarak korunur.

Tam kaynak15/temiz kaynak ve bağımsız sourcepeer; aynı SHA’da özgün dış owner ile native11/60need/normalPG/readiness/retained tüm kaynak cleanup/public proof; ardından exact belge overlay’inde final11 ve temiz kapanış kaydı zorunludur. Native/final kabulü açık; K311/GB120 düşük-açık kalır. K306–K310 dar kapanışları kendi exact kaynaklarında, ilk başarısız denemeler ve tarihsel UNVERIFIED sınırlar yerinde kalır. Effect şu an tek açık PG tüketicisidir; ancak dar effect kapanışı sonrasında bu beş-harness PG kuyruğu tamamlanır.39 statü korunur; P09/P13/P20/P23/P36 kısmi, P37 sürekli, GB046/109 açık. Genel P09/ürün/provider/harici proje/insan/platform, RC/soak/IDE ve C1/K288-P33/K293 ret sınırları değişmez.

> **Güncel durum:** K306/GB115,K307/GB116,K308/GB117 ve K309/GB118 dar kapanışları kendi exact kaynaklarında korunur. K310/GB119 dar kapsamda kapalı; Exact kaynak 1130569460203049374e16146a955e208da9dd16 tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta metrics8/29 require ve iki ayrı ignored Cargo sahibi frozen kabulü; özgün PG/readiness/FD/cleanup ve beş direct worker/two alias2 doğrulandı.
> **Kalan yerel adım:** Yalnız gerçek metrics kapanışından sonra effect tek açık PG tüketicisi; diğer teknik/ürün/insan/sağlayıcı/platform/dış girdiler korunur.39iş tamamlanmış sayılmaz.
> **Kapanış kaydı:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

## K-310 — metrics harness cleanup ve özgün PG/FD sahibi: dar kapsamda kapalı

K-310 / GB-119 dar kapsamda kapalı: Metrics native kabul harness cleanup/ilk hata, yapışkan iptal/publication ve özgün PG/worker/FD sahipliği dar dilimi. Exact kaynak 1130569460203049374e16146a955e208da9dd16 tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta metrics8/29 require ve iki ayrı ignored Cargo sahibi frozen kabulü; özgün PG/readiness/FD/cleanup ve beş direct worker/two alias2 doğrulandı. Baseline15, child-free tek beklenen failure ve kontrollü standalone52/combined16 ayrı kapsamda korunur. Yalnız bu dar metrics kabulü gerçekten tamamlandığında effect tek açık PG tüketicisi kalır; genelP13 kısmi ve39plan statüsü mevcut değerlerindedir.

**Gerçek kaynak kabulü:** Exact kaynak 1130569460203049374e16146a955e208da9dd16, tree 98898300e2a5e1ecf8c77c88171dd12c3b69a2e0; tam15 kaynak kapısı 1220.731sn ve temiz tested-source commit/özgün owners doğrulandı. Aynı kaynak koşusunun tam faz998/1023 sonucu0 başarısız/25ignored ve22faz olarak frozen validator ile doğrulandı. K310 source4+OPENdocs11 atomik uygulama zinciriyle bağlıdır. Baseline15, standalone52(genericPG18+metrics34), ordinary16(eski15+tek52wrapper) ve native-driver pure28 ayrı kontrollü hazırlıklardır; Cargo/native toplamına eklenmez. Proposal1→2 yalnız Rust biçim değişikliğidir; Python model baytları değişmedi. Tarihsel ilk kaynak15 koşusu ilk12 komutu geçti; faz998/1023,0 başarısız/25ignored ayrı kayıttadır.13.diff-check native harness line540 dört trailing space nedeniyle exit2 verdi;14.secret-scan/15.security-continuous çalışmadı, özgün dış owner exit1/reaped oldu. Bu başarısız koşu kaynak kabulü değildir. Proposal2→3 yalnız bu dört whitespace baytını kaldırdı; native harness fullAST aynı, diğer iki Python/Rust/production175/shared helper değişmedi. P3 controlled modelleri ve başarılı yeni kaynak15/cleancommit yalnız kendi actual makbuzlarıyla değerlendirilir; önceki başarısız deneme kabul diye taşınmaz. İkinci kaynak15 denemesi ilk13 komutu geçti; phase22/998/1023/0failed/25ignored,clippy ve diff-check0 tarihsel ayrı sonuçlardır.14.secret-scan1 nedeniyle koşu başarısızdı,15.security-continuous çalışmadı; özgün dış owner1/reaped oldu. Public16 dosyalık redakte teşhisteki tek generic-api-key816 literal gerçek scripts/native-metrics.py public SHA256 kimliğiyle eşti, credential değildi. Proposal3→4 yalnız preflight named METRICS_SOURCE_SHA globali ve aynı selected hash verifier referansıdır; literalpublichash, native b5fd, diğer kaynaklar/production175/H/PG6e5 değişmez. P4 standalone52/.395sn ve combined16/.441sn özgün owner0/reaped kontrollü sonuçları actualpeer3 kayıtlarıyla ayrıdır, Cargo/native toplamı değildir. Erken default8.30.1/emptyignore taraması16proposed publicdosyada0bulgu verdi; taranan report öncekiNULLDRAFTnote idi, FILLEDnote/fullR/source15 kabulü değildir. Başarılı freshsource3 yalnız kendi tam15/cleancommit/owner makbuzlarıyla değerlendirilir; başarısız source1/source2 kabul diye taşınmaz.

**Gerçek metrics8 ve distinct nested libpool kabulü:** Yalnız bu exact kaynakta canonical ignored metrics Cargo sahibi ve onun çağırdığı mevcut exact ignored release lib pool Cargo sahibi geçti: iki ayrı Cargo sahibi, sekiz metrics gözlemi/29 require; native sürücü 159.862sn. Kaynak350/production175, consumed15+iki unchanged shared helper, iç11 sahipli komut/20 açık safe log ve public raw envanter frozen/root/peer ile bağlandı. Nested release pool120s ve TLSv1.2/local-loopback trust/sslmode=require+CA profili korundu. Aynı kaynak envanterindeki25ignored içinden bu iki sahibi çalıştı; kalan23 bu kampanyada çalışmadı.

PostgreSQL özgün foreground Popen normal SIGINT/observed_exit=exit=0/reaped/noFallback ve özgün readiness probe kapanışı doğrulandı. Beş doğrudan worker özgün owner ile reaped; alias ve stderr-alias için beklenen exit2 doğrulandı, tüm worker/host için normal0 sonucu türetilmedi. FD/component kapanışı, private root kaldırılması, sticky cancellation ve ayrı tek-seferlik cleanup90 sınırı doğrulandı; toplam600 ve nested120 sınırları genişletilmedi. Private credential/CA/key/DSN/PHC/env/argv bytes veya hash’leri public kanıt değildir. Metrics mevcut public kapsamı beş direct native worker içerir; bunların ikisi alias beklenen ret yoludur; alias retleri dahil bütün worker/host/component için normal0 sonucu varsayılmaz. Özgün owner/reaped/PG/readiness/FD oracleları yalnız gerçek public makbuzlarının kapsamıyla değerlendirilir.

**Tarihsel kontrollü hazırlık:** Gerçek child-free exact finally karşı örneği1test/0.003sn tek beklenen failure/exit1/reaped olarak doğrulandı; static14b25 ve actualpeer8401 kayıtları exacteski8cb/finallyb757 baytlarına bağlıdır. Yalnız owner.close fake olayı görüldü; gerçek OS/FD/PG sızıntısı veya disktePASS gözlenmedi. Mevcut değişmeyen ordinary metrics baseline15 gerçek15test/0.002sn PASS/exit0/reaped/errors[]/cancelled[] verdi. Bu eski baseline kabulüdür; ayrı focused52/combined16 kontrollü yöntemler, kaynak15 veya metrics8/nestedlib native sonucu değildir. Standalone preflight52 yöntemi (korunan genericPG18+metrics34) gerçek0.424sn PASS/exit0/reaped/errors[]/cancelled[] verdi; özgün watcher0.526857375sn, ilk koşu. Bu kontrollü fake-boundary model sonucudur; gerçek PG veya metrics8/nestedlib native kabulü değildir. Birleşik ordinary16 yöntemi (korunan15+tek nested52 preflight wrapper) gerçek0.540sn PASS/exit0/reaped/errors[]/cancelled[] verdi; özgün watcher0.601194875sn, ilk koşu. İçteki52 yeniden toplamlanıp68Cargo/native testi sayılmaz; mevcut Rust ordinary sahibi aynı kalır. Dört kaynak/test dosyası P2 hazırlığında pinlidir: scripts/tests/native-metrics-test.py SHA256=ac4c84f9a2f9948883715f4c96c0bda70f5a02d6194e3bd493e270a66d8159a0 bytes=36531; scripts/tests/native-metrics-model-test.py SHA256=a7233ef5c711fdfb46697a584f663d41473709e311e360ba0c8dc25e99c1d1a3 bytes=5329; scripts/tests/metrics-pg-owner-preflight-test.py SHA256=0db506b67b448b34fcbf990f1fb8c3396e906940e11b65e3f6011abb3183ebcf bytes=52402; compiler/tests/native_metrik_host_testi.rs SHA256=0a8277ceb3908dfd19b50cef7a4140de078adc424c1e33ccd728ce1027f1fc99 bytes=4933. P1 staticpeer e537f73d ve actualmodelpeer 7f418546997cef115a41c75387b4f1957bb432a9c88fd43199029b70e0a56cbc; P2 yalnız Rust biçim delta review dda334664c824bff97336239dee51fb4936bbd91c42927db5090bed3eae1e95c. Kaynak/test/bu OPEN belgeler aynı atomik uygulama dilimidir; bu pinler tek başına kaynak15, temiz commit veya native kabulü değildir. Dört kaynak/test dosyası ve52/16 kontrollü sonuç pinlidir; source pin toplamı yalnız ayrıca doğrulanmış tam manifestten bağlanır, burada tahmin edilmez. Mevcut15 ordinary yöntemi ve mevcut Rust Cargo sahibi korunur; tek yeni nested preflight wrapper ile Rust beklentisi gerçek combined16 sonucuna bağlıdır. Gerçek değişmeyen baseline15/0.002sn PASS/exit0/reaped ayrı kayıttadır; standalone52/combined16 kontrollü kabulü veya native sonucu ile toplanmaz. Python model yöntemleri native vaka veya Cargo sahibi toplamına eklenmez.

Child-free exact eski finally fake owner.close karşı örneği tek beklenen failure/exit1/reaped olarak tarihsel kalır: kalan fake child/FD/PG/proof callback denemeleri atlanmıştı; gerçek OS/FD/PG sızıntısı veya diskte kalmış PASS gözlemi değildi. Değişmeyen ordinary baseline15/.002sn, standalone52=genericPG18+metrics34/.424sn ve combined16=korunan15+tek nested52 wrapper/.540sn ayrı kapsamlarıdır. Bunlar68 Cargo/native test değildir. P2 yalnız Rust biçim değişikliğidir; Python3 P1 ile bitexact, actual model kanıtı aynıdır.

Mevcut metrics8 gözlemi ve29require assertion;200/200/429/404/426/200 response/DB oracleı, actual4lease/5.waiter/timeout/recovery, bounded backpressure/noSIGPIPE, stdout/stderr alias sessiz ret, iki worker rollup ve credential designated metadata kompozisyonu korunur. Metrics TLSv1.2 sunucu/local+loopback-trust HBA/kendi CA ve client sslmode=require profili korunur; job35sn/scopedCOMMIT veya effect plaintext196608 profili taşınmaz. Shared native-pg-owner.py exact6e5f helper değişmez.

Outer mevcut ignored `native_metrik_http_pg_ve_fd_sahipligi` ile nested mevcut ignored `postgresql::havuz::testler::izole_pg_pool_metrikleri_wait_timeout_ve_donus_gercek_sinirlar` aynı actual source üzerinde iki distinct Cargo sahibidir;8gözlem8Cargo testi değildir. Nested release `--lib/--exact/--ignored` seçicisinin120sn bütçesi korunur. Yalnız gerçek aynı-kaynak ignored envanteri25 ve iki sahibin ayrı başarılı sonuçları doğrulandığında2/25 ve kalan23bu kampanyada çalışmadı denebilir; başka kaynak/faz sonuçları toplanmaz. Prebuild/no-run native kabulü değildir.

**Kapanış önkoşulu:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-3/PROOF.json` | `dc35cc4d5631aade4c08d8c6ca8e7008cabc666cd073a841e5d6e210ecb6ae81` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-controller-3.json` | `7b0062fd897f8429077116d769b7346f5a986c47cb85313a58b762b1a9c6ceff` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-commit-1.json` | `25da4ddfd0ca81e82f1c5fd4c7fd69d96ab19f9e7120b4230b7970b4ca81349d` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-commit-controller-1.json` | `47bb522fa812a9077d6a4c3ea99225b16a1a6ed48c049c18c2c3ef641dd26219` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-native-metrics8-1/PROOF.json` | `2021432ed37dac8f01f1a2a5ae9a3e41a7da1ddb623d081c762ed3bb3ef0064b` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-native-metrics8-controller-1.json` | `69ab8a04a3993e61faa63a433c9f36683d996dfa111a0b638f33644cc6be09fc` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-native-metrics8-1/native/proof.json` | `5a5231335ee084f9ea39bc3a32b91027ed50889ff3574b58a16e9f3402fe8e94` |
| root_native_audit | `/private/tmp/zee-k300/K310-NATIVE-ROOT-AUDIT-1.json` | `8fd5d7e89398830f8f374c20c36763ce9f11d85934e8711c824a23903719fef1` |
| native_peer_review | `/private/tmp/zee-k300/K310-NATIVE-PEER-REVIEW-1.json` | `f6992c6a02820299d5f11c495859f1b186c79d6455b8f6ae8652444f54c13c99` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-frozen-acceptance-1.json` | `2874488d10c7b6777d1261345996eb4e5547bc3da911f8dda65413de24ef1820` |
| source_pins | `/private/tmp/zee-k300/k310-source-pins-v3.json` | `8fd3c5f281eb8e7f347d74040a07a3bc642185816458a43eabe5b31dbb8399ba` |
| closure_template | `/private/tmp/zee-k300/K310-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `99540f4b2aefe94f10e5199c690bc1da0aa646971e6d6b5dcf7bdf7d3892a176` |
| closure_template_review | `/private/tmp/zee-k300/K310-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `f44e16a514962ed3b78563ff895cb16dd79d707a0e8172260931138c1a808bd5` |
| K310-METRICS-PROPOSAL2-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-METRICS-PROPOSAL2-INDEPENDENT-STATIC-REVIEW-1.json` | `dda334664c824bff97336239dee51fb4936bbd91c42927db5090bed3eae1e95c` |
| K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-1.json` | `7f418546997cef115a41c75387b4f1957bb432a9c88fd43199029b70e0a56cbc` |
| METRICS-NATIVE-CLEANUP-ACTUAL-COUNTEREXAMPLE-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/METRICS-NATIVE-CLEANUP-ACTUAL-COUNTEREXAMPLE-INDEPENDENT-REVIEW-1.json` | `8401a089c646f72b53c6c591f035b6d69bb7225f389552d59ae8ee1556d22ae1` |
| K310-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `57b58279f1a29c35faa431eb8b797227b5b8b075a0b59fc3dd01f0c0d3d79329` |
| K310-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `a658e5a764a1caf0d2f9c1cb9017f89c23e9d940f1d6753fcd693bf1a4741ffc` |
| K310-METRICS-PROPOSAL3-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-METRICS-PROPOSAL3-INDEPENDENT-STATIC-REVIEW-1.json` | `59e7f4e8570b24c2d42fd05a30baa1c81b0c5319e54a899a9c3a12a0ca1f0399` |
| K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-2.json | `/private/tmp/zee-k300/K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-2.json` | `c9cca289dc45b62ac2e1daed5da9e1d7cf3992346874abb7ce614c868dc6b664` |
| K310-P3-ACTUAL-MODELS-ROOT-REVIEW-1.json | `/private/tmp/zee-k300/K310-P3-ACTUAL-MODELS-ROOT-REVIEW-1.json` | `6f824c817464d9220e85965a8dd552c62c59eeae69202343552d0bcd160935a7` |
| K310-SOURCE15-ATTEMPT1-ROOT-FAILURE-AUDIT-1.json | `/private/tmp/zee-k300/K310-SOURCE15-ATTEMPT1-ROOT-FAILURE-AUDIT-1.json` | `841d1b2eabbe7190eabdb0f58dae1ef05af5fb8e0d4db905edbdaf94f6bcabed` |
| K310-SOURCE15-ATTEMPT1-FAILURE-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-SOURCE15-ATTEMPT1-FAILURE-INDEPENDENT-REVIEW-1.json` | `61e70c39523d33d94a8494b858162ab26015aeae29c1fcc03a3fbbb5cf6f8dab` |
| K310-FORMAT-FOLLOWUP-SOURCE-TOOLS-DOCUMENT-CAS-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-FORMAT-FOLLOWUP-SOURCE-TOOLS-DOCUMENT-CAS-INDEPENDENT-STATIC-REVIEW-1.json` | `45bfbeb76b117f96216497fc06ed1d38c5e4649b33fb7c8a70fe9e84ba1ae430` |
| K310-FORMAT-FOLLOWUP-APPLICATION-1.json | `/private/tmp/zee-k300/K310-FORMAT-FOLLOWUP-APPLICATION-1.json` | `a2c8254541f0dee63b32ecb6b015503a81e2b6f749ff477ad14ab6d1c5e7289d` |
| K310-FORMAT-FOLLOWUP-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-FORMAT-FOLLOWUP-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `f350605fd3de178679675c2dbfb1fc72f3800628752ee930d9396d463449b214` |
| k310-metrics-proposal-4/PREPARATION.json | `/private/tmp/zee-k300/k310-metrics-proposal-4/PREPARATION.json` | `a6ffdebd90d9f2edc7b8b1a16cedf41425cb2f99fb2a197217d36a8f5d739dec` |
| K310-METRICS-PROPOSAL4-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-METRICS-PROPOSAL4-INDEPENDENT-STATIC-REVIEW-1.json` | `5b9576f7a48e1a1adc20129fba7aa491d6ffb625753bc34e2d6cc6320f86fdbf` |
| K310-MODEL-FIXTURE-PREPARATION-3.json | `/private/tmp/zee-k300/K310-MODEL-FIXTURE-PREPARATION-3.json` | `b625c646bc8bdd429d9d06cc1989a58050a78bb0110fbb598526c11347bef57f` |
| K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-3.json | `/private/tmp/zee-k300/K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-3.json` | `2f8e05bcf0c5fed4441cc9700445fa37894d043e79bdf679bdef6b6d923d7b5f` |
| K310-P4-EARLY-REDACTED-SCAN-1.json | `/private/tmp/zee-k300/K310-P4-EARLY-REDACTED-SCAN-1.json` | `e0a985151ee0891f0f4afb5885d77bfd5ffdbc64cf04d02386e01a104b2efd49` |
| K310-SOURCE-ATTEMPT2-PUBLIC-HASH-FOLLOWUP-DOCUMENT-FILLED-PREPARATION-2.json | `/private/tmp/zee-k300/K310-SOURCE-ATTEMPT2-PUBLIC-HASH-FOLLOWUP-DOCUMENT-FILLED-PREPARATION-2.json` | `2edffa553e26b9f71f8cee30691901b40b473374eef72abd499a9db25ae7d137` |
| K310-PUBLIC-HASH-FOLLOWUP-SOURCE-TOOLS-DOCUMENT-CAS-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-PUBLIC-HASH-FOLLOWUP-SOURCE-TOOLS-DOCUMENT-CAS-INDEPENDENT-STATIC-REVIEW-1.json` | `09a117ead29ccd53b4b65bbae0f212800d105bb0e39ee6092abd662e0d916a80` |
| K310-PUBLIC-HASH-FOLLOWUP-APPLICATION-1.json | `/private/tmp/zee-k300/K310-PUBLIC-HASH-FOLLOWUP-APPLICATION-1.json` | `c7040a7157dfe007fa6daef0bbe0d2b4440d8505f3b5b3ce4019407ca70e5a49` |
| K310-PUBLIC-HASH-FOLLOWUP-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-PUBLIC-HASH-FOLLOWUP-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `64698561b962971d15d9755c00c1bacf8d578096ffe670e3fb89aab20944d044` |
| K310-SOURCE15-ATTEMPT2-ROOT-FAILURE-AUDIT-1.json | `/private/tmp/zee-k300/K310-SOURCE15-ATTEMPT2-ROOT-FAILURE-AUDIT-1.json` | `b2bff5608af534f36b29ca865854f57a56abba2cd98b48b7376872f4a262923e` |
| K310-SOURCE15-ATTEMPT2-FAILURE-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-SOURCE15-ATTEMPT2-FAILURE-INDEPENDENT-REVIEW-1.json` | `5ec4d3f3b78e948ae9e061b3802a646be36bbab711d68fa9c50bed2eaaf067e7` |
| K310-EARLY-SOURCE-PREFLIGHT-ROOT-AUDIT-1.json | `/private/tmp/zee-k300/K310-EARLY-SOURCE-PREFLIGHT-ROOT-AUDIT-1.json` | `83fc9f318014ce0c76ee81ee68fc0c7c57f53890f3255e4a151be95b45e662c1` |
| K310-EARLY-SOURCE-PREFLIGHT-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-EARLY-SOURCE-PREFLIGHT-ACTUAL-INDEPENDENT-REVIEW-1.json` | `7b9f7e7d2925eb3e9fc386f33cf00fbf053e9ff3991e47bb4f173dcb4264e478` |
| k310-public-hash-followup-controller-1.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-public-hash-followup-controller-1.json` | `2c5f28d75d75380707d0ac86e789df080d15331178c919fd5b6578916ffea634` |
| k310-source-attempt2-failed-PROOF.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-2/PROOF.json` | `e20d431d6befffe569e3aeb4b14fae41d585eac425c3add8bd95e720d015f0e3` |
| k310-source-gates-controller-2.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-controller-2.json` | `15c379b19dd921174c9bf6a19f60d4dc7ec92902d71b5941363d052aa3958697` |
| k310-early-source-preflight-1-PROOF.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-early-source-preflight-1/PROOF.json` | `d4d62003530b9bf03464e14d306b5f88b7841e175cea56e1f6e6e644c20b476f` |
| k310-early-source-preflight-controller-1.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-early-source-preflight-controller-1.json` | `b9a4cd385609f7fed41325e096f0a80e18d6b2af03fa4a94486fa63d58815d50` |
| native_driver_preparation_review | `/private/tmp/zee-k300/K310-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `a658e5a764a1caf0d2f9c1cb9017f89c23e9d940f1d6753fcd693bf1a4741ffc` |

Bu yalnız dar metrics dilimidir; gerçek kapanıştan sonra effect tek açık PG tüketicisi ve diğer açık teknik işler sürer.39plan statüsü korunur; P09/P13/P20/P23/P36 kısmi/P37sürekli, GB046/109 açık, GB115–118 kendi exact kaynaklarında kapalı geçmişi ve ürün/adoption/provider/fiziksel platform/dış proje/RCsoakIDE/C1K288P33K293 ret sınırları değişmez.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans ve compiler/LSP kalıcı durum değişmez. Ürün komutları/protokol/counter/göç yolu aynı olduğundan docs/native-metrics.md, ADR119/152 ve mevcut kaynak-pinli rehberler bilinçli unchanged tutulur; ADR116 dar harness sınırını taşır. PHC/parola/cookie/request/cert/key/DSN/credential özel baytları veya hash’leri public kayda alınmaz. Yalnız explicit public artifact role envanteri bağlanır; private runtime taranmaz, eski PID/kök/port benimsenmez.


## K-309 — job dispatcher harness cleanup ve özgün PG sahibi: dar kapsamda kapalı

K-309 / GB-118 dar kapsamda kapalı: Job SQL dispatcher native kabul harness cleanup/sticky iptal/publication ve özgün foreground PG sahipliği dar dilimi. Exact kaynak d3d6a9044f1646871954e0933fe1fe43ecd5ac96 tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/job11 frozen kabulü; özgün PG/readiness/cleanup ve yalnız ilk vaka iki host0 gözlemi doğrulandı. Baseline2, child-free tek beklenen failure ve kontrollü standalone44/ordinary3 ayrı kapsamda korunur. Diğer metrics/effect iki PG tüketicisi açık; genel P09 kısmi ve39plan statüsü mevcut değerlerindedir.

**Gerçek kaynak kabulü:** Exact kaynak d3d6a9044f1646871954e0933fe1fe43ecd5ac96, tree 1f055acaa194c9eb47c51f25c74f85cf8803b273; tam15 kaynak kapısı 1248.429sn ve temiz tested-source commit/özgün owners doğrulandı. Tam faz998/1023 geçti,0 başarısız,25ignored kendi kaynak koşusudur. K309 source4+OPENdocs11 gerçek atomik uygulama ve bağımsız makbuzlarıyla bağlıdır. Standalone44(genericPG18+job26), ordinary3(oldmask2+tek44wrapper), baseline2 ve native-driver pure15 hazırlığı ayrı kapsamlardır; Cargo/native toplamına eklenmez. İlk combined3 fixture1 eksik üç public import bağımlılığı nedeniyle2maskfailure/exit1/reaped kaldı; fixture2 kopyaları tamamlandı, Proposal3 baytları değişmedi; kaynak/native kusuru olarak gösterilmez.

**Gerçek native11 kabulü:** Yalnız bu exact kaynakta canonical tek ignored Cargo sahibi/mevcut job11 geçti; native sürücü 111.174sn. Kaynak348/production175,11case/32assertion/35s handler ve bağımsız heartbeat, TLSrequire ile scoped plaintextCOMMIT/no-replay ayrımı, fixture20 ve dokuz safe tool/raw kaydı frozen/root/peer ile bağlandı. PHC bahanesiyle gizli/atlanan komut boşluğu kabul edilmez; kalan24 ignored Cargo vakası bu kampanyada çalışmadı.

PostgreSQL özgün foreground Popen normal SIGINT/observed_exit=exit=0/reaped/noFallback ve readiness özgün probe kapanışı doğrulandı; private runtime kaldırıldı, özgün dış owner0/reaped/errors[]/cancelled[] ve sahipli cleanup sınırları korundu. Host normal0/reaped/stderr0 makbuzları yalnız ilk vakadaki iki canonical-host-actor gözlemidir; tüm worker/host/component için bağımsız normal0 sonucu değildir. Public host0/reaped/stderr0 makbuzları yalnız ilk vakadaki iki canonical-host-actor gözlemini kapsar; tüm worker/host/component için bağımsız normal0 sonucu iddia edilmez. Diğer özgün sahipler için kapanış/reaped oracleları kendi kapsamındadır.

**Tarihsel kontrollü hazırlık:** Proposal3 dört harness/test kaynağı bağımsız static incelemede doğrulandı: korumalı handler kurulumu/restore, sticky iptal/ilk hata, bağımsız cleanup ve publication, özgün foreground PG/readiness adapter’ı. Mevcut11native vaka/32require assertion/35s heartbeat döngüsü ve TLSrequire/scopedCOMMIT ayrımı korundu. Bu kaynak/test/OPEN belge dilimi aynı atomik uygulama kapsamındadır; tam kaynak veya native kabul sonucu değildir. Gerçek standalone preflight44 (genericPG18+job-specific26) PASS/exit0/reaped, unittest0.331sn; yalnız inert sahiplik/cleanup/iptal/publication modelleridir, gerçek PG/native11 değildir. Değişmeyen mask baseline2/.099sn ayrı tarihsel kabul olarak korunur. Gerçek fixture2 combined ordinary3 PASS/exit0/reaped, unittest0.511sn: eski mask2+tek nested44 wrapper. İlk fixture1 koşusu3test/2maskfailure/exit1/reaped kaldı; root üç public production import bağımlılığını fixture’a ekledi, Proposal3 baytları değişmedi. Bu eksik fixture hazırlığı kaynak/native kusuru veya OS/PG sızıntısı değildir.44/3 sayıları toplanıp Cargo/native vaka yapılmaz. Exact Proposal3 `/private/tmp/zee-k300/k309-job-proposal-3/PREPARATION.json` SHA256 `2096a3744a2989830b6833748b7382dfd9f290a77fcd793f0139a2df250e6cbe`; static peer `29eed668c09d318f258d78a167df9525c6cc50a9af41161465ac475d91ee3070`; actual controlled peer `22ee3e73afa325c3aa2b5caacad725ac32d5f859ef42b6cf0c70029296268559`; fixture2 `a7c866f4914d02db9b17b4b118a66bee6e9d78c48ac4b16844c84c3b33b8e309`. Dört source pini: `scripts/tests/job-dispatcher-native-test.py`=`ef6a3ab8f990c3148a09d3234fe8b8075b9fd12083742059139df2599be5f49d`, `scripts/tests/job-dispatcher-mask-test.py`=`92357774ae9a09d55a366d844a33711564df5a924c237bdb9051135529a4b21a`, `scripts/tests/job-pg-owner-preflight-test.py`=`158af9ca7f2a97d4499c36c3af5ed5befc73380195da98bdf30d1a76b80d630f`, `compiler/tests/job_dispatcher_testi.rs`=`bda79d9307573127ee62eff6931b5dad363bbf4f2b6c3c73052d00c5f8d5a3d1`. Baseline/odaklı saf modeller/ordinary wrapper/native11 ayrı kanıt kapsamlarıdır; Python yöntemleri Cargo veya native vaka toplamına eklenmez.

Child-free exact AST fake probe.close karşı örneği tek beklenen failure/exit1/reaped olarak tarihsel kalır: cluster.close ve proof.write callback denemeleri atlanmıştı; gerçek OS/PG sızıntısı veya diskte kalmış PASS gözlemi değildi. Baseline mask2 ayrı kabul olarak korunur.

Mevcut11 case kimliği/32require/35s sabit handler, en az6bağımsız heartbeat ve eski30s süresinin aşılması korunur. CLI TLSrequire/ownedCA sınırı ile yalnız owned loopback host-component COMMIT fault fixture’ındaki plaintext profil ayrıdır; unknown COMMIT/sinyal/timeout replay/fail yazımı üretmez. Fixture20 ve privacy-safe public tool kaydı sırası/tam envanter kontrolü ayrı exact source/native sözleşmesine bağlıdır; PHC gizliliği gerekçesiyle gizli/atlanan komut boşluğu veya ham credential verisi kabul edilmez.

Yalnız bu exact kaynakta mevcut tek ignored Cargo sahibi/job11 sonucu kullanılabilir; kalan24 ignored Cargo vakası bu kampanyada çalışmış sayılmaz. K306 dispatcher4/HTTP7, K307 SQL9, K308 starter14 ve tarihsel job11 sonuçları kendi kaynaklarında kalır.

**Kapanış önkoşulu:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-gates-1/PROOF.json` | `0391ffe6ed48eb5cfa4bb87cb4eb3520a4cdb0b4fdb40bfc43cdeacf95a71d3f` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-gates-controller-1.json` | `2b719cbf8871dd4adbaf6f68256718b985843334b7368ea6ee277b8159aeeec6` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-commit-1.json` | `d6a0fc6a1dbf6011901434eedc7f375e470bf5ecb5033f5222f2cd558f39d8b9` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-commit-controller-1.json` | `9b85a6a205d8aa03d5b566b0f787fdb5365dda87bd9f42819a5c0b626e932614` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-1/PROOF.json` | `c1a13da0dee5cdd49b1924ae5f85d47fca56c59bed26eac5a884b68e53a01e44` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-controller-1.json` | `39b250307822f60c7a22a030862201e541fc3fec9ec3e66e45e861fbdf00cc4f` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-1/native/proof.json` | `4892c0280317d206eac97ad2abe1491ecaa05cdd2bfc8875fe2bffcecaf387a0` |
| root_native_audit | `/private/tmp/zee-k300/K309-NATIVE-ROOT-AUDIT-1.json` | `4cfeb107b94d7cc62b877c74cffd622f13624d67795dda45bae4db95e112efe5` |
| native_peer_review | `/private/tmp/zee-k300/K309-NATIVE-PEER-REVIEW-1.json` | `f03d38cccc6bdb4e3b6446ef7818a03c484bc6ad9045a77da0d7f90a9b5b2dd1` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-frozen-acceptance-1.json` | `957b1bc09c54be91331d5eab10233f3fd3b410564e17a293cc3d4ab268a46714` |
| source_pins | `/private/tmp/zee-k300/k309-source-pins.json` | `e772138605af3354e93791740260897bad62a8d0ce13c7790101f2674eaf5f0c` |
| closure_template | `/private/tmp/zee-k300/K309-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `534d93dc60ae2739b4746ff1a08a3c7c800f9d32c8056438890ea1f8367dbe43` |
| closure_template_review | `/private/tmp/zee-k300/K309-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `72feb8424e1a7dac1a316585264da93cf8cb15f93fe424e2e0291b94b3fe34c2` |
| K309-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K309-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `eea7ebd5b15cee1dea16846f2ffacc5a7220e5d56a3bd817c75e18a29e670653` |
| K309-JOB-PROPOSAL3-ACTUAL-MODELS-INDEPENDENT-REVIEW-2.json | `/private/tmp/zee-k300/K309-JOB-PROPOSAL3-ACTUAL-MODELS-INDEPENDENT-REVIEW-2.json` | `22ee3e73afa325c3aa2b5caacad725ac32d5f859ef42b6cf0c70029296268559` |
| K309-NATIVE-JOB11-DRIVER-INDEPENDENT-STATIC-REVIEW-3.json | `/private/tmp/zee-k300/K309-NATIVE-JOB11-DRIVER-INDEPENDENT-STATIC-REVIEW-3.json` | `7af1483c3d4a857c4d021d791f8ab6c50bfb867edc0fb4776e307a32ae7fd985` |
| native_driver_preparation_review | `/private/tmp/zee-k300/K309-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `9a9f30bd151ae85feafb77688d656a56ed6cc656fd992c06018a6554f1926dcf` |

Bu yalnız job diliminin dar kapanışıdır; metrics/effect iki PG tüketicisi açık kalır.39plan statüsü korunur; P09/P13/P20/P23/P36 kısmi/P37sürekli ve ürün/adoption/provider/fiziksel platform/dış proje/RCsoakIDE/C1K288P33K293 ret sınırları değişmez.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans ve compiler/LSP kalıcı durum değişmez. Ürün komutları/göç yolu değişmediğinden `docs/job-dispatcher.md`, ADR-114/126/139 ve mevcut kaynak-pinli rehberler bilinçli unchanged tutulur; ADR-117 dar harness güvenlik sınırını ve mevcut SQL-only pilot durumunu taşır. PHC/parola/cookie/request/cert/key/DSN/credential özel baytları veya hash’leri public kayda alınmaz; private runtime veya eski PID/kök aranmaz ve benimsenmez.



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
düzeltmesidir; Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı. Kalan beş PG tüketicisi, fiziksel platform
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

## K-308 — starter harness iptali ve özgün PG sahibi: dar kapsamda kapalı

K-308 / GB-117 dar kapsamda kapalı: Starter kabul harness sticky iptal/publication ve özgün foreground PG sahipliği dar dilimi. Exact kaynak 05e12f748503339ac18e92dabbe498c49437131e tam15/temiz commit makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/starter14 frozen9 ve özgün PG/readiness/host/dış owner kabulü doğrulandı. Diğer üç PG tüketicisi açık; P23 kısmi ve 39 plan statüsü kendi mevcut değerlerinde korunur.

**Gerçek kaynak kabulü:** Exact kaynak 05e12f748503339ac18e92dabbe498c49437131e, tree b7f024e38f43bfb92714aa8db210ccc32bf79d49; tam15 kaynak kapısı 1146.264sn ve temiz tested-source commit/özgün owners doğrulandı. Faz998/1023 geçti,0 başarısız,25ignored kendi tam kaynak koşusudur. İlk app1 değişiklik/makbuz öncesi manifest-local-shadow TypeError/exit1/reaped olarak korundu; üç localname düzeltmesinden sonra app2 source4+OPENdocs11 gerçek uygulandı. FakeFD16, native sürücü pure18 ve integration35 ayrı hazırlık sonuçlarıdır; native/Cargo toplamına eklenmez.

**Gerçek native14 kabulü:** Yalnız bu exact kaynakta canonical tek ignored Cargo sahibi/mevcut starter14 geçti; native sürücü 67.326sn. Kaynak346/production175, özgün native14 oracleları, typed release build ve exact tools/runtime pinleri frozen9/root/peer public raw kanıtlarıyla doğrulandı; kalan24 ignored Cargo vakası bu kampanyada çalışmadı.

PostgreSQL özgün foreground Popen normal SIGINT/observed_exit=exit=0/reaped/noFallback, readiness özgün probe kapanışı ve ayrı host front_terminal exit0/reaped/stderr0 doğrulandı; private runtime kaldırıldı. Ayrı dış original owner0/reaped/errors[]/cancelled[]; iç nativechild normal0 bağımsız makbuzu iddia edilmez. Host’un iç native child cleanup’ı mevcut kaynakta H.stop_owned sınırıdır; bağımsız per-worker normal0 public makbuzu veya yeni worker hook kabulü iddia edilmez. Mevcut starter14 SQL/HTTPS/kimlik/medya/pending oracleları korunur. Tarihsel S3 native14, K307 SQL9 ve K306 dispatcher4/HTTP7 kendi exact kaynaklarında kalır. Yalnız bu yeni exact kaynakta bir ignored Cargo sahibi/starter14 native kabulü kullanılabilir; kalan24 ignored Cargo vakası bu kampanyada çalışmış sayılmaz.

**Tarihsel kontrollü kanıt:** Proposal5 dört test-harness kaynak dosyası bağımsız statik incelemede doğrulandı: sticky ledger/korumalı handler/publication, foreground StarterPgCampaign ve private-root cleanup; native14 kayıt adları/eski assertion çokluğu/özgün host Popen ve ordinary8 korundu. Bu kaynak/test/belge dilimi aynı atomik uygulama kapsamındadır; temiz kaynak veya native kabul sonucu değildir. Gerçek P5 focused18 PASS/exit0/reaped, unittest0.153sn; ayrı PG preflight23 PASS/exit0/reaped (0.224sn). PG23 helper/consumer/preflight baytları P4=P5 olduğundan özgün ilk makbuz korunur, tekrar koşulmuş sayılmaz. İlk focused18 ve combined27 koşuları birer selector TypeError/exit1/reaped olarak tarihsel kalır; bu model seçimi hatası native/runtime/OS sızıntısı değildir. Gerçek P5 fixture2 birleşik ordinary27 PASS/exit0/reaped, unittest0.499sn; yapı old8+focused18+tek nested PG23 wrapper’dır. Public fixture10 kaynak kopyası exact doğrulandı. Üç kabul özgün dış owner errors[]/cancelled[] makbuzlarına bağlıdır;18/23/27 sonuçları toplanıp Cargo/native vaka veya mevcut14’ün yeniden kabulü sayılmaz. Exact Proposal5 `/private/tmp/zee-k300/k308-starter-proposal-5/PREPARATION.json` SHA-256 `cddda1f8bbcd3973fb045fa974c7a6aa927f3e485f2416d2882272f29b585b33`; bağımsız static peer `75b005a3e0e01ad4a073970a49643fc2b66e8f2cce970669cc33448bb48522f8`; actual-results peer `836b53ceb61ec95286b37e358c7d94de9ebf3be754e8ddd24ef73902c260aa4f`; selector düzeltme kaydı `6a9511ec68a6e9f8f702bb7bb865e443cdf22212e2c99a60ebe3252101c68589`. Dört source pini: `scripts/tests/starter-content-native.py`=`3dcb1da978a82ec9e2710d0d6de5e432a5c5fc2f4ce385c0ad79736fe3cbab50`, `scripts/tests/starter-content-test.py`=`93e7e23625eaf6a2732bcc99bdd86506e5e56a4d4238ba332d242e672cf1c80a`, `scripts/tests/starter-pg-owner-preflight-test.py`=`fd8c3d2b7e7c40feed710c93e410e3ce64f97535cf9a4213f43785acd70e4abe`, `compiler/tests/starter_content_testi.rs`=`d3471b0d129816c1e9c941409796172bb156a03fb905f53f979e9bb8d760d9d4`. Bu kayıtlar actual kaynak/test baytları ve bağımsız root/peer inceleme/özgün owner makbuzlarıyla bağlıdır. Ordinary baseline8, odaklı inert yöntemler ve birleşik ordinary wrapper sonucu ayrı kapsamdır; model sayıları Cargo/native sayısına eklenmez.

Shared `native-pg-owner.py` exact6e5f korunur. `ZEE_STARTER_NATIVE_OUT` yalnız internal ignored testin taze mutlak output seçimi olup ürün CLI/API kapasitesi açmaz;0700 geçici-root varsayılanı korunur.

PHC/parola/cookie/request/cert/key/DSN özel baytları veya bunların hash’leri public kanıta yazılmaz. Public proof/progress/loglar yalnız izinli yaşam döngüsü/sonuç metadata’sıdır; eski private runtime/kök/PID/credential aranmaz veya benimsenmez. Mevcut route/PHC/session/CSRF/media güvenlik oracleları dar test-harness düzeltmesiyle gevşetilmez.

**Kapanış önkoşulu:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-gates-1/PROOF.json` | `4ce1698419f7d3aecdc2e13cbfdefaa159689e2d42e3acd79cdafae2e18471d8` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-gates-controller-1.json` | `a6f7950a3753c76839caaa818a9c9bcdda06101f65c3d33929ae89d9af732bb7` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-commit-1.json` | `d68bd9fd28a9bacafcdd5955b8c292ec99b3f0bc8b83c632de44c4c1e59abe29` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-commit-controller-1.json` | `df9be65ff3cd523f08a93617a973c679f1a4746be8e8489146951eb9b681bb97` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-1/PROOF.json` | `feb09e70e89b9946e30331c7ae3b044da0c77f9c5f48ce05c1ff0c18b376d558` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-controller-1.json` | `08471af35cade3240e94f09028f37ea98b7f4389dc29afc973b77ca49fa41138` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-1/native/proof.json` | `1b67d601d6f3fa2eb54c78a89c6041b2e7f8cf1ee5d5ac0629ebc7bbba1ca8d9` |
| root_native_audit | `/private/tmp/zee-k300/K308-NATIVE-ROOT-AUDIT-1.json` | `0fe921fc2c1994bae56773a1185a4fa6ca961a966ca502afc07e9de46d183b7b` |
| native_peer_review | `/private/tmp/zee-k300/K308-NATIVE-PEER-REVIEW-1.json` | `2acfbd8dba35ea13371407b288078f3df5c3277f33c83d11119b2dab2852c856` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-frozen-acceptance-1.json` | `9e7538a35c86a142bfe7de6892a490994c748a908e9ba3094967ba0ca4b5a1a4` |
| source_pins | `/private/tmp/zee-k300/k308-source-pins.json` | `2808178e2ebe9af8af556a7b3fc685d3005d23dd0eab763126dedeff2e762180` |
| closure_template | `/private/tmp/zee-k300/K308-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `e29d24881a61b806bb46eb16456d928c6cedaa9a293251fee8526e99d3595c08` |
| closure_template_review | `/private/tmp/zee-k300/K308-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `e97aca7c28c26b83623c12b714d575b7e5f681c465762982bbed39ff1d67cca8` |
| K308-ATOMIC-APPLICATION2-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-ATOMIC-APPLICATION2-ACTUAL-INDEPENDENT-REVIEW-1.json` | `f4668a08a0614528d8add1ed04570f76d7864b3ec82d3f6acd49f0f14306a2e1` |
| K308-CONTROLLED-RESULTS-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-CONTROLLED-RESULTS-INDEPENDENT-REVIEW-1.json` | `836b53ceb61ec95286b37e358c7d94de9ebf3be754e8ddd24ef73902c260aa4f` |
| K308-ATOMIC-LOCAL-SHADOW-CORRECTION-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-ATOMIC-LOCAL-SHADOW-CORRECTION-INDEPENDENT-REVIEW-1.json` | `81641b09dd35f20c1017cadadea211abac7a6a06402e026cd820a029c706e784` |
| native_driver_preparation_review | `/private/tmp/zee-k300/K308-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `94dba2a083f1f565de1ce69c83a683ec0d5027466be7faaa74b25628e0e7a328` |

Diğer üç PG tüketicisi, P20/P23 kısmi ve diğer açık ürün/insan/platform/dış girdiler, RC/soak/IDE ve C1/K288/P33/K293 ret sınırları korunur; 39 plan statüsü değişmez. Yeni compiler özelliği yoktur.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans iş yükü ve compiler/LSP kalıcı durum değişmez. Native tüketilen `docs/versioned/0.8/icerik-paneli.md` ürün komutları değişmediğinden bilinçli unchanged tutulur; kapanışta source-pinli rehber baytları değiştirilerek native kanıtı bayatlatılmaz. ADR-148 ve bu dar durum kayıtları atomik belge kapsamını taşır.

## K-307 — SQL kabul harness’i ve foreground sahibi: dar kapsamda kapalı

Fresh exact ikinci kaynakta mevcut tek ignored Cargo sahibi1 başarılı/0 başarısız/0 ignored ve dokuz SQL native vaka geçti; frozen kaynak/native/özgün owner/raw zinciri bağımsız doğrulandı.

Tarihsel ilk sticky dilimde baseline10, ayrı inert13 ve birleşik23 geçti; exact AST fakeTERM/fakecluster.close karşı örneği tek beklenen failure/exit1/reaped kaldı. Gerçek OS/PG sızıntısı gözlemi değildir. Foreground takip2 standalone23 ve birleşik ordinary24 de kendi özgün owner0/reaped/errors[]/cancelled[] makbuzlarıyla geçti. Birleşik24 önceki23+tek nested foreground23 hazırlık yöntemidir;47, yeni Cargo sahibi veya native vaka toplamı değildir.

**İlk kaynak diliminin gerçek ayrı kanıtı:** Tam15 kaynak kapısı ve tam faz998/0/25 geçti; temiz kaynak `216d7e1a2d3012d8bef7f55030d42d83045b3fff`, tree `002ca4938767884b12b0cb395e94142059eedfbd`. Kaynakgate1/commit1 ve özgün owner makbuzları kendi kaynaklarında korunur; bu ilk kaynak kabulü SQL foreground takip2 native9 veya ikinci kaynak kapısı yerine geçmez.

Beş test-harness kaynağı SQL tüketicisini `OwnedLocalCluster` foreground özgün Popen sahibi ve `SqlPgCampaign` adapter’ına bağlar. Ayrı yapışkan iptal ledger’ı, cleanup sırasında H maskesi/garantili restore, korumalı handler kurulumu, tüm handler restore, ExitStack sonrası passed ve publication iptal/deadline/failure kontrolleri korunur. Cleanup90 kabul600 sınırını uzatmaz. Readiness/data-directory/version pinleri ve özgün probe/PG nesneleri korunur; belirsiz sahip veya private root kimliği değişiminde kök silinmez ve sonradan benimsenmez. Dokuz SQL oracle, pending/unknown no-replay ve DB+FS atomik değil sınırı aynı kalır.

`ZEE_MODULE_SQL_NATIVE_OUT` yalnız internal ignored testin taze mutlak çıktı yolu seçimidir; yokluğunda mevcut0700 geçici dizin varsayılanı korunur. Ürün CLI/API kapasitesi açılmaz.

**Yeni exact ikinci kaynak:** `9df1b4abb52372dce86c92255392032b6fa08574`; tree `e7de0cfc2dce9e9c271979cc963c9629c49263e8`. Yeni exact ikinci kaynak tam15 kapısı geçti (1118.889 sn); gerçek tam faz998/1023 geçti,0 başarısız,25 ignored. İlk216d7 kaynak15 sonucu bu sonuca eklenmez. Temiz ikinci kaynak 9df1b4abb52372dce86c92255392032b6fa08574, tree e7de0cfc2dce9e9c271979cc963c9629c49263e8; sourcegate2 tested17 overlay ve sourcecommit2 özgün owner makbuzları eşleşti; kaynak344/production175 korundu.

**Fresh SQL native kabulü:** Fresh exact ikinci kaynakta mevcut tek ignored Cargo sahibi1 başarılı/0 başarısız/0 ignored ve dokuz SQL native vaka geçti; frozen kaynak/native/özgün owner/raw zinciri bağımsız doğrulandı. SQL/HTTP/DDL/down-up/pending/unknown no-replay oracleları değişmedi. İki genel şema, parametreli CRUD/hostile veri, bağımsız readonly gözlem, HTTP/host kilidi, gerçek down-archive-up, kalıcı pending/reconcile ve collision rollback dokuz mevcut vakada doğrulandı; DB+FS atomikliği iddiası yok. Gerçek Rust sürüm günlüğü macOS aarch64-apple-darwin/rustc1.93.1; aynı foreground PG16.15 profili doğrulandı. Aynı kaynak release ikili/typed build JSON ve explicit Cargo/Rust/Python/git4 tool pini, public PG/runtime17 pini başlangıç/son denetiminde korundu. Foreground PostgreSQL aynı özgün Popen nesnesiyle normal SIGINT/observed_exit=exit=0/reaped; readiness kimliği ve tüm özgün probe sahipleri kapandı, fallback yok, private kök kaldırıldı. Native sürücü 60.761 sn, harness 3.672 sn; özgün dış owner0/reaped/errors[]/cancelled[]. Root native audit ve bağımsız peer issues=[]; frozen exact9 öncül kayıt ve sıralı public artifact seti gerçek hash/baytlarla eşleşti.

Yalnız yeni exact ikinci kaynakta mevcut tek ignored Cargo sahibi/SQL9 dar kabulü sayılabilir; diğer24/25 ignored Cargo bu kampanyada çalışmadı. Python kontrollü yöntemler ve farklı SHA’ların sonuçları Rust/native toplamına eklenmez. Diğer dört PG tüketicisi açık; SQL dar diliminin kabulü tüm beş PG sahibinin kapanışı değildir. P20 kısmi ve39 plan statüsü korunur; ürün/adoption/platform ve C1/K288/P33/K293 retleri açık kalır. RC/soak/IDE/diğer fiziksel platform/ürün sonuçları bu kaynağa aktarılmaz.

Bilinçli etki incelemesi: production Rust175/bağımlılık/katman, grammar/API/ABI, normatif spec/RFC, kullanıcı tanısı/katalog kodu, sabit performans iş yükü veya compiler/LSP kalıcı durum değişmez.

**Kapanışın koşulu:** K307/GB116 dar kapanışı yalnız exact belge overlay son11 kapısı ve sonraki tested-overlay temiz kayıt commit’i ile iki özgün owner makbuzuna bağlıdır; bu metin kendi başına kapı/commit kanıtı değildir. Dar kapanış yalnız bu exact belge overlay’inin son11 kapısı ve o test edilmiş overlay’i kaydeden sonraki temiz commit/özgün owner makbuzlarıyla geçerlidir. Bu metin kendi commit SHA’sını üretmez veya kendi başına kapı/commit kanıtı sayılmaz.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-2/PROOF.json` | `374b2c08411e4a56950b03c8f927d3c8002743920bfb0862aa1776de84af5a86` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-controller-2.json` | `9656b67c69241742721c657f2a1ed0c92d0ba7604057b896245293b97e4f4251` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-2.json` | `877b4ba036b9d35a4f0275b2709925e4a6841682e2bd7c6c67087835435eb226` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-controller-2.json` | `e647bf72ae26de16717a6b8aa5d1c71ff56e4282c3aa8c4da11d19c3c5364584` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-1/PROOF.json` | `b62fd8cf03bfd4d14fb0453e06ef1a1f0c83abddbf94464928acb0b34b13b98b` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-controller-1.json` | `4496e9ea5bf4812782d756f7f5ce80c86944c7c44a84b97f08879a0e7a62c012` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-1/native/proof.json` | `3dac8e6964993a3b4dcbe8ef36e6aae8ad3832be2e7424b735a143898abece32` |
| root_native_audit | `/private/tmp/zee-k300/K307-NATIVE-ROOT-AUDIT-1.json` | `de1632e288fa29940c29ad3e9bd785ad9013abf44f511448dd5aa31605a263cd` |
| native_peer_review | `/private/tmp/zee-k300/K307-NATIVE-PEER-REVIEW-1.json` | `1020a23986c7a998acedaf99d0efa6289bc0a395b8fa7ce81e00f019ff5ff961` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-frozen-acceptance-1.json` | `e542370e8d951ec0921e268412aeee459eca371645d4d69bca4ed9819293fb1a` |
| source_pins | `/private/tmp/zee-k300/k307-source-followup-pins.json` | `42cab50fc8be5935dc2ac71c115b7e7ef8a011dc5f67887543a6881ca6c97aad` |
| template_record_application_ready_documents | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-DOCS-APPLICATION-READY-1.json` | `ba40001b8cd92e20a2026c980b080fc74d0a995c12187148263a1720b4a65b55` |
| template_record_application17 | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-APPLICATION-1.json` | `8cee65e19ae75e8057755690ed9b58178818fc7e52381c51378de1c93c1abcde` |
| template_record_application_independent | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-APPLICATION-INDEPENDENT-REVIEW-1.json` | `8c353ca282c03e5c1af32d0718d8b685390a0d5a70139901fd11c70934370628` |
| template_record_source1_commit_root_audit | `/private/tmp/zee-k300/K307-SOURCE-COMMIT-ROOT-AUDIT-1.json` | `f8397d4f59a1c72f1d963b0bd3b7d1e0cf4f1e6eb7ac0e8f0df73ceac38971fb` |
| template_record_source1_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-1.json` | `7cd3434d0de144d09249128262025ad082a0c3d46228b175fc64c0ba597a5b86` |
| template_record_source1_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-1/PROOF.json` | `1db8c354449b0d96fa0209afd3b8761f00b142c31b6f082b6c86e1ec73d266a1` |
| template_record_first_source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-controller-1.json` | `6536aa77e7b1f65a2a817de4ef61761c270f5149b6ff3b9c3a0ef4de298205c9` |
| template_record_first_source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-controller-1.json` | `20e9306f6251a5a938a2990002877ee12f0bf8b20024e84087f06b5bdc4a35a6` |

**Kalan yerel işler:** Backlog bağımlılık sırasındaki diğer dört PG tüketicisinin özgün foreground sahiplik/iptal kabulü ve ayrı açık teknik işler sürer; genel P20, ürün/adoption, fiziksel platform ve dış girdiler açık kalır.

### Takip2 public hazırlık kayıtları

| Kayıt | SHA-256 |
| --- | --- |
| `/private/tmp/zee-k300/K307-OPEN-DOCS-PROPOSAL-4.json` | `3eb526fdc0c40865ff3d18261c933a8805a08ddf5dbfd604a74f886ae148ffe3` |
| `/private/tmp/zee-k300/k307-foreground-proposal-2/PREPARATION.json` | `de3af343ed5332801580895e4c65f40d0e15a1ac58ddbd4d0a7c30844d2eb007` |
| `/private/tmp/zee-k300/K307-FOREGROUND-PROPOSAL-INDEPENDENT-STATIC-REVIEW-1.json` | `895ada61be4f6e6f845c44d2705d42ec4a861bb91e1af3c662175f358350a443` |
| `/private/tmp/zee-k300/k307-foreground-proposal-2/scripts/tests/native-pg-owner.py` | `6e5f004a99af5685bcd9625b73df2ae574f21854f0b0bff06dc8013c18b4d9c2` |
| `/private/tmp/zee-k300/k307-foreground-proposal-2/scripts/tests/native-pg-owner-preflight-test.py` | `e0a5ea63590807e9310ad3eb12bef37defd49d3be7ea57f1390f5f8cae1d4552` |
| `/private/tmp/zee-k300/k307-foreground-proposal-2/scripts/tests/modul-sql-native.py` | `43fab53de8223c32ed197538e1a78493069a7d6fa85341dcdff360bcb92407e4` |
| `/private/tmp/zee-k300/k307-foreground-proposal-2/scripts/tests/modul-sql-test.py` | `a49a0071825a9e6761ece34f48e243290b406242d990c270cd3ee3b983ee21f7` |
| `/private/tmp/zee-k300/k307-foreground-proposal-2/compiler/tests/modul_sql_testi.rs` | `220fbf1e5710bc11c45441a0fc819bd0572857268111d8243fb3e466e7b9adc4` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-foreground-model23-controller-1.json` | `3e78c1613b63b307dbc359546b432f9b7e8c9f13ecee35a769f4c6d8b98e8091` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-foreground-model23-1.log` | `5ef4987aff494d34e03b336f6217b01ef2918aed23a300a13edfda2330564ca8` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-foreground-combined24-controller-1.json` | `10176059abe4f22c1298dd6bdf6c5eaa2c80c1edace99bb17bb29927266a1096` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-foreground-combined24-1.log` | `3373fed733da9bcbfdb3b715639fdfdb126ba835b56c6d8bd20d43d6e4b98b60` |
| `/private/tmp/zee-k300/K307-MODUL-SQL-CANCELLATION-INDEPENDENT-ASSESSMENT-1.json` | `9ced1b582755f2ba2701c274b6f86ef0c895802156f11541b2e46922252abc77` |
| `/private/tmp/zee-k300/modul-sql-cleanup-cancel-counterexample-1.py` | `6060f121186a203d86330050f274214120919fd630248eef398ffb3c5eb9b353` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-modul-sql-cancel-counterexample-controller-1.json` | `d0fe1cdd935ccd9e0f4c179795f5ec40192cca313102ffae1d80bfa73a67d701` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-modul-sql-cancel-counterexample-1.log` | `8328e4259037efc7a299b939960c15743d03e4ebfe2ac1eb52a44abae8bef88f` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-modul-sql-baseline10-controller-1.json` | `a2fb1a876322d488f2407a14257cb24d13cb77474c87ebc9bf62fae55a644102` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-modul-sql-baseline10-1.log` | `4953417aab84c5f724fae90ef047a24094aedf99179fe4023c23d308d690d15d` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-modul-sql-model13-controller-1.json` | `984504fff0225993c8f6df8b71223fd018c8ce61e940dfc57418b078fe8575f4` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-modul-sql-model13-1.log` | `ba4aad699e532fed69331fc1da28feab1a47050f7394977a07c95996ea8a0c8b` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-modul-sql-combined23-controller-1.json` | `09a8eb21d362a483e1cf0229cb7e3ad969635963334fd5ff3b3c1511ea20a10c` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-modul-sql-combined23-1.log` | `01a3361790affbbb7f05249396624d280422febb36bfe07f37a85876115130fc` |
| `/private/tmp/zee-k300/k307-modul-sql-proposal-5/PREPARATION.json` | `85d72c955ff3555f4b2a2faea973e7609e94c483c87f959775e94dbe6c6b5c63` |
| `/private/tmp/zee-k300/K307-FOREGROUND-COMBINED2-INDEPENDENT-REVIEW-1.json` | `2c9584264b39b24b92de284163f37721132c905094ee7f4b37b819fa937a35de` |

Bu tarihsel hazırlık makbuzları kaynak/normalPG/native kabulünün yerine geçmez; actual yeni native kanıtı yukarıdaki frozen kayıt zinciridir.

## K-306 — Linux kabul koşucusu: dar kapsamda kapalı

İki kabul koşucusunun canonical kök/erken handler, yapışkan iptal ve failed-body/tekrar-close doğruluğu; kısmi setup ve foreground PG sahipliği düzeltildi. Üretim compiler/dil/API değişmedi; iki mevcut ignored Cargo sahibi ve dispatcher4/HTTP7 oracle korundu.

**Tarihsel C12 kaynak hazırlığı:** `7d5e4b04391c82d2dec0a73a81d39de09357464e`; tree `803f6698e3926de68093634d40b531f1de4219a4`. Kaynak doğrulaması, ikinci denemenin geçen 12 komutu ile yalnız EOF boşluk düzeltmesinden sonra yeniden geçen beş komuta dayanır; 15 kapının bütünü yeniden çalışmış gösterilmez. Temiz `7d5e4b0` kaynak commit’i ve exact ağaç kayıtlıdır.

**Hazırlık ve gerçek inert ayrımı:** Kontrollü hazırlık Campaign16 ve PG18 yöntem/31 senaryoda geçti. Dış sahiplik hazırlığında core17, CLI9 ve runner5 modeli geçti; bu sayılar Cargo/native vaka toplamına eklenmez. Aynı16 eski Campaign modeli13 assertion failure üretirken aday0 failure verdi. Gerçek inert VM kabulünde normal komut ve EOF olumsuz yolu doğrulandı. EOF iç adımı `failed/control-eof` kaldı; özgün çocuk kapanıp reap edildi. VM normal SIGINT/exit0 ile kapandı, fallback kullanılmadı ve özel host runtime silindi. Bu iki inert prob, Linux native4+7 kabulünün yerine geçmez.

**Tarihsel kampanya sınırı:** C12 VM1–4, S13 VM5 ve C14 VM6/VM7 sonuçları aşağıdaki kendi public kaynaklarında korunur; aynı C14 kaynağın fresh attempt8 (VM8) kabulü yalnız gerçek frozen kanıtla doldurulacak ayrı bölümdedir.

**Gerçek kampanya başarısızlıkları:** V1 ve V2 ayrı fresh VM denemelerinde 45 bootstrap ve yedi başlangıç kontrolü geçti; 53. komut olan native validator prepare exit1/reaped ile durdu. Her iki VM normal exit0 ile kapandı, bütün özgün sahipler reap edildi ve dış owner exit1/reaped kaldı. Derleme ve native dispatcher4/HTTP7 başlamadı. V1 başarısız prepare ham günlüğü dışarı aktarılmadığı için kök neden bilinmiyor; V2 sonucu V1 için varsayılmaz. V2 dışarı aktarılmış günlükte gerçek `FileNotFoundError: /run/user/501/bus` kaydı vardır; günlük SHA-256 `6d28927ed5c8e7dfd529edc0a3e91ad220dc1d025f25f55f9e29be419a247bf1`. V1/V2 başarısız kanıtları korunur.

**V3 gerçek kampanya — başarısız kapanış:** `E/k306-linux-campaign-3` public kaydında 48 bootstrap ve 18 build komutu geçti. Dispatcher için mevcut tek Cargo/libtest sahibi PASSED verdi (derleme 13,36 sn; libtest 509,65 sn); ardından üç source-end kontrolü geçti. `manager-end` exit1 ve degraded günlük sonucu nedeniyle kampanya başarısız kaldı. HTTP çalışmadı. Dispatcher iç proof/günlük aktarımı 17 adımlık fazın sonunu beklediğinden yapılmadı; bu Cargo sonucu yapılandırılmış dispatcher4 kabulü sayılmaz. Dış süre 872,0194029 sn; bütün özgün sahipler reap edildi, VM normal exit0 ile kapandı, fallback kullanılmadı, dış owner exit1/reaped kaldı ve runtime korundu. Başarısız unit için reset eksikliği yalnız doğrulanmamış hipotezdir; neden kesinleştirilmedi ve reset-failed/oracle atlama yapılmadı.

**Tarihsel S13 ilk kaynak takip dilimi — o aşamada K-306/GB-115 açık:** VM4 gerçek sonucu: C12 üzerinde E/k306-linux-campaign-4 kaydında 48 bootstrap ve 18 build komutu geçti; dispatcher Cargo sahibi exit0/reaped ve dışarı aktarılmış dört senaryo geçti. Son failed-unit listesi tek sahipli zee-dispatch.service birimini gösterdi; manager-end exit1/degraded ile kampanya durdu, HTTP başlamadı. Host/guest özgün sahiplerin tamamı reap edildi, PostgreSQL ve VM normal exit0 ile kapandı, fallback kullanılmadı; dış owner exit1/reaped kaldı. Bu başarısız kampanya yeni kaynak kabulü veya K-306/GB-115 kapanışı değildir. Doğrulanmış sahipli unit kimliği: manager-failed-end.log içeriği exact `zee-dispatch.service not-found failed failed zee-dispatch.service`; dispatcher unknown/restart senaryosunun aynı kampanya tarafından kurulan birimidir. Kamu proof/log hash ve byte kayıtları T/K306-VM4-OWNED-UNIT-DIAGNOSIS-1.json bağımsız incelemesinde bağlıdır; foreign unit gözlenmedi. Public VM4 `manager-failed-end` kaydıyla kimliği doğrulanan kampanya unit’i için `Campaign.unlink` düzeltildi: mevcut exact stop+cgroup-empty ve symlink inode/hedef/kaynak hash kanıtından sonra yalnız `self.units` üyesi exact `p.name` için reset-failed çağrılır; ActiveState inactive olmalı ve bağlantı kimliği yeniden doğrulanmalıdır. Ardından mevcut unlink/daemon-reload/LoadState not-found zinciri sürer. Global veya foreign reset yasaktır; foreign failed unit genel manager kapısını yine kırar. Yapışkan failure/cancel sonucu, cgroup/path sahipliği, komut/cleanup bütçeleri ve dispatcher4/HTTP7 oracle değişmedi. Bu anlatım yalnız VM4 sahiplik kanıtı, root tarafından doğrulanmış kalıcı21+PG18 sonuçları ve aynı toplu dilimde kaynak kopyasının uygulanması altında geçerlidir.

**Kontrollü model sınırı:** Ayrı altı inert metot root tarafından exit0/reaped ile geçti: bir eski davranış karşı örneği ve beş olumlu metot; eski altı metodun başarısızlığı iddia edilmez. Mevcut 16 Campaign metodu korunup aynı preflight dosyasında ve aynı Cargo sahibi altında beş metot eklendi; toplam21. Kalıcı Campaign21 gerçek sonucu: E/k306-unit-reset-campaign21-controller-1.json ve hash bağlı günlük: 21 metot geçti; özgün owner observed_exit=exit=0, closed/reaped, hata ve iptal yok. Önceki16 metot korunur; beş yeni metot exact sahiplik, inactive durumu, foreign durumun korunması ve yapışkan iptal sınırlarını doğrular. Birleşik PG18 yöntem/31 senaryo gerçek sonucu: E/k306-unit-reset-pg18-controller-2.json ve hash bağlı günlük: 18 metot/31 senaryo geçti; özgün owner observed_exit=exit=0, closed/reaped, hata ve iptal yok. İlk denemenin iki eksik public tüketici fixture hatası korunur; yalnız exact C12 dispatcher/HTTP fixture kopyaları eklendi, oracle değişmedi. Kalıcı21 root kaydı `E/k306-unit-reset-campaign21-controller-1.json` içindedir; PG18 ikinci controller kaydı geçti. İlk PG18 denemesi yalnız iki public tüketici fixture’ı eksik olduğu için başarısız kaldı; exact C12 kopyaları T içine eklenerek tekrarlandı, oracle değişmedi. Bu kontrollü modeller ve koşullu peer incelemesi native düzeltme/kapanış kabulü değildir.

**S13 gerçek kaynak kapıları ve VM5 başarısızlığı:** Tam15 kaynak kapısı `E/k306-source-gates-3/PROOF.json` ve controller3 ile geçti; gerçek tam faz998/0/25. Temiz kaynak commit `aec09c1cced3824992077ae1ef920eb4d2c64ac2`, tree `a8643d550b39f53a1eb17c3e1b6ac99e8abcc79e`, parent C12; exact kaynakcommit2 ve bağımsız kaynak incelemesi kayıtlıdır. VM5 aynı S13 kaynakta dispatcher iç proof’un ilk iki vakasını geçti; dört vakanın tamamı geçmedi. İç proof101 komut/günlük kaydı taşır. 0096 ve0101 `reset-zee-dispatch.timer` günlüklerinde `Unit zee-dispatch.timer not loaded.` hatası vardır; önceki exact service reset adımı geçti. İç proof failed/failed_body=true, owned-unit-links cleanup Reject kaldı. Timer reset öncesi ActiveState VM5 içinde doğrudan sorgulanmadı; bu eksik gözlem actual inactive sorgusu gibi yazılmaz. HTTP başlamadı. PostgreSQL normal exit0/reaped, VM normal exit0, bütün özgün sahipler reap; dış özgün owner exit1/reaped, fallback yok; runtime korunur. Bu başarısız kampanya native4+7 veya K306/GB115 kapanışı değildir. Bağımsız VM5 tanısı: T/K306-VM5-TIMER-RESET-DIAGNOSIS-1.json ve root model incelemesi16public kayıt/101iç günlük bağını doğruladı: exact zee-dispatch.timer reset96/101 not-loaded hatası, native2/4/bodyfailed/cleanupReject; HTTP başlamadı. Timer ActiveState reset öncesinde doğrudan sorgulanmadı. PG ve VM normal0, bütün özgün sahipler reap, dış özgün owner1/reaped; VM5 kaydında K306/GB115 açık kaldı.

**Takip2 sahipli unit düzeltmesi:** Aynı iki test-harness kaynak dosyasında sahipli unit temizliği minimal daraltılır: mevcut stop+cgroup-empty, exact link inode/hedef/hash ve `self.units` üyeliği kanıtından sonra ActiveState okunur. Yalnız `failed` için exact sahipli adla reset-failed çağrılır; `inactive` durumda reset çağrılmaz. Diğer durumlar reddedilir; reset sonrasında inactive durumu ve link kimliği yeniden doğrulanır, ardından mevcut unlink/daemon-reload/LoadState not-found zinciri sürer. Global/foreign reset, sahiplenme veya oracle atlama yoktur. Yapışkan failure/cancel, path/cgroup sahipliği, komut/cleanup süre sınırları ve native4/7 oracle değişmez. Bu dar test-harness düzeltmesi actual VM5 tanısı ve kontrollü model kanıtıyla aynı kaynak/test/belge dilimindedir; yeni exact kaynak pinleriyle bağlıdır.

**Takip2 kontrollü test ve commit sınırı:** Önceki Campaign16+5=21 ile PG18/31 sonuçları S13 hazırlık tarihçesidir. Takip2 kalıcı Campaign modelinin gerçek yöntem sayısı ve sonuç/owner makbuzu: Root actual baseline21 geçti; yeni23 yöntem eski S13 hedefte iki ek yöntem boyunca beklenen7subtest failure+1error üretti. Aday kalıcı23 ve ayrı focused7 geçti; bütün original ownerlar terminal/reaped, iptal yok. Baseline21 ile counterexample23 sonuçları E/k306-unit-state-... controller/log kayıtlarında ayrı korunur; bunlar native kabulü değildir. Takip2 PG hazırlığının gerçek sonucu: E/k306-unit-state-pg18-controller-1.json ve hash bağlı günlükte18yöntem/31senaryo geçti; exact aday target2892a4f... ile original owner observed_exit=exit=0/closed/reaped/errors[]/cancelled[]. PGhazırlığı, dispatcher ve HTTP public fixture kopyaları currentS13 source341 pinleriyle birebirdir; oracle değişmedi. Bu kontrollü yöntem sayıları ve eski kaynak karşı örnekleri native Cargo toplamına eklenmez.

**Yeni exact kaynak ve tam kapılar:** Kaynak commit `42ca6817c010e97cb6dd5ff73cb7f7cd970984aa`; tree `5585785367353af22dd35b539b50ce4233dbbac2`. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. Kaynak341/native214/production175 dosya sayıları korunur; aynı iki harness kaynağının pin değerleri, transfer/arşiv ve native zinciri yeni exact kaynağa bağlıdır. Native214 ve source341 ayrı domain'lerdir; birebir altküme varsayılmaz. C12 composite12+5 ve S13 tam15/commit2 ile VM1–5 başarısız kampanyaları tarihsel kaynaklarında kalır; yeni sourcegate4/commit3 veya native sonucu yerine kullanılamaz. Diğer beş PG tüketicisi açık; P09/P13/P36 kısmi, P37 sürekli korunur.

**Tarihsel C14 VM6 — kabul başarısız:** Aynı `42ca6817c010e97cb6dd5ff73cb7f7cd970984aa` kaynak/tree `5585785367353af22dd35b539b50ce4233dbbac2` üzerinde dispatcher iç4/4 ve HTTP iç7/7 geçti; iki mevcut Cargo sahibinin her biri exit0/reaped oldu. Guest102 komutun101'i exit0, son HTTP validator exit1/reaped kaldı; boş public validate.log ilk private exception'ı açıklamaz. HTTP public kaynak/begin-end binding export'u yoktur. Her iki foreground PostgreSQL normal SIGINT/exit0/reaped, VM normal exit0/fallback yok, bütün özgün sahipler reap, dış owner exit1/reaped; runtime korunur. Manager/unit son kontrollerinin public kanıtı geçse de kampanya native kabul veya K-306/GB-115 kapanışı değildir. Eski regex'in gerçek interleaved HTTP Cargo günlüğünü deterministik reddettiği public baytlardan doğrulanır; tam ilk private validator exception'ı bilinmiyor ve sonraki private kontrollerin geçtiği çıkarılmaz.

**VM7 parser hazırlığı — inert kanıt sınırı:** V1 parser'ın gerçek27 inert metodu geçti; bağımsız inceleme Python sözlük eşitliğinde bool/sayı tür ayrımının kaybolmasını bloke etti. V2'nin tür koruyan kanonik JSON build karşılaştırması için aynı27 yöntem kaynak/AST olarak korunup iki typed negatif ve bir key-order olumlu yöntem eklendi. Eski V1, yeni30 yöntem içinde iki typed yöntemde beklenen10 subtest failure verdi; eski27 ve key-order olumlu geçti. V2'nin gerçek30 inert yöntemi exit0/reaped ile geçti; root ve bağımsız peer incelemesi bu sınırı doğruladı. Aday yalnız saf libtest framing parser'ı ve eski üç regex gereğinin parser çağrısıyla değiştirilmesidir; unique exact selector/running1/summary sayıları ve ilk unique exact typed build korunur. Native dispatcher4/HTTP7, PG, kaynak, unit, owner ve iptal oracles değişmedi. Bu model sayıları Rust/native toplamına eklenmez; hazırlık fresh VM7 başarısı veya frozen kabul değildir.

| VM6/VM7 hazırlık tarihçesi — public kayıt | SHA-256 |
| --- | --- |
| `/private/tmp/zee-k300/K306-VM6-FROZEN.json` | `4e6d60232d99092aa7f6c66a4be77df9e00479ffe03f965b34984235ef1eff41` |
| `/private/tmp/zee-k300/K306-VM6-ROOT-PUBLIC-FAILURE-AUDIT-1.json` | `1349b58b7c25c932845a85fb78db61686e9293bb1758f6ac0706a65e0d4de6c6` |
| `/private/tmp/zee-k300/K306-VM6-HTTP-LIBTEST-DIAGNOSIS-1.json` | `820a3b675db2956bbf213233012dfafb55160ddba884f5bdb4ae84ca2c59a748` |
| `/private/tmp/zee-k300/K306-LIBTEST-PARSER-INDEPENDENT-REVIEW-1.json` | `ba26894359c24670a5c76db8c5547fd63bd31a95547f1f9294278ba8450e1455` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-libtest-parser-v1-counterexample-controller-1.json` | `f2a57d9cfae34fe566a48964556f939de9ad7bd8515cf4ece779347eb02d0c59` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-libtest-parser-v1-counterexample-1.log` | `07ce5666b29796a5275105869d2e3a6700c481774c4314c526608226d488fbfa` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-libtest-parser-v2-models-controller-1.json` | `8a37771958b186a2470c738207fc500df9fd53dbcd7caec55a9dc8437b02893f` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-libtest-parser-v2-models-1.log` | `2442fbe515445265a1fbb3da729105971a4a20711fc9cb11252220ebfbd68e3b` |
| `/private/tmp/zee-k300/K306-LIBTEST-PARSER-V2-ROOT-MODEL-REVIEW-1.json` | `dacef94efff553ac8bb0aa8d07071b9b9c797cd5b98c272486cb797f644dfee9` |
| `/private/tmp/zee-k300/K306-LIBTEST-PARSER-V2-INDEPENDENT-REVIEW-1.json` | `b91b99c91805daa01ed80d90727e82ed0d53e4b182d7a883d228340aa3a6dbce` |
| `/private/tmp/zee-k300/k306-libtest-parser-proposal-v2/native_validate.py` | `98a810e1a6ff37d966e214721bdbe90a2388f4d473de9eb4eed5e6bdf43f0a9b` |

**Tarihsel C14 VM7 — calendar gövdesi başarısız:** Bootstrap48 ve build18 geçti; dispatcher iç4 gerekli vakanın yalnız ilk1'i geçti. Calendar gözleminde11 ready ve28 leased yanıtı yayımlandı; done/calendar-pass kaydı yayımlanmadı. Guest75 komutun74'ü exit0, Cargo sahibi exit101/reaped kaldı; HTTP hiç başlamadı ve yeni typed parser native final validator aşamasında çalıştırılmadı. İç114 komut/günlük ve cleanup103–114 public kanıtlıdır; timer inactive skip ve exact service reset temizliği geçti, sahipli iki unit released/not-found kaldı. Service failed gözlemi stop sonrasındadır; cleanup öncesi service durumu veya exact lease/renew/apply kök nedeni gibi kullanılamaz. PG normal SIGINT/exit0/reaped, VM normal exit0/fallback yok, bütün özgün sahipler reap, dış owner exit1/reaped; runtime korunur ve erişim/adoption yasaktır. VM7 native kabul veya K-306/GB-115 kapanışı değildir.

**VM7 host güç sınırı — korelasyon, exact job nedeni değil:** Dar birincil host günlüğü `2026-10-16 17:20:15 +0300` Clamshell Sleep ve `18:52:34 +0300` son etkileşimli Wake olaylarını doğrular. Aradaki5539 saniyenin kesintisiz uyku veya VM pause olduğu iddia edilmez; ara DarkWake döngüleri bu iki satırlık alıntının dışındadır. Calendar'ın öngörülen15:20 UTC elapse zamanı iki ana olay arasında kalır. Sleep/resume çevresel zamanlama karıştırıcısı olarak kanıtlı korelasyondur; exact private ilk exception, guest saat adımı, SQL lease expiry/renewal/commit veya uygulamanın ilk başarısız dalı kanıtlanmış değildir. Bekleme120 ile üretim165 sınırı yalnız statik bütçe bağımlılığıdır; daha uzun sürenin geçeceği veya timeout/lease/oracle gevşetmesinin doğru düzeltme olduğu çıkarılmaz.

**Fresh attempt8 (VM8) kapsamı:** Aynı temiz C14 kaynak341/native214/production175, archive3, exact runner-v7/inputs8 ve typed parser98a baytları kullanılır; yalnız fresh kampanya/controller8 ve yeni özgün VM kimliği ayrılır. Mevcut102 komut, dispatcher4/HTTP7, PG/unit/manager/kaynak/özgün owner/normal VM ve bütün bütçe/lease/cleanup oracles aynı kalır. VM7 retained runtime/PID/key/private durumu devralınmaz. Gelecekteki yeni attempt sonucu ayrı kanıttır; VM7 geriye dönük başarıya çevrilmez.

| VM7 başarısızlık ve host güç kaydı | SHA-256 |
| --- | --- |
| `/private/tmp/zee-k300/K306-VM7-ROOT-PUBLIC-FAILURE-AUDIT-1.json` | `d1986b5a31160bad07a8724a5aa91cc0acfd0f282eefc16bb5a4bee094217663` |
| `/private/tmp/zee-k300/K306-VM7-CALENDAR-FAILURE-INDEPENDENT-DIAGNOSIS-1.json` | `8862b9ff6e782672738e4b20e5174b617c56a11eab6efb478d645b0bf6b1cdc1` |
| `/private/tmp/zee-k300/K306-VM7-CALENDAR-LEASE-SOURCE-ASSESSMENT-1.json` | `7a4b439d53e799719d3c721b629ce9030f1796ec357135d61058158232f7d66b` |
| `/private/tmp/zee-k300/K306-VM7-HOST-POWER-CORRELATION-1.json` | `f3870b22af5e2ff6aad4e6b1679b584a5b3342bc71223365eaa4fb40944458f6` |
| `/private/tmp/zee-k300/K306-VM7-HOST-POWER-MAIN-EVENTS-1.log` | `78a3028547c1dfe7a8b8c33e4460b21835abe76570a23e0c4558ab3107a96bb4` |

**Fresh attempt8 (VM8) Linux kabulü:** Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı. Yalnız bu exact kaynakta2/25 mevcut ignoredCargo sahibi ve dispatcher4+HTTP7native geçti;tamfaz998/0/25 ayrı;kalan23ignoredCargo-vakasıbuAttempt8kampanyasındaçalışmadı. Native11 iç vaka Cargo toplamına eklenmez; C12 kontrollü modelleri yeni Rust/native toplamına katılmaz. Tarihsel RC/soak/IDE, fiziksel platform ve ürün kabulü bu kaynağa aktarılmaz. Fresh attempt8 gerçek public ortam günlükleri: mimari aarch64; rustc 1.93.1 (01f6ddf75 2026-02-11); postgres (PostgreSQL) 16.15. İki rolün user manager son günlüğü running; BuildJSON/ikili ve kaynak pinleri aynı yeni exact kaynakla bağlı. Foreground PostgreSQL normal SIGINT/exit0/reaped; iki kabul rolünün sahipli unit/path temizliği, manager running son durumu, guest/host bütün özgün sahiplerin reap sonucu ve VM normal exit0 kapanışı doğrulandı; fallback kullanılmadı, host runtime kaldırıldı. Gerçek source15/clean commit ve attempt8 source/native/owner/raw zinciri bağımsız review ile issues=[] olarak doğrulandı; frozen kabulün exact11 öncül kayıt ve public artifact seti eşleşti.

**Resmî systemd v252 dayanağı:** [systemctl v252 kaynak belgesi](https://raw.githubusercontent.com/systemd/systemd/v252/man/systemctl.xml) reset-failed komutunu failed durumunu temizleyen işlem olarak tanımlar. [v252 method_reset_failed_unit uygulaması](https://raw.githubusercontent.com/systemd/systemd/v252/src/core/dbus-manager.c) bu işlem için unit’i yüklemez; unloaded unit’in failed durumda olamayacağını açıklar. Bu genel sözleşmenin VM5’teki unloaded timer hatasıyla bağlantısı ayrıca yukarıdaki exact ham günlük/bağımsız tanıyla kurulmalıdır; tek başına resmî kaynak yeni düzeltmenin native kabulü değildir.

**Son belge ve kapanış:** K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. Dar K-306/GB-115 kapanışı yalnız actual yeni exact kaynakta ikiCargo/dispatcher4+HTTP7, özgün manager/unit/PG/VM sahipliği, bağımsız frozen kabul ve final11/tested-overlay temiz kayıt commit’i kanıtıyla geçerlidir.

| Gerçek public kanıt | Konum | SHA-256 |
| --- | --- | --- |
| source-freeze | `/private/tmp/zee-k300/K306-VM8-STATE-SOURCE-FREEZE-1.json` | `11a00feba64c492019cd5729747260b6202d2fe048b050e6706c120d9fbbb094` |
| acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-linux-acceptance-record-1.json` | `5a62e51841589922584cbf5f7cbfd3abf0ab2d1651ce4e5008029da5ef1ed129` |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-source-gates-4/PROOF.json` | `eb5fad310cd125a137fc479b82ac1b3d21af41a0438662ab00f8501a45037f90` |
| source_gate_controller | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-source-gates-controller-4.json` | `d2d2692b968f316da357cf024fad651eb3aebf5158c2701503510ee4971be324` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-source-commit-3.json` | `ff25931606e9aa628a6b988d6bc95728e76580109287934ace9245b406f8c5b6` |
| source_commit_controller | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-source-commit-controller-3.json` | `a4f7c9abab3dc326bd1d3d6d512603df9be13f9620a79ed16f5fcbb70cc71326` |
| source_review | `/private/tmp/zee-k300/K306-SOURCE-STATE-INDEPENDENT-REVIEW-1.json` | `f6a54bacd7f97d58f84688892ba136fb98ba7773cec9e5cafe27c44260891deb` |
| vm5_diagnosis | `/private/tmp/zee-k300/K306-VM5-TIMER-RESET-DIAGNOSIS-1.json` | `041c2a92285b7c3f9a50e07ae5f2f88b4ba677baee8a04b39618341602bc9252` |
| host_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-linux-campaign-8/PROOF.json` | `2309955c11262cee1ace8b9293efbd386d10db7b0010ce7dc659bb115031145a` |
| host_controller | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-linux-campaign-controller-8.json` | `43a7d66710cf743c5dad5708efec546cfd990ffb9d1e9df359276ed282a38ea9` |
| guest_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-linux-campaign-8/guest/GUEST-PROOF.json` | `8826c6da759ca1c558327540c2ae90d656853dc24af27b999e28c751cde5c98a` |
| dispatcher_binding | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-linux-campaign-8/guest/native-bindings/DISPATCHER.json` | `3129b74983d3b8e93b476df7aa15cd777a7c033cec7782c6d945fa441184574b` |
| http_binding | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k306-linux-campaign-8/guest/native-bindings/HTTP.json` | `8096d8000ebf5414603dd174e28697ad7db11697b9b5386656911920b639d4a1` |
| independent_review | `/private/tmp/zee-k300/K306-VM8-LINUX-NATIVE-INDEPENDENT-REVIEW-2.json` | `671fe174145409360d9898b06be83e2222f50622a0c455b344835f774f53fafd` |

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
| K-306 Linux koşucusu | Temiz7d5e4b0 kaynak12tarihsel+5yeni komutla doğrulandı; model17/9/5 ve gerçek inert2 geçti. Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı. GB-115 kapali; son belge kapısı kayıt koşulu. |
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
tüketicisinin sahiplik/iptal hazırlığı yerel açık iştir. C12 hazırlığı,
S13 tam15/commit2 ve VM5 timer hatası kendi tarihsel kaynağında korunur;
yeni exact kaynakta yalnız failed durumunda reset/inactive durumda skip
düzeltmesi kaynak/model kanıtıyla bağlıdır. Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı.
K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. Eski kaynak/native sonuçları taşınmaz.

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
| P09 — Kuyruk/dispatcher | Kısmi; S3 tarihsel. Temiz7d5e4b0 kaynak doğrulandı; Yeni 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa kaynakta mevcut tek Cargo sahibi/dispatcher4 geçti; C12 V3/V4 ve S13 VM5 başarısız kampanyaları tarihsel kalır. | Gerçek ürün/sağlayıcı benimsenmesi ve genel exactly-once açık. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. | Yerel dilim için yok; ürün için U1 + U5. |
| P10 — SMTP/PG hata sınırları | Kısmi; S3 fault11 tarihsel. | Canlı sağlayıcı, gerçek katalog ve ürün entegrasyonu açık. | U1 + U5. |
| P11 — PDF/FFI | Kısmi; temiz C2 PDF9 ve bağımsız incelemesi geçti, K-299 dar kapsamı kapandı. İlk C başarısızlığı korunur. | Gerçek katalog, genel FFI/sandbox ve yayın ayrı. | Ürün için U1 + U5. |
| P12 — Göç/ürün betikleri | Kısmi; dar yerel göç/rollback kanıtı var. | Gerçek üründeki Python betiğinin kaldırılması ve göç kabulü açık. | U1 + U5. |
| P13 — Sürekli HTTP servisi | Kısmi; K-303 havuz1 kendi47812bf kaynağında. C12 V3/V4 ve S13 VM5 HTTP çalışmadı; Yeni 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa kaynakta mevcut tek Cargo sahibi/HTTP7 geçti; C12 V3/V4 ve S13 VM5 HTTP çalışmadı. | Üretim/pilot benimsenmesi ve fiziksel platform açık. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. | Ürün U1 + U5, platform U2. |
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
| P36 — Test/ignored kapsamı | K-302 seçili9, K-303 havuz1, K-304 directweb1, K-305 bridge1 ayrı kaynaklardadır. K-306: Yalnız bu exact kaynakta2/25 mevcut ignoredCargo sahibi ve dispatcher4+HTTP7native geçti;tamfaz998/0/25 ayrı;kalan23ignoredCargo-vakasıbuAttempt8kampanyasındaçalışmadı. | Farklı SHA sonuçları toplanmaz; diğer sahiplerin mevcut kaynak ve ortam bağımlılıkları ayrı açık kalır. | Yerel dilim için yok; gerçek cihaz/ürün/servis ayrıca girdi ister. |
| P37 — Sürekli kalite kapıları | Sürekli iş. K-306 kaynak12tarihsel+5yeni doğrulaması kayıtlı. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. | Her yeni atomik dilim kendi kaynak/test/belge kapılarını tekrar sağlar. | Yerel işler için yok; bağlı dış kapılar ayrı. |
| P38 — Backend/WASM/self-hosting | **Ertelendi.** | Native WASM/backend/self-hosting gerçekleştirilmiş sayılmıyor; V1 ve ihtiyaç koşulları korunuyor. | Şimdi yok. |
| P39 — Lisans/yayın kararı | Kısmi karar: sınırlı pilot seçildi, lisans bekletildi. | Mevcut marka/site korunur; public/yayın kararı henüz verilmedi. | Zamanı gelince **U6.** |

K-298–K-305 dar kabulleri kendi kaynak/ölçüm/native/arşiv ve son belge
makbuzlarıyla tamamlandı. K-306/GB-115 dar kapsamda kapalı; önceki kaynakların
RC/RSS sonuçları bu işe aktarılmaz. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. P19'un artımlı HIR/tür ve UI gecikmesi ile tabloda kalan ürün
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

## K-310 kaynak15 ilk denemesi — tarihsel biçim kapısı başarısızlığı ve dar takip

K310 ilk kaynak15 denemesi ilk12 komutu geçti: phase-document, preparation,
counts, summary, catalog-freshness, spec-drift, fmt, phase-matrix, clippy,
conformance, semantic-regresyon ve core-freeze. Bu koşunun tam faz raporu
998/1023 geçti,0 başarısız,25ignored sonucunu verdi; bu sonuç ayrı native
metrics8 veya nested libpool kabulü değildir.13.komut diff-check exit2 ile
`scripts/tests/native-metrics-test.py:540` boş satırındaki dört trailing space'i
reddetti.14.secret-scan ve15.security-continuous çalışmadı. Özgün dış owner
exit1/observed_exit1 ve owned-group-closed/reaped verdi; tool exec session
numarası OS süreç kimliği olarak kullanılmaz. İlk koşu başarısızdır; tam15
kaynak kabulü veya temiz source commit sonucu olarak yeniden adlandırılmaz.

Biçim takip P3 adayı native harness'in yalnız bu dört whitespace baytını
kaldırır: P2 SHA256 `ac4c84f9a2f9948883715f4c96c0bda70f5a02d6194e3bd493e270a66d8159a0`
36531byte → P3 SHA256 `b5fd46b70ac21dc4882ef1d4513f0a89836bc4a2453d85528a42ecb2b2dd89f2`
36527byte. Tam Python AST aynıdır; diğer iki Python ve Rust adayları,
production175, lifecycle helper ve shared foreground PG helper6e5 değişmez.
P1/P2 standalone52 ve combined16 kontrollü kanıtları kendi exact baytlarında
tarihsel kalır; P3 özgün kontrollü koşuları yalnız kendi actual makbuzlarıyla
değerlendirilir.52/16 veya faz998, Cargo/native sayılarıyla toplanmaz.

Bu ilk başarısız koşu anında K310/GB119 açıktı; fresh tam kaynak15 ve temiz
exact kaynak commit'i, ardından aynı kaynakta fresh metrics8 ve ayrı mevcut
nested release libpool sahibi/120sn kabulü, bağımsız frozen incelemesi ve
final11-tested-overlay/temiz kapanış commit'i zorunlu zincirdir. Bu biçim
hazırlığı tek başına bu zincirde başarı veya dar kapanış sağlamaz. Metrics ve
effect PG tüketicilerinin diğer sınırları/P13kısmi/39plan statüleri değişmez.

Public tarihçe pinleri:
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-1/PROOF.json` SHA256 `647dd7cff23ef5723c2b63c19c191528435763028e8f11aa8e044f79df9f2d26`, 63065byte (source_gate).
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-controller-1.json` SHA256 `37d7ffb775ceca81c72b0f47f23d504f3172f5b356e00a497adccf7f31f95319`, 1053byte (owner).
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-driver-1.log` SHA256 `833d438c68f26b15a7805f51cc25048809b4b0a67bd36a295703f483f5cb5a58`, 252byte (owner_log).
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-1/diff-check.log` SHA256 `f76e1f6fdb94d5c840c0db346985fa1260979f38cf156f15a16fbde3f04ad829`, 69byte (diff_check_log).
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-1/phase-matrix-report.md` SHA256 `2a39d6d38455f69da6b70d111a91db6af61424402f58de16fc5717de8b0e331e`, 38262byte (phase_report).
- `/private/tmp/zee-k300/k310-metrics-proposal-3/PREPARATION.json` SHA256 `25910b5d48f5801c8b47a0ba0e6c2498a38a6570be2393cec04b834b25622dd4`, 64192byte (yalnız biçim takip hazırlığı).

## K-310 kaynak15 ikinci denemesi — tarihsel public-source hash yanlış pozitifi

K310 kaynak15 ikinci denemesi ilk13 komutu geçti. Bu koşunun tam fazı
998/1023 geçti,0 başarısız,25ignored ve22faz olarak raporlandı;
phase-matrix1061.532329sn,clippy168.882678sn ve diff-check exit0 ayrı
komut sonuçlarıdır.14.secret-scan exit1 nedeniyle koşu başarısız kaldı;
15.security-continuous çalışmadı. Özgün dış owner observed_exit=exit1 ve
owned-group-closed/reaped verdi; dış süre1300.932722375sn. Önceki ilk denemenin
dört trailing-space diff-check başarısızlığı da kendi tarihçesinde korunur.
İlk13 başarı tam15 kaynak kabulü, temiz kaynak commit'i veya native kabulü
olarak yeniden adlandırılmaz; başarısız iki deneme yeni koşuya taşınmaz.

Root'un16public dosyalık redakte teşhis kesitindeki tek bulgu,
`metrics-pg-owner-preflight-test.py:816` satırında `generic-api-key` kuralıdır.
Redakte Match `api_path,'REDACTED'`, Secret `REDACTED` olarak saklanır; gizli
ham payload burada okunmaz veya yayımlanmaz. İlgili literal gerçekte
`scripts/native-metrics.py` public kaynak dosyasının SHA256 kimliğidir:
`d5d12902358591365ff201c13c44b255d265c49c71befabcdef0575fe7e854c4`
(5009byte). Aynı literal ile `selected(api_path, public-source-hash, ['Channel'])`
AST seçicisinin kimlik denetimi yapılır; public dosyanın gerçek hash'iyle
birebir eşitliği doğrulandı. Bu tek bulgu public-source hash yanlış pozitifidir;
bu kayıttan gerçek credential sızıntısı, yeni güvenlik açığı veya yeni GB
çıkarılmaz. Bu teşhis bütün deponun başka bulgusuz olduğu iddiası değildir.

P4 takip şekli hash verifier'ı aynı public kaynak kimliğine bağlı tutar:
named `METRICS_SOURCE_SHA` üst sabiti ve seçicide tek sabit referansı.
Allowlist, tarayıcı kapatma, kural bastırma, gizli concat veya kontrol
gevşetmesi bu takip kapsamı değildir. P4 preflight yalnız public kimlik sabitini adlandırır: `scripts/tests/metrics-pg-owner-preflight-test.py` SHA256 `acfa2e28be3731759b0c64f1c1367191a4c68cdc01145496387c1934a7999dc8`,52442byte; native harness `b5fd46b70ac21dc4882ef1d4513f0a89836bc4a2453d85528a42ecb2b2dd89f2` değişmez. `METRICS_SOURCE_SHA` gerçek public d5d129 kaynak hash literalidir; seçicinin exact hash denetimi aynıdır.
P4 hazırlık `/private/tmp/zee-k300/k310-metrics-proposal-4/PREPARATION.json` SHA256 `a6ffdebd90d9f2edc7b8b1a16cedf41425cb2f99fb2a197217d36a8f5d739dec`,67033byte; independent static peer `5b9576f7a48e1a1adc20129fba7aa491d6ffb625753bc34e2d6cc6320f86fdbf`,3640byte; fixture3 `b625c646bc8bdd429d9d06cc1989a58050a78bb0110fbb598526c11347bef57f`,4817byte ile bağlandı. P4 actual standalone52/0.395sn ve combined16/0.441sn PASS/exit0/reaped/errors[]/cancelled[]; özgün dış süreler0.481812375sn ve0.494281084sn. Actual model peer `2f8e05bcf0c5fed4441cc9700445fa37894d043e79bdf679bdef6b6d923d7b5f`,15528byte, owner/log attempt3 pinlerini bağlar.52=generic18+metrics34;16=old15+tek nested52wrapper,68Cargo/native değildir. Erken default8.30.1 taraması16public proposed dosyada0bulgu/emptyignore verdi (`e0a985151ee0891f0f4afb5885d77bfd5ffdbc64cf04d02386e01a104b2efd49`,3446byte); taranan rapor önceki3NULL DRAFTnote idi, bu FILLEDnote değildir. Bu kesit fullRscan/kaynak15 veya native kabulü değildir.

Bu başarısız ikinci koşu anında K310/GB119 açıktı. P4 kontrollü sonuçları,
fresh tam kaynak15/özgün owner ve temiz exact source commit'i yalnız kendi
actual makbuzlarıyla değerlendirilir; ardından aynı kaynakta fresh metrics8
ve ayrı mevcut nested release libpool/120sn sahibi, bağımsız frozen incelemesi
ve final11-tested-overlay/temiz kapanış commit zinciri gerekir. Public-source
hash teşhisi veya isimlendirme hazırlığı bu zincirde PASS/kapanış değildir.
Metrics/effect iki PG tüketicisi, genelP13kısmi ve39plan statüsü bu notla
kapanmaz. Production175/sharedH/PG6e5/API/grammar/ürün/RC/soak/IDE sınırları
aynı kalır; private credential/CA/key/DSN/PHC baytları/hash'leri dışarı alınmaz.

Public teşhis ve başarısızlık pinleri:
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-2/PROOF.json` SHA256 `e20d431d6befffe569e3aeb4b14fae41d585eac425c3add8bd95e720d015f0e3`, 66654byte (failed_source_gate2).
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-controller-2.json` SHA256 `15c379b19dd921174c9bf6a19f60d4dc7ec92902d71b5941363d052aa3958697`, 1059byte (failed_source_owner2).
- `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-2/phase-matrix-report.md` SHA256 `23a9624abc1fd9e56cb9e800aff29e95a2969ccac5512419ba4fdfb41587cbf4`, 38262byte (source2_phase_report).
- `/private/tmp/zee-k300/k310-source2-redacted-scan-diagnosis-1/safe-findings.json` SHA256 `9722b13ffa0528162640c9cdb12b32ae7c2353e396030c7076b899edef980b32`, 373byte (safe_findings).
- `/private/tmp/zee-k300/k310-source2-redacted-scan-diagnosis-1/redacted-report.json` SHA256 `e372a319c844cf73b4971fa686840df3b024202835a86bb8be87495205512419`, 719byte (redacted_report).
- `/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014/scripts/native-metrics.py` SHA256 `d5d12902358591365ff201c13c44b255d265c49c71befabcdef0575fe7e854c4`, 5009byte (public_metrics_source).
- `/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014/scripts/tests/metrics-pg-owner-preflight-test.py` SHA256 `0db506b67b448b34fcbf990f1fb8c3396e906940e11b65e3f6011abb3183ebcf`, 52402byte (pre_P4_preflight_source).
