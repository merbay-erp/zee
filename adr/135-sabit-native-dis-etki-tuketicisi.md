# ADR-135 — Sabit native dış etki tüketicisi (K-277/P09)

- **Durum:** kabul; temiz S owned loopback nativeeffect11 kabulü var; fullP09/ürün/platform açık
- **Tarih:** 15 Ekim 2026
- **İlgili:** ADR107/114/117, ADR096, GB-093

Kayıtlı B1/B2 ihtiyacı kalıcı katalog kuyruğundan gerçek e-posta etkisine geçmeyi
ister; SQL-only idempotent handler SMTP transaction'ı değildir. Aynı DB transaction
içinde SMTP'yi exactly-once yapma, C029'dan pre-DATA çıkarma veya timeout sonucunda
otomatik replay güvenli değildir. Yeni compiler/grammar yerine mevcut Zee SQL eylem
ve MIME kalıbıyla ayrı fixed finite owned-local profile sınanır. Native consumer
aktarımı production Rust/Python SMTP bypass değildir.

Kalıcı local pending guard claim'den önce; owner/token/generation30s lease ile
ayrı committed SQL effect-start latch SMTP'den öncedir. Sonraki native send live
hak/latch'i yeniden kontrol eder. Latch otomatik expiry/reaper ile açılmaz.
Known pre-effect SQL reddi, sıfır SMTP invocation ve matching lease/latch yokluğunda
ayrı acknowledged safe-fail ile en çok toplam3 ve1/2sn backoff yapabilir. Her
started-send/unknown transaction sonucu durur; ack başarılı olsa bile client
bilinmiyorsa manuel inceleme gerekir. Otomatik reset/reconcile/re-delivery yoktur.
Ayrı readonly SQL durum okuması dış SMTP sonucunun yokluğunu ispatlamaz.

Outer Supervisor issued group leader WNOWAIT ile korunur; native çocuklar aynı
grubu devralır. Host direct-child bounded capture/TERM/KILL/wait yapar; öldürülen
hostun grubu retained outer owner tarafından kapatılır. PID keşfi/adoption yoktur.
Credential/file/source/capture/deadline sınırları fixed profile'da tutulur. Local
SQL/SMTP fixtures explicit loopback plaintext'tir; production TLS/provider/onboarding
kanıtı değildir. Existing SQL-only dispatcher, canonical helpers ve compiler
kaynağı değişmez; hiçbir eski tabloya mixed worker verilmez.

Hazırlık8 ve actual11, gerçek SMTP connection/DATA sayısı ile bağımsız PG observer
state/ack'ini ayırır. COMMIT before/after actual tel kontrolü C027 belirsizliğini
üretir; sentetik PDF eki mevcut native MIME'dan geçer. Kaynak Build4d8/d53 dirty
adayı saklanır, finalclean kimliği uydurulmaz. Tam P09, B2 PDFgeneration/K275ürün
entegrasyonu, gerçek Gmail/M365/catalog/P08/calendar/platform kabulü açıktır.

İlk test-only COMMIT fixture bağlantı-sırası varsayımı başarısızdı ve korunmuştur;
son fixed-operation router gerçek acknowledged COMMIT'e fault uygular. Ad-hoc
probe teardown hatasında kaybolan original ownership yetkisi tarihsel UNVERIFIED
sınırdır; current-absence ile geçmiş cleanup kanıtı icat edilmez. Son kaynak spawn
öncesi ExitStack cleanup kaydı ve failed-receipt finally ile bu yolu daraltır.
Bağımsız peer sonucu ve root final acceptance olmadan entegrasyon READY değildir.

## Revision2: bilinen tamamlanmanın görünür yayını ve strict inventory

Bilinen `done`, `idle` veya `dead_letter` sonucu sonrasında guard unlink görünür
olup son directory fsync başarısızsa sonuç **exit4** ve
`known_completion_visible_durability_unconfirmed` olur; `completed_status` bilinen
sonucu taşır. Guard görünür olarak yoktur, dolayısıyla yeni admission engellendi
veya `paused_manual` olduğu iddia edilmez. Tekrar çağrı yeni işi alabilir; önceki
etkiyi tekrar gönderme izni verilmez. Powerloss eski guard'ı geri getirirse
konservatif ret mümkündür. Son result receipt publication fsync'i de başarısızsa
aynı exit4/sabit publication-unconfirmed metadata korunur. Bu, SMTP/SQL effect
outcome belirsizliğinden ayrı yerel dayanıklılık sınırıdır. Bilinmeyen
claim/prepare/send/ack guard'ı tutar ve exit3 kalır; guard kurulmadan identity/build
retleri exit2'dir. Başarılı fsync dahil normal completion exit0'dır.

State inventory `Path.iterdir` eager allocation kullanmaz: streaming `os.scandir`
17'nci girdide reddeder ve iterator kapanır. Hazırlık8 içindeki durable alt-vakalar
permanent postclear fsync failure exit4/visible-guard/allowed-next-idle sınırını
kontrollü mock-native/real-filesystem modeliyle; gerçek17-entry retini ve
1000-entry kontrollü iteratorın yalnız17 gözlemle reddini sınar. Bunlar native SQL
veya SMTP kabulü değildir ve yeni native vaka olarak sayılmaz.

Fixture kanonik `compiler/tests/fixtures/native-effect-consumer` altındadır.
Native koşucu framework kökünü dolaşmaz; tam14 sabit source-role yolu, dosya128KiB
ve toplam2MiB sınırında hashlenir. Runtime/proje hashleri aynı kampanyanın sonunda
tekrar eşlenir. Revision1 frozen14/43 ve iki peer failure reproducer korunur;
R1/R2 bu ilk host feature kaynağından önce düzeltilen aday bulgularıdır.

## Revision3: yalnız pinlenen dört fixture dosyası

Native harness proje kurarken recursive discovery yapmaz. Yalnız `ana.dil`,
`catalog.pdf`, `gocler/0001_effect.sql`, `proje.dil` sırası sabittir; her biri
regular/nofollow/stable snapshot ve128KiB bound ile okunan aynı bytes üzerinden
yazılır. Ek SQL, `.dil` ve FIFO dosyası ne açılır ne kopyalanır. Hazırlık8 içindeki
actual filesystem alt-vakası extraSQL/source/FIFO varlığında yalnız bu dört
dosyanın tam beklenen bytes ile kurulduğunu sınar; SQL/SMTP kabulü değildir.
Revision2 kaynak/kanıt/peer paketleri değişmez. R3 yalnız acceptance harness ve
alt-vakadır; runtime host/compiler davranışı değişmez ve yeni native case sayılmaz.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

S common nativeSMTP/PGfault11 actual fixed sentetik loopback kabulü geçti;14canonical GitS source-role, source/binary begin-end ve readonly PG/SMTPcount oracle ayrı korundu. Known preeffectSQL sıfır-send sınırlı retry; C029/DATA/ackCOMMIT belirsizliği manualpause/no replay. Tarihsel ad-hoc authoritylost UNVERIFIED kaydı currentabsence veya yeni ownedcleanup ile geriye dönük kapanmaz. FullP09/provider/realcatalog/PDFgenerationintegration açık.

[Güncel dar kaynak kanıtı](../docs/urunlesme-final-kabul-20261015.md).

## K-311 / GB-120 — test harness lifecycle: dar kapsamda kapalı

K-311 / GB-120 dar kapsamda kapalı: effect native kabul harness’inin kalan özgün kaynak cleanup denemeleri, ilk hata kimliği, yapışkan iptal/publication ve plaintext PG sahipliği dar dilimi. Exact kaynak ea535fad3dbd4beb2dd7cd7e3b217c041686e89b tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/effect11/60need frozen kabulü; özgün plaintext PG/readiness/retained cleanup kanıtlandı, tüm inner host0 iddiası yok.

Exact kaynak ea535fad3dbd4beb2dd7cd7e3b217c041686e89b, tree a01ad84549ca0f6c59578e8feb0310fdb9ed2877; tam15 kaynak kapısı 1430.124sn ve temiz tested-source commit/özgün owner zinciri doğrulandı. Aynı koşunun faz envanteri 998/1023, 0 başarısız/25ignored/22faz; diğer SHA sonuçlarıyla toplanmaz. K311 source5+OPENdocs11 atomik uygulaması gerçek a294 makbuzuna bağlıdır. Child-free1 expectedFAIL, baseline8 ve lifecycle27/PG18/selected45/ordinary9 ile native-driver pure35 ayrı hazırlıklardır; Cargo/native toplamı değildir. Baseline1 driver KeyError ve ordinary fixture1 setup/missing-script hatası testlerden önceydi, sonraki gerçek başarı bunları silmez. Rust output-env tasarımı45fed ve rootaudit5a788 format/check0 attestation’ı ayrı korunur; ayrı formatter raw/owner kanıtı türetilmez.

Yalnız bu exact kaynakta canonical mevcut tek ignored effect Cargo sahibi geçti:11 effect gözlemi/60need, nestedlib Cargo sahibi yok; sürücü 62.825sn. Kaynak358/production175, consumed14+ayrı plaintext helper ve 25 kapalı public artifact envanteri frozen/root/peer ile bağlandı. Aynı kaynak 25ignored envanterinden kalan24 bu kampanyada çalışmadı. Explicit loopback plaintext/TLSdisabled/startup196608; SQL/SMTP C029/DATA/ackCOMMIT belirsizliği manualpause/no-replay, genel exactly-once/provider kabulü yoktur.

Özgün foreground PG normal SIGINT/observed_exit=exit0/reaped/noFallback ve SHOW/readiness kimliği doğrulandı. 55 güvenli özgün command/FD kapanış makbuzu ve 2 readiness gözlemi gerçek public proof’a bağlı; readiness retry2 dizisi son0 ile biter. Retained socket/thread/COMMIT kaynakları, özgün dış ownedgroup, private-root kaldırılması, sticky cancel/handler restore/failed-only publication ve work180/tek cleanup90 sınırı doğrulandı. Native host/signal vakalarının beklenen farklı exitleri korunur; bütün inner worker/host normal0 varsayılmaz. Private SQL/SMTP/body/DSN/PHC/password/cert/key/credential bytes veya hash’leri public kanıt değildir.

Yerel PG fixture plaintext loopback/TLSdisabled/startup196608 kalır; shared TLS6e5 helper’a çevrilmez. C029/DATA/ackCOMMIT belirsizliği manualpause/no-replay, native11/60need/source14 ve mevcut tek ignored Cargo sahibi korunur; nestedlib Cargo sahibi yoktur. Normal özgün PG/readiness/FD/socket/thread/COMMIT cleanup iddiaları yalnız gerçek public makbuzlarının kapsamına bağlıdır; bütün inner worker/host exit0 sonucu varsayılmaz. Production175/compiler/src/H/API/ABI/grammar/performans değişmez.

### Korunan kontrollü kaynak/test hazırlığı

Child-free1/0.003sn expectedFAIL/exit1/reaped; actualpeer812690ff68322169e9fa96770f397a503418f33c5100b803a432e62459cfdcb9. Yalnız stop.set/owned-sockets.close gözlendi; sonraki beş cleanup eylemi atlandı ve ilk hata korundu. Tarihsel ad-hoc UNVERIFIED kaydı geriye dönük kapanmaz.

Eski baseline8 attempt2:8/0.947sn PASS; peerab299b4506236e838d97de88a87271bdae42b005e17c8a1cd2b4430fda371b1f. İlk precommand driver KeyError ayrı başarısız hazırlıktır. Ordinary9 fixture2:9/1.101sn PASS/özgün owner0/reaped/1.1660061669535935sn; peerc3acfa3ac563a1ad8f91bac982bc29ad3805bdeb59e6fa0605792e509da8ec9d. Fixture1 setup/missing-script hatası testlerden önceydi; source kusuru veya testFAIL değildir.

Lifecycle27:27/0.081sn PASS, peer7cadf3e0bfaaf1a2c08c52d36bb906cd47bd2c4cacd0ad81b01b8049ae2b1d85. Aynı lifecycle27+PG18’i seçen preflight45:45/0.153sn PASS/özgün owner0/reaped/0.27195924997795373sn; peerbcf933e37f25472a49d419ab9415a93e0e89fbe0ce796361a1a17765a216876a. Ordinary9 nested45’i yeniden toplamlatmaz;45/27/18 native veya yeni Cargo sahibi değildir.

Standalone PG18:18/0.051sn PASS, peera6268fd4a1e08e23859d97f8eddf3f02c646f55e638c12da9fcd26f4f37cfcf4. Fake command/readiness/SIGINT/FD/uncertainty modeli; gerçek PG/native yürütümü değildir.

Son source/test batch’inin beş dosyalık pin geçmişi:

| Kaynak/test dosyası | SHA-256 | Bayt |
| --- | --- | ---: |
| `scripts/tests/effect-dispatcher-native-test.py` | `c4a91293f32d8f7463a922f15928c70cd4e058d3e0ca7a922966774f8c1bcf1f` | 46884 |
| `scripts/tests/effect-dispatcher-test.py` | `3a95add8341e83b934874cab497f64589fc2c2ca99debaa2fb4c9fdcfd7cd0f5` | 11846 |
| `scripts/tests/effect-pg-owner.py` | `37e72f521260f5dc039840bd413efa405164d0a86ea5601e4c5abfd820e0611e` | 21768 |
| `scripts/tests/effect-pg-owner-preflight-test.py` | `922d58f149c5bce69d9ef66c6523a61689b0c1c7ba842ae84becbbb826f51cfa` | 41082 |
| `compiler/tests/native_effect_consumer_testi.rs` | `8e8d3d23016dfefe1d108617602ede6fc920ec4d2d4601a5c0c16dbcf0024fa9` | 4482 |

Rust8e8d output tasarımı45fed peer’e bağlıdır; rootaudit5a788 format/check0-empty/byteexact sonucunu kaydeder, ayrı formatter raw/owner kaydı sunulmadığından kapsamı root attestation’dır. Internal ZEE_EFFECT_NATIVE_OUT freshabsolute/canonicalparent/NotFound seçimi ve default yol korunur; ürün CLI kapasitesi değildir. Eski baseline launcher1 KeyError ve ordinary fixture1 setup/missing-script kayıtları başarısız hazırlıklardır; testlerden önce oluşan sonuç source/native kusuru diye sunulmaz. Tarihsel ad-hoc cleanup UNVERIFIED yeni kabulden geriye dönük kapanmaz.

### Exact public kabul kayıtları

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k311-source-gates-1/PROOF.json` | `701a0097e6d3c1cbf72d2f7b325a38e53f003315545d42f12058ae7d0d226004` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k311-source-gates-controller-1.json` | `e62521d85ea8f57199c33c7e7851f9d29e07a7f45bba71757f51e7ba07c43159` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k311-source-commit-1.json` | `5726052b993e11b73b46743f6976988d4ea1c540fd42829c9e46707a542d64a4` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k311-source-commit-controller-1.json` | `d98898df739cc51d7a20c660a6cdc70b2ca8d802ec8bbd7a93850358907360f6` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/effect-native11-1/PROOF.json` | `e1c1640ebca31e2f7deed9edfe55746751d7588cddde2cb1ee46143de947973f` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/effect-native11-controller-1.json` | `4c39132d97fe9f77aacdc3c069bd9604d624a9fee5df9f8f66598be72e8e9ec8` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/effect-native11-1/native/proof.json` | `92d39e4c9f68fca9717e9327738eb87e1f8568422f32695b95d2ced2fc81cefc` |
| root_native_audit | `/private/tmp/zee-k300/K311-NATIVE-ROOT-AUDIT-1.json` | `25da0cccb3cc7b9809a615a5fec2bb2cff719809d3b00fad9f2804bee70c47fa` |
| native_peer_review | `/private/tmp/zee-k300/K311-NATIVE-PEER-REVIEW-1.json` | `a4d9faee4816a5a4910519526f9ef8ff6d8465b9cfd661e4c7b47d78f9e1c47b` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k311-frozen-acceptance-1.json` | `3ab436bde09d2495f65b575a1e2ff0ddc43c9e0d6169cea55becfaab6fd34667` |
| source_pins | `/private/tmp/zee-k300/k311-source-pins-1.json` | `d9287a8106833a943941bc15db9bc1720d75a02d76be038e1c5078bb989b8c2a` |
| closure_template | `/private/tmp/zee-k300/K311-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `caac85e35f53fba73060478168db8f1af235bf4874e96b9dd60c7d01d5482a0d` |
| closure_template_review | `/private/tmp/zee-k300/K311-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `6ec7ecb5568a15f95815d8b228bc0a4c39b9ebe65c41b56743c1856d19c867ac` |
| EFFECT-NATIVE-CLEANUP-ACTUAL-COUNTEREXAMPLE-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-NATIVE-CLEANUP-ACTUAL-COUNTEREXAMPLE-INDEPENDENT-REVIEW-1.json` | `812690ff68322169e9fa96770f397a503418f33c5100b803a432e62459cfdcb9` |
| EFFECT-BASELINE8-ACTUAL-INDEPENDENT-REVIEW-2.json | `/private/tmp/zee-k300/EFFECT-BASELINE8-ACTUAL-INDEPENDENT-REVIEW-2.json` | `ab299b4506236e838d97de88a87271bdae42b005e17c8a1cd2b4430fda371b1f` |
| EFFECT-LIFECYCLE27-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-LIFECYCLE27-ACTUAL-INDEPENDENT-REVIEW-1.json` | `7cadf3e0bfaaf1a2c08c52d36bb906cd47bd2c4cacd0ad81b01b8049ae2b1d85` |
| EFFECT-PG18-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-PG18-ACTUAL-INDEPENDENT-REVIEW-1.json` | `a6268fd4a1e08e23859d97f8eddf3f02c646f55e638c12da9fcd26f4f37cfcf4` |
| EFFECT-PREFLIGHT45-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-PREFLIGHT45-ACTUAL-INDEPENDENT-REVIEW-1.json` | `bcf933e37f25472a49d419ab9415a93e0e89fbe0ce796361a1a17765a216876a` |
| EFFECT-COMBINED45-ORDINARY9-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-COMBINED45-ORDINARY9-ACTUAL-INDEPENDENT-REVIEW-1.json` | `c3acfa3ac563a1ad8f91bac982bc29ad3805bdeb59e6fa0605792e509da8ec9d` |
| EFFECT-COMBINED45-ORDINARY9-ROOT-ACTUAL-AUDIT-1.json | `/private/tmp/zee-k300/EFFECT-COMBINED45-ORDINARY9-ROOT-ACTUAL-AUDIT-1.json` | `5a7886991d16e2e1623db85f741e172d7d7a409f339c1210ded290606cc0941c` |
| EFFECT-TERMINAL-INTEGRATION-P4-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-TERMINAL-INTEGRATION-P4-INDEPENDENT-STATIC-REVIEW-1.json` | `b98b1ebd364fa2d4086eac5ca69ef8837fb56d6ca4dfbb97b974ff60c06034b3` |
| EFFECT-PERMANENT-INTEGRATION-PROPOSAL2-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-PERMANENT-INTEGRATION-PROPOSAL2-INDEPENDENT-STATIC-REVIEW-1.json` | `643dcf8c3d5fa87332818cc5f63238db834fbc7a7cd36a0a8863c3fb9ea1cc0c` |
| EFFECT-PERMANENT-ROOT-FINALIZATION-1.json | `/private/tmp/zee-k300/EFFECT-PERMANENT-ROOT-FINALIZATION-1.json` | `7448dcf4e31e8fa3006fac42b3b282a83120d8ab175ed669bccc7fe3fb56cfb5` |
| EFFECT-NATIVE11-ACCEPTANCE-DESIGN-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-NATIVE11-ACCEPTANCE-DESIGN-INDEPENDENT-STATIC-REVIEW-1.json` | `45fed48bc735436707cbe488b9c60c78d516b33f0263f6687d95ea42539b3b11` |
| EFFECT-NATIVE35-MODELS-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/EFFECT-NATIVE35-MODELS-ACTUAL-INDEPENDENT-REVIEW-1.json` | `d4cfbc7ef049b223a026f07d1e7fe0a3383f11c6accf3f3015b13167c17b3565` |
| K311-NATIVE-DRIVER-ROOT-FINALIZATION-1.json | `/private/tmp/zee-k300/K311-NATIVE-DRIVER-ROOT-FINALIZATION-1.json` | `ee255fc1f91b97d154b01c74b4004b0cb87d71cf9d340607c6cbe666715d4856` |
| EFFECT-NATIVE11-ALLOWED-FINAL-BINDINGS-INDEPENDENT-STATIC-REVIEW-2.json | `/private/tmp/zee-k300/EFFECT-NATIVE11-ALLOWED-FINAL-BINDINGS-INDEPENDENT-STATIC-REVIEW-2.json` | `ce4f39dc4de0809feb84ba7476d7996bab52fbb52105caffab43f2c822d23536` |
| K311-ATOMIC-APPLICATION-1.json | `/private/tmp/zee-k300/K311-ATOMIC-APPLICATION-1.json` | `a2948b9f2032650a5f3d8589daeae9967948f8f5b2d36e5578d3b81bfabeed96` |
| K311-ATOMIC-APPLICATION-ROOT-AUDIT-1.json | `/private/tmp/zee-k300/K311-ATOMIC-APPLICATION-ROOT-AUDIT-1.json` | `508bd414d06a1da5b3ac24f4ac208f8942ccef06534704bce91d08d61133fd0c` |
| K311-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K311-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `85b3d9fe6dca828464fbe37e32a3aa85d3097dc246fbf7da11833a0fb7edb47d` |
| K311-OPEN-DOC-RENDER-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K311-OPEN-DOC-RENDER-INDEPENDENT-REVIEW-1.json` | `0a991f66f62b315b0b2bc3dc0771d3dcdf7a5cb2cdbb1730ea57f7ef3ca730b4` |
| K311-SOURCE-PIN-ROOT-FINALIZATION-1.json | `/private/tmp/zee-k300/K311-SOURCE-PIN-ROOT-FINALIZATION-1.json` | `0a553d96a2bceca04d795e156b43c4ce3b133ceb2a3c9610f6acaa9ed4ab0b89` |
| K311-SOURCE-DRIVERS-ROOT-FINALIZATION-1.json | `/private/tmp/zee-k300/K311-SOURCE-DRIVERS-ROOT-FINALIZATION-1.json` | `42c168c53687dbb7ebcd10a0baed07e7236900081b6ed617bda0eacd558e8353` |
| K311-CLOSURE-TEMPLATE-BINDER-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K311-CLOSURE-TEMPLATE-BINDER-INDEPENDENT-STATIC-REVIEW-1.json` | `b46b2cf3928d138f5080cec79bba9e339bf10f1a27af266ebfa1c87288f99ab6` |
| K311-TEMPLATE-BINDER-ROOT-FINALIZATION-1.json | `/private/tmp/zee-k300/K311-TEMPLATE-BINDER-ROOT-FINALIZATION-1.json` | `4c77cbfa6fb7239c6f4e6b8847295dee438629425dca36b0a6edf4509aca6db2` |
| K311-OPEN-DOCS-PROPOSAL-FILLED-PREPARATION-5.json | `/private/tmp/zee-k300/K311-OPEN-DOCS-PROPOSAL-FILLED-PREPARATION-5.json` | `42d363bfd429badf818c08897cc5101a057987e6fac5ddae236446d16b30aba3` |
| K311-FINAL-DRIVERS-ROOT-FINALIZATION-1.json | `/private/tmp/zee-k300/K311-FINAL-DRIVERS-ROOT-FINALIZATION-1.json` | `3ce9ee430f2f39c64893f7dc8d41904486e98d4c2ae088aa34aed786bd9cfa77` |
| K311-FINAL-DRIVERS-BOUND-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K311-FINAL-DRIVERS-BOUND-INDEPENDENT-STATIC-REVIEW-1.json` | `6011ad2f54fb36de03662be071531d16567cfd581b91652e5167b8236e567dcd` |
| native_driver_preparation_review | `/private/tmp/zee-k300/EFFECT-NATIVE35-MODELS-ACTUAL-INDEPENDENT-REVIEW-1.json` | `d4cfbc7ef049b223a026f07d1e7fe0a3383f11c6accf3f3015b13167c17b3565` |

Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her kabul kendi exact başarılı deneme kayıtlarıyla izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belge kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

K306–K310 dar kapanışları kendi exact kaynaklarında korunur. Yalnız gerçek K311 dar kapanışı sonrasında bu beş PG harness takip kuyruğu0 olur; genel P09/provider/ürün ve diğer teknik işler açık kalır.39 statü korunur; P09/P13/P20/P23/P36kısmi/P37sürekli,GB046/109açık. RC/soak/IDE/platform/proje/insan kabulü aktarılmaz; C1/K288/P33/K293 ret sınırları yeniden denenmez.
